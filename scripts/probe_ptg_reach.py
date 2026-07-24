from draftsman.data import entities
e = entities.raw["pipe-to-ground"]
print("max_underground_distance =", e.get("max_underground_distance"))
b = entities.raw["underground-belt"]
print("belt max_distance =", b.get("max_distance"))
