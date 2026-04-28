# Junction SAT, caching, and sub-pattern hinting

A guided tour of how the junction solver works today, why it's expensive,
and the layered set of optimisations the cache + palette system uses to
make it cheap. Read top-to-bottom — each section assumes the previous one.

If you "kind of know" how SAT works but can't quite explain it, the first
two sections (§1, §2) are for you. The rest is project-specific.

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

Have a junction with width `W`, height `H`, `C` channels (one per item
that flows through), some boundary ports, and some forbidden tiles. We
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

See §10 for the actual e2e and cache-file numbers.

---

## 5. Where caching runs out of road

Cache hits are great when we've seen the exact zone before, but every
new layout the user opens in the web app has *some* zones we haven't
cached. Those still cost a full SAT solve + descent.

The cache also doesn't generalise. Two zones can be **structurally
similar** without sharing a signature — different bbox, different
forbidden tiles, different channel count — yet contain the same kind
of local routing primitives. We're paying full SAT cost on each one,
even though large parts of the solution would look identical to
something we've already solved.

This is the gap the **palette + sub-pattern hinting** experiment is
trying to close.

---

## 6. Sub-pattern mining

Look at any junction solution by eye and you'll see it's an assembly of
recurring local primitives:

- A perpendicular crossing where one channel passes under another via
  a UG-in / UG-out pair.
- An L-turn where a single channel changes direction.
- A parallel-belt run for two channels going the same way.
- A "fluid bridge" where one channel jumps over a fluid trunk.

If we can extract these primitives from the cache corpus and recognise
them in fresh problems, we can pre-pin those tiles before SAT runs —
shrinking the search space without changing the encoder or solver.

The miner is dead simple:

1. For each cached record (one solved zone), iterate over every entity
   as a "seed".
2. From each seed, BFS up to `k` tile-adjacency hops, collecting
   neighbouring entities.
3. **Canonicalise** the resulting blob: try all 8 D4 orientations, shift
   to origin, **relabel channels** in encounter order (so two patterns
   that differ only in `chN` indices collapse), pick the lex-min
   representation.
4. Hash + count.
5. Filter by minimum frequency; sort by count.

Output: a **palette** of `SubPattern`s, each tagged with how many
records it appeared in.

Initial reading on 5500 records, `k=2`, `min_freq=3`:

- **607 distinct patterns**, mining time 67ms.
- Top entries are trivial 1×2 belt segments (~3000 hits combined).
- The "perpendicular crossing primitive" — exactly the visual shape
  you'd hand-write — is rank 5 with 145 hits.
- 4×3 UG-bridges show up at rank 13 with 88 hits.

Code: `crates/core/src/junction_palette.rs`, function `mine_palette`.

---

## 7. Recognition and hint injection

For a fresh zone, we want to know: **which palette patterns can fit
inside it?**

For each pattern × position × D4 transform, the recogniser checks:

- Pattern bbox fits inside the zone.
- No pattern entity lands on a forbidden tile.
- No pattern entity lands on a boundary port (those are pinned by the
  encoder; pinning them again would be redundant or contradictory).

It returns a list of **candidate placements**. Many overlap — the same
tile can be claimed by multiple candidates. A **greedy cover** picks a
non-overlapping subset, prioritising higher-frequency patterns and
larger footprints.

Each chosen placement gets converted into **SAT unit clauses** —
literal assignments like "tile (2, 1) must be a fast-transport-belt
facing east" — and added to the encoder's clause set before
`solver.solve()`. The solver propagates from the pins, the search tree
shrinks (often dramatically), and the same SAT problem now solves
faster.

**The risk:** if the recogniser pins an *invalid* placement — one that
contradicts the zone's actual flow constraints — SAT returns UNSAT
where a fresh solve would have succeeded. The mitigation is twofold:

1. **Validity checks**: the recogniser today only checks geometric
   feasibility (in bounds, free tile). It does NOT yet check **flow
   direction consistency** — that the pattern's belt directions match
   the zone's per-channel flow directions. This is the next step before
   real injection lands.
2. **Fallback retry**: if hint-on solve returns UNSAT, drop the hints
   and retry without. The trace event gets `fell_back: true` so we can
   measure the false-positive rate.

Code: `recognise`, `greedy_cover`, `Placement` in `junction_palette.rs`;
the call site is `shadow_emit_hints` in `junction_sat_strategy.rs`.

---

## 8. Shadow mode

The current state of the experiment is **shadow mode**: the recogniser
runs on every junction zone when `FUCKTORIO_USE_SAT_HINTS=1`, computes
the candidate set + greedy cover, emits a `SatHintInjection` trace
event recording what it *would* have pinned — but **does not actually
pin anything**. SAT solves as normal.

This lets us measure coverage and tune the recogniser without any risk
of correctness regressions. Once the trace events show the recogniser
is finding meaningful hints with low false-positive rate, we'll wire
real injection.

Initial coverage measurement:

```
zone   zones  candidates    hints  h/zone
4×4      23       30338      172     7.5
4×5       6       29016       82    13.7
5×3      42       35027      245     5.8
5×4      16       43692      140     8.8
8×4       8       30010      112    14.0
```

For mid-size zones (4×4, 5×3, 5×4) the recogniser pins 5-15 tiles per
zone. Each pinned tile fixes ~10-20 SAT variables, so coverage in the
hundreds of variables per zone is plausible — exactly the regime where
SAT-hint propagation tends to deliver order-of-magnitude speedups.

Run shadow stats with:

```bash
FUCKTORIO_USE_SAT_HINTS=1 cargo test --release --test e2e -- \
    --ignored diag_hint_shadow_stats --exact --nocapture
```

---

## 9. Instrumentation, end to end

The trace pipeline carries everything we need to validate the
experiment. Per-zone events:

- **`SatHintInjection`** — emitted by `shadow_emit_hints` *before* the
  SAT call. Carries `palette_size`, `candidates_considered`,
  `hints_emitted`, `fell_back`. When real injection lands, the same
  event will pair with a `SatInvocation` carrying matching
  `seed_x`/`seed_y` so a diag can correlate hint count with solve time.
- **`SatInvocation`** — already exists; emitted after the solve.
  Carries `solve_time_us`, `satisfied`, `variables`, `clauses`.

Aggregating across a full e2e run gives:

- Coverage histogram by zone size (`diag_hint_shadow_stats`).
- Per-pattern "how often was this picked by greedy cover?"
- Once injection is real: hint-on vs hint-off solve-time deltas, UNSAT
  rate, fall-back rate.

Diag tests:

| diag | what it does |
|---|---|
| `diag_corpus_sweep` | runs ~120 layouts to populate the cache |
| `diag_curated_sweep` | wider sweep (~800 layouts), only commits clean |
| `diag_sat_zone_histogram` | prints cache contents grouped by signature |
| `diag_mine_palette` | mines + prints the palette |
| `diag_hint_shadow_stats` | runs e2e with hints on, aggregates coverage |

Each is `#[ignore]` so they don't run on a normal `cargo test`. All
print to stderr; all are read-only against shared state (the cache
file) except the two sweep diags which write.

---

## 10. Benchmarks

Numbers from the run that landed this doc, on the default e2e corpus
(18 tests, single-threaded, release build).

### E2e wall-clock by mode

| run | wall | notes |
|---|---:|---|
| cache disabled (`FUCKTORIO_USE_ZONE_CACHE=0`) | 22.8s | full SAT every junction, no recording |
| cold cache (record only, no hits) | 23.6s | recording overhead is ~3% |
| **warm cache (default)** | **4.3s** | **5.3× speedup vs disabled; golden hashes byte-identical to fresh SAT** |
| warm cache + shadow hint recogniser (`FUCKTORIO_USE_SAT_HINTS=1`) | 33.9s | recogniser today is O(palette × positions × transforms) per zone — overhead is the cost of shadow observation, not of SAT itself |

The shadow-mode regression isn't fundamental — it's the recogniser's
nested loop running on every solve. A bbox-prefilter pass and per-pattern
inverted-index would cut it back near baseline; that's a tractable
optimisation if the experiment graduates from shadow mode.

### Cache file growth and format compression

The cache went through three formats. Same data, same parity, very
different bytes on disk:

| format | sweep file (119 layouts) | mean record | max record | notes |
|---|---:|---:|---:|---|
| v0 JSONL (verbose) | 1.4 MB | ~600 B | 4964 B | over POSIX `PIPE_BUF` (4096 B) — multi-process append no longer atomic |
| v1 JSONL (short keys, tuple entities) | 505 KB | 231 B | 713 B | 64% smaller than v0; multi-process safe again |
| **binary (current)** | **303 KB** | **140 B** | **389 B** | 78% smaller than v0; max 5× under PIPE_BUF |

The committed embedded blob (`crates/core/data/sat-zones.bin`) covers
the e2e suite + the diag_corpus_sweep at **88 KB / 390 records**.
This is what ships in the WASM bundle.

### Curated sweep — what we actually capture

Wider sweep (`diag_curated_sweep`, 808 recipe × rate × belt × input
combinations, rate steps of 0.5/s):

```
Curated sweep done in 286.1s: 808/808 attempted, 574 clean, 234 dirty, 0 failed
  records: 6556 committed, 24778 discarded
```

71% of layouts come back fully clean (zero errors AND zero warnings).
Of the 31334 SAT solutions produced across the sweep, 21% (6556) ended
up committed to the curated cache. The discarded 79% mostly came from
science recipes and advanced-circuit at high rates, where the bus
router still produces lane-throughput / belt-dead-end warnings.

The curated 6556 records collapsed to **95 distinct signatures** —
strict curation throws out a lot of structurally interesting junctions
that only show up in warning-producing layouts. The non-curated
embedded blob's 390 distinct signatures is the fairer "production
corpus" measure.

### Palette mining

```
Palette miner: k_hops=2, min_freq=3, mined 607 patterns from 5490
records in 67ms.
```

Mining time scales linearly with `records × max_pattern_size²` and
fits comfortably in startup latency for the lazy-init path.

Top 10 most-frequent patterns (palette rank, frequency, size):

| rank | freq | shape |
|---:|---:|---|
| 1 | 1772 | 1×2 vertical red-belt run, single channel |
| 2 | 1128 | 1×2 vertical yellow-belt run, single channel |
| 3 | 400 | 1×2 vertical blue-belt run |
| 4 | 256 | 2×1 parallel yellow belts (two channels side-by-side) |
| 5 | 145 | 3×3 perpendicular crossing with red UG-in (the "useful primitive") |
| 6 | 144 | 2×1 parallel red belts |
| 7 | 141 | 3×3 crossing variant with red UG-out |
| 13 | 88 | 4×3 UG-bridge across parallel flows |
| 21 | 65 | 2×3 four-belt run, single channel |

The trivial 1×2 and 2×1 patterns dominate counts because the long
straight belt sections of every solved zone get re-counted. The meaty
patterns are the 3×3 / 4×3 entries with UG-in/out structure — exactly
the visual primitives we'd hand-write a template for.

### Shadow-mode hint coverage

`diag_hint_shadow_stats` aggregates `SatHintInjection` events from a
4-test fixed corpus. After greedy non-overlapping cover:

```
zone   zones  candidates    hints  h/zone
1×2/3      ~      0           0     0.0   (too small to fit any pattern)
3×3       42       3K-35K   100-245   5-9
4×4       23      30338      172     7.5
4×5        6      29016       82    13.7
5×3       42      35027      245     5.8
5×4       16      43692      140     8.8
8×4        8      30010      112    14.0
```

Mid-size zones get 5-15 tile pins per zone. Each pinned tile fixes
roughly 10-20 SAT variables (kind × direction × channel for that
tile), so the hint coverage is in the 50-300 fixed-vars-per-zone
range — exactly where SAT-hint propagation tends to deliver
order-of-magnitude search-tree reductions.

Whether that translates to wall-clock wins depends on whether the
pins are *correct* — see §7 for the validity-check work that's still
needed before real injection.

### Real injection A/B (cache off, e2e subset, both passes in-process)

This is what happens when we actually pin the recognised hints and
let SAT run with them. Cache is OFF for both passes so every junction
goes through fresh SAT (or hint-injected SAT).

```
                  wall_ms     sat_ms
  hints OFF       1084.5      119.3
  hints ON        1204.4      119.8
  delta            11.1%       0.4%
```

Per case (the stress 60s one is the most informative):

```
  case                    wall_ms   sat_ms  calls  hints   fb
  ab_stress_60s_red_ore     732.6     98.5   548    —      —    (off)
  ab_stress_60s_red_ore_h   948.0     98.2   548   1564   208   (on)
```

Three honest read-outs:

1. **Hints don't reduce SAT time** at all (119.3 → 119.8 ms total). The
   pinned tiles are mostly ones SAT was already figuring out via unit
   propagation in microseconds — pinning them externally doesn't shorten
   the search.
2. **Hints add 11% wall-clock** of pure recogniser overhead. The
   palette × positions × transforms loop is per-zone and the constant
   doesn't amortise.
3. **38% fall-back rate** on the stress case (208 / 548). Roughly two
   in five hint-on solves return UNSAT and have to retry without —
   that's the cost of skipping the flow-direction validity check.

This is the kind of result the experiment was set up to measure, and
it's a "no" today. The recogniser identifies real local primitives but
they don't move the needle: SAT is already fast enough at the small
zone sizes we're hitting (median solve is ~200µs), and the cache
already covers most repeats.

Where it could become useful, in roughly the order I'd try:

1. **Bigger zones** (the long-tail 11×4 / 17×3 records). On those, SAT
   solves take 1-7 ms each, and the cache doesn't have many of them
   yet because they only appear once per unique recipe. Hints might pay
   off here even if they don't on the trivial 4×4 cases.
2. **Tighter validity checks** to drop the 38% fall-back rate. Each
   fall-back is a wasted SAT call; cutting that to <5% would shift the
   wall-clock balance.
3. **Decomposition**, not pinning — recognise large patterns that match
   a sub-rectangle, look that sub-rectangle up in the cache directly,
   stitch results. This is fundamentally different (it skips SAT
   entirely on the matched region) and has much higher potential.

What stays useful from this commit regardless:

- The miner produces an artefact (`palette`) we can eyeball to confirm
  what shapes the engine actually relies on. Useful for engine
  debugging even if hints don't ship.
- The trace event scaffolding lets future experiments swap in
  different recognisers (decomposition, learned policy) without
  re-plumbing instrumentation.

## 11. Where this might go

Updated after the §10 A/B result. The tile-pinning approach didn't
move the needle on our typical workload. Three credible follow-ups in
roughly the order I'd try them:

1. **Decomposition (highest leverage)**: the 11×4 and 17×3 records in
   the cache are visibly assemblies of smaller patterns side-by-side.
   Recognise sub-rectangles whose port topology + forbidden tiles match
   a cached small zone, look those up via the existing cache, stitch
   them together at the seams. This skips SAT entirely on the matched
   region — fundamentally different from pinning, and where caching's
   killer scale story lives.
2. **Tighten the recogniser**: add flow-direction checks and
   per-channel topology matching to drop the 38% fall-back rate on
   real injection. Each fall-back is a wasted SAT call; cutting that
   to <5% would tip the wall-clock balance and might make hints worth
   the recogniser overhead on bigger zones.
3. **Run the A/B against the long-tail (11×4, 17×3) zones only**:
   trivial 4×4 cases are SAT-bound by ~200µs solves where pinning
   can't help. The big zones take 1-7 ms each; that's where pinning
   could actually save time. Need to populate the curated cache more
   widely first to have a meaningful sample.

The ML angle, when we get there, is mostly about which decomposition
to try first — a learned policy on `(zone → best decomposition)` pairs
where the reward is total solve time. That's a future commit.

---

## Cheat sheet

```bash
# Run the full e2e suite (cache enabled by default)
cargo test --manifest-path crates/core/Cargo.toml --release --test e2e -- --test-threads=1

# Disable the cache (e.g. to investigate a stale-cache regression)
FUCKTORIO_USE_ZONE_CACHE=0 cargo test ...

# Wide curated sweep — populate ~/.cache/fucktorio/sat-zones.bin from clean layouts only
FUCKTORIO_USE_ZONE_CACHE=0 cargo test --release --test e2e -- \
    --ignored diag_curated_sweep --exact --nocapture

# Mine the palette and eyeball top-N patterns
cargo test --release --test e2e -- --ignored diag_mine_palette --exact --nocapture
PALETTE_K_HOPS=3 PALETTE_MIN_FREQ=5 cargo test ... --ignored diag_mine_palette ...

# Run the e2e suite with shadow-mode hint recognition
FUCKTORIO_USE_SAT_HINTS=1 cargo test --release --test e2e -- \
    --ignored diag_hint_shadow_stats --exact --nocapture

# Refresh the embedded committed cache
rm ~/.cache/fucktorio/sat-zones.bin
cargo test --release --test e2e -- --ignored diag_curated_sweep --exact
cargo test --release --test e2e -- --test-threads=1
cp ~/.cache/fucktorio/sat-zones.bin crates/core/data/
```

## Where to read more

- **SAT theory**: Knuth TAOCP Volume 4B (Satisfiability). Heavy but
  thorough. For a friendlier intro, *Handbook of Satisfiability* by
  Biere et al. (free PDF online).
- **CDCL**: the original Marques-Silva & Sakallah 1996 paper *GRASP — A
  New Search Algorithm for Satisfiability* is the seminal one and
  surprisingly readable.
- **varisat docs**: <https://docs.rs/varisat> — solid Rust SAT solver,
  small enough to read end-to-end if you're curious.
- **Sub-pattern mining as case-based reasoning**: Aamodt & Plaza 1994,
  *Case-Based Reasoning: Foundational Issues, Methodological Variations,
  and System Approaches* is the field's foundational paper. Modern
  applications include AlphaGo's MCTS rollout policy, which is a much
  more sophisticated version of the same idea.
- **In-codebase**: `docs/ghost-pipeline-contracts.md` for how the bus
  router builds the zones we feed SAT in the first place.
