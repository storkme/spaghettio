# Handoff: what does `PlacedEntity::rate` actually mean?

**Status (2026-08-07):** open question, blocking three things. Pick this up
first — everything else in the area queues behind it.

## The question in one line

Every belt tile in a layout carries a number called `rate`. **Nobody knows,
per stamp site, whether that number is "what flows through this tile" or
"what the whole family of parallel belts carries."** It is probably both,
depending on which code stamped it — and that ambiguity has now cost two
pieces of work.

## Why it matters

If `rate` is per-tile, then `rate > belt capacity` means a belt is
over-committed — a real, physical defect worth blocking a build over. If
it's a family total spread across several parallel belts, the same
comparison means nothing at all.

Three things are stuck on the answer:

1. **A measured PU fix cannot ship.** `processing-unit@1/s` currently
   builds a factory that starves (4 machines making green circuits feeding
   8 that need them). A one-line fix makes the generator pick a different
   layout that **measures 102% of plan in a real Factorio**. It is blocked
   because an e2e fixture claims the fix produces a "physically impossible"
   layout elsewhere — a claim made entirely by comparing `rate` to belt
   capacity.
2. **[#311](https://github.com/storkme/spaghettio/issues/311) may be partly
   fictional.** It is parked as a known defect ("60/s stamped onto a 30/s
   belt"), evidenced largely by this same comparison. Real or not depends
   on the answer.
3. **A validator check for over-capacity belts cannot be written** until we
   know whether the data supports one. One was written on 2026-08-07 and
   falsified within hours.

## What we know, and what contradicts what

**Measurements say the comparison is meaningless.** Three sim runs, all
valid (`kit_errors` empty, converged):

| layout | comparison says | actually measured |
|---|---|---|
| AC@7/s horizontal stack | 104 tiles **3.00× over** | **94%** of plan |
| EC@45/s express legendary | 72 tiles **1.11× over** | **101%** (PASS) |
| `stacking_ec_60s` S=2 | 376 tiles **3.00× over** | **96.0%** of plan |

A belt throttled to a third of its planned flow cannot deliver 96%.

**The code says the comparison is meaningful** — at least sometimes:

- `lane_planner::split_overflowing_lanes` is documented as *"Split lanes
  whose rate exceeds the available belt's per-lane capacity"* and stamps
  `split_rate = lane.rate / effective_n_splits` — a **divided, per-lane**
  figure. Its `clamp_to_consumers` path can cap the split count below what
  capacity needs, which would leave a per-lane rate genuinely over one
  belt.
- `output_merger` stamps `total_rate` on the **single** output belt of an
  N→1 merger cascade, where no parallel family exists.

**Both cannot be simply true.** Something is missing — most likely *which*
tiles get flagged, and whether the planned flow ever reaches them.

## How to settle it

There are only **76 stamp sites**: 65 in `bus/templates.rs`, 9 in
`bus/output_merger.rs`, 2 in `bus/ghost_router.rs`.

1. Enumerate them. For each, record what the stamped number denotes —
   per-tile flow, per-lane share, family total, merger total.
2. Take the 376 flagged tiles from `stacking_ec_60s` S=2 and attribute
   each to the stamp site that produced it. Reproduce with the selection
   fix — branch `fix/input-rate-delivery-counts-for-selection`, pushed to
   `origin` 2026-08-07, one commit on top of `main`: it removes
   `input-rate-delivery` from `selection_warning_count`. Config is
   EC@60/s, `assembling-machine-2`, from ore, `fast-transport-belt`
   ceiling, `stacking: 2`.
3. That table answers everything: whether the audit is valid, partly
   valid, or invalid — and if partly, exactly where.

Only then: restore or retire the guard, unblock PU, and settle #311.

## Two traps, both already sprung

- **Differing values across tiles do NOT prove the number is per-tile.**
  One item showed 28.33 and 45.00 on different tiles, which was read as
  evidence of genuine per-tile flow. It fits per-lane division equally
  well. That misreading is what made the falsified check look justified.
- **A layout being validator-clean does not mean the audit is wrong, and
  a layout delivering 96% does not mean every flagged tile is fine.** The
  flagged tiles may simply not be on the binding path. Nobody has checked.

## State of the work

- **Merged, and currently WRONG on this point:**
  [`validator-trust.md`](validator-trust.md) (#595). Its **hole 1** still
  reads *"No validator check compares stamped/planned belt rates to
  physical capacity … **Next action:** promote the audit into `validate/`
  as an Error."* **Do not do that.** That is precisely the check that was
  written on 2026-08-07 and falsified within hours by the measurements
  above. Two further claims there are also wrong: a check *does* compare
  flow to capacity (`validate::check_lane_throughput`, at `Severity::Error`,
  correctly, by walking the belt graph), and the "376 tiles" figure is
  cited as the anchor *for* the hole when the same layout measures 96% of
  plan. A corrected rewrite exists in draft #597 but is itself held by the
  premise in question — so **hole 1 stays wrong on `main` until this is
  settled.** Fix it as part of the answer, not before.
  `serve` keep-alive (#596) is merged and sound.
- **Draft, do not merge:** #597, which retires the comparison from two
  fixtures. Its premise is the thing in doubt. It also undercounts scope —
  `stacking_fanin_wall_lift_ec6_yellow_legendary` performs the same audit
  and was untouched.
- **Parked, unmerged:** a `belt-capacity` validator check (falsified by
  the measurements above).
- **Parked, ready, measured:** the `input-rate-delivery` selection fix on
  `fix/input-rate-delivery-counts-for-selection`. PU 68% → **102%**. On
  EC@60/s it costs 1.2/s (98.0% → 96.0%) for a **39% smaller** factory —
  deterministic, confirmed by three identical repeat runs. The owner has
  already accepted that trade; it is held only by the question above.

## The bigger pattern this is an instance of

On 2026-08-07, five separate things looked correct while measuring the
wrong quantity: the falsified capacity check, the fixture audit, the first
cut of the `serve` fix (77 green unit tests, wrong behaviour), a "0
result" from a server that never started, and a test assertion matching
the wrong line in a file.

[`validator-reporting.md`](validator-reporting.md) covers checks that go
quiet. [`validator-trust.md`](validator-trust.md) covers whether a check is
believed. Neither covers this: **a number whose name implies a meaning it
does not always have.** A field called `rate` on a belt entity reads
unambiguously as "what flows here". It isn't, and nothing records that.
Worth fixing where the numbers live, not just in a doc.
