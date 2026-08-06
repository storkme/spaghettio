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
use spaghettio_core::bus::layout::{build_bus_layout, LayoutOptions};
use spaghettio_core::common::QualityTier;
use spaghettio_core::recipe_db::MachinePalette;
use spaghettio_core::validate::{self, LayoutStyle, Severity};

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
    let mut tier = "assembling-machine-3".to_string();
    let mut di = DirectInsertion::Candidate;
    let mut claim: Option<DiClaimOrder> = None;
    let mut belt: Option<String> = None;
    let mut quality = QualityTier::Normal;
    let mut stacking: u8 = 1;
    let mut research_productivity: std::collections::BTreeMap<String, f64> =
        Default::default();
    let mut inserter_cap: Option<u8> = None;
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
        targets
            .iter()
            .map(|(item, rate)| format!("{item}-{rate}"))
            .collect::<Vec<_>>()
            .join("_")
            .replace('.', "_")
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

    let mut opts = LayoutOptions {
        direct_insertion: di,
        max_belt_tier: belt,
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

    let (bp, manifest) = spaghettio_core::blueprint::export_with_manifest(&layout, &solved, &label);
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
