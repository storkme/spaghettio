//! RFC-068 Phase 0 — the self-stamp fidelity probe (K68-1, plus the unit
//! half of K68-2). See `docs/rfc-068-multi-group-stamping.md`.
//!
//! NO PRODUCT CODE: the ports→`RowSpan`-semantics adapter under test lives
//! HERE, in the probe, per the RFC's P0 phasing. P1 builds the real stamp
//! path in `place_rows` only if this gate passes.
//!
//! Per seed fixture (`celldb::seed_sources()`):
//!   1. Build the NATIVE layout (`build_bus_layout`, default options) and
//!      validate it — the control.
//!   2. Select each engine seed **by provenance (`engine@…`), never by
//!      `query_unit`** — the copper-plate@48 key collision resolves the
//!      normal query to the community donor (753 < 817 interior tiles),
//!      which would silently void the isolation premise (RFC-068 decision
//!      log, review round 1, item 3).
//!   3. Run the adapter: derive the band's `RowSpan`-shaped contract from
//!      the entry's DECLARED PORTS alone — input belt ys ordered by the
//!      spec's input schedule, output edge/flow vs the slot's band role,
//!      `output_feed_x_min` from drop coverage — and check every field
//!      against the native band's actual geometry (via a fresh
//!      `extract_unit` of the same layout, which the celldb drift test
//!      independently pins to the store).
//!   4. Substitute the fragment at the band's native slot with an
//!      index-preserving swap (`LayoutResult` power-wire records reference
//!      entity indices; reordering would fabricate a record-integrity
//!      seam) and assert entity identity modulo id/order/rate. Rate stamps
//!      are restored from the native entity: rates are derived by the
//!      pipeline, never stored (the celldb schema rule), and P1's stamp
//!      path re-stamps them via normal lane planning.
//!   5. Validator verdict diff: K68-1's bar is Error-parity; identity
//!      makes full issue-list parity the expectation, so both are
//!      asserted. Stated limit (RFC): this gate cannot catch the
//!      `output_feed_x_min` throughput class — that is P2/P3's meter/sim
//!      obligation.
//!
//! Escape hatches used (K68-2 unit half): ZERO — every unmappable port,
//! schedule item without a port, or unfilled field is a hard failure here,
//! never a workaround.

use rustc_hash::FxHashMap;
use spaghettio_core::bus::layout::{self, LayoutOptions};
use spaghettio_core::celldb::{self, CellEntry, Motif, PortKind};
use spaghettio_core::common::is_machine_entity;
use spaghettio_core::models::{EntityDirection, LayoutResult, PlacedEntity, SolverResult};
use spaghettio_core::solver;
use spaghettio_core::validate::{self, LayoutStyle, Severity, ValidationIssue};

fn issues_of(l: &LayoutResult, s: &SolverResult) -> Vec<ValidationIssue> {
    match validate::validate(l, Some(s), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    }
}

fn error_count(issues: &[ValidationIssue]) -> usize {
    issues.iter().filter(|i| i.severity == Severity::Error).count()
}

/// Compact issue fingerprint for parity diffs.
fn fingerprint(issues: &[ValidationIssue]) -> Vec<String> {
    let mut v: Vec<String> = issues
        .iter()
        .map(|i| format!("{:?}|{}|{:?},{:?}|{}", i.severity, i.category, i.x, i.y, i.message))
        .collect();
    v.sort();
    v
}

/// The band: every entity `extract_unit` claims for this recipe — its
/// machines plus every `row:{recipe}:*` segment entity. Indices, so the
/// substitution can be index-preserving.
fn band_indices(entities: &[PlacedEntity], recipe: &str) -> Vec<usize> {
    let prefix = format!("row:{recipe}:");
    entities
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            (e.recipe.as_deref() == Some(recipe) && is_machine_entity(&e.name))
                || e.segment_id.as_deref().is_some_and(|s| s.starts_with(&prefix))
        })
        .map(|(i, _)| i)
        .collect()
}

/// Identity key: the full serialized entity with rate stripped and
/// coordinates normalized to the band origin. "Identical modulo entity
/// id/order" (RFC P0 verification bullet) — rate is additionally excluded
/// because the store strips it at extraction (rates are derived, never
/// stored) and the substitution restores the native stamp.
fn identity_key(e: &PlacedEntity, origin: (i32, i32)) -> String {
    let mut n = e.clone();
    n.rate = None;
    n.x -= origin.0;
    n.y -= origin.1;
    serde_json::to_string(&n).expect("PlacedEntity serializes")
}

/// What the probe's adapter produced for one band, in slot-absolute
/// coordinates.
struct AdapterOut {
    /// Per solid input item, IN THE SPEC'S SCHEDULE ORDER: the input belt
    /// port ys (multi-port per item allowed — split rows).
    input_ys: Vec<(String, Vec<i32>)>,
    output_ys: Vec<i32>,
    output_east: bool,
    output_feed_x_min: Option<i32>,
}

/// The ports→RowSpan-semantics adapter under test. Consumes ONLY the store
/// entry (ports + entities + motif) plus the slot origin, the group's
/// input schedule, and the slot's band role — never the native band.
fn adapt(
    entry: &CellEntry,
    slot: (i32, i32),
    schedule: &[String],
    role_final: bool,
) -> Result<AdapterOut, String> {
    // --- input belt ys, schedule-ordered (RFC round-1 obligation 2:
    // the lane planner indexes input belts by schedule order; an item the
    // adapter cannot place at its schedule index is a refusal).
    let mut input_ys = Vec::new();
    for item in schedule {
        let ys: Vec<i32> = entry
            .ports
            .iter()
            .filter(|p| p.kind == PortKind::BeltIn && &p.item == item)
            .map(|p| slot.1 + p.dy)
            .collect();
        if ys.is_empty() {
            return Err(format!("schedule item '{item}' has no belt-in port"));
        }
        input_ys.push((item.clone(), ys));
    }
    for p in &entry.ports {
        if p.kind == PortKind::BeltIn && !schedule.iter().any(|s| s == &p.item) {
            return Err(format!("belt-in port for '{}' not in the input schedule", p.item));
        }
    }

    // --- output edge/flow vs slot role (orientation is resolved at
    // storage time; a mismatched edge is inadmissible, never transformed).
    let out_ports: Vec<_> =
        entry.ports.iter().filter(|p| p.kind == PortKind::BeltOut).collect();
    if out_ports.is_empty() {
        return Err("no belt-out port".into());
    }
    // Edge admissibility per the RFC: the belt-out PORT edge must match
    // the slot's role — east edge for a final band, west edge for an
    // intermediate band. Judged on the port tile (the run's exit), not on
    // every run tile: runs legitimately contain corner/UG tiles whose own
    // direction is not the run's flow.
    let mut east_votes = 0usize;
    let mut west_votes = 0usize;
    for p in &out_ports {
        let on_east = p.dx == entry.metrics.bbox_w - 1;
        let on_west = p.dx == 0;
        if on_east == on_west {
            return Err(format!(
                "belt-out port at ({}, {}) is not on an x-edge of the {}x{} fragment",
                p.dx, p.dy, entry.metrics.bbox_w, entry.metrics.bbox_h
            ));
        }
        // Corroborate with the exit tile's own direction — it flows OFF
        // the fragment, so it must point outward along x.
        let tile = entry
            .entities
            .iter()
            .find(|e| e.x == p.dx && e.y == p.dy)
            .ok_or_else(|| format!("belt-out port ({}, {}) has no entity", p.dx, p.dy))?;
        let expect_dir = if on_east { EntityDirection::East } else { EntityDirection::West };
        if tile.direction != expect_dir {
            return Err(format!(
                "belt-out exit tile at ({}, {}) flows {:?}, expected {:?} for its edge",
                p.dx, p.dy, tile.direction, expect_dir
            ));
        }
        if on_east {
            east_votes += 1;
        } else {
            west_votes += 1;
        }
    }
    if east_votes > 0 && west_votes > 0 {
        return Err("belt-out ports disagree on exit edge".into());
    }
    let east = east_votes > 0;
    if role_final != east {
        return Err(format!(
            "slot role (final={role_final}) does not match stored output edge (east={east})"
        ));
    }
    let output_ys: Vec<i32> = out_ports.iter().map(|p| slot.1 + p.dy).collect();

    // --- output_feed_x_min: drop-coverage branch (RFC round-2). Engine
    // unit seeds are ordinary rows — one output drop per machine, coverage
    // continuous from the run start → None (exactly what `place_rows`
    // sets, so self-stamps stay native-identical). The discrete-drop
    // (DI-shape) arm returns Some(rightmost drop column); P2's donor
    // adapter owes the full geometric derivation — this probe's derivation
    // is the count form, sufficient for engine seeds where templates place
    // exactly one output inserter per machine.
    let machine_count = match &entry.motif {
        Motif::Unit { count, .. } => *count as usize,
        Motif::Fused { .. } => return Err("fused motifs are out of P0 scope".into()),
    };
    let drops: Vec<&PlacedEntity> = entry
        .entities
        .iter()
        .filter(|e| {
            e.segment_id
                .as_deref()
                .is_some_and(|s| s.split(':').nth(2) == Some("inserter-out"))
        })
        .collect();
    if drops.is_empty() {
        return Err("no inserter-out drops; cannot derive output coverage".into());
    }
    let output_feed_x_min = if drops.len() >= machine_count {
        None // continuous — the ordinary-row shape
    } else {
        Some(slot.0 + drops.iter().map(|e| e.x).max().unwrap())
    };

    Ok(AdapterOut { input_ys, output_ys, output_east: east, output_feed_x_min })
}

/// Run the whole probe for one fixture; returns (targets probed).
fn probe_fixture(
    item: &str,
    rate: f64,
    machine: &str,
    inputs: &[&str],
    targets: &[(&str, &str)],
    print_seam: bool,
) -> usize {
    let input_set = inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve(item, rate, &input_set, machine).expect("seed source solves");
    let native =
        layout::build_bus_layout(&sr, LayoutOptions::default()).expect("seed source lays out");
    let native_issues = issues_of(&native, &sr);

    let mut probed = 0usize;
    for (recipe, target_machine) in targets {
        // Provenance-scoped entry selection (never query_unit — see module
        // doc). Mirrors the celldb drift test's lookup exactly.
        let entry = celldb::celldb()
            .entries
            .iter()
            .find(|e| {
                e.provenance.starts_with("engine@")
                    && matches!(&e.motif, Motif::Unit { recipe: r, machine: m, .. }
                        if r == recipe && m == target_machine)
            })
            .unwrap_or_else(|| panic!("{recipe} engine seed is in the store"));

        // Native band + slot origin.
        let idx = band_indices(&native.entities, recipe);
        assert!(!idx.is_empty(), "{recipe}: native band not found");
        let slot = (
            idx.iter().map(|&i| native.entities[i].x).min().unwrap(),
            idx.iter().map(|&i| native.entities[i].y).min().unwrap(),
        );

        // Freshly extract the band; the drift test pins stored==fresh, but
        // re-assert here so a probe failure isolates locally.
        let (fresh, warnings) =
            celldb::extract_unit(&native.entities, recipe, target_machine, "probe");
        assert!(warnings.is_empty(), "{recipe}: extraction escape hatches: {warnings:?}");
        let fresh = fresh.expect("fresh extraction succeeds");
        assert_eq!(entry.ports, fresh.ports, "{recipe}: stored ports drifted from geometry");

        // The group's input schedule: solid inputs, spec order.
        let spec = sr
            .machines
            .iter()
            .find(|m| m.recipe == *recipe)
            .unwrap_or_else(|| panic!("{recipe}: no machine group in solve"));
        let schedule: Vec<String> = spec
            .inputs
            .iter()
            .filter(|f| !f.is_fluid)
            .map(|f| f.item.clone())
            .collect();
        let role_final = sr.external_outputs.iter().any(|o| &o.item == recipe);

        // --- adapter under test ---
        let out = adapt(entry, slot, &schedule, role_final)
            .unwrap_or_else(|e| panic!("{recipe}: adapter refusal (K68-2 escape hatch): {e}"));

        // Field checks against native geometry, via the fresh extraction's
        // port coordinates (slot-relative → slot-absolute).
        for (item, ys) in &out.input_ys {
            let mut expect: Vec<i32> = fresh
                .ports
                .iter()
                .filter(|p| p.kind == PortKind::BeltIn && &p.item == item)
                .map(|p| slot.1 + p.dy)
                .collect();
            let mut got = ys.clone();
            expect.sort();
            got.sort();
            assert_eq!(got, expect, "{recipe}: input belt ys for '{item}' diverge from native");
        }
        let mut expect_out: Vec<i32> = fresh
            .ports
            .iter()
            .filter(|p| p.kind == PortKind::BeltOut)
            .map(|p| slot.1 + p.dy)
            .collect();
        let mut got_out = out.output_ys.clone();
        expect_out.sort();
        got_out.sort();
        assert_eq!(got_out, expect_out, "{recipe}: output belt ys diverge from native");
        assert_eq!(out.output_east, role_final, "{recipe}: output flow vs band role");
        assert_eq!(
            out.output_feed_x_min, None,
            "{recipe}: engine seed derived a discrete-drop coverage; expected continuous"
        );

        // --- index-preserving substitution ---
        let mut frag_by_key: FxHashMap<String, &PlacedEntity> = FxHashMap::default();
        for e in &entry.entities {
            let prev = frag_by_key.insert(identity_key(e, (0, 0)), e);
            assert!(prev.is_none(), "{recipe}: duplicate identity key in fragment");
        }
        assert_eq!(
            idx.len(),
            entry.entities.len(),
            "{recipe}: native band and stored fragment entity counts differ"
        );
        let mut stamped = native.clone();
        for &i in &idx {
            let nat = &native.entities[i];
            let key = identity_key(nat, slot);
            let frag = frag_by_key.remove(&key).unwrap_or_else(|| {
                panic!("{recipe}: native band entity has no fragment counterpart: {key}")
            });
            let mut e = frag.clone();
            e.x += slot.0;
            e.y += slot.1;
            e.rate = nat.rate; // derived, never stored — restored from native
            stamped.entities[i] = e;
        }
        assert!(
            frag_by_key.is_empty(),
            "{recipe}: {} fragment entities had no native slot",
            frag_by_key.len()
        );

        // --- verdict diff: K68-1 ---
        let stamped_issues = issues_of(&stamped, &sr);
        assert_eq!(
            error_count(&stamped_issues),
            error_count(&native_issues),
            "{recipe}: ERROR-PARITY FAILED (K68-1)\nnative: {:?}\nstamped: {:?}",
            fingerprint(&native_issues),
            fingerprint(&stamped_issues),
        );
        assert_eq!(
            fingerprint(&stamped_issues),
            fingerprint(&native_issues),
            "{recipe}: full issue parity failed (identity substitution should be verdict-identical)"
        );

        if print_seam && probed == 0 {
            let y0 = idx.iter().map(|&i| native.entities[i].y).min().unwrap();
            let y1 = idx.iter().map(|&i| native.entities[i].y).max().unwrap();
            println!("── seam tiles for {recipe} band (y {y0}..{y1}) ──");
            for e in &stamped.entities {
                if e.y == y0 - 1 || e.y == y1 + 1 {
                    println!("  {} @ ({},{}) dir={:?} seg={:?}", e.name, e.x, e.y, e.direction, e.segment_id);
                }
            }
        }
        probed += 1;
    }
    probed
}

/// ec@20-from-ore: four engine seeds — copper-plate / iron-plate
/// (intermediate west-flowing furnace bands), copper-cable (intermediate),
/// electronic-circuit (final east-flowing) — both K68-1 band roles.
#[test]
fn p0_selfstamp_ec20_from_ore() {
    let sources = celldb::seed_sources();
    let (item, rate, machine, inputs, targets) = sources
        .iter()
        .find(|s| s.0 == "electronic-circuit")
        .expect("ec seed source");
    let inputs: Vec<&str> = inputs.to_vec();
    let n = probe_fixture(item, *rate, machine, &inputs, targets, true);
    assert_eq!(n, 4, "all four ec-fixture engine seeds probed");
}

/// ac@4-from-plates: the advanced-circuit final band (3-group layout — ec
/// and copper-cable co-solve, so the band has real seam neighbors; RFC-068
/// review round 3).
#[test]
fn p0_selfstamp_ac4_from_plates() {
    let sources = celldb::seed_sources();
    let (item, rate, machine, inputs, targets) = sources
        .iter()
        .find(|s| s.0 == "advanced-circuit")
        .expect("ac seed source");
    let inputs: Vec<&str> = inputs.to_vec();
    let n = probe_fixture(item, *rate, machine, &inputs, targets, false);
    assert_eq!(n, 1, "the ac engine seed probed");
}
