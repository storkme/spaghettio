//! RFC-070 Phase 2c: candidate selection expressed as
//! **policy data** instead of open-coded mechanisms.
//!
//! Phase 2b wires `select_best_decomposition` to ship this decision over
//! its scoreboard profiles. Phase 2c removes the former v1 chain, leaving
//! this policy as the sole decision path; the committed parity corpus is
//! the proof that the shipped winner and stage remain unchanged.
//!
//! # The reframe: one measurement, three comparators
//!
//! Reading the mechanisms at source dissolves RFC-070's "three verdict
//! mechanisms" into a cleaner factorization: there is **one underlying
//! measurement** and three comparators consuming different projections
//! of it. The selection path retains `classify_errors`, `count_issues` and the `clean_flags`
//! closure each independently run `validate::validate` on the same
//! layout and project the same issue list three ways, so a contested
//! candidate is validated up to three times per selection.
//! [`IssueProfile`] is that measurement taken **once**; the comparators
//! are [`StageKind`]'s three arms.
//!
//! # A `None` is a GAP, never a zero
//!
//! Inherited verbatim from the Phase-0b scoreboard, and it is the rule
//! that makes offline replay honest: an absent field means *no mechanism
//! computed this on this call*, which is not the same fact as "computed
//! it and got 0". Every stage that needs a field it does not have
//! **skips**, exactly as the retained lazy measurement sites do (`clean_flags` is not
//! computed at all when merge-tap or a scoped pairwise already decided,
//! or when only one candidate produced a layout). Reading a gap as 0 is
//! the `unwrap_or(0)` trap that has silently reported "no findings"
//! here before.
//!
//! # The K70-1 boundary, mechanically
//!
//! RFC-070's premise test: if reproducing today's decisions requires
//! **candidate-identity-conditioned verdict logic**, the "four answers to
//! one question" premise is false and the campaign stops. The boundary
//! is stated so it can be checked rather than argued:
//!
//! > Producer-*keyed configuration* — the fields of
//! > [`ProducerRegistration`] — is policy data and does not trip K70-1.
//! > Producer-*name-conditioned branches inside stage logic* do. **Stage
//! > code may read registration FIELDS, never registration NAMES.**
//!
//! The stage logic in this file is fenced between the
//! `K70-1-FENCE-BEGIN` / `K70-1-FENCE-END` markers, and
//! `k70_1_fence_holds` asserts that no candidate name and no `.name`
//! read occurs in the non-comment lines inside the fence. Grep the
//! markers before adding a branch there.
//!
//! It is mechanical **for the literal form**, which is the form the
//! failure actually takes; it is not a proof. A name assembled at
//! runtime would pass it, and an inline `// native` on a code line would
//! fail it — the second direction being the safe one. Treat it as the
//! tripwire that makes the boundary checkable, with the argument in this
//! doc as the boundary itself.

use std::collections::{BTreeMap, BTreeSet};

use crate::models::{LayoutResult, SolverResult};
use crate::trace::{SelectionCandidateOutcome, SelectionStage};

use super::decomposition_search::{
    build_k1_enrollment_plan, score_layout, CellComposedCandidate, DecompositionCandidate,
    DirectInsertionCandidate, HorizontalStackCandidate, MergeTapCandidate, ModuleSizeSplit,
    NativeCandidate,
};
use super::layout::{run_layout_with_explicit_plan, LayoutOptions, LayoutStrategy, RowLayout};
use super::partitioner::PartitionPlan;

/// The density tie-break epsilon in the `equal_and_denser` arm,
/// transcribed from the former `decomposition_search.rs` `di_choice`
/// (`di_score.score > nat_score.score + 1e-9`). Parity depends on the
/// literal, so it lives next to the comparator that uses it.
pub const DENSITY_TIEBREAK_EPSILON: f64 = 1e-9;

// ---------------------------------------------------------------------
// The measurement
// ---------------------------------------------------------------------

/// The Error-only kind classes, from `decomposition_search::ErrorKinds`.
/// Which category maps to which class is [`SelectionPolicy::
/// error_kind_classes`] — a table, not a `match`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueKind {
    /// Propagates: a wrong item on a trunk poisons every downstream
    /// consumer.
    Contamination,
    /// Stays local and is recoverable (add a belt, widen a lane).
    Starvation,
    /// The blueprint does not import. Dominates lexicographically.
    Structural,
    /// A product tier has no route to its consumer: a total stop with
    /// chain-wide back-pressure, not a throttle. RFC-071 B2 (#701): the
    /// ec30 0/s regression shipped because 3 `belt-dead-end` total-stops
    /// were classed Starvation and lost to 65 `lane-throughput`
    /// throttles at equal weight. The calibration matrix's evidence
    /// table licenses the class: every route-severing category appears
    /// ONLY on rows Factorio measures as broken — zero occurrences
    /// across all 20 working factories — so any nonzero count dominates
    /// any quantity of functional throttles, below only Structural.
    RouteSevered,
}

/// Per-kind Error counts plus the lexicographic quality key the
/// merge-tap comparison ranks on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ErrorKindCounts {
    pub contamination: usize,
    pub starvation: usize,
    pub structural: usize,
    pub route_severed: usize,
}

impl ErrorKindCounts {
    /// Weighted functional total; structural and route-severed are
    /// excluded because [`Self::quality_key`] handles them
    /// lexicographically.
    pub fn weighted_functional(&self, contamination_weight: usize) -> usize {
        contamination_weight * self.contamination + self.starvation
    }

    /// Lower is better: structural dominates (an unimportable blueprint
    /// is worse than any functional defect), then route-severed (a tier
    /// with no route delivers nothing however clean the rest looks —
    /// RFC-071 B2), then the weighted functional total breaks ties.
    ///
    /// License boundary (#716 round 2): route-severed sitting ABOVE the
    /// functional total also places it above weighted CONTAMINATION,
    /// and that relative order is a design choice, not
    /// evidence-differentiated — both classes are absent from every
    /// working factory on the calibration table, and no current corpus
    /// or fixture decision hinges on route-vs-contamination. If one
    /// ever does, adjudicate it with measurements before trusting this
    /// ordering.
    pub fn quality_key(&self, contamination_weight: usize) -> (usize, usize, usize) {
        (
            self.structural,
            self.route_severed,
            self.weighted_functional(contamination_weight),
        )
    }
}

/// The three severity channels the pairwise floor protects.
///
/// **Deliberately NOT `Ord`/`PartialOrd`**, for the reason
/// `decomposition_search::IssueCounts` spells out: a derived ordering is
/// lexicographic, so `(0 err, 0 warn, 12 layout_warn) < (0, 1, 0)` would
/// be true and a 12-layout-warning regression could hide behind a
/// one-warning improvement. Each channel is a protected floor, not a
/// tiebreaker (review finding on #474).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IssueCounts {
    /// `Severity::Error` count.
    pub errors: usize,
    /// `selection_warning_count` semantics — warnings minus the policy's
    /// excluded categories.
    pub selection_warnings: usize,
    /// `LayoutResult.warnings.len()`: the second issue channel
    /// `validate()` never sees (the #462 lesson — RFC-053 produced one
    /// false "0 errors 0 warnings" claim by reading only the validator).
    pub layout_warnings: usize,
}

impl IssueCounts {
    /// No worse on ANY channel — the floor each channel is meant to be.
    pub fn no_worse_than(&self, other: &Self) -> bool {
        self.errors <= other.errors
            && self.selection_warnings <= other.selection_warnings
            && self.layout_warnings <= other.layout_warnings
    }

    /// No worse anywhere AND better somewhere. `no_worse_than` already
    /// pins every channel at `<=`, so being unequal is exactly "at least
    /// one channel is strictly better".
    pub fn strictly_better_than(&self, other: &Self) -> bool {
        self.no_worse_than(other) && self != other
    }
}

/// One candidate's complete measurement — everything any comparator
/// needs, computed from a single `validate()` call plus the layout and
/// its soft score.
///
/// Every field that a mechanism might not have computed is `Option`, and
/// an absent one is a gap (see the module doc). Phase 1b builds these
/// two ways: [`IssueProfile::measure`] from a live layout, and — in the
/// replay harness — from the `SelectionCandidateEvaluated` rows the
/// Phase-0b scoreboard recorded.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IssueProfile {
    pub outcome: Option<SelectionCandidateOutcome>,
    /// `produce()`'s own error text, or the caught-panic tag.
    pub refusal_reason: Option<String>,
    pub score: Option<f64>,
    /// The acceptance gates' verdict. `accepted == false` disqualifies
    /// the candidate from the ranked stages; it is NOT a validation
    /// verdict.
    pub accepted: Option<bool>,
    pub accepted_reason: Option<String>,
    pub counts: Option<IssueCounts>,
    pub kinds: Option<ErrorKindCounts>,
    /// RFC-071 B3: the produced layout's own warnings carry the RFC-051
    /// registry's never-verified note
    /// ([`SelectionPolicy::unverified_geometry_substring`]). Measured on
    /// the shipping path for every produced candidate; `false` for
    /// non-produced profiles and for layouts whose geometry is verified
    /// in any declared world.
    pub unverified_geometry: bool,
}

impl IssueProfile {
    /// A candidate that produced a layout. The ranked stages consider
    /// only these.
    pub fn produced(&self) -> bool {
        self.outcome == Some(SelectionCandidateOutcome::Produced)
    }

    /// `true` only when the acceptance gates positively passed — a gap
    /// is not an acceptance.
    pub fn is_accepted(&self) -> bool {
        self.accepted == Some(true)
    }

    /// The error-free tier's ordering key: selection warnings plus
    /// layout warnings (`clean_flags`' `warnings + l.warnings.len()`).
    /// `None` when no mechanism counted this candidate.
    pub fn warning_key(&self) -> Option<usize> {
        self.counts
            .map(|c| c.selection_warnings + c.layout_warnings)
    }

    /// Measure a produced layout ONCE and derive every projection.
    ///
    /// This is the whole point of the reframe: `classify_errors`,
    /// `count_issues` and `clean_flags` each run `validate()`
    /// separately in the former loop; here one call feeds the kind classes, the
    /// three severity channels and the acceptance gates.
    ///
    /// # Eager measurement changes the deciding STAGE. Read this before
    /// wiring it.
    ///
    /// RFC-070 §"Validation-once and laziness" says eager vs lazy
    /// "cannot change outcomes, only cost", because `validate()` is
    /// deterministic. That is true of the WINNER and false of the
    /// deciding STAGE — and the corpus's equivalence rule is
    /// `(status, winner, stage)`, so the difference is a divergence
    /// (#698 review round 1, absorbed as a campaign finding).
    ///
    /// The mechanism: this constructor ALWAYS fills `counts`, so the
    /// gap rule can never fire on a live profile. v1 skips
    /// `clean_flags` entirely on a single-layout solve (`n_layouts > 1`
    /// guard), leaving the error-free tier empty so `BestAccepted`
    /// decides. Measure eagerly and that same solve has counts, the
    /// tier is populated, and `BestErrorFree` decides instead — same
    /// layout, different answer to "which question was asked", which is
    /// the column K70-1 turns on. The #694 baseline has **12
    /// `best-accepted` cells**, and the RFC's own decision log
    /// identifies them as exactly this shape (cells off → only native
    /// produces → `clean_flags` skipped).
    ///
    /// So Phase 2a must either preserve the laziness as policy (skip
    /// measuring when fewer than two candidates produced) or accept
    /// those cells as MINOR divergences and adjudicate them. It is not
    /// a free implementer's choice. **Settled**: [`MeasurementRule`] is
    /// that policy and [`decide`] enforces it, so eager and lazy reach
    /// the same stage by construction (#698 review round 3). Pinned by
    /// `the_measurement_rule_makes_eager_and_lazy_decide_alike` — the
    /// round-1 test this line used to name
    /// (`eager_measurement_moves_the_deciding_stage`) was replaced by it
    /// and no longer exists.
    pub fn measure(
        layout: &LayoutResult,
        solver_result: &SolverResult,
        policy: &SelectionPolicy,
        producer: &ProducerRegistration,
    ) -> Self {
        // MUTED, for the reason v1 wraps every one of its `validate()`
        // calls in peek/truncate: `validate()` emits a
        // `ValidationCompleted` event, and a losing candidate's
        // validation leaking into the winner's replayed stream makes the
        // web timing log and the snapshot debugger report the wrong
        // layout's error counts. That is #396, which this project has
        // hit twice; a measurement helper with no emission discipline
        // would hand it to Phase 2a a third time (#698 review round 2).
        let issues = crate::trace::with_muted(|| {
            match crate::validate::validate(layout, Some(solver_result)) {
                Ok(issues) => issues,
                // `validate()` returns Err CARRYING the issues — reading
                // only `Ok` here would blank the profile of exactly the
                // candidates that failed hardest.
                Err(e) => e.issues,
            }
        });
        let mut kinds = ErrorKindCounts::default();
        let mut errors = 0usize;
        let mut selection_warnings = 0usize;
        let mut error_categories: BTreeMap<&str, usize> = BTreeMap::new();
        for i in &issues {
            match i.severity {
                crate::validate::Severity::Error => {
                    errors += 1;
                    *error_categories.entry(i.category.as_str()).or_default() += 1;
                    match policy.kind_of(&i.category) {
                        IssueKind::Contamination => kinds.contamination += 1,
                        IssueKind::Structural => kinds.structural += 1,
                        IssueKind::Starvation => kinds.starvation += 1,
                        IssueKind::RouteSevered => kinds.route_severed += 1,
                    }
                }
                crate::validate::Severity::Warning => {
                    if !policy.excluded_warning_categories.contains(&i.category) {
                        selection_warnings += 1;
                    }
                }
            }
        }
        let score = score_layout(layout, solver_result);
        let refusal = policy
            .acceptance_gates
            .iter()
            .find_map(|g| g.refusal(layout));

        // The PRODUCE-TIME gate, as a DEFENSIVE GUARD rather than a live
        // path (softened in W3a, having been written in #698 review
        // round 2 as though it were load-bearing today). All three
        // producers that carry the flag — cell-composed, direct-insertion,
        // horizontal-stack — already self-refuse inside their own
        // `produce()`, so an error-laden layout from one of them never
        // reaches this function in the first place; on the live path this
        // branch does not fire.
        //
        // It is kept because the flag would otherwise be policy data no
        // code path applies, and the failure it guards is a construction
        // mistake rather than an input: a producer registered WITH the
        // flag but WITHOUT the produce-side refusal (a new arm, or a
        // future loop that stops calling `produce()`'s self-validation)
        // would hand `decide` an error-laden `Produced` profile able to
        // displace a healthy incumbent, inverting the asymmetry
        // `refuse_on_error_is_asymmetric_and_that_asymmetry_is_load_bearing`
        // pins. The parity corpus cannot see that class because it records
        // the shipped path's final rows. See
        // [`MeasurementRule::min_produced_for_error_free_tier`], whose
        // equality with v1's `n_layouts` rests on the same pairing.
        //
        // Unlike v1, the refusal KEEPS the measurement: v1 stringifies
        // its own validation failure as `e.to_string().lines().next()`
        // and drops the issue list, so which categories fired inside a
        // self-refused candidate is invisible (Phase-0b oracle gap (d)).
        // Here the categories travel in the reason and the counts and
        // kinds stay on the profile.
        //
        // The reason STRING therefore deliberately does not match v1's
        // ("direct insertion failed validation: …"), and nothing
        // compares them: the equivalence rule is (status, winner,
        // stage), and v1's string is the lossy artifact this replaces
        // rather than a format to reproduce (#698 review round 4).
        if producer.refuse_on_error && errors > 0 {
            let breakdown = error_categories
                .iter()
                .map(|(c, n)| format!("{c}×{n}"))
                .collect::<Vec<_>>()
                .join(", ");
            let gate_note = refusal
                .map(|r| format!("; acceptance gate also: {r}"))
                .unwrap_or_default();
            return Self {
                outcome: Some(SelectionCandidateOutcome::Refused),
                refusal_reason: Some(format!(
                    "refused on {errors} error(s) [refuse_on_error]: {breakdown}{gate_note}"
                )),
                score: Some(score.score),
                // NOT `Some(..)`. A discarded layout has no acceptance
                // verdict — v1's refused rows carry `accepted: None`
                // because there is no `CandidateScore` at all — and
                // `Refused` + `accepted: true` is a contradiction a
                // naive reader (or a naive `measure -> decide` wiring)
                // would resolve the wrong way (#698 review round 3).
                // The gate's own observation is not lost; it rides in
                // the refusal reason above.
                //
                // `score` above STAYS `Some`, and the asymmetry is the
                // point (#698 review round 7): a score is a
                // MEASUREMENT, which the refusal does not invalidate —
                // the layout really did have that density and entity
                // count. `accepted` is a VERDICT about admitting the
                // layout, and there is no admitting a discarded one.
                // v1 has neither, because it never measured. So a
                // profile built here and one built from a recorded row
                // are NOT field-identical for a refused candidate
                // (#698 review round 8): compare decisions across the
                // two construction sites, never profiles.
                accepted: None,
                accepted_reason: None,
                counts: Some(IssueCounts {
                    errors,
                    selection_warnings,
                    layout_warnings: layout.warnings.len(),
                }),
                kinds: Some(kinds),
                unverified_geometry: layout
                    .warnings
                    .iter()
                    .any(|w| w.contains(policy.unverified_geometry_substring)),
            };
        }

        Self {
            outcome: Some(SelectionCandidateOutcome::Produced),
            refusal_reason: None,
            score: Some(score.score),
            accepted: Some(refusal.is_none()),
            accepted_reason: refusal,
            counts: Some(IssueCounts {
                errors,
                selection_warnings,
                layout_warnings: layout.warnings.len(),
            }),
            kinds: Some(kinds),
            unverified_geometry: layout
                .warnings
                .iter()
                .any(|w| w.contains(policy.unverified_geometry_substring)),
        }
    }
}

// ---------------------------------------------------------------------
// Policy data
// ---------------------------------------------------------------------

/// A hard constraint that disqualifies a candidate outright, regardless
/// of score. Today there is exactly one, and it reads the LAYOUT warning
/// channel by substring rather than a validator category — transcribed
/// from `validate::count_missing_balancer_template_warnings`, which
/// filters `layout.warnings` for `"balancer template"`.
#[derive(Debug, Clone)]
pub struct AcceptanceGate {
    pub name: &'static str,
    /// Disqualify when a layout warning contains this substring.
    pub layout_warning_substring: &'static str,
}

impl AcceptanceGate {
    /// `Some(reason)` when this gate disqualifies the layout.
    pub fn refusal(&self, layout: &LayoutResult) -> Option<String> {
        let n = layout
            .warnings
            .iter()
            .filter(|w| w.contains(self.layout_warning_substring))
            .count();
        (n > 0).then(|| format!("{n} {} warning(s)", self.name))
    }
}

/// A named calibration guard, carried with the receipt that says WHY it
/// exists. The #519/#520 warning-recalibration firewall is the live one:
/// it is the record of why [`SelectionPolicy::excluded_warning_categories`]
/// contains what it does, so a future reader adjusting the set meets the
/// argument instead of rediscovering it.
///
/// It is not only a record. `justifies` names the categories the receipt
/// argues for, and `firewall_receipts_cover_the_live_exclusions` pins
/// that set against the live one — so widening or narrowing the
/// exclusions without touching the argument fails a test rather than
/// leaving a receipt that describes a policy nobody kept (#698 review
/// round 4: a thing called a firewall should have an enforcement
/// surface).
#[derive(Debug, Clone)]
pub struct Firewall {
    pub name: &'static str,
    pub receipt: &'static str,
    /// The excluded categories this receipt is the argument for.
    pub justifies: &'static [&'static str],
}

/// Whether a producer is eligible on this call, and — when it is not —
/// WHICH clause excluded it. Phase-0b oracle gap (c): today's gates are
/// conjunctions of booleans at the call site, so the scoreboard can say
/// a candidate was not tried but not why. A clause list closes that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    Eligible,
    Excluded(&'static str),
}

/// What a gate clause may read. Deliberately narrow: options, the solve,
/// and the *outcomes of producers registered earlier*, keyed by
/// registration order — never a producer's name.
pub struct GateContext<'a> {
    pub solver_result: &'a SolverResult,
    pub opts: &'a LayoutOptions,
    /// Per-registration acceptance so far: `Some(accepted)` where that
    /// producer ran and produced, `None` where it produced nothing or
    /// has not run yet. Indexed by registration order.
    pub prior: &'a [Option<bool>],
    /// Index of the registration flagged [`ProducerRegistration::incumbent`].
    pub incumbent: Option<usize>,
    /// Which registration this gate is being evaluated for. Bounds
    /// [`Self::any_prior_accepted`] so the predicate matches its
    /// contract rather than merely coinciding with it while the loop
    /// happens to fill `prior` in order (#698 review round 1).
    pub registration_index: usize,
}

impl GateContext<'_> {
    /// The one rule both readers below obey: **a gate may only consult
    /// producers registered BEFORE it.** Anything at or after its own
    /// index has not run on this call, so reading it would be reading a
    /// slot that is empty for ordering reasons rather than for
    /// eligibility ones (#698 review round 4: the bound was enforced for
    /// `any_prior_accepted` and merely true-in-practice for
    /// `incumbent_accepted`).
    fn prior_slot(&self, i: usize) -> Option<bool> {
        debug_assert!(
            i < self.registration_index,
            "a gate for registration {} read slot {i}, which is not registered before it",
            self.registration_index
        );
        self.prior.get(i).copied().flatten()
    }

    /// `Some(accepted)` when the incumbent produced, `None` when it
    /// produced nothing.
    pub fn incumbent_accepted(&self) -> Option<bool> {
        self.incumbent.and_then(|i| self.prior_slot(i))
    }

    /// Whether any registration BEFORE this one produced an accepted
    /// layout. Order-sensitive by construction — registration order is
    /// policy data, and `size-split-2`'s gate ("native and k1 both
    /// failed to land an accepted layout") is exactly this predicate
    /// over the two producers registered before it.
    ///
    /// The `[..registration_index]` bound is the contract, not an
    /// optimisation: scanning the whole array would let a LATER
    /// producer's acceptance stand `size-split-2` down. That is latent
    /// today only because the loop fills `prior` in order — which is a
    /// property of the caller, and this predicate should not depend on
    /// one.
    pub fn any_prior_accepted(&self) -> bool {
        // ASSERTED, not clamped (#698 review round 2). A `min()` here
        // would silently degrade an out-of-range index back into the
        // whole-array scan this bound exists to remove — the fix would
        // hold only for callers that already got it right. `debug_assert`
        // per the module's degradation philosophy: `cargo test` and CI
        // build debug, while a release or WASM solve must not panic over
        // a code-level mistake.
        // `<`, not `<=`: valid registration indices are `0..len`, and
        // `== len` would slice the WHOLE array through the `min` below —
        // re-admitting the very scan this bound removes (#698 review
        // round 3). The `min` stays for release safety: a debug_assert
        // must not become an out-of-bounds panic in a browser solve.
        debug_assert!(
            self.registration_index < self.prior.len(),
            "registration_index {} is not a slot of the {}-slot `prior` array — this gate \
             would scan slots that do not belong to it",
            self.registration_index,
            self.prior.len()
        );
        let end = self.registration_index.min(self.prior.len());
        self.prior[..end].contains(&Some(true))
    }
}

/// One conjunct of a producer's gate, named so an exclusion is
/// reportable rather than anonymous.
#[derive(Clone, Copy)]
pub struct GateClause {
    pub name: &'static str,
    pub test: fn(&GateContext<'_>) -> bool,
}

/// A producer's eligibility gate: an ordered conjunction that reports
/// the FIRST failing clause.
#[derive(Default)]
pub struct ProducerGate {
    pub clauses: Vec<GateClause>,
}

impl ProducerGate {
    pub fn evaluate(&self, ctx: &GateContext<'_>) -> GateVerdict {
        for c in &self.clauses {
            if !(c.test)(ctx) {
                return GateVerdict::Excluded(c.name);
            }
        }
        GateVerdict::Eligible
    }
}

/// What a [`PlanProducer`] may consult when building its plan argument.
pub struct PlanContext<'a> {
    pub solver_result: &'a SolverResult,
    pub opts: &'a LayoutOptions,
    /// The incumbent's layout, when it produced one. `k1-shape-fix`
    /// needs it: its plan is derived from the unstampable shapes in the
    /// incumbent's warnings.
    pub incumbent_layout: Option<&'a LayoutResult>,
}

/// The plan-accepting producer variant (RFC-070 §`ProducerRegistration`).
///
/// `k1-shape-fix` is not a `DecompositionCandidate`: it is an inline
/// closure in `select_best_decomposition` carrying a
/// `build_k1_enrollment_plan` argument the trait signature cannot pass.
/// Rather than widening the trait for one arm, or pretending it
/// registers as a plain one, it registers as a producer that is
/// *constructed with its plan* — a two-stage contract: derive the plan
/// from the loop state, then produce with it.
pub trait PlanProducer {
    fn name(&self) -> &str;
    /// `None` = this producer has nothing to offer on this call (v1's
    /// `"no k1 enrollment"` refusal).
    fn plan(&self, ctx: &PlanContext<'_>) -> Option<PartitionPlan>;
    fn produce_with(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
        plan: &PartitionPlan,
    ) -> Result<LayoutResult, String>;
}

/// The `k1-shape-fix` arm as a [`PlanProducer`] — the same two calls the
/// v1 closure makes, in the same order.
pub struct K1ShapeFixProducer;

impl PlanProducer for K1ShapeFixProducer {
    fn name(&self) -> &str {
        "k1-shape-fix"
    }
    fn plan(&self, ctx: &PlanContext<'_>) -> Option<PartitionPlan> {
        build_k1_enrollment_plan(ctx.incumbent_layout?, ctx.solver_result, ctx.opts)
    }
    fn produce_with(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
        plan: &PartitionPlan,
    ) -> Result<LayoutResult, String> {
        run_layout_with_explicit_plan(solver_result, opts, plan)
    }
}

/// How a registration produces a layout.
pub enum ProducerBinding {
    Candidate(Box<dyn DecompositionCandidate>),
    Plan(Box<dyn PlanProducer>),
}

/// Per-producer policy. Every field here is configuration the stages may
/// read; the producer's NAME is not (see the module doc's K70-1
/// boundary) — it exists for reporting and for the registration-order
/// checks the harnesses make.
pub struct ProducerRegistration {
    pub name: &'static str,
    pub binding: ProducerBinding,
    pub gate: ProducerGate,
    /// **A PRODUCE-TIME gate, never a stage/win gate.** When true, a
    /// produced layout carrying any `Severity::Error` is DISCARDED
    /// before any stage sees it and the producer records a refusal —
    /// exactly v1's self-validation inside `produce()` ("Errors refuse;
    /// warnings pass"). When false, an error-laden layout stays in play
    /// and can win via the accepted-rank or first-produced stages: the
    /// `ec30` witness, where a layout shipping three `belt-dead-end`
    /// Errors was measured at 0.00/s in the sim.
    ///
    /// Preserving WHICH producers carry the gate is REQUIRED for parity:
    /// DI / horizontal-stack / cell-composed true, native / k1 /
    /// size-split / merge-tap false.
    pub refuse_on_error: bool,
    /// The incumbent every pairwise stage compares against.
    pub incumbent: bool,
    /// Subject to the [`AdmissionRule`]: confined to the pairwise floor
    /// stage whenever the incumbent produced.
    pub scoped: bool,
    /// Eligible for the pairwise floor's equal-issues-and-denser arm.
    /// DI true, horizontal false — RFC-060's measured call: horizontal's
    /// wins on that arm were ≤5% entity shaves on already-clean layouts
    /// and cost ten pinned structural artifacts across two suites.
    pub equal_and_denser: bool,
    /// The challenger the quality-key stage compares against the
    /// incumbent (today: merge-tap).
    pub quality_key_rival: bool,
}

impl ProducerRegistration {
    fn new(name: &'static str, binding: ProducerBinding) -> Self {
        Self {
            name,
            binding,
            gate: ProducerGate::default(),
            refuse_on_error: false,
            incumbent: false,
            scoped: false,
            equal_and_denser: false,
            quality_key_rival: false,
        }
    }
    fn gated(mut self, clauses: Vec<GateClause>) -> Self {
        self.gate = ProducerGate { clauses };
        self
    }
}

/// When a scoped producer may enter the generic ranked stages.
///
/// This is `ranking_len` (`decomposition_search.rs`) promoted to named
/// data. It is the SINGLE enforcement point for DI/horizontal
/// never-worse defaulting: when the incumbent produced, a scoped
/// candidate is confined to the pairwise floor, whose ties-to-incumbent
/// rule is the bit-identity guarantee. Letting it into a
/// density-dominated ranking there re-admits the warnings regression
/// that guarantee exists to block — the `tier2_electronic_circuit`
/// failure that predates `ranking_len`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionRule {
    /// Scoped producers are admitted iff the incumbent produced nothing.
    ScopedOnIncumbentRefusal,
    /// Everything always ranked (not the shipped policy; here so the rule is
    /// visibly a choice rather than an assumption).
    AdmitAll,
}

/// How a stage's answer interacts with the rest of the chain when the
/// comparison names the INCUMBENT rather than a challenger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainBehavior {
    /// The incumbent wins here and the chain stops.
    Terminate,
    /// **The non-shadowing rule** (#474). The incumbent win is held, not
    /// returned: the chain continues through the remaining PAIRWISE
    /// stages, any of which may displace it. If none does, the held
    /// answer stands with this stage's tag and the chain stops — it does
    /// not fall through to the ranked stages.
    ///
    /// A plain `.or()` chain got this wrong: `merge_tap_choice` is
    /// `Some` even when it means "native beat merge-tap", so it
    /// short-circuited DI's already-computed, already-validated result
    /// This was measured live on `electronic-circuit@35/s` from ore.
    DeferToRemainingPairwiseStages,
}

/// Which comparator a stage runs. These are RFC-070's three, and the
/// tiered rank covers three of the five stages by configuration.
pub enum StageKind {
    /// Lexicographic [`ErrorKindCounts::quality_key`]: the registration
    /// flagged `quality_key_rival` against the incumbent. Ties favour
    /// the incumbent. Deliberately NOT the accepted-by-score path — an
    /// accepted challenger that is worse by KIND still loses.
    QualityKeyPairwise,
    /// The component-wise [`IssueCounts`] floor: every `scoped`
    /// registration against the incumbent, then against each other.
    /// Ties → the incumbent, then → earlier registration.
    ComponentWiseFloor,
    /// The tiered rank over the admitted slice.
    TieredRank(RankSpec),
}

/// Ordering within a ranked tier. Ties always resolve to the earliest
/// registration — the array is a PREFERENCE order (incumbent first), and
/// a tie-break pointing the other way silently breaks the additive
/// contract (#384 review finding 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankOrder {
    /// Highest score first.
    ScoreDesc,
    /// Quieter first, then denser: warnings ascending, then score
    /// descending. RFC-060's refusal-path order — horizontal's denser
    /// 0-error/6-warning `ec@15` must not outrank DI's genuinely clean
    /// 0/0 resolution.
    WarningsAscThenScoreDesc,
    /// Registration order alone.
    RegistrationOrder,
}

/// A ranked stage's admission predicate and its two orderings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankSpec {
    pub require_accepted: bool,
    /// Requires measured counts with `errors == 0`. A candidate with no
    /// counts is a GAP and is skipped, which is what reproduces v1's
    /// lazy `clean_flags` (never computed when a pairwise stage already
    /// decided, or when only one candidate produced).
    pub require_error_free: bool,
    /// Ordering when the incumbent produced a layout.
    pub success_order: RankOrder,
    /// Ordering when the incumbent produced nothing. RFC-060 scoped its
    /// warnings-first order to this path so every success-path selection
    /// stayed bit-identical to pre-RFC-060 behavior.
    pub refusal_order: RankOrder,
    /// RFC-071 B3 (#700): when true, a sim-verified geometry outranks an
    /// unverified one REGARDLESS of the configured order — the order
    /// only breaks ties within the same verification standing. gear@20's
    /// winner declared "geometry NOT sim-verified" in its own warnings
    /// and `best-error-free` ranked past it on score, shipping 75% of
    /// plan for a month. Deliberately an ordering rule and not a
    /// refusal: an unverified candidate that is the ONLY member of its
    /// tier still wins (the rescue class — cells fixing an error-laden
    /// native — which a produce-time refusal demonstrably re-broke).
    pub verified_geometry_first: bool,
}

/// One stage of the program: its trace tag, its comparator, and its
/// chain behavior.
pub struct StageSpec {
    pub tag: SelectionStage,
    pub kind: StageKind,
    pub on_incumbent_win: ChainBehavior,
}

impl StageSpec {
    /// Pairwise stages are the ones a deferred incumbent answer waits
    /// on (see [`ChainBehavior::DeferToRemainingPairwiseStages`]).
    fn is_pairwise(&self) -> bool {
        matches!(
            self.kind,
            StageKind::QualityKeyPairwise | StageKind::ComponentWiseFloor
        )
    }
}

/// When the error-free tier is measured at all — v1's `clean_flags`
/// laziness, promoted from an implementation accident to policy.
///
/// v1 skips `clean_flags` entirely below two produced layouts
/// (`n_layouts > 1`), so a single-layout solve has an empty error-free
/// tier and falls to `BestAccepted`. That is the shape of the 12
/// `best-accepted` cells in the #694 baseline.
///
/// Left implicit, this only held because the RECORDER happened not to
/// compute counts there; an eagerly-measured v2 would populate the tier
/// and decide the same layout at `BestErrorFree` instead — a stage
/// divergence on 12 cells, which the equivalence rule counts (#698
/// review rounds 2 and 3). [`decide`] therefore ENFORCES this rule
/// rather than trusting whoever built the profiles, so eager and lazy
/// measurement reach the same answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasurementRule {
    /// Below this many produced candidates, error-free admission is not
    /// evaluated — the tier is empty regardless of what was measured.
    /// v1: 2.
    ///
    /// **What "produced" has to mean for this to equal v1's
    /// `n_layouts`** (#698 review round 4, the forward-looking one):
    /// v1 counts candidates whose `produce()` returned `Ok`, and its
    /// DI / horizontal / cell-composed arms self-refuse on error INSIDE
    /// `produce()` — so an error-laden one of those has no outcome and
    /// is not counted. The equality therefore rests on
    /// [`ProducerRegistration::refuse_on_error`] being applied BEFORE
    /// the count, which is why [`IssueProfile::measure`] applies it. A
    /// producer given the flag here but no equivalent self-refusal on
    /// the produce side would be counted as produced by v1 and refused
    /// by v2, moving the error-free tier's availability — and
    /// the committed parity corpus would not see it, because it reads the
    /// shipped path's final rows rather than bypassing `measure`.
    pub min_produced_for_error_free_tier: usize,
}

/// The precedence chain as data.
pub struct SelectionProgram {
    pub stages: Vec<StageSpec>,
    pub admission: AdmissionRule,
    pub measurement: MeasurementRule,
}

/// Everything selection needs to know, in one place.
pub struct SelectionPolicy {
    /// Warning categories that do not participate in selection.
    /// **Today this set is `belt-detour` alone.** The two #632 B6
    /// demotions left it by DELETION (#684) — the checks themselves are
    /// gone — so the set is one entry, not three.
    pub excluded_warning_categories: BTreeSet<String>,
    /// Category → Error kind class. Categories absent from the table are
    /// [`IssueKind::Starvation`], matching the `_ =>` arm of
    /// `classify_errors`.
    pub error_kind_classes: BTreeMap<String, IssueKind>,
    pub acceptance_gates: Vec<AcceptanceGate>,
    /// RFC-071 B3: the substring of the RFC-051 registry's no-match
    /// verification note. The shipping loop measures each produced
    /// layout's warnings against it into
    /// [`IssueProfile::unverified_geometry`], which
    /// [`RankSpec::verified_geometry_first`] consumes. See the long
    /// note at the `current()` value.
    pub unverified_geometry_substring: &'static str,
    pub contamination_weight: usize,
    pub firewalls: Vec<Firewall>,
    pub program: SelectionProgram,
    pub producers: Vec<ProducerRegistration>,
}

impl SelectionPolicy {
    pub fn kind_of(&self, category: &str) -> IssueKind {
        self.error_kind_classes
            .get(category)
            .copied()
            .unwrap_or(IssueKind::Starvation)
    }

    /// Index of the incumbent registration.
    ///
    /// **Exactly one** registration must carry the flag. Two would
    /// silently break every pairwise stage — `position()` takes the
    /// first and the second's profile would be ranked as an ordinary
    /// challenger against it. ZERO is equally malformed and less
    /// obviously so: the two pairwise stages disagree about what it
    /// means, the quality-key one handing its rival an unconditional
    /// win while the floor abstains forever (#698 review rounds 4 and
    /// 8). The return stays `Option` because release must degrade
    /// rather than panic, not because zero is a supported shape.
    pub fn incumbent_index(&self) -> Option<usize> {
        debug_assert_eq!(
            self.producers.iter().filter(|p| p.incumbent).count(),
            1,
            "a policy declares exactly one incumbent"
        );
        self.producers.iter().position(|p| p.incumbent)
    }

    /// **The shipped policy**, transcribed from the source sites RFC-070's
    /// Phase 1b specification anchors. Every value here is a
    /// transcription, not a redesign: the committed parity corpus is the
    /// external proof, and a wrong transcription shows up as a diverging
    /// cell rather than as a plausible-looking constant.
    pub fn current() -> Self {
        let excluded_warning_categories = crate::validate::SELECTION_EXCLUDED_WARNING_CATEGORIES
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        // `classify_errors`' classification, as a table — built from the
        // SAME lists that function reads, not re-typed beside them
        // (#698 review round 2). A category added there now appears
        // here; re-typing would have let it fall silently to Starvation.
        let mut error_kind_classes = BTreeMap::new();
        for c in super::decomposition_search::CONTAMINATION_CATEGORIES {
            error_kind_classes.insert(c.to_string(), IssueKind::Contamination);
        }
        for c in super::decomposition_search::STRUCTURAL_CATEGORIES {
            error_kind_classes.insert(c.to_string(), IssueKind::Structural);
        }
        for c in super::decomposition_search::ROUTE_SEVERING_CATEGORIES {
            error_kind_classes.insert(c.to_string(), IssueKind::RouteSevered);
        }

        Self {
            excluded_warning_categories,
            error_kind_classes,
            acceptance_gates: vec![AcceptanceGate {
                name: "missing-balancer-template",
                layout_warning_substring: "balancer template",
            }],
            // RFC-071 B3 (#700): the substring of the RFC-051 registry's
            // no-match verification note. A produced layout whose
            // warnings carry it is measured `unverified_geometry`, and
            // the best-error-free tier ranks verified geometries ahead
            // of unverified ones regardless of score
            // (`RankSpec::verified_geometry_first`) — gear@20 shipped at
            // 75% for a month with this exact flag present and no stage
            // able to read it. Deliberately NOT an acceptance gate:
            // refusal re-ships a broken native in the rescue class
            // (cells fixing an error-laden incumbent), which the suite
            // demonstrated — an unverified rescue still wins when it is
            // the only error-free candidate. Matches ONLY the
            // never-verified tier; a world-mismatch note ("do NOT
            // transfer across worlds") stays rankable as verified.
            unverified_geometry_substring: "geometry NOT sim-verified",
            // Sourced from the constant the live mechanism reads, not
            // re-typed beside it: a second definition is the
            // "two values that disagree" class, and a unit test pinning
            // a magic literal only notices after they have already
            // diverged (#698 review round 1).
            contamination_weight: super::decomposition_search::KIND_CONTAMINATION_WEIGHT,
            firewalls: vec![Firewall {
                name: "warning-recalibration-firewall",
                receipt: "#519/#520: the recalibration multiplied input-rate-delivery's \
                          counts ~10x and letting an unanchored model steer selection \
                          shipped a physically over-stamped winner on stacking_ec_60s. \
                          The input-rate-delivery exemption was LIFTED 2026-08-07 (it \
                          counts again); belt-detour remains excluded. The two #632 B6 \
                          demotions left the set by DELETION — PR #684 removed the \
                          inserter-throughput check pair, under the #675 off-path \
                          campaign's Tier 2 item 9 (both numbers name the same event: \
                          the PR that did it and the issue that tracked it). Receipts: \
                          docs/validator-trust.md hole 2.",
                justifies: &["belt-detour"],
            }],
            program: current_program(),
            producers: current_producers(),
        }
    }
}

/// The shipped five stages, in policy order.
fn current_program() -> SelectionProgram {
    SelectionProgram {
        admission: AdmissionRule::ScopedOnIncumbentRefusal,
        // Former `clean_flags` guard: `n_layouts > 1`.
        measurement: MeasurementRule {
            min_produced_for_error_free_tier: 2,
        },
        stages: vec![
            StageSpec {
                tag: SelectionStage::MergeTap,
                kind: StageKind::QualityKeyPairwise,
                on_incumbent_win: ChainBehavior::DeferToRemainingPairwiseStages,
            },
            StageSpec {
                tag: SelectionStage::ScopedPairwise,
                kind: StageKind::ComponentWiseFloor,
                // Unreachable by construction: the floor never names the
                // incumbent (v1's `di_choice` returns `Some(DI)` or
                // `None`, never `Some(native)`) — returning the
                // incumbent would short-circuit the ranked stages and
                // rob k1-shape-fix or cell-composed of a legitimate win.
                on_incumbent_win: ChainBehavior::Terminate,
            },
            StageSpec {
                tag: SelectionStage::BestErrorFree,
                kind: StageKind::TieredRank(RankSpec {
                    require_accepted: true,
                    require_error_free: true,
                    success_order: RankOrder::ScoreDesc,
                    refusal_order: RankOrder::WarningsAscThenScoreDesc,
                    // RFC-071 B3: within the error-free tier, verified
                    // geometry outranks score (#700).
                    verified_geometry_first: true,
                }),
                on_incumbent_win: ChainBehavior::Terminate,
            },
            StageSpec {
                tag: SelectionStage::BestAccepted,
                kind: StageKind::TieredRank(RankSpec {
                    require_accepted: true,
                    require_error_free: false,
                    success_order: RankOrder::ScoreDesc,
                    refusal_order: RankOrder::ScoreDesc,
                    verified_geometry_first: false,
                }),
                on_incumbent_win: ChainBehavior::Terminate,
            },
            StageSpec {
                // The degraded fallback, and the pre-#701 `ec30` trap #694
                // measured: an error-laden best SHIPS rather than the
                // solve refusing. v2 reproduces it bit-for-bit under
                // parity; changing it is Phase-3 calibration work, not
                // migration work.
                tag: SelectionStage::FirstProduced,
                kind: StageKind::TieredRank(RankSpec {
                    require_accepted: false,
                    require_error_free: false,
                    success_order: RankOrder::RegistrationOrder,
                    refusal_order: RankOrder::RegistrationOrder,
                    verified_geometry_first: false,
                }),
                on_incumbent_win: ChainBehavior::Terminate,
            },
        ],
    }
}

/// The seven producers, in the canonical order every index-keyed
/// structure in `select_best_decomposition` uses.
fn current_producers() -> Vec<ProducerRegistration> {
    let partitioned = GateClause {
        name: "strategy-is-partitioned-decomposed",
        test: |c| matches!(c.opts.strategy, LayoutStrategy::PartitionedDecomposed),
    };
    let incumbent_unaccepted = GateClause {
        name: "incumbent-produced-and-unaccepted",
        test: |c| c.incumbent_accepted() == Some(false),
    };
    // Both cell-composed and horizontal-stack stand down under Forced
    // DI: it is an explicit topology request (the A/B debug control) and
    // a competing variant must not displace it.
    let di_not_forced = GateClause {
        name: "direct-insertion-not-forced",
        test: |c| c.opts.direct_insertion != super::di_cell::DirectInsertion::Forced,
    };

    let mut native = ProducerRegistration::new(
        "native",
        ProducerBinding::Candidate(Box::new(NativeCandidate)),
    );
    native.incumbent = true;

    let k1 = ProducerRegistration::new(
        "k1-shape-fix",
        ProducerBinding::Plan(Box::new(K1ShapeFixProducer)),
    )
    .gated(vec![partitioned, incumbent_unaccepted]);

    let split = ProducerRegistration::new(
        "size-split-2",
        ProducerBinding::Candidate(Box::new(ModuleSizeSplit { k: 2 })),
    )
    .gated(vec![
        partitioned,
        GateClause {
            // v1: native `is_none_or(!accepted)` AND k1 `is_none_or
            // (!accepted)` — i.e. nothing registered earlier landed an
            // accepted layout.
            name: "no-earlier-producer-accepted",
            test: |c| !c.any_prior_accepted(),
        },
    ]);

    let mut merge_tap = ProducerRegistration::new(
        "merge-tap",
        ProducerBinding::Candidate(Box::new(MergeTapCandidate)),
    )
    .gated(vec![
        GateClause {
            name: "strategy-is-pooled",
            test: |c| matches!(c.opts.strategy, LayoutStrategy::Pooled),
        },
        incumbent_unaccepted,
    ]);
    merge_tap.quality_key_rival = true;

    let mut cells = ProducerRegistration::new(
        "cell-composed",
        ProducerBinding::Candidate(Box::new(CellComposedCandidate)),
    )
    .gated(vec![
        GateClause {
            name: "cell-composition-is-candidate",
            test: |c| c.opts.cell_composition == super::cells::CellComposition::Candidate,
        },
        di_not_forced,
        GateClause {
            name: "belt-tier-unconstrained-or-express",
            test: |c| {
                c.opts
                    .max_belt_tier
                    .as_deref()
                    .is_none_or(|t| t == "express-transport-belt")
            },
        },
        GateClause {
            name: "chain-eligible",
            test: |c| super::cells::chain::chain_eligible(c.solver_result).is_ok(),
        },
    ]);
    cells.refuse_on_error = true;

    let mut di = ProducerRegistration::new(
        "direct-insertion",
        ProducerBinding::Candidate(Box::new(DirectInsertionCandidate)),
    )
    .gated(vec![
        GateClause {
            name: "direct-insertion-is-candidate",
            test: |c| c.opts.direct_insertion == super::di_cell::DirectInsertion::Candidate,
        },
        GateClause {
            name: "solve-has-di-couplings",
            test: |c| !c.solver_result.di_couplings.is_empty(),
        },
    ]);
    di.refuse_on_error = true;
    di.scoped = true;
    di.equal_and_denser = true;

    let mut horizontal = ProducerRegistration::new(
        "horizontal-stack",
        ProducerBinding::Candidate(Box::new(HorizontalStackCandidate)),
    )
    .gated(vec![
        GateClause {
            name: "horizontal-candidate-enabled",
            test: |c| c.opts.horizontal_candidate,
        },
        GateClause {
            name: "row-layout-is-vertical-split",
            test: |c| matches!(c.opts.row_layout, RowLayout::VerticalSplit),
        },
        di_not_forced,
        GateClause {
            name: "solve-has-dual-input-row",
            test: |c| super::placer::any_dual_input_row(&c.solver_result.machines),
        },
    ]);
    horizontal.refuse_on_error = true;
    horizontal.scoped = true;

    vec![native, k1, split, merge_tap, cells, di, horizontal]
}

// ---------------------------------------------------------------------
// The decision
// ---------------------------------------------------------------------

/// A selection outcome: which registration won, and which stage said so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    pub winner: usize,
    pub stage: SelectionStage,
}

/// What one stage concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StageOutcome {
    /// This candidate wins; the chain stops.
    Winner(usize),
    /// The incumbent won a pairwise comparison, under
    /// [`ChainBehavior::DeferToRemainingPairwiseStages`].
    HeldIncumbent(usize),
    /// This stage has nothing to say (inputs absent, or no candidate
    /// qualified); the chain continues.
    NoOpinion,
}

// K70-1-FENCE-BEGIN
// Everything from here to K70-1-FENCE-END is STAGE LOGIC. It may read
// `ProducerRegistration` FIELDS and `IssueProfile`s; it may not read or
// compare producer NAMES. `k70_1_fence_holds` asserts that mechanically.
// If a decision cannot be reproduced without naming a candidate here,
// that is K70-1 firing — stop and report it, do not widen this fence.

/// Run the program over one selection's profiles.
///
/// `profiles[i]` is the measurement of `policy.producers[i]`; a producer
/// that never ran carries a profile with `outcome: NotRun` (or `None`).
/// Returns `None` when no stage names a winner — v1's
/// all-candidates-failed path, which emits the scoreboard and then
/// returns `Err` with no `SelectionDecided` at all.
///
/// # Precondition: gaps must be COHERENT, and that is checked at the
/// boundary, not here
///
/// Equivalence to v1 holds for profiles that respect the recorder's own
/// invariant: **a projection is present for every candidate a mechanism
/// examined, and absent for every candidate it did not** — kinds for
/// both sides of the quality-key comparison or neither, counts for all
/// three channels or none. Hand a profile a half-gap (a produced
/// quality-key rival with no kinds, say) and stages that would have
/// decided in v1 will skip here.
///
/// Three review rounds asked for `debug_assert`s on those states inside
/// the stages. They live at the BOUNDARY instead
/// (the scoreboard recorder rejects a partial count or kind triple),
/// for two reasons: this function's stated rule is that a gap
/// SKIPS — a panic path for one gap and a skip for the next would make
/// the rule unstatable — and a pure decision function is the wrong place
/// to validate data it did not build. Untrusted profiles get checked
/// where they are constructed; `decide` is total over whatever it is
/// handed.
pub fn decide(profiles: &[IssueProfile], policy: &SelectionPolicy) -> Option<Decision> {
    // `debug_assert` for the message, and a REFUSAL rather than a
    // best-effort answer in release (#698 review round 3). The module's
    // degradation philosophy says a code-level mistake must not panic a
    // browser solve; it does not say the mistake should produce a
    // plausible wrong winner, which a longer-than-expected slice would
    // (the extra profiles rank under other producers' policy). Deciding
    // nothing is the honest degradation.
    debug_assert_eq!(
        profiles.len(),
        policy.producers.len(),
        "one profile per registration: the profile vector is keyed by registration order, \
         so a length mismatch would rank one producer's measurement under another's policy"
    );
    if profiles.len() != policy.producers.len() {
        return None;
    }

    // At most one stage may defer. The chain holds ONE deferred answer
    // and a second deferring stage would overwrite the first with
    // nothing noticing — the same policy-authoring class as two
    // incumbents, so it gets the same treatment rather than a comment
    // saying it would need thought (#698 review round 7).
    debug_assert!(
        policy
            .program
            .stages
            .iter()
            .filter(|s| s.on_incumbent_win == ChainBehavior::DeferToRemainingPairwiseStages)
            .count()
            <= 1,
        "a program may declare at most one deferring stage; this one declares several, \
         and the chain holds only one held answer"
    );

    let incumbent = policy.incumbent_index();
    let incumbent_produced = incumbent.is_some_and(|i| profiles[i].produced());
    let produced_count = profiles.iter().filter(|p| p.produced()).count();

    // The AdmissionRule, applied once: which registrations the ranked
    // stages may consider at all.
    let admitted: Vec<usize> = (0..policy.producers.len())
        .filter(|&i| match policy.program.admission {
            AdmissionRule::AdmitAll => true,
            AdmissionRule::ScopedOnIncumbentRefusal => {
                !policy.producers[i].scoped || !incumbent_produced
            }
        })
        .collect();

    let mut held: Option<Decision> = None;
    for stage in &policy.program.stages {
        // A held incumbent answer waits only on the remaining PAIRWISE
        // stages; reaching a ranked one means nothing displaced it, so
        // it stands and the chain stops here.
        if held.is_some() && !stage.is_pairwise() {
            return held;
        }
        let outcome = match &stage.kind {
            StageKind::QualityKeyPairwise => quality_key_stage(profiles, policy, incumbent),
            StageKind::ComponentWiseFloor => {
                component_wise_floor_stage(profiles, policy, incumbent)
            }
            StageKind::TieredRank(spec) => tiered_rank_stage(
                profiles,
                spec,
                &admitted,
                incumbent_produced,
                // The measurement rule, enforced here rather than
                // assumed of the profile builder.
                produced_count >= policy.program.measurement.min_produced_for_error_free_tier,
            ),
        };
        match outcome {
            StageOutcome::Winner(i) => {
                return Some(Decision {
                    winner: i,
                    stage: stage.tag,
                })
            }
            StageOutcome::HeldIncumbent(i) => match stage.on_incumbent_win {
                ChainBehavior::Terminate => {
                    return Some(Decision {
                        winner: i,
                        stage: stage.tag,
                    })
                }
                ChainBehavior::DeferToRemainingPairwiseStages => {
                    // Last defer wins. Today's program has exactly one
                    // deferring stage, so this cannot overwrite; a
                    // second one would need its own precedence rule
                    // rather than inheriting this one silently.
                    held = Some(Decision {
                        winner: i,
                        stage: stage.tag,
                    });
                }
            },
            StageOutcome::NoOpinion => {}
        }
    }
    // Program exhausted. UNREACHABLE under today's program, whose last
    // stage is ranked, not pairwise — so a held answer has already
    // returned at the loop head. It is `held` rather than `None` because
    // a program ending in a pairwise stage should still honour a
    // deferral, not discard it (#698 review round 6: the prose used to
    // imply this line does work today).
    held
}

/// The quality-key lexicograph: the `quality_key_rival` registration
/// against the incumbent, ties → incumbent.
fn quality_key_stage(
    profiles: &[IssueProfile],
    policy: &SelectionPolicy,
    incumbent: Option<usize>,
) -> StageOutcome {
    let Some(rival) = policy.producers.iter().position(|p| p.quality_key_rival) else {
        return StageOutcome::NoOpinion;
    };
    if !profiles[rival].produced() {
        return StageOutcome::NoOpinion;
    }
    let Some(rival_kinds) = profiles[rival].kinds else {
        // Gap: the rival produced but nothing classified it. Skip rather
        // than treat an unmeasured layout as flawless.
        return StageOutcome::NoOpinion;
    };
    let w = policy.contamination_weight;
    match incumbent.filter(|&i| profiles[i].produced()) {
        // The incumbent produced nothing, so the rival is the only
        // layout there is.
        //
        // KEPT despite being unreachable under today's registrations —
        // the rival's own gate requires the incumbent to have produced,
        // so it cannot itself produce when the incumbent refused. This
        // is a faithful transcription of v1's `merge_tap_choice` arm,
        // which carries the same unreachability note at its own site,
        // and deleting it would leave this stage undefined in a state
        // v1 answers. The floor stage's opposite convention (no
        // opinion when the incumbent refused) is likewise v1's, from
        // `di_choice`'s early return — two mechanisms, deliberately
        // different, not an inconsistency to reconcile (#698 review
        // round 2, half-refuted).
        None => StageOutcome::Winner(rival),
        Some(inc) => match profiles[inc].kinds {
            // Unreachable against today's recorder, which classifies
            // both sides in one place or neither. Skipping is the safe
            // reading of a gap either way — see the module doc.
            None => StageOutcome::NoOpinion,
            Some(inc_kinds) if rival_kinds.quality_key(w) < inc_kinds.quality_key(w) => {
                StageOutcome::Winner(rival)
            }
            Some(_) => StageOutcome::HeldIncumbent(inc),
        },
    }
}

/// The component-wise floor: every `scoped` registration against the
/// incumbent, then the survivors against each other.
///
/// Never names the incumbent — a scoped candidate that does not strictly
/// improve on it leaves the layout bit-identical, which is the whole
/// never-worse safety argument, and answering "incumbent" here would
/// short-circuit the ranked stages.
fn component_wise_floor_stage(
    profiles: &[IssueProfile],
    policy: &SelectionPolicy,
    incumbent: Option<usize>,
) -> StageOutcome {
    let Some(inc) = incumbent.filter(|&i| profiles[i].produced()) else {
        // The incumbent produced nothing: the AdmissionRule has already
        // let the scoped candidates into the ranked stages, where they
        // compete on the merits instead of auto-winning a refusal.
        return StageOutcome::NoOpinion;
    };
    let Some(inc_counts) = profiles[inc].counts else {
        return StageOutcome::NoOpinion;
    };
    let inc_score = profiles[inc].score;

    let mut best: Option<(usize, IssueCounts)> = None;
    for (i, reg) in policy.producers.iter().enumerate() {
        if !reg.scoped || !profiles[i].produced() {
            continue;
        }
        // `accepted` is a hard constraint the issue channels cannot see:
        // an unaccepted challenger must never displace the incumbent.
        if !profiles[i].is_accepted() {
            continue;
        }
        let Some(counts) = profiles[i].counts else {
            continue;
        };
        let strictly_better = counts.strictly_better_than(&inc_counts);
        let equal_and_denser = reg.equal_and_denser
            && counts == inc_counts
            && match (profiles[i].score, inc_score) {
                (Some(a), Some(b)) => a > b + DENSITY_TIEBREAK_EPSILON,
                _ => false,
            };
        if !(strictly_better || equal_and_denser) {
            continue;
        }
        best = Some(match best {
            None => (i, counts),
            // Same floor rule between two winners, ties → the earlier
            // registration. Deliberately not a three-way score ranking:
            // neither scoped candidate rides the soft score against the
            // incumbent, so it must not decide between them either.
            Some((_, best_counts)) if counts.strictly_better_than(&best_counts) => (i, counts),
            Some(prev) => prev,
        });
    }
    best.map(|(i, _)| StageOutcome::Winner(i))
        .unwrap_or(StageOutcome::NoOpinion)
}

/// The tiered rank: admission by `spec`, ordering by whether the
/// incumbent produced.
fn tiered_rank_stage(
    profiles: &[IssueProfile],
    spec: &RankSpec,
    admitted: &[usize],
    incumbent_produced: bool,
    error_free_tier_measured: bool,
) -> StageOutcome {
    // The error-free tier is not evaluated at all below the measurement
    // rule's threshold — v1's `clean_flags = [None; 7]`, as policy. Not
    // a shortcut: it is what makes an eagerly-measured profile decide
    // the same stage as a lazily-measured one.
    if spec.require_error_free && !error_free_tier_measured {
        return StageOutcome::NoOpinion;
    }
    let order = if incumbent_produced {
        spec.success_order
    } else {
        spec.refusal_order
    };
    let mut best: Option<usize> = None;
    for &i in admitted {
        let p = &profiles[i];
        if !p.produced() {
            continue;
        }
        if spec.require_accepted && !p.is_accepted() {
            continue;
        }
        if spec.require_error_free {
            // A gap skips: no mechanism measured this candidate, and an
            // unmeasured layout is not an error-free one.
            match p.counts {
                Some(c) if c.errors == 0 => {}
                _ => continue,
            }
        }
        best = Some(match best {
            None => i,
            // RFC-071 B3: verification standing outranks the configured
            // order when the spec asks for it — a sim-verified geometry
            // beats an unverified one regardless of score, and the order
            // below only decides WITHIN a standing (#700: the gear@20
            // winner carried its own "NOT sim-verified" flag and the
            // score ranked past it).
            Some(b)
                if spec.verified_geometry_first
                    && profiles[i].unverified_geometry != profiles[b].unverified_geometry =>
            {
                if profiles[b].unverified_geometry {
                    i
                } else {
                    b
                }
            }
            // Strictly-better-only, so an exact tie keeps the EARLIER
            // registration: the list is a preference order.
            Some(b) if ranks_ahead(profiles, i, b, order) => i,
            Some(b) => b,
        });
    }
    best.map(StageOutcome::Winner)
        .unwrap_or(StageOutcome::NoOpinion)
}

/// Does `a` rank strictly ahead of `b` under `order`?
fn ranks_ahead(profiles: &[IssueProfile], a: usize, b: usize, order: RankOrder) -> bool {
    let score = |i: usize| profiles[i].score.unwrap_or(f64::NEG_INFINITY);
    // No `usize::MAX` sentinel for a missing warning key: see the
    // warnings-first arm below, which abstains instead.
    let score_ahead = || score(a).partial_cmp(&score(b)) == Some(std::cmp::Ordering::Greater);
    match order {
        RankOrder::RegistrationOrder => false,
        RankOrder::ScoreDesc => score_ahead(),
        RankOrder::WarningsAscThenScoreDesc => {
            // A missing warning key ABSTAINS — `false`, "does not rank
            // ahead" — rather than sorting last through a sentinel OR
            // falling through to the score. Both alternatives answer a
            // question the module says is unanswerable: this order's
            // PRIMARY criterion is the warning key, and a gap means no
            // mechanism computed one, so there is nothing to compare
            // first. Falling through to score would silently promote the
            // SECONDARY criterion to primary for exactly the candidates
            // least is known about, which is the `unwrap_or(0)` trap
            // wearing a different hat (#698 review round 6 removed the
            // sentinel; the fall-through it left behind is this
            // carry-over, W3a).
            //
            // Unreachable under today's program — the error-free tier's
            // admission already requires counts, so both keys are always
            // present here. The point is that a future
            // `require_error_free: false` stage using this order cannot
            // quietly turn "unmeasured" into either "worst" or
            // "ranked on density alone". Pinned by
            // `a_missing_warning_key_abstains_rather_than_ranking_on_score`.
            //
            // The consequence, by design and worth naming (#703 review
            // round 2 nit): under such a stage, an unmeasured candidate
            // registered EARLIER becomes an immovable floor — nothing
            // ranks ahead of it, because every comparison against it
            // abstains. That is the correct reading of "the primary
            // criterion is unanswerable" combined with "ties keep the
            // earlier registration"; a stage that wants unmeasured
            // candidates ranked must either measure them or declare a
            // different `RankOrder`, not lean on this arm to invent an
            // ordering.
            match (profiles[a].warning_key(), profiles[b].warning_key()) {
                (Some(wa), Some(wb)) => match wa.cmp(&wb) {
                    std::cmp::Ordering::Less => true,
                    std::cmp::Ordering::Greater => false,
                    std::cmp::Ordering::Equal => score_ahead(),
                },
                _ => false,
            }
        }
    }
}

// K70-1-FENCE-END

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical order, spelled longhand. A list that reads the
    /// thing it checks cannot detect a wrong reorder.
    const EXPECTED_ORDER: [&str; 7] = [
        "native",
        "k1-shape-fix",
        "size-split-2",
        "merge-tap",
        "cell-composed",
        "direct-insertion",
        "horizontal-stack",
    ];

    fn produced(score: f64, accepted: bool) -> IssueProfile {
        IssueProfile {
            outcome: Some(SelectionCandidateOutcome::Produced),
            score: Some(score),
            accepted: Some(accepted),
            ..Default::default()
        }
    }

    fn not_run() -> IssueProfile {
        IssueProfile {
            outcome: Some(SelectionCandidateOutcome::NotRun),
            ..Default::default()
        }
    }

    fn counts(errors: usize, selection_warnings: usize, layout_warnings: usize) -> IssueCounts {
        IssueCounts {
            errors,
            selection_warnings,
            layout_warnings,
        }
    }

    fn kinds(contamination: usize, starvation: usize, structural: usize) -> ErrorKindCounts {
        ErrorKindCounts {
            contamination,
            starvation,
            structural,
            route_severed: 0,
        }
    }

    /// Seven `not-run` profiles, to be filled in per test.
    fn blank() -> Vec<IssueProfile> {
        (0..7).map(|_| not_run()).collect()
    }

    fn quality_key_decision(
        native: ErrorKindCounts,
        merge_tap: ErrorKindCounts,
        contamination_weight: usize,
    ) -> Decision {
        let mut profiles = blank();
        profiles[NATIVE] = produced(1.0, false);
        profiles[NATIVE].kinds = Some(native);
        profiles[MERGE_TAP] = produced(1.0, true);
        profiles[MERGE_TAP].kinds = Some(merge_tap);

        let mut policy = SelectionPolicy::current();
        policy.contamination_weight = contamination_weight;
        decide(&profiles, &policy).expect("native and merge-tap both produced")
    }

    const NATIVE: usize = 0;
    const K1: usize = 1;
    const SPLIT: usize = 2;
    const MERGE_TAP: usize = 3;
    const CELLS: usize = 4;
    const DI: usize = 5;
    const HS: usize = 6;

    // -----------------------------------------------------------------
    // Registration data
    // -----------------------------------------------------------------

    #[test]
    fn current_policy_registers_the_seven_in_canonical_order() {
        let p = SelectionPolicy::current();
        let names: Vec<&str> = p.producers.iter().map(|r| r.name).collect();
        assert_eq!(names, EXPECTED_ORDER);
        assert_eq!(p.incumbent_index(), Some(NATIVE));
    }

    /// RFC-071 B3 (#700): within the error-free tier, a sim-verified
    /// geometry outranks an unverified one regardless of score. The
    /// gear@20 mechanism: an unverified cells layout with the BETTER
    /// score must lose to a verified error-free rival.
    #[test]
    fn verified_geometry_outranks_score_in_the_error_free_tier() {
        let mut ps = blank();
        // Incumbent native: error-free, verified (no flag), lower score.
        ps[NATIVE] = produced(0.5, true);
        ps[NATIVE].counts = Some(counts(0, 0, 0));
        // Cells: error-free, HIGHER score, but its geometry is
        // unverified.
        ps[CELLS] = produced(2.0, true);
        ps[CELLS].counts = Some(counts(0, 0, 1));
        ps[CELLS].unverified_geometry = true;
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d.winner, NATIVE,
            "an unverified geometry must not displace a verified error-free rival on score"
        );
    }

    /// The rescue class stays intact: an unverified geometry that is the
    /// ONLY error-free candidate still wins its tier — the rule is an
    /// ordering, not a refusal (a produce-time refusal demonstrably
    /// re-shipped a broken native in `cell_candidate_wins_mil5_plates_
    /// over_broken_native` while this design was being built).
    #[test]
    fn unverified_geometry_alone_still_wins_the_error_free_tier() {
        let mut ps = blank();
        // Incumbent native produced WITH errors — the broken incumbent.
        ps[NATIVE] = produced(0.5, true);
        ps[NATIVE].counts = Some(counts(3, 0, 0));
        // Cells: the only error-free candidate, unverified.
        ps[CELLS] = produced(2.0, true);
        ps[CELLS].counts = Some(counts(0, 0, 1));
        ps[CELLS].unverified_geometry = true;
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: CELLS,
                stage: SelectionStage::BestErrorFree
            },
            "the unverified rescue must still win when it is the only error-free candidate"
        );
    }

    /// Carry-over (e) of #698 rounds 9-10: bind the THREE parallel
    /// candidate lists — this file's `EXPECTED_ORDER`, the engine's
    /// `decomposition_search::CANDIDATE_ORDER`, and the registration
    /// vector — and then bind the order itself to the semantics that
    /// depend on it.
    ///
    /// **Why the equality checks are not enough.** Every existing check
    /// compares one list against another, so a COORDINATED permutation
    /// (the same swap applied to all three) passes all of them while
    /// genuinely reordering the candidate field: `FirstProduced` is
    /// positional, `BestAccepted`'s ties go to the earliest index, and
    /// v1's `ranking_len` is a SLICE BOUND, not a predicate. The
    /// positional assertions below are the part a coordinated swap
    /// cannot satisfy, because they are stated against what the
    /// positions MEAN rather than against another copy of the list.
    ///
    /// The load-bearing one is the tail: v1 excludes the scoped
    /// candidates from the generic ranking with `candidates[..DI_IDX]`,
    /// and v2 excludes them with a `scoped` field test. Those two are
    /// equivalent **only while the scoped registrations are exactly the
    /// tail** — move DI to slot 2 with everything else consistent and
    /// v1's slice would drop three unrelated candidates while v2's
    /// filter drops the right two, silently. (RFC-070 decision log,
    /// "the `ranking_len` slice becomes the AdmissionRule as a FILTER".)
    #[test]
    fn the_candidate_order_is_bound_to_its_positional_semantics() {
        let p = SelectionPolicy::current();
        let names: Vec<&str> = p.producers.iter().map(|r| r.name).collect();
        assert_eq!(
            super::super::decomposition_search::CANDIDATE_ORDER.to_vec(),
            EXPECTED_ORDER.to_vec(),
            "the engine's candidate slots and this module's expectation have diverged"
        );
        assert_eq!(names, EXPECTED_ORDER.to_vec());

        // 1. The incumbent is slot 0. `FirstProduced` is positional and
        //    every rank tie resolves to the earliest registration, so the
        //    incumbent sitting anywhere else changes the degraded answer.
        assert_eq!(
            p.incumbent_index(),
            Some(0),
            "the preference order puts the incumbent first; a later slot would hand \
             `first-produced` and every score tie to a challenger"
        );

        // 2. The scoped registrations are exactly the TAIL, contiguously
        //    — the equivalence between v1's slice bound and v2's field
        //    filter, asserted rather than assumed.
        let scoped: Vec<usize> = (0..p.producers.len())
            .filter(|&i| p.producers[i].scoped)
            .collect();
        assert!(
            !scoped.is_empty(),
            "the AdmissionRule needs something to admit"
        );
        let tail: Vec<usize> = (p.producers.len() - scoped.len()..p.producers.len()).collect();
        assert_eq!(
            scoped, tail,
            "the `scoped` registrations must occupy the TAIL of the order. v1 confines \
             them with the slice `candidates[..ranking_len]` — a POSITION — while v2 \
             confines them with the `scoped` FIELD; the two agree only while `scoped` is \
             a suffix. A coordinated reorder of all three name lists passes every \
             equality check above and fails here, which is the point"
        );

        // 3. Within that tail, DI precedes horizontal: the pairwise
        //    resolution's ties go to the earlier registration, and
        //    RFC-060 fixed that as DI.
        assert_eq!(
            p.producers[scoped[0]].name, "direct-insertion",
            "ties between two scoped winners go to the earlier registration, which \
             RFC-060 fixed as DI"
        );

        // 4. Exactly one quality-key rival, and it is registered BEFORE
        //    the scoped tail — its stage runs first in the program and
        //    its gate reads only the incumbent.
        let rivals: Vec<usize> = (0..p.producers.len())
            .filter(|&i| p.producers[i].quality_key_rival)
            .collect();
        assert_eq!(
            rivals.len(),
            1,
            "the quality-key stage compares exactly one rival"
        );
        assert!(rivals[0] < scoped[0]);
    }

    #[test]
    fn refuse_on_error_is_asymmetric_and_that_asymmetry_is_load_bearing() {
        // Preserving WHICH producers self-refuse is required for parity:
        // `ec30` ships an error-laden native through the fallback stage
        // precisely because native does NOT carry the gate.
        let p = SelectionPolicy::current();
        let refusing: Vec<&str> = p
            .producers
            .iter()
            .filter(|r| r.refuse_on_error)
            .map(|r| r.name)
            .collect();
        assert_eq!(
            refusing,
            vec!["cell-composed", "direct-insertion", "horizontal-stack"]
        );
    }

    #[test]
    fn only_di_carries_the_equal_and_denser_arm() {
        let p = SelectionPolicy::current();
        let arms: Vec<&str> = p
            .producers
            .iter()
            .filter(|r| r.equal_and_denser)
            .map(|r| r.name)
            .collect();
        assert_eq!(arms, vec!["direct-insertion"]);
        let scoped: Vec<&str> = p
            .producers
            .iter()
            .filter(|r| r.scoped)
            .map(|r| r.name)
            .collect();
        assert_eq!(scoped, vec!["direct-insertion", "horizontal-stack"]);
    }

    #[test]
    fn excluded_warning_categories_is_belt_detour_alone() {
        // The two #632 B6 demotions left the set by DELETION (#684).
        let p = SelectionPolicy::current();
        assert_eq!(
            p.excluded_warning_categories
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["belt-detour"]
        );
    }

    #[test]
    fn error_kind_table_matches_classify_errors() {
        let p = SelectionPolicy::current();
        assert_eq!(p.kind_of("belt-item-isolation"), IssueKind::Contamination);
        assert_eq!(p.kind_of("pipe-isolation"), IssueKind::Contamination);
        assert_eq!(p.kind_of("entity-overlap"), IssueKind::Structural);
        assert_eq!(p.kind_of("pipe-to-ground"), IssueKind::Structural);
        // The `_ =>` arm.
        assert_eq!(p.kind_of("input-rate-delivery"), IssueKind::Starvation);
        assert_eq!(
            p.kind_of("a-category-that-does-not-exist"),
            IssueKind::Starvation
        );
        assert_eq!(p.contamination_weight, 3);
    }

    // -----------------------------------------------------------------
    // Stage 1 — quality-key lexicograph
    // -----------------------------------------------------------------

    #[test]
    fn merge_tap_wins_on_a_strictly_lower_quality_key() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, false);
        ps[NATIVE].kinds = Some(kinds(0, 4, 0));
        ps[MERGE_TAP] = produced(0.5, true);
        ps[MERGE_TAP].kinds = Some(kinds(0, 1, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: MERGE_TAP,
                stage: SelectionStage::MergeTap
            }
        );
    }

    #[test]
    fn structural_dominates_weighted_functional() {
        // 1 structural error is worse than any number of functional
        // ones: the blueprint does not import at all.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, false);
        ps[NATIVE].kinds = Some(kinds(0, 9, 0));
        ps[MERGE_TAP] = produced(1.0, true);
        ps[MERGE_TAP].kinds = Some(kinds(0, 0, 1));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d.winner, NATIVE,
            "structural must dominate the weighted functional total"
        );
    }

    #[test]
    fn contamination_outweighs_starvation_three_to_one() {
        // The ec@35 decision: merge-tap trades 2 starvation dead-ends
        // for 1 contamination, so it has FEWER errors by count (3 < 4)
        // and still loses by kind.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, false);
        ps[NATIVE].kinds = Some(kinds(0, 4, 0));
        ps[MERGE_TAP] = produced(1.0, true);
        ps[MERGE_TAP].kinds = Some(kinds(1, 2, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(d.winner, NATIVE);
    }

    /// RFC-071 B2 (#701): the ec30 shipping mechanism, pinned. Three
    /// route-severing total-stops must not beat sixty-five functional
    /// throttles — the shipped native delivered 0.00/s while the
    /// rejected merge-tap produced 17.5/s (meter receipts on #701;
    /// trigger removed by #706, taxonomy hole closed here). Before the
    /// RouteSevered class both sides classed as starvation and
    /// (0, 3) < (0, 65) held the dead layout.
    #[test]
    fn route_severed_dominates_any_functional_total() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, false);
        ps[NATIVE].kinds = Some(ErrorKindCounts {
            route_severed: 3,
            ..Default::default()
        });
        ps[MERGE_TAP] = produced(1.0, true);
        ps[MERGE_TAP].kinds = Some(ErrorKindCounts {
            starvation: 65,
            ..Default::default()
        });
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: MERGE_TAP,
                stage: SelectionStage::MergeTap
            },
            "a tier with no route must lose to any quantity of throttles"
        );
        // And structural still dominates route-severed: an unimportable
        // blueprint outranks even a severed route.
        assert!(
            ErrorKindCounts {
                structural: 1,
                ..Default::default()
            }
            .quality_key(3)
                > ErrorKindCounts {
                    route_severed: 9,
                    ..Default::default()
                }
                .quality_key(3)
        );
    }

    /// The table half of B2: every route-severing category maps to the
    /// class in the SHIPPED policy (built from the same
    /// `ROUTE_SEVERING_CATEGORIES` list the classifier reads), and a
    /// representative throttle stays Starvation.
    #[test]
    fn route_severing_categories_map_to_the_class() {
        let p = SelectionPolicy::current();
        for c in super::super::decomposition_search::ROUTE_SEVERING_CATEGORIES {
            assert_eq!(p.kind_of(c), IssueKind::RouteSevered, "{c}");
        }
        assert_eq!(p.kind_of("lane-throughput"), IssueKind::Starvation);
    }

    /// The three const lists are the table's SOURCE; a category in two
    /// lists would make classification depend on `current()`'s insert
    /// order, silently (#716 review nit — cheap to pin, expensive to
    /// debug).
    #[test]
    fn the_three_category_lists_are_pairwise_disjoint() {
        use super::super::decomposition_search::{
            CONTAMINATION_CATEGORIES, ROUTE_SEVERING_CATEGORIES, STRUCTURAL_CATEGORIES,
        };
        let all: Vec<&str> = CONTAMINATION_CATEGORIES
            .iter()
            .chain(STRUCTURAL_CATEGORIES.iter())
            .chain(ROUTE_SEVERING_CATEGORIES.iter())
            .copied()
            .collect();
        let set: BTreeSet<&str> = all.iter().copied().collect();
        assert_eq!(
            set.len(),
            all.len(),
            "a category appears in more than one kind list: {all:?}"
        );
    }

    /// The deleted v1 test's two measured profiles pin the `[3, 17]`
    /// robustness window. This exercises the v2 stage, not just the
    /// arithmetic helper: every weight in the window returns the same
    /// `(winner, stage)` pair for both merge-tap-style decisions.
    #[test]
    fn contamination_weight_window_is_stable_for_quality_key_policy() {
        let ec_native = kinds(0, 4, 0);
        let ec_merge_tap = kinds(1, 2, 0);
        let utility_native = kinds(0, 175, 0);
        let utility_merge_tap = kinds(8, 38, 0);

        let reference_decisions = (
            quality_key_decision(ec_native, ec_merge_tap, 3),
            quality_key_decision(utility_native, utility_merge_tap, 3),
        );
        assert_eq!(reference_decisions.0.winner, NATIVE);
        assert_eq!(reference_decisions.1.winner, MERGE_TAP);
        for weight in 3..=17 {
            assert_eq!(
                (
                    quality_key_decision(ec_native, ec_merge_tap, weight),
                    quality_key_decision(utility_native, utility_merge_tap, weight),
                ),
                reference_decisions,
                "v2 quality-key verdicts changed inside the [3, 17] window at weight {weight}"
            );
        }

        // Compare the rival to the incumbent, so the boundary assertions
        // pin the comparator result itself. At weight 2 the EC keys tie;
        // v2 deliberately still returns native because ties keep the
        // incumbent. The comparator decision nevertheless differs from
        // the strict native win throughout the window.
        let quality_relation = |weight| {
            (
                ec_merge_tap
                    .quality_key(weight)
                    .cmp(&ec_native.quality_key(weight)),
                utility_merge_tap
                    .quality_key(weight)
                    .cmp(&utility_native.quality_key(weight)),
            )
        };
        assert_eq!(
            quality_relation(2),
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Less),
            "weight 2 is the EC tie boundary from the deleted test"
        );
        assert_eq!(
            quality_relation(3),
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Less)
        );
        assert_ne!(
            quality_relation(2),
            quality_relation(3),
            "the lower boundary must differ from the in-window comparator verdict"
        );
        assert_eq!(
            quality_relation(17),
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Less)
        );
        assert_eq!(
            quality_relation(18),
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater),
            "weight 18 is the utility flip boundary from the deleted test"
        );
        assert_ne!(
            quality_relation(18),
            quality_relation(17),
            "the upper boundary must differ from the in-window comparator verdict"
        );

        assert_eq!(
            quality_key_decision(ec_native, ec_merge_tap, 2),
            reference_decisions.0,
            "the EC tie at weight 2 must resolve to the incumbent, native"
        );
        assert_ne!(
            quality_key_decision(utility_native, utility_merge_tap, 18),
            reference_decisions.1,
            "the utility boundary at weight 18 must change the v2 winner"
        );
        assert!((3..=17).contains(&SelectionPolicy::current().contamination_weight));
    }

    #[test]
    fn quality_key_ties_favour_the_incumbent() {
        let mut ps = blank();
        ps[NATIVE] = produced(0.0, false);
        ps[NATIVE].kinds = Some(kinds(1, 1, 0));
        ps[MERGE_TAP] = produced(99.0, true);
        ps[MERGE_TAP].kinds = Some(kinds(1, 1, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: NATIVE,
                stage: SelectionStage::MergeTap
            },
            "an equal key keeps the incumbent, and a far better score does not rescue it"
        );
    }

    #[test]
    fn a_gap_in_the_kinds_skips_the_stage_rather_than_reading_as_zero() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 0, 0));
        ps[MERGE_TAP] = produced(2.0, true);
        ps[MERGE_TAP].counts = Some(counts(0, 0, 0));
        // No kinds anywhere: nothing classified these layouts.
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d.stage,
            SelectionStage::BestErrorFree,
            "an unmeasured kind must not read as a flawless 0/0/0 and win stage 1"
        );
    }

    // -----------------------------------------------------------------
    // The non-shadowing rule (#474)
    // -----------------------------------------------------------------

    #[test]
    fn an_incumbent_win_at_stage_one_does_not_shadow_the_pairwise_floor() {
        // merge-tap ran and lost; DI is strictly better than native.
        // A plain `.or()` chain returned native here and threw DI's
        // already-computed result away unread.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].kinds = Some(kinds(0, 1, 0));
        ps[NATIVE].counts = Some(counts(0, 3, 0));
        ps[MERGE_TAP] = produced(1.0, true);
        ps[MERGE_TAP].kinds = Some(kinds(0, 2, 0));
        ps[DI] = produced(0.9, true);
        ps[DI].counts = Some(counts(0, 1, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: DI,
                stage: SelectionStage::ScopedPairwise
            }
        );
    }

    #[test]
    fn a_held_incumbent_stands_when_no_pairwise_stage_displaces_it() {
        // Same shape, but DI does not improve on native — the held
        // answer stands, tagged with the stage that held it, and does
        // NOT fall through to the ranked stages.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].kinds = Some(kinds(0, 1, 0));
        ps[NATIVE].counts = Some(counts(0, 1, 0));
        ps[MERGE_TAP] = produced(5.0, true);
        ps[MERGE_TAP].kinds = Some(kinds(0, 2, 0));
        ps[MERGE_TAP].counts = Some(counts(0, 0, 0));
        ps[DI] = produced(0.9, true);
        ps[DI].counts = Some(counts(0, 1, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: NATIVE,
                stage: SelectionStage::MergeTap
            },
            "a held incumbent must terminate the chain, not fall through to a ranked \
             stage where merge-tap's better score would win"
        );
    }

    // -----------------------------------------------------------------
    // Stage 2 — component-wise floor
    // -----------------------------------------------------------------

    #[test]
    fn the_floor_is_component_wise_not_lexicographic() {
        // A lexicographic ordering would let DI win on the warning
        // channel while regressing 12 layout warnings.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 1, 0));
        ps[DI] = produced(1.0, true);
        ps[DI].counts = Some(counts(0, 0, 12));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_ne!(
            d.winner, DI,
            "a layout-warning regression must not hide behind a \
                                  validator-warning improvement"
        );
    }

    #[test]
    fn di_takes_the_equal_and_denser_arm_and_horizontal_does_not() {
        let mut base = blank();
        base[NATIVE] = produced(1.0, true);
        base[NATIVE].counts = Some(counts(0, 2, 1));

        let mut with_di = base.clone();
        with_di[DI] = produced(1.5, true);
        with_di[DI].counts = Some(counts(0, 2, 1));
        let d = decide(&with_di, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: DI,
                stage: SelectionStage::ScopedPairwise
            },
            "equal on every channel and strictly denser: DI's arm fires"
        );

        let mut with_hs = base.clone();
        with_hs[HS] = produced(1.5, true);
        with_hs[HS].counts = Some(counts(0, 2, 1));
        let d = decide(&with_hs, &SelectionPolicy::current()).unwrap();
        assert_ne!(
            d.winner, HS,
            "horizontal has no equal-and-denser arm (RFC-060's measured call)"
        );
    }

    #[test]
    fn the_denser_arm_needs_more_than_epsilon() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 2, 1));
        ps[DI] = produced(1.0 + DENSITY_TIEBREAK_EPSILON / 2.0, true);
        ps[DI].counts = Some(counts(0, 2, 1));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_ne!(
            d.winner, DI,
            "a sub-epsilon score difference is a tie, and ties keep native"
        );
    }

    #[test]
    fn an_unaccepted_scoped_candidate_never_displaces_the_incumbent() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(2, 5, 3));
        ps[DI] = produced(9.0, false);
        ps[DI].counts = Some(counts(0, 0, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_ne!(
            d.winner, DI,
            "`accepted` is a hard constraint the issue channels cannot see"
        );
    }

    #[test]
    fn two_scoped_winners_resolve_by_the_same_floor_ties_to_the_earlier() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 4, 0));
        ps[DI] = produced(1.0, true);
        ps[DI].counts = Some(counts(0, 2, 0));
        ps[HS] = produced(9.0, true);
        ps[HS].counts = Some(counts(0, 1, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d.winner, HS,
            "horizontal is strictly better than DI on the floor"
        );

        // Tie on the floor: the far better score must NOT rescue
        // horizontal — the earlier registration keeps it.
        ps[HS].counts = Some(counts(0, 2, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(d.winner, DI);
    }

    /// The one overlap of the two scoped arms that neither the test above
    /// nor the corpus reaches: **DI qualifies only by its
    /// equal-and-denser arm while horizontal qualifies by being strictly
    /// better than the incumbent.** The arms admit them for different
    /// reasons, so "which is better" is not obviously the same question
    /// in v2's single loop as in v1's two-then-one structure.
    ///
    /// v1 answers HORIZONTAL, and it is worth spelling out why, because
    /// the reason is arithmetic rather than precedence: `di_choice` is
    /// `Some(DI)` (equal counts, denser), `horizontal_choice` is
    /// `Some(HS)` (strictly better counts), so `scoped_choice`'s
    /// both-Some arm compares them directly with
    /// `hs_counts.strictly_better_than(&di_counts)` — and DI's counts ARE
    /// the incumbent's, so horizontal is strictly better than DI too.
    /// **Density does not enter that comparison at all**: v1's pairwise
    /// resolution has no density tiebreak, deliberately (RFC-060), so
    /// DI's whole claim evaporates the moment a genuinely quieter rival
    /// exists.
    ///
    /// v2 reaches the same answer by a different route — one fold over
    /// the scoped registrations, where DI is admitted by
    /// `equal_and_denser`, seated as `best`, and then displaced by
    /// horizontal under the same `strictly_better_than`. Same answer,
    /// and the two routes agreeing is the point of the test
    /// (#698 rounds 9-10 carry-over (d)).
    #[test]
    fn a_strictly_better_horizontal_beats_a_merely_denser_di() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 3, 1));
        // DI: equal on every channel, strictly denser — its arm, and
        // ONLY its arm, admits it.
        ps[DI] = produced(9.0, true);
        ps[DI].counts = Some(counts(0, 3, 1));
        // Horizontal: strictly better on the warning channel, and far
        // sparser than DI.
        ps[HS] = produced(0.1, true);
        ps[HS].counts = Some(counts(0, 2, 1));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: HS,
                stage: SelectionStage::ScopedPairwise
            },
            "v1's both-scoped-won arm compares the two by the floor ALONE (no density \
             tiebreak), and DI's counts are the incumbent's — so a strictly-better \
             horizontal takes it despite DI's 90x score"
        );

        // The control: remove horizontal and DI's denser arm stands.
        ps[HS] = not_run();
        assert_eq!(
            decide(&ps, &SelectionPolicy::current()).unwrap(),
            Decision {
                winner: DI,
                stage: SelectionStage::ScopedPairwise
            },
        );
    }

    /// Carry-over (a) of #698 rounds 9-10: the warnings-first order's
    /// missing-key arm ABSTAINS instead of falling through to the score.
    ///
    /// Unreachable under today's program — `BestErrorFree` is the only
    /// stage using this order and its admission already requires counts —
    /// so it is exercised through a policy whose ranked stage drops
    /// `require_error_free`, which is exactly the future stage the arm
    /// exists for. Without the fix the unmeasured candidate wins on
    /// density alone; with it, the primary criterion being unanswerable
    /// means the challenger does not rank ahead and the earlier
    /// registration keeps its seat.
    #[test]
    fn a_missing_warning_key_abstains_rather_than_ranking_on_score() {
        let mut policy = SelectionPolicy::current();
        policy.program.stages = vec![StageSpec {
            tag: SelectionStage::BestAccepted,
            kind: StageKind::TieredRank(RankSpec {
                require_accepted: true,
                require_error_free: false,
                success_order: RankOrder::WarningsAscThenScoreDesc,
                refusal_order: RankOrder::WarningsAscThenScoreDesc,
                verified_geometry_first: false,
            }),
            on_incumbent_win: ChainBehavior::Terminate,
        }];

        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 5, 0));
        // No counts at all: nothing measured this candidate, so it has
        // no warning key — and a 50x score must not smuggle it past the
        // criterion it cannot answer.
        ps[CELLS] = produced(50.0, true);
        assert_eq!(
            decide(&ps, &policy).unwrap().winner,
            NATIVE,
            "an unmeasured candidate must not rank ahead on the SECONDARY criterion when \
             the primary one is a gap"
        );

        // Control, in the same policy: give it a key and the ordinary
        // warnings-asc comparison applies.
        ps[CELLS].counts = Some(counts(0, 4, 0));
        assert_eq!(decide(&ps, &policy).unwrap().winner, CELLS);
    }

    // -----------------------------------------------------------------
    // The AdmissionRule
    // -----------------------------------------------------------------

    #[test]
    fn a_scoped_candidate_is_barred_from_the_ranked_stages_when_the_incumbent_produced() {
        // DI is error-free, accepted, and denser — but it does not beat
        // native on the floor (native is error-free too, with fewer
        // warnings), so it must lose. Admitting it to the error-free
        // tier would let density outrank warnings: exactly the
        // `tier2_electronic_circuit` regression `ranking_len` blocks.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 0, 0));
        ps[DI] = produced(50.0, true);
        ps[DI].counts = Some(counts(0, 1, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: NATIVE,
                stage: SelectionStage::BestErrorFree
            }
        );
    }

    #[test]
    fn a_scoped_candidate_is_admitted_when_the_incumbent_produced_nothing() {
        // Native refused. DI now competes with cell-composed on the
        // merits rather than auto-winning the refusal.
        let mut ps = blank();
        ps[NATIVE] = IssueProfile {
            outcome: Some(SelectionCandidateOutcome::Refused),
            refusal_reason: Some("bus refused".into()),
            ..Default::default()
        };
        ps[CELLS] = produced(1.0, true);
        ps[CELLS].counts = Some(counts(0, 6, 0));
        ps[DI] = produced(0.5, true);
        ps[DI].counts = Some(counts(0, 0, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: DI,
                stage: SelectionStage::BestErrorFree
            },
            "the refusal path orders the error-free tier warnings-first, so DI's clean \
             0/0 beats cell-composed's denser 0/6"
        );
    }

    // -----------------------------------------------------------------
    // Stages 3-5 — the tiered rank
    // -----------------------------------------------------------------

    #[test]
    fn the_success_path_orders_the_error_free_tier_by_score() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(0, 6, 0));
        ps[CELLS] = produced(2.0, true);
        ps[CELLS].counts = Some(counts(0, 9, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: CELLS,
                stage: SelectionStage::BestErrorFree
            },
            "with the incumbent produced, #392's score-first order applies unchanged"
        );
    }

    #[test]
    fn an_error_laden_candidate_cannot_reach_the_error_free_tier() {
        let mut ps = blank();
        ps[NATIVE] = produced(9.0, true);
        ps[NATIVE].counts = Some(counts(3, 0, 0));
        ps[CELLS] = produced(1.0, true);
        ps[CELLS].counts = Some(counts(0, 0, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: CELLS,
                stage: SelectionStage::BestErrorFree
            }
        );
    }

    #[test]
    fn without_counts_the_error_free_tier_is_empty_and_best_accepted_decides() {
        // v1's laziness, reproduced by the gap rule: a single-layout
        // solve never computes `clean_flags` at all.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: NATIVE,
                stage: SelectionStage::BestAccepted
            }
        );
    }

    /// The [`MeasurementRule`], and why it is enforced in [`decide`]
    /// rather than left to whoever builds the profiles.
    ///
    /// Filling a single-layout solve's counts — which
    /// `IssueProfile::measure` ALWAYS does — would populate an
    /// error-free tier that v1 leaves empty, moving the same solve from
    /// `best-accepted` to `best-error-free`: same winner, different
    /// answer to "which question decided this", which the corpus
    /// equivalence rule compares. That is the shape of the 12
    /// `best-accepted` cells in the #694 baseline. So "eager vs lazy
    /// cannot change outcomes, only cost" is true of the WINNER and
    /// false of the STAGE — and the rule below is what makes it true of
    /// both.
    #[test]
    fn the_measurement_rule_makes_eager_and_lazy_decide_alike() {
        let mut lazy = blank();
        lazy[NATIVE] = produced(1.0, true);
        let mut eager = lazy.clone();
        eager[NATIVE].counts = Some(counts(0, 0, 0));

        let policy = SelectionPolicy::current();
        let lazy_d = decide(&lazy, &policy).unwrap();
        let eager_d = decide(&eager, &policy).unwrap();
        assert_eq!(
            lazy_d, eager_d,
            "the measurement rule must erase the eager/lazy split"
        );
        assert_eq!(
            lazy_d.stage,
            SelectionStage::BestAccepted,
            "v1's answer, either way"
        );

        // …and the rule is a THRESHOLD, not a blanket skip: add a second
        // produced candidate and the tier is evaluated normally.
        let mut two = eager.clone();
        two[CELLS] = produced(2.0, true);
        two[CELLS].counts = Some(counts(0, 0, 0));
        let d = decide(&two, &policy).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: CELLS,
                stage: SelectionStage::BestErrorFree
            }
        );
    }

    // No test for the length-mismatch path: the `debug_assert` fires
    // first in every build `cargo test` produces, so a test could only
    // assert that debug panics — which is the assert's own text — while
    // printing a panic trace into every suite run. The release
    // behaviour it guards (decide nothing rather than rank the wrong
    // producers) is stated at the call site.

    /// The NaN tie-break direction, pinned because #698 review round 7
    /// claimed v2 diverges from v1 here and the claim does not survive
    /// working v1's comparator out.
    ///
    /// v1: `max_by(|(ia, a), (ib, b)| a.partial_cmp(b)
    /// .unwrap_or(Equal).then(ib.cmp(ia)))`. A NaN makes `partial_cmp`
    /// return `None` → `Equal`, so the index term decides — and it is
    /// REVERSED (`ib.cmp(ia)`), which makes the SMALLER index compare
    /// as Greater, so the max is the earliest index. v2 keeps its
    /// incumbent unless a challenger is strictly ahead, and
    /// `partial_cmp` against a NaN is never `Some(Greater)` — also the
    /// earliest index. Same answer, by two routes.
    #[test]
    fn a_nan_score_keeps_the_earliest_registration_as_v1_does() {
        let mut ps = blank();
        ps[NATIVE] = produced(f64::NAN, true);
        ps[CELLS] = produced(5.0, true);
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: NATIVE,
                stage: SelectionStage::BestAccepted
            },
            "v1's reversed index tie-break makes the earliest index the max on a NaN; \
             v2's strictly-better-only fold keeps the same one"
        );
        // …and the direction is not an artifact of NaN sitting first.
        let mut swapped = blank();
        swapped[NATIVE] = produced(5.0, true);
        swapped[CELLS] = produced(f64::NAN, true);
        assert_eq!(
            decide(&swapped, &SelectionPolicy::current())
                .unwrap()
                .winner,
            NATIVE
        );
    }

    #[test]
    fn best_accepted_takes_the_highest_score_ties_to_the_earlier() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[K1] = produced(2.0, true);
        assert_eq!(decide(&ps, &SelectionPolicy::current()).unwrap().winner, K1);
        ps[K1] = produced(1.0, true);
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: NATIVE,
                stage: SelectionStage::BestAccepted
            }
        );
    }

    #[test]
    fn first_produced_ships_an_error_laden_best_rather_than_refusing() {
        // The `ec30` trap, preserved deliberately: nothing is accepted,
        // so the earliest producer that made anything ships — errors and
        // all. Changing this is Phase-3 calibration work.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, false);
        ps[NATIVE].counts = Some(counts(3, 0, 2));
        ps[SPLIT] = produced(9.0, false);
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(
            d,
            Decision {
                winner: NATIVE,
                stage: SelectionStage::FirstProduced
            },
            "the fallback is positional, not score-ranked"
        );
    }

    #[test]
    fn no_producer_no_decision() {
        assert_eq!(decide(&blank(), &SelectionPolicy::current()), None);
    }

    // -----------------------------------------------------------------
    // The K70-1 fence
    // -----------------------------------------------------------------

    /// The mechanical form of RFC-070's expressibility boundary: stage
    /// code may read registration FIELDS, never registration NAMES.
    /// Reads this file's own source between the fence markers.
    #[test]
    fn k70_1_fence_holds() {
        const SRC: &str = include_str!("selection_policy.rs");
        // The COMMENT form of each marker, so the module doc's
        // backticked mention of the same words cannot be mistaken for
        // the fence itself (which would shrink the checked region to a
        // doc paragraph and pass vacuously).
        let begin = SRC.find("// K70-1-FENCE-BEGIN").expect("fence opens");
        let end = SRC.find("// K70-1-FENCE-END").expect("fence closes");
        assert!(begin < end, "fence markers are out of order");
        // CODE only: comment lines are stripped before scanning. The
        // boundary is about what stage logic BRANCHES on, and a fence
        // that also policed prose would push a future author to write
        // worse comments or to widen the fence — both worse than the
        // thing it prevents (#698 review round 5). Its limits, stated
        // rather than implied: it catches the literal form, not a name
        // assembled at runtime, and it needs `EXPECTED_ORDER` kept in
        // step with the registrations by hand — deliberately, since a
        // list that reads the thing it checks cannot detect a reorder.
        let fenced: String = SRC[begin..end]
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let fenced = fenced.as_str();
        for name in EXPECTED_ORDER {
            assert!(
                !fenced.contains(name),
                "candidate name `{name}` appears inside the K70-1 fence. Stage logic must \
                 branch on ProducerRegistration FIELDS, never on names — a name-keyed \
                 branch here is K70-1 FIRING, which the campaign lead adjudicates. Do not \
                 widen the fence to make this pass"
            );
        }
        assert!(
            !fenced.contains(".name"),
            "stage logic reads a registration's `.name`. See above: fields, never names"
        );
    }

    // -----------------------------------------------------------------
    // Gates (Phase-0b oracle gap (c): a gate must say WHICH conjunct)
    // -----------------------------------------------------------------

    fn gear_solve() -> SolverResult {
        let inputs: rustc_hash::FxHashSet<String> =
            ["iron-plate"].iter().map(|s| (*s).to_string()).collect();
        crate::solver::solve("iron-gear-wheel", 10.0, &inputs, "assembling-machine-1")
            .expect("gear solves")
    }

    #[test]
    fn a_gate_reports_the_first_failing_clause() {
        let policy = SelectionPolicy::current();
        let sr = gear_solve();
        let opts = LayoutOptions::default();
        let prior = [None; 7];
        let ctx = GateContext {
            solver_result: &sr,
            opts: &opts,
            prior: &prior,
            incumbent: Some(NATIVE),
            registration_index: MERGE_TAP,
        };
        // The incumbent is always eligible.
        assert_eq!(
            policy.producers[NATIVE].gate.evaluate(&ctx),
            GateVerdict::Eligible
        );
        // The default strategy IS Pooled, so merge-tap clears its first
        // clause and is stopped by the second — the incumbent has not
        // produced an unaccepted layout (it has not run at all here).
        assert_eq!(
            policy.producers[MERGE_TAP].gate.evaluate(&ctx),
            GateVerdict::Excluded("incumbent-produced-and-unaccepted")
        );
        // A single-ingredient gear solve has no DI couplings.
        assert_eq!(
            policy.producers[DI].gate.evaluate(&ctx),
            GateVerdict::Excluded("solve-has-di-couplings")
        );
        // Change the strategy and the FIRST clause is what reports —
        // which is the property gap (c) needed: "not tried" is no longer
        // an anonymous conjunction.
        let partitioned = LayoutOptions {
            strategy: LayoutStrategy::PartitionedDecomposed,
            ..Default::default()
        };
        assert_eq!(
            policy.producers[MERGE_TAP].gate.evaluate(&GateContext {
                opts: &partitioned,
                ..ctx
            }),
            GateVerdict::Excluded("strategy-is-pooled")
        );
    }

    #[test]
    fn size_split_stands_down_once_an_earlier_producer_landed_an_accepted_layout() {
        let policy = SelectionPolicy::current();
        let sr = gear_solve();
        let opts = LayoutOptions {
            strategy: LayoutStrategy::PartitionedDecomposed,
            ..Default::default()
        };
        let mut prior = [None; 7];
        prior[NATIVE] = Some(false);
        let excluded_by = |prior: &[Option<bool>; 7]| {
            policy.producers[SPLIT].gate.evaluate(&GateContext {
                solver_result: &sr,
                opts: &opts,
                prior,
                incumbent: Some(NATIVE),
                registration_index: SPLIT,
            })
        };
        assert_eq!(excluded_by(&prior), GateVerdict::Eligible);
        prior[K1] = Some(true);
        assert_eq!(
            excluded_by(&prior),
            GateVerdict::Excluded("no-earlier-producer-accepted")
        );

        // …and a LATER registration's acceptance must NOT stand it
        // down. Unbounded, this scanned the whole array, so a
        // cell-composed win at slot 4 would have silently disabled the
        // size-split arm — invisible today only because the loop fills
        // `prior` in order (#698 review round 1).
        let mut later = [None; 7];
        later[NATIVE] = Some(false);
        later[CELLS] = Some(true);
        later[HS] = Some(true);
        assert_eq!(excluded_by(&later), GateVerdict::Eligible);
    }

    /// The four gates the corpus cannot reach and the two tests
    /// above did not cover (#698 review round 4). The harness feeds
    /// already-produced profiles to `decide`, which never evaluates a
    /// gate at all — so a mis-transcribed eligibility clause survives
    /// 140/140 and first appears in Phase 2a, where it moves the
    /// candidate SET rather than the ranking.
    ///
    /// **What this is not** (#698 review round 5): an equivalence check
    /// against production. It pins each clause against the v1 condition
    /// as READ from `select_best_decomposition`'s `try_*` booleans; a
    /// mis-reading of that source would be reproduced faithfully here.
    /// Only the Phase-2a shadow, which runs both dispatches on the same
    /// solve, can close that — this closes the weaker but real hole of
    /// a clause nothing exercises at all.
    #[test]
    fn every_registered_gate_is_pinned_against_its_v1_conjunction() {
        let policy = SelectionPolicy::current();
        let sr = gear_solve();
        let mut prior = [None; 7];
        let verdict = |idx: usize, opts: &LayoutOptions, prior: &[Option<bool>; 7]| {
            policy.producers[idx].gate.evaluate(&GateContext {
                solver_result: &sr,
                opts,
                prior,
                incumbent: Some(NATIVE),
                registration_index: idx,
            })
        };

        // k1-shape-fix: PartitionedDecomposed AND the incumbent produced
        // an UNACCEPTED layout. v1: `matches!(opts.strategy,
        // PartitionedDecomposed) && native_run.outcome.is_some_and(|(_, s)| !s.accepted)`.
        let pooled = LayoutOptions::default();
        let partitioned = LayoutOptions {
            strategy: LayoutStrategy::PartitionedDecomposed,
            ..Default::default()
        };
        assert_eq!(
            verdict(K1, &pooled, &prior),
            GateVerdict::Excluded("strategy-is-partitioned-decomposed")
        );
        assert_eq!(
            verdict(K1, &partitioned, &prior),
            GateVerdict::Excluded("incumbent-produced-and-unaccepted"),
            "the incumbent has not produced, which is not the same as producing \
             unaccepted — v1's `is_some_and` distinguishes them"
        );
        prior[NATIVE] = Some(true);
        assert_eq!(
            verdict(K1, &partitioned, &prior),
            GateVerdict::Excluded("incumbent-produced-and-unaccepted"),
            "an ACCEPTED incumbent stands k1 down: there is no unstampable shape to fix"
        );
        prior[NATIVE] = Some(false);
        assert_eq!(verdict(K1, &partitioned, &prior), GateVerdict::Eligible);

        // cell-composed: Candidate mode, DI not Forced, belt tier
        // unconstrained-or-express, chain-eligible.
        let cells_on = LayoutOptions {
            cell_composition: crate::bus::cells::CellComposition::Candidate,
            ..Default::default()
        };
        assert_eq!(
            verdict(
                CELLS,
                &LayoutOptions {
                    cell_composition: crate::bus::cells::CellComposition::Off,
                    ..Default::default()
                },
                &prior
            ),
            GateVerdict::Excluded("cell-composition-is-candidate")
        );
        assert_eq!(
            verdict(
                CELLS,
                &LayoutOptions {
                    direct_insertion: crate::bus::di_cell::DirectInsertion::Forced,
                    ..cells_on.clone()
                },
                &prior
            ),
            GateVerdict::Excluded("direct-insertion-not-forced"),
            "Forced DI is an explicit topology request; a competing variant stands down"
        );
        assert_eq!(
            verdict(
                CELLS,
                &LayoutOptions {
                    max_belt_tier: Some("transport-belt".to_string()),
                    ..cells_on.clone()
                },
                &prior
            ),
            GateVerdict::Excluded("belt-tier-unconstrained-or-express")
        );
        assert_eq!(
            verdict(
                CELLS,
                &LayoutOptions {
                    max_belt_tier: Some("express-transport-belt".to_string()),
                    ..cells_on.clone()
                },
                &prior
            ),
            GateVerdict::Eligible,
            "express IS allowed by that clause — an unconstrained tier is not the only \
             admissible one"
        );
        // NOT pinned here: a negative case for the `chain-eligible`
        // clause. Every fixture reachable from this test's cheap solves
        // is chain-eligible (measured: gear/am1, plastic/chem and ec/am1
        // all return `Ok(())`), so the clause is covered only in the
        // positive direction. Stated rather than left as an apparent
        // full sweep.

        // horizontal-stack: enabled, VerticalSplit, DI not Forced, and
        // the solve has a dual-input row. A gear solve has none.
        assert_eq!(
            verdict(
                HS,
                &LayoutOptions {
                    horizontal_candidate: false,
                    ..Default::default()
                },
                &prior
            ),
            GateVerdict::Excluded("horizontal-candidate-enabled")
        );
        assert_eq!(
            verdict(
                HS,
                &LayoutOptions {
                    direct_insertion: crate::bus::di_cell::DirectInsertion::Forced,
                    ..Default::default()
                },
                &prior
            ),
            GateVerdict::Excluded("direct-insertion-not-forced")
        );
        assert_eq!(
            verdict(HS, &pooled, &prior),
            GateVerdict::Excluded("solve-has-dual-input-row"),
            "iron-gear-wheel from plate is single-input, so the variant would be \
             bit-identical and the extra layout pass is pure waste"
        );

        // native's gate is empty by construction: the incumbent always runs.
        assert!(policy.producers[NATIVE].gate.clauses.is_empty());
    }

    #[test]
    fn firewall_receipts_cover_the_live_exclusions() {
        // A "firewall" that nothing checks is a comment. This pins the
        // receipts' declared scope against the set they argue for, so
        // changing the exclusions without touching the argument fails
        // here (#698 review round 4).
        let p = SelectionPolicy::current();
        let justified: BTreeSet<String> = p
            .firewalls
            .iter()
            .flat_map(|f| f.justifies.iter().map(|c| (*c).to_string()))
            .collect();
        assert_eq!(
            justified, p.excluded_warning_categories,
            "every excluded warning category needs a firewall receipt saying WHY, and a \
             receipt must not claim a category that is no longer excluded"
        );
        assert!(p.firewalls.iter().all(|f| !f.receipt.is_empty()));
        // Per-receipt, not just the union: a receipt claiming nothing
        // would otherwise ride along inside a set-equality that another
        // receipt satisfied, which is weaker than "each receipt names
        // the categories it argues for" (#698 review round 8).
        assert!(
            p.firewalls.iter().all(|f| !f.justifies.is_empty()),
            "a firewall that justifies no category is a comment wearing the type"
        );
    }

    // -----------------------------------------------------------------
    // measure(): the one-call projection
    // -----------------------------------------------------------------

    #[test]
    fn measure_reads_the_acceptance_gate_off_the_layout_warning_channel() {
        let layout = LayoutResult {
            warnings: vec!["no balancer template for (4, 9)".to_string()],
            ..Default::default()
        };
        let sr = SolverResult::default();
        let policy = SelectionPolicy::current();
        let profile = IssueProfile::measure(&layout, &sr, &policy, &policy.producers[NATIVE]);
        assert_eq!(profile.accepted, Some(false));
        assert!(profile
            .accepted_reason
            .unwrap()
            .contains("missing-balancer-template"));
        // The layout channel is counted separately from the validator's.
        assert_eq!(profile.counts.unwrap().layout_warnings, 1);
    }

    /// `refuse_on_error` is a PRODUCE-TIME gate, and it has to be
    /// applied by something. Here it is applied by `measure`, so the
    /// asymmetry survives a `measure -> decide` wiring: the same
    /// error-laden layout is `Produced` for the incumbent (which is how
    /// `ec30` ships an error-laden best) and `Refused` for DI.
    #[test]
    fn measure_applies_the_produce_time_refusal_only_where_policy_says_so() {
        // One entity overlapping itself: an `entity-overlap` Error that
        // needs no solver context to fire.
        let e = |name: &str| crate::models::PlacedEntity {
            name: name.to_string(),
            x: 0,
            y: 0,
            ..Default::default()
        };
        let layout = LayoutResult {
            entities: vec![e("transport-belt"), e("transport-belt")],
            ..Default::default()
        };
        let sr = SolverResult::default();
        let policy = SelectionPolicy::current();

        let native = IssueProfile::measure(&layout, &sr, &policy, &policy.producers[NATIVE]);
        assert!(
            native.counts.unwrap().errors > 0,
            "the fixture must actually produce Errors"
        );
        assert!(
            native.produced(),
            "native does NOT carry refuse_on_error — an error-laden layout stays in play, \
             which is the ec30 witness and is REQUIRED for parity"
        );

        let di = IssueProfile::measure(&layout, &sr, &policy, &policy.producers[DI]);
        assert!(
            !di.produced(),
            "DI carries refuse_on_error, so this layout is discarded"
        );
        assert_eq!(di.outcome, Some(SelectionCandidateOutcome::Refused));
        let reason = di
            .refusal_reason
            .clone()
            .expect("a refusal names its reason");
        assert!(
            reason.contains("entity-overlap"),
            "the refusal must retain WHICH categories fired (Phase-0b oracle gap (d)); \
             got {reason:?}"
        );
        assert!(
            di.counts.is_some() && di.kinds.is_some(),
            "…and the measurement survives the refusal rather than being stringified away"
        );

        // A refused profile cannot win any stage.
        let mut ps = blank();
        ps[NATIVE] = native;
        ps[DI] = di;
        let d = decide(&ps, &policy).unwrap();
        assert_eq!(d.winner, NATIVE);
    }
}
