//! RFC-062 Phase 2: dual-purpose lanes for shared solid rows (a target
//! item that is ALSO consumed internally by another row). See
//! `docs/rfc-062-multi-target-outputs.md` — §Layout, the Phase 0/Phase 2
//! decision-log entries, and the new `validate::check_shared_row_outflow_
//! conservation` invariant.
//!
//! Companion to `solver_multi_target.rs` (Phase 1). Bypasses the
//! single-target `RunParams` harness in `e2e.rs` (built around
//! `solver::solve*`'s scalar `target_item`/`target_rate`) — multi-target
//! solves go through `solve_netflow_multi` directly, same as Phase 1's
//! own test file.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::cells::CellComposition;
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::models::SolverResult;
use spaghettio_core::netflow::{solve_netflow_multi, CostTable, RecipeScope};
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::trace::{self, TraceEvent};
use spaghettio_core::validate::{self, LayoutStyle, Severity};

fn set(items: &[&str]) -> FxHashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// The canonical shared-intermediate case: `electronic-circuit@10/s` +
/// `advanced-circuit@3/s` from ore, AM2 — same params as Phase 1's
/// `kc2_ec_ac_shared_copper_cable_exact`.
fn ec_ac_solve() -> SolverResult {
    let inputs = set(&["iron-ore", "copper-ore", "coal", "water", "crude-oil"]);
    let targets = vec![
        ("electronic-circuit".to_string(), 10.0),
        ("advanced-circuit".to_string(), 3.0),
    ];
    solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-2",
        &FxHashSet::default(),
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("EC+AC from ore should solve")
}

/// Mechanism fixture: pins `cell_composition`/`direct_insertion` `Off` so
/// the NATIVE dual-purpose-lane mechanism — the one this phase actually
/// changed (`placer::is_final`, `lane_planner::plan_bus_lanes`'s
/// `solid_target_items` gate, `ghost_router` Step 7's `output_items`
/// skip) — is what builds the layout. Phase 0's own confound note: under
/// shipped defaults, the cell-composed candidate wins this exact shape
/// first and the native mechanism is never exercised at all. See
/// `ec_ac_default_options_candidate_choice` below for what the DEFAULT
/// engine actually does with this shape.
#[test]
fn ec_ac_shared_row_native_mechanism_zero_errors() {
    let _guard = trace::start_trace();
    let solver_result = ec_ac_solve();

    let layout = build_bus_layout(
        &solver_result,
        LayoutOptions {
            cell_composition: CellComposition::Off,
            direct_insertion: DirectInsertion::Off,
            ..Default::default()
        },
    )
    .expect("EC+AC native layout should build");

    let issues = match validate::validate(&layout, Some(&solver_result), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Warning).collect();
    eprintln!(
        "ec_ac_shared_row_native_mechanism_zero_errors: {} entities, {} errors, {} warnings",
        layout.entities.len(),
        errors.len(),
        warnings.len(),
    );
    assert!(
        errors.is_empty(),
        "expected zero validation errors on the native EC+AC layout, got: {:#?}",
        errors
    );

    // RFC-062 Phase 2 review finding (F1): `perimeter_exit_y` landing one
    // row PAST the layout's real bottom edge (`total_height`, an
    // EXCLUSIVE bound, instead of `total_height - 1`) shifted
    // `check_belt_flow_reachability`'s belt-derived boundary (its
    // `on_boundary` is computed from `max_by` over every belt tile) and
    // demoted every legitimate y = total_height-1 export tail from
    // "boundary" to "interior" — 24 false `belt-flow-reachability`
    // "items cannot leave" warnings, one per AC machine. The fluid
    // dual-purpose lane never hit this: a fluid exit is a pipe, invisible
    // to this belt-only check. Asserted explicitly (not just folded into
    // the total warning count) because the original fixture printed "28
    // warnings" and only inspected the `input-rate-delivery` subset,
    // which is exactly how this went unnoticed the first time.
    let belt_flow_reachability_warnings: Vec<_> =
        warnings.iter().filter(|i| i.category == "belt-flow-reachability").collect();
    assert!(
        belt_flow_reachability_warnings.is_empty(),
        "expected zero belt-flow-reachability warnings — a non-empty count here means the \
         perimeter exit is shifting the belt boundary again, got: {:#?}",
        belt_flow_reachability_warnings
    );

    // Neither new failure direction should fire on the fixed mechanism —
    // this is the check's own "stays quiet on the case it was built to
    // pass" regression.
    for category in ["shared-row-outflow-overclaim", "shared-row-outflow-underclaim"] {
        assert!(
            !issues.iter().any(|i| i.category == category),
            "expected zero `{category}` issues on the fixed native mechanism, got: {:#?}",
            issues.iter().filter(|i| i.category == category).collect::<Vec<_>>()
        );
    }

    // Phase 0's original failure mode: AC's electronic-circuit input
    // starved with `input-rate-delivery` issues because the row-level
    // export claim won the tile-level fight and the lane's tap-off lost.
    // Must be gone — scoped to electronic-circuit specifically (not
    // `input-rate-delivery` in general): copper-cable is ALSO shared
    // between EC's and AC's rows (AC's recipe draws 2 cable/craft
    // directly, per the RFC Motivation table), an ordinary multi-consumer
    // lane untouched by this phase's dual-purpose-lane fix (copper-cable
    // is not itself an external target, so `solid_target_items` never
    // gates it) — its own lane-balancing warnings are a pre-existing,
    // separate concern, logged below rather than asserted on.
    let ec_starvation: Vec<_> = issues
        .iter()
        .filter(|i| i.category == "input-rate-delivery" && i.message.contains("electronic-circuit"))
        .collect();
    assert!(
        ec_starvation.is_empty(),
        "AC's electronic-circuit input should be fully fed by the shared EC lane, got: {:#?}",
        ec_starvation
    );
    let other_delivery: Vec<_> = issues
        .iter()
        .filter(|i| i.category == "input-rate-delivery" && !i.message.contains("electronic-circuit"))
        .collect();
    if !other_delivery.is_empty() {
        eprintln!(
            "residual (not electronic-circuit, not this phase's regression): {} \
             input-rate-delivery issue(s) on other items: {:#?}",
            other_delivery.len(),
            other_delivery,
        );
    }

    // The EC row must physically show BOTH claims: the lane's internal
    // tap-off to AC's row AND a perimeter exit reaching the boundary —
    // asserted on entities/trace, not just the error count
    // (docs/validator-reporting.md rules 4-5).
    let events = trace::drain_events();
    let lanes_planned = events
        .iter()
        .find_map(|ev| if let TraceEvent::LanesPlanned { lanes, .. } = ev { Some(lanes.clone()) } else { None })
        .expect("expected a LanesPlanned trace event");
    let ec_lane = lanes_planned
        .iter()
        .find(|l| l.item == "electronic-circuit")
        .expect("electronic-circuit must have a bus lane (the dual-purpose lane)");
    assert!(
        !ec_lane.consumer_rows.is_empty(),
        "electronic-circuit lane must have real consumer rows (AC's tap-off), got {:?}",
        ec_lane.consumer_rows
    );
    assert!(
        !ec_lane.tap_off_ys.is_empty(),
        "electronic-circuit lane must have a physical tap-off y, got {:?}",
        ec_lane.tap_off_ys
    );

    // Physical perimeter exit: a real belt entity, not just a ledger
    // claim (validator-reporting rule 4) — this is exactly what
    // `check_shared_row_outflow_conservation`'s under-claim direction
    // cross-checks.
    let exit = layout
        .surplus_exits
        .iter()
        .find(|(item, _, _)| item == "electronic-circuit")
        .unwrap_or_else(|| {
            panic!(
                "expected an electronic-circuit surplus_exits (perimeter-exit) record; got {:?}",
                layout.surplus_exits
            )
        });
    let (_, exit_x, exit_y) = *exit;
    assert!(
        layout.entities.iter().any(|e| {
            e.x == exit_x
                && e.y == exit_y
                && e.carries.as_deref() == Some("electronic-circuit")
                && spaghettio_core::common::is_belt_entity(&e.name)
        }),
        "no real belt entity carrying electronic-circuit at the recorded perimeter exit ({exit_x},{exit_y})"
    );

    // Site 3 (ghost_router.rs Step 7's `output_items` skip) is otherwise
    // untested: reverting it alone (still routing electronic-circuit
    // through Step 7's row-level merge IN ADDITION to the lane) doesn't
    // change the error/warning counts above by itself — Phase 0's
    // original collision came from `is_final` (site 1) forcing
    // `output_east=true`, which site 3 alone cannot undo. The direct
    // signal is the absence of a `merger:electronic-circuit` segment —
    // Step 7's `merge_output_rows` always stamps `segment_id =
    // Some(format!("merger:{item}"))`, so a reverted site 3 would leave
    // this vestigial (electronic-circuit's row is west-flowing per site
    // 1, so the merge would source from the wrong belt entirely, but the
    // segment tag would still appear).
    //
    // Side effect worth noting for a future reader diffing this test: a
    // reverted site 3 also changes `output_items.len()` from 1
    // (`["advanced-circuit"]`, electronic-circuit skipped) to 2
    // (`["electronic-circuit", "advanced-circuit"]`), which flips Step
    // 7's `merge_x_cursor` from its single-item start (0) to its
    // multi-item start (`row_spans.iter().map(row_width).max() + 1`) —
    // every merger entity's x-position would shift east. That's a
    // SECOND independent signal (position drift) a reverted site 3
    // would trip, on top of the segment-tag check below.
    assert!(
        !layout.entities.iter().any(|e| {
            e.segment_id.as_deref() == Some("merger:electronic-circuit")
        }),
        "found a `merger:electronic-circuit` segment — site 3 (ghost_router Step 7's \
         output_items skip) isn't suppressing the row-level merge for the dual-purpose-lane item"
    );

    // AC's machines must actually exist and be fed — cross-check the
    // solver's AC machine count against real entities in the layout.
    let ac_machines_solver = solver_result
        .machines
        .iter()
        .find(|m| m.recipe == "advanced-circuit")
        .map(|m| m.count.ceil() as usize)
        .unwrap_or(0);
    assert!(ac_machines_solver > 0, "expected advanced-circuit machines in the solve");
    let ac_entities = layout
        .entities
        .iter()
        .filter(|e| e.recipe.as_deref() == Some("advanced-circuit"))
        .count();
    assert_eq!(
        ac_entities, ac_machines_solver,
        "expected {ac_machines_solver} advanced-circuit machine entities, found {ac_entities}"
    );
}

/// Documents what the DEFAULT engine (shipped `LayoutOptions::default()`
/// — `cell_composition: Candidate`, `direct_insertion: Candidate`) does
/// with this exact shape. Phase 0's spike found the cell-composed
/// candidate wins this collision-shape race and, under an earlier
/// belt-tier-capped run, silently dropped electronic-circuit's own
/// target export from `boundary_outputs` with zero validator errors.
/// This is not a regression THIS phase introduces — the native-mechanism
/// fix above never runs unless `cell_composition`/`direct_insertion` are
/// forced off — but Phase 0's decision log required verifying which
/// candidate wins under real (Phase-1-solved) multi-target input rather
/// than assuming. See the Phase 2 decision log for the resulting call:
/// this is documented as a found gap and explicitly out of THIS phase's
/// scope (the native 3-site fix), not silently left unverified.
#[test]
fn ec_ac_default_options_candidate_choice() {
    let _guard = trace::start_trace();
    let solver_result = ec_ac_solve();

    let layout = build_bus_layout(&solver_result, LayoutOptions::default())
        .expect("EC+AC default-options layout should build");

    let issues = match validate::validate(&layout, Some(&solver_result), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();

    let events = trace::drain_events();
    let winner = events.iter().find_map(|ev| {
        if let TraceEvent::DecompositionChosen { name, .. } = ev { Some(name.clone()) } else { None }
    });

    let ec_exported = layout
        .boundary_outputs
        .iter()
        .any(|r| r.item == "electronic-circuit")
        || layout.surplus_exits.iter().any(|(item, _, _)| item == "electronic-circuit");

    eprintln!(
        "ec_ac_default_options_candidate_choice: winner={winner:?}, errors={}, \
         electronic-circuit export claim present={ec_exported}",
        errors.len(),
    );

    // LOUD, not silent: whichever way this falls, print it so a reader
    // scanning `--nocapture` output sees the finding directly, and pin
    // the winner's identity so a future engine change that flips which
    // candidate wins this shape is a visible test diff, not a silent
    // drift. Intentionally NOT asserting `errors.is_empty()` or
    // `ec_exported` here — under the cell-composed candidate this is
    // known-broken (Phase 0), and asserting success would just be
    // deleted the next time someone "fixes" this test instead of the
    // underlying candidate.
    //
    // Asserting the IDENTITY, not just presence (RFC-062 Phase 2 review
    // F4) — `winner.is_some()` alone doesn't back the decision log's
    // claim that this pins which candidate wins. `PR #553`'s DI-scoring
    // changes (concurrent work, same session) could legitimately flip
    // this later; if it does, THIS assertion is the visibility that
    // matters — update it deliberately rather than let the test go on
    // "passing" while silently exercising a different candidate than the
    // decision log describes.
    assert_eq!(
        winner.as_deref(),
        Some("native"),
        "expected the native candidate to win the EC+AC shape on the real Phase-1 solver \
         output (Phase 0's cell-composition confound was observed on a hand-built \
         SolverResult and doesn't reproduce here) — if this legitimately changed (e.g. a \
         DI-scoring or decomposition-search update), update this assertion deliberately and \
         re-verify the CONFIRMED/else branch below still matches reality"
    );
    if winner.as_deref() != Some("native") && !ec_exported {
        eprintln!(
            "CONFIRMED (Phase 0 finding, reproduced on the real Phase-1 solver output): \
             the '{}' candidate wins the EC+AC shape and drops electronic-circuit's own \
             target export — zero physical export record for a requested target, with \
             {} validator error(s). This is the cell-composition/DI candidate-selection \
             gap named in the RFC-062 Phase 2 decision log, out of scope for the native \
             3-site fix this phase ships.",
            winner.as_deref().unwrap_or("<unknown>"),
            errors.len(),
        );
    }
}

/// The adversarial reviewer's deterministic surplus+target case:
/// `uranium-235@0.1` + `uranium-238@0.05` with `kovarex-enrichment-process`
/// excluded. Both items come off the SAME `uranium-processing` recipe
/// (probability 0.007 / 0.993 per craft) — U-238 is simultaneously a
/// requested export AND, at the machine count U-235's demand drives, a
/// large surplus (~14.13/s: `0.1/0.007 * 0.993 - 0.05`). Neither item has
/// an internal consumer (kovarex excluded), so this exercises a DIFFERENT
/// collision than the EC+AC dual-purpose-lane mechanism above: it's the
/// pre-existing D2a/D2b solid-surplus-secondary-belt path
/// (`docs/rfc-fulgora-scrap.md`), now asked to treat BOTH of a row's two
/// distinct solid outputs as external targets at once — untouched by
/// this phase's 3-site fix (neither item gets a dual-purpose lane: both
/// have zero internal consumers, so `lane_planner`'s `solid_target_items`
/// gate never applies). See the Phase 2 decision log for the outcome.
#[test]
fn u235_u238_target_and_surplus_overlap() {
    let _guard = trace::start_trace();
    let inputs = set(&["uranium-ore"]);
    let excluded = set(&["kovarex-enrichment-process"]);
    let targets = vec![
        ("uranium-235".to_string(), 0.1),
        ("uranium-238".to_string(), 0.05),
    ];
    let solver_result = solve_netflow_multi(
        &targets,
        &inputs,
        &MachinePalette::default(),
        "assembling-machine-3",
        &excluded,
        RecipeScope::Free,
        &CostTable::default(),
    )
    .expect("U-235+U-238 from ore, kovarex excluded, should solve");

    let u238_surplus = solver_result
        .surplus_outputs
        .iter()
        .find(|f| f.item == "uranium-238")
        .map(|f| f.rate)
        .unwrap_or(0.0);
    eprintln!("u235_u238_target_and_surplus_overlap: solver uranium-238 surplus = {u238_surplus:.3}/s");
    assert!(
        u238_surplus > 10.0,
        "expected a large uranium-238 surplus (~14.13/s hand-derived) alongside its \
         0.05/s target — got {u238_surplus:.3}/s; the fixture no longer exercises the \
         target+surplus overlap it was built for"
    );

    let layout = build_bus_layout(
        &solver_result,
        LayoutOptions {
            cell_composition: CellComposition::Off,
            direct_insertion: DirectInsertion::Off,
            ..Default::default()
        },
    )
    .expect("U-235+U-238 layout should build (even if validation reports issues)");

    let issues = match validate::validate(&layout, Some(&solver_result), LayoutStyle::Bus) {
        Ok(issues) => issues,
        Err(e) => e.issues,
    };
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
    eprintln!(
        "u235_u238_target_and_surplus_overlap: {} entities, {} errors, {} warnings",
        layout.entities.len(),
        errors.len(),
        issues.iter().filter(|i| i.severity == Severity::Warning).count(),
    );
    if !errors.is_empty() {
        eprintln!("errors: {:#?}", errors);
    }

    // Physical claim check for BOTH items, entity-cross-checked exactly
    // like `check_shared_row_outflow_conservation`'s under-claim
    // direction (and `check_stranded_byproducts`).
    let exported = |item: &str| -> bool {
        layout.boundary_outputs.iter().any(|r| {
            r.item == item
                && layout.entities.iter().any(|e| {
                    e.x == r.x
                        && e.y == r.y
                        && e.carries.as_deref() == Some(item)
                        && spaghettio_core::common::is_belt_entity(&e.name)
                })
        }) || layout.surplus_exits.iter().any(|(ei, ex, ey)| {
            ei == item
                && layout.entities.iter().any(|e| {
                    e.x == *ex
                        && e.y == *ey
                        && e.carries.as_deref() == Some(item)
                        && spaghettio_core::common::is_belt_entity(&e.name)
                })
        })
    };
    let u235_exported = exported("uranium-235");
    let u238_exported = exported("uranium-238");
    eprintln!(
        "u235_u238_target_and_surplus_overlap: uranium-235 export claim present={u235_exported}, \
         uranium-238 export claim present={u238_exported}"
    );

    // uranium-235 (the row's PRIMARY output) always gets a clean physical
    // export — that mechanism is `tier_uranium_processing_surplus_export`
    // (existing e2e fixture), untouched by this phase.
    assert!(u235_exported, "uranium-235 (primary output, existing D2a/D2b path) must export");

    // CONFIRMED GAP, documented LOUD rather than silently passed (per the
    // task brief: decide and make the behavior visible either way).
    // `exported()`'s own ledger+single-tile entity check reads TRUE for
    // uranium-238 — a `boundary_outputs`/`surplus_exits` record exists and
    // a real belt sits at its tile — but the layout is NOT actually
    // correct: `ghost_router` Step 7's per-item merge unconditionally
    // sources every item in `output_items` from `RowSpan::output_belt_y`
    // (the PRIMARY belt, uranium-235's), never
    // `RowSpan::secondary_output_belt` (uranium-238's real belt). Treating
    // uranium-238 as a Step-7 target this way collides physically with
    // uranium-235's own merge at the same tiles. This is a DIFFERENT
    // mechanism from the EC+AC dual-purpose-lane fix above (that fix only
    // applies to items with a real internal consumer; uranium-238 has
    // none, so `lane_planner`'s `solid_target_items` gate never applies
    // here) — a pre-existing Step-7/D2b gap that Phase 1's multi-target
    // solver newly makes reachable (single-target callers can never put
    // TWO of one row's distinct solid outputs in `external_outputs` at
    // once). Explicitly out of THIS phase's 3-site-fix scope; tracked as
    // a followup in the Phase 2 decision log rather than fixed here.
    //
    // Asserted as a hard characterization (not just logged) so a future
    // fix to Step 7's D2b targeting is forced to update this test instead
    // of silently going unnoticed either direction.
    assert!(
        u238_exported,
        "uranium-238's own boundary/surplus-exit ledger entry should still exist even \
         though the geometry backing it is broken — if this now fails, the ledger-level \
         symptom changed and the finding below needs re-verifying"
    );
    let overlap_errors: Vec<_> = errors.iter().filter(|i| i.category == "entity-overlap").collect();
    let isolation_errors: Vec<_> =
        errors.iter().filter(|i| i.category == "belt-item-isolation").collect();
    assert!(
        !overlap_errors.is_empty() && !isolation_errors.is_empty(),
        "expected the known Step-7/D2b uranium-238-as-target collision (entity-overlap + \
         belt-item-isolation errors mixing uranium-235/uranium-238 at the shared row) — \
         got {} entity-overlap and {} belt-item-isolation errors instead. If this is now \
         empty, either Step 7's D2b targeting was fixed (great — delete this \
         characterization and assert clean validation instead) or the collision moved to \
         a different, unaccounted-for failure shape (investigate before deleting).",
        overlap_errors.len(),
        isolation_errors.len(),
    );
    eprintln!(
        "CONFIRMED (documented, out of RFC-062 Phase 2 scope): {} entity-overlap + {} \
         belt-item-isolation error(s) from Step 7 sourcing uranium-238's merge from \
         uranium-235's `output_belt_y`. See the Phase 2 decision log.",
        overlap_errors.len(),
        isolation_errors.len(),
    );
}
