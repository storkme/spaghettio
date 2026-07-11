"""Extract Factorio recipe and entity data from draftsman into a static JSON file.

This JSON is embedded in the Rust WASM build so the solver can run without draftsman.
Run: uv run python scripts/extract_factorio_data.py
"""

import json
from pathlib import Path

from draftsman.data import entities as _entities
from draftsman.data import recipes as _recipes

# Recipe categories to exclude (not useful for production chains)
EXCLUDED_CATEGORIES = {"recycling", "crushing", "recycling-or-hand-crafting"}

# Machines we care about for crafting speed lookups
MACHINES = [
    "assembling-machine-1",
    "assembling-machine-2",
    "assembling-machine-3",
    "chemical-plant",
    "electric-furnace",
    "oil-refinery",
    "stone-furnace",
    "steel-furnace",
    "foundry",
    "electromagnetic-plant",
    "cryogenic-plant",
    "biochamber",
    "recycler",
    "crusher",
]


def extract_recipes(include_categories: set[str] | None = None) -> dict:
    """Extract recipes.

    Default mode (include_categories=None): all non-excluded recipes, same
    behaviour as before recycling support was added.

    When include_categories is given, the normal EXCLUDED_CATEGORIES filter
    is bypassed and ONLY recipes whose category is in the given set are
    returned. Used by the --recycling-out mode below to pull just the
    recycling-category recipes into a standalone file for surgical append,
    without touching the main (default) extraction path at all.
    """
    recipes = {}
    for name, raw in _recipes.raw.items():
        category = raw.get("category", "crafting")
        if include_categories is not None:
            if category not in include_categories:
                continue
        elif category in EXCLUDED_CATEGORIES:
            continue

        ingredients = []
        for ing in raw.get("ingredients", []):
            ingredients.append(
                {
                    "name": ing["name"],
                    "amount": ing["amount"],
                    "type": ing.get("type", "item"),
                }
            )

        results = []
        for prod in raw.get("results", []):
            entry = {
                "name": prod["name"],
                "amount": prod["amount"],
                "type": prod.get("type", "item"),
            }
            prob = prod.get("probability", 1.0)
            if prob != 1.0:
                entry["probability"] = prob
            results.append(entry)

        recipe = {
            "name": raw["name"],
            "category": category,
            "energy": raw.get("energy_required", 0.5),
            "ingredients": ingredients,
            "results": results,
        }
        recipes[name] = recipe

    return recipes


def extract_machines() -> dict:
    """Extract crafting speeds and fluid box definitions for relevant machines."""
    machines = {}
    for entity_name in MACHINES:
        raw = _entities.raw.get(entity_name)
        if raw is None:
            continue

        entry = {
            "crafting_speed": raw.get("crafting_speed", 1.0),
        }

        # Extract fluid box definitions if present
        fluid_boxes = raw.get("fluid_boxes")
        if fluid_boxes:
            boxes = []
            for fb in fluid_boxes:
                if not isinstance(fb, dict):
                    continue
                connections = []
                for pc in fb.get("pipe_connections", []):
                    conn = {}
                    if "position" in pc:
                        conn["position"] = pc["position"]
                    if "direction" in pc:
                        conn["direction"] = pc["direction"]
                    if conn:
                        connections.append(conn)
                if connections:
                    boxes.append(
                        {
                            "pipe_connections": connections,
                            "production_type": fb.get("production_type", "input"),
                        }
                    )
            if boxes:
                entry["fluid_boxes"] = boxes

        machines[entity_name] = entry

    return machines


# Recycling-category recipes (Fulgora scrap economy): scrap-recycling plus
# the ~309 auto-generated per-item `*-recycling` recipes. NOTE: scrap-recycling
# itself is category "recycling-or-hand-crafting" in the game data, not plain
# "recycling" as its name might suggest — both categories are needed to
# capture the full recycling recipe set. Kept separate from EXCLUDED_CATEGORIES
# (which still excludes both by default for normal solving) — this constant is
# only consulted by --recycling-out below.
RECYCLING_CATEGORIES = {"recycling", "recycling-or-hand-crafting"}


def main():
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--recycling-out",
        help=(
            "Also extract recycling-category recipes (scrap-recycling + "
            "per-item *-recycling recipes) to this standalone JSON path, "
            "for manual/surgical append into recipes.json. Does not modify "
            "the main recipes.json write below."
        ),
    )
    args = parser.parse_args()

    data = {
        "recipes": extract_recipes(),
        "machines": extract_machines(),
    }

    out_path = Path(__file__).parent.parent / "crates" / "core" / "data" / "recipes.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)

    with open(out_path, "w") as f:
        json.dump(data, f, indent=2)

    recipe_count = len(data["recipes"])
    machine_count = len(data["machines"])
    size_kb = out_path.stat().st_size / 1024
    print(f"Extracted {recipe_count} recipes, {machine_count} machines -> {out_path} ({size_kb:.1f} KB)")

    if args.recycling_out:
        recycling_recipes = extract_recipes(include_categories=RECYCLING_CATEGORIES)
        rec_path = Path(args.recycling_out)
        rec_path.parent.mkdir(parents=True, exist_ok=True)
        with open(rec_path, "w") as f:
            json.dump(recycling_recipes, f, indent=2)
        print(f"Extracted {len(recycling_recipes)} recycling recipes -> {rec_path}")


if __name__ == "__main__":
    main()
