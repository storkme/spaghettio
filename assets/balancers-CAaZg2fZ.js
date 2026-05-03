import { A as y, T as b, C as x, U as f, d as v, a as w, b as S, c as T } from "./index-IkibIibZ.js";
const h = 10, u = 10, C = 12, E = 12;
function P(n, t) {
  n.innerHTML = "", n.style.display = "block", n.style.height = "100vh", n.style.width = "100vw", n.style.overflow = "auto";
  const s = document.createElement("div");
  s.id = "balancer-showcase", s.style.padding = "24px 32px 64px", s.style.maxWidth = "1600px", s.style.margin = "0 auto", s.style.fontFamily = "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif", n.appendChild(s), H();
  const e = document.createElement("header");
  e.className = "bs-header", e.innerHTML = `
    <div>
      <h1>Balancer template showcase</h1>
      <p>Library (Factorio-SAT) vs compose-bake generator, side-by-side. Issue #274.</p>
    </div>
    <a href="#/" class="bs-back">\u2190 back</a>
  `, s.appendChild(e);
  const r = document.createElement("div");
  r.className = "bs-grid", r.style.setProperty("--bs-cols", String(u)), s.appendChild(r);
  const a = $(), i = /* @__PURE__ */ new Map();
  for (let o = 1; o <= h; o++) for (let c = 1; c <= u; c++) {
    const d = L(o, c, a);
    i.set(m(o, c), d), r.appendChild(d);
  }
  if (a) {
    const o = i.get(m(a.n, a.m));
    o && requestAnimationFrame(() => {
      o.scrollIntoView({ behavior: "auto", block: "center" });
    });
  }
  M(t, i);
}
function $() {
  const n = window.location.hash, t = n.indexOf("?");
  if (t < 0) return null;
  const e = new URLSearchParams(n.slice(t + 1)).get("focus");
  if (!e) return null;
  const [r, a] = e.split(","), i = Number.parseInt(r, 10), o = Number.parseInt(a, 10);
  return !Number.isFinite(i) || !Number.isFinite(o) || i < 1 || i > h || o < 1 || o > u ? null : { n: i, m: o };
}
function m(n, t) {
  return `${n},${t}`;
}
function L(n, t, s) {
  const e = document.createElement("div");
  return e.className = "bs-cell", e.dataset.n = String(n), e.dataset.m = String(t), s && s.n === n && s.m === t && e.classList.add("bs-focus"), e.innerHTML = `
    <header class="bs-cell-header">
      <span class="bs-shape">(${n}, ${t})</span>
      <span class="bs-summary"></span>
    </header>
    <div class="bs-panels">
      <div class="bs-panel" data-side="library">
        <div class="bs-panel-caption">loading\u2026</div>
        <div class="bs-panel-canvas"></div>
      </div>
      <div class="bs-panel" data-side="generated">
        <div class="bs-panel-caption">loading\u2026</div>
        <div class="bs-panel-canvas"></div>
      </div>
    </div>
  `, e;
}
async function M(n, t) {
  let s;
  try {
    s = await n.balancerShowcase(h, u);
  } catch (r) {
    for (const a of t.values()) {
      const i = a.querySelector(".bs-summary");
      i && (i.textContent = "showcase fetch failed");
    }
    console.error("[balancers] showcase fetch failed", r);
    return;
  }
  const e = new y();
  await e.init({ width: C * b, height: E * b, background: 1973790, antialias: true, autoStart: false, sharedTicker: false, preference: "webgl" });
  for (const r of s) {
    const a = t.get(m(r.n_inputs, r.n_outputs));
    if (!a) continue;
    g(e, a, "library", r.library), g(e, a, "generated", r.generated);
    const i = a.querySelector(".bs-summary");
    i && (i.innerHTML = k(r));
  }
  e.destroy(true, { children: true, texture: true });
}
function g(n, t, s, e) {
  const r = t.querySelector(`.bs-panel[data-side="${s}"]`);
  if (!r) return;
  const a = r.querySelector(".bs-panel-caption"), i = r.querySelector(".bs-panel-canvas");
  if (!a || !i) return;
  if (!e) {
    r.classList.add("bs-empty"), a.textContent = s === "library" ? "no library entry" : "no compose entry", i.innerHTML = "";
    return;
  }
  const o = e.entities.length;
  a.innerHTML = `
    <div class="bs-source">${_(e.source)}</div>
    <div class="bs-stats">
      ${e.n_inputs} \u2192 ${e.n_outputs} \xB7
      ${e.width}\xD7${e.height} \xB7
      <strong>${o} ${o === 1 ? "entity" : "entities"}</strong>
    </div>
  `;
  const c = I(n, e);
  i.innerHTML = "", i.appendChild(c);
}
function I(n, t) {
  const s = Math.max(1, t.width) * b, e = Math.max(1, t.height) * b;
  (n.renderer.width !== s || n.renderer.height !== e) && n.renderer.resize(s, e);
  const r = new x(), a = T(), i = /* @__PURE__ */ new Map();
  for (const l of t.entities) f.has(l.name) && i.set(`${l.x ?? 0},${l.y ?? 0}`, l);
  for (const l of t.entities) {
    if (!f.has(l.name) || l.io_type !== "input") continue;
    const p = v(l, i);
    p && r.addChild(p);
  }
  for (const l of t.entities) w(l, a);
  for (const l of t.entities) {
    const p = S(l, a);
    r.addChild(p);
  }
  const o = n.renderer.extract.canvas(r), c = document.createElement("canvas");
  c.width = o.width, c.height = o.height;
  const d = c.getContext("2d");
  return d && d.drawImage(o, 0, 0), c.style.maxWidth = "100%", c.style.height = "auto", c.style.imageRendering = "pixelated", r.removeChildren(), c;
}
function k(n) {
  const t = n.library, s = n.generated;
  if (t && s) {
    const e = t.entities.length, r = s.entities.length;
    if (e === 0) return "library has 0 entities (?)";
    const a = r - e;
    if (a === 0) return '<span class="bs-eq">parity</span>';
    if (a < 0) {
      const o = Math.round(100 * -a / e);
      return `<span class="bs-win">compose \u2212${-a} (\u2212${o}%)</span>`;
    }
    const i = Math.round(100 * a / e);
    return `<span class="bs-lose">compose +${a} (+${i}%)</span>`;
  }
  return t && !s ? '<span class="bs-only-lib">library only</span>' : !t && s ? '<span class="bs-only-gen">compose only</span>' : '<span class="bs-none">no template</span>';
}
function _(n) {
  return n.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function H() {
  if (document.getElementById("bs-styles")) return;
  const n = document.createElement("style");
  n.id = "bs-styles", n.textContent = `
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
    .bs-summary { color: #888; font-size: 11px; }
    .bs-summary .bs-win { color: #4eb072; }
    .bs-summary .bs-lose { color: #d35a5a; }
    .bs-summary .bs-eq { color: #888; }
    .bs-summary .bs-only-lib { color: #6a9ed8; }
    .bs-summary .bs-only-gen { color: #d39a3a; }
    .bs-summary .bs-none { color: #555; }

    .bs-panels {
      display: grid;
      grid-template-columns: 1fr 1fr;
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
    }
    .bs-panel-caption .bs-stats {
      color: #888;
      font-variant-numeric: tabular-nums;
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
  `, document.head.appendChild(n);
}
export {
  P as renderBalancerShowcase
};
