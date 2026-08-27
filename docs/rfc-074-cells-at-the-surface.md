# RFC-074: Cells at the surface — RFC-072 Phase 3, scoped

Registry: [`rfcs.md`](rfcs.md). Status: **Design → Active 2026-08-27.**
This is RFC-072's uncommitted Phase 3 ("heterogeneous composition and
the library; the web surface") scoped to what its own evidence supports:
the surface is a measured defect and ships; the fluid/mega grid path gets
its receipt or its refusal; heterogeneous composition is adjudicated by
measurement, not built; the library stays parked where K67-3 left it.

## Summary

RFC-072 Phases 0–2 made rates past the lane wall ship: the cell-chain
grid composes ec@240 as 2×12 strips and sims at plan where the native
bus panics. Nothing above the engine knows. The browser already reaches
the grid — `cell_composition: Candidate` is the engine default, the wasm
option parser maps an absent value to it, and `web/` has no rate ceiling
— so a user who types electronic-circuit at 240/s gets the composed
layout and then **cannot export it**: the composition's registry note
(a *receipt* — "SIM-VERIFIED at plan … PASS produced 240.00/s") is
pushed onto `LayoutResult.warnings`, and `web/src/ui/sidebar.ts` hides
the blueprint section whenever that list is non-empty. The renderer
draws no strip or cell outline, and the winning candidate's name reaches
the trace but no UI. This RFC (1) gives composition a typed receipt on
`LayoutResult` and a surface in the web — export restored, the receipt
shown as what it is, strips outlined; (2) closes RFC-072 residual (f)
by receipting or loudly refusing the mega/fluid grid path, and receipts
the from-plates ec@240 grid the browser actually builds; (3) adjudicates
"chains of unlike cell groups" with one number — how much a uniform K
over-provisions the chain-eligible corpus — and either closes it or
hands it to its own RFC. Kill criteria keep every part additive: no
shipped geometry moves.

## Motivation

Reproducible today (`examples/web_probe_ec240.rs`, the browser's entry
point `build_bus_layout(LayoutOptions::default())`, 2026-08-27):

| request | winner | entities | `warnings[0]` |
|---|---|---|---|
| ec@240 from ore | `cell-composed` (2×12, 4 pole bridges) | 17,148 | `cell-composed: geometry SIM-VERIFIED at plan (… PASS produced 240.00/s …)` |
| ec@240 from plates | `cell-composed` (2×12) | 4,460 | `cell-composed: geometry NOT sim-verified (hash 5c83b4199631358a)` |
| ec@150 from ore | `cell-composed` (K=12 strip) | 10,392 | `SIM-VERIFIED … PASS produced 150.00/s` |

In every row `layout.warnings.length > 0`, so
`blueprintSection.style.display = "none"` (`sidebar.ts:1164`) — the
receipt that says the layout is good is what stops the user exporting
it. The from-plates row is the request a browser user makes first
(plates are the ordinary external inputs), and it has no receipt at all.

The other two items are RFC-072's own residual list: (f) "the mega/fluid
grid path is untested by construction — no fluid-touching chain reaches
K > 12 in the corpus", and Phase 3's "chains of unlike cell groups",
for which no failing case exists today — the recon
(`~/.local/state/codex-task/p3-recon/report.md`) found the composer
bakes K-uniformity into quantization, cell cloning, corridor scope,
bypass rows, pole placement, and the registry key, and that no
heterogeneous composer exists in the tree. Building one on no measured
need is the rework shape the RFC template exists to stop.

## Design

### Unit 1 — the receipt (engine, additive)

`LayoutResult` gains

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub composition: Option<CompositionReceipt>,

pub struct CompositionReceipt {
    pub kind: String,            // "cell-chain" | "cell-grid" | "mega-cell"
    pub copies_per_strip: Vec<i32>,
    pub strips: Vec<StripRect>,  // { x, y, width, height, copies } in layout frame
    pub verification: String,    // the registry note, verbatim
    pub verified: bool,          // registry row present with a non-FAIL verdict
}
```

filled by the chain/grid composer (strip rects are known at
`append_strip_translated` time) and by the `CellComposedCandidate`
where `verification_note` is computed today. `warnings` no longer
carries the note. The note's text and the registry's mismatch
precedence are untouched (the `mismatch_note_is_fail_dominant…` tests
keep passing on `composition.verification`). Additive: entities,
blueprint bytes, registry hashes, bank fingerprints all unchanged.

### Unit 2 — the surface (web)

- `sidebar.ts`: the blueprint section hides on `warnings` only; a
  composed layout shows a composition badge — kind, `strips × copies`,
  the verdict phrase, and the full note on hover.
- Renderer: strip outlines from `composition.strips` (a layer next to
  the trace overlay's row boundaries; off when `composition` is null).
- The winner's name (`SelectionDecided.winner`, already in the trace
  the streaming path retains) shown in the validation panel header.

### Unit 3 — receipts (sim)

- **ec@240 from plates** (hash `5c83b419…`): export, sim, registry row.
  Whatever it measures is the receipt; selection is unbiased by design
  (RFC-051 kill 3) so a short measurement does not change what ships,
  it changes what the badge says.
- **The mega/fluid grid.** The smallest fluid-touching request past
  K_MAX is advanced-circuit from ore + crude at ≈48/s (cable ≈10 per
  AC → 480/s → K=13). Compose it through `compose_grid_with_capacity`:
  either it carries zero validator errors, its per-strip boundary
  records (fluid heads included, #732's direction fix in) let the
  harness rig it, and it sims with the two strips within 2 points of
  each other — then it is receipted and (f) closes — or
  `chain_eligible_at` refuses fluid-touching chains past K_MAX **by
  name** and (f) closes as "refused, not shipped". Both are closes;
  the RFC does not commit to which.

### Unit 4 — heterogeneous adjudication (measurement only)

An ignored probe over every chain-eligible corpus solve: for the K the
quantizer picks, the over-provisioning ratio
`Σ_spec ceil(count/K)·K / Σ_spec count` (machines placed over machines
needed) and the worst single spec. The decision-log entry records the
distribution. K74-3 decides. No composer is written under this RFC.

### Out of scope, deliberately

The library ("celldb-backed reuse with receipts as library rows in the
calibration bank"). RFC-067 killed its consumer under K67-3 and parked
template candidates; #619/#629 hold the state. The recon's gap list
(stable fragment + port identity, receipt fields on fragments, a
composition record, a regeneration gate) is a design of its own, and
nothing in Units 1–4 needs it. It reopens only by a decision-log
amendment on RFC-067, with a measured consumer.

## Kill criteria

- **K74-1 (additive or nothing).** If Unit 1 needs any change to
  entities, blueprint bytes, or selection — anything beyond an
  additive `#[serde(default)]` field and the note's channel — stop; the
  registry gate `cell_registry_hashes_current`, the calibration
  fingerprint probe, and the copy-count pins must be byte-for-byte
  green throughout.
- **K74-2 (the fluid grid closes, either way).** If the AC≈48 grid
  neither receipts (0 errors, strips within 2 points, sim ≥ 97.9%
  produced — the family bar) nor can be refused by name at the
  eligibility gate without touching `mega.rs` internals, Unit 3's
  fluid half stops and (f) stays open with the reason recorded.
- **K74-3 (heterogeneous pays or closes).** If the corpus median
  over-provisioning under uniform K is ≤ 5% and no fixture exceeds
  15%, "chains of unlike cell groups" closes by measurement (with the
  library). Above that, it becomes its own RFC with these numbers as
  its motivation — still not built here.
- **K74-4 (runtime).** End-to-end runtime > 2× on the existing corpus
  drops the offending unit even if the surface improves (K72-5's
  rule).
- **K74-5 (the surface is honest).** The badge must show FAIL/WARN
  verdicts and "NOT sim-verified" as prominently as PASS; if the
  from-plates ec@240 receipt comes back short and the badge would read
  as an endorsement, the badge is wrong, not the receipt.

## Verification plan

Per the CLAUDE.md layout-engine protocol. Unit 1: `cargo test` green;
registry gate + fingerprint probe unchanged (K74-1); the receipt pinned
on ec@240 (kind `cell-grid`, `copies_per_strip [12,12]`, two strip rects
separated by 32, `verified == true`) and on an unregistered hash
(`verified == false`). Unit 2: `tsc` + vitest; the six-site rule for
sidebar controls; the user eyeballs (per the standing UI rule). Unit 3:
sim receipts in `cell-sim-registry.json` with the gate config + probe
entry, as #733 did. Unit 4: the probe's table in the decision log.

## Phasing

- **Unit 1** — engine receipt (one PR, ~200 lines).
- **Unit 2** — web surface (one PR).
- **Unit 3** — receipts (registry rows; the fluid-grid test either
  way).
- **Unit 4** — probe + adjudication (probe + decision-log entry).
- **Close-out** — RFC-072 Phase 3 marked COMPLETE-as-scoped with the
  pointer here; `status.md` row.

## Decision log

- *2026-08-27 — RFC opened on the browser-entry-point probe.* Scoped
  from the Phase-3 recon: the surface is a defect (the receipt hides
  the export), the fluid grid is a receipt-or-refuse, heterogeneity is
  a number, the library is parked under K67-3. RFC-073's Phase 0 census
  (same day) is the precedent for measuring before building.
- *2026-08-27 — Unit 1 adjudication: the note does NOT leave
  `warnings`.* The design said "`warnings` no longer carries the note";
  the code says otherwise, deliberately. `warnings.len()` is a
  selection input twice over — `IssueCounts::layout_warnings` is a
  never-worse floor channel and half of the error-free tier's ordering
  key (`selection_policy.rs`), and RFC-071 B3's `unverified_geometry`
  reads the never-verified substring out of the same list — so moving
  the note would change what ships on any fixture where a cell
  candidate and a native one sit within one warning of each other.
  That is exactly what K74-1 forbids. Unit 1 therefore ADDS the typed
  receipt (`CompositionReceipt { kind, copies_per_strip, strips,
  verification, verified }`, filled by the chain/grid composers and
  completed by `CellComposedCandidate` from
  `registry::verification_status`) and leaves the note in place;
  `verification` is the note verbatim, so the web tells the receipt
  from a warning by string equality. The one-warning penalty every cell
  candidate carries in selection is pre-existing and recorded here as
  a policy question for RFC-071's owner, not silently removed. Zero
  geometry change: the receipt is derived after composition; the
  registry gate, copy-count pins and the grid tests hold unchanged.
- *2026-08-27 — Unit 2 landed with Unit 1 (one PR).* Sidebar: the
  blueprint section hides on `realWarnings` (warnings minus the
  receipt) — the ec@240 grid exports again; a composition badge shows
  `kind · strips × copies · sim-verified | not sim-verified | sim
  FAILED` (colour by the typed flag and the note's wording, full note
  on hover — K74-5); the validation panel drops the receipt row.
  Renderer: `compositionOverlay.ts` outlines each strip and labels its
  copy count, always on when a receipt exists. The "winner name" item
  is satisfied by the badge (native layouts have no receipt, so no
  badge) rather than a separate `SelectionDecided` reader — fewer
  sites, same information. User eyeball pending per the standing UI
  rule.
- *2026-08-27 — Unit 4 adjudicated: K74-3 closes heterogeneous
  composition by measurement.* `probe_uniform_k_overprovisioning`
  (`tests/cell_composition.rs`, release) over every chain-eligible
  fixture in the registry plus the ladder/mega corpus — ratio =
  machines placed under uniform K ÷ machines the solve needs:

  | fixture | K | specs | needed → placed | ratio | worst spec |
  |---|---|---|---|---|---|
  | chain-ac1 | 1 | 3 | 7.60 → 8 | 1.053 | EC 0.80 → 1 (1.25) |
  | chain-ec15 / ec15g2 | 2 | 2 | 15 → 16 | 1.067 | cable 9 → 10 (1.11) |
  | chain-ec30 | 3 | 2 | 30 → 30 | 1.000 | — |
  | chain-ec75 / ec150 | 6 / 12 | 4 | 375 → 378 / 750 → 756 | 1.008 | cable (1.067) |
  | chain-ec240 (ore) / ec240 (plates) | 24 | 4 / 2 | 1200 → 1200 / 240 → 240 | 1.000 | — |
  | ec600-ore | 48 | 4 | 3000 → 3024 | 1.008 | cable 360 → 384 (1.067) |
  | chain-gear20, gear15, ec5 | 1 | 1–2 | exact | 1.000 | — |
  | chain-mil5ore / mil5plates | 2 / 1 | 9 / 5 | 146 → 146 / 46 → 46 | 1.000 | — |
  | ac2 / ac4-ore | 1 | 3 / 7 | 15.2 → 16 / 88.1 → 90 | 1.053 / 1.022 | EC (1.25) |
  | mega-chain-ac2raw | 1 | 7 | 44.0 → 46 | 1.044 | EC 1.6 → 2 (1.25) |
  | mega-chain-chem5raw / csp5-ore | 2 | 10 / 13 | 180.6 → 184 / 376.6 → 380 | 1.019 / 1.009 | sulfur 1.25 → 2 (1.6) |
  | mega-chain-usp2raw | 3 | 22 | 464.8 → 495 | 1.065 | gear 0.27 → 3 (**11.25**) |
  | mega-chain-pu4raw | 8 | 10 | 613.6 → 640 | 1.043 | sulfuric-acid 0.40 → 8 (**20.0**) |

  Chain-wide, uniform K over-builds by 0–6.7% (median 0.8%); no
  fixture reaches K74-3's 15%, and the median is under its 5%. The
  per-spec extremes are real — a 0.27-machine gear spec in USP@2 ships
  as 3 machines, a 0.40 sulfuric-acid spec in PU@4 as 8 — but they are
  the chain's cheapest machines and cost 30 and 26 machines out of 495
  and 640. A per-group K would buy back ≤ 6.7% of machines on the
  worst fixture in exchange for a composer that gives up cell cloning,
  the copy-scoped corridor rule, per-copy bypass rows, uniform pole
  bands and the registry key (recon §A). Not worth it on this
  evidence: **"chains of unlike cell groups" closes**, and with it the
  library reuse it would have needed (still parked under K67-3; #619
  / #629 hold the state). Reopens only with a chain whose uniform-K
  ratio exceeds 15% — the probe is the instrument, and it stays.
- *2026-08-27 — Unit 3: both receipts PASS; RFC-072 residual (f) closes
  as RECEIPTED, not refused.* (1) **ec@240 from plates** (hash
  `5c83b419…`, the browser's own request): 2×12 grid of the 4-machine
  10/s cell, 4,460 entities — sim PASS, produced **240.00/240.00
  (+0.0%)**, delivered +4.0%, all 240 machines working, converged, kit
  clean. The badge now reads sim-verified for the layout a user gets
  first. (2) **The mega/fluid grid** — advanced-circuit from ore +
  crude at 56/s, the smallest fluid-touching request past K_MAX: K=14
  → a 2×7 grid, each strip carrying its own refinery+chem mega block
  and its own fluid feed heads (translated with the strip; heads face
  into the layout, so #732's class never arose — the mega block's
  heads are pipe-to-ground). Composed with **0 validator errors, 0
  warnings**, 26,454 entities; sim PASS, produced **57.40/56.00
  (+2.5%)**, converged, kit clean, 0 fluid errors, 1,246 machines
  working, 14 ingredient-short. K74-2 read honestly: the report is
  aggregate (no per-strip census in `report.json`, unlike the
  K72-3-era probe), so "strips within 2 points" is not directly
  measured; the 14 short machines are exactly one per copy across
  both strips, which is the symmetric signature (an asymmetric
  starvation reads as a per-strip multiple), and produced is above
  the family bar with margin. Recorded as such — (f) closes, and a
  per-strip census in the harness report is noted as the instrument
  that would have made this a measurement rather than a signature.
  Both rows are in `cell-sim-registry.json` with gate configs and
  probe entries; `grid_composes_ac56_from_ore_with_fluid_heads_on_every_strip`
  pins the composition contract.
