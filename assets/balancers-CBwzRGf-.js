import { A as y, T as b, C as x, U as g, d as w, a as v, b as S, c as T } from "./index-CTRsdU8m.js";
const h = 10, f = 10, C = 12, E = 12;
function q(t, s) {
  t.innerHTML = "", t.style.display = "block", t.style.height = "100vh", t.style.width = "100vw", t.style.overflow = "auto";
  const e = document.createElement("div");
  e.id = "balancer-showcase", e.style.padding = "24px 32px 64px", e.style.maxWidth = "1600px", e.style.margin = "0 auto", e.style.fontFamily = "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif", t.appendChild(e), H();
  const n = document.createElement("header");
  n.className = "bs-header", n.innerHTML = `
    <div>
      <h1>Balancer template showcase</h1>
      <p>Library (Factorio-SAT) vs compose-bake generator, side-by-side. Issue #274.</p>
    </div>
    <a href="#/" class="bs-back">\u2190 back</a>
  `, e.appendChild(n);
  const a = document.createElement("div");
  a.className = "bs-grid", a.style.setProperty("--bs-cols", String(f)), e.appendChild(a);
  const r = L(), o = /* @__PURE__ */ new Map();
  for (let i = 1; i <= h; i++) for (let c = 1; c <= f; c++) {
    const d = $(i, c, r);
    o.set(m(i, c), d), a.appendChild(d);
  }
  if (r) {
    const i = o.get(m(r.n, r.m));
    i && requestAnimationFrame(() => {
      i.scrollIntoView({ behavior: "auto", block: "center" });
    });
  }
  k(s, o);
}
function L() {
  const t = window.location.hash, s = t.indexOf("?");
  if (s < 0) return null;
  const n = new URLSearchParams(t.slice(s + 1)).get("focus");
  if (!n) return null;
  const [a, r] = n.split(","), o = Number.parseInt(a, 10), i = Number.parseInt(r, 10);
  return !Number.isFinite(o) || !Number.isFinite(i) || o < 1 || o > h || i < 1 || i > f ? null : { n: o, m: i };
}
function m(t, s) {
  return `${t},${s}`;
}
function $(t, s, e) {
  const n = document.createElement("div");
  return n.className = "bs-cell", n.dataset.n = String(t), n.dataset.m = String(s), e && e.n === t && e.m === s && n.classList.add("bs-focus"), n.innerHTML = `
    <header class="bs-cell-header">
      <span class="bs-shape">(${t}, ${s})</span>
      <span class="bs-summary"></span>
    </header>
    <div class="bs-panels">
      <div class="bs-panel" data-side="library">
        <div class="bs-panel-caption">loading\u2026</div>
        <div class="bs-panel-canvas"></div>
      </div>
    </div>
  `, n;
}
async function k(t, s) {
  let e;
  try {
    e = await t.balancerShowcase(h, f);
  } catch (a) {
    for (const r of s.values()) {
      const o = r.querySelector(".bs-summary");
      o && (o.textContent = "showcase fetch failed");
    }
    console.error("[balancers] showcase fetch failed", a);
    return;
  }
  const n = new y();
  await n.init({ width: C * b, height: E * b, background: 1973790, antialias: true, autoStart: false, sharedTicker: false, preference: "webgl" });
  for (const a of e) {
    const r = s.get(m(a.n_inputs, a.n_outputs));
    if (!r) continue;
    M(n, r, a.library);
    const o = r.querySelector(".bs-summary");
    o && (o.innerHTML = _(a));
  }
  n.destroy(true, { children: true, texture: true });
}
function M(t, s, e) {
  const n = s.querySelector('.bs-panel[data-side="library"]');
  if (!n) return;
  const a = n.querySelector(".bs-panel-caption"), r = n.querySelector(".bs-panel-canvas");
  if (!a || !r) return;
  if (!e) {
    n.classList.add("bs-empty"), a.textContent = "no library entry", r.innerHTML = "";
    return;
  }
  const o = e.entities.length, i = e.strategy ? `<div class="bs-strategy">${u(e.strategy)}</div>` : "", c = e.reference ? `<div class="bs-reference"><a href="${u(e.reference)}" target="_blank" rel="noopener">ref</a></div>` : "";
  a.innerHTML = `
    <div class="bs-source">${u(e.source)}${c}</div>
    ${i}
    <div class="bs-stats">
      ${e.n_inputs} \u2192 ${e.n_outputs} \xB7
      ${e.width}\xD7${e.height} \xB7
      <strong>${o} ${o === 1 ? "entity" : "entities"}</strong>
    </div>
  `;
  const d = I(t, e);
  r.innerHTML = "", r.appendChild(d);
}
function I(t, s) {
  const e = Math.max(1, s.width) * b, n = Math.max(1, s.height) * b;
  (t.renderer.width !== e || t.renderer.height !== n) && t.renderer.resize(e, n);
  const a = new x(), r = T(), o = /* @__PURE__ */ new Map();
  for (const l of s.entities) g.has(l.name) && o.set(`${l.x ?? 0},${l.y ?? 0}`, l);
  for (const l of s.entities) {
    if (!g.has(l.name) || l.io_type !== "input") continue;
    const p = w(l, o);
    p && a.addChild(p);
  }
  for (const l of s.entities) v(l, r);
  for (const l of s.entities) {
    const p = S(l, r);
    a.addChild(p);
  }
  const i = t.renderer.extract.canvas(a), c = document.createElement("canvas");
  c.width = i.width, c.height = i.height;
  const d = c.getContext("2d");
  return d && d.drawImage(i, 0, 0), c.style.maxWidth = "100%", c.style.height = "auto", c.style.imageRendering = "pixelated", a.removeChildren(), c;
}
function _(t) {
  const s = t.library;
  return s ? `<span class="${s.source.startsWith("Raynquist") ? "bs-source-raynquist" : s.source === "compose" ? "bs-source-compose" : s.source === "Factorio-SAT" ? "bs-source-fsat" : "bs-source-other"}">${u(s.source)}</span>` : '<span class="bs-none">no template</span>';
}
function u(t) {
  return t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function H() {
  if (document.getElementById("bs-styles")) return;
  const t = document.createElement("style");
  t.id = "bs-styles", t.textContent = `
    #balancer-showcase {
      color: #d8d8d8;
    }
    .bs-header {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      margin-bottom: 24px;
      gap: 16px;
    }
    .bs-header h1 {
      font-size: 22px;
      margin: 0 0 4px;
      font-weight: 500;
      letter-spacing: 0.2px;
    }
    .bs-header p {
      margin: 0;
      color: #888;
      font-size: 13px;
    }
    .bs-back {
      color: #6a9ed8;
      text-decoration: none;
      font-size: 13px;
    }
    .bs-back:hover { text-decoration: underline; }

    .bs-grid {
      display: grid;
      grid-template-columns: repeat(var(--bs-cols, 10), minmax(0, 1fr));
      gap: 12px;
    }
    @media (max-width: 1400px) {
      .bs-grid { grid-template-columns: repeat(5, minmax(0, 1fr)); }
    }
    @media (max-width: 800px) {
      .bs-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    }

    .bs-cell {
      background: #232323;
      border: 1px solid #2e2e2e;
      border-radius: 6px;
      padding: 8px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      min-width: 0;
    }
    .bs-cell.bs-focus {
      border-color: #d39a3a;
      box-shadow: 0 0 0 2px rgba(211, 154, 58, 0.25);
    }
    .bs-cell-header {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      gap: 6px;
      font-size: 12px;
    }
    .bs-shape {
      font-weight: 600;
      color: #f0f0f0;
      font-variant-numeric: tabular-nums;
    }
    .bs-summary { color: #888; font-size: 11px; font-weight: 500; }
    .bs-summary .bs-source-raynquist { color: #d39a3a; }
    .bs-summary .bs-source-compose   { color: #4eb072; }
    .bs-summary .bs-source-fsat      { color: #6a9ed8; }
    .bs-summary .bs-source-other     { color: #aaa; }
    .bs-summary .bs-none             { color: #555; }

    .bs-panels {
      display: grid;
      grid-template-columns: 1fr;
      gap: 6px;
    }
    .bs-panel {
      background: #1a1a1a;
      border-radius: 4px;
      padding: 6px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      min-height: 80px;
    }
    .bs-panel.bs-empty {
      opacity: 0.45;
      justify-content: center;
      align-items: center;
      text-align: center;
    }
    .bs-panel-caption {
      font-size: 10px;
      line-height: 1.3;
      color: #aaa;
    }
    .bs-panel-caption .bs-source {
      font-weight: 500;
      color: #d8d8d8;
      display: flex;
      align-items: baseline;
      gap: 6px;
    }
    .bs-panel-caption .bs-strategy {
      color: #888;
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      font-size: 9px;
      margin-top: 2px;
      word-break: break-word;
    }
    .bs-panel-caption .bs-reference a {
      color: #6a9ed8;
      text-decoration: none;
      font-size: 9px;
    }
    .bs-panel-caption .bs-reference a:hover {
      text-decoration: underline;
    }
    .bs-panel-caption .bs-stats {
      color: #888;
      font-variant-numeric: tabular-nums;
      margin-top: 2px;
    }
    .bs-panel-caption strong {
      color: #d8d8d8;
      font-weight: 600;
    }
    .bs-panel-canvas {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 0;
      max-height: 320px;
      overflow: hidden;
    }
    .bs-panel-canvas canvas {
      display: block;
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }
  `, document.head.appendChild(t);
}
export {
  q as renderBalancerShowcase
};
