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
| 4.1–4.3 | shipped, verified mechanically | compose_parallel, solve_pure_routing, compose_series wired and producing layouts |
| 4.4 | **fails — `Singular` classification** | (4, 9) Clos via composition runs end-to-end (junction_height=10 in 354s) but the placed template is Singular, not Balanced |

### The verified (4, 9) compose result (from the previous box)

```
=== phase 4.4: (4, 9) Clos via compose_parallel + compose_series ===
  stage1: parallel((1, 3), 4) = 12×9, 4 inputs, 12 outputs, 80 entities
  stage2: parallel((4, 3), 3) = 12×12, 12 inputs, 9 outputs, 111 entities
  clos_interleave(4, 3) perm: [0, 4, 8, 1, 5, 9, 2, 6, 10, 3, 7, 11]
  composed: 12×31, junction_height=10 (compose+route in 354.6s)
    279 entities total
  ✗ compose stress: classify_ref: Singular
```

Mechanical pieces work: `compose_parallel` produces correctly-sized
templates, `solve_pure_routing` finds a permutation routing in 354s,
`compose_series` stitches the result without panicking. **But the
classifier reports `Singular`** — the recovered graph from the placed
entities has a degenerate flow matrix.

`classify_graph(big)` on the *source* topology reports `Balanced`
(verified in `stress_clos_4_9` in 3.3 wiring). So the source topology
is fine; what's wrong is the mapping from source topology to placed
entities.

This is Open Question #2 — IO-port-ordering between `parallel`
(graph-level, in `balancer_topology.rs`) and `compose_parallel`
(template-level, in `main.rs`) almost certainly disagree. The fix
likely lives in `compose_parallel`'s output ordering or in how
`compose_series` indexes the perm.

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

1. **Debug the (4, 9) compose `Singular` result** — see "Debugging
   approach" below. This is the next critical step; phase 4 and 3.4
   both depend on getting this right.
2. **Once 4.4 passes**: kick off phase 3.4 — synth → SplitterGraph
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

## Debugging approach for the Singular result

The classifier walks placed entities to recover a `SplitterGraph`,
then runs the same Gaussian-elimination flow analysis on the
recovered graph. `Singular` means the recovered graph has a
degenerate flow matrix.

Likely causes (most → least likely):

1. **Port ordering mismatch.** `compose_parallel` numbers ports as
   `c * atom.n_ports + i` (i.e., copy-major). `parallel` in
   `balancer_topology.rs` may number ports differently — check it.
   If they differ, then `clos_interleave`'s indices don't point at
   the same physical lanes that `compose_series` is feeding into the
   permutation.

2. **`(1, 3)` atom uses a back-loop**, with one west-facing splitter.
   `compose_parallel` blindly stamps the atom 4 times side-by-side,
   shifted by `template.width = 3`. The west-facing splitter in copy
   `c` emits west *into copy c-1's territory* (since "west" means
   smaller x, and copy c's anchor x = (c-1) * atom.width + something).
   This may cause the back-loop edges to fold into a neighbouring
   atom instead of the same atom.

   Quick test: try `compose_parallel(library_atom_for_dyadic_shape, k)`
   on a shape that's all-south, e.g., `parallel((1, 4), 2) = (2, 8)`.
   If that classifies correctly through `compose_parallel + identity
   compose_series`, the bug is specific to non-south atoms.

3. **`compose_series` IO tile mapping wrong**. The perm might be
   getting applied as input_tile_index→output_tile_index but the
   classifier expects something else. Print
   `top.output_tiles` and `bot.input_tiles` and trace one perm entry
   manually.

### Concrete debug steps

Run with `RUST_LOG=trace` and sprinkle `eprintln!` in:
- `compose_parallel`: print each copy's IO tile positions.
- `compose_series`: print `junction_input_tiles` and the resolved
  `junction_output_tiles` for the chosen `junction_height`.
- After `compose_series` returns, before `classify_ref`, print
  `composed.input_tiles` and `composed.output_tiles`.

Then walk through `clos_interleave(4, 3)` by hand for the simpler
case `clos_interleave(2, 2)` (= `[0, 2, 1, 3]`) and verify the
composed template would route lane 0's input to lane 0's output (etc.)
through the Clos network.

If port ordering is the issue, the fix is to make `compose_parallel`
number ports the same way `parallel` does. `parallel`'s ordering is
the canonical one since the classifier and verifier both rely on it.

### Faster iteration alternative

Instead of `(4, 9)` (354s solve time), reproduce the Singular issue
on a smaller composition first. Try `(2, 4)` Clos:
```
parallel((1, 2), 2) → clos_interleave(2, 2) → parallel((2, 1), 2)
```
That's `(2, 4)` from atoms — only 6 splitters, junction routing
trivial. If that classifies Balanced via composition, the (4, 9)
issue is elsewhere; if it's Singular, the bug is in the simpler case
and easier to debug.

## Open questions / known issues

1. **Pre-existing CI failure on `partition_strategy_scoreboard`**:
   reproduces with our changes stashed. Not from this branch. User
   has acknowledged and is aware.

2. **(4, 9) Clos compose IO-port-ordering — confirmed broken**: this
   was a hypothesis pre-result; the result is in (Singular
   classification). See "Debugging approach for the Singular result"
   above. Likely fix is in `compose_parallel`'s ordering or in how
   `compose_series` indexes the perm.

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
