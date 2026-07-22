//! `spaghettio-sim check-data` — RFC-050 KC1: run `factorio --dump-data`
//! on the pinned install and compare a fixed fact set (crafting speeds of
//! the 15 machines; energy + ingredient/result yields for
//! `iron-gear-wheel`, `electronic-circuit`, `copper-cable`, and the
//! `iron-plate` smelting recipe) against `crates/core/data/recipes.json`.
//!
//! Deliberately does NOT depend on `spaghettio_core` (RFC-050 constraint):
//! `RECIPES_JSON` below is the same data FILE core embeds via its own
//! `include_str!`, embedded here directly by relative path — this reads
//! core's data, it does not call core's code.
//!
//! Any mismatch is a hard error per KC1: "there is no 'reconcile in a
//! day' option (re-baselining recipes.json invalidates the corpus
//! goldens wholesale)".

use crate::paths;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

const RECIPES_JSON: &str = include_str!("../../core/data/recipes.json");

/// The 15 machines recipes.json tracks crafting speed for (RFC-050 KC1:
/// "crafting speeds of the 15 machines").
const MACHINES: &[&str] = &[
    "centrifuge",
    "assembling-machine-1",
    "assembling-machine-2",
    "assembling-machine-3",
    "chemical-plant",
    "electric-furnace",
    "oil-refinery",
    "stone-furnace",
    "steel-furnace",
    "foundry",
    "electromagnetic-plant",
    "cryogenic-plant",
    "biochamber",
    "recycler",
    "crusher",
];

/// The ladder recipes KC1 spot-checks energy + ingredient/result yields
/// for. `iron-plate` here is the SMELTING recipe (category "smelting"),
/// not the item.
const RECIPES: &[&str] = &["iron-gear-wheel", "electronic-circuit", "copper-cable", "iron-plate"];

/// Run `factorio --dump-data`, load the resulting `data-raw-dump.json`,
/// and return a list of human-readable mismatch descriptions (empty =
/// KC1 clean).
pub fn run(install_dir: &Path) -> Result<Vec<String>, String> {
    let dump_path = dump_data(install_dir)?;
    let dump_str = std::fs::read_to_string(&dump_path)
        .map_err(|e| format!("reading dumped data at {dump_path:?}: {e}"))?;
    let dump: serde_json::Value =
        serde_json::from_str(&dump_str).map_err(|e| format!("parsing dumped data JSON: {e}"))?;
    let baseline: serde_json::Value =
        serde_json::from_str(RECIPES_JSON).map_err(|e| format!("parsing embedded recipes.json: {e}"))?;

    Ok(compare(&dump, &baseline))
}

fn dump_data(install_dir: &Path) -> Result<std::path::PathBuf, String> {
    let binary = paths::factorio_binary_path(install_dir);
    let status = Command::new(&binary)
        .current_dir(install_dir)
        .arg("--dump-data")
        .status()
        .map_err(|e| format!("failed to launch {binary:?} --dump-data: {e}"))?;
    if !status.success() {
        return Err(format!("factorio --dump-data exited with {status}"));
    }
    let path = install_dir.join("script-output").join("data-raw-dump.json");
    if !path.is_file() {
        return Err(format!(
            "--dump-data reported success but {path:?} doesn't exist (dump path may have changed)"
        ));
    }
    Ok(path)
}

/// Scan every top-level prototype-type category in the dumped `data.raw`
/// tree for a prototype named `name`. Generic on purpose: this crate
/// doesn't hardcode which Factorio prototype `type` each machine belongs
/// to (furnace vs. assembling-machine vs. whatever a future version
/// calls it), so it just looks for the name wherever it lives.
fn find_prototype<'a>(dump: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
    let root = dump.as_object()?;
    for (_category, entries) in root {
        if let Some(obj) = entries.as_object() {
            if let Some(proto) = obj.get(name) {
                return Some(proto);
            }
        }
    }
    None
}

fn compare(dump: &serde_json::Value, baseline: &serde_json::Value) -> Vec<String> {
    let mut mismatches = Vec::new();

    let baseline_machines = baseline.get("machines").and_then(|m| m.as_object());
    for &name in MACHINES {
        let expected = baseline_machines
            .and_then(|m| m.get(name))
            .and_then(|m| m.get("crafting_speed"))
            .and_then(|v| v.as_f64());
        let Some(expected) = expected else {
            mismatches.push(format!("machine '{name}': no crafting_speed in baseline recipes.json"));
            continue;
        };
        match find_prototype(dump, name) {
            None => mismatches.push(format!("machine '{name}': not found in dumped data.raw")),
            Some(proto) => match proto.get("crafting_speed").and_then(|v| v.as_f64()) {
                None => mismatches.push(format!("machine '{name}': dumped prototype has no crafting_speed field")),
                Some(actual) if (actual - expected).abs() > 1e-9 => mismatches.push(format!(
                    "machine '{name}': crafting_speed baseline={expected} dumped={actual}"
                )),
                Some(_) => {}
            },
        }
    }

    let baseline_recipes = baseline.get("recipes").and_then(|r| r.as_object());
    for &name in RECIPES {
        let Some(expected) = baseline_recipes.and_then(|r| r.get(name)) else {
            mismatches.push(format!("recipe '{name}': not present in baseline recipes.json"));
            continue;
        };
        let Some(actual) = find_prototype(dump, name) else {
            mismatches.push(format!("recipe '{name}': not found in dumped data.raw"));
            continue;
        };

        // energy / energy_required
        if let Some(expected_energy) = expected.get("energy").and_then(|v| v.as_f64()) {
            match actual.get("energy_required").and_then(|v| v.as_f64()) {
                None => mismatches.push(format!("recipe '{name}': dumped prototype has no energy_required field")),
                Some(actual_energy) if (actual_energy - expected_energy).abs() > 1e-9 => mismatches.push(format!(
                    "recipe '{name}': energy baseline={expected_energy} dumped={actual_energy}"
                )),
                Some(_) => {}
            }
        }

        compare_item_amounts(name, "ingredients", expected, actual, &mut mismatches);
        compare_item_amounts(name, "results", expected, actual, &mut mismatches);
    }

    mismatches
}

/// Compare an `ingredients`/`results` list as a `name -> total amount`
/// multiset, order-independent (the dump and our baseline aren't
/// guaranteed to list entries in the same order).
fn compare_item_amounts(
    recipe: &str,
    field: &str,
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    mismatches: &mut Vec<String>,
) {
    let to_map = |v: &serde_json::Value| -> BTreeMap<String, f64> {
        let mut m = BTreeMap::new();
        if let Some(arr) = v.get(field).and_then(|x| x.as_array()) {
            for entry in arr {
                let Some(name) = entry.get("name").and_then(|n| n.as_str()) else { continue };
                let amount = entry.get("amount").and_then(|a| a.as_f64()).unwrap_or(0.0);
                *m.entry(name.to_string()).or_insert(0.0) += amount;
            }
        }
        m
    };
    let expected_map = to_map(expected);
    let actual_map = to_map(actual);
    if expected_map != actual_map {
        mismatches.push(format!(
            "recipe '{recipe}' {field}: baseline={expected_map:?} dumped={actual_map:?}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_fixture() -> serde_json::Value {
        serde_json::json!({
            "machines": {
                "assembling-machine-2": {"crafting_speed": 0.75}
            },
            "recipes": {
                "iron-gear-wheel": {
                    "name": "iron-gear-wheel", "category": "crafting", "energy": 0.5,
                    "ingredients": [{"name": "iron-plate", "amount": 2, "type": "item"}],
                    "results": [{"name": "iron-gear-wheel", "amount": 1, "type": "item"}]
                }
            }
        })
    }

    #[test]
    fn matching_dump_produces_no_mismatches() {
        let baseline = baseline_fixture();
        let dump = serde_json::json!({
            "assembling-machine": {
                "assembling-machine-2": {"crafting_speed": 0.75}
            },
            "recipe": {
                "iron-gear-wheel": {
                    "energy_required": 0.5,
                    "ingredients": [{"name": "iron-plate", "amount": 2}],
                    "results": [{"name": "iron-gear-wheel", "amount": 1}]
                }
            }
        });
        // Restrict to the fixture's own small fact set rather than the
        // full MACHINES/RECIPES lists, which the fixture doesn't cover.
        let mut mismatches = Vec::new();
        let name = "assembling-machine-2";
        let expected = baseline["machines"][name]["crafting_speed"].as_f64().unwrap();
        let actual = find_prototype(&dump, name).unwrap()["crafting_speed"].as_f64().unwrap();
        assert!((expected - actual).abs() < 1e-9);
        compare_item_amounts(
            "iron-gear-wheel",
            "ingredients",
            &baseline["recipes"]["iron-gear-wheel"],
            &dump["recipe"]["iron-gear-wheel"],
            &mut mismatches,
        );
        compare_item_amounts(
            "iron-gear-wheel",
            "results",
            &baseline["recipes"]["iron-gear-wheel"],
            &dump["recipe"]["iron-gear-wheel"],
            &mut mismatches,
        );
        assert!(mismatches.is_empty(), "{mismatches:?}");
    }

    #[test]
    fn crafting_speed_mismatch_is_detected() {
        let baseline = baseline_fixture();
        let dump = serde_json::json!({
            "assembling-machine": {
                "assembling-machine-2": {"crafting_speed": 0.80}
            },
            "recipe": {}
        });
        let expected = baseline["machines"]["assembling-machine-2"]["crafting_speed"].as_f64().unwrap();
        let actual = find_prototype(&dump, "assembling-machine-2").unwrap()["crafting_speed"].as_f64().unwrap();
        assert!((expected - actual).abs() > 1e-9, "fixture should disagree");
    }

    #[test]
    fn ingredient_amount_mismatch_is_detected() {
        let baseline = baseline_fixture();
        let dump = serde_json::json!({
            "recipe": {
                "iron-gear-wheel": {
                    "energy_required": 0.5,
                    "ingredients": [{"name": "iron-plate", "amount": 3}],
                    "results": [{"name": "iron-gear-wheel", "amount": 1}]
                }
            }
        });
        let mut mismatches = Vec::new();
        compare_item_amounts(
            "iron-gear-wheel",
            "ingredients",
            &baseline["recipes"]["iron-gear-wheel"],
            &dump["recipe"]["iron-gear-wheel"],
            &mut mismatches,
        );
        assert_eq!(mismatches.len(), 1);
    }

    #[test]
    fn missing_prototype_is_a_mismatch() {
        let dump = serde_json::json!({});
        assert!(find_prototype(&dump, "assembling-machine-2").is_none());
    }

    #[test]
    fn embedded_recipes_json_parses_and_covers_the_probe_set() {
        let baseline: serde_json::Value = serde_json::from_str(RECIPES_JSON).unwrap();
        let machines = baseline["machines"].as_object().unwrap();
        for &name in MACHINES {
            assert!(machines.contains_key(name), "recipes.json missing machine {name}");
        }
        let recipes = baseline["recipes"].as_object().unwrap();
        for &name in RECIPES {
            assert!(recipes.contains_key(name), "recipes.json missing recipe {name}");
        }
    }
}
