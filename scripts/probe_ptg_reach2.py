from draftsman.data import entities
import json
e = entities.raw["pipe-to-ground"]
fb = e.get("fluid_box", {})
for c in fb.get("pipe_connections", []):
    print(json.dumps(c))
