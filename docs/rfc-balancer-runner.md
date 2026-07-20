# RFC: Parallel balancer-generation runner

## Summary

Replace `scripts/generate_balancer_library.py`'s shape-level
`ProcessPoolExecutor` with a finer-grained async task runner that keeps
all CPU cores busy for the duration of a tier-9/tier-10 generation run.
Add a strict separation between the runner (writes only to an
append-only journal) and a commit step (validates and merges journal
entries into `src/bus/balancer_library.py`). The existing
`sync_balancer_to_rust.py` step is unchanged.

The infrastructure is shaped so a future junction-zone-precomputation
consumer can reuse it, but that work is **out of scope here**.

## Motivation

Concrete failure case observed during the in-progress tier-9 run:

- Current script: `ProcessPoolExecutor(max_workers=min(cpu_count,
  len(todo)))` — one worker per shape (17 shapes, 16 cores).
- Each worker sweeps `for height in heights: for width in range(3,
  max_width+1):` sequentially, calling `belt_balancer_net_free` once
  per probe (subprocess, single-threaded).
- As easy shapes complete (e.g. `(1,9)` solved in <5 minutes) their
  workers retire. Workers are not refilled.
- Current observed state: 17 → 16 → 15 → ... → 2-3 active workers as
  the easy shapes drain. The hardest 2-3 shapes (likely `(8,9)`,
  `(9,8)`, `(7,9)`) will run on 2-3 cores for the long tail of the
  overnight run while 13+ cores sit idle.

Estimated waste: 30-50% of overnight wall-clock on a 16-core machine.
The waste grows as tier-10 (which has even more asymmetric net_free
shapes) is added.

## Design

### Task model

Atomic unit of work is a single `(shape, height, width)` probe:

```python
@dataclass(frozen=True)
class Probe:
    shape: tuple[int, int]   # (n, m)
    height: int              # SAT height (pre-rotation)
    width: int               # SAT width (pre-rotation)
    timeout_s: int
    solver: str              # "kissat404" | "glucose3" | ...
    mode: str                # "fast" | "full" | "net_free"
```

Each probe spawns one `belt_balancer*` subprocess. For tier-9 with
`max_width=50, extra_heights=5`: 17 shapes × 6 heights × 48 widths ≈
**4,900 probes total**.

### Priority ordering

Probes ordered by:

1. `(max(n,m), n+m)` — easier shapes first (cheap progress early).
2. Within shape: `(height, width)` ascending — so the smallest grid
   that solves the shape is found first (preserves the
   "near-minimal solution" property of the current script).

The runner only schedules a probe if its shape is not yet marked
solved. Once a probe succeeds, the shape's remaining queued probes are
discarded without execution. In-flight probes for the now-solved shape
are *not* killed — kissat doesn't react cleanly to `SIGTERM` mid-solve;
we let them run to completion (or timeout) and discard the result.

Worst-case wasted work after a shape solves: `min(in_flight, workers)
× timeout_s`. With `workers=16`, `timeout_s=300`, that's bounded at ~4800
core-seconds per shape. Across 17 shapes that's ~22 core-hours of
worst-case waste — but in practice only the *last* probe per shape
overlaps with the solving probe.

### Concurrency model

`asyncio` in a single Python process. `os.cpu_count()` worker slots via
`asyncio.Semaphore`. Each "worker" is just a coroutine pulling from the
priority queue:

```python
async def worker(queue, sem, state):
    while not state.done:
        probe = await queue.get_high_priority(state.solved_shapes)
        if probe is None:
            return
        async with sem:
            result = await run_probe(probe)
        if result is not None and probe.shape not in state.solved_shapes:
            state.solved_shapes.add(probe.shape)
            await journal.append(probe.shape, result)
```

Why asyncio over multiprocessing:
- Workload is pure subprocess I/O wait — no Python-side CPU work.
- No GIL contention.
- Lightweight cancellation: removing a probe from the queue is O(1).
- Single-process state (the solved-shape set) is simpler than IPC.

### Persistence: append-only journal

Two files in `scripts/`:

- `balancer_library.py` (in `src/bus/`) — **validated truth**. The
  runner *never writes here*.
- `balancer_journal.jsonl` — append-only log. One line per solved
  template, written with `flush + fsync` after each successful probe.

```jsonl
{"timestamp": "...", "shape": [4, 9], "width": 18, "height": 9, "solver": "kissat404", "blueprint": "0eN..."}
{"timestamp": "...", "shape": [9, 4], "width": 18, "height": 9, "solver": "kissat404", "blueprint": "0eN..."}
```

On startup:
1. Load `balancer_library.py` → set of already-validated shapes.
2. Replay `balancer_journal.jsonl` → set of solved-but-uncommitted shapes.
3. `todo = target_shapes - validated - journaled`.

The runner never deletes or rewrites the journal. A separate
`scripts/commit_balancer_journal.py` is the only thing that writes
`balancer_library.py`:

1. Read existing `balancer_library.py` → `templates_dict`.
2. Read `balancer_journal.jsonl`.
3. For each journal entry:
   - Decode the blueprint, run the existing extract/rotate/normalize
     pipeline, build a `BalancerTemplate`.
   - **Validate**: round-trip the blueprint, sanity-check ports against
     `(n, m)`, check footprint area is reasonable.
   - If validation fails: print error, leave entry in journal, skip.
4. Merge: `templates_dict[shape] = new_template` (only if shape not
   already present — never overwrite).
5. Atomic write: `balancer_library.py.tmp` → `os.replace()` →
   `balancer_library.py`.
6. Truncate journal to retain only entries that failed validation.

Atomic rename + never-overwrite-existing means:
- Crash mid-write of library: `.tmp` is discarded, library is
  untouched.
- Bad solve: easy to spot in journal, easy to delete the bad line, no
  side-effects on library.
- Concurrent runners (two terminals): both append to journal safely;
  commit step deduplicates.

### Configuration

Same difficulty knobs as today, exposed as constants at the top of
`balancer_runner.py`:

```python
SOLVER_DEFAULT = "kissat404"
TIMEOUT_NET_FREE = 300
TIMEOUT_FAST = 180
TIMEOUT_FULL = 300

# Per-tier search-space limits (matches existing find_balancer)
TIER_LIMITS = {
    9: {"max_width": 50, "extra_heights": 5},
    10: {"max_width": 50, "extra_heights": 5},
    8: {"max_width": 40, "extra_heights": 4},
    # ...
}
```

`BALANCER_WORKERS` env var override remains, defaults to `os.cpu_count()`.

### File structure

```
scripts/
  balancer_runner.py            # NEW — async task queue
  commit_balancer_journal.py    # NEW — validates + merges journal into library
  sync_balancer_to_rust.py      # EXISTING, unchanged
  generate_balancer_library.py  # EXISTING, kept until parity proven
  balancer_journal.jsonl        # NEW — append-only solve log
  balancer_checkpoint.json      # EXISTING (used by old script; deprecated post-migration)
src/bus/
  balancer_library.py           # only updated by commit script
crates/core/src/bus/
  balancer_library.rs           # only updated by sync script
```

### Trade-offs considered

- **Multiprocessing instead of asyncio** — rejected. The orchestrator
  has no CPU work; subprocesses do. Asyncio + `asyncio.create_subprocess_exec`
  is enough and gives O(1) cancellation.
- **Rust orchestrator with `tokio::process`** — rejected for now. The
  SAT subprocesses are Python (`belt_balancer_net_free`); a Rust
  orchestrator would just shell out the same way. Adds toolchain
  complexity for no measurable speedup.
- **No journal, write directly to library** — rejected. Violates the
  user's explicit "must not corrupt validated templates" requirement.
  An atomic-rename of the library every solve is technically safe but
  noisy (4900 atomic renames per tier-9 run) and harder to inspect.
- **Within-shape parallelism only (smaller scope)** — possible
  intermediate step, but doesn't address the long tail. A single
  `(shape, h, w)` task granularity is barely more code and fully solves
  the utilisation problem.
- **Killing in-flight subprocesses on shape-solve** — rejected.
  `kissat` doesn't react cleanly to `SIGTERM` mid-solve, and the
  bounded-waste analysis above shows it's not necessary.

## Kill criteria

- **Throughput regression.** If the new runner produces fewer solved
  shapes per hour on tier-9 generation than the existing script on the
  same machine, the asyncio overhead is unexpectedly large and we
  should revert.
- **Library corruption in any test.** If `commit_balancer_journal.py`
  produces a `balancer_library.py` that fails to import, or that
  contains a template with mismatched `(n, m)` vs `input_tiles` /
  `output_tiles` lengths, or that overwrites a previously-validated
  template — kill it. The whole point of this RFC is correctness, not
  speed.
- **Parity break.** If the runner generates a different template
  geometry than the old script for any tier ≤ 8 shape (re-solving from
  scratch, comparing footprints), we have a probe-ordering bug and
  should fix or revert before relying on it for tier-9/10.
- **Resume corruption.** If killing the runner mid-run and restarting
  it reproduces a different set of shapes than running uninterrupted,
  the resume logic is broken — kill until fixed.
- **Per-shape waste >2× current.** If post-solve "wasted" core-seconds
  exceed 2× what the current script wastes (i.e., the
  no-cancellation strategy turns out to be too lossy), reconsider
  adding cancellation via `proc.kill()`.

## Verification plan

Following [the layout-engine verification
protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Parity check on tier ≤ 8.** Move existing
   `src/bus/balancer_library.py` aside. Run the new runner against
   `--max-tier 8` (matches what's already shipped). Run
   `commit_balancer_journal.py`. Diff the regenerated file against the
   archived one — shape coverage must match exactly. Footprints may
   differ by ±1 (different solver, different probe order); inspect any
   shape with >1 difference.
2. **Resume safety test.** Start tier-9 generation. After ~5 minutes,
   `kill -9` the process. Restart. Confirm:
   - Journal still parses cleanly.
   - Already-solved shapes are not re-attempted.
   - No partial JSON line in the journal (atomic line append works).
3. **Concurrent runner safety.** Start two runners pointing at the
   same journal. Confirm both make progress without journal
   corruption. (This isn't a normal use case but it shouldn't break.)
4. **Commit dry-run.** Add a `--dry-run` flag to
   `commit_balancer_journal.py` that prints the diff without writing.
   Use it during development to inspect proposed library updates.
5. **Cargo tests stay green** after `sync_balancer_to_rust.py` runs
   against a regenerated library: `cargo test --manifest-path
   crates/core/Cargo.toml`. Specifically `tier9_*` shape lookups in
   `balancer.rs` must still resolve.
6. **Web-app smoke.** Open the tier-9 problem case (the PU@3/s from
   ore that motivates this work) in `npm run dev`. Confirm the new
   `(4,9)` template stamps correctly and validators are happy.

## Phasing

- **Phase 1 — runner + journal**, no commit step.
  Ship `balancer_runner.py` + journal format. Validate it produces
  correct journal entries by running on tier-7/tier-8 shapes (already
  in the library) and diffing journal output against existing entries.
  No changes to `balancer_library.py` yet. Land behind a feature gate
  (just don't run it on a real generation pass).
- **Phase 2 — commit script**.
  Ship `commit_balancer_journal.py` with `--dry-run`. Validate parity
  by re-deriving tier-1..8 from scratch and confirming the merged
  library matches the existing one (modulo solver-induced footprint
  variation, which we'll inspect).
- **Phase 3 — switch tier-9/10 to the runner**.
  Use the new runner for the in-progress tier-9 work. Old script stays
  in tree for one more cycle as fallback.
- **Phase 4 — retire old script** once Phase 3 has produced at least
  one full tier-9/tier-10 corpus end-to-end.
- **Future (out of scope)**: junction zone precomputation. The runner's
  `Probe` abstraction generalises to any SAT-shaped job; we leave a
  comment noting this but don't build it.

## Decision log

- *2026-04-29 — drafted. Awaiting user approval before implementation.*
