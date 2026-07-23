import { L as y, z as Ce, M as Te, N as K, O as B, P as b, Q as Ue, y as qe, V as Le, n as Pe, h as ke, W as Oe, X as Ne, Y as Ye, Z as je, _ as Ve, $ as $e, a0 as _e, a1 as Xe, a2 as Qe, a3 as ee } from "./index-DxohrT3V.js";
import { R as ze, S as Je, B as Ke, a as Ze, b as et, c as tt, A as st, C as nt } from "./RenderTargetSystem-CRiFoTVm.js";
import "./Filter-Bvv95Cgl.js";
const he = class q {
  static _getPatternRepeat(e, t) {
    const n = e && e !== "clamp-to-edge", a = t && t !== "clamp-to-edge";
    return n && a ? "repeat" : n ? "repeat-x" : a ? "repeat-y" : "no-repeat";
  }
  start(e, t, n) {
  }
  execute(e, t) {
    var _a, _b, _c, _d;
    const n = t.elements;
    if (!n || !n.length) return;
    const a = e.renderer, o = a.canvasContext, r = o.activeContext;
    for (let c = 0; c < n.length; c++) {
      const d = n[c];
      if (!d.packAsQuad) continue;
      const u = d, l = u.texture, T = l ? y.getCanvasSource(l) : null;
      if (!T) continue;
      const x = l.source.style, j = o.smoothProperty, E = x.scaleMode !== "nearest";
      r[j] !== E && (r[j] = E), o.setBlendMode(t.blendMode);
      const H = ((_a = a.globalUniforms.globalUniformData) == null ? void 0 : _a.worldColor) ?? 4294967295, P = u.color, w = (H >>> 24 & 255) / 255, g = (P >>> 24 & 255) / 255, I = ((_b = a.filter) == null ? void 0 : _b.alphaMultiplier) ?? 1, W = w * g * I;
      if (W <= 0) continue;
      r.globalAlpha = W;
      const X = H & 16777215, k = P & 16777215, F = Te(K(k, X)), R = l.frame, V = x.addressModeU ?? x.addressMode, L = x.addressModeV ?? x.addressMode, p = q._getPatternRepeat(V, L), m = l.source._resolution ?? l.source.resolution ?? 1, f = (_d = (_c = u.renderable) == null ? void 0 : _c.renderGroup) == null ? void 0 : _d.isCachedAsTexture, G = R.x * m, O = R.y * m, _ = R.width * m, D = R.height * m, v = u.bounds, S = a.renderTarget.renderTarget.isRoot, M = v.minX, $ = v.minY, U = v.maxX - v.minX, A = v.maxY - v.minY, C = l.rotate, i = l.uvs, te = Math.min(i.x0, i.x1, i.x2, i.x3, i.y0, i.y1, i.y2, i.y3), Ee = Math.max(i.x0, i.x1, i.x2, i.x3, i.y0, i.y1, i.y2, i.y3), de = p !== "no-repeat" && (te < 0 || Ee > 1), se = C && !(!de && (F !== 16777215 || C));
      se ? (q._tempPatternMatrix.copyFrom(u.transform), Ce.matrixAppendRotationInv(q._tempPatternMatrix, C, M, $, U, A), o.setContextTransform(q._tempPatternMatrix, u.roundPixels === 1, void 0, f && S)) : o.setContextTransform(u.transform, u.roundPixels === 1, void 0, f && S);
      const Q = se ? 0 : M, z = se ? 0 : $, ne = U, ae = A;
      if (de) {
        let oe = T;
        const N = F !== 16777215 && !C, Y = R.width <= l.source.width && R.height <= l.source.height;
        N && Y && (oe = y.getTintedCanvas({ texture: l }, F));
        const re = r.createPattern(oe, p);
        if (!re) continue;
        const pe = ne, ue = ae;
        if (pe === 0 || ue === 0) continue;
        const me = 1 / pe, fe = 1 / ue, ge = (i.x1 - i.x0) * me, ve = (i.y1 - i.y0) * me, ye = (i.x3 - i.x0) * fe, xe = (i.y3 - i.y0) * fe, We = i.x0 - ge * Q - ye * z, Fe = i.y0 - ve * Q - xe * z, ie = l.source.pixelWidth, ce = l.source.pixelHeight;
        q._tempPatternMatrix.set(ge * ie, ve * ce, ye * ie, xe * ce, We * ie, Fe * ce), y.applyPatternTransform(re, q._tempPatternMatrix), r.fillStyle = re, r.fillRect(Q, z, ne, ae);
      } else {
        const N = F !== 16777215 || C ? y.getTintedCanvas({ texture: l }, F) : T, Y = N !== T;
        r.drawImage(N, Y ? 0 : G, Y ? 0 : O, Y ? N.width : _, Y ? N.height : D, Q, z, ne, ae);
      }
    }
  }
};
he._tempPatternMatrix = new B();
he.extension = { type: [b.CanvasPipesAdaptor], name: "batch" };
let at = he;
class be {
  constructor(e) {
    this._colorStack = [], this._colorStackIndex = 0, this._currentColor = 0, this._renderer = e;
  }
  buildStart() {
    this._colorStack[0] = 15, this._colorStackIndex = 1, this._currentColor = 15;
  }
  push(e, t, n) {
    this._renderer.renderPipes.batch.break(n);
    const a = this._colorStack;
    a[this._colorStackIndex] = a[this._colorStackIndex - 1] & e.mask;
    const o = this._colorStack[this._colorStackIndex];
    o !== this._currentColor && (this._currentColor = o, n.add({ renderPipeId: "colorMask", colorMask: o, canBundle: false })), this._colorStackIndex++;
  }
  pop(e, t, n) {
    this._renderer.renderPipes.batch.break(n);
    const a = this._colorStack;
    this._colorStackIndex--;
    const o = a[this._colorStackIndex - 1];
    o !== this._currentColor && (this._currentColor = o, n.add({ renderPipeId: "colorMask", colorMask: o, canBundle: false }));
  }
  execute(e) {
  }
  destroy() {
    this._renderer = null, this._colorStack = null;
  }
}
be.extension = { type: [b.CanvasPipes], name: "colorMask" };
function ot(s, e, t, n, a, o) {
  o = Math.max(0, Math.min(o, Math.min(n, a) / 2)), s.moveTo(e + o, t), s.lineTo(e + n - o, t), s.quadraticCurveTo(e + n, t, e + n, t + o), s.lineTo(e + n, t + a - o), s.quadraticCurveTo(e + n, t + a, e + n - o, t + a), s.lineTo(e + o, t + a), s.quadraticCurveTo(e, t + a, e, t + a - o), s.lineTo(e, t + o), s.quadraticCurveTo(e, t, e + o, t);
}
function Se(s, e) {
  switch (e.type) {
    case "rectangle": {
      const t = e;
      s.rect(t.x, t.y, t.width, t.height);
      break;
    }
    case "roundedRectangle": {
      const t = e;
      ot(s, t.x, t.y, t.width, t.height, t.radius);
      break;
    }
    case "circle": {
      const t = e;
      s.moveTo(t.x + t.radius, t.y), s.arc(t.x, t.y, t.radius, 0, Math.PI * 2);
      break;
    }
    case "ellipse": {
      const t = e;
      s.ellipse ? (s.moveTo(t.x + t.halfWidth, t.y), s.ellipse(t.x, t.y, t.halfWidth, t.halfHeight, 0, 0, Math.PI * 2)) : (s.save(), s.translate(t.x, t.y), s.scale(t.halfWidth, t.halfHeight), s.moveTo(1, 0), s.arc(0, 0, 1, 0, Math.PI * 2), s.restore());
      break;
    }
    case "triangle": {
      const t = e;
      s.moveTo(t.x, t.y), s.lineTo(t.x2, t.y2), s.lineTo(t.x3, t.y3), s.closePath();
      break;
    }
    case "polygon":
    default: {
      const t = e, n = t.points;
      if (!(n == null ? void 0 : n.length)) break;
      s.moveTo(n[0], n[1]);
      for (let a = 2; a < n.length; a += 2) s.lineTo(n[a], n[a + 1]);
      t.closePath && s.closePath();
      break;
    }
  }
}
function rt(s, e) {
  if (!(e == null ? void 0 : e.length)) return false;
  for (let t = 0; t < e.length; t++) {
    const n = e[t];
    if (!(n == null ? void 0 : n.shape)) continue;
    const a = n.transform, o = a && !a.isIdentity();
    o && (s.save(), s.transform(a.a, a.b, a.c, a.d, a.tx, a.ty)), Se(s, n.shape), o && s.restore();
  }
  return true;
}
class Me {
  constructor(e) {
    this._warnedMaskTypes = /* @__PURE__ */ new Set(), this._canvasMaskStack = [], this._renderer = e;
  }
  push(e, t, n) {
    this._renderer.renderPipes.batch.break(n), n.add({ renderPipeId: "stencilMask", action: "pushMaskBegin", mask: e, inverse: t._maskOptions.inverse, canBundle: false });
  }
  pop(e, t, n) {
    this._renderer.renderPipes.batch.break(n), n.add({ renderPipeId: "stencilMask", action: "popMaskEnd", mask: e, inverse: t._maskOptions.inverse, canBundle: false });
  }
  execute(e) {
    var _a, _b, _c, _d;
    if (e.action !== "pushMaskBegin" && e.action !== "popMaskEnd") return;
    const t = this._renderer, n = t.canvasContext, a = n == null ? void 0 : n.activeContext;
    if (!a) return;
    if (e.action === "popMaskEnd") {
      this._canvasMaskStack.pop() && a.restore();
      return;
    }
    e.inverse && this._warnOnce("inverse", "CanvasRenderer: inverse masks are not supported on Canvas2D; ignoring inverse flag.");
    const o = e.mask.mask;
    if (!(o instanceof Ue)) {
      this._warnOnce("nonGraphics", "CanvasRenderer: only Graphics masks are supported in Canvas2D; skipping mask."), this._canvasMaskStack.push(false);
      return;
    }
    const r = o, c = (_a = r.context) == null ? void 0 : _a.instructions;
    if (!(c == null ? void 0 : c.length)) {
      this._canvasMaskStack.push(false);
      return;
    }
    a.save(), n.setContextTransform(r.groupTransform, (t._roundPixels | r._roundPixels) === 1), a.beginPath();
    let d = false, u = false;
    for (let l = 0; l < c.length; l++) {
      const T = c[l], x = T.action;
      if (x !== "fill" && x !== "stroke") continue;
      const E = (_c = (_b = T.data) == null ? void 0 : _b.path) == null ? void 0 : _c.shapePath;
      if (!((_d = E == null ? void 0 : E.shapePrimitives) == null ? void 0 : _d.length)) continue;
      const H = E.shapePrimitives;
      for (let P = 0; P < H.length; P++) {
        const w = H[P];
        if (!(w == null ? void 0 : w.shape)) continue;
        const g = w.transform, I = g && !g.isIdentity();
        I && (a.save(), a.transform(g.a, g.b, g.c, g.d, g.tx, g.ty)), Se(a, w.shape), u = rt(a, w.holes) || u, d = true, I && a.restore();
      }
    }
    if (!d) {
      a.restore(), this._canvasMaskStack.push(false);
      return;
    }
    u ? a.clip("evenodd") : a.clip(), this._canvasMaskStack.push(true);
  }
  destroy() {
    this._renderer = null, this._warnedMaskTypes = null, this._canvasMaskStack = null;
  }
  _warnOnce(e, t) {
    this._warnedMaskTypes.has(e) || (this._warnedMaskTypes.add(e), qe(t));
  }
}
Me.extension = { type: [b.CanvasPipes], name: "stencilMask" };
const h = "source-over";
function it() {
  const s = Le(), e = /* @__PURE__ */ Object.create(null);
  return e.inherit = h, e.none = h, e.normal = "source-over", e.add = "lighter", e.multiply = s ? "multiply" : h, e.screen = s ? "screen" : h, e.overlay = s ? "overlay" : h, e.darken = s ? "darken" : h, e.lighten = s ? "lighten" : h, e["color-dodge"] = s ? "color-dodge" : h, e["color-burn"] = s ? "color-burn" : h, e["hard-light"] = s ? "hard-light" : h, e["soft-light"] = s ? "soft-light" : h, e.difference = s ? "difference" : h, e.exclusion = s ? "exclusion" : h, e.saturation = s ? "saturation" : h, e.color = s ? "color" : h, e.luminosity = s ? "luminosity" : h, e["linear-burn"] = s ? "color-burn" : h, e["linear-dodge"] = s ? "color-dodge" : h, e["linear-light"] = s ? "hard-light" : h, e["pin-light"] = s ? "hard-light" : h, e["vivid-light"] = s ? "hard-light" : h, e["hard-mix"] = h, e.negation = s ? "difference" : h, e["normal-npm"] = e.normal, e["add-npm"] = e.add, e["screen-npm"] = e.screen, e.erase = "destination-out", e.subtract = h, e.divide = h, e.min = h, e.max = h, e;
}
const ct = new B();
class we {
  constructor(e) {
    this.activeResolution = 1, this.smoothProperty = "imageSmoothingEnabled", this.blendModes = it(), this._activeBlendMode = "normal", this._projTransform = null, this._outerBlend = false, this._warnedBlendModes = /* @__PURE__ */ new Set(), this._renderer = e;
  }
  resolutionChange(e) {
    this.activeResolution = e;
  }
  init() {
    const e = this._renderer.background.alpha < 1;
    if (this.rootContext = this._renderer.canvas.getContext("2d", { alpha: e }), this.activeContext = this.rootContext, this.activeResolution = this._renderer.resolution, !this.rootContext.imageSmoothingEnabled) {
      const t = this.rootContext;
      t.webkitImageSmoothingEnabled ? this.smoothProperty = "webkitImageSmoothingEnabled" : t.mozImageSmoothingEnabled ? this.smoothProperty = "mozImageSmoothingEnabled" : t.oImageSmoothingEnabled ? this.smoothProperty = "oImageSmoothingEnabled" : t.msImageSmoothingEnabled && (this.smoothProperty = "msImageSmoothingEnabled");
    }
  }
  setContextTransform(e, t, n, a) {
    var _a;
    const o = a ? B.IDENTITY : ((_a = this._renderer.globalUniforms.globalUniformData) == null ? void 0 : _a.worldTransformMatrix) || B.IDENTITY;
    let r = ct;
    r.copyFrom(o), r.append(e);
    const c = this._projTransform, d = this.activeResolution;
    if (n = n || d, c) {
      const u = B.shared;
      u.copyFrom(r), u.prepend(c), r = u;
    }
    t ? this.activeContext.setTransform(r.a * n, r.b * n, r.c * n, r.d * n, r.tx * d | 0, r.ty * d | 0) : this.activeContext.setTransform(r.a * n, r.b * n, r.c * n, r.d * n, r.tx * d, r.ty * d);
  }
  clear(e, t) {
    const n = this.activeContext, a = this._renderer;
    if (n.clearRect(0, 0, a.width, a.height), e) {
      const o = Pe.shared.setValue(e);
      n.globalAlpha = t ?? o.alpha, n.fillStyle = o.toHex(), n.fillRect(0, 0, a.width, a.height), n.globalAlpha = 1;
    }
  }
  setBlendMode(e) {
    if (this._activeBlendMode === e) return;
    this._activeBlendMode = e, this._outerBlend = false;
    const t = this.blendModes[e];
    if (!t) {
      this._warnedBlendModes.has(e) || (console.warn(`CanvasRenderer: blend mode "${e}" is not supported in Canvas2D; falling back to "source-over".`), this._warnedBlendModes.add(e)), this.activeContext.globalCompositeOperation = "source-over";
      return;
    }
    this.activeContext.globalCompositeOperation = t;
  }
  destroy() {
    this.rootContext = null, this.activeContext = null, this._warnedBlendModes.clear();
  }
}
we.extension = { type: [b.CanvasSystem], name: "canvasContext" };
class Ie {
  constructor() {
    this.maxTextures = 16, this.maxBatchableTextures = 16, this.maxUniformBindings = 0;
  }
  init() {
  }
}
Ie.extension = { type: [b.CanvasSystem], name: "limits" };
const lt = "#808080", J = new B(), ht = new B(), dt = new B(), le = new B();
function pt(s, e, t) {
  s.beginPath();
  for (let n = 0; n < t.length; n += 3) {
    const a = t[n] * 2, o = t[n + 1] * 2, r = t[n + 2] * 2;
    s.moveTo(e[a], e[a + 1]), s.lineTo(e[o], e[o + 1]), s.lineTo(e[r], e[r + 1]), s.closePath();
  }
  s.fill();
}
function ut(s) {
  return `#${(s & 16777215).toString(16).padStart(6, "0")}`;
}
function mt(s, e, t, n, a, o) {
  o = Math.max(0, Math.min(o, Math.min(n, a) / 2)), s.moveTo(e + o, t), s.lineTo(e + n - o, t), s.quadraticCurveTo(e + n, t, e + n, t + o), s.lineTo(e + n, t + a - o), s.quadraticCurveTo(e + n, t + a, e + n - o, t + a), s.lineTo(e + o, t + a), s.quadraticCurveTo(e, t + a, e, t + a - o), s.lineTo(e, t + o), s.quadraticCurveTo(e, t, e + o, t);
}
function Z(s, e) {
  switch (e.type) {
    case "rectangle": {
      const t = e;
      s.rect(t.x, t.y, t.width, t.height);
      break;
    }
    case "roundedRectangle": {
      const t = e;
      mt(s, t.x, t.y, t.width, t.height, t.radius);
      break;
    }
    case "circle": {
      const t = e;
      s.arc(t.x, t.y, t.radius, 0, Math.PI * 2);
      break;
    }
    case "ellipse": {
      const t = e;
      s.ellipse ? s.ellipse(t.x, t.y, t.halfWidth, t.halfHeight, 0, 0, Math.PI * 2) : (s.save(), s.translate(t.x, t.y), s.scale(t.halfWidth, t.halfHeight), s.arc(0, 0, 1, 0, Math.PI * 2), s.restore());
      break;
    }
    case "triangle": {
      const t = e;
      s.moveTo(t.x, t.y), s.lineTo(t.x2, t.y2), s.lineTo(t.x3, t.y3), s.closePath();
      break;
    }
    case "polygon":
    default: {
      const t = e, n = t.points;
      if (!(n == null ? void 0 : n.length)) break;
      s.moveTo(n[0], n[1]);
      for (let a = 2; a < n.length; a += 2) s.lineTo(n[a], n[a + 1]);
      t.closePath && s.closePath();
      break;
    }
  }
}
function ft(s, e) {
  if (!(e == null ? void 0 : e.length)) return false;
  for (let t = 0; t < e.length; t++) {
    const n = e[t];
    if (!(n == null ? void 0 : n.shape)) continue;
    const a = n.transform, o = a && !a.isIdentity();
    o && (s.save(), s.transform(a.a, a.b, a.c, a.d, a.tx, a.ty)), Z(s, n.shape), o && s.restore();
  }
  return true;
}
function gt(s, e, t, n) {
  const a = s.fill;
  if (a instanceof je) {
    a.buildGradient();
    const r = a.texture;
    if (r) {
      const c = y.getTintedPattern(r, e), d = t ? le.copyFrom(t).scale(r.source.pixelWidth, r.source.pixelHeight) : le.copyFrom(a.transform);
      return n && !s.textureSpace && d.append(n), y.applyPatternTransform(c, d), c;
    }
  }
  if (a instanceof Ve) {
    const r = y.getTintedPattern(a.texture, e);
    return y.applyPatternTransform(r, a.transform), r;
  }
  const o = s.texture;
  if (o && o !== ke.WHITE) {
    if (!o.source.resource) return lt;
    const r = y.getTintedPattern(o, e), c = t ? le.copyFrom(t).scale(o.source.pixelWidth, o.source.pixelHeight) : s.matrix;
    return y.applyPatternTransform(r, c), r;
  }
  return ut(e);
}
class Re {
  constructor() {
    this.shader = null;
  }
  contextChange(e) {
  }
  execute(e, t) {
    var _a, _b, _c, _d, _e2, _f, _g, _h;
    const n = e.renderer, a = n.canvasContext, o = a.activeContext, r = t.groupTransform, c = ((_a = n.globalUniforms.globalUniformData) == null ? void 0 : _a.worldColor) ?? 4294967295, d = t.groupColorAlpha, u = (c >>> 24 & 255) / 255, l = (d >>> 24 & 255) / 255, T = ((_b = n.filter) == null ? void 0 : _b.alphaMultiplier) ?? 1, x = u * l * T;
    if (x <= 0) return;
    const j = c & 16777215, E = d & 16777215, H = Te(K(E, j)), P = n._roundPixels | t._roundPixels;
    o.save(), a.setContextTransform(r, P === 1), a.setBlendMode(t.groupBlendMode);
    const w = t.context.instructions;
    for (let g = 0; g < w.length; g++) {
      const I = w[g];
      if (I.action === "texture") {
        const p = I.data, m = p.image, f = m ? y.getCanvasSource(m) : null;
        if (!f) continue;
        const G = p.alpha * x;
        if (G <= 0) continue;
        const O = K(p.style, H);
        o.globalAlpha = G;
        let _ = f;
        O !== 16777215 && (_ = y.getTintedCanvas({ texture: m }, O));
        const D = m.frame, v = m.source._resolution ?? m.source.resolution ?? 1;
        let S = D.x * v, M = D.y * v;
        const $ = D.width * v, U = D.height * v;
        _ !== f && (S = 0, M = 0);
        const A = p.transform, C = A && !A.isIdentity(), i = m.rotate;
        C || i ? (J.copyFrom(r), C && J.append(A), i && Ce.matrixAppendRotationInv(J, i, p.dx, p.dy, p.dw, p.dh), a.setContextTransform(J, P === 1)) : a.setContextTransform(r, P === 1), o.drawImage(_, S, M, _ === f ? $ : _.width, _ === f ? U : _.height, i ? 0 : p.dx, i ? 0 : p.dy, p.dw, p.dh), (C || i) && a.setContextTransform(r, P === 1);
        continue;
      }
      const W = I.data, X = (_c = W == null ? void 0 : W.path) == null ? void 0 : _c.shapePath;
      if (!((_d = X == null ? void 0 : X.shapePrimitives) == null ? void 0 : _d.length)) continue;
      const k = W.style, F = K(k.color, H), R = k.alpha * x;
      if (R <= 0) continue;
      const V = I.action === "stroke";
      if (o.globalAlpha = R, V) {
        const p = k;
        o.lineWidth = p.width, o.lineCap = p.cap, o.lineJoin = p.join, o.miterLimit = p.miterLimit;
      }
      const L = X.shapePrimitives;
      if (!V && ((_g = (_f = (_e2 = W.hole) == null ? void 0 : _e2.shapePath) == null ? void 0 : _f.shapePrimitives) == null ? void 0 : _g.length)) {
        const p = L[L.length - 1];
        p.holes = W.hole.shapePath.shapePrimitives;
      }
      for (let p = 0; p < L.length; p++) {
        const m = L[p];
        if (!(m == null ? void 0 : m.shape)) continue;
        const f = m.transform, G = f && !f.isIdentity(), O = k.texture && k.texture !== ke.WHITE, _ = k.textureSpace === "global" ? f : null, D = O ? Oe(ht, k, m.shape, _) : null, v = G ? dt.copyFrom(r).append(f) : r, S = gt(k, F, D, v);
        if (G && (o.save(), o.transform(f.a, f.b, f.c, f.d, f.tx, f.ty)), V) {
          const M = k;
          if (M.alignment !== 0.5 && !M.pixelLine) {
            const U = [], A = [], C = [];
            if ((_h = Ne[m.shape.type]) == null ? void 0 : _h.build(m.shape, U)) {
              const te = m.shape.closePath ?? true;
              Ye(U, M, false, te, A, C), o.fillStyle = S, pt(o, A, C);
            } else o.strokeStyle = S, o.beginPath(), Z(o, m.shape), o.stroke();
          } else o.strokeStyle = S, o.beginPath(), Z(o, m.shape), o.stroke();
        } else o.fillStyle = S, o.beginPath(), Z(o, m.shape), ft(o, m.holes) ? o.fill("evenodd") : o.fill();
        G && o.restore();
      }
    }
    o.restore();
  }
  destroy() {
    this.shader = null;
  }
}
Re.extension = { type: [b.CanvasPipesAdaptor], name: "graphics" };
class vt {
  init(e, t) {
    this._renderer = e, this._renderTargetSystem = t;
  }
  initGpuRenderTarget(e) {
    const t = e.colorTexture, { canvas: n, context: a } = this._ensureCanvas(t);
    return { canvas: n, context: a, width: n.width, height: n.height };
  }
  resizeGpuRenderTarget(e) {
    const t = e.colorTexture, { canvas: n } = this._ensureCanvas(t);
    n.width = e.pixelWidth, n.height = e.pixelHeight;
  }
  startRenderPass(e, t, n, a) {
    const o = this._renderTargetSystem.getGpuRenderTarget(e);
    this._renderer.canvasContext.activeContext = o.context, this._renderer.canvasContext.activeResolution = e.resolution, t && this.clear(e, t, n, a);
  }
  clear(e, t, n, a) {
    const r = this._renderTargetSystem.getGpuRenderTarget(e).context, c = a || { x: 0, y: 0, width: e.pixelWidth, height: e.pixelHeight };
    if (r.setTransform(1, 0, 0, 1, 0, 0), r.clearRect(c.x, c.y, c.width, c.height), n) {
      const d = Pe.shared.setValue(n);
      d.alpha > 0 && (r.globalAlpha = d.alpha, r.fillStyle = d.toHex(), r.fillRect(c.x, c.y, c.width, c.height), r.globalAlpha = 1);
    }
  }
  finishRenderPass() {
  }
  copyToTexture(e, t, n, a, o) {
    const c = this._renderTargetSystem.getGpuRenderTarget(e).canvas, d = t.source, { context: u } = this._ensureCanvas(d), l = (o == null ? void 0 : o.x) ?? 0, T = (o == null ? void 0 : o.y) ?? 0;
    return u.drawImage(c, n.x, n.y, a.width, a.height, l, T, a.width, a.height), d.update(), t;
  }
  destroyGpuRenderTarget(e) {
  }
  _ensureCanvas(e) {
    let t = e.resource;
    (!t || !$e.test(t)) && (t = _e.get().createCanvas(e.pixelWidth, e.pixelHeight), e.resource = t), (t.width !== e.pixelWidth || t.height !== e.pixelHeight) && (t.width = e.pixelWidth, t.height = e.pixelHeight);
    const n = t.getContext("2d");
    return { canvas: t, context: n };
  }
}
class Ae extends ze {
  constructor(e) {
    super(e), this.adaptor = new vt(), this.adaptor.init(e, this);
  }
}
Ae.extension = { type: [b.CanvasSystem], name: "renderTarget" };
class Be {
  constructor(e) {
  }
  init() {
  }
  initSource(e) {
  }
  generateCanvas(e) {
    const t = _e.get().createCanvas(), n = t.getContext("2d"), a = y.getCanvasSource(e);
    if (!a) return t;
    const o = e.frame, r = e.source._resolution ?? e.source.resolution ?? 1, c = o.x * r, d = o.y * r, u = o.width * r, l = o.height * r;
    return t.width = Math.ceil(u), t.height = Math.ceil(l), n.drawImage(a, c, d, u, l, 0, 0, u, l), t;
  }
  getPixels(e) {
    const t = this.generateCanvas(e);
    return { pixels: t.getContext("2d", { willReadFrequently: true }).getImageData(0, 0, t.width, t.height).data, width: t.width, height: t.height };
  }
  destroy() {
  }
}
Be.extension = { type: [b.CanvasSystem], name: "texture" };
const yt = [...Je, we, Ie, Be, Ae], xt = [Ke, Ze, et, tt, st, Me, be, nt], Ct = [at, Re], He = [], Ge = [], De = [];
ee.handleByNamedList(b.CanvasSystem, He);
ee.handleByNamedList(b.CanvasPipes, Ge);
ee.handleByNamedList(b.CanvasPipesAdaptor, De);
ee.add(...yt, ...xt, ...Ct);
class bt extends Xe {
  constructor() {
    const e = { name: "canvas", type: Qe.CANVAS, systems: He, renderPipes: Ge, renderPipeAdaptors: De };
    super(e);
  }
}
export {
  bt as CanvasRenderer
};
