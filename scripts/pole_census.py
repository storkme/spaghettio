#!/usr/bin/env python3
"""Power re-census (docs/rfp-power-supply.md + docs/rfp-power-reservation.md).

Originally the post-Phase-0f re-census; re-run post-Phase-3b (commit ca8730e)
to re-anchor the trigger-(b) pole-count baseline after 3a-ii's reactive
band-widening and 3b's kovarex substation. Reads the .fls snapshots
(regenerated from the two committed commands — the e2e/stress suite + the
`census_science_pack_snapshots` #[ignore]d test, both with
SPAGHETTIO_DUMP_SNAPSHOTS=1) and computes, per case and corpus-wide:
  A. power warning footprint vs the pinned reds
  B. slack distribution under the two-band placement (MEDIUM poles only)
  C. pole totals per case, medium + substation (Phase 3 trigger-(b) baseline)
  D. in-span fraction
  E. fluid-row free-tile budget inputs

SUBSTATION HANDLING (RFP Phase 3b — a pole TYPE the pre-3b census never saw):
  substations are counted in the part-C pole totals (real_pole_count now =
  medium + substation) and modelled as 2x2 obstacles in the medium-pole slack
  occupancy, but are EXCLUDED from the part-B ±3 slack/densification analysis
  (they have ±9 supply / a 2x2 footprint — a single-tile ±3 slack window is
  meaningless for them). Two corpus cases carry a substation post-3b:
  tier_kovarex_self_loop (top-edge band) and census_utility_science_pack
  (deep-geometry fallback).

METHODOLOGY CAVEATS (read before trusting the numbers):
  - Ground truth pole positions come from the final snapshot only. We do
    NOT replay place_poles' decision order (explicitly forbidden by the
    task -- a prior attempt diverged unexplained). All "slack"/"alternatives"
    figures are therefore order-independent proxies computed against the
    FINAL occupancy (all non-pole entities, plus all OTHER real poles
    treated as occupied single tiles) -- not the true decision-time state
    (which only differs by poles placed *later* in the same greedy sweep).
  - "window" is a census-defined interpretation of place_poles' probe
    shape, not a re-derivation of the engine's own search order. Two
    windows are reported:
      * local_slack: at the pole's own y (the specific ring it landed on),
        x in [px-3, px+3] (7 tiles, POLE_RANGE=3 each way) minus the pole's
        own tile -> up to 6 alternatives. This is the closest analogue to
        the pre-0f single-band probe the RFP's Motivation section measured.
      * band_slack: the same x-window, but summed across every y-ring in
        the pole's *entire* matched band (up to 4 rings: the seed candidate
        row out to POLE_RANGE further outward) -- a fuller picture of how
        much room the two-band outward search actually had, since Phase 0f
        searches multiple rings, not just one.
    Neither is "the" engine-true slack; both are stated explicitly so the
    reader can judge which better answers their question.
  - Row/band assignment: every row's two seed candidate y's
    (common::pole_candidate_ys: top_y-1, top_y+mh) are provably unique
    across rows (rows cannot overlap in y), so the *seed* (d=0) assignment
    of any inserter or pole to a row+band is unambiguous. Outward rings
    (d=1..3) CAN collide with a neighbouring row's territory when the
    inter-row gap is small; ties are broken by preferring the smallest d,
    then smallest |y - top_y|. Collisions are counted and reported.
  - "within-inserter-span" vs "beyond-inserter-span" is defined the same
    way the pre-0f census defined it (census-coined, not an engine fact):
    within/beyond the x-range spanned by REAL inserters sitting exactly on
    that band's seed row (d=0). A pole matched to a band whose seed row has
    zero real inserters at all is classified "no-inserters-on-face".
  - Fluid vs solid row classification comes from the embedded solver
    output (MachineSpec.inputs/outputs .is_fluid), keyed by recipe -- not
    from the internal (unserialized) RowKind enum.
  - Occupancy for slack purposes mirrors bus/layout.rs's own construction
    exactly: machine footprints expanded via MACHINE_DIMS, splitters get
    their second tile via direction, everything else is a single tile.
    Medium poles are modelled as single-tile, which is EXACT: a
    medium-electric-pole is 1x1 in Factorio (`common::entity_size`), so the
    single tile IS its true footprint and it matches place_poles' own internal
    collision check (`placed.insert((px, py))`). (Substations are the only 2x2
    pole; they are modelled at their real 2x2 footprint separately -- see the
    substation handling below.)
"""
import base64
import gzip
import json
import os
import statistics as stats
from collections import defaultdict, Counter

# Snapshot directory: override via SPAGHETTIO_SNAP_DIR; defaults to the
# repo's own dump location (run the suite with SPAGHETTIO_DUMP_SNAPSHOTS=1
# first — see the census decision-log entries in docs/rfp-power-supply.md).
SNAP_DIR = os.environ.get(
    "SPAGHETTIO_SNAP_DIR",
    os.path.join(os.path.dirname(__file__), "..", "crates", "core", "target", "tmp"),
)

MACHINE_DIMS = {
    "assembling-machine-1": (3, 3),
    "assembling-machine-2": (3, 3),
    "assembling-machine-3": (3, 3),
    "chemical-plant": (3, 3),
    "electric-furnace": (3, 3),
    "biochamber": (3, 3),
    "centrifuge": (3, 3),
    "oil-refinery": (5, 5),
    "cryogenic-plant": (5, 5),
    "foundry": (5, 5),
    "electromagnetic-plant": (4, 4),
    "recycler": (2, 4),
}
MACHINE_NAMES = set(MACHINE_DIMS.keys())
INSERTER_NAMES = {"inserter", "long-handed-inserter", "fast-inserter", "stack-inserter"}
SPLITTER_NAMES = {"splitter", "fast-splitter", "express-splitter"}
POLE_RANGE = 3
# Substation (RFP `docs/rfp-power-reservation.md` Phase 3b — kovarex top-edge
# band, plus the USP deep-geometry fallback): a NEW pole TYPE the pre-3b census
# never saw. 2x2 footprint, ±9 supply (18x18) — a fundamentally different beast
# from the medium pole's 1x1 / ±3, so it is COUNTED in the corpus pole totals
# (part C) but deliberately EXCLUDED from the medium-pole ±3 slack/densification
# analysis (part B), where a ±9 substation would be meaningless. Modelled as a
# 2x2 obstacle in the medium-pole slack occupancy (a medium pole cannot land on
# a substation tile).
SUBSTATION_SIZE = 2
NEEDS_ELECTRICITY_MACHINES = MACHINE_NAMES - {"biochamber"}  # biochamber is burner-fueled


def decode_fls(path):
    with open(path, "rb") as f:
        data = f.read()
    assert data[:4] == b"fls1", f"{path}: bad magic {data[:4]!r}"
    raw = gzip.decompress(base64.b64decode(data[4:]))
    return json.loads(raw)


def splitter_second_tile(x, y, direction):
    if direction in ("North", "South"):
        return (x + 1, y)
    return (x, y + 1)


def pole_candidate_ys(top_y, mh):
    ys = []
    if top_y > 0:
        ys.append(top_y - 1)
    ys.append(top_y + mh)
    return ys


def band_ys_for_seed(cy, top_y):
    d = -1 if cy < top_y else 1
    return [cy + d * k for k in range(0, POLE_RANGE + 1) if cy + d * k >= 0]


def build_occupied_base(entities):
    """Mirror bus/layout.rs's occupancy construction, excluding MEDIUM poles.

    Medium poles are handled separately as single-tile obstacles via `pole_set`
    in the slack loop. Substations (RFP Phase 3b) are NOT medium poles, so they
    stay in the occupancy set at their real 2x2 footprint — a medium pole cannot
    land on a substation tile."""
    occ = set()
    for e in entities:
        name = e["name"]
        if name == "medium-electric-pole":
            continue
        if name == "substation":
            for dx in range(SUBSTATION_SIZE):
                for dy in range(SUBSTATION_SIZE):
                    occ.add((e["x"] + dx, e["y"] + dy))
            continue
        if name in MACHINE_NAMES:
            w, h = MACHINE_DIMS[name]
            for dx in range(w):
                for dy in range(h):
                    occ.add((e["x"] + dx, e["y"] + dy))
        else:
            occ.add((e["x"], e["y"]))
            if name in SPLITTER_NAMES:
                occ.add(splitter_second_tile(e["x"], e["y"], e["direction"]))
    return occ


def recipe_fluid_map(solver):
    """recipe -> True if any input/output of any MachineSpec for that recipe is a fluid."""
    m = {}
    if not solver:
        return m
    for spec in solver.get("machines", []):
        recipe = spec.get("recipe")
        if recipe is None:
            continue
        has_fluid = any(f.get("is_fluid") for f in spec.get("inputs", []) + spec.get("outputs", []))
        m[recipe] = m.get(recipe, False) or has_fluid
    return m


def build_rows(entities):
    """Group machine entities into rows keyed by (top_y, mh); mirrors place_poles' by_row."""
    by_row = defaultdict(list)
    for e in entities:
        name = e["name"]
        if name not in MACHINE_NAMES:
            continue
        w, h = MACHINE_DIMS[name]
        cx = e["x"] + w // 2
        by_row[(e["y"], h)].append(
            {"cx": cx, "x": e["x"], "w": w, "h": h, "recipe": e.get("recipe"), "machine": name}
        )
    rows = []
    for (top_y, mh), machines in by_row.items():
        machines.sort(key=lambda m: m["cx"])
        recipes = Counter(m["recipe"] for m in machines)
        recipe = recipes.most_common(1)[0][0] if recipes else None
        heterogeneous = len(recipes) > 1
        machine_name = Counter(m["machine"] for m in machines).most_common(1)[0][0]
        rows.append(
            {
                "top_y": top_y,
                "mh": mh,
                "recipe": recipe,
                "heterogeneous_recipe": heterogeneous,
                "machine": machine_name,
                "cxs": [m["cx"] for m in machines],
                "xs": [m["x"] for m in machines],
                "seed_ys": pole_candidate_ys(top_y, mh),
            }
        )
    return rows


def build_y_owner(rows):
    """seed_y (d=0) -> (row_idx, band_dir) ; provably unique (rows can't overlap in y)."""
    owner = {}
    collisions = []
    for ridx, row in enumerate(rows):
        for band_dir, cy in enumerate(row["seed_ys"]):
            if cy in owner:
                collisions.append((cy, owner[cy], (ridx, band_dir)))
            else:
                owner[cy] = (ridx, band_dir)
    return owner, collisions


def match_y(y, rows):
    """Return list of (row_idx, band_dir, d) for every row/band whose band_ys contains y."""
    matches = []
    for ridx, row in enumerate(rows):
        for band_dir, cy in enumerate(row["seed_ys"]):
            band_ys = band_ys_for_seed(cy, row["top_y"])
            if y in band_ys:
                d = band_ys.index(y)
                matches.append((ridx, band_dir, d))
    return matches


def best_match(matches, rows, y):
    if not matches:
        return None, False
    matches_sorted = sorted(matches, key=lambda m: (m[2], abs(rows[m[0]]["top_y"] - y)))
    best = matches_sorted[0]
    ambiguous = len(matches_sorted) > 1 and matches_sorted[1][2] == best[2]
    return best, ambiguous


def census_one(path):
    snap = decode_fls(path)
    name = os.path.basename(path)[len("snapshot-"):-len(".fls")]
    entities = snap["layout"]["entities"]
    issues = snap["validation"]["issues"]
    solver = snap.get("solver")

    power_issues = [i for i in issues if i["category"] == "power"]
    power_warnings = [i for i in power_issues if i["severity"] == "Warning"]
    power_errors = [i for i in power_issues if i["severity"] == "Error"]
    all_issue_cats = Counter((i["severity"], i["category"]) for i in issues)

    # `poles` = MEDIUM poles only — the subjects of the ±3 slack/densification
    # analysis (parts B/D/E). Substations (RFP Phase 3b) are counted separately
    # (part C pole totals) but are NOT ±3 slack subjects (they have ±9 supply and
    # a 2x2 footprint; a "single-tile slack window" is meaningless for them).
    poles = [(e["x"], e["y"]) for e in entities if e["name"] == "medium-electric-pole"]
    pole_set = set(poles)
    substations = [(e["x"], e["y"]) for e in entities if e["name"] == "substation"]
    inserters = [
        (e["x"], e["y"]) for e in entities if e["name"] in INSERTER_NAMES
    ]
    electric_inserter_count = len(inserters)  # engine only places electric inserter tiers

    occupied_base = build_occupied_base(entities)
    rfmap = recipe_fluid_map(solver)

    rows = build_rows(entities)
    y_owner, y_collisions = build_y_owner(rows)

    # in-band inserter spans (per row, per band): x-range of real inserters
    # sitting exactly on that band's seed row (d=0).
    inserter_span = {}  # (ridx, band_dir) -> (min_x, max_x) or None
    inserter_assignment_off_band = 0
    for (ix, iy) in inserters:
        matches = match_y(iy, rows)
        best, _ambig = best_match(matches, rows, iy)
        if best is None:
            inserter_assignment_off_band += 1
            continue
        ridx, band_dir, d = best
        if d == 0:
            key = (ridx, band_dir)
            lo, hi = inserter_span.get(key, (ix, ix))
            inserter_span[key] = (min(lo, ix), max(hi, ix))

    # classify + measure slack for every real pole
    pole_records = []
    ambiguous_pole_matches = 0
    for (px, py) in poles:
        occ_excl_self = occupied_base | (pole_set - {(px, py)})
        matches = match_y(py, rows)
        best, ambig = best_match(matches, rows, py)
        if ambig:
            ambiguous_pole_matches += 1

        # local_slack: same y, x in [px-3, px+3], excluding self
        local_window = [(x, py) for x in range(px - POLE_RANGE, px + POLE_RANGE + 1)]
        local_free = sum(1 for t in local_window if t != (px, py) and t not in occ_excl_self)
        local_window_size = len(local_window) - 1

        band_free = None
        band_window_size = None
        row_kind_is_fluid = None
        classification = "unmatched-bridge-or-mopup"
        row_recipe = None
        row_machine = None

        if best is not None:
            ridx, band_dir, d = best
            row = rows[ridx]
            row_recipe = row["recipe"]
            row_machine = row["machine"]
            row_kind_is_fluid = rfmap.get(row_recipe, False)
            cy = row["seed_ys"][band_dir]
            band_ys = band_ys_for_seed(cy, row["top_y"])
            band_window = [(x, y) for y in band_ys for x in range(px - POLE_RANGE, px + POLE_RANGE + 1)]
            band_free = sum(1 for t in band_window if t != (px, py) and t not in occ_excl_self)
            band_window_size = len(band_window) - 1

            span = inserter_span.get((ridx, band_dir))
            if span is None:
                classification = "no-inserters-on-face"
            elif span[0] <= px <= span[1]:
                classification = "within-inserter-span"
            else:
                classification = "beyond-inserter-span"

        pole_records.append(
            {
                "x": px,
                "y": py,
                "matched": best is not None,
                "row_recipe": row_recipe,
                "row_machine": row_machine,
                "row_is_fluid": row_kind_is_fluid,
                "band_dir": best[1] if best else None,
                "d": best[2] if best else None,
                "classification": classification,
                "local_alternatives": local_free,
                "local_window_size": local_window_size,
                "band_alternatives": band_free,
                "band_window_size": band_window_size,
            }
        )

    # Part E inputs: per fluid row, free-tile budget within the shared bands.
    fluid_row_budgets = []
    for ridx, row in enumerate(rows):
        is_fluid = rfmap.get(row["recipe"], False)
        if not is_fluid:
            continue
        # Real poles matched (by seed-y ownership) to this specific row.
        poles_here = []
        for (px, py) in poles:
            matches = match_y(py, rows)
            best, _ = best_match(matches, rows, py)
            if best and best[0] == ridx:
                poles_here.append((px, py, best[1], best[2]))
        zero_local = 0
        for (px, py, band_dir, d) in poles_here:
            occ_excl_self = occupied_base | (pole_set - {(px, py)})
            local_window = [(x, py) for x in range(px - POLE_RANGE, px + POLE_RANGE + 1)]
            local_free = sum(1 for t in local_window if t != (px, py) and t not in occ_excl_self)
            if local_free == 0:
                zero_local += 1
        # Whole-band free-tile budget (both bands, all rings, full row
        # x-extent + POLE_RANGE margin either side) -- the quantity Phase
        # 1's reservation formula would need to compare against coverage
        # requirements.
        total_free_both_bands = 0
        total_window_both_bands = 0
        lo_x = min(row["xs"]) - POLE_RANGE
        hi_x = max(x + row["mh"] for x in row["xs"]) + POLE_RANGE  # mh as a same-magnitude width proxy
        occ_row_only = occupied_base  # real poles are "used", not part of the budget question
        for cy in row["seed_ys"]:
            band_ys = band_ys_for_seed(cy, row["top_y"])
            for y in band_ys:
                for x in range(lo_x, hi_x + 1):
                    total_window_both_bands += 1
                    if (x, y) not in occ_row_only and (x, y) not in pole_set:
                        total_free_both_bands += 1

        fluid_row_budgets.append(
            {
                "recipe": row["recipe"],
                "machine": row["machine"],
                "top_y": row["top_y"],
                "mh": row["mh"],
                "n_machines": len(row["cxs"]),
                "real_pole_count": len(poles_here),
                "zero_local_slack_poles": zero_local,
                "free_tiles_both_bands_full_row": total_free_both_bands,
                "window_tiles_both_bands_full_row": total_window_both_bands,
            }
        )

    return {
        "name": name,
        "n_entities": len(entities),
        "width": snap["layout"]["width"],
        "height": snap["layout"]["height"],
        "all_issue_categories": {f"{sev}:{cat}": n for (sev, cat), n in all_issue_cats.items()},
        "power_warning_count": len(power_warnings),
        "power_error_count": len(power_errors),
        "power_messages_sample": [i["message"] for i in power_warnings[:5]],
        # `real_pole_count` = ALL power poles (medium + substation), the part-C
        # corpus total. `medium_pole_count` / `substation_count` break it down;
        # the part-B ±3 slack analysis operates on `medium_pole_count` only.
        "real_pole_count": len(poles) + len(substations),
        "medium_pole_count": len(poles),
        "substation_count": len(substations),
        "electric_inserter_count": electric_inserter_count,
        "n_rows": len(rows),
        "y_collisions": y_collisions,
        "ambiguous_pole_matches": ambiguous_pole_matches,
        "inserter_assignment_off_band": inserter_assignment_off_band,
        "poles": pole_records,
        "fluid_row_budgets": fluid_row_budgets,
        "rows_summary": [
            {"top_y": r["top_y"], "mh": r["mh"], "recipe": r["recipe"], "machine": r["machine"],
             "is_fluid": rfmap.get(r["recipe"], False), "n_machines": len(r["cxs"]),
             "heterogeneous_recipe": r["heterogeneous_recipe"]}
            for r in rows
        ],
    }


def main():
    files = sorted(f for f in os.listdir(SNAP_DIR) if f.startswith("snapshot-") and f.endswith(".fls"))
    # 49 = the 45-case post-0f corpus + 4 `phase0e1_*` fixtures landed with
    # Phase 0e-i (emag/cryo/foundry-molten/biolubricant) AFTER the post-0f
    # census. Bump this deliberately when the corpus grows so a partial/failed
    # snapshot dump is caught rather than silently under-censused.
    assert len(files) == 49, f"expected 49 snapshots, found {len(files)}"
    results = []
    for f in files:
        results.append(census_one(os.path.join(SNAP_DIR, f)))
    return results


if __name__ == "__main__":
    results = main()
    out = {
        "meta": {
            "generated_at": "2026-07-20",
            "commit": "ca8730e",
            "n_snapshots": len(results),
            "method": "post-3b re-census; see module docstring in pole_census.py for full methodology + caveats",
        },
        "cases": results,
    }
    outpath = os.environ.get(
        "SPAGHETTIO_CENSUS_OUT",
        "/tmp/census-raw.json",
    )
    with open(outpath, "w") as fh:
        json.dump(out, fh, indent=1)
    print("wrote", outpath)
