//! RFC-064 P2b: a general candidate-evaluation loop — produce → validate →
//! measure → verdict-vs-incumbent → rank — that lets a base production
//! strategy (any [`DecompositionCandidate`]) compete against an incumbent
//! without hand-rolling its own scoring/gating, the way
//! `bus::decomposition_search::score_layout` did before [`crate::objective`]
//! (P1) and [`crate::verdict`] (P2a) existed.
//!
//! Originally this ran a two-stage pipeline — base production followed by a
//! chain of post-layout `LayoutTransform`s (`CompactTransform`/
//! `FoldTransform`, proving the abstraction could reproduce
//! `build_bus_layout`'s `compact_layout`/`fold_layout` flags byte-for-byte).
//! Both flags and their transforms were deleted 2026-08-14 (#632 A2, owner
//! call) after the underlying `bus::compaction` relocation research never
//! shipped past three falsified attempts (RFC-057/058/064-P3 decision
//! logs) — see `git log --follow` on this file for the removed
//! `LayoutTransform` trait, `TransformOutcome`, and `compose_chain`'s
//! [`crate::verdict::MatchTier`]-composition rule. The framework itself
//! survives: it is the live entry point for the RFC-068 celldb campaign
//! (`crates/core/tests/celldb_template.rs`), which competes
//! [`FullSelectionCandidate`] against `template_candidate::TemplateCandidate`
//! with no transform chain at all — every field candidate's issues are
//! verdicted against the incumbent at [`crate::verdict::MatchTier::Count`],
//! the only tier that requires no per-transform correspondence data.
//!
//! ## Searchable vs. pinned knobs
//!
//! [`run_candidate_field`] takes one `&LayoutOptions` and passes it through
//! UNCHANGED to every base producer's `produce` — belt tier in particular is
//! a hard user constraint, never a search axis (long-standing project rule;
//! see the doc comment on `LayoutOptions` itself for the full pinned/
//! searchable field legend this module's existence motivated). Variation is
//! expressed exclusively by WHICH `DecompositionCandidate` a [`CandidatePlan`]
//! names — never by mutating the options struct.

use crate::models::{LayoutResult, SolverResult};
use crate::objective::{self, ObjectiveScores};
use crate::trace::TraceEvent;
use crate::validate::{self, LayoutStyle};
use crate::verdict::{self, MatchTier, Policy, Verdict};

use super::decomposition_search::{self, DecompositionCandidate};
use super::layout::LayoutOptions;

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

/// A named plan: a base-production step, any existing
/// [`DecompositionCandidate`] — [`FullSelectionCandidate`],
/// `decomposition_search::NativeCandidate`, or any other.
///
/// Prior to 2026-08-14 (#632 A2) a plan also carried an ordered chain of
/// post-layout `LayoutTransform`s applied to the base producer's output;
/// that mechanism (and its only two implementations, `CompactTransform`/
/// `FoldTransform`) was deleted with the compact/fold stack — see this
/// module's header doc.
pub struct CandidatePlan {
    pub name: String,
    pub base: Box<dyn DecompositionCandidate>,
}

impl CandidatePlan {
    pub fn new(name: impl Into<String>, base: impl DecompositionCandidate + 'static) -> Self {
        Self {
            name: name.into(),
            base: Box::new(base),
        }
    }
}

// ---------------------------------------------------------------------------
// Running one plan
// ---------------------------------------------------------------------------

/// Runs one [`CandidatePlan`]'s base production. A thin, named wrapper
/// around `plan.base.produce` — kept as its own function (rather than
/// inlined at the two call sites below) because it used to also drive the
/// transform chain; the wrapper stays so a future transform-like mechanism
/// has one obvious seam to hook back into.
fn run_plan(
    plan: &CandidatePlan,
    solver: &SolverResult,
    opts: &LayoutOptions,
) -> Result<LayoutResult, String> {
    plan.base
        .produce(solver, opts)
        .map_err(|e| format!("{}: {e}", plan.base.name()))
}

/// Produces `plan`'s layout directly, without competing it against anything.
///
/// This makes NO claim about whether `plan` is any good: unlike
/// [`run_candidate_field`], there is no incumbent, no validation, no
/// measurement, no verdict, no ranking.
pub fn produce_plan(
    plan: &CandidatePlan,
    solver: &SolverResult,
    opts: &LayoutOptions,
) -> Result<LayoutResult, String> {
    run_plan(plan, solver, opts)
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
}

/// One field candidate's outcome — always present for every entry in
/// `field`, whether it made it into the ranking or not.
pub enum CandidateOutcome {
    /// Base production failed. Never entered validation/measurement/ranking
    /// at all.
    Refused {
        name: String,
        reason: String,
    },
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
    // Winner resolution and event replay look candidates up BY NAME — two
    // same-named plans would tie in ranking and replay the wrong plan's
    // events/layout (round-3 bot review, minor 5). Refuse up front, before
    // the sink swap, so this early return needs no restore.
    {
        let mut seen = std::collections::BTreeSet::new();
        for name in std::iter::once(incumbent.name.as_str())
            .chain(field.iter().map(|p| p.name.as_str()))
        {
            if !seen.insert(name) {
                return Err(format!("duplicate candidate plan name '{name}'"));
            }
        }
    }

    let original_sink = crate::trace::swap_sink(None);
    // The two incumbent-path failures below MUST restore the sink before
    // returning — a bare `?` here permanently disabled the thread's trace
    // streaming on the error path (PR #569 bot review, finding 1; compare
    // `select_best_decomposition`, which has no early exit between its swap
    // and restore).
    let restore_sink = |sink: Option<Box<dyn FnMut(&TraceEvent)>>| {
        if let Some(s) = sink {
            crate::trace::swap_sink(Some(s));
        }
    };

    let inc_start = crate::trace::peek_events_len();
    let inc_layout = match run_plan(incumbent, solver, opts) {
        Ok(l) => l,
        Err(e) => {
            restore_sink(original_sink);
            return Err(format!("incumbent '{}' failed to produce: {e}", incumbent.name));
        }
    };
    let inc_events = crate::trace::peek_events_since(inc_start);
    crate::trace::truncate_events(inc_start);

    let incumbent_issues = issues_of(&inc_layout, solver);
    let incumbent_measure = match objective::measure(&inc_layout, solver) {
        Ok(m) => m,
        Err(e) => {
            restore_sink(original_sink);
            return Err(format!("incumbent '{}' failed to measure: {e}", incumbent.name));
        }
    };
    let incumbent_scores = objective::score_vs_native(&incumbent_measure, &incumbent_measure);

    struct Captured {
        events: Vec<TraceEvent>,
        outcome: CandidateOutcome,
    }

    let mut captured: Vec<Captured> = Vec::with_capacity(field.len());
    for plan in field {
        let start = crate::trace::peek_events_len();
        let evaluated: Result<(LayoutResult, ObjectiveScores, Verdict), String> = (|| {
            let layout = run_plan(plan, solver, opts)?;
            let issues = issues_of(&layout, solver);
            let measure = objective::measure(&layout, solver)
                .map_err(|e| format!("measure failed: {e}"))?;
            let scores = objective::score_vs_native(&measure, &incumbent_measure);
            // No transform chain exists any more (#632 A2), so no candidate
            // can supply positional provenance for its issues relative to
            // the incumbent's — every field candidate is verdicted at
            // `MatchTier::Count`, same as an empty transform chain always
            // was even before the deletion.
            let v = verdict::never_worse(&incumbent_issues, &issues, policy, MatchTier::Count, None);
            Ok((layout, scores, v))
        })();
        let events = crate::trace::peek_events_since(start);
        crate::trace::truncate_events(start);
        let outcome = match evaluated {
            Ok((layout, scores, verdict)) => {
                CandidateOutcome::Evaluated(Box::new(EvaluatedCandidate {
                    name: plan.name.clone(),
                    layout,
                    scores,
                    verdict,
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
        inc_layout.entities.len(),
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
        inc_layout.clone()
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
        incumbent: inc_layout,
        entries: captured.into_iter().map(|c| c.outcome).collect(),
    })
}

fn issues_of(layout: &LayoutResult, solver: &SolverResult) -> Vec<validate::ValidationIssue> {
    match validate::validate(layout, Some(solver), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    }
}

