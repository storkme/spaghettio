use spaghettio_meter::factory::Factory;
use spaghettio_meter::manifest::Manifest;
fn main() {
    let root = std::env::args().nth(1).unwrap();
    let bp = std::fs::read_to_string(format!("{root}/bp.txt")).unwrap();
    let m = Manifest::from_path(format!("{root}/manifest-real.json")).unwrap();
    let mut f = Factory::build(&bp, m).unwrap();
    let r = f.measure(108_000, 216_000);
    println!("notes: {:?}", r.notes);
    println!("produced: {:?}", r.produced_per_s);
    println!("delivered: {:?}", r.delivered_per_s);
    println!("census: {:?}", r.machine_census);
    // per-machine state + fluid details
    let items = f.items;
    for (i, mc) in f.machines.iter().enumerate() {
        let name = &mc.name;
        let needs: Vec<(String, u32)> = mc
            .fluid_needs
            .iter()
            .map(|(id, a)| {
                (
                    items.name(spaghettio_meter::belt::ItemId(*id)).to_string(),
                    *a,
                )
            })
            .collect();
        let have: Vec<(String, u32)> = mc
            .fluid_input
            .iter()
            .map(|(id, a)| {
                (
                    items.name(spaghettio_meter::belt::ItemId(*id)).to_string(),
                    *a,
                )
            })
            .collect();
        let out: Vec<(String, u32)> = mc
            .fluid_output
            .iter()
            .map(|(id, a)| {
                (
                    items.name(spaghettio_meter::belt::ItemId(*id)).to_string(),
                    *a,
                )
            })
            .collect();
        if !needs.is_empty() || !out.is_empty() || mc.name == "oil-refinery" {
            println!(
                "machine#{i} {name} state={:?} needs={needs:?} have={have:?} fout={out:?}",
                mc.state
            );
        }
    }
}
