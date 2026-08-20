#!/usr/bin/env python3
"""Count blueprints that carry no usable `version`, in a directory of
blueprint JSON files or raw blueprint strings.

Written for #664, whose decision to REFUSE a version-less blueprint on
import rests on the claim that the population is empty in practice. That
claim was originally made against a local blueprint collection which is
gitignored (`scripts/blueprints/`, see .gitignore), so the measurement was
real but not reproducible by a reader — which is what the review caught.
The data cannot be committed; the METHOD can, so here it is.

Usage:
    python3 scripts/count_blueprint_versions.py <dir> [<dir> ...]

Handles three shapes: a factorio.school export wrapping `blueprintString`,
a bare blueprint/blueprint_book JSON, and a raw `0e...` string in a file.
Books are walked recursively so every leaf blueprint is counted.
"""
import base64
import glob
import json
import os
import sys
import zlib


def decode(text):
    """A raw blueprint string -> its JSON, or None if it is not one."""
    text = text.strip()
    if not text or text[0] not in "01":
        return None
    try:
        return json.loads(zlib.decompress(base64.b64decode(text[1:])))
    except Exception:
        return None


def leaves(node, out):
    if not isinstance(node, dict):
        return
    if "blueprint_book" in node:
        for entry in node["blueprint_book"].get("blueprints", []):
            leaves(entry, out)
    elif "blueprint" in node:
        out.append(node["blueprint"])


def usable_version(version):
    """Mirror import_balancer's `direction_scale` predicate exactly.

    A version is usable if it is a non-zero integer. Packed 0 is the ABSENT
    sentinel; everything else states a version, and the importer decodes
    what is stated (>= 2.0 is 16-way, below it 8-way).

    NOT `(version >> 48) != 0`. A #664 review reading suggested tightening
    both this predicate and the importer to require a non-zero MAJOR --
    tried, and wrong in both places. Factorio 0.x packs
    `0 << 48 | minor << 32 | ...`, so 0.15/0.16 blueprints have major 0
    legitimately, and the strict predicate reports 31 of this corpus's 6120
    as unusable when the importer accepts every one of them. The point of
    this script is to mirror the importer, so it mirrors the importer.
    """
    if not isinstance(version, int) or isinstance(version, bool):
        return False
    return version != 0


def main(dirs):
    total = 0
    missing = []
    files = 0
    for d in dirs:
        for path in sorted(glob.glob(os.path.join(d, "**", "*"), recursive=True)):
            if not os.path.isfile(path):
                continue
            try:
                text = open(path, errors="ignore").read()
            except OSError:
                continue
            node = None
            try:
                data = json.loads(text)
                node = decode(data["blueprintString"]) if (
                    isinstance(data, dict) and "blueprintString" in data
                ) else data
            except Exception:
                node = decode(text)
            if node is None:
                continue
            files += 1
            found = []
            leaves(node, found)
            for bp in found:
                total += 1
                if not usable_version(bp.get("version")):
                    missing.append((path, bp.get("label", "<unlabelled>")))

    print(f"files parsed as blueprints: {files}")
    print(f"blueprints found:           {total}")
    print(f"without a usable version:   {len(missing)}")
    for path, label in missing[:20]:
        print(f"  {path}: {label}")
    if len(missing) > 20:
        print(f"  ... and {len(missing) - 20} more")
    return 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1:]))
