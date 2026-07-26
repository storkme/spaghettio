//! Is the military residual a steady state, or an unfinished transient?
//!
//! Sweeps warmup length on one fixture and reports the measured rate plus
//! the convergence flag at each. If the deficit is a buffer-fill artifact
//! it shrinks with warmup; if it is a model defect it does not move.
use std::path::PathBuf;
use spaghettio_meter::{Factory, Manifest};

fn main() {
    let label = std::env::args().nth(1).unwrap_or_else(|| "chain-mil5plates-d0".into());
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/target/tmp");
    let bp = std::fs::read_to_string(dir.join(format!("{label}.bp"))).expect("blueprint");
    const WINDOW: u64 = 60 * 60 * 3;
    println!("{label}  (window {WINDOW} ticks)");
    println!("{:>10} {:>12} {:>10} {:>11}", "warmup", "target/s", "d%", "converged");
    for mins in [40u64, 80, 160, 320] {
        let manifest = Manifest::from_path(dir.join(format!("{label}.manifest.json"))).expect("m");
        let target = manifest.targets.first().map(|t| t.item.clone()).unwrap_or_default();
        let planned = manifest.planned_rates.get(&target).copied().unwrap_or(1.0);
        let mut f = Factory::build(&bp, manifest).expect("build");
        let r = f.measure(60 * 60 * mins, WINDOW);
        let got = r.produced_per_s.get(&target).copied().unwrap_or(0.0);
        println!("{:>8}m {:>12.2} {:>9.1}% {:>11}",
            mins, got, 100.0 * (got / planned - 1.0), r.converged);
    }
}
