use spaghettio_meter::belt::ItemId;
use spaghettio_meter::factory::Factory;
use spaghettio_meter::manifest::Manifest;
fn main() {
    let root = std::env::args().nth(1).unwrap();
    let bp = std::fs::read_to_string(format!("{root}/bp.txt")).unwrap();
    let m = Manifest::from_path(format!("{root}/manifest-real.json")).unwrap();
    let f = Factory::build(&bp, m).unwrap();
    let items = &f.items;
    for (i, net) in f.fluids.networks.iter().enumerate() {
        if net.ports.is_empty() && net.boundary.is_empty() { continue; }
        println!("== network#{i} boundary={:?}", net.boundary.iter().map(|b| items.name(ItemId(*b)).to_string()).collect::<Vec<_>>());
        for p in &net.ports {
            let item = items.name(ItemId(p.item)).to_string();
            let mname = &f.machines[p.machine].name;
            println!("   m#{} {mname} {} {} ({},{})", p.machine, if p.is_input{"<-"}else{"->"}, item, p.x, p.y);
        }
    }
}
