//! Runtime template generator. Produces `BalancerTemplate`-equivalent
//! layouts for shapes the library lacks (or where a smaller-footprint
//! alternative would help).
//!
//! Phase 2.0: only divisible cases handled — pass-through for `m == n`,
//! 1→2 atom, and horizontal replication of an atom for `(m, k·m)` and
//! `(k·m, m)` shapes. Non-divisible (coprime) shapes return `None` and
//! fall through to the library / decomposition fallback in
//! [`super::balancer`].
//!
//! Every generated template is verified by [`classify_ref`] before being
//! returned. A generator bug that produces an MX1 layout fails loudly
//! (panics in tests; logs in release).

use crate::bus::balancer_classify::{classify_ref, BalancerClass, BalancerTemplateRef};
use crate::bus::balancer_library::BalancerTemplateEntity;
use crate::models::PlacedEntity;

#[derive(Debug, Clone)]
pub struct OwnedTemplate {
    pub n_inputs: u32,
    pub n_outputs: u32,
    pub width: u32,
    pub height: u32,
    pub entities: Vec<BalancerTemplateEntity>,
    pub input_tiles: Vec<(i32, i32)>,
    pub output_tiles: Vec<(i32, i32)>,
}

impl OwnedTemplate {
    pub fn as_ref(&self) -> BalancerTemplateRef<'_> {
        BalancerTemplateRef {
            n_inputs: self.n_inputs,
            n_outputs: self.n_outputs,
            width: self.width,
            height: self.height,
            entities: &self.entities,
            input_tiles: &self.input_tiles,
            output_tiles: &self.output_tiles,
        }
    }

    /// Stamp the template at the given origin, substituting belt-tier
    /// names. Mirrors `BalancerTemplate::stamp`.
    pub fn stamp(
        &self,
        origin_x: i32,
        origin_y: i32,
        belt_name: &str,
        splitter_name: &str,
        ug_name: &str,
        item: Option<&str>,
    ) -> Vec<PlacedEntity> {
        self.entities
            .iter()
            .map(|e| e.stamp(origin_x, origin_y, belt_name, splitter_name, ug_name, item))
            .collect()
    }

    /// Render the template as a Factorio-importable blueprint string.
    /// Useful for quick visualisation: paste into Factorio (or a blueprint
    /// inspector) to verify a generated layout looks sane.
    pub fn to_blueprint(&self, label: &str) -> String {
        let entities = self.stamp(0, 0, "transport-belt", "splitter", "underground-belt", None);
        let layout = crate::models::LayoutResult {
            entities,
            width: self.width as i32,
            height: self.height as i32,
            warnings: Vec::new(),
            ..Default::default()
        };
        crate::blueprint::export(&layout, label)
    }
}

/// Try to generate a template for `(m, n)`. Returns `None` if the
/// generator doesn't yet support this shape — caller falls back to the
/// library or the existing decomposition path.
pub fn generate(m: u32, n: u32) -> Option<OwnedTemplate> {
    if m == 0 || n == 0 || (m == 1 && n == 1) {
        return None;
    }
    let candidate = if m == n {
        passthrough(m)
    } else if m == 1 && n == 2 {
        one_to_two()
    } else if m == 2 && n == 1 {
        two_to_one()
    } else if n.is_multiple_of(m) && m >= 1 {
        let k = n / m; // each input fans out to k outputs
        let atom = match k {
            2 => one_to_two(),
            _ => return None,
        };
        replicate_horizontally(&atom, m)
    } else if m.is_multiple_of(n) && n >= 1 {
        let k = m / n; // every k inputs merge into 1 output
        let atom = match k {
            2 => two_to_one(),
            _ => return None,
        };
        replicate_horizontally(&atom, n)
    } else {
        return None;
    };

    // Self-verify. A generator bug that yields anything weaker than the
    // intended class is unacceptable.
    let report = classify_ref(candidate.as_ref()).ok()?;
    match report.class {
        BalancerClass::Balanced
        | BalancerClass::ThroughputUnlimited
        | BalancerClass::ThroughputBalancedRate => Some(candidate),
        BalancerClass::ThroughputLimited => None,
    }
}

// ---------------------------------------------------------------------------
// Atoms
// ---------------------------------------------------------------------------

/// `m` parallel south-facing belts, length 1. Each input port maps directly
/// to the output port at the same column. Trivially MX2b (no mixing, but
/// max-flow holds in both directions).
fn passthrough(m: u32) -> OwnedTemplate {
    let mut entities = Vec::with_capacity(m as usize);
    for i in 0..m as i32 {
        entities.push(BalancerTemplateEntity {
            name: "transport-belt",
            x: i,
            y: 0,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        });
    }
    let input_tiles: Vec<(i32, i32)> = (0..m as i32).map(|i| (i, 0)).collect();
    let output_tiles = input_tiles.clone();
    OwnedTemplate {
        n_inputs: m,
        n_outputs: m,
        width: m,
        height: 1,
        entities,
        input_tiles,
        output_tiles,
    }
}

/// 1 input → 1 splitter → 2 outputs. Identical to the library's `(1, 2)`.
fn one_to_two() -> OwnedTemplate {
    // Layout (column 0/1, rows 0/1/2):
    //   y=0: input belt at (0, 0) facing south
    //   y=1: splitter anchor at (0, 1), second tile (1, 1)
    //   y=2: output belts at (0, 2) and (1, 2)
    let entities = vec![
        BalancerTemplateEntity {
            name: "transport-belt",
            x: 0,
            y: 0,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
        BalancerTemplateEntity {
            name: "splitter",
            x: 0,
            y: 1,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
        BalancerTemplateEntity {
            name: "transport-belt",
            x: 0,
            y: 2,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
        BalancerTemplateEntity {
            name: "transport-belt",
            x: 1,
            y: 2,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
    ];
    OwnedTemplate {
        n_inputs: 1,
        n_outputs: 2,
        width: 2,
        height: 3,
        entities,
        input_tiles: vec![(0, 0)],
        output_tiles: vec![(0, 2), (1, 2)],
    }
}

/// 2 inputs → 1 splitter (one output dangles) → 1 output. Throughput
/// capped at 1 (output belt cap), but composition is balanced (each
/// input contributes 1/2 to the output) — classifier reports MX3 under
/// the saturated linear model.
///
/// **Verifier note**: this construction relies on Factorio's splitter
/// back-pressure behaviour — items can't go to the empty `(1, 2)` tile,
/// so the splitter saturates internally and balances 50/50 between input
/// belts in steady state. The all-fluid Couëtoux verifier proposed in
/// PR #270 will report this as an unbalanced steady state because it
/// doesn't model the back-pressure-induced saturation. Both signals are
/// honest about a different question: this classifier checks the
/// saturated-model invariants we rely on at the layout layer; the all-
/// fluid verifier checks an unsaturated-flow invariant. Cross-validation
/// disagreements on `two_to_one`-using templates are expected.
fn two_to_one() -> OwnedTemplate {
    // Layout:
    //   y=0: 2 input belts at (0, 0) and (1, 0) facing south
    //   y=1: splitter anchor at (0, 1), second (1, 1)
    //   y=2: 1 output belt at (0, 2). The (1, 2) splitter output is
    //        intentionally empty — items going there are lost (or,
    //        equivalently, the splitter routes everything to (0, 2)
    //        per S5 once that tile is full).
    let entities = vec![
        BalancerTemplateEntity {
            name: "transport-belt",
            x: 0,
            y: 0,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
        BalancerTemplateEntity {
            name: "transport-belt",
            x: 1,
            y: 0,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
        BalancerTemplateEntity {
            name: "splitter",
            x: 0,
            y: 1,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
        BalancerTemplateEntity {
            name: "transport-belt",
            x: 0,
            y: 2,
            direction: 4,
            io_type: None,
            input_priority: None,
            output_priority: None,
        },
    ];
    OwnedTemplate {
        n_inputs: 2,
        n_outputs: 1,
        width: 2,
        height: 3,
        entities,
        input_tiles: vec![(0, 0), (1, 0)],
        output_tiles: vec![(0, 2)],
    }
}

// ---------------------------------------------------------------------------
// Composers
// ---------------------------------------------------------------------------

/// Stamp `count` copies of `atom` side by side along the x-axis.
/// The replicated template's ports are the union of the atoms' ports,
/// renumbered. Useful for `(m, k·m)` and `(k·m, m)` shapes.
fn replicate_horizontally(atom: &OwnedTemplate, count: u32) -> OwnedTemplate {
    let dx = atom.width as i32;
    let mut entities = Vec::with_capacity(atom.entities.len() * count as usize);
    let mut input_tiles = Vec::with_capacity(atom.input_tiles.len() * count as usize);
    let mut output_tiles = Vec::with_capacity(atom.output_tiles.len() * count as usize);
    for c in 0..count as i32 {
        let off = c * dx;
        for e in &atom.entities {
            entities.push(BalancerTemplateEntity {
                name: e.name,
                x: e.x + off,
                y: e.y,
                direction: e.direction,
                io_type: e.io_type,
                input_priority: e.input_priority,
                output_priority: e.output_priority,
            });
        }
        for &(x, y) in &atom.input_tiles {
            input_tiles.push((x + off, y));
        }
        for &(x, y) in &atom.output_tiles {
            output_tiles.push((x + off, y));
        }
    }
    OwnedTemplate {
        n_inputs: atom.n_inputs * count,
        n_outputs: atom.n_outputs * count,
        width: atom.width * count,
        height: atom.height,
        entities,
        input_tiles,
        output_tiles,
    }
}

// ---------------------------------------------------------------------------
// Merge-trees (n → 1) — RFC docs/rfc-merge-tap-trunks.md D2/D3
// ---------------------------------------------------------------------------

/// Build an `n → 1` splitter merge-tree: `n` producer input belts on the top
/// row, merged onto a single output belt at the bottom, using only splitters
/// and straight (both-lane-preserving) belts — NO sideloads (D3). Merging is
/// associative, so any `n` composes from `(2 → 1)` splitter merges with no
/// coprime arithmetic — the property that lets the merge-tap fallback retire
/// unstampable balancer shapes.
///
/// Geometry is a diagonal "staircase" cascade that mirrors the trunk-head
/// placement frame `stamp_family_balancer` uses (inputs on row 0 facing south,
/// the single output on the bottom row, so the trunk picks up below just like
/// a balancer's outputs):
///   * column `i` carries `input_i` straight south until it meets its merge;
///   * `splitter_k` (k = 1..n) sits at row `2k-1` spanning columns `(k-1, k)`,
///     merging the running trunk (its left input, straight from above) with
///     `input_k` (its right input, straight from above); the right output
///     continues the trunk one column right and two rows down, the left output
///     is left empty so backpressure routes all flow to the used output (S5 —
///     the same construction as [`two_to_one`]).
///
/// Result: width `n`, height `2n-1`, exactly `n-1` splitters, `n²` entities,
/// and a single output tile at `(n-1, 2n-2)`. `n == 1` is the degenerate
/// pass-through (one belt) — a trunk with a single producer needs no merge.
///
/// Returned as an [`OwnedTemplate`] so the caller stamps it through the same
/// `.stamp()` path a balancer family uses.
pub fn merge_tree(n: u32) -> OwnedTemplate {
    assert!(n >= 1, "merge_tree needs at least one input");

    let belt = |x: i32, y: i32| BalancerTemplateEntity {
        name: "transport-belt",
        x,
        y,
        direction: 4, // south
        io_type: None,
        input_priority: None,
        output_priority: None,
    };

    if n == 1 {
        return OwnedTemplate {
            n_inputs: 1,
            n_outputs: 1,
            width: 1,
            height: 1,
            entities: vec![belt(0, 0)],
            input_tiles: vec![(0, 0)],
            output_tiles: vec![(0, 0)],
        };
    }

    let n_i = n as i32;
    let mut entities: Vec<BalancerTemplateEntity> = Vec::with_capacity((n * n) as usize);

    // input_0: single belt feeding splitter_1's left input from the north.
    entities.push(belt(0, 0));
    // input_i (i >= 1): straight south feeder from row 0 down to row 2i-2,
    // where it enters splitter_i's right input.
    for i in 1..n_i {
        for y in 0..=(2 * i - 2) {
            entities.push(belt(i, y));
        }
    }
    // splitter_k + its trunk-continuation (right) output belt.
    for k in 1..n_i {
        entities.push(BalancerTemplateEntity {
            name: "splitter",
            x: k - 1,
            y: 2 * k - 1,
            direction: 4, // south; spans (k-1, ·) and (k, ·)
            io_type: None,
            input_priority: None,
            output_priority: None,
        });
        // Used (right) output at (k, 2k): trunk continuation / final output.
        // The left output (k-1, 2k) is deliberately left empty (S5).
        entities.push(belt(k, 2 * k));
    }

    OwnedTemplate {
        n_inputs: n,
        n_outputs: 1,
        width: n,
        height: 2 * n - 1,
        entities,
        input_tiles: (0..n_i).map(|i| (i, 0)).collect(),
        output_tiles: vec![(n_i - 1, 2 * n_i - 2)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::balancer_library::balancer_templates;

    /// Every generated template must classify cleanly (no `Err`) and not
    /// be MX1 (throughput-limited). The verifier inside `generate` already
    /// enforces this; this test catches any internal generation that
    /// somehow bypasses it.
    #[test]
    fn generated_templates_are_not_mx1() {
        for m in 1..=10 {
            for n in 1..=10 {
                let Some(t) = generate(m, n) else { continue };
                let report = classify_ref(t.as_ref()).expect("classify failed");
                assert_ne!(
                    report.class,
                    BalancerClass::ThroughputLimited,
                    "({m}, {n}) generated as MX1"
                );
            }
        }
    }

    #[test]
    fn passthrough_classifies_as_mx2b() {
        for m in 2..=10 {
            let t = generate(m, m).expect("passthrough");
            let r = classify_ref(t.as_ref()).unwrap();
            assert_eq!(
                r.class,
                BalancerClass::ThroughputUnlimited,
                "({m}, {m})"
            );
            assert_eq!(t.entities.len() as u32, m, "entity count");
        }
    }

    #[test]
    fn one_to_two_matches_library() {
        let gen = generate(1, 2).expect("1→2");
        assert_eq!(gen.entities.len(), 4);
        let r = classify_ref(gen.as_ref()).unwrap();
        assert_eq!(r.class, BalancerClass::Balanced);
    }

    #[test]
    fn parallel_one_to_two_for_m_to_2m() {
        // (2, 4), (3, 6), (4, 8), (5, 10) all decompose into m × (1, 2).
        for m in 2..=5 {
            let n = 2 * m;
            let t = generate(m, n).expect("({m}, {n})");
            assert_eq!(t.n_inputs, m);
            assert_eq!(t.n_outputs, n);
            let r = classify_ref(t.as_ref()).expect("classify");
            // Disjoint 1→2 atoms: each output is fed by exactly one input
            // tree (no cross-mixing). Under uniform input that's still
            // balanced rate, but max-flow over an output subset that lies
            // entirely within one tree drops to 1, short of |T|. So the
            // class is MX2a (saturation + balanced rate), not MX2b.
            assert!(
                matches!(r.class, BalancerClass::ThroughputBalancedRate),
                "({m}, {n}) class = {:?}",
                r.class
            );
        }
    }

    /// Mirror of `parallel_one_to_two_for_m_to_2m` for the `(2m, m)`
    /// family. The user's review comment requested this as a pin so a
    /// future bug that elevates these to MX2/MX3 gets caught — but
    /// tracing through the math shows the symmetry doesn't quite hold:
    ///
    /// `m × (1, 2)`: each input feeds 2 outputs at rate 1/2. Input-
    /// subset max-flow of `|S| = 1` reaches `min(1, 2m) = 1`. Output-
    /// subset max-flow of `|T| = 2` confined to one column drops to 1,
    /// short of `min(m, 2) = 2`. → MX2a (passes input, fails output).
    ///
    /// `m × (2, 1)`: each *pair* of inputs feeds 1 output, capped at 1
    /// by the dangling-output back-pressure trick. Input-subset
    /// `|S| = 2` confined to one column delivers 1 to all `m` outputs,
    /// short of `min(2, m) = 2` for `m ≥ 2`. → MX1 (fails input
    /// already; output check is moot).
    ///
    /// The asymmetry: in `(m, 2m)` the per-input cap is the limit; in
    /// `(2m, m)` the per-output cap of the dangling-output trick is the
    /// limit, and that limit *bottlenecks* the input subset. So
    /// `generate(2m, m)` returns `None` (caught by the self-verify
    /// gate). The test pins this — both that the underlying composition
    /// classifies as MX1 *and* that the public generator correctly
    /// rejects it.
    #[test]
    fn parallel_two_to_one_for_2m_to_m() {
        for m in 2..=5 {
            let n = 2 * m;

            // Pin 1: the underlying parallel composition is MX1 because
            // the per-column merger cap = 1 short-circuits input-subset
            // Menger.
            let composed = replicate_horizontally(&two_to_one(), m);
            let r = classify_ref(composed.as_ref()).expect("classify");
            assert!(
                matches!(r.class, BalancerClass::ThroughputLimited),
                "({n}, {m}) parallel two_to_one composition class = {:?}",
                r.class
            );

            // Pin 2: the public generator's self-verify gate rejects
            // MX1, so callers see `None` for these shapes today. If a
            // future change accepts MX1 (or upgrades the class), this
            // assertion catches it.
            assert!(
                generate(n, m).is_none(),
                "generate({n}, {m}) unexpectedly returned a template; \
                 the (2m, m) parallel composition is MX1 and should be \
                 filtered by the self-verify gate"
            );
        }
    }

    /// Compare entity counts and footprint of generated templates to the
    /// existing library. Run with `--nocapture` to see numbers.
    #[test]
    fn footprint_vs_library() {
        let lib = balancer_templates();
        eprintln!();
        eprintln!(
            "| (m, n) | gen entities | gen tiles | lib entities | lib tiles | savings |"
        );
        eprintln!(
            "|--------|--------------|-----------|--------------|-----------|---------|"
        );
        let mut shapes: Vec<(u32, u32)> = (1..=10)
            .flat_map(|m| (1..=10).map(move |n| (m, n)))
            .collect();
        shapes.sort();
        for (m, n) in shapes {
            let Some(gen) = generate(m, n) else { continue };
            let gen_entities = gen.entities.len();
            let gen_tiles = (gen.width * gen.height) as usize;
            let (lib_entities, lib_tiles, savings) = match lib.get(&(m, n)) {
                Some(t) => {
                    let lib_e = t.entities.len();
                    let lib_t = (t.width * t.height) as usize;
                    let pct = if lib_e > 0 {
                        100.0 * (lib_e as f64 - gen_entities as f64) / lib_e as f64
                    } else {
                        0.0
                    };
                    (
                        format!("{lib_e}"),
                        format!("{lib_t}"),
                        format!("{pct:+.0}%"),
                    )
                }
                None => ("—".to_string(), "—".to_string(), "(new)".to_string()),
            };
            eprintln!(
                "| ({m}, {n}) | {gen_entities} | {gen_tiles} | {lib_entities} | {lib_tiles} | {savings} |"
            );
        }
    }

    /// Print Factorio blueprint strings for a handful of representative
    /// generated shapes. Run with `--nocapture` to copy-paste into the
    /// game (or a blueprint inspector) for visual sanity-checking.
    /// `(9, 9)` is the smallest shape this generator covers that the
    /// library lacks; the others demonstrate atom replication.
    #[test]
    fn dump_blueprints_for_visualisation() {
        let shapes = [(2, 2), (4, 4), (9, 9), (10, 10), (1, 2), (2, 4), (4, 8)];
        eprintln!();
        for (m, n) in shapes {
            let Some(gen) = generate(m, n) else {
                eprintln!("({m}, {n}): generator returned None");
                continue;
            };
            let bp = gen.to_blueprint(&format!("gen_{m}x{n}"));
            eprintln!("({m}, {n}): {} entities, {}×{}", gen.entities.len(), gen.width, gen.height);
            eprintln!("  blueprint: {bp}");
        }
    }

    // -----------------------------------------------------------------------
    // Merge-trees (RFC merge-tap-trunks D2/D3)
    // -----------------------------------------------------------------------

    /// Expand a merge-tree's template entities into occupied tiles (splitters
    /// occupy two tiles side by side along x, since they face south).
    fn merge_tree_tiles(t: &OwnedTemplate) -> Vec<(i32, i32)> {
        let mut tiles = Vec::new();
        for e in &t.entities {
            tiles.push((e.x, e.y));
            if e.name == "splitter" {
                tiles.push((e.x + 1, e.y));
            }
        }
        tiles
    }

    #[test]
    fn merge_tree_shapes_including_primes() {
        // Primes are the deliberate stress: associativity means any n must
        // compose, so 11 and 13 (no balancer atom) MUST build cleanly.
        for n in [2u32, 3, 5, 7, 11, 13] {
            let t = merge_tree(n);
            assert_eq!(t.n_inputs, n, "n_inputs");
            assert_eq!(t.n_outputs, 1, "n_outputs (a merge is n→1)");
            assert_eq!(t.width, n, "width == n");
            assert_eq!(t.height, 2 * n - 1, "height == 2n-1");
            assert_eq!(t.entities.len() as u32, n * n, "entity count == n²");
            let splitters = t.entities.iter().filter(|e| e.name == "splitter").count();
            assert_eq!(splitters as u32, n - 1, "exactly n-1 splitters");
            // No sideloads (D3): every belt/splitter faces south.
            assert!(t.entities.iter().all(|e| e.direction == 4), "all south-facing");
            assert_eq!(t.input_tiles.len(), n as usize, "n input tiles");
            assert_eq!(
                t.input_tiles,
                (0..n as i32).map(|i| (i, 0)).collect::<Vec<_>>(),
                "inputs on the top row"
            );
            assert_eq!(
                t.output_tiles,
                vec![(n as i32 - 1, 2 * n as i32 - 2)],
                "single output at the bottom-right"
            );
        }
    }

    #[test]
    fn merge_tree_has_no_tile_overlap() {
        // Splitter-only, straight-fed: no two entities may claim a tile
        // (overlaps would surface as entity-overlap errors downstream).
        for n in [2u32, 3, 5, 7, 11, 13] {
            let t = merge_tree(n);
            let tiles = merge_tree_tiles(&t);
            let unique: std::collections::HashSet<_> = tiles.iter().copied().collect();
            assert_eq!(
                tiles.len(),
                unique.len(),
                "merge_tree({n}) has overlapping tiles"
            );
            // Everything stays inside the declared bbox.
            for (x, y) in tiles {
                assert!(
                    x >= 0 && (x as u32) < t.width && y >= 0 && (y as u32) < t.height,
                    "tile ({x},{y}) outside {}×{} bbox for n={n}",
                    t.width,
                    t.height
                );
            }
        }
    }

    #[test]
    fn merge_tree_is_deterministic() {
        // Determinism is a hard project contract — same n, byte-identical
        // entities every time.
        for n in [2u32, 7, 13] {
            assert_eq!(merge_tree(n).entities, merge_tree(n).entities);
        }
    }

    #[test]
    fn merge_tree_n1_is_passthrough() {
        let t = merge_tree(1);
        assert_eq!(t.n_inputs, 1);
        assert_eq!(t.n_outputs, 1);
        assert_eq!(t.entities.len(), 1);
        assert_eq!(t.entities[0].name, "transport-belt");
        assert_eq!(t.input_tiles, vec![(0, 0)]);
        assert_eq!(t.output_tiles, vec![(0, 0)]);
    }

    #[test]
    fn merge_tree_output_tile_is_a_belt() {
        // The declared output tile must actually carry a belt (the trunk
        // picks up there) — and the two-tile inputs must all be belts.
        for n in [2u32, 3, 5] {
            let t = merge_tree(n);
            let out = t.output_tiles[0];
            assert!(
                t.entities.iter().any(|e| (e.x, e.y) == out && e.name == "transport-belt"),
                "output tile {out:?} must be a belt for n={n}"
            );
            for &inp in &t.input_tiles {
                assert!(
                    t.entities.iter().any(|e| (e.x, e.y) == inp && e.name == "transport-belt"),
                    "input tile {inp:?} must be a belt for n={n}"
                );
            }
        }
    }
}
