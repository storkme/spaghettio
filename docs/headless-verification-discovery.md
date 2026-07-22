# Headless-Factorio verification — discovery notes (2026-07-22)

Notes-class doc: empirical discovery for the architecture audit's §8.3
("headless-Factorio verification harness", `architecture-audit-2026-07.md`).
Everything below was **run live** on a WSL2 dev machine, not researched.
Mine into the harness RFC when that work starts, then archive.

## Findings (all verified by execution)

1. **Anonymous headless download works.** `curl -L
   https://factorio.com/get-download/latest/headless/linux64` → HTTP 200,
   58 MB, no credentials. Delivered **2.1.12** (note: the game has moved
   past our 2.0.76 draftsman data baseline — see finding 8).
2. **Runs on WSL2 out of the box** (`--version`, `--create`, server mode).
3. **Space Age prototypes load anonymously — the audit's #1 flagged risk
   is retired.** `space-age`, `quality`, `elevated-rails` ship inside the
   headless tarball's `data/`; enabling them in `mods/mod-list.json` gives
   a `headless, space-age` binary with `FeatureFlag quality = true` and a
   successful `--create`. No entitlement wall server-side.
4. **The import→build→revive loop works end to end** in a scenario
   `on_init`: `LuaItemStack::import_stack(bp)` → returns 0 on our exported
   strings → `build_blueprint{build_mode = defines.build_mode.superforced}`
   → `ghost.revive()` each → `helpers.write_file` a JSON result. Run via
   `--start-server-load-scenario bp-smoke`, poll `script-output/`, kill the
   server (~6 s wall total for a 149-entity factory).
5. **A full generated factory imports clean**: EC@10/s from plates
   (149 entities — belts ×98 incl. UGs, inserters ×27, AM3 ×10, poles ×12)
   → 149/149 ghosts revived, 0 failures, every entity class placed.
6. **The `wires` array is honored by the game engine**:
   `electric_network_id` sweep shows all 12 poles in ONE network. This is
   the first automated artifact-boundary verification of the RFC-040 wires
   fix (previously user-paste only).
7. **Module insert plans materialize as `item-request-proxy` entities**
   (RFC-044 anchor: 3 machines with modules → 3 proxies), exactly as the
   audit predicted — the harness must fulfill module requests (insert +
   destroy proxy) before measuring rates.
8. **New constraints discovered:**
   - **Resource-requiring entities don't ghost on bare lab tiles**: the
     anchor's electric-mining-drill and pumpjack produced no ghosts (no
     ore/oil under them). The harness must `create_entity` resource
     patches for mining fixtures, or measurement fixtures simply avoid
     drills (our generated factories never place them — only imported
     blueprints do).
   - **Version drift**: headless `latest` is 2.1.12 with a new `recycler`
     core data module; our recipes.json is 2.0.76-era. Either pin the
     download (`factorio.com/get-download/<version>/headless/linux64`) to
     the data baseline, or re-extract against 2.1.x and diff — do this
     check FIRST in the harness RFC (a silent recipe/prototype drift would
     poison rate comparisons).

## Repro

```bash
# ~2 minutes end to end, scratch dir of your choice
curl -sL -o fh.tar.xz "https://factorio.com/get-download/latest/headless/linux64"
tar -xJf fh.tar.xz
cd factorio
printf '%s' '{"mods":[{"name":"base","enabled":true},{"name":"space-age","enabled":true},{"name":"quality","enabled":true},{"name":"elevated-rails","enabled":true}]}' > mods/mod-list.json
mkdir -p scenarios/bp-smoke   # control.lua: import_stack + build_blueprint + revive + write_file
./bin/x64/factorio --start-server-load-scenario bp-smoke \
    --server-settings data/server-settings.example.json
# poll script-output/bp-smoke.json, then kill
```

Blueprint strings come from the engine directly:
`cargo run --example export_bp_string <item> <rate>` (local example,
`crates/core/examples/` is gitignored) or `rfc044_anchor` for the moduled
four-class anchor.

## What remains for the real harness (unchanged from the audit's sketch)

Power injection (`electric-energy-interface` per network), input feeding
(infinity-chest + loaders per the solver's external-input manifest),
output drain, warmup + steady-state detection,
`get_item_production_statistics` reading ("input" = produced), per-quality
`FlowStatisticsID`, orchestration + CI wiring. None of these were blocked
or complicated by today's findings; the audit's 2–4 day estimate looks
right, and its ~1 min/blueprint cost estimate looks conservative (the
149-entity import cycle was ~6 s).

## Suggested next step

A harness RFC (next registry number) with Phase 0 = the version-drift
check (finding 8b) and Phase 1 = rate measurement on the tier-1 gear
fixture — the first automated number the validator chain has ever had
from the game itself.
