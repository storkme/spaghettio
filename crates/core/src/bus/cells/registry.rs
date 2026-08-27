//! RFC-051: the sim-verified registry (Tier-1 verification cache).
//!
//! Key = chain config + a GEOMETRY HASH of the composed layout's
//! entities (#375 review, finding 1) + the DECLARED WORLD the
//! measurement ran in (#391): cells regenerate from the live engine,
//! so a config-only key would let row-template/placer/inserter changes
//! silently decay "sim-verified" into "unverified with a stale
//! verdict" — and #390 proved the same decay exists on the WORLD axis
//! (the stacking-parity change retired every pre-#390 measurement
//! without tripping a single geometry hash). The hash is
//! self-maintaining for geometry; the declared-world fields make the
//! instrument part of the claim: an entry only reads as fully verified
//! for a layout declaring the same `inserter_capacity` and `stacking`
//! it was measured under. A hash-matching entry from a DIFFERENT
//! declared world is reported as a scoped claim, not silently reused
//! (#383: EC rows measure at plan only above declared capacity 0
//! until RFC-049 Phase 3 sizing lands).
//!
//! Data: `crates/core/data/cell-sim-registry.json`, embedded via
//! `include_str!` like the recipe DB. Grown deliberately — an entry is
//! added only with a real sim PASS behind it (scenario + date + the
//! harness's realized-world line recorded), OR — policy decision
//! 2026-07-24 (#383, user-approved) — a WARN verdict whose deficit is
//! BOTH priced by a generation-time validator warning on the exact
//! geometry AND sim-measured at the warned magnitude. Such entries
//! carry `known_residual` with the attribution and are rendered
//! "SIM-VERIFIED AS WARNED", never "at plan": the registry's job is
//! catching drift, and an unregistered cell catches nothing.

use std::sync::OnceLock;

use crate::models::LayoutResult;
use serde::Deserialize;

/// One sim-verified composed geometry, in one declared world.
#[derive(Debug, Clone, Deserialize)]
pub struct RegistryEntry {
    pub target: String,
    pub rate: f64,
    /// Hex FNV-1a-64 of the composed layout's entity list.
    pub geometry_hash: String,
    /// The `inserter_capacity` the measured layout DECLARED — the sim
    /// forces research bonuses to this level (#378), so the verdict is
    /// scoped to it. REQUIRED, no serde default: a default would equal
    /// production's declared values exactly, so an entry omitting the
    /// field would fail OPEN (silent full-match) instead of failing
    /// loudly at parse (#397 review note).
    pub declared_inserter_capacity: u8,
    /// The `stacking` the measured layout declared (#390: the sim
    /// world matches declared stacking). REQUIRED — same fail-closed
    /// reasoning as above.
    pub declared_stacking: u8,
    /// The harness's realized-world line from the measurement report
    /// (provenance prose, e.g. "nb=0 bulk=1, S=1" — not matched on;
    /// the declared fields above are the checked key).
    #[serde(default)]
    pub harness_world: String,
    pub verdict: String,
    pub produced_per_s: f64,
    /// Present only on entries admitted under the "honestly warned +
    /// sim-confirmed" bar (#383 policy, 2026-07-24): the attribution of
    /// a known, generation-time-warned deficit that the sim measured at
    /// the warned magnitude. `None` = measured at plan.
    #[serde(default)]
    pub known_residual: Option<String>,
    pub scenario: String,
    pub date: String,
}

fn registry() -> &'static [RegistryEntry] {
    static REG: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
    REG.get_or_init(|| {
        serde_json::from_str(include_str!("../../../data/cell-sim-registry.json"))
            .expect("cell-sim-registry.json must parse")
    })
}

/// All registry entries — the gate iterates these so every seeded
/// claim is re-derived, not just a hardcoded list.
pub fn entries() -> &'static [RegistryEntry] {
    registry()
}

/// Stable FNV-1a-64 over the sorted entity list. Deliberately NOT the
/// std hasher (its output is not guaranteed stable across toolchains,
/// and the registry is checked-in data). Covers every field that
/// changes measured behavior: name, position, direction, io role,
/// carries, RECIPE, and module contents (#384 review finding 2 — a
/// recipe reassignment or module change at the same tile must miss the
/// registry, not keep a stale SIM-VERIFIED claim).
pub fn geometry_hash(l: &LayoutResult) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut tuples: Vec<String> = l
        .entities
        .iter()
        .map(|e| {
            let modules: String = {
                let mut ms: Vec<String> = e
                    .items
                    .iter()
                    .map(|m| format!("{}x{}", m.item, m.count))
                    .collect();
                ms.sort_unstable();
                ms.join(",")
            };
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}",
                e.name,
                e.x,
                e.y,
                e.direction as u8,
                e.io_type.as_deref().unwrap_or(""),
                e.carries.as_deref().unwrap_or(""),
                e.recipe.as_deref().unwrap_or(""),
                modules
            )
        })
        .collect();
    tuples.sort_unstable();
    let mut h = OFFSET;
    for t in &tuples {
        for b in t.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(PRIME);
        }
        h ^= 0x1f;
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Look up a composed layout's verification status by geometry. May
/// return an entry measured under a DIFFERENT declared world than the
/// caller's layout — `verification_note` surfaces that distinction;
/// callers doing their own matching must compare the declared fields.
pub fn lookup(target: &str, rate: f64, hash: u64) -> Option<&'static RegistryEntry> {
    let hex = format!("{hash:016x}");
    registry()
        .iter()
        .find(|e| e.target == target && (e.rate - rate).abs() < 1e-9 && e.geometry_hash == hex)
}

/// The annotation `CellComposedCandidate` attaches to its layouts.
/// Three tiers: full match (geometry + declared world), scoped match
/// (geometry verified, but in a different declared world than this
/// layout's), and no match. Warned entries (`known_residual`) render
/// "AS WARNED", never "at plan" — the claim is exactly as strong as
/// the measurement.
pub fn verification_note(target: &str, rate: f64, l: &LayoutResult) -> String {
    verification_status(target, rate, l).0
}

/// [`verification_note`] plus the typed verdict it rests on (RFC-074
/// Unit 1, `CompositionReceipt::verified`): `true` iff a registry row
/// matches THIS geometry in THIS declared world with a non-FAIL verdict
/// — the two "SIM-VERIFIED" arms below. Every other arm (no row, FAIL,
/// other-world match) is `false`; the note says which. Derived from the
/// same match, never from the note's wording.
pub fn verification_status(target: &str, rate: f64, l: &LayoutResult) -> (String, bool) {
    let hash = geometry_hash(l);
    let hex = format!("{hash:016x}");
    let matches: Vec<&RegistryEntry> = registry()
        .iter()
        .filter(|e| e.target == target && (e.rate - rate).abs() < 1e-9 && e.geometry_hash == hex)
        .collect();
    // Prefer the entry measured under THIS layout's declared world. The
    // same geometry can be registered under several declared worlds
    // (#383: capacity never reaches composed cells until #415, so d1/d7
    // share a hash) — first-in-file order must not decide which world
    // the note reports.
    let full = matches.iter().find(|e| {
        e.declared_inserter_capacity == l.inserter_capacity && e.declared_stacking == l.stacking
    });
    // #730 round 4 (2/3): the world-MISMATCH sibling must be chosen
    // deterministically, never by file order — one hash can carry
    // FAIL(d1) + PASS(d2) + WARN(d7) rows (8f2473ec does today), and a
    // JSON re-sort must not flip which tier the note lands in. Any
    // sibling FAIL dominates: an other-world PASS transfers nothing
    // (#383), so mixed evidence stays unverified until THIS world is
    // measured. Among several FAILs, or several non-FAILs (best
    // receipt first: at-plan before warned), the lowest declared world
    // is reported — an arbitrary but stable pick.
    let mismatch = matches
        .iter()
        .filter(|e| e.verdict == "FAIL")
        .min_by_key(|e| (e.declared_inserter_capacity, e.declared_stacking))
        .or_else(|| {
            matches.iter().min_by_key(|e| {
                (e.known_residual.is_some(), e.declared_inserter_capacity, e.declared_stacking)
            })
        });
    let verified = matches!(full, Some(e) if e.verdict != "FAIL");
    let note = match (full, mismatch) {
        (Some(e), _) if e.verdict == "FAIL" => format!(
            // A FAILED sim is the OPPOSITE of verification: carry the
            // policy's not-verified substring so `verified_geometry_first`
            // ranks this geometry with the unverified, never above them
            // (codex review of RFC-072 P2 unit 1 — the re-bless introduced
            // the first FAIL rows and the old arms read them as verified).
            "cell-composed: geometry NOT sim-verified — the sim FAILED it at its declared world ({} — {} produced {:.2}/s at declared capacity {}, {})",
            e.scenario, e.verdict, e.produced_per_s, e.declared_inserter_capacity, e.date
        ),
        (Some(e), _) => match &e.known_residual {
            None => format!(
                "cell-composed: geometry SIM-VERIFIED at plan ({} — {} produced {:.2}/s at declared capacity {}, {})",
                e.scenario, e.verdict, e.produced_per_s, e.declared_inserter_capacity, e.date
            ),
            Some(res) => format!(
                "cell-composed: geometry SIM-VERIFIED AS WARNED, not at plan — {res} ({} — {} produced {:.2}/s at declared capacity {}, {})",
                e.scenario, e.verdict, e.produced_per_s, e.declared_inserter_capacity, e.date
            ),
        },
        // A FAIL row in the world-MISMATCH arm too (#730 round 2, 3/3):
        // the mismatch sibling can be a FAIL row sharing the hash (and
        // after round 4, any sibling FAIL lands here by construction), and
        // the old wording never emitted the not-verified substring —
        // ranking a proven-failing geometry as verified. Guard on the
        // VERDICT, not on known_residual (round-2 minor: a verdict-less
        // FAIL row would otherwise read "at plan").
        // #730 round 3 wording: THIS layout's world was never measured —
        // the hash-sharing row failed in a DIFFERENT world. Unverified is
        // the safe standing; the note must not assert a measurement that
        // did not happen.
        (None, Some(e)) if e.verdict == "FAIL" => format!(
            "cell-composed: geometry NOT sim-verified — a hash-sharing build sim-FAILED in a DIFFERENT declared world (capacity {} / stacking {}: {} produced {:.2}/s, {}); this layout declares capacity {} / stacking {}, which was never measured",
            e.declared_inserter_capacity,
            e.declared_stacking,
            e.verdict,
            e.produced_per_s,
            e.date,
            l.inserter_capacity,
            l.stacking
        ),
        (None, Some(e)) => format!(
            "cell-composed: geometry sim-verified {} ONLY under declared capacity {} / stacking {} ({} produced {:.2}/s, {}); this layout declares capacity {} / stacking {} — measurements do NOT transfer across worlds (#383)",
            if e.known_residual.is_some() { "as warned" } else { "at plan" },
            e.declared_inserter_capacity,
            e.declared_stacking,
            e.verdict,
            e.produced_per_s,
            e.date,
            l.inserter_capacity,
            l.stacking
        ),
        (None, None) => format!(
            "cell-composed: geometry NOT sim-verified (hash {hex}) — run spaghettio-sim and add the entry to cell-sim-registry.json"
        ),
    };
    (note, verified)
}
