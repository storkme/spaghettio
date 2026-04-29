"""Async parallel runner for balancer SAT generation (Phase 1).

Phase 1 produces an append-only journal at ``scripts/balancer_journal.jsonl``.
The runner NEVER writes to ``src/bus/balancer_library.py``. A separate commit
script (Phase 2 of docs/rfp-balancer-runner.md) validates and merges journal
entries into the library.

Run:
    uv run python scripts/balancer_runner.py --max-tier 9 --skip-existing

Re-running with --skip-existing reads both the library (validated truth) and
the journal (uncommitted solves), and only schedules probes for shapes not
present in either.

Crash safety:
- Journal lines are written with flush + fsync per solve.
- Partial JSON lines from a kill are skipped on reload.
- The runner never modifies the library; the worst case after a crash is
  that the journal contains a couple of uncommitted solves to be picked up
  on the next commit step.

Concurrency:
- N workers (default = ``os.cpu_count()``) cooperate over an
  ``asyncio.PriorityQueue`` of probes.
- Each worker spawns one ``belt_balancer*`` subprocess at a time.
- When a shape is solved, queued probes for that shape are drained without
  execution. In-flight probes for a now-solved shape are NOT killed; they
  run to completion (or timeout) and their results are discarded. Bounded
  waste: at most ``workers * timeout_s`` core-seconds per shape.
"""

from __future__ import annotations

import argparse
import asyncio
import importlib.util
import json
import os
import signal
import subprocess
import sys
import time
from dataclasses import dataclass, field
from itertools import count
from pathlib import Path

SAT_DIR = Path(__file__).parent.parent / "external" / "factorio-sat"
SAT_PY = SAT_DIR / ".venv" / "bin" / "python"
LIBRARY_PATH = Path(__file__).parent.parent / "src" / "bus" / "balancer_library.py"
JOURNAL_PATH = Path(__file__).parent / "balancer_journal.jsonl"

# All shapes up to 10x10 except (1,1) identity.
SHAPES: list[tuple[int, int]] = [
    (n, m) for n in range(1, 11) for m in range(1, 11) if (n, m) != (1, 1)
]

# Per-tier search-space limits (mirrors find_balancer in the legacy script).
TIER_CONFIG: dict[int, dict] = {
    9: {"max_width": 50, "extra_heights": 5, "fast_timeout": 180, "use_full_interleave": True},
    10: {"max_width": 50, "extra_heights": 5, "fast_timeout": 180, "use_full_interleave": True},
    8: {"max_width": 40, "extra_heights": 4, "fast_timeout": 120, "use_full_interleave": True},
    7: {"max_width": 40, "extra_heights": 4, "fast_timeout": 120, "use_full_interleave": True},
    6: {"max_width": 30, "extra_heights": 3, "fast_timeout": 120, "use_full_interleave": True},
    5: {"max_width": 25, "extra_heights": 3, "fast_timeout": 120, "use_full_interleave": False},
    4: {"max_width": 21, "extra_heights": 2, "fast_timeout": 120, "use_full_interleave": False},
    3: {"max_width": 21, "extra_heights": 2, "fast_timeout": 120, "use_full_interleave": False},
    2: {"max_width": 21, "extra_heights": 2, "fast_timeout": 120, "use_full_interleave": False},
}

NET_FREE_TIMEOUT_FLOOR = 300


@dataclass(frozen=True)
class Probe:
    shape: tuple[int, int]
    height: int
    width: int
    timeout_s: int
    solver: str
    mode: str  # "fast" | "full" | "net_free"


def probe_priority(p: Probe) -> tuple:
    """Lower = scheduled sooner. Easy shapes first; smaller grids first."""
    n, m = p.shape
    # mode_pri: prefer fast probes over full probes at the same (h, w),
    # since fast typically finishes faster (fewer clauses).
    mode_pri = {"fast": 0, "net_free": 0, "full": 1}[p.mode]
    return (max(n, m), n + m, p.height, p.width, mode_pri)


def generate_probes(n: int, m: int, config: dict, default_solver: str) -> list[Probe]:
    """Build the full probe sequence for a single shape, mirroring the
    search strategy in the legacy ``find_balancer``."""
    base_h = max(n, m)
    max_width = config["max_width"]
    extra_heights = config["extra_heights"]
    fast_timeout = config["fast_timeout"]
    use_full_interleave = config["use_full_interleave"]

    network_path = SAT_DIR / "networks" / f"{n}x{m}"
    use_net_free = not network_path.exists()

    heights = [base_h + i for i in range(extra_heights + 1)]
    probes: list[Probe] = []

    if use_net_free:
        timeout = max(fast_timeout, NET_FREE_TIMEOUT_FLOOR)
        for h in heights:
            for w in range(3, max_width + 1):
                probes.append(Probe((n, m), h, w, timeout, default_solver, "net_free"))
        return probes

    for h in heights:
        for w in range(3, max_width + 1):
            probes.append(Probe((n, m), h, w, fast_timeout, default_solver, "fast"))
            if use_full_interleave and w >= base_h:
                probes.append(Probe((n, m), h, w, NET_FREE_TIMEOUT_FLOOR, default_solver, "full"))

    if not use_full_interleave:
        for h in heights:
            for w in range(3, max_width + 1):
                probes.append(Probe((n, m), h, w, NET_FREE_TIMEOUT_FLOOR, default_solver, "full"))

    return probes


def ensure_symmetric_network(n: int) -> None:
    """Generate the n×n Benes network file if missing. Sync (one-time setup)."""
    path = SAT_DIR / "networks" / f"{n}x{n}"
    if path.exists():
        return
    print(f"Generating {n}x{n} Benes network...", flush=True)
    subprocess.run(
        [str(SAT_PY), "-m", "factorio_sat.network", "create", str(path), str(n)],
        cwd=str(SAT_DIR),
        check=True,
    )


def load_validated_shapes() -> set[tuple[int, int]]:
    """Shapes already in src/bus/balancer_library.py (validated truth)."""
    if not LIBRARY_PATH.exists():
        return set()
    spec = importlib.util.spec_from_file_location("_runner_lib", LIBRARY_PATH)
    if spec is None or spec.loader is None:
        return set()
    mod = importlib.util.module_from_spec(spec)
    sys.modules["_runner_lib"] = mod
    spec.loader.exec_module(mod)
    return set(mod.BALANCER_TEMPLATES.keys())


def load_journaled_shapes() -> set[tuple[int, int]]:
    """Shapes already present in the journal (uncommitted solves)."""
    if not JOURNAL_PATH.exists():
        return set()
    shapes: set[tuple[int, int]] = set()
    with open(JOURNAL_PATH) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                # Partial line from a crash — ignore.
                continue
            shape = entry.get("shape")
            if isinstance(shape, list) and len(shape) == 2:
                shapes.add((shape[0], shape[1]))
    return shapes


async def kill_process_group(proc: asyncio.subprocess.Process) -> None:
    """SIGTERM the process group (5s grace) then SIGKILL.

    Required because ``belt_balancer*`` Python wrappers spawn ``kissat`` as
    a grandchild via ``subprocess.Popen``. ``proc.kill()`` only signals the
    direct child (the wrapper); kissat inherits init and continues to pin a
    core. Spawning with ``start_new_session=True`` puts the wrapper in its
    own process group with kissat as a member, and ``os.killpg`` reaps the
    whole group.
    """
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        await asyncio.wait_for(proc.communicate(), timeout=5)
        return
    except asyncio.TimeoutError:
        pass
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    try:
        await asyncio.wait_for(proc.communicate(), timeout=5)
    except asyncio.TimeoutError:
        pass


async def run_probe(probe: Probe) -> bytes | None:
    """Run one SAT probe. Returns raw stdout on success, None on UNSAT/timeout.

    Re-raises ``asyncio.CancelledError`` after killing the subprocess group
    so callers can implement graceful shutdown.
    """
    n, m = probe.shape
    if probe.mode == "net_free":
        cmd = [
            str(SAT_PY), "-m", "factorio_sat.belt_balancer_net_free",
            "--solver", probe.solver,
            str(probe.width), str(probe.height), str(n), str(m),
        ]
    else:
        network_path = SAT_DIR / "networks" / f"{n}x{m}"
        if not network_path.exists():
            return None
        cmd = [
            str(SAT_PY), "-m", "factorio_sat.belt_balancer",
            "--solver", probe.solver,
        ]
        if probe.mode == "fast":
            cmd.append("--fast")
        cmd.extend([str(network_path), str(probe.width), str(probe.height)])

    proc = await asyncio.create_subprocess_exec(
        *cmd,
        cwd=str(SAT_DIR),
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.DEVNULL,
        start_new_session=True,
    )
    try:
        stdout, _ = await asyncio.wait_for(proc.communicate(), timeout=probe.timeout_s)
    except asyncio.TimeoutError:
        await kill_process_group(proc)
        return None
    except asyncio.CancelledError:
        await kill_process_group(proc)
        raise
    if proc.returncode != 0 or not stdout:
        return None
    return stdout


async def encode_blueprint(raw: bytes) -> str | None:
    """Pipe raw SAT output through factorio_sat.blueprint encode."""
    proc = await asyncio.create_subprocess_exec(
        str(SAT_PY), "-m", "factorio_sat.blueprint", "encode",
        cwd=str(SAT_DIR),
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.DEVNULL,
        start_new_session=True,
    )
    try:
        stdout, _ = await asyncio.wait_for(proc.communicate(input=raw), timeout=30)
    except asyncio.TimeoutError:
        await kill_process_group(proc)
        return None
    except asyncio.CancelledError:
        await kill_process_group(proc)
        raise
    if not stdout:
        return None
    lines = stdout.decode().strip().splitlines()
    if not lines or not lines[0].startswith("0"):
        return None
    return lines[0]


@dataclass
class State:
    solved_shapes: set[tuple[int, int]]
    journal_lock: asyncio.Lock
    counters: dict[str, int]
    drain_event: asyncio.Event
    kill_event: asyncio.Event
    # asyncio.Task -> (Probe, perf_counter at start)
    in_flight: dict = field(default_factory=dict)


LONG_PROBE_THRESHOLD_S = 120.0
KILL_GRACE_S = 30.0


async def append_journal(probe: Probe, blueprint: str) -> None:
    """Append a successful solve. fsync per write so a kill leaves a
    consistent journal — at worst missing the last line."""
    entry = {
        "timestamp": time.time(),
        "shape": list(probe.shape),
        "sat_width": probe.width,
        "sat_height": probe.height,
        "solver": probe.solver,
        "mode": probe.mode,
        "blueprint": blueprint,
    }
    line = json.dumps(entry) + "\n"
    # Synchronous I/O is fine here — we hold the journal_lock and the
    # write is small.
    fd = os.open(JOURNAL_PATH, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    try:
        os.write(fd, line.encode())
        os.fsync(fd)
    finally:
        os.close(fd)


async def worker(name: str, queue: asyncio.PriorityQueue, state: State) -> None:
    """Pull probes from the queue and run them until drained or queue is empty.

    Cooperative shutdown: returns on ``state.drain_event``. In-flight probes
    finish (or hit their per-probe timeout) before the worker exits.
    Cancellation (second SIGINT) propagates through ``run_probe`` /
    ``encode_blueprint``, which kill the subprocess group before re-raising.
    """
    task = asyncio.current_task()
    try:
        while not state.drain_event.is_set():
            try:
                _, _, probe = queue.get_nowait()
            except asyncio.QueueEmpty:
                return

            try:
                if probe.shape in state.solved_shapes:
                    state.counters["skipped_solved"] += 1
                    continue

                state.in_flight[task] = (probe, time.perf_counter())
                state.counters["probes_started"] += 1
                t0 = time.perf_counter()
                raw = await run_probe(probe)
                elapsed = time.perf_counter() - t0

                if raw is None:
                    # TODO(nonsolves): record (shape, h, w, mode, solver,
                    # timeout_s) for timed-out probes to skip on resume.
                    # Deferred — design discussion: timeout-scaling, key
                    # scope, solver-version invalidation. Separate journal
                    # file (balancer_nonsolve_journal.jsonl).
                    state.counters["probes_unsat_or_timeout"] += 1
                    continue

                bp = await encode_blueprint(raw)
                if bp is None:
                    state.counters["encode_failed"] += 1
                    continue

                async with state.journal_lock:
                    if probe.shape in state.solved_shapes:
                        state.counters["raced"] += 1
                        continue
                    state.solved_shapes.add(probe.shape)
                    await append_journal(probe, bp)
                    state.counters["solved"] += 1

                print(
                    f"[{name}] SOLVED {probe.shape} mode={probe.mode} "
                    f"sat={probe.width}x{probe.height} solver={probe.solver} "
                    f"elapsed={elapsed:.1f}s",
                    flush=True,
                )
            finally:
                state.in_flight.pop(task, None)
                queue.task_done()
    except asyncio.CancelledError:
        # Force-kill path. The subprocess group (if any) was already
        # reaped inside run_probe / encode_blueprint via kill_process_group.
        return


def install_signal_handlers(loop: asyncio.AbstractEventLoop, state: State) -> None:
    """Install a two-stage SIGINT handler.

    First SIGINT  -> set drain_event. Workers stop pulling from the queue.
                     In-flight probes finish naturally (subject to per-probe
                     timeout). Print a status line naming long-running probes.
    Second SIGINT -> set kill_event, cancel all worker tasks (which causes
                     their in-flight subprocesses to be killed via the
                     CancelledError path in run_probe/encode_blueprint), and
                     schedule a hard exit after KILL_GRACE_S seconds.
    """

    def force_exit() -> None:
        print(
            f"\nGrace period ({KILL_GRACE_S:.0f}s) exceeded. Hard exit.",
            file=sys.stderr,
            flush=True,
        )
        os._exit(130)

    def on_sigint() -> None:
        if not state.drain_event.is_set():
            state.drain_event.set()
            now = time.perf_counter()
            in_flight = sorted(
                state.in_flight.values(),
                key=lambda pt: pt[1],
            )
            print(
                f"\nDraining: {len(in_flight)} probes in flight. "
                f"Ctrl+C again to force-kill.",
                flush=True,
            )
            for probe, t0 in in_flight[:8]:
                age = now - t0
                marker = "  (LONG)" if age > LONG_PROBE_THRESHOLD_S else ""
                print(
                    f"  shape={probe.shape} mode={probe.mode} "
                    f"sat={probe.width}x{probe.height} "
                    f"solver={probe.solver} elapsed={age:.0f}s{marker}",
                    flush=True,
                )
            if len(in_flight) > 8:
                print(f"  ... and {len(in_flight) - 8} more", flush=True)
            return

        if state.kill_event.is_set():
            return  # already handling
        state.kill_event.set()
        in_flight_count = len(state.in_flight)
        long_count = sum(
            1 for _, t0 in state.in_flight.values()
            if (time.perf_counter() - t0) > LONG_PROBE_THRESHOLD_S
        )
        msg = (
            f"\nForce-killing {in_flight_count} in-flight probe(s). "
            f"{KILL_GRACE_S:.0f}s grace, then hard exit."
        )
        if long_count:
            msg += f"  WARNING: {long_count} of these have been running >120s."
        print(msg, flush=True)
        # Cancel all in-flight worker tasks. run_probe's CancelledError
        # handler reaps the subprocess group.
        for task in list(state.in_flight.keys()):
            task.cancel()
        loop.call_later(KILL_GRACE_S, force_exit)

    loop.add_signal_handler(signal.SIGINT, on_sigint)


async def main_async(args: argparse.Namespace) -> int:
    target_shapes = [
        s for s in SHAPES
        if args.max_tier == 0 or max(s) <= args.max_tier
    ]

    validated = load_validated_shapes() if args.skip_existing else set()
    journaled = load_journaled_shapes() if args.skip_existing else set()
    already_solved = validated | journaled

    todo = [s for s in target_shapes if s not in already_solved]

    print(
        f"Target {len(target_shapes)} shapes — "
        f"{len(validated)} validated, {len(journaled)} journaled, "
        f"{len(todo)} to solve",
        flush=True,
    )

    if not todo:
        print("Nothing to do.")
        return 0

    # One-time sync setup: generate any missing symmetric Benes networks
    # before we start parallel work.
    for (n, m) in todo:
        if n == m:
            ensure_symmetric_network(n)

    # Build probe queue. Use a counter as a tiebreaker so the queue never
    # has to compare two Probe instances directly (Probe is hashable but
    # not orderable).
    queue: asyncio.PriorityQueue = asyncio.PriorityQueue()
    seq = count()
    total_probes = 0
    for shape in todo:
        n, m = shape
        tier = max(n, m)
        config = TIER_CONFIG.get(tier)
        if config is None:
            print(
                f"WARN: no tier config for shape {shape} (tier={tier}); skipping",
                file=sys.stderr,
            )
            continue
        for probe in generate_probes(n, m, config, args.solver):
            await queue.put((probe_priority(probe), next(seq), probe))
            total_probes += 1

    print(
        f"Queued {total_probes} probes across {len(todo)} shapes, "
        f"{args.workers} workers",
        flush=True,
    )

    state = State(
        solved_shapes=set(already_solved),
        journal_lock=asyncio.Lock(),
        counters={
            "probes_started": 0,
            "probes_unsat_or_timeout": 0,
            "encode_failed": 0,
            "skipped_solved": 0,
            "solved": 0,
            "raced": 0,
        },
        drain_event=asyncio.Event(),
        kill_event=asyncio.Event(),
    )

    install_signal_handlers(asyncio.get_running_loop(), state)

    workers = [
        asyncio.create_task(worker(f"w{i}", queue, state))
        for i in range(args.workers)
    ]
    # return_exceptions so a cancelled worker doesn't poison the gather.
    await asyncio.gather(*workers, return_exceptions=True)

    new_solves = sorted(state.solved_shapes - already_solved)
    print()
    if state.kill_event.is_set():
        print(f"Force-killed. {len(new_solves)} new solves committed: {new_solves}")
    elif state.drain_event.is_set():
        print(f"Drained. {len(new_solves)} new solves: {new_solves}")
    else:
        print(f"Done. {len(new_solves)} new solves: {new_solves}")
    print(f"Counters: {state.counters}")

    missing = [s for s in todo if s not in state.solved_shapes]
    if missing and not state.drain_event.is_set():
        print(f"NOT solved ({len(missing)}): {sorted(missing)}")
        return 1
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument(
        "--max-tier", type=int, default=10,
        help="Only attempt shapes where max(N,M) <= K (0 = no limit). Default: 10.",
    )
    p.add_argument(
        "--skip-existing", action="store_true",
        help="Skip shapes already present in the library OR the journal.",
    )
    default_workers = int(os.environ.get("BALANCER_WORKERS", "0")) or os.cpu_count() or 4
    p.add_argument("--workers", type=int, default=default_workers)
    p.add_argument("--solver", type=str, default="kissat404")
    args = p.parse_args()
    return asyncio.run(main_async(args))


if __name__ == "__main__":
    sys.exit(main())
