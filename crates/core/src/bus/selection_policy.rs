//! RFC-070 Phase 1b (#689 track W2b): candidate selection expressed as
//! **policy data** instead of open-coded mechanisms.
//!
//! Nothing here is wired into production. `select_best_decomposition`
//! remains the only selection path; this module is the target
//! architecture plus the proof that it reproduces the decisions the
//! Phase-0 oracle recorded (`tests/parity_corpus.rs::policy_replay`).
//! Phase 2a is what runs it against freshly produced layouts.
//!
//! # The reframe: one measurement, three comparators
//!
//! Reading the mechanisms at source dissolves RFC-070's "three verdict
//! mechanisms" into a cleaner factorization: there is **one underlying
//! measurement** and three comparators consuming different projections
//! of it. Today `classify_errors`, `count_issues` and the `clean_flags`
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
//! **skips**, exactly as today's lazy sites do (`clean_flags` is not
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
//! read occurs inside the fence. Grep the markers before adding a branch
//! there.

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
/// transcribed from `decomposition_search.rs`'s `di_choice`
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
}

/// Per-kind Error counts plus the lexicographic quality key the
/// merge-tap comparison ranks on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ErrorKindCounts {
    pub contamination: usize,
    pub starvation: usize,
    pub structural: usize,
}

impl ErrorKindCounts {
    /// Weighted functional total; structural is excluded because
    /// [`Self::quality_key`] handles it lexicographically.
    pub fn weighted_functional(&self, contamination_weight: usize) -> usize {
        contamination_weight * self.contamination + self.starvation
    }

    /// Lower is better: structural dominates (an unimportable blueprint
    /// is worse than any functional defect), then the weighted
    /// functional total breaks ties within equal structural.
    pub fn quality_key(&self, contamination_weight: usize) -> (usize, usize) {
        (self.structural, self.weighted_functional(contamination_weight))
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
        self.counts.map(|c| c.selection_warnings + c.layout_warnings)
    }

    /// Measure a produced layout ONCE and derive every projection.
    ///
    /// This is the whole point of the reframe: `classify_errors`,
    /// `count_issues` and `clean_flags` each run `validate()`
    /// separately today; here one call feeds the kind classes, the
    /// three severity channels and the acceptance gates.
    /// `validate()` is deterministic, so eager measurement cannot
    /// change any outcome — only cost (RFC-070 §"Validation-once and
    /// laziness"; K70-3's isolated-run comparator adjudicates which
    /// Phase 2a ships).
    pub fn measure(
        layout: &LayoutResult,
        solver_result: &SolverResult,
        policy: &SelectionPolicy,
    ) -> Self {
        let issues = match crate::validate::validate(layout, Some(solver_result)) {
            Ok(issues) => issues,
            // `validate()` returns Err CARRYING the issues — reading only
            // `Ok` here would blank the profile of exactly the candidates
            // that failed hardest.
            Err(e) => e.issues,
        };
        let mut kinds = ErrorKindCounts::default();
        let mut errors = 0usize;
        let mut selection_warnings = 0usize;
        for i in &issues {
            match i.severity {
                crate::validate::Severity::Error => {
                    errors += 1;
                    match policy.kind_of(&i.category) {
                        IssueKind::Contamination => kinds.contamination += 1,
                        IssueKind::Structural => kinds.structural += 1,
                        IssueKind::Starvation => kinds.starvation += 1,
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
        let refusal = policy.acceptance_gates.iter().find_map(|g| g.refusal(layout));
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
#[derive(Debug, Clone)]
pub struct Firewall {
    pub name: &'static str,
    pub receipt: &'static str,
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
}

impl GateContext<'_> {
    /// `Some(accepted)` when the incumbent produced, `None` when it
    /// produced nothing.
    pub fn incumbent_accepted(&self) -> Option<bool> {
        self.incumbent.and_then(|i| self.prior.get(i).copied().flatten())
    }

    /// Whether any EARLIER registration produced an accepted layout.
    /// Order-sensitive by construction — registration order is policy
    /// data, and `size-split-2`'s gate ("native and k1 both failed to
    /// land an accepted layout") is exactly this predicate over the two
    /// producers registered before it.
    pub fn any_prior_accepted(&self) -> bool {
        self.prior.contains(&Some(true))
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
    /// Everything always ranked (not today's policy; here so the rule is
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
    /// unread. Measured live on `electronic-circuit@35/s` from ore.
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
        matches!(self.kind, StageKind::QualityKeyPairwise | StageKind::ComponentWiseFloor)
    }
}

/// The precedence chain as data.
pub struct SelectionProgram {
    pub stages: Vec<StageSpec>,
    pub admission: AdmissionRule,
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
    pub contamination_weight: usize,
    pub firewalls: Vec<Firewall>,
    pub program: SelectionProgram,
    pub producers: Vec<ProducerRegistration>,
}

impl SelectionPolicy {
    pub fn kind_of(&self, category: &str) -> IssueKind {
        self.error_kind_classes.get(category).copied().unwrap_or(IssueKind::Starvation)
    }

    /// Index of the incumbent registration.
    pub fn incumbent_index(&self) -> Option<usize> {
        self.producers.iter().position(|p| p.incumbent)
    }

    /// **Today's policy**, transcribed from the source sites RFC-070's
    /// Phase 1b specification anchors. Every value here is a
    /// transcription, not a redesign: `policy_replay` is the proof, and
    /// a wrong transcription shows up as a diverging cell rather than as
    /// a plausible-looking constant.
    pub fn current() -> Self {
        let excluded_warning_categories = crate::validate::SELECTION_EXCLUDED_WARNING_CATEGORIES
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        // `classify_errors`' match, as a table.
        let mut error_kind_classes = BTreeMap::new();
        for c in [
            "belt-item-isolation",
            "fluid-network",
            "pipe-isolation",
            "fluid-connectivity",
            "belt-junction",
        ] {
            error_kind_classes.insert(c.to_string(), IssueKind::Contamination);
        }
        for c in ["entity-overlap", "pipe-to-ground"] {
            error_kind_classes.insert(c.to_string(), IssueKind::Structural);
        }

        Self {
            excluded_warning_categories,
            error_kind_classes,
            acceptance_gates: vec![AcceptanceGate {
                name: "missing-balancer-template",
                layout_warning_substring: "balancer template",
            }],
            contamination_weight: 3,
            firewalls: vec![Firewall {
                name: "warning-recalibration-firewall",
                receipt: "#519/#520: the recalibration multiplied input-rate-delivery's \
                          counts ~10x and letting an unanchored model steer selection \
                          shipped a physically over-stamped winner on stacking_ec_60s. \
                          The input-rate-delivery exemption was LIFTED 2026-08-07 (it \
                          counts again); belt-detour remains excluded. Receipts: \
                          docs/validator-trust.md hole 2.",
            }],
            program: current_program(),
            producers: current_producers(),
        }
    }
}

/// Today's five stages, in precedence order.
fn current_program() -> SelectionProgram {
    SelectionProgram {
        admission: AdmissionRule::ScopedOnIncumbentRefusal,
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
                }),
                on_incumbent_win: ChainBehavior::Terminate,
            },
            StageSpec {
                // The degraded fallback, and the `ec30` trap #694
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
                c.opts.max_belt_tier.as_deref().is_none_or(|t| t == "express-transport-belt")
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
pub fn decide(profiles: &[IssueProfile], policy: &SelectionPolicy) -> Option<Decision> {
    assert_eq!(
        profiles.len(),
        policy.producers.len(),
        "one profile per registration: the profile vector is keyed by registration order, \
         so a length mismatch would silently rank one producer's measurement under \
         another's policy"
    );

    let incumbent = policy.incumbent_index();
    let incumbent_produced = incumbent.is_some_and(|i| profiles[i].produced());

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
            StageKind::ComponentWiseFloor => component_wise_floor_stage(profiles, policy, incumbent),
            StageKind::TieredRank(spec) => {
                tiered_rank_stage(profiles, spec, &admitted, incumbent_produced)
            }
        };
        match outcome {
            StageOutcome::Winner(i) => return Some(Decision { winner: i, stage: stage.tag }),
            StageOutcome::HeldIncumbent(i) => match stage.on_incumbent_win {
                ChainBehavior::Terminate => {
                    return Some(Decision { winner: i, stage: stage.tag })
                }
                ChainBehavior::DeferToRemainingPairwiseStages => {
                    held = Some(Decision { winner: i, stage: stage.tag });
                }
            },
            StageOutcome::NoOpinion => {}
        }
    }
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
        None => StageOutcome::Winner(rival),
        Some(inc) => match profiles[inc].kinds {
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
    best.map(|(i, _)| StageOutcome::Winner(i)).unwrap_or(StageOutcome::NoOpinion)
}

/// The tiered rank: admission by `spec`, ordering by whether the
/// incumbent produced.
fn tiered_rank_stage(
    profiles: &[IssueProfile],
    spec: &RankSpec,
    admitted: &[usize],
    incumbent_produced: bool,
) -> StageOutcome {
    let order = if incumbent_produced { spec.success_order } else { spec.refusal_order };
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
            // Strictly-better-only, so an exact tie keeps the EARLIER
            // registration: the list is a preference order.
            Some(b) if ranks_ahead(profiles, i, b, order) => i,
            Some(b) => b,
        });
    }
    best.map(StageOutcome::Winner).unwrap_or(StageOutcome::NoOpinion)
}

/// Does `a` rank strictly ahead of `b` under `order`?
fn ranks_ahead(profiles: &[IssueProfile], a: usize, b: usize, order: RankOrder) -> bool {
    let score = |i: usize| profiles[i].score.unwrap_or(f64::NEG_INFINITY);
    // `usize::MAX` for an unmeasured candidate mirrors v1's unclean
    // `warn_key`: a candidate with no warning key sorts last, never
    // first.
    let warn = |i: usize| profiles[i].warning_key().unwrap_or(usize::MAX);
    let score_ahead = || score(a).partial_cmp(&score(b)) == Some(std::cmp::Ordering::Greater);
    match order {
        RankOrder::RegistrationOrder => false,
        RankOrder::ScoreDesc => score_ahead(),
        RankOrder::WarningsAscThenScoreDesc => match warn(a).cmp(&warn(b)) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => score_ahead(),
        },
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
        IssueCounts { errors, selection_warnings, layout_warnings }
    }

    fn kinds(contamination: usize, starvation: usize, structural: usize) -> ErrorKindCounts {
        ErrorKindCounts { contamination, starvation, structural }
    }

    /// Seven `not-run` profiles, to be filled in per test.
    fn blank() -> Vec<IssueProfile> {
        (0..7).map(|_| not_run()).collect()
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

    #[test]
    fn refuse_on_error_is_asymmetric_and_that_asymmetry_is_load_bearing() {
        // Preserving WHICH producers self-refuse is required for parity:
        // `ec30` ships an error-laden native through the fallback stage
        // precisely because native does NOT carry the gate.
        let p = SelectionPolicy::current();
        let refusing: Vec<&str> =
            p.producers.iter().filter(|r| r.refuse_on_error).map(|r| r.name).collect();
        assert_eq!(refusing, vec!["cell-composed", "direct-insertion", "horizontal-stack"]);
    }

    #[test]
    fn only_di_carries_the_equal_and_denser_arm() {
        let p = SelectionPolicy::current();
        let arms: Vec<&str> =
            p.producers.iter().filter(|r| r.equal_and_denser).map(|r| r.name).collect();
        assert_eq!(arms, vec!["direct-insertion"]);
        let scoped: Vec<&str> = p.producers.iter().filter(|r| r.scoped).map(|r| r.name).collect();
        assert_eq!(scoped, vec!["direct-insertion", "horizontal-stack"]);
    }

    #[test]
    fn excluded_warning_categories_is_belt_detour_alone() {
        // The two #632 B6 demotions left the set by DELETION (#684).
        let p = SelectionPolicy::current();
        assert_eq!(
            p.excluded_warning_categories.iter().map(String::as_str).collect::<Vec<_>>(),
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
        assert_eq!(p.kind_of("a-category-that-does-not-exist"), IssueKind::Starvation);
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
        assert_eq!(d, Decision { winner: MERGE_TAP, stage: SelectionStage::MergeTap });
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
        assert_eq!(d.winner, NATIVE, "structural must dominate the weighted functional total");
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
            Decision { winner: NATIVE, stage: SelectionStage::MergeTap },
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
        assert_eq!(d, Decision { winner: DI, stage: SelectionStage::ScopedPairwise });
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
            Decision { winner: NATIVE, stage: SelectionStage::MergeTap },
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
        assert_ne!(d.winner, DI, "a layout-warning regression must not hide behind a \
                                  validator-warning improvement");
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
            Decision { winner: DI, stage: SelectionStage::ScopedPairwise },
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
        assert_ne!(d.winner, DI, "a sub-epsilon score difference is a tie, and ties keep native");
    }

    #[test]
    fn an_unaccepted_scoped_candidate_never_displaces_the_incumbent() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[NATIVE].counts = Some(counts(2, 5, 3));
        ps[DI] = produced(9.0, false);
        ps[DI].counts = Some(counts(0, 0, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_ne!(d.winner, DI, "`accepted` is a hard constraint the issue channels cannot see");
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
        assert_eq!(d.winner, HS, "horizontal is strictly better than DI on the floor");

        // Tie on the floor: the far better score must NOT rescue
        // horizontal — the earlier registration keeps it.
        ps[HS].counts = Some(counts(0, 2, 0));
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(d.winner, DI);
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
        assert_eq!(d, Decision { winner: NATIVE, stage: SelectionStage::BestErrorFree });
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
            Decision { winner: DI, stage: SelectionStage::BestErrorFree },
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
            Decision { winner: CELLS, stage: SelectionStage::BestErrorFree },
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
        assert_eq!(d, Decision { winner: CELLS, stage: SelectionStage::BestErrorFree });
    }

    #[test]
    fn without_counts_the_error_free_tier_is_empty_and_best_accepted_decides() {
        // v1's laziness, reproduced by the gap rule: a single-layout
        // solve never computes `clean_flags` at all.
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(d, Decision { winner: NATIVE, stage: SelectionStage::BestAccepted });
    }

    #[test]
    fn best_accepted_takes_the_highest_score_ties_to_the_earlier() {
        let mut ps = blank();
        ps[NATIVE] = produced(1.0, true);
        ps[K1] = produced(2.0, true);
        assert_eq!(decide(&ps, &SelectionPolicy::current()).unwrap().winner, K1);
        ps[K1] = produced(1.0, true);
        let d = decide(&ps, &SelectionPolicy::current()).unwrap();
        assert_eq!(d, Decision { winner: NATIVE, stage: SelectionStage::BestAccepted });
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
            Decision { winner: NATIVE, stage: SelectionStage::FirstProduced },
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
        let fenced = &SRC[begin..end];
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
        };
        // The incumbent is always eligible.
        assert_eq!(policy.producers[NATIVE].gate.evaluate(&ctx), GateVerdict::Eligible);
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
            })
        };
        assert_eq!(excluded_by(&prior), GateVerdict::Eligible);
        prior[K1] = Some(true);
        assert_eq!(excluded_by(&prior), GateVerdict::Excluded("no-earlier-producer-accepted"));
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
        let profile = IssueProfile::measure(&layout, &sr, &SelectionPolicy::current());
        assert_eq!(profile.accepted, Some(false));
        assert!(profile.accepted_reason.unwrap().contains("missing-balancer-template"));
        // The layout channel is counted separately from the validator's.
        assert_eq!(profile.counts.unwrap().layout_warnings, 1);
    }
}
