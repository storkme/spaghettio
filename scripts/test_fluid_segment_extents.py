#!/usr/bin/env python3
import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import fluid_segment_extents as fluid


def entity(x, y, name="pipe", direction=0):
    return [x, y, name, direction, []]


class FluidSegmentExtentsTests(unittest.TestCase):
    def component_count(self, entities):
        at, adj = fluid.build(entities)
        return len(fluid.components(at, adj))

    def test_plain_pipe_cannot_connect_to_closed_ptg_side(self):
        self.assertEqual(
            self.component_count([
                entity(0, 0),
                entity(1, 0, "pipe-to-ground", 0),
            ]),
            2,
        )

    def test_opposite_ptgs_pair_past_surface_pipe(self):
        self.assertEqual(
            self.component_count([
                entity(0, 0, "pipe-to-ground", 4),
                entity(-2, 0),
                entity(-5, 0, "pipe-to-ground", 12),
            ]),
            2,
        )

    def test_pump_splits_two_pipe_segments(self):
        self.assertEqual(
            self.component_count([
                entity(0, 0),
                entity(1, 0, "pump", 4),
                entity(2, 0),
            ]),
            2,
        )

    def test_storage_tank_connects_at_3x3_footprint_edge(self):
        self.assertEqual(
            self.component_count([
                entity(0, 0, "storage-tank"),
                entity(2, 0),
                entity(-2, 0, "pipe-to-ground", 4),
            ]),
            1,
        )

    def test_unknown_pipe_class_entity_fails_loudly(self):
        with self.assertRaisesRegex(ValueError, "unsupported"):
            fluid.build([entity(0, 0, "future-fluid-widget")])


if __name__ == "__main__":
    unittest.main()
