# Module-support followups

Status: **all items open** (created 2026-07-21 at the RFC-044 retro-review
close-out). Deferred-work backlog with pick-up notes, per the
followups-doc convention. Owner RFC: `rfc-044-machine-modules.md`.

## 1. Full item-request fidelity through the model (retro m1)

The parser collapses ALL entity `items` (fuel, ammo, modules) into
`ModuleItem`, discarding the insert-plan `inventory` id and per-position
`count` (a `{"count": 50}` coal request parses as count 1). The exporter
now filters to module-shaped names (restoring pre-RFC-044 drop semantics
for the rest), so nothing mis-encodes — but import → re-export still
LOSES non-module requests. Pick-up: carry `inventory: Option<u8>` and the
true count through `ModuleItem` (or a sibling type) and re-emit verbatim;
the export filter then becomes "module-shaped OR carried-verbatim".

## 2. Modded module hosts default to inventory 4 (retro n3)

`common::module_inventory_id` returns 4 for unknown entities — wrong for
modded labs/beacons/drills. Acceptable and documented; revisit only if
modded-blueprint fidelity becomes a goal (would need a prototype-type
field through the parser).

## 3. Parser footprints missing big-mining-drill and pumpjack

`blueprint_parser::entity_footprint` lacks `big-mining-drill` (5×5) and
`pumpjack` (3×3) → both fall to the 1×1 default, so imported blueprints
containing them get wrong top-left anchors (center − size/2). Pre-existing
(predates RFC-044), spotted during the retro-fix pass but not
adversarially reviewed — fix with a targeted parse-position test. The web
`MACHINE_SIZES` half of this was fixed in the retro-fix PR.

## 4. analysis.rs is module-quality-blind (Phase 1 close-out note)

`blueprint-analyze` computes module effects at base values — a
quality-moduled blueprint's rate estimate ignores the ×(1+0.3·level)
1%-floored scaling that `module_policy::quality_scaled` implements. The
RFC's verification plan confines the re-analysis gate to normal-quality
modules because of this. Pick-up: route analysis.rs through the shared
scaling fn (it now exists) and lift the gate's restriction.

## 5. Beacon arc (deliberate RFC-044 non-goal)

Geometry work: 3×3 rows, supply distance 3, count-based transmission
falloff (the `BEACON_DISTRIBUTION_EFFECTIVITY = 0.5` constant is a
documented 1.x-era simplification of 2.0's ~1.5/√N profile), beacon power
draw. Starts as its own RFC; all module math (`module_policy.rs`,
per-quality scaling, insert-plan export incl. beacon inventory 1) is
already in place.
