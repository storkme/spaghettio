//! Recipe and entity lookups backed by a bundled `recipes.json`.

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Raw JSON payload bundled at compile time.
static RAW: &str = include_str!("../data/recipes.json");

/// Default crafting time when not specified in data.
const DEFAULT_ENERGY: f64 = 0.5;

/// Recipe categories that should never be used for production chains.
const EXCLUDED_CATEGORIES: &[&str] = &["recycling", "crushing", "recycling-or-hand-crafting"];

/// Recipes that should never be used to *produce* an ingredient, even though
/// they technically list one as a product. Barrel-emptying recipes
/// (`empty-X-barrel`) produce a fluid + empty barrel, but only by consuming a
/// pre-filled barrel — treating them as fluid producers makes the solver
/// recurse through the whole barrel chain (steel-plate → barrel → fill →
/// empty) for any recipe that needs the fluid. Callers who actually want to
/// empty barrels should target the filled-barrel item directly.
pub(crate) fn is_excluded_recipe(recipe: &Recipe) -> bool {
    if EXCLUDED_CATEGORIES.contains(&recipe.category.as_str()) {
        return true;
    }
    if recipe.name.starts_with("empty-") && recipe.name.ends_with("-barrel") {
        return true;
    }
    false
}

fn default_ingredient_type() -> String {
    "item".to_string()
}

fn default_probability() -> f64 {
    1.0
}

fn default_category() -> String {
    "crafting".to_string()
}

fn default_energy() -> f64 {
    DEFAULT_ENERGY
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub amount: f64,
    #[serde(default = "default_ingredient_type", rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub amount: f64,
    #[serde(default = "default_ingredient_type", rename = "type")]
    pub type_: String,
    #[serde(default = "default_probability")]
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_energy", alias = "energy_required")]
    pub energy: f64,
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(default, alias = "results")]
    pub products: Vec<Product>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineData {
    pub crafting_speed: f64,
    /// Maximum number of solid+fluid ingredient slots. `None` = unbounded
    /// (Factorio represents this as `-1` on prototypes; we treat absence in
    /// the data file as unbounded). Today only `assembling-machine-1` has a
    /// finite limit (2) in the bundled `recipes.json`.
    #[serde(default)]
    pub ingredient_slots: Option<usize>,
    /// Raw fluid-box descriptors copied from the prototype data. We don't
    /// inspect their shape — only whether the array is non-empty (machine
    /// supports fluids at all). Defaults to empty.
    #[serde(default)]
    pub fluid_boxes: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RawRoot {
    recipes: indexmap::IndexMap<String, Recipe>,
    #[serde(default)]
    machines: FxHashMap<String, MachineData>,
}

#[derive(Debug)]
pub struct RecipeDb {
    /// Recipes in JSON insertion order. Used for deterministic recipe selection
    /// (matches Python behaviour and is identical across native / WASM targets).
    pub recipes: indexmap::IndexMap<String, Recipe>,
    pub machines: FxHashMap<String, MachineData>,
}

static DB: LazyLock<RecipeDb> = LazyLock::new(|| {
    let root: RawRoot = serde_json::from_str(RAW).expect("recipes.json is malformed");
    RecipeDb {
        recipes: root.recipes,
        machines: root.machines,
    }
});

/// Global recipe database (lazily parsed from the bundled JSON).
pub fn db() -> &'static RecipeDb {
    &DB
}

/// Find the canonical recipe whose products include *item*.
///
/// Prefers the recipe whose `name` matches *item* (avoids e.g.
/// bioplastic → plastic-bar). Falls back to iterating in JSON insertion order
/// (`IndexMap`) so recipe selection is deterministic and identical across
/// native / WASM targets. Skips recycling / crushing recipes. Returns `None`
/// if no recipe produces this item.
pub fn find_recipe_for_item(item: &str) -> Option<&'static Recipe> {
    find_recipe_for_item_excluding(item, &FxHashSet::default())
}

/// Like [`find_recipe_for_item`] but skips recipes in `excluded`.
///
/// Used by the solver to honour caller-supplied recipe exclusions
/// (e.g. "don't use `coal-liquefaction`").
pub fn find_recipe_for_item_excluding(
    item: &str,
    excluded: &FxHashSet<String>,
) -> Option<&'static Recipe> {
    // Prefer name-matched recipe (canonical, avoids bioplastic→plastic-bar etc.)
    if !excluded.contains(item) {
        if let Some(recipe) = db().recipes.get(item) {
            if !is_excluded_recipe(recipe)
                && recipe.products.iter().any(|p| p.name == item)
            {
                return Some(recipe);
            }
        }
    }
    // Fall back to any non-excluded recipe producing this item (JSON order).
    for (name, recipe) in &db().recipes {
        if excluded.contains(name) {
            continue;
        }
        if is_excluded_recipe(recipe) {
            continue;
        }
        if recipe.products.iter().any(|p| p.name == item) {
            return Some(recipe);
        }
    }
    None
}

/// Return the crafting_speed of an entity, defaulting to 1.0 if unknown.
pub fn get_crafting_speed(entity: &str) -> f64 {
    db()
        .machines
        .get(entity)
        .map(|m| m.crafting_speed)
        .unwrap_or(1.0)
}

/// [`get_crafting_speed`] scaled by build quality — THE choke point for
/// quality-aware machine math (`docs/rfp-build-quality.md` Phase 1).
/// Both net-flow modes (free and compat) read speeds through this via
/// `NetflowOptions.quality`; there is deliberately no `Normal` branch, so
/// the default path exercises the same multiplication (kill criterion 2:
/// `× 1.0` is bit-exact in IEEE 754, unit-tested in `quality_identity_*`).
///
/// The legacy tree walk does NOT call this: it is a recipe-*selection*
/// oracle and selection is quality-invariant (JSON-first per item / cost
/// table — neither consults speed); its counts are documented known-wrong
/// and never reach a `SolverResult` callers use.
pub fn effective_crafting_speed(entity: &str, quality: crate::common::QualityTier) -> f64 {
    get_crafting_speed(entity) * quality.multiplier()
}

/// Reasons a chosen `(machine, recipe)` pair can't run as configured.
///
/// Surfaced via [`SolverError::IncompatibleMachine`](crate::solver::SolverError)
/// and rendered in the web UI's config-error banner. Messages are
/// user-facing so the wording in `Display` should suggest a fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineIncompatibility {
    /// Recipe lists more ingredients than the machine has slots
    /// (e.g. `assembling-machine-1` caps at 2).
    TooManyIngredients { limit: usize, got: usize },
    /// Recipe has fluid ingredients or products and the machine has no
    /// fluid boxes (AM1, stone/steel/electric furnaces, recycler, crusher).
    FluidNotSupported { items: Vec<String> },
    /// The chosen machine isn't a valid producer for the recipe's category.
    /// Only triggers for hand-edited URLs that map a category to a machine
    /// that never handles it (e.g. `?craft=stone-furnace`); the UI's
    /// dropdown prevents this through normal use.
    CategoryNotSupported { category: String },
}

impl std::fmt::Display for MachineIncompatibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineIncompatibility::TooManyIngredients { limit, got } => {
                write!(
                    f,
                    "{got} ingredients exceed the {limit}-slot limit — switch to a machine with more slots"
                )
            }
            MachineIncompatibility::FluidNotSupported { items } => {
                write!(
                    f,
                    "needs fluid {} but the machine has no fluid connections — switch to a fluid-capable machine",
                    items.join(", ")
                )
            }
            MachineIncompatibility::CategoryNotSupported { category } => {
                write!(f, "doesn't support recipe category `{category}`")
            }
        }
    }
}

/// Verify that `machine` can actually run `recipe` as configured.
///
/// Returns `Ok(())` if the pair is buildable, otherwise the first
/// incompatibility it hits (slot count → fluid → category). Unknown
/// machines (not in the bundled data) are treated as unconstrained — the
/// solver will hit `MissingCraftingSpeed` separately if the entity is
/// completely unknown.
pub fn machine_can_run_recipe(
    machine: &str,
    recipe: &Recipe,
) -> Result<(), MachineIncompatibility> {
    let Some(data) = db().machines.get(machine) else {
        return Ok(());
    };

    if let Some(limit) = data.ingredient_slots {
        let got = recipe.ingredients.len();
        if got > limit {
            return Err(MachineIncompatibility::TooManyIngredients { limit, got });
        }
    }

    if data.fluid_boxes.is_empty() {
        let mut fluids: Vec<String> = recipe
            .ingredients
            .iter()
            .filter(|i| i.type_ == "fluid")
            .map(|i| i.name.clone())
            .chain(
                recipe
                    .products
                    .iter()
                    .filter(|p| p.type_ == "fluid")
                    .map(|p| p.name.clone()),
            )
            .collect();
        fluids.sort();
        fluids.dedup();
        if !fluids.is_empty() {
            return Err(MachineIncompatibility::FluidNotSupported { items: fluids });
        }
    }

    // Category whitelist: a machine is valid for a category if either
    //   (a) `machine_for_recipe` would naturally pick it for that category, or
    //   (b) the category falls through to `default` (the assembler tier),
    //       in which case any non-specialised machine is fine.
    // The check guards hand-edited URLs like `?craft=stone-furnace`.
    if !machine_handles_category(machine, &recipe.category) {
        return Err(MachineIncompatibility::CategoryNotSupported {
            category: recipe.category.clone(),
        });
    }

    Ok(())
}

/// Assembler tiers — the general-purpose machines that handle every
/// non-specialised recipe category. Used both as the valid-machine set for
/// fall-through categories in [`machine_handles_category`] and as the
/// allowed-options gate in the web sidebar's crafting dropdown.
pub const ASSEMBLER_TIERS: &[&str] =
    &["assembling-machine-1", "assembling-machine-2", "assembling-machine-3"];

/// Single source of truth for the recipe-category → valid-machines mapping.
///
/// The first entry in each slice is the canonical default that
/// [`machine_for_recipe`] picks when the user hasn't overridden it via the
/// palette. An empty slice means the category is "general-purpose": the
/// caller's `default` machine is used (caller is expected to provide one
/// of [`ASSEMBLER_TIERS`]).
fn category_machines(category: &str) -> &'static [&'static str] {
    match category {
        "chemistry" | "chemistry-or-cryogenics" | "organic-or-chemistry" => &["chemical-plant"],
        "oil-processing" => &["oil-refinery"],
        "smelting" => &["electric-furnace", "stone-furnace", "steel-furnace"],
        "electromagnetics" => &["electromagnetic-plant"],
        "cryogenics" | "cryogenics-or-assembling" => &["cryogenic-plant"],
        "metallurgy" | "metallurgy-or-assembling" | "pressing" => &["foundry"],
        "organic" | "organic-or-assembling" => &["biochamber"],
        "centrifuging" => &["centrifuge"],
        // Fulgora scrap economy (docs/rfp-solver-net-flow.md spike): both
        // categories are excluded from normal solving by
        // EXCLUDED_CATEGORIES, so this mapping is inert on every default
        // path — it only matters once a caller opts in via
        // `NetflowOptions::allow_recycling`, at which point the recycler
        // needs to be resolvable like any other specialised machine.
        "recycling" | "recycling-or-hand-crafting" => &["recycler"],
        _ => &[],
    }
}

/// Categories that genuinely run on general-purpose assemblers. An explicit
/// whitelist, not a fall-through: before Phase 1 of
/// docs/rfp-solver-net-flow.md, *any* unmapped category (`centrifuging`,
/// `rocket-building`, …) silently landed on an assembler at assembler speed
/// — confidently wrong output with zero signal. Unknown categories now fail
/// [`machine_handles_category`] for every machine, surfacing as a typed
/// `IncompatibleMachine` error instead.
const GENERAL_CATEGORIES: &[&str] = &[
    "crafting",
    "advanced-crafting",
    "crafting-with-fluid",
    "crafting-with-fluid-or-metallurgy",
    "electronics",
    "electronics-or-assembling",
    "electronics-with-fluid",
    "organic-or-hand-crafting",
    "parameters",
];

/// Compatibility check between a machine and a recipe category. Specialised
/// categories require one of their listed machines; whitelisted general
/// categories require an [assembler tier](ASSEMBLER_TIERS); anything else
/// (`rocket-building`, `captive-spawner-process`, future unknowns) is
/// unsupported on every machine — silo/spawner semantics aren't modeled,
/// and saying so beats pretending.
fn machine_handles_category(machine: &str, category: &str) -> bool {
    let valid = category_machines(category);
    if !valid.is_empty() {
        valid.contains(&machine)
    } else if GENERAL_CATEGORIES.contains(&category) {
        ASSEMBLER_TIERS.contains(&machine)
    } else {
        false
    }
}

/// User-supplied per-category machine overrides.
///
/// Maps a recipe `category` string (the same key used in [`machine_for_recipe`])
/// to the entity name the user picked for it (e.g. `crafting →
/// assembling-machine-2`). Categories absent here fall through to the
/// hardcoded mapping in [`machine_for_recipe`], which itself falls through to
/// the caller's `default`.
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachinePalette {
    pub by_category: FxHashMap<String, String>,
}

/// Choose the right machine entity for a recipe based on its category.
pub fn machine_for_recipe(recipe: &Recipe, default: &str) -> String {
    machine_for_recipe_with_palette(recipe, &MachinePalette::default(), default)
}

/// Like [`machine_for_recipe`], but consults `palette` first. The palette is
/// the caller's per-category override; on miss we consult the same
/// category → machine table that [`machine_handles_category`] uses, and
/// finally fall through to `default` for general-purpose categories.
pub fn machine_for_recipe_with_palette(
    recipe: &Recipe,
    palette: &MachinePalette,
    default: &str,
) -> String {
    if let Some(override_machine) = palette.by_category.get(&recipe.category) {
        return override_machine.clone();
    }
    match category_machines(&recipe.category) {
        [] => default.to_string(),
        [first, ..] => (*first).to_string(),
    }
}

/// List all items that at least one non-excluded recipe produces.
pub fn all_producible_items() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: rustc_hash::FxHashSet<&str> = rustc_hash::FxHashSet::default();
    for recipe in db().recipes.values() {
        if is_excluded_recipe(recipe) {
            continue;
        }
        for p in &recipe.products {
            if seen.insert(p.name.as_str()) {
                out.push(p.name.clone());
            }
        }
    }
    out.sort();
    out
}

/// List all machine entity names known to the database.
pub fn all_producer_machines() -> Vec<String> {
    let mut out: Vec<String> = db().machines.keys().cloned().collect();
    out.sort();
    out
}

/// Return the canonical machine for an item, falling back to `fallback` if
/// the item has no recipe or the recipe uses the default crafting category.
pub fn default_machine_for_item(item: &str, fallback: &str) -> String {
    default_machine_for_item_with_palette(item, &MachinePalette::default(), fallback)
}

/// Like [`default_machine_for_item`], but consults the caller's palette before
/// the hardcoded category mapping.
pub fn default_machine_for_item_with_palette(
    item: &str,
    palette: &MachinePalette,
    fallback: &str,
) -> String {
    match find_recipe_for_item(item) {
        Some(recipe) => machine_for_recipe_with_palette(recipe, palette, fallback),
        None => fallback.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_iron_gear_wheel_recipe() {
        let recipe = find_recipe_for_item("iron-gear-wheel").expect("recipe exists");
        assert_eq!(recipe.name, "iron-gear-wheel");
        let iron_plate_ings: Vec<&Ingredient> = recipe
            .ingredients
            .iter()
            .filter(|i| i.name == "iron-plate")
            .collect();
        assert_eq!(iron_plate_ings.len(), 1);
        assert_eq!(iron_plate_ings[0].amount, 2.0);
    }

    /// Kill criterion 2a (rfp-build-quality): `effective_crafting_speed`
    /// at Normal must be bit-identical to the raw speed for every known
    /// machine (and the unknown-name fallback) — `×1.0` is an IEEE 754
    /// no-op, and this pins the helper against ever growing a divergent
    /// code path.
    #[test]
    fn effective_speed_normal_is_bit_identical() {
        use crate::common::{QualityTier, MACHINE_ENTITY_NAMES};
        for m in MACHINE_ENTITY_NAMES.iter().chain(&["nonexistent-machine"]) {
            assert_eq!(
                effective_crafting_speed(m, QualityTier::Normal).to_bits(),
                get_crafting_speed(m).to_bits(),
                "{m}"
            );
        }
        // And the scaling itself, spot-checked on the RFP's anchor values.
        assert_eq!(
            effective_crafting_speed("assembling-machine-3", QualityTier::Legendary),
            3.125
        );
        assert_eq!(
            effective_crafting_speed("electric-furnace", QualityTier::Legendary),
            5.0
        );
    }

    #[test]
    fn crafting_speed_defaults_to_one() {
        assert_eq!(get_crafting_speed("nonexistent-machine"), 1.0);
        assert!(get_crafting_speed("assembling-machine-3") > 0.0);
    }

    #[test]
    fn default_machine_for_item_picks_correct_machine() {
        assert_eq!(
            default_machine_for_item("plastic-bar", "assembling-machine-3"),
            "chemical-plant"
        );
        assert_eq!(
            default_machine_for_item("iron-gear-wheel", "assembling-machine-3"),
            "assembling-machine-3"
        );
        assert_eq!(
            default_machine_for_item("nonexistent-item-xyz", "assembling-machine-3"),
            "assembling-machine-3"
        );
    }

    #[test]
    fn space_age_category_mappings() {
        // electromagnetics → electromagnetic-plant
        let recipe = Recipe {
            name: "test-recipe".into(),
            category: "electromagnetics".into(),
            energy: 1.0,
            ingredients: vec![],
            products: vec![],
        };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "electromagnetic-plant");

        // cryogenics → cryogenic-plant
        let recipe = Recipe { category: "cryogenics".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "cryogenic-plant");

        // cryogenics-or-assembling → cryogenic-plant (prefer SA machine)
        let recipe = Recipe { category: "cryogenics-or-assembling".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "cryogenic-plant");

        // metallurgy → foundry
        let recipe = Recipe { category: "metallurgy".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "foundry");

        // metallurgy-or-assembling → foundry
        let recipe = Recipe { category: "metallurgy-or-assembling".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "foundry");

        // pressing → foundry
        let recipe = Recipe { category: "pressing".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "foundry");

        // organic → biochamber
        let recipe = Recipe { category: "organic".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "biochamber");

        // organic-or-assembling → biochamber
        let recipe = Recipe { category: "organic-or-assembling".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "biochamber");

        // chemistry-or-cryogenics still maps to chemical-plant
        let recipe = Recipe { category: "chemistry-or-cryogenics".into(), ..recipe.clone() };
        assert_eq!(machine_for_recipe(&recipe, "assembling-machine-3"), "chemical-plant");
    }

    fn palette_with(entries: &[(&str, &str)]) -> MachinePalette {
        let mut p = MachinePalette::default();
        for (k, v) in entries {
            p.by_category.insert((*k).to_string(), (*v).to_string());
        }
        p
    }

    fn make_recipe(category: &str) -> Recipe {
        Recipe {
            name: "test-recipe".into(),
            category: category.into(),
            energy: 1.0,
            ingredients: vec![],
            products: vec![],
        }
    }

    #[test]
    fn palette_overrides_default_for_crafting() {
        let palette = palette_with(&[("crafting", "assembling-machine-2")]);
        let recipe = make_recipe("crafting");
        assert_eq!(
            machine_for_recipe_with_palette(&recipe, &palette, "assembling-machine-3"),
            "assembling-machine-2"
        );
    }

    #[test]
    fn palette_miss_falls_through_to_hardcoded() {
        // Empty palette: smelting still picks the hardcoded electric-furnace.
        let palette = MachinePalette::default();
        let recipe = make_recipe("smelting");
        assert_eq!(
            machine_for_recipe_with_palette(&recipe, &palette, "assembling-machine-3"),
            "electric-furnace"
        );
    }

    #[test]
    fn palette_miss_falls_through_to_default() {
        // Empty palette + crafting category falls through to caller's default.
        let palette = MachinePalette::default();
        let recipe = make_recipe("crafting");
        assert_eq!(
            machine_for_recipe_with_palette(&recipe, &palette, "assembling-machine-1"),
            "assembling-machine-1"
        );
    }

    #[test]
    fn palette_does_not_override_hardcoded_when_absent() {
        // Palette only sets crafting; smelting recipe still hits hardcoded path.
        let palette = palette_with(&[("crafting", "assembling-machine-1")]);
        let recipe = make_recipe("smelting");
        assert_eq!(
            machine_for_recipe_with_palette(&recipe, &palette, "assembling-machine-3"),
            "electric-furnace"
        );
    }

    #[test]
    fn am1_rejects_recipe_with_too_many_ingredients() {
        // advanced-circuit needs 3 ingredients; AM1 has 2 slots.
        let recipe = find_recipe_for_item("advanced-circuit").expect("recipe exists");
        let err = machine_can_run_recipe("assembling-machine-1", recipe).unwrap_err();
        match err {
            MachineIncompatibility::TooManyIngredients { limit, got } => {
                assert_eq!(limit, 2);
                assert!(got >= 3, "expected ≥3 ingredients, got {got}");
            }
            other => panic!("expected TooManyIngredients, got {other:?}"),
        }
    }

    #[test]
    fn am1_rejects_recipe_with_fluid_ingredient() {
        // plastic-bar uses petroleum-gas (fluid). AM1 has no fluid boxes.
        let recipe = find_recipe_for_item("plastic-bar").expect("recipe exists");
        // plastic-bar's category is "chemistry" → category check would fire
        // first. Test against a fluid-using crafting recipe that AM1 fails on
        // the fluid check, not the category check.
        // Use the 2-ingredient lubricant recipe: any AM2/3-only recipe with
        // a fluid will do. Fall back to a hardcoded constructed recipe if
        // none is in the DB to keep this test stable.
        let _ = recipe; // keep the lookup as a sanity check on data.
        let synthetic = Recipe {
            name: "synthetic".into(),
            category: "crafting-with-fluid".into(),
            energy: 1.0,
            ingredients: vec![
                Ingredient {
                    name: "iron-plate".into(),
                    amount: 1.0,
                    type_: "item".into(),
                },
                Ingredient {
                    name: "water".into(),
                    amount: 10.0,
                    type_: "fluid".into(),
                },
            ],
            products: vec![],
        };
        let err = machine_can_run_recipe("assembling-machine-1", &synthetic).unwrap_err();
        assert!(matches!(err, MachineIncompatibility::FluidNotSupported { .. }));
    }

    #[test]
    fn am3_accepts_arbitrary_recipes() {
        let recipe = find_recipe_for_item("advanced-circuit").expect("recipe exists");
        machine_can_run_recipe("assembling-machine-3", recipe)
            .expect("AM3 handles 3+ ingredients");
    }

    #[test]
    fn category_mismatch_rejected() {
        // Hand-edited URL would put stone-furnace in the crafting palette.
        let recipe = make_recipe("crafting");
        let err = machine_can_run_recipe("stone-furnace", &recipe).unwrap_err();
        assert!(matches!(
            err,
            MachineIncompatibility::CategoryNotSupported { .. }
        ));
    }

    #[test]
    fn smelting_accepts_stone_furnace() {
        // Conversely, stone-furnace IS valid for the smelting category.
        let recipe = make_recipe("smelting");
        machine_can_run_recipe("stone-furnace", &recipe)
            .expect("stone-furnace handles smelting");
    }

    #[test]
    fn unknown_machine_passes_through() {
        // Unknown entities are treated as unconstrained — the solver hits
        // MissingCraftingSpeed separately for these.
        let recipe = make_recipe("crafting");
        machine_can_run_recipe("totally-made-up-machine", &recipe)
            .expect("unknown machines bypass the capability check");
    }

    #[test]
    fn palette_can_override_hardcoded_smelting() {
        // If a future palette wants stone-furnace for smelting, it wins over
        // the hardcoded electric-furnace.
        let palette = palette_with(&[("smelting", "stone-furnace")]);
        let recipe = make_recipe("smelting");
        assert_eq!(
            machine_for_recipe_with_palette(&recipe, &palette, "assembling-machine-3"),
            "stone-furnace"
        );
    }
}
