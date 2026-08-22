//! "Is this candidate no worse than the incumbent?" — one primitive, per
//! `docs/validator-reporting.md`'s discipline that comparing issue counts
//! by category is a nine-time-recurring failure mode (`{"category": 1}`
//! reads alike for 2 and 218 instances; churn on one row nets against a fix
//! on another and passes).
//!
//! Before this module the question was answered three incompatible ways:
//! `bus::compaction::search_snake_fold`'s `profile` closure (per-category
//! count diff — blind to intra-category churn; `bus::compaction` itself was
//! deleted 2026-08-14, #632 A2 — this is a historical example, not a live
//! pointer), `bus::decomposition_search::score_layout` (a single hard-gated
//! category, absolute `count > 0` rather than a comparison against any
//! baseline), and ad-hoc corpus comparisons in RFC dry sweeps that never
//! lived in-tree at all. [`never_worse`] is the
//! structural fix: it always computes the same positioned new/resolved/
//! matched breakdown ([`CategoryOutcome`]), and lets a per-category
//! [`GatePolicy`] decide how that breakdown turns into a pass/fail —
//! instead of every caller re-deriving its own comparison and re-introducing
//! the count-collapse bug on its own schedule.
//!
//! This is [`crate::objective`]'s sibling: `objective` MEASURES (continuous
//! scores, for ranking candidates against each other); this module GATES
//! (a boolean per category, for admitting or refusing one candidate against
//! its own incumbent). Follow `objective.rs`'s documentation style — these
//! docs are transcriptions of design decisions, not independent restatements
//! of the code.
//!
//! ## Match tiers: why three, and why the caller must pick
//!
//! Comparing two issue lists positionally requires knowing where an issue's
//! position in `native` ENDS UP in `candidate`'s coordinate frame. Three
//! answers exist, in decreasing order of precision, and no single one covers
//! every transform in this codebase:
//!
//! - [`MatchTier::Provenance`]: the caller supplies a [`CorrespondenceMap`]
//!   (built from the transform's own geometry — `bus::compaction::
//!   fold_point_correspondence`, for the now-deleted snake fold, #632 A2,
//!   was the original motivating example) that maps a native tile position
//!   to where that tile landed in the candidate. The most precise tier, and
//!   the only one that survives a transform that both moves AND reorients
//!   geometry.
//! - [`MatchTier::Positional`]: no map, exact-position match. Correct only
//!   for transforms that substitute IN PLACE — `bus::compaction::
//!   undergroundify_straight_belts` (deleted along with the rest of that
//!   module, #632 A2) was the verified example: entities that survive the
//!   transform keep their exact `(x, y)` (only their entity type changes,
//!   surface belt -> underground), and entities the transform removes
//!   entirely (a run's interior tiles) have no candidate-side counterpart,
//!   which correctly reads as "resolved" rather than a false match. A
//!   transform that translates or mirrors geometry (the fold) is NOT safe
//!   at this tier — the exact positions simply won't recur, so every
//!   surviving issue would misread as new-plus-resolved.
//! - [`MatchTier::Count`]: no position is consulted at all, every issue in
//!   a category is fungible with every other. This was the historical
//!   behavior of both `search_snake_fold` (deleted) and `score_layout`,
//!   and remains the explicit last resort for transforms with neither a
//!   map nor an in-place guarantee — which is most of them; it is what
//!   `run_candidate_field` uses unconditionally now that no transform
//!   supplies a tier at all (`bus::candidate_runner`'s module doc). It is
//!   also the tier that CANNOT see the churn case this module exists to
//!   catch.
//!
//! `never_worse` takes the tier as an explicit argument rather than
//! inferring it from `Option<&CorrespondenceMap>` alone, because `None` is
//! ambiguous between Positional and Count — both describe "no map", and
//! only the caller knows whether its transform's positions are stable. If
//! `tier` is [`MatchTier::Provenance`] but `correspondence` is `None`
//! (a caller bug), every category degrades to the count-only comparison
//! (see below) rather than panicking — silently less precise, never wrong.
//!
//! ## Matching algorithm
//!
//! All positions are integer tile coordinates produced by deterministic
//! integer transforms (translation, 180-degree rotation) — never
//! floating-point geometry — so matching is EXACT tile equality, no
//! tolerance. Per category:
//!
//! - **Count tier**: every native and every candidate issue in the category
//!   goes into the "unpositioned" bucket regardless of whether it actually
//!   carries a position — position is deliberately not consulted. Matched
//!   count is `min(native, candidate)`; the tier cannot produce a positioned
//!   `new_issues`/`resolved_issues` list, only aggregate counts.
//! - **Positional/Provenance tiers**: first, every native issue's expected
//!   candidate-space position is resolved (identity for Positional, a map
//!   lookup for Provenance). If EVEN ONE native issue in the category can't
//!   be resolved — no position on the issue at all, or (Provenance) no map
//!   entry covering it — the WHOLE category falls back to the same
//!   count-only comparison Count tier uses, not just that one instance.
//!   This is a whole-category fallback rather than a per-issue one on
//!   purpose: leaving one unresolvable native issue out of the accounting
//!   while still positionally matching every other one has no honest
//!   answer for which candidate issue (if any) is that native issue's
//!   continuation — a real bug caught by this module's own test suite,
//!   where a single unmapped native issue made an untouched, identically-
//!   positioned candidate issue misread as a fresh regression. Once every
//!   native issue resolves, each is matched against candidate issues in the
//!   same category at its exact expected position: found -> `matched`; not
//!   found -> `resolved_issues` (an improvement). Every candidate issue not
//!   consumed as a match is `new_issues` (if positioned) or folds into the
//!   unpositioned-candidate count (if it genuinely carries no coordinate —
//!   possible only when every native issue resolved, so this bucket is
//!   compared against an `unpositioned_native` that is always 0 in that
//!   branch).
//!
//! ## Gate policy: how a diff becomes a pass/fail
//!
//! [`GatePolicy`] is a second, independent axis from the match tier: the
//! tier decides HOW the diff is computed (uniformly, for every category,
//! every call); the policy decides HOW a category's diff is read for
//! pass/fail, per category:
//!
//! - [`GatePolicy::GateInstances`]: regress if `new_issues` is non-empty, or
//!   the unpositioned-candidate count exceeds the unpositioned-native count.
//!   Under [`MatchTier::Count`] this is arithmetically identical to
//!   `GateCount` (there is no finer data available to disagree with it).
//! - [`GatePolicy::GateCount`]: regress if the category's raw candidate
//!   count exceeds its raw native count — ignoring the positioned
//!   breakdown even when one was computed. This is what makes
//!   [`Policy::fold`] reproduce today's `search_snake_fold` exactly even
//!   though `never_worse` always computes the finer diff: `GateCount`
//!   simply doesn't consult it for the pass/fail bit.
//! - [`GatePolicy::ReportOnly`]: never regresses; the diff is still computed
//!   and stored for observability.
//!
//! ## Severity channels (RFC-070 Phase 1b)
//!
//! The category-count model above is severity-BLIND: an error and a warning
//! in one category are fungible counts. That is exactly the #519/#644 failure
//! class, so [`CategoryOutcome`] now also carries the per-category
//! `{errors, warnings}` split on both sides, and [`Policy`] carries the
//! selection-excluded warning categories.
//!
//! **Additive by construction.** The channels are RECORDED, never consulted
//! by `regressed` or `pass`; [`Policy::fold`] and [`Policy::decomposition`]
//! keep their exact meaning, so `bus::candidate_runner` and the RFC-068
//! celldb harness (`tests/celldb_template.rs`, `tests/candidate_runner.rs`)
//! stay green unmodified — the celldb-harness-green obligation RFC-070 took
//! on in exchange for leaving RFC-068 P1 owner-gated. The severity-blind
//! path remains available to RFC-068 until its own campaign migrates.

use std::collections::{BTreeMap, BTreeSet};

use rustc_hash::FxHashMap;

use crate::validate::{Severity, ValidationIssue};

/// Maps a tile position in one layout's coordinate frame to its position in
/// another. Built by the transform itself (`bus::compaction::
/// fold_point_correspondence`, for the snake fold, was the original
/// example — that module is deleted, #632 A2), never guessed at by this
/// module — a wrong entry here silently mismatches two unrelated issues.
///
/// Exact tile lookup, no tolerance: every position this module deals with
/// comes from an integer-grid transform, never continuous geometry.
#[derive(Debug, Clone, Default)]
pub struct CorrespondenceMap {
    map: FxHashMap<(i32, i32), (i32, i32)>,
}

impl CorrespondenceMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from `(old, new)` position pairs, e.g. one per surviving
    /// entity's anchor tile, or one per tile in a fully-enumerated point
    /// transform (`fold_point_correspondence` uses the latter — see its own
    /// docs for why a per-entity map isn't actually the finer-grained
    /// option here).
    pub fn from_pairs(pairs: impl IntoIterator<Item = ((i32, i32), (i32, i32))>) -> Self {
        Self { map: pairs.into_iter().collect() }
    }

    pub fn insert(&mut self, from: (i32, i32), to: (i32, i32)) {
        self.map.insert(from, to);
    }

    /// Where `from` lands in the candidate's coordinate frame, or `None` if
    /// this map has no entry for it (out of the transform's domain, or the
    /// map is simply incomplete for this caller's purposes).
    pub fn get(&self, from: (i32, i32)) -> Option<(i32, i32)> {
        self.map.get(&from).copied()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Every `from` key this map has an entry for. Purely additive (P2b,
    /// RFC-064): `never_worse` itself only ever calls `get`, so no existing
    /// caller depends on iteration existing at all, let alone its order.
    /// Added so `bus::candidate_runner::compose_chain` can walk one
    /// transform's whole domain through the next map in a chain without
    /// the caller having to track that domain separately.
    pub fn keys(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.map.keys().copied()
    }
}

/// Which strategy [`never_worse`] used to decide whether a positioned native
/// issue's problem persists in the candidate. Always recorded on the
/// resulting [`Verdict`] — see the module docs' "Match tiers" section for
/// what each one assumes and when that assumption is safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchTier {
    /// Native positions are mapped through a caller-supplied
    /// [`CorrespondenceMap`] before matching.
    Provenance,
    /// Native positions are matched against candidate positions unchanged
    /// — correct only when the transform substitutes in place.
    Positional,
    /// No position is consulted; every issue in a category is fungible.
    Count,
}

/// Per-category policy: how a computed diff turns into a pass/fail bit.
/// See the module docs' "Gate policy" section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatePolicy {
    /// Regress on any new positioned instance (or a net increase in
    /// unpositioned instances the tier couldn't attribute).
    GateInstances,
    /// Regress only if the category's raw count increased, ignoring the
    /// positioned breakdown.
    GateCount,
    /// Never regress; still computed and recorded.
    ReportOnly,
}

/// A default policy plus per-category overrides. Categories not named in
/// `overrides` use `default`.
#[derive(Debug, Clone)]
pub struct Policy {
    pub default: GatePolicy,
    pub overrides: BTreeMap<String, GatePolicy>,
    /// Warning categories that do not participate in selection —
    /// `validate::SELECTION_EXCLUDED_WARNING_CATEGORIES` as policy data
    /// rather than a hardcoded constant (RFC-070 Phase 1b).
    ///
    /// Read ONLY by [`Verdict::candidate_selection_warnings`] and its native
    /// twin; it does not touch `regressed` or `pass`, so every existing
    /// preset behaves exactly as before with the set left empty.
    pub excluded_warning_categories: BTreeSet<String>,
    /// Whether this policy DECLARED a selection scope at all. An empty
    /// `excluded_warning_categories` is ambiguous between "exclude
    /// nothing, deliberately" and "never thought about it", and the
    /// selection accessors must not answer for the second — see
    /// [`Verdict::candidate_selection_warnings`]. Set by
    /// [`Policy::selection`], [`Policy::with_live_selection_exclusions`]
    /// and [`Policy::with_excluded_warning_category`].
    pub selection_scoped: bool,
}

impl Policy {
    pub fn new(default: GatePolicy) -> Self {
        Self {
            default,
            overrides: BTreeMap::new(),
            excluded_warning_categories: BTreeSet::new(),
            selection_scoped: false,
        }
    }

    pub fn with_override(mut self, category: impl Into<String>, policy: GatePolicy) -> Self {
        self.overrides.insert(category.into(), policy);
        self
    }

    /// Exclude one warning category from the selection-scoped warning
    /// count. Excluding a category does NOT stop it being diffed or gated
    /// — it only removes it from the selection channel.
    pub fn with_excluded_warning_category(mut self, category: impl Into<String>) -> Self {
        self.excluded_warning_categories.insert(category.into());
        self.selection_scoped = true;
        self
    }

    /// The live selection exclusions, from the canonical constant. Today
    /// that is `belt-detour` alone: the two #632 B6 demotions left the set
    /// by DELETION (#684).
    pub fn with_live_selection_exclusions(mut self) -> Self {
        for c in crate::validate::SELECTION_EXCLUDED_WARNING_CATEGORIES {
            self.excluded_warning_categories.insert(c.to_string());
        }
        self.selection_scoped = true;
        self
    }

    /// The preset for selection-scoped comparisons: gate nothing, carry
    /// the live exclusions, and declare the scope so
    /// [`Verdict::candidate_selection_warnings`] answers. Named so the
    /// correct setup is a thing you reach for rather than a step you
    /// remember (#698 review round 3).
    ///
    /// **Its `pass` is always `true`**, because it gates nothing — this
    /// preset supplies COUNTS, and the pass/fail bit is not one of its
    /// outputs. Reading both from one verdict would read a gate that was
    /// never asked to gate; add overrides if you want it to (#698 review
    /// round 5).
    pub fn selection() -> Self {
        Self::new(GatePolicy::ReportOnly).with_live_selection_exclusions()
    }

    pub fn policy_for(&self, category: &str) -> GatePolicy {
        self.overrides.get(category).copied().unwrap_or(self.default)
    }

    /// `search_snake_fold`'s HISTORICAL (pre-refit) accept test: every
    /// category gated, compared by raw count, no positional information
    /// consulted. This preserved the semantics of the per-category count
    /// profile that function used before it was refit onto this module.
    /// `search_snake_fold` and the rest of `bus::compaction` were deleted
    /// 2026-08-14 (#632 A2, owner call) — this preset is now the ONLY
    /// record of that history, kept alive because it is a reusable,
    /// well-understood Count-tier gate shape (every category gated, no
    /// positional information required) rather than because anything still
    /// literally calls it "the fold policy". Current live consumer: the
    /// RFC-068 celldb campaign
    /// (`crates/core/tests/celldb_template.rs`) passes it to
    /// `run_candidate_field`, whose transform-free candidates are always
    /// verdicted at [`MatchTier::Count`] — the same tier this preset was
    /// designed to pair with, so nothing about pairing it with a
    /// positional tier not mattering (see the note that used to be here)
    /// is exercised by that call site either.
    pub fn fold() -> Self {
        Self::new(GatePolicy::GateCount)
    }

    /// `score_layout`'s hard gate as it exists today
    /// (`bus::decomposition_search::score_layout`): only
    /// `missing-balancer-template` blocks a candidate, every other category
    /// is informational. Note `score_layout` itself checks an ABSOLUTE
    /// `count > 0` on the candidate, not a comparison against any native
    /// baseline — decomposition candidates are scored standalone. Routing
    /// that decision through `never_worse` with this preset reproduces it
    /// exactly PROVIDED the caller compares against a `native` issue list
    /// with zero `missing-balancer-template` issues (true today, since
    /// there is no native to compare against at all — an empty slice
    /// serves as that zero baseline). `GateCount`'s `candidate_count >
    /// native_count` with `native_count == 0` is exactly `candidate_count
    /// > 0`.
    pub fn decomposition() -> Self {
        Self::new(GatePolicy::ReportOnly).with_override("missing-balancer-template", GatePolicy::GateCount)
    }
}

/// One category's computed diff, plus the gate policy that was applied to
/// it and the resulting pass/fail bit. Always populated the same way
/// regardless of `policy` — a `ReportOnly` category's diff is exactly as
/// detailed as a `GateInstances` one, it is just never allowed to fail the
/// overall [`Verdict`].
#[derive(Debug, Clone)]
pub struct CategoryOutcome {
    pub policy: GatePolicy,
    pub native_count: usize,
    pub candidate_count: usize,
    /// The severity split of `native_count` (RFC-070 Phase 1b).
    /// Recorded, never gated on: `regressed` is computed exactly as it
    /// was before these fields existed.
    pub native_errors: usize,
    pub native_warnings: usize,
    /// The severity split of `candidate_count`.
    pub candidate_errors: usize,
    pub candidate_warnings: usize,
    /// Candidate issues not attributable to any native issue at the tier's
    /// granularity — positioned instances, per `docs/validator-reporting.md`
    /// rule 1. These are the actual regressions this module exists to
    /// surface; empty under [`MatchTier::Count`] (it produces no positioned
    /// attribution at all).
    pub new_issues: Vec<ValidationIssue>,
    /// Native issues that no longer appear (at their expected candidate
    /// position) — improvements. Also empty under [`MatchTier::Count`].
    pub resolved_issues: Vec<ValidationIssue>,
    /// Native issues found at their expected candidate position — carried
    /// over unchanged.
    pub matched: usize,
    /// Equal to `native_count` under [`MatchTier::Count`] or the
    /// whole-category positional-fallback case (see module docs); `0`
    /// whenever positional matching actually ran, since that only happens
    /// once every native issue in the category resolved to a position.
    pub unpositioned_native: usize,
    /// Candidate issues with no position, left over after positioned
    /// matching. Equal to `candidate_count` under `MatchTier::Count` or the
    /// fallback case.
    pub unpositioned_candidate: usize,
    /// This category's contribution to [`Verdict::pass`], per `policy`.
    pub regressed: bool,
}

/// Per-category outcomes plus the overall pass/fail bit — a datum, not a
/// bool, per this module's whole reason to exist: a caller that only reads
/// `pass` gets today's behavior, but every regression is inspectable and
/// positioned for whoever needs to know WHY.
#[derive(Debug, Clone)]
pub struct Verdict {
    pub tier: MatchTier,
    pub categories: BTreeMap<String, CategoryOutcome>,
    pub pass: bool,
    /// The policy's selection exclusions, carried so the severity
    /// accessors below are answerable from the verdict alone rather than
    /// requiring the caller to hold onto the policy that produced it.
    pub excluded_warning_categories: BTreeSet<String>,
    /// Whether the governing policy declared a selection scope. See
    /// [`Policy::selection_scoped`].
    pub selection_scoped: bool,
}

impl Verdict {
    /// Category names whose outcome contributed a regression to `!pass`.
    pub fn regressed_categories(&self) -> impl Iterator<Item = &str> {
        self.categories.iter().filter(|(_, o)| o.regressed).map(|(cat, _)| cat.as_str())
    }

    /// Total `Severity::Error` count on the candidate side.
    ///
    /// A WHOLE-SIDE total, not a regression count: it includes errors
    /// that matched a native one and were carried over unchanged. For
    /// "what got worse", read `regressed_categories` or
    /// [`Verdict::all_new_issues`] — summing this against the native
    /// total overcounts on a partially-matched category (#698 review
    /// round 5).
    pub fn candidate_errors(&self) -> usize {
        self.categories.values().map(|o| o.candidate_errors).sum()
    }

    /// Total `Severity::Error` count on the native side.
    pub fn native_errors(&self) -> usize {
        self.categories.values().map(|o| o.native_errors).sum()
    }

    /// Total `Severity::Warning` count on the candidate side, every
    /// category included. Always answerable — no selection scope
    /// involved.
    pub fn candidate_warnings(&self) -> usize {
        self.categories.values().map(|o| o.candidate_warnings).sum()
    }

    /// The native side of the raw warning channel.
    pub fn native_warnings(&self) -> usize {
        self.categories.values().map(|o| o.native_warnings).sum()
    }

    /// Candidate warnings minus the policy's excluded categories —
    /// `validate::selection_warning_count`'s semantics over a verdict.
    ///
    /// **`None` when the governing policy never declared a selection
    /// scope**, which is the case for [`Policy::fold`] and
    /// [`Policy::decomposition`]: both deliberately carry no exclusions,
    /// and returning a number there would silently count `belt-detour`
    /// — the opposite of the selection-scoped figure, and precisely the
    /// starvation channel #519/#520 firewalled (#698 review rounds 2-3).
    ///
    /// A gap, not a zero and not a plausible wrong number, per the rule
    /// this campaign's instruments run on. Declare the scope with
    /// [`Policy::selection`] or
    /// [`Policy::with_excluded_warning_category`] to get a value —
    /// including a deliberately empty exclusion set, which is a
    /// declaration too.
    pub fn candidate_selection_warnings(&self) -> Option<usize> {
        self.selection_warnings(|o| o.candidate_warnings)
    }

    /// The native side of the same channel, with the same `None` rule.
    pub fn native_selection_warnings(&self) -> Option<usize> {
        self.selection_warnings(|o| o.native_warnings)
    }

    fn selection_warnings(&self, pick: fn(&CategoryOutcome) -> usize) -> Option<usize> {
        if !self.selection_scoped {
            return None;
        }
        Some(
            self.categories
                .iter()
                .filter(|(cat, _)| !self.excluded_warning_categories.contains(*cat))
                .map(|(_, o)| pick(o))
                .sum(),
        )
    }

    /// Every new positioned issue across every category, regardless of
    /// that category's gate policy — for reporting/trace purposes, not for
    /// re-deriving `pass` (use `regressed_categories` for that).
    pub fn all_new_issues(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.categories.values().flat_map(|o| o.new_issues.iter())
    }
}

/// Compare `native`'s and `candidate`'s validation issues and decide
/// whether the candidate is no worse, per `policy` and `tier`. See the
/// module docs for the full algorithm; in short: every category present in
/// either list gets a [`CategoryOutcome`] computed the same way (governed by
/// `tier`), and `policy` decides which categories' outcomes can fail the
/// overall `pass`.
///
/// `tier` is required explicitly rather than inferred from
/// `correspondence.is_some()`, because `None` is ambiguous between
/// [`MatchTier::Positional`] (no map needed, positions are stable) and
/// [`MatchTier::Count`] (positions are meaningless, don't try) — only the
/// caller knows which is true of its transform. `correspondence` is
/// ignored unless `tier == MatchTier::Provenance`.
pub fn never_worse(
    native: &[ValidationIssue],
    candidate: &[ValidationIssue],
    policy: &Policy,
    tier: MatchTier,
    correspondence: Option<&CorrespondenceMap>,
) -> Verdict {
    let mut categories_seen: Vec<&str> = Vec::new();
    for i in native.iter().chain(candidate.iter()) {
        if !categories_seen.contains(&i.category.as_str()) {
            categories_seen.push(&i.category);
        }
    }

    let mut categories: BTreeMap<String, CategoryOutcome> = BTreeMap::new();
    for cat in categories_seen {
        let native_in_cat: Vec<&ValidationIssue> = native.iter().filter(|i| i.category == cat).collect();
        let candidate_in_cat: Vec<&ValidationIssue> =
            candidate.iter().filter(|i| i.category == cat).collect();
        let gate = policy.policy_for(cat);
        let outcome = diff_category(&native_in_cat, &candidate_in_cat, tier, correspondence, gate);
        categories.insert(cat.to_string(), outcome);
    }

    let pass = !categories.values().any(|o| o.regressed);
    Verdict {
        tier,
        categories,
        pass,
        excluded_warning_categories: policy.excluded_warning_categories.clone(),
        selection_scoped: policy.selection_scoped,
    }
}

/// `(errors, warnings)` for one category's issues.
fn severity_split(issues: &[&ValidationIssue]) -> (usize, usize) {
    let errors = issues.iter().filter(|i| i.severity == Severity::Error).count();
    (errors, issues.len() - errors)
}

/// The position a native issue's problem is expected to occupy in the
/// candidate, or `None` if this tier/map cannot say. `MatchTier::Count`
/// never calls this — see `diff_category`.
fn expected_position(
    n: &ValidationIssue,
    tier: MatchTier,
    correspondence: Option<&CorrespondenceMap>,
) -> Option<(i32, i32)> {
    let (x, y) = (n.x?, n.y?);
    match tier {
        MatchTier::Provenance => correspondence.and_then(|m| m.get((x, y))),
        MatchTier::Positional => Some((x, y)),
        MatchTier::Count => unreachable!("Count tier never resolves positions"),
    }
}

/// The count-only shape of a `CategoryOutcome`: no positional attribution,
/// every issue in the category fungible with every other. Used both for
/// `MatchTier::Count` itself and as the fallback described in
/// `diff_category`'s docs.
fn count_outcome(
    native: &[&ValidationIssue],
    candidate: &[&ValidationIssue],
    policy: GatePolicy,
) -> CategoryOutcome {
    let (native_count, candidate_count) = (native.len(), candidate.len());
    let (native_errors, native_warnings) = severity_split(native);
    let (candidate_errors, candidate_warnings) = severity_split(candidate);
    let matched = native_count.min(candidate_count);
    let regressed = match policy {
        GatePolicy::ReportOnly => false,
        GatePolicy::GateCount | GatePolicy::GateInstances => candidate_count > native_count,
    };
    CategoryOutcome {
        policy,
        native_count,
        candidate_count,
        native_errors,
        native_warnings,
        candidate_errors,
        candidate_warnings,
        new_issues: Vec::new(),
        resolved_issues: Vec::new(),
        matched,
        unpositioned_native: native_count,
        unpositioned_candidate: candidate_count,
        regressed,
    }
}

fn diff_category(
    native: &[&ValidationIssue],
    candidate: &[&ValidationIssue],
    tier: MatchTier,
    correspondence: Option<&CorrespondenceMap>,
    policy: GatePolicy,
) -> CategoryOutcome {
    let native_count = native.len();
    let candidate_count = candidate.len();
    let (native_errors, native_warnings) = severity_split(native);
    let (candidate_errors, candidate_warnings) = severity_split(candidate);

    if tier == MatchTier::Count {
        return count_outcome(native, candidate, policy);
    }

    // Positional or Provenance: resolve every native issue's expected
    // candidate-space position UP FRONT. If any one of them can't be
    // resolved (no position on the issue at all, or — Provenance only — no
    // map entry covering it), positional attribution for the category AS A
    // WHOLE is unsound and this falls back to `count_outcome`, exactly as
    // `MatchTier::Count` would.
    //
    // This is a whole-category fallback, not a per-issue one, because
    // per-issue attribution has no honest answer for "which candidate issue
    // corresponds to the one native issue we couldn't map": leaving that
    // native issue out of the accounting entirely while still matching
    // every OTHER native issue positionally would let an unrelated,
    // perfectly real candidate issue get blamed as "new" merely because it
    // happened to be the one left over — an asymmetry a churn-detection
    // test in this module's own suite caught (a single unmapped native
    // issue made an identical, unmoved candidate issue read as a fresh
    // regression). Falling back for the whole category is the conservative,
    // symmetric choice: never claim precision the inputs don't support.
    let expected: Vec<Option<(i32, i32)>> =
        native.iter().map(|n| expected_position(n, tier, correspondence)).collect();
    if expected.iter().any(Option::is_none) {
        return count_outcome(native, candidate, policy);
    }

    // Every native issue resolved to a position. Consume candidate issues
    // against them exactly-once (a `Vec` with removal, not a count map, so
    // two issues at the same position are matched one-for-one rather than
    // collapsing back into the count-tier's blindness).
    let mut candidate_pool: Vec<&ValidationIssue> = candidate.to_vec();
    let mut resolved_issues: Vec<ValidationIssue> = Vec::new();
    let mut matched = 0usize;

    for (n, pos) in native.iter().zip(expected.into_iter().map(Option::unwrap)) {
        if let Some(idx) = candidate_pool.iter().position(|c| c.x == Some(pos.0) && c.y == Some(pos.1)) {
            candidate_pool.remove(idx);
            matched += 1;
        } else {
            resolved_issues.push((*n).clone());
        }
    }

    // Leftover candidate issues: positioned ones are genuinely new
    // (nothing in native resolved to their position); positionless ones —
    // a legacy check that doesn't carry a coordinate — compare by count,
    // same as `unpositioned_native` below (always 0 here, since the
    // fallback above already caught any unresolved native issue).
    let mut new_issues: Vec<ValidationIssue> = Vec::new();
    let mut unpositioned_candidate = 0usize;
    for c in candidate_pool {
        if c.x.is_some() && c.y.is_some() {
            new_issues.push(c.clone());
        } else {
            unpositioned_candidate += 1;
        }
    }
    let unpositioned_native = 0usize;

    let regressed = match policy {
        GatePolicy::ReportOnly => false,
        GatePolicy::GateCount => candidate_count > native_count,
        GatePolicy::GateInstances => {
            !new_issues.is_empty() || unpositioned_candidate > unpositioned_native
        }
    };

    CategoryOutcome {
        policy,
        native_count,
        candidate_count,
        native_errors,
        native_warnings,
        candidate_errors,
        candidate_warnings,
        new_issues,
        resolved_issues,
        matched,
        unpositioned_native,
        unpositioned_candidate,
        regressed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::Severity;

    fn issue_at(category: &str, x: i32, y: i32) -> ValidationIssue {
        ValidationIssue::with_pos(Severity::Warning, category, "test issue", x, y)
    }

    fn issue_no_pos(category: &str) -> ValidationIssue {
        ValidationIssue::new(Severity::Warning, category, "test issue, no position")
    }

    // -----------------------------------------------------------------
    // Count tier
    // -----------------------------------------------------------------

    #[test]
    fn count_tier_nets_churn_to_zero() {
        // 3 resolved + 3 new in the same category is invisible to a count
        // diff: the totals are equal.
        let native = vec![issue_at("power", 0, 0), issue_at("power", 1, 0), issue_at("power", 2, 0)];
        let candidate =
            vec![issue_at("power", 10, 0), issue_at("power", 11, 0), issue_at("power", 12, 0)];
        let v = never_worse(&native, &candidate, &Policy::fold(), MatchTier::Count, None);
        assert!(v.pass, "count tier must net 3 resolved + 3 new to zero and pass");
        assert_eq!(v.categories["power"].matched, 3);
    }

    #[test]
    fn count_tier_flags_a_genuine_increase() {
        let native = vec![issue_at("power", 0, 0)];
        let candidate = vec![issue_at("power", 0, 0), issue_at("power", 1, 1)];
        let v = never_worse(&native, &candidate, &Policy::fold(), MatchTier::Count, None);
        assert!(!v.pass);
        assert_eq!(v.categories["power"].candidate_count, 2);
        assert_eq!(v.categories["power"].native_count, 1);
    }

    // -----------------------------------------------------------------
    // Positional / Provenance tiers: the churn case, pinned
    // -----------------------------------------------------------------

    /// This pair of tests (with `count_tier_nets_churn_to_zero` above) is
    /// this module's reason to exist: the SAME native/candidate pair passes
    /// at Count tier and fails at Positional tier, because the 3 "resolved"
    /// issues and 3 "new" issues sit at different positions rather than
    /// being the same 3 issues moved.
    #[test]
    fn positional_tier_catches_churn_count_tier_misses() {
        let native = vec![issue_at("power", 0, 0), issue_at("power", 1, 0), issue_at("power", 2, 0)];
        let candidate =
            vec![issue_at("power", 10, 0), issue_at("power", 11, 0), issue_at("power", 12, 0)];
        let policy = Policy::new(GatePolicy::GateInstances);
        let v = never_worse(&native, &candidate, &policy, MatchTier::Positional, None);
        assert!(!v.pass, "positional tier must see 3 new instances at unmatched positions and fail");
        let outcome = &v.categories["power"];
        assert_eq!(outcome.new_issues.len(), 3);
        assert_eq!(outcome.resolved_issues.len(), 3);
        assert_eq!(outcome.matched, 0);
    }

    #[test]
    fn positional_tier_matches_issues_at_unchanged_positions() {
        let native = vec![issue_at("power", 5, 5)];
        let candidate = vec![issue_at("power", 5, 5)];
        let policy = Policy::new(GatePolicy::GateInstances);
        let v = never_worse(&native, &candidate, &policy, MatchTier::Positional, None);
        assert!(v.pass);
        assert_eq!(v.categories["power"].matched, 1);
        assert!(v.categories["power"].new_issues.is_empty());
        assert!(v.categories["power"].resolved_issues.is_empty());
    }

    #[test]
    fn provenance_tier_catches_churn_via_map() {
        let native = vec![issue_at("power", 0, 0), issue_at("power", 1, 0), issue_at("power", 2, 0)];
        // Candidate issues sit where the map does NOT send any native
        // issue — a transform that genuinely introduced new problems,
        // rather than relocating the same three.
        let candidate =
            vec![issue_at("power", 100, 0), issue_at("power", 101, 0), issue_at("power", 102, 0)];
        let mut map = CorrespondenceMap::new();
        map.insert((0, 0), (0, 0));
        map.insert((1, 0), (1, 0));
        map.insert((2, 0), (2, 0));
        let policy = Policy::new(GatePolicy::GateInstances);
        let v = never_worse(&native, &candidate, &policy, MatchTier::Provenance, Some(&map));
        assert!(!v.pass);
        assert_eq!(v.categories["power"].new_issues.len(), 3);
        assert_eq!(v.categories["power"].resolved_issues.len(), 3);
    }

    #[test]
    fn provenance_tier_matches_through_moved_geometry() {
        // The same issue survives the transform, just at a different tile
        // — exactly what a fold's translation/mirror does to a persistent
        // problem. Provenance tier must see through that, unlike Positional.
        let native = vec![issue_at("power", 0, 0)];
        let candidate = vec![issue_at("power", 50, 50)];
        let mut map = CorrespondenceMap::new();
        map.insert((0, 0), (50, 50));
        let policy = Policy::new(GatePolicy::GateInstances);
        let v = never_worse(&native, &candidate, &policy, MatchTier::Provenance, Some(&map));
        assert!(v.pass, "provenance tier must match the issue through its mapped position");
        assert_eq!(v.categories["power"].matched, 1);
    }

    #[test]
    fn provenance_tier_without_a_map_degrades_to_unpositioned_bucket() {
        // Caller bug: tier says Provenance but passes no map. Must not
        // panic, and must not misattribute the identical, unmoved
        // candidate issue as "new" just because it couldn't be positionally
        // resolved — the whole category falls back to a count comparison
        // instead (see module docs), which correctly nets this pair to a
        // pass with the issue counted as matched.
        let native = vec![issue_at("power", 0, 0)];
        let candidate = vec![issue_at("power", 0, 0)];
        let policy = Policy::new(GatePolicy::GateInstances);
        let v = never_worse(&native, &candidate, &policy, MatchTier::Provenance, None);
        assert!(v.pass);
        assert_eq!(v.categories["power"].unpositioned_native, 1);
        assert_eq!(v.categories["power"].unpositioned_candidate, 1);
        assert_eq!(v.categories["power"].matched, 1);
        assert!(v.categories["power"].new_issues.is_empty());
    }

    #[test]
    fn unpositioned_issues_compare_by_count_under_any_tier() {
        let native = vec![issue_no_pos("solver")];
        let candidate = vec![issue_no_pos("solver"), issue_no_pos("solver")];
        let policy = Policy::new(GatePolicy::GateInstances);
        let v = never_worse(&native, &candidate, &policy, MatchTier::Positional, None);
        assert!(!v.pass, "an unpositioned category increase must still regress under GateInstances");
    }

    // -----------------------------------------------------------------
    // Policy presets
    // -----------------------------------------------------------------

    #[test]
    fn fold_preset_gates_every_category_by_raw_count() {
        let native = vec![issue_at("power", 0, 0)];
        let candidate = vec![issue_at("power", 0, 0), issue_at("power", 1, 1)];
        let v = never_worse(&native, &candidate, &Policy::fold(), MatchTier::Count, None);
        assert!(!v.pass);
        assert_eq!(v.categories["power"].policy, GatePolicy::GateCount);
    }

    #[test]
    fn decomposition_preset_ignores_new_warning_in_ungated_category() {
        let native: Vec<ValidationIssue> = vec![];
        let candidate = vec![issue_at("pole-connectivity", 5, 5)];
        let v = never_worse(&native, &candidate, &Policy::decomposition(), MatchTier::Count, None);
        assert!(v.pass, "an ungated category's new issue must not fail the verdict");
        assert_eq!(v.categories["pole-connectivity"].policy, GatePolicy::ReportOnly);
    }

    #[test]
    fn decomposition_preset_gates_on_missing_balancer_template() {
        let native: Vec<ValidationIssue> = vec![];
        let candidate = vec![issue_at("missing-balancer-template", 5, 5)];
        let v = never_worse(&native, &candidate, &Policy::decomposition(), MatchTier::Count, None);
        assert!(!v.pass, "missing-balancer-template must gate even from a zero baseline");
    }

    // -----------------------------------------------------------------
    // Severity channels (RFC-070 Phase 1b) — recorded, never gated on
    // -----------------------------------------------------------------

    fn error_at(category: &str, x: i32, y: i32) -> ValidationIssue {
        ValidationIssue::with_pos(Severity::Error, category, "test error", x, y)
    }

    #[test]
    fn severity_channels_split_a_category_that_carries_both() {
        let native = vec![issue_at("power", 0, 0)];
        let candidate = vec![error_at("power", 0, 0), issue_at("power", 1, 1)];
        let v = never_worse(&native, &candidate, &Policy::fold(), MatchTier::Count, None);
        let o = &v.categories["power"];
        assert_eq!((o.native_errors, o.native_warnings), (0, 1));
        assert_eq!((o.candidate_errors, o.candidate_warnings), (1, 1));
        assert_eq!(v.candidate_errors(), 1);
        assert_eq!(v.native_errors(), 0);
    }

    #[test]
    fn the_severity_split_survives_the_positional_tier() {
        // The positional branch builds its outcome by hand rather than
        // through `count_outcome`, so it is a second site that has to
        // populate the channels.
        let native = vec![error_at("belt-flow", 0, 0)];
        let candidate = vec![error_at("belt-flow", 0, 0), issue_at("belt-flow", 9, 9)];
        let policy = Policy::new(GatePolicy::GateInstances);
        let v = never_worse(&native, &candidate, &policy, MatchTier::Positional, None);
        let o = &v.categories["belt-flow"];
        assert_eq!((o.native_errors, o.native_warnings), (1, 0));
        assert_eq!((o.candidate_errors, o.candidate_warnings), (1, 1));
    }

    #[test]
    fn an_excluded_category_leaves_the_selection_channel_but_not_the_gate() {
        let native: Vec<ValidationIssue> = vec![];
        let candidate = vec![issue_at("belt-detour", 3, 3), issue_at("power", 4, 4)];
        let policy = Policy::new(GatePolicy::GateCount).with_live_selection_exclusions();
        let v = never_worse(&native, &candidate, &policy, MatchTier::Count, None);
        assert_eq!(
            v.candidate_selection_warnings(),
            Some(1),
            "belt-detour must not count toward the selection channel"
        );
        assert_eq!(
            v.categories["belt-detour"].candidate_warnings, 1,
            "…while still being recorded and diffed"
        );
        assert!(!v.pass, "and still gated: exclusion is a channel rule, not a gate rule");
    }

    #[test]
    fn an_undeclared_selection_scope_is_a_gap_not_a_number() {
        // The compat claim `Policy::fold`'s callers rest on…
        assert!(Policy::fold().excluded_warning_categories.is_empty());
        assert!(Policy::decomposition().excluded_warning_categories.is_empty());
        // …and the trap that removes: under those presets the accessor
        // would have counted `belt-detour`, i.e. returned the OPPOSITE
        // of the selection-scoped number to a caller who asked for it.
        let candidate = vec![issue_at("belt-detour", 0, 0)];
        let v = never_worse(&[], &candidate, &Policy::fold(), MatchTier::Count, None);
        assert_eq!(v.candidate_selection_warnings(), None);
        assert_eq!(v.native_selection_warnings(), None);
        assert_eq!(v.candidate_warnings(), 1, "the raw channel still answers");

        // Declaring the scope — even with an empty set — is what makes
        // it answerable.
        let declared = never_worse(&[], &candidate, &Policy::selection(), MatchTier::Count, None);
        assert_eq!(declared.candidate_selection_warnings(), Some(0));
    }

    // -----------------------------------------------------------------
    // CorrespondenceMap
    // -----------------------------------------------------------------

    #[test]
    fn correspondence_map_from_pairs_round_trips() {
        let map = CorrespondenceMap::from_pairs([((0, 0), (1, 1)), ((2, 2), (3, 3))]);
        assert_eq!(map.get((0, 0)), Some((1, 1)));
        assert_eq!(map.get((2, 2)), Some((3, 3)));
        assert_eq!(map.get((9, 9)), None);
        assert_eq!(map.len(), 2);
    }
}
