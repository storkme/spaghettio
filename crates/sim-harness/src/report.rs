//! Report computation: planned vs measured per item, PASS/WARN/FAIL
//! verdict per RFC-050 KC2 (one-sided — overshoot is expected because
//! placed machine counts are ceil'd above the fractional plan, and is
//! reported informationally, never penalized).

use crate::manifest::Manifest;
use std::collections::BTreeMap;
use std::fmt;

/// KC2's PASS boundary, verbatim from the RFC: "Measured target rate >=
/// 0.98 x planned ... at steady state".
const PASS_RATIO: f64 = 0.98;
/// WARN/FAIL split. The RFC only pins the PASS boundary; this floor is a
/// resolved ambiguity (documented here rather than silently invented) —
/// below it the shortfall looks like more than measurement noise and
/// should read as a hard failure rather than "close but under".
const WARN_RATIO: f64 = 0.90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
    /// No measurement was available to judge (e.g. the run never reached
    /// a stability checkpoint AND produced no samples either).
    NoData,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Verdict::Pass => "PASS",
            Verdict::Warn => "WARN",
            Verdict::Fail => "FAIL",
            Verdict::NoData => "NO DATA",
        })
    }
}

fn verdict_for_ratio(ratio: Option<f64>) -> Verdict {
    match ratio {
        None => Verdict::NoData,
        Some(r) if r >= PASS_RATIO => Verdict::Pass,
        Some(r) if r >= WARN_RATIO => Verdict::Warn,
        _ => Verdict::Fail,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ItemReport {
    pub item: String,
    pub planned_rate: f64,
    pub measured_produced_rate: Option<f64>,
    pub measured_delivered_rate: Option<f64>,
    pub delta_pct_produced: Option<f64>,
    pub delta_pct_delivered: Option<f64>,
    pub is_target: bool,
    pub verdict: Option<Verdict>,
}

impl serde::Serialize for Verdict {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

/// How much measurement the reported rates actually rest on.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct MeasurementQuality {
    /// Length in ticks of the trailing window the rates were taken over.
    pub window_ticks: u64,
    /// Target items counted in that window. Compare against
    /// `window_item_floor`; below it, the rate is quantization-noisy.
    pub window_items: f64,
    /// The sample size the stability tolerance was designed around.
    pub window_item_floor: f64,
    /// The window closed on the tick cap rather than on the item floor —
    /// the factory is running far enough below plan that a full sample
    /// would not fit the budget.
    pub short_sampled: bool,
    /// Spread (widest vs narrowest) across the trailing stability group
    /// of window rates: the exact quantity the convergence test
    /// thresholds. Small on a converged run; a large value means the
    /// factory was still on a transient, so the reported rate is a point
    /// on a slope rather than a steady state.
    pub drift_pct: Option<f64>,
    /// Signed slope across that same group — which way it was heading.
    pub trend_pct: Option<f64>,
    /// Every closed window's produced rate, oldest first — a monotone
    /// series is a ramp or a decay, not noise.
    pub window_rates: Vec<f64>,
    /// Checkpoints taken. Fewer than `STABILITY_WINDOWS + 1` means the
    /// convergence test never ran and `converged: false` says nothing
    /// about the factory (#454).
    pub checkpoints: usize,
}

/// One crafting machine's state at a single checkpoint window close.
/// Identified by `unit` (Factorio's `unit_number`) — stable across
/// samples even where `name`/position alone would collide (#537).
#[derive(Debug, Clone, serde::Serialize)]
pub struct MachineSample {
    pub unit: u64,
    pub name: String,
    /// Tile coordinates. `f64` rather than an integer type: Factorio's
    /// `helpers.table_to_json` is not guaranteed to encode a whole-number
    /// Lua float without a decimal point, and `serde_json::Value::as_i64`
    /// returns `None` (not a rounded value) for a JSON number parsed with
    /// one — silently dropping the entry via `filter_map` rather than
    /// erroring loudly. `f64` reads either encoding without loss for tile-
    /// scale coordinates.
    pub x: f64,
    pub y: f64,
    /// Delta of `products_finished` since the previous checkpoint (not a
    /// running total) — this window's craft count.
    pub crafts_delta: f64,
    /// `defines.entity_status` mapped to its short symbolic name (e.g.
    /// `"working"`, `"no_power"`, `"item_ingredient_shortage"`).
    pub status: String,
}

/// One checkpoint window's machine + item time-series sample. Sampled on
/// the SAME cadence as the checkpoint windows the rates in `items` (the
/// report's `ItemReport`s) are already computed over — a series entry
/// always corresponds to one closed window, not a fixed wall-clock tick.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimeseriesPoint {
    pub tick: u64,
    pub machines: Vec<MachineSample>,
    /// item -> produced-count delta since the previous checkpoint (the
    /// per-window value the force production statistics counter moved
    /// by, not the cumulative total).
    pub items: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Report {
    pub label: String,
    pub items: Vec<ItemReport>,
    pub import_rc: i64,
    pub ghosts: u64,
    pub revived: u64,
    pub pole_networks: u64,
    pub factory_eeis: u64,
    pub proxies_fulfilled: u64,
    pub converged: bool,
    pub final_tick: u64,
    /// Measurement quality of the window the rates were actually taken
    /// over (#454). Rates come from the trailing checkpoint pair, so a
    /// thin or drifting sample used to be invisible: the report printed a
    /// number to two decimals whether it rested on 300 items or 60, and
    /// whether the series was flat or sliding 30% a window.
    pub measurement: MeasurementQuality,
    pub fluid_fed: bool,
    pub uncalibrated_direction: bool,
    pub fluid_errors: BTreeMap<String, String>,
    /// Boundary-kit self-audit failures (overlapping bank chests etc.).
    /// Non-empty means the kit itself is compromised — measured rates are
    /// meaningless (wrong-item cross-feeds poison the factory; #357) and
    /// the overall verdict is forced to NO DATA.
    pub kit_errors: Vec<String>,
    pub machine_census: BTreeMap<String, u64>,
    pub overall_verdict: Verdict,
    /// Manifest context (RFC-050: "config axes (quality/stacking/
    /// inserter-capacity)" and "external_inputs" are report context, not
    /// measurements — surfaced here rather than dropped).
    pub entities: usize,
    pub stacking: u8,
    pub inserter_capacity: u8,
    pub external_inputs: Vec<(String, f64, bool)>,
    /// Realized force capacity bonuses at finalize (tech-state parity,
    /// #370) — surfaced so the parity the rates were measured under is
    /// part of the report, not buried in raw_result. The scenario also
    /// self-audits the assignment into `kit_errors` at init.
    pub inserter_stack_size_bonus: f64,
    pub bulk_inserter_capacity_bonus: f64,
    /// RFC-064 item 7 productivity-parity probe (2026-08-06). The scenario
    /// calls `research_all_technologies()` while the tech-state parity block
    /// corrects only inserter capacity (#370) and belt stacking (#385) —
    /// nothing corrects productivity. The fast meter models no productivity
    /// at all, so any bonus here is a divergence between instrument and
    /// reference rather than a layout property. Three channels because a
    /// research source and a module source imply different fixes; carried as
    /// raw JSON since the probe reads defensively and may report
    /// `FIELD_ABSENT` for an API spelling this harness guessed wrong.
    pub productivity_force: serde_json::Value,
    pub productivity_entity: serde_json::Value,
    pub productivity_modules: serde_json::Value,
    /// Per-machine + per-item rate-vs-time record, one entry per closed
    /// checkpoint window (#537 — see `docs/sim-harness.md`'s "Reading the
    /// time-series" section). Additive field: absent or malformed
    /// `timeseries` in `raw_result` (e.g. a report captured before this
    /// field existed) parses to an empty vec rather than an error, so
    /// older reports and `baseline.rs`'s targeted-field reads are
    /// unaffected.
    pub timeseries: Vec<TimeseriesPoint>,
}

fn get_u64(v: &serde_json::Value, key: &str) -> u64 {
    v.get(key).and_then(|x| x.as_u64()).unwrap_or(0)
}
fn get_i64(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0)
}
fn get_bool(v: &serde_json::Value, key: &str) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(false)
}

/// `(tick, produced_count_for_item)` pairs pulled out of the `samples`
/// array for a single item.
fn sample_series(result: &serde_json::Value, item: &str) -> Vec<(f64, f64)> {
    result
        .get("samples")
        .and_then(|s| s.as_array())
        .into_iter()
        .flatten()
        .filter_map(|s| {
            let tick = s.get("tick")?.as_f64()?;
            let produced = s.get("produced")?.get(item)?.as_f64()?;
            Some((tick, produced))
        })
        .collect()
}

/// Rate over the trailing measurement window: from the sample at or
/// before `window_start` (the second-to-last checkpoint's tick — the
/// same window the target rate is measured over) to the last sample.
///
/// Falls back to the last two samples when no checkpoint window exists.
/// The last two samples span only 20 game-seconds, which is badly
/// aliased for bursty intermediate producers: the #357 recon caught a
/// gear machine (crafting in bursts between plate deliveries) reported
/// at 0.40/s on the 20s snapshot vs 0.80/s over the real window.
fn rate_over_window(series: &[(f64, f64)], window_start: Option<f64>) -> Option<f64> {
    let (t1, v1) = *series.last()?;
    let (t0, v0) = match window_start {
        Some(ws) => series
            .iter()
            .rev()
            .find(|(t, _)| *t <= ws)
            .copied()
            .or_else(|| series.first().copied()),
        None => (series.len() >= 2).then(|| series[series.len() - 2]),
    }?;
    let dt = (t1 - t0) / 60.0;
    if dt <= 0.0 {
        None
    } else {
        Some((v1 - v0) / dt)
    }
}

/// Parse `raw_result.timeseries` into typed points. Tolerant of a missing
/// or malformed key (older `raw_result`s pre-date this field) — returns
/// an empty vec rather than propagating an error, matching how the rest
/// of `compute` treats absent optional fields (`get(...).unwrap_or(...)`
/// throughout).
fn parse_timeseries(result: &serde_json::Value) -> Vec<TimeseriesPoint> {
    result
        .get("timeseries")
        .and_then(|t| t.as_array())
        .into_iter()
        .flatten()
        .filter_map(|point| {
            let tick = point.get("tick")?.as_u64()?;
            let machines = point
                .get("machines")
                .and_then(|m| m.as_array())
                .into_iter()
                .flatten()
                .filter_map(|m| {
                    Some(MachineSample {
                        unit: m.get("unit")?.as_u64()?,
                        name: m.get("name")?.as_str()?.to_string(),
                        x: m.get("x")?.as_f64()?,
                        y: m.get("y")?.as_f64()?,
                        crafts_delta: m.get("crafts_delta")?.as_f64()?,
                        status: m.get("status")?.as_str()?.to_string(),
                    })
                })
                .collect();
            let items = point
                .get("items")
                .and_then(|i| i.as_object())
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| v.as_f64().map(|v| (k.clone(), v)))
                .collect();
            Some(TimeseriesPoint { tick, machines, items })
        })
        .collect()
}

/// RFC-062 Phase 3: `(tick, produced, delivered)` triples for ONE target
/// item, read from each checkpoint's per-item `items[item]` sub-object
/// (`scenario.rs`'s `checkpoint_items()`). Generalizes the old
/// single-scalar `checkpoint.produced`/`.delivered`, which only ever
/// tracked the FIRST target — see the comment this replaces below.
///
/// Tolerant of older `raw_result`s that predate this field: a checkpoint
/// with no `items` key, or no entry for this specific item, is simply
/// skipped rather than erroring, so an old report (or a checkpoint from
/// before the run's first window closed) contributes fewer points rather
/// than none at all. Callers needing the OLD behavior on old data use the
/// flat `checkpoint_series` fallback below (`compute`'s first-target
/// branch), which this function does not replace.
fn checkpoint_item_series(checkpoints: &[serde_json::Value], item: &str) -> Vec<(f64, f64, f64)> {
    checkpoints
        .iter()
        .filter_map(|c| {
            let tick = c.get("tick")?.as_f64()?;
            let entry = c.get("items")?.get(item)?;
            let produced = entry.get("produced")?.as_f64()?;
            let delivered = entry.get("delivered")?.as_f64()?;
            Some((tick, produced, delivered))
        })
        .collect()
}

/// Rate over the trailing two points of a per-item checkpoint series —
/// same delta-over-window math as the primary-target computation this
/// generalizes (`(t1,p1,d1)` vs `(t0,p0,d0)`, dt in seconds). `(None,
/// None)` when fewer than 2 points exist for this item.
fn rate_over_checkpoints(series: &[(f64, f64, f64)]) -> (Option<f64>, Option<f64>) {
    if series.len() < 2 {
        return (None, None);
    }
    let (t0, p0, d0) = series[series.len() - 2];
    let (t1, p1, d1) = series[series.len() - 1];
    let dt = (t1 - t0) / 60.0;
    if dt > 0.0 {
        (Some((p1 - p0) / dt), Some((d1 - d0) / dt))
    } else {
        (None, None)
    }
}

fn delta_pct(measured: Option<f64>, planned: f64) -> Option<f64> {
    if planned <= 0.0 {
        return None;
    }
    measured.map(|m| (m - planned) / planned * 100.0)
}

pub fn compute(manifest: &Manifest, result: &serde_json::Value) -> Report {
    let target_items: Vec<&str> = manifest.targets.iter().map(|t| t.item.as_str()).collect();

    let checkpoints = result
        .get("checkpoints")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();
    let checkpoint_series: Vec<(f64, f64, f64)> = checkpoints
        .iter()
        .filter_map(|c| {
            Some((
                c.get("tick")?.as_f64()?,
                c.get("produced")?.as_f64()?,
                c.get("delivered")?.as_f64()?,
            ))
        })
        .collect();
    let (target_produced_rate, target_delivered_rate) = if checkpoint_series.len() >= 2 {
        let (t0, p0, d0) = checkpoint_series[checkpoint_series.len() - 2];
        let (t1, p1, d1) = checkpoint_series[checkpoint_series.len() - 1];
        let dt = (t1 - t0) / 60.0;
        if dt > 0.0 {
            (Some((p1 - p0) / dt), Some((d1 - d0) / dt))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    // Intermediates measure over the same trailing window as the target.
    let window_start = (checkpoint_series.len() >= 2)
        .then(|| checkpoint_series[checkpoint_series.len() - 2].0);

    // Per-window produced rates, oldest first. Checkpoint 1 opens window
    // 1 (zero length), so the series starts at the second checkpoint.
    let window_rates: Vec<f64> = checkpoint_series
        .windows(2)
        .filter_map(|w| {
            let dt = (w[1].0 - w[0].0) / 60.0;
            (dt > 0.0).then(|| (w[1].1 - w[0].1) / dt)
        })
        .collect();
    // The spread across the trailing stability group — the exact
    // quantity the convergence test thresholds. Measured widest-vs-
    // narrowest over `STABILITY_WINDOWS`, not as the last pairwise step,
    // because a decelerating ramp's last step goes small while the group
    // stays spread (#454).
    let group = crate::scenario::STABILITY_WINDOWS as usize;
    let drift_pct = (window_rates.len() >= group)
        .then(|| {
            let tail = &window_rates[window_rates.len() - group..];
            let lo = tail.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = tail.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            (lo > 0.0).then_some((hi - lo) / lo * 100.0)
        })
        .flatten();
    // Signed slope across the same group, so the printout can say which
    // way an unconverged run was heading.
    let trend_pct = (window_rates.len() >= group)
        .then(|| {
            let tail = &window_rates[window_rates.len() - group..];
            (tail[0] > 0.0).then_some((tail[group - 1] - tail[0]) / tail[0] * 100.0)
        })
        .flatten();
    let last_cp = checkpoints.last();
    let measurement = MeasurementQuality {
        window_ticks: last_cp
            .and_then(|c| c.get("window_ticks"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        window_items: last_cp
            .and_then(|c| c.get("window_items"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        window_item_floor: crate::scenario::WINDOW_ITEM_FLOOR,
        short_sampled: last_cp
            .and_then(|c| c.get("short_sampled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        drift_pct,
        trend_pct,
        window_rates,
        checkpoints: checkpoint_series.len(),
    };

    let mut items = Vec::new();
    for (item, planned_rate) in &manifest.planned_rates {
        let is_target = target_items.contains(&item.as_str());
        let is_first_target = target_items.first() == Some(&item.as_str());
        let is_fluid_target = manifest
            .targets
            .iter()
            .any(|t| t.item == *item && t.is_fluid);

        // RFC-062 Phase 3: every target reads its OWN per-item checkpoint
        // series first (`scenario.rs`'s `checkpoint_items()`, additive
        // next to the old flat `produced`/`delivered` fields) — this is
        // what gives a second/third target an honest DELIVERED-rate
        // verdict instead of the produced-only fallback every non-first
        // target used to be stuck with. `own_produced.is_some()` iff both
        // per-item points existed (see `checkpoint_item_series`), so
        // `own_delivered` is never a false `None` when `own_produced`
        // isn't.
        let (measured_produced_rate, measured_delivered_rate) = if is_target {
            let own_series = checkpoint_item_series(&checkpoints, item);
            let (own_produced, own_delivered) = rate_over_checkpoints(&own_series);
            if own_produced.is_some() {
                (own_produced, if is_fluid_target { None } else { own_delivered })
            } else if is_first_target {
                // Older raw_result (pre-RFC-062: no `items` key on any
                // checkpoint) — fall back to the flat scalar fields,
                // which only ever tracked the first target. Keeps old
                // fixtures/goldens byte-identical.
                (
                    target_produced_rate,
                    if is_fluid_target { None } else { target_delivered_rate },
                )
            } else {
                // Non-first target on an old raw_result with no per-item
                // series at all — the pre-Phase-3 fallback: produced only
                // (from the shared sample series), no delivered
                // attribution. Matches what every non-first target got
                // before this phase.
                (rate_over_window(&sample_series(result, item), window_start), None)
            }
        } else {
            (rate_over_window(&sample_series(result, item), window_start), None)
        };

        let verdict = if is_target {
            if is_fluid_target {
                // Fluid targets have no drain rig (voids are
                // uncounted): verdict on PRODUCED rate, honestly
                // labeled by the missing delivered column.
                Some(verdict_for_ratio(measured_produced_rate.map(|m| m / planned_rate)))
            } else {
                Some(verdict_for_ratio(measured_delivered_rate.map(|m| m / planned_rate)))
            }
        } else {
            None
        };
        items.push(ItemReport {
            item: item.clone(),
            planned_rate: *planned_rate,
            measured_produced_rate,
            measured_delivered_rate,
            delta_pct_produced: delta_pct(measured_produced_rate, *planned_rate),
            delta_pct_delivered: delta_pct(measured_delivered_rate, *planned_rate),
            is_target,
            verdict,
        });
    }
    items.sort_by(|a, b| b.is_target.cmp(&a.is_target).then_with(|| a.item.cmp(&b.item)));

    let kit_errors: Vec<String> = result
        .get("kit_errors")
        .and_then(|k| k.as_array())
        .into_iter()
        .flatten()
        .filter_map(|e| e.as_str().map(str::to_string))
        .collect();

    let overall_verdict = if kit_errors.is_empty() {
        items
            .iter()
            .filter(|i| i.is_target)
            .map(|i| i.verdict.unwrap_or(Verdict::NoData))
            .fold(Verdict::Pass, worst)
    } else {
        Verdict::NoData
    };

    let mut machine_census = BTreeMap::new();
    if let Some(obj) = result.get("machine_census").and_then(|c| c.as_object()) {
        for (k, v) in obj {
            machine_census.insert(k.clone(), v.as_u64().unwrap_or(0));
        }
    }
    let mut fluid_errors = BTreeMap::new();
    if let Some(obj) = result.get("fluid_errors").and_then(|c| c.as_object()) {
        for (k, v) in obj {
            fluid_errors.insert(k.clone(), v.as_str().unwrap_or_default().to_string());
        }
    }

    Report {
        label: manifest.label.clone(),
        items,
        import_rc: get_i64(result, "import_rc"),
        ghosts: get_u64(result, "ghosts"),
        revived: get_u64(result, "revived"),
        pole_networks: get_u64(result, "pole_networks"),
        factory_eeis: get_u64(result, "factory_eeis"),
        proxies_fulfilled: get_u64(result, "proxies_fulfilled"),
        converged: get_bool(result, "converged"),
        final_tick: get_u64(result, "final_tick"),
        measurement,
        fluid_fed: manifest.has_fluid_boundary(),
        uncalibrated_direction: manifest.has_uncalibrated_direction(),
        fluid_errors,
        kit_errors,
        machine_census,
        overall_verdict,
        entities: manifest.entities,
        stacking: manifest.stacking,
        inserter_capacity: manifest.inserter_capacity,
        external_inputs: manifest
            .external_inputs
            .iter()
            .map(|i| (i.item.clone(), i.rate, i.is_fluid))
            .collect(),
        inserter_stack_size_bonus: result
            .get("inserter_stack_size_bonus")
            .and_then(|v| v.as_f64())
            .unwrap_or(-1.0),
        bulk_inserter_capacity_bonus: result
            .get("bulk_inserter_capacity_bonus")
            .and_then(|v| v.as_f64())
            .unwrap_or(-1.0),
        productivity_force: result
            .get("productivity_force")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        productivity_entity: result
            .get("productivity_entity")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        productivity_modules: result
            .get("productivity_modules")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        timeseries: parse_timeseries(result),
    }
}

fn worst(a: Verdict, b: Verdict) -> Verdict {
    fn rank(v: Verdict) -> u8 {
        match v {
            Verdict::Pass => 0,
            Verdict::Warn => 1,
            Verdict::NoData => 2,
            Verdict::Fail => 3,
        }
    }
    if rank(b) > rank(a) {
        b
    } else {
        a
    }
}

/// Print what the reported rates rest on, and name the ways they can be
/// untrustworthy rather than leaving them to be inferred from a bare
/// `converged=false` (#454).
fn print_measurement(m: &MeasurementQuality, converged: bool) {
    println!(
        "measurement: window={}t {:.0} items (floor {:.0}), {} checkpoint(s){}",
        m.window_ticks,
        m.window_items,
        m.window_item_floor,
        m.checkpoints,
        match m.drift_pct {
            Some(d) => format!(", drift={d:+.1}%"),
            None => String::new(),
        }
    );
    let need = crate::scenario::STABILITY_WINDOWS as usize + 1;
    if m.checkpoints < need {
        println!(
            "  WARNING: the convergence test needs {need} checkpoints and got {} — \
             the tick ceiling could not fit it, so converged={converged} is a property \
             of the budget, not the factory.",
            m.checkpoints
        );
    }
    if m.short_sampled {
        println!(
            "  WARNING: window closed on the tick cap with {:.0} of {:.0} items — \
             the factory is running far below plan, so this rate is \
             quantization-noisy.",
            m.window_items, m.window_item_floor
        );
    }
    if !converged && m.checkpoints >= need {
        let trend: String = match m.trend_pct {
            Some(t) if t.abs() >= 2.0 => format!(
                " The series is still {} — {:+.1}% across the last {} windows.",
                if t > 0.0 { "climbing" } else { "decaying" },
                t,
                crate::scenario::STABILITY_WINDOWS
            ),
            _ => String::new(),
        };
        let rates: Vec<String> = m.window_rates.iter().map(|r| format!("{r:.2}")).collect();
        println!(
            "  NOT CONVERGED: rates are a point on a transient, not a steady state.{trend} \
             (window rates: {})",
            rates.join(" -> ")
        );
    }
}

/// Surface the productivity-parity probe (RFC-064 item 7).
///
/// Printed rather than merely stored, because this repo's own rule is that
/// "a verification channel nobody checks is not a verification channel"
/// (`scenario.rs`, the tech-state parity self-audit). A boosted recipe here
/// means the sim and the fast meter — which models no productivity at all —
/// are measuring different worlds on that recipe, so any rate comparison
/// against the meter inherits the gap.
///
/// Only non-zero entries and probe faults are printed: a fully-zero result is
/// the common case and says "no parity gap on the probed recipes", which the
/// one-line summary covers.
fn productivity_parity_lines(
    force: &serde_json::Value,
    entity: &serde_json::Value,
    modules: &serde_json::Value,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut boosted: Vec<String> = Vec::new();
    let mut faults: Vec<String> = Vec::new();
    let mut numeric = 0usize;

    // `productivity_force` maps recipe -> number (or a sentinel string).
    // Recipes the layout actually crafts, from the entity channel. The force
    // channel probes a fixed list, so without this a run that never crafts
    // processing-unit would still print a processing-unit banner (PR #580
    // review). Empty entity channel → report everything rather than nothing,
    // since suppressing on missing evidence is the worse failure here.
    let crafted: Option<Vec<&String>> = entity
        .as_object()
        .map(|m| m.keys().collect())
        .filter(|k: &Vec<&String>| !k.is_empty());
    if let Some(map) = force.as_object() {
        for (name, v) in map {
            if crafted.as_ref().is_some_and(|c| !c.contains(&name)) {
                continue;
            }
            match v.as_f64() {
                Some(n) => {
                    numeric += 1;
                    if n.abs() > f64::EPSILON {
                        boosted.push(format!("{name}={:+.1}% (force)", n * 100.0));
                    }
                }
                // The Lua probe reports a missing/erroring field as a
                // sentinel STRING rather than crashing the run.
                None => faults.push(format!("force:{name}={v}")),
            }
        }
    }
    // `productivity_entity` maps recipe -> {min, max, n, faults}, aggregated
    // over every machine of that recipe rather than the first one seen.
    if let Some(map) = entity.as_object() {
        for (name, v) in map {
            let lo = v.get("min").and_then(|x| x.as_f64());
            let hi = v.get("max").and_then(|x| x.as_f64());
            let n = v.get("n").and_then(|x| x.as_u64()).unwrap_or(0);
            let f = v.get("faults").and_then(|x| x.as_u64()).unwrap_or(0);
            if n > 0 {
                numeric += 1;
                let (lo, hi) = (lo.unwrap_or(0.0), hi.unwrap_or(0.0));
                if lo.abs() > f64::EPSILON || hi.abs() > f64::EPSILON {
                    boosted.push(if (hi - lo).abs() > f64::EPSILON {
                        format!("{name}={:+.1}..{:+.1}% (entity, n={n})", lo * 100.0, hi * 100.0)
                    } else {
                        format!("{name}={:+.1}% (entity, n={n})", lo * 100.0)
                    });
                }
            }
            if f > 0 {
                faults.push(format!("entity:{name}={f} machine(s) faulted"));
            }
        }
    }
    let modules = modules.as_object().map(|m| m.len()).unwrap_or(0);

    if numeric == 0 && faults.is_empty() {
        return out; // probe absent entirely (e.g. a report from before it existed)
    }
    if numeric == 0 {
        // Self-audit: every channel faulted. Without this line an all-sentinel
        // run is indistinguishable from a genuine "no productivity anywhere",
        // which would silently license a wrong conclusion.
        out.push(format!(
            "productivity parity: PROBE FAILED — no channel returned a number \
             ({} fault(s): {}). Treat any productivity claim from this run as \
             unmeasured.",
            faults.len(),
            faults.join(", ")
        ));
        return out;
    }
    // BOOSTED is gated on a non-empty boost list ALONE. Folding the module
    // count into this condition made a layout with any module at all print
    // "BOOSTED []" — an empty boost list under a boosted header, claiming a
    // parity gap that the numbers do not show (PR #580 review, 3/3).
    if boosted.is_empty() {
        out.push(format!("productivity parity: none on the probed recipes ({numeric} read, 0 boosted)"));
    } else {
        out.push(format!(
            "productivity parity: BOOSTED [{}] — the fast meter models no \
             productivity, so meter-vs-sim rates on these recipes are not \
             like-for-like (RFC-064 item 7)",
            boosted.join(", ")
        ));
    }
    if modules > 0 {
        // Informational and separate: these are productivity-family modules
        // only (the Lua side filters), reported as distinct (recipe, module)
        // pairs rather than slot counts.
        out.push(format!("  productivity modules present: {modules} (recipe, module) pair(s)"));
    }
    if !faults.is_empty() {
        out.push(format!("  probe faults: {}", faults.join(", ")));
    }
    out
}

pub fn print_human(report: &Report) {
    println!("=== spaghettio-sim report: {} ===", report.label);
    println!(
        "import: rc={} ghosts={} revived={} (failed={})",
        report.import_rc,
        report.ghosts,
        report.revived,
        report.ghosts.saturating_sub(report.revived)
    );
    println!(
        "power: {} pole network(s), {} factory EEI(s)",
        report.pole_networks, report.factory_eeis
    );
    println!("module proxies fulfilled: {}", report.proxies_fulfilled);
    println!(
        "run: final_tick={} converged={}",
        report.final_tick, report.converged
    );
    print_measurement(&report.measurement, report.converged);
    println!(
        "layout: {} entities, stacking={} inserter_capacity={} (realized bonuses: nb={} bulk={})",
        report.entities,
        report.stacking,
        report.inserter_capacity,
        report.inserter_stack_size_bonus,
        report.bulk_inserter_capacity_bonus
    );
    for line in productivity_parity_lines(
        &report.productivity_force,
        &report.productivity_entity,
        &report.productivity_modules,
    ) {
        println!("{line}");
    }
    if !report.external_inputs.is_empty() {
        let inputs: Vec<String> = report
            .external_inputs
            .iter()
            .map(|(item, rate, is_fluid)| format!("{item}@{rate:.1}/s{}", if *is_fluid { " (fluid)" } else { "" }))
            .collect();
        println!("external inputs: {}", inputs.join(", "));
    }
    if report.uncalibrated_direction {
        println!("NOTE: at least one boundary is not south-facing — the jog geometry is a faithful vector generalization of the calibrated south-only prototype, but has never been measured live.");
    }
    if !report.fluid_errors.is_empty() {
        println!("fluid rig errors:");
        for (k, v) in &report.fluid_errors {
            println!("  {k}: {v}");
        }
    }
    if !report.kit_errors.is_empty() {
        println!("KIT ERRORS — boundary kit compromised, RUN INVALID (verdict forced NO DATA):");
        for e in &report.kit_errors {
            println!("  {e}");
        }
    }
    println!();
    println!(
        "{:<28} {:>10} {:>12} {:>10} {:>12} {:>10} {:>8}",
        "item", "planned/s", "produced/s", "d%", "delivered/s", "d%", "verdict"
    );
    for item in &report.items {
        println!(
            "{:<28} {:>10.2} {:>12} {:>10} {:>12} {:>10} {:>8}",
            item.item,
            item.planned_rate,
            fmt_opt(item.measured_produced_rate),
            fmt_pct(item.delta_pct_produced),
            fmt_opt(item.measured_delivered_rate),
            fmt_pct(item.delta_pct_delivered),
            item.verdict.map(|v| v.to_string()).unwrap_or_default(),
        );
    }
    println!();
    if !report.machine_census.is_empty() {
        println!("machine census:");
        for (status, count) in &report.machine_census {
            println!("  {status}: {count}");
        }
    }
    println!();
    if !report.timeseries.is_empty() {
        println!(
            "timeseries: {} checkpoint window(s) recorded (per-machine crafts_delta/status, \
             per-item produced_delta) — see the `timeseries` key in --out, or docs/sim-harness.md \
             \"Reading the time-series\".",
            report.timeseries.len()
        );
    }
    println!("OVERALL: {}", report.overall_verdict);
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.2}")).unwrap_or_else(|| "-".to_string())
}
fn fmt_pct(v: Option<f64>) -> String {
    v.map(|x| format!("{x:+.1}%")).unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {


    fn productivity_parity_lines_of(
        force: serde_json::Value,
        entity: serde_json::Value,
        modules: serde_json::Value,
    ) -> Vec<String> {
        productivity_parity_lines(&force, &entity, &modules)
    }

    /// All three branches of the productivity-parity summary, pinned.
    ///
    /// The review that prompted these noted every branch was unasserted — and
    /// two of them were wrong at the time: `BOOSTED` fired on any module at
    /// all (including speed/efficiency), printing an empty boost list under a
    /// boosted header.
    #[test]
    fn productivity_parity_reports_boosted_only_when_something_is_boosted() {
        let lines = productivity_parity_lines_of(
            serde_json::json!({"processing-unit": 0.1, "electronic-circuit": 0.0}),
            serde_json::json!({"processing-unit": {"min": 0.0, "max": 0.0, "n": 4, "faults": 0}}),
            serde_json::json!({}),
        );
        assert!(lines[0].contains("BOOSTED"), "got {lines:?}");
        assert!(lines[0].contains("processing-unit=+10.0%"), "got {lines:?}");
        // electronic-circuit is probed but not crafted in this layout, and is
        // 0.0 anyway — it must not appear.
        assert!(!lines[0].contains("electronic-circuit"), "got {lines:?}");
    }

    #[test]
    fn productivity_parity_reports_none_when_nothing_is_boosted() {
        let lines = productivity_parity_lines_of(
            serde_json::json!({"iron-plate": 0.0}),
            serde_json::json!({"iron-plate": {"min": 0.0, "max": 0.0, "n": 2, "faults": 0}}),
            serde_json::json!({}),
        );
        assert_eq!(lines.len(), 1, "got {lines:?}");
        assert!(lines[0].contains("none on the probed recipes"), "got {lines:?}");
        assert!(!lines[0].contains("BOOSTED"), "got {lines:?}");
    }

    /// A module present but zero productivity is NOT a parity gap. This is the
    /// case that used to print "BOOSTED []".
    #[test]
    fn productivity_parity_does_not_claim_boost_from_modules_alone() {
        let lines = productivity_parity_lines_of(
            serde_json::json!({"iron-plate": 0.0}),
            serde_json::json!({"iron-plate": {"min": 0.0, "max": 0.0, "n": 1, "faults": 0}}),
            serde_json::json!({"iron-plate/productivity-module": 2}),
        );
        assert!(lines[0].contains("none on the probed recipes"), "got {lines:?}");
        assert!(
            lines.iter().any(|l| l.contains("productivity modules present")),
            "module presence must still be reported, separately: {lines:?}"
        );
    }

    /// Self-audit: an all-sentinel run must say so, not read as "no productivity".
    #[test]
    fn productivity_parity_flags_a_wholly_failed_probe() {
        let lines = productivity_parity_lines_of(
            serde_json::json!({"processing-unit": "FIELD_ABSENT"}),
            serde_json::json!({}),
            serde_json::json!({}),
        );
        assert!(lines[0].contains("PROBE FAILED"), "got {lines:?}");
    }

    /// A report predating the probe prints nothing rather than a false "none".
    #[test]
    fn productivity_parity_is_silent_when_the_probe_is_absent() {
        let lines = productivity_parity_lines_of(
            serde_json::Value::Null, serde_json::Value::Null, serde_json::Value::Null,
        );
        assert!(lines.is_empty(), "got {lines:?}");
    }

    use super::*;
    use crate::manifest::Manifest;

    fn fixture_manifest() -> Manifest {
        Manifest::from_str(include_str!("../tests/fixtures/manifest_gear10.json")).unwrap()
    }

    #[test]
    fn pass_at_or_above_98_percent() {
        assert_eq!(verdict_for_ratio(Some(0.98)), Verdict::Pass);
        assert_eq!(verdict_for_ratio(Some(1.5)), Verdict::Pass, "overshoot is informational, still PASS");
        assert_eq!(verdict_for_ratio(Some(0.979999)), Verdict::Warn);
    }

    #[test]
    fn warn_band_between_90_and_98_percent() {
        assert_eq!(verdict_for_ratio(Some(0.95)), Verdict::Warn);
        assert_eq!(verdict_for_ratio(Some(0.90)), Verdict::Warn);
    }

    #[test]
    fn fail_below_90_percent() {
        assert_eq!(verdict_for_ratio(Some(0.89999)), Verdict::Fail);
        assert_eq!(verdict_for_ratio(Some(0.0)), Verdict::Fail);
    }

    #[test]
    fn no_data_when_ratio_missing() {
        assert_eq!(verdict_for_ratio(None), Verdict::NoData);
    }

    #[test]
    fn compute_reads_checkpoints_for_target_rate() {
        let m = fixture_manifest();
        let result = serde_json::json!({
            "import_rc": 0, "ghosts": 428, "revived": 428,
            "pole_networks": 1, "factory_eeis": 1, "proxies_fulfilled": 0,
            "converged": true, "final_tick": 12600,
            "checkpoints": [
                {"tick": 9000, "produced": 1000.0, "delivered": 980.0},
                {"tick": 10800, "produced": 1300.0, "delivered": 1276.0}
            ],
            "samples": [],
            "machine_census": {"full_output": 5}
        });
        let report = compute(&m, &result);
        let target = report.items.iter().find(|i| i.item == "iron-gear-wheel").unwrap();
        // (1300-1000)/((10800-9000)/60) = 300/30 = 10.0/s
        assert!((target.measured_produced_rate.unwrap() - 10.0).abs() < 1e-9);
        // (1276-980)/30 = 9.866...
        assert!((target.measured_delivered_rate.unwrap() - 9.8666666667).abs() < 1e-6);
        assert_eq!(target.verdict, Some(Verdict::Pass));
        assert_eq!(report.overall_verdict, Verdict::Pass);
    }

    /// RFC-062 Phase 3: a SECOND (non-first) target must get a real
    /// delivered-rate verdict from its own `items[item]` checkpoint
    /// series, not the produced-only fallback every non-first target was
    /// stuck with before this phase (the gap `docs/rfc-062-multi-target-
    /// outputs.md`'s Phase 2 decision log named — "non-first SOLID
    /// targets have no per-item drain attribution"). Numbers mirror the
    /// canonical EC@10/s + AC@3/s fixture's shape: EC is first
    /// (10.0 produced / 9.8667 delivered, matching
    /// `compute_reads_checkpoints_for_target_rate` above exactly), AC is
    /// second (3.0 produced / 2.96 delivered — a deliberate, small
    /// under-delivery so the verdict math is exercised, not just the
    /// "measured at all" question).
    #[test]
    fn second_target_gets_its_own_delivered_rate_verdict() {
        use crate::manifest::ItemRate;
        let mut m = fixture_manifest();
        m.targets = vec![
            ItemRate { item: "electronic-circuit".to_string(), rate: 10.0, is_fluid: false },
            ItemRate { item: "advanced-circuit".to_string(), rate: 3.0, is_fluid: false },
        ];
        m.planned_rates = BTreeMap::from([
            ("electronic-circuit".to_string(), 10.0),
            ("advanced-circuit".to_string(), 3.0),
        ]);
        let result = serde_json::json!({
            "converged": true, "final_tick": 12600,
            "checkpoints": [
                {"tick": 9000, "produced": 1000.0, "delivered": 980.0,
                 "items": {
                    "electronic-circuit": {"produced": 1000.0, "delivered": 980.0},
                    "advanced-circuit": {"produced": 300.0, "delivered": 294.0}
                 }},
                {"tick": 10800, "produced": 1300.0, "delivered": 1276.0,
                 "items": {
                    "electronic-circuit": {"produced": 1300.0, "delivered": 1276.0},
                    "advanced-circuit": {"produced": 390.0, "delivered": 382.8}
                 }}
            ],
            "samples": []
        });
        let report = compute(&m, &result);

        let ec = report.items.iter().find(|i| i.item == "electronic-circuit").unwrap();
        assert!((ec.measured_produced_rate.unwrap() - 10.0).abs() < 1e-9);
        assert!((ec.measured_delivered_rate.unwrap() - 9.866_666_666_7).abs() < 1e-6);
        assert_eq!(ec.verdict, Some(Verdict::Pass));

        let ac = report.items.iter().find(|i| i.item == "advanced-circuit").unwrap();
        assert!(ac.is_target);
        assert!((ac.measured_produced_rate.unwrap() - 3.0).abs() < 1e-9);
        // (382.8-294)/30 = 2.96 -- was `None` before this phase for any
        // non-first target; this is the fix.
        assert!(
            ac.measured_delivered_rate.is_some(),
            "second target must get a delivered-rate measurement, not None"
        );
        assert!((ac.measured_delivered_rate.unwrap() - 2.96).abs() < 1e-6);
        // 2.96/3.0 = 0.98667 -> at/above the 0.98 PASS boundary, and the
        // verdict must be computed on the DELIVERED rate, not produced
        // (produced alone would also pass here by coincidence, so the
        // real proof is `measured_delivered_rate.is_some()` above).
        assert_eq!(ac.verdict, Some(Verdict::Pass));

        assert_eq!(report.overall_verdict, Verdict::Pass);
    }

    /// #454: the reported rate is the trailing window, so a run that
    /// never settled reports a point on a slope. The drift the
    /// convergence test thresholds must reach the report, not be thrown
    /// away with the verdict.
    #[test]
    fn measurement_quality_exposes_drift_and_sample_size() {
        let m = fixture_manifest();
        // usp2-sup120's real shape: a monotone ramp, 0.70 -> 0.74 -> 0.88
        // over three 9000-tick windows, reported as "0.88/s" with nothing
        // to say it was still climbing 19% a window.
        let result = serde_json::json!({
            "converged": false, "final_tick": 197_700,
            "checkpoints": [
                {"tick": 162_000, "produced": 1011.0, "delivered": 912.0,
                 "window_ticks": 0, "window_items": 0.0, "short_sampled": false},
                {"tick": 171_000, "produced": 1116.0, "delivered": 1024.0,
                 "window_ticks": 9000, "window_items": 105.0, "short_sampled": true},
                {"tick": 180_000, "produced": 1227.0, "delivered": 1144.0,
                 "window_ticks": 9000, "window_items": 111.0, "short_sampled": true},
                {"tick": 189_000, "produced": 1359.0, "delivered": 1248.0,
                 "window_ticks": 9000, "window_items": 132.0, "short_sampled": true}
            ],
            "samples": []
        });
        let q = compute(&m, &result).measurement;
        assert_eq!(q.checkpoints, 4);
        assert_eq!(q.window_rates.len(), 3);
        assert!((q.window_rates[0] - 0.70).abs() < 1e-9);
        assert!((q.window_rates[2] - 0.88).abs() < 1e-9);
        // Group spread (0.88-0.70)/0.70 = +25.7%, far outside the 2% the
        // convergence test wants — the number is not a steady state.
        // Note the last PAIRWISE step is only +18.9%, and on a flatter
        // ramp it would fall under tolerance while the group stays
        // spread: that is exactly the ramp chem5 was certified on.
        assert!((q.drift_pct.unwrap() - 25.714_285_7).abs() < 1e-6);
        assert!((q.trend_pct.unwrap() - 25.714_285_7).abs() < 1e-6);
        // 132 items against a 300 floor: quantization-noisy.
        assert!(q.short_sampled);
        assert_eq!(q.window_items, 132.0);
        assert_eq!(q.window_item_floor, crate::scenario::WINDOW_ITEM_FLOOR);
    }

    /// The ramp chem5 was actually certified on (#454), in real numbers:
    /// 4.62 -> 4.92 -> 5.00/s produced. The final pairwise step is
    /// +1.63%, under the 2% tolerance, so the old last-two-windows test
    /// called it converged and reported the trailing window as
    /// "5.00/s EXACT at plan" — while the measured span averaged 4.84/s.
    /// Compared as a group the spread is +8.3%, correctly rejected.
    #[test]
    fn a_decelerating_ramp_is_not_mistaken_for_a_steady_state() {
        let m = fixture_manifest();
        let result = serde_json::json!({
            "converged": true, "final_tick": 71_760,
            "checkpoints": [
                {"tick": 60_600, "produced": 4036.0, "delivered": 3792.0,
                 "window_ticks": 0, "window_items": 0.0, "short_sampled": false},
                {"tick": 64_500, "produced": 4336.0, "delivered": 4112.0,
                 "window_ticks": 3900, "window_items": 300.0, "short_sampled": false},
                {"tick": 68_160, "produced": 4636.0, "delivered": 4416.0,
                 "window_ticks": 3660, "window_items": 300.0, "short_sampled": false},
                {"tick": 71_760, "produced": 4936.0, "delivered": 4704.0,
                 "window_ticks": 3600, "window_items": 300.0, "short_sampled": false}
            ],
            "samples": []
        });
        let q = compute(&m, &result).measurement;
        let r = &q.window_rates;
        assert_eq!(r.len(), 3);
        // The last step alone looks settled...
        let last_step = (r[2] - r[1]) / r[1] * 100.0;
        assert!(last_step < 2.0, "last step was {last_step:.2}%");
        // ...but the group has not stopped moving.
        assert!(
            q.drift_pct.unwrap() > 2.0,
            "group spread was {:.2}%",
            q.drift_pct.unwrap()
        );
        // 300/65 -> 300/61 -> 300/60 items-per-second: spread is exactly
        // (5 - 60/13)/(60/13) = 5/60 = 8.333%.
        assert!((q.drift_pct.unwrap() - 100.0 * 5.0 / 60.0).abs() < 1e-6);
        assert!(q.trend_pct.unwrap() > 0.0, "the ramp was climbing");
    }

    /// A run whose ceiling could not fit the convergence test must be
    /// distinguishable from a factory that genuinely refused to settle:
    /// `mega-chain-usp2raw --warmup 480000` produced one checkpoint and a
    /// bare `converged: false` (#454).
    #[test]
    fn single_checkpoint_run_is_visible_as_a_budget_failure() {
        let m = fixture_manifest();
        let result = serde_json::json!({
            "converged": false, "final_tick": 489_000,
            "checkpoints": [
                {"tick": 486_000, "produced": 5574.0, "delivered": 5472.0,
                 "window_ticks": 0, "window_items": 0.0, "short_sampled": false}
            ],
            "samples": []
        });
        let report = compute(&m, &result);
        assert_eq!(report.measurement.checkpoints, 1);
        assert!(report.measurement.window_rates.is_empty());
        assert_eq!(report.measurement.drift_pct, None);
        // One checkpoint cannot yield a rate at all -> NO DATA, which is
        // correct, but the checkpoint count is what explains it.
        assert_eq!(report.overall_verdict, Verdict::NoData);
    }

    #[test]
    fn intermediate_rates_use_the_checkpoint_window_not_the_last_sample_pair() {
        // Bursty producer: 30 items in the first 20s of the window, none
        // in the last 20s. Last-two-samples reads 0.0/s; the honest
        // window rate is 30 items / 60s = 0.5/s.
        let series = vec![(9000.0, 100.0), (10200.0, 130.0), (11400.0, 130.0), (12600.0, 130.0)];
        assert_eq!(rate_over_window(&series, Some(9000.0)), Some(30.0 / 60.0));
        // Fallback without a window: the old 20s snapshot behavior.
        assert_eq!(rate_over_window(&series, None), Some(0.0));
        // Window start before the first sample clamps to the first sample.
        assert_eq!(rate_over_window(&series, Some(0.0)), Some(30.0 / 60.0));
        assert_eq!(rate_over_window(&[], Some(9000.0)), None);
    }

    #[test]
    fn compute_flags_fail_when_delivered_short() {
        let m = fixture_manifest();
        let result = serde_json::json!({
            "import_rc": 0, "ghosts": 428, "revived": 428,
            "pole_networks": 1, "factory_eeis": 1, "proxies_fulfilled": 0,
            "converged": true, "final_tick": 12600,
            "checkpoints": [
                {"tick": 9000, "produced": 1000.0, "delivered": 700.0},
                {"tick": 10800, "produced": 1300.0, "delivered": 850.0}
            ],
            "samples": [],
            "machine_census": {}
        });
        let report = compute(&m, &result);
        let target = report.items.iter().find(|i| i.item == "iron-gear-wheel").unwrap();
        // (850-700)/30 = 5.0/s vs planned 10.0/s -> 50%, FAIL
        assert_eq!(target.verdict, Some(Verdict::Fail));
        assert_eq!(report.overall_verdict, Verdict::Fail);
    }

    #[test]
    fn no_fluid_or_uncalibrated_flags_on_gear_fixture() {
        let m = fixture_manifest();
        let result = serde_json::json!({"checkpoints": [], "samples": []});
        let report = compute(&m, &result);
        assert!(!report.fluid_fed);
        assert!(!report.uncalibrated_direction);
    }

    /// #537: a rate-vs-time series distinguishes "feed never arrived"
    /// from "buffer-fill mirage then jam" at a glance. Pins the schema —
    /// per-checkpoint tick + per-machine {unit, name, x, y, crafts_delta,
    /// status} + per-item produced_delta — round-trips out of a raw
    /// `timeseries` array the way the generated Lua emits it.
    #[test]
    fn timeseries_parses_per_checkpoint_machine_and_item_deltas() {
        let m = fixture_manifest();
        let result = serde_json::json!({
            "checkpoints": [], "samples": [],
            "timeseries": [
                {
                    "tick": 9000,
                    "machines": [
                        {"unit": 42, "name": "assembling-machine-2", "x": 3, "y": -1,
                         "crafts_delta": 12.0, "status": "working"},
                        {"unit": 43, "name": "electric-furnace", "x": 5, "y": 2,
                         "crafts_delta": 0.0, "status": "item_ingredient_shortage"}
                    ],
                    "items": {"iron-gear-wheel": 24.0, "iron-plate": 48.0}
                },
                {
                    "tick": 10800,
                    "machines": [
                        {"unit": 42, "name": "assembling-machine-2", "x": 3, "y": -1,
                         "crafts_delta": 15.0, "status": "working"}
                    ],
                    "items": {"iron-gear-wheel": 30.0}
                }
            ]
        });
        let report = compute(&m, &result);
        assert_eq!(report.timeseries.len(), 2);
        let first = &report.timeseries[0];
        assert_eq!(first.tick, 9000);
        assert_eq!(first.machines.len(), 2);
        let am = first.machines.iter().find(|ms| ms.unit == 42).unwrap();
        assert_eq!(am.name, "assembling-machine-2");
        assert_eq!(am.x, 3.0);
        assert_eq!(am.y, -1.0);
        assert_eq!(am.crafts_delta, 12.0);
        assert_eq!(am.status, "working");
        let starved = first.machines.iter().find(|ms| ms.unit == 43).unwrap();
        assert_eq!(starved.status, "item_ingredient_shortage");
        assert_eq!(first.items.get("iron-gear-wheel"), Some(&24.0));
        assert_eq!(first.items.get("iron-plate"), Some(&48.0));
        assert_eq!(report.timeseries[1].tick, 10800);
        assert_eq!(report.timeseries[1].machines[0].crafts_delta, 15.0);
    }

    /// A `raw_result` from before this field existed (or `bless`/`check`'s
    /// hand-built test fixtures) must parse cleanly to an empty series
    /// rather than erroring — additive-only per the PR's compatibility
    /// requirement.
    #[test]
    fn timeseries_absent_defaults_to_empty_vec() {
        let m = fixture_manifest();
        let result = serde_json::json!({"checkpoints": [], "samples": []});
        let report = compute(&m, &result);
        assert!(report.timeseries.is_empty());
    }
}
