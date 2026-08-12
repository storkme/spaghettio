//! Community-donor translator for the celldb store (RFC-067 donor probe,
//! decision log 2026-08-12).
//!
//! Translates hand-shortlisted community blueprint cells into celldb
//! entries. The split of labor is deliberate and mirrors the reopening
//! path's wording ("community/hand donors with inferred ports"): geometry
//! comes mechanically from the parsed blueprint, `carries` assignment is
//! derived from the engine's own inserter semantics and refused on any
//! ambiguity, and the PORTS are hand-declared in the spec table below —
//! port inference from wild blueprints stays out of scope (the recorded
//! v1 gap). Everything hand-declared is machine-verified: ports by
//! `celldb::check_entry`, provenance by SHA256 against the Phase-0 corpus
//! manifest, and metrics are derived here, never typed.
//!
//! Usage:
//!   CORPUS=<dir> cargo run -p spaghettio_core --example celldb_donor
//!
//! Reads `$CORPUS/<spec.source_file>`, verifies its SHA256 against
//! `scripts/celldb-phase0/corpus-manifest.tsv`, and rewrites
//! `crates/core/data/celldb.json` with this tool's donor entries replaced
//! in place (engine@ entries untouched). Refuses (exit 2) on any
//! extraction ambiguity or invariant violation.

use spaghettio_core::analysis;
use spaghettio_core::celldb::{self, CellDb, CellEntry, Metrics, Motif, Port, PortKind};
use spaghettio_core::common::{dir_to_vec, inserter_reach, is_inserter, oriented_entity_dims};
use spaghettio_core::connectivity::build_ug_pairs;
use spaghettio_core::models::PlacedEntity;
use std::collections::{BTreeMap, BTreeSet};

struct DonorSpec {
    /// Corpus filename (factorioprints export) — provenance + SHA source.
    source_file: &'static str,
    /// Index into `analyze_blueprint_string_any`'s record list.
    record_index: usize,
    /// Asserted against the record's label — drift guard on the index.
    record_label: &'static str,
    /// Short provenance id written into the entry.
    provenance: &'static str,
    recipe: &'static str,
    machine: &'static str,
    in_item: &'static str,
    out_item: &'static str,
    /// Hand-declared ports, in POST-normalization coordinates (after pole
    /// strip). check_entry verifies boundary-ness, occupancy, and carries.
    ports: &'static [(i32, i32, PortKind, &'static str)],
}

/// Entity names allowed to remain in a donor fragment. Anything else after
/// the pole strip is a refusal, not a silent drop.
const KEEP: &[&str] = &[
    "electric-furnace",
    "transport-belt",
    "fast-transport-belt",
    "express-transport-belt",
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt",
    "splitter",
    "fast-splitter",
    "express-splitter",
    "inserter",
    "fast-inserter",
    "long-handed-inserter",
];
const STRIP: &[&str] = &["small-electric-pole", "medium-electric-pole", "big-electric-pole", "substation"];

fn specs() -> Vec<DonorSpec> {
    vec![
        // Double-row fast-belt smelter: ore enters a west-edge splitter
        // feeding top/bottom ore belts via two distribution columns; the
        // shared middle belt drains east. 48 furnaces.
        DonorSpec {
            source_file: "-OL38TX27JmPIivo_F3R_factorio_handbook.json",
            record_index: 8,
            record_label: "[item=electric-furnace] 30/s",
            provenance: "community:factorioprints/-OL38TX27JmPIivo#8",
            recipe: "copper-plate",
            machine: "electric-furnace",
            in_item: "copper-ore",
            out_item: "copper-plate",
            ports: &[
                (0, 6, PortKind::BeltIn, "copper-ore"),
                (73, 6, PortKind::BeltOut, "copper-plate"),
            ],
        },
        // "Mass Plate Production": tileable double-row yellow... fast-belt
        // cell, long-handed feeders from the lower ore belt, plates drain
        // onto the upper belt; both belts exit east (ore pass-through is
        // the design's chaining feature). 50 furnaces. Ports are in
        // POST-pole-strip coordinates (strip shifts x by -1).
        DonorSpec {
            source_file: "-Lxr8KxgJup5AsKyTygI_factory_blueprints.json",
            record_index: 6,
            record_label: "Mass Plate Production",
            provenance: "community:factorioprints/-Lxr8KxgJup5AsKyTygI#6",
            recipe: "copper-plate",
            machine: "electric-furnace",
            in_item: "copper-ore",
            out_item: "copper-plate",
            ports: &[
                (0, 5, PortKind::BeltIn, "copper-ore"),
                (86, 4, PortKind::BeltOut, "copper-plate"),
            ],
        },
        // Vertical express column cell: ore enters a south-edge splitter,
        // climbs both side columns (inline splitters as lane balancers);
        // plates merge onto the center column and exit north. 52 furnaces.
        DonorSpec {
            source_file: "-ON0-7RmLQJjDfgq7s_W_all_the_book.json",
            record_index: 2,
            record_label: "[item=iron-plate][item=copper-plate]45/s 2700/m",
            provenance: "community:factorioprints/-ON0-7RmLQJjDfgq7s_W#2",
            recipe: "copper-plate",
            machine: "electric-furnace",
            in_item: "copper-ore",
            out_item: "copper-plate",
            ports: &[
                (8, 56, PortKind::BeltIn, "copper-ore"),
                (9, 0, PortKind::BeltOut, "copper-plate"),
            ],
        },
    ]
}

fn fail(msg: &str) -> ! {
    eprintln!("REFUSE: {msg}");
    std::process::exit(2);
}

fn sha256_hex(bytes: &[u8]) -> String {
    // No sha2 dependency in core; shell out (tool runs dev-side only).
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    let mut c = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sha256sum");
    c.stdin.as_mut().unwrap().write_all(bytes).unwrap();
    let out = c.wait_with_output().unwrap();
    String::from_utf8(out.stdout).unwrap().split_whitespace().next().unwrap().to_string()
}

fn manifest_sha(file: &str) -> String {
    let manifest = std::fs::read_to_string("scripts/celldb-phase0/corpus-manifest.tsv")
        .expect("corpus-manifest.tsv (run from repo root)");
    for line in manifest.lines() {
        let mut it = line.split('\t');
        if it.next() == Some(file) {
            return it.next().expect("manifest sha").to_string();
        }
    }
    fail(&format!("{file} not in corpus-manifest.tsv"));
}

fn is_transport(name: &str) -> bool {
    name.ends_with("transport-belt") || name.ends_with("underground-belt") || name.ends_with("splitter")
}

fn occupied_tiles(e: &PlacedEntity) -> Vec<(i32, i32)> {
    let (w, h) = oriented_entity_dims(&e.name, e.direction);
    let mut v = Vec::new();
    for dx in 0..w {
        for dy in 0..h {
            v.push((e.x + dx, e.y + dy));
        }
    }
    v
}

/// Assign `carries` to every transport entity and inserter, mechanically:
/// seed from inserter pick/drop tiles (engine semantics: direction is the
/// DROP side, reach from `inserter_reach`), propagate across the directed
/// belt-flow graph's weak components, refuse on conflict or unreached
/// transport.
fn assign_carries(entities: &mut [PlacedEntity], spec: &DonorSpec) {
    let furnace_tiles: BTreeSet<(i32, i32)> = entities
        .iter()
        .filter(|e| e.name == spec.machine)
        .flat_map(|e| occupied_tiles(e))
        .collect();
    // tile -> transport entity index
    let mut tile_of: BTreeMap<(i32, i32), usize> = BTreeMap::new();
    for (i, e) in entities.iter().enumerate() {
        if is_transport(&e.name) {
            for t in occupied_tiles(e) {
                tile_of.insert(t, i);
            }
        }
    }
    // Union-find over transport indices, joined by directed flow adjacency
    // (a belt's output tile landing on another transport entity — covers
    // in-line flow and sideloads; parallel belts never join) plus UG pairs.
    let n = entities.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut Vec<usize>, i: usize) -> usize {
        if parent[i] != i {
            let r = find(parent, parent[i]);
            parent[i] = r;
        }
        parent[i]
    }
    let join = |parent: &mut Vec<usize>, a: usize, b: usize| {
        let (ra, rb) = (find(parent, a), find(parent, b));
        if ra != rb {
            parent[ra] = rb;
        }
    };
    let ug_pairs = build_ug_pairs(entities);
    for (i, e) in entities.iter().enumerate() {
        if !is_transport(&e.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(e.direction);
        for t in occupied_tiles(e) {
            let out = (t.0 + dx, t.1 + dy);
            if let Some(&j) = tile_of.get(&out) {
                if j != i {
                    join(&mut parent, i, j);
                }
            }
        }
    }
    for (a, b) in &ug_pairs {
        if let (Some(&i), Some(&j)) = (tile_of.get(a), tile_of.get(b)) {
            if i != j {
                join(&mut parent, i, j);
            }
        }
    }
    // Seeds from inserters; also set inserter carries.
    let mut seed: BTreeMap<usize, &'static str> = BTreeMap::new();
    let mut ins_item: Vec<Option<&'static str>> = vec![None; n];
    for (i, e) in entities.iter().enumerate() {
        if !is_inserter(&e.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(e.direction);
        let r = inserter_reach(&e.name);
        let drop = (e.x + dx * r, e.y + dy * r);
        let pick = (e.x - dx * r, e.y - dy * r);
        let feeds_machine = furnace_tiles.contains(&drop);
        let drains_machine = furnace_tiles.contains(&pick);
        match (feeds_machine, drains_machine) {
            (true, false) => {
                ins_item[i] = Some(spec.in_item);
                if let Some(&t) = tile_of.get(&pick) {
                    seed.insert(find(&mut parent, t), spec.in_item);
                } else {
                    fail(&format!("feeder inserter at ({},{}) picks from a non-transport tile", e.x, e.y));
                }
            }
            (false, true) => {
                ins_item[i] = Some(spec.out_item);
                if let Some(&t) = tile_of.get(&drop) {
                    let root = find(&mut parent, t);
                    if let Some(prev) = seed.insert(root, spec.out_item) {
                        if prev != spec.out_item {
                            fail(&format!("belt run reached by both items near ({},{})", e.x, e.y));
                        }
                    }
                } else {
                    fail(&format!("drain inserter at ({},{}) drops onto a non-transport tile", e.x, e.y));
                }
            }
            (true, true) => fail(&format!("inserter at ({},{}) touches machines on both sides", e.x, e.y)),
            (false, false) => fail(&format!("inserter at ({},{}) touches no machine", e.x, e.y)),
        }
    }
    // Conflict check: a root seeded twice with different items already
    // refused above for drains; feeders can conflict with drains too.
    // (seed.insert refuses via the drain arm; feeder arm inserts blindly —
    // re-verify all seeds agree per root.)
    let mut root_item: BTreeMap<usize, &'static str> = BTreeMap::new();
    for (i, e) in entities.iter().enumerate() {
        if !is_inserter(&e.name) {
            continue;
        }
        let (dx, dy) = dir_to_vec(e.direction);
        let r = inserter_reach(&e.name);
        let (t, item) = if ins_item[i] == Some(spec.in_item) {
            ((e.x - dx * r, e.y - dy * r), spec.in_item)
        } else {
            ((e.x + dx * r, e.y + dy * r), spec.out_item)
        };
        let root = find(&mut parent, *tile_of.get(&t).unwrap());
        if let Some(prev) = root_item.insert(root, item) {
            if prev != item {
                fail(&format!("belt run near ({},{}) is claimed by both {} and {}", t.0, t.1, spec.in_item, spec.out_item));
            }
        }
    }
    // Apply.
    let roots: Vec<Option<usize>> = (0..n)
        .map(|i| if is_transport(&entities[i].name) { Some(find(&mut parent, i)) } else { None })
        .collect();
    for (i, e) in entities.iter_mut().enumerate() {
        if let Some(root) = roots[i] {
            match root_item.get(&root) {
                Some(item) => e.carries = Some(item.to_string()),
                None => fail(&format!("transport {} at ({},{}) belongs to a run no inserter touches", e.name, e.x, e.y)),
            }
        } else if is_inserter(&e.name) {
            e.carries = ins_item[i].map(|s| s.to_string());
        }
    }
}

fn translate(spec: &DonorSpec, corpus: &str) -> CellEntry {
    let path = format!("{corpus}/{}", spec.source_file);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| fail(&format!("{path}: {e}")));
    let want = manifest_sha(spec.source_file);
    let got = sha256_hex(&bytes);
    if got != want {
        fail(&format!("{}: SHA {got} != manifest {want} — not the Phase-0 corpus bytes", spec.source_file));
    }
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let bp = v["blueprintString"].as_str().unwrap_or_else(|| fail("no blueprintString"));
    let records = analysis::analyze_blueprint_string_any(bp).unwrap_or_else(|e| fail(&e));
    let rec = records.get(spec.record_index).unwrap_or_else(|| fail("record index out of range"));
    let label = rec.label.as_deref().unwrap_or("");
    if label != spec.record_label {
        fail(&format!("record {} label {:?} != expected {:?}", spec.record_index, label, spec.record_label));
    }
    let mut entities: Vec<PlacedEntity> = Vec::new();
    for e in &rec.layout.entities {
        if STRIP.contains(&e.name.as_str()) {
            continue;
        }
        if !KEEP.contains(&e.name.as_str()) {
            fail(&format!("unexpected entity {} at ({},{}) — extend KEEP/STRIP deliberately", e.name, e.x, e.y));
        }
        entities.push(e.clone());
    }
    // Donors donate GEOMETRY only. Module payloads are stripped: the
    // incumbent competes module-less (module planning is the solver's
    // RFC-044 axis, not the fragment's), and a speed-moduled donor would
    // claim capability the plan never funded — the ON0 donor shipped with
    // speed modules that made its first sim read 45/s from a 32.5/s plan.
    let mut stripped_modules = 0usize;
    for e in entities.iter_mut() {
        if !e.items.is_empty() {
            stripped_modules += e.items.len();
            e.items.clear();
        }
    }
    if stripped_modules > 0 {
        println!(
            "{}: stripped {stripped_modules} module payload(s) (geometry-only donation)",
            spec.provenance
        );
    }
    // The motif recipe is a declaration, not a read: furnace arrays are
    // item-agnostic and community furnaces carry no recipe field. Refuse
    // if the source DID declare one — overriding a real declaration would
    // be misattribution, not translation.
    let mut count = 0u32;
    for e in entities.iter_mut() {
        if e.name == spec.machine {
            if let Some(r) = e.recipe.as_deref() {
                fail(&format!("furnace at ({},{}) declares recipe {r}; expected none", e.x, e.y));
            }
            e.recipe = Some(spec.recipe.to_string());
            count += 1;
        }
        e.rate = None;
        e.segment_id = None;
    }
    assign_carries(&mut entities, spec);
    // Normalize to origin.
    let min_x = entities.iter().map(|e| e.x).min().unwrap();
    let min_y = entities.iter().map(|e| e.y).min().unwrap();
    for e in entities.iter_mut() {
        e.x -= min_x;
        e.y -= min_y;
    }
    // Metrics: derived with check_entry's own loop shape.
    let (mut max_x, mut max_y, mut tiles) = (0i32, 0i32, 0u32);
    for e in &entities {
        for (tx, ty) in occupied_tiles(e) {
            max_x = max_x.max(tx);
            max_y = max_y.max(ty);
            tiles += 1;
        }
    }
    let entry = CellEntry {
        motif: Motif::Unit {
            recipe: spec.recipe.to_string(),
            machine: spec.machine.to_string(),
            count,
        },
        ports: spec
            .ports
            .iter()
            .map(|&(dx, dy, kind, item)| Port { dx, dy, kind, item: item.to_string() })
            .collect(),
        metrics: Metrics {
            bbox_w: max_x + 1,
            bbox_h: max_y + 1,
            interior_tiles: tiles,
            entity_count: entities.len() as u32,
        },
        provenance: spec.provenance.to_string(),
        sim_anchor: "unanchored".to_string(),
        entities,
    };
    let issues = celldb::check_entry(&entry);
    if !issues.is_empty() {
        fail(&format!("{}: check_entry: {issues:#?}", spec.provenance));
    }
    entry
}

fn main() {
    let corpus = std::env::var("CORPUS")
        .unwrap_or_else(|_| fail("set CORPUS to the Phase-0 blueprint corpus directory"));
    let path = "crates/core/data/celldb.json";
    let mut db: CellDb = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let specs = specs();
    let ours: BTreeSet<&str> = specs.iter().map(|s| s.provenance).collect();
    db.entries.retain(|e| !ours.contains(e.provenance.as_str()));
    for spec in &specs {
        let entry = translate(spec, &corpus);
        let Motif::Unit { count, .. } = &entry.motif else { unreachable!() };
        println!(
            "{}: {}x{} bbox, {} interior tiles, {} entities, count={count}",
            spec.provenance,
            entry.metrics.bbox_w,
            entry.metrics.bbox_h,
            entry.metrics.interior_tiles,
            entry.metrics.entity_count,
        );
        db.entries.push(entry);
    }
    std::fs::write(path, serde_json::to_string_pretty(&db).unwrap()).unwrap();
    println!("wrote {} entries to {path}", db.entries.len());
}
