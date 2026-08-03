use spaghettio_meter::factory::Factory;
use spaghettio_meter::manifest::Manifest;
fn main() {
    let root = std::env::args().nth(1).unwrap();
    let bp = std::fs::read_to_string(format!("{root}/bp.txt")).unwrap();
    let m = Manifest::from_path(format!("{root}/manifest-real.json")).unwrap();
    let mut f = Factory::build(&bp, m).unwrap();
    let r = f.measure(108_000, 216_000);
    println!("produced: {:?}", r.produced_per_s);
    println!("planned:  {:?}", r.planned_per_s);
    println!("notes:    {:?}", r.notes);
}
