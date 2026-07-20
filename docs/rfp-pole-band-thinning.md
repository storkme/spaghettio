# RFP: Quality-aware pole-band thinning (#310)

Status: ACCEPTED-PENDING-IMPLEMENTATION v2 (2026-07-20) — Phase 0
census done; adversarial review round 1 complete (one blocker + two
must-fixes, all resolved in v2 — decision log).

## Summary

At build quality ≥ Uncommon, a single medium-pole band can cover BOTH
of a machine row's inserter rows — the reason `place_poles` emits two
bands per row is purely the Normal-tier radius (floor 3, which cannot
span the `mh+1` distance from one band to the opposite inserter row).
This RFP gates the south band off per row when the quality radius
affords it **with at least one tile of depth slack** (`budget ≥ 1` —
see the gate), one unified placement pass per qualifying row with
per-target fallback, and the existing mop-up + reactive-substation
machinery unchanged as backstops. Normal-quality layouts are
structurally untouched (the gate can never fire at radius 3), and the
corpus's only live substation user is already dormant at every
qualifying tier (kovarex 3b fallback fires at Normal only — pinned by
`quality_differential_kovarex_self_loop_normal_vs_legendary`, #315). Follow-up to `docs/rfp-build-quality.md`; tracked as
[#310](https://github.com/storkme/spaghettio/issues/310); sibling
thread [#315](https://github.com/storkme/spaghettio/issues/315)
(power-arc quality verification) proceeds in parallel and this RFP must
not touch its surface (see kill criterion 4).

## Motivation (Phase 0 census, 2026-07-20)

`place_poles`' along-band cadence already scales with quality (it
derives from `supply_area_distance`), so pole counts drop steeply with
tier — but the band COUNT per row does not. EC@45/s from ore on
express (the build-quality reference config), measured via
`debug_quality_layout`:

| Config | Tier | Medium poles |
|---|---|---|
| EC@45/s from ore, express (solid rows only) | Normal | 239 |
| | Uncommon | 129 |
| | Legendary | 60 |
| PU@2/s from ore, red (fluid rows; `tier5` fixture config, #315 sweep numbers) | Normal | 343 |
| | Rare | 156 |
| | Legendary | 98 |

(The PU config carries non-power junction/belt errors at Rare+ — the
documented machine-count-collapse gap, #135/#136/#312 — but its
power-category results are clean at every tier, so its pole census is
valid for this RFP's purposes.)

Every remaining pole row is one of a north/south pair whose partner is
redundant at quality: a legendary pole's ±8 vertical reach covers a
3-tall machine row's opposite inserter row (distance 4) with 4 tiles of
depth to spare. Expected effect of thinning: roughly half the
remaining poles at qualifying tiers (legendary ~60 → ~35 on the census
config), zero effect at Normal. The user-visible symptom today is the
browser eyeball: legendary layouts show doubled bands of nearly-idle
poles (the exact trigger condition the build-quality RFP recorded for
pulling this work forward).

## Design

### The gate (per machine row, per tier)

`pole_range = floor(supply_area_distance("medium-electric-pole", q))`
(3/4/5/6/8 across the tiers). The north band's candidate at
`cy = top_y - 1` sits `mh + 1` tiles from the south inserter row
(`top_y + mh`). A pole placed at depth `d` above the candidate
(`place_poles` searches outward, away from the machine) covers the
south inserter row iff `mh + 1 + d ≤ pole_range`. Define

```
single_band_depth_budget(mh, q) = pole_range(q) − (mh + 1)   // may be negative
```

The gate requires **budget ≥ 1**, not ≥ 0 (v2, from adversarial
review findings 4+5):

- Budget ≥ 1 → **single-band mode** for this row: one unified pass
  (below), searches truncated to depths `0..=budget` (deeper
  placements would break opposite-row coverage — the truncation is
  what makes the mode sound, not hoped-for).
- Budget ≤ 0 → today's two-band behavior, unchanged.

Why ≥ 1: at budget 0 the search is d=0 only, and `fluid_only_row` /
`fluid_dual_input_row` (msz=3) **fully pack the north candidate row**
— single-band mode would deterministically fail into fallback for
every machine of every fluid-touching 3×3 row at its first ≥0 tier,
yielding zero thinning exactly where the v1 table claimed it started.
Requiring one tile of depth slack gives packed rows their d=1 escape
and (see wire section) lands activation on tiers whose wire reach
clears every row pitch with margin.

Per-tier qualification (v2): 3-tall machines thin at **Rare+**
(budget 1/2/4 at Rare/Epic/Legendary); 4-tall (EM plant) at **Epic+**
(1/3); 5-tall (refinery/foundry/cryo) at **Legendary** (2). Uncommon
never thins. At Normal the budget is negative for every machine
height — the gate structurally cannot fire (kill criterion 1). The
floor-based comparison is EXACT, not conservative: the validator's
continuous bound is `3.5 + level` (fixed `.5` fraction) against an
always-integer center distance, so `n ≤ 3.5+level ⟺ n ≤ floor(...)`
(review finding 1).

North band is the keeper (not south): fluid-row templates reserve
their pole-gap tiles on the north band (`templates.rs` asserts
`pole_candidate_ys`' north row), and the machine center at distance
`mh/2 + 0.5 + d` is always covered whenever the opposite inserter row
is (`mh/2 + 0.5 < mh + 1` for every mh).

### Unified single pass + per-target fallback (v2, review finding 3)

In single-band mode the row runs ONE left-to-right pass over machine
targets (replacing the two-band loop for that row only). Key soundness
fact: within the truncated depth window, north and south poles are
coverage-INTERCHANGEABLE — a south pole at depth d covers the north
inserter row by the same `mh + 1 + d ≤ pole_range` bound — so
skip-ahead crediting is band-agnostic and pole-count bookkeeping stays
coherent (v1's inline retry would have leaked un-credited fallback
poles past the north loop's cursor, review finding 3). Per target:

1. Try north band, depths `0..=budget` (candidate window + fallback
   tiles as today).
2. Else try south band, depths `0..=budget` — interchangeable, so the
   unified skip-ahead applies to whichever placed.
3. Else mark the target DEGENERATE. After the pass, degenerate targets
   (if any) run today's two-band placement restricted to their
   x-windows — strictly-no-worse-than-today per target, with each
   band's own crediting.

Below all of that, the uncovered-inserter mop-up and the reactive
substation chain (3a-ii/3b) are untouched backstops — thinning changes
where bands try first, never what happens when coverage fails.

### Wire connectivity (v2 — per-tier, per-shape arithmetic)

One band per row raises inter-band vertical distance to the row pitch:
north-to-north spacing ≈ row_height + 2 (inter-recipe gap). Row
heights (msz=3): single_input_row 7→spacing 9, dual 8→10, triple
9→11, quad/fluid_dual 10→12. Against wire reach at the v2 qualifying
tiers: **Rare 13 ≥ 12** (every msz=3 shape clears, margin ≥1), Epic
15, Legendary 19. v1's Uncommon activation would have sat exactly at
(triple, 11=11) and over (quad, 12>11) the wire boundary — the second
reason the gate is budget ≥ 1 (review finding 5). Balancer/retry gaps
still stretch pitch beyond reach occasionally; the quality-aware
`repair_pole_connectivity` bridges those, same as today, and kill 2's
per-tier NET pole assertion catches any bridge-bloat wash.

### What this does NOT touch

- `compute_substation_bands` / top-edge bands / convergence signal —
  the 3a-ii/3b reactive contract stays byte-identical (kill 4), and
  the #315 sweep owns that surface in parallel.
- Normal-quality placement — structurally unreachable gate.
- Cross-row band sharing (one band powering two adjacent rows' facing
  inserter rows) — Phase 2, only if Phase 1's eyeball still shows
  redundancy; needs pitch analysis across row layouts and interacts
  with variable gaps.

## Kill criteria

1. **Normal identity (structural + empirical).** The gate cannot fire
   at Normal (negative budget for all machine heights — unit-tested per
   (mh, tier) pair), and the full suite + `SPAGHETTIO_STRESS_GOLDEN=check`
   stay clean. Any Normal-quality diff ⇒ the gate leaked — halt.
2. **The reduction must pay — at every qualifying tier, both configs
   (v2).** NET medium-pole count (bands + repair bridges) must strictly
   decrease vs unthinned at EVERY qualifying tier on BOTH census
   configs (EC solid, PU fluid), with no new warning categories; and on
   the EC config at Legendary the reduction must be ≥ 20% (predicted
   60 → ~35, gate ≤ 48). A wash-or-worse at any qualifying tier (e.g.
   repair-bridge bloat eating the band savings) fails this criterion —
   abandon or re-scope to the tiers that pay, and record which.
3. **No coverage regressions at any tier.** Per-tier differential runs
   of the census config plus the #315 sweep fixtures: any power
   coverage/connectivity warning present with thinning but absent
   without it means the depth-budget math or fallback is wrong — halt
   and fix root-cause, never widen the fallback to paper over it.
4. **Scope fence.** If a correct implementation requires modifying the
   reactive substation trigger/band code (not merely reading shared
   helpers), stop — that contract belongs to 3a-ii/3b and the #315
   sweep; redesign the gate instead.

## Verification plan

- Unit: `single_band_depth_budget` table per (mh ∈ {3,4,5}, tier) —
  including every Normal entry negative; a place_poles-level test in
  the style of `repair_pole_connectivity_uses_quality_wire_reach`
  asserting a two-row legendary field powers both inserter rows from
  one band and that the same field at Normal still emits two bands.
- Differential: census-config pole counts per tier asserted (kill 2
  numbers) with validator cleanliness; reuse the
  `quality_differential_ec_normal_vs_legendary` fixture, extending its
  assertions with pole-count expectations.
- Corpus: full suite; STRESSGOLD check; the #315 fixtures at Legendary
  after both threads land (sequencing note below).
- Browser eyeball (user validates): the legendary census URL —
  `#/l/ecl/45/am3/ior,coo/etb?q=l` — should visibly show single pole
  rows between machine rows.

## Sequencing with #315

#315 (verification sweep) runs first/parallel on the UNthinned
geometry, establishing the per-tier baseline the kill-3 differentials
compare against. This RFP's changes land after the sweep's baseline is
recorded, so any regression is attributable. Both threads read the
same shared helpers; only this one writes `place_poles`.

## Decision log

- *2026-07-20 — Phase 0 census run (numbers above); v1 draft written.*
- *2026-07-20 — **Phase 1 landed** (gate + unified pass in
  `place_poles`, extracted `place_band_line` /
  `place_unified_band_line` / `band_y_lists` /
  `single_band_depth_budget` helpers). Kill-2/3 census, thinned vs
  unthinned NET medium poles:
  EC@45/s — Normal 239→239 and Uncommon 129→129 (bit-identical,
  gate off), **Legendary 60→30 (50%, beats the ≤48 gate and the ~35
  prediction)**, validation issues identical per tier;
  PU@2/s (tier5 fixture config) — Normal 343→343 identical with 0
  issues, Rare 156→89, Epic 149→76, Legendary 98→53, failure
  categories at qualifying tiers unchanged (the pre-existing
  non-power Finding-B classes from #315 — belt-dead-end at Rare,
  unresolved-junction at Legendary — zero power issues introduced
  anywhere). Kovarex differential clean at both tiers, substation
  behavior unchanged. Unit gates: `single_band_gate_table_per_mh_and_tier`
  (structural kill-1: Normal/Uncommon can never thin),
  `single_band_mode_halves_row_bands_and_stays_covered` (validator's
  own continuous check on the thinned field), kill-2 pole pin (== 30)
  in `quality_ec_45s_express_legendary_from_ore`.*
- *2026-07-20 — adversarial review round 1: depth-budget math verified
  EXACT (floor vs continuous `.5`-fraction bound — no rounding slop);
  Chebyshev coverage confirmed from `check_power_coverage`; scope
  fence verified real (substation pass independent of the band loop).
  One blocker: fluid msz=3 templates fully pack the north candidate
  row, so budget-0 tiers get zero thinning there (census was
  solid-only and blind to it). Two must-fixes: Uncommon wire margins
  (triple 11=11, quad 12>11) and v1's underspecified fallback/skip-
  ahead interaction. **v2 resolves all three with one design change —
  gate raised to budget ≥ 1** (activation Rare+/Epic+/Legendary by
  machine height; Uncommon never thins) — plus the unified
  interchangeable-coverage pass and a per-tier/per-config NET-pole
  kill criterion. Fluid census added from the #315 sweep's valid PU
  numbers (my own PU probe had mis-set inputs — Gleba chain — and was
  discarded). Kovarex substation dormancy at Rare+ (pinned in #315's
  differential) removes the last thinning×substation interaction at
  qualifying tiers.*
