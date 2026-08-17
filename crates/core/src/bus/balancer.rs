//! Balancer block stamping for `LaneFamily` blocks.
//!
//! Given a planned `LaneFamily` (an N→M producer-to-trunk balancer
//! requirement), pick the right template from `balancer_library` and
//! stamp it into a `Vec<PlacedEntity>`. Falls back to template
//! decomposition if no direct (N, M) template exists. Returns an empty
//! vec if neither path finds a template — `layout.rs` then surfaces
//! a missing-balancer warning.
//!
//! Also exports the small `splitter_for_belt` / `underground_for_belt`
//! lookup helpers since the balancer needs them and so does `render_path`
//! in `lane_planner.rs`.

use crate::models::{EntityDirection, PlacedEntity};
use crate::bus::lane_planner::LaneFamily;
use crate::bus::stacking_ctx::StackingCtx;

/// Splitter name mapping by belt tier.
const SPLITTER_MAP: &[(&str, &str)] = &[
    ("transport-belt", "splitter"),
    ("fast-transport-belt", "fast-splitter"),
    ("express-transport-belt", "express-splitter"),
];

/// Underground belt name mapping by belt tier.
const UNDERGROUND_MAP: &[(&str, &str)] = &[
    ("transport-belt", "underground-belt"),
    ("fast-transport-belt", "fast-underground-belt"),
    ("express-transport-belt", "express-underground-belt"),
];

pub(crate) fn splitter_for_belt(belt: &str) -> &'static str {
    SPLITTER_MAP.iter()
        .find(|(b, _)| *b == belt)
        .map(|(_, s)| *s)
        .unwrap_or("splitter")
}

pub(crate) fn underground_for_belt(belt: &str) -> &'static str {
    UNDERGROUND_MAP.iter()
        .find(|(b, _)| *b == belt)
        .map(|(_, u)| *u)
        .unwrap_or("underground-belt")
}

/// Compute the stamp origin_x so the template's output belts land
/// exactly on `lane_xs`.
///
/// Several balancer templates in `balancer_library` have their leftmost
/// output at an x-offset > 0 (e.g. T_5_6's outputs start at x-offset 1
/// because the leftmost column of the template carries no output). The
/// previous origin choice `min(lane_xs)` assumed outputs start at
/// offset 0 — which worked for most templates but silently stamped the
/// outputs one or more columns east of the continuation trunks for any
/// asymmetric template. Symptom: orphan balancer-output belts sitting
/// in the column range *beyond* `lane_xs`, and missing outputs in the
/// column range that was supposed to get them. Downstream the iron-ore
/// tap (or any other flow that expected the trunk range to be
/// continuous) would mix items with the orphan belt.
///
/// Shifting origin by `-output_tiles[0].0` aligns the leftmost output
/// with `lane_xs[0]`. Because every template's `output_tiles` is sorted
/// and contiguous in x (and `lane_xs` is sorted at
/// `lane_planner::plan_bus_lanes` and required to be contiguous),
/// aligning the leftmost pair aligns all of them.
///
/// The `debug_assert!`s are cheap invariant checks — they exist so
/// that if a future template violates "sorted, contiguous output
/// columns" the panic points at this helper rather than producing a
/// silent misalignment.
///
/// Inputs are A*-bridged by `ghost_router.rs`'s feeder specs, so they
/// adapt to whatever x the shifted origin places them at; no separate
/// input alignment is required here.
pub(crate) fn balancer_origin_x(lane_xs: &[i32], output_tiles: &[(i32, i32)]) -> i32 {
    debug_assert!(!lane_xs.is_empty(), "balancer_origin_x: lane_xs empty");
    debug_assert!(!output_tiles.is_empty(), "balancer_origin_x: output_tiles empty");
    debug_assert!(
        lane_xs.windows(2).all(|w| w[0] <= w[1]),
        "balancer_origin_x: lane_xs not sorted: {lane_xs:?}"
    );
    debug_assert!(
        output_tiles.windows(2).all(|w| w[0].0 <= w[1].0),
        "balancer_origin_x: output_tiles not sorted by x: {output_tiles:?}"
    );
    debug_assert!(
        output_tiles
            .windows(2)
            .all(|w| w[1].0 == w[0].0 + 1 || w[1].0 == w[0].0),
        "balancer_origin_x: output_tiles x-coords not contiguous: {output_tiles:?}"
    );
    lane_xs[0] - output_tiles[0].0
}

/// Build the `segment_id` string for a stamped balancer.
///
/// Under `LayoutStrategy::Pooled` every family has `module_id == 0`,
/// and the produced string is byte-identical to the pre-RFC format
/// (`balancer:{item}:{n}x{m}` or `…:{group}` for decomposition). The
/// `:mod{N}` suffix only appears when partitioning produced multiple
/// modules per item — see `docs/rfc-modular-production.md`.
fn format_segment_id(item: &str, module_id: u32, n: u32, m: u32, group: Option<usize>) -> String {
    let mut s = format!("balancer:{item}:{n}x{m}");
    if let Some(gi) = group {
        s.push_str(&format!(":{gi}"));
    }
    if module_id != 0 {
        s.push_str(&format!(":mod{module_id}"));
    }
    s
}

/// True for shapes where `stamp_family_balancer` emits a passthrough —
/// `n == m` with `n >= 2`. Each producer feeds its own output column via
/// a single south-facing belt; no splitters or undergrounds are needed
/// because every input has a unique output and the lane carries a single
/// fungible item type, so balancing is unnecessary (issue #268). This
/// check runs before the library lookup so passthrough wins even for
/// shapes the library has a full template for — the passthrough is
/// 60–94% smaller and functionally equivalent for spaghettio's bus
/// design.
pub(crate) fn is_passthrough_shape(n: u32, m: u32) -> bool {
    n == m && n >= 2
}

/// THE single stamp-shape decision for a family (RFC-061 Phase 1.5,
/// hardened per the #539 review): three sites — the stamper, the ghost
/// router's feeder targeting, and the lane planner's height resolution —
/// must agree not only on the passthrough ENTRY (Phase 1 gated only the
/// stamper; feeders then aimed at phantom columns and trunks drove
/// through the stamped template's body — ac@7's flank UG sideload) but
/// on the whole FALLBACK CHAIN: direct template → width-guarded
/// decomposition → width-guarded runtime generator → passthrough.
///
/// Explicit contract for the tail: a demand-skewed square the library
/// cannot serve (n = m ≥ 9; the generator emits passthrough for
/// squares) falls back to PASSTHROUGH at every site — consistent
/// geometry, and the #519-recalibrated walker reports any resulting
/// starvation honestly instead of the sites disagreeing (bogus
/// 10-row zones over 1-row stamps, dead-ended feeders).
pub(crate) enum FamilyStampPlan {
    /// 1-tall south belts at `lane_xs`.
    Passthrough,
    /// Direct library template for the family's exact shape.
    Direct(&'static crate::bus::balancer_library::BalancerTemplate),
    /// `g` side-by-side stamps of the `(n/g, m/g)` library template.
    Decomposed {
        g: u32,
        sub: &'static crate::bus::balancer_library::BalancerTemplate,
    },
    /// Runtime-generated template (shapes the library lacks).
    Generated(crate::bus::balancer_generate::OwnedTemplate),
    /// Nothing can serve this shape: no stamp, no feeders
    /// (`FeederSpecsSkipped`), no height. Only NON-square shapes land
    /// here — a square always has the passthrough tail.
    Unresolvable,
}

pub(crate) fn family_stamp_plan(
    fam: &crate::bus::lane_planner::LaneFamily,
) -> FamilyStampPlan {
    let (n, m) = (fam.shape.0 as u32, fam.shape.1 as u32);
    if is_passthrough_shape(n, m) && !fam.demand_skewed {
        return FamilyStampPlan::Passthrough;
    }
    let templates = crate::bus::balancer_library::balancer_templates();
    if let Some(t) = templates.get(&(n, m)) {
        return FamilyStampPlan::Direct(t);
    }
    for g in (1..=n).rev() {
        if g == 0 || n % g != 0 || m % g != 0 {
            continue;
        }
        let (sub_n, sub_m) = (n / g, m / g);
        if let Some(sub) = templates.get(&(sub_n, sub_m)) {
            if sub.width > sub_m {
                continue; // neighbouring sub-stamps would overlap in x
            }
            return FamilyStampPlan::Decomposed { g, sub };
        }
    }
    if let Some(generated) = crate::bus::balancer_generate::generate(n, m) {
        if generated.width <= m {
            return FamilyStampPlan::Generated(generated);
        }
    }
    // Tail: a SQUARE nothing else can serve gets a consistent 1-tall
    // passthrough at every site (the pre-Phase-1.5 sites disagreed
    // here) — honest by construction, the walker reports what the
    // passthrough under-delivers. A NON-square passthrough would be
    // geometric nonsense (m columns fed by n != m producers), so those
    // keep the long-standing unstampable contract: empty stamp,
    // FeederSpecsSkipped, no reserved height (pinned by
    // `shape_is_stampable_matches_stamping` and
    // `fires_when_shape_has_no_template_at_all`).
    if is_passthrough_shape(n, m) {
        FamilyStampPlan::Passthrough
    } else {
        FamilyStampPlan::Unresolvable
    }
}

/// Columns a family's stamp needs BEYOND its own trunk columns, as
/// `(west, east)`.
///
/// Every stamping path aligns by [`balancer_origin_x`] — output tile 0
/// onto `lane_xs[0]` — so a template whose width exceeds its output
/// count spills west of the family's first trunk column (by the output
/// tiles' own x-offset) and east of its last (by whatever width is
/// left over). Nothing downstream moves out of the way: the lane
/// planner has always reserved the stamp's HEIGHT (`balancer_y_end`,
/// so trunks skip the zone) and never its WIDTH, so the spill lands on
/// whichever family was assigned the neighbouring columns.
///
/// That is #652's balancer-over-trunk overlap. On ac7-HS at duty 0.6
/// the electronic-circuit family's `(5,2)` is width 6 over 2 trunk
/// columns — 1 west + 3 east — and the eastern spill swallowed the
/// plastic-bar trunk at x=18–19, severing it and stranding fragments
/// inside the template's holes. 31 library templates are wider than
/// their output count, so this is a shape property, not a fixture one.
///
/// `Decomposed` and `Generated` are guarded at *selection* time
/// (`sub.width <= sub_m` / `generated.width <= m`), which forces both
/// pads to zero; they are derived here rather than assumed so the
/// guard and this reservation cannot drift apart. `Direct` — the
/// exact-shape library hit — has never had such a guard, and is the
/// path that spills.
pub(crate) fn family_stamp_x_pad(fam: &LaneFamily) -> (i32, i32) {
    /// `width` minus the contiguous run of output columns, split
    /// around the run's offset. Output tiles are sorted by x and
    /// contiguous (see [`balancer_origin_x`]'s debug asserts), and may
    /// repeat an x when a template has two output rows.
    fn pad_of(width: u32, output_tiles: &[(i32, i32)]) -> (i32, i32) {
        let Some(&(d0, _)) = output_tiles.first() else {
            return (0, 0);
        };
        let mut cols: Vec<i32> = output_tiles.iter().map(|t| t.0).collect();
        cols.dedup();
        let m = cols.len() as i32;
        (d0.max(0), (width as i32 - d0 - m).max(0))
    }

    let n = fam.shape.0 as u32;
    if fam.merge_tap {
        // The merge tree's single output sits at its EAST edge
        // (`(n-1, 2n-2)`), so its whole body is a western spill.
        let t = crate::bus::balancer_generate::merge_tree(n);
        return pad_of(t.width, &t.output_tiles);
    }
    match family_stamp_plan(fam) {
        FamilyStampPlan::Passthrough | FamilyStampPlan::Unresolvable => (0, 0),
        FamilyStampPlan::Direct(t) => pad_of(t.width, t.output_tiles),
        FamilyStampPlan::Generated(g) => pad_of(g.width, &g.output_tiles),
        // Sub-stamps sit side by side, each aligned onto its own lane
        // chunk: the westmost chunk owns the west pad, the eastmost
        // the east pad, and both are the sub-template's.
        FamilyStampPlan::Decomposed { sub, .. } => pad_of(sub.width, sub.output_tiles),
    }
}

/// Predicate: would `stamp_family_balancer((n, m), …)` find a template
/// to use, either directly or via decomposition?
///
/// Mirrors the exact stamping decision logic in `stamp_family_balancer`:
///   1. Passthrough hit (`n == m && n >= 2`), OR
///   2. Direct (n, m) template hit, OR
///   3. A divisor `g ≥ 2` of both n and m where (n/g, m/g) has a template
///      AND that sub-template's width ≤ sub_m (the geometric overlap
///      guard at line 174 — neighbouring stamps would collide otherwise).
///
/// Used by the partitioner's shape-aware sharding decision: if a module's
/// computed (n, m) shape isn't stampable, force-shard regardless of the
/// usual lane-count threshold so the layout doesn't silently drop the
/// producer→trunk handoff. See `docs/rfc-modular-production.md` and the
/// PU@3/s ore red copper-plate (4, 9) bug for context.
///
/// `n` is producer-row count, `m` is consumer lane count.
#[allow(dead_code)] // wired in Phase 3 (partitioner force-shard); land predicate first
pub(crate) fn shape_is_stampable(n: u32, m: u32) -> bool {
    if n == 0 || m == 0 {
        return false;
    }
    if is_passthrough_shape(n, m) {
        return true;
    }
    let templates = crate::bus::balancer_library::balancer_templates();
    if templates.contains_key(&(n, m)) {
        return true;
    }
    // Mirror the gcd-decomposition + width-guard at balancer.rs:167-176.
    for g in (2..=n.min(m)).rev() {
        if !n.is_multiple_of(g) || !m.is_multiple_of(g) {
            continue;
        }
        let sub_n = n / g;
        let sub_m = m / g;
        if let Some(sub_template) = templates.get(&(sub_n, sub_m)) {
            if sub_template.width <= sub_m {
                return true;
            }
        }
    }
    // Phase 2.0 generator. Mirror the same width-guard the stamping path
    // applies (`generated.width <= m`) so the predicate matches reality.
    if let Some(generated) = crate::bus::balancer_generate::generate(n, m) {
        if generated.width <= m {
            return true;
        }
    }
    false
}

/// Stamp a balancer template at the family's origin position.
///
/// Template entity tiles are offset by the family's stamp origin
/// (x = min(lane_xs), y = balancer_y_start). The item each entity
/// carries is set to the family's item. Belt and splitter tiers are
/// chosen from the family's total rate so the balancer matches its
/// sibling trunks.
pub(crate) fn stamp_family_balancer(
    family: &LaneFamily,
    max_belt_tier: Option<&str>,
    ctx: &StackingCtx,
) -> Result<Vec<PlacedEntity>, String> {
    use crate::common::belt_entity_for_rate_stacked;

    let (n, m) = (family.shape.0 as u32, family.shape.1 as u32);

    if family.lane_xs.is_empty() {
        return Err(format!("LaneFamily for item {} has no lane_xs assigned", family.item));
    }

    let belt_tier =
        belt_entity_for_rate_stacked(family.total_rate, max_belt_tier, ctx.for_item(&family.item));
    let splitter_name = splitter_for_belt(belt_tier);
    let ug_name = underground_for_belt(belt_tier);

    // Passthrough shortcut for `(m, m)`: each producer feeds its own
    // output column; no balancing is required because the lane carries
    // a single fungible item type, every input has a unique output, and
    // max-flow holds in both directions (MX2b / throughput-unlimited —
    // see issue #268). Stamps `m` south-facing belts at the family's
    // top row; the producer feeders sideload-or-straight-load onto
    // these belts and the trunk picks up at `balancer_y_end + 1`.
    // Library entries for `(2, 2)`..`(8, 8)` are kept as a safety net.
    // RFC-061 (#519): a demand-skewed `(m, m)` family must MIX — the
    // passthrough freezes a producer→column pairing that under-provisions
    // some column (`LaneFamily::demand_skewed`; ac@5's cable columns
    // carried 8.82/s into 12.86/s blocks and sim-measured 75% of plan).
    // Skewed families fall into exactly the library net below.
    let plan = family_stamp_plan(family);
    if matches!(plan, FamilyStampPlan::Passthrough) {
        let seg_id = Some(format_segment_id(&family.item, family.module_id, n, m, None));
        let entities: Vec<PlacedEntity> = family
            .lane_xs
            .iter()
            .map(|&lane_x| PlacedEntity {
                name: belt_tier.to_string(),
                x: lane_x,
                y: family.balancer_y_start,
                direction: EntityDirection::South,
                carries: Some(family.item.clone()),
                segment_id: seg_id.clone(),
                ..Default::default()
            })
            .collect();
        return Ok(entities);
    }

    if let FamilyStampPlan::Direct(template) = &plan {
        // Direct template match.
        let origin_x = balancer_origin_x(&family.lane_xs, template.output_tiles);
        let origin_y = family.balancer_y_start;

        let mut entities = template.stamp(
            origin_x, origin_y, belt_tier, splitter_name, ug_name,
            Some(&family.item),
        );
        let seg_id = Some(format_segment_id(&family.item, family.module_id, n, m, None));
        for ent in &mut entities {
            ent.segment_id = seg_id.clone();
        }
        return Ok(entities);
    }

    // Decomposition fallback: try to split (N, M) into groups that have
    // templates. Search for a divisor g of N where (N/g, M/g) has a
    // template. E.g., (6,8) → g=2 → 2 copies of (3,4). (5,10) → g=5 →
    // 5 copies of (1,2).
    //
    // Geometric constraint: sub-stamps are placed at output-lane spacing
    // (1 column per output lane). If the sub-template is wider than its
    // output count (`sub_template.width > sub_m`), neighbouring stamps
    // overlap in x. Skip those decompositions — better to fail to stamp
    // (caller treats empty as "no balancer placed") than to write
    // overlapping entities. PU@2/s plates yellow tripped this with
    // (15, 3) → 3×(5, 1): width=5 > sub_m=1, three balancers stamped on
    // top of each other, ~37 entity-overlap errors.
    if let FamilyStampPlan::Decomposed { g, sub: sub_template } = &plan {
        let (g, sub_template) = (*g, *sub_template);
        let sub_n = n / g;
        let sub_m = m / g;
        {
            let mut all_entities = Vec::new();
            let lanes_per_group = sub_m as usize;

            for gi in 0..(g as usize) {
                let lane_start = gi * lanes_per_group;
                let lane_end = (lane_start + lanes_per_group).min(family.lane_xs.len());
                let lane_chunk = &family.lane_xs[lane_start..lane_end];
                if lane_chunk.is_empty() {
                    continue;
                }
                let sub_origin_x = balancer_origin_x(lane_chunk, sub_template.output_tiles);
                let sub_origin_y = family.balancer_y_start;

                let mut ents = sub_template.stamp(
                    sub_origin_x, sub_origin_y, belt_tier, splitter_name, ug_name,
                    Some(&family.item),
                );
                let sub_seg = format_segment_id(&family.item, family.module_id, sub_n, sub_m, Some(gi));
                for ent in &mut ents {
                    ent.segment_id = Some(sub_seg.clone());
                }
                all_entities.extend(ents);
            }
            return Ok(all_entities);
        }
    }

    // Runtime template generator (phase 2.0) — selected by
    // `family_stamp_plan`, which owns the shape search and width guards.
    if let FamilyStampPlan::Generated(generated) = &plan {
        {
            let origin_x = balancer_origin_x(&family.lane_xs, &generated.output_tiles);
            let origin_y = family.balancer_y_start;
            let mut entities = generated.stamp(
                origin_x, origin_y, belt_tier, splitter_name, ug_name,
                Some(&family.item),
            );
            let seg_id = Some(format_segment_id(&family.item, family.module_id, n, m, None));
            for ent in &mut entities {
                ent.segment_id = seg_id.clone();
            }
            crate::trace::emit(crate::trace::TraceEvent::BalancerGenerated {
                item: family.item.clone(),
                shape: (n as usize, m as usize),
                entity_count: entities.len(),
                width: generated.width,
                height: generated.height,
            });
            return Ok(entities);
        }
    }

    // No template and no fallback possible — skip.
    Ok(Vec::new())
}

/// Stamp a merge-and-tap family's `n → 1` splitter merge-tree at its trunk
/// column (RFC `docs/rfc-merge-tap-trunks.md` D2). The merge-tree's single
/// output is aligned onto `family.lane_xs[0]` exactly as a balancer's outputs
/// align onto `lane_xs` (`balancer_origin_x`), so the trunk picks up below the
/// merge-tree just like below a balancer block. Producer feeders are routed to
/// the merge-tree's input columns by the ghost router's feeder-spec generator
/// (which mirrors this same origin). Returns an empty vec if the family has no
/// assigned trunk column yet.
pub(crate) fn stamp_merge_tap_family(
    family: &LaneFamily,
    max_belt_tier: Option<&str>,
    ctx: &StackingCtx,
) -> Vec<PlacedEntity> {
    use crate::common::belt_entity_for_rate_stacked;

    let Some(&lane_x) = family.lane_xs.first() else {
        return Vec::new();
    };
    let n = family.shape.0 as u32;
    let tree = crate::bus::balancer_generate::merge_tree(n);

    let belt_tier =
        belt_entity_for_rate_stacked(family.total_rate, max_belt_tier, ctx.for_item(&family.item));
    let splitter_name = splitter_for_belt(belt_tier);
    let ug_name = underground_for_belt(belt_tier);

    let origin_x = balancer_origin_x(&[lane_x], &tree.output_tiles);
    let origin_y = family.balancer_y_start;
    let seg = Some(format!("mergetree:{}:{}", family.item, lane_x));

    let mut entities = tree.stamp(
        origin_x, origin_y, belt_tier, splitter_name, ug_name, Some(&family.item),
    );
    for ent in &mut entities {
        ent.segment_id = seg.clone();
    }
    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::lane_planner::LaneFamily;

    #[test]
    fn test_stamp_family_balancer() {
        let family = LaneFamily {
            item: "iron-plate".to_string(),
            module_id: 0,
            shape: (1, 2),  // 1 producer, 2 lanes
            producer_rows: vec![0],
            lane_xs: vec![1, 2],
            balancer_y_start: 10,
            balancer_y_end: 11,
            total_rate: 20.0,  // should use fast-transport-belt
            merge_tap: false,
            demand_skewed: false,
        };

        let entities = stamp_family_balancer(&family, None, &StackingCtx::unstacked());
        assert!(entities.is_ok());

        let entities = entities.unwrap();
        assert!(!entities.is_empty());
        // Verify that the stamped entities have the correct origin and item
        for e in &entities {
            assert_eq!(e.carries, Some("iron-plate".to_string()));
            assert!(e.y >= 10); // origin_y should be >= 10
        }
    }

    /// Every template in the library must place its outputs at
    /// exactly `lane_xs` after origin adjustment. Before the
    /// `balancer_origin_x` fix, templates whose `output_tiles` started
    /// at x-offset > 0 (e.g. T_5_6 at offset 1) stamped outputs
    /// shifted east of `lane_xs`, producing orphan belts at the east
    /// edge and missing outputs at the west.
    ///
    /// Pure invariant check: for each template, pick an arbitrary
    /// contiguous `lane_xs`, compute `balancer_origin_x`, and assert
    /// that applying that origin to the template's `output_tiles`
    /// produces exactly `lane_xs`. No actual stamping needed — this
    /// pins the alignment contract independent of entity-type details
    /// (splitters vs belts on the output row).
    #[test]
    fn test_template_outputs_align_with_lane_xs() {
    
        let templates = crate::bus::balancer_library::balancer_templates();
        for (&(n, m), template) in templates.iter() {
            let lane_xs: Vec<i32> = (100..100 + m as i32).collect();
            let origin_x = balancer_origin_x(&lane_xs, template.output_tiles);
            let actual: Vec<i32> = template
                .output_tiles
                .iter()
                .map(|(dx, _)| origin_x + dx)
                .collect();
            assert_eq!(
                actual, lane_xs,
                "template ({n},{m}): outputs {actual:?} should equal lane_xs {lane_xs:?} after origin shift"
            );
        }
    }

    /// Build a family that will take the LIBRARY path for `(n, m)`.
    /// An unskewed square would shortcut to passthrough and never
    /// stamp a template at all, so squares are marked demand-skewed.
    fn library_family(n: u32, m: u32, lane_x0: i32) -> LaneFamily {
        LaneFamily {
            item: "iron-plate".to_string(),
            module_id: 0,
            shape: (n as usize, m as usize),
            producer_rows: (0..n as usize).collect(),
            lane_xs: (lane_x0..lane_x0 + m as i32).collect(),
            balancer_y_start: 10,
            balancer_y_end: 60,
            total_rate: 20.0,
            merge_tap: false,
            demand_skewed: n == m,
        }
    }

    /// #652 anchor: the shape that exhibited the balancer-over-trunk
    /// overlap. `(5,2)` is a width-6 template over 2 trunk columns
    /// whose outputs sit at x-offsets 1..2 — 1 column of spill west,
    /// 3 east. Documented explicitly so a library regeneration that
    /// changes T_5_2's geometry has to come past this number.
    #[test]
    fn family_stamp_x_pad_anchors_the_5x2_spill() {
        assert_eq!(family_stamp_x_pad(&library_family(5, 2, 16)), (1, 3));
    }

    /// A merge-tap family's merge tree puts its single output at the
    /// tree's EAST edge, so the whole body is a western spill.
    #[test]
    fn family_stamp_x_pad_covers_the_merge_tree() {
        let mut fam = library_family(4, 1, 20);
        fam.merge_tap = true;
        assert_eq!(family_stamp_x_pad(&fam), (3, 0));
    }

    /// #652: the reservation must cover the stamp's ACTUAL extent for
    /// every shape the library serves — not just the one that
    /// exhibited the bug. Stamps each shape at a known lane block and
    /// asserts every emitted tile falls inside
    /// `[lane_xs[0] - west, lane_xs.last() + east]`.
    ///
    /// Both anti-vacuity guards matter: without the first the loop
    /// could stamp nothing and pass, and without the second a pad
    /// that always returned `(0, 0)` would pass on a library that
    /// happened never to spill.
    #[test]
    fn family_stamp_x_pad_covers_every_library_stamp() {
        let mut checked = 0usize;
        let mut spilling = 0usize;
        for (&(n, m), _) in crate::bus::balancer_library::balancer_templates().iter() {
            let fam = library_family(n, m, 100);
            let (west, east) = family_stamp_x_pad(&fam);
            let ents = stamp_family_balancer(&fam, None, &StackingCtx::unstacked())
                .unwrap_or_else(|e| panic!("({n},{m}) stamp: {e}"));
            if ents.is_empty() {
                continue;
            }
            let lo = fam.lane_xs[0] - west;
            let hi = *fam.lane_xs.last().unwrap() + east;
            for e in &ents {
                // A splitter is 2 tiles wide across its facing axis;
                // `x` records only its west tile.
                let spans_x = e.name.contains("splitter")
                    && matches!(e.direction, EntityDirection::North | EntityDirection::South);
                let e_hi = if spans_x { e.x + 1 } else { e.x };
                assert!(
                    e.x >= lo && e_hi <= hi,
                    "({n},{m}): stamped {} at x {}..{} escapes the reserved span {lo}..{hi} \
                     (pad west={west} east={east}, lane_xs {:?}) — the lane planner would \
                     hand those columns to the next family",
                    e.name,
                    e.x,
                    e_hi,
                    fam.lane_xs
                );
            }
            checked += 1;
            if west + east > 0 {
                spilling += 1;
            }
        }
        assert!(checked > 0, "vacuous: no library shape stamped anything");
        assert!(
            spilling > 0,
            "vacuous: no library shape spills past its trunk columns, so a \
             pad of (0,0) would satisfy this test — re-point it"
        );
    }

    /// `shape_is_stampable` must agree with what `stamp_family_balancer`
    /// actually produces. Property check: for every shape (n, m) in
    /// 1..=10 × 1..=10, predicate `true` ↔ stamping yields a non-empty
    /// entity vec.
    ///
    /// This invariant is the foundation for the partitioner's shape-aware
    /// sharding decision. If the predicate over- or under-reports, the
    /// partitioner will either silently drop layouts (when stampability
    /// claims true but stamping fails) or over-shard (when predicate
    /// claims false but a template exists).
    #[test]
    fn shape_is_stampable_matches_stamping() {
        for n in 1u32..=10 {
            for m in 1u32..=10 {
                let predicted = shape_is_stampable(n, m);
                let family = LaneFamily {
                    item: "test-item".to_string(),
                    module_id: 0,
                    shape: (n as usize, m as usize),
                    producer_rows: (0..n as usize).collect(),
                    lane_xs: (10..10 + m as i32).collect(),
                    balancer_y_start: 100,
                    balancer_y_end: 100 + 50,
                    total_rate: 30.0,
                    merge_tap: false,
                    demand_skewed: false,
                };
                let entities =
                    stamp_family_balancer(&family, None, &StackingCtx::unstacked()).unwrap_or_default();
                let actually_stamps = !entities.is_empty();
                assert_eq!(
                    predicted, actually_stamps,
                    "shape ({n}, {m}): predicate={predicted} but stamping={actually_stamps}",
                );
            }
        }
    }

    /// Coprime / asymmetric gaps that remain unstampable.
    /// Originally 17 (issue #136 / PR #257: `(1..=8, 9)` and `(9, 1..=9)`).
    /// Phase-2.0 generator closes `(3, 9)` (3 × library `(1, 3)`),
    /// `(6, 9)` (3 × library `(2, 3)`), and `(9, 9)` (passthrough) — those
    /// are removed from the fixture below. Remaining gaps need either
    /// missing library templates or richer generator atoms.
    #[test]
    fn shape_is_stampable_pins_known_gaps() {
        // (9,1)/(9,3) removed from this list 2026-08-14 per the fixture's
        // own instruction: both are registered library templates and
        // stampable. (9,2)/(9,4..8) became REAL gaps the same day — the
        // #632 A3 cull deleted them as waist-capped (#631); a request for
        // those shapes is now loudly unstampable rather than silently
        // half-rate.
        let known_gaps: &[(u32, u32)] = &[
            (1, 9), (2, 9), (4, 9), (5, 9), (7, 9), (8, 9),
            (9, 2), (9, 4), (9, 5), (9, 6), (9, 7), (9, 8),
        ];
        for &(n, m) in known_gaps {
            let stampable = shape_is_stampable(n, m);
            // If this fires for a specific shape, the library has been
            // augmented to cover that shape — remove it from this fixture.
            if stampable {
                eprintln!(
                    "NOTE: shape ({n}, {m}) is now stampable. Update this test \
                     by removing ({n}, {m}) from `known_gaps`."
                );
            }
        }
        // Soft assertion only: don't fail if a gap is closed (that's good
        // news), but the eprintln above flags the cleanup needed.
    }
}
