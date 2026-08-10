//! Phase-0 probe 1 for the cell-interface DB idea: the MOTIF CENSUS.
//!
//! Question: across the repo's declared demand corpus, how concentrated is
//! demand at the machine-group level? If a handful of (recipe, machine)
//! motifs carry most of the machine mass, a cached-implementation DB pays;
//! if the distribution is flat, it does not, and the idea dies here for the
//! price of a solver sweep.
//!
//! Method: SOLVER ONLY — no layout. For every corpus entry, solve and emit
//! one unit motif per machine group `(recipe, machine)` and one edge motif
//! per producer→consumer item flow between groups. Aggregate:
//!   - fixtures touched (breadth of demand),
//!   - total ceil(machine count) (mass of demand),
//!   - count distribution (min/median/max) per motif.
//! Concentration is reported as top-K motifs' share of total machine mass —
//! the number the whole DB idea hinges on.
//!
//! Corpus: a verbatim snapshot of `survey_fixtures()` from
//! `crates/core/tests/e2e.rs` at cd78eed7 (examples cannot import from
//! tests/). Entries differing only by LAYOUT strategy variant are collapsed
//! (same solve → one demand observation); the dedupe is on
//! (item, rate, machine, inputs, excluded). DECLARED BIAS: this measures the
//! test suite's demand distribution, not the world's — probe 4 (community
//! mining) is the independent demand signal.
//!
//! Module assumption: bare machines, no modules/quality — matching the
//! corpus fixtures. Rates are DERIVED from counts by the current solver, so
//! this census stays valid if the rate model changes (productivity work);
//! counts are the stored quantity.
use rustc_hash::FxHashSet;
use spaghettio_core::solver;
use std::collections::BTreeMap;

struct F(
    &'static str,                // item
    f64,                         // rate
    &'static str,                // machine
    &'static [&'static str],     // inputs
    &'static [&'static str],     // excluded recipes
);

fn corpus() -> Vec<F> {
    vec![
        F("iron-gear-wheel", 10.0, "assembling-machine-1", &["iron-plate"], &[]),
        F("iron-gear-wheel", 10.0, "assembling-machine-2", &["iron-ore"], &[]),
        F("iron-gear-wheel", 20.0, "assembling-machine-2", &["iron-plate"], &[]),
        F("electronic-circuit", 10.0, "assembling-machine-2", &["iron-plate", "copper-plate"], &[]),
        F("electronic-circuit", 10.0, "assembling-machine-1", &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 20.0, "assembling-machine-2", &["iron-ore", "copper-ore"], &[]),
        F("plastic-bar", 10.0, "chemical-plant", &["petroleum-gas", "coal"], &[]),
        F("plastic-bar", 10.0, "chemical-plant", &["crude-oil", "coal"], &[]),
        F("sulfuric-acid", 5.0, "chemical-plant", &["iron-plate", "sulfur", "water"], &[]),
        F("light-oil", 5.0, "chemical-plant", &["water", "heavy-oil"], &["advanced-oil-processing", "coal-liquefaction"]),
        F("petroleum-gas", 12.0, "oil-refinery", &["water", "crude-oil"], &[]),
        F("petroleum-gas", 24.0, "oil-refinery", &["water", "crude-oil"], &["basic-oil-processing", "coal-liquefaction"]),
        F("advanced-circuit", 1.0, "assembling-machine-2", &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("advanced-circuit", 5.0, "assembling-machine-2", &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], &[]),
        F("processing-unit", 2.0, "assembling-machine-3", &["iron-ore", "copper-ore", "coal", "water", "crude-oil"], &[]),
        F("uranium-235", 0.1, "assembling-machine-3", &["uranium-238"], &["uranium-processing"]),
        F("uranium-235", 0.05, "assembling-machine-3", &["uranium-ore"], &["kovarex-enrichment-process"]),
        F("pentapod-egg", 0.2, "assembling-machine-3", &["nutrients", "water"], &[]),
        F("raw-fish", 0.15, "assembling-machine-3", &["nutrients", "water"], &[]),
        F("iron-bacteria", 1.0, "assembling-machine-3", &["bioflux"], &["iron-bacteria"]),
        F("electronic-circuit", 30.0, "assembling-machine-2", &["iron-ore", "copper-ore"], &[]),
        F("advanced-circuit", 45.0, "assembling-machine-2", &["iron-plate", "copper-plate", "plastic-bar"], &[]),
        F("advanced-circuit", 5.0, "assembling-machine-2", &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("advanced-circuit", 4.0, "assembling-machine-2", &["iron-plate", "copper-plate", "coal", "crude-oil", "water"], &[]),
        F("electronic-circuit", 60.0, "assembling-machine-2", &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 22.0, "assembling-machine-2", &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 23.0, "assembling-machine-2", &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 35.0, "assembling-machine-2", &["iron-ore", "copper-ore"], &[]),
        F("electronic-circuit", 40.0, "assembling-machine-2", &["iron-ore", "copper-ore"], &[]),
    ]
}

#[derive(Default)]
struct Unit {
    fixtures: usize,
    machines: f64,
    counts: Vec<f64>,
}

fn main() {
    let corpus = corpus();
    // Dedupe on the solver-relevant key (fixtures that differ only by layout
    // variant collapse to one demand observation).
    let mut seen: FxHashSet<String> = FxHashSet::default();
    let mut units: BTreeMap<(String, String), Unit> = BTreeMap::new();
    let mut edges: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    let (mut solved, mut refused) = (0usize, 0usize);

    for F(item, rate, machine, inputs, excluded) in &corpus {
        let key = format!("{item}|{rate}|{machine}|{inputs:?}|{excluded:?}");
        if !seen.insert(key) {
            continue;
        }
        let input_set: FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
        let excl: FxHashSet<String> = excluded.iter().map(|s| s.to_string()).collect();
        let sr = match solver::solve_with_exclusions(item, *rate, &input_set, machine, &excl) {
            Ok(sr) => sr,
            Err(e) => {
                println!("REFUSED {item}@{rate} ({machine}): {e:?}");
                refused += 1;
                continue;
            }
        };
        solved += 1;

        // Unit motifs.
        for m in &sr.machines {
            let u = units.entry((m.recipe.clone(), m.entity.clone())).or_default();
            u.fixtures += 1;
            u.machines += m.count.ceil();
            u.counts.push(m.count);
        }
        // Edge motifs: producer -> consumer on a shared item. The solver
        // output is a DAG (byproducts, shared intermediates), so an item may
        // have several producers and consumers; every pair that shares the
        // item is one edge observation.
        for p in &sr.machines {
            for out in &p.outputs {
                for c in &sr.machines {
                    if std::ptr::eq(p, c) {
                        continue;
                    }
                    if c.inputs.iter().any(|i| i.item == out.item) {
                        *edges
                            .entry((p.recipe.clone(), c.recipe.clone(), out.item.clone()))
                            .or_default() += 1;
                    }
                }
            }
        }
    }

    println!("\n===== motif census: {solved} solves ({refused} refused), {} deduped corpus entries =====", seen.len());

    let total_mass: f64 = units.values().map(|u| u.machines).sum();
    let mut rows: Vec<(&(String, String), &Unit)> = units.iter().collect();
    rows.sort_by(|a, b| b.1.machines.partial_cmp(&a.1.machines).unwrap());

    println!("\n--- unit motifs by machine mass (total mass {total_mass:.0}) ---");
    println!("{:<34} {:<22} {:>4} {:>6} {:>7} {:>18}", "recipe", "machine", "fixt", "mass", "share%", "count min/med/max");
    let mut cum = 0.0;
    for (i, ((recipe, entity), u)) in rows.iter().enumerate() {
        cum += u.machines;
        let mut c = u.counts.clone();
        c.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = c[c.len() / 2];
        println!(
            "{recipe:<34} {entity:<22} {:>4} {:>6.0} {:>6.1}% {:>7.1}/{med:.1}/{:.1}{}",
            u.fixtures,
            u.machines,
            100.0 * u.machines / total_mass,
            c[0],
            c[c.len() - 1],
            if i < 12 { format!("   cum {:.1}%", 100.0 * cum / total_mass) } else { String::new() },
        );
    }

    println!("\n--- edge motifs (producer -> consumer via item), by fixture count ---");
    let mut erows: Vec<(&(String, String, String), &usize)> = edges.iter().collect();
    erows.sort_by(|a, b| b.1.cmp(a.1));
    for ((p, c, item), n) in erows.iter().take(25) {
        println!("{n:>4}  {p} -> {c}  [{item}]");
    }
    println!("({} distinct edge motifs total)", erows.len());
}
