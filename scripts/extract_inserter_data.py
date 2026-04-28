"""Pull inserter prototype data from draftsman: rotation speed, reach,
extension speed, and which variants exist in vanilla Factorio 2.0 /
Space Age. Used to verify rules I3-I8 in docs/factorio-mechanics.md.
"""

from draftsman.data import entities
import json


# Vanilla inserter prototypes we care about. Vanilla 2.0 dropped
# stack-inserter as a base entity (replaced by bulk-inserter); both
# names are checked.
INSERTERS = [
    "burner-inserter",
    "inserter",
    "long-handed-inserter",
    "fast-inserter",
    "filter-inserter",
    "stack-inserter",
    "stack-filter-inserter",
    "bulk-inserter",
    "bulk-filter-inserter",
]

print("=== Vanilla inserter prototypes that exist in entities.raw ===\n")
for name in INSERTERS:
    if name in entities.raw:
        e = entities.raw[name]
        print(f"  {name}: present")
    else:
        print(f"  {name}: ABSENT")

print("\n=== Detailed prototype data ===\n")
for name in INSERTERS:
    if name not in entities.raw:
        continue
    e = entities.raw[name]
    rot = e.get("rotation_speed", "-")
    ext = e.get("extension_speed", "-")
    pickup_pos = e.get("pickup_position", "-")
    insert_pos = e.get("insert_position", "-")
    stack = e.get("stack_size_bonus", "-")
    print(f"  {name}:")
    print(f"    rotation_speed   = {rot}")
    print(f"    extension_speed  = {ext}")
    print(f"    pickup_position  = {pickup_pos}")
    print(f"    insert_position  = {insert_pos}")
    print(f"    stack_size_bonus = {stack}")
    # Reach derivation: pickup_position is (dx, dy) from inserter
    # centre. abs(dy) gives the tile-distance along the facing axis;
    # 1.0 for short-arm, 2.0 for long-arm.
    if isinstance(pickup_pos, list) and len(pickup_pos) == 2:
        reach = abs(pickup_pos[1]) if abs(pickup_pos[1]) > abs(pickup_pos[0]) else abs(pickup_pos[0])
        print(f"    derived reach    = {reach}")
    print()


# Throughput math, per FFF / Wube wiki:
#   throughput = (rotation_speed * 60) / 360 * 2  swings per second
#                (60 ticks/s, 360 deg/swing, 2 swings = full cycle pickup+drop)
#              = rotation_speed * 60 / 180
#              = rotation_speed / 3 ticks/deg ... actually
# rotation_speed is given in deg-per-tick; one half-cycle (180 deg) is
# rotation_speed^-1 * 180 ticks. Two half-cycles = one full pickup+drop
# = 2 * 180 / rotation_speed ticks. Divide 60 ticks/s by that to get
# swings/s.
# Net: items_per_second = 60 * rotation_speed / (2 * 180) = rotation_speed / 6.
print("=== Computed throughput (no research, base) ===\n")
for name in INSERTERS:
    if name not in entities.raw:
        continue
    e = entities.raw[name]
    rot = e.get("rotation_speed")
    if rot is None:
        continue
    # From wiki formula: items/s = rotation_speed * 60 / (2 * 180) = rot / 6
    # for default-size stacks (1 item per swing for non-stack).
    swings_per_sec = rot * 60.0 / (2 * 180.0)
    stack_bonus = e.get("stack_size_bonus", 0)
    base_stack = 1
    if "stack" in name or "bulk" in name:
        base_stack = 12  # stack/bulk inserters carry 12 default
    items_per_sec = swings_per_sec * (base_stack + stack_bonus)
    print(f"  {name}: rot={rot} → ~{swings_per_sec:.2f} swings/s × {base_stack + stack_bonus} items = ~{items_per_sec:.2f} items/s")
