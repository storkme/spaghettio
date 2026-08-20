//! Export a layout as a `spaghettio-sim` blueprint + manifest pair.
//!
//! **This is the tracked generator.** `crates/core/examples/` is otherwise
//! gitignored (local-only debug scripts), and `.gitignore` carries an explicit
//! negation for this one file — so on a fresh clone there is a supported way to
//! produce the two artifacts `spaghettio-sim run` consumes. Closing that gap is
//! the whole reason it exists; see [`docs/sim-harness.md`].
//!
//! ```text
//! cargo run --release --example sim_export -- <item> <rate> [flags]
//! cargo run --release --example sim_export -- --multi <item1>:<rate1> <item2>:<rate2> ... [flags]
//!
//!   --multi item:rate [item:rate ...]   RFC-062 Phase 3: N >= 1 simultaneous
//!                       targets instead of one, solved as a single combined
//!                       plan (spaghettio_core::solver::
//!                       solve_multi_with_palette_exclusions_quality_and_modules
//! the same solve the wasm `solve_multi` boundary uses, reached through
//! `netflow::solve_netflow_multi_with_options` rather than the
//! `solve_multi_with_palette_exclusions_quality_and_modules` wrapper — the
//! wrapper cannot pass the declared research-productivity axis. Every other
//! option is left at the value that wrapper set.
//!                       boundary uses). Replaces the <item> <rate>
//!                       positionals; every flag below still applies. N=1
//!                       is bit-identical to the plain positional form by
//!                       construction (RFC-062 kill criterion 5) — this
//!                       example always calls the multi entry point
//!                       internally, single-target or not.
//!   --tier <entity>        crafting machine (default assembling-machine-3)
//!   --di off|candidate|forced        direct insertion (default candidate)
//!   --claim up|down|search           DI claim order (default: engine default)
//!   --belt <entity>        max belt tier (default: engine picks by rate)
//!   --row-layout <kind>    native (default) | horizontal-stack
//!   --strategy <kind>      pooled (default) | partitioned-decomposed
//!   --quality <name>       normal|uncommon|rare|epic|legendary (default normal)
//!   --stacking <1..4>      belt stacking (default 1)
//!   --research-productivity <recipe=bonus,...>   declared research
//!                          productivity, e.g. processing-unit=0.10 (default none)
//!   --inserter-cap <n>     inserter capacity level (default: engine default)
//!   --inputs a,b,c         raw inputs (default: the six-ore set)
//!   --label <name>         output subdirectory + manifest label
//!   --out <dir>            parent output dir (default $SIM_PROBE_OUT or /tmp)
//! ```
//!
//! Writes `<out>/<label>/bp.txt` and `<out>/<label>/manifest-real.json`. Pass
//! the **`manifest-real.json`** to `run` — it is the `export_with_manifest`
//! output the harness parses. (The older, untracked `sim_probe_export` also
//! wrote a sibling `manifest.json` in a pre-Phase-0 ad hoc shape that the
//! harness rejects with a missing-field error; this example deliberately writes
//! only the real one, so there is nothing to pass by mistake.)
//!
//! Exit status is 0 even when the layout validates with errors — a deliberately
//! broken layout is a legitimate thing to sim, and refusing to export one would
//! make the harness unable to measure exactly the cases worth measuring. The
//! issue counts are printed so a caller can decide.

use rustc_hash::FxHashSet;
use spaghettio_core::bus::di_cell::{DiClaimOrder, DirectInsertion};
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions, LayoutStrategy, RowLayout};
use spaghettio_core::common::QualityTier;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::validate::{self, LayoutStyle, Severity};

/// Default crafting machine — the one `--tier` overrides. Named so the
/// label logic and the parser default cannot drift apart.
const DEFAULT_TIER: &str = "assembling-machine-3";

const DEFAULT_INPUTS: &[&str] = &[
    "iron-ore",
    "copper-ore",
    "stone",
    "coal",
    "water",
    "crude-oil",
];

fn usage(msg: &str) -> ! {
    eprintln!("error: {msg}\n");
    eprintln!("usage: cargo run --release --example sim_export -- <item> <rate> [flags]");
    eprintln!("       cargo run --release --example sim_export -- --multi <item>:<rate> ... [flags]");
    eprintln!("       see the module doc comment for the flag list");
    std::process::exit(2);
}

/// Parse one `item:rate` token for `--multi` mode. `rsplit_once` (not
/// `split_once`) so an item slug can never itself contain `:` without
/// ambiguity — moot for real Factorio item names (hyphens only) but keeps
/// the parse unambiguous either way.
fn parse_target_token(tok: &str) -> (String, f64) {
    let (item, rate_str) = tok
        .rsplit_once(':')
        .unwrap_or_else(|| usage(&format!("--multi target must be item:rate, got {tok:?}")));
    let rate: f64 = rate_str
        .parse()
        .unwrap_or_else(|_| usage(&format!("bad rate in target {tok:?}")));
    (item.to_string(), rate)
}


/// Every axis that changes the exported artifact, as one value — so the
/// label logic is a pure function and can be tested (#661 round 3: it was
/// the PR's entire purpose and had no coverage, which is the same class of
/// gap that produced the collision in the first place).
struct LabelAxes<'a> {
    strategy: LayoutStrategy,
    row_layout: RowLayout,
    duty: f64,
    tier: &'a str,
    belt: Option<&'a str>,
    quality: QualityTier,
    stacking: u8,
    di: DirectInsertion,
    claim: Option<&'a DiClaimOrder>,
    inserter_cap: Option<u8>,
    inputs: &'a [String],
    research_productivity: &'a std::collections::BTreeMap<String, f64>,
}

/// The auto label, which is also the OUTPUT DIRECTORY (`<out>/<label>/`).
///
/// Two runs whose artifacts differ must not share it. They did: the label
/// was `{item}-{rate}` and encoded no configuration at all, so
/// `--strategy pooled X 5` and `--strategy partitioned-decomposed X 5`
/// both wrote `<out>/X-5/` and the second silently overwrote the first
/// (#661). That is a wrong-A/B generator, and A/B is why this binary takes
/// these flags.
///
/// Suffixes are emitted only where the value differs from the ENGINE
/// default — not merely where a flag was passed (#661 round 3) — so
/// `--inserter-cap 2` and a default run share a directory, because they
/// produce the same artifact. Default paths stay byte-identical to
/// pre-#661.
fn auto_label(base: &str, ax: &LabelAxes<'_>) -> String {
    // Every "is this at default?" test compares against the ENGINE's own
    // default, never a literal (#661 round 5). Round 4 fixed only `claim`
    // this way, after hardcoding `Upstream` inverted the contract when the
    // real default turned out to be `Downstream`. Every other axis had the
    // same latent bug: this repo flips defaults on measurement (RFC-051,
    // RFC-053, RFC-059, RFC-060 all did), and a flip would silently
    // reintroduce the collision this whole function exists to prevent.
    //
    // `LayoutOptions::default()` IS the reference, so the comparison cannot
    // drift from the thing it is describing.
    let d = LayoutOptions::default();
    let mut tags: Vec<String> = Vec::new();

    if ax.strategy != d.strategy {
        tags.push("pd".to_string());
    }
    if ax.row_layout != d.row_layout {
        tags.push(format!("{:?}", ax.row_layout).to_lowercase());
    }
    // Exact float comparison, not epsilon (#661 round 2) and not a formatted
    // string (round 3): `1.0` never tags, `0.9999999999999999` does.
    if ax.duty != d.planning_duty {
        tags.push(format!("duty{}", format!("{}", ax.duty).replace('.', "_")));
    }
    if ax.tier != DEFAULT_TIER {
        tags.push(ax.tier.replace("assembling-machine-", "am").replace('-', ""));
    }
    // NOTE the asymmetry, stated because it breaks the "only non-default"
    // rule on purpose: `max_belt_tier: None` means "engine picks by rate", so
    // an explicit `--belt` matching what the engine would have picked yields a
    // byte-identical artifact in a separate directory. That over-forks, which
    // is the SAFE direction (a redundant dir, never a shared one); resolving
    // it properly needs the rate-dependent `belt_entity_for_rate`, which the
    // label does not have. Documented rather than silently tolerated.
    if ax.belt != d.max_belt_tier.as_deref() {
        if let Some(b) = ax.belt {
            tags.push(b.replace("-transport-belt", "").replace("transport-belt", "yellow"));
        }
    }
    if ax.quality != d.quality {
        tags.push(format!("{:?}", ax.quality).to_lowercase());
    }
    if ax.stacking != d.stacking {
        tags.push(format!("stack{}", ax.stacking));
    }
    if ax.di != d.direct_insertion {
        tags.push(format!("di{:?}", ax.di).to_lowercase());
    }
    if let Some(c) = ax.claim {
        if *c != d.di_claim_order {
            tags.push(format!("claim{c:?}").to_lowercase());
        }
    }
    if let Some(ic) = ax.inserter_cap {
        if ic != d.inserter_capacity {
            tags.push(format!("icap{ic}"));
        }
    }

    // List-valued axes, compared and digested as SETS because the solve
    // consumes `--inputs` as an `FxHashSet` (#661 round 4).
    let mut listy = String::new();
    let mut given: Vec<&str> = ax.inputs.iter().map(|s| s.as_str()).collect();
    given.sort_unstable();
    given.dedup();
    let mut default_inputs: Vec<&str> = DEFAULT_INPUTS.to_vec();
    default_inputs.sort_unstable();
    let inputs_differ = given != default_inputs;
    if inputs_differ {
        listy.push_str(&given.join(","));
    }
    for (k, v) in ax.research_productivity {
        listy.push_str(&format!(";{k}={v}"));
    }
    // Keyed on "does it DIFFER", not on "is the digest string non-empty"
    // (#661 round 5): `--inputs ""` parses to `[""]`, which differs from the
    // default set but joins to the empty string, so the old test skipped the
    // tag and dropped the run into the DEFAULT directory — the exact
    // collision, at the empty edge.
    if inputs_differ || !ax.research_productivity.is_empty() {
        // FNV-1a 64-bit (#661 round 5). The 32-bit version could alias two
        // distinct sets onto one directory, which made the "cannot collide"
        // claim false; 64 bits makes it true for any realistic number of
        // fixture configs. Deterministic across runs and platforms, which
        // `DefaultHasher` explicitly is not.
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in listy.as_bytes() {
            h ^= *byte as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        tags.push(format!("cfg{h:016x}"));
    }

    if tags.is_empty() {
        base.to_string()
    } else {
        format!("{base}-{}", tags.join("-"))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage("need at least <item> and <rate>, or --multi item:rate ...");
    }

    // RFC-062 Phase 3: `--multi item:rate [item:rate ...]` replaces the two
    // positionals with N >= 1 targets, consumed greedily until the next
    // `--flag` token. Every flag below still applies after the target list.
    // The single-target positional form below is unchanged and still the
    // common case — this example always calls the SAME multi-target solve
    // entry point internally either way (N=1 is bit-identical by
    // construction, RFC-062 kill criterion 5), so there is exactly one
    // solve code path to maintain, not two.
    let mut targets: Vec<(String, f64)> = Vec::new();
    let mut i;
    if args[0] == "--multi" {
        i = 1;
        while i < args.len() && !args[i].starts_with("--") {
            targets.push(parse_target_token(&args[i]));
            i += 1;
        }
        if targets.is_empty() {
            usage("--multi needs at least one item:rate target");
        }
    } else {
        if args.len() < 2 {
            usage("need at least <item> and <rate>");
        }
        let rate: f64 = args[1]
            .parse()
            .unwrap_or_else(|_| usage("<rate> must be a number"));
        targets.push((args[0].clone(), rate));
        i = 2;
    }

    // Flags after the positionals/target list. Unknown flags are an ERROR
    // rather than ignored: a silently-dropped `--belt` would export a
    // layout the caller did not ask for and then be simmed as though it
    // had.
    let mut tier = DEFAULT_TIER.to_string();
    let mut di = DirectInsertion::Candidate;
    let mut claim: Option<DiClaimOrder> = None;
    let mut belt: Option<String> = None;
    let mut duty: f64 = 1.0;
    let mut quality = QualityTier::Normal;
    let mut stacking: u8 = 1;
    let mut research_productivity: std::collections::BTreeMap<String, f64> =
        Default::default();
    let mut inserter_cap: Option<u8> = None;
    let mut row_layout = RowLayout::default();
    let mut strategy = LayoutStrategy::default();
    let mut inputs: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
    let mut label: Option<String> = None;
    let mut out = std::env::var("SIM_PROBE_OUT").unwrap_or_else(|_| "/tmp".to_string());

    while i < args.len() {
        let need = |i: usize| -> String {
            args.get(i + 1)
                .cloned()
                .unwrap_or_else(|| usage(&format!("{} needs a value", args[i])))
        };
        match args[i].as_str() {
            "--tier" => tier = need(i),
            "--di" => {
                di = match need(i).as_str() {
                    "off" => DirectInsertion::Off,
                    "forced" => DirectInsertion::Forced,
                    "candidate" => DirectInsertion::Candidate,
                    other => usage(&format!("--di: expected off|candidate|forced, got {other}")),
                }
            }
            "--claim" => {
                claim = Some(match need(i).as_str() {
                    "up" => DiClaimOrder::Upstream,
                    "down" => DiClaimOrder::Downstream,
                    "search" => DiClaimOrder::Search,
                    other => usage(&format!("--claim: expected up|down|search, got {other}")),
                })
            }
            "--belt" => belt = Some(need(i)),
            // #652: without this the tracked generator cannot express a
            // horizontal-stack fixture at all, so the corpus's HS class
            // (ac7-HS and friends) had no supported route to a
            // blueprint+manifest pair and could not be sim-anchored.
            "--row-layout" => {
                row_layout = match need(i).as_str() {
                    "native" | "default" => RowLayout::default(),
                    "horizontal-stack" | "hs" => RowLayout::HorizontalStack,
                    other => usage(&format!(
                        "--row-layout must be native|horizontal-stack (got {other})"
                    )),
                }
            }
            // Same gap `--row-layout` was added for: without this the
            // tracked generator cannot express a PartitionedDecomposed
            // fixture at all, so the RFC-modular-production strategy had
            // no supported route to a blueprint+manifest pair and could
            // not be sim-anchored. Its Phase 1 kill criteria are stated
            // against sim measurement, so the axis has to be reachable
            // from here.
            "--strategy" => {
                strategy = match need(i).as_str() {
                    "pooled" | "default" => LayoutStrategy::Pooled,
                    "partitioned-decomposed" | "pd" => LayoutStrategy::PartitionedDecomposed,
                    other => usage(&format!(
                        "--strategy must be pooled|default|partitioned-decomposed|pd (got {other})"
                    )),
                }
            }
            // RFC-069 Phase 1: planning-duty knob for the K69-1 sim A/B.
            "--duty" => {
                duty = need(i)
                    .parse()
                    .ok()
                    .filter(|d: &f64| *d > 0.0 && *d <= 1.0)
                    .unwrap_or_else(|| usage("--duty must be a float in (0, 1]"))
            }
            "--quality" => {
                let q = need(i);
                quality = QualityTier::from_name(&q)
                    .unwrap_or_else(|| usage(&format!("--quality: unknown tier {q}")))
            }
            "--stacking" => {
                stacking = need(i)
                    .parse()
                    .unwrap_or_else(|_| usage("--stacking must be 1..4"))
            }
            // Declared research productivity, e.g.
            //   --research-productivity processing-unit=0.10,plastic-bar=0.10
            // Passed to BOTH the solve and the layout, deliberately from one
            // parsed value: they are separate knobs (NetflowOptions vs
            // LayoutOptions) with no consistency guard between them, so a
            // caller that sets one and not the other would plan in a
            // different world from the one it declares on the manifest — the
            // exact class this axis exists to eliminate.
            "--research-productivity" => {
                match spaghettio_core::module_policy::parse_declared_productivity(&need(i)) {
                    Ok(map) => research_productivity.extend(map),
                    Err(e) => usage(&e),
                }
            }
            "--inserter-cap" => {
                inserter_cap = Some(
                    need(i)
                        .parse()
                        .unwrap_or_else(|_| usage("--inserter-cap must be a number")),
                )
            }
            "--inputs" => {
                inputs = need(i).split(',').map(|s| s.trim().to_string()).collect();
            }
            "--label" => label = Some(need(i)),
            "--out" => out = need(i),
            other => usage(&format!("unknown flag {other}")),
        }
        i += 2;
    }

    // Joins every target into one label/description — `{item}-{rate}` for
    // N=1 (byte-identical to the pre-Phase-3 default label) and
    // `{item1}-{rate1}_{item2}-{rate2}...` for N>1.
    let targets_desc = || -> String {
        targets
            .iter()
            .map(|(item, rate)| format!("{item}@{rate}"))
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let label = label.unwrap_or_else(|| {
        let base = targets
            .iter()
            .map(|(item, rate)| format!("{item}-{rate}"))
            .collect::<Vec<_>>()
            .join("_")
            .replace('.', "_");
        auto_label(
            &base,
            &LabelAxes {
                strategy,
                row_layout,
                duty,
                tier: &tier,
                belt: belt.as_deref(),
                quality,
                stacking,
                di,
                claim: claim.as_ref(),
                inserter_cap,
                inputs: &inputs,
                research_productivity: &research_productivity,
            },
        )
    });

    let input_set: FxHashSet<String> = inputs.iter().cloned().collect();

    // RFC-062 Phase 3: always the multi-target entry point, even for N=1
    // — bit-identical to the old scalar
    // `solve_with_palette_exclusions_and_quality` call by construction
    // (kill criterion 5), so there is one solve code path here, not a
    // single/multi fork.
    let solved = spaghettio_core::netflow::solve_netflow_multi_with_options(
        &targets,
        &input_set,
        &MachinePalette::default(),
        &tier,
        &FxHashSet::default(),
        spaghettio_core::netflow::RecipeScope::Free,
        &spaghettio_core::netflow::CostTable::default(),
        &spaghettio_core::netflow::NetflowOptions {
            quality,
            module_policy: spaghettio_core::module_policy::ModulePolicy::default(),
            research_productivity: research_productivity.clone(),
            // Everything else exactly as
            // `solve_multi_with_palette_exclusions_quality_and_modules` set
            // it — this call replaces that wrapper only to reach the one
            // field it cannot pass, and must not change anything else.
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        eprintln!("solve failed for {} on {tier}: {e}", targets_desc());
        std::process::exit(1);
    });

    // The fitted duty values are TIER-RELATIVE (bot review round 2):
    // block = floor(in_lane_cap(tier) × 2 × duty / rate), and with no
    // --belt the cap resolves to the express default (belt_cap 45), so
    // --duty 0.6 silently computes the measured-DEAD block 6 instead of
    // the gate-clearing block 2. Require the tier to be explicit.
    if duty < 1.0 && belt.is_none() {
        usage("--duty < 1 requires an explicit --belt (the fitted duty is tier-relative; the RFC-069 gate receipts are on transport-belt)");
    }
    let mut opts = LayoutOptions {
        direct_insertion: di,
        max_belt_tier: belt,
        planning_duty: duty,
        row_layout,
        strategy,
        quality,
        stacking,
        research_productivity,
        ..Default::default()
    };
    if let Some(c) = claim {
        opts.di_claim_order = c;
    }
    if let Some(c) = inserter_cap {
        opts.inserter_capacity = c;
    }

    let layout = build_bus_layout(&solved, opts).unwrap_or_else(|e| {
        eprintln!("layout failed for {} on {tier}: {e}", targets_desc());
        std::process::exit(1);
    });

    let issues = validate::validate(&layout, Some(&solved), LayoutStyle::Bus)
        .unwrap_or_else(|e| e.issues);
    let errors = issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warnings = issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .count();

    // Pass the issues from just above so the manifest records this layout's
    // validator state — this generator's whole output is sim fixtures, which
    // is exactly where a rate must not be read without it.
    // `export_with_manifest` is a pure export and emits no `validator` key.
    let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest_validated(
        &layout, &solved, &label, &issues,
    );
    let dir = format!("{out}/{label}");
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| {
        eprintln!("cannot create {dir}: {e}");
        std::process::exit(1);
    });
    let bp_path = format!("{dir}/bp.txt");
    let mf_path = format!("{dir}/manifest-real.json");
    std::fs::write(&bp_path, &bp).unwrap_or_else(|e| {
        eprintln!("cannot write {bp_path}: {e}");
        std::process::exit(1);
    });
    std::fs::write(
        &mf_path,
        serde_json::to_string_pretty(&manifest).expect("manifest serializes"),
    )
    .unwrap_or_else(|e| {
        eprintln!("cannot write {mf_path}: {e}");
        std::process::exit(1);
    });

    println!(
        "{label}: {}x{} {} entities, {errors} error(s) / {warnings} warning(s)",
        layout.width,
        layout.height,
        layout.entities.len()
    );
    for f in &solved.external_inputs {
        println!(
            "    boundary input  {:<22} {:>9.4}/s  fluid={}",
            f.item, f.rate, f.is_fluid
        );
    }
    for f in &solved.external_outputs {
        println!(
            "    target output   {:<22} {:>9.4}/s  fluid={}",
            f.item, f.rate, f.is_fluid
        );
    }
    println!("  {bp_path}");
    println!("  {mf_path}");
    println!("\nrun it:");
    println!(
        "  cargo run --release -p spaghettio_sim_harness -- run \\\n    --bp {bp_path} --manifest {mf_path} --warmup 288000"
    );
}


#[cfg(test)]
mod tests {
    use super::*;

    fn axes<'a>() -> LabelAxes<'a> {
        // Every field taken FROM `LayoutOptions::default()` rather than named
        // (#661 round 5) — a fixture that hardcodes defaults goes stale the
        // same way the production code did, and then passes anyway.
        let d = LayoutOptions::default();
        LabelAxes {
            strategy: d.strategy,
            row_layout: d.row_layout,
            duty: d.planning_duty,
            tier: DEFAULT_TIER,
            belt: None,
            quality: d.quality,
            stacking: d.stacking,
            di: d.direct_insertion,
            claim: None,
            inserter_cap: None,
            inputs: &[],
            research_productivity: &EMPTY_RP,
        }
    }

    static EMPTY_RP: std::sync::LazyLock<std::collections::BTreeMap<String, f64>> =
        std::sync::LazyLock::new(std::collections::BTreeMap::new);
    static DEFAULT_CLAIM: std::sync::LazyLock<DiClaimOrder> =
        std::sync::LazyLock::new(DiClaimOrder::default);
    static NON_DEFAULT_CLAIM: std::sync::LazyLock<DiClaimOrder> =
        std::sync::LazyLock::new(|| {
            // Whichever arm is NOT the default, derived rather than named, so
            // a default flip does not silently make this test vacuous.
            if DiClaimOrder::default() == DiClaimOrder::Upstream {
                DiClaimOrder::Downstream
            } else {
                DiClaimOrder::Upstream
            }
        });

    /// The blocker from round 4: the default arm must not tag, and the
    /// non-default arm MUST — checked against `DiClaimOrder::default()` so
    /// neither direction can silently invert if the default changes.
    #[test]
    fn claim_tags_track_the_real_default() {
        let default_inputs: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
        let mk = |c| {
            let mut ax = axes();
            ax.inputs = &default_inputs;
            ax.claim = Some(c);
            auto_label("ec-5", &ax)
        };
        assert_eq!(mk(&DEFAULT_CLAIM), "ec-5", "the default claim order must not fork the path");
        assert_ne!(
            mk(&NON_DEFAULT_CLAIM),
            "ec-5",
            "a non-default claim order changes the artifact and must tag"
        );
    }

    /// `--inputs` is consumed as an unordered set, so a reordering of the
    /// same items is the same artifact and must share a directory.
    #[test]
    fn input_order_does_not_fork_the_path() {
        let fwd: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
        let mut rev = fwd.clone();
        rev.reverse();
        let mut a = axes();
        a.inputs = &fwd;
        let mut b = axes();
        b.inputs = &rev;
        assert_eq!(auto_label("ec-5", &a), auto_label("ec-5", &b));
        assert_eq!(auto_label("ec-5", &a), "ec-5", "the default set must not tag at all");
    }

    /// The contract: default config keeps the pre-#661 path byte-identical.
    #[test]
    fn defaults_produce_the_bare_base() {
        let default_inputs: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
        let mut ax = axes();
        ax.inputs = &default_inputs;
        assert_eq!(auto_label("electronic-circuit-5", &ax), "electronic-circuit-5");
    }

    /// The bug this PR exists to fix: two strategies must not share a path.
    #[test]
    fn strategy_separates_the_two_arms() {
        let default_inputs: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
        let mut pooled = axes();
        pooled.inputs = &default_inputs;
        let mut pd = axes();
        pd.inputs = &default_inputs;
        pd.strategy = LayoutStrategy::PartitionedDecomposed;

        let a = auto_label("ec-5", &pooled);
        let b = auto_label("ec-5", &pd);
        assert_ne!(a, b, "pooled and partitioned-decomposed collided: {a}");
        assert_eq!(b, "ec-5-pd");
    }

    /// The cross-comparison the round-2 review named: `--duty < 1` requires an
    /// explicit `--belt`, so duty fixtures always pin a belt tier and the two
    /// belt tiers must still separate.
    #[test]
    fn duty_and_belt_separate_together() {
        let default_inputs: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
        let mk = |belt| {
            let mut ax = axes();
            ax.inputs = &default_inputs;
            ax.duty = 0.6;
            ax.belt = Some(belt);
            auto_label("ec-5", &ax)
        };
        let yellow = mk("transport-belt");
        let fast = mk("fast-transport-belt");
        assert_ne!(yellow, fast, "duty/belt cross-comparison collided");
        assert_eq!(yellow, "ec-5-duty0_6-yellow");
    }

    /// Exact float comparison, not epsilon: a duty a hair under 1.0 lays out
    /// differently and must not share the default's directory.
    #[test]
    fn near_one_duty_still_tags() {
        let default_inputs: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
        let mut ax = axes();
        ax.inputs = &default_inputs;
        ax.duty = 0.9999999999999999;
        assert_ne!(
            auto_label("ec-5", &ax),
            "ec-5",
            "a duty within f64::EPSILON of 1.0 collided with the default"
        );
    }

    /// Passing a flag whose value IS the engine default must not fork the
    /// directory — the artifact is identical, so the path should be too.
    #[test]
    fn explicit_default_values_do_not_fork_the_path() {
        let default_inputs: Vec<String> = DEFAULT_INPUTS.iter().map(|s| s.to_string()).collect();
        let mut ax = axes();
        ax.inputs = &default_inputs;
        ax.inserter_cap = Some(spaghettio_core::common::DEFAULT_INSERTER_CAPACITY);
        ax.claim = Some(&DEFAULT_CLAIM);
        assert_eq!(
            auto_label("ec-5", &ax),
            "ec-5",
            "explicitly passing the DEFAULT inserter-cap and claim order \
             produces the default artifact and must not create a second \
             directory for it"
        );
    }

    /// `--inputs ""` parses to `[""]` — a NON-default set that joins to the
    /// empty string. Keying the tag on "digest string is non-empty" dropped
    /// it into the default directory; keying on "does the set differ" does
    /// not (#661 round 5).
    #[test]
    fn empty_input_string_does_not_collapse_onto_the_default() {
        let empty: Vec<String> = vec![String::new()];
        let mut ax = axes();
        ax.inputs = &empty;
        assert_ne!(
            auto_label("ec-5", &ax),
            "ec-5",
            "an empty-string input set differs from the default and must tag"
        );
    }

    /// Non-default list-valued axes are digested rather than spelled out, but
    /// must still separate.
    #[test]
    fn custom_inputs_digest_and_separate() {
        let a_in: Vec<String> = vec!["iron-plate".into()];
        let b_in: Vec<String> = vec!["copper-plate".into()];
        let mut a = axes();
        a.inputs = &a_in;
        let mut b = axes();
        b.inputs = &b_in;
        let la = auto_label("ec-5", &a);
        let lb = auto_label("ec-5", &b);
        assert_ne!(la, lb, "different input sets collided");
        assert!(la.contains("-cfg"), "expected a digest tag, got {la}");
    }
}
