//! Which of the two lane-rate walkers is wrong? Distribution + the runaway.
use spaghettio_core::bus::di_cell::DirectInsertion;
use spaghettio_core::{bus::layout, solver};

/// `(tile, belt_flow lanes, belt_structural lanes, max per-lane delta)`.
type DivergenceRow = ((i32, i32), [f64; 2], [f64; 2], f64);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let which = args.get(1).map(|s| s.as_str()).unwrap_or("sci2");
    let (item, rate, machine, inputs): (&str, f64, &str, Vec<&str>) = match which {
        "sci2" => ("logistic-science-pack", 5.0, "assembling-machine-2", vec!["iron-plate", "copper-plate"]),
        "pu" => ("processing-unit", 2.0, "assembling-machine-3", vec!["iron-plate", "copper-plate", "coal", "crude-oil", "water", "sulfur"]),
        _ => ("electronic-circuit", 30.0, "assembling-machine-2", vec!["iron-ore", "copper-ore"]),
    };
    let set: rustc_hash::FxHashSet<String> = inputs.iter().map(|s| s.to_string()).collect();
    let sr = solver::solve(item, rate, &set, machine).unwrap();
    let opts = layout::LayoutOptions {
        direct_insertion: DirectInsertion::Forced,
        ..Default::default()
    };
    let l = layout::build_bus_layout(&sr, opts).unwrap();

    let flow = spaghettio_core::validate::belt_flow::compute_lane_rates(&l, Some(&sr));
    let strct = spaghettio_core::validate::belt_structural::compute_lane_rates(&l, &sr);

    let seg_of: std::collections::HashMap<(i32, i32), String> = l
        .entities
        .iter()
        .map(|e| ((e.x, e.y), format!("{}|{}", e.name, e.segment_id.as_deref().unwrap_or("-"))))
        .collect();

    let mut f_tot: Vec<f64> = flow.values().map(|v| v[0] + v[1]).collect();
    let mut s_tot: Vec<f64> = strct.values().map(|v| v.0 + v.1).collect();
    f_tot.sort_by(|a, b| b.partial_cmp(a).unwrap());
    s_tot.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let sum = |v: &[f64]| v.iter().sum::<f64>();
    println!("=== {which}: {} tiles(flow) {} tiles(struct) ===", flow.len(), strct.len());
    println!("belt_flow   total={:.0} max={:.1} top10={:?}", sum(&f_tot), f_tot[0],
        f_tot.iter().take(10).map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>());
    println!("belt_struct total={:.0} max={:.1} top10={:?}", sum(&s_tot), s_tot[0],
        s_tot.iter().take(10).map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>());

    // biggest divergences, with what the tile actually is
    let mut rows: Vec<DivergenceRow> = Vec::new();
    for (&k, &f) in &flow {
        let s = strct.get(&k).copied().map(|(a, b)| [a, b]).unwrap_or([0.0, 0.0]);
        let d = (f[0] - s[0]).abs().max((f[1] - s[1]).abs());
        rows.push((k, f, s, d));
    }
    rows.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    println!("\n--- top 20 divergences ---");
    for (k, f, s, d) in rows.iter().take(20) {
        println!(
            "  {k:?} d={d:>8.1}  flow=[{:.1},{:.1}] struct=[{:.1},{:.1}]  {}",
            f[0], f[1], s[0], s[1],
            seg_of.get(k).map(|x| x.as_str()).unwrap_or("?")
        );
    }

    // how many tiles does each model think are over a yellow lane (7.5/s)?
    let over = |v: [f64; 2]| v[0] > 7.5 || v[1] > 7.5;
    println!(
        "\ntiles over 7.5/s per lane:  flow={}  struct={}",
        flow.values().filter(|v| over(**v)).count(),
        strct.values().filter(|v| over([v.0, v.1])).count()
    );
}
