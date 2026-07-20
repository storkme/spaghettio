# balancer-gen session handoff

This doc captures the state of the throughput-priority-merges
balancer-gen workstream as of branch
`claude/throughput-priority-merges-YcXbi`. Use it to resume work on a
different machine without losing context.

## TL;DR — where we are

Phase 4 fully shipped; post-bake quality work landed on branch tip:

| Phase | Status | Notes |
|-------|--------|-------|
| 3.1 | done   | CP-SAT spike via Python+OR-tools subprocess |
| 3.2A.2 | done | Slot assignment as CP-SAT vars |
| 3.2D.1 | done | Synth-place fast path (south-only) |
| 3.2D.2 | done | UGs re-enabled + entity-count objective |
| 3.2D.3 | done | Splitter direction freedom (opt-in, MX2b on (1, 3)) |
| 3.3 bench | done | Mode D ≈ library on south-only corpus, (4, 4) -2 entities |
| 3.3 stress | done — negative result | Direct Mode D on (4, 9) Clos OOMs at every bbox tried |
| 4.1–4.3 | shipped, verified mechanically | compose_parallel, solve_pure_routing, compose_series wired and producing layouts |
| 4.4 | **done — verified MX3** | (4, 9) Clos via composition: junction_height=9, 284 entities, recovered topology classifies as `Balanced` |
| 5 — L=1 UG fix | **done** | `solve_pure_routing` UG arcs now start at L=2; 6 broken templates removed + re-baked via circuit encoder |
| 6 — lane gate | **done** | `bake_missing_shapes` now gates on `underground-belt` `Severity::Error` and retries with higher jh (up to 3 attempts) |
| 7 — smarter jh search | in progress (ug-plumber) | Two-budget climb in `compose_series`; RFC at `docs/rfc-smarter-jh-search.md` |

### The verified (4, 9) compose result

```
=== phase 4.4: (4, 9) Clos via compose_* ===
  stage1: parallel((1, 3), 4) = 12×9, 4 inputs, 12 outputs, 80 entities
  stage2: parallel((4, 3), 3) = 12×12, 12 inputs, 9 outputs, 111 entities
  clos_interleave(4, 3) perm: [0, 4, 8, 1, 5, 9, 2, 6, 10, 3, 7, 11]
  composed: 12×30, junction_height=9 (compose+route in 1173.3s)
    284 entities total
    classified: Balanced
  ✓ (4, 9) Clos placed via composition combinator and verified MX3
```

A reduced repro (`debug_compose_clos_2_2`, gated by
`SPAGHETTIO_DEBUG_2_2=1`) builds the same Beneš pattern on `(2, 2)` —
finds `junction_height=6`, recovers 8 edges, classifies `Balanced`.
Useful as a fast regression check (~10s) when iterating on the
junction model.

### What was actually broken (and fixed)

The handoff hypothesis blamed IO-port-ordering, but the bug was
deeper: `solve_pure_routing` in `crates/balancer-gen/scripts/place.py`
was structurally inadequate for the inter-stage junction. Two failure
modes, both invisible to per-edge conservation:

1. **Identity edges (`perm[i] == i`):** the source/destination
   coincide, so `is_src AND is_dst` forced `outflow == 0 AND
   inflow == 0`. The cell ends up empty. Stage1's south-emit drops
   into a void; the walker returns `None`; the splitter→splitter edge
   disappears from the recovered topology.
2. **Swap pairs:** the model's per-edge conservation was satisfied by
   placing edge 1's source-belt facing east at `(1, 0)` and edge 2's
   source-belt facing west at `(2, 0)`. Each edge's `inflow == 1` is
   ticked off by the *other* edge's source-arc — but physically those
   two belts face each other, so the walker steps east → west → east
   → west and returns `None` at the visited-set check.

The fix in `solve_pure_routing` is three small changes:

1. Allow south arcs at `y = jh-1` to leave the grid (junction exit
   into stage2's input belt).
2. Force a south-facing belt at every `dst` cell:
   `model.Add(arcs[(dst.x, dst.y, 2, e)] == 1)`.
3. Drop the `is_dst` special-case from conservation. The forced
   south-belt's off-grid south arc is the per-edge outflow; conservation
   becomes a single rule: `outflow == inflow + (1 if src else 0)`.

After these changes, the swap case at `jh=1` is correctly infeasible
(forced south-belt at edge 1's dst conflicts with edge 2's required
source-outflow, and vice versa), and the solver bumps `jh` until the
permutation can be routed without belt collisions.

## How to re-verify phase 4.4

The full spike runs all phases sequentially and ends with the (4, 9)
compose stress (the slow one — ~20 min on a release build). For a
fast smoke check, two env-var hatches in `main()` short-circuit
straight to the relevant phase:

```bash
# Fast (~10s after build): (2, 2) Beneš via compose, with diagnostics.
SPAGHETTIO_DEBUG_2_2=1 cargo run -p balancer-gen

# Slow (~20 min release): the headline (4, 9) Clos compose.
SPAGHETTIO_DEBUG_4_9=1 cargo run --release -p balancer-gen
```

Both should print `classified: Balanced` and a `✓` line.

## Setup on the new box

```bash
# Clone
git clone git@github.com:storkme/spaghettio.git
cd spaghettio
git checkout claude/throughput-priority-merges-YcXbi

# Rust
# (assumes rustup; project pins via rust-toolchain.toml if present)
cargo build -p balancer-gen

# Python solver — needs uv
curl -LsSf https://astral.sh/uv/install.sh | sh
# uv is invoked automatically by run_solver via the PEP 723 inline
# deps header in scripts/place.py — first run installs ortools (~28MB
# wheel) into an ephemeral venv.

# Verify Python solver works standalone
echo '{"n_splitters": 2, "bounds": [4, 4]}' \
  | uv run --no-project --script crates/balancer-gen/scripts/place.py

# Should print {"status": "OPTIMAL", "elapsed_s": ..., "splitters": [...]}.
```

If you see "uv: command not found" after install, restart your shell
or `source ~/.bashrc`.

## Reading list (in order)

1. `docs/rfc-balancer-place-routing.md` — full design history and
   decision log. The phase 3.3 stress test entry is the most recent
   substantive entry; phase 4 isn't yet documented there.
2. `crates/balancer-gen/src/main.rs` — Rust spike runner. Skim
   `main()` to see the phase progression. The phase 4 functions
   (`compose_parallel`, `compose_series`, `stress_compose_clos_4_9`,
   `smoke_compose_parallel`) live near the bottom.
3. `crates/balancer-gen/scripts/place.py` — Python CP-SAT solver.
   `solve_pure_routing` is the new function for phase 4.2; the
   other `solve_*` functions are from earlier phases.
4. `crates/core/src/bus/balancer_topology.rs` — `clos_interleave`,
   `parallel`, `series_permuted` graph-level combinators (these
   produce `SplitterGraph`s that `classify_graph` can verify).
5. `crates/core/src/bus/balancer_classify.rs` — `classify_ref` and
   the `BalancerClass` enum used to verify MX3.

## Recent commits worth knowing

```
3d3ec84 phase 4 — composition combinator [WIP — (4, 9) running]
c0d56c5 phase 3.3 stress result — Mode D doesn't scale to (4, 9) Clos
88298be wire (4, 9) Clos stress test [WIP — running]
923c1e8 phase 3.3 — bench Mode D vs library
32a9f61 phase 3.2D.3 — splitter direction freedom (opt-in)
8227279 phase 3.2D.2 — re-enable UGs in Mode D + entity-count objective
311c666 phase 3.2D.1 — synth-place via Mode D
5fd4d19 phase 3.2A.2 — CP-SAT picks slot assignments
```

## Next actions in priority order

1. **Phase 3.4 — bake compositions into the library.** Synth →
   SplitterGraph adapter, bbox heuristic, codegen for
   `BalancerTemplate` constants. Target the 37 missing-from-library
   shapes (issue #136). The (4, 9) Clos at 12×30 / 284 entities is a
   reasonable upper bound on what compose_* produces today; lots of
   shapes will be smaller (and more cells worth of routing slack would
   probably let the optimiser shave entity count further).
2. **Tune `solve_pure_routing` solve time.** (4, 9) takes ~20 minutes
   on release. Most of that is jh-search burn at infeasible heights
   before the solver can prove infeasibility. Two cheap wins likely
   available: (a) start the search at a heuristic jh ≈ ceil(log2(N))
   plus slack instead of jh=1, (b) cap per-attempt `max_time_s` at
   60-90s with a fallback to mark-as-infeasible-and-bump.
3. **RAM-headroom retest of direct Mode D**: with 20GB available,
   retry phase 3.3 stress on (4, 9) Clos at the bboxes that OOM'd on
   the old box (16×16, 12×18, 10×20, 9×24). Tells us whether direct
   Mode D is fundamentally capped at ~10 splitters or just RAM-capped
   on the old box. Result changes how we scope phase 3.4.

## Open questions / known issues

1. **Pre-existing CI failure on `partition_strategy_scoreboard`**:
   reproduces with our changes stashed. Not from this branch. User
   has acknowledged and is aware.

2. **MX2b on (1, 3) with direction freedom**: documented in the RFC.
   User said pure-balancers (MX3) are a separate workstream so MX2b
   is acceptable. No action needed.

3. **Width alignment in `compose_series`**: currently left-aligned
   with `composed_width = max(top.width, bot.width)`. If `top` and
   `bot` have different widths, the narrower one trails empty cells.
   For (4, 9) Clos, both stages are 12 wide so no padding needed.
   For other compositions, may need to revisit (RFC option: pad,
   re-layout, or shift).

4. **Junction height search is linear from jh=1**:
   `compose_series(top, bot, perm, initial_jh=1, max_jh=20)`. For
   small permutations this is fine; for larger N a smarter heuristic
   would skip over guaranteed-infeasible jh values. See "Next
   actions" #2.

## Code map quick reference

```
crates/balancer-gen/
├── Cargo.toml
├── src/
│   └── main.rs           ← spike runner, all Mode A/B/C/D + compose
├── scripts/
│   └── place.py          ← Python CP-SAT solver (5 entry points)

crates/core/src/bus/
├── balancer_classify.rs  ← classify_ref, BalancerClass, SplitterGraph
├── balancer_generate.rs  ← OwnedTemplate, runtime template builder
├── balancer_library.rs   ← static templates (63 shapes)
├── balancer_topology.rs  ← parallel, series_permuted, clos_interleave

docs/
├── rfc-balancer-place-routing.md  ← full design + decision log
├── balancer-gen-handoff.md        ← this file
```

## Solver entry points (place.py)

| Mode | Function | Trigger | Purpose |
|------|----------|---------|---------|
| A | `solve_overlap_only` | `req` has `n_splitters` + `bounds` only | Splitter no-overlap placement, no routing |
| B | `solve_routing` | `req` has `splitter_positions` + `edges` | Routing with given splitter layout |
| D (south) | `solve_synth_place` | `req` has `n_splitters` + `edges`, no `allow_dirs` (or `allow_dirs == [4]`) | Full synth-place, all-south splitters |
| D (dirs) | `solve_synth_place_dirs` | `req` has `allow_dirs` other than `[4]` | Full synth-place with direction freedom |
| Pure | `solve_pure_routing` | `req["kind"] == "pure_routing"` | Belt/UG routing between IO tiles, no splitters |

Dispatcher is at the bottom of `main()` in `place.py`.

## Mode D model size scaling (for capacity planning)

From phase 3.3 stress, OOM on a ~8GB box:

| splitters | edges | bbox tried | result |
|-----------|-------|------------|--------|
| 33 | 67 | 16×16 | OOM (exit 137) |
| 33 | 67 | 12×18 | OOM |
| 33 | 67 | 10×20 | OOM |
| 33 | 67 | 9×24  | OOM |

Rough model size at 9×24 with 67 edges:
- arc bool vars: 9 · 24 · 4 · 67 ≈ 58K
- UG bool vars: 9 · 24 · 5 · 67 ≈ 72K (south-only); 4× this for full direction freedom
- reified anchor + is_src/is_dst term bools: ~40K (estimate)
- conservation + at-most-one + UG pairing constraints: ~150K
- **total ~300K vars + 200K constraints**

CP-SAT memory per bool var with constraint participation is on the
order of 100-200 bytes, so the model alone is 50-100MB. The OS killed
us probably at 1-2GB resident which suggests CP-SAT's working set
during construction is much larger than the bare model.

On a 20GB box, expect direct Mode D to handle up to maybe 50 splitters
with similar bbox dimensions. Past that, decomposition (phase 4) is
mandatory.
