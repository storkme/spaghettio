"""Verify pipe-to-ground direction/type semantics from draftsman."""

from draftsman.classes.blueprint import Blueprint
from draftsman.classes.entity import Entity
from draftsman.constants import Direction
import json


# The pipe-to-ground prototype defines pipe_connections:
#   - position [0,0], direction 0 (NORTH) — surface side
#   - position [0,0], connection_type "underground", direction 8 (SOUTH)
#
# In modern Factorio, the blueprint format also includes a "type" field
# on pipe-to-ground that flips input/output. The question:
# When we serialize {direction: South (8), type: "input"}, which side is
# the surface mouth?

# Construct via raw blueprint dict (because draftsman's PipeToGround class
# may strip the type field).
bp_dict = {
    "blueprint": {
        "icons": [],
        "entities": [
            # input dir=South at (0, 0)
            {
                "entity_number": 1,
                "name": "pipe-to-ground",
                "position": {"x": 0.5, "y": 0.5},
                "direction": 8,
                "type": "input",
            },
            # paired output dir=North at (0, 1)
            {
                "entity_number": 2,
                "name": "pipe-to-ground",
                "position": {"x": 0.5, "y": 1.5},
                "direction": 0,
                "type": "output",
            },
            # surface pipe at (0, -1) — north of input, "behind" input direction
            {
                "entity_number": 3,
                "name": "pipe",
                "position": {"x": 0.5, "y": -0.5},
            },
            # surface pipe at (0, 2) — south of output, "behind" output direction
            {
                "entity_number": 4,
                "name": "pipe",
                "position": {"x": 0.5, "y": 2.5},
            },
        ],
        "item": "blueprint",
        "version": 562949955518464,
        "label": "ptg-test",
    }
}

bp = Blueprint(bp_dict)
print("=== Blueprint loaded ===")
for e in bp.entities:
    print(f"  {e.name} at ({e.global_position}) dir={e.direction}", end="")
    if hasattr(e, 'type') and getattr(e, 'type', None) is not None:
        print(f" type={e.type}", end="")
    print()

print()
print("=== Try to find connection points ===")
for e in bp.entities:
    if e.name == "pipe-to-ground":
        # Inspect the entity's connection points
        print(f"\nEntity: {e.name} at {e.global_position}, direction={e.direction}")
        if hasattr(e, 'fluid_connections'):
            print(f"  fluid_connections: {e.fluid_connections}")
        # All attributes
        for attr in dir(e):
            if 'connection' in attr.lower() or 'pipe' in attr.lower() or 'surface' in attr.lower() or 'underground' in attr.lower():
                if not attr.startswith('_'):
                    try:
                        val = getattr(e, attr)
                        if not callable(val):
                            print(f"  {attr} = {val}")
                    except Exception as ex:
                        pass

# Also try get_world_bounding_box and any pipe-tracker logic
print("\n=== Round-trip JSON ===")
out = bp.to_dict()
for e in out["blueprint"]["entities"]:
    print(json.dumps(e, default=str))
