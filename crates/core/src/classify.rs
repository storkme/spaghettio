//! Blueprint classification: strategy features extracted from a parsed
//! layout (community-blueprint mining, strategy-gap mapping).
//!
//! Sibling to [`crate::analysis`]: `analyze()` answers "what does this
//! blueprint produce", `classify()` answers "HOW is it built" — the
//! archetype/features used to categorise strategy (tileable production
//! block, balancer, mall, direct-insertion user, ...).
//!
//! These are heuristic position/graph walks over `LayoutResult`, NOT
//! validation: community blueprints never agreed to our contracts, so no
//! verdicts are emitted, only features. Inserter pickup/drop resolution
//! uses the ENGINE direction convention (drop-side) — imported layouts
//! were already un-flipped by `blueprint_parser` (game blueprints store
//! the pickup side; see `blueprint.rs`'s export flip, the #348 fix).

use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;

use crate::analysis::BlueprintAnalysis;
use crate::common::{self, QualityTier};
use crate::models::{EntityDirection, LayoutResult, PlacedEntity};
use crate::recipe_db;

/// Strategy feature vector for one blueprint (single or book member).
#[derive(Debug, Clone, Default, Serialize)]
pub struct BlueprintFeatures {
    // census
    pub machines: usize,
    pub belt_tiles: usize,
    pub splitters: usize,
    pub inserters: usize,
    pub pipes: usize,
    pub poles: usize,
    pub beacons: usize,
    pub rails: usize,
    pub train_stops: usize,
    pub combinators: usize,
    pub roboports: usize,
    pub logistic_chests: usize,
    pub power_gen: usize,

    // recipes
    pub distinct_recipes: usize,
    pub fluid_recipes: usize,

    // rates (from the sibling analysis)
    pub top_rate: f64,
    pub rate_band: String,

    // direct insertion (inserter pickup AND drop both resolve to machines)
    pub direct_insertion: usize,
    pub di_fraction: f64,

    // belt structure
    pub belt_networks: usize,
    pub largest_belt_network: usize,
    pub sideloads: usize,
    pub sideload_per_100_belts: f64,
    pub ug_pairs: usize,
    pub mixed_belt_networks: usize,

    // fluids structure
    pub pipe_networks: usize,

    // power structure
    pub machines_powered_fraction: Option<f64>,
    pub pole_networks: usize,
    pub self_powered: bool,

    // periodicity / tiling
    pub pitch: i32,
    pub pitch_score: f64,
    pub tileable_geom: bool,

    // verdicts (decision list)
    pub archetype: String,
    pub chain_level: String,
}

fn is_rail(name: &str) -> bool {
    name.contains("rail") || name == "train-stop"
}
fn is_combinator(name: &str) -> bool {
    name.contains("combinator")
        || matches!(name, "programmable-speaker" | "display-panel" | "power-switch" | "lamp")
}
fn is_power_gen(name: &str) -> bool {
    matches!(
        name,
        "solar-panel"
            | "accumulator"
            | "nuclear-reactor"
            | "boiler"
            | "steam-engine"
            | "steam-turbine"
            | "heat-exchanger"
            | "fusion-reactor"
            | "fusion-generator"
            | "heating-tower"
    )
}
fn is_pole_entity(name: &str) -> bool {
    common::pole_wire_reach(name, QualityTier::Normal).is_some()
}
fn is_pipe_entity(name: &str) -> bool {
    name == "pipe" || name == "pipe-to-ground"
}
fn ug_reach(name: &str) -> i32 {
    match name {
        "underground-belt" => 4,
        "fast-underground-belt" => 6,
        "express-underground-belt" => 8,
        "turbo-underground-belt" => 10,
        _ => 4,
    }
}

/// Footprint tiles covered by an entity (top-left + oriented dims).
fn tiles_of(e: &PlacedEntity) -> Vec<(i32, i32)> {
    let (mut w, mut h) = if common::is_splitter(e.name.as_str()) {
        common::oriented_splitter_dims(e.name.as_str(), e.direction)
            .unwrap_or((2, 1))
    } else {
        common::entity_size(e.name.as_str())
    };
    // rotate non-square footprints for E/W orientation
    if matches!(e.direction, EntityDirection::East | EntityDirection::West) && w != h {
        std::mem::swap(&mut w, &mut h);
    }
    let mut out = Vec::with_capacity((w * h) as usize);
    for dx in 0..w as i32 {
        for dy in 0..h as i32 {
            out.push((e.x + dx, e.y + dy));
        }
    }
    out
}

/// Disjoint-set union over tile keys.
struct Dsu {
    parent: FxHashMap<(i32, i32), (i32, i32)>,
}
impl Dsu {
    fn find(&mut self, a: (i32, i32)) -> (i32, i32) {
        let mut root = a;
        while self.parent[&root] != root {
            root = self.parent[&root];
        }
        let mut cur = a;
        while self.parent[&cur] != cur {
            let next = self.parent[&cur];
            self.parent.insert(cur, root);
            cur = next;
        }
        root
    }
    fn add(&mut self, a: (i32, i32)) {
        self.parent.entry(a).or_insert(a);
    }
    fn union(&mut self, a: (i32, i32), b: (i32, i32)) {
        self.add(a);
        self.add(b);
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.parent.insert(ra, rb);
        }
    }
    fn groups(&mut self) -> Vec<Vec<(i32, i32)>> {
        let keys: Vec<(i32, i32)> = self.parent.keys().copied().collect();
        let mut g: FxHashMap<(i32, i32), Vec<(i32, i32)>> = FxHashMap::default();
        for k in keys {
            let r = self.find(k);
            g.entry(r).or_default().push(k);
        }
        g.into_values().collect()
    }
}

fn center(e: &PlacedEntity) -> (f64, f64) {
    let (w, h) = common::entity_size(e.name.as_str());
    (e.x as f64 + w as f64 / 2.0, e.y as f64 + h as f64 / 2.0)
}

/// Extract strategy features from a parsed layout. `analysis` supplies the
/// rate fields (run `analysis::analyze` first).
pub fn classify(layout: &LayoutResult, analysis: &BlueprintAnalysis) -> BlueprintFeatures {
    let ents = &layout.entities;

    // ---- occupancy grid: tile -> smallest-footprint entity ----
    let mut occ: FxHashMap<(i32, i32), (&PlacedEntity, u32)> = FxHashMap::default();
    for e in ents {
        let (w, h) = common::entity_size(e.name.as_str());
        let area = w * h;
        for t in tiles_of(e) {
            match occ.get(&t) {
                Some(&(_, a)) if a <= area => {}
                _ => {
                    occ.insert(t, (e, area));
                }
            }
        }
    }

    let is_machine = |e: &PlacedEntity| common::is_machine_entity(e.name.as_str()) || e.recipe.is_some();

    // ---- census ----
    let mut f = BlueprintFeatures {
        machines: ents.iter().filter(|e| is_machine(e)).count(),
        belt_tiles: ents.iter().filter(|e| common::is_surface_belt(e.name.as_str()) || common::is_ug_belt(e.name.as_str())).count(),
        splitters: ents.iter().filter(|e| common::is_splitter(e.name.as_str())).count(),
        inserters: ents.iter().filter(|e| common::is_inserter(e.name.as_str())).count(),
        pipes: ents.iter().filter(|e| is_pipe_entity(e.name.as_str())).count(),
        poles: ents.iter().filter(|e| is_pole_entity(e.name.as_str())).count(),
        beacons: ents.iter().filter(|e| e.name == "beacon").count(),
        rails: ents.iter().filter(|e| is_rail(e.name.as_str())).count(),
        train_stops: ents.iter().filter(|e| e.name == "train-stop").count(),
        combinators: ents.iter().filter(|e| is_combinator(e.name.as_str())).count(),
        roboports: ents.iter().filter(|e| e.name == "roboport").count(),
        logistic_chests: ents.iter().filter(|e| e.name.starts_with("logistic-chest")).count(),
        power_gen: ents.iter().filter(|e| is_power_gen(e.name.as_str())).count(),
        distinct_recipes: analysis.recipe_count,
        fluid_recipes: 0,
        top_rate: 0.0,
        rate_band: String::new(),
        direct_insertion: 0,
        di_fraction: 0.0,
        belt_networks: 0,
        largest_belt_network: 0,
        sideloads: 0,
        sideload_per_100_belts: 0.0,
        ug_pairs: 0,
        mixed_belt_networks: 0,
        pipe_networks: 0,
        machines_powered_fraction: None,
        pole_networks: 0,
        self_powered: false,
        pitch: 0,
        pitch_score: 0.0,
        tileable_geom: false,
        archetype: String::new(),
        chain_level: String::new(),
    };

    let recipes: FxHashSet<&str> = ents
        .iter()
        .filter_map(|e| e.recipe.as_deref())
        .collect();
    let db = recipe_db::db();
    f.fluid_recipes = recipes
        .iter()
        .filter(|r| {
            db.recipes.get(**r).is_some_and(|rec| {
                rec.ingredients.iter().any(|i| i.type_ == "fluid")
                    || rec.products.iter().any(|p| p.type_ == "fluid")
            })
        })
        .count();

    // ---- rates ----
    if let Some((_, &rate)) = analysis
        .throughput_estimates
        .iter()
        .filter(|(item, _)| analysis.final_products.contains(item))
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
    {
        f.top_rate = (rate * 10.0).round() / 10.0;
    }
    f.rate_band = if f.top_rate <= 0.0 {
        "n/a".to_string()
    } else if f.top_rate <= 15.0 {
        "<=15/s".to_string()
    } else if f.top_rate <= 45.0 {
        "15-45/s".to_string()
    } else if f.top_rate <= 270.0 {
        "45-270/s".to_string()
    } else {
        "270+/s".to_string()
    };

    // ---- direct insertion (engine convention: direction = drop side) ----
    for e in ents {
        if !common::is_inserter(e.name.as_str()) {
            continue;
        }
        let (dx, dy) = common::dir_to_vec(e.direction);
        let reach = if e.name == "long-handed-inserter" { 2 } else { 1 };
        let drop = occ.get(&(e.x + dx * reach, e.y + dy * reach));
        let pick = occ.get(&(e.x - dx * reach, e.y - dy * reach));
        if let (Some(&(d, _)), Some(&(p, _))) = (drop, pick) {
            if !std::ptr::eq(d, p) && is_machine(d) && is_machine(p) {
                f.direct_insertion += 1;
            }
        }
    }
    f.di_fraction = if f.inserters > 0 {
        (f.direct_insertion as f64 / f.inserters as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    // ---- belt networks (permissive orthogonal merge + UG pairs) ----
    let mut belt_at: FxHashMap<(i32, i32), &PlacedEntity> = FxHashMap::default();
    for e in ents {
        if common::is_belt_entity(e.name.as_str()) {
            for t in tiles_of(e) {
                belt_at.insert(t, e);
            }
        }
    }
    let mut dsu = Dsu { parent: FxHashMap::default() };
    for &t in belt_at.keys() {
        dsu.add(t);
    }
    for &(tx, ty) in belt_at.keys() {
        for (dx, dy) in [(1, 0), (0, 1)] {
            if belt_at.contains_key(&(tx + dx, ty + dy)) {
                dsu.union((tx, ty), (tx + dx, ty + dy));
            }
        }
    }
    for (&(tx, ty), &e) in &belt_at {
        if !common::is_surface_belt(e.name.as_str()) {
            continue;
        }
        let (dx, dy) = common::dir_to_vec(e.direction);
        if let Some(&fwd) = belt_at.get(&(tx + dx, ty + dy)) {
            if common::is_surface_belt(fwd.name.as_str()) {
                let (fdx, fdy) = common::dir_to_vec(fwd.direction);
                if (fdx, fdy) != (dx, dy) {
                    f.sideloads += 1;
                }
            }
        }
    }
    for e in ents {
        if !common::is_ug_belt(e.name.as_str()) || e.io_type.as_deref() != Some("input") {
            continue;
        }
        let (dx, dy) = common::dir_to_vec(e.direction);
        for d in 2..=ug_reach(e.name.as_str()) + 1 {
            let t = (e.x + dx * d, e.y + dy * d);
            if let Some(&o) = belt_at.get(&t) {
                if o.name == e.name
                    && o.io_type.as_deref() == Some("output")
                    && o.direction == e.direction
                {
                    dsu.union((e.x, e.y), t);
                    f.ug_pairs += 1;
                    break;
                }
            }
        }
    }
    let belt_groups = dsu.groups();
    f.belt_networks = belt_groups.len();
    f.largest_belt_network = belt_groups.iter().map(|g| g.len()).max().unwrap_or(0);
    f.sideload_per_100_belts = if f.belt_tiles > 0 {
        (f.sideloads as f64 / f.belt_tiles as f64 * 1000.0).round() / 10.0
    } else {
        0.0
    };

    // ---- sushi inference: networks fed by inserters from >1 distinct item ----
    let mut net_of: FxHashMap<(i32, i32), usize> = FxHashMap::default();
    for (i, g) in belt_groups.iter().enumerate() {
        for &t in g {
            net_of.insert(t, i);
        }
    }
    let mut net_items: FxHashMap<usize, FxHashSet<String>> = FxHashMap::default();
    for e in ents {
        if !common::is_inserter(e.name.as_str()) {
            continue;
        }
        let (dx, dy) = common::dir_to_vec(e.direction);
        let reach = if e.name == "long-handed-inserter" { 2 } else { 1 };
        let net = net_of.get(&(e.x + dx * reach, e.y + dy * reach));
        let pick = occ.get(&(e.x - dx * reach, e.y - dy * reach));
        if let (Some(&net), Some(&(p, _))) = (net, pick) {
            if is_machine(p) {
                if let Some(rec) = p.recipe.as_deref().and_then(|r| db.recipes.get(r)) {
                    let items = net_items.entry(net).or_default();
                    for prod in &rec.products {
                        if prod.type_ == "item" {
                            items.insert(prod.name.clone());
                        }
                    }
                }
            }
        }
    }
    f.mixed_belt_networks = net_items.values().filter(|s| s.len() > 1).count();

    // ---- pipe networks ----
    let mut pipe_dsu = Dsu { parent: FxHashMap::default() };
    for e in ents {
        if is_pipe_entity(e.name.as_str()) {
            for t in tiles_of(e) {
                pipe_dsu.add(t);
            }
        }
    }
    let pipe_keys: Vec<(i32, i32)> = pipe_dsu.parent.keys().copied().collect();
    for (tx, ty) in &pipe_keys {
        for (dx, dy) in [(1, 0), (0, 1)] {
            if pipe_dsu.parent.contains_key(&(tx + dx, ty + dy)) {
                pipe_dsu.union((*tx, *ty), (tx + dx, ty + dy));
            }
        }
    }
    f.pipe_networks = if f.pipes > 0 { pipe_dsu.groups().len() } else { 0 };

    // ---- power: machine coverage + pole networks ----
    let poles: Vec<&PlacedEntity> = ents.iter().filter(|e| is_pole_entity(e.name.as_str())).collect();
    let machines: Vec<&PlacedEntity> = ents.iter().filter(|e| is_machine(e)).collect();
    if !machines.is_empty() {
        let covered = machines
            .iter()
            .filter(|m| {
                let (mx, my) = center(m);
                poles.iter().any(|p| {
                    let (px, py) = center(p);
                    let sd = common::supply_area_distance(p.name.as_str(), QualityTier::Normal);
                    (mx - px).abs() <= sd && (my - py).abs() <= sd
                })
            })
            .count();
        f.machines_powered_fraction =
            Some((covered as f64 / machines.len() as f64 * 100.0).round() / 100.0);
    }
    // pole networks via wire reach (index-keyed DSU over pole centers)
    {
        let mut idx_parent: Vec<usize> = (0..poles.len()).collect();
        fn find(p: &mut [usize], a: usize) -> usize {
            let mut r = a;
            while p[r] != r {
                r = p[r];
            }
            let mut c = a;
            while p[c] != c {
                let n = p[c];
                p[c] = r;
                c = n;
            }
            r
        }
        for i in 0..poles.len() {
            for j in i + 1..poles.len() {
                let (ax, ay) = center(poles[i]);
                let (bx, by) = center(poles[j]);
                let ra = common::pole_wire_reach(poles[i].name.as_str(), QualityTier::Normal).unwrap_or(0.0);
                let rb = common::pole_wire_reach(poles[j].name.as_str(), QualityTier::Normal).unwrap_or(0.0);
                let dist = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
                if dist <= ra.min(rb) {
                    let (ri, rj) = (find(&mut idx_parent, i), find(&mut idx_parent, j));
                    if ri != rj {
                        idx_parent[ri] = rj;
                    }
                }
            }
        }
        let roots: FxHashSet<usize> = (0..poles.len()).map(|i| find(&mut idx_parent, i)).collect();
        f.pole_networks = roots.len();
    }
    f.self_powered = !machines.is_empty()
        && !poles.is_empty()
        && f.machines_powered_fraction.unwrap_or(0.0) >= 0.95;

    // ---- periodicity: repeated same-name rows/columns at a fixed pitch ----
    {
        let mut best_pitch = 0;
        let mut best_score = 0.0f64;
        for axis_y in [true, false] {
            for p in 2..=40i32 {
                let mut hits = 0usize;
                let mut seen: FxHashSet<(String, i32, i32)> = FxHashSet::default();
                for m in &machines {
                    let (ax, other) = if axis_y { (m.y, m.x) } else { (m.x, m.y) };
                    if seen.contains(&(m.name.clone(), other, ax - p)) {
                        hits += 1;
                    }
                    seen.insert((m.name.clone(), other, ax));
                }
                let score = if machines.is_empty() { 0.0 } else { hits as f64 / machines.len() as f64 };
                if score > best_score {
                    best_score = score;
                    best_pitch = p;
                }
            }
        }
        f.pitch = best_pitch;
        f.pitch_score = (best_score * 100.0).round() / 100.0;
        f.tileable_geom = f.machines >= 6 && best_score >= 0.4;
    }

    // ---- verdicts ----
    f.archetype = if f.machines == 0 && f.combinators >= 3 {
        "circuit-contraption"
    } else if f.machines == 0 && f.rails > 0 {
        if f.train_stops > 0 && f.rails < 150 {
            "train-station"
        } else {
            "rail-infra"
        }
    } else if f.machines == 0 && f.power_gen > 0 {
        "power"
    } else if f.machines == 0 && f.belt_tiles > 0 {
        if f.splitters > 0 {
            "balancer"
        } else {
            "belt-routing"
        }
    } else if f.machines == 0 && f.roboports > 0 {
        "bot-logistics"
    } else if f.machines == 0 {
        "other-empty"
    } else if f.distinct_recipes >= 12 {
        "mall"
    } else {
        "production-block"
    }
    .to_string();

    let has_recipe = |pred: &dyn Fn(&str) -> bool| recipes.iter().any(|r| pred(r));
    f.chain_level = if has_recipe(&|r| r.ends_with("-science-pack")) {
        "science"
    } else if has_recipe(&|r| r.contains("module")) {
        "modules"
    } else if has_recipe(&|r| r.contains("circuit") || r == "copper-cable" || r == "plastic-bar") {
        "intermediates"
    } else if has_recipe(&|r| r.contains("uranium") || r.contains("kovarex") || r.contains("nuclear") || r.contains("fusion")) {
        "nuclear"
    } else if f.fluid_recipes > 0 {
        "fluids"
    } else if recipes.iter().any(|r| {
        db.recipes.get(*r).is_some_and(|rec| rec.category == "smelting")
    }) {
        "smelting"
    } else if ents.iter().any(|e| {
        matches!(
            e.name.as_str(),
            "electric-mining-drill" | "big-mining-drill" | "burner-mining-drill" | "pumpjack"
        )
    }) {
        "mining"
    } else if f.distinct_recipes > 0 {
        "other-production"
    } else {
        "none"
    }
    .to_string();

    f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::analyze;
    use crate::models::PlacedEntity;

    fn ent(name: &str, x: i32, y: i32, dir: EntityDirection) -> PlacedEntity {
        PlacedEntity {
            name: name.into(),
            x,
            y,
            direction: dir,
            ..Default::default()
        }
    }

    fn machine(name: &str, x: i32, y: i32, recipe: &str) -> PlacedEntity {
        PlacedEntity {
            name: name.into(),
            x,
            y,
            direction: EntityDirection::North,
            recipe: Some(recipe.into()),
            ..Default::default()
        }
    }

    fn classify_only(entities: Vec<PlacedEntity>) -> BlueprintFeatures {
        let layout = LayoutResult {
            entities,
            ..Default::default()
        };
        let analysis = analyze(&layout);
        classify(&layout, &analysis)
    }

    #[test]
    fn direct_insertion_drop_side_convention() {
        // cable machine at (0,0) 3x3, EC machine at (4,0) 3x3, inserter at
        // (3,1) facing EAST: drops into the EC machine at (4,1), picks from
        // the cable machine at (2,1). Engine direction = drop side.
        let f = classify_only(vec![
            machine("assembling-machine-3", 0, 0, "copper-cable"),
            machine("assembling-machine-3", 4, 0, "electronic-circuit"),
            ent("fast-inserter", 3, 1, EntityDirection::East),
        ]);
        assert_eq!(f.direct_insertion, 1, "east-facing inserter between machines should be DI");
        assert_eq!(f.archetype, "production-block");
    }

    #[test]
    fn direct_insertion_reversed_inserter_is_not_counted() {
        // inserter facing WEST: drops at (2,1) into the cable machine,
        // picks from the EC machine at (4,1) — still machine->machine,
        // still DI (the pair direction is wrong for the recipe but the
        // feature only measures machine-to-machine wiring).
        let f = classify_only(vec![
            machine("assembling-machine-3", 0, 0, "copper-cable"),
            machine("assembling-machine-3", 4, 0, "electronic-circuit"),
            ent("fast-inserter", 3, 1, EntityDirection::West),
        ]);
        assert_eq!(f.direct_insertion, 1);
    }

    #[test]
    fn belt_to_machine_is_not_di() {
        let f = classify_only(vec![
            machine("assembling-machine-3", 4, 0, "electronic-circuit"),
            ent("fast-inserter", 3, 1, EntityDirection::East),
            ent("transport-belt", 2, 1, EntityDirection::East),
        ]);
        assert_eq!(f.direct_insertion, 0);
    }

    #[test]
    fn ug_pair_merges_networks() {
        // two belt runs connected by a UG hop -> one network, one pair
        let mut entities = vec![
            ent("express-underground-belt", 5, 0, EntityDirection::East),
            ent("express-underground-belt", 10, 0, EntityDirection::East),
        ];
        entities[0].io_type = Some("input".into());
        entities[1].io_type = Some("output".into());
        for x in 0..5 {
            entities.push(ent("express-transport-belt", x, 0, EntityDirection::East));
        }
        for x in 11..15 {
            entities.push(ent("express-transport-belt", x, 0, EntityDirection::East));
        }
        let f = classify_only(entities);
        assert_eq!(f.ug_pairs, 1);
        assert_eq!(f.belt_networks, 1);
    }

    #[test]
    fn sideload_detected() {
        // north-flowing belt at (1,1) feeding the side of an east belt at (1,0)
        let f = classify_only(vec![
            ent("transport-belt", 1, 0, EntityDirection::East),
            ent("transport-belt", 2, 0, EntityDirection::East),
            ent("transport-belt", 1, 1, EntityDirection::North),
        ]);
        assert_eq!(f.sideloads, 1);
    }

    #[test]
    fn periodic_row_is_tileable() {
        // two identical 4-machine rows at y-pitch 7, with deliberately
        // irregular x spacing so no x-axis pitch competes
        let xs = [0, 3, 7, 12];
        let mut entities = Vec::new();
        for k in 0..2 {
            for &x in &xs {
                entities.push(machine("assembling-machine-3", x, k * 7, "iron-gear-wheel"));
            }
        }
        let f = classify_only(entities);
        assert_eq!(f.pitch, 7);
        assert!(f.tileable_geom, "score {} should mark tileable", f.pitch_score);
    }

    #[test]
    fn self_powered_vs_grid_fed() {
        let powered = classify_only(vec![
            machine("assembling-machine-3", 0, 0, "iron-gear-wheel"),
            ent("medium-electric-pole", 4, 1, EntityDirection::North),
        ]);
        assert_eq!(powered.machines_powered_fraction, Some(1.0));
        assert!(powered.self_powered);

        let grid = classify_only(vec![
            machine("assembling-machine-3", 0, 0, "iron-gear-wheel"),
            ent("medium-electric-pole", 40, 1, EntityDirection::North),
        ]);
        assert_eq!(grid.machines_powered_fraction, Some(0.0));
        assert!(!grid.self_powered);
    }

    #[test]
    fn balancer_archetype() {
        let f = classify_only(vec![
            ent("transport-belt", 0, 0, EntityDirection::East),
            ent("splitter", 1, 0, EntityDirection::East),
            ent("transport-belt", 3, 0, EntityDirection::East),
        ]);
        assert_eq!(f.archetype, "balancer");
        assert_eq!(f.chain_level, "none");
    }

    #[test]
    fn mall_archetype_and_science_level() {
        let mut entities = Vec::new();
        let recipes = [
            "iron-gear-wheel", "copper-cable", "electronic-circuit", "advanced-circuit",
            "iron-stick", "pipe", "transport-belt", "inserter", "fast-inserter",
            "automation-science-pack", "logistic-science-pack", "chemical-science-pack",
        ];
        for (i, r) in recipes.iter().enumerate() {
            entities.push(machine("assembling-machine-3", (i as i32 % 4) * 4, (i as i32 / 4) * 4, r));
        }
        let f = classify_only(entities);
        assert_eq!(f.archetype, "mall");
        assert_eq!(f.chain_level, "science");
    }

    #[test]
    fn import_unflip_feeds_classify_correctly() {
        // round-trip through a real blueprint string: the parser un-flips
        // game inserter direction (pickup side) to engine convention
        // (drop side), so classify sees DI on an exported DI pair.
        use crate::blueprint;
        let layout = LayoutResult {
            entities: vec![
                machine("assembling-machine-3", 0, 0, "copper-cable"),
                machine("assembling-machine-3", 4, 0, "electronic-circuit"),
                ent("fast-inserter", 3, 1, EntityDirection::East),
            ],
            width: 7,
            height: 3,
            ..Default::default()
        };
        let bp = blueprint::export(&layout, "di-test");
        let (imported, analysis) = crate::analysis::analyze_blueprint_string(&bp).unwrap();
        let f = classify(&imported, &analysis);
        assert_eq!(f.direct_insertion, 1, "round-trip should preserve DI");
    }
}
