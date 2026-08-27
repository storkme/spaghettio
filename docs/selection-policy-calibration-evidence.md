# Selection-policy calibration evidence

Source bank: `/tmp/calibration-matrix-2026-08-27-732`. Validator probe: `/tmp/calibration-matrix-2026-08-27-732/calibration-issue-breakdown.json`. Corpus-definition fingerprint (`corpus_sha256` — the ordered fixture declarations, deliberately independent of any generated layout): `88c59075b90b4b22fdc0a4711b1b43c7aa47ab676488b4d93e19754fb5004617` — must match the same field in the committed `crates/core/data/calibration-bank/matrix.json` for these rows to describe the shipped engine's corpus; per-row geometry binds via each row's `blueprint_sha256`/`manifest_sha256`, which the CI probe checks.

Status preserves campaign state: `awaiting-measurement` has no `report.json`; `non-converged` and `kit-error` retain their measured values but are excluded from the clean-row findings; `excluded` covers every probe-side determinism refusal — the probe's `exclusion_reason` names which: `blueprint-sha256-mismatch`, `manifest-sha256-mismatch`, `validator-totals-mismatch`, or `build-failed`.

## Table

| label | status | converged | kit error class | delivered % | produced % | exclusion reason | belt-dead-end E | belt-detour E | input-rate-delivery E | row-input-belt-margin E | belt-dead-end W | belt-detour W | input-rate-delivery W | row-input-belt-margin W |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| tier1_iron_gear_wheel | measured | true |  | 101.333 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier1_iron_gear_wheel_from_ore | measured | true |  | 98.667 | 100.333 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier1_iron_gear_wheel_20s | measured | true |  | 100.000 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier2_electronic_circuit | measured | true |  | 100.645 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier2_electronic_circuit_from_ore | measured | true |  | 92.121 | 93.333 |  | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 |
| tier2_electronic_circuit_20s_from_ore | measured | true |  | 102.500 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| tier3_plastic_bar | measured | true |  | 98.667 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier3_plastic_bar_from_crude | measured | true |  | 98.667 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier3_sulfuric_acid | measured | true |  |  | 375.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier3_heavy_oil_cracking | measured | true |  |  | 300.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier3_advanced_oil_processing_multi_machine | measured | true |  |  | 150.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier3_advanced_oil_processing_forced_multi_machine_pipe_isolation | non-converged | false |  |  | 158.333 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier4_advanced_circuit_from_plates | measured | true |  | 101.672 | 100.334 |  | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| tier4_advanced_circuit_partitioned | kit-error | true | research-productivity parity | 98.997 | 100.669 |  | 0 | 0 | 0 | 0 | 0 | 2 | 1 | 0 |
| tier4_advanced_circuit_from_ore_am2 | measured | true |  | 100.645 | 98.065 |  | 0 | 0 | 0 | 0 | 0 | 0 | 6 | 0 |
| tier5_processing_unit_from_ore_am3 | measured | true |  | 86.550 | 87.719 |  | 0 | 0 | 0 | 0 | 0 | 1 | 10 | 0 |
| tier_kovarex_self_loop | non-converged | false |  | 0.000 | 0.000 |  | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| tier_uranium_processing_surplus_export | non-converged | false |  | 0.000 | 95.709 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier_uranium_processing_voider | non-converged | false |  | 0.000 | 0.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier_pentapod_egg_self_loop | non-converged | false |  | 0.000 | 0.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier_fish_breeding_self_loop | non-converged | false |  | 0.000 | 0.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tier_bacteria_self_loop_regression | non-converged | false |  | 0.000 | 0.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| stress_electronic_circuit_30s_from_ore | measured | true |  | 92.121 | 90.909 |  | 0 | 0 | 0 | 0 | 0 | 0 | 8 | 5 |
| stress_advanced_circuit_45s_from_plates | measured | true |  | 99.556 | 100.444 |  | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| stress_advanced_circuit_partitioned_5s_from_plates_pooled | non-converged | false |  | 100.339 | 101.695 |  | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| stress_advanced_circuit_partitioned_5s_from_plates_partitioned | measured | true |  | 98.667 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 2 | 1 | 0 |
| stress_advanced_circuit_partitioned_4s_from_plates_pooled | measured | true |  | 101.333 | 100.333 |  | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| stress_advanced_circuit_partitioned_4s_from_plates_partitioned | measured | true |  | 100.000 | 100.329 |  | 0 | 0 | 0 | 0 | 0 | 2 | 1 | 0 |
| stress_electronic_circuit_30s_decomposed_pooled | measured | true |  | 92.121 | 90.909 |  | 0 | 0 | 0 | 0 | 0 | 0 | 8 | 5 |
| stress_electronic_circuit_30s_decomposed_partitioned | measured | true |  | 99.394 | 99.394 |  | 0 | 0 | 0 | 0 | 0 | 0 | 6 | 5 |
| stress_electronic_circuit_60s_red_from_ore | measured | true |  | 90.667 | 90.667 |  | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 5 |
| stress_electronic_circuit_22s_from_ore | measured | true |  | 99.394 | 95.455 |  | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| stress_electronic_circuit_23s_from_ore | measured | true |  | 99.379 | 100.000 |  | 0 | 0 | 0 | 0 | 0 | 0 | 6 | 0 |
| stress_electronic_circuit_35s_from_ore | measured | true |  | 96.000 | 94.286 |  | 0 | 0 | 0 | 0 | 0 | 0 | 10 | 0 |
| stress_electronic_circuit_40s_from_ore | measured | true |  | 92.000 | 93.000 |  | 1 | 0 | 0 | 0 | 0 | 0 | 24 | 4 |

## Findings

Clean-row comparison uses a 95% threshold. A converged, kit-clean row is a shortfall if either available target rate is below it; it is at plan only if both target rates are available and at or above it. Rows with missing target metrics are not classified — note the structural asymmetry this creates: fluid targets carry no delivered rate (RFC-050 fluid boundaries are uncalibrated), so fluid rows can never classify as at-plan and are barred from the false-alarm section by construction; overproduction rows (the two oil fixtures measure ~150% produced) classify as at-plan only when both metrics exist and are otherwise unremarked here.

### Validator categories co-occurring with clean-row shortfall

- `belt-dead-end`: `stress_electronic_circuit_40s_from_ore`
- `belt-detour`: `tier5_processing_unit_from_ore_am3`
- `input-rate-delivery`: `tier2_electronic_circuit_from_ore`, `tier5_processing_unit_from_ore_am3`, `stress_electronic_circuit_30s_from_ore`, `stress_electronic_circuit_30s_decomposed_pooled`, `stress_electronic_circuit_60s_red_from_ore`, `stress_electronic_circuit_35s_from_ore`, `stress_electronic_circuit_40s_from_ore`
- `row-input-belt-margin`: `tier2_electronic_circuit_from_ore`, `stress_electronic_circuit_30s_from_ore`, `stress_electronic_circuit_30s_decomposed_pooled`, `stress_electronic_circuit_60s_red_from_ore`, `stress_electronic_circuit_40s_from_ore`

### Categories never seen on a clean measured row
(fires only on rows outside the converged, kit-clean set — non-converged, kit-errored, awaiting, or excluded)

- None.

### Categories firing on clean rows at plan (false-alarm candidates)

- `belt-detour`: `tier2_electronic_circuit_20s_from_ore`, `tier4_advanced_circuit_from_plates`, `stress_advanced_circuit_partitioned_5s_from_plates_partitioned`, `stress_advanced_circuit_partitioned_4s_from_plates_pooled`, `stress_advanced_circuit_partitioned_4s_from_plates_partitioned`
- `input-rate-delivery`: `tier4_advanced_circuit_from_ore_am2`, `stress_advanced_circuit_partitioned_5s_from_plates_partitioned`, `stress_advanced_circuit_partitioned_4s_from_plates_partitioned`, `stress_electronic_circuit_30s_decomposed_partitioned`, `stress_electronic_circuit_22s_from_ore`, `stress_electronic_circuit_23s_from_ore`
- `row-input-belt-margin`: `stress_electronic_circuit_30s_decomposed_partitioned`

These are candidates, not adjudicated false positives: this table establishes co-occurrence, not causal attribution.

## Coverage

kit-error: 1, measured: 26, non-converged: 8.
