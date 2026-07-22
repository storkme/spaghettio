#!/usr/bin/env python3
"""Quick UG-belt span checker.

An underground belt pair's entrance and exit may be at most REACH tiles
apart (center to center), i.e. up to REACH - 1 tiles of covered gap
between them. Reach per tier matches ug_max_reach in
crates/core/src/common.rs: yellow 4, red 6, blue 8.

Usage: ug_span_check.py <gap_tiles> [belt]
Prints which tiers can bridge a gap of that many covered tiles.
"""

import sys

REACH = {
    "transport-belt": 4,
    "fast-transport-belt": 6,
    "express-transport-belt": 8,
}


def can_span(gap_tiles: int, belt: str) -> bool:
    """True if a <belt> UG pair can bridge gap_tiles covered tiles."""
    return gap_tiles <= REACH[belt]


def main() -> None:
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    gap = int(sys.argv[1])
    belts = [sys.argv[2]] if len(sys.argv) > 2 else list(REACH)
    for belt in belts:
        verdict = "ok" if can_span(gap, belt) else "too far"
        print(f"{belt:>24}  gap={gap}  {verdict}")


if __name__ == "__main__":
    main()
