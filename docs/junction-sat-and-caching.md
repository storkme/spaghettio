# Junction SAT and caching

A guided tour of how the junction solver works today, why it's expensive,
and the layered set of optimisations the cache uses to make it cheap.
Read top-to-bottom — each section assumes the previous one.

If you "kind of know" how SAT works but can't quite explain it, the
first two sections are for you. The rest is project-specific.

---

## 1. The problem in one paragraph

A bus layout produces **junction zones**: small rectangles where two or
more belt flows meet and need to weave past each other without colliding.
Each zone has fixed entry/exit ports on its boundary (one per channel,
one per direction), some interior tiles that are off-limits (machines,
poles, other entities), and we need to fill the rest with belts and
underground belts so every input port routes to its matching output port.
The constraints stack up fast: belts have direction, undergrounds come
in pairs with a tier-dependent maximum reach, no two entities can share
a tile, and every channel's flow has to be conserved end-to-end. Our
junction sizes are tiny by SAT standards — typically 3×3 to 11×4 — but
even at that scale the search space is combinatorial, and we run the
solver many times per layout.

---

## 2. SAT in five minutes

**SAT** (Boolean satisfiability) is the question: given a propositional
formula, is there an assignment of `true`/`false` to its variables that
makes the formula evaluate to `true`?

The formula is presented in **CNF** — Conjunctive Normal Form — a big
AND of ORs of literals. Each OR is called a **clause**, each individual
"variable or its negation" is a **literal**:

```
(a ∨ ¬b ∨ c) ∧ (¬a ∨ d) ∧ (b ∨ ¬c ∨ ¬d)
└────────────┘   └──────┘   └─────────────┘
   clause 1       clause 2      clause 3
```

A **satisfying assignment** is one that makes every clause true at the
same time. A SAT solver returns:

- `SAT` + a satisfying assignment, OR
- `UNSAT` if no such assignment exists.

The naive algorithm is "try every combination" — `2^n` for `n` variables,
which is hopeless past ~30 vars. Modern solvers (we use **varisat**)
beat this by:

1. **Unit propagation**: if a clause becomes `(x)` because every other
   literal is already false, then `x` must be true. This cascades.
2. **Conflict-driven clause learning** (CDCL): when a guess leads to a
   contradiction, derive a new clause that captures *why*, add it to
   the formula, and backtrack. Future guesses can't repeat the mistake.
3. **Branching heuristics**: pick which variable to guess next based on
   which one will simplify the formula the most.

In practice, modern SAT solvers handle problems with millions of
variables and clauses in seconds, *as long as the problem has structure
the propagation can exploit*. Our junction problems do — that's why SAT
works for us, even though theoretically it's NP-complete.

The trick to using SAT is **encoding**: turn your real problem into a
CNF formula such that satisfying assignments correspond to solutions
you actually want.

---

## 3. Encoding a junction as SAT

For a junction with width `W`, height `H`, `C` channels (one per item
that flows through), some boundary ports, and some forbidden tiles, we
want SAT to choose what entity sits on each interior tile.

Per tile `(x, y)`, the encoder allocates Boolean variables for:

- **Kind** — is this tile empty, a surface belt, a UG-in, or a UG-out?
  (Sometimes encoded as multiple bits.)
- **Direction** — N / E / S / W flow direction.
- **Channel** — which of the `C` flows is using this tile?

For a 5×3 zone with 2 channels, that's roughly `5 × 3 × (kind + dir +
channel) ≈ 200-500 boolean variables`. Then the encoder adds **clauses**
that say things like:

- "Each tile has exactly one kind" (a few clauses per tile).
- "If tile `t` is a belt facing east on channel `c`, then tile `t+(1,0)`
  must accept incoming east-flow on channel `c` — meaning it's another
  east-belt, an east-UG-in, or an output port."
- "UG-in must pair with a UG-out at distance ≤ tier-reach in the same
  direction, with empty tiles between them."
- "Boundary ports are pinned: tile at `(0, 1)` is an east-belt on
  channel 0."
- "Forbidden tiles must be empty."

The output is a CNF formula with a few hundred variables and a few
thousand clauses. varisat solves it in microseconds for trivial zones,
maybe a few milliseconds for awkward 8×4 ones, and (rarely) hundreds of
milliseconds when the problem is near-UNSAT — the solver has to exhaust
many branches before either finding a solution or proving impossibility.

When SAT returns `SAT`, the encoder walks the satisfying assignment and
emits one `PlacedEntity` per non-empty tile. These get pruned (dangling
belts removed) and stamped into the layout.

After the first solve, we run **cost descent**: re-solve with an
extra clause "total cost ≤ C - 1" until UNSAT, which proves the current
layout is optimal at minimum-tile-count. This is what's expensive — each
descent step is another full SAT call.

Code: see `crates/core/src/sat.rs` for the encoder and
`crates/core/src/bus/junction_sat_strategy.rs` for the per-zone caller
that orchestrates the descent loop.

---

## 4. Why caching works: the canonical signature

Two zones with the same `(W, H, port topology, forbidden tiles, channel
reach, UG cap)` produce the **same SAT problem**, so they have the same
solution. Most layouts re-solve the same handful of zone shapes hundreds
of times — different recipes, different rates, but the same junction
geometry comes out of the bus router.

The cache exploits this with a **canonical signature**: a deterministic
string that's the same for two zones iff their SAT problems are
identical. Format:

```
{W}x{H}:{channels}|F:{forbidden}|UG:{cap}
```

- **`channels`** — per-channel tuples `{ins}>{outs}@{reach}`, e.g.
  `N1+S2>E0@5`. Channels are sorted, so labelling is invariant.
- **`forbidden`** — sorted local `(x, y)` interior obstacle tiles.
- **`cap`** — `max_ug_ins`, or `*` for unlimited.

The signature is computed under all 8 **D4 dihedral symmetries** (4
rotations × 2 reflections); we keep the **lexicographically smallest**
one. So a 5×3 zone and the same shape rotated 90° to 3×5 collapse to
the same cache key — there's no benefit to re-solving a zone that's just
a flip of one we've seen.

When SAT solves a zone for the first time, we record:

- the canonical signature,
- the solution entities in **canonical-frame local coords** (apply the
  same D4 transform that produced the signature, so the entities sit in
  the canonical orientation),
- per-canonical-position items (so we can reverse-map `carries` on
  replay),
- the orientation `(rotation, reflect)` that won.

On a cache hit, we apply the inverse D4 transform to the cached
entities to put them back in the new zone's frame, and remap `carries`
tokens (`ch0`, `ch1`, ...) to the new zone's items.

Code: `crates/core/src/zone_cache.rs`. The on-disk format is a custom
binary file (`sat-zones.bin`); each record is length-prefixed, well
under POSIX `PIPE_BUF` so concurrent appends are atomic. Native loads
from `~/.cache/fucktorio/sat-zones.bin`; WASM loads a pre-baked blob
embedded via `include_bytes!()`.

---

## 5. Benchmarks

Numbers from the e2e suite (28 tests, single-threaded, release build):

| run | wall | notes |
|---|---:|---|
| cache disabled (`FUCKTORIO_USE_ZONE_CACHE=0`) | ~50s | full SAT every junction, no recording |
| cold cache (record only, no hits) | ~50s | recording overhead is negligible |
| **warm cache (default)** | **~5s** | **~10× speedup vs disabled; golden hashes byte-identical to fresh SAT** |

Cache file growth — the binary format is tightly packed:

| metric | value | notes |
|---|---:|---|
| committed `crates/core/data/sat-zones.bin` | ~88 KB / 390 records | what ships in the WASM bundle |
| max record size | ~389 B | comfortably under POSIX `PIPE_BUF` (4096 B); concurrent appends atomic |
| mean record size | ~140 B | most are simple 1×N or 3×3 patterns |

Curated wide sweep (`diag_curated_sweep`, ~800 combos at 0.5/s steps,
cache disabled so every junction goes through fresh SAT):

```
574 clean / 234 dirty / 0 failed
6556 records committed (clean), 24778 discarded (dirty)
```

71% of layouts come back fully clean (zero errors AND zero warnings).
Of the 31334 SAT solutions produced across the sweep, 21% (6556) ended
up in the curated cache. The discarded 79% mostly came from science
recipes and advanced-circuit at high rates, where the bus router still
produces lane-throughput / belt-dead-end warnings.

Decomposition viability (geometric, `diag_decomposition_potential`):

```
Large zones (w>=5 or h>=5): 45 unique shapes
  41 (91%) have at least one cut where both halves' sizes
  also appear in the cache.
Total candidate cuts: 122 vertical + 35 horizontal
```

Strong upper bound — almost every big cached zone could in principle be
sliced into cached small pieces.

The stricter probe (`diag_decomposition_signature_match`) checks whether
the implied sub-zone signatures — including boundary topology, forbidden
tiles, and per-channel reach — actually appear in the cache. Result on
the same 46 large shapes:

```
with at least one CLEAN cut:    13/46 (28%)
with at least one MATCHING cut:  0/46 (0%)
```

A "clean cut" is one where no UG corridor would be sliced and no
original boundary lands on a corner of either sub-zone. 28% of large
zones have at least one such cut. **Zero** of those cuts produce
sub-signatures that appear in the cache.

Interpretation: the bus router never emits "half a junction" shapes,
so synthetic-cut sub-zones don't match anything organically cached.
For decomposition to be a real strategy, you'd need either:

1. **A synthetic-fragment populator** that enumerates plausible
   sub-shapes (likely-cuttable boundaries, no UG corridors) and SAT-
   solves each one, seeding the cache with fragments the bus router
   wouldn't produce on its own.
2. **A different decomposition strategy** that doesn't require exact
   signature match — e.g., recognise that a residual sub-rectangle
   has the same channel topology as a cached shape modulo extra
   forbidden tiles, and use the cached solution as a seed for a
   smaller fresh SAT solve.

Either is a non-trivial chunk of work. For now: decomposition is parked
behind "the geometric structure is there but the corpus shape isn't",
not behind a missing technique. The signature-match probe stays in
`tests/e2e.rs` so you can re-run it as the corpus grows.

---

## 6. Cheat sheet

```bash
# Run the full e2e suite (cache enabled by default)
cargo test --manifest-path crates/core/Cargo.toml --release --test e2e -- --test-threads=1

# Disable the cache (e.g. to investigate a stale-cache regression)
FUCKTORIO_USE_ZONE_CACHE=0 cargo test ...

# Wide curated sweep — only keeps records from clean (zero errors AND
# zero warnings) layouts. Ignored test, run on demand:
FUCKTORIO_USE_ZONE_CACHE=0 cargo test --release --test e2e -- \
    --ignored diag_curated_sweep --exact --nocapture

# Decomposition geometric potential
cargo test --release --test e2e -- \
    --ignored diag_decomposition_potential --exact --nocapture

# Per-signature cache contents histogram
cargo test --release --test e2e -- \
    --ignored diag_sat_zone_histogram --exact --nocapture

# Refresh the embedded committed cache
rm ~/.cache/fucktorio/sat-zones.bin
cargo test --release --test e2e -- --ignored diag_curated_sweep --exact
cargo test --release --test e2e -- --test-threads=1
cp ~/.cache/fucktorio/sat-zones.bin crates/core/data/
```

## Where to read more

- **SAT theory**: Knuth TAOCP Volume 4B (Satisfiability). For a
  friendlier intro, *Handbook of Satisfiability* by Biere et al.
  (free PDF online).
- **CDCL**: the Marques-Silva & Sakallah 1996 paper *GRASP — A New
  Search Algorithm for Satisfiability* is the seminal one.
- **varisat docs**: <https://docs.rs/varisat> — solid Rust SAT solver,
  small enough to read end-to-end if you're curious.
- **In-codebase**: `docs/ghost-pipeline-contracts.md` for how the bus
  router builds the zones we feed SAT in the first place.
