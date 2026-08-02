//! RFC-064 P2b: a general candidate-evaluation loop — produce → transform →
//! validate → measure → verdict-vs-incumbent → rank — that every current and
//! future layout-shuffling idea can plug into as a [`LayoutTransform`],
//! instead of each one hand-rolling its own scoring/gating like
//! `bus::compaction::search_snake_fold`'s `profile` closure and
//! `bus::decomposition_search::score_layout` did before
//! [`crate::objective`] (P1) and [`crate::verdict`] (P2a) existed.
//!
//! **This module ships nothing to any existing entry point.**
//! `bus::layout::build_bus_layout` and
//! `bus::decomposition_search::select_best_decomposition` are UNCHANGED —
//! this is a new, parallel entry point, consumed today only by this crate's
//! own tests. `CompactTransform`/`FoldTransform` below are wrappers proving
//! the abstraction can reproduce those two functions' existing
//! `compact_layout`/`fold_layout` behavior byte-for-byte (see
//! `crates/core/tests/candidate_runner.rs`'s parity tests); swapping the
//! shipping call sites onto this runner is explicitly a LATER decision with
//! its own gate, per the P2b brief.
//!
//! ## Two axes of "how much do we know about where an issue went"
//!
//! [`LayoutTransform::apply`] declares, per call, the strongest
//! [`crate::verdict::MatchTier`] it can support for THAT invocation, plus a
//! [`crate::verdict::CorrespondenceMap`] when the tier is
//! [`MatchTier::Provenance`]. A candidate's PLAN can chain several
//! transforms; [`compose_chain`] folds that chain into one tier/map for the
//! whole plan (see its own docs for the exact rule) before
//! [`run_candidate_field`] calls [`crate::verdict::never_worse`].
//!
//! ## Searchable vs. pinned knobs
//!
//! [`run_candidate_field`] takes one `&LayoutOptions` and passes it through
//! UNCHANGED to every base producer's `produce` and every transform's
//! `apply` — belt tier in particular is a hard user constraint, never a
//! search axis (long-standing project rule; see the doc comment on
//! `LayoutOptions` itself for the full pinned/searchable field legend this
//! module's existence motivated). Variation is expressed exclusively by
//! WHICH `DecompositionCandidate`/`LayoutTransform`s a [`CandidatePlan`]
//! names — never by mutating the options struct.

use crate::models::{LayoutResult, SolverResult};
use crate::objective::{self, ObjectiveScores};
use crate::trace::TraceEvent;
use crate::validate::{self, LayoutStyle};
use crate::verdict::{self, CorrespondenceMap, MatchTier, Policy, Verdict};

use super::compaction;
use super::decomposition_search::{self, DecompositionCandidate};
use super::layout::{LayoutOptions, FOLD_SEARCH_ENTITY_THRESHOLD};

// ---------------------------------------------------------------------------
// LayoutTransform
// ---------------------------------------------------------------------------

/// One post-layout arrangement/fabric transform, applied after a
/// [`CandidatePlan`]'s base production step. Implementations wrap EXISTING
/// pipeline functions (`compaction::compact_validated_geometry`,
/// `compaction::search_snake_fold`) rather than reimplementing them — see
/// [`CompactTransform`]/[`FoldTransform`] below.
pub trait LayoutTransform {
    /// Short, stable identifier for reporting (trace events, refusal
    /// messages) — not necessarily unique across a plan's whole chain.
    fn name(&self) -> &str;

    /// The transform's declared latency/applicability budget for `layout`
    /// (the layout as it stands AFTER every prior transform in this plan's
    /// chain has run). `Err` means "this transform does not apply to input
    /// this shape" and is a SOFT skip: `run_candidate_field` leaves `layout`
    /// unchanged and moves on to the next transform in the chain, exactly
    /// as `build_bus_layout`'s own fold latency guard falls back to the
    /// (compacted, unfolded) layout rather than refusing the whole build.
    /// It is NOT a hard candidate refusal — that is what an `Err` from
    /// [`Self::apply`] itself means.
    fn admissible_input(&self, layout: &LayoutResult) -> Result<(), String>;

    /// Perform the transform. Declares the strongest [`MatchTier`] this
    /// invocation can support and supplies a [`CorrespondenceMap`] when the
    /// tier is [`MatchTier::Provenance`] — see the module docs' "Two axes"
    /// section. An `Err` here is a HARD refusal: the whole candidate plan is
    /// dropped from the field (mirrors `DecompositionCandidate::produce`'s
    /// existing `Err` semantics).
    fn apply(
        &self,
        layout: &LayoutResult,
        solver: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<TransformOutcome, String>;
}

/// One [`LayoutTransform::apply`] call's result.
pub struct TransformOutcome {
    pub layout: LayoutResult,
    /// Present only when `tier == MatchTier::Provenance`. A `None` here
    /// under `Provenance` is a caller bug — [`compose_chain`] degrades to
    /// [`MatchTier::Count`] for the whole plan rather than panicking or
    /// guessing, the same "silently less precise, never wrong" rule
    /// `crate::verdict::never_worse` itself applies to the analogous case.
    pub correspondence: Option<CorrespondenceMap>,
    pub tier: MatchTier,
}

// ---------------------------------------------------------------------------
// Concrete transforms
// ---------------------------------------------------------------------------

/// Wraps `compaction::compact_validated_geometry` — exactly the function
/// `build_bus_layout` calls for both `compact_layout: true` and (always,
/// first) `fold_layout: true`.
///
/// **Tier: [`MatchTier::Count`].** The full pipeline (transport resynthesis
/// via `undergroundify_straight_belts`, then `strip_empty_columns`/
/// `strip_empty_rows`, then up to 3 rounds of validated transactional
/// column/row cut-collapsing, all iterated to a fixed point) genuinely
/// TRANSLATES geometry — so [`MatchTier::Positional`] would be wrong (only
/// safe for transforms that substitute in place, e.g.
/// `undergroundify_straight_belts` ALONE, per `verdict`'s own docs) — and a
/// closed-form [`CorrespondenceMap`] IS derivable in principle
/// (`strip_empty_columns`/`strip_empty_rows` already compute an internal
/// `remap_x`/`remap_y` closure per call, and each `collapse_vertical_cut`/
/// `collapse_horizontal_cut` is a simple integer shift) but wiring it
/// through means threading a map out of several currently-void-returning,
/// currently-private internal functions across TWO nested fixed-point loops
/// (`compact_transport_geometry`'s and `compact_validated_geometry`'s own),
/// each iteration re-deciding via a live `validate()` call whether to keep
/// a candidate cut. That is genuinely invasive by this unit's own kill
/// criterion ("extraction must leave the inline path byte-identical"), so
/// per the P2b brief's own escape hatch this ships at Count tier. Follow-up:
/// thread `strip_empty_columns`/`strip_empty_rows`'s own remap closures out
/// first (cheap — already computed, just discarded today) as a first step
/// toward Positional/Provenance-quality tracking for that sub-step; the
/// transactional cut-collapse loops would need more substantial extraction.
pub struct CompactTransform;

impl LayoutTransform for CompactTransform {
    fn name(&self) -> &str {
        "compact"
    }

    /// No guard exists for plain compaction in the shipping pipeline today
    /// (`build_bus_layout`'s `compact_layout` branch applies it
    /// unconditionally, unlike `fold_layout`'s entity-count threshold) —
    /// mirrored here as an unconditional `Ok`.
    fn admissible_input(&self, _layout: &LayoutResult) -> Result<(), String> {
        Ok(())
    }

    fn apply(
        &self,
        layout: &LayoutResult,
        solver: &SolverResult,
        _opts: &LayoutOptions,
    ) -> Result<TransformOutcome, String> {
        let compacted = compaction::compact_validated_geometry(layout, solver);
        Ok(TransformOutcome {
            layout: compacted,
            correspondence: None,
            tier: MatchTier::Count,
        })
    }
}

/// Wraps `compaction::search_snake_fold` + `compaction::fold_point_correspondence`
/// — exactly what `build_bus_layout`'s `fold_layout: true` branch runs
/// (always on top of an already-compacted layout; see [`CompactTransform`]'s
/// docs and RFC-064 Phase 0/1's own finding that every established fold
/// number in this repo was measured against `compact_validated_geometry`'s
/// output, never raw layout geometry).
///
/// `max_folds` mirrors `build_bus_layout`'s hardcoded `4` — exposed as a
/// field rather than a constant so tests can probe smaller search budgets
/// without waiting on the full one.
pub struct FoldTransform {
    pub max_folds: usize,
}

impl Default for FoldTransform {
    fn default() -> Self {
        Self { max_folds: 4 }
    }
}

impl LayoutTransform for FoldTransform {
    fn name(&self) -> &str {
        "fold"
    }

    /// Ports `build_bus_layout`'s own fold latency guard (layout.rs, the
    /// `fold_layout` branch) verbatim — same threshold constant, same
    /// condition — so the budget lives with the transform, not the call
    /// site, per the P2b brief.
    fn admissible_input(&self, layout: &LayoutResult) -> Result<(), String> {
        if layout.entities.len() > FOLD_SEARCH_ENTITY_THRESHOLD {
            return Err(format!(
                "fold search skipped: layout too large ({} entities > {} threshold)",
                layout.entities.len(),
                FOLD_SEARCH_ENTITY_THRESHOLD,
            ));
        }
        Ok(())
    }

    fn apply(
        &self,
        layout: &LayoutResult,
        solver: &SolverResult,
        _opts: &LayoutOptions,
    ) -> Result<TransformOutcome, String> {
        let search = compaction::search_snake_fold(layout, solver, self.max_folds);
        match search.best {
            Some(found) => {
                let correspondence = compaction::fold_point_correspondence(layout, &found.folds);
                Ok(TransformOutcome {
                    layout: found.layout,
                    correspondence: Some(correspondence),
                    tier: MatchTier::Provenance,
                })
            }
            // No admissible fold: `build_bus_layout` falls back to the
            // input (compacted, unfolded) layout unchanged — a literal
            // no-op, which is exactly what MatchTier::Positional asserts
            // ("substitutes in place"); trivially true for a transform that
            // substituted nothing at all.
            None => Ok(TransformOutcome {
                layout: layout.clone(),
                correspondence: None,
                tier: MatchTier::Positional,
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Base production adapter
// ---------------------------------------------------------------------------

/// Wraps `decomposition_search::select_best_decomposition` — the FULL
/// candidate competition `build_bus_layout` itself uses as its base
/// production step (k1-shape-fix, size-split, merge-tap, cell-composed,
/// direct-insertion, horizontal-stack — not merely `NativeCandidate`, which
/// is only one of that competition's entrants). Needed so a
/// [`CandidatePlan`]'s base-production slot can reproduce
/// `build_bus_layout`'s exact incumbent (parity tests compare against
/// `build_bus_layout`'s real output, which always goes through the full
/// selection, never bare `NativeCandidate`), without duplicating the
/// selection logic itself — this is a one-line call-through.
pub struct FullSelectionCandidate;

impl DecompositionCandidate for FullSelectionCandidate {
    fn name(&self) -> &str {
        "full-selection"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        decomposition_search::select_best_decomposition(solver_result, opts.clone())
    }
}

// ---------------------------------------------------------------------------
// CandidatePlan
// ---------------------------------------------------------------------------

/// A named plan: a base-production step (any existing
/// [`DecompositionCandidate`] — [`FullSelectionCandidate`],
/// `decomposition_search::NativeCandidate`, or any other) plus an ordered
/// chain of [`LayoutTransform`]s applied to its output in sequence.
pub struct CandidatePlan {
    pub name: String,
    pub base: Box<dyn DecompositionCandidate>,
    pub transforms: Vec<Box<dyn LayoutTransform>>,
}

impl CandidatePlan {
    pub fn new(name: impl Into<String>, base: impl DecompositionCandidate + 'static) -> Self {
        Self {
            name: name.into(),
            base: Box::new(base),
            transforms: Vec::new(),
        }
    }

    pub fn with_transform(mut self, transform: impl LayoutTransform + 'static) -> Self {
        self.transforms.push(Box::new(transform));
        self
    }
}

// ---------------------------------------------------------------------------
// Chain composition
// ---------------------------------------------------------------------------

/// Folds a [`CandidatePlan`]'s whole transform chain into ONE
/// `(MatchTier, Option<CorrespondenceMap>)` for `run_candidate_field`'s
/// single `never_worse` call against the incumbent.
///
/// Rule (documented here since no single transform embodies it):
///
/// - **Empty chain** (no transforms ran — including every transform being
///   admissible-skipped): [`MatchTier::Count`], no map. A plan with no
///   transforms is not guaranteed to share the incumbent's coordinate
///   frame at all (its base producer may differ from the incumbent's), so
///   this does NOT default to `Positional`/identity — that would silently
///   claim precision the inputs don't support, the same discipline
///   `crate::verdict`'s own docs apply throughout.
/// - **Any step declares [`MatchTier::Count`]**: the WHOLE chain degrades
///   to `Count`, regardless of position in the sequence. Once one link
///   provides zero positional information, no later step's map — however
///   precise on its own — can be composed through it: the map's domain is
///   that step's OUTPUT frame, and nothing here can say where an original
///   position landed after an opaque Count-tier step.
/// - **Every step is [`MatchTier::Positional`]**: `Positional`, no map
///   needed — every step is (by that tier's own definition) an in-place
///   substitution, so the composed identity needs no map object.
/// - **Mixed Positional/Provenance, no Count**: `Provenance`, with a
///   composed map built by walking each Provenance step's own domain
///   forward through the rest of the chain — a Positional step passes a
///   position through unchanged (by definition), a Provenance step looks
///   it up in its own map. This is symmetric under reordering: a
///   Provenance step first, followed by Positional steps, uses its own
///   pairs verbatim (nothing downstream moves them again); a Positional
///   prefix followed by a Provenance step ALSO uses that step's own pairs
///   verbatim (nothing upstream moved them either) — either order composes
///   to the same answer for a chain with exactly one Provenance step, and
///   `compose_chain`'s own tests pin both orderings plus a genuine
///   two-Provenance-step composition.
fn compose_chain(
    steps: &[(MatchTier, Option<CorrespondenceMap>)],
) -> (MatchTier, Option<CorrespondenceMap>) {
    if steps.is_empty() {
        return (MatchTier::Count, None);
    }
    if steps.iter().any(|(tier, _)| *tier == MatchTier::Count) {
        return (MatchTier::Count, None);
    }
    if steps.iter().all(|(tier, _)| *tier == MatchTier::Positional) {
        return (MatchTier::Positional, None);
    }

    let mut pairs: Option<Vec<((i32, i32), (i32, i32))>> = None;
    for (tier, corr) in steps {
        match tier {
            MatchTier::Positional => {}
            MatchTier::Provenance => {
                let Some(map) = corr.as_ref() else {
                    // Caller bug: this step declared Provenance without a
                    // map. Degrade the WHOLE chain to Count rather than
                    // panicking or guessing which step's absence should
                    // "count" — same rule `never_worse`'s own
                    // `expected_position` applies to this exact case.
                    return (MatchTier::Count, None);
                };
                pairs = Some(match pairs.take() {
                    None => map.keys().filter_map(|k| map.get(k).map(|v| (k, v))).collect(),
                    Some(prev) => prev
                        .into_iter()
                        .filter_map(|(from, mid)| map.get(mid).map(|to| (from, to)))
                        .collect(),
                });
            }
            MatchTier::Count => unreachable!("filtered out by the guard above"),
        }
    }
    (MatchTier::Provenance, pairs.map(CorrespondenceMap::from_pairs))
}

// ---------------------------------------------------------------------------
// Running one plan
// ---------------------------------------------------------------------------

/// One plan's produced-and-transformed layout, plus the chain's composed
/// tier/map and every transform that was admissible-skipped along the way
/// (reporting only — a skip is not a failure).
struct PlanRun {
    layout: LayoutResult,
    tier: MatchTier,
    correspondence: Option<CorrespondenceMap>,
    skipped_transforms: Vec<(String, String)>,
}

/// Runs one [`CandidatePlan`]'s base production, then its transform chain
/// in order. An `Err` from base production OR from any transform's `apply`
/// refuses the WHOLE plan (see [`LayoutTransform::apply`]'s docs for why
/// `apply` errors are hard while `admissible_input` errors are soft skips).
fn run_plan(
    plan: &CandidatePlan,
    solver: &SolverResult,
    opts: &LayoutOptions,
) -> Result<PlanRun, String> {
    let mut layout = plan
        .base
        .produce(solver, opts)
        .map_err(|e| format!("{}: {e}", plan.base.name()))?;

    let mut steps: Vec<(MatchTier, Option<CorrespondenceMap>)> = Vec::new();
    let mut skipped_transforms: Vec<(String, String)> = Vec::new();
    for transform in &plan.transforms {
        if let Err(reason) = transform.admissible_input(&layout) {
            skipped_transforms.push((transform.name().to_string(), reason));
            continue;
        }
        let outcome = transform
            .apply(&layout, solver, opts)
            .map_err(|e| format!("{}: {e}", transform.name()))?;
        layout = outcome.layout;
        steps.push((outcome.tier, outcome.correspondence));
    }

    let (tier, correspondence) = compose_chain(&steps);
    Ok(PlanRun {
        layout,
        tier,
        correspondence,
        skipped_transforms,
    })
}

/// Produces `plan`'s layout directly — base production followed by its own
/// transform chain — without competing it against anything.
///
/// This makes NO claim about whether `plan` is any good: unlike
/// [`run_candidate_field`], there is no incumbent, no validation, no
/// measurement, no verdict, no ranking. It exists because `compact_layout`/
/// `fold_layout` (the flags [`CompactTransform`]/[`FoldTransform`] wrap)
/// apply UNCONDITIONALLY in `build_bus_layout` — there is no "does this
/// beat native" gate at that call site at all, so a parity test comparing
/// this runner's transform chain against `build_bus_layout`'s output is a
/// claim about what the CHAIN PRODUCES, not about whether
/// `run_candidate_field`'s own (fixture-dependent) objective score would
/// have ranked it above the incumbent — those are different questions, and
/// only the first one is what "parity" means here.
pub fn produce_plan(
    plan: &CandidatePlan,
    solver: &SolverResult,
    opts: &LayoutOptions,
) -> Result<LayoutResult, String> {
    run_plan(plan, solver, opts).map(|r| r.layout)
}

// ---------------------------------------------------------------------------
// Field result
// ---------------------------------------------------------------------------

/// A produced, validated, measured, and verdicted field candidate.
/// `verdict.pass` decides whether it was eligible for ranking —
/// `CandidateOutcome::Evaluated` does not itself mean "won" or even
/// "competed". Boxed inside `CandidateOutcome` (clippy `large_enum_variant`
/// — `LayoutResult` dwarfs `Refused`'s two `String`s).
pub struct EvaluatedCandidate {
    pub name: String,
    pub layout: LayoutResult,
    pub scores: ObjectiveScores,
    pub verdict: Verdict,
    pub skipped_transforms: Vec<(String, String)>,
}

/// One field candidate's outcome — always present for every entry in
/// `field`, whether it made it into the ranking or not.
pub enum CandidateOutcome {
    /// Base production failed, or a transform's `apply` failed. Never
    /// entered validation/measurement/ranking at all.
    Refused { name: String, reason: String },
    Evaluated(Box<EvaluatedCandidate>),
}

impl CandidateOutcome {
    pub fn name(&self) -> &str {
        match self {
            CandidateOutcome::Refused { name, .. } => name,
            CandidateOutcome::Evaluated(c) => &c.name,
        }
    }
}

/// The full result of one [`run_candidate_field`] call: the winner (by
/// name and layout — the incumbent's own name/layout if nothing in `field`
/// beat it) and every field candidate's outcome, for reporting.
pub struct FieldResult {
    pub winner_name: String,
    pub winner: LayoutResult,
    pub incumbent_name: String,
    pub incumbent: LayoutResult,
    pub entries: Vec<CandidateOutcome>,
}

// ---------------------------------------------------------------------------
// The runner
// ---------------------------------------------------------------------------

/// Produces the incumbent, then every plan in `field`; validates and
/// measures each; verdicts each field candidate against the incumbent
/// (`policy` governs the verdict's per-category gating, same as any other
/// `never_worse` call site — e.g. `verdict::Policy::fold()` reproduces
/// `search_snake_fold`'s own historical gate); ranks every PASSING
/// candidate plus the incumbent via `objective::rank_admissible`.
///
/// The incumbent is always in the ranking, scored `0.0` against itself by
/// construction (`objective::score_vs_native` of a measure against itself)
/// — no verdict is computed for it (there is nothing to verdict it
/// against). This is what makes "a transform that scores worse than doing
/// nothing loses by construction" true: the incumbent is a real, ranked
/// competitor, not a fallback consulted only when the field is empty.
///
/// Trace events are captured per plan and only the WINNING plan's are
/// replayed to the live sink/collector at the end — the same
/// capture-and-replay discipline `decomposition_search::
/// select_best_decomposition` uses for its own (differently-shaped) inner
/// candidate competition, reimplemented here at the plan granularity rather
/// than shared code, since that function's `CandidateRun`/`run_candidate`
/// are private and scoped to its own 7-candidate array shape (see the P2b
/// report for why extracting them was judged not worth the coupling).
///
/// Errors only when the INCUMBENT itself fails to produce or measure —
/// there is no baseline to rank against otherwise. A field candidate that
/// fails is recorded as `CandidateOutcome::Refused` and simply excluded;
/// it never fails the whole call.
pub fn run_candidate_field(
    solver: &SolverResult,
    opts: &LayoutOptions,
    incumbent: &CandidatePlan,
    field: &[CandidatePlan],
    policy: &Policy,
) -> Result<FieldResult, String> {
    let original_sink = crate::trace::swap_sink(None);

    let inc_start = crate::trace::peek_events_len();
    let inc_run = run_plan(incumbent, solver, opts)
        .map_err(|e| format!("incumbent '{}' failed to produce: {e}", incumbent.name))?;
    let inc_events = crate::trace::peek_events_since(inc_start);
    crate::trace::truncate_events(inc_start);

    let incumbent_issues = issues_of(&inc_run.layout, solver);
    let incumbent_measure = objective::measure(&inc_run.layout, solver)
        .map_err(|e| format!("incumbent '{}' failed to measure: {e}", incumbent.name))?;
    let incumbent_scores = objective::score_vs_native(&incumbent_measure, &incumbent_measure);

    struct Captured {
        events: Vec<TraceEvent>,
        outcome: CandidateOutcome,
    }

    let mut captured: Vec<Captured> = Vec::with_capacity(field.len());
    for plan in field {
        let start = crate::trace::peek_events_len();
        let evaluated: Result<(LayoutResult, ObjectiveScores, Verdict, Vec<(String, String)>), String> =
            (|| {
                let run = run_plan(plan, solver, opts)?;
                let issues = issues_of(&run.layout, solver);
                let measure = objective::measure(&run.layout, solver)
                    .map_err(|e| format!("measure failed: {e}"))?;
                let scores = objective::score_vs_native(&measure, &incumbent_measure);
                let v = verdict::never_worse(
                    &incumbent_issues,
                    &issues,
                    policy,
                    run.tier,
                    run.correspondence.as_ref(),
                );
                Ok((run.layout, scores, v, run.skipped_transforms))
            })();
        let events = crate::trace::peek_events_since(start);
        crate::trace::truncate_events(start);
        let outcome = match evaluated {
            Ok((layout, scores, verdict, skipped_transforms)) => {
                CandidateOutcome::Evaluated(Box::new(EvaluatedCandidate {
                    name: plan.name.clone(),
                    layout,
                    scores,
                    verdict,
                    skipped_transforms,
                }))
            }
            Err(reason) => CandidateOutcome::Refused {
                name: plan.name.clone(),
                reason,
            },
        };
        captured.push(Captured { events, outcome });
    }

    let mut ranking: Vec<(String, ObjectiveScores, usize)> = vec![(
        incumbent.name.clone(),
        incumbent_scores,
        inc_run.layout.entities.len(),
    )];
    for c in &captured {
        if let CandidateOutcome::Evaluated(ec) = &c.outcome {
            if ec.verdict.pass {
                ranking.push((ec.name.clone(), ec.scores, ec.layout.entities.len()));
            }
        }
    }
    let winner_name = objective::rank_admissible(&ranking)
        .into_iter()
        .next()
        .expect("the incumbent is unconditionally in `ranking`");

    // Reattach the live sink, then replay ONLY the winner's captured events
    // — mirrors `select_best_decomposition`'s own replay-the-winner-only
    // discipline so a losing plan's work never bleeds into the streaming
    // web UI or the final `LayoutResult.trace` snapshot.
    if let Some(sink) = original_sink {
        crate::trace::swap_sink(Some(sink));
    }

    let winner_layout = if winner_name == incumbent.name {
        for ev in inc_events {
            crate::trace::emit(ev);
        }
        inc_run.layout.clone()
    } else {
        let winner = captured
            .iter()
            .find(|c| c.outcome.name() == winner_name)
            .expect("a ranked non-incumbent name must be a captured field entry");
        for ev in winner.events.clone() {
            crate::trace::emit(ev);
        }
        match &winner.outcome {
            CandidateOutcome::Evaluated(ec) => ec.layout.clone(),
            CandidateOutcome::Refused { .. } => {
                unreachable!("a Refused candidate never enters `ranking`")
            }
        }
    };

    Ok(FieldResult {
        winner_name,
        winner: winner_layout,
        incumbent_name: incumbent.name.clone(),
        incumbent: inc_run.layout,
        entries: captured.into_iter().map(|c| c.outcome).collect(),
    })
}

fn issues_of(layout: &LayoutResult, solver: &SolverResult) -> Vec<validate::ValidationIssue> {
    match validate::validate(layout, Some(solver), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance(pairs: &[((i32, i32), (i32, i32))]) -> (MatchTier, Option<CorrespondenceMap>) {
        (
            MatchTier::Provenance,
            Some(CorrespondenceMap::from_pairs(pairs.iter().copied())),
        )
    }

    fn positional() -> (MatchTier, Option<CorrespondenceMap>) {
        (MatchTier::Positional, None)
    }

    fn count() -> (MatchTier, Option<CorrespondenceMap>) {
        (MatchTier::Count, None)
    }

    #[test]
    fn empty_chain_is_count() {
        let (tier, map) = compose_chain(&[]);
        assert_eq!(tier, MatchTier::Count);
        assert!(map.is_none());
    }

    #[test]
    fn single_provenance_step_reproduces_its_own_map() {
        let steps = [provenance(&[((0, 0), (5, 5)), ((1, 0), (6, 5))])];
        let (tier, map) = compose_chain(&steps);
        assert_eq!(tier, MatchTier::Provenance);
        let map = map.expect("provenance tier must carry a map");
        assert_eq!(map.get((0, 0)), Some((5, 5)));
        assert_eq!(map.get((1, 0)), Some((6, 5)));
    }

    #[test]
    fn count_anywhere_collapses_the_whole_chain() {
        // Provenance, then Count, then Provenance: even though both
        // Provenance steps are individually precise, the Count step in the
        // middle severs the chain of custody — the whole thing must
        // degrade, not just the step after the Count one.
        let steps = [
            provenance(&[((0, 0), (1, 1))]),
            count(),
            provenance(&[((1, 1), (2, 2))]),
        ];
        let (tier, map) = compose_chain(&steps);
        assert_eq!(tier, MatchTier::Count);
        assert!(map.is_none());
    }

    #[test]
    fn all_positional_is_positional_with_no_map() {
        let steps = [positional(), positional()];
        let (tier, map) = compose_chain(&steps);
        assert_eq!(tier, MatchTier::Positional);
        assert!(map.is_none());
    }

    #[test]
    fn positional_then_provenance_uses_the_provenance_steps_own_pairs() {
        let steps = [positional(), provenance(&[((3, 3), (9, 9))])];
        let (tier, map) = compose_chain(&steps);
        assert_eq!(tier, MatchTier::Provenance);
        assert_eq!(map.unwrap().get((3, 3)), Some((9, 9)));
    }

    #[test]
    fn provenance_then_positional_uses_the_provenance_steps_own_pairs() {
        let steps = [provenance(&[((3, 3), (9, 9))]), positional()];
        let (tier, map) = compose_chain(&steps);
        assert_eq!(tier, MatchTier::Provenance);
        assert_eq!(map.unwrap().get((3, 3)), Some((9, 9)));
    }

    #[test]
    fn two_provenance_steps_compose_by_lookup_chaining() {
        // (0,0) -> (5,5) -> (100,100); a point only in the first map's
        // domain that the second map doesn't cover must drop out cleanly.
        let steps = [
            provenance(&[((0, 0), (5, 5)), ((1, 1), (9, 9))]),
            provenance(&[((5, 5), (100, 100))]),
        ];
        let (tier, map) = compose_chain(&steps);
        assert_eq!(tier, MatchTier::Provenance);
        let map = map.unwrap();
        assert_eq!(map.get((0, 0)), Some((100, 100)));
        assert_eq!(
            map.get((1, 1)),
            None,
            "(9,9) has no entry in the second map, so (1,1) must not resolve"
        );
    }

    #[test]
    fn provenance_without_a_map_degrades_the_whole_chain_to_count() {
        // Caller bug: a step claims Provenance but carries no map.
        let steps: Vec<(MatchTier, Option<CorrespondenceMap>)> =
            vec![(MatchTier::Provenance, None)];
        let (tier, map) = compose_chain(&steps);
        assert_eq!(tier, MatchTier::Count);
        assert!(map.is_none());
    }
}
