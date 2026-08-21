#!/usr/bin/env python3
"""Tests for scripts/sim-localize.py.

Run: python3 -m pytest scripts/test_sim_localize.py -q
"""
import importlib.util
import json
import pathlib

import pytest

HERE = pathlib.Path(__file__).parent
REPO_ROOT = HERE.parent
TESTDATA = REPO_ROOT / "web" / "src" / "ui" / "testdata"

spec = importlib.util.spec_from_file_location("_sim_localize", HERE / "sim-localize.py")
sim_localize = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim_localize)


def run(report_path, argv=()):
    """Run main() with the given report path + extra args, capturing stdout
    via capsys is done by the caller; this just builds sys.argv."""
    import sys

    old_argv = sys.argv
    sys.argv = ["sim-localize.py", str(report_path), *argv]
    try:
        return sim_localize.main()
    finally:
        sys.argv = old_argv


# --- old-format testdata --------------------------------------------------


def test_ec10_fail_ranks_both_shortage_assemblers_first(capsys):
    rc = run(TESTDATA / "sim-report-ec10-fail.json")
    out = capsys.readouterr().out
    assert rc == 0
    assert "timeseries: absent — falling back to final frame" in out

    top = json.loads((TESTDATA / "sim-report-ec10-fail.json").read_text())
    rows = sim_localize.rank_from_final_frame(top["sim_state"])
    top2 = {(r["x"], r["y"]) for r in rows[:2]}
    assert top2 == {(15, 64), (15, 72)}
    assert all(r["frac_shortage"] == 1.0 for r in rows[:2])
    # both are item_ingredient_shortage, ranked above the full_output ones
    assert rows[2]["frac_backpressure"] == 1.0


def test_ec10_fail_prints_verdict_and_below_plan_listing(capsys):
    run(TESTDATA / "sim-report-ec10-fail.json")
    out = capsys.readouterr().out
    assert "verdict: FAIL" in out
    assert "*electronic-circuit" in out
    # cable (52%) sorts before iron (56%); copper-plate (100%) is not listed
    line = next(l for l in out.splitlines() if l.startswith("below-plan intermediates"))
    assert line.index("copper-cable 52%") < line.index("iron-plate 5")
    assert "copper-plate" not in line


def test_gear10_pass_has_no_starved_machines(capsys):
    rc = run(TESTDATA / "sim-report-gear10-pass.json")
    out = capsys.readouterr().out
    assert rc == 0
    assert "no starved/backpressured machines found." in out
    top = json.loads((TESTDATA / "sim-report-gear10-pass.json").read_text())
    assert sim_localize.rank_from_final_frame(top["sim_state"]) == []


def test_old_format_map_uses_hash_and_dot(capsys):
    run(TESTDATA / "sim-report-ec10-fail.json")
    out = capsys.readouterr().out
    map_section = out.split("--- map ---")[1]
    assert "#" in map_section  # nonempty old-format belt


# --- kit_errors location (report.kit_errors preferred, raw_result fallback) --


def test_kit_errors_of_prefers_report_then_falls_back_to_raw_result():
    assert sim_localize.kit_errors_of({"report": {"kit_errors": ["a"]}, "raw_result": {}}) == ["a"]
    assert sim_localize.kit_errors_of({"report": {}, "raw_result": {"kit_errors": ["b"]}}) == ["b"]
    assert sim_localize.kit_errors_of({"report": {}, "raw_result": {}}) == []


# --- shape classifier ------------------------------------------------------


def test_classify_shape_flat_zero():
    assert sim_localize.classify_shape([0, 0, 0, 0], ["item_ingredient_shortage"] * 4) == "flat-zero"


def test_classify_shape_ramp_then_decay():
    assert sim_localize.classify_shape([2, 10, 15, 3], ["working", "working", "working", "full_output"]) == "ramp-then-decay"


def test_classify_shape_healthy():
    assert sim_localize.classify_shape([10, 10.5, 9.8, 10.1], ["working"] * 4) == "healthy"


def test_classify_shape_stable_below():
    deltas = [5, 5.1, 4.9, 5.0]
    statuses = ["item_ingredient_shortage", "working", "item_ingredient_shortage", "working"]
    assert sim_localize.classify_shape(deltas, statuses) == "stable-below"


# --- synthetic new-format report -------------------------------------------


def make_new_format_report():
    belts = [
        # nonempty regular belt, direction=east(4)
        [10, 10, 4, [[["iron-plate", 4]], [["iron-plate", 4]]], "transport-belt", 4, None],
        # empty regular belt
        [11, 10, 0, [], "transport-belt", 4, None],
        # UG pair: input then output, direction=east(4)
        [12, 10, 8, [[["copper-plate", 2]], []], "fast-underground-belt", 4, "input"],
        [15, 10, 8, [[["copper-plate", 2]], []], "fast-underground-belt", 4, "output"],
    ]
    machines = [
        [10, 12, "assembling-machine-2", "item_ingredient_shortage", {"iron-plate": 0}],
        [20, 12, "assembling-machine-2", "working", {"iron-plate": 10}],
    ]
    inserters = [[10, 11, "waiting_for_source_items", 0]]
    timeseries = []
    for i in range(4):
        timeseries.append({
            "tick": 1200 * (i + 1),
            "machines": [
                {"unit": 1, "name": "assembling-machine-2", "x": 10, "y": 12,
                 "crafts_delta": 0.0, "status": "item_ingredient_shortage"},
                {"unit": 2, "name": "assembling-machine-2", "x": 20, "y": 12,
                 "crafts_delta": 15.0 + i * 0.1, "status": "working"},
            ],
            "items": {"iron-plate": 0.0, "copper-plate": 30.0},
        })
    return {
        "game_version": "2.0.77",
        "raw_result": {"kit_errors": []},
        "report": {
            "label": "synthetic",
            "overall_verdict": "FAIL",
            "converged": True,
            "kit_errors": [],
            "validator_standing": "warned",
            "validator": {"errors": 0, "warnings": 2, "layout_warnings": 0,
                          "by_category": {"input-rate-delivery": {"errors": 0, "warnings": 2}}},
            "items": [
                {"item": "iron-plate", "planned_rate": 10.0, "measured_produced_rate": 2.0,
                 "measured_delivered_rate": 1.8, "delta_pct_produced": -80.0, "delta_pct_delivered": -82.0,
                 "is_target": True, "verdict": "FAIL"},
                {"item": "copper-plate", "planned_rate": 10.0, "measured_produced_rate": 9.9,
                 "measured_delivered_rate": None, "delta_pct_produced": -1.0, "delta_pct_delivered": None,
                 "is_target": False, "verdict": None},
            ],
            "timeseries": timeseries,
        },
        "run_params": {"scenario_name": "synthetic-scenario"},
        "sim_state": {"offx": 0, "offy": 0, "belts": belts, "machines": machines, "inserters": inserters},
    }


def test_synthetic_new_format_ranking_and_shapes(tmp_path, capsys):
    report = make_new_format_report()
    path = tmp_path / "report.json"
    path.write_text(json.dumps(report))

    rc = run(path)
    out = capsys.readouterr().out
    assert rc == 0
    assert "checkpoint window(s)" in out  # timeseries path taken, not the frame fallback
    assert "flat-zero" in out
    assert "1 healthy machine(s) omitted" in out  # unit 2 is healthy: not listed, not lane-detailed
    assert "machine 2 assembling-machine-2" not in out

    rows = sim_localize.rank_from_timeseries(report["report"]["timeseries"])
    assert rows[0]["unit"] == 1  # flat-zero shortage machine outranks the healthy one
    assert rows[0]["shape"] == "flat-zero"
    assert rows[0]["frac_shortage"] == 1.0
    unit2 = next(r for r in rows if r["unit"] == 2)
    assert unit2["shape"] == "healthy"
    assert unit2["frac_shortage"] == 0.0


def test_synthetic_new_format_map_has_direction_arrows_and_ug_glyphs(capsys):
    report = make_new_format_report()
    grid, all_xy = sim_localize.build_grid(report["sim_state"], window=None)
    assert grid[(10, 10)] == ">"  # nonempty, direction=east
    assert grid[(11, 10)] == "."  # empty regular belt
    assert grid[(12, 10)] == "U"  # UG input, nonempty
    assert grid[(15, 10)] == "O"  # UG output, nonempty
    assert grid[(10, 12)] == "S"  # machine anchor wins over nothing else here
    assert grid[(20, 12)] == "W"


def test_synthetic_new_format_lane_detail_shows_per_lane_contents():
    report = make_new_format_report()
    worst_rows = [{"unit": 1, "name": "assembling-machine-2", "x": 10, "y": 12}]
    belts = [sim_localize.parse_belt(b) for b in report["sim_state"]["belts"]]
    b = next(b for b in belts if b["x"] == 10 and b["y"] == 10)
    assert sim_localize.lane_str(b["det"], 0) == "iron-plate×4"
    assert sim_localize.lane_str(b["det"], 1) == "iron-plate×4"
    empty = next(b for b in belts if b["x"] == 11 and b["y"] == 10)
    assert sim_localize.lane_str(empty["det"], 0) == "—"


def test_kit_errors_case_prints_loud_banner(tmp_path, capsys):
    report = make_new_format_report()
    report["report"]["kit_errors"] = ["overlapping bank chests at (3,4)"]
    path = tmp_path / "report.json"
    path.write_text(json.dumps(report))

    rc = run(path)
    out = capsys.readouterr().out
    assert rc == 0
    assert "!! KIT ERRORS" in out
    assert "overlapping bank chests at (3,4)" in out
    # the map still renders despite kit errors (may show the kit fault)
    assert "--- map ---" in out


def test_validator_line_variants():
    assert sim_localize.validator_line({"report": {}}) == "unknown (pre-field report)"
    assert sim_localize.validator_line(
        {"report": {"validator_standing": "unknown", "validator": None}}
    ).startswith("? (manifest predates")
    clean = sim_localize.validator_line(
        {"report": {"validator_standing": "unflagged",
                     "validator": {"errors": 0, "warnings": 0, "layout_warnings": 0, "by_category": {}}}}
    )
    assert clean.startswith("clean")
    warned = sim_localize.validator_line(
        {"report": {"validator_standing": "warned",
                     "validator": {"errors": 0, "warnings": 3, "layout_warnings": 0,
                                    "by_category": {"input-rate-delivery": {"errors": 0, "warnings": 3}}}}}
    )
    assert warned == "3W — input-rate-delivery×3"


# --- review-bot findings on #697 -------------------------------------------


@pytest.mark.parametrize("bad", ["10", "a,b", "1,2,x", "", "1,2,3,4"])
def test_malformed_around_degrades_to_full_extent(tmp_path, capsys, bad):
    path = tmp_path / "report.json"
    path.write_text(json.dumps(make_new_format_report()))
    rc = run(path, ["--around", bad])
    captured = capsys.readouterr()
    assert rc == 0
    assert "--- map ---" in captured.out
    assert "--around expects X,Y[,R]" in captured.err


def test_below_plan_gate_uses_the_worst_of_several_targets(tmp_path, capsys):
    report = make_new_format_report()
    # a starved primary target followed (in table order) by a healthy secondary
    report["report"]["items"].append(
        {"item": "iron-gear-wheel", "planned_rate": 5.0, "measured_produced_rate": 5.1,
         "measured_delivered_rate": 5.05, "delta_pct_produced": 2.0, "delta_pct_delivered": 1.0,
         "is_target": True, "verdict": "PASS"})
    path = tmp_path / "report.json"
    path.write_text(json.dumps(report))
    run(path)
    out = capsys.readouterr().out
    # the primary target is at 20% of plan, so the gate must open even though
    # the LAST target in table order is healthy; copper-plate (99%) is the only
    # intermediate and sits above the 98% bar, so this exact branch prints.
    assert "no intermediate below 98%" in out


def test_no_ingredients_and_unknown_statuses_are_not_hidden():
    ts = [{"tick": 1200 * (i + 1), "machines": [
        {"unit": 1, "name": "assembling-machine-2", "x": 0, "y": 0, "crafts_delta": 0.0, "status": "no_ingredients"},
        {"unit": 2, "name": "assembling-machine-2", "x": 5, "y": 0, "crafts_delta": 10.0, "status": "some_new_status"},
        {"unit": 3, "name": "assembling-machine-2", "x": 9, "y": 0, "crafts_delta": 10.0, "status": "working"},
    ], "items": {}} for i in range(3)]
    rows = sim_localize.rank_from_timeseries(ts)
    by_unit = {r["unit"]: r for r in rows}
    assert by_unit[1]["frac_shortage"] == 1.0  # no_ingredients counts as a shortage, as the overlay does
    assert by_unit[2]["frac_other"] == 1.0  # unfamiliar status surfaces rather than reading as healthy
    assert rows[0]["unit"] == 1 and rows[1]["unit"] == 2


def test_both_ranking_paths_share_the_tie_break():
    frame = {"machines": [[9, 0, "m", "full_output"], [3, 0, "m", "full_output"], [3, -1, "m", "full_output"]]}
    frame_rows = sim_localize.rank_from_final_frame(frame)
    ts = [{"tick": 1, "machines": [
        {"unit": u, "name": "m", "x": x, "y": y, "crafts_delta": 0.0, "status": "full_output"}
        for u, (x, y) in enumerate([(9, 0), (3, 0), (3, -1)])], "items": {}}]
    ts_rows = sim_localize.rank_from_timeseries(ts)
    assert [(r["x"], r["y"]) for r in frame_rows] == [(r["x"], r["y"]) for r in ts_rows] == [(3, -1), (3, 0), (9, 0)]


# --- second-round findings on #697 -------------------------------------------


def test_classify_shape_never_calls_an_impaired_machine_healthy():
    cs = sim_localize.classify_shape
    assert cs([10, 50, 10, 50], ["item_ingredient_shortage"] * 4) == "unsteady"
    assert cs([10, 50, 10, 50], ["working"] * 4) == "healthy"
    assert cs([15, 5, 3, 2], ["working"] * 4) == "decaying"  # peak-first: winding down, not a jam
    assert cs([2, 15, 5, 3], ["working"] * 4) == "ramp-then-decay"
    assert cs([-1, -1], ["working"] * 2) == "unknown"  # can't occur; must not read as flat-zero


def test_empty_new_format_belt_gets_lane_detail_not_the_old_format_fallback(capsys):
    sim_state = {"belts": [[10, 10, 0, [], "transport-belt", 4, None],
                           [11, 10, 0, [{}, {}], "transport-belt", 4, None]]}
    sim_localize.print_lane_detail(sim_state, [{"unit": 7, "name": "m", "x": 10, "y": 11}], 2)
    out = capsys.readouterr().out
    assert "no per-lane detail" not in out
    assert out.count("L1: —  L2: —") == 2


def test_ranking_and_lane_detail_tolerate_missing_fields(capsys):
    ts = [{"tick": 1, "machines": [{"unit": 1, "name": "m", "crafts_delta": 0.0, "status": "no_power"}], "items": {}}]
    sim_localize.print_ranking({"report": {"timeseries": ts}, "sim_state": {}}, 3)  # packet lacks x/y
    assert "(?,?)" in capsys.readouterr().out
    assert sim_localize.lane_str([[["iron-plate", 4, "extra"]]], 0).startswith("?")


def test_around_pivots_lane_detail_to_that_tile(tmp_path, capsys):
    path = tmp_path / "report.json"
    path.write_text(json.dumps(make_new_format_report()))
    run(path, ["--around", "12,10,2"])
    out = capsys.readouterr().out
    assert "--- lane detail (around (12,10)" in out
    assert "tile (12,10):" in out
    assert "(12,10) fast-underground-belt > L1: copper-plate×2  L2: —" in out
    assert "machine 1 assembling-machine-2" not in out  # worst-machine detail replaced, not appended


# --- third-round findings on #697 ---------------------------------------------


@pytest.mark.parametrize("form", [["--around", "-5,-3,2"], ["--around=-5,-3,2"]])
def test_negative_coordinates_parse_in_both_forms(tmp_path, capsys, form):
    path = tmp_path / "report.json"
    path.write_text(json.dumps(make_new_format_report()))
    rc = run(path, form)
    out = capsys.readouterr().out
    assert rc == 0
    assert "--- lane detail (around (-5,-3), radius 2) ---" in out  # R doubles as lane radius


def test_map_glyphs_follow_the_ranking_status_sets():
    mg = sim_localize.machine_glyph
    assert mg("no_ingredients") == "S" and mg("fluid_ingredient_shortage") == "S"
    assert mg("no_fuel") == "P" and mg("low_power") == "P"
    assert mg("fluid_production_overload") == "F"
    assert mg("disabled") == "?"


def test_top_governs_the_ranking_table(capsys):
    frame = {"machines": [[x, 0, "m", "full_output"] for x in range(6)]}
    sim_localize.print_ranking({"report": {}, "sim_state": frame}, 1, 2)
    out = capsys.readouterr().out
    assert out.count("full_output") == 0 and "... and 4 more" in out


def test_negative_radius_is_clamped_with_a_note(tmp_path, capsys):
    path = tmp_path / "report.json"
    path.write_text(json.dumps(make_new_format_report()))
    rc = run(path, ["--radius", "-2"])
    captured = capsys.readouterr()
    assert rc == 0
    assert "--radius -2 is negative" in captured.err
    assert "radius 0)" in captured.out


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-q"]))
