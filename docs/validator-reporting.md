# How a validation check should report

A check that runs, works, and finds the problem can still be useless — if the
way it reports makes the problem invisible to whoever is reading. That failure
mode hit this codebase **ten times**. Nine were found in one day (2026-07-29),
and three of those were written *while fixing the other six*. It is easy to
reintroduce, so it is written down.

Number ten (2026-07-31) is the one that cost the most: it let `main` ship a
factory running at **half its planned rate**, with the validator reporting zero
errors and zero warnings.

## The shape

Consumers of validation compare **issue counts by category**: the fold's
admission gate, the compaction gate, a human scanning a summary. A check that
collapses N problems into one issue is therefore invisible to all of them —
2 and 218 both read as `{"category": 1}`.

## Rules for writing a check

1. **One issue per instance, positioned.** Not one issue with a count in its
   message text. `check_power_coverage` and `check_row_output_lane_budget` are
   the reference shape; use `ValidationIssue::with_pos` so it can be found in
   the snapshot debugger.
2. **Measure an absolute property, not one relative to an arbitrary element.**
   "Unreachable from element 0" is a fine boolean and a misleading magnitude:
   it moves the wrong way when the layout genuinely improves. Count components.
3. **Never truncate silently.** A cap, an early return, or an exhausted budget
   must emit a trace event. A caller cannot otherwise distinguish "repaired"
   from "gave up".
4. **Cross-check derived metadata against the entities.** A `LayoutResult`
   field is a claim, not a fact. `check_stranded_byproducts` is the model:
   an exit record counts only if a real entity carrying that item sits at the
   recorded tile.

## Rules for reading validation

5. **A check going quiet is not evidence the problem is fixed.** It is equally
   consistent with the check having stopped discriminating. Verify the specific
   invariant — instrument it and count.
6. **Count, don't sample.** Summarising instrumentation by frequency
   (`sort -rn`, `head`) surfaces the common case and hides the tail, which is
   usually where the interesting minority lives.
7. **Validator-clean and sim-green are each necessary and neither is
   sufficient.** Two independent examples: a fold validated at exact parity
   with its control and produced 0.00/s in Factorio (a relocated belt left its
   boundary record behind); and the sim harness energises every pole network it
   finds, so it reported 146/146 machines working for a blueprint that pastes
   as two dead halves.

## The ten

Kept as evidence that this is a pattern rather than a run of bad luck.

| # | Where | Shape |
|---|---|---|
| 1 | `boundary_outputs` | no check existed at all, while byproduct exits had one |
| 2 | `check_pole_network_connectivity` | count in message text; 2 and 89 read alike |
| 3 | `repair_pole_connectivity` | flat 20-bridge cap, silent on exhaustion |
| 4 | `power_wires::disconnected_poles` | measured from `pole[0]`; a real repair read as regression |
| 5 | `check_sushi_saturation` | N belts collapsed to one arbitrary entry |
| 6 | bridge give-up path | traced only under an env var |
| 7 | `check_belt_network_topology` | count in prose *and* origin-relative |
| 8 | `claude-review` guard | asked "was this PR reviewed", not "was this code reviewed" |
| 9 | a PR-watch monitor | reported `passing: 0` for CI that had not started |
| 10 | `check_belt_flow_reachability` | asked per MACHINE over the union of its input belts; one fed input masked a starved one |

Numbers 6, 8 and 9 were written during the session that fixed 1–5. Number 7 was
found in the same audit and fixed last, in
[#491](https://github.com/storkme/spaghettio/pull/491) — so all six of the
others are fixed on `main`.

### Number ten in detail (2026-07-31, #520)

Worth more than a table row, because it is the first one with a measured cost in
a real Factorio, and because the shape is subtler than "a count in a message".

`check_belt_flow_reachability` seeded one BFS with **all** of a machine's input
belts and asked whether the union reached a source. On `display-panel`
(iron-plate + electronic-circuit) the iron-plate belt's path back to the furnaces
satisfied the test, so the electronic-circuit belt — which nothing fed, because
the only drop onto it was *downstream* of the pickup — was never examined. A
per-machine question cannot distinguish **"every input is fed"** from **"some
input is fed"**, which is rule 1's problem wearing different clothes: the check
did not collapse N issues into one, it collapsed N *questions* into one.

Compounding it, belt-to-belt lift inserters were not modelled at all. The
classifier recognised only machine→belt and belt→machine, so a lift's drop was
not a source of its belt and its own pickup was never checked — which is why the
fault localised nowhere even when something downstream was visibly starving.

**Measured cost.** `small-electric-pole@5` on am1 shipped a 126-entity DI layout,
denser than native's 163 and clean on every channel, that a headless run measured
at **2.52/s against a planned 5.00/s** — converged, so a steady state. Native
measures 5.08/s. `di_choice` preferred the broken one correctly, by every signal
the engine had.

**The rule this adds.** Rules 1 and 2 are about how many issues a check emits.
This one is about how many questions it asks: **a check that aggregates over a
set must ask its question per element of that set, not over the union.** The
union form is strictly weaker and its weakness is invisible — it reports success
whenever *any* element passes. Both directions of this check are now one forward
sweep from every source and one backward sweep from every sink, tested per tile;
that is also cheaper than the per-machine BFS it replaced.

**And the fix's own near-miss**, kept because it is the same lesson one level
down: the first version seeded those sweeps with the source tiles themselves,
which let a boundary output tile count as its own sink. The check's existing
`flow_reachability_output_dead_end_fails` unit test caught it. The sweeps are now
seeded one step in.
