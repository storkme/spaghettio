# Handoff — CP-SAT placer Phase 4 (UG-belt unblock for coprime shapes)

Branch: `claude/cp-sat-lane-balancer` (6 commits ahead of `main`).

**Status update 2026-05-02 (later):** Phase 4 a.3 begun — `(1, 6)`
shipped (no lane balancer needed; 2 feedback arcs at 0.167 each fit
M.in1 lane 0 directly). Adding the test surfaced and fixed a real
bug in `_route_belts`: it didn't forbid `d_drop = OPPOSITE(splitter_dir)`
at splitter-typed sources, so the solver could pick belts that push
items back into the splitter face and break recovery with self-loops.
Bug was masked by the original `(1, 5)` timeout being just barely
short enough; longer timeouts let CP-SAT find these invalid alternates.
Tests now use a 180 s default timeout (with `round_trip_with_timeout`
for per-shape tuning). Suite goes 11 → 12 green.

**Earlier 2026-05-02 update:** Phase 4 increment a.2 shipped in
`9fb5276` — `place_one_to_five()` + `round_trip_1_5`. Pickup point
remains a.3 (roll out to `(1, 7)`, `(1, 9)`, `(1, 10)`).

**Key insight from a.2:** the lane-saturation problem at M.in1 was
real but solvable with a *placement-only* lane-balancer splitter B'
above M. The synth graph stays untouched; recovery sees a 9-splitter
graph and `verify_balancer` accepts it via the standard R7 +
conservation flow. The trick is feeding M.in1 from both north
(back-feed, lanes preserved) and east-side sideload (lane 0) via
B's two outputs — yielding lanes 0.45 / 0.15 at M.in1, both within
cap. See the RFP decision log for the full lane analysis.

## What's shipped

- `f580dfc` — per-arc rate plumbing into the CP-SAT placer (rate-aware
  per-lane cap; `_SYNTH_CTX` populated from the request).
- `7eecf22` — lane-balancer splitter helper (`_add_lane_balancer_south`)
  for placement-only splitters that rebalance lanes after a
  perpendicular turn.
- `a1b810b` — RFP decision-log entry documenting the `(1, 5)` layout
  wall and the three unblock directions considered.
- `1e3d17c` — **underground-belt support in `_route_belts`**. Yellow
  tier (`UG_MAX_REACH = 4`), global (any route may use UG pairs).
  Three entity kinds per (tile, direction): `surf`, `ug_in`, `ug_out`.
  Pair vars couple entries to exits; same-direction tunnel non-overlap
  prevents Factorio's auto-pairing rule from re-pairing tunnels.
  Return signature changed to `dict[tile -> (direction, kind)]`. All
  10 dyadic round-trips green; `(1, 16)`/`(2, 16)` solve in ~60 s now
  (vs ~16 s pre-UG) — model is bigger but optimiser still picks
  surface-only paths for shapes that don't need UG.

## Plan: where this fits

Plan file: `/root/.claude/plans/do-we-have-enough-sharded-brook.md`.
Five increments toward closing issue #136 (20 missing coprime balancer
shapes in 1..=10 × 1..=10):

1. ✅ Per-arc rate plumbing (`f580dfc`)
2. ✅ Lane-balancer splitter helper (`7eecf22`) — used by `(1, 5)`
3. **Increment a (UG-belt direction)** — chosen unblock path:
   - ✅ a.1: UG infrastructure in `_route_belts` (`1e3d17c`)
   - ✅ a.2: `(1, 5)` placer + round-trip test (`9fb5276`)
   - 🟢 a.3: `(1, m)` 3-level shapes generalised
     - ✅ `(1, 6)`, `(1, 7)` shipped via `place_one_to_m_from_synth`
       — synth-driven placer that derives geometry from the
       BalancerGraph + arc throughputs.
     - 🟡 `(1, 5)` still on hand-tuned placer; runs fine on the generalised
       one in isolation but flakes in full-suite runs (CP-SAT finds
       OutputDegree-violating alternates with longer timeouts when other
       tests share the system). Follow-up: add structural constraints
       (e.g., explicit forbid-direction at specific tiles) so the
       generalised placer can subsume `(1, 5)` reliably.
     - ⏳ `(1, 9)`, `(1, 10)` need the generalised placer extended to
       4-level trees (16 splitters with an L3 layer). Two paths:
       (a) hand-tune them like the original `(1, 5)`; (b) extend the
       generalised placer with deeper layout primitives. See
       `docs/rfp-lane-aware-routing.md` decision log for why the
       4-level head-on cascade is structurally harder.
4. ⏳ Coprime coverage for the rest of the 20 missing shapes
5. ⏳ Library regen, browser eyeball, cleanup

## Next session: pick up at a.2

Goal: implement `place_one_to_five()` in `scripts/cp_sat_placer.py`
and add `round_trip_1_5` to `crates/core/tests/cp_sat_round_trip.rs`.

### `(1, 5)` synth structure (verified by running synth at this branch)

```
n_inputs = 1, n_outputs = 5, n_splitters = 8

arcs (idx: src -> dst, throughput):
  0: Input(0)         -> Splitter{0, 0}    1.0
  1: Splitter{0, 0}   -> Splitter{1, 0}    0.8
  2: Splitter{0, 1}   -> Splitter{1, 1}    0.8
  3: Splitter{1, 0}   -> Splitter{2, 0}    0.8
  4: Splitter{1, 1}   -> Splitter{3, 0}    0.8
  5: Splitter{2, 0}   -> Splitter{4, 0}    0.4
  6: Splitter{2, 1}   -> Splitter{5, 0}    0.4
  7: Splitter{3, 0}   -> Splitter{6, 0}    0.4
  8: Splitter{3, 1}   -> Splitter{7, 0}    0.4
  9-13: Splitter{4..6}.{0,1} -> Output(0..4)   0.2 each (5 outputs)
 14: Splitter{6, 1}   -> Splitter{0, 1}    0.2  (feedback)
 15: Splitter{7, 0}   -> Splitter{0, 1}    0.2  (feedback)
 16: Splitter{7, 1}   -> Splitter{0, 1}    0.2  (feedback)
```

So: M = idx 0 (1+0.6 in, splits to 1.0+1.0 = wait, this is the merger
that takes 1.0 input + 0.6 feedback = 1.6 across two ports = 0.8 each).
S1 = idx 1 (root). L1 = idx 2, 3. L2 = idx 4, 5, 6, 7. Three feedback
arcs at rate 0.2 (S6.out1, S7.out0, S7.out1 → M.in1).

### The wall before UG, recap

Rate-0.8 arcs (M→S1, S1→L1) need head-on flow to fit the 0.5 lane cap,
which forces tight-stack placement (S1 directly below M, S2/S3 directly
below S1). That puts S2 and S3 in adjacent columns at the same row.
Their L1→L2 routes (0.4 each) need to land on splitters one row below,
so S2.out1 (right port) wants to go east while S3.out0 (left port)
wants to go west — they cross in the middle of the routing row.

Without UG belts, no surface-only routing works. **With a.1 shipped,
the placer can now use UG belts to skip past the crossing.** One of
the two crossing routes goes underground for 1 tile (k=2 reach), the
other stays on surface.

### Suggested layout for a.2

10×9 grid, all splitters south-facing.

| Row | Tiles                                         |
|-----|-----------------------------------------------|
| 0   | M.in0 input belt at col 4                     |
| 1   | M splitter anchor at (4, 1) span (4,1)-(5,1)  |
| 2   | tight-stack: S1 at (4, 2)                     |
| 3   | tight-stack: S2 at (3, 3) and S3 at (5, 3) — adjacent! Wait, S2 anchor (3,3) spans (3,3)-(4,3); S3 anchor (5,3) spans (5,3)-(6,3). Gap at no tile, since col 4 is S2's right tile and col 5 is S3's left. They're adjacent. |
| 4   | routing row for L1→L2. Drops at (3,4), (4,4), (5,4), (6,4). Crossing problem here. |
| 5   | L2 splitters S4, S5, S6, S7. Place with offset so each receives its proper feed. |
| 6   | output row + feedback channel start            |
| 7-8 | feedback channel south-detour and east-bound   |
| 0   | (wraps around) feedback channel west-bound back to M.in1 |

Concretely:
- S2.out0 drop at (3, 4): goes south into S4 at (3, 5).
- S2.out1 drop at (4, 4): wants to feed S5 (at col ~6, 5). Goes east —
  but S3.out0 drop at (5, 4) wants to feed S4 (at col ~3, 5), going
  west. They cross at row 4 between cols 4-5.
- **UG fix**: route from (5, 4) west goes underground at (4, 4)... no
  wait, (4, 4) is also a route source. Let me think.

Actually the cleanest UG layout for this crossing:
- S2.out1 = (4, 4), going east toward S5. Surface belt east at row 4.
- S3.out0 = (5, 4), going west toward S4. Goes UG: ug_in at (5, 4)
  heading west, ug_out at (3, 4) heading west. Wait, (3, 4) is S2.out0,
  already a surface tile — collision.

Hmm, the layout needs more care. Possible alternative: shift L2
splitters to row 6 (with row 5 as routing row), giving more room for
the L1→L2 crossings to use UG with reach-3 or reach-4. The CP-SAT
solver will pick the cheapest UG configuration that works — we just
need to make the splitter positions feasible and let `_route_belts`
find the routing.

**Suggested approach for a.2**:
1. Pick splitter positions that mimic `place_x_to_eight` style — root,
   level-1 (S2, S3), level-2 (S4-S7) on a wider routing row.
2. Build routes for: input belt (M.in0), feedback channel (3 drops →
   M.in1), each splitter-to-splitter arc, each output drop.
3. Use rate-aware cap: `lane_cap = LANE_CAP_SCALED = 5`, per-route
   rates from `_find_arc_rate(src, dst)`.
4. Feed it through `_route_belts` and let CP-SAT figure out where to
   put UG belts.

If `_route_belts` returns `None` after a 60s solve, the layout
geometry is wrong — adjust splitter positions.

### Concerns for a.2

- **Feedback channel**: 3 feedback arcs at 0.2 each total 0.6 to M.in1.
  M's input port is one tile (col 5, row 0 if M anchor is (4, 1)). All
  3 feedback drops need to merge onto M.in1's lane. With rate-aware
  caps, two lanes carry 0.5 each = 1.0 total cap; 3×0.2 = 0.6 fits if
  spread across both lanes. The lane-balancer splitter helper from
  `7eecf22` may be useful here — but the source-lane forcing infra
  from PR #273 should handle it without an explicit balancer if the
  layout permits.
- **Solve time**: with the bigger UG-aware model, `(1, 5)` may take
  >60 s. The test timeout in `cp_sat_round_trip.rs` is 60 s; bump if
  needed.
- **Topology recovery**: `topology_of_template` (in
  `crates/core/src/bus/balancer_classify.rs`) walks UG pairs by
  searching forward from each `ug_input`. As long as our UG pairs are
  correctly pinned (same direction, nothing else in between), recovery
  should just work.

## Files touched on this branch

- `scripts/cp_sat_placer.py` — main placer; UG infra + per-arc rates +
  lane-balancer helper.
- `crates/core/src/balancer/placement/cp_sat.rs` — Rust adapter; added
  `arc_throughputs` field to request.
- `crates/core/tests/cp_sat_round_trip.rs` — bumped timeout to 60s.
- `docs/rfp-lane-aware-routing.md` — decision log for `(1, 5)` wall
  and unblock directions.

## Files to read first next session

- `scripts/cp_sat_placer.py` — focus on `_route_belts` (UG model),
  `place_x_to_eight` (template for `place_one_to_five`), `_find_arc_rate`.
- This handoff doc.
- `docs/rfp-lane-aware-routing.md` decision log entries for context.

## Verification commands

```bash
# Round-trip suite (tests/cp_sat_round_trip.rs):
FUCKTORIO_RUN_CP_SAT=1 cargo test --manifest-path crates/core/Cargo.toml \
    --test cp_sat_round_trip -- --test-threads=1

# Unit tests:
cargo test --manifest-path crates/core/Cargo.toml --lib cp_sat

# Clippy:
cargo clippy --manifest-path crates/core/Cargo.toml --all-targets
```

Pre-PR: all 10 dyadic round-trips green, no new clippy warnings.
