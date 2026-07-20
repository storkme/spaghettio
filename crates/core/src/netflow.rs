//! Net-flow (LP) solver — see `docs/rfp-solver-net-flow.md`.
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

/// Frozen cost table (docs/rfp-solver-net-flow.md, "Design → Formulation";
/// revision 1 logged 2026-07-10). Kill criterion 3 forbids retuning these to
/// pass cross-validation; any revision needs an RFP decision-log entry.
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
/// docs/rfp-solver-net-flow.md decision log). Both default to `false`, so
/// every existing caller (`solve_netflow`, both `solve_*` entry points in
/// `solver.rs`) is behaviorally unchanged.
#[derive(Debug, Clone, Copy, Default)]
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
    /// Build quality of the machines being planned
    /// (`docs/rfp-build-quality.md` Phase 1). Scales every column's
    /// crafting speed via [`effective_crafting_speed`] — `Normal`
    /// (default) multiplies by exactly 1.0, bit-identical to the
    /// pre-quality behavior.
    pub quality: crate::common::QualityTier,
}

/// True for both recycling-shaped categories in the bundled data.
///
/// NOTE: the RFP's Fulgora spike brief assumed `scrap-recycling` itself was
/// category `"recycling"`; draftsman 3.3.0 / Space Age data says its actual
/// category is `"recycling-or-hand-crafting"` (verified via the extractor
/// spike — see the recipes.json append). Both are admitted here so
/// `allow_recycling` actually reaches scrap-recycling, the entry point for
/// the whole scrap chain — `"crushing"` (the third `EXCLUDED_CATEGORIES`
/// member) stays excluded regardless, per the RFP brief.
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
    /// Free cost-based selection over all non-excluded recipes (Phase 0
    /// reporting; becomes the default in Phase 3).
    Free,
}

/// One resolved LP column.
struct Column {
    recipe: &'static Recipe,
    machine: String,
    crafting_speed: f64,
    /// Netted per-craft coefficient per item index (products·probability
    /// minus ingredients), exactly one entry per touched item.
    net: Vec<(usize, f64)>,
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

/// Self-loop support classification (RFP Phase 2, "Cycle policy"; extended
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
    // Acyclic-fallback loop (RFP "Cycle policy", amended after Phase 0
    // found the fluoroketone coolant loop on cryogenic-science-pack): when
    // the optimum contains an unsupported cycle, deterministically exclude
    // the first cycle member whose demanded outputs all have alternative
    // producers and re-solve. Genuinely forced cycles (kovarex with
    // uranium-processing excluded) still refuse with a typed error. Each
    // retry removes at least one recipe, so the cap is just a backstop.
    let mut extra_excluded: FxHashSet<String> = FxHashSet::default();
    let mut last_refusal: Option<SolverError> = None;
    for _ in 0..8 {
        match solve_attempt(
            target_item,
            target_rate,
            available_inputs,
            palette,
            default_machine,
            excluded_recipes,
            &extra_excluded,
            scope,
            costs,
            options,
        ) {
            Ok(r) => return Ok(r),
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
        target: target_item.to_string(),
        detail: "acyclic fallback did not converge".to_string(),
    }))
}

#[allow(clippy::too_many_arguments)]
fn solve_attempt(
    target_item: &str,
    target_rate: f64,
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

    // Ensure the target item has a row even if nothing in scope touches it.
    let target_idx = items.intern(target_item, false);

    // Demand closure over net-signed edges: a candidate joins when it
    // net-produces a demanded item; its net-consumed items become demanded.
    let mut demanded = vec![false; items.len()];
    demanded[target_idx] = true;
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

    // Pure-voider admission (RFP Fulgora spike, gated on allow_voiding):
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
        // Quality-scaled (rfp-build-quality Phase 1); ×1.0 at Normal, so the
        // `<= 0.0` guard sees the same sign as the raw speed.
        let crafting_speed = effective_crafting_speed(&machine, options.quality);
        if crafting_speed <= 0.0 {
            return Err(AttemptError::Hard(SolverError::MissingCraftingSpeed {
                entity: machine,
            }));
        }
        columns.push(Column {
            recipe,
            machine,
            crafting_speed,
            net: cand.net,
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
                Some(problem.add_var(costs.eps_surplus, (0.0, f64::INFINITY)))
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
            // An untouched non-target item simply has no flow. The TARGET
            // row going empty means nothing can produce or supply it —
            // skipping it would let the LP "solve" with an empty plan.
            // Surface the stored machine-incompatibility (the usual cause:
            // every producer column was dropped for the configured
            // machine) or an explicit unproducible error.
            if i == target_idx {
                if let Some(err) = dropped_incompat.into_iter().next() {
                    return Err(AttemptError::Hard(err));
                }
                return Err(AttemptError::Hard(SolverError::LpFailed {
                    target: target_item.to_string(),
                    detail: "target has no producer, no external supply, and no surplus sink".to_string(),
                }));
            }
            continue;
        }
        let rhs = if i == target_idx { target_rate } else { 0.0 };
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
            target: target_item.to_string(),
            detail: format!("{e:?}"),
        })
    })?;

    // ---------------------------------------------------------------
    // 4. Cycle policy over the ACTIVE recipe graph (RFP "Cycle policy").
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
    let snap = |v: f64| -> f64 {
        let r = v.round();
        if (v - r).abs() < 1e-9 * r.abs().max(1.0) {
            r
        } else {
            v
        }
    };
    let x_of = |c: usize| snap(solution[x_vars[c]]);
    let s_of = |i: usize| snap(s_vars[i].map(|v| solution[v]).unwrap_or(0.0));
    let o_of = |i: usize| snap(o_vars[i].map(|v| solution[v]).unwrap_or(0.0));

    // Builds one MachineSpec for column `c`. Factored out of the DFS below
    // so the pure-voider post-pass (RFP Fulgora spike) can emit voider
    // machines the same way — voiders are demand-pulled sinks, not
    // producers, so they're structurally invisible to the producer-of-item
    // DFS and need their own emission pass (see the reachability-exemption
    // comment after the DFS loop).
    let build_machine_spec = |c: usize| -> MachineSpec {
        let col = &columns[c];
        let crafts_per_sec_per_machine = col.crafting_speed / col.recipe.energy;
        let count = snap(x_of(c) / crafts_per_sec_per_machine);
        // Self-loop items (RFP Phase 2): excluded from the ordinary
        // ingredient/product mapping below and emitted instead as a
        // single net flow (into inputs or outputs, by sign) plus a
        // `self_loop` entry carrying the raw per-machine rates for
        // the row template's loop-back belt sizing. Pure voiders (RFP
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
        let mut outputs: Vec<ItemFlow> = col
            .recipe
            .products
            .iter()
            .filter(|p| !self_loop_names.contains(&p.name.as_str()))
            .map(|p| ItemFlow {
                item: p.name.clone(),
                rate: p.amount * p.probability * crafts_per_sec_per_machine,
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
            let produced_rate = col
                .recipe
                .products
                .iter()
                .filter(|p| p.name == *name)
                .map(|p| p.amount * p.probability)
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
    let mut stack = vec![Work::Item(target_idx)];
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
                // the stack), matching resolve()'s loop at solver.rs:257.
                for ing in columns[c].recipe.ingredients.iter().rev() {
                    let i = items.index[ing.name.as_str()];
                    stack.push(Work::Item(i));
                }
            }
        }
    }

    // Pure-voider emission pass (RFP Fulgora spike, gated on allow_voiding).
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
                target: target_item.to_string(),
                detail: format!(
                    "surplus-processor exclusion did not converge (last: {first_unreachable})"
                ),
            },
            excludable: Some(first_unreachable),
        });
    }

    let surplus_outputs: Vec<ItemFlow> = (0..items.len())
        .filter(|&i| o_of(i) > ACTIVE_TOL)
        .map(|i| ItemFlow {
            item: items.names[i].to_string(),
            rate: o_of(i),
            is_fluid: items.is_fluid[i],
            module_id: 0,
        })
        .collect();

    let target_is_fluid = items.is_fluid[target_idx];
    Ok(SolverResult {
        machines,
        external_inputs,
        external_outputs: vec![ItemFlow {
            item: target_item.to_string(),
            rate: target_rate,
            is_fluid: target_is_fluid,
            module_id: 0,
        }],
        surplus_outputs,
        dependency_order,
    })
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
