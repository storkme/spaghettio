import type { LayoutResult, TraceEvent } from "../engine";

type EventOf<T extends string> = Extract<TraceEvent, { phase: T }>;

function filter<T extends string>(events: TraceEvent[], phase: T): EventOf<T>[] {
  return events.filter(e => e.phase === phase) as EventOf<T>[];
}

const HDR = "color:#9cdcfe;font-weight:bold";
const LBL = "color:#888";
const VAL = "color:#e0e0e0";
const GOOD = "color:#6a6";
const WARN = "color:#ffaa00";
const BAD = "color:#f66";
const ACCENT = "color:#c586c0";

/** Pretty-print layout timing + interesting stats to the browser console. */
export function logLayoutStats(layout: LayoutResult): void {
  const events = Array.isArray(layout.trace)
    ? (layout.trace as TraceEvent[])
    : [];
  if (events.length === 0) return;

  const phases = filter(events, "PhaseTime");
  const sat = filter(events, "SatInvocation");
  const solved = filter(events, "JunctionSolved");
  const capped = filter(events, "JunctionGrowthCapped");
  const clusters = filter(events, "GhostClusterSolved");
  const ghost = filter(events, "GhostRoutingComplete");
  const vetoes = filter(events, "RegionWalkerVeto");
  const growthIters = filter(events, "JunctionGrowthIteration");
  const negotiate = filter(events, "NegotiateComplete");
  const validation = filter(events, "ValidationCompleted");

  const totalMs = phases.reduce((s, t) => s + t.data.duration_ms, 0);
  const satUs = sat.reduce((s, t) => s + t.data.solve_time_us, 0);
  const satSatisfied = sat.filter(s => s.data.satisfied).length;
  const entityCount = layout.entities?.length ?? 0;

  // One-liner summary (always visible).
  const summaryStyle = capped.length > 0 ? WARN : GOOD;
  const note = capped.length > 0 ? ` · ${capped.length} capped` : "";
  console.log(
    `%c▶ layout %c${layout.width}×${layout.height}  %c${entityCount} entities  %c${totalMs}ms  %cSAT ${Math.round(satUs / 1000)}ms (${sat.length}×)%c${note}`,
    HDR, VAL, LBL, VAL, ACCENT, summaryStyle,
  );

  // Details collapsed by default.
  console.groupCollapsed("%c  ↳ breakdown", LBL);

  // Phase timeline (sorted by duration desc).
  if (phases.length > 0) {
    const sorted = [...phases].sort((a, b) => b.data.duration_ms - a.data.duration_ms);
    console.log(`%cphases%c ${totalMs}ms total`, HDR, LBL);
    for (const t of sorted) {
      const d = t.data;
      const pct = totalMs > 0 ? (d.duration_ms / totalMs) * 100 : 0;
      const barLen = Math.max(1, Math.round((pct / 100) * 24));
      const bar = "█".repeat(barLen);
      console.log(
        `  %c${d.phase.padEnd(18)}%c ${String(d.duration_ms).padStart(5)}ms  %c${bar}%c ${pct.toFixed(1)}%`,
        LBL, VAL, ACCENT, LBL,
      );
    }
  }

  // SAT stats — the user suspected this is ~99% of the work.
  if (sat.length > 0) {
    const satMs = satUs / 1000;
    const satPct = totalMs > 0 ? (satMs / totalMs) * 100 : 0;
    const avgUs = satUs / sat.length;
    const slowest = [...sat].sort((a, b) => b.data.solve_time_us - a.data.solve_time_us)[0];
    const biggest = [...sat].sort((a, b) =>
      (b.data.zone_w * b.data.zone_h) - (a.data.zone_w * a.data.zone_h)
    )[0];
    console.log(`%cSAT%c ${sat.length} invocations · ${satMs.toFixed(1)}ms (%c${satPct.toFixed(1)}%%%c of total)`, HDR, VAL, ACCENT, VAL);
    console.log(`  %csatisfied%c ${satSatisfied}  %cunsat%c ${sat.length - satSatisfied}  %cavg%c ${(avgUs / 1000).toFixed(2)}ms`,
      LBL, GOOD, LBL, BAD, LBL, VAL);
    if (slowest) {
      console.log(`  %cslowest call%c ${(slowest.data.solve_time_us / 1000).toFixed(1)}ms — %c${slowest.data.zone_w}×${slowest.data.zone_h} @ (${slowest.data.zone_x},${slowest.data.zone_y}), ${slowest.data.variables} vars, ${slowest.data.clauses} clauses`,
        LBL, VAL, LBL);
    }
    if (biggest && biggest !== slowest) {
      console.log(`  %cbiggest zone%c ${biggest.data.zone_w}×${biggest.data.zone_h} @ (${biggest.data.zone_x},${biggest.data.zone_y}) — ${biggest.data.variables} vars`,
        LBL, VAL);
    }
  }

  // Junction solve outcomes.
  if (clusters.length > 0 || solved.length > 0 || capped.length > 0) {
    console.log(`%cjunctions`, HDR);
    console.log(`  %cclusters%c ${clusters.length}  %csolved%c ${solved.length}  %ccapped%c ${capped.length}  %cvetoes%c ${vetoes.length}`,
      LBL, VAL, LBL, GOOD, LBL, capped.length > 0 ? WARN : VAL, LBL, VAL);
    if (growthIters.length > 0) {
      // "Hardest junction" = the one that needed the most growth iters.
      const byJunction = new Map<string, number>();
      for (const g of growthIters) {
        const key = `${g.data.seed_x},${g.data.seed_y}`;
        byJunction.set(key, Math.max(byJunction.get(key) ?? 0, g.data.iter));
      }
      const hardest = [...byJunction.entries()].sort((a, b) => b[1] - a[1])[0];
      if (hardest && hardest[1] > 0) {
        console.log(`  %chardest%c junction at (${hardest[0]}) needed ${hardest[1] + 1} growth iters`,
          LBL, VAL);
      }
    }
    if (capped.length > 0) {
      for (const c of capped) {
        console.log(`    %c⚠ capped at (${c.data.tile_x},${c.data.tile_y})%c — ${c.data.reason}, ${c.data.region_tiles} tiles after ${c.data.iters} iters`,
          WARN, LBL);
      }
    }
  }

  // Ghost routing summary.
  if (ghost.length > 0) {
    const g = ghost[0].data;
    const unroutStyle = g.unroutable_count > 0 ? BAD : GOOD;
    console.log(`%cghost router%c ${g.entity_count} routed entities, ${g.cluster_count} clusters, max cluster ${g.max_cluster_tiles} tiles  %c${g.unroutable_count} unroutable`,
      HDR, VAL, unroutStyle);
  }

  // Negotiate.
  if (negotiate.length > 0) {
    const n = negotiate[0].data;
    console.log(`%cA* negotiate%c ${n.specs} specs, ${n.iterations} iters, ${n.duration_ms}ms`, HDR, VAL);
  }

  // Validation.
  if (validation.length > 0) {
    const v = validation[0].data;
    const errColor = v.error_count > 0 ? BAD : GOOD;
    const warnColor = v.warning_count > 0 ? WARN : GOOD;
    console.log(`%cvalidation  %c${v.error_count} errors  %c${v.warning_count} warnings`, HDR, errColor, warnColor);
  }

  console.groupEnd();
}
