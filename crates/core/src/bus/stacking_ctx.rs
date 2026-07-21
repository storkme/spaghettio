//! RFC-046 belt-stacking context.
//!
//! Carries the layout's belt stack size together with the statically
//! derived set of **stacking-exempt items** — the family-level exemption
//! that keeps uniform ×S capacity crediting sound (see
//! `docs/rfc-046-belt-stacking.md`, "Known hole" / decision log
//! 2026-07-21). An item is exempt when any producer row of it is
//! unstackable-classed:
//!
//! - **self-loop rows** (kovarex-class): the minor export is a reach-2
//!   long-handed belt-drop (BS5 — cannot stack) and shares its item
//!   (hence its lane family) with the stacked-capable major output, so
//!   the whole family must plan unstacked or trunks would carry mixed
//!   stacked/unstacked flow, which uniform crediting cannot express
//!   (fractional occupancy). All the spec's outputs and self-loop items
//!   are exempt.
//! - **voider / scrap-recycling rows** (kill 4): recycler direct belt
//!   ejection only stacks when ≥2 of an item type are buffered — not
//!   guaranteed for probabilistic multi-product outputs — so their
//!   output belts keep unstacked capacity (documented conservatism).
//! - **secondary solid outputs** (RFC Fulgora D2b): index ≥1 solid
//!   outputs are extracted by a fixed reach-2 long-handed inserter
//!   (templates.rs `secondary_output`) — cannot stack.
//!
//! Exemption is item-keyed (not `(item, module_id)`): coarser than
//! strictly necessary under partitioned strategies, but conservative —
//! an exempt item plans unstacked everywhere. Per-lane dynamic
//! stackedness stays deferred (RFC-046 Phase 3).

use rustc_hash::FxHashSet;

use crate::models::SolverResult;

/// Belt-stacking planning context, derived once per layout run.
#[derive(Debug, Clone)]
pub struct StackingCtx {
    /// The layout's belt stack size (`LayoutOptions.stacking`), clamped
    /// to the physical 1..=4.
    stacking: u8,
    /// Items whose lane families plan at unstacked capacity.
    exempt: FxHashSet<String>,
}

impl StackingCtx {
    /// The unstacked context: `for_item` returns 1 for everything.
    pub fn unstacked() -> Self {
        Self { stacking: 1, exempt: FxHashSet::default() }
    }

    /// Derive the context from the solver result. At `stacking ≤ 1` the
    /// exempt set is left empty (everything is unstacked anyway), which
    /// also keeps the S=1 path allocation-free.
    pub fn derive(sr: &SolverResult, stacking: u8) -> Self {
        let stacking = stacking.clamp(1, 4);
        let mut exempt = FxHashSet::default();
        if stacking > 1 {
            for spec in &sr.machines {
                // Recycler entity covers both voider and scrap-recycling
                // rows (`placer::row_kind` short-circuits on the same
                // facts). Their outputs are recycler-ejected (kill 4).
                let unstackable_row =
                    spec.entity == "recycler" || !spec.self_loop.is_empty();
                let mut solid_idx = 0usize;
                for out in &spec.outputs {
                    if out.is_fluid {
                        continue;
                    }
                    if unstackable_row || solid_idx >= 1 {
                        exempt.insert(out.item.clone());
                    }
                    solid_idx += 1;
                }
                // Voider rows export nothing (`outputs` is always empty);
                // their unstacked surface is the row-INTERNAL recirc belt,
                // which carries the voided INPUT item recycler-ejected.
                if spec.voider {
                    for inp in &spec.inputs {
                        if !inp.is_fluid {
                            exempt.insert(inp.item.clone());
                        }
                    }
                }
                for lp in &spec.self_loop {
                    if !lp.is_fluid {
                        exempt.insert(lp.item.clone());
                    }
                }
            }
        }
        Self { stacking, exempt }
    }

    /// The layout-wide belt stack size (1..=4).
    pub fn stacking(&self) -> u8 {
        self.stacking
    }

    /// Effective belt stack size for belts carrying `item`: 1 for exempt
    /// items, the layout's stack size otherwise.
    pub fn for_item(&self, item: &str) -> u8 {
        if self.exempt.contains(item) {
            1
        } else {
            self.stacking
        }
    }

    /// Whether `item`'s family is stacking-exempt.
    pub fn is_exempt(&self, item: &str) -> bool {
        self.exempt.contains(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemFlow, MachineSpec, SelfLoopFlow};

    fn flow(item: &str, fluid: bool) -> ItemFlow {
        ItemFlow { item: item.into(), rate: 1.0, is_fluid: fluid, module_id: 0 }
    }

    fn plain_spec(entity: &str, outputs: Vec<ItemFlow>) -> MachineSpec {
        MachineSpec {
            entity: entity.into(),
            recipe: "r".into(),
            count: 1.0,
            inputs: vec![],
            outputs,
            self_loop: vec![],
            voider: false,
        }
    }

    fn sr(machines: Vec<MachineSpec>) -> SolverResult {
        SolverResult {
            machines,
            external_inputs: vec![],
            external_outputs: vec![],
            surplus_outputs: vec![],
            dependency_order: vec![],
        }
    }

    #[test]
    fn plain_rows_are_not_exempt() {
        let ctx = StackingCtx::derive(
            &sr(vec![plain_spec("assembling-machine-3", vec![flow("electronic-circuit", false)])]),
            4,
        );
        assert_eq!(ctx.for_item("electronic-circuit"), 4);
        assert!(!ctx.is_exempt("electronic-circuit"));
    }

    #[test]
    fn s1_exempts_nothing_and_clamps() {
        let ctx = StackingCtx::derive(
            &sr(vec![plain_spec("recycler", vec![flow("iron-plate", false)])]),
            1,
        );
        assert_eq!(ctx.for_item("iron-plate"), 1);
        assert!(!ctx.is_exempt("iron-plate")); // S=1: empty exempt set
        assert_eq!(StackingCtx::derive(&sr(vec![]), 9).stacking(), 4);
        assert_eq!(StackingCtx::derive(&sr(vec![]), 0).stacking(), 1);
    }

    #[test]
    fn recycler_and_voider_streams_exempt() {
        // Voider: outputs always empty; the voided INPUT item is exempt.
        let mut voider = plain_spec("recycler", vec![]);
        voider.voider = true;
        voider.inputs = vec![flow("uranium-238", false)];
        // Scrap recycling: sushi OUTPUT items are exempt; scrap input is
        // trunk-fed (stack-preserving from stacked externals) — NOT exempt.
        let mut scrap = plain_spec("recycler", vec![flow("iron-gear-wheel", false)]);
        scrap.inputs = vec![flow("scrap", false)];
        let ctx = StackingCtx::derive(&sr(vec![voider, scrap]), 2);
        assert_eq!(ctx.for_item("uranium-238"), 1);
        assert_eq!(ctx.for_item("iron-gear-wheel"), 1);
        assert_eq!(ctx.for_item("scrap"), 2);
    }

    #[test]
    fn self_loop_outputs_and_loop_items_exempt() {
        let mut spec = plain_spec("centrifuge", vec![flow("uranium-235", false)]);
        spec.self_loop = vec![
            SelfLoopFlow {
                item: "uranium-235".into(),
                is_fluid: false,
                consumed_rate: 1.0,
                produced_rate: 1.1,
                net_rate: 0.1,
            },
            SelfLoopFlow {
                item: "uranium-238".into(),
                is_fluid: false,
                consumed_rate: 0.2,
                produced_rate: 0.1,
                net_rate: -0.1,
            },
        ];
        let ctx = StackingCtx::derive(&sr(vec![spec]), 3);
        assert_eq!(ctx.for_item("uranium-235"), 1);
        assert_eq!(ctx.for_item("uranium-238"), 1);
    }

    #[test]
    fn secondary_solid_output_exempt_primary_not() {
        // D2b: solid output index ≥1 is the fixed long-handed extraction.
        let spec = plain_spec(
            "assembling-machine-2",
            vec![flow("primary-item", false), flow("secondary-item", false)],
        );
        let ctx = StackingCtx::derive(&sr(vec![spec]), 4);
        assert_eq!(ctx.for_item("primary-item"), 4);
        assert_eq!(ctx.for_item("secondary-item"), 1);
        // Fluid outputs don't consume a solid index.
        let spec2 = plain_spec(
            "chemical-plant",
            vec![flow("steam", true), flow("solid-a", false), flow("solid-b", false)],
        );
        let ctx2 = StackingCtx::derive(&sr(vec![spec2]), 4);
        assert_eq!(ctx2.for_item("solid-a"), 4);
        assert_eq!(ctx2.for_item("solid-b"), 1);
    }
}
