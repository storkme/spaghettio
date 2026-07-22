# Sim-harness forensics — reading and debugging measured results

Reference doc for `spaghettio-sim` (RFC-050). What each reported number
actually is, the measurement-artifact classes we have hit (each was found
by challenging a number that looked decisive), and the forensic playbook
that localizes a bad result. Keep current as the harness changes.

## What each number is

- **Target item rates** (`measured_produced_rate` / `delivered` for the
  manifest target): Δ(cumulative production counter) over the **last
  300-game-second checkpoint window**. `converged=true` means the last
  two consecutive windows agreed within 2%.
- **Intermediate item rates**: measured over the **same trailing
  checkpoint window** as the target (since #362). Before that they were
  the last two 20-second samples — badly aliased for bursty producers (a
  gear machine crafting in bursts read 0.40/s on a snapshot vs 0.80/s
  honest).
- **`samples`** (in `raw_result`): cumulative production counters for
  every planned item every 1200 ticks (20 game-seconds), covering the
  whole run *including warmup*. This is the trajectory record — bin it
  to see transients, plateaus, and oscillations. Whole-run averages and
  first-divergence ordering both come from here.
- **`sim_state`** (frame at finalize): per-belt per-line item contents,
  machine statuses + input/output inventories, inserter statuses, UG
  pairing as the game resolved it, splitter priority/filter state, kit
  chest census. A *single frame* — statuses are instantaneous (a
  demand-limited machine flickers `working`/`full_output`; do not read
  one frame's status as a time-average).
- **`kit_errors`**: the boundary kit's self-audit. Non-empty ⇒ the run
  is invalid and the verdict is forced NO DATA. Never interpret rates
  from a run with kit errors.

## Measurement-artifact classes (all real, all found in one day)

1. **Buffer-fill transient read as convergence.** The 2% stability
   window cannot distinguish steady state from a slow drift: a
   deep-chain fixture "converges" while trunk buffers are still
   filling. Signature: intermediate stages at or *above* plan while the
   target fails — above-plan draws are never steady state. Cure:
   `--warmup 216000`-class steady-state probes; measure after the
   transient.
2. **Snapshot aliasing.** Any rate computed over a window much shorter
   than the producer's burst cycle is noise. Signature: an intermediate
   rate wildly inconsistent with its neighbors' consumption arithmetic.
   Cure: recompute from `samples` over a long trailing window (the
   default reports now do).
3. **Kit contamination.** Boundary-kit rigs that collide cross-feed
   items: overlapping bank chests (Factorio's `create_entity` in script
   mode **stacks entities silently**) let an inserter latch the wrong
   item's chest. Wrong-tier items then poison the factory — see the
   poison-plug mechanic below. Signature: `item_ingredient_shortage`
   beside a full input belt; wrong item in a belt-in's per-line
   contents; nonuniform "starvation" that no capacity arithmetic
   explains. Cure: the kit self-audit (chest census → `kit_errors`),
   plus depth-staggered rigs. **When a sim result shows wrong-item or
   inexplicable starvation signatures, suspect the kit before the
   layout.**

## The poison-plug mechanic (game truth, mechanics rule I11)

Inserters refuse to pick items their destination cannot accept. On a
dead-end feed belt the inserter is the only exit, so a single wrong
item reaching the front tile **plugs that lane permanently** — the
machine starves with a "full" belt beside it. One contaminant item is
enough; the plug never clears. This is also why contamination is so
destructive out of proportion to its rate: ~17 stray copper plates
capped an entire factory.

## Forensic playbook (in escalation order)

1. **Trajectory first** (`samples`): bin per-item rates over game-time.
   Distinguishes transient vs plateau vs oscillation, and the *order*
   in which stages diverge from plan points at the causal root.
2. **Frame reading** (`sim_state`): machine statuses + inventories
   joined with belt per-line contents against `entities.json` segments.
   Full-upstream/starved-downstream = backpressure from below; wrong
   item anywhere = stop, check the kit.
3. **Micro-fixture isolation**: rebuild the suspect geometry as a
   ~10-entity blueprint + hand manifest, flood it, measure. If the
   micro passes at capacity, the local mechanic is innocent — the
   defect is systemic or infrastructural. **Derive `bbox_min`/dims/
   boundaries from the entity list programmatically** — two of our
   three false reproductions were hand-typed manifest errors (anchor
   off by one; drain boundary one tile past the belt end).
4. **Infrastructure census**: kit chests (overlaps, contents), UG
   pairing vs engine intent, splitter priority state as revived. The
   instrument is a suspect too.

## Operational pitfalls

- **Factorio instance lock**: back-to-back runs race the previous
  server's shutdown (`Couldn't acquire exclusive lock`). Guard with
  `until flock -n <install>/.lock -c true; do sleep 2; done` between
  runs. Never `pgrep`-wait with a pattern that matches your own shell.
- **Belt counts are per transport line** of the entity: a 2-tile
  single-lane line reads as "8" on *each* entity; splitters (8 lines)
  and UGs read higher. Use the per-line contents, not the total, when
  lanes matter.
- **`create_entity` in script mode ignores collisions.** Anything the
  scenario builds must be followed by an occupancy audit; a silent
  overlap is invisible on every belt and poisons everything downstream.
