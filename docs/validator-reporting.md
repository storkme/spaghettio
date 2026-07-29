# How a validation check should report

A check that runs, works, and finds the problem can still be useless — if the
way it reports makes the problem invisible to whoever is reading. That failure
mode hit this codebase **nine times**, was found in one day (2026-07-29), and
three of the nine were written *while fixing five of the others*. It is easy to
reintroduce, so it is written down.

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

## The nine

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

Numbers 6, 8 and 9 were written during the session that fixed 1–5. Number 7 was
found in the same audit and is **not yet fixed on `main`** — the fix is in
flight ([#491](https://github.com/storkme/spaghettio/pull/491)), tracked by
[#490](https://github.com/storkme/spaghettio/issues/490).
