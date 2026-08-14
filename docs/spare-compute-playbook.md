# Spare-compute playbook

**Followups-class doc.** Jobs for when there is lots of local compute but
little frontier-agent budget — each executable by a basic local agent
(qwen-class, e.g. the pi-coding-agent container in this repo) following the
recipe verbatim. Written 2026-08-01. Status: none started.

## Ground rules for the local agent (read first, apply to every job)

- **Capture, don't interpret.** Save every raw output (JSON, logs, .fls,
  bp.txt, PNGs). A later session does the analysis. Never summarize-and-
  discard; disk is the cheap resource.
- **Read-only repo.** No commits, no branch changes, no cache blessing.
  (`SPAGHETTIO_STRESS_GOLDEN` is safe to set — since 2026-08-15 it only
  prints STRESSGOLD hash lines; the golden-file bless flow is deleted.)
  Artifacts go to a dated directory under `~/spaghettio-corpora/<job>/<date>/`.
- **Stamp provenance in every artifact dir**: `git rev-parse HEAD`, `rustc
  --version`, Factorio version, and the exact command used. (Sim results
  are tech-state- and version-sensitive; unstamped data is undiagnosable.)
- **One row per instance** in any index file — never aggregate counts only.
- **Fixed retry rule**: any command that fails gets exactly one retry; a
  second failure is recorded as a row (`status: failed`, stderr tail
  attached) and the loop continues. Never debug, never tune.

## Job 1 — Scaling atlas (highest value per compute-hour)

**What**: sweep recipe × rate × options; record metrics + snapshot + PNG
per cell so scaling behavior becomes *visible*.

**Grid**: ~10 representative recipes (gear, EC, AC, plastic, sulfur, PU,
low-density-structure, chem/util science, mil science) × rates
{1, 2, 5, 10, 15, 22, 30, 45, 60}/s × options {native, `compact=1`,
`fold=1`} × belt tier per rate feasibility. ~800–1200 cells.

**How**: a Rust driver modeled on
`crates/core/examples/rfc064_phase2_dry_sweep.rs` (preserved on the dev
host; the pattern: build solve → layout per option → collect) emitting per
cell: dims, AR, entities, belts, machine count, validator issue categories
(per category, per instance), belt-detour count, refusal/error if the build
fails, wall-time. Dump `.fls` via `SPAGHETTIO_DUMP_SNAPSHOTS` machinery and
`export_blueprint` string. PNGs: the dev server + a playwright screenshot
loop over snapshot loads (the RFC-064 Phase 0 calibration built exactly
this pipeline: load .fls, hide sidebar, zoom-to-fit, screenshot). Finish
with a contact-sheet HTML (labels + thumbnails, grouped by recipe, rate
ascending) so a human can scan 1000 layouts in minutes.

**Later consumption**: where does AR blow up, where do refusals cluster,
at which rates does lane-splitting/balancer machinery kick in and what
does it cost. Directly feeds RFC-064 Phases 3–5 targeting.

## Job 2 — Sim baseline bank (RFC-064 Phase 2 Stage B and beyond)

**What**: the 68-run Phase 2 campaign (spec: RFC-064 decision log, "Stage
B pick-up notes") and then measured native baselines for the full corpus.
Every run: `cargo run --release -p spaghettio_sim_harness -- run --bp ...
--manifest ... --warmup <per spec> --out <dir>/fixture__variant.json`,
≤3 concurrent, poll report files to completion. Adjudication rows only
(compacted-vs-native produced/delivered); verdicts are for a smart session.
A complete baseline bank makes every future layout change regression-
checkable by rerunning and diffing — the single most durable asset spare
compute can buy this repo.

## Job 3 — Whole-recipe-DB robustness fuzz

**What**: every recipe in `crates/core/data/recipes.json` × a small rate
ladder × input-set variants → run solve+layout+validate, bucket outcomes:
OK-clean / OK-with-warnings (categories listed) / typed refusal (which) /
panic (backtrace saved) / timeout. Output: one CSV row per attempt + a
reproducer URL per non-clean outcome (the web app URL scheme encodes
everything). **Why**: the tetris direction's declared risk is engine
robustness; this maps the refusal/panic surface exhaustively so router
investment is aimed by data. A panic corpus with reproducers is gold.

## Job 4 — Fold + detour maps over the full DB

**What**: (a) `search_snake_fold` admissibility over every recipe/rate in
Job 3's grid (which shapes fold, dominant refusal otherwise — extends the
14-fixture Phase 1 spike to the whole space); (b) belt-detour measurement
(`measure_belt_runs`, on main once PR #565 merges) over the same grid —
detour hotspot map by recipe/rate/strategy. Both are pure loops over
existing pub APIs.

## Job 5 — Community blueprint survey

**What**: run `blueprint-analyze --batch --json` over large community
blueprint collections (user supplies the dumps; the tool already expands
books). Record per blueprint: entity mix, dims, AR, belt/UG counts,
machine spacing stats. **Why**: empirical anchors for the spaghetti
objective — what do *human*-built factories measure as on AR and belt
length? Currently the objective is calibrated on N=1 owner judgment;
a distribution over thousands of human designs is the cheapest possible
second calibration source.

## Job 6 — Long-horizon soak sims

**What**: flagship fixtures (mil5ore folded, EC@30, PU@2) at very long
sim horizons (hours of game time, checkpoint series on) to catch slow
drift the standard measurement window misses (buffer oscillation,
gradual starvation). Low priority; run when Jobs 1–2 are done.

## Suggested order

Job 2 (unblocks RFC-064 Phase 2 directly) → Job 1 (atlas) → Job 3 (fuzz)
→ Job 4 → Job 5 (needs user-supplied dumps) → Job 6.
