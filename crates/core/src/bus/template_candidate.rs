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
use crate::celldb::{self, DerivedConstraints};
use crate::common::{belt_throughput, entity_size, is_inserter, is_machine_entity, QualityTier};
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
        let m = &solver_result.machines[0];
        let need = m.count.ceil().max(1.0) as u32;
        let cap_rate = opts
            .max_belt_tier
            .as_deref()
            .map(belt_throughput)
            .unwrap_or(f64::INFINITY);

        let entry = celldb::query_unit(&m.recipe, &m.entity, need, None)
            .into_iter()
            .find(|e| {
                DerivedConstraints::of(e)
                    .belt_tiers
                    .iter()
                    .all(|b| belt_throughput(b) <= cap_rate)
            })
            .ok_or_else(|| {
                format!(
                    "no celldb entry for ({}, {}, >= {need}) within belt cap",
                    m.recipe, m.entity
                )
            })?;

        // Stamp the fragment, then place poles exactly the way the shipping
        // pipeline does — poles LAST, never obstacles (layout.rs invariant).
        let mut entities = entry.entities.clone();
        let mut occupied: FxHashSet<(i32, i32)> = FxHashSet::default();
        let mut machines: Vec<(i32, i32, i32)> = Vec::new();
        let mut inserters: Vec<(i32, i32)> = Vec::new();
        for e in &entities {
            let (w, h) = entity_size(&e.name);
            for dx in 0..w as i32 {
                for dy in 0..h as i32 {
                    occupied.insert((e.x + dx, e.y + dy));
                }
            }
            if is_machine_entity(&e.name) {
                machines.push((e.x, e.y, w as i32));
            } else if is_inserter(&e.name) {
                inserters.push((e.x, e.y));
            }
        }
        let (poles, _) = layout::place_poles(
            &machines,
            &inserters,
            &occupied,
            &[],
            QualityTier::Normal,
        );
        entities.extend(poles);

        let width = entities
            .iter()
            .map(|e| e.x + entity_size(&e.name).0 as i32)
            .max()
            .unwrap_or(0);
        let height = entities
            .iter()
            .map(|e| e.y + entity_size(&e.name).1 as i32)
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
