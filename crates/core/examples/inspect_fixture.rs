//! Inspect a region fixture: replay and print the resulting entities.

use spaghettio_core::fixture::{replay_region_fixture, RegionFixture};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: inspect_fixture <path-to-fixture.json>");
    let raw = std::fs::read_to_string(&path).expect("cannot read fixture");
    let fixture: RegionFixture = serde_json::from_str(&raw).expect("cannot parse");
    let result = replay_region_fixture(&fixture);
    println!("fixture: {}", fixture.name);
    println!("seeds: {:?}", fixture.seeds);
    println!("cost: {:?}, capped: {}", result.cost, result.capped);
    println!("entities ({}):", result.entities.len());
    let mut sorted: Vec<_> = result.entities.iter().collect();
    sorted.sort_by_key(|e| (e.y, e.x));
    for e in sorted {
        println!(
            "  ({},{}) {} dir={:?} carries={:?} io={:?}",
            e.x, e.y, e.name, e.direction, e.carries, e.io_type
        );
    }
}
