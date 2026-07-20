"""Async parallel runner for balancer SAT generation (Phase 1).

Phase 1 produces an append-only journal at ``scripts/balancer_journal.jsonl``.
The runner NEVER writes to ``src/bus/balancer_library.py``. A separate commit
script (Phase 2 of docs/rfc-balancer-runner.md) validates and merges journal
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
- N workers cooperate over an ``asyncio.PriorityQueue`` of probes.
  Default count is the smaller of ``os.cpu_count()`` and a memory-derived
  cap (see ``default_worker_count``); each kissat instance can peak at
  ~1-1.5 GB on tier-9 SAT, so blindly using all cores on a memory-poor
  host OOM-kills systemd. Override with ``--workers`` or
  ``BALANCER_WORKERS``.
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
ATTEMPTS_PATH = Path(__file__).parent / "balancer_attempts.jsonl"

def default_worker_count() -> int:
    """Pick a worker count that fits both core count and physical memory.

    A tier-9 kissat instance can peak around 1-1.5 GB RSS. With 16 workers
    on a 16 GB WSL host we OOM-killed systemd mid-run; assume ~1.5 GB per
    worker and reserve ~3 GB for the surrounding Python/Rust/Node stack.
    """
    cores = os.cpu_count() or 4
    try:
        with open("/proc/meminfo") as f:
            mem_kb = next(int(line.split()[1]) for line in f if line.startswith("MemTotal:"))
    except (OSError, StopIteration, ValueError):
        return cores
    mem_gb = mem_kb / (1024 * 1024)
    mem_cap = max(1, int((mem_gb - 3) / 1.5))
    return max(1, min(cores, mem_cap))


# Practical cap: tier 16 = (16, 16), already an unphysically large balancer.
MAX_PRACTICAL_TIER = 16

# All shapes up to MAX_PRACTICAL_TIER × MAX_PRACTICAL_TIER except (1,1) identity.
SHAPES: list[tuple[int, int]] = [
    (n, m)
    for n in range(1, MAX_PRACTICAL_TIER + 1)
    for m in range(1, MAX_PRACTICAL_TIER + 1)
    if (n, m) != (1, 1)
]

# Base (pass-0) per-tier search-space. Higher passes escalate width/height.
TIER_CONFIG: dict[int, dict] = {
    2: {"max_width": 21, "extra_heights": 2, "fast_timeout": 120, "use_full_interleave": False},
    3: {"max_width": 21, "extra_heights": 2, "fast_timeout": 120, "use_full_interleave": False},
    4: {"max_width": 21, "extra_heights": 2, "fast_timeout": 120, "use_full_interleave": False},
    5: {"max_width": 25, "extra_heights": 3, "fast_timeout": 120, "use_full_interleave": False},
    6: {"max_width": 30, "extra_heights": 3, "fast_timeout": 120, "use_full_interleave": True},
    7: {"max_width": 40, "extra_heights": 4, "fast_timeout": 120, "use_full_interleave": True},
    8: {"max_width": 40, "extra_heights": 4, "fast_timeout": 120, "use_full_interleave": True},
    9: {"max_width": 50, "extra_heights": 5, "fast_timeout": 180, "use_full_interleave": True},
    10: {"max_width": 50, "extra_heights": 5, "fast_timeout": 180, "use_full_interleave": True},
}


def base_tier_config(tier: int) -> dict:
    """Return the pass-0 config for any tier. Tiers >10 extrapolate."""
    if tier in TIER_CONFIG:
        return TIER_CONFIG[tier]
    # Tier 11+ extrapolation: each tier above 10 adds 5 to max_width and a bit
    # of head-room on heights/timeouts.
    extra = tier - 10
    return {
        "max_width": 50 + extra * 5,
        "extra_heights": 5 + extra,
        "fast_timeout": 180 + extra * 30,
        "use_full_interleave": True,
    }


# Practical ceiling on per-shape escalation. Once a shape's pass_index reaches
# this, further passes are skipped (the shape is intractable for us).
MAX_PASS_INDEX = 3

# Each pass beyond 0 enlarges the search rectangle and timeout budget.
PASS_WIDTH_STEP = 20
PASS_HEIGHT_STEP = 2
PASS_TIMEOUT_MULT = 1.5

NET_FREE_TIMEOUT_FLOOR = 300


def budget_for(tier: int, pass_index: int) -> dict:
    """Compute the per-shape budget for the given tier at escalation level
    ``pass_index`` (0 = base, 1+ = escalations)."""
    base = base_tier_config(tier)
    return {
        "max_width": base["max_width"] + pass_index * PASS_WIDTH_STEP,
        "extra_heights": base["extra_heights"] + pass_index * PASS_HEIGHT_STEP,
        "fast_timeout": int(base["fast_timeout"] * (PASS_TIMEOUT_MULT ** pass_index)),
        "use_full_interleave": base["use_full_interleave"],
    }


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


def generate_probes(
    n: int,
    m: int,
    config: dict,
    default_solver: str,
    prev_config: dict | None = None,
) -> list[Probe]:
    """Build the probe sequence for a single shape.

    If ``prev_config`` is given, only emit probes for the (width, height) pairs
    in ``config``'s rectangle that are NOT in ``prev_config``'s rectangle.
    This is the per-shape attempt frontier in action: pass P only re-explores
    the strict extension of pass P-1's swept area, never re-attempting probes
    that already failed at this solver/timeout.
    """
    base_h = max(n, m)
    max_width = config["max_width"]
    extra_heights = config["extra_heights"]
    fast_timeout = config["fast_timeout"]
    use_full_interleave = config["use_full_interleave"]

    prev_max_width = prev_config["max_width"] if prev_config else 2  # ie no widths covered
    prev_extra_heights = prev_config["extra_heights"] if prev_config else -1  # no heights covered

    network_path = SAT_DIR / "networks" / f"{n}x{m}"
    use_net_free = not network_path.exists()

    heights = [base_h + i for i in range(extra_heights + 1)]
    prev_heights_count = prev_extra_heights + 1  # number of heights already swept
    probes: list[Probe] = []

    def is_new(w: int, h_idx: int) -> bool:
        """Has this (w, height_index) been covered by a prior pass?"""
        if w > prev_max_width:
            return True
        return h_idx >= prev_heights_count

    if use_net_free:
        timeout = max(fast_timeout, NET_FREE_TIMEOUT_FLOOR)
        for h_idx, h in enumerate(heights):
            for w in range(3, max_width + 1):
                if is_new(w, h_idx):
                    probes.append(Probe((n, m), h, w, timeout, default_solver, "net_free"))
        return probes

    for h_idx, h in enumerate(heights):
        for w in range(3, max_width + 1):
            if not is_new(w, h_idx):
                continue
            probes.append(Probe((n, m), h, w, fast_timeout, default_solver, "fast"))
            if use_full_interleave and w >= base_h:
                probes.append(Probe((n, m), h, w, NET_FREE_TIMEOUT_FLOOR, default_solver, "full"))

    if not use_full_interleave:
        for h_idx, h in enumerate(heights):
            for w in range(3, max_width + 1):
                if is_new(w, h_idx):
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


def load_attempts() -> dict[tuple[int, int], int]:
    """Read per-shape pass index from the attempts journal.

    Returns a dict {shape: highest_pass_completed}. A shape not present in the
    dict has never been attempted (next pass = 0). The attempts file is
    append-only; the highest pass_index seen for each shape wins.
    """
    if not ATTEMPTS_PATH.exists():
        return {}
    out: dict[tuple[int, int], int] = {}
    with open(ATTEMPTS_PATH) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue  # partial line from a crash — ignore
            shape = entry.get("shape")
            pass_idx = entry.get("pass_index")
            if not (isinstance(shape, list) and len(shape) == 2 and isinstance(pass_idx, int)):
                continue
            key = (shape[0], shape[1])
            if pass_idx > out.get(key, -1):
                out[key] = pass_idx
    return out


def append_attempt(shape: tuple[int, int], pass_index: int, budget: dict, n_probes: int) -> None:
    """Record that ``shape`` has been swept at ``pass_index``. Append-only,
    fsynced. Used by ``--continuous`` mode to skip already-tried probes on
    the next round."""
    entry = {
        "timestamp": time.time(),
        "shape": list(shape),
        "pass_index": pass_index,
        "max_width": budget["max_width"],
        "extra_heights": budget["extra_heights"],
        "fast_timeout": budget["fast_timeout"],
        "n_probes_in_pass": n_probes,
    }
    line = json.dumps(entry) + "\n"
    fd = os.open(ATTEMPTS_PATH, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    try:
        os.write(fd, line.encode())
        os.fsync(fd)
    finally:
        os.close(fd)


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


def _sync_sigkill_group(pid: int) -> None:
    """Synchronously SIGKILL a process group. Safe to call from cancellation
    handlers and signal handlers — no awaits, no allocations beyond os.killpg.
    """
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


# Process tracker for emergency cleanup. Populated by run_probe /
# encode_blueprint as they spawn subprocesses; entries removed when the
# subprocess returns naturally. Used by the SIGTERM handler to reap groups
# we'd otherwise orphan.
_LIVE_SUBPROCESSES: set[int] = set()


async def kill_process_group(proc: asyncio.subprocess.Process) -> None:
    """SIGTERM the process group (5s grace) then SIGKILL. Use only for the
    *polite* shutdown path (per-probe timeout). For the cancellation path
    use ``_sync_sigkill_group`` directly — its awaits are cancellation-safe
    while ours are not.

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
        await asyncio.wait_for(proc.wait(), timeout=5)
        return
    except asyncio.TimeoutError:
        pass
    except asyncio.CancelledError:
        # The outer task was cancelled while we were waiting on SIGTERM. Fall
        # through to SIGKILL synchronously and re-raise.
        _sync_sigkill_group(proc.pid)
        raise
    _sync_sigkill_group(proc.pid)
    try:
        await asyncio.wait_for(proc.wait(), timeout=5)
    except (asyncio.TimeoutError, asyncio.CancelledError):
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
    _LIVE_SUBPROCESSES.add(proc.pid)
    try:
        try:
            stdout, _ = await asyncio.wait_for(proc.communicate(), timeout=probe.timeout_s)
        except asyncio.TimeoutError:
            await kill_process_group(proc)
            return None
        except asyncio.CancelledError:
            # Force-kill path: do NOT await — the await chain may itself be
            # re-cancelled, leaving the subprocess group orphaned. SIGKILL
            # synchronously and re-raise.
            _sync_sigkill_group(proc.pid)
            raise
    finally:
        _LIVE_SUBPROCESSES.discard(proc.pid)
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
    _LIVE_SUBPROCESSES.add(proc.pid)
    try:
        try:
            stdout, _ = await asyncio.wait_for(proc.communicate(input=raw), timeout=30)
        except asyncio.TimeoutError:
            await kill_process_group(proc)
            return None
        except asyncio.CancelledError:
            _sync_sigkill_group(proc.pid)
            raise
    finally:
        _LIVE_SUBPROCESSES.discard(proc.pid)
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
            f"\nGrace period ({KILL_GRACE_S:.0f}s) exceeded. Reaping "
            f"{len(_LIVE_SUBPROCESSES)} remaining subprocess group(s) and "
            f"hard-exiting.",
            file=sys.stderr,
            flush=True,
        )
        for pid in list(_LIVE_SUBPROCESSES):
            _sync_sigkill_group(pid)
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

    def on_sigterm() -> None:
        """Emergency exit on SIGTERM. Reap all known subprocess groups
        synchronously then hard-exit. SIGTERM commonly comes from the
        ``timeout`` command, ``systemctl stop``, or ``pkill`` — none of
        which give us time to drain politely."""
        print(
            f"\nSIGTERM received. Reaping {len(_LIVE_SUBPROCESSES)} subprocess "
            f"group(s) and exiting.",
            file=sys.stderr,
            flush=True,
        )
        for pid in list(_LIVE_SUBPROCESSES):
            _sync_sigkill_group(pid)
        os._exit(143)

    loop.add_signal_handler(signal.SIGTERM, on_sigterm)


def select_pass_shapes(
    args: argparse.Namespace,
    validated: set[tuple[int, int]],
    journaled: set[tuple[int, int]],
    attempts: dict[tuple[int, int], int],
    continuous: bool,
) -> tuple[list[tuple[int, int]], int | None]:
    """Pick shapes for the next pass.

    Single-pass mode: every unsolved shape up to ``max_tier``.
    Continuous mode: only shapes in the lowest tier that has any unsolved
    members (frontier focus) — finishes the grid before moving up.

    Returns (shapes_for_pass, frontier_tier_or_None). frontier_tier is set
    only in continuous mode and used for logging.
    """
    already_solved = validated | journaled
    cap = args.max_tier
    candidates = [s for s in SHAPES if cap == 0 or max(s) <= cap]
    unsolved = [s for s in candidates if s not in already_solved]

    if not continuous:
        return unsolved, None

    # Frontier focus: lowest tier with any missing shape, AND that tier's
    # shapes haven't all hit the escalation ceiling.
    by_tier: dict[int, list[tuple[int, int]]] = {}
    for s in unsolved:
        by_tier.setdefault(max(s), []).append(s)

    for tier in sorted(by_tier):
        # Skip shapes that have hit the escalation ceiling.
        eligible = [s for s in by_tier[tier] if attempts.get(s, -1) + 1 <= MAX_PASS_INDEX]
        if eligible:
            return eligible, tier
    return [], None


async def run_pass(
    args: argparse.Namespace,
    pass_shapes: list[tuple[int, int]],
    attempts: dict[tuple[int, int], int],
    already_solved: set[tuple[int, int]],
) -> tuple[State, dict[tuple[int, int], dict]]:
    """Build a probe queue for ``pass_shapes`` and run it to completion.

    Returns (state, per_shape_budget) where per_shape_budget is the budget
    actually used for each shape this pass — written to the attempt log
    afterward so the next pass extends from here.
    """
    # Make sure any symmetric Benes networks we'll need exist.
    for (n, m) in pass_shapes:
        if n == m:
            ensure_symmetric_network(n)

    queue: asyncio.PriorityQueue = asyncio.PriorityQueue()
    seq = count()
    total_probes = 0
    per_shape_budget: dict[tuple[int, int], dict] = {}

    for shape in pass_shapes:
        n, m = shape
        tier = max(n, m)
        last_pass = attempts.get(shape, -1)
        next_pass = last_pass + 1
        if next_pass > MAX_PASS_INDEX:
            continue  # ceiling — skip
        budget = budget_for(tier, next_pass)
        prev_budget = budget_for(tier, last_pass) if last_pass >= 0 else None
        probes = generate_probes(n, m, budget, args.solver, prev_config=prev_budget)
        for probe in probes:
            await queue.put((probe_priority(probe), next(seq), probe))
        total_probes += len(probes)
        per_shape_budget[shape] = {"budget": budget, "pass_index": next_pass, "n_probes": len(probes)}

    print(
        f"Queued {total_probes} probes across {len(pass_shapes)} shapes, "
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
    await asyncio.gather(*workers, return_exceptions=True)

    return state, per_shape_budget


async def main_async(args: argparse.Namespace) -> int:
    pass_num = 0
    grand_total_solved: set[tuple[int, int]] = set()

    while True:
        validated = load_validated_shapes() if args.skip_existing or args.continuous else set()
        journaled = load_journaled_shapes() if args.skip_existing or args.continuous else set()
        attempts = load_attempts() if args.continuous else {}
        already_solved = validated | journaled

        pass_shapes, frontier_tier = select_pass_shapes(
            args, validated, journaled, attempts, args.continuous
        )

        if not pass_shapes:
            if args.continuous:
                # Either everything is solved, or remaining shapes have all
                # hit the escalation ceiling.
                print()
                print(
                    "Continuous mode: nothing more to attempt. "
                    "Either all targets solved, or remaining shapes have hit "
                    f"the per-shape escalation ceiling (pass_index > {MAX_PASS_INDEX}).",
                    flush=True,
                )
                return 0
            print("Nothing to do.")
            return 0

        if pass_num == 0:
            print(
                f"Target {len(SHAPES)} shape positions — "
                f"{len(validated)} validated, {len(journaled)} journaled. "
                f"Mode: {'continuous' if args.continuous else 'single-pass'}.",
                flush=True,
            )

        if args.continuous:
            print()
            print(f"=== Pass {pass_num} (frontier tier: {frontier_tier}) ===", flush=True)
            print(
                f"  {len(pass_shapes)} shape(s) eligible: {sorted(pass_shapes)}",
                flush=True,
            )

        state, per_shape_budget = await run_pass(args, pass_shapes, attempts, already_solved)

        # Record attempt frontier ONLY if the pass completed naturally — a
        # drained or force-killed pass swept only part of the rectangle and
        # writing it as "fully attempted" would cause the next pass to skip
        # legitimately-pending probes.
        pass_completed_naturally = not (state.kill_event.is_set() or state.drain_event.is_set())
        if pass_completed_naturally:
            for shape, info in per_shape_budget.items():
                append_attempt(shape, info["pass_index"], info["budget"], info["n_probes"])

        new_solves = sorted(state.solved_shapes - already_solved)
        grand_total_solved.update(new_solves)

        # Per-pass summary
        print()
        if state.kill_event.is_set():
            print(f"Pass {pass_num} force-killed. New solves: {new_solves}")
        elif state.drain_event.is_set():
            print(f"Pass {pass_num} drained. New solves: {new_solves}")
        else:
            print(f"Pass {pass_num} complete. New solves: {new_solves}")
        print(f"Counters: {state.counters}")

        # If this pass was killed/drained, stop the outer loop too.
        if state.drain_event.is_set() or state.kill_event.is_set():
            break

        if not args.continuous:
            # single-pass mode: we're done after one pass
            break

        pass_num += 1

    if args.continuous:
        print()
        if state.kill_event.is_set():
            verb = "force-killed"
        elif state.drain_event.is_set():
            verb = "drained"
        else:
            verb = "finished"
        print(
            f"Continuous run {verb}. Total new solves across all passes: "
            f"{len(grand_total_solved)} {sorted(grand_total_solved)}"
        )
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument(
        "--max-tier", type=int, default=0,
        help=(
            "Only attempt shapes where max(N,M) <= K (0 = no limit). "
            "Default: 0 (unbounded — runs up to MAX_PRACTICAL_TIER = "
            f"{MAX_PRACTICAL_TIER})."
        ),
    )
    p.add_argument(
        "--skip-existing", action="store_true",
        help="Skip shapes already present in the library OR the journal.",
    )
    p.add_argument(
        "--continuous", action="store_true",
        help=(
            "Run pass after pass until killed. Each pass focuses on the "
            "lowest-tier still-unsolved shapes (frontier focus); per-shape "
            "attempt state in scripts/balancer_attempts.jsonl ensures "
            "subsequent passes only try (width, height) pairs not yet "
            "attempted at that solver/timeout. Implies --skip-existing."
        ),
    )
    default_workers = int(os.environ.get("BALANCER_WORKERS", "0")) or default_worker_count()
    p.add_argument("--workers", type=int, default=default_workers)
    p.add_argument("--solver", type=str, default="kissat404")
    args = p.parse_args()
    if args.continuous:
        args.skip_existing = True
    return asyncio.run(main_async(args))


if __name__ == "__main__":
    sys.exit(main())
