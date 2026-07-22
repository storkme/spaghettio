import { parseSimReport, type SimReport } from "./simReportLoader";
import "./simReportPanel.css";

/** Minimal "how big is the currently-rendered layout" fact the mismatch
 *  guard needs. The report carries no width/height (RFC-050's manifest is
 *  entity-boundary-shaped, not dimension-shaped) — `entities` is the only
 *  size signal both sides have, so it's the one thing compared. */
export interface SimLayoutInfo {
  entities: number;
}

export interface SimReportPanelControls {
  /** Parse + display a sim report from raw JSON text. Never throws — a
   *  malformed file renders an error message in the panel instead. */
  loadFromText(jsonText: string): void;
  clear(): void;
  getReport(): SimReport | null;
  /** Called whenever the rendered layout changes, so the mismatch
   *  warning (deliverable D) tracks whichever layout is on screen. */
  setLayoutInfo(info: SimLayoutInfo | null): void;
}

function verdictColor(v: string | null | undefined): string {
  if (v === "PASS") return "#4caf50";
  if (v === "WARN") return "#ff9a3c";
  if (v === "FAIL") return "#f66";
  return "#aaa";
}

function fmtRate(v: number | null): string {
  return v == null ? "?" : `${v}/s`;
}

/**
 * Verdict banner + legend panel (RFC-050 Phase 4, deliverables C + D).
 * `onChange` fires after every load/clear (including failed loads, with
 * `null`) so the caller can gate the `sim-state` overlay toggle and
 * re-render `simStateOverlay`.
 */
export function createSimReportPanel(
  container: HTMLElement,
  onChange: (report: SimReport | null) => void,
): SimReportPanelControls {
  const panel = document.createElement("div");
  panel.className = "sim-report-panel";
  container.appendChild(panel);

  let report: SimReport | null = null;
  let layoutInfo: SimLayoutInfo | null = null;

  function renderLegend(): HTMLDivElement {
    const legend = document.createElement("div");
    legend.className = "sim-report-legend";
    const title = document.createElement("div");
    title.className = "sim-legend-title";
    title.textContent = "Legend";
    legend.appendChild(title);

    const rows: [string, string][] = [
      ["#4caf50", "machine working"],
      ["#d9432e", "machine shortage"],
      ["#ff9a3c", "machine full output"],
      ["#9b59b6", "machine no power"],
      ["#8a8a8a", "machine / inserter other"],
    ];
    for (const [color, label] of rows) {
      const row = document.createElement("div");
      const swatch = document.createElement("span");
      swatch.className = "sim-swatch";
      swatch.style.background = color;
      row.appendChild(swatch);
      row.appendChild(document.createTextNode(label));
      legend.appendChild(row);
    }

    const beltRow = document.createElement("div");
    const beltSwatch = document.createElement("span");
    beltSwatch.className = "sim-swatch sim-swatch-grad";
    beltRow.appendChild(beltSwatch);
    beltRow.appendChild(document.createTextNode("belt fill (low → high)"));
    legend.appendChild(beltRow);

    const dotRows: [string, string][] = [
      ["#d9432e", "inserter waiting for source"],
      ["#ff9a3c", "inserter waiting for space"],
    ];
    for (const [color, label] of dotRows) {
      const row = document.createElement("div");
      const dot = document.createElement("span");
      dot.className = "sim-dot";
      dot.style.background = color;
      row.appendChild(dot);
      row.appendChild(document.createTextNode(label));
      legend.appendChild(row);
    }
    return legend;
  }

  function render(): void {
    panel.replaceChildren();
    if (!report) {
      panel.classList.remove("visible");
      return;
    }
    panel.classList.add("visible");
    const r = report.report;

    // Mismatch guard (deliverable D): still renders underneath, per spec
    // — this is advisory, not a block.
    if (layoutInfo && layoutInfo.entities !== r.entities) {
      const warn = document.createElement("div");
      warn.className = "sim-report-mismatch";
      warn.textContent =
        `⚠ report is for a different layout ` +
        `(${r.entities} entities vs current ${layoutInfo.entities})`;
      panel.appendChild(warn);
    }

    const title = document.createElement("div");
    title.className = "sim-report-title";
    title.style.color = verdictColor(r.overall_verdict);
    title.textContent = `SIM ${r.overall_verdict ?? "?"} — ${r.label ?? "report"}`;
    panel.appendChild(title);

    for (const item of r.items) {
      const row = document.createElement("div");
      row.className = "sim-report-row";
      if (item.is_target) row.classList.add("target");
      let text = `${item.item}: ${fmtRate(item.planned_rate)} planned → ${fmtRate(item.measured_produced_rate)} produced`;
      if (item.measured_delivered_rate != null) {
        text += ` → ${fmtRate(item.measured_delivered_rate)} delivered`;
      }
      row.appendChild(document.createTextNode(text));
      if (item.verdict) {
        const badge = document.createElement("span");
        badge.className = "sim-report-verdict";
        badge.style.color = verdictColor(item.verdict);
        badge.textContent = ` [${item.verdict}]`;
        row.appendChild(badge);
      }
      panel.appendChild(row);
    }

    panel.appendChild(renderLegend());
  }

  function showError(message: string): void {
    panel.replaceChildren();
    panel.classList.add("visible");
    const el = document.createElement("div");
    el.className = "sim-report-error";
    el.textContent = `Failed to load sim report: ${message}`;
    panel.appendChild(el);
  }

  return {
    loadFromText(jsonText: string): void {
      try {
        report = parseSimReport(jsonText);
      } catch (err) {
        report = null;
        showError(err instanceof Error ? err.message : String(err));
        onChange(null);
        return;
      }
      render();
      onChange(report);
    },
    clear(): void {
      report = null;
      panel.classList.remove("visible");
      panel.replaceChildren();
      onChange(null);
    },
    getReport(): SimReport | null {
      return report;
    },
    setLayoutInfo(info: SimLayoutInfo | null): void {
      layoutInfo = info;
      if (report) render();
    },
  };
}
