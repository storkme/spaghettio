import type { LayoutResult } from "../engine";
import { getTracePhases, type TraceEvent } from "../renderer/traceOverlay";
import "./stepThrough.css";

export interface StepThroughDeps {
  getLayout(): LayoutResult | null;
  isEnabled(): boolean;
  onPhaseChange(): void;
  onJumpToFailure(fromX: number, fromY: number): void;
  isModalBlocking?: () => boolean;
}

export interface StepThroughControls {
  update(): void;
  getPhaseIndex(): number;
  reset(): void;
}

type PhaseTimeEvent     = Extract<TraceEvent, { phase: "PhaseTime" }>;
type PhaseCompleteEvent = Extract<TraceEvent, { phase: "PhaseComplete" }>;
type PhaseSnapshotEvent = Extract<TraceEvent, { phase: "PhaseSnapshot" }>;

function shortName(name: string): string {
  // Emitted PhaseComplete names (rows_placed, lanes_planned, poles_placed,
  // bus_routed) all shorten cleanly to their first word.
  const first = name.split("_")[0];
  return first.length > 6 ? first.slice(0, 5) + "…" : first;
}

export function createStepThrough(
  container: HTMLElement,
  deps: StepThroughDeps,
): StepThroughControls {
  let phaseIndex = -1;

  /* ── Shell ─────────────────────────────────────────────────────── */
  const bar = document.createElement("div");
  bar.className = "step-through-bar";

  const prevBtn = document.createElement("button");
  prevBtn.className = "step-through-btn";
  prevBtn.textContent = "◀";
  prevBtn.title = "Previous phase (←)";

  const trackWrap = document.createElement("div");
  trackWrap.className = "st-track-wrap";

  const nextBtn = document.createElement("button");
  nextBtn.className = "step-through-btn";
  nextBtn.textContent = "▶";
  nextBtn.title = "Next phase (→)";

  const failBtn = document.createElement("button");
  failBtn.className = "step-through-fail";

  bar.append(prevBtn, trackWrap, nextBtn, failBtn);
  container.appendChild(bar);

  const tooltip = document.createElement("div");
  tooltip.className = "st-tooltip";
  document.body.appendChild(tooltip);

  /* ── Track builder ──────────────────────────────────────────────── */
  function buildTrack(
    trace: TraceEvent[],
    phases: { name: string; eventIndex: number }[],
  ): void {
    trackWrap.innerHTML = "";
    if (phases.length === 0) return;

    /* Which phases have a PhaseSnapshot (entity-level detail). */
    const snapshotPhases = new Set<string>();
    for (const evt of trace) {
      if (evt.phase === "PhaseSnapshot") {
        snapshotPhases.add((evt as PhaseSnapshotEvent).data.phase);
      }
    }

    /* Cumulative ms at each event index. O(n). */
    let runningMs = 0;
    const cumMs = new Array<number>(trace.length);
    for (let i = 0; i < trace.length; i++) {
      if (trace[i].phase === "PhaseTime") {
        runningMs += (trace[i] as PhaseTimeEvent).data.duration_ms;
      }
      cumMs[i] = runningMs;
    }
    const totalMs = runningMs;

    const phaseUpTo  = phases.map(p => cumMs[p.eventIndex] ?? 0);
    const phaseDelta = phases.map((_, i) =>
      i === 0 ? phaseUpTo[0] : phaseUpTo[i] - phaseUpTo[i - 1],
    );

    function entityCountForPhase(name: string): number {
      for (const evt of trace) {
        if (evt.phase === "PhaseComplete" &&
            (evt as PhaseCompleteEvent).data.phase === name) {
          return (evt as PhaseCompleteEvent).data.entity_count;
        }
      }
      return 0;
    }

    const totalEntities = phases.length > 0
      ? entityCountForPhase(phases[phases.length - 1].name)
      : 0;

    const slots = document.createElement("div");
    slots.className = "st-slots";

    /* "all" anchor — leftmost slot, always clickable. */
    const allSlot = document.createElement("div");
    allSlot.className = "st-slot st-all" + (phaseIndex === -1 ? " active" : "");

    const allTick = document.createElement("div");
    allTick.className = "st-tick";

    const allLabel = document.createElement("div");
    allLabel.className = "st-label";
    allLabel.textContent = "all";

    allSlot.append(allTick, allLabel);

    function showTooltip(el: HTMLElement, text: string): void {
      tooltip.textContent = text;
      tooltip.style.display = "block";
      const r = el.getBoundingClientRect();
      tooltip.style.left = (r.left + r.width / 2) + "px";
      tooltip.style.top  = (r.top - 34) + "px";
    }

    allSlot.addEventListener("mouseenter", () =>
      showTooltip(allSlot, `all phases — ${totalEntities} entities, ${totalMs.toFixed(0)}ms total`),
    );
    allSlot.addEventListener("mouseleave", () => { tooltip.style.display = "none"; });
    allSlot.addEventListener("click", () => { phaseIndex = -1; deps.onPhaseChange(); });

    slots.appendChild(allSlot);

    /* One slot per phase. */
    for (let i = 0; i < phases.length; i++) {
      const ph = phases[i];
      const isActive    = i === phaseIndex;
      const hasSnapshot = snapshotPhases.has(ph.name);
      const deltaMs     = phaseDelta[i];
      const entities    = entityCountForPhase(ph.name);

      const slot = document.createElement("div");
      slot.className = "st-slot"
        + (isActive    ? " active" : "")
        + (!hasSnapshot ? " st-dim" : "");

      const tick = document.createElement("div");
      tick.className = "st-tick";

      const labelEl = document.createElement("div");
      labelEl.className = "st-label";
      labelEl.textContent = shortName(ph.name);

      slot.append(tick, labelEl);

      const tipText = hasSnapshot
        ? `${ph.name} — ${entities} entities, +${deltaMs.toFixed(0)}ms (${phaseUpTo[i].toFixed(0)}ms total)`
        : `${ph.name} — trace events only, +${deltaMs.toFixed(0)}ms (no entity snapshot)`;

      slot.addEventListener("mouseenter", () => showTooltip(slot, tipText));
      slot.addEventListener("mouseleave", () => { tooltip.style.display = "none"; });
      slot.addEventListener("click", () => { phaseIndex = i; deps.onPhaseChange(); });

      slots.appendChild(slot);
    }

    trackWrap.appendChild(slots);
  }

  /* ── Public update ──────────────────────────────────────────────── */
  function update(): void {
    const layout = deps.getLayout();
    if (!deps.isEnabled() || !layout?.trace?.length) {
      bar.style.display = "none";
      return;
    }
    const trace  = layout.trace as TraceEvent[];
    const phases = getTracePhases(trace);
    if (phases.length === 0) {
      bar.style.display = "none";
      return;
    }
    bar.style.display = "flex";

    buildTrack(trace, phases);

    const failCount = trace.filter(e => e.phase === "RouteFailure").length;
    failBtn.textContent = `⚠ ${failCount}`;
    failBtn.style.display = failCount > 0 ? "inline-block" : "none";

    /* ◀ is disabled only at "all" — nothing comes before it.
     * At phase 0, ◀ returns to "all". ▶ is disabled at the last phase. */
    prevBtn.disabled = phaseIndex === -1;
    nextBtn.disabled = phaseIndex >= phases.length - 1;
  }

  /* ── Navigation ─────────────────────────────────────────────────── */
  function stepPrev(): void {
    if (phaseIndex === -1) return; // already at "all", button is disabled
    const layout = deps.getLayout();
    const phases = getTracePhases((layout?.trace ?? []) as TraceEvent[]);
    if (phases.length === 0) return;
    if (phaseIndex > 0) phaseIndex--;
    else phaseIndex = -1; // phase 0 → "all"
    deps.onPhaseChange();
  }

  function stepNext(): void {
    const layout = deps.getLayout();
    const phases = getTracePhases((layout?.trace ?? []) as TraceEvent[]);
    if (phaseIndex < phases.length - 1) phaseIndex++;
    deps.onPhaseChange();
  }

  function jumpToFailure(): void {
    const layout = deps.getLayout();
    if (!layout?.trace) return;
    const failures = (layout.trace as TraceEvent[]).filter(
      e => e.phase === "RouteFailure",
    ) as Extract<TraceEvent, { phase: "RouteFailure" }>[];
    if (failures.length === 0) return;
    const first = failures[0].data;
    deps.onJumpToFailure(first.from_x, first.from_y);
  }

  prevBtn.addEventListener("click", stepPrev);
  nextBtn.addEventListener("click", stepNext);
  failBtn.addEventListener("click", jumpToFailure);

  document.addEventListener("keydown", (e) => {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
    if (bar.style.display === "none") return;
    if (deps.isModalBlocking?.()) return;

    if (e.key === "ArrowLeft") {
      e.preventDefault();
      stepPrev();
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      stepNext();
    } else if (e.key === "f") {
      e.preventDefault();
      jumpToFailure();
    }
  });

  return {
    update,
    getPhaseIndex: () => phaseIndex,
    reset: () => { phaseIndex = -1; },
  };
}
