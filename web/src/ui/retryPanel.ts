import * as debugState from "../state/debugState";
import "./retryPanel.css";

// Mirrors the Rust `TraceEvent::LayoutRetried` variant. After a WASM
// rebuild, the auto-generated `TraceEvent` union from
// `wasm-pkg/fucktorio_wasm.d.ts` will include this shape and can
// replace the local declaration.
interface LayoutRetriedEvent {
  phase: "LayoutRetried";
  data: {
    gaps: Array<[number, number]>;
    caps_before: number;
    recipes: string[];
  };
}

interface MinimalLayout {
  trace?: Array<{ phase: string; data?: unknown }>;
}

export interface RetryPanelControls {
  /**
   * Re-read the current layout's trace and re-render the panel.
   * Hides if no `LayoutRetried` event is present or debug mode is off.
   */
  update(layout: MinimalLayout | null): void;
}

export function createRetryPanel(container: HTMLElement): RetryPanelControls {
  const panel = document.createElement("div");
  panel.className = "retry-panel";
  container.appendChild(panel);

  let lastLayout: MinimalLayout | null = null;

  function findRetryEvent(layout: MinimalLayout | null): LayoutRetriedEvent["data"] | null {
    if (!layout?.trace) return null;
    for (const evt of layout.trace) {
      if (evt.phase === "LayoutRetried") {
        return (evt as LayoutRetriedEvent).data;
      }
    }
    return null;
  }

  function render(): void {
    const debugOn = debugState.get().master;
    const data = findRetryEvent(lastLayout);
    if (!debugOn || !data) {
      panel.classList.remove("visible");
      panel.replaceChildren();
      return;
    }
    panel.classList.add("visible");
    panel.replaceChildren();

    const title = document.createElement("div");
    title.className = "retry-panel-title";
    const widened = data.gaps.length;
    title.textContent = `Layout retry: ${widened} row${widened === 1 ? "" : "s"} widened`;
    panel.appendChild(title);

    const summary = document.createElement("div");
    summary.className = "retry-panel-summary";
    summary.textContent = `${data.caps_before} junction cap${data.caps_before === 1 ? "" : "s"} before retry`;
    panel.appendChild(summary);

    for (let i = 0; i < data.gaps.length; i++) {
      const [rowIdx, extra] = data.gaps[i];
      const recipe = data.recipes[i] ?? "?";
      const row = document.createElement("div");
      row.className = "retry-panel-row";
      const recipeSpan = document.createElement("span");
      recipeSpan.className = "recipe";
      recipeSpan.textContent = recipe;
      const gapSpan = document.createElement("span");
      gapSpan.className = "gap";
      gapSpan.textContent = `+${extra} tile${extra === 1 ? "" : "s"}`;
      row.appendChild(document.createTextNode(`row ${rowIdx} (`));
      row.appendChild(recipeSpan);
      row.appendChild(document.createTextNode("): "));
      row.appendChild(gapSpan);
      panel.appendChild(row);
    }
  }

  debugState.subscribe(() => render());

  return {
    update(layout) {
      lastLayout = layout;
      render();
    },
  };
}
