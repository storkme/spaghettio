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


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-q"]))
