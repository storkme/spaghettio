"""Extract inserter-capacity research effects from game data (RFC-048 spike).

Answers authoritatively, from prototypes rather than wiki prose:
(a) per-level hand-size bonuses for non-bulk / bulk / stack inserters,
(b) whether transport-belt-capacity research grants inserter capacity too.
"""
from draftsman.data import technologies

def dump(tech_name_prefixes):
    hits = []
    for name, tech in sorted(technologies.raw.items()):
        if not any(name.startswith(p) for p in tech_name_prefixes):
            continue
        effects = tech.get("effects", [])
        rel = [e for e in effects if "inserter" in e.get("type", "") or "belt" in e.get("type", "")]
        if rel:
            hits.append((name, rel))
    return hits

for name, effects in dump(["inserter-capacity", "stack-inserter", "transport-belt-capacity", "turbo-transport", "bulk-inserter"]):
    print(name)
    for e in effects:
        print("   ", e)
