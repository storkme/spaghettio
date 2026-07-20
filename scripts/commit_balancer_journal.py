"""Validate journal entries and atomically merge them into the library.

Phase 2 of docs/rfc-balancer-runner.md. The runner
(``scripts/balancer_runner.py``) writes solves to
``scripts/balancer_journal.jsonl`` but never touches
``src/bus/balancer_library.py``. This script is the only thing that does.

Pipeline per journal entry:
1. Skip if the shape already exists in the library — never overwrite a
   validated template (warn, drop the entry from the post-commit journal).
2. Run the blueprint through ``import_balancer --stdin --json --dry-run``
   (the existing Rust validator). It performs blueprint decode,
   non-belt-entity rejection, port detection, and BFS-based lane
   connectivity — much stronger than anything we could reasonably
   reimplement in Python.
3. Build the same template via the Python pipeline in
   ``scripts/generate_balancer_library.py`` (``extract_entities``,
   ``rotate_cw``, ``normalize``, ``identify_ports``, ``entity_tile``)
   and confirm key fields match the Rust output. Any divergence is a
   serious bug in one of the pipelines and the entry is rejected.
4. Geometric sanity checks: ``len(input_tiles) == n``,
   ``len(output_tiles) == m``, ports on top/bottom rows, output tiles
   have contiguous x-coords (a runtime invariant in
   ``crates/core/src/bus/balancer.rs``), bbox <= SAT grid.

Atomic commit:
- ``fcntl.flock`` on the library file for the lifetime of the script.
- Stale ``balancer_library.py.tmp.*`` files are unlinked on startup.
- New library is written via ``tempfile.NamedTemporaryFile`` next to the
  target, then ``os.replace()``-renamed.
- Journal truncation is a separate atomic step performed *only after*
  the library rename succeeds. A crash between the two leaves the next
  run with already-committed entries in the journal — these are
  recognised as no-ops on the next pass and dropped silently.

Usage:
    uv run python scripts/commit_balancer_journal.py
    uv run python scripts/commit_balancer_journal.py --dry-run
    uv run python scripts/commit_balancer_journal.py --no-sync
"""

from __future__ import annotations

import argparse
import fcntl
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

# Reuse the blueprint-pipeline helpers in the existing generator script.
SCRIPT_DIR = Path(__file__).parent
REPO = SCRIPT_DIR.parent
LIBRARY_PATH = REPO / "src" / "bus" / "balancer_library.py"
JOURNAL_PATH = SCRIPT_DIR / "balancer_journal.jsonl"
SYNC_SCRIPT = SCRIPT_DIR / "sync_balancer_to_rust.py"
CORE_MANIFEST = REPO / "crates" / "core" / "Cargo.toml"
# Cargo workspaces use a single target dir at the workspace root; non-workspace
# crates use crate-local target/. Search both locations.
_IMPORT_BALANCER_CANDIDATES = (
    REPO / "target" / "debug" / "import_balancer",
    REPO / "crates" / "core" / "target" / "debug" / "import_balancer",
)


def import_balancer_bin() -> Path | None:
    for p in _IMPORT_BALANCER_CANDIDATES:
        if p.exists():
            return p
    return None


def _load_generator_module():
    """Import ``scripts/generate_balancer_library.py`` for its pipeline helpers.

    The module is heavyweight (it has top-level ``include_str!``-style data
    references and a ``main`` only invoked via ``__name__``); importing it
    is fine.
    """
    src = SCRIPT_DIR / "generate_balancer_library.py"
    spec = importlib.util.spec_from_file_location("_balancer_gen_helpers", src)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not import {src}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules["_balancer_gen_helpers"] = mod
    spec.loader.exec_module(mod)
    return mod


GEN = _load_generator_module()


# --------------------------------------------------------------------------- #
# Existing-library load                                                       #
# --------------------------------------------------------------------------- #


def load_library_templates() -> dict[tuple[int, int], dict]:
    """Load all templates from ``src/bus/balancer_library.py`` as plain dicts."""
    if not LIBRARY_PATH.exists():
        return {}
    spec = importlib.util.spec_from_file_location("_balancer_lib_commit", LIBRARY_PATH)
    if spec is None or spec.loader is None:
        return {}
    mod = importlib.util.module_from_spec(spec)
    sys.modules["_balancer_lib_commit"] = mod
    spec.loader.exec_module(mod)
    out: dict[tuple[int, int], dict] = {}
    for key, t in mod.BALANCER_TEMPLATES.items():
        out[key] = {
            "n_inputs": t.n_inputs,
            "n_outputs": t.n_outputs,
            "width": t.width,
            "height": t.height,
            "entities": [
                {"name": e.name, "x": e.x, "y": e.y, "direction": e.direction, "io_type": e.io_type}
                for e in t.entities
            ],
            "input_tiles": [list(p) for p in t.input_tiles],
            "output_tiles": [list(p) for p in t.output_tiles],
            "source_blueprint": t.source_blueprint,
        }
    return out


# --------------------------------------------------------------------------- #
# Validation: Rust import_balancer round-trip                                 #
# --------------------------------------------------------------------------- #


def ensure_import_balancer_built() -> Path:
    """Build import_balancer if it isn't there. Returns the binary path."""
    bin_path = import_balancer_bin()
    if bin_path is not None:
        return bin_path
    print("Building import_balancer...", flush=True)
    subprocess.run(
        ["cargo", "build", "--manifest-path", str(CORE_MANIFEST), "--bin", "import_balancer", "--quiet"],
        cwd=str(REPO),
        check=True,
    )
    bin_path = import_balancer_bin()
    if bin_path is None:
        raise RuntimeError(
            f"import_balancer built but binary not found in any of: "
            f"{[str(p) for p in _IMPORT_BALANCER_CANDIDATES]}"
        )
    return bin_path


def rust_validate(bin_path: Path, blueprint: str) -> tuple[bool, str, dict | None]:
    """Run import_balancer --stdin --json --dry-run.

    Returns (ok, message, parsed_json_or_None). ``ok=False`` on any failure
    mode (build error, non-zero exit, JSON parse failure).
    """
    proc = subprocess.run(
        [str(bin_path), "--stdin", "--json", "--dry-run"],
        input=blueprint,
        cwd=str(REPO),
        capture_output=True,
        text=True,
        timeout=60,
    )
    if proc.returncode != 0:
        return False, f"import_balancer rejected blueprint: {proc.stderr.strip() or proc.stdout.strip()}", None

    # The validator prints two informational lines before the JSON dump; find
    # the JSON object by locating the first '{' at column 0 of any line.
    lines = proc.stdout.splitlines()
    json_start = None
    for i, line in enumerate(lines):
        if line.startswith("{"):
            json_start = i
            break
    if json_start is None:
        return False, "import_balancer --json produced no JSON object", None
    try:
        parsed = json.loads("\n".join(lines[json_start:]))
    except json.JSONDecodeError as e:
        return False, f"could not parse import_balancer JSON: {e}", None
    return True, "ok", parsed


# --------------------------------------------------------------------------- #
# Building the .py template via the existing pipeline                         #
# --------------------------------------------------------------------------- #


def build_template_from_blueprint(
    n: int, m: int, blueprint: str, sat_width: int, sat_height: int
) -> tuple[dict | None, str]:
    """Run the blueprint through the Python extract→rotate→normalize pipeline.

    Returns (template_dict_or_None, message).
    """
    try:
        entities = GEN.extract_entities(blueprint)
    except Exception as exc:
        return None, f"extract_entities failed: {exc}"
    if not entities:
        return None, "extract_entities returned no entities"

    rotated = GEN.rotate_cw(entities, sat_height)
    normalized, _, _ = GEN.normalize(rotated)
    inputs, outputs = GEN.identify_ports(normalized)

    if not inputs or not outputs:
        return None, f"identify_ports returned inputs={inputs} outputs={outputs}"

    # bbox from actual entity tiles (post-rotation), mirrors build_template
    # in generate_balancer_library.py:437-449.
    tiles = []
    for e in normalized:
        tx, ty = GEN.entity_tile(e)
        tiles.append((tx, ty))
        if e.name == "splitter":
            if e.direction in (GEN.FACTORIO_NORTH, GEN.FACTORIO_SOUTH):
                tiles.append((tx + 1, ty))
            else:
                tiles.append((tx, ty + 1))
    if not tiles:
        return None, "no entity tiles after normalization"
    tpl_width = max(tx for tx, _ in tiles) + 1
    tpl_height = max(ty for _, ty in tiles) + 1

    template = {
        "n_inputs": n,
        "n_outputs": m,
        "width": tpl_width,
        "height": tpl_height,
        "entities": [
            {
                "name": e.name,
                "x": GEN.entity_tile(e)[0],
                "y": GEN.entity_tile(e)[1],
                "direction": e.direction,
                "io_type": e.io_type,
            }
            for e in normalized
        ],
        "input_tiles": list(inputs),
        "output_tiles": list(outputs),
        "source_blueprint": blueprint,
    }
    return template, "ok"


# --------------------------------------------------------------------------- #
# Geometric sanity checks                                                     #
# --------------------------------------------------------------------------- #


def geometry_checks(
    template: dict, n: int, m: int, sat_width: int, sat_height: int
) -> list[str]:
    """Return a list of error messages. Empty list = template is valid."""
    errs: list[str] = []
    w = template["width"]
    h = template["height"]
    inputs = [tuple(p) for p in template["input_tiles"]]
    outputs = [tuple(p) for p in template["output_tiles"]]

    if w <= 0 or h <= 0:
        errs.append(f"non-positive bbox: {w}W x {h}H")
        return errs
    if w > sat_height:
        errs.append(f"width {w} exceeds sat height {sat_height}")
    if h > sat_width:
        errs.append(f"height {h} exceeds sat width {sat_width}")

    if len(inputs) != n:
        errs.append(f"input_tiles count {len(inputs)} != n_inputs {n}")
    if len(outputs) != m:
        errs.append(f"output_tiles count {len(outputs)} != n_outputs {m}")

    if any(y != 0 for _, y in inputs):
        errs.append(f"input_tiles not all on y=0: {inputs}")
    if any(y != h - 1 for _, y in outputs):
        errs.append(f"output_tiles not all on y={h - 1}: {outputs}")

    # balancer.rs:88 invariant: output_tiles x-coords form a contiguous run.
    if outputs:
        xs = sorted(x for x, _ in outputs)
        for prev, nxt in zip(xs, xs[1:]):
            if nxt - prev not in (0, 1):
                errs.append(f"output_tiles x-coords non-contiguous: {xs}")
                break

    bp = template.get("source_blueprint", "")
    if not isinstance(bp, str) or not bp.startswith("0"):
        errs.append("source_blueprint missing or wrong prefix")

    return errs


def cross_check_with_rust(template: dict, rust_json: dict) -> list[str]:
    """Confirm the Python pipeline and Rust validator agree on structure."""
    errs: list[str] = []
    for field in ("n_inputs", "n_outputs", "width", "height"):
        if template[field] != rust_json.get(field):
            errs.append(
                f"{field} mismatch: python={template[field]} rust={rust_json.get(field)}"
            )
    if len(template["entities"]) != len(rust_json.get("entities", [])):
        errs.append(
            f"entity count mismatch: python={len(template['entities'])} "
            f"rust={len(rust_json.get('entities', []))}"
        )
    if len(template["input_tiles"]) != len(rust_json.get("input_tiles", [])):
        errs.append("input_tiles count mismatch python vs rust")
    if len(template["output_tiles"]) != len(rust_json.get("output_tiles", [])):
        errs.append("output_tiles count mismatch python vs rust")
    return errs


# --------------------------------------------------------------------------- #
# Library writer (atomic)                                                     #
# --------------------------------------------------------------------------- #


def render_library(templates: dict[tuple[int, int], dict]) -> str:
    """Build the full balancer_library.py contents as a string. Mirrors
    ``emit_library`` in generate_balancer_library.py:475-537."""
    lines = [
        '"""Pre-generated N-to-M balancer templates.',
        "",
        "DO NOT EDIT MANUALLY. Regenerate with:",
        "    uv run python scripts/generate_balancer_library.py",
        "",
        "Shapes are oriented for vertical SOUTH flow: inputs at the top",
        "(facing SOUTH), outputs at the bottom (facing SOUTH).",
        '"""',
        "from __future__ import annotations",
        "",
        "from dataclasses import dataclass",
        "",
        "",
        "@dataclass(frozen=True)",
        "class BalancerTemplateEntity:",
        "    name: str",
        "    x: int  # top-left tile (splitters span 2 tiles in their broad axis)",
        "    y: int",
        "    direction: int  # Factorio 1.0 direction (0=N, 2=E, 4=S, 6=W)",
        "    io_type: str | None = None  # 'input'/'output' for underground-belt",
        "",
        "",
        "@dataclass(frozen=True)",
        "class BalancerTemplate:",
        "    n_inputs: int",
        "    n_outputs: int",
        "    width: int",
        "    height: int",
        "    entities: tuple[BalancerTemplateEntity, ...]",
        "    input_tiles: tuple[tuple[int, int], ...]  # (dx, dy) relative",
        "    output_tiles: tuple[tuple[int, int], ...]",
        "    source_blueprint: str  # for debugging / regeneration",
        "",
        "",
        "BALANCER_TEMPLATES: dict[tuple[int, int], BalancerTemplate] = {",
    ]

    for (n, m), t in sorted(templates.items()):
        lines.append(f"    ({n}, {m}): BalancerTemplate(")
        lines.append(f"        n_inputs={t['n_inputs']},")
        lines.append(f"        n_outputs={t['n_outputs']},")
        lines.append(f"        width={t['width']},")
        lines.append(f"        height={t['height']},")
        lines.append("        entities=(")
        for e in t["entities"]:
            io_suffix = f', io_type="{e["io_type"]}"' if e.get("io_type") is not None else ""
            lines.append(
                f'            BalancerTemplateEntity(name="{e["name"]}", '
                f"x={e['x']}, y={e['y']}, direction={e['direction']}{io_suffix}),"
            )
        lines.append("        ),")
        it = ", ".join(f"({x}, {y})" for x, y in t["input_tiles"])
        ot = ", ".join(f"({x}, {y})" for x, y in t["output_tiles"])
        lines.append(f"        input_tiles=({it}{',' if len(t['input_tiles']) == 1 else ''}),")
        lines.append(f"        output_tiles=({ot}{',' if len(t['output_tiles']) == 1 else ''}),")
        lines.append(f'        source_blueprint="{t["source_blueprint"]}",')
        lines.append("    ),")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def cleanup_stale_tmpfiles() -> None:
    """Remove any leftover .py.tmp.* siblings of the library from a prior crash."""
    for sibling in LIBRARY_PATH.parent.iterdir():
        if sibling.name.startswith(LIBRARY_PATH.name + ".tmp."):
            try:
                sibling.unlink()
                print(f"Removed stale tmp file: {sibling.name}")
            except OSError:
                pass


def atomic_write(path: Path, content: str) -> None:
    """Write content to path via a sibling tempfile + os.replace."""
    fd, tmp_name = tempfile.mkstemp(
        prefix=path.name + ".tmp.",
        dir=path.parent,
        text=True,
    )
    try:
        with os.fdopen(fd, "w") as f:
            f.write(content)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp_name, path)
    except Exception:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise


# --------------------------------------------------------------------------- #
# Journal I/O                                                                 #
# --------------------------------------------------------------------------- #


def read_journal() -> list[dict]:
    if not JOURNAL_PATH.exists():
        return []
    out: list[dict] = []
    with open(JOURNAL_PATH) as f:
        for lineno, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"WARN: skipping unparseable journal line {lineno}: {e}", file=sys.stderr)
                continue
            entry["_lineno"] = lineno
            out.append(entry)
    return out


def write_residual_journal(entries: list[dict]) -> None:
    """Replace the journal file with only the still-failing entries."""
    if not entries:
        if JOURNAL_PATH.exists():
            JOURNAL_PATH.unlink()
        return
    lines = []
    for e in entries:
        clean = {k: v for k, v in e.items() if k != "_lineno"}
        lines.append(json.dumps(clean))
    atomic_write(JOURNAL_PATH, "\n".join(lines) + "\n")


# --------------------------------------------------------------------------- #
# Main                                                                         #
# --------------------------------------------------------------------------- #


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--dry-run", action="store_true", help="Validate but don't write anything")
    p.add_argument("--no-sync", action="store_true", help="Skip sync_balancer_to_rust.py at the end")
    args = p.parse_args()

    cleanup_stale_tmpfiles()
    bin_path = ensure_import_balancer_built()

    # Acquire an exclusive lock on the library file so a second commit can't
    # race with us. The lock is released when the file descriptor is closed
    # (i.e., on process exit).
    LIBRARY_PATH.parent.mkdir(parents=True, exist_ok=True)
    if not LIBRARY_PATH.exists():
        # Touch so flock has something to grab.
        LIBRARY_PATH.touch()
    lock_fd = os.open(LIBRARY_PATH, os.O_RDONLY)
    try:
        fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        print(
            f"Another commit script is holding the lock on {LIBRARY_PATH}. Aborting.",
            file=sys.stderr,
        )
        os.close(lock_fd)
        return 1

    try:
        templates = load_library_templates()
        journal_entries = read_journal()
        if not journal_entries:
            print("Journal is empty. Nothing to commit.")
            return 0

        new_templates: dict[tuple[int, int], dict] = {}
        residual: list[dict] = []  # entries that failed validation
        skipped_already_present: list[tuple[int, int]] = []
        skipped_duplicate_in_journal: list[tuple[int, int]] = []

        seen_in_journal: set[tuple[int, int]] = set()

        for entry in journal_entries:
            shape_raw = entry.get("shape")
            if not (isinstance(shape_raw, list) and len(shape_raw) == 2):
                print(
                    f"WARN: journal line {entry['_lineno']} has bad shape={shape_raw}; "
                    "leaving in journal",
                    file=sys.stderr,
                )
                residual.append(entry)
                continue
            shape = (int(shape_raw[0]), int(shape_raw[1]))
            n, m = shape

            # Already in the validated library — drop silently with a note.
            if shape in templates:
                skipped_already_present.append(shape)
                continue

            # Already won by an earlier journal entry this run — drop with a note.
            if shape in seen_in_journal:
                skipped_duplicate_in_journal.append(shape)
                continue

            blueprint = entry.get("blueprint")
            sat_w = entry.get("sat_width")
            sat_h = entry.get("sat_height")
            if not (isinstance(blueprint, str) and isinstance(sat_w, int) and isinstance(sat_h, int)):
                print(
                    f"WARN: journal line {entry['_lineno']} has missing/bad fields; "
                    "leaving in journal",
                    file=sys.stderr,
                )
                residual.append(entry)
                continue

            ok, msg, rust_json = rust_validate(bin_path, blueprint)
            if not ok:
                print(
                    f"REJECT {shape} (line {entry['_lineno']}): {msg}",
                    file=sys.stderr,
                )
                residual.append(entry)
                continue

            tpl, msg = build_template_from_blueprint(n, m, blueprint, sat_w, sat_h)
            if tpl is None:
                print(
                    f"REJECT {shape} (line {entry['_lineno']}): python pipeline: {msg}",
                    file=sys.stderr,
                )
                residual.append(entry)
                continue

            geom_errs = geometry_checks(tpl, n, m, sat_w, sat_h)
            if geom_errs:
                print(
                    f"REJECT {shape} (line {entry['_lineno']}): geometry: "
                    f"{'; '.join(geom_errs)}",
                    file=sys.stderr,
                )
                residual.append(entry)
                continue

            cross_errs = cross_check_with_rust(tpl, rust_json)
            if cross_errs:
                print(
                    f"REJECT {shape} (line {entry['_lineno']}): "
                    f"cross-check: {'; '.join(cross_errs)}",
                    file=sys.stderr,
                )
                residual.append(entry)
                continue

            new_templates[shape] = tpl
            seen_in_journal.add(shape)
            print(f"VALIDATED {shape}: {tpl['width']}W x {tpl['height']}H")

        # ---- Report ---- #
        print()
        print(f"Existing library: {len(templates)} templates")
        print(f"Journal entries:  {len(journal_entries)}")
        print(f"  ✓ validated:           {len(new_templates)} {sorted(new_templates.keys())}")
        print(f"  • already in library:  {len(skipped_already_present)} {sorted(set(skipped_already_present))}")
        print(f"  • dup in journal:      {len(skipped_duplicate_in_journal)} {sorted(set(skipped_duplicate_in_journal))}")
        print(f"  ✗ failed validation:   {len(residual)}")

        if args.dry_run:
            print()
            print("[dry-run] Library NOT written, journal NOT truncated.")
            return 0

        # ---- Write the library ---- #
        if not new_templates:
            # No new entries to commit. Still want to drop already-committed
            # entries from the journal so it doesn't grow forever.
            print()
            print("No new templates to commit.")
            if skipped_already_present or skipped_duplicate_in_journal:
                write_residual_journal(residual)
                print(
                    f"Journal pruned to {len(residual)} entries "
                    "(already-committed and duplicate entries dropped)."
                )
            return 0

        merged = dict(templates)
        merged.update(new_templates)
        rendered = render_library(merged)
        atomic_write(LIBRARY_PATH, rendered)
        print()
        print(f"Wrote {LIBRARY_PATH} ({len(merged)} templates).")

        # ---- Truncate the journal ---- #
        # Only entries that failed validation remain. Already-present and
        # duplicate-in-journal entries are dropped.
        write_residual_journal(residual)
        if residual:
            print(f"Journal kept {len(residual)} failed entries for inspection.")
        else:
            print("Journal cleared.")

        # ---- Optional Rust sync ---- #
        if args.no_sync:
            print()
            print("Skipping sync_balancer_to_rust.py (--no-sync).")
            return 0

        print()
        print("Running sync_balancer_to_rust.py...")
        rc = subprocess.run(["uv", "run", "python", str(SYNC_SCRIPT)], cwd=str(REPO))
        if rc.returncode != 0:
            print(
                f"WARN: sync_balancer_to_rust.py exited {rc.returncode}. "
                "The .py library is up to date but the .rs library may not be. "
                "Re-run sync manually.",
                file=sys.stderr,
            )
            return rc.returncode
        return 0
    finally:
        try:
            fcntl.flock(lock_fd, fcntl.LOCK_UN)
        except OSError:
            pass
        os.close(lock_fd)


if __name__ == "__main__":
    sys.exit(main())
