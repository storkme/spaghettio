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
//!   --strategy <kind>      pooled (aka default) | partitioned-decomposed (aka pd)
//!   --duty <0..1>          planning duty (default 1.0; <1 needs --belt)
//!   --quality <name>       normal|uncommon|rare|epic|legendary (default normal)
//!   --stacking <1..4>      belt stacking (default 1)
//!   --research-productivity <recipe=bonus,...>   declared research
//!                          productivity, e.g. processing-unit=0.10 (default none)
//!   --inserter-cap <n>     inserter capacity level (default: engine default)
//!   --inputs a,b,c         raw inputs (default: the six-ore set)
//!   --label <name>         output subdirectory + manifest label
//!   --force                overwrite an existing <out>/<label>/bp.txt
//!                          REQUIRED whenever any layout-changing flag is used
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

/// Default crafting machine — the one `--tier` overrides.
const DEFAULT_TIER: &str = "assembling-machine-3";

/// Flags that change the exported artifact, and therefore make the default
/// `{item}-{rate}` label ambiguous.
///
/// The label IS the output directory (`<out>/<label>/`), so two runs that
/// differ by any of these must not share it. The original bug (#661) was that
/// the label encoded none of them and the second run silently overwrote the
/// first's `bp.txt` and `manifest-real.json` — a wrong-A/B generator, in the
/// tool whose whole purpose is A/B.
///
/// Rather than encode each axis into the name — which needs every axis
/// enumerated and every engine default stated correctly, and which took five
/// review rounds without converging — passing any of these simply REQUIRES an
/// explicit `--label`. Collisions between runs that differ only by an axis
/// become impossible by construction — reusing the same `--label` twice
/// still collides, and deliberately so — and the
/// check consults no defaults, so a future default flip cannot invert it.
///
/// Slightly over-strict on purpose: `--strategy pooled` is the default value
/// and still demands a label. Erring toward "be explicit" costs a flag; erring
/// the other way silently corrupts a measurement.
const AXIS_FLAGS: &[&str] = &[
    "--tier",
    "--di",
    "--claim",
    "--belt",
    "--row-layout",
    "--strategy",
    "--duty",
    "--quality",
    "--stacking",
    "--research-productivity",
    "--inserter-cap",
    "--inputs",
];

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


/// The label a run falls back to when `--label` is absent.
///
/// One definition, because the refusal message quotes it: when these were
/// two expressions the message described `{item}-{rate}` while a --multi
/// run actually used the joined list (#661 review).
fn default_label(targets: &[(String, f64)]) -> String {
    targets
        .iter()
        .map(|(item, rate)| format!("{item}-{rate}"))
        .collect::<Vec<_>>()
        .join("_")
        .replace('.', "_")
}

/// Does this invocation need an explicit `--label`?
///
/// True when any artifact-changing flag was passed. Kept as a pure function
/// so the rule is testable — the previous design put per-axis default
/// comparisons inline in `main()`, where nothing exercised them and five
/// review rounds each found another axis compared against the wrong default.
///
/// Note what it does NOT take: any value, and any default. It is a question
/// about the command line, not about the configuration — which is also its
/// limit: it cannot tell two runs apart that pass the SAME axis with
/// DIFFERENT values under one `--label`. That case is caught at the write
/// instead, where the artifact is.
fn label_required(axis_flags_seen: &[String], label: &Option<String>) -> bool {
    !axis_flags_seen.is_empty() && label.is_none()
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
    let mut force = false;
    let mut axis_flags_seen: Vec<String> = Vec::new();
    let mut out = std::env::var("SIM_PROBE_OUT").unwrap_or_else(|_| "/tmp".to_string());

    while i < args.len() {
        let need = |i: usize| -> String {
            args.get(i + 1)
                .cloned()
                .unwrap_or_else(|| usage(&format!("{} needs a value", args[i])))
        };
        // Purely SYNTACTIC: was the flag passed? This deliberately does not
        // look at the VALUE or compare it to any default — see
        // `AXIS_FLAGS`. Five review rounds went into getting per-axis
        // default comparisons right and each one found another axis I had
        // wrong; not consulting a default at all is the fix.
        if AXIS_FLAGS.contains(&args[i].as_str()) {
            axis_flags_seen.push(args[i].clone());
        }
        // The one valueless flag, so it advances by 1 rather than the 2
        // every other branch assumes.
        if args[i] == "--force" {
            force = true;
            i += 1;
            continue;
        }
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
                    "pooled" => LayoutStrategy::Pooled,
                    // Delegate, matching `--row-layout`'s "native"|"default"
                    // (#661 review). Hardcoding Pooled here meant a change to
                    // the #[default] would silently stop being what
                    // `--strategy default` selects.
                    "default" => LayoutStrategy::default(),
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
    // The fitted duty values are TIER-RELATIVE (bot review round 2):
    // block = floor(in_lane_cap(tier) × 2 × duty / rate), and with no
    // --belt the cap resolves to the express default (belt_cap 45), so
    // --duty 0.6 silently computes the measured-DEAD block 6 instead of
    // the gate-clearing block 2. Require the tier to be explicit.
    if duty < 1.0 && belt.is_none() {
        usage("--duty < 1 requires an explicit --belt (the fitted duty is tier-relative; the RFC-069 gate receipts are on transport-belt)");
    }

    // Any axis flag without an explicit label is refused, because the label
    // is the output directory and `{item}-{rate}` cannot distinguish the runs.
    if label_required(&axis_flags_seen, &label) {
        // Name the directory this run would ACTUALLY have used. The
        // default label is `{item}-{rate}` only for a single target; under
        // --multi it is the joined list, so the old hardcoded
        // `<out>/{item}-{rate}/` described the wrong path back to anyone
        // hitting this in multi mode (#661 review).
        usage(&format!(
            "{} changes the exported layout, so `--label <name>` is required: \
             without it this run writes to <out>/{}/ and silently overwrites \
             any other run of the same target",
            axis_flags_seen.join(", "),
            default_label(&targets)
        ));
    }
    let label = label.unwrap_or_else(|| default_label(&targets));

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
    let bp_path = format!("{dir}/bp.txt");
    let mf_path = format!("{dir}/manifest-real.json");

    // The label guard is necessary but NOT sufficient (#661 review, major).
    // It reasons about flag PRESENCE, so it cannot separate two runs that
    // pass the same axis with different values under one `--label` —
    // `--strategy pd --label x` then `--strategy pooled --label x` both
    // satisfy it and the second silently overwrote the first. That is the
    // original wrong-A/B bug, reachable through the guard added to prevent
    // it. Presence is knowable at parse time; artifact identity is only
    // knowable here, so the second half of the invariant belongs at the
    // write.
    if !force && std::path::Path::new(&bp_path).exists() {
        eprintln!(
            "error: {bp_path} already exists — refusing to overwrite it.\n\
             \n\
             A previous run wrote this label. If that run used different flags, \
             overwriting it is the silent wrong-A/B this tool exists to avoid; \
             give this run its own --label. If you are deliberately \
             regenerating the same configuration, pass --force."
        );
        std::process::exit(1);
    }

    std::fs::create_dir_all(&dir).unwrap_or_else(|e| {
        eprintln!("cannot create {dir}: {e}");
        std::process::exit(1);
    });
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

    /// The rule: any artifact-changing flag demands an explicit `--label`.
    #[test]
    fn axis_flags_require_a_label() {
        let none: Option<String> = None;
        let some = Some("ac7hs-duty06-pd".to_string());

        // No axis flags: the `{item}-{rate}` default is unambiguous.
        assert!(!label_required(&[], &none));
        assert!(!label_required(&[], &some));

        // Any axis flag without a label is refused...
        for f in AXIS_FLAGS {
            assert!(
                label_required(&[f.to_string()], &none),
                "{f} changes the artifact and must demand a label"
            );
            // ...and is fine with one.
            assert!(!label_required(&[f.to_string()], &some), "{f} with a label");
        }
    }

    /// `AXIS_FLAGS` and the documented flag list must agree, BOTH ways.
    ///
    /// One direction catches "a new axis was added and forgotten", which is
    /// the failure that produced five review rounds. The other catches the
    /// hole this test itself shipped with: `--duty` was in `AXIS_FLAGS` but
    /// missing from the usage block, so the pin could not see it and would
    /// have passed while `--duty` silently lost its guard.
    ///
    /// `--label`, `--out` and `--multi` are excluded deliberately: they name
    /// or select the output, they do not change it.
    #[test]
    fn axis_flag_list_and_docs_agree_both_ways() {
        // Not axes: these change WHERE a run writes, or WHETHER it may,
        // never WHAT it exports. `--force` joined them when the overwrite
        // guard landed — and this test caught the omission on its first
        // run, which is the whole reason it is bidirectional.
        const NOT_AXES: &[&str] = &["--label", "--out", "--multi", "--force"];
        let doc = include_str!("sim_export.rs");
        let documented: Vec<String> = doc
            .lines()
            .filter(|l| l.trim_start().starts_with("//!   --"))
            .filter_map(|l| l.split_whitespace().nth(1).map(str::to_string))
            .filter(|f| f.starts_with("--") && !NOT_AXES.contains(&f.as_str()))
            .collect();
        assert!(!documented.is_empty(), "usage block not found");

        for f in &documented {
            assert!(
                AXIS_FLAGS.contains(&f.as_str()),
                "{f} is documented but missing from AXIS_FLAGS, so passing it \
                 would not demand a label and two runs could collide"
            );
        }
        for f in AXIS_FLAGS {
            assert!(
                documented.iter().any(|d| d == f),
                "{f} is in AXIS_FLAGS but undocumented, so the check above \
                 cannot see it — the guard would decay silently"
            );
        }
    }
}
