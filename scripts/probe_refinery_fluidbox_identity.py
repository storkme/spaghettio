# Phase C precondition probe (#411 follow-on): per-fluidbox IDENTITY for
# oil-refinery under advanced-oil-processing. Which port position gets
# which fluid, per the prototype's fluid_boxes order + the recipe's
# ingredient/product order — the raw data our engine's fluid_ports table
# and the mirror-as-rotation export must agree with (FFF #394 class).
from draftsman.data import entities, recipes

e = entities.raw["oil-refinery"]
print("=== oil-refinery fluid_boxes (prototype order) ===")
fbs = e.get("fluid_boxes") or []
for i, fb in enumerate(fbs):
    conns = fb.get("pipe_connections", [])
    io = fb.get("production_type")
    filt = fb.get("filter")
    for c in conns:
        print(f"  box[{i}] {io:8} filter={filt} pos={c.get('position')} dir={c.get('direction')}")

r = recipes.raw["advanced-oil-processing"]
print("\n=== advanced-oil-processing recipe order ===")
print("  ingredients:", [(i.get("name"), i.get("type")) for i in r["ingredients"]])
print("  results:    ", [(p.get("name"), p.get("type")) for p in r["results"]])

r2 = recipes.raw["basic-oil-processing"]
print("\n=== basic-oil-processing (control) ===")
print("  ingredients:", [(i.get("name"), i.get("type")) for i in r2["ingredients"]])
print("  results:    ", [(p.get("name"), p.get("type")) for p in r2["results"]])
