//! RFC-067 Phase 3: the celldb template candidate — a
//! [`DecompositionCandidate`] that produces a layout by STAMPING a stored
//! implementation instead of running row placement.
//!
//! Ships INERT by construction: `candidate_runner` is a parallel harness
//! consumed only by tests (its own module docs), and nothing in
//! `build_bus_layout`'s selection references this producer. Promotion into
//! shipping selection is a later decision gated on sim anchoring — the
//! standing #519/#520 firewall, restated in RFC-067 K67-4.
//!
//! v1 scope, deliberately narrow: single-machine-group solves only (the
//! store's unit motifs). Multi-group composition — stamping several
//! fragments and routing fabric between them — is exactly the hard problem
//! RFC-057 died on, and it is NOT smuggled in here; a multi-group solve is
//! a clean refusal, which `run_candidate_field` records as a refused
//! candidate without failing the field.
//!
//! Belt tier is a hard user constraint (never a search axis): an entry
//! whose derived belt vocabulary exceeds `opts.max_belt_tier` is
//! inadmissible for that call, full stop.

use super::decomposition_search::DecompositionCandidate;
use super::layout::{self, LayoutOptions};
use crate::celldb::{self, DerivedConstraints, Motif};
use crate::common::{belt_throughput, is_inserter, is_machine_entity, QualityTier};
use crate::models::{LayoutResult, SolverResult};
use rustc_hash::FxHashSet;

pub struct TemplateCandidate;

impl DecompositionCandidate for TemplateCandidate {
    fn name(&self) -> &str {
        "celldb-template"
    }

    fn produce(
        &self,
        solver_result: &SolverResult,
        opts: &LayoutOptions,
    ) -> Result<LayoutResult, String> {
        if solver_result.machines.len() != 1 {
            return Err(format!(
                "celldb-template v1 covers single-group solves; this one has {} groups",
                solver_result.machines.len()
            ));
        }
        // Refuse-by-name for options the stamp cannot honor — the
        // build_bus_layout convention (its stacking refusal), never silent
        // degradation (round-5 review). A stored fragment IS its inserters
        // and belts; options that would change them cannot apply to a stamp.
        if opts.stacking > 1 {
            return Err("celldb-template cannot honor stacking > 1 (fragment is pre-built)".into());
        }
        if opts.quality != QualityTier::Normal {
            return Err("celldb-template v1 stamps Normal-quality fragments only".into());
        }
        let m = &solver_result.machines[0];
        let need = m.count.ceil().max(1.0) as u32;
        let cap_rate = opts
            .max_belt_tier
            .as_deref()
            .map(belt_throughput)
            .unwrap_or(f64::INFINITY);

        // Map every transport prototype to its SURFACE tier before the cap
        // test: belt_throughput returns the yellow default for names it
        // does not know (UGs, splitters), which would pass an express-UG
        // entry under a yellow cap — silently violating the hard-constraint
        // contract (round-1 review on this PR). Unknown names REFUSE.
        let surface_tier_rate = |name: &str| -> Option<f64> {
            let surface = if let Some(base) = name.strip_suffix("-underground-belt") {
                format!("{base}-transport-belt")
            } else if name == "underground-belt" {
                "transport-belt".to_string()
            } else if let Some(base) = name.strip_suffix("-splitter") {
                format!("{base}-transport-belt")
            } else if name == "splitter" {
                "transport-belt".to_string()
            } else {
                name.to_string()
            };
            crate::common::BELT_TIERS
                .iter()
                .find(|(n, _)| *n == surface)
                .map(|(_, r)| *r)
        };
        let entry = celldb::query_unit(&m.recipe, &m.entity, need, None)
            .into_iter()
            .find(|e| {
                DerivedConstraints::of(e)
                    .belt_tiers
                    .iter()
                    .all(|b| surface_tier_rate(b).is_some_and(|r| r <= cap_rate))
            })
            .ok_or_else(|| {
                format!(
                    "no celldb entry for ({}, {}, >= {need}) within belt cap",
                    m.recipe, m.entity
                )
            })?;
        // v1 stamps EXACT-count matches only. A count>need entry would
        // silently overproduce and the measured verdicts would score
        // overproduction, not fragment quality — the demand-matched
        // harness lesson, enforced in the producer itself (round-4
        // review). Count ladders relax this when Phase 3 reopens.
        // Unit is the only reachable arm: query_unit filters Fused out
        // before returning (a Fused match arm here would be dead code
        // reading as unbuilt support — round-5 review).
        let Motif::Unit { count: entry_count, .. } = &entry.motif else {
            return Err("query_unit returned a non-unit motif (unreachable)".into());
        };
        let entry_count = *entry_count;
        if entry_count != need {
            return Err(format!(
                "smallest entry has {entry_count} machines for a {need}-machine demand; \
                 v1 refuses inexact stamps (count ladders are the reopening path)"
            ));
        }

        // Stamp the fragment, then place poles exactly the way the shipping
        // pipeline does — poles LAST, never obstacles (layout.rs invariant).
        let mut entities = entry.entities.clone();
        let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut machines: Vec<(i32, i32, i32)> = Vec::new();
        let mut inserters: Vec<(i32, i32)> = Vec::new();
        for e in &entities {
            // Direction-aware, matching the store's own invariant pass —
            // entity_size is direction-blind and the two diverging meant
            // check_entry could certify geometry this stamp then broke
            // (round-3 review on this PR).
            let (w, h) = crate::common::oriented_entity_dims(&e.name, e.direction);
            let (w, h) = (w as u32, h as u32);
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    occupied.insert((e.x + dx, e.y + dy));
                }
            }
            // entity_size reports (1,1) for splitters; the shipping
            // pipeline reserves the second tile via splitter_second_tile —
            // without this a pole could stamp onto it (latent: no current
            // seed has a splitter; round-2 review on this PR).
            if crate::common::is_splitter(&e.name) {
                occupied.insert(crate::common::splitter_second_tile(e));
            }
            if is_machine_entity(&e.name) {
                // place_poles' tuple is (center_x, top_y, HEIGHT) — mirror
                // layout.rs:1209 exactly. Passing (x, y, width) was silent
                // on the all-square current seeds and wrong for any
                // non-square machine (round-1 review on this PR).
                machines.push((e.x + w as i32 / 2, e.y, h as i32));
            } else if is_inserter(&e.name) {
                inserters.push((e.x, e.y));
            }
        }
        let (poles, uncovered) = layout::place_poles(
            &machines,
            &inserters,
            &occupied,
            &[],
            QualityTier::Normal,
        );
        // place_poles never errors — it RETURNS the inserters it could not
        // cover, and discarding that set shipped underpowered fragments
        // surfaced only as a validation warning nothing asserted on
        // (round-3 review). An uncoverable fragment is a refusal.
        if !uncovered.is_empty() {
            // "subject(s)": place_poles' give_up set holds unmopable
            // inserters AND uncoverable machine centers (fluid-only rows,
            // the #400 pass) — calling them all inserters misdiagnosed the
            // one case this message exists for (round-6 review).
            return Err(format!(
                "pole placement left {} coverage subject(s) unreachable at {:?}",
                uncovered.len(),
                &uncovered[..uncovered.len().min(4)]
            ));
        }
        entities.extend(poles);

        // Same direction-aware dims as the occupancy pass — a rotated
        // non-square machine must not report a smaller bbox than it stamps
        // (round-4 review).
        let width = entities
            .iter()
            .map(|e| e.x + crate::common::oriented_entity_dims(&e.name, e.direction).0)
            .max()
            .unwrap_or(0);
        let height = entities
            .iter()
            .map(|e| e.y + crate::common::oriented_entity_dims(&e.name, e.direction).1)
            .max()
            .unwrap_or(0);

        Ok(LayoutResult {
            entities,
            width,
            height,
            ..Default::default()
        })
    }
}
