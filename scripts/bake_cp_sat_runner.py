#!/usr/bin/env -S uv run --no-project
# /// script
# requires-python = ">=3.11"
# ///
"""Async parallel seed-sweep runner for the canonical CP-SAT placer.

For each (shape, seed) probe, spawns `scripts/cp_sat_placer.py` with
`SPAGHETTIO_CP_SAT_SEED=<seed>`, parses the JSON response, and:
  - On `kind=ok`: appends the template to the journal and DRAINS all
    other queued probes for that shape (first solve per shape wins).
  - On `unsat` / `timeout` / `engine`: logs to TSV and moves on.

Adapted from `scripts/balancer_runner.py` (the Factorio-SAT runner) —
same orchestration patterns (async worker pool, priority queue, journal
with fsync, signal-handled drain, process-group SIGKILL on timeout) but
replacing the Factorio-SAT-specific probe model with `(shape, seed)`
tuples and the subprocess invocation with our placer.

Output:
  scripts/cp_sat_journal.jsonl   — successful solves (resume-safe)
  /tmp/cp_sat_sweep_<ts>.tsv     — per-probe wall + status

Usage:
  scripts/bake_cp_sat_runner.py --shapes 1,9 1,10 --seeds 0-127 --timeout 60
  scripts/bake_cp_sat_runner.py --resume   # skips shapes already in journal
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import signal
import subprocess
import sys
import time
from dataclasses import dataclass, field
from itertools import count
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PLACER = REPO_ROOT / "scripts" / "cp_sat_placer.py"
JOURNAL = REPO_ROOT / "scripts" / "cp_sat_journal.jsonl"
DUMP_SYNTH_BIN = REPO_ROOT / "crates" / "core" / "target" / "debug" / "examples" / "dump_synth"

# Probes that take longer than this without solving are flagged as suspiciously
# slow in stderr — useful for spotting layouts that genuinely won't solve vs.
# ones that just need more time.
LONG_PROBE_THRESHOLD_S = 120.0


def default_worker_count() -> int:
    """Workers = min(cpu_count, memory-derived cap).

    Each CP-SAT solve in single-worker mode peaks around 200-500 MB; with
    the 8-worker option we'd be ~1.5-3 GB per worker. Default to the safer
    single-worker count and let the user override.
    """
    cores = os.cpu_count() or 1
    # Try to read /proc/meminfo for an actual budget; fall back to assuming
    # 4 GB if unavailable.
    try:
        with open("/proc/meminfo") as f:
            for line in f:
                if line.startswith("MemAvailable:"):
                    kb = int(line.split()[1])
                    gb = kb / (1024 * 1024)
                    # Reserve 2 GB for everything else; assume 0.5 GB / worker.
                    cap = max(1, int((gb - 2) / 0.5))
                    return min(cores, cap)
    except OSError:
        pass
    return min(cores, 8)


# ---------------------------------------------------------------------------
# Probe model
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Probe:
    shape: tuple[int, int]
    seed: int
    timeout_s: int
    cp_sat_workers: int = 1


def probe_priority(p: Probe) -> tuple:
    """Lower → scheduled sooner. Smaller shapes first (more likely to solve
    fast and free workers); within a shape, lower seeds first."""
    n, m = p.shape
    return (max(n, m), n + m, p.seed)


# ---------------------------------------------------------------------------
# Synth context (cache per shape — calling cargo run repeatedly is expensive)
# ---------------------------------------------------------------------------


_SYNTH_CACHE: dict[tuple[int, int], dict] = {}


def get_synth(shape: tuple[int, int]) -> dict:
    """Return the BalancerGraph + arc throughputs for `shape`, cached.

    Calls the `dump_synth` example binary to get the synth output, parses
    its stdout into a dict suitable for the placer's request body.
    """
    if shape in _SYNTH_CACHE:
        return _SYNTH_CACHE[shape]
    n, m = shape
    out = subprocess.run(
        ["cargo", "run", "--manifest-path", "crates/core/Cargo.toml",
         "--example", "dump_synth", "--quiet", "--", str(n), str(m)],
        cwd=str(REPO_ROOT),
        capture_output=True,
        check=True,
    )
    text = out.stdout.decode()
    arcs: list[dict] = []
    throughputs: list[float] = []
    n_inputs = n_outputs = n_splitters = 0
    for line in text.splitlines():
        if line.startswith("(") and "n_inputs=" in line:
            for kv in line.split(":")[1].split():
                if "=" in kv:
                    k, v = kv.split("=")
                    if k == "n_inputs":
                        n_inputs = int(v)
                    elif k == "n_outputs":
                        n_outputs = int(v)
                    elif k == "n_splitters":
                        n_splitters = int(v)
        elif line.strip().startswith("arc"):
            # `arc 0: Input(0) -> Splitter { idx: 0, port: 0 }  rate=1.000000`
            head, _, rate_part = line.partition("rate=")
            try:
                rate = float(rate_part.strip())
            except ValueError:
                continue
            # Parse src/dst from "X -> Y" inside head.
            try:
                lhs, rhs = head.split("->")
            except ValueError:
                continue
            src_str = lhs.split(":", 1)[1].strip()
            dst_str = rhs.strip()
            src = _parse_endpoint(src_str)
            dst = _parse_endpoint(dst_str)
            if src is None or dst is None:
                continue
            arcs.append({"src": src, "dst": dst})
            throughputs.append(rate)
    graph = {
        "n_inputs": n_inputs,
        "n_outputs": n_outputs,
        "n_splitters": n_splitters,
        "input_caps": [1.0] * n_inputs,
        "output_caps": [1.0] * n_outputs,
        "arcs": arcs,
    }
    payload = {"graph": graph, "throughputs": throughputs}
    _SYNTH_CACHE[shape] = payload
    return payload


def _parse_endpoint(s: str) -> dict | None:
    """Parse 'Input(0)' / 'Output(2)' / 'Splitter { idx: 0, port: 1 }'
    into the JSON-shape the placer expects."""
    s = s.strip()
    if s.startswith("Input("):
        return {"Input": int(s[len("Input(") : -1])}
    if s.startswith("Output("):
        return {"Output": int(s[len("Output(") : -1])}
    if s.startswith("Splitter"):
        # Splitter { idx: N, port: P }
        idx = port = 0
        for kv in s.split("{")[1].rstrip("}").split(","):
            k, _, v = kv.partition(":")
            k = k.strip()
            v = v.strip()
            if k == "idx":
                idx = int(v)
            elif k == "port":
                port = int(v)
        return {"Splitter": {"idx": idx, "port": port}}
    return None


# ---------------------------------------------------------------------------
# Subprocess management (lifted from balancer_runner.py)
# ---------------------------------------------------------------------------

_LIVE_SUBPROCESSES: set[int] = set()


def _sync_sigkill_group(pid: int) -> None:
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


async def kill_process_group(proc: asyncio.subprocess.Process) -> None:
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        await asyncio.wait_for(proc.wait(), timeout=5)
        return
    except asyncio.TimeoutError:
        pass
    except asyncio.CancelledError:
        _sync_sigkill_group(proc.pid)
        raise
    _sync_sigkill_group(proc.pid)
    try:
        await asyncio.wait_for(proc.wait(), timeout=5)
    except (asyncio.TimeoutError, asyncio.CancelledError):
        pass


async def run_probe(probe: Probe) -> dict | None:
    """Run one placer probe. Returns the parsed response dict on success
    (kind == "ok"), None otherwise. Side effect: kills the subprocess
    group on timeout."""
    payload = get_synth(probe.shape)
    n, m = probe.shape
    request = {
        "graph": payload["graph"],
        "n": n,
        "m": m,
        "timeout_ms": probe.timeout_s * 1000,
        "seed": probe.seed,
        "arc_throughputs": payload["throughputs"],
    }
    env = dict(os.environ)
    env["SPAGHETTIO_CP_SAT_SEED"] = str(probe.seed)
    proc = await asyncio.create_subprocess_exec(
        "uv", "run", "--no-project", str(PLACER),
        cwd=str(REPO_ROOT),
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.DEVNULL,
        start_new_session=True,
        env=env,
    )
    _LIVE_SUBPROCESSES.add(proc.pid)
    try:
        # Subprocess timeout = probe budget + 30s slack for startup/teardown.
        try:
            stdout, _ = await asyncio.wait_for(
                proc.communicate(input=json.dumps(request).encode()),
                timeout=probe.timeout_s + 30,
            )
        except asyncio.TimeoutError:
            await kill_process_group(proc)
            return None
        except asyncio.CancelledError:
            _sync_sigkill_group(proc.pid)
            raise
    finally:
        _LIVE_SUBPROCESSES.discard(proc.pid)
    if proc.returncode != 0 or not stdout:
        return None
    try:
        resp = json.loads(stdout.decode())
    except json.JSONDecodeError:
        return None
    if resp.get("kind") != "ok":
        return None
    return resp


# ---------------------------------------------------------------------------
# State + journal
# ---------------------------------------------------------------------------


@dataclass
class State:
    solved_shapes: set[tuple[int, int]]
    journal_lock: asyncio.Lock
    tsv_lock: asyncio.Lock
    drain_event: asyncio.Event
    kill_event: asyncio.Event
    counters: dict[str, int]
    in_flight: dict = field(default_factory=dict)
    tsv_path: Path | None = None


async def append_journal(probe: Probe, response: dict, wall_s: float) -> None:
    """Append a winning solve to the journal. fsync per write."""
    entry = {
        "timestamp": time.time(),
        "shape": list(probe.shape),
        "seed": probe.seed,
        "wall_s": wall_s,
        "template": response["template"],
        "solve_wall_ms": response.get("solve_wall_ms"),
    }
    line = json.dumps(entry) + "\n"
    fd = os.open(JOURNAL, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    try:
        os.write(fd, line.encode())
        os.fsync(fd)
    finally:
        os.close(fd)


async def append_tsv(state: State, probe: Probe, status: str, wall_s: float, n_entities: int | None) -> None:
    if state.tsv_path is None:
        return
    line = f"{int(time.time())}\t{probe.shape[0]},{probe.shape[1]}\t{probe.seed}\t{status}\t{wall_s:.2f}\t{n_entities or ''}\n"
    async with state.tsv_lock:
        with open(state.tsv_path, "a") as f:
            f.write(line)


def load_solved_shapes() -> set[tuple[int, int]]:
    if not JOURNAL.exists():
        return set()
    out: set[tuple[int, int]] = set()
    with open(JOURNAL) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            shape = entry.get("shape")
            if isinstance(shape, list) and len(shape) == 2:
                out.add((shape[0], shape[1]))
    return out


# ---------------------------------------------------------------------------
# Worker
# ---------------------------------------------------------------------------


async def worker(name: str, queue: asyncio.PriorityQueue, state: State) -> None:
    while not state.drain_event.is_set():
        try:
            _, _, probe = await asyncio.wait_for(queue.get(), timeout=1.0)
        except asyncio.TimeoutError:
            if queue.empty():
                return
            continue
        if probe.shape in state.solved_shapes:
            queue.task_done()
            continue
        state.counters["started"] += 1
        t0 = time.perf_counter()
        state.in_flight[asyncio.current_task()] = (probe, t0)
        try:
            resp = await run_probe(probe)
        except asyncio.CancelledError:
            queue.task_done()
            raise
        finally:
            state.in_flight.pop(asyncio.current_task(), None)
        wall = time.perf_counter() - t0
        if resp is not None and probe.shape not in state.solved_shapes:
            state.solved_shapes.add(probe.shape)
            n_ents = len(resp["template"].get("entities", []))
            async with state.journal_lock:
                await append_journal(probe, resp, wall)
            await append_tsv(state, probe, "OK", wall, n_ents)
            state.counters["solved"] += 1
            print(
                f"[{name}] SOLVED {probe.shape} seed={probe.seed} "
                f"wall={wall:.1f}s entities={n_ents}",
                flush=True,
            )
        else:
            status = "OK_DUP" if resp is not None else "UNSAT_OR_TIMEOUT"
            await append_tsv(state, probe, status, wall, None)
            state.counters["failed" if resp is None else "duplicate"] += 1
            if wall >= LONG_PROBE_THRESHOLD_S:
                print(
                    f"[{name}] long {status} {probe.shape} seed={probe.seed} "
                    f"wall={wall:.1f}s",
                    flush=True,
                )
        queue.task_done()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def parse_shapes(specs: list[str]) -> list[tuple[int, int]]:
    out: list[tuple[int, int]] = []
    for s in specs:
        n_str, m_str = s.split(",")
        out.append((int(n_str), int(m_str)))
    return out


def parse_seeds(spec: str) -> list[int]:
    """`'0-127'` → [0..127]; `'1,7,42'` → [1, 7, 42]; `'0-7,42'` → mixed."""
    seeds: list[int] = []
    for part in spec.split(","):
        if "-" in part:
            lo, hi = part.split("-")
            seeds.extend(range(int(lo), int(hi) + 1))
        else:
            seeds.append(int(part))
    return seeds


def install_signal_handlers(loop: asyncio.AbstractEventLoop, state: State) -> None:
    """First SIGINT: cooperative drain. Second SIGINT: hard cancel + kill all."""
    sigint_count = 0

    def handler():
        nonlocal sigint_count
        sigint_count += 1
        if sigint_count == 1:
            print("\n[runner] SIGINT — draining workers (Ctrl-C again to force)", flush=True)
            state.drain_event.set()
        else:
            print("\n[runner] second SIGINT — killing subprocesses + cancelling", flush=True)
            state.kill_event.set()
            for pid in list(_LIVE_SUBPROCESSES):
                _sync_sigkill_group(pid)
            for task in list(state.in_flight):
                task.cancel()

    loop.add_signal_handler(signal.SIGINT, handler)
    loop.add_signal_handler(signal.SIGTERM, handler)


async def main_async(args: argparse.Namespace) -> int:
    # Resolve shape / seed list.
    shapes = parse_shapes(args.shapes)
    seeds = parse_seeds(args.seeds)

    # Resume: skip shapes already solved in the journal.
    solved = load_solved_shapes() if args.resume else set()
    if solved:
        before = len(shapes)
        shapes = [s for s in shapes if s not in solved]
        print(f"[runner] resume: skipping {before - len(shapes)} already-solved shapes", flush=True)

    # Build probe queue.
    queue: asyncio.PriorityQueue = asyncio.PriorityQueue()
    counter = count()
    for shape in shapes:
        for seed in seeds:
            probe = Probe(shape=shape, seed=seed, timeout_s=args.timeout, cp_sat_workers=args.cp_sat_workers)
            await queue.put((probe_priority(probe), next(counter), probe))

    # State + TSV log.
    ts = time.strftime("%Y%m%d_%H%M%S")
    tsv_path = Path(f"/tmp/cp_sat_sweep_{ts}.tsv")
    with open(tsv_path, "w") as f:
        f.write("ts\tshape\tseed\tstatus\twall_s\tentities\n")
    print(f"[runner] TSV log: {tsv_path}", flush=True)

    state = State(
        solved_shapes=set(solved),
        journal_lock=asyncio.Lock(),
        tsv_lock=asyncio.Lock(),
        drain_event=asyncio.Event(),
        kill_event=asyncio.Event(),
        counters={"started": 0, "solved": 0, "failed": 0, "duplicate": 0},
        tsv_path=tsv_path,
    )

    install_signal_handlers(asyncio.get_event_loop(), state)

    # Spawn workers.
    n_workers = args.workers
    print(f"[runner] {n_workers} workers, {queue.qsize()} probes "
          f"({len(shapes)} shapes × {len(seeds)} seeds), "
          f"timeout={args.timeout}s/probe", flush=True)
    workers = [asyncio.create_task(worker(f"w{i}", queue, state)) for i in range(n_workers)]
    try:
        await asyncio.gather(*workers, return_exceptions=True)
    finally:
        # Final drain.
        for task in workers:
            if not task.done():
                task.cancel()
        await asyncio.gather(*workers, return_exceptions=True)
        print(
            f"[runner] done — started={state.counters['started']} "
            f"solved={state.counters['solved']} "
            f"failed={state.counters['failed']} "
            f"duplicate={state.counters['duplicate']}",
            flush=True,
        )
        print(f"[runner] TSV: {tsv_path}", flush=True)
        print(f"[runner] journal: {JOURNAL}", flush=True)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    parser.add_argument(
        "--shapes",
        nargs="+",
        required=True,
        metavar="N,M",
        help="Shapes to bake (e.g. --shapes 1,9 1,10 4,9).",
    )
    parser.add_argument(
        "--seeds",
        default="0-63",
        help="Seeds to try: '0-127' or '1,7,42' or '0-7,42'.",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=60,
        help="Per-probe CP-SAT solver timeout in seconds (default 60).",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=default_worker_count(),
        help="Concurrent probes (default: memory-derived).",
    )
    parser.add_argument(
        "--cp-sat-workers",
        type=int,
        default=1,
        help="num_search_workers for each CP-SAT instance (default 1 — keeps "
             "each probe single-core so the runner can fan out).",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Skip shapes already in the journal.",
    )
    args = parser.parse_args()
    return asyncio.run(main_async(args))


if __name__ == "__main__":
    sys.exit(main())
