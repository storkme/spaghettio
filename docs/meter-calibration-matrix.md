# Meter calibration matrix

Status: active measurement workflow. The matrix is the current 35-fixture
generator regression corpus, exported once and then measured by headless
Factorio. It is deliberately broader than the historical Job-2 and post-lift
banks, which remain useful evidence but are incomplete snapshots of older
generator configurations.

## What it establishes

Every row starts from the same source used by the e2e belt-detour differential:
recipe, rate, machine tier, raw inputs, belt tier, exclusions, and the one
declared layout variant. The e2e drivers build through the suite's own
`run_e2e*` helpers and the exporter through `calibration_matrix::build`; the
two option builders are pinned to each other on every declared variant by
`calibration_matrix_options_match_harness_options` in `e2e.rs`, so an axis
added to one and not the other fails a free test rather than producing a bank
the suite never ran (the RFC-070 W2c fossil class).
The exporter writes the exact blueprint and manifest that Factorio runs; the
meter reads those same files. A row is therefore a meter-vs-game comparison of
one concrete generated factory, not a comparison between separately
reconstructed configurations.

This is a representative current-production corpus, not an exhaustive Cartesian
product of every recipe and every engine option. A newly added e2e fixture
automatically appears as an unmeasured row in the next bank. Broader fuzzing is
a separate robustness concern and should not be mistaken for calibrated physics
coverage.

## Create and measure a bank

Use a new directory for each engine revision or deliberate fixture change:

```text
cargo run --release -p spaghettio_core --example calibration_matrix_export -- \
  /path/to/calibration-bank-YYYY-MM-DD
scripts/run-calibration-matrix.sh /path/to/calibration-bank-YYYY-MM-DD
cargo run --release -p spaghettio_meter --example sweep_postlift -- \
  /path/to/calibration-bank-YYYY-MM-DD /tmp/meter-vs-sim.csv
```

The exporter refuses a non-empty target directory. That is intentional: do not
replace a blueprint beneath an old `report.json`, because a label match alone
cannot prove that the report was made from the current file. `matrix.json`
records the fixture declaration, geometry, validator summary, and SHA-256 of
each exported blueprint, and `sweep_postlift` re-hashes every `bp.txt` it
reads against that entry — a row whose blueprint no longer matches is excluded,
not vetted. A fixture that fails to build at export is recorded under
`build_failures` in `matrix.json` and the export continues (exiting non-zero):
an engine regression is exactly when the other rows' measurements are wanted.

The runner resumes a stopped campaign by skipping existing reports — one
completed Factorio run per fixture; a changed factory always requires a new
bank. A run that completed but did not converge, or reported kit errors, still
wrote its report: that **is** the row's result (deterministic — re-running it
reproduces it), and the sweep lists the row as excluded with the reason. To
re-measure such a row deliberately, delete its `report.json` first. Two things
are not results and are retried on the next invocation: a harness failure
(timeout, crash, pre-flight — no report is written, the failure is logged, the
script exits non-zero at the end) and a `report.json` that does not parse (a
kill or full disk mid-write — treated as absent).

`matrix.json` carries `schema_version` 2: per-row `blueprint_sha256` **and**
`manifest_sha256` (the immutable pair, both checked by the sweep),
`corpus_size`, and `build_failures`. The sweep still reads a version-1 bank
(blueprint hash only, no failure record) and says which branch it took.

Each Factorio run uses a 432,000-tick warmup at speed 32, matching the
post-lift provenance bar. Runs remain sequential because the main purpose is
reproducibility and the largest factories are CPU-bound. Parallel campaigns
need an explicit resource budget and independent Factorio installs.

## The committed fingerprint and the CI probe

`crates/core/data/calibration-bank/matrix.json` is a committed copy of the
current bank's `matrix.json` — the corpus fingerprint the engine on `main` is
expected to reproduce. CI's `rust` job runs the ignored
`selection_policy_calibration_issue_breakdown` driver against it (with
a scratch copy of the committed zone cache) and fails on any blueprint-hash,
manifest-hash, or validator-total drift — the manifest half matters because it
carries the planned rates the calibration compares against, so a rate-only
change with identical geometry also makes a row's measurement stale. That is
the golden discipline applied to calibration: a PR that changes what a
calibrated row ships or claims has made that row's Factorio measurement
stale, and the failure surfaces it at PR time instead of at the next
calibration round.

When the drift is intended, refresh in the same PR:

1. Re-export a fresh bank (command above) and diff `blueprint_sha256` per
   label against the previous bank — the diff names exactly which rows'
   measurements went stale.
2. Copy the new `matrix.json` over the committed one.
3. Carry unchanged rows' reports into the new bank (byte-verify `bp.txt` and
   `manifest-real.json` first), re-measure the changed rows, and regenerate
   [`selection-policy-calibration-evidence.md`](selection-policy-calibration-evidence.md)
   via `scripts/calibration_evidence.py`. If measurement must lag the merge,
   say so in the PR body — the fingerprint keeps the record honest either way.

The probe run is deterministic in CI because the corpus solves entirely from
the committed zone cache (verified 2026-08-24: a full export left a copy of
the cache byte-identical). If a new fixture introduces uncached zones, the
zone-cache refresh protocol in `.github/workflows/ci.yml` applies first.

## Reading coverage honestly

`sweep_postlift` prints every fixture directory without a usable report as an
exclusion, rather than silently shrinking the denominator. Its
`MATRIX COVERAGE` line reconciles to the corpus the bank declares: vetted +
awaiting measurement + excluded for another reason + failed to build = corpus
size, so a shortfall in any bucket is visible as a shortfall rather than
absorbed into a smaller denominator. It compares solid targets on both produced
and delivered rates, reports threshold classifications, and keeps fluid target
rows visible but unjudged until a comparable fluid metric is established. Its
output, plus the `matrix.json` fingerprint, is the calibration matrix record;
neither a green e2e test nor a meter-only run is a substitute for a Factorio
measurement.

Historical context and the known divergence results live in
[`meter-divergence.md`](meter-divergence.md). This document owns the workflow
that prevents the next result from becoming another local-only, partial bank.
