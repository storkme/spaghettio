//! Crafting machines: ingredient buffers, a craft timer, an output slot.
//!
//! # What is deliberately taken from `spaghettio_core`, and what is not
//!
//! Taken: `recipe_db::db()` — recipe ingredients, products, `energy`
//! (craft time), and `MachineData::crafting_speed`. These are Factorio
//! prototype facts.
//!
//! **Not taken**: `effective_crafting_speed`, which folds in the engine's
//! quality model, and anything from `module_policy`. The meter reads
//! declared inputs from the manifest and applies them itself; inheriting
//! the engine's derived speed would put a hand-calibrated number inside
//! the instrument meant to check it. Same argument as KC4, which the
//! independence test enforces.
//!
//! # States
//!
//! Machine states use the *harness's* census vocabulary — `working`,
//! `full_output`, `item_ingredient_shortage` — so a `MeterReport` census
//! is directly comparable with a `spaghettio-sim` one and the forensics in
//! `scripts/sim-capture-state.sh` transfer unchanged.

use rustc_hash::FxHashMap;
use spaghettio_core::recipe_db;

use crate::belt::ItemId;
use crate::world::ItemInterner;

/// How many crafts' worth of each ingredient a machine will buffer.
///
/// **This is an approximation and it is not measured.** Factorio's real
/// insertion limit for assembling machines is a threshold check followed
/// by a whole-hand insert, so the observed ceiling depends on the
/// inserter's hand size as well as the recipe. #448's dumps show EC
/// machines (3 copper-cable per craft) holding 42 — far more than the
/// "~2 crafts" rule of thumb would predict.
///
/// Rather than reverse-engineer a constant from one observation — the
/// mistake that produced the mis-transcribed hand-size ladders — this is
/// left as an explicit, sweepable parameter with a stated default, and
/// flagged as a top candidate for `docs/meter-divergence.md` and for
/// measurement in PR 2. It matters: buffer depth is one of the inputs to
/// the head-buffers-starve-tail mechanism the meter exists to study.
pub const DEFAULT_BUFFER_CRAFTS: u32 = 14;

/// Machine status, matching `spaghettio-sim`'s census names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MachineState {
    Working,
    FullOutput,
    ItemIngredientShortage,
}

impl MachineState {
    /// The exact string the sim harness reports, so censuses line up.
    pub fn as_str(self) -> &'static str {
        match self {
            MachineState::Working => "working",
            MachineState::FullOutput => "full_output",
            MachineState::ItemIngredientShortage => "item_ingredient_shortage",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub recipe: String,
    /// Top-left tile and footprint, so inserters can find it.
    pub pos: (i32, i32),
    pub size: (u32, u32),
    /// Ticks for one craft: `recipe.energy / crafting_speed * 60`.
    craft_ticks: f64,
    progress: f64,
    /// (item, per-craft amount) — solids only; fluids are PR-3 out of scope.
    pub ingredients: Vec<(ItemId, u32)>,
    /// Products per craft, as **expected** amounts — fractional for
    /// probabilistic recipes. Whole units are emitted via `product_debt`.
    ///
    /// **Private on purpose.** Anything outside this module that derives
    /// "what was produced" from these expectations needs its own carry and
    /// will drift from the one here — which is precisely what happened:
    /// `Factory` cast the expectation straight to `u64`, truncating 0.25 to
    /// 0 while the machine's carry got it right, so two halves of one report
    /// disagreed. Read [`Machine::emitted_this_tick`] instead; the mistake
    /// is now unrepresentable rather than merely fixed.
    products: Vec<(ItemId, f64)>,
    /// Ingredients on hand.
    pub input: FxHashMap<u16, u32>,
    /// Fractional carry per product, for probabilistic recipes.
    product_debt: FxHashMap<u16, f64>,
    /// Whole units emitted into `output` by the most recent `tick`.
    ///
    /// **The single source of truth for "what was produced".** `products`
    /// holds fractional *expectations*, so anything that re-derives
    /// production from it needs its own carry and will drift from the
    /// carry here. `Factory` credited `crafted` by casting the expectation
    /// straight to `u64`, which truncated every sub-1 product to zero and
    /// lost a third of a 1.5 — so `produced_per_s` and belt-delivered
    /// throughput, two halves of one report, silently disagreed. Reading
    /// the emitted units means there is only one accumulator.
    pub emitted_this_tick: Vec<(u16, u32)>,
    /// Finished products awaiting an output inserter.
    pub output: FxHashMap<u16, u32>,
    /// Per-ingredient buffer ceiling.
    pub buffer_cap: FxHashMap<u16, u32>,
    /// Output ceiling before the machine blocks.
    pub output_cap: u32,
    pub state: MachineState,
    pub crafts: u64,
    /// Recipe ingredients that are fluids — recorded so the report can say
    /// so rather than silently treating the machine as solid-fed.
    pub fluid_ingredients: Vec<String>,
}

impl Machine {
    /// Build from a blueprint entity. `None` when the recipe is unknown.
    pub fn new(
        name: &str,
        recipe_name: &str,
        pos: (i32, i32),
        size: (u32, u32),
        items: &mut ItemInterner,
        buffer_crafts: u32,
    ) -> Option<Self> {
        let db = recipe_db::db();
        let recipe = db.recipes.get(recipe_name)?;
        // Raw prototype speed, NOT the engine's effective_crafting_speed.
        let speed = db.machines.get(name).map(|m| m.crafting_speed)?;
        if speed <= 0.0 {
            return None;
        }

        let mut ingredients = Vec::new();
        let mut fluid_ingredients = Vec::new();
        let mut buffer_cap = FxHashMap::default();
        for ing in &recipe.ingredients {
            if ing.type_ == "fluid" {
                fluid_ingredients.push(ing.name.clone());
                continue;
            }
            let id = items.intern(&ing.name);
            let amount = ing.amount.ceil().max(1.0) as u32;
            ingredients.push((id, amount));
            buffer_cap.insert(id.0, amount * buffer_crafts);
        }

        let mut products = Vec::new();
        for p in &recipe.products {
            if p.type_ == "fluid" {
                continue;
            }
            // Probabilistic products are credited at **expectation**, and
            // the meter does not roll dice: a stochastic meter cannot be
            // compared tick-for-tick against a deterministic baseline.
            //
            // Kept as an f64 expectation rather than a rounded integer.
            // `(amount * probability).round().max(1.0)` -- the first
            // version -- credited a whole unit per craft no matter how
            // small the probability, so a p=0.25 recycling product was
            // over-credited 4x and uranium-235 at p=0.007 by ~143x, while
            // the comment above it claimed expectation. No corpus fixture
            // exercises a probabilistic recipe, so nothing caught it.
            products.push((items.intern(&p.name), p.amount * p.probability));
        }

        Some(Machine {
            name: name.to_string(),
            recipe: recipe_name.to_string(),
            pos,
            size,
            craft_ticks: (recipe.energy / speed) * 60.0,
            progress: 0.0,
            ingredients,
            products,
            product_debt: FxHashMap::default(),
            emitted_this_tick: Vec::new(),
            input: FxHashMap::default(),
            output: FxHashMap::default(),
            buffer_cap,
            output_cap: 100,
            state: MachineState::ItemIngredientShortage,
            crafts: 0,
            fluid_ingredients,
        })
    }

    pub fn covers(&self, tile: (i32, i32)) -> bool {
        tile.0 >= self.pos.0
            && tile.0 < self.pos.0 + self.size.0 as i32
            && tile.1 >= self.pos.1
            && tile.1 < self.pos.1 + self.size.1 as i32
    }

    /// How many more of `item` this machine will accept.
    pub fn room_for(&self, item: ItemId) -> u32 {
        let Some(&cap) = self.buffer_cap.get(&item.0) else {
            return 0; // not an ingredient of this recipe
        };
        cap.saturating_sub(self.input.get(&item.0).copied().unwrap_or(0))
    }

    /// Insert up to `room_for`; returns how many were taken.
    pub fn insert(&mut self, item: ItemId, count: u32) -> u32 {
        let take = count.min(self.room_for(item));
        if take > 0 {
            *self.input.entry(item.0).or_insert(0) += take;
        }
        take
    }

    /// Remove up to `max` finished products of any kind.
    pub fn take_output(&mut self, max: u32, out: &mut Vec<ItemId>) {
        let mut remaining = max;
        // Deterministic order — a HashMap iteration order would make runs
        // irreproducible, and reproducibility is the whole basis for
        // comparing against frozen baselines.
        let mut keys: Vec<u16> = self.output.keys().copied().collect();
        keys.sort_unstable();
        for k in keys {
            if remaining == 0 {
                break;
            }
            let held = self.output.get(&k).copied().unwrap_or(0);
            let take = held.min(remaining);
            if take > 0 {
                *self.output.get_mut(&k).unwrap() -= take;
                for _ in 0..take {
                    out.push(ItemId(k));
                }
                remaining -= take;
            }
        }
        self.output.retain(|_, v| *v > 0);
    }

    fn total_output(&self) -> u32 {
        self.output.values().sum()
    }

    fn has_ingredients(&self) -> bool {
        self.ingredients
            .iter()
            .all(|(id, amount)| self.input.get(&id.0).copied().unwrap_or(0) >= *amount)
    }

    /// Advance one tick.
    pub fn tick(&mut self) {
        self.emitted_this_tick.clear();
        // Fluid-fed machines cannot run in PR 3. They are held in a
        // shortage state rather than allowed to craft from nothing, so a
        // fluid chain under-reports honestly instead of over-reporting.
        if !self.fluid_ingredients.is_empty() {
            self.state = MachineState::ItemIngredientShortage;
            return;
        }
        if self.total_output() >= self.output_cap {
            self.state = MachineState::FullOutput;
            return;
        }
        if self.progress <= 0.0 {
            if !self.has_ingredients() {
                self.state = MachineState::ItemIngredientShortage;
                return;
            }
            // Consume and start a craft.
            for (id, amount) in &self.ingredients {
                *self.input.get_mut(&id.0).unwrap() -= amount;
            }
            self.progress = self.craft_ticks;
        }
        self.state = MachineState::Working;
        self.progress -= 1.0;
        if self.progress <= 0.0 {
            // Fractional expectations accumulate and emit whole units as
            // they cross 1.0, so the long-run rate is the expectation
            // without ever inventing a fraction of an item. Rounding per
            // craft instead would credit 0 forever for anything under 0.5.
            for (id, amount) in &self.products {
                let debt = self.product_debt.entry(id.0).or_insert(0.0);
                *debt += *amount;
                let whole = debt.floor();
                if whole >= 1.0 {
                    *debt -= whole;
                    *self.output.entry(id.0).or_insert(0) += whole as u32;
                    self.emitted_this_tick.push((id.0, whole as u32));
                }
            }
            self.crafts += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    /// A probabilistic product must be credited at **expectation over many
    /// crafts**, not a whole unit per craft. The first version rounded up
    /// to `.max(1.0)`, so p=0.25 read 4x high and p=0.007 ~143x high while
    /// claiming expectation in its own comment.
    #[test]
    fn probabilistic_products_accumulate_to_expectation() {
        let mut items = ItemInterner::default();
        let id = items.intern("scrap-output");
        let mut m = Machine {
            name: "test".into(),
            recipe: "test".into(),
            pos: (0, 0),
            size: (3, 3),
            craft_ticks: 1.0,
            progress: 0.0,
            ingredients: Vec::new(),
            products: vec![(id, 0.25)],
            product_debt: FxHashMap::default(),
            emitted_this_tick: Vec::new(),
            input: FxHashMap::default(),
            output: FxHashMap::default(),
            buffer_cap: FxHashMap::default(),
            output_cap: u32::MAX,
            crafts: 0,
            state: MachineState::Working,
            fluid_ingredients: Vec::new(),
        };
        // output_cap is unbounded here, so nothing blocks and the machine
        // crafts every tick.
        for _ in 0..400 {
            m.tick();
        }
        assert_eq!(m.crafts, 400, "sanity: one craft per tick at craft_ticks=1");
        let produced = m.output.get(&id.0).copied().unwrap_or(0);
        assert_eq!(
            produced, 100,
            "400 crafts at p=0.25 must credit 100 whole units, not 400"
        );
    }

    fn ec_machine(items: &mut ItemInterner) -> Machine {
        Machine::new(
            "assembling-machine-3",
            "electronic-circuit",
            (0, 0),
            (3, 3),
            items,
            DEFAULT_BUFFER_CRAFTS,
        )
        .expect("EC on AM3 is a known recipe")
    }

    /// Craft time must be derived from prototype data, not assumed.
    /// EC is 0.5 s on an AM3 at crafting speed 1.25 => 0.4 s => 24 ticks.
    #[test]
    fn craft_time_derives_from_recipe_and_machine_speed() {
        let mut items = ItemInterner::default();
        let m = ec_machine(&mut items);
        assert!(
            (m.craft_ticks - 24.0).abs() < 1e-9,
            "expected 24 ticks, got {}",
            m.craft_ticks
        );
    }

    #[test]
    fn ingredients_and_products_match_the_recipe() {
        let mut items = ItemInterner::default();
        let m = ec_machine(&mut items);
        // EC: 1 iron-plate + 3 copper-cable -> 1 electronic-circuit.
        assert_eq!(m.ingredients.len(), 2);
        let amounts: Vec<u32> = m.ingredients.iter().map(|(_, a)| *a).collect();
        assert!(amounts.contains(&1) && amounts.contains(&3), "{amounts:?}");
        assert_eq!(m.products.len(), 1);
        assert_eq!(m.products[0].1, 1.0);
    }

    /// A fed machine must produce at exactly the rate the prototype data
    /// implies — 2.5/s for EC on an AM3. This is the number the whole
    /// instrument rests on.
    #[test]
    fn a_fed_machine_crafts_at_the_prototype_rate() {
        let mut items = ItemInterner::default();
        let mut m = ec_machine(&mut items);
        let (iron, cable) = (items.intern("iron-plate"), items.intern("copper-cable"));

        let ticks = 36_000u64;
        let mut produced = 0u64;
        let mut sink = Vec::new();
        for _ in 0..ticks {
            m.insert(iron, 4);
            m.insert(cable, 12);
            m.tick();
            sink.clear();
            m.take_output(100, &mut sink);
            produced += sink.len() as u64;
        }
        let rate = produced as f64 / (ticks as f64 / 60.0);
        assert!(
            (rate - 2.5).abs() < 0.02,
            "EC on AM3 should craft 2.5/s, got {rate:.3}"
        );
    }

    /// Starve one ingredient and the machine must report the shortage,
    /// not quietly craft anyway.
    #[test]
    fn missing_one_ingredient_is_a_shortage() {
        let mut items = ItemInterner::default();
        let mut m = ec_machine(&mut items);
        let cable = items.intern("copper-cable");
        for _ in 0..600 {
            m.insert(cable, 12); // plenty of cable, zero iron
            m.tick();
        }
        assert_eq!(m.state, MachineState::ItemIngredientShortage);
        assert_eq!(m.crafts, 0);
    }

    /// An unemptied machine must block rather than produce forever.
    #[test]
    fn unemptied_output_blocks_the_machine() {
        let mut items = ItemInterner::default();
        let mut m = ec_machine(&mut items);
        let (iron, cable) = (items.intern("iron-plate"), items.intern("copper-cable"));
        for _ in 0..36_000 {
            m.insert(iron, 4);
            m.insert(cable, 12);
            m.tick();
        }
        assert_eq!(m.state, MachineState::FullOutput);
        assert!(m.crafts <= m.output_cap as u64 + 1);
    }

    /// The buffer ceiling must bind — an unbounded input is what made the
    /// PR-1 row fixture unfaithful.
    #[test]
    fn ingredient_buffer_has_a_ceiling() {
        let mut items = ItemInterner::default();
        let mut m = ec_machine(&mut items);
        let cable = items.intern("copper-cable");
        let mut total = 0;
        for _ in 0..1000 {
            total += m.insert(cable, 50);
        }
        assert_eq!(total, 3 * DEFAULT_BUFFER_CRAFTS, "3 cable/craft x buffer");
        assert_eq!(m.room_for(cable), 0);
    }

    /// A fluid-fed recipe must refuse to run rather than craft from
    /// nothing — under-report honestly, never over-report.
    #[test]
    fn fluid_fed_machines_refuse_to_craft_in_pr3() {
        let mut items = ItemInterner::default();
        let m = Machine::new(
            "chemical-plant",
            "sulfuric-acid",
            (0, 0),
            (3, 3),
            &mut items,
            DEFAULT_BUFFER_CRAFTS,
        );
        if let Some(mut m) = m {
            assert!(!m.fluid_ingredients.is_empty(), "sulfuric acid needs water");
            for _ in 0..600 {
                m.tick();
            }
            assert_eq!(m.crafts, 0);
            assert_eq!(m.state, MachineState::ItemIngredientShortage);
        }
    }

    #[test]
    fn unknown_recipe_or_machine_is_refused() {
        let mut items = ItemInterner::default();
        assert!(Machine::new(
            "assembling-machine-3",
            "not-a-real-recipe",
            (0, 0),
            (3, 3),
            &mut items,
            14
        )
        .is_none());
        assert!(Machine::new(
            "not-a-real-machine",
            "electronic-circuit",
            (0, 0),
            (3, 3),
            &mut items,
            14
        )
        .is_none());
    }
}
