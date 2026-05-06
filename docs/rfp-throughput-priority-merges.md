# RFP: Throughput-priority merges (audit phase)

## Summary

The current bus generator routes between asymmetric belt counts using a
SAT-generated balancer library (`crates/core/src/bus/balancer_library.rs`).
The library is *intended* to ship true balancers (composition-balanced,
"MX3" in [`docs/factorio-mechanics.md`](factorio-mechanics.md#belt-merger-taxonomy-mn-splitter-networks)),
which is overkill for spaghettio's single-recipe-per-trunk bus design — every
belt carries one fungible item, so the only property we actually need is
throughput-unlimited (MX2). MX3 is significantly larger than MX2 for
asymmetric `(m, n)` and is the source of the coprime barrier in
[#136](https://github.com/storkme/spaghettio/issues/136) and the oversize
problem in [#135](https://github.com/storkme/spaghettio/issues/135).

This RFP covers **phase 1 only**: build a classifier that takes any
`BalancerTemplate` and assigns it to one of the three taxonomy classes
(MX1 throughput-limited / MX2 throughput-unlimited / MX3 balanced), then
run it across the entire library and produce a taxonomy report. Phases 2+
(generating MX2-only templates for currently-missing or oversized shapes)
are out of scope here and will be scoped in a follow-up RFP once we have
the audit results.

The audit is the load-bearing decision input: if the library is uniformly
MX3 today, the gap to MX2 is the entire project; if some shapes are
already MX2 (or worse, MX1), the project shape changes accordingly.

## Motivation

### Concrete failure cases driving this

- [#136](https://github.com/storkme/spaghettio/issues/136) — missing coprime
  balancer shapes block tier-4 advanced-circuit-from-ores (per CLAUDE.md
  recipe complexity ladder).
- [#135](https://github.com/storkme/spaghettio/issues/135) — oversized
  balancers inflate the bus footprint even when smaller MX2 equivalents
  would suffice.
- Tier-5/6 (processing-unit, rocket-control-unit) are not yet wired up, but
  every deeper recipe chain is more m→n connectors. The library's coverage
  is `(N, M)` for `N, M ∈ 1..10` minus `(1,1)`; tier-5+ will routinely need
  shapes outside that envelope.

### Why audit first

Three reasons:

1. **The fungibility argument is architectural.** Bus rows group machines
   by recipe (`crates/core/src/bus/placer.rs`); each trunk carries a single
   item type. So MX2 is sufficient *by construction* for every existing
   call site. We do not need a separate fungibility-tagging audit; we need
   an audit of *what the library actually delivers today*.
2. **Spec hypothesis vs evidence.** The brief that motivates this work
   assumes the library is uniformly MX3. The generator
   (`scripts/generate_balancer_library.py`) uses `belt_balancer`
   (network-guided, Benes) for symmetric shapes and
   `belt_balancer_net_free` for asymmetric tier 9-10 — both *intended* to
   produce MX3. But "intended" is not "verified". A tier 9-10 net-free
   solve that hits a SAT timeout could in principle land on a weaker
   solution; we should know.
3. **It bounds the project.** If the existing library is mostly already
   MX2 (because the SAT solver finds the smallest valid network and the
   smallest valid networks for some asymmetric shapes are MX2-not-MX3),
   then the "throughput-priority merge" project shrinks to filling
   coprime/missing shapes, not replacing the library wholesale.

The taxonomy itself is documented in
[`docs/factorio-mechanics.md`](factorio-mechanics.md#belt-merger-taxonomy-mn-splitter-networks)
(MX1/MX2/MX3 + lane-level caveat MX5).

## Design

### New module: `crates/core/src/bus/balancer_classify.rs`

Public API:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalancerClass {
    /// MX1: outputs may starve under saturated input.
    ThroughputLimited,
    /// MX2: max-flow property holds for every matched k-subset.
    ThroughputUnlimited,
    /// MX3: every output is uniform 1/m mix of inputs (composition guarantee).
    Balanced,
}

pub struct ClassificationReport {
    pub class: BalancerClass,
    /// Reason / counterexample if not Balanced.
    pub notes: Vec<String>,
    /// Realized vs ideal output rates under saturated inputs (linear model).
    pub composition_matrix: Vec<Vec<f64>>, // [output_idx][input_idx]
}

pub fn classify(template: &BalancerTemplate) -> ClassificationReport;
```

### Algorithm

**Step A — recover the splitter graph.** Walk the entity list and build a
DAG:

- Nodes: each input port (template `input_tiles[i]`), each output port
  (`output_tiles[j]`), each splitter (one node per splitter, 2 inputs +
  2 outputs).
- Edges: belt-flow connectivity. Trace from each output port of every node
  forward through `transport-belt` and `underground-belt` segments until
  hitting another node's input.
  - UG pairing: same-tier UGs on the same axis with opposite/matching
    facing per game rules (U1, U3, U5 in factorio-mechanics.md). Within a
    template, ties are unambiguous — all UGs are the same tier.
  - Side-loads / merges: the SAT-generated templates are believed to use
    only straight feeds and splitter-based merges. The classifier will
    *detect* sideload patterns (perpendicular belt feeding into another
    belt's side, B8) and fail loudly with a clear diagnostic — we do not
    silently model them. If real templates do use sideloads, that's an
    audit finding, not an algorithm bug.

**Step B — composition check (is it MX3?).** Each splitter is modeled as a
default 50/50 distributor (S3); none of the library templates use
input/output priority (the `BalancerTemplateEntity` struct has no priority
field, confirming this). Solve the linear system:

- For each splitter with inputs `(a, b)` and outputs `(c, d)`:
  `rate(c) = rate(d) = (rate(a) + rate(b)) / 2` if both outputs are
  unblocked. Treat all outputs as unblocked for this check.
- Express each output port's rate as a linear combination of input port
  rates — i.e. compute the m×n composition matrix `M` where `output[j] =
  sum_i M[j][i] * input[i]`.
- **MX3 iff** `M[j][i] = 1/m` for all `i, j`.

**Step C — throughput-unlimited check (is it MX2?).** Build a flow
network: input ports as sources, output ports as sinks, splitter
input-output edges with capacity 1 (representing one saturated belt of
flow), belt edges with capacity 1, splitter internal capacity 2 (sum of
its two outputs).

Verify the **max-flow property** in both directions, which together imply
the matched-subset property (Menger):

- For every subset `S ⊆ inputs`: `max_flow(S → all outputs) = min(|S|, n)`.
- For every subset `T ⊆ outputs`: `max_flow(all inputs → T) = min(m, |T|)`.

For library shapes (m, n ≤ 10), `2^m + 2^n ≤ 2048` max-flow runs per
template. Each max-flow is on a graph with ≤ ~50 nodes. Trivially fast
(<10ms per template, <10s for the full library).

**Step D — classification:**

- MX3 if Step B succeeds.
- Else MX2 if Step C succeeds.
- Else MX1 (with the failing subset recorded in `notes` as a
  counterexample).

### Where it runs

A single integration test in `crates/core/tests/balancer_audit.rs`:

```rust
#[test]
fn audit_balancer_library() {
    let mut by_class: BTreeMap<BalancerClass, Vec<(u32, u32)>> = BTreeMap::default();
    for template in all_templates() {
        let report = classify(template);
        by_class.entry(report.class).or_default().push((template.n_inputs, template.n_outputs));
    }
    // Print the taxonomy table; assert nothing — this test is a report.
    print_taxonomy_report(&by_class);
}
```

Run with `cargo test ... audit_balancer_library -- --nocapture` to see the
report. We may later promote this to a `#[ignore]`'d benchmark or a
`crates/core/examples/` binary if it's useful to run standalone, but for
phase 1 a `--nocapture` test is the lowest-friction way to keep it
runnable.

### Trade-offs considered

- **Direct in-game verification (place templates in Factorio, observe).**
  Rejected for phase 1: too slow to iterate on, no automation, and the
  static linear/max-flow checks are exact for default-splitter networks
  (no priorities). In-game spot-checks are reserved for the *generation*
  phase if we add new templates.
- **Reuse Factorio-SAT's own balancer-checker.** Rejected: introduces a
  Python dependency on the test path, and Factorio-SAT's checker is
  designed to verify a single network during solve, not classify a corpus.
  Reimplementing the checks in Rust is cheap (a few hundred LOC) and keeps
  the audit reproducible from `cargo test` alone.
- **Hand-classify by inspecting the generator's solver mode.** Rejected:
  what the generator *intended* and what it *produced* aren't guaranteed
  to match (especially for net-free solves under timeout). The whole point
  of the audit is to verify, not assume.
- **Skip the audit, just write an MX2 generator.** Rejected upthread: the
  audit cost is small (one module + one test), it could materially shrink
  the project ("we already have MX2 for these shapes, only need to
  regenerate these others"), and it gives us a regression test for any
  future generator work.

## Kill criteria

- **Sideloads in the templates.** If the splitter-graph reconstruction
  encounters sideload merges in any library template, the
  default-50/50-splitter linear model is not exact for those templates and
  the classification is unreliable. Stop, document the finding, and
  either extend the model (with explicit lane semantics) or restrict the
  audit to the templates we *can* classify and flag the rest. Do not
  publish a misleading taxonomy.
- **Library is uniformly MX3.** Then the audit confirms the spec brief's
  premise; phase 2 (generator work) proceeds as originally framed. Not a
  kill, but a clear "no surprises, proceed".
- **Library is uniformly MX2 (no MX3 anywhere).** Surprising, but would
  mean the generator was already producing throughput-unlimited-not-balanced
  templates and the coprime barrier in #136 is *not* about MX2 vs MX3 —
  it's about the SAT solver failing to find any solution for those shapes.
  In that case, the throughput-priority project is re-aimed at
  "extend the library with hand-generated MX2 templates for missing
  shapes" and the linear-algebra-vs-SAT story changes. **Treat this as a
  pivot trigger**, not a kill.
- **Mixed MX1 in the library.** If any library template is classified as
  MX1, that's a latent layout bug — the bus would underdeliver under
  partial saturation. File an issue per affected shape and verify the
  in-game behaviour matches the prediction (place the template, block one
  output, observe the input belt backing up). This is a kill of the
  *current* library's correctness, separate from this RFP's main aim.
- **Classifier disagrees with hand-verification on >10% of inspected
  shapes.** During verification (below) we will spot-check ≥10 templates
  by hand. If the classifier mis-classifies more than 1 of 10, it has a
  bug; do not trust the corpus-wide report until fixed.

## Verification plan

Per [the layout-engine verification
protocol](../CLAUDE.md#verification-protocol-for-layout-engine-changes):

1. **Hand-verify the classifier on known shapes.**
   - `(2, 2)`: a single splitter is MX3 (and MX2). The classifier must
     return MX3.
   - `(1, 2)`: a single splitter is MX3 (each output is exactly 1/2 of
     the single input).
   - `(2, 1)`: a single splitter feeding one output (the other output
     port unused) is MX1 — half the input flow is lost. But the library's
     `(2, 1)` template is presumably built differently (priority splitter
     or two-stage). Whatever it is, the classifier should match what
     in-game inspection shows.
   - `(4, 4)`: Benes-network output, expected MX3.
   - `(4, 9)` if present: this is the headline asymmetric case; we expect
     MX3 (it's what Factorio-SAT was asked for) but want explicit
     confirmation.
2. **Cross-check with `belt_flow.rs` validator.** The
   [belt-flow lane-rate walker](../crates/core/src/validate/belt_flow.rs)
   already does Kahn topo sort with splitter pairing. If it can be run on
   a single template in isolation, its predicted output rates under
   saturated input should match the classifier's composition matrix.
   Otherwise, run the full e2e suite (`cargo test --manifest-path
   crates/core/Cargo.toml`) and confirm no regressions — the classifier
   is an offline analysis, no production-path code changes.
3. **Trace-event sanity.** No new trace events introduced; the classifier
   is read-only over `BalancerTemplate` data.
4. **Clippy clean.** `cargo clippy --manifest-path crates/core/Cargo.toml`
   passes with no new warnings.
5. **Report-driven decision.** The taxonomy report (printed by the audit
   test) is the artifact this phase produces. Once we read it, we update
   the **decision log** below with the finding and either open the phase-2
   RFP or pivot per the kill criteria.

## Phasing

- **Phase 1 (this RFP).** Classifier module + audit test + taxonomy
  report. Deliverable: a markdown table in the decision log below listing
  every `(m, n)` template and its class, plus a one-paragraph summary of
  what we learned.
- **Phase 2 (out of scope here, next RFP).** Action depends on phase 1's
  finding. Most likely options:
  - *MX2 generator* — write a hand-coded throughput-unlimited generator
    for the shapes where MX3 is oversized. Replaces or supplements
    Factorio-SAT for those shapes.
  - *Library extension* — generate MX2 templates for shapes Factorio-SAT
    can't currently solve (4-9 and beyond), using a different algorithm
    (decomposition? hand-coded patterns?).
  - *No generator work needed* — if MX3 turns out to be the right size
    and the coprime barrier is purely a SAT-solver issue, this whole
    project re-aims at the SAT-runner work
    ([rfp-balancer-runner.md](rfp-balancer-runner.md)) and we ship.
- **Phase 3+** (out of scope here). Integration into the layout pipeline,
  lane-level checks (MX5), and any in-game verification of new templates.

## Decision log

- *2026-05-01 — drafted. Awaiting user approval before implementing the
  classifier. Discussion thread:
  taxonomy + audit-first approach agreed; phase 2 generator design
  deferred until audit results are in.*

- *2026-05-01 — phase 1 implemented in
  `crates/core/src/bus/balancer_classify.rs`; report produced via
  `cargo test --lib audit_report -- --nocapture`.*

- *2026-05-01 — initial run had 15 templates trip the sideload kill
  criterion. After discussion, kill removed (Factorio-SAT places splitter
  outputs on downstream splitter anchor tiles intentionally; the linear-
  system composition handles multi-feeder splitter inputs via flow
  conservation, and lane-level semantics are an MX5 concern). Final
  findings:*

  | class | count |
  |-------|-------|
  | MX3 balanced | 60 |
  | MX1 throughput-limited | 2 |
  | kill: singular linear system | 1 |
  | **total** | **63** |

  **Headline:** the user's hypothesis is largely confirmed —
  60/63 templates are unambiguously MX3 (true balanced balancers,
  including back-loop "universal" patterns and side-loaded splitter
  inputs that Factorio-SAT generates for compactness). Three outliers
  warrant follow-up:

  - **(5, 8) — MX1 throughput-limited.** The classifier returns a
    non-uniform composition matrix (inputs 1-2 contribute 0.0625 to outer
    outputs vs 0.1875 to inner outputs, instead of the uniform 0.125
    target). Max-flow counterexample: under saturated inputs `{1, 2}`,
    realized throughput is 1 (expected 2). Tracked in
    [#266](https://github.com/storkme/spaghettio/issues/266).
  - **(8, 6) — MX1 throughput-limited.** Composition is uniform across
    inputs but uneven across outputs (outputs 0-3 each get 0.1458 from
    every input, outputs 4-5 each get 0.2083). Max-flow counterexample:
    under saturated inputs `{0..5}`, realized throughput is 5 (expected
    6). Tracked in [#266](https://github.com/storkme/spaghettio/issues/266).
  - **(7, 6) — kill: singular.** The linear system describing the
    saturated 50/50 splitter network is singular (a recirculation loop
    the simple model can't resolve). Max-flow finds paths so this isn't
    a topology bug per se — it's a model limitation. Likely also MX3 in
    reality but not provable with the current classifier.

  **What changed during implementation:**
  - Topo-sort composition replaced with a Gaussian solve over the
    splitter outflow vector. Cycles dominate the corpus (universal-
    balancer back-loops); topo-prop would have killed on most templates.
  - Sideload kill criterion removed entirely. Belt-level B8 and U7 side-
    loads, plus side-loaded splitter inputs (Factorio-SAT's compaction
    pattern), are accepted as valid flow merges. The walker emits one
    edge per upstream source; the linear-system handles multi-feeder
    splitter inputs via flow conservation.
  - Walker gained visited-tile tracking — once side-loaded splitter
    outputs can re-enter the network, belts can form literal cycles, so
    looping flow drops the edge silently.

  **Implication for phase 2:** the spec brief's premise (library is
  uniformly MX3) holds for 60/63 templates. The two MX1 findings are
  independent of the throughput-priority project; tracked separately.
  Phase 2 (generator) deferred until concretely needed — most likely
  trigger is tier-5/6 layouts demanding shapes outside the current
  library envelope.
