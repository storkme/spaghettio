//! Net-flow (LP) solver — see `docs/rfc-solver-net-flow.md`.
//!
//! Replaces the recursive tree walk's per-branch demand math with a single
//! linear program over the recipe graph:
//!
//! ```text
//! minimize   Σ w[i]·s[i]  +  ε_o·Σ o[i]  +  ε_m·Σ x[r]·energy[r]/speed[r]
//! subject to Σ_r net(i,r)·x[r] + s[i] − o[i] = target(i)   for every item i
//!            x, s, o ≥ 0
//! ```
//!
//! where `x[r]` is crafts/sec of recipe `r`, `s[i]` is external input rate
//! (eligible only for `available_inputs` and items with no producer in
//! scope), and `o[i]` is surplus (byproduct beyond internal demand).
//!
//! Two invariants matter for correctness and reproducibility:
//!
//! 1. **Netted coefficients.** `net(i,r)` is one scalar per (item, recipe).
//!    Six recipes have the same item on both sides (kovarex et al.) and
//!    `microlp::LinearExpr` panics on duplicate variables in a constraint.
//! 2. **Determinism.** Column order = recipes.json order; row order =
//!    first-seen item order; no wall-clock limits anywhere. Byte-identical
//!    output for identical input is a project-level contract (URL state,
//!    `.fls` snapshots).

use crate::models::{ItemFlow, MachineSpec, SelfLoopFlow, SolverResult};
use crate::recipe_db::{
    db, effective_crafting_speed, is_excluded_recipe, machine_can_run_recipe,
    machine_for_recipe_with_palette, MachinePalette, Recipe,
};
use crate::solver::SolverError;
use microlp::{ComparisonOp, OptimizationDirection, Problem, Variable};
use rustc_hash::{FxHashMap, FxHashSet};

/// Frozen cost table (docs/rfc-solver-net-flow.md, "Design → Formulation";
/// revision 1 logged 2026-07-10). Kill criterion 3 forbids retuning these to
/// pass cross-validation; any revision needs an RFC decision-log entry.
///
/// Weight ordering is load-bearing:
/// `w_default ≫ w_available ≫ eps_machine ≫ eps_surplus`.
/// Available inputs are cheap but NOT free — a strictly positive
/// `w_available` is what makes every form of surplus-laundering
/// unprofitable (converting a high-rate byproduct into a lower-rate item
/// always consumes extra raw inputs, so it now always costs more than the
/// ε_o it saves). With inputs free, three exploit variants showed up in
/// Phase 0: sink chains outside the demand closure, sink chains through
/// demanded items, and overdriving legitimate recipes past demand.
#[derive(Debug, Clone, Copy)]
pub struct CostTable {
    pub w_default: f64,
    pub w_available: f64,
    pub w_water: f64,
    pub eps_surplus: f64,
    pub eps_machine: f64,
}

impl Default for CostTable {
    fn default() -> Self {
        CostTable {
            w_default: 1.0,
            w_available: 1e-4,
            w_water: 0.01,
            eps_surplus: 1e-8,
            eps_machine: 1e-6,
        }
    }
}

/// Threshold below which an LP variable is treated as zero when reading the
/// solution back out (real rates are ≥ 1e-3 in practice; simplex residues
/// are ≤ 1e-12).
const ACTIVE_TOL: f64 = 1e-9;

/// Additive, opt-in options for the Fulgora scrap-economy spike (see
/// docs/rfc-solver-net-flow.md decision log). Both default to `false`, so
/// every existing caller (`solve_netflow`, both `solve_*` entry points in
/// `solver.rs`) is behaviorally unchanged.
#[derive(Debug, Clone, Default)]
pub struct NetflowOptions {
    /// Admit `category == "recycling"` (and `"recycling-or-hand-crafting"`
    /// — see the note on [`is_recycling_category`]) recipes as LP columns
    /// despite [`is_excluded_recipe`] refusing them. Non-voider recycling
    /// recipes (e.g. `iron-gear-wheel-recycling`: gear → plates) behave as
    /// ordinary columns once admitted — no special casing needed.
    pub allow_recycling: bool,
    /// Additionally accept "pure voider" recycling recipes (see
    /// [`is_pure_voider`]) as a supported net-flow shape, with the closure
    /// and reachability guard exemptions documented at their call sites.
    /// Requires `allow_recycling` to have any candidates to exempt.
    pub allow_voiding: bool,
    /// Require every produced fluid to be consumed by the plan. Enabled
    /// internally for the advanced-oil retry because vanilla Factorio has no
    /// ordinary fluid void.
    pub disallow_fluid_surplus: bool,
    /// Build quality of the machines being planned
    /// (`docs/rfc-build-quality.md` Phase 1). Scales every column's
    /// crafting speed via [`effective_crafting_speed`] — `Normal`
    /// (default) multiplies by exactly 1.0, bit-identical to the
    /// pre-quality behavior.
    pub quality: crate::common::QualityTier,
    /// Global module policy (RFC-044 Phase 3). Per eligible column:
    /// crafting speed × the loadout's speed multiplier (floored at 0.2),
    /// results scaled ×(1+prod) on the catalyst-exempt portion at all
    /// three result sites (candidate net, per-machine rates, self-loop
    /// rates), and the loadout emitted on `MachineSpec::game_modules`
    /// for the layout stamp pass. `None` (default) resolves to the exact
    /// no-op — bit-identical to pre-module behavior (KC1). NOTE: module
    /// factors change LP column costs and coefficients, so a policy can
    /// legitimately flip free-mode recipe selection (accepted; RFC-044
    /// rev 2 decision log).
    pub module_policy: crate::module_policy::ModulePolicy,
    /// Research-sourced productivity to plan at, per recipe (e.g.
    /// `{"processing-unit": 0.10}`).
    ///
    /// A **declared axis**, carried on the manifest beside `stacking` and
    /// `inserter_capacity` — see
    /// [`crate::models::LayoutResult::research_productivity`]. Empty (the
    /// default) is bit-identical to the pre-existing behaviour.
    ///
    /// Distinct from [`Self::module_policy`], which covers productivity from
    /// *modules* and a machine's `base_effect`. Research productivity was
    /// modelled nowhere: the sim runs `research_all_technologies()`, so it
    /// planned against a world the solver did not know about. Concretely, on
    /// `tier5_processing_unit_from_ore_am3` the sim carries +10% on
    /// `processing-unit`, and planning without it over-provisions the AC stage
    /// — measured at +19% more AC per PU craft than the recipe needs, which
    /// then eats the EC that PU wanted. See `docs/meter-divergence.md`.
    ///
    /// Composed **additively** with module and base-effect productivity, which
    /// is how Factorio composes them.
    pub research_productivity: std::collections::BTreeMap<String, f64>,
}

/// True for both recycling-shaped categories in the bundled data.
///
/// NOTE: the RFC's Fulgora spike brief assumed `scrap-recycling` itself was
/// category `"recycling"`; draftsman 3.3.0 / Space Age data says its actual
/// category is `"recycling-or-hand-crafting"` (verified via the extractor
/// spike — see the recipes.json append). Both are admitted here so
/// `allow_recycling` actually reaches scrap-recycling, the entry point for
/// the whole scrap chain — `"crushing"` (the third `EXCLUDED_CATEGORIES`
/// member) stays excluded regardless, per the RFC brief.
fn is_recycling_category(recipe: &Recipe) -> bool {
    matches!(recipe.category.as_str(), "recycling" | "recycling-or-hand-crafting")
}

/// A "pure voider": a recycling recipe with exactly one ingredient and
/// exactly one product, both the same item, with strictly negative net
/// (produces less than it consumes) — e.g. `iron-plate-recycling`
/// (1 iron-plate in, 0.25 out) or `holmium-ore-recycling` (1 in, 0.25 out).
/// Shape-laundering-safe by construction: the ONLY item it ever touches is
/// the one it nets-destroys, so admitting it can never manufacture a path
/// from a cheap item to a demanded one — see the closure/reachability
/// exemptions in `solve_attempt` for where this shape gets special-cased.
/// Distinct from [`classify_self_loop`]'s net-negative single-item
/// `Unsupported` case, which this function does NOT change the behavior
/// of — voiders are accepted only at the solver level, gated on
/// `NetflowOptions::allow_voiding`.
pub(crate) fn is_pure_voider(recipe: &Recipe) -> bool {
    if recipe.ingredients.len() != 1 || recipe.products.len() != 1 {
        return false;
    }
    let ing = &recipe.ingredients[0];
    let prod = &recipe.products[0];
    ing.name == prod.name && raw_net_per_craft(recipe, &ing.name) < 0.0
}

/// Which recipe columns enter the LP.
#[derive(Clone, Copy)]
pub enum RecipeScope<'a> {
    /// Compatibility mode (Phase 1 default): only the named recipes — the
    /// set the tree walk would have selected. Recipe *selection* deltas vs
    /// the walk are zero by construction; only flow accounting changes.
    Restricted(&'a FxHashSet<String>),
    /// Free cost-based selection over all non-excluded recipes (the
    /// default since Phase 3, 2026-07).
    Free,
}

/// One resolved LP column.
struct Column {
    recipe: &'static Recipe,
    machine: String,
    crafting_speed: f64,
    /// Netted per-craft coefficient per item index (products·probability
    /// minus ingredients), exactly one entry per touched item. Under a
    /// productivity policy the product side uses EFFECTIVE amounts
    /// (RFC-044 site 1: applied to gross products minus catalyst
    /// exemptions BEFORE netting).
    net: Vec<(usize, f64)>,
    /// Resolved module effects for this (machine, recipe) pair
    /// (RFC-044 Phase 3). The speed multiplier is already folded into
    /// `crafting_speed`; `prod_bonus` and `loadout` feed the
    /// machine-spec build.
    effects: crate::module_policy::MachineModuleEffects,
}

/// Item interner: index by name, tracking fluid-ness from the first typed
/// reference seen (the data is consistent about item types).
#[derive(Default)]
struct Items<'a> {
    index: FxHashMap<&'a str, usize>,
    names: Vec<&'a str>,
    is_fluid: Vec<bool>,
}

impl<'a> Items<'a> {
    fn intern(&mut self, name: &'a str, is_fluid: bool) -> usize {
        if let Some(&i) = self.index.get(name) {
            i
        } else {
            let i = self.names.len();
            self.index.insert(name, i);
            self.names.push(name);
            self.is_fluid.push(is_fluid);
            i
        }
    }
    fn len(&self) -> usize {
        self.names.len()
    }
}

/// Item names present on both sides of a recipe (raw overlap, pre-netting;
/// recipe ingredient order — determinism). Empty for ordinary recipes.
fn raw_self_loop_items(recipe: &Recipe) -> Vec<&str> {
    recipe
        .ingredients
        .iter()
        .map(|i| i.name.as_str())
        .filter(|name| recipe.products.iter().any(|p| p.name == *name))
        .collect()
}

/// Raw per-craft net (produced − consumed) of one item in one recipe.
/// Sums all matching entries, though every self-loop recipe in the current
/// data has exactly one ingredient/product entry per self-loop item.
fn raw_net_per_craft(recipe: &Recipe, item: &str) -> f64 {
    let produced: f64 = recipe
        .products
        .iter()
        .filter(|p| p.name == item)
        .map(|p| p.amount * p.probability)
        .sum();
    let consumed: f64 = recipe
        .ingredients
        .iter()
        .filter(|i| i.name == item)
        .map(|i| i.amount)
        .sum();
    produced - consumed
}

/// Self-loop support classification (RFC Phase 2, "Cycle policy"; extended
/// for the fluid-ingredient row variant). v1 supports pure-solid self-loops
/// with 1 net-positive self-loop item (bacteria cultivations) or 2
/// self-loop items with opposite net signs (kovarex: U-235 +1/craft, U-238
/// −3/craft). The self-loop item itself must stay solid — fluid self-loops
/// (coal-liquefaction: heavy-oil is both consumed and produced) stay
/// refused, since no row template recirculates a fluid.
///
/// A single non-self-loop fluid INGREDIENT is now also supported (pentapod-
/// egg's water, fish-breeding's water), via `templates::self_loop_row`'s
/// fluid-header row — but only alongside the 1-item self-loop shape; the
/// 2-item (kovarex) shape's template has no fluid-header row. Any
/// non-self-loop fluid PRODUCT, or more than one non-self-loop fluid
/// ingredient, stays refused: multi-fluid self-loop rows aren't modeled.
enum SelfLoopShape {
    /// Not a self-loop recipe at all — no item on both sides.
    None,
    /// v1-supported shape; net flows can be emitted.
    Supported,
    /// Self-referencing but outside v1's supported shapes — keep refusing.
    Unsupported,
}

fn classify_self_loop(recipe: &Recipe) -> SelfLoopShape {
    let names = raw_self_loop_items(recipe);
    if names.is_empty() {
        return SelfLoopShape::None;
    }
    if names.len() > 2 {
        return SelfLoopShape::Unsupported;
    }
    // The self-loop item itself must be solid — fluid self-loops
    // (coal-liquefaction's heavy-oil) stay refused regardless of the
    // fluid-header row support added below.
    let self_loop_has_fluid = names.iter().any(|&name| {
        recipe.ingredients.iter().any(|i| i.name == name && i.type_ == "fluid")
    });
    if self_loop_has_fluid {
        return SelfLoopShape::Unsupported;
    }
    let non_self_loop_fluid_ingredients = recipe
        .ingredients
        .iter()
        .filter(|i| i.type_ == "fluid" && !names.contains(&i.name.as_str()))
        .count();
    let has_non_self_loop_fluid_product = recipe
        .products
        .iter()
        .any(|p| p.type_ == "fluid" && !names.contains(&p.name.as_str()));
    if has_non_self_loop_fluid_product
        || non_self_loop_fluid_ingredients > 1
        || (non_self_loop_fluid_ingredients == 1 && names.len() != 1)
    {
        return SelfLoopShape::Unsupported;
    }
    let supported = match names.len() {
        1 => raw_net_per_craft(recipe, names[0]) > 0.0,
        2 => {
            let n0 = raw_net_per_craft(recipe, names[0]);
            let n1 = raw_net_per_craft(recipe, names[1]);
            (n0 > 0.0 && n1 < 0.0) || (n0 < 0.0 && n1 > 0.0)
        }
        _ => unreachable!("names.len() > 2 handled above"),
    };
    if supported {
        SelfLoopShape::Supported
    } else {
        SelfLoopShape::Unsupported
    }
}

/// Outcome of one LP attempt: success, a hard error, or a cycle refusal
/// that the outer fallback loop may be able to break by excluding one
/// member recipe (when every demanded item it supplies has an alternative
/// in-closure producer).
enum AttemptError {
    Hard(SolverError),
    Cycle {
        refusal: SolverError,
        excludable: Option<String>,
    },
}

#[allow(clippy::too_many_arguments)]
pub fn solve_netflow(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    scope: RecipeScope<'_>,
    costs: &CostTable,
) -> Result<SolverResult, SolverError> {
    solve_netflow_with_options(
        target_item,
        target_rate,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        scope,
        costs,
        &NetflowOptions::default(),
    )
}

/// Like [`solve_netflow`] but accepts [`NetflowOptions`] (Fulgora
/// scrap-economy spike — additive, both flags default `false`).
///
/// RFC-062 Phase 1: a thin one-element-slice caller of
/// [`solve_netflow_multi_with_options`] — this and every function that
/// forwards to it (the 8 scalar wrappers in `solver.rs`) share the single
/// N-target implementation, so N=1 bit-identity is a construction
/// guarantee, not a tested coincidence.
#[allow(clippy::too_many_arguments)]
pub fn solve_netflow_with_options(
    target_item: &str,
    target_rate: f64,
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    scope: RecipeScope<'_>,
    costs: &CostTable,
    options: &NetflowOptions,
) -> Result<SolverResult, SolverError> {
    solve_netflow_multi_with_options(
        &[(target_item.to_string(), target_rate)],
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        scope,
        costs,
        options,
    )
}

/// Multi-target entry point (RFC-062 Phase 1,
/// `docs/rfc-062-multi-target-outputs.md` §Solver): solve for N ≥ 1
/// simultaneous targets in one LP instead of gluing together N independent
/// solves. `targets` is `(item, rate)` pairs; an item requested more than
/// once has its rates summed into a single demand row (RFC-062 Phase 1
/// decision log — summing rather than refusing duplicates).
#[allow(clippy::too_many_arguments)]
pub fn solve_netflow_multi(
    targets: &[(String, f64)],
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    scope: RecipeScope<'_>,
    costs: &CostTable,
) -> Result<SolverResult, SolverError> {
    solve_netflow_multi_with_options(
        targets,
        available_inputs,
        palette,
        default_machine,
        excluded_recipes,
        scope,
        costs,
        &NetflowOptions::default(),
    )
}

/// Like [`solve_netflow_multi`] but accepts [`NetflowOptions`]. This is the
/// single choke point every scalar entry point (`solve_netflow`,
/// `solve_netflow_with_options`, and transitively the 8 wrappers in
/// `solver.rs`) now forwards to via a one-element `targets` slice — see
/// [`solve_netflow_with_options`].
#[allow(clippy::too_many_arguments)]
pub fn solve_netflow_multi_with_options(
    targets: &[(String, f64)],
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    scope: RecipeScope<'_>,
    costs: &CostTable,
    options: &NetflowOptions,
) -> Result<SolverResult, SolverError> {
    // Acyclic-fallback loop (RFC "Cycle policy", amended after Phase 0
    // found the fluoroketone coolant loop on cryogenic-science-pack): when
    // the optimum contains an unsupported cycle, deterministically exclude
    // the first cycle member whose demanded outputs all have alternative
    // producers and re-solve. Genuinely forced cycles (kovarex with
    // uranium-processing excluded) still refuse with a typed error. Each
    // retry removes at least one recipe, so the cap is just a backstop.
    //
    // Target-count-agnostic by construction (RFC-062 Phase 1): every branch
    // below inspects `r.machines` / `r.surplus_outputs` post-solve, never a
    // single `target_idx`, so this loop needed no changes to generalize.
    let mut extra_excluded: FxHashSet<String> = FxHashSet::default();
    let mut attempt_options = options.clone();
    let mut last_refusal: Option<SolverError> = None;
    // Eight acyclic-fallback exclusions plus at most one free-mode oil-path
    // exclusivity re-solve (#476).
    for _ in 0..9 {
        match solve_attempt(
            targets,
            available_inputs,
            palette,
            default_machine,
            excluded_recipes,
            &extra_excluded,
            scope,
            costs,
            &attempt_options,
        ) {
            Ok(r) => {
                // Physical recipe-path exclusivity (#476): mixing basic and
                // advanced oil processing is arithmetically valid but can
                // deadlock in-game. Advanced processing is forced whenever
                // heavy oil is demanded; its petroleum-gas co-product then
                // shares a network with basic processing. Whole placed
                // refineries can fill that network, blocking advanced
                // processing's entire multi-output recipe and starving
                // heavy oil/lubricant.
                //
                // In free-selection mode, once the optimum proves advanced
                // processing is required, re-solve without basic processing.
                // The advanced-only plan makes unavoidable excess explicit
                // in `surplus_outputs`, so the layout gives it a physical
                // perimeter exit. Restricted compatibility mode remains the
                // exact caller-requested recipe set.
                let has_advanced =
                    r.machines.iter().any(|m| m.recipe == "advanced-oil-processing");
                let has_basic =
                    r.machines.iter().any(|m| m.recipe == "basic-oil-processing");
                let has_fluid_surplus = r.surplus_outputs.iter().any(|f| f.is_fluid);
                let needs_oil_physical_retry = matches!(scope, RecipeScope::Free)
                    && has_advanced
                    && (has_basic || has_fluid_surplus);
                if needs_oil_physical_retry {
                    if has_basic {
                        extra_excluded.insert("basic-oil-processing".to_string());
                    }
                    attempt_options.disallow_fluid_surplus = true;
                    continue;
                }
                return Ok(r);
            }
            Err(AttemptError::Hard(e)) => return Err(e),
            Err(AttemptError::Cycle { refusal, excludable }) => match excludable {
                Some(m) => {
                    extra_excluded.insert(m);
                    last_refusal = Some(refusal);
                }
                None => return Err(refusal),
            },
        }
    }
    Err(last_refusal.unwrap_or_else(|| SolverError::LpFailed {
        target: target_label(targets),
        detail: "acyclic fallback did not converge".to_string(),
    }))
}

/// Human-readable label for an error's `target` field when more than one
/// target is in play — joins every requested item name. Bit-identical to
/// the single-item label (`target_item.to_string()`) when `targets.len() ==
/// 1`, since `join` on a one-element slice is a no-op.
fn target_label(targets: &[(String, f64)]) -> String {
    targets.iter().map(|(item, _)| item.as_str()).collect::<Vec<_>>().join(", ")
}

#[allow(clippy::too_many_arguments)]
fn solve_attempt(
    targets: &[(String, f64)],
    available_inputs: &FxHashSet<String>,
    palette: &MachinePalette,
    default_machine: &str,
    excluded_recipes: &FxHashSet<String>,
    extra_excluded: &FxHashSet<String>,
    scope: RecipeScope<'_>,
    costs: &CostTable,
    options: &NetflowOptions,
) -> Result<SolverResult, AttemptError> {
    // ---------------------------------------------------------------
    // 1. Collect scope-eligible candidates (recipes.json order —
    //    determinism), then restrict to the target's DEMAND CLOSURE.
    //
    // The closure is the fixpoint of "recipes that net-produce a demanded
    // item; their ingredients become demanded". Without it, the
    // rate-proportional surplus penalty ε_o invites "surplus compression":
    // the LP activates recipes whose only effect is converting a
    // high-rate byproduct into a low-rate one (observed: an `ice` solve
    // activating solid-fuel-from-ammonia + ammonia-rocket-fuel purely to
    // shrink Σo). Cracking and byproduct crediting survive the closure —
    // their outputs are genuinely demanded; pure sinks do not.
    // ---------------------------------------------------------------
    let mut items = Items::default();
    struct Candidate {
        recipe: &'static Recipe,
        net: Vec<(usize, f64)>,
    }
    let mut candidates: Vec<Candidate> = Vec::new();

    for (name, recipe) in &db().recipes {
        // `allow_recycling` admits recycling-category recipes despite
        // `is_excluded_recipe` — every OTHER excluded category (crushing)
        // stays excluded regardless. See `is_recycling_category` for why
        // both recycling-shaped categories are checked.
        let excluded_by_category =
            is_excluded_recipe(recipe) && !(options.allow_recycling && is_recycling_category(recipe));
        if excluded_recipes.contains(name) || extra_excluded.contains(name) || excluded_by_category {
            continue;
        }
        // Placeholder rows (`parameter-N`, `recipe-unknown`) have no
        // products; filter explicitly rather than leaving all-zero columns
        // to the solver.
        if recipe.products.is_empty() {
            continue;
        }
        // Barreling recipes (fill/empty pairs referencing `*-barrel` items)
        // are graph noise: they form trivial fill↔empty cycles and pose as
        // fake "alternative producers" of every barrelable fluid, burning
        // acyclic-fallback retries. factoriolab excludes them by default
        // for the same reason. The `barrel` item itself (steel → barrel)
        // is unaffected.
        if recipe
            .products
            .iter()
            .map(|p| p.name.as_str())
            .chain(recipe.ingredients.iter().map(|i| i.name.as_str()))
            .any(|n| n.ends_with("-barrel"))
        {
            continue;
        }
        if let RecipeScope::Restricted(set) = &scope {
            if !set.contains(name) {
                continue;
            }
        }

        // Netted coefficients: REQUIRED single value per (item, recipe) —
        // microlp panics on duplicate variables in one constraint.
        let mut net: FxHashMap<usize, f64> = FxHashMap::default();
        let mut touch_order: Vec<usize> = Vec::new();
        for p in &recipe.products {
            let i = items.intern(&p.name, p.type_ == "fluid");
            if !net.contains_key(&i) {
                touch_order.push(i);
            }
            *net.entry(i).or_insert(0.0) += p.amount * p.probability;
        }
        for ing in &recipe.ingredients {
            let i = items.intern(&ing.name, ing.type_ == "fluid");
            if !net.contains_key(&i) {
                touch_order.push(i);
            }
            *net.entry(i).or_insert(0.0) -= ing.amount;
        }
        let net_vec: Vec<(usize, f64)> = touch_order
            .into_iter()
            .map(|i| (i, net[&i]))
            .filter(|(_, c)| *c != 0.0)
            .collect();

        candidates.push(Candidate { recipe, net: net_vec });
    }

    // Ensure every target item has a row even if nothing in scope touches
    // it, and build the demand vector (RFC-062 Phase 1 multi-seed
    // generalization). `target_order` is deduplicated, first-seen order —
    // requesting the same item twice sums its rates into one row rather
    // than adding a second row or refusing the request (Phase 1 decision
    // log: summing, not a typed duplicate-target error). `target_order`
    // also drives the demand-closure seed and output-assembly DFS seed
    // below, and `external_outputs`'s order, so a caller's first-requested
    // target is visited/emitted first — matching the N=1 single-target
    // traversal exactly when `targets.len() == 1`.
    let mut target_order: Vec<usize> = Vec::new();
    let mut target_rate_of: FxHashMap<usize, f64> = FxHashMap::default();
    for (name, rate) in targets {
        let idx = items.intern(name, false);
        match target_rate_of.entry(idx) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                *e.get_mut() += rate;
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(*rate);
                target_order.push(idx);
            }
        }
    }

    // NOTE (RFC-044): the closure below runs on RAW nets, before module
    // resolution. Productivity only ever increases product coefficients,
    // so nothing admitted here can become inadmissible under a policy —
    // but a recipe whose product side nets ≤0 raw and >0 only under prod
    // would be missed as a producer. No recipe in the bundled data has
    // that shape (Phase 3 review, NIT 5).
    // Demand closure over net-signed edges: a candidate joins when it
    // net-produces a demanded item; its net-consumed items become demanded.
    // RFC-062 Phase 1: seeded from every target, not just one.
    let mut demanded = vec![false; items.len()];
    for &idx in &target_order {
        demanded[idx] = true;
    }
    let mut in_closure = vec![false; candidates.len()];
    loop {
        let mut grew = false;
        for (c, cand) in candidates.iter().enumerate() {
            if in_closure[c] {
                continue;
            }
            let supplies_demand = cand
                .net
                .iter()
                .any(|&(i, coeff)| coeff > 0.0 && demanded[i]);
            if supplies_demand {
                in_closure[c] = true;
                grew = true;
                for &(i, coeff) in &cand.net {
                    if coeff < 0.0 && !demanded[i] {
                        demanded[i] = true;
                    }
                }
            }
        }
        if !grew {
            break;
        }
    }

    // Pure-voider admission (RFC Fulgora spike, gated on allow_voiding):
    // a voider's only net coefficient is negative (it strictly destroys its
    // own item), so it can never satisfy `supplies_demand` above and would
    // never join the closure through the ordinary fixpoint. Admit it
    // separately, but ONLY for items a closure column already net-produces
    // — this is what keeps the exemption laundering-safe: it lets the LP
    // dispose of genuine excess of an already-demanded item, never invents
    // a path from an unrelated cheap item to a demanded one (a voider only
    // ever touches the one item it destroys, so there is no such path to
    // invent). No further fixpoint iteration is needed — a voider's single
    // negative coefficient touches an item that's already demanded (by
    // construction, since we required a closure column producing it).
    if options.allow_voiding {
        let produced_by_closure: FxHashSet<usize> = candidates
            .iter()
            .enumerate()
            .filter(|&(c, _)| in_closure[c])
            .flat_map(|(_, cand)| cand.net.iter().filter(|&&(_, coeff)| coeff > 0.0).map(|&(i, _)| i))
            .collect();
        for (c, cand) in candidates.iter().enumerate() {
            if in_closure[c] || !is_pure_voider(cand.recipe) {
                continue;
            }
            if cand.net.iter().any(|&(i, _)| produced_by_closure.contains(&i)) {
                in_closure[c] = true;
            }
        }
    }

    // Finalize columns for closure members only (machine resolution +
    // pre-flight happen here, so out-of-closure recipes can't error).
    let mut columns: Vec<Column> = Vec::new();
    // Free-mode incompatibility bookkeeping: a column dropped because the
    // configured machine can't run it must NOT make its products
    // externally suppliable — otherwise an AM1-pinned advanced-circuit
    // solve would silently "solve" by importing advanced-circuit from
    // nowhere instead of surfacing the typed error the tree walk gave.
    let mut dropped_incompat: Vec<SolverError> = Vec::new();
    let mut has_dropped_producer = vec![false; items.len()];
    for (c, cand) in candidates.into_iter().enumerate() {
        if !in_closure[c] {
            continue;
        }
        let recipe = cand.recipe;
        let machine = machine_for_recipe_with_palette(recipe, palette, default_machine);
        if let Err(reason) = machine_can_run_recipe(&machine, recipe) {
            match scope {
                // Compatibility mode mirrors the tree walk: a chosen recipe
                // the configured machine can't run is a hard error.
                RecipeScope::Restricted(_) => {
                    return Err(AttemptError::Hard(SolverError::IncompatibleMachine {
                        recipe: recipe.name.clone(),
                        machine,
                        reason,
                    }));
                }
                // Free mode: the column is not available as configured —
                // drop it, let cost pick among the rest, but remember the
                // error and the products it would have supplied.
                RecipeScope::Free => {
                    for &(i, coeff) in &cand.net {
                        if coeff > 0.0 {
                            has_dropped_producer[i] = true;
                        }
                    }
                    dropped_incompat.push(SolverError::IncompatibleMachine {
                        recipe: recipe.name.clone(),
                        machine,
                        reason,
                    });
                    continue;
                }
            }
        }
        // Quality-scaled (rfc-build-quality Phase 1); ×1.0 at Normal, so the
        // `<= 0.0` guard sees the same sign as the raw speed.
        let crafting_speed = effective_crafting_speed(&machine, options.quality);
        if crafting_speed <= 0.0 {
            return Err(AttemptError::Hard(SolverError::MissingCraftingSpeed {
                entity: machine,
            }));
        }
        // RFC-044 Phase 3: module effects per (machine, recipe). Speed
        // folds into the column speed (× exactly 1.0 on the no-op path —
        // KC1); productivity rewrites the netted product coefficients
        // from EFFECTIVE amounts, applied to gross products minus the
        // catalyst-exempt portion BEFORE netting (site 1 of 3 — the raw
        // sign logic in `classify_self_loop` deliberately stays on raw
        // amounts).
        let mut effects =
            crate::module_policy::resolve_machine_modules(&options.module_policy, &machine, recipe);
        // Research productivity stacks on top of module + base-effect
        // productivity, additively, as Factorio composes them. Folding it into
        // `effects` here means all three downstream result sites (candidate
        // net, per-machine rates, self-loop rates) pick it up through the
        // existing `prod_bonus` path rather than each growing its own copy —
        // which is how this codebase ended up with two transit metrics.
        effects.prod_bonus += options
            .research_productivity
            .get(recipe.name.as_str())
            .copied()
            .unwrap_or(0.0);
        let crafting_speed = crafting_speed * effects.speed_multiplier;
        let net = if effects.prod_bonus > 0.0 {
            let mut net_map: FxHashMap<usize, f64> = FxHashMap::default();
            let mut touch_order: Vec<usize> = Vec::new();
            for p in &recipe.products {
                let i = items.intern(&p.name, p.type_ == "fluid");
                if !net_map.contains_key(&i) {
                    touch_order.push(i);
                }
                *net_map.entry(i).or_insert(0.0) += crate::module_policy::effective_product_amount(
                    p.amount,
                    p.ignored_by_productivity,
                    effects.prod_bonus,
                ) * p.probability;
            }
            for ing in &recipe.ingredients {
                let i = items.intern(&ing.name, ing.type_ == "fluid");
                if !net_map.contains_key(&i) {
                    touch_order.push(i);
                }
                *net_map.entry(i).or_insert(0.0) -= ing.amount;
            }
            touch_order
                .into_iter()
                .map(|i| (i, net_map[&i]))
                .filter(|(_, c)| *c != 0.0)
                .collect()
        } else {
            cand.net
        };
        columns.push(Column {
            recipe,
            machine,
            crafting_speed,
            net,
            effects,
        });
    }

    // ---------------------------------------------------------------
    // 2. Producer analysis → s-eligibility (per-solve, post-exclusion).
    //
    // Items whose only producers were dropped for machine incompatibility
    // are NOT s-eligible: they must stay infeasible so the stored
    // IncompatibleMachine error surfaces instead of a silent import.
    // ---------------------------------------------------------------
    let mut has_producer = vec![false; items.len()];
    for col in &columns {
        for &(i, c) in &col.net {
            if c > 0.0 {
                has_producer[i] = true;
            }
        }
    }
    let s_eligible: Vec<bool> = items
        .names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            available_inputs.contains(*name)
                || (!has_producer[i] && !has_dropped_producer[i])
        })
        .collect();

    // ---------------------------------------------------------------
    // 3. Build and solve the LP.
    // ---------------------------------------------------------------
    let mut problem = Problem::new(OptimizationDirection::Minimize);

    let x_vars: Vec<Variable> = columns
        .iter()
        .map(|col| {
            let machine_time = col.recipe.energy / col.crafting_speed;
            problem.add_var(costs.eps_machine * machine_time, (0.0, f64::INFINITY))
        })
        .collect();

    let s_vars: Vec<Option<Variable>> = items
        .names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if !s_eligible[i] {
                return None;
            }
            let w = if available_inputs.contains(*name) {
                costs.w_available
            } else if *name == "water" {
                costs.w_water
            } else {
                costs.w_default
            };
            Some(problem.add_var(w, (0.0, f64::INFINITY)))
        })
        .collect();

    let o_vars: Vec<Option<Variable>> = (0..items.len())
        .map(|i| {
            if has_producer[i] {
                let upper = if options.disallow_fluid_surplus && items.is_fluid[i] {
                    0.0
                } else {
                    f64::INFINITY
                };
                Some(problem.add_var(costs.eps_surplus, (0.0, upper)))
            } else {
                None
            }
        })
        .collect();

    // One flow-conservation constraint per item.
    let mut rows: Vec<Vec<(Variable, f64)>> = vec![Vec::new(); items.len()];
    for (c, col) in columns.iter().enumerate() {
        for &(i, coeff) in &col.net {
            rows[i].push((x_vars[c], coeff));
        }
    }
    for (i, row) in rows.iter_mut().enumerate() {
        if let Some(s) = s_vars[i] {
            row.push((s, 1.0));
        }
        if let Some(o) = o_vars[i] {
            row.push((o, -1.0));
        }
        if row.is_empty() {
            // An untouched non-target item simply has no flow. A TARGET
            // row going empty means nothing can produce or supply it —
            // skipping it would let the LP "solve" with an empty plan.
            // Surface the stored machine-incompatibility (the usual cause:
            // every producer column was dropped for the configured
            // machine) or an explicit unproducible error. RFC-062 Phase 1:
            // any of the N targets can hit this, not just a single one —
            // report the specific item whose row is empty (more precise
            // than the old single-target error, and identical to it when
            // `targets.len() == 1`, since `items.names[target_idx] ==
            // target_item` there).
            if target_rate_of.contains_key(&i) {
                if let Some(err) = dropped_incompat.into_iter().next() {
                    return Err(AttemptError::Hard(err));
                }
                return Err(AttemptError::Hard(SolverError::LpFailed {
                    target: items.names[i].to_string(),
                    detail: "target has no producer, no external supply, and no surplus sink".to_string(),
                }));
            }
            continue;
        }
        let rhs = target_rate_of.get(&i).copied().unwrap_or(0.0);
        problem.add_constraint(row.as_slice(), ComparisonOp::Eq, rhs);
    }

    let solution = problem.solve().map_err(|e| {
        // Infeasibility with dropped-incompatible columns in the closure
        // means the configured machine set can't make the target — surface
        // the first stored typed error (recipes.json order) rather than a
        // generic LP failure. The web sidebar routes it to the config
        // banner via INCOMPATIBLE_MACHINE_PREFIX.
        if let Some(err) = dropped_incompat.into_iter().next() {
            return AttemptError::Hard(err);
        }
        AttemptError::Hard(SolverError::LpFailed {
            target: target_label(targets),
            detail: format!("{e:?}"),
        })
    })?;

    // ---------------------------------------------------------------
    // 4. Cycle policy over the ACTIVE recipe graph (RFC "Cycle policy").
    //
    // Offending members are reported to the outer fallback loop with the
    // first member that is safely excludable — i.e. every *demanded* item
    // it net-supplies has another in-closure producer column. Excluding
    // such a member keeps the target feasible while breaking the cycle
    // (e.g. drop fluoroketone-cooling; fresh fluoroketone remains).
    // ---------------------------------------------------------------
    let active: Vec<usize> = (0..columns.len())
        .filter(|&c| solution[x_vars[c]] > ACTIVE_TOL)
        .collect();

    let excludable_member = |members: &[usize]| -> Option<String> {
        members
            .iter()
            .find(|&&m| {
                columns[m].net.iter().all(|&(i, coeff)| {
                    if coeff <= 0.0 || !demanded[i] {
                        return true;
                    }
                    columns.iter().enumerate().any(|(c2, col2)| {
                        c2 != m && col2.net.iter().any(|&(i2, c2f)| i2 == i && c2f > 0.0)
                    })
                })
            })
            .map(|&m| columns[m].recipe.name.clone())
    };

    for &c in &active {
        let r = columns[c].recipe;
        // Pure voiders are net-negative single-item self-loops by shape —
        // `classify_self_loop` correctly calls that Unsupported for every
        // OTHER caller (it's still not a row-template-able recipe). Only
        // the solver-level spike, gated on allow_voiding, treats it as an
        // accepted shape (netted emission, no row template needed since
        // voiders never reach layout).
        let accepted_voider = options.allow_voiding && is_pure_voider(r);
        if !accepted_voider && matches!(classify_self_loop(r), SelfLoopShape::Unsupported) {
            return Err(AttemptError::Cycle {
                refusal: SolverError::UnsupportedSelfLoop {
                    recipe: r.name.clone(),
                },
                excludable: excludable_member(&[c]),
            });
        }
    }
    if let Some(cycle_members) = find_active_cycle_indices(&columns, &active) {
        let names: Vec<String> = cycle_members
            .iter()
            .map(|&c| columns[c].recipe.name.clone())
            .collect();
        return Err(AttemptError::Cycle {
            refusal: SolverError::UnsupportedCycle {
                recipes: names.join(" → "),
            },
            excludable: excludable_member(&cycle_members),
        });
    }

    // ---------------------------------------------------------------
    // 5. Assemble SolverResult in tree-walk traversal order.
    //
    // dependency_order and external_inputs order reproduce the tree walk's
    // DFS pre-order exactly (target-rooted, ingredients in recipe order,
    // first visit wins) — golden-hash stability depends on this.
    // ---------------------------------------------------------------
    // Snap simplex float residue to exact integers (relative 1e-9): a
    // solution value of 15.000000000000016 must not become 16 machines at
    // the layout's ceil, or flip a 15/s belt-tier threshold. Real
    // fractional plans (e.g. 1.06 refineries) are far outside the snap
    // window.
    let snap = snap_value;
    let x_of = |c: usize| snap(solution[x_vars[c]]);
    let s_of = |i: usize| snap(s_vars[i].map(|v| solution[v]).unwrap_or(0.0));
    let o_of = |i: usize| snap(o_vars[i].map(|v| solution[v]).unwrap_or(0.0));

    // Builds one MachineSpec for column `c`. Factored out of the DFS below
    // so the pure-voider post-pass (RFC Fulgora spike) can emit voider
    // machines the same way — voiders are demand-pulled sinks, not
    // producers, so they're structurally invisible to the producer-of-item
    // DFS and need their own emission pass (see the reachability-exemption
    // comment after the DFS loop).
    let build_machine_spec = |c: usize| -> MachineSpec {
        let col = &columns[c];
        let crafts_per_sec_per_machine = col.crafting_speed / col.recipe.energy;
        let count = snap(x_of(c) / crafts_per_sec_per_machine);
        // Self-loop items (RFC Phase 2): excluded from the ordinary
        // ingredient/product mapping below and emitted instead as a
        // single net flow (into inputs or outputs, by sign) plus a
        // `self_loop` entry carrying the raw per-machine rates for
        // the row template's loop-back belt sizing. Pure voiders (RFC
        // Fulgora spike) fall through this same machinery: their one
        // self-loop item nets negative, so it lands in `inputs` as a
        // netted consumption with empty `outputs` — exactly the "netted
        // emission" shape the spike calls for, with no extra code.
        let self_loop_names = raw_self_loop_items(col.recipe);
        let mut inputs: Vec<ItemFlow> = col
            .recipe
            .ingredients
            .iter()
            .filter(|ing| !self_loop_names.contains(&ing.name.as_str()))
            .map(|ing| ItemFlow {
                item: ing.name.clone(),
                rate: ing.amount * crafts_per_sec_per_machine,
                is_fluid: ing.type_ == "fluid",
                module_id: 0,
            })
            .collect();
        // RFC-044 site 2 of 3: per-machine output rates use EFFECTIVE
        // product amounts (catalyst-exempt prod scaling; exact
        // passthrough at prod_bonus == 0.0). Desyncing this from the LP
        // net (site 1) silently splits planned flows from the rates
        // inserter sizing and the validators consume.
        let prod_bonus = col.effects.prod_bonus;
        let mut outputs: Vec<ItemFlow> = col
            .recipe
            .products
            .iter()
            .filter(|p| !self_loop_names.contains(&p.name.as_str()))
            .map(|p| ItemFlow {
                item: p.name.clone(),
                rate: crate::module_policy::effective_product_amount(
                    p.amount,
                    p.ignored_by_productivity,
                    prod_bonus,
                ) * p.probability
                    * crafts_per_sec_per_machine,
                is_fluid: p.type_ == "fluid",
                module_id: 0,
            })
            .collect();
        let mut self_loop: Vec<SelfLoopFlow> = Vec::new();
        for name in &self_loop_names {
            let consumed_rate = col
                .recipe
                .ingredients
                .iter()
                .filter(|i| i.name == *name)
                .map(|i| i.amount)
                .sum::<f64>()
                * crafts_per_sec_per_machine;
            // RFC-044 site 3 of 3: self-loop produced rates (kovarex's
            // loop-back belt sizing) use the same effective amounts.
            let produced_rate = col
                .recipe
                .products
                .iter()
                .filter(|p| p.name == *name)
                .map(|p| {
                    crate::module_policy::effective_product_amount(
                        p.amount,
                        p.ignored_by_productivity,
                        prod_bonus,
                    ) * p.probability
                })
                .sum::<f64>()
                * crafts_per_sec_per_machine;
            let net_rate = produced_rate - consumed_rate;
            let is_fluid = col
                .recipe
                .ingredients
                .iter()
                .find(|i| i.name == *name)
                .map(|i| i.type_ == "fluid")
                .unwrap_or(false);
            self_loop.push(SelfLoopFlow {
                item: name.to_string(),
                is_fluid,
                consumed_rate,
                produced_rate,
                net_rate,
            });
            if net_rate > 0.0 {
                outputs.push(ItemFlow {
                    item: name.to_string(),
                    rate: net_rate,
                    is_fluid,
                    module_id: 0,
                });
            } else if net_rate < 0.0 {
                inputs.push(ItemFlow {
                    item: name.to_string(),
                    rate: -net_rate,
                    is_fluid,
                    module_id: 0,
                });
            }
            // net_rate == 0.0 cannot occur for `SelfLoopShape::Supported`
            // columns (both the 1-item net-positive and 2-item
            // opposite-sign checks require nonzero net) nor for accepted
            // voiders (net < 0 by definition), so no branch is needed here.
        }
        MachineSpec {
            entity: col.machine.clone(),
            recipe: col.recipe.name.clone(),
            self_loop,
            count,
            inputs,
            outputs,
            voider: false,
            game_modules: col.effects.loadout.clone(),
        }
    };

    // item → active producing columns (net > 0), in column order.
    let mut producers_of: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for &c in &active {
        for &(i, coeff) in &columns[c].net {
            if coeff > 0.0 {
                producers_of.entry(i).or_default().push(c);
            }
        }
    }

    let mut machines: Vec<MachineSpec> = Vec::new();
    let mut dependency_order: Vec<String> = Vec::new();
    let mut external_inputs: Vec<ItemFlow> = Vec::new();
    let mut visited_items: FxHashSet<usize> = FxHashSet::default();
    let mut visited_cols: FxHashSet<usize> = FxHashSet::default();

    // Iterative DFS mirroring `resolve()`'s recursion.
    enum Work {
        Item(usize),
        Col(usize),
    }
    // RFC-062 Phase 1: seeded from every target, reverse-pushed so the
    // first-requested target pops (and is thus visited/emitted) first —
    // bit-identical to the old single-item seed when `targets.len() == 1`.
    let mut stack: Vec<Work> = target_order.iter().rev().map(|&idx| Work::Item(idx)).collect();
    while let Some(w) = stack.pop() {
        match w {
            Work::Item(i) => {
                if !visited_items.insert(i) {
                    continue;
                }
                if s_of(i) > ACTIVE_TOL {
                    external_inputs.push(ItemFlow {
                        item: items.names[i].to_string(),
                        rate: s_of(i),
                        is_fluid: items.is_fluid[i],
                        module_id: 0,
                    });
                }
                if let Some(cols) = producers_of.get(&i) {
                    // Reverse-push so the first producer pops first.
                    for &c in cols.iter().rev() {
                        stack.push(Work::Col(c));
                    }
                }
            }
            Work::Col(c) => {
                if !visited_cols.insert(c) {
                    continue;
                }
                machines.push(build_machine_spec(c));
                dependency_order.push(columns[c].recipe.name.clone());
                // Recurse ingredients in declaration order (reversed for
                // the stack), matching the ingredient loop in
                // `solver.rs::resolve`.
                for ing in columns[c].recipe.ingredients.iter().rev() {
                    let i = items.index[ing.name.as_str()];
                    stack.push(Work::Item(i));
                }
            }
        }
    }

    // Pure-voider emission pass (RFC Fulgora spike, gated on allow_voiding).
    // Voiders are demand-pulled SINKS (their only net coefficient is
    // negative), so `producers_of` never lists them and the DFS above can
    // never discover them by walking producer→ingredient edges — the same
    // reason they're exempt from the surplus-compression guard just below.
    // Emit them explicitly here and mark visited, so both the report (their
    // MachineSpec must actually appear in the machine mix) and the guard
    // (which would otherwise flag them as an unreachable active column) see
    // consistent state.
    if options.allow_voiding {
        for &c in &active {
            if is_pure_voider(columns[c].recipe) && visited_cols.insert(c) {
                machines.push(build_machine_spec(c));
                dependency_order.push(columns[c].recipe.name.clone());
            }
        }
    }

    // Surplus-compression guard: an active column none of whose products
    // sit on the active demand tree is a pure surplus-processor — machines
    // whose only effect is converting a high-rate byproduct into a
    // lower-rate one to shrink the ε_o term (observed: light-oil-cracking
    // + sulfur laundering AOP's gas byproduct on an electric-engine-unit
    // solve). Real factories stall on surplus either way; honest surplus
    // beats garbage machines. Exclude the first such column (column order —
    // deterministic) and let the outer loop re-solve. Always excludable:
    // by construction its products aren't load-bearing for the target.
    if visited_cols.len() != active.len() {
        let first_unreachable = active
            .iter()
            .find(|c| !visited_cols.contains(c))
            .map(|&c| columns[c].recipe.name.clone())
            .expect("mismatch implies at least one unreachable column");
        return Err(AttemptError::Cycle {
            refusal: SolverError::LpFailed {
                target: target_label(targets),
                detail: format!(
                    "surplus-processor exclusion did not converge (last: {first_unreachable})"
                ),
            },
            excludable: Some(first_unreachable),
        });
    }

    // Byproduct produced beyond internal demand + target-row RHS. RFC-062
    // Phase 1 semantics (decision log): this is NOT mutually exclusive with
    // `external_outputs` — a target item can appear in BOTH when its net
    // production (driven by another target's recipe tree) exceeds its own
    // requested rate; `o_of(i)` is computed identically regardless of
    // whether `i` is a target (no code branch here checks target
    // membership). `external_outputs` conveys the guaranteed per-target
    // export rate (the row's RHS); `surplus_outputs` conveys any additional
    // production of that same item beyond what internal consumers + the
    // target RHS need. Phase 2 (layout) must give a target item that also
    // carries surplus two physical exports of the same item, exactly as it
    // already must for a non-target byproduct today.
    let surplus_outputs: Vec<ItemFlow> = (0..items.len())
        .filter(|&i| o_of(i) > ACTIVE_TOL)
        .map(|i| ItemFlow {
            item: items.names[i].to_string(),
            rate: o_of(i),
            is_fluid: items.is_fluid[i],
            module_id: 0,
        })
        .collect();

    // RFC-062 Phase 1 DI-coupling guard: an item that is itself one of the
    // requested targets must never be stamped as a direct-insertion
    // coupling, even if it otherwise qualifies (exactly one active
    // producer, exactly one active consumer, no external supply/surplus,
    // matching rates) — DI fuses producer straight into consumer with no
    // exposed belt, and the item's export path (still demanded by its row's
    // RHS) would have nowhere to attach. See `detect_di_couplings`.
    let target_index_set: FxHashSet<usize> = target_order.iter().copied().collect();

    // Direct-insertion coupling detection (RFC decomposition-search Phase 3).
    // For each item with exactly one active producer and one active consumer,
    // no external supply, no surplus, and matching supply↔demand rates, emit a
    // coupling. Fluids are excluded (inserter DI only; pipe adjacency is a
    // separate concern). Voider columns are excluded as consumers (they
    // destroy items, they don't use them to make something). Self-loop items
    // within a single recipe are not producer↔consumer pairs. Target items
    // are excluded (RFC-062 Phase 1, above).
    let di_couplings = detect_di_couplings(
        &columns,
        &active,
        &producers_of,
        &items,
        &target_index_set,
        &|i| s_of(i),
        &|i| o_of(i),
        &|c| x_of(c),
    );

    Ok(SolverResult {
        machines,
        external_inputs,
        // RFC-062 Phase 1: one ItemFlow per unique requested target, in
        // first-seen order, rate = the (possibly summed) demand this
        // attempt's LP rows were built against. Bit-identical to the old
        // single-element vec when `targets.len() == 1`.
        external_outputs: target_order
            .iter()
            .map(|&idx| ItemFlow {
                item: items.names[idx].to_string(),
                rate: target_rate_of[&idx],
                is_fluid: items.is_fluid[idx],
                module_id: 0,
            })
            .collect(),
        surplus_outputs,
        dependency_order,
        di_couplings,
    })
}

/// Snap a fractional value to the nearest integer when within tolerance
/// (1e-9 of the rounded magnitude). Used to clean LP solutions so e.g.
/// 2.000000001 reads as 2.0.
fn snap_value(v: f64) -> f64 {
    let r = v.round();
    if (v - r).abs() < 1e-9 * r.abs().max(1.0) {
        r
    } else {
        v
    }
}

/// Detect direct-insertion couplings: producer↔consumer pairs where the
/// producer's entire output of an item flows to exactly one consumer with
/// no branching, no surplus, and no external supply.
///
/// Eligibility criteria (see `docs/rfc-decomposition-search.md` Phase 3):
/// - exactly one active producer column for the item (net > 0)
/// - exactly one active consumer column for the item (net < 0)
/// - the consumer is not a voider (voiders destroy, they don't produce)
/// - no external supply (`s_of(i)` ≈ 0)
/// - no surplus (`o_of(i)` ≈ 0)
/// - the item is not a fluid (inserter DI only; pipe DI is separate)
/// - the item is not itself one of the requested export targets (RFC-062
///   Phase 1 — see `target_indices` below)
/// - supply rate ≈ demand rate (producer's output matches consumer's input)
///
/// Emits one `DICoupling` per qualifying (producer, consumer, item) triple.
/// The placer MAY use these; presence does not force DI layout.
///
/// `target_indices` (RFC-062 Phase 1,
/// `docs/rfc-062-multi-target-outputs.md` §Solver "Correctness gap"): under
/// single-target solving a target item never qualified here by construction
/// (its export is a property of the row's RHS, invisible to this function).
/// Under multi-target, an item can legitimately have exactly one producer
/// and one consumer AND be a target simultaneously — stamping that pair as
/// DI would let the placer fuse producer directly into consumer with no
/// exposed belt, leaving the item's still-demanded export path with nowhere
/// to attach. Membership in `target_indices` is checked first and refuses
/// the pair outright, before the supply/demand rate comparison runs (which
/// is not itself a reliable guard here — see the Phase 1 decision log).
fn detect_di_couplings(
    columns: &[Column],
    active: &[usize],
    producers_of: &FxHashMap<usize, Vec<usize>>,
    items: &Items,
    target_indices: &FxHashSet<usize>,
    s_of: &dyn Fn(usize) -> f64,
    o_of: &dyn Fn(usize) -> f64,
    x_of: &dyn Fn(usize) -> f64,
) -> Vec<crate::models::DICoupling> {
    // Build consumers_of: item → active columns with net < 0 for that item.
    let mut consumers_of: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for &c in active {
        for &(i, coeff) in &columns[c].net {
            if coeff < 0.0 {
                consumers_of.entry(i).or_default().push(c);
            }
        }
    }

    let mut couplings = Vec::new();
    for (&i, producers) in producers_of {
        // RFC-062 Phase 1: an item that's itself a requested export target
        // must never be DI-coupled, regardless of how the rest of the
        // eligibility check would score it.
        if target_indices.contains(&i) {
            continue;
        }
        // Must be exactly one producer and one consumer.
        let consumers = match consumers_of.get(&i) {
            Some(cs) if cs.len() == 1 => &cs[..],
            _ => continue,
        };
        if producers.len() != 1 {
            continue;
        }
        let producer = producers[0];
        let consumer = consumers[0];

        // Voider columns are sinks, not real consumers — skip.
        if is_pure_voider(columns[consumer].recipe) {
            continue;
        }
        // Fluid items need pipe adjacency, not inserter DI — skip.
        if items.is_fluid[i] {
            continue;
        }
        // No external supply — the item is entirely internally produced.
        if s_of(i) > ACTIVE_TOL {
            continue;
        }
        // No surplus — the producer's output exactly satisfies the consumer.
        if o_of(i) > ACTIVE_TOL {
            continue;
        }

        // Supply = x[producer] * net_coeff (positive); demand = x[consumer] *
        // |net_coeff| (negative, negated). Check they match within tolerance.
        let supply = x_of(producer)
            * columns[producer]
                .net
                .iter()
                .find(|&&(j, _)| j == i)
                .map(|&(_, c)| c)
                .unwrap_or(0.0);
        let demand = x_of(consumer)
            * columns[consumer]
                .net
                .iter()
                .find(|&&(j, _)| j == i)
                .map(|&(_, c)| -c)
                .unwrap_or(0.0);
        if (supply - demand).abs() > ACTIVE_TOL * supply.max(1.0).max(demand) {
            continue;
        }

        // Machine counts (same computation as build_machine_spec).
        let producer_count = snap_machine_count(x_of(producer), &columns[producer]);
        let consumer_count = snap_machine_count(x_of(consumer), &columns[consumer]);

        couplings.push(crate::models::DICoupling {
            producer_recipe: columns[producer].recipe.name.to_string(),
            consumer_recipe: columns[consumer].recipe.name.to_string(),
            item: items.names[i].to_string(),
            producer_count,
            consumer_count,
        });
    }

    // Deterministic order: by recipe DB declaration order (column index).
    couplings.sort_by(|a, b| {
        let pa = active.iter().position(|&c| columns[c].recipe.name == a.producer_recipe);
        let pb = active.iter().position(|&c| columns[c].recipe.name == b.producer_recipe);
        pa.cmp(&pb)
    });
    couplings
}

/// Snap a fractional machine count to the nearest integer (same logic as
/// `build_machine_spec`'s `snap(x / (speed / energy))`).
fn snap_machine_count(x_of_c: f64, col: &Column) -> f64 {
    let crafts_per_sec = col.crafting_speed / col.recipe.energy;
    snap_value(x_of_c / crafts_per_sec)
}

/// Find a multi-recipe cycle among the active columns, if any. Returns the
/// column indices of one offending strongly-connected component (size ≥ 2).
/// Self-loops are checked separately (raw same-item-both-sides test) before
/// this runs.
fn find_active_cycle_indices(columns: &[Column], active: &[usize]) -> Option<Vec<usize>> {
    // Edge r_a → r_b when r_a produces (raw product) something r_b consumes
    // (raw ingredient).
    let mut produces: FxHashMap<&str, Vec<usize>> = FxHashMap::default();
    for &c in active {
        for p in &columns[c].recipe.products {
            produces.entry(p.name.as_str()).or_default().push(c);
        }
    }
    let mut adj: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for &c in active {
        for ing in &columns[c].recipe.ingredients {
            if let Some(ps) = produces.get(ing.name.as_str()) {
                for &p in ps {
                    if p != c {
                        adj.entry(p).or_default().push(c);
                    }
                }
            }
        }
    }

    // Iterative Tarjan SCC.
    let mut index_of: FxHashMap<usize, usize> = FxHashMap::default();
    let mut lowlink: FxHashMap<usize, usize> = FxHashMap::default();
    let mut on_stack: FxHashSet<usize> = FxHashSet::default();
    let mut scc_stack: Vec<usize> = Vec::new();
    let mut next_index = 0usize;
    let empty: Vec<usize> = Vec::new();

    for &start in active {
        if index_of.contains_key(&start) {
            continue;
        }
        let mut call: Vec<(usize, usize)> = vec![(start, 0)]; // (node, child cursor)
        index_of.insert(start, next_index);
        lowlink.insert(start, next_index);
        next_index += 1;
        scc_stack.push(start);
        on_stack.insert(start);

        while let Some(&(v, cursor)) = call.last() {
            let children = adj.get(&v).unwrap_or(&empty);
            if cursor < children.len() {
                call.last_mut().unwrap().1 += 1;
                let w = children[cursor];
                if let std::collections::hash_map::Entry::Vacant(e) = index_of.entry(w) {
                    e.insert(next_index);
                    lowlink.insert(w, next_index);
                    next_index += 1;
                    scc_stack.push(w);
                    on_stack.insert(w);
                    call.push((w, 0));
                } else if on_stack.contains(&w) {
                    let lw = index_of[&w];
                    let lv = lowlink[&v];
                    lowlink.insert(v, lv.min(lw));
                }
            } else {
                call.pop();
                if let Some(&(parent, _)) = call.last() {
                    let lv = lowlink[&v];
                    let lp = lowlink[&parent];
                    lowlink.insert(parent, lp.min(lv));
                }
                if lowlink[&v] == index_of[&v] {
                    let mut comp = Vec::new();
                    while let Some(w) = scc_stack.pop() {
                        on_stack.remove(&w);
                        comp.push(w);
                        if w == v {
                            break;
                        }
                    }
                    if comp.len() >= 2 {
                        comp.sort_unstable();
                        return Some(comp);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_policy::MachineModuleEffects;
    use crate::recipe_db::db;

    /// RFC-062 Phase 1 (`docs/rfc-062-multi-target-outputs.md` §Solver,
    /// "Correctness gap"): `detect_di_couplings` must refuse to stamp a
    /// producer→consumer pair when the intermediate item is itself a
    /// requested export target, even when supply and demand happen to
    /// match exactly (the shape that would otherwise qualify for DI).
    ///
    /// The live KC2 fixture (EC@10/s + AC@3/s, see
    /// `crates/core/tests/solver_multi_target.rs`) never actually exercises
    /// this branch: EC's target rate forces the producer's gross output to
    /// exceed AC's ingredient draw by exactly the target rate, so
    /// supply != demand there regardless of the guard (confirmed by
    /// probing the live solve with the guard temporarily removed — see the
    /// Phase 1 decision log). This test constructs the supply == demand
    /// coincidence directly with synthetic columns, so the guard is proven
    /// on its own terms rather than relying on a numeric accident in one
    /// fixture.
    #[test]
    fn di_coupling_guard_suppresses_target_item_coupling() {
        let ec_recipe = db().recipes.get("electronic-circuit").expect("electronic-circuit recipe");
        let ac_recipe = db().recipes.get("advanced-circuit").expect("advanced-circuit recipe");

        let mut items = Items::default();
        let ec_idx = items.intern("electronic-circuit", false);
        let ac_idx = items.intern("advanced-circuit", false);

        // Column 0: EC producer, net +1 EC/craft. Column 1: AC consumer,
        // net -2 EC/craft (AC's real per-craft EC coefficient) and +1
        // AC/craft. x-rates (6.0, 3.0) are chosen so supply (6*1=6) exactly
        // equals demand (3*2=6) — the coincidence the guard must catch
        // regardless of whether EC's row also carries a nonzero target
        // rate (irrelevant here, since this test calls
        // `detect_di_couplings` directly rather than solving a full LP).
        let columns = vec![
            Column {
                recipe: ec_recipe,
                machine: "assembling-machine-2".to_string(),
                crafting_speed: 0.75,
                net: vec![(ec_idx, 1.0)],
                effects: MachineModuleEffects::none(),
            },
            Column {
                recipe: ac_recipe,
                machine: "assembling-machine-2".to_string(),
                crafting_speed: 0.75,
                net: vec![(ec_idx, -2.0), (ac_idx, 1.0)],
                effects: MachineModuleEffects::none(),
            },
        ];
        let active = vec![0usize, 1usize];
        let mut producers_of: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        producers_of.insert(ec_idx, vec![0]);
        producers_of.insert(ac_idx, vec![1]);

        let x_of = |c: usize| if c == 0 { 6.0 } else { 3.0 };
        let s_of = |_i: usize| 0.0;
        let o_of = |_i: usize| 0.0;

        // Control: EC is NOT a target — the pair qualifies for DI exactly
        // like plain single-target AC-from-EC solving does today.
        let no_targets: FxHashSet<usize> = FxHashSet::default();
        let couplings = detect_di_couplings(
            &columns,
            &active,
            &producers_of,
            &items,
            &no_targets,
            &s_of,
            &o_of,
            &x_of,
        );
        assert_eq!(
            couplings.len(),
            1,
            "expected EC->AC coupling when EC is not a target, got {couplings:?}"
        );
        assert_eq!(couplings[0].producer_recipe, "electronic-circuit");
        assert_eq!(couplings[0].consumer_recipe, "advanced-circuit");
        assert_eq!(couplings[0].item, "electronic-circuit");

        // EC as a requested target — the guard must suppress the coupling.
        let mut ec_is_target: FxHashSet<usize> = FxHashSet::default();
        ec_is_target.insert(ec_idx);
        let couplings2 = detect_di_couplings(
            &columns,
            &active,
            &producers_of,
            &items,
            &ec_is_target,
            &s_of,
            &o_of,
            &x_of,
        );
        assert!(
            couplings2.is_empty(),
            "EC as target must suppress the EC->AC coupling, got {couplings2:?}"
        );
    }
}
