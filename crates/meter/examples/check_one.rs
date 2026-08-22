use spaghettio_meter::factory::Factory;
use spaghettio_meter::manifest::Manifest;
fn main() {
    let root = std::env::args().nth(1).unwrap();
    let bp = std::fs::read_to_string(format!("{root}/bp.txt")).unwrap();
    let m = Manifest::from_path(format!("{root}/manifest-real.json")).unwrap();
    let mut f = Factory::build(&bp, m).unwrap();
    let r = f.measure(108_000, 216_000);
    println!("produced: {:?}", r.produced_per_s);
    println!("delivered: {:?}", r.delivered_per_s);
    println!("planned:  {:?}", r.planned_per_s);
    println!("notes:    {:?}", r.notes);
    println!("recipe attribution:");
    for (recipe, a) in &r.recipe_attribution {
        println!(
            "  {recipe}: machines={} crafts={} working_ticks={} output_blocked_ticks={} output_inserter_blocked_ticks={} item_shortage_ticks={} fluid_shortage_ticks={} supplied={:?} consumed={:?}",
            a.machines,
            a.crafts,
            a.working_ticks,
            a.output_blocked_ticks,
            a.output_inserter_blocked_ticks,
            a.item_shortage_ticks,
            a.fluid_shortage_ticks,
            a.fluid_supplied,
            a.fluid_consumed,
        );
    }
}
