# balancer-gen session handoff

This doc captures the state of the throughput-priority-merges
balancer-gen workstream as of branch
`claude/throughput-priority-merges-YcXbi` tip `3d3ec84`. Use it to
resume work on a different machine without losing context.

## TL;DR — where we are

Five phases shipped, one in flight:

| Phase | Status | Notes |
|-------|--------|-------|
| 3.1 | done   | CP-SAT spike via Python+OR-tools subprocess |
| 3.2A.2 | done | Slot assignment as CP-SAT vars |
| 3.2D.1 | done | Synth-place fast path (south-only) |
| 3.2D.2 | done | UGs re-enabled + entity-count objective |
| 3.2D.3 | done | Splitter direction freedom (opt-in, MX2b on (1, 3)) |
| 3.3 bench | done | Mode D ≈ library on south-only corpus, (4, 4) -2 entities |
| 3.3 stress | done — negative result | Direct Mode D on (4, 9) Clos OOMs at every bbox tried |
| 4.1–4.3 | shipped, **unverified** | compose_parallel, solve_pure_routing, compose_series wired |
| 4.4 | **incomplete** | (4, 9) Clos via composition test was running >30min on the previous box and got killed at session end |

The big unknown right now is whether **phase 4.4 actually works** —
whether `compose_series` produces a layout that classifies MX3 for the
(4, 9) Clos topology. The wiring is in `main.rs`, untested end-to-end
because the previous box couldn't finish the run.

## What needs verification on the new box

Run the spike and look at the phase 4.4 section:

```bash
cargo run -p balancer-gen 2>&1 | tail -100
```

Phase 4.4 will print:

```
=== phase 4.4: (4, 9) Clos via compose_parallel + compose_series ===
  stage1: parallel((1, 3), 4) = 12×9, ...
  stage2: parallel((4, 3), 3) = ...
  clos_interleave(4, 3) perm: [0, 4, 8, 1, 5, 9, 2, 6, 10, 3, 7, 11]
  composed: WxH, junction_height=N (compose+route in T.Ts)
    M entities total
    classified: ...
```

Three possible outcomes:

1. **Classification = `Balanced`**: ✓ phase 4 done. Compose
   combinator works; commit "phase 4.4 verified MX3" and proceed to
   phase 3.4 (bake into library_extra).
2. **Classification ≠ `Balanced`**: layout-vs-graph mismatch. The
   topology is MX3 (verified by `classify_graph(big)` in the existing
   stress_clos_4_9 function in 3.3) but the placed template
   classifies as something else — likely the IO port ordering of
   parallel composition isn't lining up with the perm. See "Open
   questions" #2 below.
3. **Subprocess timeout / infeasible at every junction_height**: the
   junction routing CP-SAT subproblem can't find a permutation
   layout in the candidate junction_height range `1..=12`. Try
   bumping `max_jh` parameter in `stress_compose_clos_4_9` (in
   `crates/balancer-gen/src/main.rs`). Or the perm is being passed
   wrong — print the resolved source/dest cells from
   `compose_series` to see.

## Setup on the new box

```bash
# Clone
git clone git@github.com:storkme/fucktorio.git
cd fucktorio
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

1. `docs/rfp-balancer-place-routing.md` — full design history and
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

1. **Verify phase 4.4** — run the spike, check the (4, 9) Clos
   composition classifies MX3. See "What needs verification" above.
2. **If 4.4 passes**: kick off phase 3.4 — synth → SplitterGraph
   adapter, bbox heuristic, codegen for `BalancerTemplate` constants.
   Target the 37 missing-from-library shapes (issue #136). RAM
   headroom on the new box also unblocks retrying direct Mode D on
   smaller compositions like (3, 5) or (5, 3) where the
   sideloading-classify-as-MX2b limitation was the only blocker
   (these need direction freedom and Mode D, OOM on the old box).
3. **RAM-headroom retest**: with 20GB available, retry phase 3.3
   stress on (4, 9) Clos at the bboxes that OOM'd on the old box
   (16×16, 12×18, 10×20, 9×24). Tells us whether direct Mode D is
   fundamentally capped at ~10 splitters or just RAM-capped on the
   old box. Result changes how we scope phase 3.4.

## Open questions / known issues

1. **Pre-existing CI failure on `partition_strategy_scoreboard`**:
   reproduces with our changes stashed. Not from this branch. User
   has acknowledged and is aware.

2. **(4, 9) Clos compose IO-port-ordering risk**: `compose_parallel`
   concatenates IO tiles in copy order: input port `c * n_in + i` is
   the i-th input of the c-th copy. `clos_interleave(4, 3)` indexes
   into the post-parallel port list. If `parallel` and
   `compose_parallel` index ports differently, the perm is
   semantically wrong and the composed layout won't classify MX3.
   Worth double-checking by printing the input/output cells in
   `compose_series` and tracing one lane through the topology. The
   relevant code: `parallel` in `balancer_topology.rs` (graph-level)
   vs `compose_parallel` in `main.rs` (template-level). If they
   diverge, the fix is to make `compose_parallel`'s ordering match
   `parallel`'s.

3. **MX2b on (1, 3) with direction freedom**: documented in the RFP.
   User said pure-balancers (MX3) are a separate workstream so MX2b
   is acceptable. No action needed.

4. **Width alignment in `compose_series`**: currently left-aligned
   with `composed_width = max(top.width, bot.width)`. If `top` and
   `bot` have different widths, the narrower one trails empty cells.
   For (4, 9) Clos, both stages are 12 wide so no padding needed.
   For other compositions, may need to revisit (RFP option: pad,
   re-layout, or shift).

5. **Junction height search bounds**: hardcoded
   `initial_junction_height = 1`, `max_junction_height = 12` for the
   (4, 9) test. Search is linear; for larger N may need a smarter
   heuristic.

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
├── rfp-balancer-place-routing.md  ← full design + decision log
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
