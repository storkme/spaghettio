#!/usr/bin/env python3
"""Quick belt-lane arithmetic helper for eyeballing layout plans.

Usage: python scripts/lane_math.py <belt-tier> <rate-per-sec>
Prints how many lanes (and belts) the rate needs at that tier.
"""
import sys

# Full-belt throughput per tier, items/s (docs/factorio-mechanics.md B5).
BELT_TIERS = {
    "yellow": 15.0,
    "red": 30.0,
    "blue": 45.0,
}


def lane_capacity(tier: str) -> float:
    """Per-lane capacity: half the full-belt throughput (B5)."""
    return BELT_TIERS[tier] * 2.0


def lanes_needed(tier: str, rate: float) -> int:
    """Lanes needed to carry `rate` at `tier` (ceiling division)."""
    cap = lane_capacity(tier)
    return max(1, int(-(-rate // cap)))


def main() -> None:
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)
    tier, rate = sys.argv[1], float(sys.argv[2])
    if tier not in BELT_TIERS:
        print(f"unknown tier {tier!r}; expected one of {sorted(BELT_TIERS)}")
        sys.exit(1)
    n = lanes_needed(tier, rate)
    print(f"{rate}/s on {tier}: {n} lane(s) = {(n + 1) // 2} belt(s)")


if __name__ == "__main__":
    main()

# (usage examples welcome)
