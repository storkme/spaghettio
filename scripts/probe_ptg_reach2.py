# Probe (#407): locate max_underground_distance inside the UG pipe_connection
# (top-level key is absent) and compare with underground-belt max_distance.
from draftsman.data import entities
import json
e = entities.raw["pipe-to-ground"]
fb = e.get("fluid_box", {})
for c in fb.get("pipe_connections", []):
    print(json.dumps(c))
