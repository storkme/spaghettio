# Balancer theory — constructing an (N, M) belt balancer

**Type**: Reference (evergreen). No decision log, no kill criteria.

**Question this document answers**: given arbitrary `N` and `M`, how do you
build a correct `(N, M)` belt balancer, and what can our code do about it
today?

**Short answer, stated up front so nobody reads 600 lines for it**: a
general construction *does* exist and is proved below (§5.5, the Clos
sandwich), but it is **not** in Raynquist's book, and it is not the
compact one. The book gives a general `(1, M)` / `(N, 1)` construction, a
set of composition and rewriting rules, and 184 hand-and-SAT-made
artifacts — it does *not* contain a closed-form general `(N, M)`
construction, and it does not claim to. Compact balancers for arbitrary
`(N, M)` remain a search problem. See §10 for exactly what is open.

---

## 1. Scope and sources

### 1.1 Primary source and its limits

Raynquist's **"balancer book (fall 2025)"** —
<https://factoriobin.com/post/cgn0od> — a Factorio 2.0 blueprint book,
184 leaf blueprints plus an FAQ and a "How do I make my own balancers?"
sub-book.

Be aware of what the book actually contains in prose. Its two most
foundational nodes:

> **Basic balancing theory** — "see: https://www.reddit.com/gallery/jqfhlu"
>
> **Basic lane balancing theory** — "see: https://www.reddit.com/gallery/jyxv5w"

Both are bare links to **image galleries**, unreachable from this
environment and not textual in any case. So the theory in §2–§5 below is
reconstructed from four things, and each claim says which:

1. the book's FAQ prose (quoted, with the node title);
2. the book's *Advanced techniques* nodes, which *are* prose (quoted);
3. **measurement of the decoded book** — every entity of all 184 leaves
   was decoded and censused for this document (splitter/belt/UG counts,
   bounding boxes, priority and filter annotations); geometry claims
   below are from that census, and are marked *(measured, book)*;
4. standard flow-network results (Menger / max-flow-min-cut), marked as
   such.

Where the book and our implementation disagree, §3.2 and §9 say so
explicitly rather than smoothing it over.

### 1.2 Our code

| File | Role |
|------|------|
| `crates/core/src/bus/balancer_library.rs` | 64 baked templates keyed `(n_inputs, n_outputs)` (count at `51fea377`, the merge-base of this document; it was 77 at `39a587f2`, before the #632 A3 defective-template cull); `template_provenance()` |
| `crates/core/src/bus/balancer_generate.rs` | `generate(n, m)` runtime generator; `merge_tree(n)` |
| `crates/core/src/bus/balancer_topology.rs` | `SplitterGraph` combinators: `parallel`, `series_permuted`, `clos_interleave` |
| `crates/core/src/bus/balancer_classify.rs` | `classify_ref()` → the MX class lattice |
| `crates/core/src/bus/template_validate.rs` | `validate_template_lanes()`, `check_throughput_unlimited()` |
| `crates/core/src/bus/balancer.rs` | `family_stamp_plan()` — the shape→stamp decision |
| `crates/core/tests/balancer_lane_audit.rs` | `audit_min_cut_capacity` — the CI waist gate |
| `crates/balancer-gen/src/main.rs` | `bake_missing_shapes()` and its `Recipe` grammar |

All *counts* below are from this document's merge-base, `51fea377`
(registry = 64). A separate PR (#664) imports FOUR book shapes —
`(3,2) (7,2) (7,3) (7,4)` — raising the registry to 68. It is discussed
in §9.5 because it is the cheapest resolution of the fan-in holes and it
changes the recommendations there.

`(8,6)` is deliberately NOT among them: it failed two existing library
gates, so importing it would have meant weakening them. An earlier draft
of this document said five shapes and 69, written against a working tree
rather than a merged state — the number was wrong before it was ever
true, which is worth noting in a document that is otherwise about
trusting counts.

### 1.3 Notation

`(N, M)` means **N inputs, M outputs**, in that order, matching
`balancer_templates()`'s key and `LaneFamily::shape`. Beware: the book
labels shapes the same way ("3-2 TU balancer" = 3 in, 2 out), but
`balancer_generate::generate(m, n)` names its parameters the other way
round (`m` = inputs) — a naming trap, not a semantic one.

- `rated = min(N, M)` — the maximum belts/second the shape can pass,
  in units of full belts.
- **fan-out** = `N < M`; **fan-in** = `N > M`; **square** = `N == M`.
- `S ⊆ inputs`, `T ⊆ outputs` denote *utilized* subsets: `S` is the set
  of inputs actually supplying items, `T` the set of outputs actually
  draining.

---

## 2. The model: what a splitter is

A Factorio splitter is a **2-tile-wide, 2-in / 2-out flow device**:

- **S1 (equal output).** When both outputs can accept, it emits equally
  to both.
- **S2 (equal input).** When both inputs are supplying more than it can
  pass, it draws equally from both.
- **S3 (back-pressure).** When one output is blocked, everything goes to
  the other. When one input is dry, everything comes from the other.
- **S4 (lanes are independent).** Left lanes are balanced against left
  lanes, right against right — the two lanes never mix. From the book's
  FAQ, *"Don't splitters already balance lanes?"*: "They do not.
  Splitters balance left lanes with left lanes … and right lanes with
  right lanes. As a result belt balancers also balance left and right
  lanes separately."
- **S5 (a dangling output is not an output).** A splitter whose left
  output leads nowhere behaves as a 2→1 merger, because S3 routes
  everything to the live side once the dead side backs up. This is the
  trick behind `two_to_one()` (`balancer_generate.rs:221`) and every
  merge stage in the library.
- **S6 (priorities and filters).** `input_priority` / `output_priority`
  override S1/S2 on one side. A splitter with an output **filter** set
  to an item that never appears on the belt is a hard 2→1: the book's
  FAQ, *"What's with the deconstruction-planner filter?"*, spells this
  out — "Setting the filter to deconstruction planner means only
  deconstruction planners can output on that side … and the splitter
  functions as a 2-1 splitter as intended."

**Modelling caveat that runs through everything below.** Both the theory
and our classifier work in a *fluid* model: rates, not items. The book
is explicit that item-level mixing is not guaranteed (FAQ, *"Why is the
balancer not mixing the items?"*): "Splitters only care about the amount
of items going through them. As long as the amount of items are correct,
the splitter doesn't guarantee that it'll produce any particular item
mixing pattern." So a "balanced" verdict is a statement about flow
rates, never about the physical interleaving on the belt.

---

## 3. The three properties

"Correct balancer" is three independent properties. A construction can
have any subset.

### 3.1 Balance

**Output-balanced**: with all `N` inputs saturated and all `M` outputs
unblocked, every output carries `total_in / M`.

**Input-balanced**: with all inputs saturated and outputs restricted so
the network is the bottleneck, every input is drawn from equally.

The book's *"Complete the square (cont.)"* states the link between them,
and states its precondition:

> "ABCDE only describes output balance, however because the 5-5 consists
> of entirely 2-2 splitters, if one side is balanced then the other side
> must also be balanced."

That is the **duality principle**, and the "entirely 2-2 splitters"
clause is load-bearing: it fails the moment a splitter has a dangling
output (S5) or an unused input.

Our classifier's `BalancerClass::Balanced` (MX3) is *stronger* than
either: it requires the full composition matrix to be uniform — every
output receives exactly `1/M` **of each individual input**
(`balancer_classify.rs:324-334`). For a homogeneous bus that extra
strength is free and unused; for a mixed belt it is the whole point.

### 3.2 Throughput — the subset lattice, and where our names diverge

Define, for utilized subsets `S` and `T`:

> **P(S, T)**: with exactly the inputs in `S` supplying and exactly the
> outputs in `T` draining, the network passes `min(|S|, |T|)` belts.

By max-flow/min-cut this is exactly `maxflow(S → T) = min(|S|, |T|)` on
the splitter graph with unit-capacity edges — which is what our
classifier computes (Edmonds–Karp, `balancer_classify.rs:684-757`).

Three tiers matter:

| Tier | Requirement | Book's name |
|------|-------------|-------------|
| A | `P(S, all)` for every `S` | half of "regular" |
| B | `P(all, T)` for every `T` | the other half of "regular" |
| C | `P(S, T)` for **every pair** | **TU** |

The book's FAQ, *"What does TU mean?"*, defines the boundary precisely:

> "Throughput-unlimited (TU) balancers always provide full throughput.
> Regular n-n balancers are only guaranteed to provide full throughput
> when all inputs **or** all outputs are utilized. Regular n-m balancers
> are only guaranteed to provide full throughput when all belts on the
> **larger side** are utilized."

Read carefully, that says: *regular n-n* = tiers A **and** B; *regular
n-m* = only the tier belonging to the larger side; *TU* = tier C.

Now map that onto our classes (`balancer_classify.rs:52-68`):

| Our class | What `classify_graph` actually tests | Book equivalent |
|-----------|--------------------------------------|-----------------|
| `ThroughputLimited` (MX1) | tier A fails | worse than regular |
| `ThroughputBalancedRate` (MX2a) | tier A holds, tier B fails | regular **fan-out** (`N < M`) |
| `ThroughputUnlimited` (MX2b) | tiers A and B hold, composition not uniform | regular **n-n** |
| `Balanced` (MX3) | composition uniform (checked first) | balanced, throughput unstated |

**Three consequences worth internalising.**

1. **`ThroughputUnlimited` does not mean TU.** Our MX2b is tiers A+B;
   the book's TU is tier C, which is strictly stronger and which
   `classify_graph` never tests. Concretely: a graph made of `g`
   disjoint `(k, k)` blocks (`k ≥ 2`) passes tier A — every `S` reaches
   `min(|S|, M)` outputs through its own blocks — and passes tier B by
   the same argument, so it is reported `ThroughputUnlimited`; yet
   `maxflow(S → T) = 0` when `S` and `T` sit in different blocks, and
   output `j` never sees input `i` at all. This is not hypothetical: it
   is exactly what `passthrough(m)` is, and
   `balancer_topology.rs:253` pins it as `ThroughputUnlimited`
   deliberately. (Derived from reading `classify_graph`; not separately
   executed.)

2. **For fan-in shapes our ladder tests the wrong half first.** For
   `N > M` the book's regular guarantee is tier **B** (all inputs
   utilized, the larger side). `classify_graph` bails to
   `ThroughputLimited` the moment tier A fails
   (`balancer_classify.rs:336-343`), so a fan-in network that is a
   perfectly good regular balancer by the book's definition can be
   rejected by our self-verify gate at `balancer_generate.rs:109-115`.
   No *library* shape is known to trip this, but any new fan-in
   generator will meet it.

3. **MX2b does not imply balanced, and MX3's implication of MX2b is
   model-dependent.** `docs/factorio-mechanics.md` §"Belt merger
   taxonomy" says "MX3 strictly subsumes MX2b". As a statement about
   *sets of networks under the fluid model* that is fine; as intuition
   it misleads, because the classifier reports the strongest class it
   can prove, so a `ThroughputUnlimited` verdict means "and **not**
   balanced". Superposing `k` unit flows to certify tier A can also put
   more than one belt on an internal edge, which the linear composition
   model permits and physics does not.

### 3.3 Lane balance

Splitters balance left-with-left and right-with-right (S4), so a belt
balancer leaves the two lanes independent. A **lane balancer** adds
machinery that swaps lanes so the left and right halves of every belt
are also equalised. The book's FAQ, *"Why do the lane balancers only
balance half the lanes?"*:

> "The sideloading parts may resemble output lane balancers, but because
> they sideload onto undergrounds instead of belts, they don't actually
> do any balancing. Their purpose is to **swap** the left and right
> lanes; balancing is done by the other splitters."

And the reason the naive fix fails (FAQ, *"Why not just use a splitter
then sideload the 2 belts onto 1 belt to balance lanes?"*):

> "You can use this if you just want output balancing, but if you want
> input balancing this does not work … when belts are backed up the Left
> and Right lanes merely swap places. This is because sideloading
> prioritizes the lane in the back instead of using the two lanes
> evenly."

This is the same mechanism as the repo's standing rule that sideloading
onto an underground input loads only the far lane. Lane balance is the
project's **MX5** concern and is out of scope for the constructions
below — everything in §5 produces belt balancers, not lane balancers.
`(1,1)` exists in the book *only* as a lane balancer; at belt level a
`(1,1)` "balancer" is a piece of belt, which is why
`generate(1, 1)` returning `None` (`balancer_generate.rs:80`) is correct
and the `(1,1)` "hole" in our library is not a hole.

### 3.4 What the verifiers test

The book names three (*"Balancer verifiers"*) and is candid about them:

> "github.com/d4rkc0d3r/FactorioSimulation … github.com/tzwaan/factorio_balancers
> … github.com/alegnani/verifactory … I only have experience with the
> first one and I know that one can give false positives so it's not
> always right. Don't know about the correctness of the other two."

Our four instruments and what each is worth:

| Instrument | Tests | Blind to |
|------------|-------|----------|
| `classify_ref` (`balancer_classify.rs:123`) | composition uniformity; tiers A and B | tier C; splitter priorities and filters (`recover_graph` ignores them); lanes |
| `validate_template_lanes` (`template_validate.rs:60`) | UG pairing, UG sideload, lane throughput on a synthesised standalone stamp | belt-level topology; noisy on templates with recirculation (see the RFC's Phase-1 triage) |
| `check_throughput_unlimited` (`template_validate.rs:241`) | uniform output at `k = 1, ⌊N/2⌋, N-1` active inputs | only 3 of the `2^N` subsets; warns, never blocks |
| `audit_min_cut_capacity` (`balancer_lane_audit.rs:618`) | every horizontal row cut carries ≥ `rated` forward capacity | one-sided — it counts capacity *potentials*, so it can never report a false waist and can miss a real cap |

None of them is ground truth. The one-sided ones are the useful ones:
`audit_min_cut_capacity` refutes cheaply and clears nothing.

---

## 4. Invariants any construction must satisfy

These are necessary conditions. Check a candidate against them before
running anything expensive.

1. **`rated = min(N, M)`.** No more can pass; a design that delivers
   `rated` is optimal in throughput terms.

2. **Width ≥ N** for a top-fed, bottom-emitting template: the `N` input
   belts occupy `N` distinct columns on the entry row. Measured across
   the library at `HEAD`: no template has `width < n_inputs`, and all 26
   fan-in templates have `width > n_outputs`. This has a sharp
   consequence for our code — see §9.2.

3. **Min-cut ≥ rated at every horizontal cut.** For every pair of
   adjacent rows, the forward-flowing capacity crossing that boundary
   must be at least `rated`. This is the *waist* invariant, enforced in
   CI by `audit_min_cut_capacity`. §6 is about it.

4. **Every input must reach every output** (for balance). Disjoint
   blocks satisfy 1–3 and are still not balancers (§3.2 consequence 1).

---

## 5. Construction techniques

### 5.1 The two atoms

- **`(1, 2)`** — one splitter, both outputs live. Balanced, and TU
  (tier C): with one output blocked, S3 sends everything to the other,
  giving `min(1,1) = 1`.
- **`(2, 1)`** — one splitter, one output dangling (S5). Balanced, and
  TU: `min(2,1) = 1` delivered whichever input supplies.
- **`(2, 2)`** — one splitter, everything live. Balanced and TU. This
  is the universal mixer; it is what the book means by "entirely 2-2
  splitters".

Everything else is these three wired together.

### 5.2 The general `(1, M)`: loopback trees

For `M = 2^k`, a binary fan-out tree gives `(1, M)` in `M − 1`
splitters. *(measured, book)*: 1-4 TU = 3 splitters, 1-8 TU = 7.

For arbitrary `M`, take the smallest binary tree with `L = 2^⌈log₂ M⌉`
leaves and **loop the surplus `L − M` leaves back into the root's second
input**. Solve for the steady state: let `r` be the total returned rate.
The root passes `1 + r`; each leaf carries `(1 + r)/L`; the looped-back
leaves carry `r` in total, so

```
r = (L − M)·(1 + r)/L   ⟹   r = (L − M)/M
```

and each live leaf carries `(1 + r)/L = (L/M)/L = 1/M`. **Exactly
balanced, for any `M`, with `L − 1` splitters** — at most `2M − 2`,
i.e. linear.

Check against the book *(measured)*: `(1,3)` predicts `L=4`, 3 splitters
— book's 1-3 TU has 3. `(1,5)` predicts `L=8`, 7 — book has 7. `(1,7)`
predicts 7 — book has 7.

Two refinements:

- **Factorise first.** For composite `M`, chaining fan-outs is usually
  cheaper than the loopback tree. `(1,6)` = `(1,3)` then three `(1,2)` =
  3 + 3 = **6** splitters, versus 7 for the `L=8` tree — and the book
  ships 6. `(1,10)` = `(1,5)` then five `(1,2)` = 12, versus 15 — and
  that is literally our library's recipe
  (`balancer-gen/src/main.rs`, `Recipe { shape: (1,10), stage1: Lib(1,5),
  stage2: Parallel(1,2,5) }`). Order matters: `(1,2)` then two `(1,5)`
  costs 15.
- **Input priority is required for TU.** The book's *"Input priority for
  TU 1-n"*:

  > "Intuitively one would think that 1-n balancers are all TU. However
  > when loopbacks are involved the loopbacks can compete with inputs for
  > throughput. This can be fixed by prioritizing the input over the
  > loopback. Usually setting input priorities would unbalance the input,
  > but 1-n only has one input belt so it's always balanced with itself."

  So: set `input_priority` on the root splitter to the *external* side.
  Our `detect_priority_needed()` (`balancer_classify.rs:229`) computes
  exactly this recommendation from the graph — it finds splitters whose
  input is reachable from their own output and names the non-feedback
  port.

### 5.3 The general `(N, 1)`: merge trees are **not** balancers

A chained merge — merge inputs 0 and 1, merge the result with input 2,
and so on — delivers all the flow but draws **unequally**: under
saturation the last input supplies `1/2` of the output, the one before
`1/4`, and so on. That is a *merge*, not a balancer.

This is what `merge_tree(n)` (`balancer_generate.rs:350`) builds, and its
docstring is honest about it: it is used for merge-taps onto a trunk,
where every input is the same item and only aggregate rate matters. Its
`n − 1` splitters and freedom from any arithmetic precondition ("merging
is associative") are exactly why it works as an unconditional fallback —
and exactly why it must not be mistaken for a balancer.

A **balanced** `(N, 1)` is the flow-reverse of a `(1, N)` fan-out
(§5.4): same structure, same splitter count. *(measured, book)*: the
`(1,M)` and `(M,1)` splitter counts are identical across the whole book —
3/3, 3/3, 7/7, 6/6, 7/7, 7/7, 11/11 for `M = 3…9`.

### 5.4 Flow reversal — the transpose, and what it does and does not buy

**At the graph level the transpose is free.** Reverse every edge of a
`SplitterGraph` and you get a valid graph for the transposed shape: a
splitter reversed is a splitter (S1↔S2 by symmetry), a 1-in/2-out
splitter reverses to 2-in/1-out (S5), and max-flow is invariant under
edge reversal, so tier A of `G` is tier B of `Gᵀ` and vice versa. MX2b is
therefore self-dual; MX2a maps to its mirror. Balance maps to balance by
the book's duality principle (§3.1) when all splitters are full 2-2.

**At the layout level it is not free**, and the repo's standing rule
about this is correct: a 180° rotation of an `(N, M)` layout is *not* an
`(M, N)` layout. Flow reversal is not a rigid motion — it is "negate
every belt direction in place", which additionally requires swapping
every UG `io_type` and every `input_priority` ↔ `output_priority`, and
which **breaks on sideloads**: a perpendicular belt merging into a main
belt is a merge; reversed, it would have to be a sideways split, which
belts cannot do.

*(measured, book)*: the reflection-across-the-flow-axis test — mirror
`y`, keep directions, swap UG io — reproduces `4-2` from `2-4`, `4-2 TU`
from `2-4 TU`, and `4-1 TU` from `1-4 TU` **exactly, entity for entity**.
It fails on every sideload-bearing pair (`2-8`/`8-2`, `5-8`/`8-5`,
`7-8`/`8-7`, `9-8`/`8-9`, …) even though those pairs have *identical
entity counts and bounding boxes* — strong evidence that Raynquist
derives one from the other logically and re-lays out the physical form.

**So**: the transpose halves the *topology* search, never the
*placement* search. In a graph→place pipeline (which is what
`balancer_topology.rs` + `balancer-gen`'s CP-SAT placer is) that is a
real, usable 2×.

### 5.5 The Clos sandwich — a general `(N, M)`, with proof

This is the construction that makes "arbitrary `(N, M)`" answerable. It
is the book's *"Upsize then balance"* generalised:

> "n-m TU balancers are typically made by connecting regular n-m to a
> regular m-m. However in some cases it may be more efficient to multiply
> each of the n belts until there are a total of m belts, then balance
> them. For TU 2-4, multiply each input belt with TU 1-2, then balance
> two pairs of belts using TU 2-2. Similarly, for TU 2-8, multiply each
> input belt with TU 1-4, then balance four pairs of belts using TU 2-2."

**Construction.** For any `N ≥ 1`, `M ≥ 1`:

```
stage 1:  N parallel (1, M) fan-outs           →  N·M belts
junction: clos_interleave(N, M):  belt (t·M + j)  ↦  slot (j·N + t)
stage 2:  M parallel balanced (N, 1) mergers   →  M outputs
```

Fan-out tree `t` sends one belt to merger `j` for every `j`; merger `j`
receives one belt from every input tree.

**Balance.** Belt `(t, j)` carries `i_t / M`. Merger `j` is balanced, so
output `j` = `Σ_t i_t / M`. Every output is the same uniform `1/M`
combination of every input — MX3 by construction, for **any** `N, M`,
with no divisibility precondition.

> **This is MX3 in the classifier's fluid model, and for fan-in
> (`N > M`) that is not the same as physical MX3.** With every input
> saturated the expression above gives `N/M > 1` belts per output, which
> one belt cannot carry; the stage-2 `(N, 1)` merger saturates each
> output at 1 instead. The *proportions* are right — every output is the
> same mixture — but the *rate* is capped by belt physics the linear
> model does not represent. This is §3.2's consequence 3 ("superposing
> `k` unit flows … physics does not permit") reappearing, and it is the
> mechanism behind the culled `(8,6)`: MX3 verdict, waisted in practice.
> The fan-in rows of the table below — `(3,2) (7,2) (7,3) (8,6)` — are
> exactly the shapes where the sandwich's balance is model-only.

**Throughput (tier C, true TU).** For any `S`, `T`: each `(t, j)` pair is
joined by exactly one unit-capacity path, so the middle is a complete
bipartite `S × T` of unit edges. Max-flow over it is `min(|S|, |T|)`,
bounded above by the input edges (`|S|`) and the output edges (`|T|`),
and achieved by a matching. So `P(S, T)` holds for every pair — the
sandwich is TU in the book's strong sense, not merely tiers A+B.

**No waist.** The middle is `N·M` belts wide, and `N·M ≥ min(N, M)` for
all `N, M ≥ 1`. Every cut inside a stage is a cut of a `(1, M)` or
`(N, 1)` sub-balancer, whose own rated is 1 ≤ rated of the whole. §6.

**Cost.** With loopback trees for the stages:

```
splitters  =  N·(2^⌈log₂ M⌉ − 1)  +  M·(2^⌈log₂ N⌉ − 1)
middle width = N·M
```

| Shape | Sandwich splitters | Book's shipped design *(measured)* | Overhead |
|-------|-------------------:|-----------------------------------:|---------:|
| `(3,2)` | 3·1 + 2·3 = **9** | 4 (3-2 TU) | 2.3× |
| `(7,2)` | 7·1 + 2·7 = **21** | 8 (7-2) | 2.6× |
| `(7,3)` | 7·3 + 3·7 = **42** | 12 (7-3) | 3.5× |
| `(8,6)` | 8·7 + 6·7 = **98** | 18 (8-6) | 5.4× |
| `(1,11)` | 1·15 + 11·0 = **15** | — (not in the book) | — |

So the sandwich is the **correct fallback**, not the design of choice.
Its value is that it has no preconditions: it works for primes, for
coprime pairs, for anything.

**Practical caveats.** Both stages need priority annotations to be TU in
the game rather than merely in the fluid model — the fan-outs need
input priority against their loopbacks (§5.2), and the mergers need
priority so partial input does not dribble into dead-end splitter
outputs (this is precisely the failure mode
`check_throughput_unlimited`'s docstring warns about,
`template_validate.rs:236-240`). And the junction is an `N·M`-wire
permutation, which is where the physical cost actually lands (§9.3).

### 5.6 Divisor-factorised Clos — the cheap path when it applies

The full `N·M` middle is only necessary when nothing factorises. When
`d | M`, the same argument works with a narrower middle:

```
stage 1:  N parallel (1, d) fan-outs        →  N·d belts
junction: clos_interleave(N, d)
stage 2:  d parallel (N, M/d) balancers     →  M outputs
```

and dually when `d | N`. This is exactly the pattern our bake recipes
already use — `Recipe { shape: (4,9), stage1: Parallel(1,3,4),
stage2: Parallel(4,3,3), perm: Clos(4,3) }` is `d = 3` on `M = 9`. Note
that it *recurses*: it needs an `(N, M/d)` balancer, so it terminates on
either a library atom or the full sandwich.

**The permutation does not affect balance.** If stage 2's blocks are
genuine MX3 balancers, output `j` = `(Σ of its inputs)/(its M/d)`
regardless of which stage-1 belt lands where, because each stage-1 belt
carries `i_t/d` and every block receives the same multiset of rates
under any interleave. The interleave matters for **tier-C throughput**
(a bad permutation leaves `S`–`T` pairs unmatched) and for the physical
junction cost — not for balance. This corrects a natural intuition worth
naming, because it means an identity-junction recipe can be a valid
*balancer* while failing to be TU.

### 5.7 Making a construction TU: two copies back to back

The book's *"Basic TU balancing theory"*:

> "TU balancers are typically made by taking regular versions of the
> balancers and using two copies of them back-to-back. Recall that
> regular balancers guarantee full throughput if the inputs or outputs
> are fully utilized. In a TU balancer the first balancer always has its
> outputs fully utilized by the second balancer, and the second balancer
> always has its inputs fully utilized by the first balancer. Therefore
> the combination always provides full throughput."

In the vocabulary of §3.2: stage 1 is invoked with `T = all` (tier A),
stage 2 with `S = all` (tier B). Compose and you get tier C.

And for unequal shapes (*"Basic n-m TU balancing theory"*):

> "They're typically made by connecting a regular n-m with a regular m-m,
> assuming m > n. If n > m then connect a regular n-n with a regular n-m."

Redundant splitters can then be removed. The FAQ, *"Why does the 4-4
have two extra splitters at the end?"*, walks it: "If you take two
regular 4-4's and combine them, you'd get a TU 4-4. But two of the
splitters are redundant, so they can be removed."

*(measured, book)*: the shipped 4-4 TU has **6** splitters — two plain
4-4s would be 4 + 4 = 8, minus the 2 redundant. Its structure decoded is
`2 + 1` splitters, a crossover, then `1 + 2` — the asymmetry left behind
by the removal. The 8-8 TU has 20 = 12 + 12 − 4, the same pattern one
size up.

**Deferred loopback** is a companion optimisation (book, *"Deferred
loopback"*): "If a TU balancer is constructed by combining two plain
balancers that have top level loopbacks (a 3-3 for example), those
loopbacks can instead be done after the two balancers are combined."

### 5.8 The substitution family — how the book actually *finds* designs

These are rewriting rules on an existing balancer. They do not construct
from nothing; they are how you get from a correct-but-large design to a
correct-and-small one. Each carries a validity condition; the conditions
are the interesting part.

**Emergent sub-balancer substitution.**
> "'Unintentional' sub-balancers found in balancers can be replaced with
> equivalent sub-balancers. E.g. if you take two 3-3s and combine them
> into a 6-6, you may find a new 4-4 inside it. You can replace it with
> another 4-4 (in this case, changing the input belt pairings) and the
> balancer will still work. **In TU balancers, only TU sub-balancers can
> be substituted.**"

*Validity*: the sub-graph you replace must be an `(a, b)` balancer as a
unit — every edge crossing the boundary is a port — and the replacement
must match the *class* required by the context, not just the shape.

**Sub-tree merge.**
> "1-5 is typically constructed using two loopbacks. The two loopbacks
> are usually from the same branch, but they don't have to be. If we
> ignore one of the splitters, the two loopback splitters can be merged
> into one loopback splitter. The belts neighboring the loopbacks are
> also merged, and can be split 1-3 to make a new 1-5."

*Validity*: two loopback paths carrying equal rates can share one
splitter. This is the trick that gets `(1,9)` down to 11 splitters from
the 12 that a naive `(1,3)`→3×`(1,3)` chain costs *(measured, book)*.

**Loopback substitution.**
> "Looking at the outputs, the three output splitters not only output the
> five outputs, but they also output one loopback. Because the loopback
> comes from an output splitter, it is also balanced with the five
> outputs. This means that any of the five outputs can serve as the
> loopback instead, possibly leading to a more convenient layout."

*Validity*: the loopback source must be *balanced with* the outputs —
i.e. it carries `1/M` like they do. Then it is interchangeable with any
of them. This is a pure placement optimisation: same graph up to
isomorphism, different geometry.

**Equal belt substitution** generalises it:
> "Equivalent belts in general are interchangeable, not just
> inputs/outputs … The three output splitters are all 1-2, so their
> inputs are also balanced with each other."

*Validity*: two belts carrying provably equal rates from provably equal
provenance may be swapped. Our composition matrix computes exactly this
equivalence relation — two belts are interchangeable when their rows in
the composition matrix are equal.

**Re-balance substitution.**
> "A 1-9 can be made by combining four 1-3's. Observe that eight of the
> outputs come from two internal belts. So we can just use a 2-8 instead.
> The 2-8 doesn't function exactly the same as what was there before; in
> fact **it disturbs the input balance. But since we only have one input
> belt, the input is always balanced**, so it's still a valid 1-9."

*Validity*: this is the sharpest one. You may substitute a sub-graph that
is **weaker** than what it replaces, provided the property it loses is
already guaranteed by the context. Single-input networks are always
input-balanced, so input-balance is free to break inside them.

**Sub-graph substitution.**
> "Sub-graphs don't need to be balancers to be substituted. Any sub-graph
> can be substituted with something that's equivalent in functionality. A
> conventionally constructed 2-5 has a 3:2 ratio splitter hidden in it. It
> can be replaced with a different 3:2 ratio splitter to create a new 2-5."

*Validity*: functional equivalence at the boundary — the replacement must
produce the same rate vector on the outgoing edges for every rate vector
on the incoming edges. "3:2 ratio splitter" is a sub-graph that splits
one stream into a 3:2 ratio; there are several, and they interchange.

### 5.9 Complete the square / rectangle

**Complete the square** turns a `(1, M)` into an `(M, M)`:

> "It naturally has four unused inputs. Think of it as being fed four
> belts of emptiness. So each output belt has 1/5 belt of items and 4/5
> belt of emptiness. If instead of emptiness we fed it four balanced
> belts (BCDE), then the output would become 1/5 A + 4/5 BCDE, which is
> ABCDE. In other words, it becomes a 5-5."

*Validity* (the "cont." node, quoted in §3.1): the network must consist
entirely of 2-2 splitters, so that output balance implies input balance.
"This is also why only the 4-4 works. If you instead combine 1-5 with
something like 2-4 or 1-4, the result wouldn't be input balanced."

The book records the method's reach: "The 9-9 and 10-10 were also made
using this method, by combining 8-8 with 1-9 or 2-10." So the recipe is
`(N, N)` = `(2^k, 2^k)` butterfly + `(N − 2^k, N)`, for the largest
`2^k < N`.

*(measured, book)*: `(n, n)` splitter counts follow `(n/2)·log₂ n` for
`n = 8, 16, 32` — 12, 32, 80 exactly. The 64-64 (224) and 128-128 (544)
exceed it, so the closed form is not the general law; treat it as the
power-of-two butterfly cost and nothing more.

**Complete the rectangle** is the same idea on a ratio splitter, and it
runs backwards too: "The rectangle can also be 'uncompleted'. Removing
the original input belt creates a new 3:2 ratio splitter." Uncompleting
is how you *discover* sub-graphs to substitute.

### 5.10 Priorities and filters are construction devices, not decorations

The book uses `output_priority`, `input_priority` and splitter output
**filters** as first-class structural elements. *(measured, book)*: the
3-2 TU balancer carries `output_priority: right` on two of its four
splitters — one of them sits directly behind an underground-belt exit,
and the priority is what stops it sideloading onto that exit. The FAQ's
deconstruction-planner trick is the hard version of the same idea, used
to make a splitter a true 2→1.

Our `BalancerTemplateEntity` carries `input_priority` and
`output_priority` (`balancer_library.rs:29-34`) and they are exported to
the blueprint JSON (`blueprint.rs:422-423`). There is **no filter
field**. And crucially, `recover_graph` ignores both priority fields
entirely when building the `SplitterGraph`
(`balancer_classify.rs:358-478`) — so a priority-bearing template is
classified as if both splitter outputs were live. §9.4.

---

## 6. The waist problem

### 6.1 What it is

Every horizontal cut of a top-fed template must carry at least `rated`
belts of forward capacity. If a composition funnels through a narrow
middle, the min cut drops below `rated` and the balancer is throughput-
capped **no matter what the classifier says**, because both the
composition matrix and the max-flow check operate on the *graph*, where
a belt has capacity 1 and nobody counts how many belts cross a row.

The canonical failure, from the RFC's decision log
(`docs/rfc-balancer-bake-lane-validation.md`, 2026-08-13/14): a `(6,4)`
built as `parallel((3,2), 2) → (4,4)` "passed every gate — Balanced, 0/0
audit — and was WITHDRAWN on a 3/3-pass review finding, structurally
confirmed: the library `(3,2)` atom is itself the `Lib(3,1) → Lib(1,2)`
compose whose entire flow crosses one south belt, so the composition
physically caps at 2 of 4 rated belts."

Thirteen templates were culled on 2026-08-14 for this class — twelve
waist-capped plus one lane-imbalanced — and the invariant is now enforced
in CI by `audit_min_cut_capacity` (`balancer_lane_audit.rs:618`).

### 6.2 Why *our* compositions waist

The bake grammar's dominant fan-in pattern is **merge-then-balance**:
`Parallel((k,1), j) → Lib(j, M)` with `j·k = N`. The middle is `j` belts
wide. That is safe only when `j ≥ rated = min(N, M)`.

For prime `N` the factorisation is `j·k ∈ {(N,1), (1,N)}`:

- `j = N, k = 1` — the trivial "merge nothing" case; needs `Lib(N, M)`,
  which is the shape you were trying to build.
- `j = 1, k = N` — everything through **one belt**. Middle width 1, and
  `1 < rated` whenever `M ≥ 2`. This is the waist, by construction.

So the culled recipes were not careless: they were the *only* thing the
merge-then-balance pattern could produce for those shapes. `(3,2)` =
`Lib(3,1) → Lib(1,2)`: mid cut 1, rated 2. `(12,7)` =
`Parallel(4,1,3) → Lib(3,7)`: mid cut 3, rated 7. `(15,7)` =
`Parallel(3,1,5) → Lib(5,7)`: mid cut 5, rated 7.

### 6.3 How the theory avoids it

**Fan out first, merge second** — never the reverse. In the Clos sandwich
(§5.5) the middle is `N·M` wide, so the waist condition is satisfied with
enormous margin. In the divisor-factorised form (§5.6) the middle is
`N·d` wide, so the rule is simply **`N·d ≥ rated`**, which for `d ≥ 1`
holds automatically since `N ≥ min(N, M)`. Fan-out-first compositions
cannot waist.

The dual rule for the merge-then-balance shape is: **`j ≥ rated`**. It is
checkable at recipe-construction time from the recipe alone, before any
solving. That check does not exist in `bake_missing_shapes` today (§9.3).

### 6.4 Undergrounds carry capacity, and the census knows it

*(measured, book)*: the 4-4 TU's crossover row is the instructive case.
Reading its decoded geometry, at the cut between the crossover rows the
surface carries only 2 north-facing belts — the outer two columns are
turning east and west and carry nothing forward — while two
underground-belt runs pass *beneath* the turning belts. Total forward
capacity at that cut: 2 surface + 2 underground = 4 = rated. Exactly
tight.

This is why counting surface belts alone reads false waists, and why
`row_cut_capacities` counts a UG pair across cuts `y1..y2` **inclusive**
(`balancer_lane_audit.rs:516-521`) — the output tile itself emits across
its own cut. It is also the reason undergrounds appear so heavily in the
book's larger designs: they are the mechanism for keeping the min cut at
`rated` while the surface does routing work.

---

## 7. A decision procedure for arbitrary (N, M)

Given `N` and `M`, in order. Each step says when it applies and what you
get.

**Step 0 — is it a balancer you need?**
If the `M` outputs feed identical consumers of one fungible item and the
`N` inputs are equal-rate producers, you need equal *output rate*, not
balance. A passthrough (`N == M`) or a `merge_tree` onto a trunk
(`M == 1`) is correct and vastly smaller. Our engine takes this path
deliberately (`balancer.rs:122-124`, `balancer_generate.rs:350`).
If input rates can differ, skip to step 1 — an unbalanced merge will
propagate the skew.

**Step 1 — look it up.** For `N, M ≤ 9` (and many larger shapes), a
hand-tuned design exists in the book and is 2–5× smaller than anything
you will construct. Import beats generate. §9.5.

**Step 2 — `M == 1`.** Balanced merger = flow-reverse of the `(1, N)` of
step 3. Cost `2^⌈log₂ N⌉ − 1` splitters, or less by factorising.
If balance is not needed, `merge_tree(N)`: `N − 1` splitters, no
preconditions.

**Step 3 — `N == 1`.** Loopback tree (§5.2): `2^⌈log₂ M⌉ − 1` splitters,
or the cheaper factorised chain when `M` is composite. Root splitter
needs `input_priority` toward the external input for TU. **This step
always succeeds, for every `M`.**

**Step 4 — `N == M`.** Butterfly for `N = 2^k`: `(N/2)·log₂ N`
splitters. Otherwise "complete the square" from `(2^k, 2^k)` plus
`(N − 2^k, N)`, valid only if the whole network is 2-2 splitters (§5.9).
Or fall through to step 5.

**Step 5 — divisor-factorised Clos.** Pick `d > 1` with `d | M`, recurse
on `(N, M/d)`; or `d | N`, recurse dually. Middle width `N·d`, which
satisfies the waist invariant automatically. Terminates at step 1/2/3 or
at step 6.

**Step 6 — the Clos sandwich.** `N` fan-outs of `(1, M)`, the
`clos_interleave(N, M)` permutation, `M` balanced mergers of `(N, 1)`.
Balanced and TU by §5.5's proof, waist-free, `N·(2^⌈log₂M⌉ − 1) +
M·(2^⌈log₂N⌉ − 1)` splitters, `N·M` middle. **This step always
succeeds.** It is the reason "arbitrary `(N, M)`" is answerable.

Read "always succeeds" precisely: it always yields a *topology* with the
claimed graph properties. Two things it does not promise, both of which
bite in practice — for fan-in (`N > M`) the balance is MX3 in the fluid
model only, because saturated inputs imply `N/M > 1` belts per output
and the mergers cap each at 1 (§5.5's note); and the placement may be
infeasible at the sizes below.

**Step 7 — compaction, optional.** Apply §5.8's substitutions to shrink
the result. This is where the book's designs come from and it is not
automatable from the rules alone — the rules tell you when a rewrite is
*legal*, not which rewrite to try.

### Where the procedure is *not* known to work

- **It gives no compact design.** Steps 5–6 produce something 2–5× the
  book's size. If footprint matters — and on a bus it does — the
  procedure's answer may be unusable in practice even though it is
  correct. That is the honest limit.
- **Step 6's physical realisation is unproven at scale.** The proof is
  about the graph. Turning an `N·M`-wire permutation into belts is a
  placement problem, and our CP-SAT placer already reports UNKNOWN
  through `jh = 24` at 1800 s on a `(6,4)` Clos (RFC decision log,
  2026-08-13/14). For `N·M > ~12` assume placement, not topology, is the
  binding constraint.
- **Lane balance is not addressed at any step.** Every construction here
  is a belt balancer. A lane balancer needs the lane-swap machinery of
  §3.3 and has no general construction in the book either.
- **TU in-game needs priorities that no step emits.** Steps 3 and 6 both
  require `input_priority` annotations that the constructions above
  describe but that no code path currently generates (§9.4).

---

## 8. Worked examples

### 8.1 `(3, 2)` — the smallest fan-in hole

`rated = 2`. `gcd(3,2) = 1`.

- *What the culled recipe did*: `Lib(3,1) → Lib(1,2)`. Middle width 1 <
  rated 2. **Waist.** Correctly culled.
- *Divisor-factorised Clos (step 5)*: `d = 2 | M`. Stage 1 =
  `Parallel((1,2), 3)` → 6 belts. Stage 2 = `Parallel((3,1), 2)` → 2
  outputs. Middle width 6 ≥ 2. Balanced: each stage-1 belt carries
  `i_t/2`; merger `j` sums three of them → `(i₁+i₂+i₃)/2` ✓. Splitters:
  3·1 + 2·3 = 9. This is exactly the sandwich, since `M = 2` and `d = M`.
- *An alternative that also works*: `Parallel((1,2), 3) → Lib(6,2)`.
  Middle 6, and `Lib(6,2)` is itself waist-clean (its own recipe is
  `Parallel(3,1,2) → Lib(2,2)`, mid cut 2 = rated 2). Balanced for **any
  permutation**, because a genuine MX3 `(6,2)` outputs `(Σ inputs)/2`
  regardless of which belt lands where (§5.6). This is expressible in the
  bake grammar *today*: `Recipe { shape: (3,2), stage1: Parallel(1,2,3),
  stage2: Lib(6,2), perm: Identity, max_jh: … }`.
- *The book's design* *(measured)*: 4 splitters, 24 entities, 4×8, using
  one UG pair and two `output_priority: right` annotations. Less than
  half the size of anything the procedure produces.

### 8.2 `(7, 3)` — prime fan-in, no divisor help

`rated = 3`. `gcd = 1`, `N` prime, `M = 3` prime.

- Step 5 needs `d | M` with `d > 1` → `d = 3`, which recurses to
  `(7, 1)`: stage 1 = `Parallel((1,3), 7)` → 21 belts, stage 2 =
  `Parallel((7,1), 3)`. That *is* the sandwich.
- Cost: `7·3 + 3·7 = 42` splitters, 21-wide middle, 21-wire permutation.
- The book's `7-3` *(measured)*: 12 splitters, 53 entities, 9×8.
- **Verdict**: the sandwich is correct but a 21-wire junction is far
  outside our placer's demonstrated range. For this shape, import.

### 8.3 `(1, 11)` — prime fan-out, nothing in the book

`rated = 1`. No `(1,11)` exists in the book or our library.

- Step 3 applies unconditionally: `L = 16`, loop 5 leaves back into the
  root, `input_priority` on the root's external port. 15 splitters, each
  live leaf carries `1/11` by the §5.2 algebra.
- This is the clean win case: a shape nobody has, buildable from first
  principles, linear cost, one construction.

### 8.4 `(12, 7)` — why it was culled and what would replace it

`rated = 7`. The deleted recipe was `Parallel(4,1,3) → Lib(3,7)`: middle
width 3 < rated 7. **Waist by construction** — and detectable from the
recipe alone, without solving anything.

- Step 5: `d = 7 | M` → recurse on `(12, 1)`; sandwich with a 84-wide
  middle. Not realistic.
- Step 5 the other way: `d | N`, `d ∈ {2, 3, 4, 6}` → `Parallel((1,d),
  12)` then `d` copies of `(12, 7/d)` — `7/d` is not an integer for any
  of them. The `d | N` form requires `d | N` **and** recursion on
  `(N/d, M)`; with `d = 4`: stage 1 = `Parallel((4,1), 3)`… which is the
  waisted recipe again.
- The honest answer for `(12,7)`: no cheap composition exists; it needs a
  native solve or the book (which has no 12-7 either, though it has 12-10
  and 12-12).

---

## 9. Gap analysis against our code

### 9.1 Technique-by-technique

| Technique (§) | Expressible today? | What it would take |
|---|---|---|
| `(1,2)` / `(2,1)` atoms (5.1) | **Yes** — `balancer_generate.rs:152,221` | — |
| Binary fan-out tree, `M = 2^k` (5.2) | **No** — `generate` handles `k = 2` only (`balancer_generate.rs:89-95`) | generalise `replicate_horizontally` to a recursive tree builder |
| **Loopback tree, arbitrary `(1,M)`** (5.2) | **No** | new; needs a back-edge-emitting builder plus `input_priority` on the root. Highest value-per-line item in this table |
| Merge tree, `(N,1)`, unbalanced (5.3) | **Yes** — `merge_tree(n)` | — |
| Balanced `(N,1)` (5.3) | **No** | flow-reverse of the loopback tree |
| **Flow reversal / transpose** (5.4) | **No** | ~20 lines on `SplitterGraph`; free 2× on topology search. Not valid on layouts with sideloads |
| Two-stage series with permutation (5.5–5.6) | **Yes** — `series_permuted` + `clos_interleave` (`balancer_topology.rs:157,221`); `Recipe`/`compose_series` at the template level | — |
| **Clos sandwich as a named fallback** (5.5) | **Expressible, not invoked** — `Recipe { stage1: Parallel(1,M,N), stage2: Parallel(N,1,M), perm: Clos(N,M) }` typechecks against today's grammar for all `2 ≤ N,M ≤ 10` (`Lib(1,k)` and `Lib(k,1)` both exist for `k = 2..10`) | add the recipes; the blocker is junction placement, not the grammar |
| Back-to-back for TU (5.7) | **Partly** — `series` composes two stages, but nothing removes the redundant splitters | a graph-level redundancy pass |
| Deferred loopback (5.7) | **No** | needs loopbacks first |
| Substitution family (5.8) | **No** | needs sub-graph matching over `SplitterGraph`; the composition matrix already computes the "equal belts" equivalence relation used by two of the five rules |
| Complete the square (5.9) | **No** | needs a `(1,M)` with unused inputs, i.e. the loopback tree first |
| Priorities as structure (5.10) | **Data yes, semantics no** — fields exist and export; `recover_graph` ignores them | see 9.4 |
| Splitter output **filters** (5.10) | **No** — no field on `BalancerTemplateEntity` | add field + export + classifier semantics |

### 9.2 The width guard silently excludes every fan-in decomposition

`family_stamp_plan` (`balancer.rs:159-200`) tries, in order: passthrough
→ direct → gcd-decomposition → runtime generator → passthrough-or-
`Unresolvable`.

The decomposition arm requires `sub.width <= sub_m`
(`balancer.rs:176-178`). Combine with invariant §4.2 (`width ≥ N`):

> for any fan-in sub-template, `width ≥ sub_n > sub_m`, so
> `sub.width <= sub_m` is **unsatisfiable by construction**.

Measured at `HEAD`: all 26 fan-in templates have `width > n_outputs`, and
no template has `width < n_inputs`. So the `Decomposed` arm can only ever
fire for fan-out or square shapes. It is not that fan-in shapes have
unlucky widths — they cannot have lucky ones.

This also refines a premise worth correcting: the fan-in holes are *not*
all gcd failures. `(3,2) (7,2) (7,3) (7,4)` have `gcd = 1`, so
decomposition never gets a candidate. But `(8,6)` has `gcd = 2` and
`(4,3)` **is** in the library (`width: 4`, `n_outputs: 3`) — it is the
width guard, not the arithmetic, that blocks it.

Both failures land on `FamilyStampPlan::Unresolvable`
(`balancer.rs:199`), which stamps nothing, skips feeder specs, and
reserves no height. The cull adjudication chose that deliberately — "a
defective template serving an exotic future request silently at half-rate
is worse than that request failing loudly as unstampable" — so the
silence is a design decision, not a bug. It is still the reason a hole is
invisible until something under-delivers.

### 9.3 The bake grammar has no waist check

`bake_missing_shapes` (`balancer-gen/src/main.rs:1427`) gates each bake
on: junction routing succeeds → UG lane issues empty → `classify_ref`
returns `Balanced`. **None of the three can see a waist** — that is the
lesson the 13-shape cull paid for.

The check is arithmetic and costs nothing:

> for `Recipe { stage1, stage2 }`, `mid = stage1.n_outputs`; require
> `mid ≥ min(shape.0, shape.1)`.

Every one of the eight deleted recipes fails it: `(9,4..8)` mid 3 vs
rated 4..8, `(12,7)` mid 3 vs 7, `(15,7)` mid 5 vs 7, `(15,14)` mid 10 vs
14. It would have refused them before the first solver call. Today the
invariant is only enforced *downstream*, on the compiled registry, by
`audit_min_cut_capacity` — which is the right place for the final gate
but the wrong place to learn that a recipe was doomed.

### 9.4 The classifier ignores priorities, so imports are misclassified

`recover_graph` reads `name`, `x`, `y`, `direction`, `io_type` and
nothing else (`balancer_classify.rs:358-478`). A splitter with
`output_priority: right` is modelled with **both** outputs live.

For the book's designs this is not cosmetic. The 3-2 TU's
`output_priority: right` splitter sits directly behind an underground
exit; modelled with a live left output, the walker follows an edge that
carries nothing in the real game, and the recovered graph is not the
balancer's graph. The composition matrix and both Menger checks then run
on the wrong topology. `detect_priority_needed` can *recommend*
priorities from a graph, but nothing consumes annotations *into* one.

The min-cut census has a milder version of the same blindness: it counts
a splitter as 2 tiles of forward capacity regardless of a priority or
filter that disables one side (`balancer_lane_audit.rs:574-578`). That
direction is safe — over-counting can never manufacture a false waist —
but it can miss a real one.

### 9.5 Import beats generate, for the shapes the book has

*(measured, book)*: the book's designs are 2–5× smaller than the
sandwich for every shape in §5.5's table, and it covers all of
`1..9 × 1..9` plus 20 miscellaneous larger shapes, in Factorio 2.0 form
(16-way directions, express tier — both normalisable on import).

The in-flight change to `balancer_library.rs` does exactly this for the
five fan-in holes and, per its own comment, each import clears
`audit_min_cut_capacity` — the invariant the culled composes failed. That
is the correct move and it makes §5.5–§5.6 a fallback for shapes the book
*lacks*, not a replacement for it.

The caveat is §9.4: an imported priority-bearing template is classified
on a graph that does not match it. Import and classification need to
learn priorities together.

---

## 10. What is actually open

Stated plainly, because overstating coverage here sends someone down a
dead end.

1. **A general *compact* `(N, M)` construction does not exist** — not in
   the book, not here. The sandwich (§5.5) is general and provably
   correct; it is 2–5× larger than hand-tuned designs and its junction is
   beyond our placer past `N·M ≈ 12`. Compactness remains a search
   problem, which is why the book's own last word on the subject is
   *"Balancer layouts can be auto-generated using Factorio-SAT"*.

2. **Tier-C throughput (real TU) is unverified in our stack.**
   `classify_graph` tests tiers A and B only. Nothing checks
   `P(S, T)` for arbitrary pairs, so no template in the library has
   been certified TU in the book's sense — including the ones whose
   provenance says "Raynquist (TU)".

3. **The fan-in class ladder is oriented for fan-out** (§3.2 consequence
   2). Whether any real shape is misclassified by it is unmeasured.

4. **Lane balancing has no general construction anywhere.** The book's
   lane balancers are per-shape artifacts; its "Basic lane balancing
   theory" node is an image link. Our MX5 handling is a set of avoidance
   rules, not a construction.

5. **Priorities are structure we can emit but not reason about**
   (§9.4). Until the classifier models them, any construction that
   *needs* them for TU — which includes both stages of the sandwich and
   every loopback tree — can be built and shipped but not verified.

6. **`(N, M)` for `N, M ≥ 10` and coprime** is uncovered by every route:
   not in the book (mostly), not decomposable, sandwich-infeasible to
   place. `(12, 7)` (§8.4) is the worked instance. These need a native
   solve or a genuinely new idea.

---

## Appendix — measured book data

Method: the book blueprint string was decoded to JSON and every leaf
censused for entity counts by type and bounding box. Figures marked
*(measured, book)* above come from that pass. Selected rows, for sizing
intuition:

| Shape | Entities | Splitters | UG | Bounding box |
|-------|---------:|----------:|---:|--------------|
| 1-3 TU | 16 | 3 | 0 | 4×5 |
| 1-5 TU | 25 | 7 | 0 | 5×7 |
| 3-1 TU | 18 | 3 | 0 | 4×6 |
| 3-2 TU | 24 | 4 | 2 | 4×8 |
| 4-4 TU | 32 | 6 | 4 | 4×10 |
| 7-2 | 34 | 8 | 4 | 8×6 |
| 7-3 | 53 | 12 | 4 | 9×8 |
| 7-4 | 58 | 11 | 8 | 9×9 |
| 8-6 | 76 | 18 | 14 | 8×13 |
| 8-8 | 76 | 12 | 14 | 8×12 |
| 8-8 TU | 119 | 20 | 26 | 8×18 |
| 16-16 | 211 | 32 | 50 | 16×16 |
| 32-32 | 727 | 80 | 220 | 32×27 |

Splitter-count regularities worth knowing: `(1, 2^k)` and `(2^k, 1)` both
cost `2^k − 1`; `(1, M)` and `(M, 1)` have identical counts for every `M`
in the book; `(n, n)` costs `(n/2)·log₂ n` for `n = 8, 16, 32` and more
than that for `n = 64, 128`.
