#!/usr/bin/env python3
"""POST-Phase-0f power re-census (docs/rfp-power-supply.md).

Reads the 45 committed .fls snapshots (regenerated from the two committed
commands at commit debb398, post Phase 0f) and computes, per case and
corpus-wide:
  A. power warning footprint vs the pinned Phase-0f-landed reds
  B. slack distribution under the new two-band placement
  C. pole totals per case (Phase 3 trigger-(b) baseline)
  D. in-span fraction
  E. Phase 1 fluid-row free-tile budget inputs

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
    Poles are modelled as single-tile (matches place_poles' own internal
    collision check, `placed.insert((px, py))` -- NOT the pole's real 2x2
    Factorio footprint). This is a deliberate fidelity-to-the-algorithm
    choice, flagged here so it isn't mistaken for "true" occupancy.
"""
import base64
import gzip
import json
import os
import statistics as stats
from collections import defaultdict, Counter

SNAP_DIR = "/home/stork/code/fucktorio/.claude/worktrees/agent-ab1a93f51384ebc0e/crates/core/target/tmp"

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
    """Mirror bus/layout.rs's occupancy construction, excluding poles."""
    occ = set()
    for e in entities:
        name = e["name"]
        if name == "medium-electric-pole":
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

    poles = [(e["x"], e["y"]) for e in entities if e["name"] == "medium-electric-pole"]
    pole_set = set(poles)
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
        "real_pole_count": len(poles),
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
    assert len(files) == 45, f"expected 45 snapshots, found {len(files)}"
    results = []
    for f in files:
        results.append(census_one(os.path.join(SNAP_DIR, f)))
    return results


if __name__ == "__main__":
    results = main()
    out = {
        "meta": {
            "generated_at": "2026-07-19",
            "commit": "debb398",
            "n_snapshots": len(results),
            "method": "post-0f re-census; see module docstring in census.py for full methodology + caveats",
        },
        "cases": results,
    }
    outpath = "/tmp/claude-1000/-home-stork-code-fucktorio/e861777b-2d64-4a65-b2a3-ce5957b45a51/scratchpad/census-raw.json"
    with open(outpath, "w") as fh:
        json.dump(out, fh, indent=1)
    print("wrote", outpath)
