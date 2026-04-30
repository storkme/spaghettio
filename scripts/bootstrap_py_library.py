"""Bootstrap src/bus/balancer_library.py from blueprints already in balancer_library.rs.

Reads source_blueprint strings from the Rust library, runs them through the
same extraction pipeline used by generate_balancer_library.py, and writes the
Python library file.  After this, generate_balancer_library.py --skip-existing
will only need to solve the shapes not yet in the Rust library.

Usage:
    uv run python scripts/bootstrap_py_library.py
"""

from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path

REPO = Path(__file__).parent.parent
RUST_LIB = REPO / "crates" / "core" / "src" / "bus" / "balancer_library.rs"
GEN_SCRIPT = REPO / "scripts" / "generate_balancer_library.py"
OUT_PATH = REPO / "src" / "bus" / "balancer_library.py"


def _load_gen_module():
    spec = importlib.util.spec_from_file_location("_gen", GEN_SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["_gen"] = mod
    spec.loader.exec_module(mod)
    return mod


def shapes_and_blueprints() -> dict[tuple[int, int], str]:
    """Extract (n,m) -> source_blueprint from balancer_library.rs."""
    src = RUST_LIB.read_text()
    result = {}
    # Pattern: m.insert((N, M), BalancerTemplate { ... source_blueprint: "..." })
    # The source_blueprint field is a long base64 string in quotes.
    for m in re.finditer(
        r'm\.insert\(\((\d+),\s*(\d+)\),\s*BalancerTemplate\s*\{[^}]*?source_blueprint:\s*"([^"]+)"',
        src,
        re.DOTALL,
    ):
        n, mo, bp = int(m.group(1)), int(m.group(2)), m.group(3)
        result[(n, mo)] = bp
    return result


def main() -> None:
    gen = _load_gen_module()

    blueprints = shapes_and_blueprints()
    print(f"Found {len(blueprints)} shapes in Rust library")

    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)

    templates: dict[tuple[int, int], dict] = {}
    failed = []
    for (n, m), bp in sorted(blueprints.items()):
        try:
            entities = gen.extract_entities(bp)
            # Need height for rotate_cw. Decode the blueprint to get the grid dims.
            decoded = gen.decode_blueprint(bp)
            raw_entities = decoded.get("blueprint", {}).get("entities", [])
            if not raw_entities:
                print(f"  ({n},{m}): no entities in blueprint, skipping")
                failed.append((n, m))
                continue
            ys = []
            for e in raw_entities:
                pos = e.get("position", {})
                ys.append(pos.get("y", 0))
            height = int(max(ys) - min(ys)) + 2  # approximate; rotate_cw uses grid_h
            rotated = gen.rotate_cw(entities, height)
            normalized, _, _ = gen.normalize(rotated)
            inputs, outputs = gen.identify_ports(normalized)
            tiles = []
            for e in normalized:
                tx, ty = gen.entity_tile(e)
                tiles.append((tx, ty))
                if e.name == "splitter":
                    if e.direction in (gen.FACTORIO_NORTH, gen.FACTORIO_SOUTH):
                        tiles.append((tx + 1, ty))
                    else:
                        tiles.append((tx, ty + 1))
            tpl_width = max(tx for tx, _ in tiles) + 1
            tpl_height = max(ty for _, ty in tiles) + 1
            templates[(n, m)] = {
                "n_inputs": n,
                "n_outputs": m,
                "width": tpl_width,
                "height": tpl_height,
                "entities": [
                    {
                        "name": e.name,
                        "x": gen.entity_tile(e)[0],
                        "y": gen.entity_tile(e)[1],
                        "direction": e.direction,
                        "io_type": e.io_type,
                    }
                    for e in normalized
                ],
                "input_tiles": inputs,
                "output_tiles": outputs,
                "source_blueprint": bp,
            }
            print(f"  ({n},{m}): {tpl_width}W x {tpl_height}H, {len(inputs)} inputs, {len(outputs)} outputs")
        except Exception as exc:
            print(f"  ({n},{m}): ERROR — {exc}", file=sys.stderr)
            failed.append((n, m))

    gen.emit_library(templates)
    print(f"\nDone: {len(templates)} shapes written to {OUT_PATH}")
    if failed:
        print(f"Failed ({len(failed)}): {failed}", file=sys.stderr)


if __name__ == "__main__":
    main()
