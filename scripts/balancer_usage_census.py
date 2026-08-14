#!/usr/bin/env python3
"""A3 census: which balancer templates does the corpus actually consume?

Decodes every .fls snapshot (4-byte magic + base64(gzip(json))), extracts
BalancerStamped trace events (requested shape per family), then replays
family_stamp_plan's resolution order offline against the library's
registered shapes/widths:
  1. passthrough if n == m (and not demand-skewed — skew isn't traced, so
     n==m shapes are counted under BOTH possible paths, conservatively)
  2. direct template (n, m)
  3. gcd decomposition: largest g >= 2 dividing both, template
     (n/g, m/g) with width <= m/g
  4. runtime generator (self-contained atoms — consumes NO library
     templates; verified in balancer_generate.rs generate())
"""
import base64
import gzip
import json
import re
import sys
from collections import Counter, defaultdict
from glob import glob

SNAP_GLOB = "crates/core/target/tmp/*.fls"
LIB = "crates/core/src/bus/balancer_library.rs"

# --- library registrations: shape -> width ---
src = open(LIB).read()
REG = re.compile(
    r"m\.insert\(\((\d+), (\d+)\), BalancerTemplate \{\s*"
    r"n_inputs: \d+, n_outputs: \d+, width: (\d+), height: (\d+)", re.S)
lib = {(int(a), int(b)): {"width": int(w), "height": int(h)}
       for a, b, w, h in REG.findall(src)}

# --- snapshot events ---
def walk(obj, out):
    # Trace events serialize internally tagged: {"phase": "<Variant>",
    # "data": {...}} — not {"<Variant>": {...}}.
    if isinstance(obj, dict):
        if obj.get("phase") == "BalancerStamped" and isinstance(obj.get("data"), dict):
            out.append(obj["data"])
        for v in obj.values():
            walk(v, out)
    elif isinstance(obj, list):
        for v in obj:
            walk(v, out)

events = []
files = sorted(glob(SNAP_GLOB))
per_fixture = defaultdict(set)
for path in files:
    raw = open(path, "rb").read()
    try:
        doc = json.loads(gzip.decompress(base64.b64decode(raw[4:])))
    except Exception as e:
        print(f"!! failed to decode {path}: {e}", file=sys.stderr)
        continue
    found = []
    walk(doc, found)
    for ev in found:
        events.append(ev)
        per_fixture[path.split("/")[-1]].add(tuple(ev["shape"]))

requested = Counter(tuple(e["shape"]) for e in events)
not_found = Counter(tuple(e["shape"]) for e in events if not e.get("template_found", True))

def resolve(n, m):
    """Replay family_stamp_plan; returns (paths, consumed_templates)."""
    paths, consumed = [], set()
    if n == m and n >= 2:
        paths.append("passthrough(unless-skewed)")
    if (n, m) in lib:
        paths.append("direct")
        consumed.add((n, m))
    for g in range(min(n, m), 1, -1):
        if n % g or m % g:
            continue
        sub = (n // g, m // g)
        if sub in lib and lib[sub]["width"] <= m // g:
            paths.append(f"decomposed(g={g},sub={sub})")
            consumed.add(sub)
            break
    if not paths:
        paths.append("generator-or-unresolvable")
    return paths, consumed

print(f"snapshots decoded: {len(per_fixture)}/{len(files)}; "
      f"BalancerStamped events: {len(events)}")
print(f"\n== requested shapes ({len(requested)} distinct) ==")
consumed_all = set()
for shape, count in sorted(requested.items()):
    n, m = shape
    paths, consumed = resolve(n, m)
    consumed_all |= consumed
    nf = f"  [{not_found[shape]} not-found]" if not_found.get(shape) else ""
    print(f"  ({n},{m}) x{count}{nf}  -> {'; '.join(paths)}")

print(f"\n== library: {len(lib)} registered templates ==")
used = sorted(s for s in lib if s in consumed_all)
unused = sorted(s for s in lib if s not in consumed_all)
print(f"consumed by corpus ({len(used)}): {used}")
print(f"NOT consumed by corpus ({len(unused)}): {unused}")

# For each unused template: what request shapes COULD consume it?
print("\n== unused templates: consumption preconditions ==")
for (a, b) in unused:
    direct = f"direct request ({a},{b})"
    dec = []
    for g in range(2, 5):
        if lib[(a, b)]["width"] <= b:
            dec.append(f"({g*a},{g*b}) via g={g}")
    dec_s = ", ".join(dec) if dec else "none (width guard blocks decomposition)"
    print(f"  ({a},{b}): {direct}; decomposition consumers: {dec_s}")
