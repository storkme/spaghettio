# Meter attribution — open work

**Status (2026-07-25):** RFC-054's KC1 is **tripped and unfixed**. The meter
agrees with real Factorio to 0.3–0.6pp on the EC family and is ~56pp wrong on
the military family. Three attribution rounds have eliminated four hypotheses
and found two real defects, but the headline gap is untouched. The next step
needs **a Factorio install**, which is why this is a handoff rather than a
continuation.

Owning RFC: [`rfc-054-fast-meter.md`](rfc-054-fast-meter.md) — its decision
log is the full history. This file is the short version plus the pickup plan.

## The gap

| config | real band | real | meter | note |
|---|---|---|---|---|
| chain-ec15 d1/d2/d7 | Marginal | −8.0 / −6.0 / −5.3% | −7.4 / −5.6 / −5.6% | agrees |
| chain-ec30-d2 | Marginal | −5.3% | −5.6% | agrees |
| **chain-mil5plates-d0** | **Pass** | **−3.3%** | **−59.6%** | **causes every rank inversion** |
| chain-mil5ore-d2 | Fail | −28.7% | −64.0% | same direction |
| logistic-science-pack | — | not measured | −68.3% | same family of symptom |

`chain-mil5plates` is the whole KC1 failure. It is solid-only (no fluid
confound), small (46 machines, 883 belt tiles, 160 inserters), and real
Factorio measures it essentially at plan.

## What has been eliminated

Each of these was tested directly and **falsified** — recorded so nobody
re-runs them:

1. **Supply / topology.** The coal belt is fully compressed (4/4 both lanes)
   along its entire length, verified by a tile-level downstream walk
   (`--example trace_belt`). Coal is on the belt, near the head, and not
   reaching the tail.
2. **Inserter swing rate.** An env-gated multiplier on the cycle plateaus
   near −39% — at 2.2×, beyond the top of RFC-049's measured margin band, the
   deficit is still 36pp from real. Real contributor, not the binding one.
3. **Machine input buffering.** Sweeping `DEFAULT_BUFFER_CRAFTS` over
   1, 2, 4, 14, 40 returns **byte-identical** −100% on logistic. Buffer depth
   governs how long a machine rides out a gap; it cannot manufacture supply.
4. **Belt→machine rate model** (the most arithmetically persuasive one — a
   grenade machine needs 1.5625 coal/s from one regular inserter rated 0.84/s,
   a 1.86× shortfall that sits squarely inside RFC-049's measured band).
   Explains ~20pp and then stops.

**What is left:** how much of a compressed belt a single inserter can actually
claim per swing — specifically `drop_onto_tile`'s far-lane placement via
`try_insert_anywhere`, and whether **I6** (pickup draws from BOTH lanes) is
honoured in the take path once the first lane is exhausted.

## What was found and fixed en route

Not the answer, but real defects the attribution surfaced:

- **Splitter second cell** derived from `left_of(direction)`, which flips sign
  between north and south — south/west splitters claimed a cell outside their
  own footprint, silently unlinking tap-off branches on every bus layout.
  Logistic −100% → −68.3%.
- **I11** — inserters grabbed items the destination could not accept and
  jammed permanently.
- Plus nine review findings (see the RFC log), three of which were latent
  under the whole suite: an orphan-splitter panic, an inverted drop lane for
  reach-2 inserters, and a truncated product expectation.

## Pickup plan (needs Factorio)

**The decisive experiment is a two-instrument comparison on one fixture.**
Both tools already exist and emit the same shape of dump.

```bash
# 0. once per install
cargo run -p spaghettio_sim_harness -- fetch

# 1. generate fixtures (no Factorio needed for this step)
cargo test --manifest-path crates/core/Cargo.toml \
    --test cell_composition -- --ignored export_chain_fixtures_for_sim

# 2. ground truth: per-machine status + inventory from real Factorio
scripts/sim-capture-state.sh chain-mil5plates-d0

# 3. the meter's answer for the same fixture
cargo run --release -p spaghettio_meter --example attribute chain-mil5plates-d0
```

The meter currently claims: 6 machines in `item_ingredient_shortage`, grenade
at 1.35/2.50, coal declining head→tail (9,9,8,8,7,7,6,6,5,5) while iron sits
capped at 70/70.

**Read the game's answer to one question: are the grenade machines actually
short of coal?**

| game says | means | where to look |
|---|---|---|
| machines working, coal buffers healthy | the meter's belt→inserter extraction is too pessimistic | `take_from_tile_filtered` (I6, both lanes), `drop_onto_tile` |
| same shortages, but output still at plan | census right, rate accounting wrong | `MeterReport` construction, window/warmup |
| coal arrives by a path the meter doesn't model | an ingestion gap like the splitter bug | `NetworkBuilder`, tile linking near the grenade row |

These need different fixes, and no amount of reasoning from the meter alone
distinguishes them — which is exactly why three rounds converged on
elimination rather than a cause.

## Also blocked on Factorio

- **The five blessed sim baselines are stale.** Independently corroborated by
  the gauntlet numbers in [`status.md`](status.md).
  `crates/core/tests/export_sim_baselines.rs` makes them locally reproducible
  and **reports** drift rather than asserting it. Re-bless once there is an
  install to re-measure against.
- **Eyeball `chain-mil5plates` in-game.** Nobody has. The repo's verification
  protocol is explicit that a zero-warning layout which visibly has
  disconnected belts is a validator bug, not a success.

## Standing habit, earned the hard way

**Fix on merit, test the story separately.** Five times in this RFC a
correct-looking change arrived with a plausible causal story attached, and the
story did not survive testing. Twice the wrong theory still routed to the right
defect. Apply the fix if it stands on its own; verify the explanation
independently or leave it unclaimed.
