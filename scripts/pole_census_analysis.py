#!/usr/bin/env python3
"""Build the committed census JSON (parts A-E + diagnostics) from the raw
per-case output of `pole_census.py`.

The raw census script produces only the per-case `cases` list; the parts A-E
summary layer that the committed JSON carries was, for the post-0f census, built
by an uncommitted ad-hoc script. This closes that reproducibility gap: run this
after regenerating snapshots (see pole_census.py's regeneration commands) and it
emits the full analysis JSON.

  SPAGHETTIO_SNAP_DIR=<dir> python scripts/pole_census_analysis.py

Env:
  SPAGHETTIO_SNAP_DIR   snapshot dir (passed through to pole_census)
  SPAGHETTIO_CENSUS_OUT output path (default: scripts/pole-census-2026-07-20-post3b.json)

Parts B/D/E operate on MEDIUM poles only (the ±3 densification subjects);
part C's pole totals include substations (RFC Phase 3b). See pole_census.py's
module docstring for the substation-handling rationale + methodology caveats.
"""
import json
import os
import statistics as stats
import sys
from collections import Counter

import pole_census  # same directory

HERE = os.path.dirname(os.path.abspath(__file__))
POST0F = os.path.join(HERE, "pole-census-2026-07-19-post0f.json")
OUT = os.environ.get(
    "SPAGHETTIO_CENSUS_OUT", os.path.join(HERE, "pole-census-2026-07-20-post3b.json")
)


def slack_stats(vals):
    if not vals:
        return {"n": 0, "median": 0, "mean": 0.0, "zero_slack_count": 0,
                "zero_slack_fraction": 0.0, "max": 0}
    zeros = sum(1 for v in vals if v == 0)
    return {
        "n": len(vals),
        "median": stats.median(vals),
        "mean": round(stats.mean(vals), 3),
        "zero_slack_count": zeros,
        "zero_slack_fraction": round(zeros / len(vals), 4),
        "max": max(vals),
    }


def main():
    cases = pole_census.main()
    post0f = json.load(open(POST0F))
    p0f_percase = {r["name"]: r for r in post0f["part_c_pole_totals"]["per_case"]}

    # ---------------------------------------------------------------- Part A
    pinned = [
        "tier2_electronic_circuit_20s_from_ore",
        "stress_electronic_circuit_60s_red_from_ore",
        "tier5_processing_unit_from_ore_am3",
        "processing_unit_2s_am2_fast_belts_validation_baseline",
        "tier_kovarex_self_loop",
    ]
    actual, tot_pw, tot_pe, cases_with_pw = {}, 0, 0, 0
    for c in cases:
        tot_pw += c["power_warning_count"]
        tot_pe += c["power_error_count"]
        if c["power_warning_count"] > 0:
            actual[c["name"]] = c["power_warning_count"]
            cases_with_pw += 1
    pw_of = {c["name"]: c["power_warning_count"] for c in cases}
    part_a = {
        "pinned_reds_post0f": {
            "tier2_electronic_circuit_20s_from_ore": 14,
            "stress_electronic_circuit_60s_red_from_ore": 60,
            "tier5_processing_unit_from_ore_am3": 20,
            "processing_unit_2s_am2_fast_belts_validation_baseline": 43,
            "tier_kovarex_self_loop": 16,
        },
        "known_nongating_post0f": {"census_utility_science_pack": 16},
        "actual_power_warnings_post3b": actual,
        "all_five_gating_pins_now_zero": all(pw_of.get(n) == 0 for n in pinned),
        "usp_now_zero": pw_of.get("census_utility_science_pack") == 0,
        "total_power_warnings_corpuswide": tot_pw,
        "total_power_errors_corpuswide": tot_pe,
        "cases_with_any_power_warning": cases_with_pw,
        "note": (
            "All five gating pins (EC@20 14, EC@60-red 60, PU-am3 20, PU-am2-baseline "
            "43, kovarex 16) and the known-hard non-gating census_utility_science_pack "
            "(16) are now ZERO power warnings/errors corpus-wide. Phase 3's reactive "
            "widen-plus-substation cleared every pinned honest-red power warning the "
            "arc opened with."
        ),
    }

    # ---------------------------------------------------- Part B (MEDIUM poles)
    solid_local, fluid_local, solid_band, fluid_band = [], [], [], []
    classif, unmatched, solid_zero_detail = Counter(), 0, []
    for c in cases:
        for p in c["poles"]:
            classif[p["classification"]] += 1
            if not p["matched"]:
                unmatched += 1
                continue
            if p["row_is_fluid"]:
                fluid_local.append(p["local_alternatives"])
                if p["band_alternatives"] is not None:
                    fluid_band.append(p["band_alternatives"])
            else:
                solid_local.append(p["local_alternatives"])
                if p["band_alternatives"] is not None:
                    solid_band.append(p["band_alternatives"])
                if p["local_alternatives"] == 0:
                    solid_zero_detail.append({
                        "case": c["name"], "x": p["x"], "y": p["y"],
                        "recipe": p["row_recipe"], "machine": p["row_machine"],
                        "band_alternatives": p["band_alternatives"],
                    })
    part_b = {
        "total_medium_poles": len(solid_local) + len(fluid_local) + unmatched,
        "unmatched_bridge_or_mopup": unmatched,
        "local_slack": {"solid": slack_stats(solid_local), "fluid": slack_stats(fluid_local)},
        "band_slack": {"solid": slack_stats(solid_band), "fluid": slack_stats(fluid_band)},
        "classification_breakdown": dict(classif),
        "post_0f_comparison": {
            "post_0f_solid_zero_slack": post0f["part_b_slack_distribution"]["local_slack"]["solid"]["zero_slack_count"],
            "post_0f_solid_median": post0f["part_b_slack_distribution"]["local_slack"]["solid"]["median"],
            "post_0f_fluid_zero_slack": post0f["part_b_slack_distribution"]["local_slack"]["fluid"]["zero_slack_count"],
            "post_0f_fluid_median": post0f["part_b_slack_distribution"]["local_slack"]["fluid"]["median"],
        },
        "HEADLINE_FINDING_trigger_a": (
            f"Solid-row zero-local-slack medium poles: {slack_stats(solid_local)['zero_slack_count']} "
            f"(post-0f: 18 — UNCHANGED count). Phase 3 trigger (a) ('any solid-row zero-slack pole') "
            f"was already TRUE post-0f and stays TRUE post-3b; Phase 3 is already activated, so NO "
            f"state change. Same advanced-circuit / deep AM south-band character (15 advanced-circuit "
            f"+ 3 deep science-pack rows); the exact pole set shifted by 4 members vs post-0f from "
            f"unrelated layout movement (0e-i fixtures, trim rider) between the two censuses. "
            f"Conservative FINAL-occupancy proxy — see caveats."
        ),
        "solid_zero_slack_detail": solid_zero_detail,
    }

    # ------------------------------------------------- Part C (all power poles)
    per_case, tot_post3b = [], 0
    base_post0f_matched = tot_post3b_matched = tot_new = 0
    for c in sorted(cases, key=lambda c: c["name"]):
        name, post3b = c["name"], c["real_pole_count"]
        tot_post3b += post3b
        p0 = p0f_percase.get(name)
        post0f_val = p0["post_0f"] if p0 else None
        if post0f_val is not None:
            base_post0f_matched += post0f_val
            tot_post3b_matched += post3b
            delta = post3b - post0f_val
            pct = round(100.0 * delta / post0f_val, 1) if post0f_val else None
        else:
            tot_new += post3b
            delta = pct = None
        per_case.append({
            "name": name, "post_0f": post0f_val, "post_3b": post3b,
            "medium": c["medium_pole_count"], "substation": c["substation_count"],
            "delta_vs_post_0f": delta, "pct_vs_post_0f": pct,
            "new_fixture_since_post_0f": post0f_val is None,
        })
    part_c = {
        "corpus_total_post_0f_45case": post0f["part_c_pole_totals"]["corpus_total_post_0f"],
        "corpus_total_post_3b_49case": tot_post3b,
        "corpus_total_post_3b_matched_45case": tot_post3b_matched,
        "corpus_total_new_phase0e1_fixtures": tot_new,
        "corpus_delta_matched_45case_vs_post0f": tot_post3b_matched - base_post0f_matched,
        "corpus_pct_delta_matched_45case": round(
            100.0 * (tot_post3b_matched - base_post0f_matched) / base_post0f_matched, 2),
        "corpus_substation_total": sum(c["substation_count"] for c in cases),
        "trigger_b_baseline_note": (
            "This 49-case corpus total (post_3b, 4251) is the NEW Phase 3 trigger-(b) "
            "baseline, superseding the post-0f 45-case anchor (4226). Matched-45 growth is "
            "+9 poles (+0.21%), every one of which is a Phase-3 case (EC@20 +1, EC@60-red +4, "
            "PU-am2-baseline +1, kovarex +2 = +1 medium +1 substation, USP +1 substation; "
            "PU-am3 net 0 — its widening repositioned poles without net-adding). All other 40 "
            "matched cases are ZERO delta. The growth is explained by Phase 3's design work and "
            "is nowhere near the >20% trigger threshold."
        ),
        "per_case": per_case,
    }

    # ---------------------------------------------------------------- Part D
    wd, bd = classif["within-inserter-span"], classif["beyond-inserter-span"]
    noi, unm = classif["no-inserters-on-face"], classif["unmatched-bridge-or-mopup"]
    tot_med, matched = wd + bd + noi + unm, wd + bd + noi
    part_d = {
        "within_inserter_span": wd, "beyond_inserter_span": bd,
        "no_inserters_on_face": noi, "unmatched_bridge_or_mopup": unm,
        "total_medium_poles": tot_med,
        "in_span_fraction_of_all_medium_poles": round(wd / tot_med, 4) if tot_med else 0,
        "in_span_fraction_of_matched_medium_poles": round(wd / matched, 4) if matched else 0,
        "post_0f_comparison": {
            "post_0f_within_span": post0f["part_d_in_span_fraction"]["within_inserter_span"],
            "post_0f_in_span_fraction": post0f["part_d_in_span_fraction"]["in_span_fraction_of_all_poles"],
        },
    }

    # ---------------------------------------------------------------- Part E
    all_budgets = []
    for c in cases:
        for b in c["fluid_row_budgets"]:
            row = dict(b)
            row["case"] = c["name"]
            all_budgets.append(row)
    zero_rows = [b for b in all_budgets if b["zero_local_slack_poles"] > 0]
    part_e = {
        "total_fluid_rows_corpuswide": len(all_budgets),
        "fluid_rows_with_zero_slack_pole": len(zero_rows),
        "worst_case_ranked_by_zero_slack_pole_count": sorted(
            zero_rows, key=lambda b: -b["zero_local_slack_poles"]),
        "all_fluid_row_budgets": all_budgets,
        "note": (
            "MEDIUM-pole free-tile budget within the shared pole_candidate_ys geometry. "
            "All worst-case fluid rows remain basic-oil-processing / oil-refinery rows — "
            "Phase 1 territory, unchanged by Phase 3."
        ),
    }

    # ------------------------------------------------------------- Diagnostics
    diag = {
        "ambiguous_pole_row_band_matches": sum(c["ambiguous_pole_matches"] for c in cases),
        "off_band_inserters": sum(c["inserter_assignment_off_band"] for c in cases),
        "seed_y_collisions_between_distinct_rows": sum(len(c["y_collisions"]) for c in cases),
        "heterogeneous_recipe_rows": sum(
            1 for c in cases for r in c["rows_summary"] if r["heterogeneous_recipe"]),
        "total_rows_post_3b": sum(c["n_rows"] for c in cases),
        "total_electric_inserters_post_3b": sum(c["electric_inserter_count"] for c in cases),
        "total_entities_post_3b": sum(c["n_entities"] for c in cases),
        "substation_cases": [
            {"name": c["name"], "substation_count": c["substation_count"],
             "medium": c["medium_pole_count"]}
            for c in cases if c["substation_count"] > 0
        ],
    }

    out = {
        "meta": {
            "purpose": "POST-Phase-3b power re-census (docs/rfc-power-reservation.md Phase 3c)",
            "commit": "ca8730e (Phase 3b landed; reactive band-widening 3a-ii + kovarex top-edge substation)",
            "generated": "2026-07-20",
            "n_snapshots": len(cases),
            "predecessor_census": "scripts/pole-census-2026-07-19-post0f.json (post-0f, 45 cases)",
            "analysis_script": "scripts/pole_census_analysis.py (imports scripts/pole_census.py)",
            "corpus_delta_vs_post0f": (
                "+4 cases (49 vs 45): the phase0e1_* fixtures (biolubricant/biochamber, "
                "fusion-power-cell/cryogenic-plant, molten-iron/foundry, "
                "superconductor/electromagnetic-plant) landed with Phase 0e-i AFTER the "
                "post-0f census."
            ),
            "regeneration_commands": post0f["meta"]["regeneration_commands"],
            "substation_handling": (
                "Substation (RFC Phase 3b) is a NEW pole TYPE: counted in part-C pole totals "
                "(real_pole_count = medium + substation) and modelled as a 2x2 slack obstacle, "
                "but EXCLUDED from the part-B ±3 medium-pole slack analysis. pole_census.py was "
                "extended for this (previously it counted medium-electric-pole only)."
            ),
            "methodology_caveats": post0f["meta"]["methodology_caveats"],
        },
        "part_a_power_warning_footprint": part_a,
        "part_b_slack_distribution": part_b,
        "part_c_pole_totals": part_c,
        "part_d_in_span_fraction": part_d,
        "part_e_fluid_row_budgets": part_e,
        "diagnostics": diag,
        "cases": cases,
    }
    with open(OUT, "w") as fh:
        json.dump(out, fh, indent=1)
    print("wrote", OUT, file=sys.stderr)


if __name__ == "__main__":
    main()
