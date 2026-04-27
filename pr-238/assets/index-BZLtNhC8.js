const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/browserAll-7EbimO6Z.js","assets/webworkerAll-PmF0G_P5.js","assets/Filter-xCwzRZN3.js","assets/WebGPURenderer-DZwUSLU_.js","assets/BufferResource-3oR7nhl5.js","assets/RenderTargetSystem-B3psJFHM.js","assets/WebGLRenderer-wnOXY49J.js","assets/CanvasRenderer-CIpnLXhq.js"])))=>i.map(i=>d[i]);
let Iu, Os, jn, rn, Bc, Qe, it, ti, ut, fs, Dg, Bm, Ki, _t, Tm, fn, Ji, jc, Nt, Xi, Ct, Mt, uh, so, Ot, Nn, Hs, At, Oi, zt, If, _s, ja, Ii, Bo, ch, ke, Fo, am, lm, hm, ym, mm, xm, cm, gm, pm, yu, bm, _e, Lc, Se, ji, Ea, De, by, Xu, Pa, fr, Ia, qu, Oc, Zi, Gh, ph, Qs, qp, Up, Sh, Ff, Oo, Hn, Et, yo, xo, yi, Gt, Le, Uh, jh, Pe, ww, ly, qg, te, St, zs, Bi, $f, Tp, qt, Vt, re, Kt, jt;
let __tla = (async () => {
  (function() {
    const t = document.createElement("link").relList;
    if (t && t.supports && t.supports("modulepreload")) return;
    for (const i of document.querySelectorAll('link[rel="modulepreload"]')) s(i);
    new MutationObserver((i) => {
      for (const r of i) if (r.type === "childList") for (const o of r.addedNodes) o.tagName === "LINK" && o.rel === "modulepreload" && s(o);
    }).observe(document, {
      childList: true,
      subtree: true
    });
    function e(i) {
      const r = {};
      return i.integrity && (r.integrity = i.integrity), i.referrerPolicy && (r.referrerPolicy = i.referrerPolicy), i.crossOrigin === "use-credentials" ? r.credentials = "include" : i.crossOrigin === "anonymous" ? r.credentials = "omit" : r.credentials = "same-origin", r;
    }
    function s(i) {
      if (i.ep) return;
      i.ep = true;
      const r = e(i);
      fetch(i.href, r);
    }
  })();
  const su = "modulepreload", iu = function(n) {
    return "/fucktorio/pr-238/" + n;
  }, pa = {}, Gn = function(t, e, s) {
    let i = Promise.resolve();
    if (e && e.length > 0) {
      let o = function(c) {
        return Promise.all(c.map((h) => Promise.resolve(h).then((d) => ({
          status: "fulfilled",
          value: d
        }), (d) => ({
          status: "rejected",
          reason: d
        }))));
      };
      document.getElementsByTagName("link");
      const a = document.querySelector("meta[property=csp-nonce]"), l = (a == null ? void 0 : a.nonce) || (a == null ? void 0 : a.getAttribute("nonce"));
      i = o(e.map((c) => {
        if (c = iu(c), c in pa) return;
        pa[c] = true;
        const h = c.endsWith(".css"), d = h ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${c}"]${d}`)) return;
        const u = document.createElement("link");
        if (u.rel = h ? "stylesheet" : su, h || (u.as = "script"), u.crossOrigin = "", u.href = c, l && u.setAttribute("nonce", l), document.head.appendChild(u), h) return new Promise((p, f) => {
          u.addEventListener("load", p), u.addEventListener("error", () => f(new Error(`Unable to preload CSS for ${c}`)));
        });
      }));
    }
    function r(o) {
      const a = new Event("vite:preloadError", {
        cancelable: true
      });
      if (a.payload = o, window.dispatchEvent(a), !a.defaultPrevented) throw o;
    }
    return i.then((o) => {
      for (const a of o || []) a.status === "rejected" && r(a.reason);
      return t().catch(r);
    });
  };
  it = ((n) => (n.Application = "application", n.WebGLPipes = "webgl-pipes", n.WebGLPipesAdaptor = "webgl-pipes-adaptor", n.WebGLSystem = "webgl-system", n.WebGPUPipes = "webgpu-pipes", n.WebGPUPipesAdaptor = "webgpu-pipes-adaptor", n.WebGPUSystem = "webgpu-system", n.CanvasSystem = "canvas-system", n.CanvasPipesAdaptor = "canvas-pipes-adaptor", n.CanvasPipes = "canvas-pipes", n.Asset = "asset", n.LoadParser = "load-parser", n.ResolveParser = "resolve-parser", n.CacheParser = "cache-parser", n.DetectionParser = "detection-parser", n.MaskEffect = "mask-effect", n.BlendMode = "blend-mode", n.TextureSource = "texture-source", n.Environment = "environment", n.ShapeBuilder = "shape-builder", n.Batcher = "batcher", n))(it || {});
  let Xr, si, ru, ou;
  Xr = (n) => {
    if (typeof n == "function" || typeof n == "object" && n.extension) {
      if (!n.extension) throw new Error("Extension class must have an extension object");
      n = {
        ...typeof n.extension != "object" ? {
          type: n.extension
        } : n.extension,
        ref: n
      };
    }
    if (typeof n == "object") n = {
      ...n
    };
    else throw new Error("Invalid extension type");
    return typeof n.type == "string" && (n.type = [
      n.type
    ]), n;
  };
  si = (n, t) => Xr(n).priority ?? t;
  Ot = {
    _addHandlers: {},
    _removeHandlers: {},
    _queue: {},
    remove(...n) {
      return n.map(Xr).forEach((t) => {
        t.type.forEach((e) => {
          var _a2, _b2;
          return (_b2 = (_a2 = this._removeHandlers)[e]) == null ? void 0 : _b2.call(_a2, t);
        });
      }), this;
    },
    add(...n) {
      return n.map(Xr).forEach((t) => {
        t.type.forEach((e) => {
          var _a2, _b2;
          const s = this._addHandlers, i = this._queue;
          s[e] ? (_a2 = s[e]) == null ? void 0 : _a2.call(s, t) : (i[e] = i[e] || [], (_b2 = i[e]) == null ? void 0 : _b2.push(t));
        });
      }), this;
    },
    handle(n, t, e) {
      var _a2;
      const s = this._addHandlers, i = this._removeHandlers;
      if (s[n] || i[n]) throw new Error(`Extension type ${n} already has a handler`);
      s[n] = t, i[n] = e;
      const r = this._queue;
      return r[n] && ((_a2 = r[n]) == null ? void 0 : _a2.forEach((o) => t(o)), delete r[n]), this;
    },
    handleByMap(n, t) {
      return this.handle(n, (e) => {
        e.name && (t[e.name] = e.ref);
      }, (e) => {
        e.name && delete t[e.name];
      });
    },
    handleByNamedList(n, t, e = -1) {
      return this.handle(n, (s) => {
        t.findIndex((r) => r.name === s.name) >= 0 || (t.push({
          name: s.name,
          value: s.ref
        }), t.sort((r, o) => si(o.value, e) - si(r.value, e)));
      }, (s) => {
        const i = t.findIndex((r) => r.name === s.name);
        i !== -1 && t.splice(i, 1);
      });
    },
    handleByList(n, t, e = -1) {
      return this.handle(n, (s) => {
        t.includes(s.ref) || (t.push(s.ref), t.sort((i, r) => si(r, e) - si(i, e)));
      }, (s) => {
        const i = t.indexOf(s.ref);
        i !== -1 && t.splice(i, 1);
      });
    },
    mixin(n, ...t) {
      for (const e of t) Object.defineProperties(n.prototype, Object.getOwnPropertyDescriptors(e));
    }
  };
  ru = {
    extension: {
      type: it.Environment,
      name: "browser",
      priority: -1
    },
    test: () => true,
    load: async () => {
      await Gn(() => import("./browserAll-7EbimO6Z.js"), __vite__mapDeps([0,1,2]));
    }
  };
  ou = {
    extension: {
      type: it.Environment,
      name: "webworker",
      priority: 0
    },
    test: () => typeof self < "u" && self.WorkerGlobalScope !== void 0,
    load: async () => {
      await Gn(() => import("./webworkerAll-PmF0G_P5.js"), __vite__mapDeps([1,2]));
    }
  };
  class le {
    constructor(t, e, s) {
      this._x = e || 0, this._y = s || 0, this._observer = t;
    }
    clone(t) {
      return new le(t ?? this._observer, this._x, this._y);
    }
    set(t = 0, e = t) {
      return (this._x !== t || this._y !== e) && (this._x = t, this._y = e, this._observer._onUpdate(this)), this;
    }
    copyFrom(t) {
      return (this._x !== t.x || this._y !== t.y) && (this._x = t.x, this._y = t.y, this._observer._onUpdate(this)), this;
    }
    copyTo(t) {
      return t.set(this._x, this._y), t;
    }
    equals(t) {
      return t.x === this._x && t.y === this._y;
    }
    toString() {
      return `[pixi.js/math:ObservablePoint x=${this._x} y=${this._y} scope=${this._observer}]`;
    }
    get x() {
      return this._x;
    }
    set x(t) {
      this._x !== t && (this._x = t, this._observer._onUpdate(this));
    }
    get y() {
      return this._y;
    }
    set y(t) {
      this._y !== t && (this._y = t, this._observer._onUpdate(this));
    }
  }
  function wc(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var rr = {
    exports: {}
  }, fa;
  function au() {
    return fa || (fa = 1, (function(n) {
      var t = Object.prototype.hasOwnProperty, e = "~";
      function s() {
      }
      Object.create && (s.prototype = /* @__PURE__ */ Object.create(null), new s().__proto__ || (e = false));
      function i(l, c, h) {
        this.fn = l, this.context = c, this.once = h || false;
      }
      function r(l, c, h, d, u) {
        if (typeof h != "function") throw new TypeError("The listener must be a function");
        var p = new i(h, d || l, u), f = e ? e + c : c;
        return l._events[f] ? l._events[f].fn ? l._events[f] = [
          l._events[f],
          p
        ] : l._events[f].push(p) : (l._events[f] = p, l._eventsCount++), l;
      }
      function o(l, c) {
        --l._eventsCount === 0 ? l._events = new s() : delete l._events[c];
      }
      function a() {
        this._events = new s(), this._eventsCount = 0;
      }
      a.prototype.eventNames = function() {
        var c = [], h, d;
        if (this._eventsCount === 0) return c;
        for (d in h = this._events) t.call(h, d) && c.push(e ? d.slice(1) : d);
        return Object.getOwnPropertySymbols ? c.concat(Object.getOwnPropertySymbols(h)) : c;
      }, a.prototype.listeners = function(c) {
        var h = e ? e + c : c, d = this._events[h];
        if (!d) return [];
        if (d.fn) return [
          d.fn
        ];
        for (var u = 0, p = d.length, f = new Array(p); u < p; u++) f[u] = d[u].fn;
        return f;
      }, a.prototype.listenerCount = function(c) {
        var h = e ? e + c : c, d = this._events[h];
        return d ? d.fn ? 1 : d.length : 0;
      }, a.prototype.emit = function(c, h, d, u, p, f) {
        var g = e ? e + c : c;
        if (!this._events[g]) return false;
        var m = this._events[g], y = arguments.length, b, x;
        if (m.fn) {
          switch (m.once && this.removeListener(c, m.fn, void 0, true), y) {
            case 1:
              return m.fn.call(m.context), true;
            case 2:
              return m.fn.call(m.context, h), true;
            case 3:
              return m.fn.call(m.context, h, d), true;
            case 4:
              return m.fn.call(m.context, h, d, u), true;
            case 5:
              return m.fn.call(m.context, h, d, u, p), true;
            case 6:
              return m.fn.call(m.context, h, d, u, p, f), true;
          }
          for (x = 1, b = new Array(y - 1); x < y; x++) b[x - 1] = arguments[x];
          m.fn.apply(m.context, b);
        } else {
          var _ = m.length, v;
          for (x = 0; x < _; x++) switch (m[x].once && this.removeListener(c, m[x].fn, void 0, true), y) {
            case 1:
              m[x].fn.call(m[x].context);
              break;
            case 2:
              m[x].fn.call(m[x].context, h);
              break;
            case 3:
              m[x].fn.call(m[x].context, h, d);
              break;
            case 4:
              m[x].fn.call(m[x].context, h, d, u);
              break;
            default:
              if (!b) for (v = 1, b = new Array(y - 1); v < y; v++) b[v - 1] = arguments[v];
              m[x].fn.apply(m[x].context, b);
          }
        }
        return true;
      }, a.prototype.on = function(c, h, d) {
        return r(this, c, h, d, false);
      }, a.prototype.once = function(c, h, d) {
        return r(this, c, h, d, true);
      }, a.prototype.removeListener = function(c, h, d, u) {
        var p = e ? e + c : c;
        if (!this._events[p]) return this;
        if (!h) return o(this, p), this;
        var f = this._events[p];
        if (f.fn) f.fn === h && (!u || f.once) && (!d || f.context === d) && o(this, p);
        else {
          for (var g = 0, m = [], y = f.length; g < y; g++) (f[g].fn !== h || u && !f[g].once || d && f[g].context !== d) && m.push(f[g]);
          m.length ? this._events[p] = m.length === 1 ? m[0] : m : o(this, p);
        }
        return this;
      }, a.prototype.removeAllListeners = function(c) {
        var h;
        return c ? (h = e ? e + c : c, this._events[h] && o(this, h)) : (this._events = new s(), this._eventsCount = 0), this;
      }, a.prototype.off = a.prototype.removeListener, a.prototype.addListener = a.prototype.on, a.prefixed = e, a.EventEmitter = a, n.exports = a;
    })(rr)), rr.exports;
  }
  var lu = au();
  let cu, hu, du;
  Qe = wc(lu);
  cu = Math.PI * 2;
  hu = 180 / Math.PI;
  du = Math.PI / 180;
  At = class {
    constructor(t = 0, e = 0) {
      this.x = 0, this.y = 0, this.x = t, this.y = e;
    }
    clone() {
      return new At(this.x, this.y);
    }
    copyFrom(t) {
      return this.set(t.x, t.y), this;
    }
    copyTo(t) {
      return t.set(this.x, this.y), t;
    }
    equals(t) {
      return t.x === this.x && t.y === this.y;
    }
    set(t = 0, e = t) {
      return this.x = t, this.y = e, this;
    }
    toString() {
      return `[pixi.js/math:Point x=${this.x} y=${this.y}]`;
    }
    static get shared() {
      return or.x = 0, or.y = 0, or;
    }
  };
  const or = new At();
  _t = class {
    constructor(t = 1, e = 0, s = 0, i = 1, r = 0, o = 0) {
      this.array = null, this.a = t, this.b = e, this.c = s, this.d = i, this.tx = r, this.ty = o;
    }
    fromArray(t) {
      this.a = t[0], this.b = t[1], this.c = t[3], this.d = t[4], this.tx = t[2], this.ty = t[5];
    }
    set(t, e, s, i, r, o) {
      return this.a = t, this.b = e, this.c = s, this.d = i, this.tx = r, this.ty = o, this;
    }
    toArray(t, e) {
      this.array || (this.array = new Float32Array(9));
      const s = e || this.array;
      return t ? (s[0] = this.a, s[1] = this.b, s[2] = 0, s[3] = this.c, s[4] = this.d, s[5] = 0, s[6] = this.tx, s[7] = this.ty, s[8] = 1) : (s[0] = this.a, s[1] = this.c, s[2] = this.tx, s[3] = this.b, s[4] = this.d, s[5] = this.ty, s[6] = 0, s[7] = 0, s[8] = 1), s;
    }
    apply(t, e) {
      e = e || new At();
      const s = t.x, i = t.y;
      return e.x = this.a * s + this.c * i + this.tx, e.y = this.b * s + this.d * i + this.ty, e;
    }
    applyInverse(t, e) {
      e = e || new At();
      const s = this.a, i = this.b, r = this.c, o = this.d, a = this.tx, l = this.ty, c = 1 / (s * o + r * -i), h = t.x, d = t.y;
      return e.x = o * c * h + -r * c * d + (l * r - a * o) * c, e.y = s * c * d + -i * c * h + (-l * s + a * i) * c, e;
    }
    translate(t, e) {
      return this.tx += t, this.ty += e, this;
    }
    scale(t, e) {
      return this.a *= t, this.d *= e, this.c *= t, this.b *= e, this.tx *= t, this.ty *= e, this;
    }
    rotate(t) {
      const e = Math.cos(t), s = Math.sin(t), i = this.a, r = this.c, o = this.tx;
      return this.a = i * e - this.b * s, this.b = i * s + this.b * e, this.c = r * e - this.d * s, this.d = r * s + this.d * e, this.tx = o * e - this.ty * s, this.ty = o * s + this.ty * e, this;
    }
    append(t) {
      const e = this.a, s = this.b, i = this.c, r = this.d;
      return this.a = t.a * e + t.b * i, this.b = t.a * s + t.b * r, this.c = t.c * e + t.d * i, this.d = t.c * s + t.d * r, this.tx = t.tx * e + t.ty * i + this.tx, this.ty = t.tx * s + t.ty * r + this.ty, this;
    }
    appendFrom(t, e) {
      const s = t.a, i = t.b, r = t.c, o = t.d, a = t.tx, l = t.ty, c = e.a, h = e.b, d = e.c, u = e.d;
      return this.a = s * c + i * d, this.b = s * h + i * u, this.c = r * c + o * d, this.d = r * h + o * u, this.tx = a * c + l * d + e.tx, this.ty = a * h + l * u + e.ty, this;
    }
    setTransform(t, e, s, i, r, o, a, l, c) {
      return this.a = Math.cos(a + c) * r, this.b = Math.sin(a + c) * r, this.c = -Math.sin(a - l) * o, this.d = Math.cos(a - l) * o, this.tx = t - (s * this.a + i * this.c), this.ty = e - (s * this.b + i * this.d), this;
    }
    prepend(t) {
      const e = this.tx;
      if (t.a !== 1 || t.b !== 0 || t.c !== 0 || t.d !== 1) {
        const s = this.a, i = this.c;
        this.a = s * t.a + this.b * t.c, this.b = s * t.b + this.b * t.d, this.c = i * t.a + this.d * t.c, this.d = i * t.b + this.d * t.d;
      }
      return this.tx = e * t.a + this.ty * t.c + t.tx, this.ty = e * t.b + this.ty * t.d + t.ty, this;
    }
    decompose(t) {
      const e = this.a, s = this.b, i = this.c, r = this.d, o = t.pivot, a = -Math.atan2(-i, r), l = Math.atan2(s, e), c = Math.abs(a + l);
      return c < 1e-5 || Math.abs(cu - c) < 1e-5 ? (t.rotation = l, t.skew.x = t.skew.y = 0) : (t.rotation = 0, t.skew.x = a, t.skew.y = l), t.scale.x = Math.sqrt(e * e + s * s), t.scale.y = Math.sqrt(i * i + r * r), t.position.x = this.tx + (o.x * e + o.y * i), t.position.y = this.ty + (o.x * s + o.y * r), t;
    }
    invert() {
      const t = this.a, e = this.b, s = this.c, i = this.d, r = this.tx, o = t * i - e * s;
      return this.a = i / o, this.b = -e / o, this.c = -s / o, this.d = t / o, this.tx = (s * this.ty - i * r) / o, this.ty = -(t * this.ty - e * r) / o, this;
    }
    isIdentity() {
      return this.a === 1 && this.b === 0 && this.c === 0 && this.d === 1 && this.tx === 0 && this.ty === 0;
    }
    identity() {
      return this.a = 1, this.b = 0, this.c = 0, this.d = 1, this.tx = 0, this.ty = 0, this;
    }
    clone() {
      const t = new _t();
      return t.a = this.a, t.b = this.b, t.c = this.c, t.d = this.d, t.tx = this.tx, t.ty = this.ty, t;
    }
    copyTo(t) {
      return t.a = this.a, t.b = this.b, t.c = this.c, t.d = this.d, t.tx = this.tx, t.ty = this.ty, t;
    }
    copyFrom(t) {
      return this.a = t.a, this.b = t.b, this.c = t.c, this.d = t.d, this.tx = t.tx, this.ty = t.ty, this;
    }
    equals(t) {
      return t.a === this.a && t.b === this.b && t.c === this.c && t.d === this.d && t.tx === this.tx && t.ty === this.ty;
    }
    toString() {
      return `[pixi.js:Matrix a=${this.a} b=${this.b} c=${this.c} d=${this.d} tx=${this.tx} ty=${this.ty}]`;
    }
    static get IDENTITY() {
      return pu.identity();
    }
    static get shared() {
      return uu.identity();
    }
  };
  const uu = new _t(), pu = new _t(), Rn = [
    1,
    1,
    0,
    -1,
    -1,
    -1,
    0,
    1,
    1,
    1,
    0,
    -1,
    -1,
    -1,
    0,
    1
  ], Ln = [
    0,
    1,
    1,
    1,
    0,
    -1,
    -1,
    -1,
    0,
    1,
    1,
    1,
    0,
    -1,
    -1,
    -1
  ], $n = [
    0,
    -1,
    -1,
    -1,
    0,
    1,
    1,
    1,
    0,
    1,
    1,
    1,
    0,
    -1,
    -1,
    -1
  ], Bn = [
    1,
    1,
    0,
    -1,
    -1,
    -1,
    0,
    1,
    -1,
    -1,
    0,
    1,
    1,
    1,
    0,
    -1
  ], qr = [], vc = [], ii = Math.sign;
  function fu() {
    for (let n = 0; n < 16; n++) {
      const t = [];
      qr.push(t);
      for (let e = 0; e < 16; e++) {
        const s = ii(Rn[n] * Rn[e] + $n[n] * Ln[e]), i = ii(Ln[n] * Rn[e] + Bn[n] * Ln[e]), r = ii(Rn[n] * $n[e] + $n[n] * Bn[e]), o = ii(Ln[n] * $n[e] + Bn[n] * Bn[e]);
        for (let a = 0; a < 16; a++) if (Rn[a] === s && Ln[a] === i && $n[a] === r && Bn[a] === o) {
          t.push(a);
          break;
        }
      }
    }
    for (let n = 0; n < 16; n++) {
      const t = new _t();
      t.set(Rn[n], Ln[n], $n[n], Bn[n], 0, 0), vc.push(t);
    }
  }
  fu();
  let ri;
  St = {
    E: 0,
    SE: 1,
    S: 2,
    SW: 3,
    W: 4,
    NW: 5,
    N: 6,
    NE: 7,
    MIRROR_VERTICAL: 8,
    MAIN_DIAGONAL: 10,
    MIRROR_HORIZONTAL: 12,
    REVERSE_DIAGONAL: 14,
    uX: (n) => Rn[n],
    uY: (n) => Ln[n],
    vX: (n) => $n[n],
    vY: (n) => Bn[n],
    inv: (n) => n & 8 ? n & 15 : -n & 7,
    add: (n, t) => qr[n][t],
    sub: (n, t) => qr[n][St.inv(t)],
    rotate180: (n) => n ^ 4,
    isVertical: (n) => (n & 3) === 2,
    byDirection: (n, t) => Math.abs(n) * 2 <= Math.abs(t) ? t >= 0 ? St.S : St.N : Math.abs(t) * 2 <= Math.abs(n) ? n > 0 ? St.E : St.W : t > 0 ? n > 0 ? St.SE : St.SW : n > 0 ? St.NE : St.NW,
    matrixAppendRotationInv: (n, t, e = 0, s = 0, i = 0, r = 0) => {
      const o = vc[St.inv(t)], a = o.a, l = o.b, c = o.c, h = o.d, d = e - Math.min(0, a * i, c * r, a * i + c * r), u = s - Math.min(0, l * i, h * r, l * i + h * r), p = n.a, f = n.b, g = n.c, m = n.d;
      n.a = a * p + l * g, n.b = a * f + l * m, n.c = c * p + h * g, n.d = c * f + h * m, n.tx = d * p + u * g + n.tx, n.ty = d * f + u * m + n.ty;
    },
    transformRectCoords: (n, t, e, s) => {
      const { x: i, y: r, width: o, height: a } = n, { x: l, y: c, width: h, height: d } = t;
      return e === St.E ? (s.set(i + l, r + c, o, a), s) : e === St.S ? s.set(h - r - a + l, i + c, a, o) : e === St.W ? s.set(h - i - o + l, d - r - a + c, o, a) : e === St.N ? s.set(r + l, d - i - o + c, a, o) : s.set(i + l, r + c, o, a);
    }
  };
  ri = [
    new At(),
    new At(),
    new At(),
    new At()
  ];
  Nt = class {
    constructor(t = 0, e = 0, s = 0, i = 0) {
      this.type = "rectangle", this.x = Number(t), this.y = Number(e), this.width = Number(s), this.height = Number(i);
    }
    get left() {
      return this.x;
    }
    get right() {
      return this.x + this.width;
    }
    get top() {
      return this.y;
    }
    get bottom() {
      return this.y + this.height;
    }
    isEmpty() {
      return this.left === this.right || this.top === this.bottom;
    }
    static get EMPTY() {
      return new Nt(0, 0, 0, 0);
    }
    clone() {
      return new Nt(this.x, this.y, this.width, this.height);
    }
    copyFromBounds(t) {
      return this.x = t.minX, this.y = t.minY, this.width = t.maxX - t.minX, this.height = t.maxY - t.minY, this;
    }
    copyFrom(t) {
      return this.x = t.x, this.y = t.y, this.width = t.width, this.height = t.height, this;
    }
    copyTo(t) {
      return t.copyFrom(this), t;
    }
    contains(t, e) {
      return this.width <= 0 || this.height <= 0 ? false : t >= this.x && t < this.x + this.width && e >= this.y && e < this.y + this.height;
    }
    strokeContains(t, e, s, i = 0.5) {
      const { width: r, height: o } = this;
      if (r <= 0 || o <= 0) return false;
      const a = this.x, l = this.y, c = s * (1 - i), h = s - c, d = a - c, u = a + r + c, p = l - c, f = l + o + c, g = a + h, m = a + r - h, y = l + h, b = l + o - h;
      return t >= d && t <= u && e >= p && e <= f && !(t > g && t < m && e > y && e < b);
    }
    intersects(t, e) {
      if (!e) {
        const E = this.x < t.x ? t.x : this.x;
        if ((this.right > t.right ? t.right : this.right) <= E) return false;
        const T = this.y < t.y ? t.y : this.y;
        return (this.bottom > t.bottom ? t.bottom : this.bottom) > T;
      }
      const s = this.left, i = this.right, r = this.top, o = this.bottom;
      if (i <= s || o <= r) return false;
      const a = ri[0].set(t.left, t.top), l = ri[1].set(t.left, t.bottom), c = ri[2].set(t.right, t.top), h = ri[3].set(t.right, t.bottom);
      if (c.x <= a.x || l.y <= a.y) return false;
      const d = Math.sign(e.a * e.d - e.b * e.c);
      if (d === 0 || (e.apply(a, a), e.apply(l, l), e.apply(c, c), e.apply(h, h), Math.max(a.x, l.x, c.x, h.x) <= s || Math.min(a.x, l.x, c.x, h.x) >= i || Math.max(a.y, l.y, c.y, h.y) <= r || Math.min(a.y, l.y, c.y, h.y) >= o)) return false;
      const u = d * (l.y - a.y), p = d * (a.x - l.x), f = u * s + p * r, g = u * i + p * r, m = u * s + p * o, y = u * i + p * o;
      if (Math.max(f, g, m, y) <= u * a.x + p * a.y || Math.min(f, g, m, y) >= u * h.x + p * h.y) return false;
      const b = d * (a.y - c.y), x = d * (c.x - a.x), _ = b * s + x * r, v = b * i + x * r, w = b * s + x * o, C = b * i + x * o;
      return !(Math.max(_, v, w, C) <= b * a.x + x * a.y || Math.min(_, v, w, C) >= b * h.x + x * h.y);
    }
    pad(t = 0, e = t) {
      return this.x -= t, this.y -= e, this.width += t * 2, this.height += e * 2, this;
    }
    fit(t) {
      const e = Math.max(this.x, t.x), s = Math.min(this.x + this.width, t.x + t.width), i = Math.max(this.y, t.y), r = Math.min(this.y + this.height, t.y + t.height);
      return this.x = e, this.width = Math.max(s - e, 0), this.y = i, this.height = Math.max(r - i, 0), this;
    }
    ceil(t = 1, e = 1e-3) {
      const s = Math.ceil((this.x + this.width - e) * t) / t, i = Math.ceil((this.y + this.height - e) * t) / t;
      return this.x = Math.floor((this.x + e) * t) / t, this.y = Math.floor((this.y + e) * t) / t, this.width = s - this.x, this.height = i - this.y, this;
    }
    scale(t, e = t) {
      return this.x *= t, this.y *= e, this.width *= t, this.height *= e, this;
    }
    enlarge(t) {
      const e = Math.min(this.x, t.x), s = Math.max(this.x + this.width, t.x + t.width), i = Math.min(this.y, t.y), r = Math.max(this.y + this.height, t.y + t.height);
      return this.x = e, this.width = s - e, this.y = i, this.height = r - i, this;
    }
    getBounds(t) {
      return t || (t = new Nt()), t.copyFrom(this), t;
    }
    containsRect(t) {
      if (this.width <= 0 || this.height <= 0) return false;
      const e = t.x, s = t.y, i = t.x + t.width, r = t.y + t.height;
      return e >= this.x && e < this.x + this.width && s >= this.y && s < this.y + this.height && i >= this.x && i < this.x + this.width && r >= this.y && r < this.y + this.height;
    }
    set(t, e, s, i) {
      return this.x = t, this.y = e, this.width = s, this.height = i, this;
    }
    toString() {
      return `[pixi.js/math:Rectangle x=${this.x} y=${this.y} width=${this.width} height=${this.height}]`;
    }
  };
  const ar = {
    default: -1
  };
  Kt = function(n = "default") {
    return ar[n] === void 0 && (ar[n] = -1), ++ar[n];
  };
  let ma, mu, rs;
  ma = /* @__PURE__ */ new Set();
  qt = "8.0.0";
  mu = "8.3.4";
  rs = {
    quiet: false,
    noColor: false
  };
  Et = ((n, t, e = 3) => {
    if (rs.quiet || ma.has(t)) return;
    let s = new Error().stack;
    const i = `${t}
Deprecated since v${n}`, r = typeof console.groupCollapsed == "function" && !rs.noColor;
    typeof s > "u" ? console.warn("PixiJS Deprecation Warning: ", i) : (s = s.split(`
`).splice(e).join(`
`), r ? (console.groupCollapsed("%cPixiJS Deprecation Warning: %c%s", "color:#614108;background:#fffbe6", "font-weight:normal;color:#614108;background:#fffbe6", i), console.warn(s), console.groupEnd()) : (console.warn("PixiJS Deprecation Warning: ", i), console.warn(s))), ma.add(t);
  });
  Object.defineProperties(Et, {
    quiet: {
      get: () => rs.quiet,
      set: (n) => {
        rs.quiet = n;
      },
      enumerable: true,
      configurable: false
    },
    noColor: {
      get: () => rs.noColor,
      set: (n) => {
        rs.noColor = n;
      },
      enumerable: true,
      configurable: false
    }
  });
  const Cc = () => {
  };
  function us(n) {
    return n += n === 0 ? 1 : 0, --n, n |= n >>> 1, n |= n >>> 2, n |= n >>> 4, n |= n >>> 8, n |= n >>> 16, n + 1;
  }
  function ga(n) {
    return !(n & n - 1) && !!n;
  }
  function Sc(n) {
    const t = {};
    for (const e in n) n[e] !== void 0 && (t[e] = n[e]);
    return t;
  }
  const ya = /* @__PURE__ */ Object.create(null);
  function gu(n) {
    const t = ya[n];
    return t === void 0 && (ya[n] = Kt("resource")), t;
  }
  const kc = class Tc extends Qe {
    constructor(t = {}) {
      super(), this._resourceType = "textureSampler", this._touched = 0, this._maxAnisotropy = 1, this.destroyed = false, t = {
        ...Tc.defaultOptions,
        ...t
      }, this.addressMode = t.addressMode, this.addressModeU = t.addressModeU ?? this.addressModeU, this.addressModeV = t.addressModeV ?? this.addressModeV, this.addressModeW = t.addressModeW ?? this.addressModeW, this.scaleMode = t.scaleMode, this.magFilter = t.magFilter ?? this.magFilter, this.minFilter = t.minFilter ?? this.minFilter, this.mipmapFilter = t.mipmapFilter ?? this.mipmapFilter, this.lodMinClamp = t.lodMinClamp, this.lodMaxClamp = t.lodMaxClamp, this.compare = t.compare, this.maxAnisotropy = t.maxAnisotropy ?? 1;
    }
    set addressMode(t) {
      this.addressModeU = t, this.addressModeV = t, this.addressModeW = t;
    }
    get addressMode() {
      return this.addressModeU;
    }
    set wrapMode(t) {
      Et(qt, "TextureStyle.wrapMode is now TextureStyle.addressMode"), this.addressMode = t;
    }
    get wrapMode() {
      return this.addressMode;
    }
    set scaleMode(t) {
      this.magFilter = t, this.minFilter = t, this.mipmapFilter = t;
    }
    get scaleMode() {
      return this.magFilter;
    }
    set maxAnisotropy(t) {
      this._maxAnisotropy = Math.min(t, 16), this._maxAnisotropy > 1 && (this.scaleMode = "linear");
    }
    get maxAnisotropy() {
      return this._maxAnisotropy;
    }
    get _resourceId() {
      return this._sharedResourceId || this._generateResourceId();
    }
    update() {
      this._sharedResourceId = null, this.emit("change", this);
    }
    _generateResourceId() {
      const t = `${this.addressModeU}-${this.addressModeV}-${this.addressModeW}-${this.magFilter}-${this.minFilter}-${this.mipmapFilter}-${this.lodMinClamp}-${this.lodMaxClamp}-${this.compare}-${this._maxAnisotropy}`;
      return this._sharedResourceId = gu(t), this._resourceId;
    }
    destroy() {
      this.destroyed = true, this.emit("destroy", this), this.emit("change", this), this.removeAllListeners();
    }
  };
  kc.defaultOptions = {
    addressMode: "clamp-to-edge",
    scaleMode: "linear"
  };
  Hn = kc;
  const Ac = class Ec extends Qe {
    constructor(t = {}) {
      super(), this.options = t, this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.uid = Kt("textureSource"), this._resourceType = "textureSource", this._resourceId = Kt("resource"), this.uploadMethodId = "unknown", this._resolution = 1, this.pixelWidth = 1, this.pixelHeight = 1, this.width = 1, this.height = 1, this.sampleCount = 1, this.mipLevelCount = 1, this.autoGenerateMipmaps = false, this.format = "rgba8unorm", this.dimension = "2d", this.viewDimension = "2d", this.arrayLayerCount = 1, this.antialias = false, this._touched = 0, this._batchTick = -1, this._textureBindLocation = -1, t = {
        ...Ec.defaultOptions,
        ...t
      }, this.label = t.label ?? "", this.resource = t.resource, this.autoGarbageCollect = t.autoGarbageCollect, this._resolution = t.resolution, t.width ? this.pixelWidth = t.width * this._resolution : this.pixelWidth = this.resource ? this.resourceWidth ?? 1 : 1, t.height ? this.pixelHeight = t.height * this._resolution : this.pixelHeight = this.resource ? this.resourceHeight ?? 1 : 1, this.width = this.pixelWidth / this._resolution, this.height = this.pixelHeight / this._resolution, this.format = t.format, this.dimension = t.dimensions, this.viewDimension = t.viewDimension ?? t.dimensions, this.arrayLayerCount = t.arrayLayerCount, this.mipLevelCount = t.mipLevelCount, this.autoGenerateMipmaps = t.autoGenerateMipmaps, this.sampleCount = t.sampleCount, this.antialias = t.antialias, this.alphaMode = t.alphaMode, this.style = new Hn(Sc(t)), this.destroyed = false, this._refreshPOT();
    }
    get source() {
      return this;
    }
    get style() {
      return this._style;
    }
    set style(t) {
      var _a2, _b2;
      this.style !== t && ((_a2 = this._style) == null ? void 0 : _a2.off("change", this._onStyleChange, this), this._style = t, (_b2 = this._style) == null ? void 0 : _b2.on("change", this._onStyleChange, this), this._onStyleChange());
    }
    set maxAnisotropy(t) {
      this._style.maxAnisotropy = t;
    }
    get maxAnisotropy() {
      return this._style.maxAnisotropy;
    }
    get addressMode() {
      return this._style.addressMode;
    }
    set addressMode(t) {
      this._style.addressMode = t;
    }
    get repeatMode() {
      return this._style.addressMode;
    }
    set repeatMode(t) {
      this._style.addressMode = t;
    }
    get magFilter() {
      return this._style.magFilter;
    }
    set magFilter(t) {
      this._style.magFilter = t;
    }
    get minFilter() {
      return this._style.minFilter;
    }
    set minFilter(t) {
      this._style.minFilter = t;
    }
    get mipmapFilter() {
      return this._style.mipmapFilter;
    }
    set mipmapFilter(t) {
      this._style.mipmapFilter = t;
    }
    get lodMinClamp() {
      return this._style.lodMinClamp;
    }
    set lodMinClamp(t) {
      this._style.lodMinClamp = t;
    }
    get lodMaxClamp() {
      return this._style.lodMaxClamp;
    }
    set lodMaxClamp(t) {
      this._style.lodMaxClamp = t;
    }
    _onStyleChange() {
      this.emit("styleChange", this);
    }
    update() {
      if (this.resource) {
        const t = this._resolution;
        if (this.resize(this.resourceWidth / t, this.resourceHeight / t)) return;
      }
      this.emit("update", this);
    }
    destroy() {
      this.destroyed = true, this.unload(), this.emit("destroy", this), this._style && (this._style.destroy(), this._style = null), this.uploadMethodId = null, this.resource = null, this.removeAllListeners();
    }
    unload() {
      var _a2, _b2;
      this._resourceId = Kt("resource"), this.emit("change", this), this.emit("unload", this);
      for (const t in this._gpuData) (_b2 = (_a2 = this._gpuData[t]) == null ? void 0 : _a2.destroy) == null ? void 0 : _b2.call(_a2);
      this._gpuData = /* @__PURE__ */ Object.create(null);
    }
    get resourceWidth() {
      const { resource: t } = this;
      return t.naturalWidth || t.videoWidth || t.displayWidth || t.width;
    }
    get resourceHeight() {
      const { resource: t } = this;
      return t.naturalHeight || t.videoHeight || t.displayHeight || t.height;
    }
    get resolution() {
      return this._resolution;
    }
    set resolution(t) {
      this._resolution !== t && (this._resolution = t, this.width = this.pixelWidth / t, this.height = this.pixelHeight / t);
    }
    resize(t, e, s) {
      s || (s = this._resolution), t || (t = this.width), e || (e = this.height);
      const i = Math.round(t * s), r = Math.round(e * s);
      return this.width = i / s, this.height = r / s, this._resolution = s, this.pixelWidth === i && this.pixelHeight === r ? false : (this._refreshPOT(), this.pixelWidth = i, this.pixelHeight = r, this.emit("resize", this), this._resourceId = Kt("resource"), this.emit("change", this), true);
    }
    updateMipmaps() {
      this.autoGenerateMipmaps && this.mipLevelCount > 1 && this.emit("updateMipmaps", this);
    }
    set wrapMode(t) {
      this._style.wrapMode = t;
    }
    get wrapMode() {
      return this._style.wrapMode;
    }
    set scaleMode(t) {
      this._style.scaleMode = t;
    }
    get scaleMode() {
      return this._style.scaleMode;
    }
    _refreshPOT() {
      this.isPowerOfTwo = ga(this.pixelWidth) && ga(this.pixelHeight);
    }
    static test(t) {
      throw new Error("Unimplemented");
    }
  };
  Ac.defaultOptions = {
    resolution: 1,
    format: "bgra8unorm",
    alphaMode: "premultiply-alpha-on-upload",
    dimensions: "2d",
    viewDimension: "2d",
    arrayLayerCount: 1,
    mipLevelCount: 1,
    autoGenerateMipmaps: false,
    sampleCount: 1,
    antialias: false,
    autoGarbageCollect: false
  };
  ke = Ac;
  class Ro extends ke {
    constructor(t) {
      const e = t.resource || new Float32Array(t.width * t.height * 4);
      let s = t.format;
      s || (e instanceof Float32Array ? s = "rgba32float" : e instanceof Int32Array || e instanceof Uint32Array ? s = "rgba32uint" : e instanceof Int16Array || e instanceof Uint16Array ? s = "rgba16uint" : (e instanceof Int8Array, s = "bgra8unorm")), super({
        ...t,
        resource: e,
        format: s
      }), this.uploadMethodId = "buffer";
    }
    static test(t) {
      return t instanceof Int8Array || t instanceof Uint8Array || t instanceof Uint8ClampedArray || t instanceof Int16Array || t instanceof Uint16Array || t instanceof Int32Array || t instanceof Uint32Array || t instanceof Float32Array;
    }
  }
  Ro.extension = it.TextureSource;
  const xa = new _t();
  yu = class {
    constructor(t, e) {
      this.mapCoord = new _t(), this.uClampFrame = new Float32Array(4), this.uClampOffset = new Float32Array(2), this._textureID = -1, this._updateID = 0, this.clampOffset = 0, typeof e > "u" ? this.clampMargin = t.width < 10 ? 0 : 0.5 : this.clampMargin = e, this.isSimple = false, this.texture = t;
    }
    get texture() {
      return this._texture;
    }
    set texture(t) {
      var _a2;
      this.texture !== t && ((_a2 = this._texture) == null ? void 0 : _a2.removeListener("update", this.update, this), this._texture = t, this._texture.addListener("update", this.update, this), this.update());
    }
    multiplyUvs(t, e) {
      e === void 0 && (e = t);
      const s = this.mapCoord;
      for (let i = 0; i < t.length; i += 2) {
        const r = t[i], o = t[i + 1];
        e[i] = r * s.a + o * s.c + s.tx, e[i + 1] = r * s.b + o * s.d + s.ty;
      }
      return e;
    }
    update() {
      const t = this._texture;
      this._updateID++;
      const e = t.uvs;
      this.mapCoord.set(e.x1 - e.x0, e.y1 - e.y0, e.x3 - e.x0, e.y3 - e.y0, e.x0, e.y0);
      const s = t.orig, i = t.trim;
      i && (xa.set(s.width / i.width, 0, 0, s.height / i.height, -i.x / i.width, -i.y / i.height), this.mapCoord.append(xa));
      const r = t.source, o = this.uClampFrame, a = this.clampMargin / r._resolution, l = this.clampOffset / r._resolution;
      return o[0] = (t.frame.x + a + l) / r.width, o[1] = (t.frame.y + a + l) / r.height, o[2] = (t.frame.x + t.frame.width - a + l) / r.width, o[3] = (t.frame.y + t.frame.height - a + l) / r.height, this.uClampOffset[0] = this.clampOffset / r.pixelWidth, this.uClampOffset[1] = this.clampOffset / r.pixelHeight, this.isSimple = t.frame.width === r.width && t.frame.height === r.height && t.rotate === 0, true;
    }
  };
  Ct = class extends Qe {
    constructor({ source: t, label: e, frame: s, orig: i, trim: r, defaultAnchor: o, defaultBorders: a, rotate: l, dynamic: c } = {}) {
      if (super(), this.uid = Kt("texture"), this.uvs = {
        x0: 0,
        y0: 0,
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
        x3: 0,
        y3: 0
      }, this.frame = new Nt(), this.noFrame = false, this.dynamic = false, this.isTexture = true, this.label = e, this.source = (t == null ? void 0 : t.source) ?? new ke(), this.noFrame = !s, s) this.frame.copyFrom(s);
      else {
        const { width: h, height: d } = this._source;
        this.frame.width = h, this.frame.height = d;
      }
      this.orig = i || this.frame, this.trim = r, this.rotate = l ?? 0, this.defaultAnchor = o, this.defaultBorders = a, this.destroyed = false, this.dynamic = c || false, this.updateUvs();
    }
    set source(t) {
      this._source && this._source.off("resize", this.update, this), this._source = t, t.on("resize", this.update, this), this.emit("update", this);
    }
    get source() {
      return this._source;
    }
    get textureMatrix() {
      return this._textureMatrix || (this._textureMatrix = new yu(this)), this._textureMatrix;
    }
    get width() {
      return this.orig.width;
    }
    get height() {
      return this.orig.height;
    }
    updateUvs() {
      const { uvs: t, frame: e } = this, { width: s, height: i } = this._source, r = e.x / s, o = e.y / i, a = e.width / s, l = e.height / i;
      let c = this.rotate;
      if (c) {
        const h = a / 2, d = l / 2, u = r + h, p = o + d;
        c = St.add(c, St.NW), t.x0 = u + h * St.uX(c), t.y0 = p + d * St.uY(c), c = St.add(c, 2), t.x1 = u + h * St.uX(c), t.y1 = p + d * St.uY(c), c = St.add(c, 2), t.x2 = u + h * St.uX(c), t.y2 = p + d * St.uY(c), c = St.add(c, 2), t.x3 = u + h * St.uX(c), t.y3 = p + d * St.uY(c);
      } else t.x0 = r, t.y0 = o, t.x1 = r + a, t.y1 = o, t.x2 = r + a, t.y2 = o + l, t.x3 = r, t.y3 = o + l;
    }
    destroy(t = false) {
      this._source && (this._source.off("resize", this.update, this), t && (this._source.destroy(), this._source = null)), this._textureMatrix = null, this.destroyed = true, this.emit("destroy", this), this.removeAllListeners();
    }
    update() {
      this.noFrame && (this.frame.width = this._source.width, this.frame.height = this._source.height), this.updateUvs(), this.emit("update", this);
    }
    get baseTexture() {
      return Et(qt, "Texture.baseTexture is now Texture.source"), this._source;
    }
  };
  Ct.EMPTY = new Ct({
    label: "EMPTY",
    source: new ke({
      label: "EMPTY"
    })
  });
  Ct.EMPTY.destroy = Cc;
  Ct.WHITE = new Ct({
    source: new Ro({
      resource: new Uint8Array([
        255,
        255,
        255,
        255
      ]),
      width: 1,
      height: 1,
      alphaMode: "premultiply-alpha-on-upload",
      label: "WHITE"
    }),
    label: "WHITE"
  });
  Ct.WHITE.destroy = Cc;
  function Mc(n, t, e) {
    const { width: s, height: i } = e.orig, r = e.trim;
    if (r) {
      const o = r.width, a = r.height;
      n.minX = r.x - t._x * s, n.maxX = n.minX + o, n.minY = r.y - t._y * i, n.maxY = n.minY + a;
    } else n.minX = -t._x * s, n.maxX = n.minX + s, n.minY = -t._y * i, n.maxY = n.minY + i;
  }
  const ba = new _t();
  Se = class {
    constructor(t = 1 / 0, e = 1 / 0, s = -1 / 0, i = -1 / 0) {
      this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = ba, this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
    }
    isEmpty() {
      return this.minX > this.maxX || this.minY > this.maxY;
    }
    get rectangle() {
      this._rectangle || (this._rectangle = new Nt());
      const t = this._rectangle;
      return this.minX > this.maxX || this.minY > this.maxY ? (t.x = 0, t.y = 0, t.width = 0, t.height = 0) : t.copyFromBounds(this), t;
    }
    clear() {
      return this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = ba, this;
    }
    set(t, e, s, i) {
      this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
    }
    addFrame(t, e, s, i, r) {
      r || (r = this.matrix);
      const o = r.a, a = r.b, l = r.c, c = r.d, h = r.tx, d = r.ty;
      let u = this.minX, p = this.minY, f = this.maxX, g = this.maxY, m = o * t + l * e + h, y = a * t + c * e + d;
      m < u && (u = m), y < p && (p = y), m > f && (f = m), y > g && (g = y), m = o * s + l * e + h, y = a * s + c * e + d, m < u && (u = m), y < p && (p = y), m > f && (f = m), y > g && (g = y), m = o * t + l * i + h, y = a * t + c * i + d, m < u && (u = m), y < p && (p = y), m > f && (f = m), y > g && (g = y), m = o * s + l * i + h, y = a * s + c * i + d, m < u && (u = m), y < p && (p = y), m > f && (f = m), y > g && (g = y), this.minX = u, this.minY = p, this.maxX = f, this.maxY = g;
    }
    addRect(t, e) {
      this.addFrame(t.x, t.y, t.x + t.width, t.y + t.height, e);
    }
    addBounds(t, e) {
      this.addFrame(t.minX, t.minY, t.maxX, t.maxY, e);
    }
    addBoundsMask(t) {
      this.minX = this.minX > t.minX ? this.minX : t.minX, this.minY = this.minY > t.minY ? this.minY : t.minY, this.maxX = this.maxX < t.maxX ? this.maxX : t.maxX, this.maxY = this.maxY < t.maxY ? this.maxY : t.maxY;
    }
    applyMatrix(t) {
      const e = this.minX, s = this.minY, i = this.maxX, r = this.maxY, { a: o, b: a, c: l, d: c, tx: h, ty: d } = t;
      let u = o * e + l * s + h, p = a * e + c * s + d;
      this.minX = u, this.minY = p, this.maxX = u, this.maxY = p, u = o * i + l * s + h, p = a * i + c * s + d, this.minX = u < this.minX ? u : this.minX, this.minY = p < this.minY ? p : this.minY, this.maxX = u > this.maxX ? u : this.maxX, this.maxY = p > this.maxY ? p : this.maxY, u = o * e + l * r + h, p = a * e + c * r + d, this.minX = u < this.minX ? u : this.minX, this.minY = p < this.minY ? p : this.minY, this.maxX = u > this.maxX ? u : this.maxX, this.maxY = p > this.maxY ? p : this.maxY, u = o * i + l * r + h, p = a * i + c * r + d, this.minX = u < this.minX ? u : this.minX, this.minY = p < this.minY ? p : this.minY, this.maxX = u > this.maxX ? u : this.maxX, this.maxY = p > this.maxY ? p : this.maxY;
    }
    fit(t) {
      return this.minX < t.left && (this.minX = t.left), this.maxX > t.right && (this.maxX = t.right), this.minY < t.top && (this.minY = t.top), this.maxY > t.bottom && (this.maxY = t.bottom), this;
    }
    fitBounds(t, e, s, i) {
      return this.minX < t && (this.minX = t), this.maxX > e && (this.maxX = e), this.minY < s && (this.minY = s), this.maxY > i && (this.maxY = i), this;
    }
    pad(t, e = t) {
      return this.minX -= t, this.maxX += t, this.minY -= e, this.maxY += e, this;
    }
    ceil() {
      return this.minX = Math.floor(this.minX), this.minY = Math.floor(this.minY), this.maxX = Math.ceil(this.maxX), this.maxY = Math.ceil(this.maxY), this;
    }
    clone() {
      return new Se(this.minX, this.minY, this.maxX, this.maxY);
    }
    scale(t, e = t) {
      return this.minX *= t, this.minY *= e, this.maxX *= t, this.maxY *= e, this;
    }
    get x() {
      return this.minX;
    }
    set x(t) {
      const e = this.maxX - this.minX;
      this.minX = t, this.maxX = t + e;
    }
    get y() {
      return this.minY;
    }
    set y(t) {
      const e = this.maxY - this.minY;
      this.minY = t, this.maxY = t + e;
    }
    get width() {
      return this.maxX - this.minX;
    }
    set width(t) {
      this.maxX = this.minX + t;
    }
    get height() {
      return this.maxY - this.minY;
    }
    set height(t) {
      this.maxY = this.minY + t;
    }
    get left() {
      return this.minX;
    }
    get right() {
      return this.maxX;
    }
    get top() {
      return this.minY;
    }
    get bottom() {
      return this.maxY;
    }
    get isPositive() {
      return this.maxX - this.minX > 0 && this.maxY - this.minY > 0;
    }
    get isValid() {
      return this.minX + this.minY !== 1 / 0;
    }
    addVertexData(t, e, s, i) {
      let r = this.minX, o = this.minY, a = this.maxX, l = this.maxY;
      i || (i = this.matrix);
      const c = i.a, h = i.b, d = i.c, u = i.d, p = i.tx, f = i.ty;
      for (let g = e; g < s; g += 2) {
        const m = t[g], y = t[g + 1], b = c * m + d * y + p, x = h * m + u * y + f;
        r = b < r ? b : r, o = x < o ? x : o, a = b > a ? b : a, l = x > l ? x : l;
      }
      this.minX = r, this.minY = o, this.maxX = a, this.maxY = l;
    }
    containsPoint(t, e) {
      return this.minX <= t && this.minY <= e && this.maxX >= t && this.maxY >= e;
    }
    toString() {
      return `[pixi.js:Bounds minX=${this.minX} minY=${this.minY} maxX=${this.maxX} maxY=${this.maxY} width=${this.width} height=${this.height}]`;
    }
    copyFrom(t) {
      return this.minX = t.minX, this.minY = t.minY, this.maxX = t.maxX, this.maxY = t.maxY, this;
    }
  };
  var xu = {
    grad: 0.9,
    turn: 360,
    rad: 360 / (2 * Math.PI)
  }, en = function(n) {
    return typeof n == "string" ? n.length > 0 : typeof n == "number";
  }, ie = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = Math.pow(10, t)), Math.round(e * n) / e + 0;
  }, Me = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = 1), n > e ? e : n > t ? n : t;
  }, Pc = function(n) {
    return (n = isFinite(n) ? n % 360 : 0) > 0 ? n : n + 360;
  }, _a = function(n) {
    return {
      r: Me(n.r, 0, 255),
      g: Me(n.g, 0, 255),
      b: Me(n.b, 0, 255),
      a: Me(n.a)
    };
  }, lr = function(n) {
    return {
      r: ie(n.r),
      g: ie(n.g),
      b: ie(n.b),
      a: ie(n.a, 3)
    };
  }, bu = /^#([0-9a-f]{3,8})$/i, oi = function(n) {
    var t = n.toString(16);
    return t.length < 2 ? "0" + t : t;
  }, Ic = function(n) {
    var t = n.r, e = n.g, s = n.b, i = n.a, r = Math.max(t, e, s), o = r - Math.min(t, e, s), a = o ? r === t ? (e - s) / o : r === e ? 2 + (s - t) / o : 4 + (t - e) / o : 0;
    return {
      h: 60 * (a < 0 ? a + 6 : a),
      s: r ? o / r * 100 : 0,
      v: r / 255 * 100,
      a: i
    };
  }, Rc = function(n) {
    var t = n.h, e = n.s, s = n.v, i = n.a;
    t = t / 360 * 6, e /= 100, s /= 100;
    var r = Math.floor(t), o = s * (1 - e), a = s * (1 - (t - r) * e), l = s * (1 - (1 - t + r) * e), c = r % 6;
    return {
      r: 255 * [
        s,
        a,
        o,
        o,
        l,
        s
      ][c],
      g: 255 * [
        l,
        s,
        s,
        a,
        o,
        o
      ][c],
      b: 255 * [
        o,
        o,
        l,
        s,
        s,
        a
      ][c],
      a: i
    };
  }, wa = function(n) {
    return {
      h: Pc(n.h),
      s: Me(n.s, 0, 100),
      l: Me(n.l, 0, 100),
      a: Me(n.a)
    };
  }, va = function(n) {
    return {
      h: ie(n.h),
      s: ie(n.s),
      l: ie(n.l),
      a: ie(n.a, 3)
    };
  }, Ca = function(n) {
    return Rc((e = (t = n).s, {
      h: t.h,
      s: (e *= ((s = t.l) < 50 ? s : 100 - s) / 100) > 0 ? 2 * e / (s + e) * 100 : 0,
      v: s + e,
      a: t.a
    }));
    var t, e, s;
  }, Bs = function(n) {
    return {
      h: (t = Ic(n)).h,
      s: (i = (200 - (e = t.s)) * (s = t.v) / 100) > 0 && i < 200 ? e * s / 100 / (i <= 100 ? i : 200 - i) * 100 : 0,
      l: i / 2,
      a: t.a
    };
    var t, e, s, i;
  }, _u = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s*,\s*([+-]?\d*\.?\d+)%\s*,\s*([+-]?\d*\.?\d+)%\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, wu = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s+([+-]?\d*\.?\d+)%\s+([+-]?\d*\.?\d+)%\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, vu = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Cu = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Kr = {
    string: [
      [
        function(n) {
          var t = bu.exec(n);
          return t ? (n = t[1]).length <= 4 ? {
            r: parseInt(n[0] + n[0], 16),
            g: parseInt(n[1] + n[1], 16),
            b: parseInt(n[2] + n[2], 16),
            a: n.length === 4 ? ie(parseInt(n[3] + n[3], 16) / 255, 2) : 1
          } : n.length === 6 || n.length === 8 ? {
            r: parseInt(n.substr(0, 2), 16),
            g: parseInt(n.substr(2, 2), 16),
            b: parseInt(n.substr(4, 2), 16),
            a: n.length === 8 ? ie(parseInt(n.substr(6, 2), 16) / 255, 2) : 1
          } : null : null;
        },
        "hex"
      ],
      [
        function(n) {
          var t = vu.exec(n) || Cu.exec(n);
          return t ? t[2] !== t[4] || t[4] !== t[6] ? null : _a({
            r: Number(t[1]) / (t[2] ? 100 / 255 : 1),
            g: Number(t[3]) / (t[4] ? 100 / 255 : 1),
            b: Number(t[5]) / (t[6] ? 100 / 255 : 1),
            a: t[7] === void 0 ? 1 : Number(t[7]) / (t[8] ? 100 : 1)
          }) : null;
        },
        "rgb"
      ],
      [
        function(n) {
          var t = _u.exec(n) || wu.exec(n);
          if (!t) return null;
          var e, s, i = wa({
            h: (e = t[1], s = t[2], s === void 0 && (s = "deg"), Number(e) * (xu[s] || 1)),
            s: Number(t[3]),
            l: Number(t[4]),
            a: t[5] === void 0 ? 1 : Number(t[5]) / (t[6] ? 100 : 1)
          });
          return Ca(i);
        },
        "hsl"
      ]
    ],
    object: [
      [
        function(n) {
          var t = n.r, e = n.g, s = n.b, i = n.a, r = i === void 0 ? 1 : i;
          return en(t) && en(e) && en(s) ? _a({
            r: Number(t),
            g: Number(e),
            b: Number(s),
            a: Number(r)
          }) : null;
        },
        "rgb"
      ],
      [
        function(n) {
          var t = n.h, e = n.s, s = n.l, i = n.a, r = i === void 0 ? 1 : i;
          if (!en(t) || !en(e) || !en(s)) return null;
          var o = wa({
            h: Number(t),
            s: Number(e),
            l: Number(s),
            a: Number(r)
          });
          return Ca(o);
        },
        "hsl"
      ],
      [
        function(n) {
          var t = n.h, e = n.s, s = n.v, i = n.a, r = i === void 0 ? 1 : i;
          if (!en(t) || !en(e) || !en(s)) return null;
          var o = (function(a) {
            return {
              h: Pc(a.h),
              s: Me(a.s, 0, 100),
              v: Me(a.v, 0, 100),
              a: Me(a.a)
            };
          })({
            h: Number(t),
            s: Number(e),
            v: Number(s),
            a: Number(r)
          });
          return Rc(o);
        },
        "hsv"
      ]
    ]
  }, Sa = function(n, t) {
    for (var e = 0; e < t.length; e++) {
      var s = t[e][0](n);
      if (s) return [
        s,
        t[e][1]
      ];
    }
    return [
      null,
      void 0
    ];
  }, Su = function(n) {
    return typeof n == "string" ? Sa(n.trim(), Kr.string) : typeof n == "object" && n !== null ? Sa(n, Kr.object) : [
      null,
      void 0
    ];
  }, cr = function(n, t) {
    var e = Bs(n);
    return {
      h: e.h,
      s: Me(e.s + 100 * t, 0, 100),
      l: e.l,
      a: e.a
    };
  }, hr = function(n) {
    return (299 * n.r + 587 * n.g + 114 * n.b) / 1e3 / 255;
  }, ka = function(n, t) {
    var e = Bs(n);
    return {
      h: e.h,
      s: e.s,
      l: Me(e.l + 100 * t, 0, 100),
      a: e.a
    };
  }, Jr = (function() {
    function n(t) {
      this.parsed = Su(t)[0], this.rgba = this.parsed || {
        r: 0,
        g: 0,
        b: 0,
        a: 1
      };
    }
    return n.prototype.isValid = function() {
      return this.parsed !== null;
    }, n.prototype.brightness = function() {
      return ie(hr(this.rgba), 2);
    }, n.prototype.isDark = function() {
      return hr(this.rgba) < 0.5;
    }, n.prototype.isLight = function() {
      return hr(this.rgba) >= 0.5;
    }, n.prototype.toHex = function() {
      return t = lr(this.rgba), e = t.r, s = t.g, i = t.b, o = (r = t.a) < 1 ? oi(ie(255 * r)) : "", "#" + oi(e) + oi(s) + oi(i) + o;
      var t, e, s, i, r, o;
    }, n.prototype.toRgb = function() {
      return lr(this.rgba);
    }, n.prototype.toRgbString = function() {
      return t = lr(this.rgba), e = t.r, s = t.g, i = t.b, (r = t.a) < 1 ? "rgba(" + e + ", " + s + ", " + i + ", " + r + ")" : "rgb(" + e + ", " + s + ", " + i + ")";
      var t, e, s, i, r;
    }, n.prototype.toHsl = function() {
      return va(Bs(this.rgba));
    }, n.prototype.toHslString = function() {
      return t = va(Bs(this.rgba)), e = t.h, s = t.s, i = t.l, (r = t.a) < 1 ? "hsla(" + e + ", " + s + "%, " + i + "%, " + r + ")" : "hsl(" + e + ", " + s + "%, " + i + "%)";
      var t, e, s, i, r;
    }, n.prototype.toHsv = function() {
      return t = Ic(this.rgba), {
        h: ie(t.h),
        s: ie(t.s),
        v: ie(t.v),
        a: ie(t.a, 3)
      };
      var t;
    }, n.prototype.invert = function() {
      return Ve({
        r: 255 - (t = this.rgba).r,
        g: 255 - t.g,
        b: 255 - t.b,
        a: t.a
      });
      var t;
    }, n.prototype.saturate = function(t) {
      return t === void 0 && (t = 0.1), Ve(cr(this.rgba, t));
    }, n.prototype.desaturate = function(t) {
      return t === void 0 && (t = 0.1), Ve(cr(this.rgba, -t));
    }, n.prototype.grayscale = function() {
      return Ve(cr(this.rgba, -1));
    }, n.prototype.lighten = function(t) {
      return t === void 0 && (t = 0.1), Ve(ka(this.rgba, t));
    }, n.prototype.darken = function(t) {
      return t === void 0 && (t = 0.1), Ve(ka(this.rgba, -t));
    }, n.prototype.rotate = function(t) {
      return t === void 0 && (t = 15), this.hue(this.hue() + t);
    }, n.prototype.alpha = function(t) {
      return typeof t == "number" ? Ve({
        r: (e = this.rgba).r,
        g: e.g,
        b: e.b,
        a: t
      }) : ie(this.rgba.a, 3);
      var e;
    }, n.prototype.hue = function(t) {
      var e = Bs(this.rgba);
      return typeof t == "number" ? Ve({
        h: t,
        s: e.s,
        l: e.l,
        a: e.a
      }) : ie(e.h);
    }, n.prototype.isEqual = function(t) {
      return this.toHex() === Ve(t).toHex();
    }, n;
  })(), Ve = function(n) {
    return n instanceof Jr ? n : new Jr(n);
  }, Ta = [], ku = function(n) {
    n.forEach(function(t) {
      Ta.indexOf(t) < 0 && (t(Jr, Kr), Ta.push(t));
    });
  };
  function Tu(n, t) {
    var e = {
      white: "#ffffff",
      bisque: "#ffe4c4",
      blue: "#0000ff",
      cadetblue: "#5f9ea0",
      chartreuse: "#7fff00",
      chocolate: "#d2691e",
      coral: "#ff7f50",
      antiquewhite: "#faebd7",
      aqua: "#00ffff",
      azure: "#f0ffff",
      whitesmoke: "#f5f5f5",
      papayawhip: "#ffefd5",
      plum: "#dda0dd",
      blanchedalmond: "#ffebcd",
      black: "#000000",
      gold: "#ffd700",
      goldenrod: "#daa520",
      gainsboro: "#dcdcdc",
      cornsilk: "#fff8dc",
      cornflowerblue: "#6495ed",
      burlywood: "#deb887",
      aquamarine: "#7fffd4",
      beige: "#f5f5dc",
      crimson: "#dc143c",
      cyan: "#00ffff",
      darkblue: "#00008b",
      darkcyan: "#008b8b",
      darkgoldenrod: "#b8860b",
      darkkhaki: "#bdb76b",
      darkgray: "#a9a9a9",
      darkgreen: "#006400",
      darkgrey: "#a9a9a9",
      peachpuff: "#ffdab9",
      darkmagenta: "#8b008b",
      darkred: "#8b0000",
      darkorchid: "#9932cc",
      darkorange: "#ff8c00",
      darkslateblue: "#483d8b",
      gray: "#808080",
      darkslategray: "#2f4f4f",
      darkslategrey: "#2f4f4f",
      deeppink: "#ff1493",
      deepskyblue: "#00bfff",
      wheat: "#f5deb3",
      firebrick: "#b22222",
      floralwhite: "#fffaf0",
      ghostwhite: "#f8f8ff",
      darkviolet: "#9400d3",
      magenta: "#ff00ff",
      green: "#008000",
      dodgerblue: "#1e90ff",
      grey: "#808080",
      honeydew: "#f0fff0",
      hotpink: "#ff69b4",
      blueviolet: "#8a2be2",
      forestgreen: "#228b22",
      lawngreen: "#7cfc00",
      indianred: "#cd5c5c",
      indigo: "#4b0082",
      fuchsia: "#ff00ff",
      brown: "#a52a2a",
      maroon: "#800000",
      mediumblue: "#0000cd",
      lightcoral: "#f08080",
      darkturquoise: "#00ced1",
      lightcyan: "#e0ffff",
      ivory: "#fffff0",
      lightyellow: "#ffffe0",
      lightsalmon: "#ffa07a",
      lightseagreen: "#20b2aa",
      linen: "#faf0e6",
      mediumaquamarine: "#66cdaa",
      lemonchiffon: "#fffacd",
      lime: "#00ff00",
      khaki: "#f0e68c",
      mediumseagreen: "#3cb371",
      limegreen: "#32cd32",
      mediumspringgreen: "#00fa9a",
      lightskyblue: "#87cefa",
      lightblue: "#add8e6",
      midnightblue: "#191970",
      lightpink: "#ffb6c1",
      mistyrose: "#ffe4e1",
      moccasin: "#ffe4b5",
      mintcream: "#f5fffa",
      lightslategray: "#778899",
      lightslategrey: "#778899",
      navajowhite: "#ffdead",
      navy: "#000080",
      mediumvioletred: "#c71585",
      powderblue: "#b0e0e6",
      palegoldenrod: "#eee8aa",
      oldlace: "#fdf5e6",
      paleturquoise: "#afeeee",
      mediumturquoise: "#48d1cc",
      mediumorchid: "#ba55d3",
      rebeccapurple: "#663399",
      lightsteelblue: "#b0c4de",
      mediumslateblue: "#7b68ee",
      thistle: "#d8bfd8",
      tan: "#d2b48c",
      orchid: "#da70d6",
      mediumpurple: "#9370db",
      purple: "#800080",
      pink: "#ffc0cb",
      skyblue: "#87ceeb",
      springgreen: "#00ff7f",
      palegreen: "#98fb98",
      red: "#ff0000",
      yellow: "#ffff00",
      slateblue: "#6a5acd",
      lavenderblush: "#fff0f5",
      peru: "#cd853f",
      palevioletred: "#db7093",
      violet: "#ee82ee",
      teal: "#008080",
      slategray: "#708090",
      slategrey: "#708090",
      aliceblue: "#f0f8ff",
      darkseagreen: "#8fbc8f",
      darkolivegreen: "#556b2f",
      greenyellow: "#adff2f",
      seagreen: "#2e8b57",
      seashell: "#fff5ee",
      tomato: "#ff6347",
      silver: "#c0c0c0",
      sienna: "#a0522d",
      lavender: "#e6e6fa",
      lightgreen: "#90ee90",
      orange: "#ffa500",
      orangered: "#ff4500",
      steelblue: "#4682b4",
      royalblue: "#4169e1",
      turquoise: "#40e0d0",
      yellowgreen: "#9acd32",
      salmon: "#fa8072",
      saddlebrown: "#8b4513",
      sandybrown: "#f4a460",
      rosybrown: "#bc8f8f",
      darksalmon: "#e9967a",
      lightgoldenrodyellow: "#fafad2",
      snow: "#fffafa",
      lightgrey: "#d3d3d3",
      lightgray: "#d3d3d3",
      dimgray: "#696969",
      dimgrey: "#696969",
      olivedrab: "#6b8e23",
      olive: "#808000"
    }, s = {};
    for (var i in e) s[e[i]] = i;
    var r = {};
    n.prototype.toName = function(o) {
      if (!(this.rgba.a || this.rgba.r || this.rgba.g || this.rgba.b)) return "transparent";
      var a, l, c = s[this.toHex()];
      if (c) return c;
      if (o == null ? void 0 : o.closest) {
        var h = this.toRgb(), d = 1 / 0, u = "black";
        if (!r.length) for (var p in e) r[p] = new n(e[p]).toRgb();
        for (var f in e) {
          var g = (a = h, l = r[f], Math.pow(a.r - l.r, 2) + Math.pow(a.g - l.g, 2) + Math.pow(a.b - l.b, 2));
          g < d && (d = g, u = f);
        }
        return u;
      }
    }, t.string.push([
      function(o) {
        var a = o.toLowerCase(), l = a === "transparent" ? "#0000" : e[a];
        return l ? new n(l).toRgb() : null;
      },
      "name"
    ]);
  }
  ku([
    Tu
  ]);
  const ps = class Rs {
    constructor(t = 16777215) {
      this._value = null, this._components = new Float32Array(4), this._components.fill(1), this._int = 16777215, this.value = t;
    }
    get red() {
      return this._components[0];
    }
    get green() {
      return this._components[1];
    }
    get blue() {
      return this._components[2];
    }
    get alpha() {
      return this._components[3];
    }
    setValue(t) {
      return this.value = t, this;
    }
    set value(t) {
      if (t instanceof Rs) this._value = this._cloneSource(t._value), this._int = t._int, this._components.set(t._components);
      else {
        if (t === null) throw new Error("Cannot set Color#value to null");
        (this._value === null || !this._isSourceEqual(this._value, t)) && (this._value = this._cloneSource(t), this._normalize(this._value));
      }
    }
    get value() {
      return this._value;
    }
    _cloneSource(t) {
      return typeof t == "string" || typeof t == "number" || t instanceof Number || t === null ? t : Array.isArray(t) || ArrayBuffer.isView(t) ? t.slice(0) : typeof t == "object" && t !== null ? {
        ...t
      } : t;
    }
    _isSourceEqual(t, e) {
      const s = typeof t;
      if (s !== typeof e) return false;
      if (s === "number" || s === "string" || t instanceof Number) return t === e;
      if (Array.isArray(t) && Array.isArray(e) || ArrayBuffer.isView(t) && ArrayBuffer.isView(e)) return t.length !== e.length ? false : t.every((r, o) => r === e[o]);
      if (t !== null && e !== null) {
        const r = Object.keys(t), o = Object.keys(e);
        return r.length !== o.length ? false : r.every((a) => t[a] === e[a]);
      }
      return t === e;
    }
    toRgba() {
      const [t, e, s, i] = this._components;
      return {
        r: t,
        g: e,
        b: s,
        a: i
      };
    }
    toRgb() {
      const [t, e, s] = this._components;
      return {
        r: t,
        g: e,
        b: s
      };
    }
    toRgbaString() {
      const [t, e, s] = this.toUint8RgbArray();
      return `rgba(${t},${e},${s},${this.alpha})`;
    }
    toUint8RgbArray(t) {
      const [e, s, i] = this._components;
      return this._arrayRgb || (this._arrayRgb = []), t || (t = this._arrayRgb), t[0] = Math.round(e * 255), t[1] = Math.round(s * 255), t[2] = Math.round(i * 255), t;
    }
    toArray(t) {
      this._arrayRgba || (this._arrayRgba = []), t || (t = this._arrayRgba);
      const [e, s, i, r] = this._components;
      return t[0] = e, t[1] = s, t[2] = i, t[3] = r, t;
    }
    toRgbArray(t) {
      this._arrayRgb || (this._arrayRgb = []), t || (t = this._arrayRgb);
      const [e, s, i] = this._components;
      return t[0] = e, t[1] = s, t[2] = i, t;
    }
    toNumber() {
      return this._int;
    }
    toBgrNumber() {
      const [t, e, s] = this.toUint8RgbArray();
      return (s << 16) + (e << 8) + t;
    }
    toLittleEndianNumber() {
      const t = this._int;
      return (t >> 16) + (t & 65280) + ((t & 255) << 16);
    }
    multiply(t) {
      const [e, s, i, r] = Rs._temp.setValue(t)._components;
      return this._components[0] *= e, this._components[1] *= s, this._components[2] *= i, this._components[3] *= r, this._refreshInt(), this._value = null, this;
    }
    premultiply(t, e = true) {
      return e && (this._components[0] *= t, this._components[1] *= t, this._components[2] *= t), this._components[3] = t, this._refreshInt(), this._value = null, this;
    }
    toPremultiplied(t, e = true) {
      if (t === 1) return (255 << 24) + this._int;
      if (t === 0) return e ? 0 : this._int;
      let s = this._int >> 16 & 255, i = this._int >> 8 & 255, r = this._int & 255;
      return e && (s = s * t + 0.5 | 0, i = i * t + 0.5 | 0, r = r * t + 0.5 | 0), (t * 255 << 24) + (s << 16) + (i << 8) + r;
    }
    toHex() {
      const t = this._int.toString(16);
      return `#${"000000".substring(0, 6 - t.length) + t}`;
    }
    toHexa() {
      const e = Math.round(this._components[3] * 255).toString(16);
      return this.toHex() + "00".substring(0, 2 - e.length) + e;
    }
    setAlpha(t) {
      return this._components[3] = this._clamp(t), this._value = null, this;
    }
    _normalize(t) {
      let e, s, i, r;
      if ((typeof t == "number" || t instanceof Number) && t >= 0 && t <= 16777215) {
        const o = t;
        e = (o >> 16 & 255) / 255, s = (o >> 8 & 255) / 255, i = (o & 255) / 255, r = 1;
      } else if ((Array.isArray(t) || t instanceof Float32Array) && t.length >= 3 && t.length <= 4) t = this._clamp(t), [e, s, i, r = 1] = t;
      else if ((t instanceof Uint8Array || t instanceof Uint8ClampedArray) && t.length >= 3 && t.length <= 4) t = this._clamp(t, 0, 255), [e, s, i, r = 255] = t, e /= 255, s /= 255, i /= 255, r /= 255;
      else if (typeof t == "string" || typeof t == "object") {
        if (typeof t == "string") {
          const a = Rs.HEX_PATTERN.exec(t);
          a && (t = `#${a[2]}`);
        }
        const o = Ve(t);
        o.isValid() && ({ r: e, g: s, b: i, a: r } = o.rgba, e /= 255, s /= 255, i /= 255);
      }
      if (e !== void 0) this._components[0] = e, this._components[1] = s, this._components[2] = i, this._components[3] = r, this._refreshInt();
      else throw new Error(`Unable to convert color ${t}`);
    }
    _refreshInt() {
      this._clamp(this._components);
      const [t, e, s] = this._components;
      this._int = (t * 255 << 16) + (e * 255 << 8) + (s * 255 | 0);
    }
    _clamp(t, e = 0, s = 1) {
      return typeof t == "number" ? Math.min(Math.max(t, e), s) : (t.forEach((i, r) => {
        t[r] = Math.min(Math.max(i, e), s);
      }), t);
    }
    static isColorLike(t) {
      return typeof t == "number" || typeof t == "string" || t instanceof Number || t instanceof Rs || Array.isArray(t) || t instanceof Uint8Array || t instanceof Uint8ClampedArray || t instanceof Float32Array || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 && t.a !== void 0;
    }
  };
  ps.shared = new ps();
  ps._temp = new ps();
  ps.HEX_PATTERN = /^(#|0x)?(([a-f0-9]{3}){1,2}([a-f0-9]{2})?)$/i;
  Gt = ps;
  const Au = {
    cullArea: null,
    cullable: false,
    cullableChildren: true
  };
  let dr = 0;
  const Aa = 500;
  Vt = function(...n) {
    dr !== Aa && (dr++, dr === Aa ? console.warn("PixiJS Warning: too many warnings, no more warnings will be reported to the console by PixiJS.") : console.warn("PixiJS Warning: ", ...n));
  };
  Qs = {
    _registeredResources: /* @__PURE__ */ new Set(),
    register(n) {
      this._registeredResources.add(n);
    },
    unregister(n) {
      this._registeredResources.delete(n);
    },
    release() {
      this._registeredResources.forEach((n) => n.clear());
    },
    get registeredCount() {
      return this._registeredResources.size;
    },
    isRegistered(n) {
      return this._registeredResources.has(n);
    },
    reset() {
      this._registeredResources.clear();
    }
  };
  class Eu {
    constructor(t, e) {
      this._pool = [], this._count = 0, this._index = 0, this._classType = t, e && this.prepopulate(e);
    }
    prepopulate(t) {
      for (let e = 0; e < t; e++) this._pool[this._index++] = new this._classType();
      this._count += t;
    }
    get(t) {
      var _a2;
      let e;
      return this._index > 0 ? e = this._pool[--this._index] : (e = new this._classType(), this._count++), (_a2 = e.init) == null ? void 0 : _a2.call(e, t), e;
    }
    return(t) {
      var _a2;
      (_a2 = t.reset) == null ? void 0 : _a2.call(t), this._pool[this._index++] = t;
    }
    get totalSize() {
      return this._count;
    }
    get totalFree() {
      return this._index;
    }
    get totalUsed() {
      return this._count - this._index;
    }
    clear() {
      if (this._pool.length > 0 && this._pool[0].destroy) for (let t = 0; t < this._index; t++) this._pool[t].destroy();
      this._pool.length = 0, this._count = 0, this._index = 0;
    }
  }
  class Mu {
    constructor() {
      this._poolsByClass = /* @__PURE__ */ new Map();
    }
    prepopulate(t, e) {
      this.getPool(t).prepopulate(e);
    }
    get(t, e) {
      return this.getPool(t).get(e);
    }
    return(t) {
      this.getPool(t.constructor).return(t);
    }
    getPool(t) {
      return this._poolsByClass.has(t) || this._poolsByClass.set(t, new Eu(t)), this._poolsByClass.get(t);
    }
    stats() {
      const t = {};
      return this._poolsByClass.forEach((e) => {
        const s = t[e._classType.name] ? e._classType.name + e._classType.ID : e._classType.name;
        t[s] = {
          free: e.totalFree,
          used: e.totalUsed,
          size: e.totalSize
        };
      }), t;
    }
    clear() {
      this._poolsByClass.forEach((t) => t.clear()), this._poolsByClass.clear();
    }
  }
  _e = new Mu();
  Qs.register(_e);
  const Pu = {
    get isCachedAsTexture() {
      var _a2;
      return !!((_a2 = this.renderGroup) == null ? void 0 : _a2.isCachedAsTexture);
    },
    cacheAsTexture(n) {
      typeof n == "boolean" && n === false ? this.disableRenderGroup() : (this.enableRenderGroup(), this.renderGroup.enableCacheAsTexture(n === true ? {} : n));
    },
    updateCacheTexture() {
      var _a2;
      (_a2 = this.renderGroup) == null ? void 0 : _a2.updateCacheTexture();
    },
    get cacheAsBitmap() {
      return this.isCachedAsTexture;
    },
    set cacheAsBitmap(n) {
      Et("v8.6.0", "cacheAsBitmap is deprecated, use cacheAsTexture instead."), this.cacheAsTexture(n);
    }
  };
  Iu = function(n, t, e) {
    const s = n.length;
    let i;
    if (t >= s || e === 0) return;
    e = t + e > s ? s - t : e;
    const r = s - e;
    for (i = t; i < r; ++i) n[i] = n[i + e];
    n.length = r;
  };
  const Ru = {
    allowChildren: true,
    removeChildren(n = 0, t) {
      var _a2;
      const e = t ?? this.children.length, s = e - n, i = [];
      if (s > 0 && s <= e) {
        for (let o = e - 1; o >= n; o--) {
          const a = this.children[o];
          a && (i.push(a), a.parent = null);
        }
        Iu(this.children, n, e);
        const r = this.renderGroup || this.parentRenderGroup;
        r && r.removeChildren(i);
        for (let o = 0; o < i.length; ++o) {
          const a = i[o];
          (_a2 = a.parentRenderLayer) == null ? void 0 : _a2.detach(a), this.emit("childRemoved", a, this, o), i[o].emit("removed", this);
        }
        return i.length > 0 && this._didViewChangeTick++, i;
      } else if (s === 0 && this.children.length === 0) return i;
      throw new RangeError("removeChildren: numeric values are outside the acceptable range.");
    },
    removeChildAt(n) {
      const t = this.getChildAt(n);
      return this.removeChild(t);
    },
    getChildAt(n) {
      if (n < 0 || n >= this.children.length) throw new Error(`getChildAt: Index (${n}) does not exist.`);
      return this.children[n];
    },
    setChildIndex(n, t) {
      if (t < 0 || t >= this.children.length) throw new Error(`The index ${t} supplied is out of bounds ${this.children.length}`);
      this.getChildIndex(n), this.addChildAt(n, t);
    },
    getChildIndex(n) {
      const t = this.children.indexOf(n);
      if (t === -1) throw new Error("The supplied Container must be a child of the caller");
      return t;
    },
    addChildAt(n, t) {
      this.allowChildren || Et(qt, "addChildAt: Only Containers will be allowed to add children in v8.0.0");
      const { children: e } = this;
      if (t < 0 || t > e.length) throw new Error(`${n}addChildAt: The index ${t} supplied is out of bounds ${e.length}`);
      const s = n.parent === this;
      if (n.parent) {
        const r = n.parent.children.indexOf(n);
        if (s) {
          if (r === t) return n;
          n.parent.children.splice(r, 1);
        } else n.removeFromParent();
      }
      t === e.length ? e.push(n) : e.splice(t, 0, n), n.parent = this, n.didChange = true, n._updateFlags = 15;
      const i = this.renderGroup || this.parentRenderGroup;
      return i && i.addChild(n), this.sortableChildren && (this.sortDirty = true), s || (this.emit("childAdded", n, this, t), n.emit("added", this)), n;
    },
    swapChildren(n, t) {
      if (n === t) return;
      const e = this.getChildIndex(n), s = this.getChildIndex(t);
      this.children[e] = t, this.children[s] = n;
      const i = this.renderGroup || this.parentRenderGroup;
      i && (i.structureDidChange = true), this._didContainerChangeTick++;
    },
    removeFromParent() {
      var _a2;
      (_a2 = this.parent) == null ? void 0 : _a2.removeChild(this);
    },
    reparentChild(...n) {
      return n.length === 1 ? this.reparentChildAt(n[0], this.children.length) : (n.forEach((t) => this.reparentChildAt(t, this.children.length)), n[0]);
    },
    reparentChildAt(n, t) {
      if (n.parent === this) return this.setChildIndex(n, t), n;
      const e = n.worldTransform.clone();
      n.removeFromParent(), this.addChildAt(n, t);
      const s = this.worldTransform.clone();
      return s.invert(), e.prepend(s), n.setFromMatrix(e), n;
    },
    replaceChild(n, t) {
      n.updateLocalTransform(), this.addChildAt(t, this.getChildIndex(n)), t.setFromMatrix(n.localTransform), t.updateLocalTransform(), this.removeChild(n);
    }
  }, Lu = {
    collectRenderables(n, t, e) {
      this.parentRenderLayer && this.parentRenderLayer !== e || this.globalDisplayStatus < 7 || !this.includeInBuild || (this.sortableChildren && this.sortChildren(), this.isSimple ? this.collectRenderablesSimple(n, t, e) : this.renderGroup ? t.renderPipes.renderGroup.addRenderGroup(this.renderGroup, n) : this.collectRenderablesWithEffects(n, t, e));
    },
    collectRenderablesSimple(n, t, e) {
      const s = this.children, i = s.length;
      for (let r = 0; r < i; r++) s[r].collectRenderables(n, t, e);
    },
    collectRenderablesWithEffects(n, t, e) {
      const { renderPipes: s } = t;
      for (let i = 0; i < this.effects.length; i++) {
        const r = this.effects[i];
        s[r.pipe].push(r, this, n);
      }
      this.collectRenderablesSimple(n, t, e);
      for (let i = this.effects.length - 1; i >= 0; i--) {
        const r = this.effects[i];
        s[r.pipe].pop(r, this, n);
      }
    }
  };
  Ea = class {
    constructor() {
      this.pipe = "filter", this.priority = 1;
    }
    destroy() {
      for (let t = 0; t < this.filters.length; t++) this.filters[t].destroy();
      this.filters = null, this.filterArea = null;
    }
  };
  class $u {
    constructor() {
      this._effectClasses = [], this._tests = [], this._initialized = false;
    }
    init() {
      this._initialized || (this._initialized = true, this._effectClasses.forEach((t) => {
        this.add({
          test: t.test,
          maskClass: t
        });
      }));
    }
    add(t) {
      this._tests.push(t);
    }
    getMaskEffect(t) {
      this._initialized || this.init();
      for (let e = 0; e < this._tests.length; e++) {
        const s = this._tests[e];
        if (s.test(t)) return _e.get(s.maskClass, t);
      }
      return t;
    }
    returnMaskEffect(t) {
      _e.return(t);
    }
  }
  const Zr = new $u();
  Ot.handleByList(it.MaskEffect, Zr._effectClasses);
  const Bu = {
    _maskEffect: null,
    _maskOptions: {
      inverse: false
    },
    _filterEffect: null,
    effects: [],
    _markStructureAsChanged() {
      const n = this.renderGroup || this.parentRenderGroup;
      n && (n.structureDidChange = true);
    },
    addEffect(n) {
      this.effects.indexOf(n) === -1 && (this.effects.push(n), this.effects.sort((e, s) => e.priority - s.priority), this._markStructureAsChanged(), this._updateIsSimple());
    },
    removeEffect(n) {
      const t = this.effects.indexOf(n);
      t !== -1 && (this.effects.splice(t, 1), this._markStructureAsChanged(), this._updateIsSimple());
    },
    set mask(n) {
      const t = this._maskEffect;
      (t == null ? void 0 : t.mask) !== n && (t && (this.removeEffect(t), Zr.returnMaskEffect(t), this._maskEffect = null), n != null && (this._maskEffect = Zr.getMaskEffect(n), this.addEffect(this._maskEffect)));
    },
    get mask() {
      var _a2;
      return (_a2 = this._maskEffect) == null ? void 0 : _a2.mask;
    },
    setMask(n) {
      this._maskOptions = {
        ...this._maskOptions,
        ...n
      }, n.mask && (this.mask = n.mask), this._markStructureAsChanged();
    },
    set filters(n) {
      var _a2;
      !Array.isArray(n) && n && (n = [
        n
      ]);
      const t = this._filterEffect || (this._filterEffect = new Ea());
      n = n;
      const e = (n == null ? void 0 : n.length) > 0, s = ((_a2 = t.filters) == null ? void 0 : _a2.length) > 0, i = e !== s;
      n = Array.isArray(n) ? n.slice(0) : n, t.filters = Object.freeze(n), i && (e ? this.addEffect(t) : (this.removeEffect(t), t.filters = n ?? null));
    },
    get filters() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filters;
    },
    set filterArea(n) {
      this._filterEffect || (this._filterEffect = new Ea()), this._filterEffect.filterArea = n;
    },
    get filterArea() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filterArea;
    }
  }, Ou = {
    label: null,
    get name() {
      return Et(qt, "Container.name property has been removed, use Container.label instead"), this.label;
    },
    set name(n) {
      Et(qt, "Container.name property has been removed, use Container.label instead"), this.label = n;
    },
    getChildByName(n, t = false) {
      return this.getChildByLabel(n, t);
    },
    getChildByLabel(n, t = false) {
      const e = this.children;
      for (let s = 0; s < e.length; s++) {
        const i = e[s];
        if (i.label === n || n instanceof RegExp && n.test(i.label)) return i;
      }
      if (t) for (let s = 0; s < e.length; s++) {
        const r = e[s].getChildByLabel(n, true);
        if (r) return r;
      }
      return null;
    },
    getChildrenByLabel(n, t = false, e = []) {
      const s = this.children;
      for (let i = 0; i < s.length; i++) {
        const r = s[i];
        (r.label === n || n instanceof RegExp && n.test(r.label)) && e.push(r);
      }
      if (t) for (let i = 0; i < s.length; i++) s[i].getChildrenByLabel(n, true, e);
      return e;
    }
  }, me = _e.getPool(_t), hn = _e.getPool(Se), Fu = new _t(), Nu = {
    getFastGlobalBounds(n, t) {
      t || (t = new Se()), t.clear(), this._getGlobalBoundsRecursive(!!n, t, this.parentRenderLayer), t.isValid || t.set(0, 0, 0, 0);
      const e = this.renderGroup || this.parentRenderGroup;
      return t.applyMatrix(e.worldTransform), t;
    },
    _getGlobalBoundsRecursive(n, t, e) {
      let s = t;
      if (n && this.parentRenderLayer && this.parentRenderLayer !== e || this.localDisplayStatus !== 7 || !this.measurable) return;
      const i = !!this.effects.length;
      if ((this.renderGroup || i) && (s = hn.get().clear()), this.boundsArea) t.addRect(this.boundsArea, this.worldTransform);
      else {
        if (this.renderPipeId) {
          const o = this.bounds;
          s.addFrame(o.minX, o.minY, o.maxX, o.maxY, this.groupTransform);
        }
        const r = this.children;
        for (let o = 0; o < r.length; o++) r[o]._getGlobalBoundsRecursive(n, s, e);
      }
      if (i) {
        let r = false;
        const o = this.renderGroup || this.parentRenderGroup;
        for (let a = 0; a < this.effects.length; a++) this.effects[a].addBounds && (r || (r = true, s.applyMatrix(o.worldTransform)), this.effects[a].addBounds(s, true));
        r && s.applyMatrix(o.worldTransform.copyTo(Fu).invert()), t.addBounds(s), hn.return(s);
      } else this.renderGroup && (t.addBounds(s, this.relativeGroupTransform), hn.return(s));
    }
  };
  Lc = function(n, t, e) {
    e.clear();
    let s, i;
    return n.parent ? t ? s = n.parent.worldTransform : (i = me.get().identity(), s = Lo(n, i)) : s = _t.IDENTITY, $c(n, e, s, t), i && me.return(i), e.isValid || e.set(0, 0, 0, 0), e;
  };
  function $c(n, t, e, s) {
    var _a2, _b2;
    if (!n.visible || !n.measurable) return;
    let i;
    s ? i = n.worldTransform : (n.updateLocalTransform(), i = me.get(), i.appendFrom(n.localTransform, e));
    const r = t, o = !!n.effects.length;
    if (o && (t = hn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, i);
    else {
      const a = n.bounds;
      a && !a.isEmpty() && (t.matrix = i, t.addBounds(a));
      for (let l = 0; l < n.children.length; l++) $c(n.children[l], t, i, s);
    }
    if (o) {
      for (let a = 0; a < n.effects.length; a++) (_b2 = (_a2 = n.effects[a]).addBounds) == null ? void 0 : _b2.call(_a2, t);
      r.addBounds(t, _t.IDENTITY), hn.return(t);
    }
    s || me.return(i);
  }
  function Lo(n, t) {
    const e = n.parent;
    return e && (Lo(e, t), e.updateLocalTransform(), t.append(e.localTransform)), t;
  }
  Bc = function(n, t) {
    if (n === 16777215 || !t) return t;
    if (t === 16777215 || !n) return n;
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = t >> 16 & 255, o = t >> 8 & 255, a = t & 255, l = e * r / 255 | 0, c = s * o / 255 | 0, h = i * a / 255 | 0;
    return (l << 16) + (c << 8) + h;
  };
  const Ma = 16777215;
  Pa = function(n, t) {
    return n === Ma ? t : t === Ma ? n : Bc(n, t);
  };
  Os = function(n) {
    return ((n & 255) << 16) + (n & 65280) + (n >> 16 & 255);
  };
  const Wu = {
    getGlobalAlpha(n) {
      if (n) return this.renderGroup ? this.renderGroup.worldAlpha : this.parentRenderGroup ? this.parentRenderGroup.worldAlpha * this.alpha : this.alpha;
      let t = this.alpha, e = this.parent;
      for (; e; ) t *= e.alpha, e = e.parent;
      return t;
    },
    getGlobalTransform(n = new _t(), t) {
      if (t) return n.copyFrom(this.worldTransform);
      this.updateLocalTransform();
      const e = Lo(this, me.get().identity());
      return n.appendFrom(this.localTransform, e), me.return(e), n;
    },
    getGlobalTint(n) {
      if (n) return this.renderGroup ? Os(this.renderGroup.worldColor) : this.parentRenderGroup ? Os(Pa(this.localColor, this.parentRenderGroup.worldColor)) : this.tint;
      let t = this.localColor, e = this.parent;
      for (; e; ) t = Pa(t, e.localColor), e = e.parent;
      return Os(t);
    }
  };
  Oc = function(n, t, e) {
    return t.clear(), e || (e = _t.IDENTITY), Fc(n, t, e, n, true), t.isValid || t.set(0, 0, 0, 0), t;
  };
  function Fc(n, t, e, s, i) {
    var _a2, _b2;
    let r;
    if (i) r = me.get(), r = e.copyTo(r);
    else {
      if (!n.visible || !n.measurable) return;
      n.updateLocalTransform();
      const l = n.localTransform;
      r = me.get(), r.appendFrom(l, e);
    }
    const o = t, a = !!n.effects.length;
    if (a && (t = hn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, r);
    else {
      n.renderPipeId && (t.matrix = r, t.addBounds(n.bounds));
      const l = n.children;
      for (let c = 0; c < l.length; c++) Fc(l[c], t, r, s, false);
    }
    if (a) {
      for (let l = 0; l < n.effects.length; l++) (_b2 = (_a2 = n.effects[l]).addLocalBounds) == null ? void 0 : _b2.call(_a2, t, s);
      o.addBounds(t, _t.IDENTITY), hn.return(t);
    }
    me.return(r);
  }
  function Nc(n, t) {
    const e = n.children;
    for (let s = 0; s < e.length; s++) {
      const i = e[s], r = i.uid, o = (i._didViewChangeTick & 65535) << 16 | i._didContainerChangeTick & 65535, a = t.index;
      (t.data[a] !== r || t.data[a + 1] !== o) && (t.data[t.index] = r, t.data[t.index + 1] = o, t.didChange = true), t.index = a + 2, i.children.length && Nc(i, t);
    }
    return t.didChange;
  }
  const Gu = new _t(), zu = {
    _localBoundsCacheId: -1,
    _localBoundsCacheData: null,
    _setWidth(n, t) {
      const e = Math.sign(this.scale.x) || 1;
      t !== 0 ? this.scale.x = n / t * e : this.scale.x = e;
    },
    _setHeight(n, t) {
      const e = Math.sign(this.scale.y) || 1;
      t !== 0 ? this.scale.y = n / t * e : this.scale.y = e;
    },
    getLocalBounds() {
      this._localBoundsCacheData || (this._localBoundsCacheData = {
        data: [],
        index: 1,
        didChange: false,
        localBounds: new Se()
      });
      const n = this._localBoundsCacheData;
      return n.index = 1, n.didChange = false, n.data[0] !== this._didViewChangeTick && (n.didChange = true, n.data[0] = this._didViewChangeTick), Nc(this, n), n.didChange && Oc(this, n.localBounds, Gu), n.localBounds;
    },
    getBounds(n, t) {
      return Lc(this, n, t || new Se());
    }
  }, Du = {
    _onRender: null,
    set onRender(n) {
      const t = this.renderGroup || this.parentRenderGroup;
      if (!n) {
        this._onRender && (t == null ? void 0 : t.removeOnRender(this)), this._onRender = null;
        return;
      }
      this._onRender || (t == null ? void 0 : t.addOnRender(this)), this._onRender = n;
    },
    get onRender() {
      return this._onRender;
    }
  }, Hu = {
    _zIndex: 0,
    sortDirty: false,
    sortableChildren: false,
    get zIndex() {
      return this._zIndex;
    },
    set zIndex(n) {
      this._zIndex !== n && (this._zIndex = n, this.depthOfChildModified());
    },
    depthOfChildModified() {
      this.parent && (this.parent.sortableChildren = true, this.parent.sortDirty = true), this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true);
    },
    sortChildren() {
      this.sortDirty && (this.sortDirty = false, this.children.sort(Uu));
    }
  };
  function Uu(n, t) {
    return n._zIndex - t._zIndex;
  }
  const ju = {
    getGlobalPosition(n = new At(), t = false) {
      return this.parent ? this.parent.toGlobal(this._position, n, t) : (n.x = this._position.x, n.y = this._position.y), n;
    },
    toGlobal(n, t, e = false) {
      const s = this.getGlobalTransform(me.get(), e);
      return t = s.apply(n, t), me.return(s), t;
    },
    toLocal(n, t, e, s) {
      t && (n = t.toGlobal(n, e, s));
      const i = this.getGlobalTransform(me.get(), s);
      return e = i.applyInverse(n, e), me.return(i), e;
    }
  };
  class $o {
    constructor() {
      this.uid = Kt("instructionSet"), this.instructions = [], this.instructionSize = 0, this.renderables = [], this.gcTick = 0;
    }
    reset() {
      this.instructionSize = 0;
    }
    destroy() {
      this.instructions.length = 0, this.renderables.length = 0, this.renderPipes = null, this.gcTick = 0;
    }
    add(t) {
      this.instructions[this.instructionSize++] = t;
    }
    log() {
      this.instructions.length = this.instructionSize, console.table(this.instructions, [
        "type",
        "action"
      ]);
    }
  }
  let Vu = 0;
  class Yu {
    constructor(t) {
      this._poolKeyHash = /* @__PURE__ */ Object.create(null), this._texturePool = {}, this.textureOptions = t || {}, this.enableFullScreen = false, this.textureStyle = new Hn(this.textureOptions);
    }
    createTexture(t, e, s, i) {
      const r = new ke({
        ...this.textureOptions,
        width: t,
        height: e,
        resolution: 1,
        antialias: s,
        autoGarbageCollect: false,
        autoGenerateMipmaps: i
      });
      return new Ct({
        source: r,
        label: `texturePool_${Vu++}`
      });
    }
    getOptimalTexture(t, e, s = 1, i, r = false) {
      let o = Math.ceil(t * s - 1e-6), a = Math.ceil(e * s - 1e-6);
      o = us(o), a = us(a);
      const l = i ? 1 : 0, c = r ? 1 : 0, h = (o << 17) + (a << 2) + (c << 1) + l;
      this._texturePool[h] || (this._texturePool[h] = []);
      let d = this._texturePool[h].pop();
      return d || (d = this.createTexture(o, a, i, r)), d.source._resolution = s, d.source.width = o / s, d.source.height = a / s, d.source.pixelWidth = o, d.source.pixelHeight = a, d.frame.x = 0, d.frame.y = 0, d.frame.width = t, d.frame.height = e, d.updateUvs(), this._poolKeyHash[d.uid] = h, d;
    }
    getSameSizeTexture(t, e = false) {
      const s = t.source;
      return this.getOptimalTexture(t.width, t.height, s._resolution, e);
    }
    returnTexture(t, e = false) {
      const s = this._poolKeyHash[t.uid];
      e && (t.source.style = this.textureStyle), this._texturePool[s].push(t);
    }
    clear(t) {
      if (t = t !== false, t) for (const e in this._texturePool) {
        const s = this._texturePool[e];
        if (s) for (let i = 0; i < s.length; i++) s[i].destroy(true);
      }
      this._texturePool = {};
    }
  }
  ji = new Yu();
  Qs.register(ji);
  Xu = class {
    constructor() {
      this.renderPipeId = "renderGroup", this.root = null, this.canBundle = false, this.renderGroupParent = null, this.renderGroupChildren = [], this.worldTransform = new _t(), this.worldColorAlpha = 4294967295, this.worldColor = 16777215, this.worldAlpha = 1, this.childrenToUpdate = /* @__PURE__ */ Object.create(null), this.updateTick = 0, this.gcTick = 0, this.childrenRenderablesToUpdate = {
        list: [],
        index: 0
      }, this.structureDidChange = true, this.instructionSet = new $o(), this._onRenderContainers = [], this.textureNeedsUpdate = true, this.isCachedAsTexture = false, this._matrixDirty = 7;
    }
    init(t) {
      this.root = t, t._onRender && this.addOnRender(t), t.didChange = true;
      const e = t.children;
      for (let s = 0; s < e.length; s++) {
        const i = e[s];
        i._updateFlags = 15, this.addChild(i);
      }
    }
    enableCacheAsTexture(t = {}) {
      this.textureOptions = t, this.isCachedAsTexture = true, this.textureNeedsUpdate = true;
    }
    disableCacheAsTexture() {
      this.isCachedAsTexture = false, this.texture && (ji.returnTexture(this.texture, true), this.texture = null);
    }
    updateCacheTexture() {
      this.textureNeedsUpdate = true;
      const t = this._parentCacheAsTextureRenderGroup;
      t && !t.textureNeedsUpdate && t.updateCacheTexture();
    }
    reset() {
      this.renderGroupChildren.length = 0;
      for (const t in this.childrenToUpdate) {
        const e = this.childrenToUpdate[t];
        e.list.fill(null), e.index = 0;
      }
      this.childrenRenderablesToUpdate.index = 0, this.childrenRenderablesToUpdate.list.fill(null), this.root = null, this.updateTick = 0, this.structureDidChange = true, this._onRenderContainers.length = 0, this.renderGroupParent = null, this.disableCacheAsTexture();
    }
    get localTransform() {
      return this.root.localTransform;
    }
    addRenderGroupChild(t) {
      t.renderGroupParent && t.renderGroupParent._removeRenderGroupChild(t), t.renderGroupParent = this, this.renderGroupChildren.push(t);
    }
    _removeRenderGroupChild(t) {
      const e = this.renderGroupChildren.indexOf(t);
      e > -1 && this.renderGroupChildren.splice(e, 1), t.renderGroupParent = null;
    }
    addChild(t) {
      if (this.structureDidChange = true, t.parentRenderGroup = this, t.updateTick = -1, t.parent === this.root ? t.relativeRenderGroupDepth = 1 : t.relativeRenderGroupDepth = t.parent.relativeRenderGroupDepth + 1, t.didChange = true, this.onChildUpdate(t), t.renderGroup) {
        this.addRenderGroupChild(t.renderGroup);
        return;
      }
      t._onRender && this.addOnRender(t);
      const e = t.children;
      for (let s = 0; s < e.length; s++) this.addChild(e[s]);
    }
    removeChild(t) {
      if (this.structureDidChange = true, t._onRender && (t.renderGroup || this.removeOnRender(t)), t.parentRenderGroup = null, t.renderGroup) {
        this._removeRenderGroupChild(t.renderGroup);
        return;
      }
      const e = t.children;
      for (let s = 0; s < e.length; s++) this.removeChild(e[s]);
    }
    removeChildren(t) {
      for (let e = 0; e < t.length; e++) this.removeChild(t[e]);
    }
    onChildUpdate(t) {
      let e = this.childrenToUpdate[t.relativeRenderGroupDepth];
      e || (e = this.childrenToUpdate[t.relativeRenderGroupDepth] = {
        index: 0,
        list: []
      }), e.list[e.index++] = t;
    }
    updateRenderable(t) {
      t.globalDisplayStatus < 7 || (this.instructionSet.renderPipes[t.renderPipeId].updateRenderable(t), t.didViewUpdate = false);
    }
    onChildViewUpdate(t) {
      this.childrenRenderablesToUpdate.list[this.childrenRenderablesToUpdate.index++] = t;
    }
    get isRenderable() {
      return this.root.localDisplayStatus === 7 && this.worldAlpha > 0;
    }
    addOnRender(t) {
      this._onRenderContainers.push(t);
    }
    removeOnRender(t) {
      this._onRenderContainers.splice(this._onRenderContainers.indexOf(t), 1);
    }
    runOnRender(t) {
      for (let e = 0; e < this._onRenderContainers.length; e++) this._onRenderContainers[e]._onRender(t);
    }
    destroy() {
      this.disableCacheAsTexture(), this.renderGroupParent = null, this.root = null, this.childrenRenderablesToUpdate = null, this.childrenToUpdate = null, this.renderGroupChildren = null, this._onRenderContainers = null, this.instructionSet = null;
    }
    getChildren(t = []) {
      const e = this.root.children;
      for (let s = 0; s < e.length; s++) this._getChildren(e[s], t);
      return t;
    }
    _getChildren(t, e = []) {
      if (e.push(t), t.renderGroup) return e;
      const s = t.children;
      for (let i = 0; i < s.length; i++) this._getChildren(s[i], e);
      return e;
    }
    invalidateMatrices() {
      this._matrixDirty = 7;
    }
    get inverseWorldTransform() {
      return (this._matrixDirty & 1) === 0 ? this._inverseWorldTransform : (this._matrixDirty &= -2, this._inverseWorldTransform || (this._inverseWorldTransform = new _t()), this._inverseWorldTransform.copyFrom(this.worldTransform).invert());
    }
    get textureOffsetInverseTransform() {
      return (this._matrixDirty & 2) === 0 ? this._textureOffsetInverseTransform : (this._matrixDirty &= -3, this._textureOffsetInverseTransform || (this._textureOffsetInverseTransform = new _t()), this._textureOffsetInverseTransform.copyFrom(this.inverseWorldTransform).translate(-this._textureBounds.x, -this._textureBounds.y));
    }
    get inverseParentTextureTransform() {
      if ((this._matrixDirty & 4) === 0) return this._inverseParentTextureTransform;
      this._matrixDirty &= -5;
      const t = this._parentCacheAsTextureRenderGroup;
      return t ? (this._inverseParentTextureTransform || (this._inverseParentTextureTransform = new _t()), this._inverseParentTextureTransform.copyFrom(this.worldTransform).prepend(t.inverseWorldTransform).translate(-t._textureBounds.x, -t._textureBounds.y)) : this.worldTransform;
    }
    get cacheToLocalTransform() {
      return this.isCachedAsTexture ? this.textureOffsetInverseTransform : this._parentCacheAsTextureRenderGroup ? this._parentCacheAsTextureRenderGroup.textureOffsetInverseTransform : null;
    }
  };
  function Qr(n, t, e = {}) {
    for (const s in t) !e[s] && t[s] !== void 0 && (n[s] = t[s]);
  }
  let ur, ai, pr, li;
  ur = new le(null);
  ai = new le(null);
  pr = new le(null, 1, 1);
  li = new le(null);
  Ia = 1;
  qu = 2;
  fr = 4;
  zt = class extends Qe {
    constructor(t = {}) {
      var _a2, _b2;
      super(), this.uid = Kt("renderable"), this._updateFlags = 15, this.renderGroup = null, this.parentRenderGroup = null, this.parentRenderGroupIndex = 0, this.didChange = false, this.didViewUpdate = false, this.relativeRenderGroupDepth = 0, this.children = [], this.parent = null, this.includeInBuild = true, this.measurable = true, this.isSimple = true, this.parentRenderLayer = null, this.updateTick = -1, this.localTransform = new _t(), this.relativeGroupTransform = new _t(), this.groupTransform = this.relativeGroupTransform, this.destroyed = false, this._position = new le(this, 0, 0), this._scale = pr, this._pivot = ai, this._origin = li, this._skew = ur, this._cx = 1, this._sx = 0, this._cy = 0, this._sy = 1, this._rotation = 0, this.localColor = 16777215, this.localAlpha = 1, this.groupAlpha = 1, this.groupColor = 16777215, this.groupColorAlpha = 4294967295, this.localBlendMode = "inherit", this.groupBlendMode = "normal", this.localDisplayStatus = 7, this.globalDisplayStatus = 7, this._didContainerChangeTick = 0, this._didViewChangeTick = 0, this._didLocalTransformChangeId = -1, this.effects = [], Qr(this, t, {
        children: true,
        parent: true,
        effects: true
      }), (_a2 = t.children) == null ? void 0 : _a2.forEach((e) => this.addChild(e)), (_b2 = t.parent) == null ? void 0 : _b2.addChild(this);
    }
    static mixin(t) {
      Et("8.8.0", "Container.mixin is deprecated, please use extensions.mixin instead."), Ot.mixin(zt, t);
    }
    set _didChangeId(t) {
      this._didViewChangeTick = t >> 12 & 4095, this._didContainerChangeTick = t & 4095;
    }
    get _didChangeId() {
      return this._didContainerChangeTick & 4095 | (this._didViewChangeTick & 4095) << 12;
    }
    addChild(...t) {
      if (this.allowChildren || Et(qt, "addChild: Only Containers will be allowed to add children in v8.0.0"), t.length > 1) {
        for (let i = 0; i < t.length; i++) this.addChild(t[i]);
        return t[0];
      }
      const e = t[0], s = this.renderGroup || this.parentRenderGroup;
      return e.parent === this ? (this.children.splice(this.children.indexOf(e), 1), this.children.push(e), s && (s.structureDidChange = true), e) : (e.parent && e.parent.removeChild(e), this.children.push(e), this.sortableChildren && (this.sortDirty = true), e.parent = this, e.didChange = true, e._updateFlags = 15, s && s.addChild(e), this.emit("childAdded", e, this, this.children.length - 1), e.emit("added", this), this._didViewChangeTick++, e._zIndex !== 0 && e.depthOfChildModified(), e);
    }
    removeChild(...t) {
      if (t.length > 1) {
        for (let i = 0; i < t.length; i++) this.removeChild(t[i]);
        return t[0];
      }
      const e = t[0], s = this.children.indexOf(e);
      return s > -1 && (this._didViewChangeTick++, this.children.splice(s, 1), this.renderGroup ? this.renderGroup.removeChild(e) : this.parentRenderGroup && this.parentRenderGroup.removeChild(e), e.parentRenderLayer && e.parentRenderLayer.detach(e), e.parent = null, this.emit("childRemoved", e, this, s), e.emit("removed", this)), e;
    }
    _onUpdate(t) {
      t && t === this._skew && this._updateSkew(), this._didContainerChangeTick++, !this.didChange && (this.didChange = true, this.parentRenderGroup && this.parentRenderGroup.onChildUpdate(this));
    }
    set isRenderGroup(t) {
      !!this.renderGroup !== t && (t ? this.enableRenderGroup() : this.disableRenderGroup());
    }
    get isRenderGroup() {
      return !!this.renderGroup;
    }
    enableRenderGroup() {
      if (this.renderGroup) return;
      const t = this.parentRenderGroup;
      t == null ? void 0 : t.removeChild(this), this.renderGroup = _e.get(Xu, this), this.groupTransform = _t.IDENTITY, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
    }
    disableRenderGroup() {
      if (!this.renderGroup) return;
      const t = this.parentRenderGroup;
      t == null ? void 0 : t.removeChild(this), _e.return(this.renderGroup), this.renderGroup = null, this.groupTransform = this.relativeGroupTransform, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
    }
    _updateIsSimple() {
      this.isSimple = !this.renderGroup && this.effects.length === 0;
    }
    get worldTransform() {
      return this._worldTransform || (this._worldTransform = new _t()), this.renderGroup ? this._worldTransform.copyFrom(this.renderGroup.worldTransform) : this.parentRenderGroup && this._worldTransform.appendFrom(this.relativeGroupTransform, this.parentRenderGroup.worldTransform), this._worldTransform;
    }
    get x() {
      return this._position.x;
    }
    set x(t) {
      this._position.x = t;
    }
    get y() {
      return this._position.y;
    }
    set y(t) {
      this._position.y = t;
    }
    get position() {
      return this._position;
    }
    set position(t) {
      this._position.copyFrom(t);
    }
    get rotation() {
      return this._rotation;
    }
    set rotation(t) {
      this._rotation !== t && (this._rotation = t, this._onUpdate(this._skew));
    }
    get angle() {
      return this.rotation * hu;
    }
    set angle(t) {
      this.rotation = t * du;
    }
    get pivot() {
      return this._pivot === ai && (this._pivot = new le(this, 0, 0)), this._pivot;
    }
    set pivot(t) {
      this._pivot === ai && (this._pivot = new le(this, 0, 0), this._origin !== li && Vt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._pivot.set(t) : this._pivot.copyFrom(t);
    }
    get skew() {
      return this._skew === ur && (this._skew = new le(this, 0, 0)), this._skew;
    }
    set skew(t) {
      this._skew === ur && (this._skew = new le(this, 0, 0)), this._skew.copyFrom(t);
    }
    get scale() {
      return this._scale === pr && (this._scale = new le(this, 1, 1)), this._scale;
    }
    set scale(t) {
      this._scale === pr && (this._scale = new le(this, 0, 0)), typeof t == "string" && (t = parseFloat(t)), typeof t == "number" ? this._scale.set(t) : this._scale.copyFrom(t);
    }
    get origin() {
      return this._origin === li && (this._origin = new le(this, 0, 0)), this._origin;
    }
    set origin(t) {
      this._origin === li && (this._origin = new le(this, 0, 0), this._pivot !== ai && Vt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._origin.set(t) : this._origin.copyFrom(t);
    }
    get width() {
      return Math.abs(this.scale.x * this.getLocalBounds().width);
    }
    set width(t) {
      const e = this.getLocalBounds().width;
      this._setWidth(t, e);
    }
    get height() {
      return Math.abs(this.scale.y * this.getLocalBounds().height);
    }
    set height(t) {
      const e = this.getLocalBounds().height;
      this._setHeight(t, e);
    }
    getSize(t) {
      t || (t = {});
      const e = this.getLocalBounds();
      return t.width = Math.abs(this.scale.x * e.width), t.height = Math.abs(this.scale.y * e.height), t;
    }
    setSize(t, e) {
      const s = this.getLocalBounds();
      typeof t == "object" ? (e = t.height ?? t.width, t = t.width) : e ?? (e = t), t !== void 0 && this._setWidth(t, s.width), e !== void 0 && this._setHeight(e, s.height);
    }
    _updateSkew() {
      const t = this._rotation, e = this._skew;
      this._cx = Math.cos(t + e._y), this._sx = Math.sin(t + e._y), this._cy = -Math.sin(t - e._x), this._sy = Math.cos(t - e._x);
    }
    updateTransform(t) {
      return this.position.set(typeof t.x == "number" ? t.x : this.position.x, typeof t.y == "number" ? t.y : this.position.y), this.scale.set(typeof t.scaleX == "number" ? t.scaleX || 1 : this.scale.x, typeof t.scaleY == "number" ? t.scaleY || 1 : this.scale.y), this.rotation = typeof t.rotation == "number" ? t.rotation : this.rotation, this.skew.set(typeof t.skewX == "number" ? t.skewX : this.skew.x, typeof t.skewY == "number" ? t.skewY : this.skew.y), this.pivot.set(typeof t.pivotX == "number" ? t.pivotX : this.pivot.x, typeof t.pivotY == "number" ? t.pivotY : this.pivot.y), this.origin.set(typeof t.originX == "number" ? t.originX : this.origin.x, typeof t.originY == "number" ? t.originY : this.origin.y), this;
    }
    setFromMatrix(t) {
      t.decompose(this);
    }
    updateLocalTransform() {
      const t = this._didContainerChangeTick;
      if (this._didLocalTransformChangeId === t) return;
      this._didLocalTransformChangeId = t;
      const e = this.localTransform, s = this._scale, i = this._pivot, r = this._origin, o = this._position, a = s._x, l = s._y, c = i._x, h = i._y, d = -r._x, u = -r._y;
      e.a = this._cx * a, e.b = this._sx * a, e.c = this._cy * l, e.d = this._sy * l, e.tx = o._x - (c * e.a + h * e.c) + (d * e.a + u * e.c) - d, e.ty = o._y - (c * e.b + h * e.d) + (d * e.b + u * e.d) - u;
    }
    set alpha(t) {
      t !== this.localAlpha && (this.localAlpha = t, this._updateFlags |= Ia, this._onUpdate());
    }
    get alpha() {
      return this.localAlpha;
    }
    set tint(t) {
      const s = Gt.shared.setValue(t ?? 16777215).toBgrNumber();
      s !== this.localColor && (this.localColor = s, this._updateFlags |= Ia, this._onUpdate());
    }
    get tint() {
      return Os(this.localColor);
    }
    set blendMode(t) {
      this.localBlendMode !== t && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= qu, this.localBlendMode = t, this._onUpdate());
    }
    get blendMode() {
      return this.localBlendMode;
    }
    get visible() {
      return !!(this.localDisplayStatus & 2);
    }
    set visible(t) {
      const e = t ? 2 : 0;
      (this.localDisplayStatus & 2) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= fr, this.localDisplayStatus ^= 2, this._onUpdate(), this.emit("visibleChanged", t));
    }
    get culled() {
      return !(this.localDisplayStatus & 4);
    }
    set culled(t) {
      const e = t ? 0 : 4;
      (this.localDisplayStatus & 4) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= fr, this.localDisplayStatus ^= 4, this._onUpdate());
    }
    get renderable() {
      return !!(this.localDisplayStatus & 1);
    }
    set renderable(t) {
      const e = t ? 1 : 0;
      (this.localDisplayStatus & 1) !== e && (this._updateFlags |= fr, this.localDisplayStatus ^= 1, this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._onUpdate());
    }
    get isRenderable() {
      return this.localDisplayStatus === 7 && this.groupAlpha > 0;
    }
    destroy(t = false) {
      var _a2;
      if (this.destroyed) return;
      this.destroyed = true;
      let e;
      if (this.children.length && (e = this.removeChildren(0, this.children.length)), this.removeFromParent(), this.parent = null, this._maskEffect = null, this._filterEffect = null, this.effects = null, this._position = null, this._scale = null, this._pivot = null, this._origin = null, this._skew = null, this.emit("destroyed", this), this.removeAllListeners(), (typeof t == "boolean" ? t : t == null ? void 0 : t.children) && e) for (let i = 0; i < e.length; ++i) e[i].destroy(t);
      (_a2 = this.renderGroup) == null ? void 0 : _a2.destroy(), this.renderGroup = null;
    }
  };
  Ot.mixin(zt, Ru, Nu, ju, Du, zu, Bu, Ou, Hu, Au, Pu, Wu, Lu);
  class Vi extends zt {
    constructor(t) {
      super(t), this.canBundle = true, this.allowChildren = false, this._roundPixels = 0, this._lastUsed = -1, this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this._bounds = new Se(0, 1, 0, 0), this._boundsDirty = true, this.autoGarbageCollect = t.autoGarbageCollect ?? true;
    }
    get bounds() {
      return this._boundsDirty ? (this.updateBounds(), this._boundsDirty = false, this._bounds) : this._bounds;
    }
    get roundPixels() {
      return !!this._roundPixels;
    }
    set roundPixels(t) {
      this._roundPixels = t ? 1 : 0;
    }
    containsPoint(t) {
      const e = this.bounds, { x: s, y: i } = t;
      return s >= e.minX && s <= e.maxX && i >= e.minY && i <= e.maxY;
    }
    onViewUpdate() {
      if (this._didViewChangeTick++, this._boundsDirty = true, this.didViewUpdate) return;
      this.didViewUpdate = true;
      const t = this.renderGroup || this.parentRenderGroup;
      t && t.onChildViewUpdate(this);
    }
    unload() {
      var _a2;
      this.emit("unload", this);
      for (const t in this._gpuData) (_a2 = this._gpuData[t]) == null ? void 0 : _a2.destroy();
      this._gpuData = /* @__PURE__ */ Object.create(null), this.onViewUpdate();
    }
    destroy(t) {
      this.unload(), super.destroy(t), this._bounds = null;
    }
    collectRenderablesSimple(t, e, s) {
      const { renderPipes: i } = e;
      i.blendMode.pushBlendMode(this, this.groupBlendMode, t);
      const o = i[this.renderPipeId];
      (o == null ? void 0 : o.addRenderable) && o.addRenderable(this, t), this.didViewUpdate = false;
      const a = this.children, l = a.length;
      for (let c = 0; c < l; c++) a[c].collectRenderables(t, e, s);
      i.blendMode.popBlendMode(t);
    }
  }
  De = class extends Vi {
    constructor(t = Ct.EMPTY) {
      t instanceof Ct && (t = {
        texture: t
      });
      const { texture: e = Ct.EMPTY, anchor: s, roundPixels: i, width: r, height: o, ...a } = t;
      super({
        label: "Sprite",
        ...a
      }), this.renderPipeId = "sprite", this.batched = true, this._visualBounds = {
        minX: 0,
        maxX: 1,
        minY: 0,
        maxY: 0
      }, this._anchor = new le({
        _onUpdate: () => {
          this.onViewUpdate();
        }
      }), s ? this.anchor = s : e.defaultAnchor && (this.anchor = e.defaultAnchor), this.texture = e, this.allowChildren = false, this.roundPixels = i ?? false, r !== void 0 && (this.width = r), o !== void 0 && (this.height = o);
    }
    static from(t, e = false) {
      return t instanceof Ct ? new De(t) : new De(Ct.from(t, e));
    }
    set texture(t) {
      t || (t = Ct.EMPTY);
      const e = this._texture;
      e !== t && (e && e.dynamic && e.off("update", this.onViewUpdate, this), t.dynamic && t.on("update", this.onViewUpdate, this), this._texture = t, this._width && this._setWidth(this._width, this._texture.orig.width), this._height && this._setHeight(this._height, this._texture.orig.height), this.onViewUpdate());
    }
    get texture() {
      return this._texture;
    }
    get visualBounds() {
      return Mc(this._visualBounds, this._anchor, this._texture), this._visualBounds;
    }
    get sourceBounds() {
      return Et("8.6.1", "Sprite.sourceBounds is deprecated, use visualBounds instead."), this.visualBounds;
    }
    updateBounds() {
      const t = this._anchor, e = this._texture, s = this._bounds, { width: i, height: r } = e.orig;
      s.minX = -t._x * i, s.maxX = s.minX + i, s.minY = -t._y * r, s.maxY = s.minY + r;
    }
    destroy(t = false) {
      if (super.destroy(t), typeof t == "boolean" ? t : t == null ? void 0 : t.texture) {
        const s = typeof t == "boolean" ? t : t == null ? void 0 : t.textureSource;
        this._texture.destroy(s);
      }
      this._texture = null, this._visualBounds = null, this._bounds = null, this._anchor = null;
    }
    get anchor() {
      return this._anchor;
    }
    set anchor(t) {
      typeof t == "number" ? this._anchor.set(t) : this._anchor.copyFrom(t);
    }
    get width() {
      return Math.abs(this.scale.x) * this._texture.orig.width;
    }
    set width(t) {
      this._setWidth(t, this._texture.orig.width), this._width = t;
    }
    get height() {
      return Math.abs(this.scale.y) * this._texture.orig.height;
    }
    set height(t) {
      this._setHeight(t, this._texture.orig.height), this._height = t;
    }
    getSize(t) {
      return t || (t = {}), t.width = Math.abs(this.scale.x) * this._texture.orig.width, t.height = Math.abs(this.scale.y) * this._texture.orig.height, t;
    }
    setSize(t, e) {
      typeof t == "object" ? (e = t.height ?? t.width, t = t.width) : e ?? (e = t), t !== void 0 && this._setWidth(t, this._texture.orig.width), e !== void 0 && this._setHeight(e, this._texture.orig.height);
    }
  };
  const Ku = new Se();
  function Wc(n, t, e) {
    const s = Ku;
    n.measurable = true, Lc(n, e, s), t.addBoundsMask(s), n.measurable = false;
  }
  function Gc(n, t, e) {
    const s = hn.get();
    n.measurable = true;
    const i = me.get().identity(), r = zc(n, e, i);
    Oc(n, s, r), n.measurable = false, t.addBoundsMask(s), me.return(i), hn.return(s);
  }
  function zc(n, t, e) {
    return n ? (n !== t && (zc(n.parent, t, e), n.updateLocalTransform(), e.append(n.localTransform)), e) : (Vt("Mask bounds, renderable is not inside the root container"), e);
  }
  class Dc {
    constructor(t) {
      this.priority = 0, this.inverse = false, this.pipe = "alphaMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t, this.renderMaskToTexture = !(t instanceof De), this.mask.renderable = this.renderMaskToTexture, this.mask.includeInBuild = !this.renderMaskToTexture, this.mask.measurable = false;
    }
    reset() {
      this.mask !== null && (this.mask.measurable = true, this.mask = null);
    }
    addBounds(t, e) {
      this.inverse || Wc(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      Gc(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof De;
    }
  }
  Dc.extension = it.MaskEffect;
  class Hc {
    constructor(t) {
      this.priority = 0, this.pipe = "colorMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t;
    }
    destroy() {
    }
    static test(t) {
      return typeof t == "number";
    }
  }
  Hc.extension = it.MaskEffect;
  class Uc {
    constructor(t) {
      this.priority = 0, this.pipe = "stencilMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t, this.mask.includeInBuild = false, this.mask.measurable = false;
    }
    reset() {
      this.mask !== null && (this.mask.measurable = true, this.mask.includeInBuild = true, this.mask = null);
    }
    addBounds(t, e) {
      Wc(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      Gc(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof zt;
    }
  }
  Uc.extension = it.MaskEffect;
  const Ju = {
    createCanvas: (n, t) => {
      const e = document.createElement("canvas");
      return e.width = n, e.height = t, e;
    },
    createImage: () => new Image(),
    getCanvasRenderingContext2D: () => CanvasRenderingContext2D,
    getWebGLRenderingContext: () => WebGLRenderingContext,
    getNavigator: () => navigator,
    getBaseUrl: () => document.baseURI ?? window.location.href,
    getFontFaceSet: () => document.fonts,
    fetch: (n, t) => fetch(n, t),
    parseXML: (n) => new DOMParser().parseFromString(n, "text/xml")
  };
  let Ra = Ju;
  Mt = {
    get() {
      return Ra;
    },
    set(n) {
      Ra = n;
    }
  };
  jc = class extends ke {
    constructor(t) {
      t.resource || (t.resource = Mt.get().createCanvas()), t.width || (t.width = t.resource.width, t.autoDensity || (t.width /= t.resolution)), t.height || (t.height = t.resource.height, t.autoDensity || (t.height /= t.resolution)), super(t), this.uploadMethodId = "image", this.autoDensity = t.autoDensity, this.resizeCanvas(), this.transparent = !!t.transparent;
    }
    resizeCanvas() {
      this.autoDensity && "style" in this.resource && (this.resource.style.width = `${this.width}px`, this.resource.style.height = `${this.height}px`), (this.resource.width !== this.pixelWidth || this.resource.height !== this.pixelHeight) && (this.resource.width = this.pixelWidth, this.resource.height = this.pixelHeight);
    }
    resize(t = this.width, e = this.height, s = this._resolution) {
      const i = super.resize(t, e, s);
      return i && this.resizeCanvas(), i;
    }
    static test(t) {
      return globalThis.HTMLCanvasElement && t instanceof HTMLCanvasElement || globalThis.OffscreenCanvas && t instanceof OffscreenCanvas;
    }
    get context2D() {
      return this._context2D || (this._context2D = this.resource.getContext("2d"));
    }
  };
  jc.extension = it.TextureSource;
  fs = class extends ke {
    constructor(t) {
      super(t), this.uploadMethodId = "image", this.autoGarbageCollect = true;
    }
    static test(t) {
      return globalThis.HTMLImageElement && t instanceof HTMLImageElement || typeof ImageBitmap < "u" && t instanceof ImageBitmap || globalThis.VideoFrame && t instanceof VideoFrame;
    }
  };
  fs.extension = it.TextureSource;
  Hs = ((n) => (n[n.INTERACTION = 50] = "INTERACTION", n[n.HIGH = 25] = "HIGH", n[n.NORMAL = 0] = "NORMAL", n[n.LOW = -25] = "LOW", n[n.UTILITY = -50] = "UTILITY", n))(Hs || {});
  class mr {
    constructor(t, e = null, s = 0, i = false) {
      this.next = null, this.previous = null, this._destroyed = false, this._fn = t, this._context = e, this.priority = s, this._once = i;
    }
    match(t, e = null) {
      return this._fn === t && this._context === e;
    }
    emit(t) {
      this._fn && (this._context ? this._fn.call(this._context, t) : this._fn(t));
      const e = this.next;
      return this._once && this.destroy(true), this._destroyed && (this.next = null), e;
    }
    connect(t) {
      this.previous = t, t.next && (t.next.previous = this), this.next = t.next, t.next = this;
    }
    destroy(t = false) {
      this._destroyed = true, this._fn = null, this._context = null, this.previous && (this.previous.next = this.next), this.next && (this.next.previous = this.previous);
      const e = this.next;
      return this.next = t ? null : e, this.previous = null, e;
    }
  }
  const Vc = class we {
    constructor() {
      this.autoStart = false, this.deltaTime = 1, this.lastTime = -1, this.speed = 1, this.started = false, this._requestId = null, this._maxElapsedMS = 100, this._minElapsedMS = 0, this._protected = false, this._lastFrame = -1, this._head = new mr(null, null, 1 / 0), this.deltaMS = 1 / we.targetFPMS, this.elapsedMS = 1 / we.targetFPMS, this._tick = (t) => {
        this._requestId = null, this.started && (this.update(t), this.started && this._requestId === null && this._head.next && (this._requestId = requestAnimationFrame(this._tick)));
      };
    }
    _requestIfNeeded() {
      this._requestId === null && this._head.next && (this.lastTime = performance.now(), this._lastFrame = this.lastTime, this._requestId = requestAnimationFrame(this._tick));
    }
    _cancelIfNeeded() {
      this._requestId !== null && (cancelAnimationFrame(this._requestId), this._requestId = null);
    }
    _startIfPossible() {
      this.started ? this._requestIfNeeded() : this.autoStart && this.start();
    }
    add(t, e, s = Hs.NORMAL) {
      return this._addListener(new mr(t, e, s));
    }
    addOnce(t, e, s = Hs.NORMAL) {
      return this._addListener(new mr(t, e, s, true));
    }
    _addListener(t) {
      let e = this._head.next, s = this._head;
      if (!e) t.connect(s);
      else {
        for (; e; ) {
          if (t.priority > e.priority) {
            t.connect(s);
            break;
          }
          s = e, e = e.next;
        }
        t.previous || t.connect(s);
      }
      return this._startIfPossible(), this;
    }
    remove(t, e) {
      let s = this._head.next;
      for (; s; ) s.match(t, e) ? s = s.destroy() : s = s.next;
      return this._head.next || this._cancelIfNeeded(), this;
    }
    get count() {
      if (!this._head) return 0;
      let t = 0, e = this._head;
      for (; e = e.next; ) t++;
      return t;
    }
    start() {
      this.started || (this.started = true, this._requestIfNeeded());
    }
    stop() {
      this.started && (this.started = false, this._cancelIfNeeded());
    }
    destroy() {
      if (!this._protected) {
        this.stop();
        let t = this._head.next;
        for (; t; ) t = t.destroy(true);
        this._head.destroy(), this._head = null;
      }
    }
    update(t = performance.now()) {
      let e;
      if (t > this.lastTime) {
        if (e = this.elapsedMS = t - this.lastTime, e > this._maxElapsedMS && (e = this._maxElapsedMS), e *= this.speed, this._minElapsedMS) {
          const r = t - this._lastFrame | 0;
          if (r < this._minElapsedMS) return;
          this._lastFrame = t - r % this._minElapsedMS;
        }
        this.deltaMS = e, this.deltaTime = this.deltaMS * we.targetFPMS;
        const s = this._head;
        let i = s.next;
        for (; i; ) i = i.emit(this);
        s.next || this._cancelIfNeeded();
      } else this.deltaTime = this.deltaMS = this.elapsedMS = 0;
      this.lastTime = t;
    }
    get FPS() {
      return 1e3 / this.elapsedMS;
    }
    get minFPS() {
      return 1e3 / this._maxElapsedMS;
    }
    set minFPS(t) {
      const e = Math.min(Math.max(0, t) / 1e3, we.targetFPMS);
      this._maxElapsedMS = 1 / e, this._minElapsedMS && t > this.maxFPS && (this.maxFPS = t);
    }
    get maxFPS() {
      return this._minElapsedMS ? Math.round(1e3 / this._minElapsedMS) : 0;
    }
    set maxFPS(t) {
      t === 0 ? this._minElapsedMS = 0 : (t < this.minFPS && (this.minFPS = t), this._minElapsedMS = 1 / (t / 1e3));
    }
    static get shared() {
      if (!we._shared) {
        const t = we._shared = new we();
        t.autoStart = true, t._protected = true;
      }
      return we._shared;
    }
    static get system() {
      if (!we._system) {
        const t = we._system = new we();
        t.autoStart = true, t._protected = true;
      }
      return we._system;
    }
  };
  Vc.targetFPMS = 0.06;
  let gr;
  Nn = Vc;
  async function Yc() {
    return gr ?? (gr = (async () => {
      var _a2;
      const t = Mt.get().createCanvas(1, 1).getContext("webgl");
      if (!t) return "premultiply-alpha-on-upload";
      const e = await new Promise((o) => {
        const a = document.createElement("video");
        a.onloadeddata = () => o(a), a.onerror = () => o(null), a.autoplay = false, a.crossOrigin = "anonymous", a.preload = "auto", a.src = "data:video/webm;base64,GkXfo59ChoEBQveBAULygQRC84EIQoKEd2VibUKHgQJChYECGFOAZwEAAAAAAAHTEU2bdLpNu4tTq4QVSalmU6yBoU27i1OrhBZUrmtTrIHGTbuMU6uEElTDZ1OsggEXTbuMU6uEHFO7a1OsggG97AEAAAAAAABZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAVSalmoCrXsYMPQkBNgIRMYXZmV0GETGF2ZkSJiEBEAAAAAAAAFlSua8yuAQAAAAAAAEPXgQFzxYgAAAAAAAAAAZyBACK1nIN1bmSIgQCGhVZfVlA5g4EBI+ODhAJiWgDglLCBArqBApqBAlPAgQFVsIRVuYEBElTDZ9Vzc9JjwItjxYgAAAAAAAAAAWfInEWjh0VOQ09ERVJEh49MYXZjIGxpYnZweC12cDlnyKJFo4hEVVJBVElPTkSHlDAwOjAwOjAwLjA0MDAwMDAwMAAAH0O2dcfngQCgwqGggQAAAIJJg0IAABAAFgA4JBwYSgAAICAAEb///4r+AAB1oZ2mm+6BAaWWgkmDQgAAEAAWADgkHBhKAAAgIABIQBxTu2uRu4+zgQC3iveBAfGCAXHwgQM=", a.load();
      });
      if (!e) return "premultiply-alpha-on-upload";
      const s = t.createTexture();
      t.bindTexture(t.TEXTURE_2D, s);
      const i = t.createFramebuffer();
      t.bindFramebuffer(t.FRAMEBUFFER, i), t.framebufferTexture2D(t.FRAMEBUFFER, t.COLOR_ATTACHMENT0, t.TEXTURE_2D, s, 0), t.pixelStorei(t.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false), t.pixelStorei(t.UNPACK_COLORSPACE_CONVERSION_WEBGL, t.NONE), t.texImage2D(t.TEXTURE_2D, 0, t.RGBA, t.RGBA, t.UNSIGNED_BYTE, e);
      const r = new Uint8Array(4);
      return t.readPixels(0, 0, 1, 1, t.RGBA, t.UNSIGNED_BYTE, r), t.deleteFramebuffer(i), t.deleteTexture(s), (_a2 = t.getExtension("WEBGL_lose_context")) == null ? void 0 : _a2.loseContext(), r[0] <= r[3] ? "premultiplied-alpha" : "premultiply-alpha-on-upload";
    })()), gr;
  }
  const Yi = class Xc extends ke {
    constructor(t) {
      super(t), this.isReady = false, this.uploadMethodId = "video", t = {
        ...Xc.defaultOptions,
        ...t
      }, this._autoUpdate = true, this._isConnectedToTicker = false, this._updateFPS = t.updateFPS || 0, this._msToNextUpdate = 0, this.autoPlay = t.autoPlay !== false, this.alphaMode = t.alphaMode ?? "premultiply-alpha-on-upload", this._videoFrameRequestCallback = this._videoFrameRequestCallback.bind(this), this._videoFrameRequestCallbackHandle = null, this._load = null, this._resolve = null, this._reject = null, this._onCanPlay = this._onCanPlay.bind(this), this._onCanPlayThrough = this._onCanPlayThrough.bind(this), this._onError = this._onError.bind(this), this._onPlayStart = this._onPlayStart.bind(this), this._onPlayStop = this._onPlayStop.bind(this), this._onSeeked = this._onSeeked.bind(this), t.autoLoad !== false && this.load();
    }
    updateFrame() {
      if (!this.destroyed) {
        if (this._updateFPS) {
          const t = Nn.shared.elapsedMS * this.resource.playbackRate;
          this._msToNextUpdate = Math.floor(this._msToNextUpdate - t);
        }
        (!this._updateFPS || this._msToNextUpdate <= 0) && (this._msToNextUpdate = this._updateFPS ? Math.floor(1e3 / this._updateFPS) : 0), this.isValid && this.update();
      }
    }
    _videoFrameRequestCallback() {
      this.updateFrame(), this.destroyed ? this._videoFrameRequestCallbackHandle = null : this._videoFrameRequestCallbackHandle = this.resource.requestVideoFrameCallback(this._videoFrameRequestCallback);
    }
    get isValid() {
      return !!this.resource.videoWidth && !!this.resource.videoHeight;
    }
    async load() {
      if (this._load) return this._load;
      const t = this.resource, e = this.options;
      return (t.readyState === t.HAVE_ENOUGH_DATA || t.readyState === t.HAVE_FUTURE_DATA) && t.width && t.height && (t.complete = true), t.addEventListener("play", this._onPlayStart), t.addEventListener("pause", this._onPlayStop), t.addEventListener("seeked", this._onSeeked), this._isSourceReady() ? this._mediaReady() : (e.preload || t.addEventListener("canplay", this._onCanPlay), t.addEventListener("canplaythrough", this._onCanPlayThrough), t.addEventListener("error", this._onError, true)), this.alphaMode = await Yc(), this._load = new Promise((s, i) => {
        this.isValid ? s(this) : (this._resolve = s, this._reject = i, e.preloadTimeoutMs !== void 0 && (this._preloadTimeout = setTimeout(() => {
          this._onError(new ErrorEvent(`Preload exceeded timeout of ${e.preloadTimeoutMs}ms`));
        })), t.load());
      }), this._load;
    }
    _onError(t) {
      this.resource.removeEventListener("error", this._onError, true), this.emit("error", t), this._reject && (this._reject(t), this._reject = null, this._resolve = null);
    }
    _isSourcePlaying() {
      const t = this.resource;
      return !t.paused && !t.ended;
    }
    _isSourceReady() {
      return this.resource.readyState > 2;
    }
    _onPlayStart() {
      this.isValid || this._mediaReady(), this._configureAutoUpdate();
    }
    _onPlayStop() {
      this._configureAutoUpdate();
    }
    _onSeeked() {
      this._autoUpdate && !this._isSourcePlaying() && (this._msToNextUpdate = 0, this.updateFrame(), this._msToNextUpdate = 0);
    }
    _onCanPlay() {
      this.resource.removeEventListener("canplay", this._onCanPlay), this._mediaReady();
    }
    _onCanPlayThrough() {
      this.resource.removeEventListener("canplaythrough", this._onCanPlay), this._preloadTimeout && (clearTimeout(this._preloadTimeout), this._preloadTimeout = void 0), this._mediaReady();
    }
    _mediaReady() {
      const t = this.resource;
      this.isValid && (this.isReady = true, this.resize(t.videoWidth, t.videoHeight)), this._msToNextUpdate = 0, this.updateFrame(), this._msToNextUpdate = 0, this._resolve && (this._resolve(this), this._resolve = null, this._reject = null), this._isSourcePlaying() ? this._onPlayStart() : this.autoPlay && this.resource.play();
    }
    destroy() {
      this._configureAutoUpdate();
      const t = this.resource;
      t && (t.removeEventListener("play", this._onPlayStart), t.removeEventListener("pause", this._onPlayStop), t.removeEventListener("seeked", this._onSeeked), t.removeEventListener("canplay", this._onCanPlay), t.removeEventListener("canplaythrough", this._onCanPlayThrough), t.removeEventListener("error", this._onError, true), t.pause(), t.src = "", t.load()), super.destroy();
    }
    get autoUpdate() {
      return this._autoUpdate;
    }
    set autoUpdate(t) {
      t !== this._autoUpdate && (this._autoUpdate = t, this._configureAutoUpdate());
    }
    get updateFPS() {
      return this._updateFPS;
    }
    set updateFPS(t) {
      t !== this._updateFPS && (this._updateFPS = t, this._configureAutoUpdate());
    }
    _configureAutoUpdate() {
      this._autoUpdate && this._isSourcePlaying() ? !this._updateFPS && this.resource.requestVideoFrameCallback ? (this._isConnectedToTicker && (Nn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0), this._videoFrameRequestCallbackHandle === null && (this._videoFrameRequestCallbackHandle = this.resource.requestVideoFrameCallback(this._videoFrameRequestCallback))) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker || (Nn.shared.add(this.updateFrame, this), this._isConnectedToTicker = true, this._msToNextUpdate = 0)) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker && (Nn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0));
    }
    static test(t) {
      return globalThis.HTMLVideoElement && t instanceof HTMLVideoElement;
    }
  };
  Yi.extension = it.TextureSource;
  Yi.defaultOptions = {
    ...ke.defaultOptions,
    autoLoad: true,
    autoPlay: true,
    updateFPS: 0,
    crossorigin: true,
    loop: false,
    muted: true,
    playsinline: true,
    preload: false
  };
  Yi.MIME_TYPES = {
    ogv: "video/ogg",
    mov: "video/quicktime",
    m4v: "video/mp4"
  };
  let Fs = Yi;
  const We = (n, t, e = false) => (Array.isArray(n) || (n = [
    n
  ]), t ? n.map((s) => typeof s == "string" || e ? t(s) : s) : n);
  class Zu {
    constructor() {
      this._parsers = [], this._cache = /* @__PURE__ */ new Map(), this._cacheMap = /* @__PURE__ */ new Map();
    }
    reset() {
      this._cacheMap.clear(), this._cache.clear();
    }
    has(t) {
      return this._cache.has(t);
    }
    get(t) {
      const e = this._cache.get(t);
      return e || Vt(`[Assets] Asset id ${t} was not found in the Cache`), e;
    }
    set(t, e) {
      const s = We(t);
      let i;
      for (let l = 0; l < this.parsers.length; l++) {
        const c = this.parsers[l];
        if (c.test(e)) {
          i = c.getCacheableAssets(s, e);
          break;
        }
      }
      const r = new Map(Object.entries(i || {}));
      i || s.forEach((l) => {
        r.set(l, e);
      });
      const o = [
        ...r.keys()
      ], a = {
        cacheKeys: o,
        keys: s
      };
      s.forEach((l) => {
        this._cacheMap.set(l, a);
      }), o.forEach((l) => {
        const c = i ? i[l] : e;
        this._cache.has(l) && this._cache.get(l) !== c && Vt("[Cache] already has key:", l), this._cache.set(l, r.get(l));
      });
    }
    remove(t) {
      if (!this._cacheMap.has(t)) {
        Vt(`[Assets] Asset id ${t} was not found in the Cache`);
        return;
      }
      const e = this._cacheMap.get(t);
      e.cacheKeys.forEach((i) => {
        this._cache.delete(i);
      }), e.keys.forEach((i) => {
        this._cacheMap.delete(i);
      });
    }
    get parsers() {
      return this._parsers;
    }
  }
  let to;
  te = new Zu();
  to = [];
  Ot.handleByList(it.TextureSource, to);
  function qc(n = {}) {
    const t = n && n.resource, e = t ? n.resource : n, s = t ? n : {
      resource: n
    };
    for (let i = 0; i < to.length; i++) {
      const r = to[i];
      if (r.test(e)) return new r(s);
    }
    throw new Error(`Could not find a source type for resource: ${s.resource}`);
  }
  function Qu(n = {}, t = false) {
    const e = n && n.resource, s = e ? n.resource : n, i = e ? n : {
      resource: n
    };
    if (!t && te.has(s)) return te.get(s);
    const r = new Ct({
      source: qc(i)
    });
    return r.on("destroy", () => {
      te.has(s) && te.remove(s);
    }), t || te.set(s, r), r;
  }
  function tp(n, t = false) {
    return typeof n == "string" ? te.get(n) : n instanceof ke ? new Ct({
      source: n
    }) : Qu(n, t);
  }
  Ct.from = tp;
  ke.from = qc;
  Ot.add(Dc, Hc, Uc, Fs, fs, jc, Ro);
  var kn = ((n) => (n[n.Low = 0] = "Low", n[n.Normal = 1] = "Normal", n[n.High = 2] = "High", n))(kn || {});
  function Oe(n) {
    if (typeof n != "string") throw new TypeError(`Path must be a string. Received ${JSON.stringify(n)}`);
  }
  function vs(n) {
    return n.split("?")[0].split("#")[0];
  }
  function ep(n) {
    return n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  function np(n, t, e) {
    return n.replace(new RegExp(ep(t), "g"), e);
  }
  function sp(n, t) {
    let e = "", s = 0, i = -1, r = 0, o = -1;
    for (let a = 0; a <= n.length; ++a) {
      if (a < n.length) o = n.charCodeAt(a);
      else {
        if (o === 47) break;
        o = 47;
      }
      if (o === 47) {
        if (!(i === a - 1 || r === 1)) if (i !== a - 1 && r === 2) {
          if (e.length < 2 || s !== 2 || e.charCodeAt(e.length - 1) !== 46 || e.charCodeAt(e.length - 2) !== 46) {
            if (e.length > 2) {
              const l = e.lastIndexOf("/");
              if (l !== e.length - 1) {
                l === -1 ? (e = "", s = 0) : (e = e.slice(0, l), s = e.length - 1 - e.lastIndexOf("/")), i = a, r = 0;
                continue;
              }
            } else if (e.length === 2 || e.length === 1) {
              e = "", s = 0, i = a, r = 0;
              continue;
            }
          }
        } else e.length > 0 ? e += `/${n.slice(i + 1, a)}` : e = n.slice(i + 1, a), s = a - i - 1;
        i = a, r = 0;
      } else o === 46 && r !== -1 ? ++r : r = -1;
    }
    return e;
  }
  const Ce = {
    toPosix(n) {
      return np(n, "\\", "/");
    },
    isUrl(n) {
      return /^https?:/.test(this.toPosix(n));
    },
    isDataUrl(n) {
      return /^data:([a-z]+\/[a-z0-9-+.]+(;[a-z0-9-.!#$%*+.{}|~`]+=[a-z0-9-.!#$%*+.{}()_|~`]+)*)?(;base64)?,([a-z0-9!$&',()*+;=\-._~:@\/?%\s<>]*?)$/i.test(n);
    },
    isBlobUrl(n) {
      return n.startsWith("blob:");
    },
    hasProtocol(n) {
      return /^[^/:]+:/.test(this.toPosix(n));
    },
    getProtocol(n) {
      Oe(n), n = this.toPosix(n);
      const t = /^file:\/\/\//.exec(n);
      if (t) return t[0];
      const e = /^[^/:]+:\/{0,2}/.exec(n);
      return e ? e[0] : "";
    },
    toAbsolute(n, t, e) {
      if (Oe(n), this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      const s = vs(this.toPosix(t ?? Mt.get().getBaseUrl())), i = vs(this.toPosix(e ?? this.rootname(s)));
      return n = this.toPosix(n), n.startsWith("/") ? Ce.join(i, n.slice(1)) : this.isAbsolute(n) ? n : this.join(s, n);
    },
    normalize(n) {
      if (Oe(n), n.length === 0) return ".";
      if (this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      n = this.toPosix(n);
      let t = "";
      const e = n.startsWith("/");
      this.hasProtocol(n) && (t = this.rootname(n), n = n.slice(t.length));
      const s = n.endsWith("/");
      return n = sp(n), n.length > 0 && s && (n += "/"), e ? `/${n}` : t + n;
    },
    isAbsolute(n) {
      return Oe(n), n = this.toPosix(n), this.hasProtocol(n) ? true : n.startsWith("/");
    },
    join(...n) {
      if (n.length === 0) return ".";
      let t;
      for (let e = 0; e < n.length; ++e) {
        const s = n[e];
        if (Oe(s), s.length > 0) if (t === void 0) t = s;
        else {
          const i = n[e - 1] ?? "";
          this.joinExtensions.includes(this.extname(i).toLowerCase()) ? t += `/../${s}` : t += `/${s}`;
        }
      }
      return t === void 0 ? "." : this.normalize(t);
    },
    dirname(n) {
      if (Oe(n), n.length === 0) return ".";
      n = this.toPosix(n);
      let t = n.charCodeAt(0);
      const e = t === 47;
      let s = -1, i = true;
      const r = this.getProtocol(n), o = n;
      n = n.slice(r.length);
      for (let a = n.length - 1; a >= 1; --a) if (t = n.charCodeAt(a), t === 47) {
        if (!i) {
          s = a;
          break;
        }
      } else i = false;
      return s === -1 ? e ? "/" : this.isUrl(o) ? r + n : r : e && s === 1 ? "//" : r + n.slice(0, s);
    },
    rootname(n) {
      Oe(n), n = this.toPosix(n);
      let t = "";
      if (n.startsWith("/") ? t = "/" : t = this.getProtocol(n), this.isUrl(n)) {
        const e = n.indexOf("/", t.length);
        e !== -1 ? t = n.slice(0, e) : t = n, t.endsWith("/") || (t += "/");
      }
      return t;
    },
    basename(n, t) {
      Oe(n), t && Oe(t), n = vs(this.toPosix(n));
      let e = 0, s = -1, i = true, r;
      if (t !== void 0 && t.length > 0 && t.length <= n.length) {
        if (t.length === n.length && t === n) return "";
        let o = t.length - 1, a = -1;
        for (r = n.length - 1; r >= 0; --r) {
          const l = n.charCodeAt(r);
          if (l === 47) {
            if (!i) {
              e = r + 1;
              break;
            }
          } else a === -1 && (i = false, a = r + 1), o >= 0 && (l === t.charCodeAt(o) ? --o === -1 && (s = r) : (o = -1, s = a));
        }
        return e === s ? s = a : s === -1 && (s = n.length), n.slice(e, s);
      }
      for (r = n.length - 1; r >= 0; --r) if (n.charCodeAt(r) === 47) {
        if (!i) {
          e = r + 1;
          break;
        }
      } else s === -1 && (i = false, s = r + 1);
      return s === -1 ? "" : n.slice(e, s);
    },
    extname(n) {
      Oe(n), n = vs(this.toPosix(n));
      let t = -1, e = 0, s = -1, i = true, r = 0;
      for (let o = n.length - 1; o >= 0; --o) {
        const a = n.charCodeAt(o);
        if (a === 47) {
          if (!i) {
            e = o + 1;
            break;
          }
          continue;
        }
        s === -1 && (i = false, s = o + 1), a === 46 ? t === -1 ? t = o : r !== 1 && (r = 1) : t !== -1 && (r = -1);
      }
      return t === -1 || s === -1 || r === 0 || r === 1 && t === s - 1 && t === e + 1 ? "" : n.slice(t, s);
    },
    parse(n) {
      Oe(n);
      const t = {
        root: "",
        dir: "",
        base: "",
        ext: "",
        name: ""
      };
      if (n.length === 0) return t;
      n = vs(this.toPosix(n));
      let e = n.charCodeAt(0);
      const s = this.isAbsolute(n);
      let i;
      t.root = this.rootname(n), s || this.hasProtocol(n) ? i = 1 : i = 0;
      let r = -1, o = 0, a = -1, l = true, c = n.length - 1, h = 0;
      for (; c >= i; --c) {
        if (e = n.charCodeAt(c), e === 47) {
          if (!l) {
            o = c + 1;
            break;
          }
          continue;
        }
        a === -1 && (l = false, a = c + 1), e === 46 ? r === -1 ? r = c : h !== 1 && (h = 1) : r !== -1 && (h = -1);
      }
      return r === -1 || a === -1 || h === 0 || h === 1 && r === a - 1 && r === o + 1 ? a !== -1 && (o === 0 && s ? t.base = t.name = n.slice(1, a) : t.base = t.name = n.slice(o, a)) : (o === 0 && s ? (t.name = n.slice(1, r), t.base = n.slice(1, a)) : (t.name = n.slice(o, r), t.base = n.slice(o, a)), t.ext = n.slice(r, a)), t.dir = this.dirname(n), t;
    },
    sep: "/",
    delimiter: ":",
    joinExtensions: [
      ".html"
    ]
  };
  function Kc(n, t, e, s, i) {
    const r = t[e];
    for (let o = 0; o < r.length; o++) {
      const a = r[o];
      e < t.length - 1 ? Kc(n.replace(s[e], a), t, e + 1, s, i) : i.push(n.replace(s[e], a));
    }
  }
  function ip(n) {
    const t = /\{(.*?)\}/g, e = n.match(t), s = [];
    if (e) {
      const i = [];
      e.forEach((r) => {
        const o = r.substring(1, r.length - 1).split(",");
        i.push(o);
      }), Kc(n, i, 0, e, s);
    } else s.push(n);
    return s;
  }
  const $i = (n) => !Array.isArray(n);
  class ys {
    constructor() {
      this._defaultBundleIdentifierOptions = {
        connector: "-",
        createBundleAssetId: (t, e) => `${t}${this._bundleIdConnector}${e}`,
        extractAssetIdFromBundle: (t, e) => e.replace(`${t}${this._bundleIdConnector}`, "")
      }, this._bundleIdConnector = this._defaultBundleIdentifierOptions.connector, this._createBundleAssetId = this._defaultBundleIdentifierOptions.createBundleAssetId, this._extractAssetIdFromBundle = this._defaultBundleIdentifierOptions.extractAssetIdFromBundle, this._assetMap = {}, this._preferredOrder = [], this._parsers = [], this._resolverHash = {}, this._bundles = {};
    }
    setBundleIdentifier(t) {
      if (this._bundleIdConnector = t.connector ?? this._bundleIdConnector, this._createBundleAssetId = t.createBundleAssetId ?? this._createBundleAssetId, this._extractAssetIdFromBundle = t.extractAssetIdFromBundle ?? this._extractAssetIdFromBundle, this._extractAssetIdFromBundle("foo", this._createBundleAssetId("foo", "bar")) !== "bar") throw new Error("[Resolver] GenerateBundleAssetId are not working correctly");
    }
    prefer(...t) {
      t.forEach((e) => {
        this._preferredOrder.push(e), e.priority || (e.priority = Object.keys(e.params));
      }), this._resolverHash = {};
    }
    set basePath(t) {
      this._basePath = t;
    }
    get basePath() {
      return this._basePath;
    }
    set rootPath(t) {
      this._rootPath = t;
    }
    get rootPath() {
      return this._rootPath;
    }
    get parsers() {
      return this._parsers;
    }
    reset() {
      this.setBundleIdentifier(this._defaultBundleIdentifierOptions), this._assetMap = {}, this._preferredOrder = [], this._resolverHash = {}, this._rootPath = null, this._basePath = null, this._manifest = null, this._bundles = {}, this._defaultSearchParams = null;
    }
    setDefaultSearchParams(t) {
      if (typeof t == "string") this._defaultSearchParams = t;
      else {
        const e = t;
        this._defaultSearchParams = Object.keys(e).map((s) => `${encodeURIComponent(s)}=${encodeURIComponent(e[s])}`).join("&");
      }
    }
    getAlias(t) {
      const { alias: e, src: s } = t;
      return We(e || s, (r) => typeof r == "string" ? r : Array.isArray(r) ? r.map((o) => (o == null ? void 0 : o.src) ?? o) : (r == null ? void 0 : r.src) ? r.src : r, true);
    }
    removeAlias(t, e) {
      this._assetMap[t] && (e && e !== this._resolverHash[t] || (delete this._resolverHash[t], delete this._assetMap[t]));
    }
    addManifest(t) {
      this._manifest && Vt("[Resolver] Manifest already exists, this will be overwritten"), this._manifest = t, t.bundles.forEach((e) => {
        this.addBundle(e.name, e.assets);
      });
    }
    addBundle(t, e) {
      const s = [];
      let i = e;
      Array.isArray(e) || (i = Object.entries(e).map(([r, o]) => typeof o == "string" || Array.isArray(o) ? {
        alias: r,
        src: o
      } : {
        alias: r,
        ...o
      })), i.forEach((r) => {
        const o = r.src, a = r.alias;
        let l;
        if (typeof a == "string") {
          const c = this._createBundleAssetId(t, a);
          s.push(c), l = [
            a,
            c
          ];
        } else {
          const c = a.map((h) => this._createBundleAssetId(t, h));
          s.push(...c), l = [
            ...a,
            ...c
          ];
        }
        this.add({
          ...r,
          alias: l,
          src: o
        });
      }), this._bundles[t] = s;
    }
    add(t) {
      const e = [];
      Array.isArray(t) ? e.push(...t) : e.push(t);
      let s;
      s = (r) => {
        this.hasKey(r) && Vt(`[Resolver] already has key: ${r} overwriting`);
      }, We(e).forEach((r) => {
        const { src: o } = r;
        let { data: a, format: l, loadParser: c, parser: h } = r;
        const d = We(o).map((g) => typeof g == "string" ? ip(g) : Array.isArray(g) ? g : [
          g
        ]), u = this.getAlias(r);
        Array.isArray(u) ? u.forEach(s) : s(u);
        const p = [], f = (g) => {
          const m = this._parsers.find((y) => y.test(g));
          return {
            src: g,
            ...m == null ? void 0 : m.parse(g)
          };
        };
        d.forEach((g) => {
          g.forEach((m) => {
            let y = {};
            if (typeof m != "object" ? y = f(m) : (a = m.data ?? a, l = m.format ?? l, (m.loadParser || m.parser) && (c = m.loadParser ?? c, h = m.parser ?? h), y = {
              ...f(m.src),
              ...m
            }), !u) throw new Error(`[Resolver] alias is undefined for this asset: ${y.src}`);
            y = this._buildResolvedAsset(y, {
              aliases: u,
              data: a,
              format: l,
              loadParser: c,
              parser: h,
              progressSize: r.progressSize
            }), p.push(y);
          });
        }), u.forEach((g) => {
          this._assetMap[g] = p;
        });
      });
    }
    resolveBundle(t) {
      const e = $i(t);
      t = We(t);
      const s = {};
      return t.forEach((i) => {
        const r = this._bundles[i];
        if (r) {
          const o = this.resolve(r), a = {};
          for (const l in o) {
            const c = o[l];
            a[this._extractAssetIdFromBundle(i, l)] = c;
          }
          s[i] = a;
        }
      }), e ? s[t[0]] : s;
    }
    resolveUrl(t) {
      const e = this.resolve(t);
      if (typeof t != "string") {
        const s = {};
        for (const i in e) s[i] = e[i].src;
        return s;
      }
      return e.src;
    }
    resolve(t) {
      const e = $i(t);
      t = We(t);
      const s = {};
      return t.forEach((i) => {
        if (!this._resolverHash[i]) if (this._assetMap[i]) {
          let r = this._assetMap[i];
          const o = this._getPreferredOrder(r);
          o == null ? void 0 : o.priority.forEach((a) => {
            o.params[a].forEach((l) => {
              const c = r.filter((h) => h[a] ? h[a] === l : false);
              c.length && (r = c);
            });
          }), this._resolverHash[i] = r[0];
        } else this._resolverHash[i] = this._buildResolvedAsset({
          alias: [
            i
          ],
          src: i
        }, {});
        s[i] = this._resolverHash[i];
      }), e ? s[t[0]] : s;
    }
    hasKey(t) {
      return !!this._assetMap[t];
    }
    hasBundle(t) {
      return !!this._bundles[t];
    }
    _getPreferredOrder(t) {
      for (let e = 0; e < t.length; e++) {
        const s = t[e], i = this._preferredOrder.find((r) => r.params.format.includes(s.format));
        if (i) return i;
      }
      return this._preferredOrder[0];
    }
    _appendDefaultSearchParams(t) {
      if (!this._defaultSearchParams) return t;
      const e = /\?/.test(t) ? "&" : "?";
      return `${t}${e}${this._defaultSearchParams}`;
    }
    _buildResolvedAsset(t, e) {
      const { aliases: s, data: i, loadParser: r, parser: o, format: a, progressSize: l } = e;
      return (this._basePath || this._rootPath) && (t.src = Ce.toAbsolute(t.src, this._basePath, this._rootPath)), t.alias = s ?? t.alias ?? [
        t.src
      ], t.src = this._appendDefaultSearchParams(t.src), t.data = {
        ...i || {},
        ...t.data
      }, t.loadParser = r ?? t.loadParser, t.parser = o ?? t.parser, t.format = a ?? t.format ?? rp(t.src), l !== void 0 && (t.progressSize = l), t;
    }
  }
  ys.RETINA_PREFIX = /@([0-9\.]+)x/;
  function rp(n) {
    return n.split(".").pop().split("?").shift().split("#").shift();
  }
  const eo = (n, t) => {
    const e = t.split("?")[1];
    return e && (n += `?${e}`), n;
  }, Jc = class Ls {
    constructor(t, e) {
      this.linkedSheets = [];
      let s = t;
      (t == null ? void 0 : t.source) instanceof ke && (s = {
        texture: t,
        data: e
      });
      const { texture: i, data: r, cachePrefix: o = "" } = s;
      this.cachePrefix = o, this._texture = i instanceof Ct ? i : null, this.textureSource = i.source, this.textures = {}, this.animations = {}, this.data = r;
      const a = parseFloat(r.meta.scale);
      a ? (this.resolution = a, i.source.resolution = this.resolution) : this.resolution = i.source._resolution, this._frames = this.data.frames, this._frameKeys = Object.keys(this._frames), this._batchIndex = 0, this._callback = null;
    }
    parse() {
      return new Promise((t) => {
        this._callback = t, this._batchIndex = 0, this._frameKeys.length <= Ls.BATCH_SIZE ? (this._processFrames(0), this._processAnimations(), this._parseComplete()) : this._nextBatch();
      });
    }
    parseSync() {
      return this._processFrames(0, true), this._processAnimations(), this.textures;
    }
    _processFrames(t, e = false) {
      let s = t;
      const i = e ? 1 / 0 : Ls.BATCH_SIZE;
      for (; s - t < i && s < this._frameKeys.length; ) {
        const r = this._frameKeys[s], o = this._frames[r], a = o.frame;
        if (a) {
          let l = null, c = null;
          const h = o.trimmed !== false && o.sourceSize ? o.sourceSize : o.frame, d = new Nt(0, 0, Math.floor(h.w) / this.resolution, Math.floor(h.h) / this.resolution);
          o.rotated ? l = new Nt(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.h) / this.resolution, Math.floor(a.w) / this.resolution) : l = new Nt(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution), o.trimmed !== false && o.spriteSourceSize && (c = new Nt(Math.floor(o.spriteSourceSize.x) / this.resolution, Math.floor(o.spriteSourceSize.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution)), this.textures[r] = new Ct({
            source: this.textureSource,
            frame: l,
            orig: d,
            trim: c,
            rotate: o.rotated ? 2 : 0,
            defaultAnchor: o.anchor,
            defaultBorders: o.borders,
            label: r.toString()
          });
        }
        s++;
      }
    }
    _processAnimations() {
      const t = this.data.animations || {};
      for (const e in t) {
        this.animations[e] = [];
        for (let s = 0; s < t[e].length; s++) {
          const i = t[e][s];
          this.animations[e].push(this.textures[i]);
        }
      }
    }
    _parseComplete() {
      const t = this._callback;
      this._callback = null, this._batchIndex = 0, t.call(this, this.textures);
    }
    _nextBatch() {
      this._processFrames(this._batchIndex * Ls.BATCH_SIZE), this._batchIndex++, setTimeout(() => {
        this._batchIndex * Ls.BATCH_SIZE < this._frameKeys.length ? this._nextBatch() : (this._processAnimations(), this._parseComplete());
      }, 0);
    }
    destroy(t = false) {
      var _a2;
      for (const e in this.textures) this.textures[e].destroy();
      this._frames = null, this._frameKeys = null, this.data = null, this.textures = null, t && ((_a2 = this._texture) == null ? void 0 : _a2.destroy(), this.textureSource.destroy()), this._texture = null, this.textureSource = null, this.linkedSheets = [];
    }
  };
  Jc.BATCH_SIZE = 1e3;
  let La = Jc;
  const op = [
    "jpg",
    "png",
    "jpeg",
    "avif",
    "webp",
    "basis",
    "etc2",
    "bc7",
    "bc6h",
    "bc5",
    "bc4",
    "bc3",
    "bc2",
    "bc1",
    "eac",
    "astc"
  ];
  function Zc(n, t, e) {
    const s = {};
    if (n.forEach((i) => {
      s[i] = t;
    }), Object.keys(t.textures).forEach((i) => {
      s[`${t.cachePrefix}${i}`] = t.textures[i];
    }), !e) {
      const i = Ce.dirname(n[0]);
      t.linkedSheets.forEach((r, o) => {
        const a = Zc([
          `${i}/${t.data.meta.related_multi_packs[o]}`
        ], r, true);
        Object.assign(s, a);
      });
    }
    return s;
  }
  const ap = {
    extension: it.Asset,
    cache: {
      test: (n) => n instanceof La,
      getCacheableAssets: (n, t) => Zc(n, t, false)
    },
    resolver: {
      extension: {
        type: it.ResolveParser,
        name: "resolveSpritesheet"
      },
      test: (n) => {
        const e = n.split("?")[0].split("."), s = e.pop(), i = e.pop();
        return s === "json" && op.includes(i);
      },
      parse: (n) => {
        var _a2;
        const t = n.split(".");
        return {
          resolution: parseFloat(((_a2 = ys.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
          format: t[t.length - 2],
          src: n
        };
      }
    },
    loader: {
      name: "spritesheetLoader",
      id: "spritesheet",
      extension: {
        type: it.LoadParser,
        priority: kn.Normal,
        name: "spritesheetLoader"
      },
      async testParse(n, t) {
        return Ce.extname(t.src).toLowerCase() === ".json" && !!n.frames;
      },
      async parse(n, t, e) {
        var _a2, _b2;
        const { texture: s, imageFilename: i, textureOptions: r, cachePrefix: o } = (t == null ? void 0 : t.data) ?? {};
        let a = Ce.dirname(t.src);
        a && a.lastIndexOf("/") !== a.length - 1 && (a += "/");
        let l;
        if (s instanceof Ct) l = s;
        else {
          const d = eo(a + (i ?? n.meta.image), t.src);
          l = (await e.load([
            {
              src: d,
              data: r
            }
          ]))[d];
        }
        const c = new La({
          texture: l.source,
          data: n,
          cachePrefix: o
        });
        await c.parse();
        const h = (_a2 = n == null ? void 0 : n.meta) == null ? void 0 : _a2.related_multi_packs;
        if (Array.isArray(h)) {
          const d = [];
          for (const p of h) {
            if (typeof p != "string") continue;
            let f = a + p;
            ((_b2 = t.data) == null ? void 0 : _b2.ignoreMultiPack) || (f = eo(f, t.src), d.push(e.load({
              src: f,
              data: {
                textureOptions: r,
                ignoreMultiPack: true
              }
            })));
          }
          const u = await Promise.all(d);
          c.linkedSheets = u, u.forEach((p) => {
            p.linkedSheets = [
              c
            ].concat(c.linkedSheets.filter((f) => f !== p));
          });
        }
        return c;
      },
      async unload(n, t, e) {
        await e.unload(n.textureSource._sourceOrigin), n.destroy(false);
      }
    }
  };
  Ot.add(ap);
  const yr = /* @__PURE__ */ Object.create(null), $a = /* @__PURE__ */ Object.create(null);
  Bo = function(n, t) {
    let e = $a[n];
    return e === void 0 && (yr[t] === void 0 && (yr[t] = 1), $a[n] = e = yr[t]++), e;
  };
  let ci;
  function Qc() {
    return (!ci || (ci == null ? void 0 : ci.isContextLost())) && (ci = Mt.get().createCanvas().getContext("webgl", {})), ci;
  }
  let hi;
  function lp() {
    if (!hi) {
      hi = "mediump";
      const n = Qc();
      n && n.getShaderPrecisionFormat && (hi = n.getShaderPrecisionFormat(n.FRAGMENT_SHADER, n.HIGH_FLOAT).precision ? "highp" : "mediump");
    }
    return hi;
  }
  function cp(n, t, e) {
    return t ? n : e ? (n = n.replace("out vec4 finalColor;", ""), `

        #ifdef GL_ES // This checks if it is WebGL1
        #define in varying
        #define finalColor gl_FragColor
        #define texture texture2D
        #endif
        ${n}
        `) : `

        #ifdef GL_ES // This checks if it is WebGL1
        #define in attribute
        #define out varying
        #endif
        ${n}
        `;
  }
  function hp(n, t, e) {
    const s = e ? t.maxSupportedFragmentPrecision : t.maxSupportedVertexPrecision;
    if (n.substring(0, 9) !== "precision") {
      let i = e ? t.requestedFragmentPrecision : t.requestedVertexPrecision;
      return i === "highp" && s !== "highp" && (i = "mediump"), `precision ${i} float;
${n}`;
    } else if (s !== "highp" && n.substring(0, 15) === "precision highp") return n.replace("precision highp", "precision mediump");
    return n;
  }
  function dp(n, t) {
    return t ? `#version 300 es
${n}` : n;
  }
  const up = {}, pp = {};
  function fp(n, { name: t = "pixi-program" }, e = true) {
    t = t.replace(/\s+/g, "-"), t += e ? "-fragment" : "-vertex";
    const s = e ? up : pp;
    return s[t] ? (s[t]++, t += `-${s[t]}`) : s[t] = 1, n.indexOf("#define SHADER_NAME") !== -1 ? n : `${`#define SHADER_NAME ${t}`}
${n}`;
  }
  function mp(n, t) {
    return t ? n.replace("#version 300 es", "") : n;
  }
  const xr = {
    stripVersion: mp,
    ensurePrecision: hp,
    addProgramDefines: cp,
    setProgramName: fp,
    insertVersion: dp
  }, Cs = /* @__PURE__ */ Object.create(null), th = class no {
    constructor(t) {
      t = {
        ...no.defaultOptions,
        ...t
      };
      const e = t.fragment.indexOf("#version 300 es") !== -1, s = {
        stripVersion: e,
        ensurePrecision: {
          requestedFragmentPrecision: t.preferredFragmentPrecision,
          requestedVertexPrecision: t.preferredVertexPrecision,
          maxSupportedVertexPrecision: "highp",
          maxSupportedFragmentPrecision: lp()
        },
        setProgramName: {
          name: t.name
        },
        addProgramDefines: e,
        insertVersion: e
      };
      let i = t.fragment, r = t.vertex;
      Object.keys(xr).forEach((o) => {
        const a = s[o];
        i = xr[o](i, a, true), r = xr[o](r, a, false);
      }), this.fragment = i, this.vertex = r, this.transformFeedbackVaryings = t.transformFeedbackVaryings, this._key = Bo(`${this.vertex}:${this.fragment}`, "gl-program");
    }
    destroy() {
      this.fragment = null, this.vertex = null, this._attributeData = null, this._uniformData = null, this._uniformBlockData = null, this.transformFeedbackVaryings = null, Cs[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex}:${t.fragment}`;
      return Cs[e] || (Cs[e] = new no(t), Cs[e]._cacheKey = e), Cs[e];
    }
  };
  th.defaultOptions = {
    preferredVertexPrecision: "highp",
    preferredFragmentPrecision: "mediump"
  };
  Oo = th;
  const Ba = {
    uint8x2: {
      size: 2,
      stride: 2,
      normalised: false
    },
    uint8x4: {
      size: 4,
      stride: 4,
      normalised: false
    },
    sint8x2: {
      size: 2,
      stride: 2,
      normalised: false
    },
    sint8x4: {
      size: 4,
      stride: 4,
      normalised: false
    },
    unorm8x2: {
      size: 2,
      stride: 2,
      normalised: true
    },
    unorm8x4: {
      size: 4,
      stride: 4,
      normalised: true
    },
    snorm8x2: {
      size: 2,
      stride: 2,
      normalised: true
    },
    snorm8x4: {
      size: 4,
      stride: 4,
      normalised: true
    },
    uint16x2: {
      size: 2,
      stride: 4,
      normalised: false
    },
    uint16x4: {
      size: 4,
      stride: 8,
      normalised: false
    },
    sint16x2: {
      size: 2,
      stride: 4,
      normalised: false
    },
    sint16x4: {
      size: 4,
      stride: 8,
      normalised: false
    },
    unorm16x2: {
      size: 2,
      stride: 4,
      normalised: true
    },
    unorm16x4: {
      size: 4,
      stride: 8,
      normalised: true
    },
    snorm16x2: {
      size: 2,
      stride: 4,
      normalised: true
    },
    snorm16x4: {
      size: 4,
      stride: 8,
      normalised: true
    },
    float16x2: {
      size: 2,
      stride: 4,
      normalised: false
    },
    float16x4: {
      size: 4,
      stride: 8,
      normalised: false
    },
    float32: {
      size: 1,
      stride: 4,
      normalised: false
    },
    float32x2: {
      size: 2,
      stride: 8,
      normalised: false
    },
    float32x3: {
      size: 3,
      stride: 12,
      normalised: false
    },
    float32x4: {
      size: 4,
      stride: 16,
      normalised: false
    },
    uint32: {
      size: 1,
      stride: 4,
      normalised: false
    },
    uint32x2: {
      size: 2,
      stride: 8,
      normalised: false
    },
    uint32x3: {
      size: 3,
      stride: 12,
      normalised: false
    },
    uint32x4: {
      size: 4,
      stride: 16,
      normalised: false
    },
    sint32: {
      size: 1,
      stride: 4,
      normalised: false
    },
    sint32x2: {
      size: 2,
      stride: 8,
      normalised: false
    },
    sint32x3: {
      size: 3,
      stride: 12,
      normalised: false
    },
    sint32x4: {
      size: 4,
      stride: 16,
      normalised: false
    }
  };
  Bi = function(n) {
    return Ba[n] ?? Ba.float32;
  };
  const gp = {
    f32: "float32",
    "vec2<f32>": "float32x2",
    "vec3<f32>": "float32x3",
    "vec4<f32>": "float32x4",
    vec2f: "float32x2",
    vec3f: "float32x3",
    vec4f: "float32x4",
    i32: "sint32",
    "vec2<i32>": "sint32x2",
    "vec3<i32>": "sint32x3",
    "vec4<i32>": "sint32x4",
    vec2i: "sint32x2",
    vec3i: "sint32x3",
    vec4i: "sint32x4",
    u32: "uint32",
    "vec2<u32>": "uint32x2",
    "vec3<u32>": "uint32x3",
    "vec4<u32>": "uint32x4",
    vec2u: "uint32x2",
    vec3u: "uint32x3",
    vec4u: "uint32x4",
    bool: "uint32",
    "vec2<bool>": "uint32x2",
    "vec3<bool>": "uint32x3",
    "vec4<bool>": "uint32x4"
  }, Oa = /@location\((\d+)\)\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_<>]+)(?:,|\s|\)|$)/g;
  function Fa(n, t) {
    let e;
    for (; (e = Oa.exec(n)) !== null; ) {
      const s = gp[e[3]] ?? "float32";
      t[e[2]] = {
        location: parseInt(e[1], 10),
        format: s,
        stride: Bi(s).stride,
        offset: 0,
        instance: false,
        start: 0
      };
    }
    Oa.lastIndex = 0;
  }
  function yp(n) {
    return n.replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  }
  function xp({ source: n, entryPoint: t }) {
    const e = {}, s = yp(n), i = s.indexOf(`fn ${t}(`);
    if (i === -1) return e;
    const r = s.indexOf("->", i);
    if (r === -1) return e;
    const o = s.substring(i, r);
    if (Fa(o, e), Object.keys(e).length === 0) {
      const a = o.match(/\(\s*\w+\s*:\s*(\w+)/);
      if (a) {
        const l = a[1], c = new RegExp(`struct\\s+${l}\\s*\\{([^}]+)\\}`, "s"), h = s.match(c);
        h && Fa(h[1], e);
      }
    }
    return e;
  }
  function br(n) {
    var _a2, _b2;
    const t = /(^|[^/])@(group|binding)\(\d+\)[^;]+;/g, e = /@group\((\d+)\)/, s = /@binding\((\d+)\)/, i = /var(<[^>]+>)? (\w+)/, r = /:\s*([\w<>]+)/, o = /struct\s+(\w+)\s*{([^}]+)}/g, a = /(\w+)\s*:\s*([\w\<\>]+)/g, l = /struct\s+(\w+)/, c = (_a2 = n.match(t)) == null ? void 0 : _a2.map((d) => ({
      group: parseInt(d.match(e)[1], 10),
      binding: parseInt(d.match(s)[1], 10),
      name: d.match(i)[2],
      isUniform: d.match(i)[1] === "<uniform>",
      type: d.match(r)[1]
    }));
    if (!c) return {
      groups: [],
      structs: []
    };
    const h = ((_b2 = n.match(o)) == null ? void 0 : _b2.map((d) => {
      const u = d.match(l)[1], p = d.match(a).reduce((f, g) => {
        const [m, y] = g.split(":");
        return f[m.trim()] = y.trim(), f;
      }, {});
      return p ? {
        name: u,
        members: p
      } : null;
    }).filter(({ name: d }) => c.some((u) => u.type === d || u.type.includes(`<${d}>`)))) ?? [];
    return {
      groups: c,
      structs: h
    };
  }
  var On = ((n) => (n[n.VERTEX = 1] = "VERTEX", n[n.FRAGMENT = 2] = "FRAGMENT", n[n.COMPUTE = 4] = "COMPUTE", n))(On || {});
  function bp({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = []), s.isUniform ? t[s.group].push({
        binding: s.binding,
        visibility: On.VERTEX | On.FRAGMENT,
        buffer: {
          type: "uniform"
        }
      }) : s.type === "sampler" ? t[s.group].push({
        binding: s.binding,
        visibility: On.FRAGMENT,
        sampler: {
          type: "filtering"
        }
      }) : s.type === "texture_2d" || s.type.startsWith("texture_2d<") ? t[s.group].push({
        binding: s.binding,
        visibility: On.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d",
          multisampled: false
        }
      }) : s.type === "texture_2d_array" || s.type.startsWith("texture_2d_array<") ? t[s.group].push({
        binding: s.binding,
        visibility: On.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d-array",
          multisampled: false
        }
      }) : (s.type === "texture_cube" || s.type.startsWith("texture_cube<")) && t[s.group].push({
        binding: s.binding,
        visibility: On.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "cube",
          multisampled: false
        }
      });
    }
    for (let e = 0; e < t.length; e++) t[e] || (t[e] = []);
    return t;
  }
  function _p({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = {}), t[s.group][s.name] = s.binding;
    }
    return t;
  }
  function wp(n, t) {
    const e = /* @__PURE__ */ new Set(), s = /* @__PURE__ */ new Set(), i = [
      ...n.structs,
      ...t.structs
    ].filter((o) => e.has(o.name) ? false : (e.add(o.name), true)), r = [
      ...n.groups,
      ...t.groups
    ].filter((o) => {
      const a = `${o.name}-${o.binding}`;
      return s.has(a) ? false : (s.add(a), true);
    });
    return {
      structs: i,
      groups: r
    };
  }
  const Ss = /* @__PURE__ */ Object.create(null);
  ti = class {
    constructor(t) {
      var _a2, _b2;
      this._layoutKey = 0, this._attributeLocationsKey = 0;
      const { fragment: e, vertex: s, layout: i, gpuLayout: r, name: o } = t;
      if (this.name = o, this.fragment = e, this.vertex = s, e.source === s.source) {
        const a = br(e.source);
        this.structsAndGroups = a;
      } else {
        const a = br(s.source), l = br(e.source);
        this.structsAndGroups = wp(a, l);
      }
      this.layout = i ?? _p(this.structsAndGroups), this.gpuLayout = r ?? bp(this.structsAndGroups), this.autoAssignGlobalUniforms = ((_a2 = this.layout[0]) == null ? void 0 : _a2.globalUniforms) !== void 0, this.autoAssignLocalUniforms = ((_b2 = this.layout[1]) == null ? void 0 : _b2.localUniforms) !== void 0, this._generateProgramKey();
    }
    _generateProgramKey() {
      const { vertex: t, fragment: e } = this, s = t.source + e.source + t.entryPoint + e.entryPoint;
      this._layoutKey = Bo(s, "program");
    }
    get attributeData() {
      return this._attributeData ?? (this._attributeData = xp(this.vertex)), this._attributeData;
    }
    destroy() {
      this.gpuLayout = null, this.layout = null, this.structsAndGroups = null, this.fragment = null, this.vertex = null, Ss[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex.source}:${t.fragment.source}:${t.fragment.entryPoint}:${t.vertex.entryPoint}`;
      return Ss[e] || (Ss[e] = new ti(t), Ss[e]._cacheKey = e), Ss[e];
    }
  };
  const eh = [
    "f32",
    "i32",
    "vec2<f32>",
    "vec3<f32>",
    "vec4<f32>",
    "mat2x2<f32>",
    "mat3x3<f32>",
    "mat4x4<f32>",
    "mat3x2<f32>",
    "mat4x2<f32>",
    "mat2x3<f32>",
    "mat4x3<f32>",
    "mat2x4<f32>",
    "mat3x4<f32>",
    "vec2<i32>",
    "vec3<i32>",
    "vec4<i32>"
  ], vp = eh.reduce((n, t) => (n[t] = true, n), {});
  function Cp(n, t) {
    switch (n) {
      case "f32":
        return 0;
      case "vec2<f32>":
        return new Float32Array(2 * t);
      case "vec3<f32>":
        return new Float32Array(3 * t);
      case "vec4<f32>":
        return new Float32Array(4 * t);
      case "mat2x2<f32>":
        return new Float32Array([
          1,
          0,
          0,
          1
        ]);
      case "mat3x3<f32>":
        return new Float32Array([
          1,
          0,
          0,
          0,
          1,
          0,
          0,
          0,
          1
        ]);
      case "mat4x4<f32>":
        return new Float32Array([
          1,
          0,
          0,
          0,
          0,
          1,
          0,
          0,
          0,
          0,
          1,
          0,
          0,
          0,
          0,
          1
        ]);
    }
    return null;
  }
  const nh = class sh {
    constructor(t, e) {
      this._touched = 0, this.uid = Kt("uniform"), this._resourceType = "uniformGroup", this._resourceId = Kt("resource"), this.isUniformGroup = true, this._dirtyId = 0, this.destroyed = false, e = {
        ...sh.defaultOptions,
        ...e
      }, this.uniformStructures = t;
      const s = {};
      for (const i in t) {
        const r = t[i];
        if (r.name = i, r.size = r.size ?? 1, !vp[r.type]) {
          const o = r.type.match(/^array<(\w+(?:<\w+>)?),\s*(\d+)>$/);
          if (o) {
            const [, a, l] = o;
            throw new Error(`Uniform type ${r.type} is not supported. Use type: '${a}', size: ${l} instead.`);
          }
          throw new Error(`Uniform type ${r.type} is not supported. Supported uniform types are: ${eh.join(", ")}`);
        }
        r.value ?? (r.value = Cp(r.type, r.size)), s[i] = r.value;
      }
      this.uniforms = s, this._dirtyId = 1, this.ubo = e.ubo, this.isStatic = e.isStatic, this._signature = Bo(Object.keys(s).map((i) => `${i}-${t[i].type}`).join("-"), "uniform-group");
    }
    update() {
      this._dirtyId++;
    }
  };
  nh.defaultOptions = {
    ubo: false,
    isStatic: false
  };
  Fo = nh;
  Ii = class {
    constructor(t) {
      this.resources = /* @__PURE__ */ Object.create(null), this._dirty = true;
      let e = 0;
      for (const s in t) {
        const i = t[s];
        this.setResource(i, e++);
      }
      this._updateKey();
    }
    _updateKey() {
      if (!this._dirty) return;
      this._dirty = false;
      const t = [];
      let e = 0;
      for (const s in this.resources) t[e++] = this.resources[s]._resourceId;
      this._key = t.join("|");
    }
    setResource(t, e) {
      var _a2, _b2;
      const s = this.resources[e];
      t !== s && ((_a2 = s == null ? void 0 : s.off) == null ? void 0 : _a2.call(s, "change", this.onResourceChange, this), (_b2 = t.on) == null ? void 0 : _b2.call(t, "change", this.onResourceChange, this), this.resources[e] = t, this._dirty = true);
    }
    getResource(t) {
      return this.resources[t];
    }
    _touch(t, e) {
      const s = this.resources;
      for (const i in s) s[i]._gcLastUsed = t, s[i]._touched = e;
    }
    destroy() {
      var _a2, _b2;
      const t = this.resources;
      for (const e in t) (_b2 = (_a2 = t[e]) == null ? void 0 : _a2.off) == null ? void 0 : _b2.call(_a2, "change", this.onResourceChange, this);
      this.resources = null;
    }
    onResourceChange(t) {
      this._dirty = true, t.destroyed ? this.destroy() : this._updateKey();
    }
  };
  so = ((n) => (n[n.WEBGL = 1] = "WEBGL", n[n.WEBGPU = 2] = "WEBGPU", n[n.CANVAS = 4] = "CANVAS", n[n.BOTH = 3] = "BOTH", n))(so || {});
  Xi = class extends Qe {
    constructor(t) {
      super(), this.uid = Kt("shader"), this._uniformBindMap = /* @__PURE__ */ Object.create(null), this._ownedBindGroups = [], this._destroyed = false;
      let { gpuProgram: e, glProgram: s, groups: i, resources: r, compatibleRenderers: o, groupMap: a } = t;
      this.gpuProgram = e, this.glProgram = s, o === void 0 && (o = 0, e && (o |= so.WEBGPU), s && (o |= so.WEBGL)), this.compatibleRenderers = o;
      const l = {};
      if (!r && !i && (r = {}), r && i) throw new Error("[Shader] Cannot have both resources and groups");
      if (!e && i && !a) throw new Error("[Shader] No group map or WebGPU shader provided - consider using resources instead.");
      if (!e && i && a) for (const c in a) for (const h in a[c]) {
        const d = a[c][h];
        l[d] = {
          group: c,
          binding: h,
          name: d
        };
      }
      else if (e && i && !a) {
        const c = e.structsAndGroups.groups;
        a = {}, c.forEach((h) => {
          a[h.group] = a[h.group] || {}, a[h.group][h.binding] = h.name, l[h.name] = h;
        });
      } else if (r) {
        i = {}, a = {}, e && e.structsAndGroups.groups.forEach((d) => {
          a[d.group] = a[d.group] || {}, a[d.group][d.binding] = d.name, l[d.name] = d;
        });
        let c = 0;
        for (const h in r) l[h] || (i[99] || (i[99] = new Ii(), this._ownedBindGroups.push(i[99])), l[h] = {
          group: 99,
          binding: c,
          name: h
        }, a[99] = a[99] || {}, a[99][c] = h, c++);
        for (const h in r) {
          const d = h;
          let u = r[h];
          !u.source && !u._resourceType && (u = new Fo(u));
          const p = l[d];
          p && (i[p.group] || (i[p.group] = new Ii(), this._ownedBindGroups.push(i[p.group])), i[p.group].setResource(u, p.binding));
        }
      }
      this.groups = i, this._uniformBindMap = a, this.resources = this._buildResourceAccessor(i, l);
    }
    addResource(t, e, s) {
      var i, r;
      (i = this._uniformBindMap)[e] || (i[e] = {}), (r = this._uniformBindMap[e])[s] || (r[s] = t), this.groups[e] || (this.groups[e] = new Ii(), this._ownedBindGroups.push(this.groups[e]));
    }
    _buildResourceAccessor(t, e) {
      const s = {};
      for (const i in e) {
        const r = e[i];
        Object.defineProperty(s, r.name, {
          get() {
            return t[r.group].getResource(r.binding);
          },
          set(o) {
            t[r.group].setResource(o, r.binding);
          }
        });
      }
      return s;
    }
    destroy(t = false) {
      var _a2, _b2;
      this._destroyed || (this._destroyed = true, this.emit("destroy", this), t && ((_a2 = this.gpuProgram) == null ? void 0 : _a2.destroy(), (_b2 = this.glProgram) == null ? void 0 : _b2.destroy()), this.gpuProgram = null, this.glProgram = null, this.removeAllListeners(), this._uniformBindMap = null, this._ownedBindGroups.forEach((e) => {
        e.destroy();
      }), this._ownedBindGroups = null, this.resources = null, this.groups = null);
    }
    static from(t) {
      const { gpu: e, gl: s, ...i } = t;
      let r, o;
      return e && (r = ti.from(e)), s && (o = Oo.from(s)), new Xi({
        gpuProgram: r,
        glProgram: o,
        ...i
      });
    }
  };
  const Sp = {
    normal: 0,
    add: 1,
    multiply: 2,
    screen: 3,
    overlay: 4,
    erase: 5,
    "normal-npm": 6,
    "add-npm": 7,
    "screen-npm": 8,
    min: 9,
    max: 10
  }, _r = 0, wr = 1, vr = 2, Cr = 3, Sr = 4, kr = 5, io = class ih {
    constructor() {
      this.data = 0, this.blendMode = "normal", this.polygonOffset = 0, this.blend = true, this.depthMask = true;
    }
    get blend() {
      return !!(this.data & 1 << _r);
    }
    set blend(t) {
      !!(this.data & 1 << _r) !== t && (this.data ^= 1 << _r);
    }
    get offsets() {
      return !!(this.data & 1 << wr);
    }
    set offsets(t) {
      !!(this.data & 1 << wr) !== t && (this.data ^= 1 << wr);
    }
    set cullMode(t) {
      if (t === "none") {
        this.culling = false;
        return;
      }
      this.culling = true, this.clockwiseFrontFace = t === "front";
    }
    get cullMode() {
      return this.culling ? this.clockwiseFrontFace ? "front" : "back" : "none";
    }
    get culling() {
      return !!(this.data & 1 << vr);
    }
    set culling(t) {
      !!(this.data & 1 << vr) !== t && (this.data ^= 1 << vr);
    }
    get depthTest() {
      return !!(this.data & 1 << Cr);
    }
    set depthTest(t) {
      !!(this.data & 1 << Cr) !== t && (this.data ^= 1 << Cr);
    }
    get depthMask() {
      return !!(this.data & 1 << kr);
    }
    set depthMask(t) {
      !!(this.data & 1 << kr) !== t && (this.data ^= 1 << kr);
    }
    get clockwiseFrontFace() {
      return !!(this.data & 1 << Sr);
    }
    set clockwiseFrontFace(t) {
      !!(this.data & 1 << Sr) !== t && (this.data ^= 1 << Sr);
    }
    get blendMode() {
      return this._blendMode;
    }
    set blendMode(t) {
      this.blend = t !== "none", this._blendMode = t, this._blendModeId = Sp[t] || 0;
    }
    get polygonOffset() {
      return this._polygonOffset;
    }
    set polygonOffset(t) {
      this.offsets = !!t, this._polygonOffset = t;
    }
    toString() {
      return `[pixi.js/core:State blendMode=${this.blendMode} clockwiseFrontFace=${this.clockwiseFrontFace} culling=${this.culling} depthMask=${this.depthMask} polygonOffset=${this.polygonOffset}]`;
    }
    static for2d() {
      const t = new ih();
      return t.depthTest = false, t.blend = true, t;
    }
  };
  io.default2d = io.for2d();
  Oi = io;
  const ro = [];
  Ot.handleByNamedList(it.Environment, ro);
  async function kp(n) {
    if (!n) for (let t = 0; t < ro.length; t++) {
      const e = ro[t];
      if (e.value.test()) {
        await e.value.load();
        return;
      }
    }
  }
  let ks;
  Tp = function() {
    if (typeof ks == "boolean") return ks;
    try {
      ks = new Function("param1", "param2", "param3", "return param1[param2] === param3;")({
        a: "b"
      }, "a", "b") === true;
    } catch {
      ks = false;
    }
    return ks;
  };
  function Na(n, t, e = 2) {
    const s = t && t.length, i = s ? t[0] * e : n.length;
    let r = rh(n, 0, i, e, true);
    const o = [];
    if (!r || r.next === r.prev) return o;
    let a, l, c;
    if (s && (r = Ip(n, t, r, e)), n.length > 80 * e) {
      a = n[0], l = n[1];
      let h = a, d = l;
      for (let u = e; u < i; u += e) {
        const p = n[u], f = n[u + 1];
        p < a && (a = p), f < l && (l = f), p > h && (h = p), f > d && (d = f);
      }
      c = Math.max(h - a, d - l), c = c !== 0 ? 32767 / c : 0;
    }
    return Us(r, o, e, a, l, c, 0), o;
  }
  function rh(n, t, e, s, i) {
    let r;
    if (i === Dp(n, t, e, s) > 0) for (let o = t; o < e; o += s) r = Wa(o / s | 0, n[o], n[o + 1], r);
    else for (let o = e - s; o >= t; o -= s) r = Wa(o / s | 0, n[o], n[o + 1], r);
    return r && ms(r, r.next) && (Vs(r), r = r.next), r;
  }
  function Un(n, t) {
    if (!n) return n;
    t || (t = n);
    let e = n, s;
    do
      if (s = false, !e.steiner && (ms(e, e.next) || Yt(e.prev, e, e.next) === 0)) {
        if (Vs(e), e = t = e.prev, e === e.next) break;
        s = true;
      } else e = e.next;
    while (s || e !== t);
    return t;
  }
  function Us(n, t, e, s, i, r, o) {
    if (!n) return;
    !o && r && Op(n, s, i, r);
    let a = n;
    for (; n.prev !== n.next; ) {
      const l = n.prev, c = n.next;
      if (r ? Ep(n, s, i, r) : Ap(n)) {
        t.push(l.i, n.i, c.i), Vs(n), n = c.next, a = c.next;
        continue;
      }
      if (n = c, n === a) {
        o ? o === 1 ? (n = Mp(Un(n), t), Us(n, t, e, s, i, r, 2)) : o === 2 && Pp(n, t, e, s, i, r) : Us(Un(n), t, e, s, i, r, 1);
        break;
      }
    }
  }
  function Ap(n) {
    const t = n.prev, e = n, s = n.next;
    if (Yt(t, e, s) >= 0) return false;
    const i = t.x, r = e.x, o = s.x, a = t.y, l = e.y, c = s.y, h = Math.min(i, r, o), d = Math.min(a, l, c), u = Math.max(i, r, o), p = Math.max(a, l, c);
    let f = s.next;
    for (; f !== t; ) {
      if (f.x >= h && f.x <= u && f.y >= d && f.y <= p && $s(i, a, r, l, o, c, f.x, f.y) && Yt(f.prev, f, f.next) >= 0) return false;
      f = f.next;
    }
    return true;
  }
  function Ep(n, t, e, s) {
    const i = n.prev, r = n, o = n.next;
    if (Yt(i, r, o) >= 0) return false;
    const a = i.x, l = r.x, c = o.x, h = i.y, d = r.y, u = o.y, p = Math.min(a, l, c), f = Math.min(h, d, u), g = Math.max(a, l, c), m = Math.max(h, d, u), y = oo(p, f, t, e, s), b = oo(g, m, t, e, s);
    let x = n.prevZ, _ = n.nextZ;
    for (; x && x.z >= y && _ && _.z <= b; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && $s(a, h, l, d, c, u, x.x, x.y) && Yt(x.prev, x, x.next) >= 0 || (x = x.prevZ, _.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && $s(a, h, l, d, c, u, _.x, _.y) && Yt(_.prev, _, _.next) >= 0)) return false;
      _ = _.nextZ;
    }
    for (; x && x.z >= y; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && $s(a, h, l, d, c, u, x.x, x.y) && Yt(x.prev, x, x.next) >= 0) return false;
      x = x.prevZ;
    }
    for (; _ && _.z <= b; ) {
      if (_.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && $s(a, h, l, d, c, u, _.x, _.y) && Yt(_.prev, _, _.next) >= 0) return false;
      _ = _.nextZ;
    }
    return true;
  }
  function Mp(n, t) {
    let e = n;
    do {
      const s = e.prev, i = e.next.next;
      !ms(s, i) && ah(s, e, e.next, i) && js(s, i) && js(i, s) && (t.push(s.i, e.i, i.i), Vs(e), Vs(e.next), e = n = i), e = e.next;
    } while (e !== n);
    return Un(e);
  }
  function Pp(n, t, e, s, i, r) {
    let o = n;
    do {
      let a = o.next.next;
      for (; a !== o.prev; ) {
        if (o.i !== a.i && Wp(o, a)) {
          let l = lh(o, a);
          o = Un(o, o.next), l = Un(l, l.next), Us(o, t, e, s, i, r, 0), Us(l, t, e, s, i, r, 0);
          return;
        }
        a = a.next;
      }
      o = o.next;
    } while (o !== n);
  }
  function Ip(n, t, e, s) {
    const i = [];
    for (let r = 0, o = t.length; r < o; r++) {
      const a = t[r] * s, l = r < o - 1 ? t[r + 1] * s : n.length, c = rh(n, a, l, s, false);
      c === c.next && (c.steiner = true), i.push(Np(c));
    }
    i.sort(Rp);
    for (let r = 0; r < i.length; r++) e = Lp(i[r], e);
    return e;
  }
  function Rp(n, t) {
    let e = n.x - t.x;
    if (e === 0 && (e = n.y - t.y, e === 0)) {
      const s = (n.next.y - n.y) / (n.next.x - n.x), i = (t.next.y - t.y) / (t.next.x - t.x);
      e = s - i;
    }
    return e;
  }
  function Lp(n, t) {
    const e = $p(n, t);
    if (!e) return t;
    const s = lh(e, n);
    return Un(s, s.next), Un(e, e.next);
  }
  function $p(n, t) {
    let e = t;
    const s = n.x, i = n.y;
    let r = -1 / 0, o;
    if (ms(n, e)) return e;
    do {
      if (ms(n, e.next)) return e.next;
      if (i <= e.y && i >= e.next.y && e.next.y !== e.y) {
        const d = e.x + (i - e.y) * (e.next.x - e.x) / (e.next.y - e.y);
        if (d <= s && d > r && (r = d, o = e.x < e.next.x ? e : e.next, d === s)) return o;
      }
      e = e.next;
    } while (e !== t);
    if (!o) return null;
    const a = o, l = o.x, c = o.y;
    let h = 1 / 0;
    e = o;
    do {
      if (s >= e.x && e.x >= l && s !== e.x && oh(i < c ? s : r, i, l, c, i < c ? r : s, i, e.x, e.y)) {
        const d = Math.abs(i - e.y) / (s - e.x);
        js(e, n) && (d < h || d === h && (e.x > o.x || e.x === o.x && Bp(o, e))) && (o = e, h = d);
      }
      e = e.next;
    } while (e !== a);
    return o;
  }
  function Bp(n, t) {
    return Yt(n.prev, n, t.prev) < 0 && Yt(t.next, n, n.next) < 0;
  }
  function Op(n, t, e, s) {
    let i = n;
    do
      i.z === 0 && (i.z = oo(i.x, i.y, t, e, s)), i.prevZ = i.prev, i.nextZ = i.next, i = i.next;
    while (i !== n);
    i.prevZ.nextZ = null, i.prevZ = null, Fp(i);
  }
  function Fp(n) {
    let t, e = 1;
    do {
      let s = n, i;
      n = null;
      let r = null;
      for (t = 0; s; ) {
        t++;
        let o = s, a = 0;
        for (let c = 0; c < e && (a++, o = o.nextZ, !!o); c++) ;
        let l = e;
        for (; a > 0 || l > 0 && o; ) a !== 0 && (l === 0 || !o || s.z <= o.z) ? (i = s, s = s.nextZ, a--) : (i = o, o = o.nextZ, l--), r ? r.nextZ = i : n = i, i.prevZ = r, r = i;
        s = o;
      }
      r.nextZ = null, e *= 2;
    } while (t > 1);
    return n;
  }
  function oo(n, t, e, s, i) {
    return n = (n - e) * i | 0, t = (t - s) * i | 0, n = (n | n << 8) & 16711935, n = (n | n << 4) & 252645135, n = (n | n << 2) & 858993459, n = (n | n << 1) & 1431655765, t = (t | t << 8) & 16711935, t = (t | t << 4) & 252645135, t = (t | t << 2) & 858993459, t = (t | t << 1) & 1431655765, n | t << 1;
  }
  function Np(n) {
    let t = n, e = n;
    do
      (t.x < e.x || t.x === e.x && t.y < e.y) && (e = t), t = t.next;
    while (t !== n);
    return e;
  }
  function oh(n, t, e, s, i, r, o, a) {
    return (i - o) * (t - a) >= (n - o) * (r - a) && (n - o) * (s - a) >= (e - o) * (t - a) && (e - o) * (r - a) >= (i - o) * (s - a);
  }
  function $s(n, t, e, s, i, r, o, a) {
    return !(n === o && t === a) && oh(n, t, e, s, i, r, o, a);
  }
  function Wp(n, t) {
    return n.next.i !== t.i && n.prev.i !== t.i && !Gp(n, t) && (js(n, t) && js(t, n) && zp(n, t) && (Yt(n.prev, n, t.prev) || Yt(n, t.prev, t)) || ms(n, t) && Yt(n.prev, n, n.next) > 0 && Yt(t.prev, t, t.next) > 0);
  }
  function Yt(n, t, e) {
    return (t.y - n.y) * (e.x - t.x) - (t.x - n.x) * (e.y - t.y);
  }
  function ms(n, t) {
    return n.x === t.x && n.y === t.y;
  }
  function ah(n, t, e, s) {
    const i = ui(Yt(n, t, e)), r = ui(Yt(n, t, s)), o = ui(Yt(e, s, n)), a = ui(Yt(e, s, t));
    return !!(i !== r && o !== a || i === 0 && di(n, e, t) || r === 0 && di(n, s, t) || o === 0 && di(e, n, s) || a === 0 && di(e, t, s));
  }
  function di(n, t, e) {
    return t.x <= Math.max(n.x, e.x) && t.x >= Math.min(n.x, e.x) && t.y <= Math.max(n.y, e.y) && t.y >= Math.min(n.y, e.y);
  }
  function ui(n) {
    return n > 0 ? 1 : n < 0 ? -1 : 0;
  }
  function Gp(n, t) {
    let e = n;
    do {
      if (e.i !== n.i && e.next.i !== n.i && e.i !== t.i && e.next.i !== t.i && ah(e, e.next, n, t)) return true;
      e = e.next;
    } while (e !== n);
    return false;
  }
  function js(n, t) {
    return Yt(n.prev, n, n.next) < 0 ? Yt(n, t, n.next) >= 0 && Yt(n, n.prev, t) >= 0 : Yt(n, t, n.prev) < 0 || Yt(n, n.next, t) < 0;
  }
  function zp(n, t) {
    let e = n, s = false;
    const i = (n.x + t.x) / 2, r = (n.y + t.y) / 2;
    do
      e.y > r != e.next.y > r && e.next.y !== e.y && i < (e.next.x - e.x) * (r - e.y) / (e.next.y - e.y) + e.x && (s = !s), e = e.next;
    while (e !== n);
    return s;
  }
  function lh(n, t) {
    const e = ao(n.i, n.x, n.y), s = ao(t.i, t.x, t.y), i = n.next, r = t.prev;
    return n.next = t, t.prev = n, e.next = i, i.prev = e, s.next = e, e.prev = s, r.next = s, s.prev = r, s;
  }
  function Wa(n, t, e, s) {
    const i = ao(n, t, e);
    return s ? (i.next = s.next, i.prev = s, s.next.prev = i, s.next = i) : (i.prev = i, i.next = i), i;
  }
  function Vs(n) {
    n.next.prev = n.prev, n.prev.next = n.next, n.prevZ && (n.prevZ.nextZ = n.nextZ), n.nextZ && (n.nextZ.prevZ = n.prevZ);
  }
  function ao(n, t, e) {
    return {
      i: n,
      x: t,
      y: e,
      prev: null,
      next: null,
      z: 0,
      prevZ: null,
      nextZ: null,
      steiner: false
    };
  }
  function Dp(n, t, e, s) {
    let i = 0;
    for (let r = t, o = e - s; r < e; r += s) i += (n[o] - n[r]) * (n[r + 1] + n[o + 1]), o = r;
    return i;
  }
  const Hp = Na.default || Na;
  ch = ((n) => (n[n.NONE = 0] = "NONE", n[n.COLOR = 16384] = "COLOR", n[n.STENCIL = 1024] = "STENCIL", n[n.DEPTH = 256] = "DEPTH", n[n.COLOR_DEPTH = 16640] = "COLOR_DEPTH", n[n.COLOR_STENCIL = 17408] = "COLOR_STENCIL", n[n.DEPTH_STENCIL = 1280] = "DEPTH_STENCIL", n[n.ALL = 17664] = "ALL", n))(ch || {});
  Up = class {
    constructor(t) {
      this.items = [], this._name = t;
    }
    emit(t, e, s, i, r, o, a, l) {
      const { name: c, items: h } = this;
      for (let d = 0, u = h.length; d < u; d++) h[d][c](t, e, s, i, r, o, a, l);
      return this;
    }
    add(t) {
      return t[this._name] && (this.remove(t), this.items.push(t)), this;
    }
    remove(t) {
      const e = this.items.indexOf(t);
      return e !== -1 && this.items.splice(e, 1), this;
    }
    contains(t) {
      return this.items.indexOf(t) !== -1;
    }
    removeAll() {
      return this.items.length = 0, this;
    }
    destroy() {
      this.removeAll(), this.items = null, this._name = null;
    }
    get empty() {
      return this.items.length === 0;
    }
    get name() {
      return this._name;
    }
  };
  const jp = [
    "init",
    "destroy",
    "contextChange",
    "resolutionChange",
    "resetState",
    "renderEnd",
    "renderStart",
    "render",
    "update",
    "postrender",
    "prerender"
  ], hh = class dh extends Qe {
    constructor(t) {
      super(), this.tick = 0, this.uid = Kt("renderer"), this.runners = /* @__PURE__ */ Object.create(null), this.renderPipes = /* @__PURE__ */ Object.create(null), this._initOptions = {}, this._systemsHash = /* @__PURE__ */ Object.create(null), this.type = t.type, this.name = t.name, this.config = t;
      const e = [
        ...jp,
        ...this.config.runners ?? []
      ];
      this._addRunners(...e), this._unsafeEvalCheck();
    }
    async init(t = {}) {
      const e = t.skipExtensionImports === true ? true : t.manageImports === false;
      await kp(e), this._addSystems(this.config.systems), this._addPipes(this.config.renderPipes, this.config.renderPipeAdaptors);
      for (const s in this._systemsHash) t = {
        ...this._systemsHash[s].constructor.defaultOptions,
        ...t
      };
      t = {
        ...dh.defaultOptions,
        ...t
      }, this._roundPixels = t.roundPixels ? 1 : 0;
      for (let s = 0; s < this.runners.init.items.length; s++) await this.runners.init.items[s].init(t);
      this._initOptions = t;
    }
    render(t, e) {
      this.tick++;
      let s = t;
      if (s instanceof zt && (s = {
        container: s
      }, e && (Et(qt, "passing a second argument is deprecated, please use render options instead"), s.target = e.renderTexture)), s.target || (s.target = this.view.renderTarget), s.target === this.view.renderTarget && (this._lastObjectRendered = s.container, s.clearColor ?? (s.clearColor = this.background.colorRgba), s.clear ?? (s.clear = this.background.clearBeforeRender)), s.clearColor) {
        const i = Array.isArray(s.clearColor) && s.clearColor.length === 4;
        s.clearColor = i ? s.clearColor : Gt.shared.setValue(s.clearColor).toArray();
      }
      s.transform || (s.container.updateLocalTransform(), s.transform = s.container.localTransform), s.container.visible && (s.container.enableRenderGroup(), this.runners.prerender.emit(s), this.runners.renderStart.emit(s), this.runners.render.emit(s), this.runners.renderEnd.emit(s), this.runners.postrender.emit(s));
    }
    resize(t, e, s) {
      const i = this.view.resolution;
      this.view.resize(t, e, s), this.emit("resize", this.view.screen.width, this.view.screen.height, this.view.resolution), s !== void 0 && s !== i && this.runners.resolutionChange.emit(s);
    }
    clear(t = {}) {
      const e = this;
      t.target || (t.target = e.renderTarget.renderTarget), t.clearColor || (t.clearColor = this.background.colorRgba), t.clear ?? (t.clear = ch.ALL);
      const { clear: s, clearColor: i, target: r, mipLevel: o, layer: a } = t;
      Gt.shared.setValue(i ?? this.background.colorRgba), e.renderTarget.clear(r, s, Gt.shared.toArray(), o ?? 0, a ?? 0);
    }
    get resolution() {
      return this.view.resolution;
    }
    set resolution(t) {
      this.view.resolution = t, this.runners.resolutionChange.emit(t);
    }
    get width() {
      return this.view.texture.frame.width;
    }
    get height() {
      return this.view.texture.frame.height;
    }
    get canvas() {
      return this.view.canvas;
    }
    get lastObjectRendered() {
      return this._lastObjectRendered;
    }
    get renderingToScreen() {
      return this.renderTarget.renderingToScreen;
    }
    get screen() {
      return this.view.screen;
    }
    _addRunners(...t) {
      t.forEach((e) => {
        this.runners[e] = new Up(e);
      });
    }
    _addSystems(t) {
      let e;
      for (e in t) {
        const s = t[e];
        this._addSystem(s.value, s.name);
      }
    }
    _addSystem(t, e) {
      const s = new t(this);
      if (this[e]) throw new Error(`Whoops! The name "${e}" is already in use`);
      this[e] = s, this._systemsHash[e] = s;
      for (const i in this.runners) this.runners[i].add(s);
      return this;
    }
    _addPipes(t, e) {
      const s = e.reduce((i, r) => (i[r.name] = r.value, i), {});
      t.forEach((i) => {
        const r = i.value, o = i.name, a = s[o];
        this.renderPipes[o] = new r(this, a ? new a() : null), this.runners.destroy.add(this.renderPipes[o]);
      });
    }
    destroy(t = false) {
      this.runners.destroy.items.reverse(), this.runners.destroy.emit(t), (t === true || typeof t == "object" && t.releaseGlobalResources) && Qs.release(), Object.values(this.runners).forEach((e) => {
        e.destroy();
      }), this._systemsHash = null, this.renderPipes = null, this.removeAllListeners();
    }
    generateTexture(t) {
      return this.textureGenerator.generateTexture(t);
    }
    get roundPixels() {
      return !!this._roundPixels;
    }
    _unsafeEvalCheck() {
      if (!Tp()) throw new Error("Current environment does not allow unsafe-eval, please use pixi.js/unsafe-eval module to enable support.");
    }
    resetState() {
      this.runners.resetState.emit();
    }
  };
  hh.defaultOptions = {
    resolution: 1,
    failIfMajorPerformanceCaveat: false,
    roundPixels: false
  };
  let pi;
  uh = hh;
  function Vp(n) {
    return pi !== void 0 || (pi = (() => {
      var _a2;
      const t = {
        stencil: true,
        failIfMajorPerformanceCaveat: n ?? uh.defaultOptions.failIfMajorPerformanceCaveat
      };
      try {
        if (!Mt.get().getWebGLRenderingContext()) return false;
        let s = Mt.get().createCanvas().getContext("webgl", t);
        const i = !!((_a2 = s == null ? void 0 : s.getContextAttributes()) == null ? void 0 : _a2.stencil);
        if (s) {
          const r = s.getExtension("WEBGL_lose_context");
          r && r.loseContext();
        }
        return s = null, i;
      } catch {
        return false;
      }
    })()), pi;
  }
  let fi;
  async function Yp(n = {}) {
    return fi !== void 0 || (fi = await (async () => {
      const t = Mt.get().getNavigator().gpu;
      if (!t) return false;
      try {
        return await (await t.requestAdapter(n)).requestDevice(), true;
      } catch {
        return false;
      }
    })()), fi;
  }
  const Ga = [
    "webgl",
    "webgpu",
    "canvas"
  ];
  async function Xp(n) {
    let t = [];
    n.preference ? (t.push(n.preference), Ga.forEach((r) => {
      r !== n.preference && t.push(r);
    })) : t = Ga.slice();
    let e, s = {};
    for (let r = 0; r < t.length; r++) {
      const o = t[r];
      if (o === "webgpu" && await Yp()) {
        const { WebGPURenderer: a } = await Gn(async () => {
          const { WebGPURenderer: l } = await import("./WebGPURenderer-DZwUSLU_.js");
          return {
            WebGPURenderer: l
          };
        }, __vite__mapDeps([3,4,5,2]));
        e = a, s = {
          ...n,
          ...n.webgpu
        };
        break;
      } else if (o === "webgl" && Vp(n.failIfMajorPerformanceCaveat ?? uh.defaultOptions.failIfMajorPerformanceCaveat)) {
        const { WebGLRenderer: a } = await Gn(async () => {
          const { WebGLRenderer: l } = await import("./WebGLRenderer-wnOXY49J.js");
          return {
            WebGLRenderer: l
          };
        }, __vite__mapDeps([6,4,5,2]));
        e = a, s = {
          ...n,
          ...n.webgl
        };
        break;
      } else if (o === "canvas") {
        const { CanvasRenderer: a } = await Gn(async () => {
          const { CanvasRenderer: l } = await import("./CanvasRenderer-CIpnLXhq.js");
          return {
            CanvasRenderer: l
          };
        }, __vite__mapDeps([7,5,2]));
        e = a, s = {
          ...n,
          ...n.canvasOptions
        };
        break;
      }
    }
    if (delete s.webgpu, delete s.webgl, delete s.canvasOptions, !e) throw new Error("No available renderer for the current environment");
    const i = new e();
    return await i.init(s), i;
  }
  ph = "8.17.1";
  class fh {
    static init() {
      var _a2;
      (_a2 = globalThis.__PIXI_APP_INIT__) == null ? void 0 : _a2.call(globalThis, this, ph);
    }
    static destroy() {
    }
  }
  fh.extension = it.Application;
  qp = class {
    constructor(t) {
      this._renderer = t;
    }
    init() {
      var _a2;
      (_a2 = globalThis.__PIXI_RENDERER_INIT__) == null ? void 0 : _a2.call(globalThis, this._renderer, ph);
    }
    destroy() {
      this._renderer = null;
    }
  };
  qp.extension = {
    type: [
      it.WebGLSystem,
      it.WebGPUSystem
    ],
    name: "initHook",
    priority: -10
  };
  class mh {
    static init(t) {
      Object.defineProperty(this, "resizeTo", {
        configurable: true,
        set(e) {
          globalThis.removeEventListener("resize", this.queueResize), this._resizeTo = e, e && (globalThis.addEventListener("resize", this.queueResize), this.resize());
        },
        get() {
          return this._resizeTo;
        }
      }), this.queueResize = () => {
        this._resizeTo && (this._cancelResize(), this._resizeId = requestAnimationFrame(() => this.resize()));
      }, this._cancelResize = () => {
        this._resizeId && (cancelAnimationFrame(this._resizeId), this._resizeId = null);
      }, this.resize = () => {
        if (!this._resizeTo) return;
        this._cancelResize();
        let e, s;
        if (this._resizeTo === globalThis.window) e = globalThis.innerWidth, s = globalThis.innerHeight;
        else {
          const { clientWidth: i, clientHeight: r } = this._resizeTo;
          e = i, s = r;
        }
        this.renderer.resize(e, s), this.render();
      }, this._resizeId = null, this._resizeTo = null, this.resizeTo = t.resizeTo || null;
    }
    static destroy() {
      globalThis.removeEventListener("resize", this.queueResize), this._cancelResize(), this._cancelResize = null, this.queueResize = null, this.resizeTo = null, this.resize = null;
    }
  }
  mh.extension = it.Application;
  class gh {
    static init(t) {
      t = Object.assign({
        autoStart: true,
        sharedTicker: false
      }, t), Object.defineProperty(this, "ticker", {
        configurable: true,
        set(e) {
          this._ticker && this._ticker.remove(this.render, this), this._ticker = e, e && e.add(this.render, this, Hs.LOW);
        },
        get() {
          return this._ticker;
        }
      }), this.stop = () => {
        this._ticker.stop();
      }, this.start = () => {
        this._ticker.start();
      }, this._ticker = null, this.ticker = t.sharedTicker ? Nn.shared : new Nn(), t.autoStart && this.start();
    }
    static destroy() {
      if (this._ticker) {
        const t = this._ticker;
        this.ticker = null, t.destroy();
      }
    }
  }
  gh.extension = it.Application;
  Ot.add(mh);
  Ot.add(gh);
  const yh = class lo {
    constructor(...t) {
      this.stage = new zt(), t[0] !== void 0 && Et(qt, "Application constructor options are deprecated, please use Application.init() instead.");
    }
    async init(t) {
      t = {
        ...t
      }, this.stage || (this.stage = new zt()), this.renderer = await Xp(t), lo._plugins.forEach((e) => {
        e.init.call(this, t);
      });
    }
    render() {
      this.renderer.render({
        container: this.stage
      });
    }
    get canvas() {
      return this.renderer.canvas;
    }
    get view() {
      return Et(qt, "Application.view is deprecated, please use Application.canvas instead."), this.renderer.canvas;
    }
    get screen() {
      return this.renderer.screen;
    }
    destroy(t = false, e = false) {
      const s = lo._plugins.slice(0);
      s.reverse(), s.forEach((i) => {
        i.destroy.call(this);
      }), this.stage.destroy(e), this.stage = null, this.renderer.destroy(t), this.renderer = null;
    }
  };
  yh._plugins = [];
  let No = yh;
  Ot.handleByList(it.Application, No._plugins);
  Ot.add(fh);
  const Tr = {
    test(n) {
      return typeof n == "string" && n.startsWith("info face=");
    },
    parse(n) {
      const t = n.match(/^[a-z]+\s+.+$/gm), e = {
        info: [],
        common: [],
        page: [],
        char: [],
        chars: [],
        kerning: [],
        kernings: [],
        distanceField: []
      };
      for (const d in t) {
        const u = t[d].match(/^[a-z]+/gm)[0], p = t[d].match(/[a-zA-Z]+=([^\s"']+|"([^"]*)")/gm), f = {};
        for (const g in p) {
          const m = p[g].split("="), y = m[0], b = m[1].replace(/"/gm, ""), x = parseFloat(b), _ = isNaN(x) ? b : x;
          f[y] = _;
        }
        e[u].push(f);
      }
      const s = {
        chars: {},
        pages: [],
        lineHeight: 0,
        fontSize: 0,
        fontFamily: "",
        distanceField: null,
        baseLineOffset: 0
      }, [i] = e.info, [r] = e.common, [o] = e.distanceField ?? [];
      o && (s.distanceField = {
        range: parseInt(o.distanceRange, 10),
        type: o.fieldType
      }), s.fontSize = parseInt(i.size, 10), s.fontFamily = i.face, s.lineHeight = parseInt(r.lineHeight, 10);
      const a = e.page;
      for (let d = 0; d < a.length; d++) s.pages.push({
        id: parseInt(a[d].id, 10) || 0,
        file: a[d].file
      });
      const l = {};
      s.baseLineOffset = s.lineHeight - parseInt(r.base, 10);
      const c = e.char;
      for (let d = 0; d < c.length; d++) {
        const u = c[d], p = parseInt(u.id, 10);
        let f = u.letter ?? u.char ?? String.fromCharCode(p);
        f === "space" && (f = " "), l[p] = f, s.chars[f] = {
          id: p,
          page: parseInt(u.page, 10) || 0,
          x: parseInt(u.x, 10),
          y: parseInt(u.y, 10),
          width: parseInt(u.width, 10),
          height: parseInt(u.height, 10),
          xOffset: parseInt(u.xoffset, 10),
          yOffset: parseInt(u.yoffset, 10),
          xAdvance: parseInt(u.xadvance, 10),
          kerning: {}
        };
      }
      const h = e.kerning || [];
      for (let d = 0; d < h.length; d++) {
        const u = parseInt(h[d].first, 10), p = parseInt(h[d].second, 10), f = parseInt(h[d].amount, 10);
        s.chars[l[p]] && (s.chars[l[p]].kerning[l[u]] = f);
      }
      return s;
    }
  }, za = {
    test(n) {
      const t = n;
      return typeof t != "string" && "getElementsByTagName" in t && t.getElementsByTagName("page").length && t.getElementsByTagName("info")[0].getAttribute("face") !== null;
    },
    parse(n) {
      const t = {
        chars: {},
        pages: [],
        lineHeight: 0,
        fontSize: 0,
        fontFamily: "",
        distanceField: null,
        baseLineOffset: 0
      }, e = n.getElementsByTagName("info")[0], s = n.getElementsByTagName("common")[0], i = n.getElementsByTagName("distanceField")[0];
      i && (t.distanceField = {
        type: i.getAttribute("fieldType"),
        range: parseInt(i.getAttribute("distanceRange"), 10)
      });
      const r = n.getElementsByTagName("page"), o = n.getElementsByTagName("char"), a = n.getElementsByTagName("kerning");
      t.fontSize = parseInt(e.getAttribute("size"), 10), t.fontFamily = e.getAttribute("face"), t.lineHeight = parseInt(s.getAttribute("lineHeight"), 10);
      for (let c = 0; c < r.length; c++) t.pages.push({
        id: parseInt(r[c].getAttribute("id"), 10) || 0,
        file: r[c].getAttribute("file")
      });
      const l = {};
      t.baseLineOffset = t.lineHeight - parseInt(s.getAttribute("base"), 10);
      for (let c = 0; c < o.length; c++) {
        const h = o[c], d = parseInt(h.getAttribute("id"), 10);
        let u = h.getAttribute("letter") ?? h.getAttribute("char") ?? String.fromCharCode(d);
        u === "space" && (u = " "), l[d] = u, t.chars[u] = {
          id: d,
          page: parseInt(h.getAttribute("page"), 10) || 0,
          x: parseInt(h.getAttribute("x"), 10),
          y: parseInt(h.getAttribute("y"), 10),
          width: parseInt(h.getAttribute("width"), 10),
          height: parseInt(h.getAttribute("height"), 10),
          xOffset: parseInt(h.getAttribute("xoffset"), 10),
          yOffset: parseInt(h.getAttribute("yoffset"), 10),
          xAdvance: parseInt(h.getAttribute("xadvance"), 10),
          kerning: {}
        };
      }
      for (let c = 0; c < a.length; c++) {
        const h = parseInt(a[c].getAttribute("first"), 10), d = parseInt(a[c].getAttribute("second"), 10), u = parseInt(a[c].getAttribute("amount"), 10);
        t.chars[l[d]] && (t.chars[l[d]].kerning[l[h]] = u);
      }
      return t;
    }
  }, Da = {
    test(n) {
      return typeof n == "string" && n.match(/<font(\s|>)/) ? za.test(Mt.get().parseXML(n)) : false;
    },
    parse(n) {
      return za.parse(Mt.get().parseXML(n));
    }
  }, Kp = [
    ".xml",
    ".fnt"
  ], Jp = {
    extension: {
      type: it.CacheParser,
      name: "cacheBitmapFont"
    },
    test: (n) => !!(n == null ? void 0 : n.pages) && !!(n == null ? void 0 : n.chars) && typeof (n == null ? void 0 : n.fontFamily) == "string" && n.fontFamily !== "",
    getCacheableAssets(n, t) {
      const e = {};
      return n.forEach((s) => {
        e[s] = t, e[`${s}-bitmap`] = t;
      }), e[`${t.fontFamily}-bitmap`] = t, e;
    }
  }, Zp = {
    extension: {
      type: it.LoadParser,
      priority: kn.Normal
    },
    name: "loadBitmapFont",
    id: "bitmap-font",
    test(n) {
      return Kp.includes(Ce.extname(n).toLowerCase());
    },
    async testParse(n) {
      return Tr.test(n) || Da.test(n);
    },
    async parse(n, t, e) {
      const s = Tr.test(n) ? Tr.parse(n) : Da.parse(n), { src: i } = t, { pages: r } = s, o = [], a = s.distanceField ? {
        scaleMode: "linear",
        alphaMode: "premultiply-alpha-on-upload",
        autoGenerateMipmaps: false,
        resolution: 1
      } : {};
      for (let u = 0; u < r.length; ++u) {
        const p = r[u].file;
        let f = Ce.join(Ce.dirname(i), p);
        f = eo(f, i), o.push({
          src: f,
          data: a
        });
      }
      const [l, { BitmapFont: c }] = await Promise.all([
        e.load(o),
        Gn(() => import("./BitmapFont-Dp42tdX1.js"), [])
      ]), h = o.map((u) => l[u.src]);
      return new c({
        data: s,
        textures: h
      }, i);
    },
    async load(n, t) {
      return await (await Mt.get().fetch(n)).text();
    },
    async unload(n, t, e) {
      await Promise.all(n.pages.map((s) => e.unload(s.texture.source._sourceOrigin))), n.destroy();
    }
  };
  class Qp {
    constructor(t, e = false) {
      this._loader = t, this._assetList = [], this._isLoading = false, this._maxConcurrent = 1, this.verbose = e;
    }
    add(t) {
      t.forEach((e) => {
        this._assetList.push(e);
      }), this.verbose && console.log("[BackgroundLoader] assets: ", this._assetList), this._isActive && !this._isLoading && this._next();
    }
    async _next() {
      if (this._assetList.length && this._isActive) {
        this._isLoading = true;
        const t = [], e = Math.min(this._assetList.length, this._maxConcurrent);
        for (let s = 0; s < e; s++) t.push(this._assetList.pop());
        await this._loader.load(t), this._isLoading = false, this._next();
      }
    }
    get active() {
      return this._isActive;
    }
    set active(t) {
      this._isActive !== t && (this._isActive = t, t && !this._isLoading && this._next());
    }
  }
  const tf = {
    extension: {
      type: it.CacheParser,
      name: "cacheTextureArray"
    },
    test: (n) => Array.isArray(n) && n.every((t) => t instanceof Ct),
    getCacheableAssets: (n, t) => {
      const e = {};
      return n.forEach((s) => {
        t.forEach((i, r) => {
          e[s + (r === 0 ? "" : r + 1)] = i;
        });
      }), e;
    }
  };
  async function xh(n) {
    if ("Image" in globalThis) return new Promise((t) => {
      const e = new Image();
      e.onload = () => {
        t(true);
      }, e.onerror = () => {
        t(false);
      }, e.src = n;
    });
    if ("createImageBitmap" in globalThis && "fetch" in globalThis) {
      try {
        const t = await (await fetch(n)).blob();
        await createImageBitmap(t);
      } catch {
        return false;
      }
      return true;
    }
    return false;
  }
  const ef = {
    extension: {
      type: it.DetectionParser,
      priority: 1
    },
    test: async () => xh("data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A="),
    add: async (n) => [
      ...n,
      "avif"
    ],
    remove: async (n) => n.filter((t) => t !== "avif")
  }, Ha = [
    "png",
    "jpg",
    "jpeg"
  ], nf = {
    extension: {
      type: it.DetectionParser,
      priority: -1
    },
    test: () => Promise.resolve(true),
    add: async (n) => [
      ...n,
      ...Ha
    ],
    remove: async (n) => n.filter((t) => !Ha.includes(t))
  }, sf = "WorkerGlobalScope" in globalThis && globalThis instanceof globalThis.WorkerGlobalScope;
  function qi(n) {
    return sf ? false : document.createElement("video").canPlayType(n) !== "";
  }
  const rf = {
    extension: {
      type: it.DetectionParser,
      priority: 0
    },
    test: async () => qi("video/mp4"),
    add: async (n) => [
      ...n,
      "mp4",
      "m4v"
    ],
    remove: async (n) => n.filter((t) => t !== "mp4" && t !== "m4v")
  }, of = {
    extension: {
      type: it.DetectionParser,
      priority: 0
    },
    test: async () => qi("video/ogg"),
    add: async (n) => [
      ...n,
      "ogv"
    ],
    remove: async (n) => n.filter((t) => t !== "ogv")
  }, af = {
    extension: {
      type: it.DetectionParser,
      priority: 0
    },
    test: async () => qi("video/webm"),
    add: async (n) => [
      ...n,
      "webm"
    ],
    remove: async (n) => n.filter((t) => t !== "webm")
  }, lf = {
    extension: {
      type: it.DetectionParser,
      priority: 0
    },
    test: async () => xh("data:image/webp;base64,UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAAAfQ//73v/+BiOh/AAA="),
    add: async (n) => [
      ...n,
      "webp"
    ],
    remove: async (n) => n.filter((t) => t !== "webp")
  }, bh = class Ri {
    constructor() {
      this.loadOptions = {
        ...Ri.defaultOptions
      }, this._parsers = [], this._parsersValidated = false, this.parsers = new Proxy(this._parsers, {
        set: (t, e, s) => (this._parsersValidated = false, t[e] = s, true)
      }), this.promiseCache = {};
    }
    reset() {
      this._parsersValidated = false, this.promiseCache = {};
    }
    _getLoadPromiseAndParser(t, e) {
      const s = {
        promise: null,
        parser: null
      };
      return s.promise = (async () => {
        var _a2, _b2;
        let i = null, r = null;
        if ((e.parser || e.loadParser) && (r = this._parserHash[e.parser || e.loadParser], e.loadParser && Vt(`[Assets] "loadParser" is deprecated, use "parser" instead for ${t}`), r || Vt(`[Assets] specified load parser "${e.parser || e.loadParser}" not found while loading ${t}`)), !r) {
          for (let o = 0; o < this.parsers.length; o++) {
            const a = this.parsers[o];
            if (a.load && ((_a2 = a.test) == null ? void 0 : _a2.call(a, t, e, this))) {
              r = a;
              break;
            }
          }
          if (!r) return Vt(`[Assets] ${t} could not be loaded as we don't know how to parse it, ensure the correct parser has been added`), null;
        }
        i = await r.load(t, e, this), s.parser = r;
        for (let o = 0; o < this.parsers.length; o++) {
          const a = this.parsers[o];
          a.parse && a.parse && await ((_b2 = a.testParse) == null ? void 0 : _b2.call(a, i, e, this)) && (i = await a.parse(i, e, this) || i, s.parser = a);
        }
        return i;
      })(), s;
    }
    async load(t, e) {
      this._parsersValidated || this._validateParsers();
      const s = typeof e == "function" ? {
        ...Ri.defaultOptions,
        ...this.loadOptions,
        onProgress: e
      } : {
        ...Ri.defaultOptions,
        ...this.loadOptions,
        ...e || {}
      }, { onProgress: i, onError: r, strategy: o, retryCount: a, retryDelay: l } = s;
      let c = 0;
      const h = {}, d = $i(t), u = We(t, (g) => ({
        alias: [
          g
        ],
        src: g,
        data: {}
      })), p = u.reduce((g, m) => g + (m.progressSize || 1), 0), f = u.map(async (g) => {
        const m = Ce.toAbsolute(g.src);
        h[g.src] || (await this._loadAssetWithRetry(m, g, {
          onProgress: i,
          onError: r,
          strategy: o,
          retryCount: a,
          retryDelay: l
        }, h), c += g.progressSize || 1, i && i(c / p));
      });
      return await Promise.all(f), d ? h[u[0].src] : h;
    }
    async unload(t) {
      const s = We(t, (i) => ({
        alias: [
          i
        ],
        src: i
      })).map(async (i) => {
        var _a2, _b2;
        const r = Ce.toAbsolute(i.src), o = this.promiseCache[r];
        if (o) {
          const a = await o.promise;
          delete this.promiseCache[r], await ((_b2 = (_a2 = o.parser) == null ? void 0 : _a2.unload) == null ? void 0 : _b2.call(_a2, a, i, this));
        }
      });
      await Promise.all(s);
    }
    _validateParsers() {
      this._parsersValidated = true, this._parserHash = this._parsers.filter((t) => t.name || t.id).reduce((t, e) => (!e.name && !e.id ? Vt("[Assets] parser should have an id") : (t[e.name] || t[e.id]) && Vt(`[Assets] parser id conflict "${e.id}"`), t[e.name] = e, e.id && (t[e.id] = e), t), {});
    }
    async _loadAssetWithRetry(t, e, s, i) {
      let r = 0;
      const { onError: o, strategy: a, retryCount: l, retryDelay: c } = s, h = (d) => new Promise((u) => setTimeout(u, d));
      for (; ; ) try {
        this.promiseCache[t] || (this.promiseCache[t] = this._getLoadPromiseAndParser(t, e)), i[e.src] = await this.promiseCache[t].promise;
        return;
      } catch (d) {
        delete this.promiseCache[t], delete i[e.src], r++;
        const u = a !== "retry" || r > l;
        if (a === "retry" && !u) {
          o && o(d, e), await h(c);
          continue;
        }
        if (a === "skip") {
          o && o(d, e);
          return;
        }
        o && o(d, e);
        const p = new Error(`[Loader.load] Failed to load ${t}.
${d}`);
        throw d instanceof Error && d.stack && (p.stack = d.stack), p;
      }
    }
  };
  bh.defaultOptions = {
    onProgress: void 0,
    onError: void 0,
    strategy: "throw",
    retryCount: 3,
    retryDelay: 250
  };
  let cf = bh;
  function xs(n, t) {
    if (Array.isArray(t)) {
      for (const e of t) if (n.startsWith(`data:${e}`)) return true;
      return false;
    }
    return n.startsWith(`data:${t}`);
  }
  function bs(n, t) {
    const e = n.split("?")[0], s = Ce.extname(e).toLowerCase();
    return Array.isArray(t) ? t.includes(s) : s === t;
  }
  const hf = ".json", df = "application/json", uf = {
    extension: {
      type: it.LoadParser,
      priority: kn.Low
    },
    name: "loadJson",
    id: "json",
    test(n) {
      return xs(n, df) || bs(n, hf);
    },
    async load(n) {
      return await (await Mt.get().fetch(n)).json();
    }
  }, pf = ".txt", ff = "text/plain", mf = {
    name: "loadTxt",
    id: "text",
    extension: {
      type: it.LoadParser,
      priority: kn.Low,
      name: "loadTxt"
    },
    test(n) {
      return xs(n, ff) || bs(n, pf);
    },
    async load(n) {
      return await (await Mt.get().fetch(n)).text();
    }
  }, gf = [
    "normal",
    "bold",
    "100",
    "200",
    "300",
    "400",
    "500",
    "600",
    "700",
    "800",
    "900"
  ], yf = [
    ".ttf",
    ".otf",
    ".woff",
    ".woff2"
  ], xf = [
    "font/ttf",
    "font/otf",
    "font/woff",
    "font/woff2"
  ], bf = /^(--|-?[A-Z_])[0-9A-Z_-]*$/i;
  function _f(n) {
    const t = Ce.extname(n), i = Ce.basename(n, t).replace(/(-|_)/g, " ").toLowerCase().split(" ").map((a) => a.charAt(0).toUpperCase() + a.slice(1));
    let r = i.length > 0;
    for (const a of i) if (!a.match(bf)) {
      r = false;
      break;
    }
    let o = i.join(" ");
    return r || (o = `"${o.replace(/[\\"]/g, "\\$&")}"`), o;
  }
  const wf = /^[0-9A-Za-z%:/?#\[\]@!\$&'()\*\+,;=\-._~]*$/;
  function vf(n) {
    return wf.test(n) ? n : encodeURI(n);
  }
  const Cf = {
    extension: {
      type: it.LoadParser,
      priority: kn.Low
    },
    name: "loadWebFont",
    id: "web-font",
    test(n) {
      return xs(n, xf) || bs(n, yf);
    },
    async load(n, t) {
      var _a2, _b2, _c2;
      const e = Mt.get().getFontFaceSet();
      if (e) {
        const s = [], i = ((_a2 = t.data) == null ? void 0 : _a2.family) ?? _f(n), r = ((_c2 = (_b2 = t.data) == null ? void 0 : _b2.weights) == null ? void 0 : _c2.filter((a) => gf.includes(a))) ?? [
          "normal"
        ], o = t.data ?? {};
        for (let a = 0; a < r.length; a++) {
          const l = r[a], c = new FontFace(i, `url('${vf(n)}')`, {
            ...o,
            weight: l
          });
          await c.load(), e.add(c), s.push(c);
        }
        return te.has(`${i}-and-url`) ? te.get(`${i}-and-url`).entries.push({
          url: n,
          faces: s
        }) : te.set(`${i}-and-url`, {
          entries: [
            {
              url: n,
              faces: s
            }
          ]
        }), s.length === 1 ? s[0] : s;
      }
      return Vt("[loadWebFont] FontFace API is not supported. Skipping loading font"), null;
    },
    unload(n) {
      const t = Array.isArray(n) ? n : [
        n
      ], e = t[0].family, s = te.get(`${e}-and-url`), i = s.entries.find((r) => r.faces.some((o) => t.indexOf(o) !== -1));
      i.faces = i.faces.filter((r) => t.indexOf(r) === -1), i.faces.length === 0 && (s.entries = s.entries.filter((r) => r !== i)), t.forEach((r) => {
        Mt.get().getFontFaceSet().delete(r);
      }), s.entries.length === 0 && te.remove(`${e}-and-url`);
    }
  };
  var Ar, Ua;
  function Sf() {
    if (Ua) return Ar;
    Ua = 1, Ar = e;
    var n = {
      a: 7,
      c: 6,
      h: 1,
      l: 2,
      m: 2,
      q: 4,
      s: 4,
      t: 2,
      v: 1,
      z: 0
    }, t = /([astvzqmhlc])([^astvzqmhlc]*)/ig;
    function e(r) {
      var o = [];
      return r.replace(t, function(a, l, c) {
        var h = l.toLowerCase();
        for (c = i(c), h == "m" && c.length > 2 && (o.push([
          l
        ].concat(c.splice(0, 2))), h = "l", l = l == "m" ? "l" : "L"); ; ) {
          if (c.length == n[h]) return c.unshift(l), o.push(c);
          if (c.length < n[h]) throw new Error("malformed path data");
          o.push([
            l
          ].concat(c.splice(0, n[h])));
        }
      }), o;
    }
    var s = /-?[0-9]*\.?[0-9]+(?:e[-+]?\d+)?/ig;
    function i(r) {
      var o = r.match(s);
      return o ? o.map(Number) : [];
    }
    return Ar;
  }
  var kf = Sf();
  const Tf = wc(kf);
  function Af(n, t) {
    const e = Tf(n), s = [];
    let i = null, r = 0, o = 0;
    for (let a = 0; a < e.length; a++) {
      const l = e[a], c = l[0], h = l;
      switch (c) {
        case "M":
          r = h[1], o = h[2], t.moveTo(r, o);
          break;
        case "m":
          r += h[1], o += h[2], t.moveTo(r, o);
          break;
        case "H":
          r = h[1], t.lineTo(r, o);
          break;
        case "h":
          r += h[1], t.lineTo(r, o);
          break;
        case "V":
          o = h[1], t.lineTo(r, o);
          break;
        case "v":
          o += h[1], t.lineTo(r, o);
          break;
        case "L":
          r = h[1], o = h[2], t.lineTo(r, o);
          break;
        case "l":
          r += h[1], o += h[2], t.lineTo(r, o);
          break;
        case "C":
          r = h[5], o = h[6], t.bezierCurveTo(h[1], h[2], h[3], h[4], r, o);
          break;
        case "c":
          t.bezierCurveTo(r + h[1], o + h[2], r + h[3], o + h[4], r + h[5], o + h[6]), r += h[5], o += h[6];
          break;
        case "S":
          r = h[3], o = h[4], t.bezierCurveToShort(h[1], h[2], r, o);
          break;
        case "s":
          t.bezierCurveToShort(r + h[1], o + h[2], r + h[3], o + h[4]), r += h[3], o += h[4];
          break;
        case "Q":
          r = h[3], o = h[4], t.quadraticCurveTo(h[1], h[2], r, o);
          break;
        case "q":
          t.quadraticCurveTo(r + h[1], o + h[2], r + h[3], o + h[4]), r += h[3], o += h[4];
          break;
        case "T":
          r = h[1], o = h[2], t.quadraticCurveToShort(r, o);
          break;
        case "t":
          r += h[1], o += h[2], t.quadraticCurveToShort(r, o);
          break;
        case "A":
          r = h[6], o = h[7], t.arcToSvg(h[1], h[2], h[3], h[4], h[5], r, o);
          break;
        case "a":
          r += h[6], o += h[7], t.arcToSvg(h[1], h[2], h[3], h[4], h[5], r, o);
          break;
        case "Z":
        case "z":
          t.closePath(), s.length > 0 && (i = s.pop(), i ? (r = i.startX, o = i.startY) : (r = 0, o = 0)), i = null;
          break;
        default:
          Vt(`Unknown SVG path command: ${c}`);
      }
      c !== "Z" && c !== "z" && i === null && (i = {
        startX: r,
        startY: o
      }, s.push(i));
    }
    return t;
  }
  class Wo {
    constructor(t = 0, e = 0, s = 0) {
      this.type = "circle", this.x = t, this.y = e, this.radius = s;
    }
    clone() {
      return new Wo(this.x, this.y, this.radius);
    }
    contains(t, e) {
      if (this.radius <= 0) return false;
      const s = this.radius * this.radius;
      let i = this.x - t, r = this.y - e;
      return i *= i, r *= r, i + r <= s;
    }
    strokeContains(t, e, s, i = 0.5) {
      if (this.radius === 0) return false;
      const r = this.x - t, o = this.y - e, a = this.radius, l = (1 - i) * s, c = Math.sqrt(r * r + o * o);
      return c <= a + l && c > a - (s - l);
    }
    getBounds(t) {
      return t || (t = new Nt()), t.x = this.x - this.radius, t.y = this.y - this.radius, t.width = this.radius * 2, t.height = this.radius * 2, t;
    }
    copyFrom(t) {
      return this.x = t.x, this.y = t.y, this.radius = t.radius, this;
    }
    copyTo(t) {
      return t.copyFrom(this), t;
    }
    toString() {
      return `[pixi.js/math:Circle x=${this.x} y=${this.y} radius=${this.radius}]`;
    }
  }
  class Go {
    constructor(t = 0, e = 0, s = 0, i = 0) {
      this.type = "ellipse", this.x = t, this.y = e, this.halfWidth = s, this.halfHeight = i;
    }
    clone() {
      return new Go(this.x, this.y, this.halfWidth, this.halfHeight);
    }
    contains(t, e) {
      if (this.halfWidth <= 0 || this.halfHeight <= 0) return false;
      let s = (t - this.x) / this.halfWidth, i = (e - this.y) / this.halfHeight;
      return s *= s, i *= i, s + i <= 1;
    }
    strokeContains(t, e, s, i = 0.5) {
      const { halfWidth: r, halfHeight: o } = this;
      if (r <= 0 || o <= 0) return false;
      const a = s * (1 - i), l = s - a, c = r - l, h = o - l, d = r + a, u = o + a, p = t - this.x, f = e - this.y, g = p * p / (c * c) + f * f / (h * h), m = p * p / (d * d) + f * f / (u * u);
      return g > 1 && m <= 1;
    }
    getBounds(t) {
      return t || (t = new Nt()), t.x = this.x - this.halfWidth, t.y = this.y - this.halfHeight, t.width = this.halfWidth * 2, t.height = this.halfHeight * 2, t;
    }
    copyFrom(t) {
      return this.x = t.x, this.y = t.y, this.halfWidth = t.halfWidth, this.halfHeight = t.halfHeight, this;
    }
    copyTo(t) {
      return t.copyFrom(this), t;
    }
    toString() {
      return `[pixi.js/math:Ellipse x=${this.x} y=${this.y} halfWidth=${this.halfWidth} halfHeight=${this.halfHeight}]`;
    }
  }
  function Ef(n, t, e, s, i, r) {
    const o = n - e, a = t - s, l = i - e, c = r - s, h = o * l + a * c, d = l * l + c * c;
    let u = -1;
    d !== 0 && (u = h / d);
    let p, f;
    u < 0 ? (p = e, f = s) : u > 1 ? (p = i, f = r) : (p = e + u * l, f = s + u * c);
    const g = n - p, m = t - f;
    return g * g + m * m;
  }
  let Mf, Pf;
  class Ns {
    constructor(...t) {
      this.type = "polygon";
      let e = Array.isArray(t[0]) ? t[0] : t;
      if (typeof e[0] != "number") {
        const s = [];
        for (let i = 0, r = e.length; i < r; i++) s.push(e[i].x, e[i].y);
        e = s;
      }
      this.points = e, this.closePath = true;
    }
    isClockwise() {
      let t = 0;
      const e = this.points, s = e.length;
      for (let i = 0; i < s; i += 2) {
        const r = e[i], o = e[i + 1], a = e[(i + 2) % s], l = e[(i + 3) % s];
        t += (a - r) * (l + o);
      }
      return t < 0;
    }
    containsPolygon(t) {
      const e = this.getBounds(Mf), s = t.getBounds(Pf);
      if (!e.containsRect(s)) return false;
      const i = t.points;
      for (let r = 0; r < i.length; r += 2) {
        const o = i[r], a = i[r + 1];
        if (!this.contains(o, a)) return false;
      }
      return true;
    }
    clone() {
      const t = this.points.slice(), e = new Ns(t);
      return e.closePath = this.closePath, e;
    }
    contains(t, e) {
      let s = false;
      const i = this.points.length / 2;
      for (let r = 0, o = i - 1; r < i; o = r++) {
        const a = this.points[r * 2], l = this.points[r * 2 + 1], c = this.points[o * 2], h = this.points[o * 2 + 1];
        l > e != h > e && t < (c - a) * ((e - l) / (h - l)) + a && (s = !s);
      }
      return s;
    }
    strokeContains(t, e, s, i = 0.5) {
      const r = s * s, o = r * (1 - i), a = r - o, { points: l } = this, c = l.length - (this.closePath ? 0 : 2);
      for (let h = 0; h < c; h += 2) {
        const d = l[h], u = l[h + 1], p = l[(h + 2) % l.length], f = l[(h + 3) % l.length], g = Ef(t, e, d, u, p, f), m = Math.sign((p - d) * (e - u) - (f - u) * (t - d));
        if (g <= (m < 0 ? a : o)) return true;
      }
      return false;
    }
    getBounds(t) {
      t || (t = new Nt());
      const e = this.points;
      let s = 1 / 0, i = -1 / 0, r = 1 / 0, o = -1 / 0;
      for (let a = 0, l = e.length; a < l; a += 2) {
        const c = e[a], h = e[a + 1];
        s = c < s ? c : s, i = c > i ? c : i, r = h < r ? h : r, o = h > o ? h : o;
      }
      return t.x = s, t.width = i - s, t.y = r, t.height = o - r, t;
    }
    copyFrom(t) {
      return this.points = t.points.slice(), this.closePath = t.closePath, this;
    }
    copyTo(t) {
      return t.copyFrom(this), t;
    }
    toString() {
      return `[pixi.js/math:PolygoncloseStroke=${this.closePath}points=${this.points.reduce((t, e) => `${t}, ${e}`, "")}]`;
    }
    get lastX() {
      return this.points[this.points.length - 2];
    }
    get lastY() {
      return this.points[this.points.length - 1];
    }
    get x() {
      return Et("8.11.0", "Polygon.lastX is deprecated, please use Polygon.lastX instead."), this.points[this.points.length - 2];
    }
    get y() {
      return Et("8.11.0", "Polygon.y is deprecated, please use Polygon.lastY instead."), this.points[this.points.length - 1];
    }
    get startX() {
      return this.points[0];
    }
    get startY() {
      return this.points[1];
    }
  }
  const mi = (n, t, e, s, i, r, o) => {
    const a = n - e, l = t - s, c = Math.sqrt(a * a + l * l);
    return c >= i - r && c <= i + o;
  };
  class zo {
    constructor(t = 0, e = 0, s = 0, i = 0, r = 20) {
      this.type = "roundedRectangle", this.x = t, this.y = e, this.width = s, this.height = i, this.radius = r;
    }
    getBounds(t) {
      return t || (t = new Nt()), t.x = this.x, t.y = this.y, t.width = this.width, t.height = this.height, t;
    }
    clone() {
      return new zo(this.x, this.y, this.width, this.height, this.radius);
    }
    copyFrom(t) {
      return this.x = t.x, this.y = t.y, this.width = t.width, this.height = t.height, this;
    }
    copyTo(t) {
      return t.copyFrom(this), t;
    }
    contains(t, e) {
      if (this.width <= 0 || this.height <= 0) return false;
      if (t >= this.x && t <= this.x + this.width && e >= this.y && e <= this.y + this.height) {
        const s = Math.max(0, Math.min(this.radius, Math.min(this.width, this.height) / 2));
        if (e >= this.y + s && e <= this.y + this.height - s || t >= this.x + s && t <= this.x + this.width - s) return true;
        let i = t - (this.x + s), r = e - (this.y + s);
        const o = s * s;
        if (i * i + r * r <= o || (i = t - (this.x + this.width - s), i * i + r * r <= o) || (r = e - (this.y + this.height - s), i * i + r * r <= o) || (i = t - (this.x + s), i * i + r * r <= o)) return true;
      }
      return false;
    }
    strokeContains(t, e, s, i = 0.5) {
      const { x: r, y: o, width: a, height: l, radius: c } = this, h = s * (1 - i), d = s - h, u = r + c, p = o + c, f = a - c * 2, g = l - c * 2, m = r + a, y = o + l;
      return (t >= r - h && t <= r + d || t >= m - d && t <= m + h) && e >= p && e <= p + g || (e >= o - h && e <= o + d || e >= y - d && e <= y + h) && t >= u && t <= u + f ? true : t < u && e < p && mi(t, e, u, p, c, d, h) || t > m - c && e < p && mi(t, e, m - c, p, c, d, h) || t > m - c && e > y - c && mi(t, e, m - c, y - c, c, d, h) || t < u && e > y - c && mi(t, e, u, y - c, c, d, h);
    }
    toString() {
      return `[pixi.js/math:RoundedRectangle x=${this.x} y=${this.y}width=${this.width} height=${this.height} radius=${this.radius}]`;
    }
  }
  const _h = {};
  If = function(n, t, e) {
    let s = 2166136261;
    for (let i = 0; i < t; i++) s ^= n[i].uid, s = Math.imul(s, 16777619), s >>>= 0;
    return _h[s] || Rf(n, t, s, e);
  };
  function Rf(n, t, e, s) {
    const i = {};
    let r = 0;
    for (let a = 0; a < s; a++) {
      const l = a < t ? n[a] : Ct.EMPTY.source;
      i[r++] = l.source, i[r++] = l.style;
    }
    const o = new Ii(i);
    return _h[e] = o, o;
  }
  class os {
    constructor(t) {
      typeof t == "number" ? this.rawBinaryData = new ArrayBuffer(t) : t instanceof Uint8Array ? this.rawBinaryData = t.buffer : this.rawBinaryData = t, this.uint32View = new Uint32Array(this.rawBinaryData), this.float32View = new Float32Array(this.rawBinaryData), this.size = this.rawBinaryData.byteLength;
    }
    get int8View() {
      return this._int8View || (this._int8View = new Int8Array(this.rawBinaryData)), this._int8View;
    }
    get uint8View() {
      return this._uint8View || (this._uint8View = new Uint8Array(this.rawBinaryData)), this._uint8View;
    }
    get int16View() {
      return this._int16View || (this._int16View = new Int16Array(this.rawBinaryData)), this._int16View;
    }
    get int32View() {
      return this._int32View || (this._int32View = new Int32Array(this.rawBinaryData)), this._int32View;
    }
    get float64View() {
      return this._float64Array || (this._float64Array = new Float64Array(this.rawBinaryData)), this._float64Array;
    }
    get bigUint64View() {
      return this._bigUint64Array || (this._bigUint64Array = new BigUint64Array(this.rawBinaryData)), this._bigUint64Array;
    }
    view(t) {
      return this[`${t}View`];
    }
    destroy() {
      this.rawBinaryData = null, this.uint32View = null, this.float32View = null, this.uint16View = null, this._int8View = null, this._uint8View = null, this._int16View = null, this._int32View = null, this._float64Array = null, this._bigUint64Array = null;
    }
    static sizeOf(t) {
      switch (t) {
        case "int8":
        case "uint8":
          return 1;
        case "int16":
        case "uint16":
          return 2;
        case "int32":
        case "uint32":
        case "float32":
          return 4;
        default:
          throw new Error(`${t} isn't a valid view type`);
      }
    }
  }
  ja = function(n, t, e, s) {
    if (e ?? (e = 0), s ?? (s = Math.min(n.byteLength - e, t.byteLength)), !(e & 7) && !(s & 7)) {
      const i = s / 8;
      new Float64Array(t, 0, i).set(new Float64Array(n, e, i));
    } else if (!(e & 3) && !(s & 3)) {
      const i = s / 4;
      new Float32Array(t, 0, i).set(new Float32Array(n, e, i));
    } else new Uint8Array(t).set(new Uint8Array(n, e, s));
  };
  const Lf = {
    normal: "normal-npm",
    add: "add-npm",
    screen: "screen-npm"
  };
  $f = ((n) => (n[n.DISABLED = 0] = "DISABLED", n[n.RENDERING_MASK_ADD = 1] = "RENDERING_MASK_ADD", n[n.MASK_ACTIVE = 2] = "MASK_ACTIVE", n[n.INVERSE_MASK_ACTIVE = 3] = "INVERSE_MASK_ACTIVE", n[n.RENDERING_MASK_REMOVE = 4] = "RENDERING_MASK_REMOVE", n[n.NONE = 5] = "NONE", n))($f || {});
  function co(n, t) {
    return t.alphaMode === "no-premultiply-alpha" && Lf[n] || n;
  }
  const Bf = [
    "precision mediump float;",
    "void main(void){",
    "float test = 0.1;",
    "%forloop%",
    "gl_FragColor = vec4(0.0);",
    "}"
  ].join(`
`);
  function Of(n) {
    let t = "";
    for (let e = 0; e < n; ++e) e > 0 && (t += `
else `), e < n - 1 && (t += `if(test == ${e}.0){}`);
    return t;
  }
  Ff = function(n, t) {
    if (n === 0) throw new Error("Invalid value of `0` passed to `checkMaxIfStatementsInShader`");
    const e = t.createShader(t.FRAGMENT_SHADER);
    try {
      for (; ; ) {
        const s = Bf.replace(/%forloop%/gi, Of(n));
        if (t.shaderSource(e, s), t.compileShader(e), !t.getShaderParameter(e, t.COMPILE_STATUS)) n = n / 2 | 0;
        else break;
      }
    } finally {
      t.deleteShader(e);
    }
    return n;
  };
  let Jn = null;
  function Nf() {
    var _a2;
    if (Jn) return Jn;
    const n = Qc();
    return Jn = n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS), Jn = Ff(Jn, n), (_a2 = n.getExtension("WEBGL_lose_context")) == null ? void 0 : _a2.loseContext(), Jn;
  }
  class Wf {
    constructor() {
      this.ids = /* @__PURE__ */ Object.create(null), this.textures = [], this.count = 0;
    }
    clear() {
      for (let t = 0; t < this.count; t++) {
        const e = this.textures[t];
        this.textures[t] = null, this.ids[e.uid] = null;
      }
      this.count = 0;
    }
  }
  class Gf {
    constructor() {
      this.renderPipeId = "batch", this.action = "startBatch", this.start = 0, this.size = 0, this.textures = new Wf(), this.blendMode = "normal", this.topology = "triangle-strip", this.canBundle = true;
    }
    destroy() {
      this.textures = null, this.gpuBindGroup = null, this.bindGroup = null, this.batcher = null, this.elements = null;
    }
  }
  const Ws = [];
  let Fi = 0;
  Qs.register({
    clear: () => {
      if (Ws.length > 0) for (const n of Ws) n && n.destroy();
      Ws.length = 0, Fi = 0;
    }
  });
  function Va() {
    return Fi > 0 ? Ws[--Fi] : new Gf();
  }
  function Ya(n) {
    n.elements = null, Ws[Fi++] = n;
  }
  let Ts = 0;
  const wh = class vh {
    constructor(t) {
      this.uid = Kt("batcher"), this.dirty = true, this.batchIndex = 0, this.batches = [], this._elements = [], t = {
        ...vh.defaultOptions,
        ...t
      }, t.maxTextures || (Et("v8.8.0", "maxTextures is a required option for Batcher now, please pass it in the options"), t.maxTextures = Nf());
      const { maxTextures: e, attributesInitialSize: s, indicesInitialSize: i } = t;
      this.attributeBuffer = new os(s * 4), this.indexBuffer = new Uint16Array(i), this.maxTextures = e;
    }
    begin() {
      this.elementSize = 0, this.elementStart = 0, this.indexSize = 0, this.attributeSize = 0;
      for (let t = 0; t < this.batchIndex; t++) Ya(this.batches[t]);
      this.batchIndex = 0, this._batchIndexStart = 0, this._batchIndexSize = 0, this.dirty = true;
    }
    add(t) {
      this._elements[this.elementSize++] = t, t._indexStart = this.indexSize, t._attributeStart = this.attributeSize, t._batcher = this, this.indexSize += t.indexSize, this.attributeSize += t.attributeSize * this.vertexSize;
    }
    checkAndUpdateTexture(t, e) {
      const s = t._batch.textures.ids[e._source.uid];
      return !s && s !== 0 ? false : (t._textureId = s, t.texture = e, true);
    }
    updateElement(t) {
      this.dirty = true;
      const e = this.attributeBuffer;
      t.packAsQuad ? this.packQuadAttributes(t, e.float32View, e.uint32View, t._attributeStart, t._textureId) : this.packAttributes(t, e.float32View, e.uint32View, t._attributeStart, t._textureId);
    }
    break(t) {
      const e = this._elements;
      if (!e[this.elementStart]) return;
      let s = Va(), i = s.textures;
      i.clear();
      const r = e[this.elementStart];
      let o = co(r.blendMode, r.texture._source), a = r.topology;
      this.attributeSize * 4 > this.attributeBuffer.size && this._resizeAttributeBuffer(this.attributeSize * 4), this.indexSize > this.indexBuffer.length && this._resizeIndexBuffer(this.indexSize);
      const l = this.attributeBuffer.float32View, c = this.attributeBuffer.uint32View, h = this.indexBuffer;
      let d = this._batchIndexSize, u = this._batchIndexStart, p = "startBatch", f = [];
      const g = this.maxTextures;
      for (let m = this.elementStart; m < this.elementSize; ++m) {
        const y = e[m];
        e[m] = null;
        const x = y.texture._source, _ = co(y.blendMode, x), v = o !== _ || a !== y.topology;
        if (x._batchTick === Ts && !v) {
          y._textureId = x._textureBindLocation, d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize)), y._batch = s, f.push(y);
          continue;
        }
        x._batchTick = Ts, (i.count >= g || v) && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), p = "renderBatch", u = d, o = _, a = y.topology, s = Va(), i = s.textures, i.clear(), f = [], ++Ts), y._textureId = x._textureBindLocation = i.count, i.ids[x.uid] = i.count, i.textures[i.count++] = x, y._batch = s, f.push(y), d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize));
      }
      i.count > 0 && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), u = d, ++Ts), this.elementStart = this.elementSize, this._batchIndexStart = u, this._batchIndexSize = d;
    }
    _finishBatch(t, e, s, i, r, o, a, l, c) {
      t.gpuBindGroup = null, t.bindGroup = null, t.action = l, t.batcher = this, t.textures = i, t.blendMode = r, t.topology = o, t.start = e, t.size = s, t.elements = c, ++Ts, this.batches[this.batchIndex++] = t, a.add(t);
    }
    finish(t) {
      this.break(t);
    }
    ensureAttributeBuffer(t) {
      t * 4 <= this.attributeBuffer.size || this._resizeAttributeBuffer(t * 4);
    }
    ensureIndexBuffer(t) {
      t <= this.indexBuffer.length || this._resizeIndexBuffer(t);
    }
    _resizeAttributeBuffer(t) {
      const e = Math.max(t, this.attributeBuffer.size * 2), s = new os(e);
      ja(this.attributeBuffer.rawBinaryData, s.rawBinaryData), this.attributeBuffer = s;
    }
    _resizeIndexBuffer(t) {
      const e = this.indexBuffer;
      let s = Math.max(t, e.length * 1.5);
      s += s % 2;
      const i = s > 65535 ? new Uint32Array(s) : new Uint16Array(s);
      if (i.BYTES_PER_ELEMENT !== e.BYTES_PER_ELEMENT) for (let r = 0; r < e.length; r++) i[r] = e[r];
      else ja(e.buffer, i.buffer);
      this.indexBuffer = i;
    }
    packQuadIndex(t, e, s) {
      t[e] = s + 0, t[e + 1] = s + 1, t[e + 2] = s + 2, t[e + 3] = s + 0, t[e + 4] = s + 2, t[e + 5] = s + 3;
    }
    packIndex(t, e, s, i) {
      const r = t.indices, o = t.indexSize, a = t.indexOffset, l = t.attributeOffset;
      for (let c = 0; c < o; c++) e[s++] = i + r[c + a] - l;
    }
    destroy(t = {}) {
      var _a2;
      if (this.batches !== null) {
        for (let e = 0; e < this.batchIndex; e++) Ya(this.batches[e]);
        this.batches = null, this.geometry.destroy(true), this.geometry = null, t.shader && ((_a2 = this.shader) == null ? void 0 : _a2.destroy(), this.shader = null);
        for (let e = 0; e < this._elements.length; e++) this._elements[e] && (this._elements[e]._batch = null);
        this._elements = null, this.indexBuffer = null, this.attributeBuffer.destroy(), this.attributeBuffer = null;
      }
    }
  };
  wh.defaultOptions = {
    maxTextures: null,
    attributesInitialSize: 4,
    indicesInitialSize: 6
  };
  let zf = wh;
  re = ((n) => (n[n.MAP_READ = 1] = "MAP_READ", n[n.MAP_WRITE = 2] = "MAP_WRITE", n[n.COPY_SRC = 4] = "COPY_SRC", n[n.COPY_DST = 8] = "COPY_DST", n[n.INDEX = 16] = "INDEX", n[n.VERTEX = 32] = "VERTEX", n[n.UNIFORM = 64] = "UNIFORM", n[n.STORAGE = 128] = "STORAGE", n[n.INDIRECT = 256] = "INDIRECT", n[n.QUERY_RESOLVE = 512] = "QUERY_RESOLVE", n[n.STATIC = 1024] = "STATIC", n))(re || {});
  jn = class extends Qe {
    constructor(t) {
      let { data: e, size: s } = t;
      const { usage: i, label: r, shrinkToFit: o } = t;
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.autoGarbageCollect = true, this.uid = Kt("buffer"), this._resourceType = "buffer", this._resourceId = Kt("resource"), this._touched = 0, this._updateID = 1, this._dataInt32 = null, this.shrinkToFit = true, this.destroyed = false, e instanceof Array && (e = new Float32Array(e)), this._data = e, s ?? (s = e == null ? void 0 : e.byteLength);
      const a = !!e;
      this.descriptor = {
        size: s,
        usage: i,
        mappedAtCreation: a,
        label: r
      }, this.shrinkToFit = o ?? true;
    }
    get data() {
      return this._data;
    }
    set data(t) {
      this.setDataWithSize(t, t.length, true);
    }
    get dataInt32() {
      return this._dataInt32 || (this._dataInt32 = new Int32Array(this.data.buffer)), this._dataInt32;
    }
    get static() {
      return !!(this.descriptor.usage & re.STATIC);
    }
    set static(t) {
      t ? this.descriptor.usage |= re.STATIC : this.descriptor.usage &= ~re.STATIC;
    }
    setDataWithSize(t, e, s) {
      if (this._updateID++, this._updateSize = e * t.BYTES_PER_ELEMENT, this._data === t) {
        s && this.emit("update", this);
        return;
      }
      const i = this._data;
      if (this._data = t, this._dataInt32 = null, !i || i.length !== t.length) {
        !this.shrinkToFit && i && t.byteLength < i.byteLength ? s && this.emit("update", this) : (this.descriptor.size = t.byteLength, this._resourceId = Kt("resource"), this.emit("change", this));
        return;
      }
      s && this.emit("update", this);
    }
    update(t) {
      this._updateSize = t ?? this._updateSize, this._updateID++, this.emit("update", this);
    }
    unload() {
      var _a2;
      this.emit("unload", this);
      for (const t in this._gpuData) (_a2 = this._gpuData[t]) == null ? void 0 : _a2.destroy();
      this._gpuData = /* @__PURE__ */ Object.create(null);
    }
    destroy() {
      this.destroyed = true, this.unload(), this.emit("destroy", this), this.emit("change", this), this._data = null, this.descriptor = null, this.removeAllListeners();
    }
  };
  function Ch(n, t) {
    if (!(n instanceof jn)) {
      let e = t ? re.INDEX : re.VERTEX;
      n instanceof Array && (t ? (n = new Uint32Array(n), e = re.INDEX | re.COPY_DST) : (n = new Float32Array(n), e = re.VERTEX | re.COPY_DST)), n = new jn({
        data: n,
        label: t ? "index-mesh-buffer" : "vertex-mesh-buffer",
        usage: e
      });
    }
    return n;
  }
  function Df(n, t, e) {
    const s = n.getAttribute(t);
    if (!s) return e.minX = 0, e.minY = 0, e.maxX = 0, e.maxY = 0, e;
    const i = s.buffer.data;
    let r = 1 / 0, o = 1 / 0, a = -1 / 0, l = -1 / 0;
    const c = i.BYTES_PER_ELEMENT, h = (s.offset || 0) / c, d = (s.stride || 8) / c;
    for (let u = h; u < i.length; u += d) {
      const p = i[u], f = i[u + 1];
      p > a && (a = p), f > l && (l = f), p < r && (r = p), f < o && (o = f);
    }
    return e.minX = r, e.minY = o, e.maxX = a, e.maxY = l, e;
  }
  function Hf(n) {
    return (n instanceof jn || Array.isArray(n) || n.BYTES_PER_ELEMENT) && (n = {
      buffer: n
    }), n.buffer = Ch(n.buffer, false), n;
  }
  Sh = class extends Qe {
    constructor(t = {}) {
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = Kt("geometry"), this._layoutKey = 0, this.instanceCount = 1, this._bounds = new Se(), this._boundsDirty = true;
      const { attributes: e, indexBuffer: s, topology: i } = t;
      if (this.buffers = [], this.attributes = {}, e) for (const r in e) this.addAttribute(r, e[r]);
      this.instanceCount = t.instanceCount ?? 1, s && this.addIndex(s), this.topology = i || "triangle-list";
    }
    onBufferUpdate() {
      this._boundsDirty = true, this.emit("update", this);
    }
    getAttribute(t) {
      return this.attributes[t];
    }
    getIndex() {
      return this.indexBuffer;
    }
    getBuffer(t) {
      return this.getAttribute(t).buffer;
    }
    getSize() {
      for (const t in this.attributes) {
        const e = this.attributes[t];
        return e.buffer.data.length / (e.stride / 4 || e.size);
      }
      return 0;
    }
    addAttribute(t, e) {
      const s = Hf(e);
      this.buffers.indexOf(s.buffer) === -1 && (this.buffers.push(s.buffer), s.buffer.on("update", this.onBufferUpdate, this), s.buffer.on("change", this.onBufferUpdate, this)), this.attributes[t] = s;
    }
    addIndex(t) {
      this.indexBuffer = Ch(t, true), this.buffers.push(this.indexBuffer);
    }
    get bounds() {
      return this._boundsDirty ? (this._boundsDirty = false, Df(this, "aPosition", this._bounds)) : this._bounds;
    }
    unload() {
      var _a2;
      this.emit("unload", this);
      for (const t in this._gpuData) (_a2 = this._gpuData[t]) == null ? void 0 : _a2.destroy();
      this._gpuData = /* @__PURE__ */ Object.create(null);
    }
    destroy(t = false) {
      var _a2;
      this.emit("destroy", this), this.removeAllListeners(), t && this.buffers.forEach((e) => e.destroy()), this.unload(), (_a2 = this.indexBuffer) == null ? void 0 : _a2.destroy(), this.attributes = null, this.buffers = null, this.indexBuffer = null, this._bounds = null;
    }
  };
  const Uf = new Float32Array(1), jf = new Uint32Array(1);
  class Vf extends Sh {
    constructor() {
      const e = new jn({
        data: Uf,
        label: "attribute-batch-buffer",
        usage: re.VERTEX | re.COPY_DST,
        shrinkToFit: false
      }), s = new jn({
        data: jf,
        label: "index-batch-buffer",
        usage: re.INDEX | re.COPY_DST,
        shrinkToFit: false
      }), i = 24;
      super({
        attributes: {
          aPosition: {
            buffer: e,
            format: "float32x2",
            stride: i,
            offset: 0
          },
          aUV: {
            buffer: e,
            format: "float32x2",
            stride: i,
            offset: 8
          },
          aColor: {
            buffer: e,
            format: "unorm8x4",
            stride: i,
            offset: 16
          },
          aTextureIdAndRound: {
            buffer: e,
            format: "uint16x2",
            stride: i,
            offset: 20
          }
        },
        indexBuffer: s
      });
    }
  }
  function Xa(n, t, e) {
    if (n) for (const s in n) {
      const i = s.toLocaleLowerCase(), r = t[i];
      if (r) {
        let o = n[s];
        s === "header" && (o = o.replace(/@in\s+[^;]+;\s*/g, "").replace(/@out\s+[^;]+;\s*/g, "")), e && r.push(`//----${e}----//`), r.push(o);
      } else Vt(`${s} placement hook does not exist in shader`);
    }
  }
  const Yf = /\{\{(.*?)\}\}/g;
  function qa(n) {
    var _a2;
    const t = {};
    return (((_a2 = n.match(Yf)) == null ? void 0 : _a2.map((s) => s.replace(/[{()}]/g, ""))) ?? []).forEach((s) => {
      t[s] = [];
    }), t;
  }
  function Ka(n, t) {
    let e;
    const s = /@in\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function Ja(n, t, e = false) {
    const s = [];
    Ka(t, s), n.forEach((a) => {
      a.header && Ka(a.header, s);
    });
    const i = s;
    e && i.sort();
    const r = i.map((a, l) => `       @location(${l}) ${a},`).join(`
`);
    let o = t.replace(/@in\s+[^;]+;\s*/g, "");
    return o = o.replace("{{in}}", `
${r}
`), o;
  }
  function Za(n, t) {
    let e;
    const s = /@out\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function Xf(n) {
    const e = /\b(\w+)\s*:/g.exec(n);
    return e ? e[1] : "";
  }
  function qf(n) {
    const t = /@.*?\s+/g;
    return n.replace(t, "");
  }
  function Kf(n, t) {
    const e = [];
    Za(t, e), n.forEach((l) => {
      l.header && Za(l.header, e);
    });
    let s = 0;
    const i = e.sort().map((l) => l.indexOf("builtin") > -1 ? l : `@location(${s++}) ${l}`).join(`,
`), r = e.sort().map((l) => `       var ${qf(l)};`).join(`
`), o = `return VSOutput(
            ${e.sort().map((l) => ` ${Xf(l)}`).join(`,
`)});`;
    let a = t.replace(/@out\s+[^;]+;\s*/g, "");
    return a = a.replace("{{struct}}", `
${i}
`), a = a.replace("{{start}}", `
${r}
`), a = a.replace("{{return}}", `
${o}
`), a;
  }
  function Qa(n, t) {
    let e = n;
    for (const s in t) {
      const i = t[s];
      i.join(`
`).length ? e = e.replace(`{{${s}}}`, `//-----${s} START-----//
${i.join(`
`)}
//----${s} FINISH----//`) : e = e.replace(`{{${s}}}`, "");
    }
    return e;
  }
  const wn = /* @__PURE__ */ Object.create(null), Er = /* @__PURE__ */ new Map();
  let Jf = 0;
  function Zf({ template: n, bits: t }) {
    const e = kh(n, t);
    if (wn[e]) return wn[e];
    const { vertex: s, fragment: i } = tm(n, t);
    return wn[e] = Th(s, i, t), wn[e];
  }
  function Qf({ template: n, bits: t }) {
    const e = kh(n, t);
    return wn[e] || (wn[e] = Th(n.vertex, n.fragment, t)), wn[e];
  }
  function tm(n, t) {
    const e = t.map((o) => o.vertex).filter((o) => !!o), s = t.map((o) => o.fragment).filter((o) => !!o);
    let i = Ja(e, n.vertex, true);
    i = Kf(e, i);
    const r = Ja(s, n.fragment, true);
    return {
      vertex: i,
      fragment: r
    };
  }
  function kh(n, t) {
    return t.map((e) => (Er.has(e) || Er.set(e, Jf++), Er.get(e))).sort((e, s) => e - s).join("-") + n.vertex + n.fragment;
  }
  function Th(n, t, e) {
    const s = qa(n), i = qa(t);
    return e.forEach((r) => {
      Xa(r.vertex, s, r.name), Xa(r.fragment, i, r.name);
    }), {
      vertex: Qa(n, s),
      fragment: Qa(t, i)
    };
  }
  const em = `
    @in aPosition: vec2<f32>;
    @in aUV: vec2<f32>;

    @out @builtin(position) vPosition: vec4<f32>;
    @out vUV : vec2<f32>;
    @out vColor : vec4<f32>;

    {{header}}

    struct VSOutput {
        {{struct}}
    };

    @vertex
    fn main( {{in}} ) -> VSOutput {

        var worldTransformMatrix = globalUniforms.uWorldTransformMatrix;
        var modelMatrix = mat3x3<f32>(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0
          );
        var position = aPosition;
        var uv = aUV;

        {{start}}

        vColor = vec4<f32>(1., 1., 1., 1.);

        {{main}}

        vUV = uv;

        var modelViewProjectionMatrix = globalUniforms.uProjectionMatrix * worldTransformMatrix * modelMatrix;

        vPosition =  vec4<f32>((modelViewProjectionMatrix *  vec3<f32>(position, 1.0)).xy, 0.0, 1.0);

        vColor *= globalUniforms.uWorldColorAlpha;

        {{end}}

        {{return}}
    };
`, nm = `
    @in vUV : vec2<f32>;
    @in vColor : vec4<f32>;

    {{header}}

    @fragment
    fn main(
        {{in}}
      ) -> @location(0) vec4<f32> {

        {{start}}

        var outColor:vec4<f32>;

        {{main}}

        var finalColor:vec4<f32> = outColor * vColor;

        {{end}}

        return finalColor;
      };
`, sm = `
    in vec2 aPosition;
    in vec2 aUV;

    out vec4 vColor;
    out vec2 vUV;

    {{header}}

    void main(void){

        mat3 worldTransformMatrix = uWorldTransformMatrix;
        mat3 modelMatrix = mat3(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0
          );
        vec2 position = aPosition;
        vec2 uv = aUV;

        {{start}}

        vColor = vec4(1.);

        {{main}}

        vUV = uv;

        mat3 modelViewProjectionMatrix = uProjectionMatrix * worldTransformMatrix * modelMatrix;

        gl_Position = vec4((modelViewProjectionMatrix * vec3(position, 1.0)).xy, 0.0, 1.0);

        vColor *= uWorldColorAlpha;

        {{end}}
    }
`, im = `

    in vec4 vColor;
    in vec2 vUV;

    out vec4 finalColor;

    {{header}}

    void main(void) {

        {{start}}

        vec4 outColor;

        {{main}}

        finalColor = outColor * vColor;

        {{end}}
    }
`, rm = {
    name: "global-uniforms-bit",
    vertex: {
      header: `
        struct GlobalUniforms {
            uProjectionMatrix:mat3x3<f32>,
            uWorldTransformMatrix:mat3x3<f32>,
            uWorldColorAlpha: vec4<f32>,
            uResolution: vec2<f32>,
        }

        @group(0) @binding(0) var<uniform> globalUniforms : GlobalUniforms;
        `
    }
  }, om = {
    name: "global-uniforms-bit",
    vertex: {
      header: `
          uniform mat3 uProjectionMatrix;
          uniform mat3 uWorldTransformMatrix;
          uniform vec4 uWorldColorAlpha;
          uniform vec2 uResolution;
        `
    }
  };
  am = function({ bits: n, name: t }) {
    const e = Zf({
      template: {
        fragment: nm,
        vertex: em
      },
      bits: [
        rm,
        ...n
      ]
    });
    return ti.from({
      name: t,
      vertex: {
        source: e.vertex,
        entryPoint: "main"
      },
      fragment: {
        source: e.fragment,
        entryPoint: "main"
      }
    });
  };
  lm = function({ bits: n, name: t }) {
    return new Oo({
      name: t,
      ...Qf({
        template: {
          vertex: sm,
          fragment: im
        },
        bits: [
          om,
          ...n
        ]
      })
    });
  };
  let Mr;
  cm = {
    name: "color-bit",
    vertex: {
      header: `
            @in aColor: vec4<f32>;
        `,
      main: `
            vColor *= vec4<f32>(aColor.rgb * aColor.a, aColor.a);
        `
    }
  };
  hm = {
    name: "color-bit",
    vertex: {
      header: `
            in vec4 aColor;
        `,
      main: `
            vColor *= vec4(aColor.rgb * aColor.a, aColor.a);
        `
    }
  };
  Mr = {};
  function dm(n) {
    const t = [];
    if (n === 1) t.push("@group(1) @binding(0) var textureSource1: texture_2d<f32>;"), t.push("@group(1) @binding(1) var textureSampler1: sampler;");
    else {
      let e = 0;
      for (let s = 0; s < n; s++) t.push(`@group(1) @binding(${e++}) var textureSource${s + 1}: texture_2d<f32>;`), t.push(`@group(1) @binding(${e++}) var textureSampler${s + 1}: sampler;`);
    }
    return t.join(`
`);
  }
  function um(n) {
    const t = [];
    if (n === 1) t.push("outColor = textureSampleGrad(textureSource1, textureSampler1, vUV, uvDx, uvDy);");
    else {
      t.push("switch vTextureId {");
      for (let e = 0; e < n; e++) e === n - 1 ? t.push("  default:{") : t.push(`  case ${e}:{`), t.push(`      outColor = textureSampleGrad(textureSource${e + 1}, textureSampler${e + 1}, vUV, uvDx, uvDy);`), t.push("      break;}");
      t.push("}");
    }
    return t.join(`
`);
  }
  pm = function(n) {
    return Mr[n] || (Mr[n] = {
      name: "texture-batch-bit",
      vertex: {
        header: `
                @in aTextureIdAndRound: vec2<u32>;
                @out @interpolate(flat) vTextureId : u32;
            `,
        main: `
                vTextureId = aTextureIdAndRound.y;
            `,
        end: `
                if(aTextureIdAndRound.x == 1)
                {
                    vPosition = vec4<f32>(roundPixels(vPosition.xy, globalUniforms.uResolution), vPosition.zw);
                }
            `
      },
      fragment: {
        header: `
                @in @interpolate(flat) vTextureId: u32;

                ${dm(n)}
            `,
        main: `
                var uvDx = dpdx(vUV);
                var uvDy = dpdy(vUV);

                ${um(n)}
            `
      }
    }), Mr[n];
  };
  const Pr = {};
  function fm(n) {
    const t = [];
    for (let e = 0; e < n; e++) e > 0 && t.push("else"), e < n - 1 && t.push(`if(vTextureId < ${e}.5)`), t.push("{"), t.push(`	outColor = texture(uTextures[${e}], vUV);`), t.push("}");
    return t.join(`
`);
  }
  mm = function(n) {
    return Pr[n] || (Pr[n] = {
      name: "texture-batch-bit",
      vertex: {
        header: `
                in vec2 aTextureIdAndRound;
                out float vTextureId;

            `,
        main: `
                vTextureId = aTextureIdAndRound.y;
            `,
        end: `
                if(aTextureIdAndRound.x == 1.)
                {
                    gl_Position.xy = roundPixels(gl_Position.xy, uResolution);
                }
            `
      },
      fragment: {
        header: `
                in float vTextureId;

                uniform sampler2D uTextures[${n}];

            `,
        main: `

                ${fm(n)}
            `
      }
    }), Pr[n];
  };
  let tl;
  gm = {
    name: "round-pixels-bit",
    vertex: {
      header: `
            fn roundPixels(position: vec2<f32>, targetSize: vec2<f32>) -> vec2<f32>
            {
                return (floor(((position * 0.5 + 0.5) * targetSize) + 0.5) / targetSize) * 2.0 - 1.0;
            }
        `
    }
  };
  ym = {
    name: "round-pixels-bit",
    vertex: {
      header: `
            vec2 roundPixels(vec2 position, vec2 targetSize)
            {
                return (floor(((position * 0.5 + 0.5) * targetSize) + 0.5) / targetSize) * 2.0 - 1.0;
            }
        `
    }
  };
  tl = {};
  xm = function(n) {
    let t = tl[n];
    if (t) return t;
    const e = new Int32Array(n);
    for (let s = 0; s < n; s++) e[s] = s;
    return t = tl[n] = new Fo({
      uTextures: {
        value: e,
        type: "i32",
        size: n
      }
    }, {
      isStatic: true
    }), t;
  };
  class el extends Xi {
    constructor(t) {
      const e = lm({
        name: "batch",
        bits: [
          hm,
          mm(t),
          ym
        ]
      }), s = am({
        name: "batch",
        bits: [
          cm,
          pm(t),
          gm
        ]
      });
      super({
        glProgram: e,
        gpuProgram: s,
        resources: {
          batchSamplers: xm(t)
        }
      }), this.maxTextures = t;
    }
  }
  let As = null;
  const Ah = class Eh extends zf {
    constructor(t) {
      super(t), this.geometry = new Vf(), this.name = Eh.extension.name, this.vertexSize = 6, As ?? (As = new el(t.maxTextures)), this.shader = As;
    }
    packAttributes(t, e, s, i, r) {
      const o = r << 16 | t.roundPixels & 65535, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, { positions: f, uvs: g } = t, m = t.color, y = t.attributeOffset, b = y + t.attributeSize;
      for (let x = y; x < b; x++) {
        const _ = x * 2, v = f[_], w = f[_ + 1];
        e[i++] = l * v + h * w + u, e[i++] = d * w + c * v + p, e[i++] = g[_], e[i++] = g[_ + 1], s[i++] = m, s[i++] = o;
      }
    }
    packQuadAttributes(t, e, s, i, r) {
      const o = t.texture, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, f = t.bounds, g = f.maxX, m = f.minX, y = f.maxY, b = f.minY, x = o.uvs, _ = t.color, v = r << 16 | t.roundPixels & 65535;
      e[i + 0] = l * m + h * b + u, e[i + 1] = d * b + c * m + p, e[i + 2] = x.x0, e[i + 3] = x.y0, s[i + 4] = _, s[i + 5] = v, e[i + 6] = l * g + h * b + u, e[i + 7] = d * b + c * g + p, e[i + 8] = x.x1, e[i + 9] = x.y1, s[i + 10] = _, s[i + 11] = v, e[i + 12] = l * g + h * y + u, e[i + 13] = d * y + c * g + p, e[i + 14] = x.x2, e[i + 15] = x.y2, s[i + 16] = _, s[i + 17] = v, e[i + 18] = l * m + h * y + u, e[i + 19] = d * y + c * m + p, e[i + 20] = x.x3, e[i + 21] = x.y3, s[i + 22] = _, s[i + 23] = v;
    }
    _updateMaxTextures(t) {
      this.shader.maxTextures !== t && (As = new el(t), this.shader = As);
    }
    destroy() {
      this.shader = null, super.destroy();
    }
  };
  Ah.extension = {
    type: [
      it.Batcher
    ],
    name: "default"
  };
  bm = Ah;
  _s = class {
    constructor(t) {
      this.items = /* @__PURE__ */ Object.create(null);
      const { renderer: e, type: s, onUnload: i, priority: r, name: o } = t;
      this._renderer = e, e.gc.addResourceHash(this, "items", s, r ?? 0), this._onUnload = i, this.name = o;
    }
    add(t) {
      return this.items[t.uid] ? false : (this.items[t.uid] = t, t.once("unload", this.remove, this), t._gcLastUsed = this._renderer.gc.now, true);
    }
    remove(t, ...e) {
      var _a2;
      if (!this.items[t.uid]) return;
      const s = t._gpuData[this._renderer.uid];
      s && ((_a2 = this._onUnload) == null ? void 0 : _a2.call(this, t, ...e), s.destroy(), t._gpuData[this._renderer.uid] = null, this.items[t.uid] = null);
    }
    removeAll(...t) {
      Object.values(this.items).forEach((e) => e && this.remove(e, ...t));
    }
    destroy(...t) {
      this.removeAll(...t), this.items = /* @__PURE__ */ Object.create(null), this._renderer = null, this._onUnload = null;
    }
  };
  function _m(n, t, e, s, i, r, o, a = null) {
    let l = 0;
    e *= t, i *= r;
    const c = a.a, h = a.b, d = a.c, u = a.d, p = a.tx, f = a.ty;
    for (; l < o; ) {
      const g = n[e], m = n[e + 1];
      s[i] = c * g + d * m + p, s[i + 1] = h * g + u * m + f, i += r, e += t, l++;
    }
  }
  function wm(n, t, e, s) {
    let i = 0;
    for (t *= e; i < s; ) n[t] = 0, n[t + 1] = 0, t += e, i++;
  }
  function Mh(n, t, e, s, i) {
    const r = t.a, o = t.b, a = t.c, l = t.d, c = t.tx, h = t.ty;
    e || (e = 0), s || (s = 2), i || (i = n.length / s - e);
    let d = e * s;
    for (let u = 0; u < i; u++) {
      const p = n[d], f = n[d + 1];
      n[d] = r * p + a * f + c, n[d + 1] = o * p + l * f + h, d += s;
    }
  }
  const vm = new _t();
  class Do {
    constructor() {
      this.packAsQuad = false, this.batcherName = "default", this.topology = "triangle-list", this.applyTransform = true, this.roundPixels = 0, this._batcher = null, this._batch = null;
    }
    get uvs() {
      return this.geometryData.uvs;
    }
    get positions() {
      return this.geometryData.vertices;
    }
    get indices() {
      return this.geometryData.indices;
    }
    get blendMode() {
      return this.renderable && this.applyTransform ? this.renderable.groupBlendMode : "normal";
    }
    get color() {
      const t = this.baseColor, e = t >> 16 | t & 65280 | (t & 255) << 16, s = this.renderable;
      return s ? Bc(e, s.groupColor) + (this.alpha * s.groupAlpha * 255 << 24) : e + (this.alpha * 255 << 24);
    }
    get transform() {
      var _a2;
      return ((_a2 = this.renderable) == null ? void 0 : _a2.groupTransform) || vm;
    }
    copyTo(t) {
      t.indexOffset = this.indexOffset, t.indexSize = this.indexSize, t.attributeOffset = this.attributeOffset, t.attributeSize = this.attributeSize, t.baseColor = this.baseColor, t.alpha = this.alpha, t.texture = this.texture, t.geometryData = this.geometryData, t.topology = this.topology;
    }
    reset() {
      this.applyTransform = true, this.renderable = null, this.topology = "triangle-list";
    }
    destroy() {
      this.renderable = null, this.texture = null, this.geometryData = null, this._batcher = null, this._batch = null;
    }
  }
  const Ys = {
    extension: {
      type: it.ShapeBuilder,
      name: "circle"
    },
    build(n, t) {
      let e, s, i, r, o, a;
      if (n.type === "circle") {
        const _ = n;
        if (o = a = _.radius, o <= 0) return false;
        e = _.x, s = _.y, i = r = 0;
      } else if (n.type === "ellipse") {
        const _ = n;
        if (o = _.halfWidth, a = _.halfHeight, o <= 0 || a <= 0) return false;
        e = _.x, s = _.y, i = r = 0;
      } else {
        const _ = n, v = _.width / 2, w = _.height / 2;
        e = _.x + v, s = _.y + w, o = a = Math.max(0, Math.min(_.radius, Math.min(v, w))), i = v - o, r = w - a;
      }
      if (i < 0 || r < 0) return false;
      const l = Math.ceil(2.3 * Math.sqrt(o + a)), c = l * 8 + (i ? 4 : 0) + (r ? 4 : 0);
      if (c === 0) return false;
      if (l === 0) return t[0] = t[6] = e + i, t[1] = t[3] = s + r, t[2] = t[4] = e - i, t[5] = t[7] = s - r, true;
      let h = 0, d = l * 4 + (i ? 2 : 0) + 2, u = d, p = c, f = i + o, g = r, m = e + f, y = e - f, b = s + g;
      if (t[h++] = m, t[h++] = b, t[--d] = b, t[--d] = y, r) {
        const _ = s - g;
        t[u++] = y, t[u++] = _, t[--p] = _, t[--p] = m;
      }
      for (let _ = 1; _ < l; _++) {
        const v = Math.PI / 2 * (_ / l), w = i + Math.cos(v) * o, C = r + Math.sin(v) * a, E = e + w, P = e - w, T = s + C, A = s - C;
        t[h++] = E, t[h++] = T, t[--d] = T, t[--d] = P, t[u++] = P, t[u++] = A, t[--p] = A, t[--p] = E;
      }
      f = i, g = r + a, m = e + f, y = e - f, b = s + g;
      const x = s - g;
      return t[h++] = m, t[h++] = b, t[--p] = x, t[--p] = m, i && (t[h++] = y, t[h++] = b, t[--p] = x, t[--p] = y), true;
    },
    triangulate(n, t, e, s, i, r) {
      if (n.length === 0) return;
      let o = 0, a = 0;
      for (let h = 0; h < n.length; h += 2) o += n[h], a += n[h + 1];
      o /= n.length / 2, a /= n.length / 2;
      let l = s;
      t[l * e] = o, t[l * e + 1] = a;
      const c = l++;
      for (let h = 0; h < n.length; h += 2) t[l * e] = n[h], t[l * e + 1] = n[h + 1], h > 0 && (i[r++] = l, i[r++] = c, i[r++] = l - 1), l++;
      i[r++] = c + 1, i[r++] = c, i[r++] = l - 1;
    }
  }, Cm = {
    ...Ys,
    extension: {
      ...Ys.extension,
      name: "ellipse"
    }
  }, Sm = {
    ...Ys,
    extension: {
      ...Ys.extension,
      name: "roundedRectangle"
    }
  }, Ph = 1e-4, nl = 1e-4;
  function km(n) {
    const t = n.length;
    if (t < 6) return 1;
    let e = 0;
    for (let s = 0, i = n[t - 2], r = n[t - 1]; s < t; s += 2) {
      const o = n[s], a = n[s + 1];
      e += (o - i) * (a + r), i = o, r = a;
    }
    return e < 0 ? -1 : 1;
  }
  function sl(n, t, e, s, i, r, o, a) {
    const l = n - e * i, c = t - s * i, h = n + e * r, d = t + s * r;
    let u, p;
    o ? (u = s, p = -e) : (u = -s, p = e);
    const f = l + u, g = c + p, m = h + u, y = d + p;
    return a.push(f, g), a.push(m, y), 2;
  }
  function An(n, t, e, s, i, r, o, a) {
    const l = e - n, c = s - t;
    let h = Math.atan2(l, c), d = Math.atan2(i - n, r - t);
    a && h < d ? h += Math.PI * 2 : !a && h > d && (d += Math.PI * 2);
    let u = h;
    const p = d - h, f = Math.abs(p), g = Math.sqrt(l * l + c * c), m = (15 * f * Math.sqrt(g) / Math.PI >> 0) + 1, y = p / m;
    if (u += y, a) {
      o.push(n, t), o.push(e, s);
      for (let b = 1, x = u; b < m; b++, x += y) o.push(n, t), o.push(n + Math.sin(x) * g, t + Math.cos(x) * g);
      o.push(n, t), o.push(i, r);
    } else {
      o.push(e, s), o.push(n, t);
      for (let b = 1, x = u; b < m; b++, x += y) o.push(n + Math.sin(x) * g, t + Math.cos(x) * g), o.push(n, t);
      o.push(i, r), o.push(n, t);
    }
    return m * 2;
  }
  Tm = function(n, t, e, s, i, r) {
    const o = Ph;
    if (n.length === 0) return;
    const a = t;
    let l = a.alignment;
    if (t.alignment !== 0.5) {
      let U = km(n);
      l = (l - 0.5) * U + 0.5;
    }
    const c = new At(n[0], n[1]), h = new At(n[n.length - 2], n[n.length - 1]), d = s, u = Math.abs(c.x - h.x) < o && Math.abs(c.y - h.y) < o;
    if (d) {
      n = n.slice(), u && (n.pop(), n.pop(), h.set(n[n.length - 2], n[n.length - 1]));
      const U = (c.x + h.x) * 0.5, Z = (h.y + c.y) * 0.5;
      n.unshift(U, Z), n.push(U, Z);
    }
    const p = i, f = n.length / 2;
    let g = n.length;
    const m = p.length / 2, y = a.width / 2, b = y * y, x = a.miterLimit * a.miterLimit;
    let _ = n[0], v = n[1], w = n[2], C = n[3], E = 0, P = 0, T = -(v - C), A = _ - w, H = 0, j = 0, D = Math.sqrt(T * T + A * A);
    T /= D, A /= D, T *= y, A *= y;
    const N = l, $ = (1 - N) * 2, W = N * 2;
    d || (a.cap === "round" ? g += An(_ - T * ($ - W) * 0.5, v - A * ($ - W) * 0.5, _ - T * $, v - A * $, _ + T * W, v + A * W, p, true) + 2 : a.cap === "square" && (g += sl(_, v, T, A, $, W, true, p))), p.push(_ - T * $, v - A * $), p.push(_ + T * W, v + A * W);
    for (let U = 1; U < f - 1; ++U) {
      _ = n[(U - 1) * 2], v = n[(U - 1) * 2 + 1], w = n[U * 2], C = n[U * 2 + 1], E = n[(U + 1) * 2], P = n[(U + 1) * 2 + 1], T = -(v - C), A = _ - w, D = Math.sqrt(T * T + A * A), T /= D, A /= D, T *= y, A *= y, H = -(C - P), j = w - E, D = Math.sqrt(H * H + j * j), H /= D, j /= D, H *= y, j *= y;
      const Z = w - _, R = v - C, B = w - E, O = P - C, z = Z * B + R * O, Y = R * B - O * Z, K = Y < 0;
      if (Math.abs(Y) < 1e-3 * Math.abs(z)) {
        p.push(w - T * $, C - A * $), p.push(w + T * W, C + A * W), z >= 0 && (a.join === "round" ? g += An(w, C, w - T * $, C - A * $, w - H * $, C - j * $, p, false) + 4 : g += 2, p.push(w - H * W, C - j * W), p.push(w + H * $, C + j * $));
        continue;
      }
      const nt = (-T + _) * (-A + C) - (-T + w) * (-A + v), ot = (-H + E) * (-j + C) - (-H + w) * (-j + P), ht = (Z * ot - B * nt) / Y, gt = (O * nt - R * ot) / Y, I = (ht - w) * (ht - w) + (gt - C) * (gt - C), G = w + (ht - w) * $, V = C + (gt - C) * $, J = w - (ht - w) * W, xt = C - (gt - C) * W, bt = Math.min(Z * Z + R * R, B * B + O * O), wt = K ? $ : W, Dt = bt + wt * wt * b;
      I <= Dt ? a.join === "bevel" || I / b > x ? (K ? (p.push(G, V), p.push(w + T * W, C + A * W), p.push(G, V), p.push(w + H * W, C + j * W)) : (p.push(w - T * $, C - A * $), p.push(J, xt), p.push(w - H * $, C - j * $), p.push(J, xt)), g += 2) : a.join === "round" ? K ? (p.push(G, V), p.push(w + T * W, C + A * W), g += An(w, C, w + T * W, C + A * W, w + H * W, C + j * W, p, true) + 4, p.push(G, V), p.push(w + H * W, C + j * W)) : (p.push(w - T * $, C - A * $), p.push(J, xt), g += An(w, C, w - T * $, C - A * $, w - H * $, C - j * $, p, false) + 4, p.push(w - H * $, C - j * $), p.push(J, xt)) : (p.push(G, V), p.push(J, xt)) : (p.push(w - T * $, C - A * $), p.push(w + T * W, C + A * W), a.join === "round" ? K ? g += An(w, C, w + T * W, C + A * W, w + H * W, C + j * W, p, true) + 2 : g += An(w, C, w - T * $, C - A * $, w - H * $, C - j * $, p, false) + 2 : a.join === "miter" && I / b <= x && (K ? (p.push(J, xt), p.push(J, xt)) : (p.push(G, V), p.push(G, V)), g += 2), p.push(w - H * $, C - j * $), p.push(w + H * W, C + j * W), g += 2);
    }
    _ = n[(f - 2) * 2], v = n[(f - 2) * 2 + 1], w = n[(f - 1) * 2], C = n[(f - 1) * 2 + 1], T = -(v - C), A = _ - w, D = Math.sqrt(T * T + A * A), T /= D, A /= D, T *= y, A *= y, p.push(w - T * $, C - A * $), p.push(w + T * W, C + A * W), d || (a.cap === "round" ? g += An(w - T * ($ - W) * 0.5, C - A * ($ - W) * 0.5, w - T * $, C - A * $, w + T * W, C + A * W, p, false) + 2 : a.cap === "square" && (g += sl(w, C, T, A, $, W, false, p)));
    const X = nl * nl;
    for (let U = m; U < g + m - 2; ++U) _ = p[U * 2], v = p[U * 2 + 1], w = p[(U + 1) * 2], C = p[(U + 1) * 2 + 1], E = p[(U + 2) * 2], P = p[(U + 2) * 2 + 1], !(Math.abs(_ * (C - P) + w * (P - v) + E * (v - C)) < X) && r.push(U, U + 1, U + 2);
  };
  function Am(n, t, e, s) {
    const i = Ph;
    if (n.length === 0) return;
    const r = n[0], o = n[1], a = n[n.length - 2], l = n[n.length - 1], c = t || Math.abs(r - a) < i && Math.abs(o - l) < i, h = e, d = n.length / 2, u = h.length / 2;
    for (let p = 0; p < d; p++) h.push(n[p * 2]), h.push(n[p * 2 + 1]);
    for (let p = 0; p < d - 1; p++) s.push(u + p, u + p + 1);
    c && s.push(u + d - 1, u);
  }
  function Ih(n, t, e, s, i, r, o) {
    const a = Hp(n, t, 2);
    if (!a) return;
    for (let c = 0; c < a.length; c += 3) r[o++] = a[c] + i, r[o++] = a[c + 1] + i, r[o++] = a[c + 2] + i;
    let l = i * s;
    for (let c = 0; c < n.length; c += 2) e[l] = n[c], e[l + 1] = n[c + 1], l += s;
  }
  const Em = [], Mm = {
    extension: {
      type: it.ShapeBuilder,
      name: "polygon"
    },
    build(n, t) {
      for (let e = 0; e < n.points.length; e++) t[e] = n.points[e];
      return true;
    },
    triangulate(n, t, e, s, i, r) {
      Ih(n, Em, t, e, s, i, r);
    }
  }, Pm = {
    extension: {
      type: it.ShapeBuilder,
      name: "rectangle"
    },
    build(n, t) {
      const e = n, s = e.x, i = e.y, r = e.width, o = e.height;
      return r > 0 && o > 0 ? (t[0] = s, t[1] = i, t[2] = s + r, t[3] = i, t[4] = s + r, t[5] = i + o, t[6] = s, t[7] = i + o, true) : false;
    },
    triangulate(n, t, e, s, i, r) {
      let o = 0;
      s *= e, t[s + o] = n[0], t[s + o + 1] = n[1], o += e, t[s + o] = n[2], t[s + o + 1] = n[3], o += e, t[s + o] = n[6], t[s + o + 1] = n[7], o += e, t[s + o] = n[4], t[s + o + 1] = n[5], o += e;
      const a = s / e;
      i[r++] = a, i[r++] = a + 1, i[r++] = a + 2, i[r++] = a + 1, i[r++] = a + 3, i[r++] = a + 2;
    }
  }, Im = {
    extension: {
      type: it.ShapeBuilder,
      name: "triangle"
    },
    build(n, t) {
      return t[0] = n.x, t[1] = n.y, t[2] = n.x2, t[3] = n.y2, t[4] = n.x3, t[5] = n.y3, true;
    },
    triangulate(n, t, e, s, i, r) {
      let o = 0;
      s *= e, t[s + o] = n[0], t[s + o + 1] = n[1], o += e, t[s + o] = n[2], t[s + o + 1] = n[3], o += e, t[s + o] = n[4], t[s + o + 1] = n[5];
      const a = s / e;
      i[r++] = a, i[r++] = a + 1, i[r++] = a + 2;
    }
  }, il = [
    {
      offset: 0,
      color: "white"
    },
    {
      offset: 1,
      color: "black"
    }
  ], Ho = class ho {
    constructor(...t) {
      this.uid = Kt("fillGradient"), this._tick = 0, this.type = "linear", this.colorStops = [];
      let e = Rm(t);
      e = {
        ...e.type === "radial" ? ho.defaultRadialOptions : ho.defaultLinearOptions,
        ...Sc(e)
      }, this._textureSize = e.textureSize, this._wrapMode = e.wrapMode, e.type === "radial" ? (this.center = e.center, this.outerCenter = e.outerCenter ?? this.center, this.innerRadius = e.innerRadius, this.outerRadius = e.outerRadius, this.scale = e.scale, this.rotation = e.rotation) : (this.start = e.start, this.end = e.end), this.textureSpace = e.textureSpace, this.type = e.type, e.colorStops.forEach((i) => {
        this.addColorStop(i.offset, i.color);
      });
    }
    addColorStop(t, e) {
      return this.colorStops.push({
        offset: t,
        color: Gt.shared.setValue(e).toHexa()
      }), this;
    }
    buildLinearGradient() {
      if (this.texture) return;
      let { x: t, y: e } = this.start, { x: s, y: i } = this.end, r = s - t, o = i - e;
      const a = r < 0 || o < 0;
      if (this._wrapMode === "clamp-to-edge") {
        if (r < 0) {
          const m = t;
          t = s, s = m, r *= -1;
        }
        if (o < 0) {
          const m = e;
          e = i, i = m, o *= -1;
        }
      }
      const l = this.colorStops.length ? this.colorStops : il, c = this._textureSize, { canvas: h, context: d } = ol(c, 1), u = a ? d.createLinearGradient(this._textureSize, 0, 0, 0) : d.createLinearGradient(0, 0, this._textureSize, 0);
      rl(u, l), d.fillStyle = u, d.fillRect(0, 0, c, 1), this.texture = new Ct({
        source: new fs({
          resource: h,
          addressMode: this._wrapMode
        })
      });
      const p = Math.sqrt(r * r + o * o), f = Math.atan2(o, r), g = new _t();
      g.scale(p / c, 1), g.rotate(f), g.translate(t, e), this.textureSpace === "local" && g.scale(c, c), this.transform = g;
    }
    buildGradient() {
      this.texture || this._tick++, this.type === "linear" ? this.buildLinearGradient() : this.buildRadialGradient();
    }
    buildRadialGradient() {
      if (this.texture) return;
      const t = this.colorStops.length ? this.colorStops : il, e = this._textureSize, { canvas: s, context: i } = ol(e, e), { x: r, y: o } = this.center, { x: a, y: l } = this.outerCenter, c = this.innerRadius, h = this.outerRadius, d = a - h, u = l - h, p = e / (h * 2), f = (r - d) * p, g = (o - u) * p, m = i.createRadialGradient(f, g, c * p, (a - d) * p, (l - u) * p, h * p);
      rl(m, t), i.fillStyle = t[t.length - 1].color, i.fillRect(0, 0, e, e), i.fillStyle = m, i.translate(f, g), i.rotate(this.rotation), i.scale(1, this.scale), i.translate(-f, -g), i.fillRect(0, 0, e, e), this.texture = new Ct({
        source: new fs({
          resource: s,
          addressMode: this._wrapMode
        })
      });
      const y = new _t();
      y.scale(1 / p, 1 / p), y.translate(d, u), this.textureSpace === "local" && y.scale(e, e), this.transform = y;
    }
    destroy() {
      var _a2;
      (_a2 = this.texture) == null ? void 0 : _a2.destroy(true), this.texture = null, this.transform = null, this.colorStops = [], this.start = null, this.end = null, this.center = null, this.outerCenter = null;
    }
    get styleKey() {
      return `fill-gradient-${this.uid}-${this._tick}`;
    }
  };
  Ho.defaultLinearOptions = {
    start: {
      x: 0,
      y: 0
    },
    end: {
      x: 0,
      y: 1
    },
    colorStops: [],
    textureSpace: "local",
    type: "linear",
    textureSize: 256,
    wrapMode: "clamp-to-edge"
  };
  Ho.defaultRadialOptions = {
    center: {
      x: 0.5,
      y: 0.5
    },
    innerRadius: 0,
    outerRadius: 0.5,
    colorStops: [],
    scale: 1,
    textureSpace: "local",
    type: "radial",
    textureSize: 256,
    wrapMode: "clamp-to-edge"
  };
  fn = Ho;
  function rl(n, t) {
    for (let e = 0; e < t.length; e++) {
      const s = t[e];
      n.addColorStop(s.offset, s.color);
    }
  }
  function ol(n, t) {
    const e = Mt.get().createCanvas(n, t), s = e.getContext("2d");
    return {
      canvas: e,
      context: s
    };
  }
  function Rm(n) {
    let t = n[0] ?? {};
    return (typeof t == "number" || n[1]) && (Et("8.5.2", "use options object instead"), t = {
      type: "linear",
      start: {
        x: n[0],
        y: n[1]
      },
      end: {
        x: n[2],
        y: n[3]
      },
      textureSpace: n[4],
      textureSize: n[5] ?? fn.defaultLinearOptions.textureSize
    }), t;
  }
  const Lm = new _t(), $m = new Nt();
  Bm = function(n, t, e, s) {
    const i = t.matrix ? n.copyFrom(t.matrix).invert() : n.identity();
    if (t.textureSpace === "local") {
      const o = e.getBounds($m);
      t.width && o.pad(t.width);
      const { x: a, y: l } = o, c = 1 / o.width, h = 1 / o.height, d = -a * c, u = -l * h, p = i.a, f = i.b, g = i.c, m = i.d;
      i.a *= c, i.b *= c, i.c *= h, i.d *= h, i.tx = d * p + u * g + i.tx, i.ty = d * f + u * m + i.ty;
    } else i.translate(t.texture.frame.x, t.texture.frame.y), i.scale(1 / t.texture.source.width, 1 / t.texture.source.height);
    const r = t.texture.source.style;
    return !(t.fill instanceof fn) && r.addressMode === "clamp-to-edge" && (r.addressMode = "repeat", r.update()), s && i.append(Lm.copyFrom(s).invert()), i;
  };
  Ki = {};
  Ot.handleByMap(it.ShapeBuilder, Ki);
  Ot.add(Pm, Mm, Im, Ys, Cm, Sm);
  const Om = new Nt(), Fm = new _t();
  function Nm(n, t) {
    const { geometryData: e, batches: s } = t;
    s.length = 0, e.indices.length = 0, e.vertices.length = 0, e.uvs.length = 0;
    for (let i = 0; i < n.instructions.length; i++) {
      const r = n.instructions[i];
      if (r.action === "texture") Wm(r.data, s, e);
      else if (r.action === "fill" || r.action === "stroke") {
        const o = r.action === "stroke", a = r.data.path.shapePath, l = r.data.style, c = r.data.hole;
        o && c && al(c.shapePath, l, true, s, e), c && (a.shapePrimitives[a.shapePrimitives.length - 1].holes = c.shapePath.shapePrimitives), al(a, l, o, s, e);
      }
    }
  }
  function Wm(n, t, e) {
    const s = [], i = Ki.rectangle, r = Om;
    r.x = n.dx, r.y = n.dy, r.width = n.dw, r.height = n.dh;
    const o = n.transform;
    if (!i.build(r, s)) return;
    const { vertices: a, uvs: l, indices: c } = e, h = c.length, d = a.length / 2;
    o && Mh(s, o), i.triangulate(s, a, 2, d, c, h);
    const u = n.image, p = u.uvs;
    l.push(p.x0, p.y0, p.x1, p.y1, p.x3, p.y3, p.x2, p.y2);
    const f = _e.get(Do);
    f.indexOffset = h, f.indexSize = c.length - h, f.attributeOffset = d, f.attributeSize = a.length / 2 - d, f.baseColor = n.style, f.alpha = n.alpha, f.texture = u, f.geometryData = e, t.push(f);
  }
  function al(n, t, e, s, i) {
    const { vertices: r, uvs: o, indices: a } = i;
    n.shapePrimitives.forEach(({ shape: l, transform: c, holes: h }) => {
      const d = [], u = Ki[l.type];
      if (!u.build(l, d)) return;
      const p = a.length, f = r.length / 2;
      let g = "triangle-list";
      if (c && Mh(d, c), e) {
        const x = l.closePath ?? true, _ = t;
        _.pixelLine ? (Am(d, x, r, a), g = "line-list") : Tm(d, _, false, x, r, a);
      } else if (h) {
        const x = [], _ = d.slice();
        Gm(h).forEach((w) => {
          x.push(_.length / 2), _.push(...w);
        }), Ih(_, x, r, 2, f, a, p);
      } else u.triangulate(d, r, 2, f, a, p);
      const m = o.length / 2, y = t.texture;
      if (y !== Ct.WHITE) {
        const x = Bm(Fm, t, l, c);
        _m(r, 2, f, o, m, 2, r.length / 2 - f, x);
      } else wm(o, m, 2, r.length / 2 - f);
      const b = _e.get(Do);
      b.indexOffset = p, b.indexSize = a.length - p, b.attributeOffset = f, b.attributeSize = r.length / 2 - f, b.baseColor = t.color, b.alpha = t.alpha, b.texture = y, b.geometryData = i, b.topology = g, s.push(b);
    });
  }
  function Gm(n) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e].shape, i = [];
      Ki[s.type].build(s, i) && t.push(i);
    }
    return t;
  }
  class zm {
    constructor() {
      this.batches = [], this.geometryData = {
        vertices: [],
        uvs: [],
        indices: []
      };
    }
    reset() {
      this.batches && this.batches.forEach((t) => {
        _e.return(t);
      }), this.graphicsData && _e.return(this.graphicsData), this.isBatchable = false, this.context = null, this.batches.length = 0, this.geometryData.indices.length = 0, this.geometryData.vertices.length = 0, this.geometryData.uvs.length = 0, this.graphicsData = null;
    }
    destroy() {
      this.reset(), this.batches = null, this.geometryData = null;
    }
  }
  class Dm {
    constructor() {
      this.instructions = new $o();
    }
    init(t) {
      const e = t.maxTextures;
      this.batcher ? this.batcher._updateMaxTextures(e) : this.batcher = new bm({
        maxTextures: e
      }), this.instructions.reset();
    }
    get geometry() {
      return Et(mu, "GraphicsContextRenderData#geometry is deprecated, please use batcher.geometry instead."), this.batcher.geometry;
    }
    destroy() {
      this.batcher.destroy(), this.instructions.destroy(), this.batcher = null, this.instructions = null;
    }
  }
  const Uo = class uo {
    constructor(t) {
      this._renderer = t, this._managedContexts = new _s({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      uo.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? uo.defaultOptions.bezierSmoothness;
    }
    getContextRenderData(t) {
      return t._gpuData[this._renderer.uid].graphicsData || this._initContextRenderData(t);
    }
    updateGpuContext(t) {
      const e = !!t._gpuData[this._renderer.uid], s = t._gpuData[this._renderer.uid] || this._initContext(t);
      if (t.dirty || !e) {
        e && s.reset(), Nm(t, s);
        const i = t.batchMode;
        t.customShader || i === "no-batch" ? s.isBatchable = false : i === "auto" ? s.isBatchable = s.geometryData.vertices.length < 400 : s.isBatchable = true, t.dirty = false;
      }
      return s;
    }
    getGpuContext(t) {
      return t._gpuData[this._renderer.uid] || this._initContext(t);
    }
    _initContextRenderData(t) {
      const e = _e.get(Dm, {
        maxTextures: this._renderer.limits.maxBatchableTextures
      }), s = t._gpuData[this._renderer.uid], { batches: i, geometryData: r } = s;
      s.graphicsData = e;
      const o = r.vertices.length, a = r.indices.length;
      for (let d = 0; d < i.length; d++) i[d].applyTransform = false;
      const l = e.batcher;
      l.ensureAttributeBuffer(o), l.ensureIndexBuffer(a), l.begin();
      for (let d = 0; d < i.length; d++) {
        const u = i[d];
        l.add(u);
      }
      l.finish(e.instructions);
      const c = l.geometry;
      c.indexBuffer.setDataWithSize(l.indexBuffer, l.indexSize, true), c.buffers[0].setDataWithSize(l.attributeBuffer.float32View, l.attributeSize, true);
      const h = l.batches;
      for (let d = 0; d < h.length; d++) {
        const u = h[d];
        u.bindGroup = If(u.textures.textures, u.textures.count, this._renderer.limits.maxBatchableTextures);
      }
      return e;
    }
    _initContext(t) {
      const e = new zm();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  Uo.extension = {
    type: [
      it.WebGLSystem,
      it.WebGPUSystem
    ],
    name: "graphicsContext"
  };
  Uo.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let jo = Uo;
  const Hm = 8, gi = 11920929e-14, Um = 1;
  function Rh(n, t, e, s, i, r, o, a, l, c) {
    const d = Math.min(0.99, Math.max(0, c ?? jo.defaultOptions.bezierSmoothness));
    let u = (Um - d) / 1;
    return u *= u, jm(t, e, s, i, r, o, a, l, n, u), n;
  }
  function jm(n, t, e, s, i, r, o, a, l, c) {
    po(n, t, e, s, i, r, o, a, l, c, 0), l.push(o, a);
  }
  function po(n, t, e, s, i, r, o, a, l, c, h) {
    if (h > Hm) return;
    const d = (n + e) / 2, u = (t + s) / 2, p = (e + i) / 2, f = (s + r) / 2, g = (i + o) / 2, m = (r + a) / 2, y = (d + p) / 2, b = (u + f) / 2, x = (p + g) / 2, _ = (f + m) / 2, v = (y + x) / 2, w = (b + _) / 2;
    if (h > 0) {
      let C = o - n, E = a - t;
      const P = Math.abs((e - o) * E - (s - a) * C), T = Math.abs((i - o) * E - (r - a) * C);
      if (P > gi && T > gi) {
        if ((P + T) * (P + T) <= c * (C * C + E * E)) {
          l.push(v, w);
          return;
        }
      } else if (P > gi) {
        if (P * P <= c * (C * C + E * E)) {
          l.push(v, w);
          return;
        }
      } else if (T > gi) {
        if (T * T <= c * (C * C + E * E)) {
          l.push(v, w);
          return;
        }
      } else if (C = v - (n + o) / 2, E = w - (t + a) / 2, C * C + E * E <= c) {
        l.push(v, w);
        return;
      }
    }
    po(n, t, d, u, y, b, v, w, l, c, h + 1), po(v, w, x, _, g, m, o, a, l, c, h + 1);
  }
  const Vm = 8, Ym = 11920929e-14, Xm = 1;
  function qm(n, t, e, s, i, r, o, a) {
    const c = Math.min(0.99, Math.max(0, a ?? jo.defaultOptions.bezierSmoothness));
    let h = (Xm - c) / 1;
    return h *= h, Km(t, e, s, i, r, o, n, h), n;
  }
  function Km(n, t, e, s, i, r, o, a) {
    fo(o, n, t, e, s, i, r, a, 0), o.push(i, r);
  }
  function fo(n, t, e, s, i, r, o, a, l) {
    if (l > Vm) return;
    const c = (t + s) / 2, h = (e + i) / 2, d = (s + r) / 2, u = (i + o) / 2, p = (c + d) / 2, f = (h + u) / 2;
    let g = r - t, m = o - e;
    const y = Math.abs((s - r) * m - (i - o) * g);
    if (y > Ym) {
      if (y * y <= a * (g * g + m * m)) {
        n.push(p, f);
        return;
      }
    } else if (g = p - (t + r) / 2, m = f - (e + o) / 2, g * g + m * m <= a) {
      n.push(p, f);
      return;
    }
    fo(n, t, e, c, h, p, f, a, l + 1), fo(n, p, f, d, u, r, o, a, l + 1);
  }
  function Lh(n, t, e, s, i, r, o, a) {
    let l = Math.abs(i - r);
    (!o && i > r || o && r > i) && (l = 2 * Math.PI - l), a || (a = Math.max(6, Math.floor(6 * Math.pow(s, 1 / 3) * (l / Math.PI)))), a = Math.max(a, 3);
    let c = l / a, h = i;
    c *= o ? -1 : 1;
    for (let d = 0; d < a + 1; d++) {
      const u = Math.cos(h), p = Math.sin(h), f = t + u * s, g = e + p * s;
      n.push(f, g), h += c;
    }
  }
  function Jm(n, t, e, s, i, r) {
    const o = n[n.length - 2], l = n[n.length - 1] - e, c = o - t, h = i - e, d = s - t, u = Math.abs(l * d - c * h);
    if (u < 1e-8 || r === 0) {
      (n[n.length - 2] !== t || n[n.length - 1] !== e) && n.push(t, e);
      return;
    }
    const p = l * l + c * c, f = h * h + d * d, g = l * h + c * d, m = r * Math.sqrt(p) / u, y = r * Math.sqrt(f) / u, b = m * g / p, x = y * g / f, _ = m * d + y * c, v = m * h + y * l, w = c * (y + b), C = l * (y + b), E = d * (m + x), P = h * (m + x), T = Math.atan2(C - v, w - _), A = Math.atan2(P - v, E - _);
    Lh(n, _ + t, v + e, r, T, A, c * h > d * l);
  }
  const Gs = Math.PI * 2, Ir = {
    centerX: 0,
    centerY: 0,
    ang1: 0,
    ang2: 0
  }, Rr = ({ x: n, y: t }, e, s, i, r, o, a, l) => {
    n *= e, t *= s;
    const c = i * n - r * t, h = r * n + i * t;
    return l.x = c + o, l.y = h + a, l;
  };
  function Zm(n, t) {
    const e = t === -1.5707963267948966 ? -0.551915024494 : 1.3333333333333333 * Math.tan(t / 4), s = t === 1.5707963267948966 ? 0.551915024494 : e, i = Math.cos(n), r = Math.sin(n), o = Math.cos(n + t), a = Math.sin(n + t);
    return [
      {
        x: i - r * s,
        y: r + i * s
      },
      {
        x: o + a * s,
        y: a - o * s
      },
      {
        x: o,
        y: a
      }
    ];
  }
  const ll = (n, t, e, s) => {
    const i = n * s - t * e < 0 ? -1 : 1;
    let r = n * e + t * s;
    return r > 1 && (r = 1), r < -1 && (r = -1), i * Math.acos(r);
  }, Qm = (n, t, e, s, i, r, o, a, l, c, h, d, u) => {
    const p = Math.pow(i, 2), f = Math.pow(r, 2), g = Math.pow(h, 2), m = Math.pow(d, 2);
    let y = p * f - p * m - f * g;
    y < 0 && (y = 0), y /= p * m + f * g, y = Math.sqrt(y) * (o === a ? -1 : 1);
    const b = y * i / r * d, x = y * -r / i * h, _ = c * b - l * x + (n + e) / 2, v = l * b + c * x + (t + s) / 2, w = (h - b) / i, C = (d - x) / r, E = (-h - b) / i, P = (-d - x) / r, T = ll(1, 0, w, C);
    let A = ll(w, C, E, P);
    a === 0 && A > 0 && (A -= Gs), a === 1 && A < 0 && (A += Gs), u.centerX = _, u.centerY = v, u.ang1 = T, u.ang2 = A;
  };
  function tg(n, t, e, s, i, r, o, a = 0, l = 0, c = 0) {
    if (r === 0 || o === 0) return;
    const h = Math.sin(a * Gs / 360), d = Math.cos(a * Gs / 360), u = d * (t - s) / 2 + h * (e - i) / 2, p = -h * (t - s) / 2 + d * (e - i) / 2;
    if (u === 0 && p === 0) return;
    r = Math.abs(r), o = Math.abs(o);
    const f = Math.pow(u, 2) / Math.pow(r, 2) + Math.pow(p, 2) / Math.pow(o, 2);
    f > 1 && (r *= Math.sqrt(f), o *= Math.sqrt(f)), Qm(t, e, s, i, r, o, l, c, h, d, u, p, Ir);
    let { ang1: g, ang2: m } = Ir;
    const { centerX: y, centerY: b } = Ir;
    let x = Math.abs(m) / (Gs / 4);
    Math.abs(1 - x) < 1e-7 && (x = 1);
    const _ = Math.max(Math.ceil(x), 1);
    m /= _;
    let v = n[n.length - 2], w = n[n.length - 1];
    const C = {
      x: 0,
      y: 0
    };
    for (let E = 0; E < _; E++) {
      const P = Zm(g, m), { x: T, y: A } = Rr(P[0], r, o, d, h, y, b, C), { x: H, y: j } = Rr(P[1], r, o, d, h, y, b, C), { x: D, y: N } = Rr(P[2], r, o, d, h, y, b, C);
      Rh(n, v, w, T, A, H, j, D, N), v = D, w = N, g += m;
    }
  }
  function eg(n, t, e) {
    const s = (o, a) => {
      const l = a.x - o.x, c = a.y - o.y, h = Math.sqrt(l * l + c * c), d = l / h, u = c / h;
      return {
        len: h,
        nx: d,
        ny: u
      };
    }, i = (o, a) => {
      o === 0 ? n.moveTo(a.x, a.y) : n.lineTo(a.x, a.y);
    };
    let r = t[t.length - 1];
    for (let o = 0; o < t.length; o++) {
      const a = t[o % t.length], l = a.radius ?? e;
      if (l <= 0) {
        i(o, a), r = a;
        continue;
      }
      const c = t[(o + 1) % t.length], h = s(a, r), d = s(a, c);
      if (h.len < 1e-4 || d.len < 1e-4) {
        i(o, a), r = a;
        continue;
      }
      let u = Math.asin(h.nx * d.ny - h.ny * d.nx), p = 1, f = false;
      h.nx * d.nx - h.ny * -d.ny < 0 ? u < 0 ? u = Math.PI + u : (u = Math.PI - u, p = -1, f = true) : u > 0 && (p = -1, f = true);
      const g = u / 2;
      let m, y = Math.abs(Math.cos(g) * l / Math.sin(g));
      y > Math.min(h.len / 2, d.len / 2) ? (y = Math.min(h.len / 2, d.len / 2), m = Math.abs(y * Math.sin(g) / Math.cos(g))) : m = l;
      const b = a.x + d.nx * y + -d.ny * m * p, x = a.y + d.ny * y + d.nx * m * p, _ = Math.atan2(h.ny, h.nx) + Math.PI / 2 * p, v = Math.atan2(d.ny, d.nx) - Math.PI / 2 * p;
      o === 0 && n.moveTo(b + Math.cos(_) * m, x + Math.sin(_) * m), n.arc(b, x, m, _, v, f), r = a;
    }
  }
  function ng(n, t, e, s) {
    const i = (a, l) => Math.sqrt((a.x - l.x) ** 2 + (a.y - l.y) ** 2), r = (a, l, c) => ({
      x: a.x + (l.x - a.x) * c,
      y: a.y + (l.y - a.y) * c
    }), o = t.length;
    for (let a = 0; a < o; a++) {
      const l = t[(a + 1) % o], c = l.radius ?? e;
      if (c <= 0) {
        a === 0 ? n.moveTo(l.x, l.y) : n.lineTo(l.x, l.y);
        continue;
      }
      const h = t[a], d = t[(a + 2) % o], u = i(h, l);
      let p;
      if (u < 1e-4) p = l;
      else {
        const m = Math.min(u / 2, c);
        p = r(l, h, m / u);
      }
      const f = i(d, l);
      let g;
      if (f < 1e-4) g = l;
      else {
        const m = Math.min(f / 2, c);
        g = r(l, d, m / f);
      }
      a === 0 ? n.moveTo(p.x, p.y) : n.lineTo(p.x, p.y), n.quadraticCurveTo(l.x, l.y, g.x, g.y, s);
    }
  }
  const sg = new Nt();
  class ig {
    constructor(t) {
      this.shapePrimitives = [], this._currentPoly = null, this._bounds = new Se(), this._graphicsPath2D = t, this.signed = t.checkForHoles;
    }
    moveTo(t, e) {
      return this.startPoly(t, e), this;
    }
    lineTo(t, e) {
      this._ensurePoly();
      const s = this._currentPoly.points, i = s[s.length - 2], r = s[s.length - 1];
      return (i !== t || r !== e) && s.push(t, e), this;
    }
    arc(t, e, s, i, r, o) {
      this._ensurePoly(false);
      const a = this._currentPoly.points;
      return Lh(a, t, e, s, i, r, o), this;
    }
    arcTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly.points;
      return Jm(o, t, e, s, i, r), this;
    }
    arcToSvg(t, e, s, i, r, o, a) {
      const l = this._currentPoly.points;
      return tg(l, this._currentPoly.lastX, this._currentPoly.lastY, o, a, t, e, s, i, r), this;
    }
    bezierCurveTo(t, e, s, i, r, o, a) {
      this._ensurePoly();
      const l = this._currentPoly;
      return Rh(this._currentPoly.points, l.lastX, l.lastY, t, e, s, i, r, o, a), this;
    }
    quadraticCurveTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly;
      return qm(this._currentPoly.points, o.lastX, o.lastY, t, e, s, i, r), this;
    }
    closePath() {
      return this.endPoly(true), this;
    }
    addPath(t, e) {
      this.endPoly(), e && !e.isIdentity() && (t = t.clone(true), t.transform(e));
      const s = this.shapePrimitives, i = s.length;
      for (let r = 0; r < t.instructions.length; r++) {
        const o = t.instructions[r];
        this[o.action](...o.data);
      }
      if (t.checkForHoles && s.length - i > 1) {
        let r = null;
        for (let o = i; o < s.length; o++) {
          const a = s[o];
          if (a.shape.type === "polygon") {
            const l = a.shape, c = r == null ? void 0 : r.shape;
            c && c.containsPolygon(l) ? (r.holes || (r.holes = []), r.holes.push(a), s.copyWithin(o, o + 1), s.length--, o--) : r = a;
          }
        }
      }
      return this;
    }
    finish(t = false) {
      this.endPoly(t);
    }
    rect(t, e, s, i, r) {
      return this.drawShape(new Nt(t, e, s, i), r), this;
    }
    circle(t, e, s, i) {
      return this.drawShape(new Wo(t, e, s), i), this;
    }
    poly(t, e, s) {
      const i = new Ns(t);
      return i.closePath = e, this.drawShape(i, s), this;
    }
    regularPoly(t, e, s, i, r = 0, o) {
      i = Math.max(i | 0, 3);
      const a = -1 * Math.PI / 2 + r, l = Math.PI * 2 / i, c = [];
      for (let h = 0; h < i; h++) {
        const d = a - h * l;
        c.push(t + s * Math.cos(d), e + s * Math.sin(d));
      }
      return this.poly(c, true, o), this;
    }
    roundPoly(t, e, s, i, r, o = 0, a) {
      if (i = Math.max(i | 0, 3), r <= 0) return this.regularPoly(t, e, s, i, o);
      const l = s * Math.sin(Math.PI / i) - 1e-3;
      r = Math.min(r, l);
      const c = -1 * Math.PI / 2 + o, h = Math.PI * 2 / i, d = (i - 2) * Math.PI / i / 2;
      for (let u = 0; u < i; u++) {
        const p = u * h + c, f = t + s * Math.cos(p), g = e + s * Math.sin(p), m = p + Math.PI + d, y = p - Math.PI - d, b = f + r * Math.cos(m), x = g + r * Math.sin(m), _ = f + r * Math.cos(y), v = g + r * Math.sin(y);
        u === 0 ? this.moveTo(b, x) : this.lineTo(b, x), this.quadraticCurveTo(f, g, _, v, a);
      }
      return this.closePath();
    }
    roundShape(t, e, s = false, i) {
      return t.length < 3 ? this : (s ? ng(this, t, e, i) : eg(this, t, e), this.closePath());
    }
    filletRect(t, e, s, i, r) {
      if (r === 0) return this.rect(t, e, s, i);
      const o = Math.min(s, i) / 2, a = Math.min(o, Math.max(-o, r)), l = t + s, c = e + i, h = a < 0 ? -a : 0, d = Math.abs(a);
      return this.moveTo(t, e + d).arcTo(t + h, e + h, t + d, e, d).lineTo(l - d, e).arcTo(l - h, e + h, l, e + d, d).lineTo(l, c - d).arcTo(l - h, c - h, t + s - d, c, d).lineTo(t + d, c).arcTo(t + h, c - h, t, c - d, d).closePath();
    }
    chamferRect(t, e, s, i, r, o) {
      if (r <= 0) return this.rect(t, e, s, i);
      const a = Math.min(r, Math.min(s, i) / 2), l = t + s, c = e + i, h = [
        t + a,
        e,
        l - a,
        e,
        l,
        e + a,
        l,
        c - a,
        l - a,
        c,
        t + a,
        c,
        t,
        c - a,
        t,
        e + a
      ];
      for (let d = h.length - 1; d >= 2; d -= 2) h[d] === h[d - 2] && h[d - 1] === h[d - 3] && h.splice(d - 1, 2);
      return this.poly(h, true, o);
    }
    ellipse(t, e, s, i, r) {
      return this.drawShape(new Go(t, e, s, i), r), this;
    }
    roundRect(t, e, s, i, r, o) {
      return this.drawShape(new zo(t, e, s, i, r), o), this;
    }
    drawShape(t, e) {
      return this.endPoly(), this.shapePrimitives.push({
        shape: t,
        transform: e
      }), this;
    }
    startPoly(t, e) {
      let s = this._currentPoly;
      return s && this.endPoly(), s = new Ns(), s.points.push(t, e), this._currentPoly = s, this;
    }
    endPoly(t = false) {
      const e = this._currentPoly;
      return e && e.points.length > 2 && (e.closePath = t, this.shapePrimitives.push({
        shape: e
      })), this._currentPoly = null, this;
    }
    _ensurePoly(t = true) {
      if (!this._currentPoly && (this._currentPoly = new Ns(), t)) {
        const e = this.shapePrimitives[this.shapePrimitives.length - 1];
        if (e) {
          let s = e.shape.x, i = e.shape.y;
          if (e.transform && !e.transform.isIdentity()) {
            const r = e.transform, o = s;
            s = r.a * s + r.c * i + r.tx, i = r.b * o + r.d * i + r.ty;
          }
          this._currentPoly.points.push(s, i);
        } else this._currentPoly.points.push(0, 0);
      }
    }
    buildPath() {
      const t = this._graphicsPath2D;
      this.shapePrimitives.length = 0, this._currentPoly = null;
      for (let e = 0; e < t.instructions.length; e++) {
        const s = t.instructions[e];
        this[s.action](...s.data);
      }
      this.finish();
    }
    get bounds() {
      const t = this._bounds;
      t.clear();
      const e = this.shapePrimitives;
      for (let s = 0; s < e.length; s++) {
        const i = e[s], r = i.shape.getBounds(sg);
        i.transform ? t.addRect(r, i.transform) : t.addRect(r);
      }
      return t;
    }
  }
  class dn {
    constructor(t, e = false) {
      this.instructions = [], this.uid = Kt("graphicsPath"), this._dirty = true, this.checkForHoles = e, typeof t == "string" ? Af(t, this) : this.instructions = (t == null ? void 0 : t.slice()) ?? [];
    }
    get shapePath() {
      return this._shapePath || (this._shapePath = new ig(this)), this._dirty && (this._dirty = false, this._shapePath.buildPath()), this._shapePath;
    }
    addPath(t, e) {
      return t = t.clone(), this.instructions.push({
        action: "addPath",
        data: [
          t,
          e
        ]
      }), this._dirty = true, this;
    }
    arc(...t) {
      return this.instructions.push({
        action: "arc",
        data: t
      }), this._dirty = true, this;
    }
    arcTo(...t) {
      return this.instructions.push({
        action: "arcTo",
        data: t
      }), this._dirty = true, this;
    }
    arcToSvg(...t) {
      return this.instructions.push({
        action: "arcToSvg",
        data: t
      }), this._dirty = true, this;
    }
    bezierCurveTo(...t) {
      return this.instructions.push({
        action: "bezierCurveTo",
        data: t
      }), this._dirty = true, this;
    }
    bezierCurveToShort(t, e, s, i, r) {
      const o = this.instructions[this.instructions.length - 1], a = this.getLastPoint(At.shared);
      let l = 0, c = 0;
      if (!o || o.action !== "bezierCurveTo") l = a.x, c = a.y;
      else {
        l = o.data[2], c = o.data[3];
        const h = a.x, d = a.y;
        l = h + (h - l), c = d + (d - c);
      }
      return this.instructions.push({
        action: "bezierCurveTo",
        data: [
          l,
          c,
          t,
          e,
          s,
          i,
          r
        ]
      }), this._dirty = true, this;
    }
    closePath() {
      return this.instructions.push({
        action: "closePath",
        data: []
      }), this._dirty = true, this;
    }
    ellipse(...t) {
      return this.instructions.push({
        action: "ellipse",
        data: t
      }), this._dirty = true, this;
    }
    lineTo(...t) {
      return this.instructions.push({
        action: "lineTo",
        data: t
      }), this._dirty = true, this;
    }
    moveTo(...t) {
      return this.instructions.push({
        action: "moveTo",
        data: t
      }), this;
    }
    quadraticCurveTo(...t) {
      return this.instructions.push({
        action: "quadraticCurveTo",
        data: t
      }), this._dirty = true, this;
    }
    quadraticCurveToShort(t, e, s) {
      const i = this.instructions[this.instructions.length - 1], r = this.getLastPoint(At.shared);
      let o = 0, a = 0;
      if (!i || i.action !== "quadraticCurveTo") o = r.x, a = r.y;
      else {
        o = i.data[0], a = i.data[1];
        const l = r.x, c = r.y;
        o = l + (l - o), a = c + (c - a);
      }
      return this.instructions.push({
        action: "quadraticCurveTo",
        data: [
          o,
          a,
          t,
          e,
          s
        ]
      }), this._dirty = true, this;
    }
    rect(t, e, s, i, r) {
      return this.instructions.push({
        action: "rect",
        data: [
          t,
          e,
          s,
          i,
          r
        ]
      }), this._dirty = true, this;
    }
    circle(t, e, s, i) {
      return this.instructions.push({
        action: "circle",
        data: [
          t,
          e,
          s,
          i
        ]
      }), this._dirty = true, this;
    }
    roundRect(...t) {
      return this.instructions.push({
        action: "roundRect",
        data: t
      }), this._dirty = true, this;
    }
    poly(...t) {
      return this.instructions.push({
        action: "poly",
        data: t
      }), this._dirty = true, this;
    }
    regularPoly(...t) {
      return this.instructions.push({
        action: "regularPoly",
        data: t
      }), this._dirty = true, this;
    }
    roundPoly(...t) {
      return this.instructions.push({
        action: "roundPoly",
        data: t
      }), this._dirty = true, this;
    }
    roundShape(...t) {
      return this.instructions.push({
        action: "roundShape",
        data: t
      }), this._dirty = true, this;
    }
    filletRect(...t) {
      return this.instructions.push({
        action: "filletRect",
        data: t
      }), this._dirty = true, this;
    }
    chamferRect(...t) {
      return this.instructions.push({
        action: "chamferRect",
        data: t
      }), this._dirty = true, this;
    }
    star(t, e, s, i, r, o, a) {
      r || (r = i / 2);
      const l = -1 * Math.PI / 2 + o, c = s * 2, h = Math.PI * 2 / c, d = [];
      for (let u = 0; u < c; u++) {
        const p = u % 2 ? r : i, f = u * h + l;
        d.push(t + p * Math.cos(f), e + p * Math.sin(f));
      }
      return this.poly(d, true, a), this;
    }
    clone(t = false) {
      const e = new dn();
      if (e.checkForHoles = this.checkForHoles, !t) e.instructions = this.instructions.slice();
      else for (let s = 0; s < this.instructions.length; s++) {
        const i = this.instructions[s];
        e.instructions.push({
          action: i.action,
          data: i.data.slice()
        });
      }
      return e;
    }
    clear() {
      return this.instructions.length = 0, this._dirty = true, this;
    }
    transform(t) {
      if (t.isIdentity()) return this;
      const e = t.a, s = t.b, i = t.c, r = t.d, o = t.tx, a = t.ty;
      let l = 0, c = 0, h = 0, d = 0, u = 0, p = 0, f = 0, g = 0;
      for (let m = 0; m < this.instructions.length; m++) {
        const y = this.instructions[m], b = y.data;
        switch (y.action) {
          case "moveTo":
          case "lineTo":
            l = b[0], c = b[1], b[0] = e * l + i * c + o, b[1] = s * l + r * c + a;
            break;
          case "bezierCurveTo":
            h = b[0], d = b[1], u = b[2], p = b[3], l = b[4], c = b[5], b[0] = e * h + i * d + o, b[1] = s * h + r * d + a, b[2] = e * u + i * p + o, b[3] = s * u + r * p + a, b[4] = e * l + i * c + o, b[5] = s * l + r * c + a;
            break;
          case "quadraticCurveTo":
            h = b[0], d = b[1], l = b[2], c = b[3], b[0] = e * h + i * d + o, b[1] = s * h + r * d + a, b[2] = e * l + i * c + o, b[3] = s * l + r * c + a;
            break;
          case "arcToSvg":
            l = b[5], c = b[6], f = b[0], g = b[1], b[0] = e * f + i * g, b[1] = s * f + r * g, b[5] = e * l + i * c + o, b[6] = s * l + r * c + a;
            break;
          case "circle":
            b[4] = Es(b[3], t);
            break;
          case "rect":
            b[4] = Es(b[4], t);
            break;
          case "ellipse":
            b[8] = Es(b[8], t);
            break;
          case "roundRect":
            b[5] = Es(b[5], t);
            break;
          case "addPath":
            b[0].transform(t);
            break;
          case "poly":
            b[2] = Es(b[2], t);
            break;
          default:
            Vt("unknown transform action", y.action);
            break;
        }
      }
      return this._dirty = true, this;
    }
    get bounds() {
      return this.shapePath.bounds;
    }
    getLastPoint(t) {
      let e = this.instructions.length - 1, s = this.instructions[e];
      if (!s) return t.x = 0, t.y = 0, t;
      for (; s.action === "closePath"; ) {
        if (e--, e < 0) return t.x = 0, t.y = 0, t;
        s = this.instructions[e];
      }
      switch (s.action) {
        case "moveTo":
        case "lineTo":
          t.x = s.data[0], t.y = s.data[1];
          break;
        case "quadraticCurveTo":
          t.x = s.data[2], t.y = s.data[3];
          break;
        case "bezierCurveTo":
          t.x = s.data[4], t.y = s.data[5];
          break;
        case "arc":
        case "arcToSvg":
          t.x = s.data[5], t.y = s.data[6];
          break;
        case "addPath":
          s.data[0].getLastPoint(t);
          break;
      }
      return t;
    }
  }
  function Es(n, t) {
    return n ? n.prepend(t) : t.clone();
  }
  function Xt(n, t, e) {
    const s = n.getAttribute(t);
    return s ? Number(s) : e;
  }
  function rg(n, t) {
    const e = n.querySelectorAll("defs");
    for (let s = 0; s < e.length; s++) {
      const i = e[s];
      for (let r = 0; r < i.children.length; r++) {
        const o = i.children[r];
        switch (o.nodeName.toLowerCase()) {
          case "lineargradient":
            t.defs[o.id] = og(o);
            break;
          case "radialgradient":
            t.defs[o.id] = ag();
            break;
        }
      }
    }
  }
  function og(n) {
    const t = Xt(n, "x1", 0), e = Xt(n, "y1", 0), s = Xt(n, "x2", 1), i = Xt(n, "y2", 0), r = n.getAttribute("gradientUnits") || "objectBoundingBox", o = new fn(t, e, s, i, r === "objectBoundingBox" ? "local" : "global");
    for (let a = 0; a < n.children.length; a++) {
      const l = n.children[a], c = Xt(l, "offset", 0), h = Gt.shared.setValue(l.getAttribute("stop-color")).toNumber();
      o.addColorStop(c, h);
    }
    return o;
  }
  function ag(n) {
    return Vt("[SVG Parser] Radial gradients are not yet supported"), new fn(0, 0, 1, 0);
  }
  function cl(n) {
    const t = n.match(/url\s*\(\s*['"]?\s*#([^'"\s)]+)\s*['"]?\s*\)/i);
    return t ? t[1] : "";
  }
  const hl = {
    fill: {
      type: "paint",
      default: 0
    },
    "fill-opacity": {
      type: "number",
      default: 1
    },
    stroke: {
      type: "paint",
      default: 0
    },
    "stroke-width": {
      type: "number",
      default: 1
    },
    "stroke-opacity": {
      type: "number",
      default: 1
    },
    "stroke-linecap": {
      type: "string",
      default: "butt"
    },
    "stroke-linejoin": {
      type: "string",
      default: "miter"
    },
    "stroke-miterlimit": {
      type: "number",
      default: 10
    },
    "stroke-dasharray": {
      type: "string",
      default: "none"
    },
    "stroke-dashoffset": {
      type: "number",
      default: 0
    },
    opacity: {
      type: "number",
      default: 1
    }
  };
  function $h(n, t) {
    const e = n.getAttribute("style"), s = {}, i = {}, r = {
      strokeStyle: s,
      fillStyle: i,
      useFill: false,
      useStroke: false
    };
    for (const o in hl) {
      const a = n.getAttribute(o);
      a && dl(t, r, o, a.trim());
    }
    if (e) {
      const o = e.split(";");
      for (let a = 0; a < o.length; a++) {
        const l = o[a].trim(), [c, h] = l.split(":");
        hl[c] && dl(t, r, c, h.trim());
      }
    }
    return {
      strokeStyle: r.useStroke ? s : null,
      fillStyle: r.useFill ? i : null,
      useFill: r.useFill,
      useStroke: r.useStroke
    };
  }
  function dl(n, t, e, s) {
    switch (e) {
      case "stroke":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = cl(s);
            t.strokeStyle.fill = n.defs[i];
          } else t.strokeStyle.color = Gt.shared.setValue(s).toNumber();
          t.useStroke = true;
        }
        break;
      case "stroke-width":
        t.strokeStyle.width = Number(s);
        break;
      case "fill":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = cl(s);
            t.fillStyle.fill = n.defs[i];
          } else t.fillStyle.color = Gt.shared.setValue(s).toNumber();
          t.useFill = true;
        }
        break;
      case "fill-opacity":
        t.fillStyle.alpha = Number(s);
        break;
      case "stroke-opacity":
        t.strokeStyle.alpha = Number(s);
        break;
      case "opacity":
        t.fillStyle.alpha = Number(s), t.strokeStyle.alpha = Number(s);
        break;
    }
  }
  function lg(n) {
    if (n.length <= 2) return true;
    const t = n.map((a) => a.area).sort((a, l) => l - a), [e, s] = t, i = t[t.length - 1], r = e / s, o = s / i;
    return !(r > 3 && o < 2);
  }
  function cg(n) {
    return n.split(/(?=[Mm])/).filter((s) => s.trim().length > 0);
  }
  function hg(n) {
    const t = n.match(/[-+]?[0-9]*\.?[0-9]+/g);
    if (!t || t.length < 4) return 0;
    const e = t.map(Number), s = [], i = [];
    for (let h = 0; h < e.length; h += 2) h + 1 < e.length && (s.push(e[h]), i.push(e[h + 1]));
    if (s.length === 0 || i.length === 0) return 0;
    const r = Math.min(...s), o = Math.max(...s), a = Math.min(...i), l = Math.max(...i);
    return (o - r) * (l - a);
  }
  function ul(n, t) {
    const e = new dn(n, false);
    for (const s of e.instructions) t.instructions.push(s);
  }
  function dg(n, t) {
    if (typeof n == "string") {
      const o = document.createElement("div");
      o.innerHTML = n.trim(), n = o.querySelector("svg");
    }
    const e = {
      context: t,
      defs: {},
      path: new dn()
    };
    rg(n, e);
    const s = n.children, { fillStyle: i, strokeStyle: r } = $h(n, e);
    for (let o = 0; o < s.length; o++) {
      const a = s[o];
      a.nodeName.toLowerCase() !== "defs" && Bh(a, e, i, r);
    }
    return t;
  }
  function Bh(n, t, e, s) {
    const i = n.children, { fillStyle: r, strokeStyle: o } = $h(n, t);
    r && e ? e = {
      ...e,
      ...r
    } : r && (e = r), o && s ? s = {
      ...s,
      ...o
    } : o && (s = o);
    const a = !e && !s;
    a && (e = {
      color: 0
    });
    let l, c, h, d, u, p, f, g, m, y, b, x, _, v, w, C, E;
    switch (n.nodeName.toLowerCase()) {
      case "path": {
        v = n.getAttribute("d");
        const P = n.getAttribute("fill-rule"), T = cg(v), A = P === "evenodd", H = T.length > 1;
        if (A && H) {
          const D = T.map(($) => ({
            path: $,
            area: hg($)
          }));
          if (D.sort(($, W) => W.area - $.area), T.length > 3 || !lg(D)) for (let $ = 0; $ < D.length; $++) {
            const W = D[$], X = $ === 0;
            t.context.beginPath();
            const U = new dn(void 0, true);
            ul(W.path, U), t.context.path(U), X ? (e && t.context.fill(e), s && t.context.stroke(s)) : t.context.cut();
          }
          else for (let $ = 0; $ < D.length; $++) {
            const W = D[$], X = $ % 2 === 1;
            t.context.beginPath();
            const U = new dn(void 0, true);
            ul(W.path, U), t.context.path(U), X ? t.context.cut() : (e && t.context.fill(e), s && t.context.stroke(s));
          }
        } else {
          const D = P ? P === "evenodd" : true;
          w = new dn(v, D), t.context.path(w), e && t.context.fill(e), s && t.context.stroke(s);
        }
        break;
      }
      case "circle":
        f = Xt(n, "cx", 0), g = Xt(n, "cy", 0), m = Xt(n, "r", 0), t.context.ellipse(f, g, m, m), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "rect":
        l = Xt(n, "x", 0), c = Xt(n, "y", 0), C = Xt(n, "width", 0), E = Xt(n, "height", 0), y = Xt(n, "rx", 0), b = Xt(n, "ry", 0), y || b ? t.context.roundRect(l, c, C, E, y || b) : t.context.rect(l, c, C, E), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "ellipse":
        f = Xt(n, "cx", 0), g = Xt(n, "cy", 0), y = Xt(n, "rx", 0), b = Xt(n, "ry", 0), t.context.beginPath(), t.context.ellipse(f, g, y, b), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "line":
        h = Xt(n, "x1", 0), d = Xt(n, "y1", 0), u = Xt(n, "x2", 0), p = Xt(n, "y2", 0), t.context.beginPath(), t.context.moveTo(h, d), t.context.lineTo(u, p), s && t.context.stroke(s);
        break;
      case "polygon":
        _ = n.getAttribute("points"), x = _.match(/-?\d+/g).map((P) => parseInt(P, 10)), t.context.poly(x, true), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "polyline":
        _ = n.getAttribute("points"), x = _.match(/-?\d+/g).map((P) => parseInt(P, 10)), t.context.poly(x, false), s && t.context.stroke(s);
        break;
      case "g":
      case "svg":
        break;
      default: {
        Vt(`[SVG parser] <${n.nodeName}> elements unsupported`);
        break;
      }
    }
    a && (e = null);
    for (let P = 0; P < i.length; P++) Bh(i[P], t, e, s);
  }
  const pl = {
    repeat: {
      addressModeU: "repeat",
      addressModeV: "repeat"
    },
    "repeat-x": {
      addressModeU: "repeat",
      addressModeV: "clamp-to-edge"
    },
    "repeat-y": {
      addressModeU: "clamp-to-edge",
      addressModeV: "repeat"
    },
    "no-repeat": {
      addressModeU: "clamp-to-edge",
      addressModeV: "clamp-to-edge"
    }
  };
  Ji = class {
    constructor(t, e) {
      this.uid = Kt("fillPattern"), this._tick = 0, this.transform = new _t(), this.texture = t, this.transform.scale(1 / t.frame.width, 1 / t.frame.height), e && (t.source.style.addressModeU = pl[e].addressModeU, t.source.style.addressModeV = pl[e].addressModeV);
    }
    setTransform(t) {
      const e = this.texture;
      this.transform.copyFrom(t), this.transform.invert(), this.transform.scale(1 / e.frame.width, 1 / e.frame.height), this._tick++;
    }
    get texture() {
      return this._texture;
    }
    set texture(t) {
      this._texture !== t && (this._texture = t, this._tick++);
    }
    get styleKey() {
      return `fill-pattern-${this.uid}-${this._tick}`;
    }
    destroy() {
      this.texture.destroy(true), this.texture = null;
    }
  };
  function ug(n) {
    return Gt.isColorLike(n);
  }
  function fl(n) {
    return n instanceof Ji;
  }
  function ml(n) {
    return n instanceof fn;
  }
  function pg(n) {
    return n instanceof Ct;
  }
  function fg(n, t, e) {
    const s = Gt.shared.setValue(t ?? 0);
    return n.color = s.toNumber(), n.alpha = s.alpha === 1 ? e.alpha : s.alpha, n.texture = Ct.WHITE, {
      ...e,
      ...n
    };
  }
  function mg(n, t, e) {
    return n.texture = t, {
      ...e,
      ...n
    };
  }
  function gl(n, t, e) {
    return n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, {
      ...e,
      ...n
    };
  }
  function yl(n, t, e) {
    return t.buildGradient(), n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, n.textureSpace = t.textureSpace, {
      ...e,
      ...n
    };
  }
  function gg(n, t) {
    const e = {
      ...t,
      ...n
    }, s = Gt.shared.setValue(e.color);
    return e.alpha *= s.alpha, e.color = s.toNumber(), e;
  }
  function Wn(n, t) {
    if (n == null) return null;
    const e = {}, s = n;
    return ug(n) ? fg(e, n, t) : pg(n) ? mg(e, n, t) : fl(n) ? gl(e, n, t) : ml(n) ? yl(e, n, t) : s.fill && fl(s.fill) ? gl(s, s.fill, t) : s.fill && ml(s.fill) ? yl(s, s.fill, t) : gg(s, t);
  }
  function Ni(n, t) {
    const { width: e, alignment: s, miterLimit: i, cap: r, join: o, pixelLine: a, ...l } = t, c = Wn(n, l);
    return c ? {
      width: e,
      alignment: s,
      miterLimit: i,
      cap: r,
      join: o,
      pixelLine: a,
      ...c
    } : null;
  }
  function yg(n, t) {
    let e = 1;
    const s = n.shapePath.shapePrimitives;
    for (let i = 0; i < s.length; i++) {
      const r = s[i].shape;
      if (r.type !== "polygon") continue;
      const o = r.points, a = o.length;
      if (a < 6) continue;
      const l = r.closePath;
      for (let c = 0; c < a; c += 2) {
        if (!l && (c === 0 || c === a - 2)) continue;
        const h = (c - 2 + a) % a, d = (c + 2) % a, u = o[h], p = o[h + 1], f = o[c], g = o[c + 1], m = o[d], y = o[d + 1], b = u - f, x = p - g, _ = m - f, v = y - g, w = b * b + x * x, C = _ * _ + v * v;
        if (w < 1e-12 || C < 1e-12) continue;
        let T = (b * _ + x * v) / Math.sqrt(w * C);
        T < -1 ? T = -1 : T > 1 && (T = 1);
        const A = Math.sqrt((1 - T) * 0.5);
        if (A < 1e-6) continue;
        const H = Math.min(1 / A, t);
        H > e && (e = H);
      }
    }
    return e;
  }
  const xg = new At(), xl = new _t(), Vo = class Ye extends Qe {
    constructor() {
      super(...arguments), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = Kt("graphicsContext"), this.dirty = true, this.batchMode = "auto", this.instructions = [], this.destroyed = false, this._activePath = new dn(), this._transform = new _t(), this._fillStyle = {
        ...Ye.defaultFillStyle
      }, this._strokeStyle = {
        ...Ye.defaultStrokeStyle
      }, this._stateStack = [], this._tick = 0, this._bounds = new Se(), this._boundsDirty = true;
    }
    clone() {
      const t = new Ye();
      return t.batchMode = this.batchMode, t.instructions = this.instructions.slice(), t._activePath = this._activePath.clone(), t._transform = this._transform.clone(), t._fillStyle = {
        ...this._fillStyle
      }, t._strokeStyle = {
        ...this._strokeStyle
      }, t._stateStack = this._stateStack.slice(), t._bounds = this._bounds.clone(), t._boundsDirty = true, t;
    }
    get fillStyle() {
      return this._fillStyle;
    }
    set fillStyle(t) {
      this._fillStyle = Wn(t, Ye.defaultFillStyle);
    }
    get strokeStyle() {
      return this._strokeStyle;
    }
    set strokeStyle(t) {
      this._strokeStyle = Ni(t, Ye.defaultStrokeStyle);
    }
    setFillStyle(t) {
      return this._fillStyle = Wn(t, Ye.defaultFillStyle), this;
    }
    setStrokeStyle(t) {
      return this._strokeStyle = Wn(t, Ye.defaultStrokeStyle), this;
    }
    texture(t, e, s, i, r, o) {
      return this.instructions.push({
        action: "texture",
        data: {
          image: t,
          dx: s || 0,
          dy: i || 0,
          dw: r || t.frame.width,
          dh: o || t.frame.height,
          transform: this._transform.clone(),
          alpha: this._fillStyle.alpha,
          style: e || e === 0 ? Gt.shared.setValue(e).toNumber() : 16777215
        }
      }), this.onUpdate(), this;
    }
    beginPath() {
      return this._activePath = new dn(), this;
    }
    fill(t, e) {
      let s;
      const i = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (i == null ? void 0 : i.action) === "stroke" ? s = i.data.path : s = this._activePath.clone(), s ? (t != null && (e !== void 0 && typeof t == "number" && (Et(qt, "GraphicsContext.fill(color, alpha) is deprecated, use GraphicsContext.fill({ color, alpha }) instead"), t = {
        color: t,
        alpha: e
      }), this._fillStyle = Wn(t, Ye.defaultFillStyle)), this.instructions.push({
        action: "fill",
        data: {
          style: this.fillStyle,
          path: s
        }
      }), this.onUpdate(), this._initNextPathLocation(), this._tick = 0, this) : this;
    }
    _initNextPathLocation() {
      const { x: t, y: e } = this._activePath.getLastPoint(At.shared);
      this._activePath.clear(), this._activePath.moveTo(t, e);
    }
    stroke(t) {
      let e;
      const s = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (s == null ? void 0 : s.action) === "fill" ? e = s.data.path : e = this._activePath.clone(), e ? (t != null && (this._strokeStyle = Ni(t, Ye.defaultStrokeStyle)), this.instructions.push({
        action: "stroke",
        data: {
          style: this.strokeStyle,
          path: e
        }
      }), this.onUpdate(), this._initNextPathLocation(), this._tick = 0, this) : this;
    }
    cut() {
      for (let t = 0; t < 2; t++) {
        const e = this.instructions[this.instructions.length - 1 - t], s = this._activePath.clone();
        if (e && (e.action === "stroke" || e.action === "fill")) if (e.data.hole) e.data.hole.addPath(s);
        else {
          e.data.hole = s;
          break;
        }
      }
      return this._initNextPathLocation(), this;
    }
    arc(t, e, s, i, r, o) {
      this._tick++;
      const a = this._transform;
      return this._activePath.arc(a.a * t + a.c * e + a.tx, a.b * t + a.d * e + a.ty, s, i, r, o), this;
    }
    arcTo(t, e, s, i, r) {
      this._tick++;
      const o = this._transform;
      return this._activePath.arcTo(o.a * t + o.c * e + o.tx, o.b * t + o.d * e + o.ty, o.a * s + o.c * i + o.tx, o.b * s + o.d * i + o.ty, r), this;
    }
    arcToSvg(t, e, s, i, r, o, a) {
      this._tick++;
      const l = this._transform;
      return this._activePath.arcToSvg(t, e, s, i, r, l.a * o + l.c * a + l.tx, l.b * o + l.d * a + l.ty), this;
    }
    bezierCurveTo(t, e, s, i, r, o, a) {
      this._tick++;
      const l = this._transform;
      return this._activePath.bezierCurveTo(l.a * t + l.c * e + l.tx, l.b * t + l.d * e + l.ty, l.a * s + l.c * i + l.tx, l.b * s + l.d * i + l.ty, l.a * r + l.c * o + l.tx, l.b * r + l.d * o + l.ty, a), this;
    }
    closePath() {
      var _a2;
      return this._tick++, (_a2 = this._activePath) == null ? void 0 : _a2.closePath(), this;
    }
    ellipse(t, e, s, i) {
      return this._tick++, this._activePath.ellipse(t, e, s, i, this._transform.clone()), this;
    }
    circle(t, e, s) {
      return this._tick++, this._activePath.circle(t, e, s, this._transform.clone()), this;
    }
    path(t) {
      return this._tick++, this._activePath.addPath(t, this._transform.clone()), this;
    }
    lineTo(t, e) {
      this._tick++;
      const s = this._transform;
      return this._activePath.lineTo(s.a * t + s.c * e + s.tx, s.b * t + s.d * e + s.ty), this;
    }
    moveTo(t, e) {
      this._tick++;
      const s = this._transform, i = this._activePath.instructions, r = s.a * t + s.c * e + s.tx, o = s.b * t + s.d * e + s.ty;
      return i.length === 1 && i[0].action === "moveTo" ? (i[0].data[0] = r, i[0].data[1] = o, this) : (this._activePath.moveTo(r, o), this);
    }
    quadraticCurveTo(t, e, s, i, r) {
      this._tick++;
      const o = this._transform;
      return this._activePath.quadraticCurveTo(o.a * t + o.c * e + o.tx, o.b * t + o.d * e + o.ty, o.a * s + o.c * i + o.tx, o.b * s + o.d * i + o.ty, r), this;
    }
    rect(t, e, s, i) {
      return this._tick++, this._activePath.rect(t, e, s, i, this._transform.clone()), this;
    }
    roundRect(t, e, s, i, r) {
      return this._tick++, this._activePath.roundRect(t, e, s, i, r, this._transform.clone()), this;
    }
    poly(t, e) {
      return this._tick++, this._activePath.poly(t, e, this._transform.clone()), this;
    }
    regularPoly(t, e, s, i, r = 0, o) {
      return this._tick++, this._activePath.regularPoly(t, e, s, i, r, o), this;
    }
    roundPoly(t, e, s, i, r, o) {
      return this._tick++, this._activePath.roundPoly(t, e, s, i, r, o), this;
    }
    roundShape(t, e, s, i) {
      return this._tick++, this._activePath.roundShape(t, e, s, i), this;
    }
    filletRect(t, e, s, i, r) {
      return this._tick++, this._activePath.filletRect(t, e, s, i, r), this;
    }
    chamferRect(t, e, s, i, r, o) {
      return this._tick++, this._activePath.chamferRect(t, e, s, i, r, o), this;
    }
    star(t, e, s, i, r = 0, o = 0) {
      return this._tick++, this._activePath.star(t, e, s, i, r, o, this._transform.clone()), this;
    }
    svg(t) {
      return this._tick++, dg(t, this), this;
    }
    restore() {
      const t = this._stateStack.pop();
      return t && (this._transform = t.transform, this._fillStyle = t.fillStyle, this._strokeStyle = t.strokeStyle), this;
    }
    save() {
      return this._stateStack.push({
        transform: this._transform.clone(),
        fillStyle: {
          ...this._fillStyle
        },
        strokeStyle: {
          ...this._strokeStyle
        }
      }), this;
    }
    getTransform() {
      return this._transform;
    }
    resetTransform() {
      return this._transform.identity(), this;
    }
    rotate(t) {
      return this._transform.rotate(t), this;
    }
    scale(t, e = t) {
      return this._transform.scale(t, e), this;
    }
    setTransform(t, e, s, i, r, o) {
      return t instanceof _t ? (this._transform.set(t.a, t.b, t.c, t.d, t.tx, t.ty), this) : (this._transform.set(t, e, s, i, r, o), this);
    }
    transform(t, e, s, i, r, o) {
      return t instanceof _t ? (this._transform.append(t), this) : (xl.set(t, e, s, i, r, o), this._transform.append(xl), this);
    }
    translate(t, e = t) {
      return this._transform.translate(t, e), this;
    }
    clear() {
      return this._activePath.clear(), this.instructions.length = 0, this.resetTransform(), this.onUpdate(), this;
    }
    onUpdate() {
      this._boundsDirty = true, this.dirty = true, this.emit("update", this, 16);
    }
    get bounds() {
      if (!this._boundsDirty) return this._bounds;
      this._boundsDirty = false;
      const t = this._bounds;
      t.clear();
      for (let e = 0; e < this.instructions.length; e++) {
        const s = this.instructions[e], i = s.action;
        if (i === "fill") {
          const r = s.data;
          t.addBounds(r.path.bounds);
        } else if (i === "texture") {
          const r = s.data;
          t.addFrame(r.dx, r.dy, r.dx + r.dw, r.dy + r.dh, r.transform);
        }
        if (i === "stroke") {
          const r = s.data, o = r.style.alignment;
          let a = r.style.width * (1 - o);
          r.style.join === "miter" && (a *= yg(r.path, r.style.miterLimit));
          const l = r.path.bounds;
          t.addFrame(l.minX - a, l.minY - a, l.maxX + a, l.maxY + a);
        }
      }
      return t.isValid || t.set(0, 0, 0, 0), t;
    }
    containsPoint(t) {
      var _a2;
      if (!this.bounds.containsPoint(t.x, t.y)) return false;
      const e = this.instructions;
      let s = false;
      for (let i = 0; i < e.length; i++) {
        const r = e[i], o = r.data, a = o.path;
        if (!r.action || !a) continue;
        const l = o.style, c = a.shapePath.shapePrimitives;
        for (let h = 0; h < c.length; h++) {
          const d = c[h].shape;
          if (!l || !d) continue;
          const u = c[h].transform, p = u ? u.applyInverse(t, xg) : t;
          if (r.action === "fill") s = d.contains(p.x, p.y);
          else {
            const g = l;
            s = d.strokeContains(p.x, p.y, g.width, g.alignment);
          }
          const f = o.hole;
          if (f) {
            const g = (_a2 = f.shapePath) == null ? void 0 : _a2.shapePrimitives;
            if (g) for (let m = 0; m < g.length; m++) g[m].shape.contains(p.x, p.y) && (s = false);
          }
          if (s) return true;
        }
      }
      return s;
    }
    unload() {
      var _a2;
      this.emit("unload", this);
      for (const t in this._gpuData) (_a2 = this._gpuData[t]) == null ? void 0 : _a2.destroy();
      this._gpuData = /* @__PURE__ */ Object.create(null);
    }
    destroy(t = false) {
      if (this.destroyed) return;
      if (this.destroyed = true, this._stateStack.length = 0, this._transform = null, this.unload(), this.emit("destroy", this), this.removeAllListeners(), typeof t == "boolean" ? t : t == null ? void 0 : t.texture) {
        const s = typeof t == "boolean" ? t : t == null ? void 0 : t.textureSource;
        this._fillStyle.texture && (this._fillStyle.fill && "uid" in this._fillStyle.fill ? this._fillStyle.fill.destroy() : this._fillStyle.texture.destroy(s)), this._strokeStyle.texture && (this._strokeStyle.fill && "uid" in this._strokeStyle.fill ? this._strokeStyle.fill.destroy() : this._strokeStyle.texture.destroy(s));
      }
      this._fillStyle = null, this._strokeStyle = null, this.instructions = null, this._activePath = null, this._bounds = null, this._stateStack = null, this.customShader = null, this._transform = null;
    }
  };
  Vo.defaultFillStyle = {
    color: 16777215,
    alpha: 1,
    texture: Ct.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local"
  };
  Vo.defaultStrokeStyle = {
    width: 1,
    color: 16777215,
    alpha: 1,
    alignment: 0.5,
    miterLimit: 10,
    cap: "butt",
    join: "miter",
    texture: Ct.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local",
    pixelLine: false
  };
  let Ae = Vo;
  function Yo(n, t = 1) {
    var _a2;
    const e = (_a2 = ys.RETINA_PREFIX) == null ? void 0 : _a2.exec(n);
    return e ? parseFloat(e[1]) : t;
  }
  function Xo(n, t, e) {
    n.label = e, n._sourceOrigin = e;
    const s = new Ct({
      source: n,
      label: e
    }), i = () => {
      delete t.promiseCache[e], te.has(e) && te.remove(e);
    };
    return s.source.once("destroy", () => {
      t.promiseCache[e] && (Vt("[Assets] A TextureSource managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the TextureSource."), i());
    }), s.once("destroy", () => {
      n.destroyed || (Vt("[Assets] A Texture managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the Texture."), i());
    }), s;
  }
  const bg = ".svg", _g = "image/svg+xml", wg = {
    extension: {
      type: it.LoadParser,
      priority: kn.Low,
      name: "loadSVG"
    },
    name: "loadSVG",
    id: "svg",
    config: {
      crossOrigin: "anonymous",
      parseAsGraphicsContext: false
    },
    test(n) {
      return xs(n, _g) || bs(n, bg);
    },
    async load(n, t, e) {
      var _a2;
      return ((_a2 = t.data) == null ? void 0 : _a2.parseAsGraphicsContext) ?? this.config.parseAsGraphicsContext ? Cg(n) : vg(n, t, e, this.config.crossOrigin);
    },
    unload(n) {
      n.destroy(true);
    }
  };
  async function vg(n, t, e, s) {
    var _a2, _b2, _c2;
    const i = await Mt.get().fetch(n), r = Mt.get().createImage();
    r.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(await i.text())}`, r.crossOrigin = s, await r.decode();
    const o = ((_a2 = t.data) == null ? void 0 : _a2.width) ?? r.width, a = ((_b2 = t.data) == null ? void 0 : _b2.height) ?? r.height, l = ((_c2 = t.data) == null ? void 0 : _c2.resolution) || Yo(n), c = Math.ceil(o * l), h = Math.ceil(a * l), d = Mt.get().createCanvas(c, h), u = d.getContext("2d");
    u.imageSmoothingEnabled = true, u.imageSmoothingQuality = "high", u.drawImage(r, 0, 0, o * l, a * l);
    const { parseAsGraphicsContext: p, ...f } = t.data ?? {}, g = new fs({
      resource: d,
      alphaMode: "premultiply-alpha-on-upload",
      resolution: l,
      ...f
    });
    return Xo(g, e, n);
  }
  async function Cg(n) {
    const e = await (await Mt.get().fetch(n)).text(), s = new Ae();
    return s.svg(e), s;
  }
  const Sg = `(function () {
    'use strict';

    const WHITE_PNG = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=";
    async function checkImageBitmap() {
      try {
        if (typeof createImageBitmap !== "function") return false;
        const response = await fetch(WHITE_PNG);
        const imageBlob = await response.blob();
        const imageBitmap = await createImageBitmap(imageBlob);
        return imageBitmap.width === 1 && imageBitmap.height === 1;
      } catch (_e) {
        return false;
      }
    }
    void checkImageBitmap().then((result) => {
      self.postMessage(result);
    });

})();
`;
  let cs = null, mo = class {
    constructor() {
      cs || (cs = URL.createObjectURL(new Blob([
        Sg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(cs);
    }
  };
  mo.revokeObjectURL = function() {
    cs && (URL.revokeObjectURL(cs), cs = null);
  };
  const kg = `(function () {
    'use strict';

    async function loadImageBitmap(url, alphaMode) {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(\`[WorkerManager.loadImageBitmap] Failed to fetch \${url}: \${response.status} \${response.statusText}\`);
      }
      const imageBlob = await response.blob();
      return alphaMode === "premultiplied-alpha" ? createImageBitmap(imageBlob, { premultiplyAlpha: "none" }) : createImageBitmap(imageBlob);
    }
    self.onmessage = async (event) => {
      try {
        const imageBitmap = await loadImageBitmap(event.data.data[0], event.data.data[1]);
        self.postMessage({
          data: imageBitmap,
          uuid: event.data.uuid,
          id: event.data.id
        }, [imageBitmap]);
      } catch (e) {
        self.postMessage({
          error: e,
          uuid: event.data.uuid,
          id: event.data.id
        });
      }
    };

})();
`;
  let hs = null;
  class Oh {
    constructor() {
      hs || (hs = URL.createObjectURL(new Blob([
        kg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(hs);
    }
  }
  Oh.revokeObjectURL = function() {
    hs && (URL.revokeObjectURL(hs), hs = null);
  };
  let bl = 0, Lr;
  class Tg {
    constructor() {
      this._initialized = false, this._createdWorkers = 0, this._workerPool = [], this._queue = [], this._resolveHash = {};
    }
    isImageBitmapSupported() {
      return this._isImageBitmapSupported !== void 0 ? this._isImageBitmapSupported : (this._isImageBitmapSupported = new Promise((t) => {
        const { worker: e } = new mo();
        e.addEventListener("message", (s) => {
          e.terminate(), mo.revokeObjectURL(), t(s.data);
        });
      }), this._isImageBitmapSupported);
    }
    loadImageBitmap(t, e) {
      var _a2;
      return this._run("loadImageBitmap", [
        t,
        (_a2 = e == null ? void 0 : e.data) == null ? void 0 : _a2.alphaMode
      ]);
    }
    async _initWorkers() {
      this._initialized || (this._initialized = true);
    }
    _getWorker() {
      Lr === void 0 && (Lr = navigator.hardwareConcurrency || 4);
      let t = this._workerPool.pop();
      return !t && this._createdWorkers < Lr && (this._createdWorkers++, t = new Oh().worker, t.addEventListener("message", (e) => {
        this._complete(e.data), this._returnWorker(e.target), this._next();
      })), t;
    }
    _returnWorker(t) {
      this._workerPool.push(t);
    }
    _complete(t) {
      this._resolveHash[t.uuid] && (t.error !== void 0 ? this._resolveHash[t.uuid].reject(t.error) : this._resolveHash[t.uuid].resolve(t.data), delete this._resolveHash[t.uuid]);
    }
    async _run(t, e) {
      await this._initWorkers();
      const s = new Promise((i, r) => {
        this._queue.push({
          id: t,
          arguments: e,
          resolve: i,
          reject: r
        });
      });
      return this._next(), s;
    }
    _next() {
      if (!this._queue.length) return;
      const t = this._getWorker();
      if (!t) return;
      const e = this._queue.pop(), s = e.id;
      this._resolveHash[bl] = {
        resolve: e.resolve,
        reject: e.reject
      }, t.postMessage({
        data: e.arguments,
        uuid: bl++,
        id: s
      });
    }
    reset() {
      this._workerPool.forEach((t) => t.terminate()), this._workerPool.length = 0, Object.values(this._resolveHash).forEach(({ reject: t }) => {
        t == null ? void 0 : t(new Error("WorkerManager has been reset before completion"));
      }), this._resolveHash = {}, this._queue.length = 0, this._initialized = false, this._createdWorkers = 0;
    }
  }
  const _l = new Tg(), Ag = [
    ".jpeg",
    ".jpg",
    ".png",
    ".webp",
    ".avif"
  ], Eg = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/avif"
  ];
  async function Mg(n, t) {
    var _a2;
    const e = await Mt.get().fetch(n);
    if (!e.ok) throw new Error(`[loadImageBitmap] Failed to fetch ${n}: ${e.status} ${e.statusText}`);
    const s = await e.blob();
    return ((_a2 = t == null ? void 0 : t.data) == null ? void 0 : _a2.alphaMode) === "premultiplied-alpha" ? createImageBitmap(s, {
      premultiplyAlpha: "none"
    }) : createImageBitmap(s);
  }
  const Fh = {
    name: "loadTextures",
    id: "texture",
    extension: {
      type: it.LoadParser,
      priority: kn.High,
      name: "loadTextures"
    },
    config: {
      preferWorkers: true,
      preferCreateImageBitmap: true,
      crossOrigin: "anonymous"
    },
    test(n) {
      return xs(n, Eg) || bs(n, Ag);
    },
    async load(n, t, e) {
      var _a2;
      let s = null;
      globalThis.createImageBitmap && this.config.preferCreateImageBitmap ? this.config.preferWorkers && await _l.isImageBitmapSupported() ? s = await _l.loadImageBitmap(n, t) : s = await Mg(n, t) : s = await new Promise((r, o) => {
        s = Mt.get().createImage(), s.crossOrigin = this.config.crossOrigin, s.src = n, s.complete ? r(s) : (s.onload = () => {
          r(s);
        }, s.onerror = o);
      });
      const i = new fs({
        resource: s,
        alphaMode: "premultiply-alpha-on-upload",
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || Yo(n),
        ...t.data
      });
      return Xo(i, e, n);
    },
    unload(n) {
      n.destroy(true);
    }
  }, Pg = [
    ".mp4",
    ".m4v",
    ".webm",
    ".ogg",
    ".ogv",
    ".h264",
    ".avi",
    ".mov"
  ];
  let $r, Br;
  function Ig(n, t, e) {
    e === void 0 && !t.startsWith("data:") ? n.crossOrigin = Lg(t) : e !== false && (n.crossOrigin = typeof e == "string" ? e : "anonymous");
  }
  function Rg(n) {
    return new Promise((t, e) => {
      n.addEventListener("canplaythrough", s), n.addEventListener("error", i), n.load();
      function s() {
        r(), t();
      }
      function i(o) {
        r(), e(o);
      }
      function r() {
        n.removeEventListener("canplaythrough", s), n.removeEventListener("error", i);
      }
    });
  }
  function Lg(n, t = globalThis.location) {
    if (n.startsWith("data:")) return "";
    t || (t = globalThis.location);
    const e = new URL(n, document.baseURI);
    return e.hostname !== t.hostname || e.port !== t.port || e.protocol !== t.protocol ? "anonymous" : "";
  }
  function $g() {
    const n = [], t = [];
    for (const e of Pg) {
      const s = Fs.MIME_TYPES[e.substring(1)] || `video/${e.substring(1)}`;
      qi(s) && (n.push(e), t.includes(s) || t.push(s));
    }
    return {
      validVideoExtensions: n,
      validVideoMime: t
    };
  }
  const Bg = {
    name: "loadVideo",
    id: "video",
    extension: {
      type: it.LoadParser,
      name: "loadVideo"
    },
    test(n) {
      if (!$r || !Br) {
        const { validVideoExtensions: s, validVideoMime: i } = $g();
        $r = s, Br = i;
      }
      const t = xs(n, Br), e = bs(n, $r);
      return t || e;
    },
    async load(n, t, e) {
      var _a2, _b2;
      const s = {
        ...Fs.defaultOptions,
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || Yo(n),
        alphaMode: ((_b2 = t.data) == null ? void 0 : _b2.alphaMode) || await Yc(),
        ...t.data
      }, i = document.createElement("video"), r = {
        preload: s.autoLoad !== false ? "auto" : void 0,
        "webkit-playsinline": s.playsinline !== false ? "" : void 0,
        playsinline: s.playsinline !== false ? "" : void 0,
        muted: s.muted === true ? "" : void 0,
        loop: s.loop === true ? "" : void 0,
        autoplay: s.autoPlay !== false ? "" : void 0
      };
      Object.keys(r).forEach((l) => {
        const c = r[l];
        c !== void 0 && i.setAttribute(l, c);
      }), s.muted === true && (i.muted = true), Ig(i, n, s.crossorigin);
      const o = document.createElement("source");
      let a;
      if (s.mime) a = s.mime;
      else if (n.startsWith("data:")) a = n.slice(5, n.indexOf(";"));
      else if (!n.startsWith("blob:")) {
        const l = n.split("?")[0].slice(n.lastIndexOf(".") + 1).toLowerCase();
        a = Fs.MIME_TYPES[l] || `video/${l}`;
      }
      return o.src = n, a && (o.type = a), new Promise((l, c) => {
        s.preload && !s.autoPlay && i.load(), i.addEventListener("canplay", h), i.addEventListener("error", d), o.addEventListener("error", d), i.appendChild(o);
        async function h() {
          const p = new Fs({
            ...s,
            resource: i
          });
          u(), t.data.preload && await Rg(i), l(Xo(p, e, n));
        }
        function d(p) {
          u(), c(p);
        }
        function u() {
          i.removeEventListener("canplay", h), i.removeEventListener("error", d), o.removeEventListener("error", d);
        }
      });
    },
    unload(n) {
      n.destroy(true);
    }
  }, Nh = {
    extension: {
      type: it.ResolveParser,
      name: "resolveTexture"
    },
    test: Fh.test,
    parse: (n) => {
      var _a2;
      return {
        resolution: parseFloat(((_a2 = ys.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
        format: n.split(".").pop(),
        src: n
      };
    }
  }, Og = {
    extension: {
      type: it.ResolveParser,
      priority: -2,
      name: "resolveJson"
    },
    test: (n) => ys.RETINA_PREFIX.test(n) && n.endsWith(".json"),
    parse: Nh.parse
  };
  class Fg {
    constructor() {
      this._detections = [], this._initialized = false, this.resolver = new ys(), this.loader = new cf(), this.cache = te, this._backgroundLoader = new Qp(this.loader), this._backgroundLoader.active = true, this.reset();
    }
    async init(t = {}) {
      var _a2, _b2;
      if (this._initialized) {
        Vt("[Assets]AssetManager already initialized, did you load before calling this Assets.init()?");
        return;
      }
      if (this._initialized = true, t.defaultSearchParams && this.resolver.setDefaultSearchParams(t.defaultSearchParams), t.basePath && (this.resolver.basePath = t.basePath), t.bundleIdentifier && this.resolver.setBundleIdentifier(t.bundleIdentifier), t.manifest) {
        let r = t.manifest;
        typeof r == "string" && (r = await this.load(r)), this.resolver.addManifest(r);
      }
      const e = ((_a2 = t.texturePreference) == null ? void 0 : _a2.resolution) ?? 1, s = typeof e == "number" ? [
        e
      ] : e, i = await this._detectFormats({
        preferredFormats: (_b2 = t.texturePreference) == null ? void 0 : _b2.format,
        skipDetections: t.skipDetections,
        detections: this._detections
      });
      this.resolver.prefer({
        params: {
          format: i,
          resolution: s
        }
      }), t.preferences && this.setPreferences(t.preferences), t.loadOptions && (this.loader.loadOptions = {
        ...this.loader.loadOptions,
        ...t.loadOptions
      });
    }
    add(t) {
      this.resolver.add(t);
    }
    async load(t, e) {
      this._initialized || await this.init();
      const s = $i(t), i = We(t).map((a) => {
        if (typeof a != "string") {
          const l = this.resolver.getAlias(a);
          return l.some((c) => !this.resolver.hasKey(c)) && this.add(a), Array.isArray(l) ? l[0] : l;
        }
        return this.resolver.hasKey(a) || this.add({
          alias: a,
          src: a
        }), a;
      }), r = this.resolver.resolve(i), o = await this._mapLoadToResolve(r, e);
      return s ? o[i[0]] : o;
    }
    addBundle(t, e) {
      this.resolver.addBundle(t, e);
    }
    async loadBundle(t, e) {
      this._initialized || await this.init();
      let s = false;
      typeof t == "string" && (s = true, t = [
        t
      ]);
      const i = this.resolver.resolveBundle(t), r = {}, o = Object.keys(i);
      let a = 0;
      const l = [], c = () => {
        e == null ? void 0 : e(l.reduce((d, u) => d + u, 0) / a);
      }, h = o.map((d, u) => {
        const p = i[d], f = Object.values(p), m = [
          ...new Set(f.flat())
        ].reduce((y, b) => y + (b.progressSize || 1), 0);
        return l.push(0), a += m, this._mapLoadToResolve(p, (y) => {
          l[u] = y * m, c();
        }).then((y) => {
          r[d] = y;
        });
      });
      return await Promise.all(h), s ? r[t[0]] : r;
    }
    async backgroundLoad(t) {
      this._initialized || await this.init(), typeof t == "string" && (t = [
        t
      ]);
      const e = this.resolver.resolve(t);
      this._backgroundLoader.add(Object.values(e));
    }
    async backgroundLoadBundle(t) {
      this._initialized || await this.init(), typeof t == "string" && (t = [
        t
      ]);
      const e = this.resolver.resolveBundle(t);
      Object.values(e).forEach((s) => {
        this._backgroundLoader.add(Object.values(s));
      });
    }
    reset() {
      this.resolver.reset(), this.loader.reset(), this.cache.reset(), this._initialized = false;
    }
    get(t) {
      if (typeof t == "string") return te.get(t);
      const e = {};
      for (let s = 0; s < t.length; s++) e[s] = te.get(t[s]);
      return e;
    }
    async _mapLoadToResolve(t, e) {
      const s = [
        ...new Set(Object.values(t))
      ];
      this._backgroundLoader.active = false;
      const i = await this.loader.load(s, e);
      this._backgroundLoader.active = true;
      const r = {};
      return s.forEach((o) => {
        const a = i[o.src], l = [
          o.src
        ];
        o.alias && l.push(...o.alias), l.forEach((c) => {
          r[c] = a;
        }), te.set(l, a);
      }), r;
    }
    async unload(t) {
      this._initialized || await this.init();
      const e = We(t).map((i) => typeof i != "string" ? i.src : i), s = this.resolver.resolve(e);
      await this._unloadFromResolved(s);
    }
    async unloadBundle(t) {
      this._initialized || await this.init(), t = We(t);
      const e = this.resolver.resolveBundle(t), s = Object.keys(e).map((i) => this._unloadFromResolved(e[i]));
      await Promise.all(s);
    }
    async _unloadFromResolved(t) {
      const e = Object.values(t);
      e.forEach((s) => {
        te.remove(s.src);
      }), await this.loader.unload(e);
    }
    async _detectFormats(t) {
      let e = [];
      t.preferredFormats && (e = Array.isArray(t.preferredFormats) ? t.preferredFormats : [
        t.preferredFormats
      ]);
      for (const s of t.detections) t.skipDetections || await s.test() ? e = await s.add(e) : t.skipDetections || (e = await s.remove(e));
      return e = e.filter((s, i) => e.indexOf(s) === i), e;
    }
    get detections() {
      return this._detections;
    }
    setPreferences(t) {
      this.loader.parsers.forEach((e) => {
        e.config && Object.keys(e.config).filter((s) => s in t).forEach((s) => {
          e.config[s] = t[s];
        });
      });
    }
  }
  const ze = new Fg();
  Ot.handleByList(it.LoadParser, ze.loader.parsers).handleByList(it.ResolveParser, ze.resolver.parsers).handleByList(it.CacheParser, ze.cache.parsers).handleByList(it.DetectionParser, ze.detections);
  Ot.add(tf, nf, ef, lf, rf, of, af, uf, mf, Cf, wg, Fh, Bg, Zp, Jp, Nh, Og);
  const wl = {
    loader: it.LoadParser,
    resolver: it.ResolveParser,
    cache: it.CacheParser,
    detection: it.DetectionParser
  };
  Ot.handle(it.Asset, (n) => {
    const t = n.ref;
    Object.entries(wl).filter(([e]) => !!t[e]).forEach(([e, s]) => Ot.add(Object.assign(t[e], {
      extension: t[e].extension ?? s
    })));
  }, (n) => {
    const t = n.ref;
    Object.keys(wl).filter((e) => !!t[e]).forEach((e) => Ot.remove(t[e]));
  });
  class Ng {
    constructor() {
      this.isBatchable = false;
    }
    reset() {
      this.isBatchable = false, this.context = null, this.graphicsData && (this.graphicsData.destroy(), this.graphicsData = null);
    }
    destroy() {
      this.reset();
    }
  }
  class Wg {
    constructor() {
      this.instructions = new $o();
    }
    init() {
      this.instructions.reset();
    }
    destroy() {
      this.instructions.destroy(), this.instructions = null;
    }
  }
  const qo = class go {
    constructor(t) {
      this._renderer = t, this._managedContexts = new _s({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      go.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? go.defaultOptions.bezierSmoothness;
    }
    getContextRenderData(t) {
      return this.getGpuContext(t).graphicsData || this._initContextRenderData(t);
    }
    updateGpuContext(t) {
      const e = t._gpuData, s = !!e[this._renderer.uid], i = e[this._renderer.uid] || this._initContext(t);
      return (t.dirty || !s) && (s && i.reset(), i.isBatchable = false, t.dirty = false), i;
    }
    getGpuContext(t) {
      return t._gpuData[this._renderer.uid] || this._initContext(t);
    }
    _initContextRenderData(t) {
      const e = new Wg(), s = this.getGpuContext(t);
      return s.graphicsData = e, e.init(), e;
    }
    _initContext(t) {
      const e = new Ng();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  qo.extension = {
    type: [
      it.CanvasSystem
    ],
    name: "graphicsContext"
  };
  qo.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let Gg = qo;
  class Wh {
    constructor(t, e) {
      this.state = Oi.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new _s({
        renderer: t,
        type: "renderable",
        priority: -1,
        name: "graphics"
      });
    }
    contextChange() {
      this._adaptor.contextChange(this.renderer);
    }
    validateRenderable(t) {
      return false;
    }
    addRenderable(t, e) {
      this._managedGraphics.add(t), this.renderer.renderPipes.batch.break(e), e.add(t);
    }
    updateRenderable(t) {
    }
    execute(t) {
      t.isRenderable && this._adaptor.execute(this, t);
    }
    destroy() {
      this._managedGraphics.destroy(), this.renderer = null, this._adaptor.destroy(), this._adaptor = null;
    }
  }
  Wh.extension = {
    type: [
      it.CanvasPipes
    ],
    name: "graphics"
  };
  Gh = function(n, t, e) {
    const s = (n >> 24 & 255) / 255;
    t[e++] = (n & 255) / 255 * s, t[e++] = (n >> 8 & 255) / 255 * s, t[e++] = (n >> 16 & 255) / 255 * s, t[e++] = s;
  };
  class zg {
    constructor() {
      this.batches = [], this.batched = false;
    }
    destroy() {
      this.batches.forEach((t) => {
        _e.return(t);
      }), this.batches.length = 0;
    }
  }
  class zh {
    constructor(t, e) {
      this.state = Oi.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new _s({
        renderer: t,
        type: "renderable",
        priority: -1,
        name: "graphics"
      });
    }
    contextChange() {
      this._adaptor.contextChange(this.renderer);
    }
    validateRenderable(t) {
      const e = t.context, s = !!t._gpuData, r = this.renderer.graphicsContext.updateGpuContext(e);
      return !!(r.isBatchable || s !== r.isBatchable);
    }
    addRenderable(t, e) {
      const i = this.renderer.graphicsContext.updateGpuContext(t.context);
      t.didViewUpdate && this._rebuild(t), i.isBatchable ? this._addToBatcher(t, e) : (this.renderer.renderPipes.batch.break(e), e.add(t));
    }
    updateRenderable(t) {
      const s = this._getGpuDataForRenderable(t).batches;
      for (let i = 0; i < s.length; i++) {
        const r = s[i];
        r._batcher.updateElement(r);
      }
    }
    execute(t) {
      if (!t.isRenderable) return;
      const e = this.renderer, s = t.context;
      if (!e.graphicsContext.getGpuContext(s).batches.length) return;
      const r = s.customShader || this._adaptor.shader;
      this.state.blendMode = t.groupBlendMode;
      const o = r.resources.localUniforms.uniforms;
      o.uTransformMatrix = t.groupTransform, o.uRound = e._roundPixels | t._roundPixels, Gh(t.groupColorAlpha, o.uColor, 0), this._adaptor.execute(this, t);
    }
    _rebuild(t) {
      const e = this._getGpuDataForRenderable(t), i = this.renderer.graphicsContext.updateGpuContext(t.context);
      e.destroy(), i.isBatchable && this._updateBatchesForRenderable(t, e);
    }
    _addToBatcher(t, e) {
      const s = this.renderer.renderPipes.batch, i = this._getGpuDataForRenderable(t).batches;
      for (let r = 0; r < i.length; r++) {
        const o = i[r];
        s.addToBatch(o, e);
      }
    }
    _getGpuDataForRenderable(t) {
      return t._gpuData[this.renderer.uid] || this._initGpuDataForRenderable(t);
    }
    _initGpuDataForRenderable(t) {
      const e = new zg();
      return t._gpuData[this.renderer.uid] = e, this._managedGraphics.add(t), e;
    }
    _updateBatchesForRenderable(t, e) {
      const s = t.context, r = this.renderer.graphicsContext.getGpuContext(s), o = this.renderer._roundPixels | t._roundPixels;
      e.batches = r.batches.map((a) => {
        const l = _e.get(Do);
        return a.copyTo(l), l.renderable = t, l.roundPixels = o, l;
      });
    }
    destroy() {
      this._managedGraphics.destroy(), this.renderer = null, this._adaptor.destroy(), this._adaptor = null, this.state = null;
    }
  }
  zh.extension = {
    type: [
      it.WebGLPipes,
      it.WebGPUPipes
    ],
    name: "graphics"
  };
  Ot.add(Wh);
  Ot.add(zh);
  Ot.add(Gg);
  Ot.add(jo);
  ut = class extends Vi {
    constructor(t) {
      t instanceof Ae && (t = {
        context: t
      });
      const { context: e, roundPixels: s, ...i } = t || {};
      super({
        label: "Graphics",
        ...i
      }), this.renderPipeId = "graphics", e ? this.context = e : (this.context = this._ownedContext = new Ae(), this.context.autoGarbageCollect = this.autoGarbageCollect), this.didViewUpdate = true, this.allowChildren = false, this.roundPixels = s ?? false;
    }
    set context(t) {
      t !== this._context && (this._context && (this._context.off("update", this.onViewUpdate, this), this._context.off("unload", this.unload, this)), this._context = t, this._context.on("update", this.onViewUpdate, this), this._context.on("unload", this.unload, this), this.onViewUpdate());
    }
    get context() {
      return this._context;
    }
    get bounds() {
      return this._context.bounds;
    }
    updateBounds() {
    }
    containsPoint(t) {
      return this._context.containsPoint(t);
    }
    destroy(t) {
      this._ownedContext && !t ? this._ownedContext.destroy(t) : (t === true || (t == null ? void 0 : t.context) === true) && this._context.destroy(t), this._ownedContext = null, this._context = null, super.destroy(t);
    }
    _onTouch(t) {
      this._gcLastUsed = t, this._context._gcLastUsed = t;
    }
    _callContextMethod(t, e) {
      return this.context[t](...e), this;
    }
    setFillStyle(...t) {
      return this._callContextMethod("setFillStyle", t);
    }
    setStrokeStyle(...t) {
      return this._callContextMethod("setStrokeStyle", t);
    }
    fill(...t) {
      return this._callContextMethod("fill", t);
    }
    stroke(...t) {
      return this._callContextMethod("stroke", t);
    }
    texture(...t) {
      return this._callContextMethod("texture", t);
    }
    beginPath() {
      return this._callContextMethod("beginPath", []);
    }
    cut() {
      return this._callContextMethod("cut", []);
    }
    arc(...t) {
      return this._callContextMethod("arc", t);
    }
    arcTo(...t) {
      return this._callContextMethod("arcTo", t);
    }
    arcToSvg(...t) {
      return this._callContextMethod("arcToSvg", t);
    }
    bezierCurveTo(...t) {
      return this._callContextMethod("bezierCurveTo", t);
    }
    closePath() {
      return this._callContextMethod("closePath", []);
    }
    ellipse(...t) {
      return this._callContextMethod("ellipse", t);
    }
    circle(...t) {
      return this._callContextMethod("circle", t);
    }
    path(...t) {
      return this._callContextMethod("path", t);
    }
    lineTo(...t) {
      return this._callContextMethod("lineTo", t);
    }
    moveTo(...t) {
      return this._callContextMethod("moveTo", t);
    }
    quadraticCurveTo(...t) {
      return this._callContextMethod("quadraticCurveTo", t);
    }
    rect(...t) {
      return this._callContextMethod("rect", t);
    }
    roundRect(...t) {
      return this._callContextMethod("roundRect", t);
    }
    poly(...t) {
      return this._callContextMethod("poly", t);
    }
    regularPoly(...t) {
      return this._callContextMethod("regularPoly", t);
    }
    roundPoly(...t) {
      return this._callContextMethod("roundPoly", t);
    }
    roundShape(...t) {
      return this._callContextMethod("roundShape", t);
    }
    filletRect(...t) {
      return this._callContextMethod("filletRect", t);
    }
    chamferRect(...t) {
      return this._callContextMethod("chamferRect", t);
    }
    star(...t) {
      return this._callContextMethod("star", t);
    }
    svg(...t) {
      return this._callContextMethod("svg", t);
    }
    restore(...t) {
      return this._callContextMethod("restore", t);
    }
    save() {
      return this._callContextMethod("save", []);
    }
    getTransform() {
      return this.context.getTransform();
    }
    resetTransform() {
      return this._callContextMethod("resetTransform", []);
    }
    rotateTransform(...t) {
      return this._callContextMethod("rotate", t);
    }
    scaleTransform(...t) {
      return this._callContextMethod("scale", t);
    }
    setTransform(...t) {
      return this._callContextMethod("setTransform", t);
    }
    transform(...t) {
      return this._callContextMethod("transform", t);
    }
    translateTransform(...t) {
      return this._callContextMethod("translate", t);
    }
    clear() {
      return this._callContextMethod("clear", []);
    }
    get fillStyle() {
      return this._context.fillStyle;
    }
    set fillStyle(t) {
      this._context.fillStyle = t;
    }
    get strokeStyle() {
      return this._context.strokeStyle;
    }
    set strokeStyle(t) {
      this._context.strokeStyle = t;
    }
    clone(t = false) {
      return t ? new ut(this._context.clone()) : (this._ownedContext = null, new ut(this._context));
    }
    lineStyle(t, e, s) {
      Et(qt, "Graphics#lineStyle is no longer needed. Use Graphics#setStrokeStyle to set the stroke style.");
      const i = {};
      return t && (i.width = t), e && (i.color = e), s && (i.alpha = s), this.context.strokeStyle = i, this;
    }
    beginFill(t, e) {
      Et(qt, "Graphics#beginFill is no longer needed. Use Graphics#fill to fill the shape with the desired style.");
      const s = {};
      return t !== void 0 && (s.color = t), e !== void 0 && (s.alpha = e), this.context.fillStyle = s, this;
    }
    endFill() {
      Et(qt, "Graphics#endFill is no longer needed. Use Graphics#fill to fill the shape with the desired style."), this.context.fill();
      const t = this.context.strokeStyle;
      return (t.width !== Ae.defaultStrokeStyle.width || t.color !== Ae.defaultStrokeStyle.color || t.alpha !== Ae.defaultStrokeStyle.alpha) && this.context.stroke(), this;
    }
    drawCircle(...t) {
      return Et(qt, "Graphics#drawCircle has been renamed to Graphics#circle"), this._callContextMethod("circle", t);
    }
    drawEllipse(...t) {
      return Et(qt, "Graphics#drawEllipse has been renamed to Graphics#ellipse"), this._callContextMethod("ellipse", t);
    }
    drawPolygon(...t) {
      return Et(qt, "Graphics#drawPolygon has been renamed to Graphics#poly"), this._callContextMethod("poly", t);
    }
    drawRect(...t) {
      return Et(qt, "Graphics#drawRect has been renamed to Graphics#rect"), this._callContextMethod("rect", t);
    }
    drawRoundedRect(...t) {
      return Et(qt, "Graphics#drawRoundedRect has been renamed to Graphics#roundRect"), this._callContextMethod("roundRect", t);
    }
    drawStar(...t) {
      return Et(qt, "Graphics#drawStar has been renamed to Graphics#star"), this._callContextMethod("star", t);
    }
  };
  let Zn;
  function vl(n) {
    const t = Mt.get().createCanvas(6, 1), e = t.getContext("2d");
    return e.fillStyle = n, e.fillRect(0, 0, 6, 1), t;
  }
  Dg = function() {
    if (Zn !== void 0) return Zn;
    try {
      const n = vl("#ff00ff"), t = vl("#ffff00"), s = Mt.get().createCanvas(6, 1).getContext("2d");
      s.globalCompositeOperation = "multiply", s.drawImage(n, 0, 0), s.drawImage(t, 2, 0);
      const i = s.getImageData(2, 0, 1, 1);
      if (!i) Zn = false;
      else {
        const r = i.data;
        Zn = r[0] === 255 && r[1] === 0 && r[2] === 0;
      }
    } catch {
      Zn = false;
    }
    return Zn;
  };
  jt = {
    canvas: null,
    convertTintToImage: false,
    cacheStepsPerColorChannel: 8,
    canUseMultiply: Dg(),
    tintMethod: null,
    _canvasSourceCache: /* @__PURE__ */ new WeakMap(),
    _unpremultipliedCache: /* @__PURE__ */ new WeakMap(),
    getCanvasSource: (n) => {
      const t = n.source, e = t == null ? void 0 : t.resource;
      if (!e) return null;
      const s = t.alphaMode === "premultiplied-alpha", i = t.resourceWidth ?? t.pixelWidth, r = t.resourceHeight ?? t.pixelHeight, o = i !== t.pixelWidth || r !== t.pixelHeight;
      if (s) {
        if ((e instanceof HTMLCanvasElement || typeof OffscreenCanvas < "u" && e instanceof OffscreenCanvas) && !o) return e;
        const a = jt._unpremultipliedCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
      }
      if (e instanceof Uint8Array || e instanceof Uint8ClampedArray || e instanceof Int8Array || e instanceof Uint16Array || e instanceof Int16Array || e instanceof Uint32Array || e instanceof Int32Array || e instanceof Float32Array || e instanceof ArrayBuffer) {
        const a = jt._canvasSourceCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
        const l = Mt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d"), h = c.createImageData(t.pixelWidth, t.pixelHeight), d = h.data, u = e instanceof ArrayBuffer ? new Uint8Array(e) : new Uint8Array(e.buffer, e.byteOffset, e.byteLength);
        if (t.format === "bgra8unorm") for (let p = 0; p < d.length && p + 3 < u.length; p += 4) d[p] = u[p + 2], d[p + 1] = u[p + 1], d[p + 2] = u[p], d[p + 3] = u[p + 3];
        else d.set(u.subarray(0, d.length));
        return c.putImageData(h, 0, 0), jt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      if (s) {
        const a = Mt.get().createCanvas(t.pixelWidth, t.pixelHeight), l = a.getContext("2d", {
          willReadFrequently: true
        });
        a.width = t.pixelWidth, a.height = t.pixelHeight, l.drawImage(e, 0, 0);
        const c = l.getImageData(0, 0, a.width, a.height), h = c.data;
        for (let d = 0; d < h.length; d += 4) {
          const u = h[d + 3];
          if (u > 0) {
            const p = 255 / u;
            h[d] = Math.min(255, h[d] * p + 0.5), h[d + 1] = Math.min(255, h[d + 1] * p + 0.5), h[d + 2] = Math.min(255, h[d + 2] * p + 0.5);
          }
        }
        return l.putImageData(c, 0, 0), jt._unpremultipliedCache.set(t, {
          canvas: a,
          resourceId: t._resourceId
        }), a;
      }
      if (o) {
        const a = jt._canvasSourceCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
        const l = Mt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d");
        return l.width = t.pixelWidth, l.height = t.pixelHeight, c.drawImage(e, 0, 0), jt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      return e;
    },
    getTintedCanvas: (n, t) => {
      const e = n.texture, s = Gt.shared.setValue(t).toHex(), i = e.tintCache || (e.tintCache = {}), r = i[s], o = e.source._resourceId;
      if ((r == null ? void 0 : r.tintId) === o) return r;
      const a = r && "getContext" in r ? r : Mt.get().createCanvas();
      return jt.tintMethod(e, t, a), a.tintId = o, i[s] = a, i[s];
    },
    getTintedPattern: (n, t) => {
      const e = Gt.shared.setValue(t).toHex(), s = n.patternCache || (n.patternCache = {}), i = n.source._resourceId;
      let r = s[e];
      return (r == null ? void 0 : r.tintId) === i || (jt.canvas || (jt.canvas = Mt.get().createCanvas()), jt.tintMethod(n, t, jt.canvas), r = jt.canvas.getContext("2d").createPattern(jt.canvas, "repeat"), r.tintId = i, s[e] = r), r;
    },
    applyPatternTransform: (n, t, e = true) => {
      if (!t) return;
      const s = n;
      if (!s.setTransform) return;
      const i = globalThis.DOMMatrix;
      if (!i) return;
      const r = new i([
        t.a,
        t.b,
        t.c,
        t.d,
        t.tx,
        t.ty
      ]);
      s.setTransform(e ? r.inverse() : r);
    },
    tintWithMultiply: (n, t, e) => {
      const s = e.getContext("2d"), i = n.frame.clone(), r = n.source._resolution ?? n.source.resolution ?? 1, o = n.rotate;
      i.x *= r, i.y *= r, i.width *= r, i.height *= r;
      const a = St.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.fillStyle = Gt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "multiply";
      const h = jt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && jt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.globalCompositeOperation = "destination-atop", s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
    },
    tintWithOverlay: (n, t, e) => {
      const s = e.getContext("2d"), i = n.frame.clone(), r = n.source._resolution ?? n.source.resolution ?? 1, o = n.rotate;
      i.x *= r, i.y *= r, i.width *= r, i.height *= r;
      const a = St.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy", s.fillStyle = Gt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "destination-atop";
      const h = jt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && jt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
    },
    tintWithPerPixel: (n, t, e) => {
      const s = e.getContext("2d"), i = n.frame.clone(), r = n.source._resolution ?? n.source.resolution ?? 1, o = n.rotate;
      i.x *= r, i.y *= r, i.width *= r, i.height *= r;
      const a = St.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy";
      const h = jt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && jt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
      const d = t >> 16 & 255, u = t >> 8 & 255, p = t & 255, f = s.getImageData(0, 0, l, c), g = f.data;
      for (let m = 0; m < g.length; m += 4) g[m] = g[m] * d / 255, g[m + 1] = g[m + 1] * u / 255, g[m + 2] = g[m + 2] * p / 255;
      s.putImageData(f, 0, 0);
    },
    _applyInverseRotation: (n, t, e, s) => {
      const i = St.inv(t), r = St.uX(i), o = St.uY(i), a = St.vX(i), l = St.vY(i), c = -Math.min(0, r * e, a * s, r * e + a * s), h = -Math.min(0, o * e, l * s, o * e + l * s);
      n.transform(r, o, a, l, c, h);
    }
  };
  jt.tintMethod = jt.canUseMultiply ? jt.tintWithMultiply : jt.tintWithPerPixel;
  class Hg extends Vi {
    constructor(t, e) {
      const { text: s, resolution: i, style: r, anchor: o, width: a, height: l, roundPixels: c, ...h } = t;
      super({
        ...h
      }), this.batched = true, this._resolution = null, this._autoResolution = true, this._didTextUpdate = true, this._styleClass = e, this.text = s ?? "", this.style = r, this.resolution = i ?? null, this.allowChildren = false, this._anchor = new le({
        _onUpdate: () => {
          this.onViewUpdate();
        }
      }), o && (this.anchor = o), this.roundPixels = c ?? false, a !== void 0 && (this.width = a), l !== void 0 && (this.height = l);
    }
    get anchor() {
      return this._anchor;
    }
    set anchor(t) {
      typeof t == "number" ? this._anchor.set(t) : this._anchor.copyFrom(t);
    }
    set text(t) {
      t = t.toString(), this._text !== t && (this._text = t, this.onViewUpdate());
    }
    get text() {
      return this._text;
    }
    set resolution(t) {
      this._autoResolution = t === null, this._resolution = t, this.onViewUpdate();
    }
    get resolution() {
      return this._resolution;
    }
    get style() {
      return this._style;
    }
    set style(t) {
      var _a2;
      t || (t = {}), (_a2 = this._style) == null ? void 0 : _a2.off("update", this.onViewUpdate, this), t instanceof this._styleClass ? this._style = t : this._style = new this._styleClass(t), this._style.on("update", this.onViewUpdate, this), this.onViewUpdate();
    }
    get width() {
      return Math.abs(this.scale.x) * this.bounds.width;
    }
    set width(t) {
      this._setWidth(t, this.bounds.width);
    }
    get height() {
      return Math.abs(this.scale.y) * this.bounds.height;
    }
    set height(t) {
      this._setHeight(t, this.bounds.height);
    }
    getSize(t) {
      return t || (t = {}), t.width = Math.abs(this.scale.x) * this.bounds.width, t.height = Math.abs(this.scale.y) * this.bounds.height, t;
    }
    setSize(t, e) {
      typeof t == "object" ? (e = t.height ?? t.width, t = t.width) : e ?? (e = t), t !== void 0 && this._setWidth(t, this.bounds.width), e !== void 0 && this._setHeight(e, this.bounds.height);
    }
    containsPoint(t) {
      const e = this.bounds.width, s = this.bounds.height, i = -e * this.anchor.x;
      let r = 0;
      return t.x >= i && t.x <= i + e && (r = -s * this.anchor.y, t.y >= r && t.y <= r + s);
    }
    onViewUpdate() {
      this.didViewUpdate || (this._didTextUpdate = true), super.onViewUpdate();
    }
    destroy(t = false) {
      super.destroy(t), this.owner = null, this._bounds = null, this._anchor = null, (typeof t == "boolean" ? t : t == null ? void 0 : t.style) && this._style.destroy(t), this._style = null, this._text = null;
    }
    get styleKey() {
      return `${this._text}:${this._style.styleKey}:${this._resolution}`;
    }
  }
  function Ug(n, t) {
    let e = n[0] ?? {};
    return (typeof e == "string" || n[1]) && (Et(qt, `use new ${t}({ text: "hi!", style }) instead`), e = {
      text: e,
      style: n[1]
    }), e;
  }
  class jg {
    constructor(t) {
      this._canvasPool = /* @__PURE__ */ Object.create(null), this.canvasOptions = t || {}, this.enableFullScreen = false;
    }
    _createCanvasAndContext(t, e) {
      const s = Mt.get().createCanvas();
      s.width = t, s.height = e;
      const i = s.getContext("2d");
      return {
        canvas: s,
        context: i
      };
    }
    getOptimalCanvasAndContext(t, e, s = 1) {
      t = Math.ceil(t * s - 1e-6), e = Math.ceil(e * s - 1e-6), t = us(t), e = us(e);
      const i = (t << 17) + (e << 1);
      this._canvasPool[i] || (this._canvasPool[i] = []);
      let r = this._canvasPool[i].pop();
      return r || (r = this._createCanvasAndContext(t, e)), r;
    }
    returnCanvasAndContext(t) {
      const e = t.canvas, { width: s, height: i } = e, r = (s << 17) + (i << 1);
      t.context.resetTransform(), t.context.clearRect(0, 0, s, i), this._canvasPool[r].push(t);
    }
    clear() {
      this._canvasPool = {};
    }
  }
  yo = new jg();
  Qs.register(yo);
  let En = null, an = null;
  function Vg(n, t) {
    En || (En = Mt.get().createCanvas(256, 128), an = En.getContext("2d", {
      willReadFrequently: true
    }), an.globalCompositeOperation = "copy", an.globalAlpha = 1), (En.width < n || En.height < t) && (En.width = us(n), En.height = us(t));
  }
  function Cl(n, t, e) {
    for (let s = 0, i = 4 * e * t; s < t; ++s, i += 4) if (n[i + 3] !== 0) return false;
    return true;
  }
  function Sl(n, t, e, s, i) {
    const r = 4 * t;
    for (let o = s, a = s * r + 4 * e; o <= i; ++o, a += r) if (n[a + 3] !== 0) return false;
    return true;
  }
  function Yg(...n) {
    let t = n[0];
    t.canvas || (t = {
      canvas: n[0],
      resolution: n[1]
    });
    const { canvas: e } = t, s = Math.min(t.resolution ?? 1, 1), i = t.width ?? e.width, r = t.height ?? e.height;
    let o = t.output;
    if (Vg(i, r), !an) throw new TypeError("Failed to get canvas 2D context");
    an.drawImage(e, 0, 0, i, r, 0, 0, i * s, r * s);
    const l = an.getImageData(0, 0, i, r).data;
    let c = 0, h = 0, d = i - 1, u = r - 1;
    for (; h < r && Cl(l, i, h); ) ++h;
    if (h === r) return Nt.EMPTY;
    for (; Cl(l, i, u); ) --u;
    for (; Sl(l, i, c, h, u); ) ++c;
    for (; Sl(l, i, d, h, u); ) --d;
    return ++d, ++u, an.globalCompositeOperation = "source-over", an.strokeRect(c, h, d - c, u - h), an.globalCompositeOperation = "copy", o ?? (o = new Nt()), o.set(c / s, h / s, (d - c) / s, (u - h) / s), o;
  }
  class Xg {
    constructor(t = 0, e = 0, s = false) {
      this.first = null, this.items = /* @__PURE__ */ Object.create(null), this.last = null, this.max = t, this.resetTtl = s, this.size = 0, this.ttl = e;
    }
    clear() {
      return this.first = null, this.items = /* @__PURE__ */ Object.create(null), this.last = null, this.size = 0, this;
    }
    delete(t) {
      if (this.has(t)) {
        const e = this.items[t];
        delete this.items[t], this.size--, e.prev !== null && (e.prev.next = e.next), e.next !== null && (e.next.prev = e.prev), this.first === e && (this.first = e.next), this.last === e && (this.last = e.prev);
      }
      return this;
    }
    entries(t = this.keys()) {
      const e = new Array(t.length);
      for (let s = 0; s < t.length; s++) {
        const i = t[s];
        e[s] = [
          i,
          this.get(i)
        ];
      }
      return e;
    }
    evict(t = false) {
      if (t || this.size > 0) {
        const e = this.first;
        delete this.items[e.key], --this.size === 0 ? (this.first = null, this.last = null) : (this.first = e.next, this.first.prev = null);
      }
      return this;
    }
    expiresAt(t) {
      let e;
      return this.has(t) && (e = this.items[t].expiry), e;
    }
    get(t) {
      const e = this.items[t];
      if (e !== void 0) {
        if (this.ttl > 0 && e.expiry <= Date.now()) {
          this.delete(t);
          return;
        }
        return this.moveToEnd(e), e.value;
      }
    }
    has(t) {
      return t in this.items;
    }
    moveToEnd(t) {
      this.last !== t && (t.prev !== null && (t.prev.next = t.next), t.next !== null && (t.next.prev = t.prev), this.first === t && (this.first = t.next), t.prev = this.last, t.next = null, this.last !== null && (this.last.next = t), this.last = t, this.first === null && (this.first = t));
    }
    keys() {
      const t = new Array(this.size);
      let e = this.first, s = 0;
      for (; e !== null; ) t[s++] = e.key, e = e.next;
      return t;
    }
    setWithEvicted(t, e, s = this.resetTtl) {
      let i = null;
      if (this.has(t)) this.set(t, e, true, s);
      else {
        this.max > 0 && this.size === this.max && (i = {
          ...this.first
        }, this.evict(true));
        let r = this.items[t] = {
          expiry: this.ttl > 0 ? Date.now() + this.ttl : this.ttl,
          key: t,
          prev: this.last,
          next: null,
          value: e
        };
        ++this.size === 1 ? this.first = r : this.last.next = r, this.last = r;
      }
      return i;
    }
    set(t, e, s = false, i = this.resetTtl) {
      let r = this.items[t];
      return s || r !== void 0 ? (r.value = e, s === false && i && (r.expiry = this.ttl > 0 ? Date.now() + this.ttl : this.ttl), this.moveToEnd(r)) : (this.max > 0 && this.size === this.max && this.evict(true), r = this.items[t] = {
        expiry: this.ttl > 0 ? Date.now() + this.ttl : this.ttl,
        key: t,
        prev: this.last,
        next: null,
        value: e
      }, ++this.size === 1 ? this.first = r : this.last.next = r, this.last = r), this;
    }
    values(t = this.keys()) {
      const e = new Array(t.length);
      for (let s = 0; s < t.length; s++) e[s] = this.get(t[s]);
      return e;
    }
  }
  qg = function(n = 1e3, t = 0, e = false) {
    if (isNaN(n) || n < 0) throw new TypeError("Invalid max value");
    if (isNaN(t) || t < 0) throw new TypeError("Invalid ttl value");
    if (typeof e != "boolean") throw new TypeError("Invalid resetTtl value");
    return new Xg(n, t, e);
  };
  function Dh(n) {
    return !!n.tagStyles && Object.keys(n.tagStyles).length > 0;
  }
  function Hh(n) {
    return n.includes("<");
  }
  function Kg(n, t) {
    return n.clone().assign(t);
  }
  function Jg(n, t) {
    const e = [], s = t.tagStyles;
    if (!Dh(t) || !Hh(n)) return e.push({
      text: n,
      style: t
    }), e;
    const i = [
      t
    ], r = [];
    let o = "", a = 0;
    for (; a < n.length; ) {
      const l = n[a];
      if (l === "<") {
        const c = n.indexOf(">", a);
        if (c === -1) {
          o += l, a++;
          continue;
        }
        const h = n.slice(a + 1, c);
        if (h.startsWith("/")) {
          const d = h.slice(1).trim();
          if (r.length > 0 && r[r.length - 1] === d) {
            o.length > 0 && (e.push({
              text: o,
              style: i[i.length - 1]
            }), o = ""), i.pop(), r.pop(), a = c + 1;
            continue;
          } else {
            o += n.slice(a, c + 1), a = c + 1;
            continue;
          }
        } else {
          const d = h.trim();
          if (s[d]) {
            o.length > 0 && (e.push({
              text: o,
              style: i[i.length - 1]
            }), o = "");
            const u = i[i.length - 1], p = Kg(u, s[d]);
            i.push(p), r.push(d), a = c + 1;
            continue;
          } else {
            o += n.slice(a, c + 1), a = c + 1;
            continue;
          }
        }
      } else o += l, a++;
    }
    return o.length > 0 && e.push({
      text: o,
      style: i[i.length - 1]
    }), e;
  }
  const Zg = [
    10,
    13
  ], Qg = new Set(Zg), ty = [
    9,
    32,
    8192,
    8193,
    8194,
    8195,
    8196,
    8197,
    8198,
    8200,
    8201,
    8202,
    8287,
    12288
  ], ey = new Set(ty), ny = [
    9,
    32
  ], sy = new Set(ny), iy = [
    45,
    8208,
    8211,
    8212,
    173
  ], ry = new Set(iy), oy = /(\r\n|\r|\n)/, ay = /(?:\r\n|\r|\n)/;
  function Wi(n) {
    return typeof n != "string" ? false : Qg.has(n.charCodeAt(0));
  }
  Pe = function(n, t) {
    return typeof n != "string" ? false : ey.has(n.charCodeAt(0));
  };
  ww = function(n) {
    return typeof n != "string" ? false : sy.has(n.charCodeAt(0));
  };
  ly = function(n) {
    return typeof n != "string" ? false : ry.has(n.charCodeAt(0));
  };
  Uh = function(n) {
    return n === "normal" || n === "pre-line";
  };
  jh = function(n) {
    return n === "normal";
  };
  function sn(n) {
    if (typeof n != "string") return "";
    let t = n.length - 1;
    for (; t >= 0 && Pe(n[t]); ) t--;
    return t < n.length - 1 ? n.slice(0, t + 1) : n;
  }
  function Vh(n) {
    const t = [], e = [];
    if (typeof n != "string") return t;
    for (let s = 0; s < n.length; s++) {
      const i = n[s], r = n[s + 1];
      if (Pe(i) || Wi(i)) {
        e.length > 0 && (t.push(e.join("")), e.length = 0), i === "\r" && r === `
` ? (t.push(`\r
`), s++) : t.push(i);
        continue;
      }
      e.push(i), ly(i) && r && !Pe(r) && !Wi(r) && (t.push(e.join("")), e.length = 0);
    }
    return e.length > 0 && t.push(e.join("")), t;
  }
  function Yh(n, t, e, s) {
    const i = e(n), r = [];
    for (let o = 0; o < i.length; o++) {
      let a = i[o], l = a, c = 1;
      for (; i[o + c]; ) {
        const h = i[o + c];
        if (!s(l, h, n, o, t)) a += h, l = h, c++;
        else break;
      }
      o += c - 1, r.push(a);
    }
    return r;
  }
  const cy = /\r\n|\r|\n/g;
  function hy(n, t, e, s, i, r, o, a) {
    var _a2, _b2;
    const l = Jg(n, t);
    if (jh(t.whiteSpace)) for (let N = 0; N < l.length; N++) {
      const $ = l[N];
      l[N] = {
        text: $.text.replace(cy, " "),
        style: $.style
      };
    }
    const h = [];
    let d = [];
    for (const N of l) {
      const $ = N.text.split(oy);
      for (let W = 0; W < $.length; W++) {
        const X = $[W];
        X === `\r
` || X === "\r" || X === `
` ? (h.push(d), d = []) : X.length > 0 && d.push({
          text: X,
          style: N.style
        });
      }
    }
    (d.length > 0 || h.length === 0) && h.push(d);
    const u = e ? dy(h, t, s, i, o, a) : h, p = [], f = [], g = [], m = [], y = [];
    let b = 0;
    const x = t._fontString, _ = r(x);
    _.fontSize === 0 && (_.fontSize = t.fontSize, _.ascent = t.fontSize);
    let v = "", w = !!t.dropShadow, C = ((_a2 = t._stroke) == null ? void 0 : _a2.width) || 0;
    for (const N of u) {
      let $ = 0, W = _.ascent, X = _.descent, U = "";
      for (const R of N) {
        const B = R.style._fontString, O = r(B);
        B !== v && (s.font = B, v = B);
        const z = i(R.text, R.style.letterSpacing, s);
        $ += z, W = Math.max(W, O.ascent), X = Math.max(X, O.descent), U += R.text;
        const Y = ((_b2 = R.style._stroke) == null ? void 0 : _b2.width) || 0;
        Y > C && (C = Y), !w && R.style.dropShadow && (w = true);
      }
      N.length === 0 && (W = _.ascent, X = _.descent), p.push($), f.push(W), g.push(X), y.push(U);
      const Z = t.lineHeight || W + X;
      m.push(Z + t.leading), b = Math.max(b, $);
    }
    const E = C, A = (e && t.align !== "left" ? Math.max(b, t.wordWrapWidth) : b) + E + (t.dropShadow ? t.dropShadow.distance : 0);
    let H = 0;
    for (let N = 0; N < m.length; N++) H += m[N];
    H = Math.max(H, m[0] + E);
    const j = H + (t.dropShadow ? t.dropShadow.distance : 0), D = t.lineHeight || _.fontSize;
    return {
      width: A,
      height: j,
      lines: y,
      lineWidths: p,
      lineHeight: D + t.leading,
      maxLineWidth: b,
      fontProperties: _,
      runsByLine: u,
      lineAscents: f,
      lineDescents: g,
      lineHeights: m,
      hasDropShadow: w
    };
  }
  function dy(n, t, e, s, i, r) {
    var _a2;
    const { letterSpacing: o, whiteSpace: a, wordWrapWidth: l, breakWords: c } = t, h = Uh(a), d = l + o, u = {};
    let p = "";
    const f = (m, y) => {
      const b = `${m}|${y.styleKey}`;
      let x = u[b];
      if (x === void 0) {
        const _ = y._fontString;
        _ !== p && (e.font = _, p = _), x = s(m, y.letterSpacing, e) + y.letterSpacing, u[b] = x;
      }
      return x;
    }, g = [];
    for (const m of n) {
      const y = uy(m), b = g.length, x = (A) => {
        let H = 0, j = A;
        do {
          const { token: D, style: N } = y[j];
          H += f(D, N), j++;
        } while (j < y.length && y[j].continuesFromPrevious);
        return H;
      }, _ = (A) => {
        const H = [];
        let j = A;
        do
          H.push({
            token: y[j].token,
            style: y[j].style
          }), j++;
        while (j < y.length && y[j].continuesFromPrevious);
        return H;
      };
      let v = [], w = 0, C = !h, E = null;
      const P = () => {
        E && E.text.length > 0 && v.push(E), E = null;
      }, T = () => {
        if (P(), v.length > 0) {
          const A = v[v.length - 1];
          A.text = sn(A.text), A.text.length === 0 && v.pop();
        }
        g.push(v), v = [], w = 0, C = false;
      };
      for (let A = 0; A < y.length; A++) {
        const { token: H, style: j, continuesFromPrevious: D } = y[A], N = f(H, j);
        if (h) {
          const X = Pe(H), U = (E == null ? void 0 : E.text[E.text.length - 1]) ?? ((_a2 = v[v.length - 1]) == null ? void 0 : _a2.text.slice(-1)) ?? "", Z = U ? Pe(U) : false;
          if (X && Z) continue;
        }
        const $ = !D, W = $ ? x(A) : N;
        if (W > d && $) if (w > 0 && T(), c) {
          const X = _(A);
          for (let U = 0; U < X.length; U++) {
            const Z = X[U].token, R = X[U].style, B = Yh(Z, c, r, i);
            for (const O of B) {
              const z = f(O, R);
              z + w > d && T(), !E || E.style !== R ? (P(), E = {
                text: O,
                style: R
              }) : E.text += O, w += z;
            }
          }
          A += X.length - 1;
        } else {
          const X = _(A);
          P(), g.push(X.map((U) => ({
            text: U.token,
            style: U.style
          }))), C = false, A += X.length - 1;
        }
        else if (W + w > d && $) {
          if (Pe(H)) {
            C = false;
            continue;
          }
          T(), E = {
            text: H,
            style: j
          }, w = N;
        } else if (D && !c) !E || E.style !== j ? (P(), E = {
          text: H,
          style: j
        }) : E.text += H, w += N;
        else {
          const X = Pe(H);
          if (w === 0 && X && !C) continue;
          !E || E.style !== j ? (P(), E = {
            text: H,
            style: j
          }) : E.text += H, w += N;
        }
      }
      if (P(), v.length > 0) {
        const A = v[v.length - 1];
        A.text = sn(A.text), A.text.length === 0 && v.pop();
      }
      (v.length > 0 || g.length === b) && g.push(v);
    }
    return g;
  }
  function uy(n) {
    const t = [];
    let e = false;
    for (const s of n) {
      const i = Vh(s.text);
      let r = true;
      for (const o of i) {
        const a = Pe(o) || Wi(o), l = r && e && !a;
        t.push({
          token: o,
          style: s.style,
          continuesFromPrevious: l
        }), e = !a, r = false;
      }
    }
    return t;
  }
  const py = {
    willReadFrequently: true
  };
  function kl(n, t, e, s, i) {
    let r = e[n];
    return typeof r != "number" && (r = i(n, t, s) + t, e[n] = r), r;
  }
  function fy(n, t, e, s, i, r, o) {
    const a = e.getContext("2d", py);
    a.font = t._fontString;
    let l = 0, c = "";
    const h = [], d = /* @__PURE__ */ Object.create(null), { letterSpacing: u, whiteSpace: p } = t, f = Uh(p), g = jh(p);
    let m = !f;
    const y = t.wordWrapWidth + u, b = Vh(n);
    for (let _ = 0; _ < b.length; _++) {
      let v = b[_];
      if (Wi(v)) {
        if (!g) {
          h.push(sn(c)), m = !f, c = "", l = 0;
          continue;
        }
        v = " ";
      }
      if (f) {
        const C = Pe(v), E = Pe(c[c.length - 1]);
        if (C && E) continue;
      }
      const w = kl(v, u, d, a, s);
      if (w > y) if (c !== "" && (h.push(sn(c)), c = "", l = 0), i(v, t.breakWords)) {
        const C = Yh(v, t.breakWords, o, r);
        for (const E of C) {
          const P = kl(E, u, d, a, s);
          P + l > y && (h.push(sn(c)), m = false, c = "", l = 0), c += E, l += P;
        }
      } else c.length > 0 && (h.push(sn(c)), c = "", l = 0), h.push(sn(v)), m = false, c = "", l = 0;
      else w + l > y && (m = false, h.push(sn(c)), c = "", l = 0), (c.length > 0 || !Pe(v) || m) && (c += v, l += w);
    }
    const x = sn(c);
    return x.length > 0 && h.push(x), h.join(`
`);
  }
  const Tl = {
    willReadFrequently: true
  }, mn = class yt {
    static get experimentalLetterSpacingSupported() {
      let t = yt._experimentalLetterSpacingSupported;
      if (t === void 0) {
        const e = Mt.get().getCanvasRenderingContext2D().prototype;
        t = yt._experimentalLetterSpacingSupported = "letterSpacing" in e || "textLetterSpacing" in e;
      }
      return t;
    }
    constructor(t, e, s, i, r, o, a, l, c, h) {
      this.text = t, this.style = e, this.width = s, this.height = i, this.lines = r, this.lineWidths = o, this.lineHeight = a, this.maxLineWidth = l, this.fontProperties = c, h && (this.runsByLine = h.runsByLine, this.lineAscents = h.lineAscents, this.lineDescents = h.lineDescents, this.lineHeights = h.lineHeights, this.hasDropShadow = h.hasDropShadow);
    }
    static measureText(t = " ", e, s = yt._canvas, i = e.wordWrap) {
      var _a2;
      const r = `${t}-${e.styleKey}-wordWrap-${i}`;
      if (yt._measurementCache.has(r)) return yt._measurementCache.get(r);
      if (Dh(e) && Hh(t)) {
        const v = hy(t, e, i, yt._context, yt._measureText, yt.measureFont, yt.canBreakChars, yt.wordWrapSplit), w = new yt(t, e, v.width, v.height, v.lines, v.lineWidths, v.lineHeight, v.maxLineWidth, v.fontProperties, {
          runsByLine: v.runsByLine,
          lineAscents: v.lineAscents,
          lineDescents: v.lineDescents,
          lineHeights: v.lineHeights,
          hasDropShadow: v.hasDropShadow
        });
        return yt._measurementCache.set(r, w), w;
      }
      const a = e._fontString, l = yt.measureFont(a);
      l.fontSize === 0 && (l.fontSize = e.fontSize, l.ascent = e.fontSize, l.descent = 0);
      const c = yt._context;
      c.font = a;
      const d = (i ? yt._wordWrap(t, e, s) : t).split(ay), u = new Array(d.length);
      let p = 0;
      for (let v = 0; v < d.length; v++) {
        const w = yt._measureText(d[v], e.letterSpacing, c);
        u[v] = w, p = Math.max(p, w);
      }
      const f = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0, g = e.lineHeight || l.fontSize, m = yt._getAlignWidth(p, e, i), y = yt._adjustWidthForStyle(m, e), b = Math.max(g, l.fontSize + f) + (d.length - 1) * (g + e.leading), x = yt._adjustHeightForStyle(b, e), _ = new yt(t, e, y, x, d, u, g + e.leading, p, l);
      return yt._measurementCache.set(r, _), _;
    }
    static _adjustWidthForStyle(t, e) {
      var _a2;
      const s = ((_a2 = e._stroke) == null ? void 0 : _a2.width) || 0;
      let i = t + s;
      return e.dropShadow && (i += e.dropShadow.distance), i;
    }
    static _adjustHeightForStyle(t, e) {
      let s = t;
      return e.dropShadow && (s += e.dropShadow.distance), s;
    }
    static _getAlignWidth(t, e, s) {
      return s && e.align !== "left" ? Math.max(t, e.wordWrapWidth) : t;
    }
    static _measureText(t, e, s) {
      let i = false;
      yt.experimentalLetterSpacingSupported && (yt.experimentalLetterSpacing ? (s.letterSpacing = `${e}px`, s.textLetterSpacing = `${e}px`, i = true) : (s.letterSpacing = "0px", s.textLetterSpacing = "0px"));
      const r = s.measureText(t);
      let o = r.width;
      const a = -(r.actualBoundingBoxLeft ?? 0);
      let c = (r.actualBoundingBoxRight ?? 0) - a;
      if (o > 0) if (i) o -= e, c -= e;
      else {
        const h = (yt.graphemeSegmenter(t).length - 1) * e;
        o += h, c += h;
      }
      return Math.max(o, c);
    }
    static _wordWrap(t, e, s = yt._canvas) {
      return fy(t, e, s, yt._measureText, yt.canBreakWords, yt.canBreakChars, yt.wordWrapSplit);
    }
    static isBreakingSpace(t, e) {
      return Pe(t);
    }
    static canBreakWords(t, e) {
      return e;
    }
    static canBreakChars(t, e, s, i, r) {
      return true;
    }
    static wordWrapSplit(t) {
      return yt.graphemeSegmenter(t);
    }
    static measureFont(t) {
      if (yt._fonts[t]) return yt._fonts[t];
      const e = yt._context;
      e.font = t;
      const s = e.measureText(yt.METRICS_STRING + yt.BASELINE_SYMBOL), i = s.actualBoundingBoxAscent ?? 0, r = s.actualBoundingBoxDescent ?? 0, o = {
        ascent: i,
        descent: r,
        fontSize: i + r
      };
      return yt._fonts[t] = o, o;
    }
    static clearMetrics(t = "") {
      t ? delete yt._fonts[t] : yt._fonts = {};
    }
    static get _canvas() {
      var _a2;
      if (!yt.__canvas) {
        let t;
        try {
          const e = new OffscreenCanvas(0, 0);
          if ((_a2 = e.getContext("2d", Tl)) == null ? void 0 : _a2.measureText) return yt.__canvas = e, e;
          t = Mt.get().createCanvas();
        } catch {
          t = Mt.get().createCanvas();
        }
        t.width = t.height = 10, yt.__canvas = t;
      }
      return yt.__canvas;
    }
    static get _context() {
      return yt.__context || (yt.__context = yt._canvas.getContext("2d", Tl)), yt.__context;
    }
  };
  mn.METRICS_STRING = "|\xC9q\xC5";
  mn.BASELINE_SYMBOL = "M";
  mn.BASELINE_MULTIPLIER = 1.4;
  mn.HEIGHT_MULTIPLIER = 2;
  mn.graphemeSegmenter = (() => {
    if (typeof (Intl == null ? void 0 : Intl.Segmenter) == "function") {
      const n = new Intl.Segmenter();
      return (t) => {
        const e = n.segment(t), s = [];
        let i = 0;
        for (const r of e) s[i++] = r.segment;
        return s;
      };
    }
    return (n) => [
      ...n
    ];
  })();
  mn.experimentalLetterSpacing = false;
  mn._fonts = {};
  mn._measurementCache = qg(1e3);
  rn = mn;
  const my = [
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui"
  ];
  xo = function(n) {
    const t = typeof n.fontSize == "number" ? `${n.fontSize}px` : n.fontSize;
    let e = n.fontFamily;
    Array.isArray(n.fontFamily) || (e = n.fontFamily.split(","));
    for (let s = e.length - 1; s >= 0; s--) {
      let i = e[s].trim();
      !/([\"\'])[^\'\"]+\1/.test(i) && !my.includes(i) && (i = `"${i}"`), e[s] = i;
    }
    return `${n.fontStyle} ${n.fontVariant} ${n.fontWeight} ${t} ${e.join(",")}`;
  };
  const Al = 1e5;
  yi = function(n, t, e, s = 0, i = 0, r = 0) {
    if (n.texture === Ct.WHITE && !n.fill) return Gt.shared.setValue(n.color).setAlpha(n.alpha ?? 1).toHexa();
    if (n.fill) {
      if (n.fill instanceof Ji) {
        const o = n.fill, a = t.createPattern(o.texture.source.resource, "repeat"), l = o.transform.copyTo(_t.shared);
        return l.scale(o.texture.source.pixelWidth, o.texture.source.pixelHeight), a.setTransform(l), a;
      } else if (n.fill instanceof fn) {
        const o = n.fill, a = o.type === "linear", l = o.textureSpace === "local";
        let c = 1, h = 1;
        l && e && (c = e.width + s, h = e.height + s);
        let d, u = false;
        if (a) {
          const { start: p, end: f } = o;
          d = t.createLinearGradient(p.x * c + i, p.y * h + r, f.x * c + i, f.y * h + r), u = Math.abs(f.x - p.x) < Math.abs((f.y - p.y) * 0.1);
        } else {
          const { center: p, innerRadius: f, outerCenter: g, outerRadius: m } = o;
          d = t.createRadialGradient(p.x * c + i, p.y * h + r, f * c, g.x * c + i, g.y * h + r, m * c);
        }
        if (u && l && e) {
          const p = e.lineHeight / h;
          for (let f = 0; f < e.lines.length; f++) {
            const g = (f * e.lineHeight + s / 2) / h;
            o.colorStops.forEach((m) => {
              let y = g + m.offset * p;
              y = Math.max(0, Math.min(1, y)), d.addColorStop(Math.floor(y * Al) / Al, Gt.shared.setValue(m.color).toHex());
            });
          }
        } else o.colorStops.forEach((p) => {
          d.addColorStop(p.offset, Gt.shared.setValue(p.color).toHex());
        });
        return d;
      }
    } else {
      const o = t.createPattern(n.texture.source.resource, "repeat"), a = n.matrix.copyTo(_t.shared);
      return a.scale(n.texture.source.pixelWidth, n.texture.source.pixelHeight), o.setTransform(a), o;
    }
    return Vt("FillStyle not recognised", n), "red";
  };
  const El = new Nt();
  function Qn(n) {
    let t = 0;
    for (let e = 0; e < n.length; e++) n.charCodeAt(e) === 32 && t++;
    return t;
  }
  class gy {
    getCanvasAndContext(t) {
      const { text: e, style: s, resolution: i = 1 } = t, r = s._getFinalPadding(), o = rn.measureText(e || " ", s), a = Math.ceil(Math.ceil(Math.max(1, o.width) + r * 2) * i), l = Math.ceil(Math.ceil(Math.max(1, o.height) + r * 2) * i), c = yo.getOptimalCanvasAndContext(a, l);
      this._renderTextToCanvas(s, r, i, c, o);
      const h = s.trim ? Yg({
        canvas: c.canvas,
        width: a,
        height: l,
        resolution: 1,
        output: El
      }) : El.set(0, 0, a, l);
      return {
        canvasAndContext: c,
        frame: h
      };
    }
    returnCanvasAndContext(t) {
      yo.returnCanvasAndContext(t);
    }
    _renderTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      if (r.runsByLine && r.runsByLine.length > 0) {
        this._renderTaggedTextToCanvas(r, t, e, s, i);
        return;
      }
      const { canvas: o, context: a } = i, l = xo(t), c = r.lines, h = r.lineHeight, d = r.lineWidths, u = r.maxLineWidth, p = r.fontProperties, f = o.height;
      if (a.resetTransform(), a.scale(s, s), a.textBaseline = t.textBaseline, (_a2 = t._stroke) == null ? void 0 : _a2.width) {
        const w = t._stroke;
        a.lineWidth = w.width, a.miterLimit = w.miterLimit, a.lineJoin = w.join, a.lineCap = w.cap;
      }
      a.font = l;
      let g, m;
      const y = t.dropShadow ? 2 : 1, b = t.wordWrap ? Math.max(t.wordWrapWidth, u) : u, _ = (((_b2 = t._stroke) == null ? void 0 : _b2.width) ?? 0) / 2;
      let v = (h - p.fontSize) / 2;
      h - p.fontSize < 0 && (v = 0);
      for (let w = 0; w < y; ++w) {
        const C = t.dropShadow && w === 0, E = C ? Math.ceil(Math.max(1, f) + e * 2) : 0, P = E * s;
        if (C) this._setupDropShadow(a, t, s, P);
        else {
          const T = t._gradientBounds, A = t._gradientOffset;
          if (T) {
            const H = {
              width: T.width,
              height: T.height,
              lineHeight: T.height,
              lines: r.lines
            };
            this._setFillAndStrokeStyles(a, t, H, e, _, (A == null ? void 0 : A.x) ?? 0, (A == null ? void 0 : A.y) ?? 0);
          } else A ? this._setFillAndStrokeStyles(a, t, r, e, _, A.x, A.y) : this._setFillAndStrokeStyles(a, t, r, e, _);
          a.shadowColor = "rgba(0,0,0,0)";
        }
        for (let T = 0; T < c.length; T++) {
          g = _, m = _ + T * h + p.ascent + v, g += this._getAlignmentOffset(d[T], b, t.align);
          let A = 0;
          if (t.align === "justify" && t.wordWrap && T < c.length - 1) {
            const H = Qn(c[T]);
            H > 0 && (A = (b - d[T]) / H);
          }
          ((_c2 = t._stroke) == null ? void 0 : _c2.width) && this._drawLetterSpacing(c[T], t, i, g + e, m + e - E, true, A), t._fill !== void 0 && this._drawLetterSpacing(c[T], t, i, g + e, m + e - E, false, A);
        }
      }
    }
    _renderTaggedTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      const { canvas: o, context: a } = r, { runsByLine: l, lineWidths: c, maxLineWidth: h, lineAscents: d, lineHeights: u, hasDropShadow: p } = t, f = o.height;
      a.resetTransform(), a.scale(i, i), a.textBaseline = e.textBaseline;
      const g = p ? 2 : 1, m = e.wordWrap ? Math.max(e.wordWrapWidth, h) : h;
      let y = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0;
      for (const _ of l) for (const v of _) {
        const w = ((_b2 = v.style._stroke) == null ? void 0 : _b2.width) ?? 0;
        w > y && (y = w);
      }
      const b = y / 2, x = [];
      for (let _ = 0; _ < l.length; _++) {
        const v = l[_], w = [];
        for (const C of v) {
          const E = xo(C.style);
          a.font = E, w.push({
            width: rn._measureText(C.text, C.style.letterSpacing, a),
            font: E
          });
        }
        x.push(w);
      }
      for (let _ = 0; _ < g; ++_) {
        const v = p && _ === 0, w = v ? Math.ceil(Math.max(1, f) + s * 2) : 0, C = w * i;
        v || (a.shadowColor = "rgba(0,0,0,0)");
        let E = b;
        for (let P = 0; P < l.length; P++) {
          const T = l[P], A = c[P], H = d[P], j = u[P], D = x[P];
          let N = b;
          N += this._getAlignmentOffset(A, m, e.align);
          let $ = 0;
          if (e.align === "justify" && e.wordWrap && P < l.length - 1) {
            let U = 0;
            for (const Z of T) U += Qn(Z.text);
            U > 0 && ($ = (m - A) / U);
          }
          const W = E + H;
          let X = N + s;
          for (let U = 0; U < T.length; U++) {
            const Z = T[U], { width: R, font: B } = D[U];
            if (a.font = B, a.textBaseline = Z.style.textBaseline, (_c2 = Z.style._stroke) == null ? void 0 : _c2.width) {
              const z = Z.style._stroke;
              if (a.lineWidth = z.width, a.miterLimit = z.miterLimit, a.lineJoin = z.join, a.lineCap = z.cap, v) if (Z.style.dropShadow) this._setupDropShadow(a, Z.style, i, C);
              else {
                const Y = Qn(Z.text);
                X += R + Y * $;
                continue;
              }
              else {
                const Y = rn.measureFont(B), K = Z.style.lineHeight || Y.fontSize, nt = {
                  width: R,
                  height: K,
                  lineHeight: K,
                  lines: [
                    Z.text
                  ]
                };
                a.strokeStyle = yi(z, a, nt, s * 2, X - s, E);
              }
              this._drawLetterSpacing(Z.text, Z.style, r, X, W + s - w, true, $);
            }
            const O = Qn(Z.text);
            X += R + O * $;
          }
          X = N + s;
          for (let U = 0; U < T.length; U++) {
            const Z = T[U], { width: R, font: B } = D[U];
            if (a.font = B, a.textBaseline = Z.style.textBaseline, Z.style._fill !== void 0) {
              if (v) if (Z.style.dropShadow) this._setupDropShadow(a, Z.style, i, C);
              else {
                const z = Qn(Z.text);
                X += R + z * $;
                continue;
              }
              else {
                const z = rn.measureFont(B), Y = Z.style.lineHeight || z.fontSize, K = {
                  width: R,
                  height: Y,
                  lineHeight: Y,
                  lines: [
                    Z.text
                  ]
                };
                a.fillStyle = yi(Z.style._fill, a, K, s * 2, X - s, E);
              }
              this._drawLetterSpacing(Z.text, Z.style, r, X, W + s - w, false, $);
            }
            const O = Qn(Z.text);
            X += R + O * $;
          }
          E += j;
        }
      }
    }
    _setFillAndStrokeStyles(t, e, s, i, r, o = 0, a = 0) {
      var _a2;
      if (t.fillStyle = e._fill ? yi(e._fill, t, s, i * 2, o, a) : null, (_a2 = e._stroke) == null ? void 0 : _a2.width) {
        const l = r + i * 2;
        t.strokeStyle = yi(e._stroke, t, s, l, o, a);
      }
    }
    _setupDropShadow(t, e, s, i) {
      t.fillStyle = "black", t.strokeStyle = "black";
      const r = e.dropShadow, o = r.color, a = r.alpha;
      t.shadowColor = Gt.shared.setValue(o).setAlpha(a).toRgbaString();
      const l = r.blur * s, c = r.distance * s;
      t.shadowBlur = l, t.shadowOffsetX = Math.cos(r.angle) * c, t.shadowOffsetY = Math.sin(r.angle) * c + i;
    }
    _getAlignmentOffset(t, e, s) {
      return s === "right" ? e - t : s === "center" ? (e - t) / 2 : 0;
    }
    _drawLetterSpacing(t, e, s, i, r, o = false, a = 0) {
      const { context: l } = s, c = e.letterSpacing;
      let h = false;
      if (rn.experimentalLetterSpacingSupported && (rn.experimentalLetterSpacing ? (l.letterSpacing = `${c}px`, l.textLetterSpacing = `${c}px`, h = true) : (l.letterSpacing = "0px", l.textLetterSpacing = "0px")), (c === 0 || h) && a === 0) {
        o ? l.strokeText(t, i, r) : l.fillText(t, i, r);
        return;
      }
      if (a !== 0 && (c === 0 || h)) {
        const g = t.split(" ");
        let m = i;
        const y = l.measureText(" ").width;
        for (let b = 0; b < g.length; b++) o ? l.strokeText(g[b], m, r) : l.fillText(g[b], m, r), m += l.measureText(g[b]).width + y + a;
        return;
      }
      let d = i;
      const u = rn.graphemeSegmenter(t);
      let p = l.measureText(t).width, f = 0;
      for (let g = 0; g < u.length; ++g) {
        const m = u[g];
        o ? l.strokeText(m, d, r) : l.fillText(m, d, r);
        let y = "";
        for (let b = g + 1; b < u.length; ++b) y += u[b];
        f = l.measureText(y).width, d += p - f + c, m === " " && (d += a), p = f;
      }
    }
  }
  const as = new gy(), Ko = class Fn extends Qe {
    constructor(t = {}) {
      super(), this.uid = Kt("textStyle"), this._tick = 0, this._cachedFontString = null, yy(t), t instanceof Fn && (t = t._toObject());
      const i = {
        ...Fn.defaultTextStyle,
        ...t
      };
      for (const r in i) {
        const o = r;
        this[o] = i[r];
      }
      this._tagStyles = t.tagStyles ?? void 0, this.update(), this._tick = 0;
    }
    get align() {
      return this._align;
    }
    set align(t) {
      this._align !== t && (this._align = t, this.update());
    }
    get breakWords() {
      return this._breakWords;
    }
    set breakWords(t) {
      this._breakWords !== t && (this._breakWords = t, this.update());
    }
    get dropShadow() {
      return this._dropShadow;
    }
    set dropShadow(t) {
      this._dropShadow !== t && (t !== null && typeof t == "object" ? this._dropShadow = this._createProxy({
        ...Fn.defaultDropShadow,
        ...t
      }) : this._dropShadow = t ? this._createProxy({
        ...Fn.defaultDropShadow
      }) : null, this.update());
    }
    get fontFamily() {
      return this._fontFamily;
    }
    set fontFamily(t) {
      this._fontFamily !== t && (this._fontFamily = t, this.update());
    }
    get fontSize() {
      return this._fontSize;
    }
    set fontSize(t) {
      this._fontSize !== t && (typeof t == "string" ? this._fontSize = parseInt(t, 10) : this._fontSize = t, this.update());
    }
    get fontStyle() {
      return this._fontStyle;
    }
    set fontStyle(t) {
      this._fontStyle !== t && (this._fontStyle = t.toLowerCase(), this.update());
    }
    get fontVariant() {
      return this._fontVariant;
    }
    set fontVariant(t) {
      this._fontVariant !== t && (this._fontVariant = t, this.update());
    }
    get fontWeight() {
      return this._fontWeight;
    }
    set fontWeight(t) {
      this._fontWeight !== t && (this._fontWeight = t, this.update());
    }
    get leading() {
      return this._leading;
    }
    set leading(t) {
      this._leading !== t && (this._leading = t, this.update());
    }
    get letterSpacing() {
      return this._letterSpacing;
    }
    set letterSpacing(t) {
      this._letterSpacing !== t && (this._letterSpacing = t, this.update());
    }
    get lineHeight() {
      return this._lineHeight;
    }
    set lineHeight(t) {
      this._lineHeight !== t && (this._lineHeight = t, this.update());
    }
    get padding() {
      return this._padding;
    }
    set padding(t) {
      this._padding !== t && (this._padding = t, this.update());
    }
    get filters() {
      return this._filters;
    }
    set filters(t) {
      this._filters !== t && (this._filters = Object.freeze(t), this.update());
    }
    get trim() {
      return this._trim;
    }
    set trim(t) {
      this._trim !== t && (this._trim = t, this.update());
    }
    get textBaseline() {
      return this._textBaseline;
    }
    set textBaseline(t) {
      this._textBaseline !== t && (this._textBaseline = t, this.update());
    }
    get whiteSpace() {
      return this._whiteSpace;
    }
    set whiteSpace(t) {
      this._whiteSpace !== t && (this._whiteSpace = t, this.update());
    }
    get wordWrap() {
      return this._wordWrap;
    }
    set wordWrap(t) {
      this._wordWrap !== t && (this._wordWrap = t, this.update());
    }
    get wordWrapWidth() {
      return this._wordWrapWidth;
    }
    set wordWrapWidth(t) {
      this._wordWrapWidth !== t && (this._wordWrapWidth = t, this.update());
    }
    get fill() {
      return this._originalFill;
    }
    set fill(t) {
      t !== this._originalFill && (this._originalFill = t, this._isFillStyle(t) && (this._originalFill = this._createProxy({
        ...Ae.defaultFillStyle,
        ...t
      }, () => {
        this._fill = Wn({
          ...this._originalFill
        }, Ae.defaultFillStyle);
      })), this._fill = Wn(t === 0 ? "black" : t, Ae.defaultFillStyle), this.update());
    }
    get stroke() {
      return this._originalStroke;
    }
    set stroke(t) {
      t !== this._originalStroke && (this._originalStroke = t, this._isFillStyle(t) && (this._originalStroke = this._createProxy({
        ...Ae.defaultStrokeStyle,
        ...t
      }, () => {
        this._stroke = Ni({
          ...this._originalStroke
        }, Ae.defaultStrokeStyle);
      })), this._stroke = Ni(t, Ae.defaultStrokeStyle), this.update());
    }
    get tagStyles() {
      return this._tagStyles;
    }
    set tagStyles(t) {
      this._tagStyles !== t && (this._tagStyles = t ?? void 0, this.update());
    }
    update() {
      this._tick++, this._cachedFontString = null, this.emit("update", this);
    }
    reset() {
      const t = Fn.defaultTextStyle;
      for (const e in t) this[e] = t[e];
    }
    assign(t) {
      for (const e in t) {
        const s = e;
        this[s] = t[e];
      }
      return this;
    }
    get styleKey() {
      return `${this.uid}-${this._tick}`;
    }
    get _fontString() {
      return this._cachedFontString === null && (this._cachedFontString = xo(this)), this._cachedFontString;
    }
    _toObject() {
      return {
        align: this.align,
        breakWords: this.breakWords,
        dropShadow: this._dropShadow ? {
          ...this._dropShadow
        } : null,
        fill: this._fill ? {
          ...this._fill
        } : void 0,
        fontFamily: this.fontFamily,
        fontSize: this.fontSize,
        fontStyle: this.fontStyle,
        fontVariant: this.fontVariant,
        fontWeight: this.fontWeight,
        leading: this.leading,
        letterSpacing: this.letterSpacing,
        lineHeight: this.lineHeight,
        padding: this.padding,
        stroke: this._stroke ? {
          ...this._stroke
        } : void 0,
        textBaseline: this.textBaseline,
        trim: this.trim,
        whiteSpace: this.whiteSpace,
        wordWrap: this.wordWrap,
        wordWrapWidth: this.wordWrapWidth,
        filters: this._filters ? [
          ...this._filters
        ] : void 0,
        tagStyles: this._tagStyles ? {
          ...this._tagStyles
        } : void 0
      };
    }
    clone() {
      return new Fn(this._toObject());
    }
    _getFinalPadding() {
      let t = 0;
      if (this._filters) for (let e = 0; e < this._filters.length; e++) t += this._filters[e].padding;
      return Math.max(this._padding, t);
    }
    destroy(t = false) {
      var _a2, _b2, _c2, _d2;
      if (this.removeAllListeners(), typeof t == "boolean" ? t : t == null ? void 0 : t.texture) {
        const s = typeof t == "boolean" ? t : t == null ? void 0 : t.textureSource;
        ((_a2 = this._fill) == null ? void 0 : _a2.texture) && this._fill.texture.destroy(s), ((_b2 = this._originalFill) == null ? void 0 : _b2.texture) && this._originalFill.texture.destroy(s), ((_c2 = this._stroke) == null ? void 0 : _c2.texture) && this._stroke.texture.destroy(s), ((_d2 = this._originalStroke) == null ? void 0 : _d2.texture) && this._originalStroke.texture.destroy(s);
      }
      this._fill = null, this._stroke = null, this.dropShadow = null, this._originalStroke = null, this._originalFill = null;
    }
    _createProxy(t, e) {
      return new Proxy(t, {
        set: (s, i, r) => (s[i] === r || (s[i] = r, e == null ? void 0 : e(i, r), this.update()), true)
      });
    }
    _isFillStyle(t) {
      return (t ?? null) !== null && !(Gt.isColorLike(t) || t instanceof fn || t instanceof Ji);
    }
  };
  Ko.defaultDropShadow = {
    alpha: 1,
    angle: Math.PI / 6,
    blur: 0,
    color: "black",
    distance: 5
  };
  Ko.defaultTextStyle = {
    align: "left",
    breakWords: false,
    dropShadow: null,
    fill: "black",
    fontFamily: "Arial",
    fontSize: 26,
    fontStyle: "normal",
    fontVariant: "normal",
    fontWeight: "normal",
    leading: 0,
    letterSpacing: 0,
    lineHeight: 0,
    padding: 0,
    stroke: null,
    textBaseline: "alphabetic",
    trim: false,
    whiteSpace: "pre",
    wordWrap: false,
    wordWrapWidth: 100
  };
  Le = Ko;
  function yy(n) {
    const t = n;
    if (typeof t.dropShadow == "boolean" && t.dropShadow) {
      const e = Le.defaultDropShadow;
      n.dropShadow = {
        alpha: t.dropShadowAlpha ?? e.alpha,
        angle: t.dropShadowAngle ?? e.angle,
        blur: t.dropShadowBlur ?? e.blur,
        color: t.dropShadowColor ?? e.color,
        distance: t.dropShadowDistance ?? e.distance
      };
    }
    if (t.strokeThickness !== void 0) {
      Et(qt, "strokeThickness is now a part of stroke");
      const e = t.stroke;
      let s = {};
      if (Gt.isColorLike(e)) s.color = e;
      else if (e instanceof fn || e instanceof Ji) s.fill = e;
      else if (Object.hasOwnProperty.call(e, "color") || Object.hasOwnProperty.call(e, "fill")) s = e;
      else throw new Error("Invalid stroke value.");
      n.stroke = {
        ...s,
        width: t.strokeThickness
      };
    }
    if (Array.isArray(t.fillGradientStops)) {
      if (Et(qt, "gradient fill is now a fill pattern: `new FillGradient(...)`"), !Array.isArray(t.fill) || t.fill.length === 0) throw new Error("Invalid fill value. Expected an array of colors for gradient fill.");
      t.fill.length !== t.fillGradientStops.length && Vt("The number of fill colors must match the number of fill gradient stops.");
      const e = new fn({
        start: {
          x: 0,
          y: 0
        },
        end: {
          x: 0,
          y: 1
        },
        textureSpace: "local"
      }), s = t.fillGradientStops.slice(), i = t.fill.map((r) => Gt.shared.setValue(r).toNumber());
      s.forEach((r, o) => {
        e.addColorStop(r, i[o]);
      }), n.fill = {
        fill: e
      };
    }
  }
  function xy(n, t) {
    const { texture: e, bounds: s } = n, i = t._style._getFinalPadding();
    Mc(s, t._anchor, e);
    const r = t._anchor._x * i * 2, o = t._anchor._y * i * 2;
    s.minX -= i - r, s.minY -= i - o, s.maxX -= i - r, s.maxY -= i - o;
  }
  by = class {
    constructor() {
      this.batcherName = "default", this.topology = "triangle-list", this.attributeSize = 4, this.indexSize = 6, this.packAsQuad = true, this.roundPixels = 0, this._attributeStart = 0, this._batcher = null, this._batch = null;
    }
    get blendMode() {
      return this.renderable.groupBlendMode;
    }
    get color() {
      return this.renderable.groupColorAlpha;
    }
    reset() {
      this.renderable = null, this.texture = null, this._batcher = null, this._batch = null, this.bounds = null;
    }
    destroy() {
      this.reset();
    }
  };
  class _y extends by {
  }
  class Xh {
    constructor(t) {
      this._renderer = t, t.runners.resolutionChange.add(this), this._managedTexts = new _s({
        renderer: t,
        type: "renderable",
        onUnload: this.onTextUnload.bind(this),
        name: "canvasText"
      });
    }
    resolutionChange() {
      for (const t in this._managedTexts.items) {
        const e = this._managedTexts.items[t];
        (e == null ? void 0 : e._autoResolution) && e.onViewUpdate();
      }
    }
    validateRenderable(t) {
      const e = this._getGpuText(t), s = t.styleKey;
      return e.currentKey !== s ? true : t._didTextUpdate;
    }
    addRenderable(t, e) {
      const s = this._getGpuText(t);
      if (t._didTextUpdate) {
        const i = t._autoResolution ? this._renderer.resolution : t.resolution;
        (s.currentKey !== t.styleKey || t._resolution !== i) && this._updateGpuText(t), t._didTextUpdate = false, xy(s, t);
      }
      this._renderer.renderPipes.batch.addToBatch(s, e);
    }
    updateRenderable(t) {
      const e = this._getGpuText(t);
      e._batcher.updateElement(e);
    }
    _updateGpuText(t) {
      const e = this._getGpuText(t);
      e.texture && this._renderer.canvasText.decreaseReferenceCount(e.currentKey), t._resolution = t._autoResolution ? this._renderer.resolution : t.resolution, e.texture = this._renderer.canvasText.getManagedTexture(t), e.currentKey = t.styleKey;
    }
    _getGpuText(t) {
      return t._gpuData[this._renderer.uid] || this.initGpuText(t);
    }
    initGpuText(t) {
      const e = new _y();
      return e.currentKey = "--", e.renderable = t, e.transform = t.groupTransform, e.bounds = {
        minX: 0,
        maxX: 1,
        minY: 0,
        maxY: 0
      }, e.roundPixels = this._renderer._roundPixels | t._roundPixels, t._gpuData[this._renderer.uid] = e, this._managedTexts.add(t), e;
    }
    onTextUnload(t) {
      const e = t._gpuData[this._renderer.uid];
      if (!e) return;
      const { canvasText: s } = this._renderer;
      s.getReferenceCount(e.currentKey) > 0 ? s.decreaseReferenceCount(e.currentKey) : e.texture && s.returnTexture(e.texture);
    }
    destroy() {
      this._managedTexts.destroy(), this._renderer = null;
    }
  }
  Xh.extension = {
    type: [
      it.WebGLPipes,
      it.WebGPUPipes,
      it.CanvasPipes
    ],
    name: "text"
  };
  const wy = new Se();
  function vy(n, t, e, s, i = false) {
    const r = wy;
    r.minX = 0, r.minY = 0, r.maxX = n.width / s | 0, r.maxY = n.height / s | 0;
    const o = ji.getOptimalTexture(r.width, r.height, s, false, i);
    return o.source.uploadMethodId = "image", o.source.resource = n, o.source.alphaMode = "premultiply-alpha-on-upload", o.frame.width = t / s, o.frame.height = e / s, o.source.emit("update", o.source), o.updateUvs(), o;
  }
  class qh {
    constructor(t, e) {
      this._activeTextures = {}, this._renderer = t, this._retainCanvasContext = e;
    }
    getTexture(t, e, s, i) {
      typeof t == "string" && (Et("8.0.0", "CanvasTextSystem.getTexture: Use object TextOptions instead of separate arguments"), t = {
        text: t,
        style: s,
        resolution: e
      }), t.style instanceof Le || (t.style = new Le(t.style)), t.textureStyle instanceof Hn || (t.textureStyle = new Hn(t.textureStyle)), typeof t.text != "string" && (t.text = t.text.toString());
      const { text: r, style: o, textureStyle: a, autoGenerateMipmaps: l } = t, c = t.resolution ?? this._renderer.resolution, { frame: h, canvasAndContext: d } = as.getCanvasAndContext({
        text: r,
        style: o,
        resolution: c
      }), u = vy(d.canvas, h.width, h.height, c, l);
      if (a && (u.source.style = a), o.trim && (h.pad(o.padding), u.frame.copyFrom(h), u.frame.scale(1 / c), u.updateUvs()), o.filters) {
        const p = this._applyFilters(u, o.filters);
        return this.returnTexture(u), as.returnCanvasAndContext(d), p;
      }
      return this._renderer.texture.initSource(u._source), this._retainCanvasContext || as.returnCanvasAndContext(d), u;
    }
    returnTexture(t) {
      const e = t.source, s = e.resource;
      if (this._retainCanvasContext && (s == null ? void 0 : s.getContext)) {
        const i = s.getContext("2d");
        i && as.returnCanvasAndContext({
          canvas: s,
          context: i
        });
      }
      e.resource = null, e.uploadMethodId = "unknown", e.alphaMode = "no-premultiply-alpha", ji.returnTexture(t, true);
    }
    renderTextToCanvas() {
      Et("8.10.0", "CanvasTextSystem.renderTextToCanvas: no longer supported, use CanvasTextSystem.getTexture instead");
    }
    getManagedTexture(t) {
      t._resolution = t._autoResolution ? this._renderer.resolution : t.resolution;
      const e = t.styleKey;
      if (this._activeTextures[e]) return this._increaseReferenceCount(e), this._activeTextures[e].texture;
      const s = this.getTexture({
        text: t.text,
        style: t.style,
        resolution: t._resolution,
        textureStyle: t.textureStyle,
        autoGenerateMipmaps: t.autoGenerateMipmaps
      });
      return this._activeTextures[e] = {
        texture: s,
        usageCount: 1
      }, s;
    }
    decreaseReferenceCount(t) {
      const e = this._activeTextures[t];
      e && (e.usageCount--, e.usageCount === 0 && (this.returnTexture(e.texture), this._activeTextures[t] = null));
    }
    getReferenceCount(t) {
      var _a2;
      return ((_a2 = this._activeTextures[t]) == null ? void 0 : _a2.usageCount) ?? 0;
    }
    _increaseReferenceCount(t) {
      this._activeTextures[t].usageCount++;
    }
    _applyFilters(t, e) {
      const s = this._renderer.renderTarget.renderTarget, i = this._renderer.filter.generateFilteredTexture({
        texture: t,
        filters: e
      });
      return this._renderer.renderTarget.bind(s, false), i;
    }
    destroy() {
      this._renderer = null;
      for (const t in this._activeTextures) this._activeTextures[t] && this.returnTexture(this._activeTextures[t].texture);
      this._activeTextures = null;
    }
  }
  class Kh extends qh {
    constructor(t) {
      super(t, true);
    }
  }
  Kh.extension = {
    type: [
      it.CanvasSystem
    ],
    name: "canvasText"
  };
  class Jh extends qh {
    constructor(t) {
      super(t, false);
    }
  }
  Jh.extension = {
    type: [
      it.WebGLSystem,
      it.WebGPUSystem
    ],
    name: "canvasText"
  };
  Ot.add(Kh);
  Ot.add(Jh);
  Ot.add(Xh);
  class Ge extends Hg {
    constructor(...t) {
      const e = Ug(t, "Text");
      super(e, Le), this.renderPipeId = "text", e.textureStyle && (this.textureStyle = e.textureStyle instanceof Hn ? e.textureStyle : new Hn(e.textureStyle)), this.autoGenerateMipmaps = e.autoGenerateMipmaps ?? ke.defaultOptions.autoGenerateMipmaps;
    }
    updateBounds() {
      const t = this._bounds, e = this._anchor;
      let s = 0, i = 0;
      if (this._style.trim) {
        const { frame: r, canvasAndContext: o } = as.getCanvasAndContext({
          text: this.text,
          style: this._style,
          resolution: 1
        });
        as.returnCanvasAndContext(o), s = r.width, i = r.height;
      } else {
        const r = rn.measureText(this._text, this._style);
        s = r.width, i = r.height;
      }
      t.minX = -e._x * s, t.maxX = t.minX + s, t.minY = -e._y * i, t.maxY = t.minY + i;
    }
  }
  Zi = class extends Ct {
    static create(t) {
      const { dynamic: e, ...s } = t;
      return new Zi({
        source: new ke(s),
        dynamic: e ?? false
      });
    }
    resize(t, e, s) {
      return this.source.resize(t, e, s), this;
    }
  };
  class Cy {
    execute(t, e) {
      var _a2, _b2;
      const s = t.renderer, i = s.canvasContext.activeContext, r = e.particleChildren, o = e.texture;
      i.save(), s.canvasContext.setContextTransform(e.worldTransform, e.roundPixels), s.canvasContext.setBlendMode(e.groupBlendMode);
      const a = e.groupColorAlpha, l = ((_a2 = s.filter) == null ? void 0 : _a2.alphaMultiplier) ?? 1, c = (a >>> 24 & 255) / 255 * l;
      for (let h = 0; h < r.length; h++) {
        const d = r[h], u = d.texture || o;
        if (!((_b2 = u == null ? void 0 : u.source) == null ? void 0 : _b2.resource)) continue;
        const p = d.color, f = (p >>> 24 & 255) / 255 * c;
        if (f <= 0) continue;
        const g = p & 16777215, m = ((g & 255) << 16) + (g & 65280) + (g >> 16 & 255);
        let y = u.source.resource;
        m !== 16777215 && (y = jt.getTintedCanvas({
          texture: u
        }, m));
        const b = u.frame, x = u.source.resolution, _ = b.x * x, v = b.y * x, w = b.width * x, C = b.height * x;
        i.globalAlpha = f;
        const E = -d.anchorX * b.width, P = -d.anchorY * b.height;
        d.rotation !== 0 || d.scaleX !== 1 || d.scaleY !== 1 ? (i.save(), i.translate(d.x, d.y), i.rotate(d.rotation), i.scale(d.scaleX, d.scaleY), i.drawImage(y, _, v, w, C, E, P, b.width, b.height), i.restore()) : i.drawImage(y, _, v, w, C, d.x + E, d.y + P, b.width, b.height);
      }
      i.restore();
    }
  }
  function Ml(n, t = null) {
    const e = n * 6;
    if (e > 65535 ? t || (t = new Uint32Array(e)) : t || (t = new Uint16Array(e)), t.length !== e) throw new Error(`Out buffer length is incorrect, got ${t.length} and expected ${e}`);
    for (let s = 0, i = 0; s < e; s += 6, i += 4) t[s + 0] = i + 0, t[s + 1] = i + 1, t[s + 2] = i + 2, t[s + 3] = i + 0, t[s + 4] = i + 2, t[s + 5] = i + 3;
    return t;
  }
  function Sy(n) {
    return {
      dynamicUpdate: Pl(n, true),
      staticUpdate: Pl(n, false)
    };
  }
  function Pl(n, t) {
    const e = [];
    e.push(`

        var index = 0;

        for (let i = 0; i < ps.length; ++i)
        {
            const p = ps[i];

            `);
    let s = 0;
    for (const r in n) {
      const o = n[r];
      if (t !== o.dynamic) continue;
      e.push(`offset = index + ${s}`), e.push(o.code);
      const a = Bi(o.format);
      s += a.stride / 4;
    }
    e.push(`
            index += stride * 4;
        }
    `), e.unshift(`
        var stride = ${s};
    `);
    const i = e.join(`
`);
    return new Function("ps", "f32v", "u32v", i);
  }
  class ky {
    constructor(t) {
      this._size = 0, this._generateParticleUpdateCache = {};
      const e = this._size = t.size ?? 1e3, s = t.properties;
      let i = 0, r = 0;
      for (const h in s) {
        const d = s[h], u = Bi(d.format);
        d.dynamic ? r += u.stride : i += u.stride;
      }
      this._dynamicStride = r / 4, this._staticStride = i / 4, this.staticAttributeBuffer = new os(e * 4 * i), this.dynamicAttributeBuffer = new os(e * 4 * r), this.indexBuffer = Ml(e);
      const o = new Sh();
      let a = 0, l = 0;
      this._staticBuffer = new jn({
        data: new Float32Array(1),
        label: "static-particle-buffer",
        shrinkToFit: false,
        usage: re.VERTEX | re.COPY_DST
      }), this._dynamicBuffer = new jn({
        data: new Float32Array(1),
        label: "dynamic-particle-buffer",
        shrinkToFit: false,
        usage: re.VERTEX | re.COPY_DST
      });
      for (const h in s) {
        const d = s[h], u = Bi(d.format);
        d.dynamic ? (o.addAttribute(d.attributeName, {
          buffer: this._dynamicBuffer,
          stride: this._dynamicStride * 4,
          offset: a * 4,
          format: d.format
        }), a += u.size) : (o.addAttribute(d.attributeName, {
          buffer: this._staticBuffer,
          stride: this._staticStride * 4,
          offset: l * 4,
          format: d.format
        }), l += u.size);
      }
      o.addIndex(this.indexBuffer);
      const c = this.getParticleUpdate(s);
      this._dynamicUpload = c.dynamicUpdate, this._staticUpload = c.staticUpdate, this.geometry = o;
    }
    getParticleUpdate(t) {
      const e = Ty(t);
      return this._generateParticleUpdateCache[e] ? this._generateParticleUpdateCache[e] : (this._generateParticleUpdateCache[e] = this.generateParticleUpdate(t), this._generateParticleUpdateCache[e]);
    }
    generateParticleUpdate(t) {
      return Sy(t);
    }
    update(t, e) {
      t.length > this._size && (e = true, this._size = Math.max(t.length, this._size * 1.5 | 0), this.staticAttributeBuffer = new os(this._size * this._staticStride * 4 * 4), this.dynamicAttributeBuffer = new os(this._size * this._dynamicStride * 4 * 4), this.indexBuffer = Ml(this._size), this.geometry.indexBuffer.setDataWithSize(this.indexBuffer, this.indexBuffer.byteLength, true));
      const s = this.dynamicAttributeBuffer;
      if (this._dynamicUpload(t, s.float32View, s.uint32View), this._dynamicBuffer.setDataWithSize(this.dynamicAttributeBuffer.float32View, t.length * this._dynamicStride * 4, true), e) {
        const i = this.staticAttributeBuffer;
        this._staticUpload(t, i.float32View, i.uint32View), this._staticBuffer.setDataWithSize(i.float32View, t.length * this._staticStride * 4, true);
      }
    }
    destroy() {
      this._staticBuffer.destroy(), this._dynamicBuffer.destroy(), this.geometry.destroy();
    }
  }
  function Ty(n) {
    const t = [];
    for (const e in n) {
      const s = n[e];
      t.push(e, s.code, s.dynamic ? "d" : "s");
    }
    return t.join("_");
  }
  var Ay = `varying vec2 vUV;
varying vec4 vColor;

uniform sampler2D uTexture;

void main(void){
    vec4 color = texture2D(uTexture, vUV) * vColor;
    gl_FragColor = color;
}`, Ey = `attribute vec2 aVertex;
attribute vec2 aUV;
attribute vec4 aColor;

attribute vec2 aPosition;
attribute float aRotation;

uniform mat3 uTranslationMatrix;
uniform float uRound;
uniform vec2 uResolution;
uniform vec4 uColor;

varying vec2 vUV;
varying vec4 vColor;

vec2 roundPixels(vec2 position, vec2 targetSize)
{       
    return (floor(((position * 0.5 + 0.5) * targetSize) + 0.5) / targetSize) * 2.0 - 1.0;
}

void main(void){
    float cosRotation = cos(aRotation);
    float sinRotation = sin(aRotation);
    float x = aVertex.x * cosRotation - aVertex.y * sinRotation;
    float y = aVertex.x * sinRotation + aVertex.y * cosRotation;

    vec2 v = vec2(x, y);
    v = v + aPosition;

    gl_Position = vec4((uTranslationMatrix * vec3(v, 1.0)).xy, 0.0, 1.0);

    if(uRound == 1.0)
    {
        gl_Position.xy = roundPixels(gl_Position.xy, uResolution);
    }

    vUV = aUV;
    vColor = vec4(aColor.rgb * aColor.a, aColor.a) * uColor;
}
`, Il = `
struct ParticleUniforms {
  uTranslationMatrix:mat3x3<f32>,
  uColor:vec4<f32>,
  uRound:f32,
  uResolution:vec2<f32>,
};

fn roundPixels(position: vec2<f32>, targetSize: vec2<f32>) -> vec2<f32>
{
  return (floor(((position * 0.5 + 0.5) * targetSize) + 0.5) / targetSize) * 2.0 - 1.0;
}

@group(0) @binding(0) var<uniform> uniforms: ParticleUniforms;

@group(1) @binding(0) var uTexture: texture_2d<f32>;
@group(1) @binding(1) var uSampler : sampler;

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv : vec2<f32>,
    @location(1) color : vec4<f32>,
  };
@vertex
fn mainVertex(
  @location(0) aVertex: vec2<f32>,
  @location(1) aPosition: vec2<f32>,
  @location(2) aUV: vec2<f32>,
  @location(3) aColor: vec4<f32>,
  @location(4) aRotation: f32,
) -> VSOutput {
  
   let v = vec2(
       aVertex.x * cos(aRotation) - aVertex.y * sin(aRotation),
       aVertex.x * sin(aRotation) + aVertex.y * cos(aRotation)
   ) + aPosition;

   var position = vec4((uniforms.uTranslationMatrix * vec3(v, 1.0)).xy, 0.0, 1.0);

   if(uniforms.uRound == 1.0) {
       position = vec4(roundPixels(position.xy, uniforms.uResolution), position.zw);
   }

    let vColor = vec4(aColor.rgb * aColor.a, aColor.a) * uniforms.uColor;

  return VSOutput(
   position,
   aUV,
   vColor,
  );
}

@fragment
fn mainFragment(
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @builtin(position) position: vec4<f32>,
) -> @location(0) vec4<f32> {

    var sample = textureSample(uTexture, uSampler, uv) * color;
   
    return sample;
}`;
  class My extends Xi {
    constructor() {
      const t = Oo.from({
        vertex: Ey,
        fragment: Ay
      }), e = ti.from({
        fragment: {
          source: Il,
          entryPoint: "mainFragment"
        },
        vertex: {
          source: Il,
          entryPoint: "mainVertex"
        }
      });
      super({
        glProgram: t,
        gpuProgram: e,
        resources: {
          uTexture: Ct.WHITE.source,
          uSampler: new Hn({}),
          uniforms: {
            uTranslationMatrix: {
              value: new _t(),
              type: "mat3x3<f32>"
            },
            uColor: {
              value: new Gt(16777215),
              type: "vec4<f32>"
            },
            uRound: {
              value: 1,
              type: "f32"
            },
            uResolution: {
              value: [
                0,
                0
              ],
              type: "vec2<f32>"
            }
          }
        }
      });
    }
  }
  class Qi {
    constructor(t, e) {
      this.state = Oi.for2d(), this.localUniforms = new Fo({
        uTranslationMatrix: {
          value: new _t(),
          type: "mat3x3<f32>"
        },
        uColor: {
          value: new Float32Array(4),
          type: "vec4<f32>"
        },
        uRound: {
          value: 1,
          type: "f32"
        },
        uResolution: {
          value: [
            0,
            0
          ],
          type: "vec2<f32>"
        }
      }), this.renderer = t, this.adaptor = e, this.defaultShader = new My(), this.state = Oi.for2d(), this._managedContainers = new _s({
        renderer: t,
        type: "renderable",
        name: "particleContainer"
      });
    }
    validateRenderable(t) {
      return false;
    }
    addRenderable(t, e) {
      this.renderer.renderPipes.batch.break(e), e.add(t);
    }
    getBuffers(t) {
      return t._gpuData[this.renderer.uid] || this._initBuffer(t);
    }
    _initBuffer(t) {
      return t._gpuData[this.renderer.uid] = new ky({
        size: t.particleChildren.length,
        properties: t._properties
      }), this._managedContainers.add(t), t._gpuData[this.renderer.uid];
    }
    updateRenderable(t) {
    }
    execute(t) {
      const e = t.particleChildren;
      if (e.length === 0) return;
      const s = this.renderer, i = this.getBuffers(t);
      t.texture || (t.texture = e[0].texture);
      const r = this.state;
      i.update(e, t._childrenDirty), t._childrenDirty = false, r.blendMode = co(t.blendMode, t.texture._source);
      const o = this.localUniforms.uniforms, a = o.uTranslationMatrix;
      t.worldTransform.copyTo(a);
      const l = s.globalUniforms.globalUniformData;
      a.tx -= l.offset.x, a.ty -= l.offset.y, a.prepend(l.projectionMatrix), o.uResolution = l.resolution, o.uRound = s._roundPixels | t._roundPixels, Gh(t.groupColorAlpha, o.uColor, 0), this.adaptor.execute(this, t);
    }
    destroy() {
      this._managedContainers.destroy(), this.renderer = null, this.defaultShader && (this.defaultShader.destroy(), this.defaultShader = null);
    }
  }
  Qi.extension = {
    type: [
      it.CanvasPipes
    ],
    name: "particle"
  };
  class Zh extends Qi {
    constructor(t) {
      super(t, new Cy());
    }
  }
  Zh.extension = {
    type: [
      it.CanvasPipes
    ],
    name: "particle"
  };
  class Py {
    execute(t, e) {
      const s = t.state, i = t.renderer, r = e.shader || t.defaultShader;
      r.resources.uTexture = e.texture._source, r.resources.uniforms = t.localUniforms;
      const o = i.gl, a = t.getBuffers(e);
      i.shader.bind(r), i.state.set(s), i.geometry.bind(a.geometry, r.glProgram);
      const c = a.geometry.indexBuffer.data.BYTES_PER_ELEMENT === 2 ? o.UNSIGNED_SHORT : o.UNSIGNED_INT;
      o.drawElements(o.TRIANGLES, e.particleChildren.length * 6, c, 0);
    }
  }
  class Qh extends Qi {
    constructor(t) {
      super(t, new Py());
    }
  }
  Qh.extension = {
    type: [
      it.WebGLPipes
    ],
    name: "particle"
  };
  class Iy {
    execute(t, e) {
      const s = t.renderer, i = e.shader || t.defaultShader;
      i.groups[0] = s.renderPipes.uniformBatch.getUniformBindGroup(t.localUniforms, true), i.groups[1] = s.texture.getTextureBindGroup(e.texture);
      const r = t.state, o = t.getBuffers(e);
      s.encoder.draw({
        geometry: o.geometry,
        shader: e.shader || t.defaultShader,
        state: r,
        size: e.particleChildren.length * 6
      });
    }
  }
  class td extends Qi {
    constructor(t) {
      super(t, new Iy());
    }
  }
  td.extension = {
    type: [
      it.WebGPUPipes
    ],
    name: "particle"
  };
  const ed = class bo {
    constructor(t) {
      if (t instanceof Ct) this.texture = t, Qr(this, bo.defaultOptions, {});
      else {
        const e = {
          ...bo.defaultOptions,
          ...t
        };
        Qr(this, e, {});
      }
    }
    get alpha() {
      return this._alpha;
    }
    set alpha(t) {
      this._alpha = Math.min(Math.max(t, 0), 1), this._updateColor();
    }
    get tint() {
      return Os(this._tint);
    }
    set tint(t) {
      this._tint = Gt.shared.setValue(t ?? 16777215).toBgrNumber(), this._updateColor();
    }
    _updateColor() {
      this.color = this._tint + ((this._alpha * 255 | 0) << 24);
    }
  };
  ed.defaultOptions = {
    anchorX: 0,
    anchorY: 0,
    x: 0,
    y: 0,
    scaleX: 1,
    scaleY: 1,
    rotation: 0,
    tint: 16777215,
    alpha: 1
  };
  let Gi = ed;
  const Rl = {
    vertex: {
      attributeName: "aVertex",
      format: "float32x2",
      code: `
            const texture = p.texture;
            const sx = p.scaleX;
            const sy = p.scaleY;
            const ax = p.anchorX;
            const ay = p.anchorY;
            const trim = texture.trim;
            const orig = texture.orig;

            if (trim)
            {
                w1 = trim.x - (ax * orig.width);
                w0 = w1 + trim.width;

                h1 = trim.y - (ay * orig.height);
                h0 = h1 + trim.height;
            }
            else
            {
                w1 = -ax * (orig.width);
                w0 = w1 + orig.width;

                h1 = -ay * (orig.height);
                h0 = h1 + orig.height;
            }

            f32v[offset] = w1 * sx;
            f32v[offset + 1] = h1 * sy;

            f32v[offset + stride] = w0 * sx;
            f32v[offset + stride + 1] = h1 * sy;

            f32v[offset + (stride * 2)] = w0 * sx;
            f32v[offset + (stride * 2) + 1] = h0 * sy;

            f32v[offset + (stride * 3)] = w1 * sx;
            f32v[offset + (stride * 3) + 1] = h0 * sy;
        `,
      dynamic: false
    },
    position: {
      attributeName: "aPosition",
      format: "float32x2",
      code: `
            var x = p.x;
            var y = p.y;

            f32v[offset] = x;
            f32v[offset + 1] = y;

            f32v[offset + stride] = x;
            f32v[offset + stride + 1] = y;

            f32v[offset + (stride * 2)] = x;
            f32v[offset + (stride * 2) + 1] = y;

            f32v[offset + (stride * 3)] = x;
            f32v[offset + (stride * 3) + 1] = y;
        `,
      dynamic: true
    },
    rotation: {
      attributeName: "aRotation",
      format: "float32",
      code: `
            var rotation = p.rotation;

            f32v[offset] = rotation;
            f32v[offset + stride] = rotation;
            f32v[offset + (stride * 2)] = rotation;
            f32v[offset + (stride * 3)] = rotation;
        `,
      dynamic: false
    },
    uvs: {
      attributeName: "aUV",
      format: "float32x2",
      code: `
            var uvs = p.texture.uvs;

            f32v[offset] = uvs.x0;
            f32v[offset + 1] = uvs.y0;

            f32v[offset + stride] = uvs.x1;
            f32v[offset + stride + 1] = uvs.y1;

            f32v[offset + (stride * 2)] = uvs.x2;
            f32v[offset + (stride * 2) + 1] = uvs.y2;

            f32v[offset + (stride * 3)] = uvs.x3;
            f32v[offset + (stride * 3) + 1] = uvs.y3;
        `,
      dynamic: false
    },
    color: {
      attributeName: "aColor",
      format: "unorm8x4",
      code: `
            const c = p.color;

            u32v[offset] = c;
            u32v[offset + stride] = c;
            u32v[offset + (stride * 2)] = c;
            u32v[offset + (stride * 3)] = c;
        `,
      dynamic: false
    }
  };
  Ot.add(Qh);
  Ot.add(td);
  Ot.add(Zh);
  const Ry = new Se(0, 0, 0, 0), nd = class _o extends Vi {
    constructor(t = {}) {
      t = {
        ..._o.defaultOptions,
        ...t,
        dynamicProperties: {
          ..._o.defaultOptions.dynamicProperties,
          ...t == null ? void 0 : t.dynamicProperties
        }
      };
      const { dynamicProperties: e, shader: s, roundPixels: i, texture: r, particles: o, ...a } = t;
      super({
        label: "ParticleContainer",
        ...a
      }), this.renderPipeId = "particle", this.batched = false, this._childrenDirty = false, this.texture = r || null, this.shader = s, this._properties = {};
      for (const l in Rl) {
        const c = Rl[l], h = e[l];
        this._properties[l] = {
          ...c,
          dynamic: h
        };
      }
      this.allowChildren = true, this.roundPixels = i ?? false, this.particleChildren = o ?? [];
    }
    addParticle(...t) {
      for (let e = 0; e < t.length; e++) this.particleChildren.push(t[e]);
      return this.onViewUpdate(), t[0];
    }
    removeParticle(...t) {
      let e = false;
      for (let s = 0; s < t.length; s++) {
        const i = this.particleChildren.indexOf(t[s]);
        i > -1 && (this.particleChildren.splice(i, 1), e = true);
      }
      return e && this.onViewUpdate(), t[0];
    }
    update() {
      this._childrenDirty = true;
    }
    onViewUpdate() {
      this._childrenDirty = true, super.onViewUpdate();
    }
    get bounds() {
      return Ry;
    }
    updateBounds() {
    }
    destroy(t = false) {
      var _a2, _b2;
      if (super.destroy(t), typeof t == "boolean" ? t : t == null ? void 0 : t.texture) {
        const s = typeof t == "boolean" ? t : t == null ? void 0 : t.textureSource, i = this.texture ?? ((_a2 = this.particleChildren[0]) == null ? void 0 : _a2.texture);
        i && i.destroy(s);
      }
      this.texture = null, (_b2 = this.shader) == null ? void 0 : _b2.destroy();
    }
    removeParticles(t, e) {
      t ?? (t = 0), e ?? (e = this.particleChildren.length);
      const s = this.particleChildren.splice(t, e - t);
      return this.onViewUpdate(), s;
    }
    removeParticleAt(t) {
      const e = this.particleChildren.splice(t, 1);
      return this.onViewUpdate(), e[0];
    }
    addParticleAt(t, e) {
      return this.particleChildren.splice(e, 0, t), this.onViewUpdate(), t;
    }
    addChild(...t) {
      throw new Error("ParticleContainer.addChild() is not available. Please use ParticleContainer.addParticle()");
    }
    removeChild(...t) {
      throw new Error("ParticleContainer.removeChild() is not available. Please use ParticleContainer.removeParticle()");
    }
    removeChildren(t, e) {
      throw new Error("ParticleContainer.removeChildren() is not available. Please use ParticleContainer.removeParticles()");
    }
    removeChildAt(t) {
      throw new Error("ParticleContainer.removeChildAt() is not available. Please use ParticleContainer.removeParticleAt()");
    }
    getChildAt(t) {
      throw new Error("ParticleContainer.getChildAt() is not available. Please use ParticleContainer.getParticleAt()");
    }
    setChildIndex(t, e) {
      throw new Error("ParticleContainer.setChildIndex() is not available. Please use ParticleContainer.setParticleIndex()");
    }
    getChildIndex(t) {
      throw new Error("ParticleContainer.getChildIndex() is not available. Please use ParticleContainer.getParticleIndex()");
    }
    addChildAt(t, e) {
      throw new Error("ParticleContainer.addChildAt() is not available. Please use ParticleContainer.addParticleAt()");
    }
    swapChildren(t, e) {
      throw new Error("ParticleContainer.swapChildren() is not available. Please use ParticleContainer.swapParticles()");
    }
    reparentChild(...t) {
      throw new Error("ParticleContainer.reparentChild() is not available with the particle container");
    }
    reparentChildAt(t, e) {
      throw new Error("ParticleContainer.reparentChildAt() is not available with the particle container");
    }
  };
  nd.defaultOptions = {
    dynamicProperties: {
      vertex: false,
      position: true,
      rotation: false,
      uvs: false,
      color: false
    },
    roundPixels: false
  };
  let Ly = nd;
  Ot.add(ru, ou);
  var $y = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
  function By(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var sd = {
    exports: {}
  };
  (function(n, t) {
    (function() {
      var e, s;
      s = function(i) {
        return n.exports = i;
      }, e = {
        linear: function(i, r, o, a) {
          return o * i / a + r;
        },
        easeInQuad: function(i, r, o, a) {
          return o * (i /= a) * i + r;
        },
        easeOutQuad: function(i, r, o, a) {
          return -o * (i /= a) * (i - 2) + r;
        },
        easeInOutQuad: function(i, r, o, a) {
          return (i /= a / 2) < 1 ? o / 2 * i * i + r : -o / 2 * (--i * (i - 2) - 1) + r;
        },
        easeInCubic: function(i, r, o, a) {
          return o * (i /= a) * i * i + r;
        },
        easeOutCubic: function(i, r, o, a) {
          return o * ((i = i / a - 1) * i * i + 1) + r;
        },
        easeInOutCubic: function(i, r, o, a) {
          return (i /= a / 2) < 1 ? o / 2 * i * i * i + r : o / 2 * ((i -= 2) * i * i + 2) + r;
        },
        easeInQuart: function(i, r, o, a) {
          return o * (i /= a) * i * i * i + r;
        },
        easeOutQuart: function(i, r, o, a) {
          return -o * ((i = i / a - 1) * i * i * i - 1) + r;
        },
        easeInOutQuart: function(i, r, o, a) {
          return (i /= a / 2) < 1 ? o / 2 * i * i * i * i + r : -o / 2 * ((i -= 2) * i * i * i - 2) + r;
        },
        easeInQuint: function(i, r, o, a) {
          return o * (i /= a) * i * i * i * i + r;
        },
        easeOutQuint: function(i, r, o, a) {
          return o * ((i = i / a - 1) * i * i * i * i + 1) + r;
        },
        easeInOutQuint: function(i, r, o, a) {
          return (i /= a / 2) < 1 ? o / 2 * i * i * i * i * i + r : o / 2 * ((i -= 2) * i * i * i * i + 2) + r;
        },
        easeInSine: function(i, r, o, a) {
          return -o * Math.cos(i / a * (Math.PI / 2)) + o + r;
        },
        easeOutSine: function(i, r, o, a) {
          return o * Math.sin(i / a * (Math.PI / 2)) + r;
        },
        easeInOutSine: function(i, r, o, a) {
          return -o / 2 * (Math.cos(Math.PI * i / a) - 1) + r;
        },
        easeInExpo: function(i, r, o, a) {
          return i === 0 ? r : o * Math.pow(2, 10 * (i / a - 1)) + r;
        },
        easeOutExpo: function(i, r, o, a) {
          return i === a ? r + o : o * (-Math.pow(2, -10 * i / a) + 1) + r;
        },
        easeInOutExpo: function(i, r, o, a) {
          return (i /= a / 2) < 1 ? o / 2 * Math.pow(2, 10 * (i - 1)) + r : o / 2 * (-Math.pow(2, -10 * --i) + 2) + r;
        },
        easeInCirc: function(i, r, o, a) {
          return -o * (Math.sqrt(1 - (i /= a) * i) - 1) + r;
        },
        easeOutCirc: function(i, r, o, a) {
          return o * Math.sqrt(1 - (i = i / a - 1) * i) + r;
        },
        easeInOutCirc: function(i, r, o, a) {
          return (i /= a / 2) < 1 ? -o / 2 * (Math.sqrt(1 - i * i) - 1) + r : o / 2 * (Math.sqrt(1 - (i -= 2) * i) + 1) + r;
        },
        easeInElastic: function(i, r, o, a) {
          var l, c, h;
          return h = 1.70158, c = 0, l = o, i === 0 || (i /= a), c || (c = a * 0.3), l < Math.abs(o) ? (l = o, h = c / 4) : h = c / (2 * Math.PI) * Math.asin(o / l), -(l * Math.pow(2, 10 * (i -= 1)) * Math.sin((i * a - h) * (2 * Math.PI) / c)) + r;
        },
        easeOutElastic: function(i, r, o, a) {
          var l, c, h;
          return h = 1.70158, c = 0, l = o, i === 0 || (i /= a), c || (c = a * 0.3), l < Math.abs(o) ? (l = o, h = c / 4) : h = c / (2 * Math.PI) * Math.asin(o / l), l * Math.pow(2, -10 * i) * Math.sin((i * a - h) * (2 * Math.PI) / c) + o + r;
        },
        easeInOutElastic: function(i, r, o, a) {
          var l, c, h;
          return h = 1.70158, c = 0, l = o, i === 0 || (i /= a / 2), c || (c = a * (0.3 * 1.5)), l < Math.abs(o) ? (l = o, h = c / 4) : h = c / (2 * Math.PI) * Math.asin(o / l), i < 1 ? -0.5 * (l * Math.pow(2, 10 * (i -= 1)) * Math.sin((i * a - h) * (2 * Math.PI) / c)) + r : l * Math.pow(2, -10 * (i -= 1)) * Math.sin((i * a - h) * (2 * Math.PI) / c) * 0.5 + o + r;
        },
        easeInBack: function(i, r, o, a, l) {
          return l === void 0 && (l = 1.70158), o * (i /= a) * i * ((l + 1) * i - l) + r;
        },
        easeOutBack: function(i, r, o, a, l) {
          return l === void 0 && (l = 1.70158), o * ((i = i / a - 1) * i * ((l + 1) * i + l) + 1) + r;
        },
        easeInOutBack: function(i, r, o, a, l) {
          return l === void 0 && (l = 1.70158), (i /= a / 2) < 1 ? o / 2 * (i * i * (((l *= 1.525) + 1) * i - l)) + r : o / 2 * ((i -= 2) * i * (((l *= 1.525) + 1) * i + l) + 2) + r;
        },
        easeInBounce: function(i, r, o, a) {
          var l;
          return l = e.easeOutBounce(a - i, 0, o, a), o - l + r;
        },
        easeOutBounce: function(i, r, o, a) {
          return (i /= a) < 1 / 2.75 ? o * (7.5625 * i * i) + r : i < 2 / 2.75 ? o * (7.5625 * (i -= 1.5 / 2.75) * i + 0.75) + r : i < 2.5 / 2.75 ? o * (7.5625 * (i -= 2.25 / 2.75) * i + 0.9375) + r : o * (7.5625 * (i -= 2.625 / 2.75) * i + 0.984375) + r;
        },
        easeInOutBounce: function(i, r, o, a) {
          var l;
          return i < a / 2 ? (l = e.easeInBounce(i * 2, 0, o, a), l * 0.5 + r) : (l = e.easeOutBounce(i * 2 - a, 0, o, a), l * 0.5 + o * 0.5 + r);
        }
      }, s(e);
    }).call($y);
  })(sd);
  var Oy = sd.exports;
  const Ll = By(Oy);
  function tr(n, t) {
    if (n) {
      if (typeof n == "function") return n;
      if (typeof n == "string") return Ll[n];
    } else return Ll[t];
  }
  class Fy {
    constructor(t) {
      this.viewport = t, this.touches = [], this.addListeners();
    }
    addListeners() {
      this.viewport.eventMode = "static", this.viewport.forceHitArea || (this.viewport.hitArea = new Nt(0, 0, this.viewport.worldWidth, this.viewport.worldHeight)), this.viewport.on("pointerdown", this.down, this), this.viewport.options.allowPreserveDragOutside ? this.viewport.on("globalpointermove", this.move, this) : this.viewport.on("pointermove", this.move, this), this.viewport.on("pointerup", this.up, this), this.viewport.on("pointerupoutside", this.up, this), this.viewport.on("pointercancel", this.up, this), this.viewport.options.allowPreserveDragOutside || this.viewport.on("pointerleave", this.up, this), this.wheelFunction = (t) => this.handleWheel(t), this.viewport.options.events.domElement.addEventListener("wheel", this.wheelFunction, {
        passive: this.viewport.options.passiveWheel
      }), this.isMouseDown = false;
    }
    destroy() {
      var t;
      (t = this.viewport.options.events.domElement) == null || t.removeEventListener("wheel", this.wheelFunction);
    }
    down(t) {
      if (!(this.viewport.pause || !this.viewport.visible)) {
        if (t.pointerType === "mouse" ? this.isMouseDown = true : this.get(t.pointerId) || this.touches.push({
          id: t.pointerId,
          last: null
        }), this.count() === 1) {
          this.last = t.global.clone();
          const e = this.viewport.plugins.get("decelerate", true), s = this.viewport.plugins.get("bounce", true);
          (!e || !e.isActive()) && (!s || !s.isActive()) ? this.clickedAvailable = true : this.clickedAvailable = false;
        } else this.clickedAvailable = false;
        this.viewport.plugins.down(t) && this.viewport.options.stopPropagation && t.stopPropagation();
      }
    }
    clear() {
      this.isMouseDown = false, this.touches = [], this.last = null;
    }
    checkThreshold(t) {
      return Math.abs(t) >= this.viewport.threshold;
    }
    move(t) {
      if (this.viewport.pause || !this.viewport.visible) return;
      const e = this.viewport.plugins.move(t);
      if (this.clickedAvailable && this.last) {
        const s = t.global.x - this.last.x, i = t.global.y - this.last.y;
        (this.checkThreshold(s) || this.checkThreshold(i)) && (this.clickedAvailable = false);
      }
      e && this.viewport.options.stopPropagation && t.stopPropagation();
    }
    up(t) {
      if (this.viewport.pause || !this.viewport.visible) return;
      t.pointerType === "mouse" && (this.isMouseDown = false), t.pointerType !== "mouse" && this.remove(t.pointerId);
      const e = this.viewport.plugins.up(t);
      this.clickedAvailable && this.count() === 0 && this.last && (this.viewport.emit("clicked", {
        event: t,
        screen: this.last,
        world: this.viewport.toWorld(this.last),
        viewport: this.viewport
      }), this.clickedAvailable = false), e && this.viewport.options.stopPropagation && t.stopPropagation();
    }
    getPointerPosition(t) {
      const e = new At();
      return this.viewport.options.events.mapPositionToPoint(e, t.clientX, t.clientY), e;
    }
    handleWheel(t) {
      if (this.viewport.pause || !this.viewport.visible) return;
      const e = this.viewport.toLocal(this.getPointerPosition(t));
      this.viewport.left <= e.x && e.x <= this.viewport.right && this.viewport.top <= e.y && e.y <= this.viewport.bottom && this.viewport.plugins.wheel(t) && !this.viewport.options.passiveWheel && t.preventDefault();
    }
    pause() {
      this.touches = [], this.isMouseDown = false;
    }
    get(t) {
      for (const e of this.touches) if (e.id === t) return e;
      return null;
    }
    remove(t) {
      for (let e = 0; e < this.touches.length; e++) if (this.touches[e].id === t) {
        this.touches.splice(e, 1);
        return;
      }
    }
    count() {
      return (this.isMouseDown ? 1 : 0) + this.touches.length;
    }
  }
  const Ms = [
    "drag",
    "pinch",
    "wheel",
    "follow",
    "mouse-edges",
    "decelerate",
    "animate",
    "bounce",
    "snap-zoom",
    "clamp-zoom",
    "snap",
    "clamp"
  ];
  class Ny {
    constructor(t) {
      this.viewport = t, this.list = [], this.plugins = {};
    }
    add(t, e, s = Ms.length) {
      const i = this.plugins[t];
      i && i.destroy(), this.plugins[t] = e;
      const r = Ms.indexOf(t);
      r !== -1 && Ms.splice(r, 1), Ms.splice(s, 0, t), this.sort();
    }
    get(t, e) {
      var s;
      return e && (s = this.plugins[t]) != null && s.paused ? null : this.plugins[t];
    }
    update(t) {
      for (const e of this.list) e.update(t);
    }
    resize() {
      for (const t of this.list) t.resize();
    }
    reset() {
      for (const t of this.list) t.reset();
    }
    removeAll() {
      this.list.forEach((t) => {
        t.destroy();
      }), this.plugins = {}, this.sort();
    }
    remove(t) {
      var e;
      this.plugins[t] && ((e = this.plugins[t]) == null || e.destroy(), delete this.plugins[t], this.viewport.emit("plugin-remove", t), this.sort());
    }
    pause(t) {
      var e;
      (e = this.plugins[t]) == null || e.pause();
    }
    resume(t) {
      var e;
      (e = this.plugins[t]) == null || e.resume();
    }
    sort() {
      this.list = [];
      for (const t of Ms) this.plugins[t] && this.list.push(this.plugins[t]);
    }
    down(t) {
      let e = false;
      for (const s of this.list) s.down(t) && (e = true);
      return e;
    }
    move(t) {
      let e = false;
      for (const s of this.viewport.plugins.list) s.move(t) && (e = true);
      return e;
    }
    up(t) {
      let e = false;
      for (const s of this.list) s.up(t) && (e = true);
      return e;
    }
    wheel(t) {
      let e = false;
      for (const s of this.list) s.wheel(t) && (e = true);
      return e;
    }
  }
  class $e {
    constructor(t) {
      this.parent = t, this.paused = false;
    }
    destroy() {
    }
    down(t) {
      return false;
    }
    move(t) {
      return false;
    }
    up(t) {
      return false;
    }
    wheel(t) {
      return false;
    }
    update(t) {
    }
    resize() {
    }
    reset() {
    }
    pause() {
      this.paused = true;
    }
    resume() {
      this.paused = false;
    }
  }
  const Wy = {
    removeOnInterrupt: false,
    ease: "linear",
    time: 1e3
  };
  class Gy extends $e {
    constructor(t, e = {}) {
      super(t), this.startWidth = null, this.startHeight = null, this.deltaWidth = null, this.deltaHeight = null, this.width = null, this.height = null, this.time = 0, this.options = Object.assign({}, Wy, e), this.options.ease = tr(this.options.ease), this.setupPosition(), this.setupZoom(), this.time = 0;
    }
    setupPosition() {
      typeof this.options.position < "u" ? (this.startX = this.parent.center.x, this.startY = this.parent.center.y, this.deltaX = this.options.position.x - this.parent.center.x, this.deltaY = this.options.position.y - this.parent.center.y, this.keepCenter = false) : this.keepCenter = true;
    }
    setupZoom() {
      this.width = null, this.height = null, typeof this.options.scale < "u" ? this.width = this.parent.screenWidth / this.options.scale : typeof this.options.scaleX < "u" || typeof this.options.scaleY < "u" ? (typeof this.options.scaleX < "u" && (this.width = this.parent.screenWidth / this.options.scaleX), typeof this.options.scaleY < "u" && (this.height = this.parent.screenHeight / this.options.scaleY)) : (typeof this.options.width < "u" && (this.width = this.options.width), typeof this.options.height < "u" && (this.height = this.options.height)), this.width !== null && (this.startWidth = this.parent.screenWidthInWorldPixels, this.deltaWidth = this.width - this.startWidth), this.height !== null && (this.startHeight = this.parent.screenHeightInWorldPixels, this.deltaHeight = this.height - this.startHeight);
    }
    down() {
      return this.options.removeOnInterrupt && this.parent.plugins.remove("animate"), false;
    }
    complete() {
      this.parent.plugins.remove("animate"), this.width !== null && this.parent.fitWidth(this.width, this.keepCenter, this.height === null), this.height !== null && this.parent.fitHeight(this.height, this.keepCenter, this.width === null), !this.keepCenter && this.options.position && this.parent.moveCenter(this.options.position), this.parent.emit("animate-end", this.parent), this.options.callbackOnComplete && this.options.callbackOnComplete(this.parent);
    }
    update(t) {
      if (this.paused) return;
      this.time += t;
      const e = new At(this.parent.scale.x, this.parent.scale.y);
      if (this.time >= this.options.time) {
        const s = this.parent.width, i = this.parent.height;
        this.complete(), (s !== this.parent.width || i !== this.parent.height) && this.parent.emit("zoomed", {
          viewport: this.parent,
          original: e,
          type: "animate"
        });
      } else {
        const s = this.options.ease(this.time, 0, 1, this.options.time);
        if (this.width !== null) {
          const i = this.startWidth, r = this.deltaWidth;
          this.parent.fitWidth(i + r * s, this.keepCenter, this.height === null);
        }
        if (this.height !== null) {
          const i = this.startHeight, r = this.deltaHeight;
          this.parent.fitHeight(i + r * s, this.keepCenter, this.width === null);
        }
        if (this.width === null ? this.parent.scale.x = this.parent.scale.y : this.height === null && (this.parent.scale.y = this.parent.scale.x), !this.keepCenter) {
          const i = this.startX, r = this.startY, o = this.deltaX, a = this.deltaY, l = new At(this.parent.x, this.parent.y);
          this.parent.moveCenter(i + o * s, r + a * s), this.parent.emit("moved", {
            viewport: this.parent,
            original: l,
            type: "animate"
          });
        }
        (this.width || this.height) && this.parent.emit("zoomed", {
          viewport: this.parent,
          original: e,
          type: "animate"
        });
      }
    }
  }
  const zy = {
    sides: "all",
    friction: 0.5,
    time: 150,
    ease: "easeInOutSine",
    underflow: "center",
    bounceBox: null
  };
  class Dy extends $e {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, zy, e), this.ease = tr(this.options.ease, "easeInOutSine"), this.options.sides ? this.options.sides === "all" ? this.top = this.bottom = this.left = this.right = true : this.options.sides === "horizontal" ? (this.right = this.left = true, this.top = this.bottom = false) : this.options.sides === "vertical" ? (this.left = this.right = false, this.top = this.bottom = true) : (this.top = this.options.sides.indexOf("top") !== -1, this.bottom = this.options.sides.indexOf("bottom") !== -1, this.left = this.options.sides.indexOf("left") !== -1, this.right = this.options.sides.indexOf("right") !== -1) : this.left = this.top = this.right = this.bottom = false;
      const s = this.options.underflow.toLowerCase();
      s === "center" ? (this.underflowX = 0, this.underflowY = 0) : (this.underflowX = s.indexOf("left") !== -1 ? -1 : s.indexOf("right") !== -1 ? 1 : 0, this.underflowY = s.indexOf("top") !== -1 ? -1 : s.indexOf("bottom") !== -1 ? 1 : 0), this.reset();
    }
    isActive() {
      return this.toX !== null || this.toY !== null;
    }
    down() {
      return this.toX = this.toY = null, false;
    }
    up() {
      return this.bounce(), false;
    }
    update(t) {
      if (!this.paused) {
        if (this.bounce(), this.toX) {
          const e = this.toX;
          e.time += t, this.parent.emit("moved", {
            viewport: this.parent,
            type: "bounce-x"
          }), e.time >= this.options.time ? (this.parent.x = e.end, this.toX = null, this.parent.emit("bounce-x-end", this.parent)) : this.parent.x = this.ease(e.time, e.start, e.delta, this.options.time);
        }
        if (this.toY) {
          const e = this.toY;
          e.time += t, this.parent.emit("moved", {
            viewport: this.parent,
            type: "bounce-y"
          }), e.time >= this.options.time ? (this.parent.y = e.end, this.toY = null, this.parent.emit("bounce-y-end", this.parent)) : this.parent.y = this.ease(e.time, e.start, e.delta, this.options.time);
        }
      }
    }
    calcUnderflowX() {
      let t;
      switch (this.underflowX) {
        case -1:
          t = 0;
          break;
        case 1:
          t = this.parent.screenWidth - this.parent.screenWorldWidth;
          break;
        default:
          t = (this.parent.screenWidth - this.parent.screenWorldWidth) / 2;
      }
      return t;
    }
    calcUnderflowY() {
      let t;
      switch (this.underflowY) {
        case -1:
          t = 0;
          break;
        case 1:
          t = this.parent.screenHeight - this.parent.screenWorldHeight;
          break;
        default:
          t = (this.parent.screenHeight - this.parent.screenWorldHeight) / 2;
      }
      return t;
    }
    oob() {
      const t = this.options.bounceBox;
      if (t) {
        const e = typeof t.x > "u" ? 0 : t.x, s = typeof t.y > "u" ? 0 : t.y, i = typeof t.width > "u" ? this.parent.worldWidth : t.width, r = typeof t.height > "u" ? this.parent.worldHeight : t.height;
        return {
          left: this.parent.left < e,
          right: this.parent.right > i,
          top: this.parent.top < s,
          bottom: this.parent.bottom > r,
          topLeft: new At(e * this.parent.scale.x, s * this.parent.scale.y),
          bottomRight: new At(i * this.parent.scale.x - this.parent.screenWidth, r * this.parent.scale.y - this.parent.screenHeight)
        };
      }
      return {
        left: this.parent.left < 0,
        right: this.parent.right > this.parent.worldWidth,
        top: this.parent.top < 0,
        bottom: this.parent.bottom > this.parent.worldHeight,
        topLeft: new At(0, 0),
        bottomRight: new At(this.parent.worldWidth * this.parent.scale.x - this.parent.screenWidth, this.parent.worldHeight * this.parent.scale.y - this.parent.screenHeight)
      };
    }
    bounce() {
      var t, e;
      if (this.paused) return;
      let s, i = this.parent.plugins.get("decelerate", true);
      i && (i.x || i.y) && (i.x && i.percentChangeX === ((t = i.options) == null ? void 0 : t.friction) || i.y && i.percentChangeY === ((e = i.options) == null ? void 0 : e.friction)) && (s = this.oob(), (s.left && this.left || s.right && this.right) && (i.percentChangeX = this.options.friction), (s.top && this.top || s.bottom && this.bottom) && (i.percentChangeY = this.options.friction));
      const r = this.parent.plugins.get("drag", true) || {}, o = this.parent.plugins.get("pinch", true) || {};
      if (i = i || {}, !(r != null && r.active) && !(o != null && o.active) && (!this.toX || !this.toY) && (!i.x || !i.y)) {
        s = s || this.oob();
        const a = s.topLeft, l = s.bottomRight;
        if (!this.toX && !i.x) {
          let c = null;
          s.left && this.left ? c = this.parent.screenWorldWidth < this.parent.screenWidth ? this.calcUnderflowX() : -a.x : s.right && this.right && (c = this.parent.screenWorldWidth < this.parent.screenWidth ? this.calcUnderflowX() : -l.x), c !== null && this.parent.x !== c && (this.toX = {
            time: 0,
            start: this.parent.x,
            delta: c - this.parent.x,
            end: c
          }, this.parent.emit("bounce-x-start", this.parent));
        }
        if (!this.toY && !i.y) {
          let c = null;
          s.top && this.top ? c = this.parent.screenWorldHeight < this.parent.screenHeight ? this.calcUnderflowY() : -a.y : s.bottom && this.bottom && (c = this.parent.screenWorldHeight < this.parent.screenHeight ? this.calcUnderflowY() : -l.y), c !== null && this.parent.y !== c && (this.toY = {
            time: 0,
            start: this.parent.y,
            delta: c - this.parent.y,
            end: c
          }, this.parent.emit("bounce-y-start", this.parent));
        }
      }
    }
    reset() {
      this.toX = this.toY = null, this.bounce();
    }
  }
  const Hy = {
    left: false,
    right: false,
    top: false,
    bottom: false,
    direction: null,
    underflow: "center"
  };
  class Uy extends $e {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Hy, e), this.options.direction && (this.options.left = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.right = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.top = this.options.direction === "y" || this.options.direction === "all" ? true : null, this.options.bottom = this.options.direction === "y" || this.options.direction === "all" ? true : null), this.parseUnderflow(), this.last = {
        x: null,
        y: null,
        scaleX: null,
        scaleY: null
      }, this.update();
    }
    parseUnderflow() {
      const t = this.options.underflow.toLowerCase();
      t === "none" ? this.noUnderflow = true : t === "center" ? (this.underflowX = this.underflowY = 0, this.noUnderflow = false) : (this.underflowX = t.indexOf("left") !== -1 ? -1 : t.indexOf("right") !== -1 ? 1 : 0, this.underflowY = t.indexOf("top") !== -1 ? -1 : t.indexOf("bottom") !== -1 ? 1 : 0, this.noUnderflow = false);
    }
    move() {
      return this.update(), false;
    }
    update() {
      if (this.paused || this.parent.x === this.last.x && this.parent.y === this.last.y && this.parent.scale.x === this.last.scaleX && this.parent.scale.y === this.last.scaleY) return;
      const t = new At(this.parent.x, this.parent.y), e = this.parent.plugins.decelerate || {};
      if (this.options.left !== null || this.options.right !== null) {
        let s = false;
        if (!this.noUnderflow && this.parent.screenWorldWidth < this.parent.screenWidth) switch (this.underflowX) {
          case -1:
            this.parent.x !== 0 && (this.parent.x = 0, s = true);
            break;
          case 1:
            this.parent.x !== this.parent.screenWidth - this.parent.screenWorldWidth && (this.parent.x = this.parent.screenWidth - this.parent.screenWorldWidth, s = true);
            break;
          default:
            this.parent.x !== (this.parent.screenWidth - this.parent.screenWorldWidth) / 2 && (this.parent.x = (this.parent.screenWidth - this.parent.screenWorldWidth) / 2, s = true);
        }
        else this.options.left !== null && this.parent.left < (this.options.left === true ? 0 : this.options.left) && (this.parent.x = -(this.options.left === true ? 0 : this.options.left) * this.parent.scale.x, e.x = 0, s = true), this.options.right !== null && this.parent.right > (this.options.right === true ? this.parent.worldWidth : this.options.right) && (this.parent.x = -(this.options.right === true ? this.parent.worldWidth : this.options.right) * this.parent.scale.x + this.parent.screenWidth, e.x = 0, s = true);
        s && this.parent.emit("moved", {
          viewport: this.parent,
          original: t,
          type: "clamp-x"
        });
      }
      if (this.options.top !== null || this.options.bottom !== null) {
        let s = false;
        if (!this.noUnderflow && this.parent.screenWorldHeight < this.parent.screenHeight) switch (this.underflowY) {
          case -1:
            this.parent.y !== 0 && (this.parent.y = 0, s = true);
            break;
          case 1:
            this.parent.y !== this.parent.screenHeight - this.parent.screenWorldHeight && (this.parent.y = this.parent.screenHeight - this.parent.screenWorldHeight, s = true);
            break;
          default:
            this.parent.y !== (this.parent.screenHeight - this.parent.screenWorldHeight) / 2 && (this.parent.y = (this.parent.screenHeight - this.parent.screenWorldHeight) / 2, s = true);
        }
        else this.options.top !== null && this.parent.top < (this.options.top === true ? 0 : this.options.top) && (this.parent.y = -(this.options.top === true ? 0 : this.options.top) * this.parent.scale.y, e.y = 0, s = true), this.options.bottom !== null && this.parent.bottom > (this.options.bottom === true ? this.parent.worldHeight : this.options.bottom) && (this.parent.y = -(this.options.bottom === true ? this.parent.worldHeight : this.options.bottom) * this.parent.scale.y + this.parent.screenHeight, e.y = 0, s = true);
        s && this.parent.emit("moved", {
          viewport: this.parent,
          original: t,
          type: "clamp-y"
        });
      }
      this.last.x = this.parent.x, this.last.y = this.parent.y, this.last.scaleX = this.parent.scale.x, this.last.scaleY = this.parent.scale.y;
    }
    reset() {
      this.update();
    }
  }
  const jy = {
    minWidth: null,
    minHeight: null,
    maxWidth: null,
    maxHeight: null,
    minScale: null,
    maxScale: null
  };
  class Vy extends $e {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, jy, e), this.clamp();
    }
    resize() {
      this.clamp();
    }
    clamp() {
      if (!this.paused) {
        if (this.options.minWidth || this.options.minHeight || this.options.maxWidth || this.options.maxHeight) {
          let t = this.parent.worldScreenWidth, e = this.parent.worldScreenHeight;
          if (this.options.minWidth !== null && t < this.options.minWidth) {
            const s = this.parent.scale.x;
            this.parent.fitWidth(this.options.minWidth, false, false, true), this.parent.scale.y *= this.parent.scale.x / s, t = this.parent.worldScreenWidth, e = this.parent.worldScreenHeight, this.parent.emit("zoomed", {
              viewport: this.parent,
              type: "clamp-zoom"
            });
          }
          if (this.options.maxWidth !== null && t > this.options.maxWidth) {
            const s = this.parent.scale.x;
            this.parent.fitWidth(this.options.maxWidth, false, false, true), this.parent.scale.y *= this.parent.scale.x / s, t = this.parent.worldScreenWidth, e = this.parent.worldScreenHeight, this.parent.emit("zoomed", {
              viewport: this.parent,
              type: "clamp-zoom"
            });
          }
          if (this.options.minHeight !== null && e < this.options.minHeight) {
            const s = this.parent.scale.y;
            this.parent.fitHeight(this.options.minHeight, false, false, true), this.parent.scale.x *= this.parent.scale.y / s, t = this.parent.worldScreenWidth, e = this.parent.worldScreenHeight, this.parent.emit("zoomed", {
              viewport: this.parent,
              type: "clamp-zoom"
            });
          }
          if (this.options.maxHeight !== null && e > this.options.maxHeight) {
            const s = this.parent.scale.y;
            this.parent.fitHeight(this.options.maxHeight, false, false, true), this.parent.scale.x *= this.parent.scale.y / s, this.parent.emit("zoomed", {
              viewport: this.parent,
              type: "clamp-zoom"
            });
          }
        } else if (this.options.minScale || this.options.maxScale) {
          const t = {
            x: null,
            y: null
          }, e = {
            x: null,
            y: null
          };
          if (typeof this.options.minScale == "number") t.x = this.options.minScale, t.y = this.options.minScale;
          else if (this.options.minScale !== null) {
            const r = this.options.minScale;
            t.x = typeof r.x > "u" ? null : r.x, t.y = typeof r.y > "u" ? null : r.y;
          }
          if (typeof this.options.maxScale == "number") e.x = this.options.maxScale, e.y = this.options.maxScale;
          else if (this.options.maxScale !== null) {
            const r = this.options.maxScale;
            e.x = typeof r.x > "u" ? null : r.x, e.y = typeof r.y > "u" ? null : r.y;
          }
          let s = this.parent.scale.x, i = this.parent.scale.y;
          t.x !== null && s < t.x && (s = t.x), e.x !== null && s > e.x && (s = e.x), t.y !== null && i < t.y && (i = t.y), e.y !== null && i > e.y && (i = e.y), (s !== this.parent.scale.x || i !== this.parent.scale.y) && (this.parent.scale.set(s, i), this.parent.emit("zoomed", {
            viewport: this.parent,
            type: "clamp-zoom"
          }));
        }
      }
    }
    reset() {
      this.clamp();
    }
  }
  const Yy = {
    friction: 0.98,
    bounce: 0.8,
    minSpeed: 0.01
  }, gn = 16;
  class Xy extends $e {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Yy, e), this.saved = [], this.timeSinceRelease = 0, this.reset(), this.parent.on("moved", (s) => this.handleMoved(s));
    }
    down() {
      return this.saved = [], this.x = this.y = null, false;
    }
    isActive() {
      return !!(this.x || this.y);
    }
    move() {
      if (this.paused) return false;
      const t = this.parent.input.count();
      return (t === 1 || t > 1 && !this.parent.plugins.get("pinch", true)) && (this.saved.push({
        x: this.parent.x,
        y: this.parent.y,
        time: performance.now()
      }), this.saved.length > 60 && this.saved.splice(0, 30)), false;
    }
    handleMoved(t) {
      if (this.saved.length) {
        const e = this.saved[this.saved.length - 1];
        t.type === "clamp-x" && t.original ? e.x === t.original.x && (e.x = this.parent.x) : t.type === "clamp-y" && t.original && e.y === t.original.y && (e.y = this.parent.y);
      }
    }
    up() {
      if (this.parent.input.count() === 0 && this.saved.length) {
        const t = performance.now();
        for (const e of this.saved) if (e.time >= t - 100) {
          const s = t - e.time;
          this.x = (this.parent.x - e.x) / s, this.y = (this.parent.y - e.y) / s, this.percentChangeX = this.percentChangeY = this.options.friction, this.timeSinceRelease = 0;
          break;
        }
      }
      return false;
    }
    activate(t) {
      t = t || {}, typeof t.x < "u" && (this.x = t.x, this.percentChangeX = this.options.friction), typeof t.y < "u" && (this.y = t.y, this.percentChangeY = this.options.friction);
    }
    update(t) {
      if (this.paused) return;
      const e = this.x || this.y, s = this.timeSinceRelease, i = this.timeSinceRelease + t;
      if (this.x) {
        const r = this.percentChangeX, o = Math.log(r);
        this.parent.x += this.x * gn / o * (Math.pow(r, i / gn) - Math.pow(r, s / gn)), this.x *= Math.pow(this.percentChangeX, t / gn);
      }
      if (this.y) {
        const r = this.percentChangeY, o = Math.log(r);
        this.parent.y += this.y * gn / o * (Math.pow(r, i / gn) - Math.pow(r, s / gn)), this.y *= Math.pow(this.percentChangeY, t / gn);
      }
      this.timeSinceRelease += t, this.x && this.y ? Math.abs(this.x) < this.options.minSpeed && Math.abs(this.y) < this.options.minSpeed && (this.x = 0, this.y = 0) : (Math.abs(this.x || 0) < this.options.minSpeed && (this.x = 0), Math.abs(this.y || 0) < this.options.minSpeed && (this.y = 0)), e && this.parent.emit("moved", {
        viewport: this.parent,
        type: "decelerate"
      });
    }
    reset() {
      this.x = this.y = null;
    }
  }
  const qy = {
    direction: "all",
    pressDrag: true,
    wheel: true,
    wheelScroll: 1,
    reverse: false,
    clampWheel: false,
    underflow: "center",
    factor: 1,
    mouseButtons: "all",
    keyToPress: null,
    ignoreKeyToPressOnTouch: false,
    lineHeight: 20,
    wheelSwapAxes: false
  };
  class Ky extends $e {
    constructor(t, e = {}) {
      super(t), this.windowEventHandlers = [], this.options = Object.assign({}, qy, e), this.moved = false, this.reverse = this.options.reverse ? 1 : -1, this.xDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "x", this.yDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "y", this.keyIsPressed = false, this.parseUnderflow(), this.mouseButtons(this.options.mouseButtons), this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
    }
    handleKeyPresses(t) {
      const e = (i) => {
        t.includes(i.code) && (this.keyIsPressed = true);
      }, s = (i) => {
        t.includes(i.code) && (this.keyIsPressed = false);
      };
      this.addWindowEventHandler("keyup", s), this.addWindowEventHandler("keydown", e);
    }
    addWindowEventHandler(t, e) {
      typeof window > "u" || (window.addEventListener(t, e), this.windowEventHandlers.push({
        event: t,
        handler: e
      }));
    }
    destroy() {
      typeof window > "u" || this.windowEventHandlers.forEach(({ event: t, handler: e }) => {
        window.removeEventListener(t, e);
      });
    }
    mouseButtons(t) {
      !t || t === "all" ? this.mouse = [
        true,
        true,
        true
      ] : this.mouse = [
        t.indexOf("left") !== -1,
        t.indexOf("middle") !== -1,
        t.indexOf("right") !== -1
      ];
    }
    parseUnderflow() {
      const t = this.options.underflow.toLowerCase();
      t === "center" ? (this.underflowX = 0, this.underflowY = 0) : (t.includes("left") ? this.underflowX = -1 : t.includes("right") ? this.underflowX = 1 : this.underflowX = 0, t.includes("top") ? this.underflowY = -1 : t.includes("bottom") ? this.underflowY = 1 : this.underflowY = 0);
    }
    checkButtons(t) {
      const e = t.pointerType === "mouse", s = this.parent.input.count();
      return !!((s === 1 || s > 1 && !this.parent.plugins.get("pinch", true)) && (!e || this.mouse[t.button]));
    }
    checkKeyPress(t) {
      return !this.options.keyToPress || this.keyIsPressed || this.options.ignoreKeyToPressOnTouch && t.data.pointerType === "touch";
    }
    down(t) {
      return this.paused || !this.options.pressDrag ? false : this.checkButtons(t) && this.checkKeyPress(t) ? (this.last = {
        x: t.global.x,
        y: t.global.y
      }, (this.parent.parent || this.parent).toLocal(this.last, void 0, this.last), this.current = t.pointerId, true) : (this.last = null, false);
    }
    get active() {
      return this.moved;
    }
    move(t) {
      if (this.paused || !this.options.pressDrag) return false;
      if (this.last && this.current === t.data.pointerId) {
        const e = t.global.x, s = t.global.y, i = this.parent.input.count();
        if (i === 1 || i > 1 && !this.parent.plugins.get("pinch", true)) {
          const r = {
            x: e,
            y: s
          };
          (this.parent.parent || this.parent).toLocal(r, void 0, r);
          const o = r.x - this.last.x, a = r.y - this.last.y;
          if (this.moved || this.xDirection && this.parent.input.checkThreshold(o) || this.yDirection && this.parent.input.checkThreshold(a)) return this.xDirection && (this.parent.x += (r.x - this.last.x) * this.options.factor), this.yDirection && (this.parent.y += (r.y - this.last.y) * this.options.factor), this.last = r, this.moved || this.parent.emit("drag-start", {
            event: t,
            screen: new At(this.last.x, this.last.y),
            world: this.parent.toWorld(new At(this.last.x, this.last.y)),
            viewport: this.parent
          }), this.moved = true, this.parent.emit("moved", {
            viewport: this.parent,
            type: "drag"
          }), true;
        } else this.moved = false;
      }
      return false;
    }
    up(t) {
      if (this.paused) return false;
      const e = this.parent.input.touches;
      if (e.length === 1) {
        const s = e[0];
        return s.last && (this.last = {
          x: s.last.x,
          y: s.last.y
        }, this.current = s.id), this.moved = false, true;
      } else if (this.last && this.moved) {
        const s = new At(this.last.x, this.last.y);
        return (this.parent.parent || this.parent).toGlobal(s, s, true), this.parent.emit("drag-end", {
          event: t,
          screen: s,
          world: this.parent.toWorld(s),
          viewport: this.parent
        }), this.last = null, this.moved = false, true;
      }
      return false;
    }
    wheel(t) {
      if (this.paused) return false;
      if (this.options.wheel) {
        const e = this.parent.plugins.get("wheel", true);
        if (!e || !e.options.wheelZoom && !t.ctrlKey) {
          const s = t.deltaMode ? this.options.lineHeight : 1, i = [
            t.deltaX,
            t.deltaY
          ], [r, o] = this.options.wheelSwapAxes ? i.reverse() : i;
          return this.xDirection && (this.parent.x += r * s * this.options.wheelScroll * this.reverse), this.yDirection && (this.parent.y += o * s * this.options.wheelScroll * this.reverse), this.options.clampWheel && this.clamp(), this.parent.emit("wheel-scroll", this.parent), this.parent.emit("moved", {
            viewport: this.parent,
            type: "wheel"
          }), this.parent.options.passiveWheel || t.preventDefault(), this.parent.options.stopPropagation && t.stopPropagation(), true;
        }
      }
      return false;
    }
    resume() {
      this.last = null, this.paused = false;
    }
    clamp() {
      const t = this.parent.plugins.get("decelerate", true) || {};
      if (this.options.clampWheel !== "y") if (this.parent.screenWorldWidth < this.parent.screenWidth) switch (this.underflowX) {
        case -1:
          this.parent.x = 0;
          break;
        case 1:
          this.parent.x = this.parent.screenWidth - this.parent.screenWorldWidth;
          break;
        default:
          this.parent.x = (this.parent.screenWidth - this.parent.screenWorldWidth) / 2;
      }
      else this.parent.left < 0 ? (this.parent.x = 0, t.x = 0) : this.parent.right > this.parent.worldWidth && (this.parent.x = -this.parent.worldWidth * this.parent.scale.x + this.parent.screenWidth, t.x = 0);
      if (this.options.clampWheel !== "x") if (this.parent.screenWorldHeight < this.parent.screenHeight) switch (this.underflowY) {
        case -1:
          this.parent.y = 0;
          break;
        case 1:
          this.parent.y = this.parent.screenHeight - this.parent.screenWorldHeight;
          break;
        default:
          this.parent.y = (this.parent.screenHeight - this.parent.screenWorldHeight) / 2;
      }
      else this.parent.top < 0 && (this.parent.y = 0, t.y = 0), this.parent.bottom > this.parent.worldHeight && (this.parent.y = -this.parent.worldHeight * this.parent.scale.y + this.parent.screenHeight, t.y = 0);
    }
  }
  const Jy = {
    speed: 0,
    acceleration: null,
    radius: null
  };
  class Zy extends $e {
    constructor(t, e, s = {}) {
      super(t), this.target = e, this.options = Object.assign({}, Jy, s), this.velocity = {
        x: 0,
        y: 0
      };
    }
    update(t) {
      if (this.paused) return;
      const e = this.parent.center;
      let s = this.target.x, i = this.target.y;
      if (this.options.radius) if (Math.sqrt(Math.pow(this.target.y - e.y, 2) + Math.pow(this.target.x - e.x, 2)) > this.options.radius) {
        const a = Math.atan2(this.target.y - e.y, this.target.x - e.x);
        s = this.target.x - Math.cos(a) * this.options.radius, i = this.target.y - Math.sin(a) * this.options.radius;
      } else return;
      const r = s - e.x, o = i - e.y;
      if (r || o) if (this.options.speed) if (this.options.acceleration) {
        const a = Math.atan2(i - e.y, s - e.x), l = Math.sqrt(Math.pow(r, 2) + Math.pow(o, 2));
        if (l) {
          const c = (Math.pow(this.velocity.x, 2) + Math.pow(this.velocity.y, 2)) / (2 * this.options.acceleration);
          l > c ? this.velocity = {
            x: Math.min(this.velocity.x + (this.options.acceleration * t, this.options.speed)),
            y: Math.min(this.velocity.y + (this.options.acceleration * t, this.options.speed))
          } : this.velocity = {
            x: Math.max(this.velocity.x - this.options.acceleration * this.options.speed, 0),
            y: Math.max(this.velocity.y - this.options.acceleration * this.options.speed, 0)
          };
          const h = Math.cos(a) * this.velocity.x, d = Math.sin(a) * this.velocity.y, u = Math.abs(h) > Math.abs(r) ? s : e.x + h, p = Math.abs(d) > Math.abs(o) ? i : e.y + d;
          this.parent.moveCenter(u, p), this.parent.emit("moved", {
            viewport: this.parent,
            type: "follow"
          });
        }
      } else {
        const a = Math.atan2(i - e.y, s - e.x), l = Math.cos(a) * this.options.speed, c = Math.sin(a) * this.options.speed, h = Math.abs(l) > Math.abs(r) ? s : e.x + l, d = Math.abs(c) > Math.abs(o) ? i : e.y + c;
        this.parent.moveCenter(h, d), this.parent.emit("moved", {
          viewport: this.parent,
          type: "follow"
        });
      }
      else this.parent.moveCenter(s, i), this.parent.emit("moved", {
        viewport: this.parent,
        type: "follow"
      });
    }
  }
  const Qy = {
    radius: null,
    distance: null,
    top: null,
    bottom: null,
    left: null,
    right: null,
    speed: 8,
    reverse: false,
    noDecelerate: false,
    linear: false,
    allowButtons: false
  };
  class tx extends $e {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Qy, e), this.reverse = this.options.reverse ? 1 : -1, this.radiusSquared = typeof this.options.radius == "number" ? Math.pow(this.options.radius, 2) : null, this.resize();
    }
    resize() {
      const t = this.options.distance;
      t !== null ? (this.left = t, this.top = t, this.right = this.parent.screenWidth - t, this.bottom = this.parent.screenHeight - t) : this.options.radius || (this.left = this.options.left, this.top = this.options.top, this.right = this.options.right === null ? null : this.parent.screenWidth - this.options.right, this.bottom = this.options.bottom === null ? null : this.parent.screenHeight - this.options.bottom);
    }
    down() {
      return this.paused || this.options.allowButtons || (this.horizontal = this.vertical = null), false;
    }
    move(t) {
      if (this.paused || t.pointerType !== "mouse" && t.pointerId !== 1 || !this.options.allowButtons && t.buttons !== 0) return false;
      const e = t.global.x, s = t.global.y;
      if (this.radiusSquared) {
        const i = this.parent.toScreen(this.parent.center);
        if (Math.pow(i.x - e, 2) + Math.pow(i.y - s, 2) >= this.radiusSquared) {
          const r = Math.atan2(i.y - s, i.x - e);
          this.options.linear ? (this.horizontal = Math.round(Math.cos(r)) * this.options.speed * this.reverse * (60 / 1e3), this.vertical = Math.round(Math.sin(r)) * this.options.speed * this.reverse * (60 / 1e3)) : (this.horizontal = Math.cos(r) * this.options.speed * this.reverse * (60 / 1e3), this.vertical = Math.sin(r) * this.options.speed * this.reverse * (60 / 1e3));
        } else this.horizontal && this.decelerateHorizontal(), this.vertical && this.decelerateVertical(), this.horizontal = this.vertical = 0;
      } else this.left !== null && e < this.left ? this.horizontal = Number(this.reverse) * this.options.speed * (60 / 1e3) : this.right !== null && e > this.right ? this.horizontal = -1 * this.reverse * this.options.speed * (60 / 1e3) : (this.decelerateHorizontal(), this.horizontal = 0), this.top !== null && s < this.top ? this.vertical = Number(this.reverse) * this.options.speed * (60 / 1e3) : this.bottom !== null && s > this.bottom ? this.vertical = -1 * this.reverse * this.options.speed * (60 / 1e3) : (this.decelerateVertical(), this.vertical = 0);
      return false;
    }
    decelerateHorizontal() {
      const t = this.parent.plugins.get("decelerate", true);
      this.horizontal && t && !this.options.noDecelerate && t.activate({
        x: this.horizontal * this.options.speed * this.reverse / (1e3 / 60)
      });
    }
    decelerateVertical() {
      const t = this.parent.plugins.get("decelerate", true);
      this.vertical && t && !this.options.noDecelerate && t.activate({
        y: this.vertical * this.options.speed * this.reverse / (1e3 / 60)
      });
    }
    up() {
      return this.paused || (this.horizontal && this.decelerateHorizontal(), this.vertical && this.decelerateVertical(), this.horizontal = this.vertical = null), false;
    }
    update() {
      if (!this.paused && (this.horizontal || this.vertical)) {
        const t = this.parent.center;
        this.horizontal && (t.x += this.horizontal * this.options.speed), this.vertical && (t.y += this.vertical * this.options.speed), this.parent.moveCenter(t), this.parent.emit("moved", {
          viewport: this.parent,
          type: "mouse-edges"
        });
      }
    }
  }
  const ex = {
    noDrag: false,
    percent: 1,
    center: null,
    factor: 1,
    axis: "all"
  }, nx = new At();
  class sx extends $e {
    constructor(t, e = {}) {
      super(t), this.active = false, this.pinching = false, this.moved = false, this.options = Object.assign({}, ex, e);
    }
    down() {
      return this.parent.input.count() >= 2 ? (this.active = true, true) : false;
    }
    isAxisX() {
      return [
        "all",
        "x"
      ].includes(this.options.axis);
    }
    isAxisY() {
      return [
        "all",
        "y"
      ].includes(this.options.axis);
    }
    move(t) {
      if (this.paused || !this.active) return false;
      const { x: e, y: s } = (this.parent.parent || this.parent).toLocal(t.global, void 0, nx), i = this.parent.input.touches;
      if (i.length >= 2) {
        const r = i[0], o = i[1], a = r.last && o.last ? Math.sqrt(Math.pow(o.last.x - r.last.x, 2) + Math.pow(o.last.y - r.last.y, 2)) : null;
        if (r.id === t.pointerId ? r.last = {
          x: e,
          y: s,
          data: t
        } : o.id === t.pointerId && (o.last = {
          x: e,
          y: s,
          data: t
        }), a) {
          let l;
          const c = new At(r.last.x + (o.last.x - r.last.x) / 2, r.last.y + (o.last.y - r.last.y) / 2);
          this.options.center || (l = this.parent.toLocal(c, this.parent.parent || this.parent));
          let h = Math.sqrt(Math.pow(o.last.x - r.last.x, 2) + Math.pow(o.last.y - r.last.y, 2));
          h = h === 0 ? h = 1e-10 : h;
          const d = (1 - a / h) * this.options.percent * (this.isAxisX() ? this.parent.scale.x : this.parent.scale.y);
          this.isAxisX() && (this.parent.scale.x += d), this.isAxisY() && (this.parent.scale.y += d), this.parent.emit("zoomed", {
            viewport: this.parent,
            type: "pinch",
            center: c
          });
          const u = this.parent.plugins.get("clamp-zoom", true);
          if (u && u.clamp(), this.options.center) this.parent.moveCenter(this.options.center);
          else {
            const p = (this.parent.parent || this.parent).toLocal(l, this.parent);
            this.parent.x += (c.x - p.x) * this.options.factor, this.parent.y += (c.y - p.y) * this.options.factor, this.parent.emit("moved", {
              viewport: this.parent,
              type: "pinch"
            });
          }
          !this.options.noDrag && this.lastCenter && (this.parent.x += (c.x - this.lastCenter.x) * this.options.factor, this.parent.y += (c.y - this.lastCenter.y) * this.options.factor, this.parent.emit("moved", {
            viewport: this.parent,
            type: "pinch"
          })), this.lastCenter = c, this.moved = true;
        } else this.pinching || (this.parent.emit("pinch-start", this.parent), this.pinching = true);
        return true;
      }
      return false;
    }
    up() {
      return this.pinching && this.parent.input.touches.length <= 1 ? (this.active = false, this.lastCenter = null, this.pinching = false, this.moved = false, this.parent.emit("pinch-end", this.parent), true) : false;
    }
  }
  const ix = {
    topLeft: false,
    friction: 0.8,
    time: 1e3,
    ease: "easeInOutSine",
    interrupt: true,
    removeOnComplete: false,
    removeOnInterrupt: false,
    forceStart: false
  };
  class rx extends $e {
    constructor(t, e, s, i = {}) {
      super(t), this.options = Object.assign({}, ix, i), this.ease = tr(i.ease, "easeInOutSine"), this.x = e, this.y = s, this.options.forceStart && this.snapStart();
    }
    snapStart() {
      this.percent = 0, this.snapping = {
        time: 0
      };
      const t = this.options.topLeft ? this.parent.corner : this.parent.center;
      this.deltaX = this.x - t.x, this.deltaY = this.y - t.y, this.startX = t.x, this.startY = t.y, this.parent.emit("snap-start", this.parent);
    }
    wheel() {
      return this.options.removeOnInterrupt && this.parent.plugins.remove("snap"), false;
    }
    down() {
      return this.options.removeOnInterrupt ? this.parent.plugins.remove("snap") : this.options.interrupt && (this.snapping = null), false;
    }
    up() {
      if (this.parent.input.count() === 0) {
        const t = this.parent.plugins.get("decelerate", true);
        t && (t.x || t.y) && (t.percentChangeX = t.percentChangeY = this.options.friction);
      }
      return false;
    }
    update(t) {
      if (!this.paused && !(this.options.interrupt && this.parent.input.count() !== 0)) if (this.snapping) {
        const e = this.snapping;
        e.time += t;
        let s, i, r;
        const o = this.startX, a = this.startY, l = this.deltaX, c = this.deltaY;
        if (e.time > this.options.time) s = true, i = o + l, r = a + c;
        else {
          const h = this.ease(e.time, 0, 1, this.options.time);
          i = o + l * h, r = a + c * h;
        }
        this.options.topLeft ? this.parent.moveCorner(i, r) : this.parent.moveCenter(i, r), this.parent.emit("moved", {
          viewport: this.parent,
          type: "snap"
        }), s && (this.options.removeOnComplete && this.parent.plugins.remove("snap"), this.parent.emit("snap-end", this.parent), this.snapping = null);
      } else {
        const e = this.options.topLeft ? this.parent.corner : this.parent.center;
        (e.x !== this.x || e.y !== this.y) && this.snapStart();
      }
    }
  }
  const ox = {
    width: 0,
    height: 0,
    time: 1e3,
    ease: "easeInOutSine",
    center: null,
    interrupt: true,
    removeOnComplete: false,
    removeOnInterrupt: false,
    forceStart: false,
    noMove: false
  };
  class ax extends $e {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, ox, e), this.ease = tr(this.options.ease), this.xIndependent = false, this.yIndependent = false, this.xScale = 0, this.yScale = 0, this.options.width > 0 && (this.xScale = t.screenWidth / this.options.width, this.xIndependent = true), this.options.height > 0 && (this.yScale = t.screenHeight / this.options.height, this.yIndependent = true), this.xScale = this.xIndependent ? this.xScale : this.yScale, this.yScale = this.yIndependent ? this.yScale : this.xScale, this.options.time === 0 ? (t.container.scale.x = this.xScale, t.container.scale.y = this.yScale, this.options.removeOnComplete && this.parent.plugins.remove("snap-zoom")) : e.forceStart && this.createSnapping();
    }
    createSnapping() {
      const t = this.parent.worldScreenWidth, e = this.parent.worldScreenHeight, s = this.parent.screenWidth / this.xScale, i = this.parent.screenHeight / this.yScale;
      this.snapping = {
        time: 0,
        startX: t,
        startY: e,
        deltaX: s - t,
        deltaY: i - e
      }, this.parent.emit("snap-zoom-start", this.parent);
    }
    resize() {
      this.snapping = null, this.options.width > 0 && (this.xScale = this.parent.screenWidth / this.options.width), this.options.height > 0 && (this.yScale = this.parent.screenHeight / this.options.height), this.xScale = this.xIndependent ? this.xScale : this.yScale, this.yScale = this.yIndependent ? this.yScale : this.xScale;
    }
    wheel() {
      return this.options.removeOnInterrupt && this.parent.plugins.remove("snap-zoom"), false;
    }
    down() {
      return this.options.removeOnInterrupt ? this.parent.plugins.remove("snap-zoom") : this.options.interrupt && (this.snapping = null), false;
    }
    update(t) {
      if (this.paused || this.options.interrupt && this.parent.input.count() !== 0) return;
      let e;
      if (!this.options.center && !this.options.noMove && (e = this.parent.center), !this.snapping) (this.parent.scale.x !== this.xScale || this.parent.scale.y !== this.yScale) && this.createSnapping();
      else if (this.snapping) {
        const s = this.snapping;
        if (s.time += t, s.time >= this.options.time) this.parent.scale.set(this.xScale, this.yScale), this.options.removeOnComplete && this.parent.plugins.remove("snap-zoom"), this.parent.emit("snap-zoom-end", this.parent), this.snapping = null;
        else {
          const r = this.snapping, o = this.ease(r.time, r.startX, r.deltaX, this.options.time), a = this.ease(r.time, r.startY, r.deltaY, this.options.time);
          this.parent.scale.x = this.parent.screenWidth / o, this.parent.scale.y = this.parent.screenHeight / a;
        }
        const i = this.parent.plugins.get("clamp-zoom", true);
        i && i.clamp(), this.options.noMove || (this.options.center ? this.parent.moveCenter(this.options.center) : this.parent.moveCenter(e));
      }
    }
    resume() {
      this.snapping = null, super.resume();
    }
  }
  const lx = {
    percent: 0.1,
    smooth: false,
    interrupt: true,
    reverse: false,
    center: null,
    lineHeight: 20,
    axis: "all",
    keyToPress: null,
    trackpadPinch: false,
    wheelZoom: true
  };
  class cx extends $e {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, lx, e), this.keyIsPressed = false, this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
    }
    handleKeyPresses(t) {
      typeof window > "u" || (window.addEventListener("keydown", (e) => {
        t.includes(e.code) && (this.keyIsPressed = true);
      }), window.addEventListener("keyup", (e) => {
        t.includes(e.code) && (this.keyIsPressed = false);
      }));
    }
    checkKeyPress() {
      return !this.options.keyToPress || this.keyIsPressed;
    }
    down() {
      return this.options.interrupt && (this.smoothing = null), false;
    }
    isAxisX() {
      return [
        "all",
        "x"
      ].includes(this.options.axis);
    }
    isAxisY() {
      return [
        "all",
        "y"
      ].includes(this.options.axis);
    }
    update() {
      if (this.smoothing) {
        const t = this.smoothingCenter, e = this.smoothing;
        let s;
        this.options.center || (s = this.parent.toLocal(t)), this.isAxisX() && (this.parent.scale.x += e.x), this.isAxisY() && (this.parent.scale.y += e.y), this.parent.emit("zoomed", {
          viewport: this.parent,
          type: "wheel"
        });
        const i = this.parent.plugins.get("clamp-zoom", true);
        if (i && i.clamp(), this.options.center) this.parent.moveCenter(this.options.center);
        else {
          const r = this.parent.parent || this.parent;
          r.toLocal(s, this.parent, s);
          const o = r.toLocal(t);
          this.parent.x += o.x - s.x, this.parent.y += o.y - s.y;
        }
        this.parent.emit("moved", {
          viewport: this.parent,
          type: "wheel"
        }), this.smoothingCount++, typeof this.options.smooth == "number" && this.smoothingCount >= this.options.smooth && (this.smoothing = null);
      }
    }
    pinch(t) {
      if (this.paused) return;
      const e = this.parent.input.getPointerPosition(t), s = -t.deltaY * (t.deltaMode ? this.options.lineHeight : 1) / 200, i = Math.pow(2, (1 + this.options.percent) * s);
      let r;
      this.options.center || (r = this.parent.toLocal(e)), this.isAxisX() && (this.parent.scale.x *= i), this.isAxisY() && (this.parent.scale.y *= i), this.parent.emit("zoomed", {
        viewport: this.parent,
        type: "wheel"
      });
      const o = this.parent.plugins.get("clamp-zoom", true);
      if (o && o.clamp(), this.options.center) this.parent.moveCenter(this.options.center);
      else {
        const a = this.parent.parent || this.parent;
        a.toLocal(r, this.parent, r);
        const l = a.toLocal(e);
        this.parent.x += l.x - r.x, this.parent.y += l.y - r.y;
      }
      this.parent.emit("moved", {
        viewport: this.parent,
        type: "wheel"
      }), this.parent.emit("wheel-start", {
        event: t,
        viewport: this.parent
      });
    }
    wheel(t) {
      if (this.paused || !this.checkKeyPress()) return false;
      if (t.ctrlKey && this.options.trackpadPinch) this.pinch(t);
      else if (this.options.wheelZoom) {
        const e = this.parent.input.getPointerPosition(t), s = (this.options.reverse ? -1 : 1) * -t.deltaY * (t.deltaMode ? this.options.lineHeight : 1) / 500, i = Math.pow(2, (1 + this.options.percent) * s);
        if (this.options.smooth) {
          const r = {
            x: this.smoothing ? this.smoothing.x * (this.options.smooth - this.smoothingCount) : 0,
            y: this.smoothing ? this.smoothing.y * (this.options.smooth - this.smoothingCount) : 0
          };
          this.smoothing = {
            x: ((this.parent.scale.x + r.x) * i - this.parent.scale.x) / this.options.smooth,
            y: ((this.parent.scale.y + r.y) * i - this.parent.scale.y) / this.options.smooth
          }, this.smoothingCount = 0, this.smoothingCenter = e;
        } else {
          let r;
          this.options.center || (r = this.parent.toLocal(e)), this.isAxisX() && (this.parent.scale.x *= i), this.isAxisY() && (this.parent.scale.y *= i), this.parent.emit("zoomed", {
            viewport: this.parent,
            type: "wheel"
          });
          const o = this.parent.plugins.get("clamp-zoom", true);
          if (o && o.clamp(), this.options.center) this.parent.moveCenter(this.options.center);
          else {
            const a = this.parent.parent || this.parent;
            a.toLocal(r, this.parent, r);
            const l = a.toLocal(e);
            this.parent.x += l.x - r.x, this.parent.y += l.y - r.y;
          }
        }
        this.parent.emit("moved", {
          viewport: this.parent,
          type: "wheel"
        }), this.parent.emit("wheel-start", {
          event: t,
          viewport: this.parent
        });
      }
      return !this.parent.options.passiveWheel;
    }
  }
  const hx = {
    screenWidth: typeof window > "u" ? 0 : window.innerWidth,
    screenHeight: typeof window > "u" ? 0 : window.innerHeight,
    worldWidth: null,
    worldHeight: null,
    threshold: 5,
    passiveWheel: true,
    stopPropagation: false,
    forceHitArea: null,
    noTicker: false,
    disableOnContextMenu: false,
    ticker: Nn.shared,
    allowPreserveDragOutside: false
  };
  class id extends zt {
    constructor(t) {
      super(), this._disableOnContextMenu = (e) => e.preventDefault(), this.options = {
        ...hx,
        ...t
      }, this.screenWidth = this.options.screenWidth, this.screenHeight = this.options.screenHeight, this._worldWidth = this.options.worldWidth, this._worldHeight = this.options.worldHeight, this.forceHitArea = this.options.forceHitArea, this.threshold = this.options.threshold, this.options.disableOnContextMenu && this.options.events.domElement.addEventListener("contextmenu", this._disableOnContextMenu), this.options.noTicker || (this.tickerFunction = () => this.update(this.options.ticker.elapsedMS), this.options.ticker.add(this.tickerFunction)), this.input = new Fy(this), this.plugins = new Ny(this);
    }
    destroy(t) {
      var e;
      !this.options.noTicker && this.tickerFunction && this.options.ticker.remove(this.tickerFunction), this.options.disableOnContextMenu && ((e = this.options.events.domElement) == null || e.removeEventListener("contextmenu", this._disableOnContextMenu)), this.input.destroy(), super.destroy(t);
    }
    update(t) {
      this.pause || (this.plugins.update(t), this.lastViewport && (this.lastViewport.x !== this.x || this.lastViewport.y !== this.y ? this.moving = true : this.moving && (this.emit("moved-end", this), this.moving = false), this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y ? this.zooming = true : this.zooming && (this.emit("zoomed-end", this), this.zooming = false)), this.forceHitArea || (this._hitAreaDefault = new Nt(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight), this.hitArea = this._hitAreaDefault), this._dirty = this._dirty || !this.lastViewport || this.lastViewport.x !== this.x || this.lastViewport.y !== this.y || this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y, this.lastViewport = {
        x: this.x,
        y: this.y,
        scaleX: this.scale.x,
        scaleY: this.scale.y
      }, this.emit("frame-end", this));
    }
    resize(t = typeof window > "u" ? 0 : window.innerWidth, e = typeof window > "u" ? 0 : window.innerHeight, s, i) {
      this.screenWidth = t, this.screenHeight = e, typeof s < "u" && (this._worldWidth = s), typeof i < "u" && (this._worldHeight = i), this.plugins.resize(), this.dirty = true;
    }
    get worldWidth() {
      return this._worldWidth ? this._worldWidth : this.width / this.scale.x;
    }
    set worldWidth(t) {
      this._worldWidth = t, this.plugins.resize();
    }
    get worldHeight() {
      return this._worldHeight ? this._worldHeight : this.height / this.scale.y;
    }
    set worldHeight(t) {
      this._worldHeight = t, this.plugins.resize();
    }
    getVisibleBounds() {
      return new Nt(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight);
    }
    toWorld(t, e) {
      return arguments.length === 2 ? this.toLocal(new At(t, e)) : this.toLocal(t);
    }
    toScreen(t, e) {
      return arguments.length === 2 ? this.toGlobal(new At(t, e)) : this.toGlobal(t);
    }
    get worldScreenWidth() {
      return this.screenWidth / this.scale.x;
    }
    get worldScreenHeight() {
      return this.screenHeight / this.scale.y;
    }
    get screenWorldWidth() {
      return this.worldWidth * this.scale.x;
    }
    get screenWorldHeight() {
      return this.worldHeight * this.scale.y;
    }
    get center() {
      return new At(this.worldScreenWidth / 2 - this.x / this.scale.x, this.worldScreenHeight / 2 - this.y / this.scale.y);
    }
    set center(t) {
      this.moveCenter(t);
    }
    moveCenter(...t) {
      let e, s;
      typeof t[0] == "number" ? (e = t[0], s = t[1]) : (e = t[0].x, s = t[0].y);
      const i = (this.worldScreenWidth / 2 - e) * this.scale.x, r = (this.worldScreenHeight / 2 - s) * this.scale.y;
      return (this.x !== i || this.y !== r) && (this.position.set(i, r), this.plugins.reset(), this.dirty = true), this;
    }
    get corner() {
      return new At(-this.x / this.scale.x, -this.y / this.scale.y);
    }
    set corner(t) {
      this.moveCorner(t);
    }
    moveCorner(...t) {
      let e, s;
      return t.length === 1 ? (e = -t[0].x * this.scale.x, s = -t[0].y * this.scale.y) : (e = -t[0] * this.scale.x, s = -t[1] * this.scale.y), (e !== this.x || s !== this.y) && (this.position.set(e, s), this.plugins.reset(), this.dirty = true), this;
    }
    get screenWidthInWorldPixels() {
      return this.screenWidth / this.scale.x;
    }
    get screenHeightInWorldPixels() {
      return this.screenHeight / this.scale.y;
    }
    findFitWidth(t) {
      return this.screenWidth / t;
    }
    findFitHeight(t) {
      return this.screenHeight / t;
    }
    findFit(t, e) {
      const s = this.screenWidth / t, i = this.screenHeight / e;
      return Math.min(s, i);
    }
    findCover(t, e) {
      const s = this.screenWidth / t, i = this.screenHeight / e;
      return Math.max(s, i);
    }
    fitWidth(t = this.worldWidth, e, s = true, i) {
      let r;
      e && (r = this.center), this.scale.x = this.screenWidth / t, s && (this.scale.y = this.scale.x);
      const o = this.plugins.get("clamp-zoom", true);
      return !i && o && o.clamp(), e && r && this.moveCenter(r), this;
    }
    fitHeight(t = this.worldHeight, e, s = true, i) {
      let r;
      e && (r = this.center), this.scale.y = this.screenHeight / t, s && (this.scale.x = this.scale.y);
      const o = this.plugins.get("clamp-zoom", true);
      return !i && o && o.clamp(), e && r && this.moveCenter(r), this;
    }
    fitWorld(t) {
      let e;
      t && (e = this.center), this.scale.x = this.screenWidth / this.worldWidth, this.scale.y = this.screenHeight / this.worldHeight, this.scale.x < this.scale.y ? this.scale.y = this.scale.x : this.scale.x = this.scale.y;
      const s = this.plugins.get("clamp-zoom", true);
      return s && s.clamp(), t && e && this.moveCenter(e), this;
    }
    fit(t, e = this.worldWidth, s = this.worldHeight) {
      let i;
      t && (i = this.center), this.scale.x = this.screenWidth / e, this.scale.y = this.screenHeight / s, this.scale.x < this.scale.y ? this.scale.y = this.scale.x : this.scale.x = this.scale.y;
      const r = this.plugins.get("clamp-zoom", true);
      return r && r.clamp(), t && i && this.moveCenter(i), this;
    }
    setZoom(t, e) {
      let s;
      e && (s = this.center), this.scale.set(t);
      const i = this.plugins.get("clamp-zoom", true);
      return i && i.clamp(), e && s && this.moveCenter(s), this;
    }
    zoomPercent(t, e) {
      return this.setZoom(this.scale.x + this.scale.x * t, e);
    }
    zoom(t, e) {
      return this.fitWidth(t + this.worldScreenWidth, e), this;
    }
    get scaled() {
      return this.scale.x;
    }
    set scaled(t) {
      this.setZoom(t, true);
    }
    snapZoom(t) {
      return this.plugins.add("snap-zoom", new ax(this, t)), this;
    }
    OOB() {
      return {
        left: this.left < 0,
        right: this.right > this.worldWidth,
        top: this.top < 0,
        bottom: this.bottom > this.worldHeight,
        cornerPoint: new At(this.worldWidth * this.scale.x - this.screenWidth, this.worldHeight * this.scale.y - this.screenHeight)
      };
    }
    get right() {
      return -this.x / this.scale.x + this.worldScreenWidth;
    }
    set right(t) {
      this.x = -t * this.scale.x + this.screenWidth, this.plugins.reset();
    }
    get left() {
      return -this.x / this.scale.x;
    }
    set left(t) {
      this.x = -t * this.scale.x, this.plugins.reset();
    }
    get top() {
      return -this.y / this.scale.y;
    }
    set top(t) {
      this.y = -t * this.scale.y, this.plugins.reset();
    }
    get bottom() {
      return -this.y / this.scale.y + this.worldScreenHeight;
    }
    set bottom(t) {
      this.y = -t * this.scale.y + this.screenHeight, this.plugins.reset();
    }
    get dirty() {
      return !!this._dirty;
    }
    set dirty(t) {
      this._dirty = t;
    }
    get forceHitArea() {
      return this._forceHitArea;
    }
    set forceHitArea(t) {
      t ? (this._forceHitArea = t, this.hitArea = t) : (this._forceHitArea = null, this.hitArea = new Nt(0, 0, this.worldWidth, this.worldHeight));
    }
    drag(t) {
      return this.plugins.add("drag", new Ky(this, t)), this;
    }
    clamp(t) {
      return this.plugins.add("clamp", new Uy(this, t)), this;
    }
    decelerate(t) {
      return this.plugins.add("decelerate", new Xy(this, t)), this;
    }
    bounce(t) {
      return this.plugins.add("bounce", new Dy(this, t)), this;
    }
    pinch(t) {
      return this.plugins.add("pinch", new sx(this, t)), this;
    }
    snap(t, e, s) {
      return this.plugins.add("snap", new rx(this, t, e, s)), this;
    }
    follow(t, e) {
      return this.plugins.add("follow", new Zy(this, t, e)), this;
    }
    wheel(t) {
      return this.plugins.add("wheel", new cx(this, t)), this;
    }
    animate(t) {
      return this.plugins.add("animate", new Gy(this, t)), this;
    }
    clampZoom(t) {
      return this.plugins.add("clamp-zoom", new Vy(this, t)), this;
    }
    mouseEdges(t) {
      return this.plugins.add("mouse-edges", new tx(this, t)), this;
    }
    get pause() {
      return !!this._pause;
    }
    set pause(t) {
      this._pause = t, this.lastViewport = null, this.moving = false, this.zooming = false, t && this.input.pause();
    }
    ensureVisible(t, e, s, i, r) {
      r && (s > this.worldScreenWidth || i > this.worldScreenHeight) && (this.fit(true, s, i), this.emit("zoomed", {
        viewport: this,
        type: "ensureVisible"
      }));
      let o = false;
      t < this.left ? (this.left = t, o = true) : t + s > this.right && (this.right = t + s, o = true), e < this.top ? (this.top = e, o = true) : e + i > this.bottom && (this.bottom = e + i, o = true), o && this.emit("moved", {
        viewport: this,
        type: "ensureVisible"
      });
    }
  }
  const dx = 32, wo = /* @__PURE__ */ new Set([
    "transport-belt",
    "fast-transport-belt",
    "express-transport-belt"
  ]), xi = /* @__PURE__ */ new Set([
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt"
  ]), Or = /* @__PURE__ */ new Set([
    "splitter",
    "fast-splitter",
    "express-splitter"
  ]), ux = /* @__PURE__ */ new Set([
    "inserter",
    "fast-inserter",
    "long-handed-inserter"
  ]), px = /* @__PURE__ */ new Set([
    "assembling-machine-1",
    "assembling-machine-2",
    "assembling-machine-3",
    "chemical-plant",
    "oil-refinery",
    "electric-furnace",
    "steel-furnace",
    "stone-furnace",
    "centrifuge",
    "lab",
    "rocket-silo",
    "foundry",
    "biochamber",
    "biolab",
    "electromagnetic-plant",
    "cryogenic-plant",
    "recycler",
    "crusher",
    "beacon",
    "storage-tank",
    "big-electric-pole",
    "substation",
    "electric-mining-drill"
  ]), $l = {
    "transport-belt": 14733424,
    "fast-transport-belt": 16736352,
    "express-transport-belt": 7385328,
    "underground-belt": 14733424,
    "fast-underground-belt": 16736352,
    "express-underground-belt": 7385328,
    splitter: 14733424,
    "fast-splitter": 16736352,
    "express-splitter": 7385328
  };
  function Xe(n, t) {
    return `${n},${t}`;
  }
  function zi(n) {
    switch (n) {
      case "East":
        return [
          1,
          0
        ];
      case "South":
        return [
          0,
          1
        ];
      case "West":
        return [
          -1,
          0
        ];
      default:
        return [
          0,
          -1
        ];
    }
  }
  function bi(n, t, e, s, i, r) {
    const o = {
      from: t,
      to: e,
      toLane: s,
      laneCross: i,
      isSplitterOut: r
    };
    let a = n.outEdges.get(t);
    a || (a = [], n.outEdges.set(t, a)), a.some((c) => c.to === e) || a.push(o);
    let l = n.inEdges.get(e);
    l || (l = [], n.inEdges.set(e, l)), l.some((c) => c.from === t) || l.push(o);
  }
  const fx = 9;
  function mx(n) {
    const t = {
      nodes: /* @__PURE__ */ new Map(),
      outEdges: /* @__PURE__ */ new Map(),
      inEdges: /* @__PURE__ */ new Map(),
      tileToAnchor: /* @__PURE__ */ new Map(),
      entityMap: /* @__PURE__ */ new Map()
    };
    for (const e of n.entities) t.entityMap.set(Xe(e.x ?? 0, e.y ?? 0), e);
    for (const e of n.entities) {
      if (!wo.has(e.name) && !xi.has(e.name) && !Or.has(e.name)) continue;
      const s = e.x ?? 0, i = e.y ?? 0, r = Xe(s, i);
      if (t.nodes.set(r, e), t.tileToAnchor.set(r, r), Or.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South", a = s + (o ? 1 : 0), l = i + (o ? 0 : 1);
        t.tileToAnchor.set(Xe(a, l), r);
      }
    }
    for (const [e, s] of t.nodes) {
      const i = s.x ?? 0, r = s.y ?? 0, o = s.direction ?? "North", [a, l] = zi(o);
      if (wo.has(s.name)) {
        const c = t.tileToAnchor.get(Xe(i + a, r + l));
        if (c !== void 0 && c !== e) {
          const h = t.nodes.get(c), [d, u] = zi(h.direction), p = a * u - l * d;
          bi(t, e, c, "both", p > 0, false);
        }
      } else if (xi.has(s.name)) if (s.io_type === "input") for (let c = 1; c <= fx; c++) {
        const h = t.entityMap.get(Xe(i + a * c, r + l * c));
        if (h) {
          if (xi.has(h.name) && h.name === s.name && h.io_type === "input" && h.direction === o) break;
          if (xi.has(h.name) && h.name === s.name && h.io_type === "output" && h.direction === o) {
            const d = t.tileToAnchor.get(Xe(h.x ?? 0, h.y ?? 0));
            d !== void 0 && bi(t, e, d, "both", false, false);
            break;
          }
        }
      }
      else {
        const c = t.tileToAnchor.get(Xe(i + a, r + l));
        c !== void 0 && c !== e && bi(t, e, c, "both", false, false);
      }
      else if (Or.has(s.name)) {
        const c = o === "North" || o === "South", [h, d] = c ? [
          1,
          0
        ] : [
          0,
          1
        ];
        for (const [u, p] of [
          [
            i + a,
            r + l
          ],
          [
            i + h + a,
            r + d + l
          ]
        ]) {
          const f = t.tileToAnchor.get(Xe(u, p));
          f !== void 0 && f !== e && bi(t, e, f, "both", false, true);
        }
      }
    }
    return t;
  }
  function gx(n, t) {
    const e = /* @__PURE__ */ new Set(), s = /* @__PURE__ */ new Set(), i = [
      n
    ];
    for (e.add(n); i.length > 0; ) {
      const o = i.shift();
      for (const a of t.outEdges.get(o) ?? []) e.has(a.to) || (e.add(a.to), i.push(a.to));
    }
    const r = [
      n
    ];
    for (s.add(n); r.length > 0; ) {
      const o = r.shift();
      for (const a of t.inEdges.get(o) ?? []) s.has(a.from) || (s.add(a.from), r.push(a.from));
    }
    return {
      downstream: e,
      upstream: s
    };
  }
  function yx(n, t) {
    const e = /* @__PURE__ */ new Set(), s = [
      [
        0,
        -1
      ],
      [
        1,
        0
      ],
      [
        0,
        1
      ],
      [
        -1,
        0
      ]
    ];
    for (const i of n) {
      const [r, o] = i.split(",").map(Number);
      for (const [a, l] of s) {
        const c = Xe(r + a, o + l), h = t.get(c);
        h && ux.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  function xx(n, t) {
    const e = /* @__PURE__ */ new Set(), s = [
      [
        0,
        -1
      ],
      [
        1,
        0
      ],
      [
        0,
        1
      ],
      [
        -1,
        0
      ]
    ];
    for (const i of n) {
      const [r, o] = i.split(",").map(Number);
      for (const [a, l] of s) {
        const c = Xe(r + a, o + l), h = t.get(c);
        h && px.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  const vo = {
    East_South: {
      cx: (n) => n,
      cy: (n, t, e) => t + e,
      startAngle: -Math.PI / 2,
      endAngle: 0,
      anticlockwise: false
    },
    South_West: {
      cx: (n) => n,
      cy: (n, t) => t,
      startAngle: 0,
      endAngle: Math.PI / 2,
      anticlockwise: false
    },
    West_North: {
      cx: (n, t, e) => n + e,
      cy: (n, t) => t,
      startAngle: Math.PI / 2,
      endAngle: Math.PI,
      anticlockwise: false
    },
    North_East: {
      cx: (n, t, e) => n + e,
      cy: (n, t, e) => t + e,
      startAngle: Math.PI,
      endAngle: 3 * Math.PI / 2,
      anticlockwise: false
    },
    East_North: {
      cx: (n) => n,
      cy: (n, t) => t,
      startAngle: Math.PI / 2,
      endAngle: 0,
      anticlockwise: true
    },
    North_West: {
      cx: (n) => n,
      cy: (n, t, e) => t + e,
      startAngle: 0,
      endAngle: -Math.PI / 2,
      anticlockwise: true
    },
    West_South: {
      cx: (n, t, e) => n + e,
      cy: (n, t, e) => t + e,
      startAngle: -Math.PI / 2,
      endAngle: -Math.PI,
      anticlockwise: true
    },
    South_East: {
      cx: (n, t, e) => n + e,
      cy: (n, t) => t,
      startAngle: Math.PI,
      endAngle: Math.PI / 2,
      anticlockwise: true
    }
  };
  function Bl(n, t) {
    const e = t.nodes.get(n);
    if (!e || !wo.has(e.name)) return null;
    const s = e.direction ?? "North";
    for (const i of t.inEdges.get(n) ?? []) {
      const r = t.nodes.get(i.from);
      if (!r) continue;
      const o = r.direction ?? "North";
      if (`${o}_${s}` in vo) return {
        inDir: o,
        outDir: s
      };
    }
    return null;
  }
  function bx(n, t, e, s, i, r, o) {
    const a = s - t, l = i - e, c = Math.sqrt(a * a + l * l);
    if (c === 0) return;
    const h = a / c, d = l / c;
    let u = 0, p = true;
    for (; u < c; ) {
      const f = Math.min(p ? r : o, c - u);
      p && n.moveTo(t + h * u, e + d * u).lineTo(t + h * (u + f), e + d * (u + f)).stroke(), u += f, p = !p;
    }
  }
  function _x(n, t, e, s, i, r, o, a, l) {
    let c = o ? i - r : r - i;
    c < 0 && (c += 2 * Math.PI);
    let h = 0, d = true;
    for (; h < c; ) {
      const u = Math.min(d ? a : l, c - h);
      if (d) {
        const p = o ? i - h : i + h, f = o ? p - u : p + u, g = t + s * Math.cos(p), m = e + s * Math.sin(p);
        n.moveTo(g, m).arc(t, e, s, p, f, o).stroke();
      }
      h += u, d = !d;
    }
  }
  function wx(n, t, e, s, i) {
    const r = dx, o = r / 2;
    for (const l of e) {
      if (t.has(l)) continue;
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = $l[c.name] ?? 14733424, [p, f] = zi(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.05
      }), n.setStrokeStyle({
        width: 1.5,
        color: u,
        alpha: 0.28,
        cap: "round"
      });
      const y = Bl(l, i);
      if (y) {
        const b = vo[`${y.inDir}_${y.outDir}`], x = b.cx(h, d, r), _ = b.cy(h, d, r);
        _x(n, x, _, o, b.startAngle, b.endAngle, b.anticlockwise, 5 / o, 3 / o);
      } else bx(n, g - p * r * 0.45, m - f * r * 0.45, g + p * r * 0.45, m + f * r * 0.45, 5, 3);
    }
    for (const l of t) {
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = $l[c.name] ?? 14733424, [p, f] = zi(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.2
      }), n.setStrokeStyle({
        width: 2,
        color: u,
        alpha: 0.85,
        cap: "round"
      });
      const y = Bl(l, i);
      if (y) {
        const b = vo[`${y.inDir}_${y.outDir}`], x = b.cx(h, d, r), _ = b.cy(h, d, r), v = x + o * Math.cos(b.startAngle), w = _ + o * Math.sin(b.startAngle);
        n.moveTo(v, w).arc(x, _, o, b.startAngle, b.endAngle, b.anticlockwise).stroke();
      } else n.moveTo(g - p * r * 0.45, m - f * r * 0.45).lineTo(g + p * r * 0.45, m + f * r * 0.45).stroke();
    }
    const a = i.nodes.get(s);
    if (a) {
      const l = (a.x ?? 0) * r, c = (a.y ?? 0) * r;
      n.setStrokeStyle({
        width: 2,
        color: 16777215,
        alpha: 0.8
      }), n.rect(l + 1, c + 1, r - 2, r - 2).stroke();
    }
  }
  const k = 32, vx = {
    "assembling-machine-1": 5926530,
    "assembling-machine-2": 4874872,
    "assembling-machine-3": 3822186,
    "stone-furnace": 9068608,
    "steel-furnace": 8015920,
    "electric-furnace": 6969984,
    "chemical-plant": 3832400,
    "oil-refinery": 5913226,
    centrifuge: 3832448,
    lab: 4876880,
    "rocket-silo": 4868714,
    foundry: 9071152,
    "electromagnetic-plant": 2775706,
    "cryogenic-plant": 4881034,
    biochamber: 4880954,
    biolab: 3828314,
    recycler: 6969930,
    crusher: 5917242,
    beacon: 4874368,
    "storage-tank": 4876890,
    "big-electric-pole": 9136404,
    substation: 6974091,
    "electric-mining-drill": 8022576
  }, Cx = 4872810, Yn = {
    "transport-belt": [
      11046960,
      13879429
    ],
    "fast-transport-belt": [
      11546672,
      14577776
    ],
    "express-transport-belt": [
      3174576,
      8630492
    ],
    "underground-belt": [
      11046960,
      13879429
    ],
    "fast-underground-belt": [
      11546672,
      14577776
    ],
    "express-underground-belt": [
      3174576,
      8630492
    ],
    splitter: [
      11046960,
      13879429
    ],
    "fast-splitter": [
      11546672,
      14577776
    ],
    "express-splitter": [
      3174576,
      8630492
    ]
  }, rd = {
    inserter: 6983230,
    "fast-inserter": 4886736,
    "long-handed-inserter": 13647936
  }, od = 9079434, Sx = 6974058, Ol = 2039583, Fl = 12623920, kx = 2762e3, Tx = 0.35, Ax = {
    "iron-plate": 10132122,
    "copper-plate": 13662272,
    "iron-gear-wheel": 7368816,
    "copper-cable": 14704672,
    "electronic-circuit": 5292112,
    "advanced-circuit": 12603472,
    "processing-unit": 5263552,
    "plastic-bar": 8421536,
    "steel-plate": 7370888,
    "iron-ore": 10514544,
    "copper-ore": 13664352,
    coal: 4210752,
    stone: 10522736,
    sulfur: 12632128,
    "crude-oil": 3158096,
    water: 4219056,
    "petroleum-gas": 10526816,
    "light-oil": 10526896,
    "heavy-oil": 7360576,
    "sulfuric-acid": 11579440,
    lubricant: 6336608
  };
  function Ex(n, t) {
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = 0.21 * e + 0.72 * s + 0.07 * i, o = Math.round(r + (e - r) * t), a = Math.round(r + (s - r) * t), l = Math.round(r + (i - r) * t);
    return o << 16 | a << 8 | l;
  }
  const Nl = Object.fromEntries(Object.entries(Ax).map(([n, t]) => [
    n,
    Ex(t, Tx)
  ]));
  function Mx(n, t, e) {
    const s = t * Math.min(e, 1 - e), i = (r) => {
      const o = (r + n * 12) % 12;
      return Math.round((e - s * Math.max(-1, Math.min(o - 3, 9 - o, 1))) * 255);
    };
    return i(0) << 16 | i(8) << 8 | i(4);
  }
  let ad = true;
  function _i(n) {
    ad = n;
  }
  let Co = /* @__PURE__ */ new Map();
  function Px(n) {
    return Co.get(n);
  }
  function ld(n) {
    Co = /* @__PURE__ */ new Map();
    for (const t of n) Co.set(t.recipe, {
      inputs: t.inputs.map((e) => ({
        item: e.item,
        rate: e.rate
      })),
      outputs: t.outputs.map((e) => ({
        item: e.item,
        rate: e.rate
      })),
      machineCount: Math.ceil(t.count)
    });
  }
  function Sn(n) {
    if (!ad) return 7829367;
    if (!n) return 6710886;
    if (n in Nl) return Nl[n];
    let t = 0;
    for (let s = 0; s < n.length; s++) t = (t << 5) - t + n.charCodeAt(s) | 0;
    const e = Math.abs(t) % 30 * 12;
    return Mx(e / 360, 0.2, 0.48);
  }
  const tn = {
    "assembling-machine-1": [
      3,
      3
    ],
    "assembling-machine-2": [
      3,
      3
    ],
    "assembling-machine-3": [
      3,
      3
    ],
    "chemical-plant": [
      3,
      3
    ],
    "oil-refinery": [
      5,
      5
    ],
    "electric-furnace": [
      3,
      3
    ],
    "steel-furnace": [
      2,
      2
    ],
    "stone-furnace": [
      2,
      2
    ],
    centrifuge: [
      3,
      3
    ],
    lab: [
      3,
      3
    ],
    "rocket-silo": [
      9,
      9
    ],
    foundry: [
      5,
      5
    ],
    biochamber: [
      3,
      3
    ],
    biolab: [
      5,
      5
    ],
    "electromagnetic-plant": [
      4,
      4
    ],
    "cryogenic-plant": [
      5,
      5
    ],
    recycler: [
      2,
      4
    ],
    crusher: [
      2,
      3
    ],
    beacon: [
      3,
      3
    ],
    "storage-tank": [
      3,
      3
    ],
    "big-electric-pole": [
      2,
      2
    ],
    substation: [
      2,
      2
    ],
    "electric-mining-drill": [
      3,
      3
    ]
  };
  function ue(n) {
    return n.split("-").map((t) => t.charAt(0).toUpperCase() + t.slice(1)).join(" ");
  }
  const Re = new Set(Object.keys(tn)), er = new Set(Object.keys(rd)), un = new Set(Object.keys(Yn).filter((n) => !n.includes("underground") && !n.includes("splitter"))), be = new Set(Object.keys(Yn).filter((n) => n.includes("underground"))), ee = new Set(Object.keys(Yn).filter((n) => n.includes("splitter"))), Xs = /* @__PURE__ */ new Set([
    "pipe",
    "pipe-to-ground"
  ]), Jo = /* @__PURE__ */ new Set([
    "medium-electric-pole",
    "small-electric-pole"
  ]);
  function ei(n) {
    switch (n) {
      case "East":
        return Math.PI / 2;
      case "South":
        return Math.PI;
      case "West":
        return 3 * Math.PI / 2;
      default:
        return 0;
    }
  }
  function zn(n) {
    switch (n) {
      case "East":
        return [
          1,
          0
        ];
      case "South":
        return [
          0,
          1
        ];
      case "West":
        return [
          -1,
          0
        ];
      default:
        return [
          0,
          -1
        ];
    }
  }
  function Zo(n) {
    switch (n) {
      case "South":
      case "North":
        return [
          1,
          0
        ];
      case "East":
      case "West":
        return [
          0,
          1
        ];
      default:
        return [
          1,
          0
        ];
    }
  }
  function cd(n, t) {
    const e = n.direction ?? "North", [s, i] = zn(e);
    let r = false, o = null;
    for (const [a, l] of [
      [
        0,
        -1
      ],
      [
        1,
        0
      ],
      [
        0,
        1
      ],
      [
        -1,
        0
      ]
    ]) {
      const c = (n.x ?? 0) + a, h = (n.y ?? 0) + l, d = t.get(`${c},${h}`);
      if (!d || !(un.has(d.name) || be.has(d.name) && d.io_type === "output" || ee.has(d.name))) continue;
      const [p, f] = zn(d.direction), g = ee.has(d.name) ? c : d.x ?? 0, m = ee.has(d.name) ? h : d.y ?? 0;
      if (!(g + p !== (n.x ?? 0) || m + f !== (n.y ?? 0))) if (d.direction === e) r = true;
      else {
        const y = p * i - f * s;
        y !== 0 && (o = {
          turn: y > 0 ? "cw" : "ccw"
        });
      }
    }
    return o && !r ? o : null;
  }
  function Wl(n, t) {
    const e = Math.round((n >> 16 & 255) * t), s = Math.round((n >> 8 & 255) * t), i = Math.round((n & 255) * t);
    return e << 16 | s << 8 | i;
  }
  const qs = 3, hd = 3815994, Qo = 5592405, ta = 0.9, wi = k * (1 - ta) / 2, dd = k * ta;
  function ea(n, t) {
    const e = new ut(), s = k, i = dd, [, r] = Yn[n.name] ?? [
      11046960,
      14733424
    ], o = Sn(n.carries);
    if (t) Ix(e, s, r, n.direction, t, o);
    else {
      e.rect(wi, wi, i, i).fill(hd), e.setStrokeStyle({
        width: 1,
        color: Qo,
        alignment: 0
      }), e.rect(wi, wi, i, i).stroke();
      const a = new ut();
      a.x = s / 2, a.y = s / 2, a.rotation = ei(n.direction), a.rect(-i / 2, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(1, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(-1, -i / 2, 2, i).fill(657930), Rx(a, i, r), e.addChild(a);
    }
    return e;
  }
  function Ix(n, t, e, s, i, r) {
    const o = new ut();
    o.x = t / 2, o.y = t / 2, o.rotation = ei(s), o.scale.set(ta);
    const a = t / 2, c = (i.turn === "cw" ? 1 : -1) * a, h = -a, d = i.turn === "ccw" ? 0 : Math.PI / 2, u = i.turn === "ccw" ? Math.PI / 2 : Math.PI;
    o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().fill(hd), o.setStrokeStyle({
      width: 1,
      color: Qo,
      alignment: 0
    }), o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().stroke();
    const p = t * 0.5, f = 1.5;
    o.moveTo(c, h).arc(c, h, p - f, d, u, false).closePath().fill({
      color: r,
      alpha: 0.45
    });
    const g = Math.cos(d), m = Math.sin(d), y = Math.cos(u), b = Math.sin(u);
    o.moveTo(c + (p + f) * g, h + (p + f) * m).lineTo(c + t * g, h + t * m).arc(c, h, t, d, u, false).lineTo(c + (p + f) * y, h + (p + f) * b).arc(c, h, p + f, u, d, true).closePath().fill({
      color: r,
      alpha: 0.45
    });
    const x = t * 0.22, _ = Math.max(1, t * 0.07), v = t * 0.5, w = x / v, C = v + x, E = v - x;
    o.setStrokeStyle({
      width: _,
      color: e,
      cap: "round",
      join: "round"
    });
    const P = Math.PI / 2, T = i.turn === "cw" ? Math.PI : 0;
    for (const A of [
      0.6
    ]) {
      const H = P + A * (T - P), j = i.turn === "cw" ? H - w : H + w, D = c + v * Math.cos(H), N = h + v * Math.sin(H), $ = c + C * Math.cos(j), W = h + C * Math.sin(j), X = c + E * Math.cos(j), U = h + E * Math.sin(j);
      o.moveTo($, W).lineTo(D, N).lineTo(X, U).stroke();
    }
    n.addChild(o);
  }
  function Rx(n, t, e) {
    const s = t * 0.22;
    n.setStrokeStyle({
      width: Math.max(1, t * 0.07),
      color: e,
      cap: "round",
      join: "round"
    });
    for (const i of [
      -t * 0.22,
      t * 0.22
    ]) n.moveTo(-s, i + s * 0.5).lineTo(0, i - s * 0.5).lineTo(s, i + s * 0.5).stroke();
  }
  function ud(n) {
    const t = new ut(), e = k, [, s] = Yn[n.name] ?? [
      11046960,
      14733424
    ], i = n.io_type === "input", r = e / 2, o = i ? 1 : -1, a = new ut();
    a.x = r, a.y = r, a.rotation = ei(n.direction);
    const l = Sn(n.carries), c = dd / 2, h = e * 0.25, d = o * r, u = 0;
    a.moveTo(-c, d).lineTo(c, d).lineTo(h, u).lineTo(-h, u).closePath().fill({
      color: l,
      alpha: 0.7
    }), a.setStrokeStyle({
      width: 1,
      color: Qo,
      alpha: 0.8
    }), a.moveTo(-c, d).lineTo(-h, u).lineTo(h, u).lineTo(c, d).stroke();
    const p = e * 0.38, f = e * 0.3, g = o * e * 0.22, m = g - f / 2, y = g + f / 2;
    return a.moveTo(0, m).lineTo(p / 2, y).lineTo(-p / 2, y).closePath().fill(s), t.addChild(a), t;
  }
  function pd(n) {
    const t = new ut(), [e, s] = Yn[n.name] ?? [
      11046960,
      14733424
    ], i = n.direction === "North" || n.direction === "South", r = i ? k * 2 - 1 : k - 1, o = i ? k - 1 : k * 2 - 1, a = i ? r / 2 : o / 2, l = Math.max(2, Math.min(r, o) * 0.18);
    t.roundRect(0, 0, r, o, qs).fill(e), t.roundRect(0, 0, r, o, qs).fill({
      color: Sn(n.carries),
      alpha: 0.3
    }), i ? t.rect(a - l / 2, 0, l, o).fill(Wl(e, 0.5)) : t.rect(0, a - l / 2, r, l).fill(Wl(e, 0.5));
    const c = ei(n.direction), h = a * 0.25, d = Math.max(1, a * 0.12);
    for (let u = 0; u < 2; u++) {
      const p = i ? a * u + a / 2 : r / 2, f = i ? o / 2 : a * u + a / 2, g = new ut();
      g.x = p, g.y = f, g.rotation = c, g.setStrokeStyle({
        width: d,
        color: s,
        cap: "round"
      }), g.moveTo(-h, h * 0.5).lineTo(0, -h * 0.5).lineTo(h, h * 0.5).stroke(), t.addChild(g);
    }
    return t;
  }
  function fd(n) {
    const t = new ut(), e = k - 1, s = n.carries ? Sn(n.carries) : rd[n.name] ?? 6983230;
    t.roundRect(0, 0, e, e, qs).fill(2767402);
    const i = new ut();
    i.x = e / 2, i.y = e / 2, i.rotation = ei(n.direction), i.circle(0, e * 0.2, e * 0.15).fill(4473924);
    const r = Math.max(1.5, e * 0.12);
    i.setStrokeStyle({
      width: r,
      color: s,
      cap: "round"
    }), i.moveTo(0, e * 0.2).lineTo(0, -e * 0.35).stroke();
    const o = -e * 0.35, a = e * 0.18;
    return i.moveTo(-a, o - a * 0.6).lineTo(0, o).lineTo(a, o - a * 0.6).stroke(), t.addChild(i), t;
  }
  const na = 1, sa = 2, ia = 4, ra = 8;
  function Lx(n) {
    const [t, e] = zn(n.direction);
    return [
      -t,
      -e
    ];
  }
  function md(n, t, e) {
    if (n.name === "pipe") return true;
    if (n.name === "pipe-to-ground") {
      const [s, i] = Lx(n);
      return -t === s && -e === i;
    }
    return false;
  }
  function gd(n, t) {
    const e = new ut(), s = k - 1, i = n.name === "pipe-to-ground", r = i ? Sx : od;
    e.roundRect(0, 0, s, s, qs).fill(Ol);
    const o = s / 2, a = s / 2, l = Math.max(2, s * 0.4);
    if (i) {
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      });
      const [c, h] = zn(n.direction);
      e.moveTo(o, a).lineTo(o - c * s / 2, a - h * s / 2).stroke(), e.circle(o, a, l * 0.4).fill(r), e.circle(o, a, l * 0.25).fill(Ol);
    } else if (t === 0) e.circle(o, a, l * 0.4).fill(r);
    else {
      const c = !!(t & na), h = !!(t & sa), d = !!(t & ia), u = !!(t & ra), p = (c ? 1 : 0) + (h ? 1 : 0) + (d ? 1 : 0) + (u ? 1 : 0);
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      }), p === 1 ? (c ? e.moveTo(o, a).lineTo(o, 0).stroke() : h ? e.moveTo(o, a).lineTo(s, a).stroke() : d ? e.moveTo(o, a).lineTo(o, s).stroke() : e.moveTo(o, a).lineTo(0, a).stroke(), e.circle(o, a, l * 0.4).fill(r)) : c && d && !h && !u ? e.moveTo(o, 0).lineTo(o, s).stroke() : h && u && !c && !d ? e.moveTo(0, a).lineTo(s, a).stroke() : p === 2 ? c && h ? e.moveTo(o, 0).quadraticCurveTo(o, a, s, a).stroke() : h && d ? e.moveTo(s, a).quadraticCurveTo(o, a, o, s).stroke() : d && u ? e.moveTo(o, s).quadraticCurveTo(o, a, 0, a).stroke() : e.moveTo(0, a).quadraticCurveTo(o, a, o, 0).stroke() : p === 3 ? u ? d ? h ? (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, s).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(0, a).stroke()) : (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, 0).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(s, a).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(0, a).lineTo(s, a).stroke());
    }
    return e;
  }
  function yd() {
    const n = new ut(), t = k - 1;
    n.roundRect(0, 0, t, t, qs).fill(kx);
    const e = t / 2, s = t / 2, i = t * 0.38, r = Math.max(1.5, t * 0.2);
    return n.rect(e - r / 2, s - i, r, i * 2).fill(Fl), n.rect(e - i, s - r / 2, i * 2, r).fill(Fl), n.circle(e, s, r * 0.6).fill(14729280), n;
  }
  function xd(n) {
    const t = new ut(), [e, s] = tn[n.name] ?? [
      1,
      1
    ], i = e * k - 1, r = s * k - 1;
    t.setStrokeStyle({
      width: 1,
      color: 11184810,
      alpha: 0.4
    });
    const o = 3, a = 3;
    for (const [h, d, u, p] of [
      [
        0,
        0,
        i,
        0
      ],
      [
        i,
        0,
        i,
        r
      ],
      [
        i,
        r,
        0,
        r
      ],
      [
        0,
        r,
        0,
        0
      ]
    ]) {
      const f = Math.sqrt((u - h) ** 2 + (p - d) ** 2), g = (u - h) / f, m = (p - d) / f;
      let y = 0;
      for (; y < f; ) {
        const b = Math.min(y + o, f);
        t.moveTo(h + g * y, d + m * y).lineTo(h + g * b, d + m * b).stroke(), y = b + a;
      }
    }
    const l = 1.8, c = Gl(`/fucktorio/pr-238/entity-frames/${n.name}.png`);
    if (c) {
      const h = new De(c), d = k / $x;
      h.scale.set(d * l), h.x = -i * (l - 1) / 2, h.y = -r * (l - 1) / 2, t.addChild(h);
    } else {
      const h = Gl(`/fucktorio/pr-238/icons/${n.name}.png`);
      if (h) {
        const d = new De(h), u = Math.min(i, r) * 0.8 * l;
        d.width = u, d.height = u, d.x = (i - u) / 2, d.y = (r - u) / 2, t.addChild(d);
      } else {
        const d = vx[n.name] ?? Cx;
        t.roundRect(2, 2, i - 4, r - 4, 3).fill({
          color: d,
          alpha: 0.5
        });
      }
    }
    return t;
  }
  function bd() {
    const n = new ut(), t = k - 1;
    return n.rect(0, 0, t, t).fill(4872810), n.setStrokeStyle({
      width: 1,
      color: 0,
      alpha: 0.4
    }), n.rect(0, 0, t, t).stroke(), n;
  }
  const $x = 64;
  async function Bx(n) {
    const t = "/fucktorio/pr-238/", e = [
      ...n.map((s) => `${t}icons/${s}.png`),
      ...n.map((s) => `${t}entity-frames/${s}.png`)
    ];
    await Promise.allSettled(e.map((s) => ze.load(s)));
  }
  async function _d(n) {
    const t = "/fucktorio/pr-238/";
    await Promise.allSettled(n.map((e) => ze.load(`${t}icons/${e}.png`)));
  }
  function Ox(n) {
    const t = /* @__PURE__ */ new Set();
    for (const e of n) e.carries && t.add(e.carries);
    return Array.from(t);
  }
  function Gl(n) {
    return te.has(n) ? ze.get(n) ?? null : null;
  }
  const Fx = {
    "assembling-machine-2": [
      [
        1,
        -1,
        "input",
        "always"
      ],
      [
        1,
        3,
        "output",
        "always"
      ]
    ],
    "assembling-machine-3": [
      [
        1,
        -1,
        "input",
        "always"
      ],
      [
        1,
        3,
        "output",
        "always"
      ]
    ],
    "chemical-plant": [
      [
        0,
        -1,
        "input",
        "always"
      ],
      [
        2,
        -1,
        "input",
        "always"
      ],
      [
        0,
        3,
        "output",
        "always"
      ],
      [
        2,
        3,
        "output",
        "always"
      ]
    ],
    "oil-refinery": [
      [
        1,
        5,
        "input",
        "default"
      ],
      [
        3,
        5,
        "input",
        "default"
      ],
      [
        0,
        -1,
        "output",
        "default"
      ],
      [
        2,
        -1,
        "output",
        "default"
      ],
      [
        4,
        -1,
        "output",
        "default"
      ],
      [
        1,
        -1,
        "input",
        "mirror"
      ],
      [
        3,
        -1,
        "input",
        "mirror"
      ],
      [
        0,
        5,
        "output",
        "mirror"
      ],
      [
        2,
        5,
        "output",
        "mirror"
      ],
      [
        4,
        5,
        "output",
        "mirror"
      ]
    ]
  };
  function wd(n) {
    const t = Fx[n.name];
    if (!t) return [];
    const e = n.mirror ?? false;
    return t.filter(([, , , s]) => s === "always" || s === "default" && !e || s === "mirror" && e);
  }
  function vd(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of wd(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  function nr() {
    return {
      tileMap: /* @__PURE__ */ new Map(),
      machineByTile: /* @__PURE__ */ new Map()
    };
  }
  function Di(n, t) {
    const e = n.x ?? 0, s = n.y ?? 0;
    if (t.tileMap.set(`${e},${s}`, n), ee.has(n.name)) {
      const [i, r] = Zo(n.direction);
      t.tileMap.set(`${e + i},${s + r}`, n);
    }
    if (Re.has(n.name)) {
      const [i, r] = tn[n.name] ?? [
        1,
        1
      ];
      for (let o = 0; o < r; o++) for (let a = 0; a < i; a++) t.machineByTile.set(`${e + a},${s + o}`, n);
    }
  }
  function Mn(n, t) {
    let e;
    if (un.has(n.name)) e = ea(n, cd(n, t.tileMap));
    else if (be.has(n.name)) e = ud(n);
    else if (ee.has(n.name)) e = pd(n);
    else if (er.has(n.name)) e = fd(n);
    else if (Xs.has(n.name)) {
      let s = 0;
      if (n.name === "pipe") {
        const i = n.x ?? 0, r = n.y ?? 0;
        for (const [o, a, l] of [
          [
            0,
            -1,
            na
          ],
          [
            1,
            0,
            sa
          ],
          [
            0,
            1,
            ia
          ],
          [
            -1,
            0,
            ra
          ]
        ]) {
          const c = `${i + o},${r + a}`, h = t.tileMap.get(c);
          if (h && md(h, o, a)) {
            s |= l;
            continue;
          }
          const d = t.machineByTile.get(c);
          d && vd(i, r, d) && (s |= l);
        }
      }
      e = gd(n, s);
    } else Jo.has(n.name) ? e = yd() : Re.has(n.name) ? e = xd(n) : e = bd();
    return e.x = (n.x ?? 0) * k, e.y = (n.y ?? 0) * k, e;
  }
  zs = function(n, t, e, s, i, r) {
    t.removeChildren();
    const o = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (o.set(`${f.x ?? 0},${f.y ?? 0}`, f), ee.has(f.name)) {
      const [g, m] = Zo(f.direction);
      o.set(`${(f.x ?? 0) + g},${(f.y ?? 0) + m}`, f);
    }
    if (r) for (const f of r) {
      const g = `${f.x ?? 0},${f.y ?? 0}`;
      o.has(g) || o.set(g, f);
    }
    const a = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (Re.has(f.name)) {
      const [g, m] = tn[f.name] ?? [
        1,
        1
      ], y = f.x ?? 0, b = f.y ?? 0;
      for (let x = 0; x < m; x++) for (let _ = 0; _ < g; _++) a.set(`${y + _},${b + x}`, f);
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const m of n.entities) be.has(m.name) && f.set(`${m.x ?? 0},${m.y ?? 0}`, m);
      const g = 8;
      for (const m of n.entities) {
        if (!be.has(m.name) || m.io_type !== "input") continue;
        const [y, b] = zn(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 1; v <= g; v++) {
          const w = f.get(`${x + y * v},${_ + b * v}`);
          if (w) {
            if (be.has(w.name) && w.name === m.name && w.direction === m.direction && w.io_type === "input") break;
            if (be.has(w.name) && w.name === m.name && w.direction === m.direction && w.io_type === "output") {
              const [C] = Yn[m.name] ?? [
                11046960,
                14733424
              ], E = new ut(), P = Math.abs(y) > 0;
              for (let T = 1; T < v; T++) {
                const A = (x + y * T) * k, H = (_ + b * T) * k;
                P ? E.rect(A, H + k * 0.25, k, k * 0.5).fill({
                  color: C,
                  alpha: 0.25
                }) : E.rect(A + k * 0.25, H, k * 0.5, k).fill({
                  color: C,
                  alpha: 0.25
                });
              }
              t.addChild(E);
              break;
            }
          }
        }
      }
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const m of n.entities) m.name === "pipe-to-ground" && f.set(`${m.x ?? 0},${m.y ?? 0}`, m);
      const g = 10;
      for (const m of n.entities) {
        if (m.name !== "pipe-to-ground" || m.io_type !== "input") continue;
        const [y, b] = zn(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 2; v <= g; v++) {
          const w = f.get(`${x + y * v},${_ + b * v}`);
          if (!w) continue;
          const [C, E] = zn(w.direction);
          if (w.io_type !== "output" || C !== -y || E !== -b) break;
          const P = new ut();
          P.setStrokeStyle({
            width: 2,
            color: od,
            alpha: 0.55,
            cap: "round"
          });
          const T = (x + 0.5 + y * 0.5) * k, A = (_ + 0.5 + b * 0.5) * k, H = (v - 1) * k, j = 5, D = 3;
          let N = 0;
          for (; N < H; ) {
            const $ = Math.min(N + j, H);
            P.moveTo(T + y * N, A + b * N).lineTo(T + y * $, A + b * $).stroke(), N = $ + D;
          }
          t.addChild(P);
          break;
        }
      }
    }
    const l = /* @__PURE__ */ new Map(), c = [], h = /* @__PURE__ */ new Map();
    for (const f of n.entities) {
      let g;
      if (un.has(f.name)) g = ea(f, cd(f, o));
      else if (be.has(f.name)) g = ud(f);
      else if (ee.has(f.name)) g = pd(f);
      else if (er.has(f.name)) g = fd(f);
      else if (Xs.has(f.name)) {
        let y = 0;
        if (f.name === "pipe") {
          const b = f.x ?? 0, x = f.y ?? 0;
          for (const [_, v, w] of [
            [
              0,
              -1,
              na
            ],
            [
              1,
              0,
              sa
            ],
            [
              0,
              1,
              ia
            ],
            [
              -1,
              0,
              ra
            ]
          ]) {
            if (v === -1 && x + v < 0) {
              y |= w;
              continue;
            }
            const C = `${b + _},${x + v}`, E = o.get(C);
            if (E && md(E, _, v)) {
              y |= w;
              continue;
            }
            const P = a.get(C);
            P && vd(b, x, P) && (y |= w);
          }
        }
        g = gd(f, y);
      } else Jo.has(f.name) ? g = yd() : Re.has(f.name) ? g = xd(f) : g = bd();
      g.x = (f.x ?? 0) * k, g.y = (f.y ?? 0) * k, s && (g.eventMode = "static", g.cursor = "pointer", g.on("click", () => s(f)));
      const m = zl(f);
      m && (l.has(m) || l.set(m, []), l.get(m).push(g)), h.set(g, `${f.x ?? 0},${f.y ?? 0}`), c.push(g), t.addChild(g), i == null ? void 0 : i(f, [
        g
      ]);
    }
    const d = mx(n);
    let u = null;
    function p() {
      u && (t.removeChild(u), u.destroy(), u = null);
      for (const f of c) f.alpha = 1;
    }
    return {
      highlightItem(f) {
        if (p(), !f) return;
        const g = l.get(f);
        if (!g || g.length === 0) return;
        const m = new Set(g);
        for (const y of c) y.alpha = m.has(y) ? 1 : 0.15;
      },
      highlightBeltNetwork(f) {
        if (p(), !f) return;
        const g = `${f.x ?? 0},${f.y ?? 0}`, m = d.tileToAnchor.get(g) ?? g;
        if (!d.nodes.has(m)) return;
        const { downstream: y, upstream: b } = gx(m, d), x = /* @__PURE__ */ new Set([
          ...y,
          ...b
        ]), _ = yx(x, d.entityMap), v = xx(_, d.entityMap);
        for (const w of c) {
          const C = h.get(w);
          if (!C) {
            w.alpha = 0.15;
            continue;
          }
          x.has(C) ? w.alpha = 0.5 : _.has(C) ? w.alpha = 0.9 : v.has(C) ? w.alpha = 0.75 : w.alpha = 0.15;
        }
        u = new ut(), wx(u, y, b, m, d), t.addChild(u);
      },
      clearHighlight() {
        p();
      },
      chainKey: zl
    };
  };
  function zl(n) {
    return n.carries ? n.carries : n.recipe ? n.recipe : null;
  }
  const Ks = 4096, mt = 128, qe = Ks / mt;
  let vn = null;
  const Js = /* @__PURE__ */ new Map();
  let Je = 0, Zs = null;
  function Nx(n) {
    Zs = n;
  }
  function on(n, t, e, s) {
    const i = Js.get(n);
    if (i) return i;
    if (!Zs) return console.warn("[atlas] getEntityTexture called before initAtlas; returning blank texture"), Ct.EMPTY;
    vn || (vn = Zi.create({
      width: Ks,
      height: Ks
    })), Je >= qe * qe && (console.warn("[atlas] atlas is full \u2014 variant will reuse slot 0:", n), Je = 0);
    const r = Je % qe, o = Math.floor(Je / qe), a = r * mt, l = o * mt;
    Je++;
    const c = new ut();
    s(c);
    const h = new _t(1, 0, 0, 1, a, l);
    Zs.render({
      container: c,
      target: vn,
      transform: h,
      clear: false
    }), c.destroy({
      children: true
    });
    const d = new Nt(a, l, mt, mt), u = new Ct({
      source: vn.source,
      frame: d
    });
    return Js.set(n, u), u;
  }
  function Dl(n, t, e, s) {
    const i = Js.get(n);
    if (i) return i;
    if (!Zs) return console.warn("[atlas] getMultiCellTexture called before initAtlas; returning blank texture"), Ct.EMPTY;
    vn || (vn = Zi.create({
      width: Ks,
      height: Ks
    }));
    const r = Je % qe;
    r + t > qe && (Je += qe - r);
    const a = Je % qe, l = Math.floor(Je / qe), c = a * mt, h = l * mt;
    Je = (l + e) * qe;
    const d = t * mt, u = e * mt, p = new ut();
    s(p, d, u);
    const f = new _t(1, 0, 0, 1, c, h);
    Zs.render({
      container: p,
      target: vn,
      transform: f,
      clear: false
    }), p.destroy({
      children: true
    });
    const g = new Nt(c, h, d, u), m = new Ct({
      source: vn.source,
      frame: g
    });
    return Js.set(n, m), m;
  }
  function Cd(n) {
    const t = `icon:${n}`, e = Js.get(t);
    if (e) return e;
    const s = `/fucktorio/pr-238/icons/${n}.png`;
    if (te.has(s)) {
      const r = ze.get(s);
      if (r) return on(t, mt, mt, (a) => {
        const c = mt - 16;
        a.rect(8, 8, c, c).fill({
          texture: r
        });
      });
    }
    const i = Sn(n);
    return on(t, mt, mt, (r) => {
      const o = mt / 2, a = mt / 2;
      r.circle(o, a, 7).fill({
        color: i,
        alpha: 0.85
      });
    });
  }
  function Wx(n, t, e = "straight") {
    return `belt:${n}:${t}:${e}`;
  }
  function Gx(n) {
    return `pipe:${n}`;
  }
  function zx(n, t, e) {
    return `ugbelt:${n}:${t}:${e}`;
  }
  function Dx(n, t) {
    return `splitter:${n}:${t}`;
  }
  function Hx(n, t) {
    return `inserter:${n}:${t}`;
  }
  function Ux(n) {
    return `machine:${n}`;
  }
  function jx(n) {
    return `pole:${n}`;
  }
  function Vx(n) {
    return `ptg:${n}`;
  }
  const Ke = 3200;
  let Sd = null, kd = null, Td = null;
  function Ze() {
    Sd == null ? void 0 : Sd();
  }
  function sr() {
    kd == null ? void 0 : kd();
  }
  function Dn() {
    Td == null ? void 0 : Td();
  }
  async function Yx(n) {
    const t = new No();
    await t.init({
      resizeTo: n,
      background: 1973790,
      antialias: true,
      autoStart: false,
      sharedTicker: false
    }), Nx(t.renderer), t.ticker.add(() => t.render(), null, Hs.LOW), n.appendChild(t.canvas), t.canvas.addEventListener("contextmenu", (h) => h.preventDefault());
    const e = new id({
      screenWidth: n.clientWidth,
      screenHeight: n.clientHeight,
      worldWidth: Ke,
      worldHeight: Ke,
      events: t.renderer.events
    });
    e.drag({
      mouseButtons: "left"
    }).pinch().wheel().decelerate(), t.stage.addChild(e);
    let s = false, i = 0;
    const r = () => {
      i > 0 || s || (s = true, queueMicrotask(() => {
        s = false, t.render();
      }));
    }, o = () => {
      i === 0 && t.ticker.start(), i++;
    }, a = () => {
      i !== 0 && (i--, i === 0 && t.ticker.stop());
    };
    Sd = r, kd = o, Td = a, e.on("moved", r), e.on("zoomed", r);
    const l = [
      "drag-start",
      "pinch-start",
      "snap-start",
      "snap-zoom-start",
      "bounce-x-start",
      "bounce-y-start"
    ], c = [
      "drag-end",
      "pinch-end",
      "snap-end",
      "snap-zoom-end",
      "bounce-x-end",
      "bounce-y-end"
    ];
    for (const h of l) e.on(h, o);
    for (const h of c) e.on(h, a);
    return window.addEventListener("resize", () => {
      e.resize(n.clientWidth, n.clientHeight, Ke, Ke), r();
    }), r(), {
      app: t,
      viewport: e,
      requestRender: r,
      beginAnimating: o,
      endAnimating: a
    };
  }
  const vi = 32, Hl = 2763306, Ul = 3815994;
  function Xx(n) {
    const t = new ut();
    return n.addChildAt(t, 0), t;
  }
  function Pn(n, t, e, s = 1) {
    if (n.clear(), t <= 0 || e <= 0) return;
    const i = Math.max(0, Math.min(1, s));
    if (i === 0) return;
    const r = t * vi, a = e * vi * i;
    for (let l = 0; l <= t; l++) {
      const c = l * vi, h = l % 10 === 0;
      n.moveTo(c, 0).lineTo(c, a).stroke({
        width: h ? 1.5 : 1,
        color: h ? Ul : Hl
      });
    }
    for (let l = 0; l <= e; l++) {
      const c = l * vi;
      if (c > a) break;
      const h = l % 10 === 0;
      n.moveTo(0, c).lineTo(r, c).stroke({
        width: h ? 1.5 : 1,
        color: h ? Ul : Hl
      });
    }
  }
  const Li = {
    "assembling-machine-1": 5926530,
    "assembling-machine-2": 4874872,
    "assembling-machine-3": 3822186,
    "stone-furnace": 9068608,
    "steel-furnace": 8015920,
    "electric-furnace": 6969984,
    "chemical-plant": 3832400,
    "oil-refinery": 5913226,
    centrifuge: 3832448,
    lab: 4876880,
    "rocket-silo": 4868714,
    "transport-belt": 13153632,
    "fast-transport-belt": 14700624,
    "express-transport-belt": 5284064,
    "underground-belt": 11046976,
    "fast-underground-belt": 14700624,
    "express-underground-belt": 5284064,
    splitter: 13153632,
    "fast-splitter": 14700624,
    "express-splitter": 5284064,
    inserter: 6983230,
    "fast-inserter": 4886736,
    "long-handed-inserter": 13647936,
    pipe: 4881077,
    "pipe-to-ground": 3825808,
    pump: 4881002,
    "medium-electric-pole": 9136404,
    "small-electric-pole": 10910752
  }, qx = 8947848;
  function Ad(n) {
    const t = n >> 16 & 255, e = n >> 8 & 255, s = n & 255;
    return `rgb(${t},${e},${s})`;
  }
  const Kx = [
    {
      name: "transport-belt",
      color: Li["transport-belt"],
      throughput: 15
    },
    {
      name: "fast-transport-belt",
      color: Li["fast-transport-belt"],
      throughput: 30
    },
    {
      name: "express-transport-belt",
      color: Li["express-transport-belt"],
      throughput: 45
    }
  ];
  function Ed(n) {
    for (const t of Kx) if (n <= t.throughput) return t;
    return null;
  }
  const Jx = 240, jl = 80, oa = 180, Zx = 60, Vl = 100, Yl = 100, Xl = "production-graph";
  function Qx(n) {
    return n <= 15 ? 13153632 : n <= 30 ? 14700624 : n <= 45 ? 5284064 : 16711816;
  }
  const ql = new Le({
    fontSize: 13,
    fontWeight: "bold",
    fill: 14737632,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: oa - 12
  }), Kl = new Le({
    fontSize: 11,
    fill: 10280190,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: oa - 12
  }), Jl = new Le({
    fontSize: 10,
    fill: 16777215,
    fontFamily: "sans-serif"
  });
  function Ps(n, t) {
    const e = n.getChildByName(Xl);
    e && (e.destroy({
      children: true
    }), n.removeChild(e));
    const s = new zt();
    if (s.label = Xl, n.addChild(s), !t || t.machines.length === 0) return s;
    const { dependency_order: i } = t, r = i.length, o = /* @__PURE__ */ new Map();
    i.forEach((x, _) => {
      o.set(x, r - 1 - _);
    });
    const a = 1, l = /* @__PURE__ */ new Map();
    for (const x of t.machines) {
      const _ = o.get(x.recipe) ?? 0;
      l.has(_) || l.set(_, []), l.get(_).push(x);
    }
    for (const x of l.values()) x.sort((_, v) => _.recipe.localeCompare(v.recipe));
    const c = [
      ...new Set(t.external_inputs.map((x) => x.item))
    ].sort(), h = /* @__PURE__ */ new Map();
    for (const x of t.machines) for (const _ of x.outputs) h.set(_.item, x);
    const d = /* @__PURE__ */ new Map();
    for (const x of t.external_inputs) d.set(x.item, x.rate);
    const u = /* @__PURE__ */ new Map();
    for (const [x, _] of l) _.forEach((v, w) => {
      u.set(v.recipe, {
        x: Vl + (x + a) * Jx,
        y: Yl + w * jl,
        w: oa,
        h: Zx,
        machine: v
      });
    });
    const p = /* @__PURE__ */ new Map();
    c.forEach((x, _) => {
      p.set(x, {
        x: Vl,
        y: Yl + _ * jl,
        w: 140,
        h: 40
      });
    });
    const f = new ut();
    s.addChild(f);
    for (const x of t.machines) {
      const _ = u.get(x.recipe);
      if (_) for (const v of x.inputs) {
        const w = h.get(v.item), C = w ? u.get(w.recipe) : p.get(v.item);
        if (!C) continue;
        const E = Qx(v.rate), P = C.x + C.w, T = C.y + C.h * 2 / 3, A = _.x, H = _.y + _.h / 3, j = (P + A) / 2;
        f.moveTo(P, T).lineTo(j, T).lineTo(j, H).lineTo(A, H).stroke({
          color: E,
          width: 2,
          alpha: 0.85
        });
        const D = `${v.rate.toFixed(1)}/s ${v.item}`, N = (P + j) / 2, $ = T - 14, W = rn.measureText(D, Jl), X = new ut();
        X.rect(N - 2, $ - 1, W.width + 4, W.height + 2).fill({
          color: 1973790,
          alpha: 0.7
        }), s.addChild(X);
        const U = new Ge({
          text: D,
          style: Jl
        });
        U.position.set(N, $), s.addChild(U);
      }
    }
    for (const x of u.values()) {
      const _ = x.machine, v = Li[_.entity] ?? qx, w = new ut();
      w.rect(x.x, x.y, x.w, x.h).fill({
        color: v,
        alpha: 0.6
      }).stroke({
        color: v,
        width: 2
      }), s.addChild(w);
      const C = new Ge({
        text: `${_.count.toFixed(1)} \xD7 ${_.entity}`,
        style: ql
      });
      C.position.set(x.x + 6, x.y + 6), s.addChild(C);
      const E = new Ge({
        text: _.recipe,
        style: Kl
      });
      E.position.set(x.x + 6, x.y + 24), s.addChild(E);
    }
    for (const [x, _] of p) {
      const v = d.get(x), w = v !== void 0 ? `${v.toFixed(1)}/s` : "", C = new ut();
      C.rect(_.x, _.y, _.w, _.h).fill({
        color: 2763306,
        alpha: 0.8
      }).stroke({
        color: 8947848,
        width: 1.5
      }), s.addChild(C);
      const E = new Ge({
        text: w,
        style: ql
      });
      E.position.set(_.x + 6, _.y + 4), s.addChild(E);
      const P = new Ge({
        text: x,
        style: Kl
      });
      P.position.set(_.x + 6, _.y + 20), s.addChild(P);
    }
    let g = 1 / 0, m = -1 / 0, y = 1 / 0, b = -1 / 0;
    for (const x of [
      ...u.values(),
      ...p.values()
    ]) x.x < g && (g = x.x), x.x + x.w > m && (m = x.x + x.w), x.y < y && (y = x.y), x.y + x.h > b && (b = x.y + x.h);
    return n.moveCenter((g + m) / 2, (y + b) / 2), s;
  }
  function ne(n, t) {
    return `${n},${t}`;
  }
  function xe(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function yn(n) {
    switch (n) {
      case "East":
        return [
          1,
          0
        ];
      case "South":
        return [
          0,
          1
        ];
      case "West":
        return [
          -1,
          0
        ];
      default:
        return [
          0,
          -1
        ];
    }
  }
  function de(n, t, e) {
    if (t === e) return;
    let s = n.get(t);
    s || (s = [], n.set(t, s)), s.includes(e) || s.push(e);
    let i = n.get(e);
    i || (i = [], n.set(e, i)), i.includes(t) || i.push(t);
  }
  function t0(n) {
    const t = /* @__PURE__ */ new Map();
    for (const e of n.entities) {
      const s = e.x ?? 0, i = e.y ?? 0, r = xe(e);
      if (t.set(ne(s, i), r), ee.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South";
        t.set(ne(s + (o ? 1 : 0), i + (o ? 0 : 1)), r);
      }
      if (Re.has(e.name)) {
        const [o, a] = tn[e.name] ?? [
          1,
          1
        ];
        for (let l = 0; l < a; l++) for (let c = 0; c < o; c++) c === 0 && l === 0 || t.set(ne(s + c, i + l), r);
      }
    }
    return t;
  }
  const e0 = 9;
  function Md(n) {
    const t = /* @__PURE__ */ new Map(), e = t0(n);
    for (const o of n.entities) {
      const a = xe(o);
      t.has(a) || t.set(a, []);
    }
    const s = /* @__PURE__ */ new Map();
    for (const o of n.entities) s.set(ne(o.x ?? 0, o.y ?? 0), o);
    const i = [
      [
        0,
        -1
      ],
      [
        1,
        0
      ],
      [
        0,
        1
      ],
      [
        -1,
        0
      ]
    ];
    for (const o of n.entities) {
      const a = o.x ?? 0, l = o.y ?? 0, c = xe(o), [h, d] = yn(o.direction);
      if (un.has(o.name)) {
        const u = e.get(ne(a + h, l + d));
        u && de(t, c, u);
        const p = e.get(ne(a - h, l - d));
        p && de(t, c, p);
        for (const [f, g] of i) {
          if (f === h && g === d || f === -h && g === -d) continue;
          const m = s.get(ne(a + f, l + g));
          if (!m || !un.has(m.name) && !be.has(m.name) && !ee.has(m.name)) continue;
          const [y, b] = yn(m.direction), x = ee.has(m.name) ? a + f : m.x ?? 0, _ = ee.has(m.name) ? l + g : m.y ?? 0;
          x + y === a && _ + b === l && de(t, c, xe(m));
        }
      } else if (be.has(o.name)) {
        if (o.io_type === "input") for (let u = 1; u <= e0; u++) {
          const p = s.get(ne(a + h * u, l + d * u));
          if (p) {
            if (be.has(p.name) && p.name === o.name && p.io_type === "input" && p.direction === o.direction) break;
            if (be.has(p.name) && p.name === o.name && p.io_type === "output" && p.direction === o.direction) {
              de(t, c, xe(p));
              break;
            }
          }
        }
        else {
          const u = e.get(ne(a + h, l + d));
          u && de(t, c, u);
        }
        for (const [u, p] of i) {
          const f = s.get(ne(a + u, l + p));
          if (!f || !un.has(f.name) && !ee.has(f.name)) continue;
          const [g, m] = yn(f.direction);
          (f.x ?? 0) + g === a && (f.y ?? 0) + m === l && de(t, c, xe(f));
        }
      } else if (ee.has(o.name)) {
        const u = o.direction === "North" || o.direction === "South", [p, f] = u ? [
          1,
          0
        ] : [
          0,
          1
        ];
        for (const [g, m] of [
          [
            0,
            0
          ],
          [
            p,
            f
          ]
        ]) {
          const y = e.get(ne(a + g + h, l + m + d));
          y && y !== c && de(t, c, y);
          const b = e.get(ne(a + g - h, l + m - d));
          b && b !== c && de(t, c, b);
        }
      }
    }
    for (const o of n.entities) {
      if (!er.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = xe(o), [h, d] = yn(o.direction), p = o.name === "long-handed-inserter" ? 2 : 1, f = e.get(ne(a - h * p, l - d * p)), g = e.get(ne(a + h * p, l + d * p));
      f && de(t, c, f), g && de(t, c, g);
    }
    const r = 10;
    for (const o of n.entities) {
      if (!Xs.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = xe(o);
      if (o.name === "pipe") for (const [h, d] of i) {
        const u = s.get(ne(a + h, l + d));
        if (u) {
          if (u.name === "pipe") de(t, c, xe(u));
          else if (u.name === "pipe-to-ground") {
            const [p, f] = yn(u.direction);
            h === p && d === f && de(t, c, xe(u));
          } else if (Re.has(u.name)) {
            const p = e.get(ne(u.x ?? 0, u.y ?? 0));
            p && de(t, c, p);
          }
        }
      }
      else if (o.name === "pipe-to-ground") {
        if (o.io_type === "input") {
          const [p, f] = yn(o.direction);
          for (let g = 2; g <= r; g++) {
            const m = s.get(ne(a + p * g, l + f * g));
            if (!m || m.name !== "pipe-to-ground") continue;
            const [y, b] = yn(m.direction);
            if (m.io_type === "output" && y === -p && b === -f) {
              de(t, c, xe(m));
              break;
            }
            break;
          }
        }
        const [h, d] = yn(o.direction), u = s.get(ne(a - h, l - d));
        u && u.name === "pipe" && de(t, c, xe(u));
      }
    }
    for (const o of n.entities) {
      if (!Re.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = xe(o), [h, d] = tn[o.name] ?? [
        1,
        1
      ];
      for (let u = 0; u < d; u++) for (let p = 0; p < h; p++) for (const [f, g] of i) {
        const m = a + p + f, y = l + u + g;
        if (m >= a && m < a + h && y >= l && y < l + d) continue;
        const b = s.get(ne(m, y));
        b && Xs.has(b.name) && de(t, c, xe(b));
      }
    }
    return t;
  }
  function Pd(n, t) {
    const e = /* @__PURE__ */ new Map();
    if (!n.has(t)) return e;
    e.set(t, 0);
    const s = [
      t
    ];
    let i = 0;
    for (; i < s.length; ) {
      const r = s[i++], o = e.get(r);
      for (const a of n.get(r) ?? []) e.has(a) || (e.set(a, o + 1), s.push(a));
    }
    return e;
  }
  const Id = 150, n0 = 64, Hi = 0.2, s0 = 5, i0 = 100, r0 = 200, o0 = (n) => 1 - Math.pow(1 - n, 3), a0 = (n) => n;
  function Ci() {
    return new Ly({
      dynamicProperties: {
        color: true,
        position: false,
        rotation: false,
        vertex: false,
        uvs: false
      }
    });
  }
  function Rd() {
    const n = Ci(), t = new zt(), e = Ci(), s = Ci(), i = Ci();
    return {
      beltContainer: n,
      pipeStubLayer: t,
      machineContainer: e,
      ghostContainer: s,
      iconContainer: i,
      layout: null,
      attachTo(o) {
        o.addChild(n), o.addChild(t), o.addChild(e), o.addChild(s), o.addChild(i);
      },
      clear() {
        n.removeParticles(), t.removeChildren(), e.removeParticles(), s.removeParticles(), i.removeParticles(), se.clear(), gs.clear();
      },
      count() {
        return n.particleChildren.length + e.particleChildren.length + s.particleChildren.length + i.particleChildren.length;
      }
    };
  }
  const se = /* @__PURE__ */ new Map();
  function is(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function l0(n, t) {
    const e = n.direction ?? "North", s = {
      North: [
        0,
        -1
      ],
      East: [
        1,
        0
      ],
      South: [
        0,
        1
      ],
      West: [
        -1,
        0
      ]
    }, [i, r] = s[e] ?? [
      0,
      -1
    ];
    let o = false, a = null;
    for (const [l, c] of [
      [
        0,
        -1
      ],
      [
        1,
        0
      ],
      [
        0,
        1
      ],
      [
        -1,
        0
      ]
    ]) {
      const h = (n.x ?? 0) + l, d = (n.y ?? 0) + c, u = t.get(`${h},${d}`);
      if (!u || !(un.has(u.name) || be.has(u.name) && u.io_type === "output" || ee.has(u.name))) continue;
      const [f, g] = s[u.direction ?? "North"] ?? [
        0,
        -1
      ], m = ee.has(u.name) ? h : u.x ?? 0, y = ee.has(u.name) ? d : u.y ?? 0;
      if (!(m + f !== (n.x ?? 0) || y + g !== (n.y ?? 0))) if (u.direction === e) o = true;
      else {
        const b = f * r - g * i;
        b !== 0 && (a = b > 0 ? "cw" : "ccw");
      }
    }
    return a && !o ? a === "cw" ? "corner-cw" : "corner-ccw" : "straight";
  }
  function c0(n, t) {
    if (n.name !== "pipe") return 0;
    const e = n.x ?? 0, s = n.y ?? 0;
    function i(o, a, l) {
      if (o.name === "pipe") return true;
      if (o.name === "pipe-to-ground") {
        const c = {
          North: [
            0,
            -1
          ],
          East: [
            1,
            0
          ],
          South: [
            0,
            1
          ],
          West: [
            -1,
            0
          ]
        }, [h, d] = c[o.direction ?? "North"] ?? [
          0,
          -1
        ], [u, p] = [
          -h,
          -d
        ];
        return -a === u && -l === p;
      }
      return false;
    }
    let r = 0;
    for (const [o, a, l] of [
      [
        0,
        -1,
        1
      ],
      [
        1,
        0,
        2
      ],
      [
        0,
        1,
        4
      ],
      [
        -1,
        0,
        8
      ]
    ]) {
      if (a === -1 && s + a < 0) {
        r |= l;
        continue;
      }
      const c = `${e + o},${s + a}`, h = t.tileMap.get(c);
      if (h && i(h, o, a)) {
        r |= l;
        continue;
      }
      const d = t.machineByTile.get(c);
      d && Ld(e, s, d) && (r |= l);
    }
    return r;
  }
  function Ld(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of wd(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  const h0 = 9079434;
  function d0(n, t, e) {
    const s = n.pipeStubLayer;
    s.removeChildren();
    const i = Math.max(2, (k - 1) * 0.4);
    for (const r of t.entities) {
      if (r.name !== "pipe") continue;
      const o = r.x ?? 0, a = r.y ?? 0;
      for (const [l, c] of [
        [
          0,
          -1
        ],
        [
          1,
          0
        ],
        [
          0,
          1
        ],
        [
          -1,
          0
        ]
      ]) {
        const h = `${o + l},${a + c}`, d = e.machineByTile.get(h);
        if (!d || !Ld(o, a, d)) continue;
        const u = o * k + k / 2, p = a * k + k / 2, f = k * 1.5, g = u + l * f, m = p + c * f, y = new ut();
        y.moveTo(u, p).lineTo(g, m).stroke({
          width: i,
          color: h0,
          cap: "round"
        }), s.addChild(y);
      }
    }
  }
  function aa(n, t) {
    if (un.has(n.name)) {
      const s = l0(n, t.tileMap), i = Wx(n.name, n.direction ?? "North", s);
      return on(i, mt, mt, (r) => {
        const o = mt / k;
        let a = null;
        s === "corner-cw" ? a = {
          turn: "cw"
        } : s === "corner-ccw" && (a = {
          turn: "ccw"
        });
        const l = ea(n, a);
        l.scale.set(o), r.addChild(l);
      });
    }
    if (be.has(n.name)) {
      const s = n.io_type ?? "input", i = zx(n.name, n.direction ?? "North", s);
      return on(i, mt, mt, (r) => {
        const o = mt / k, a = Mn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (ee.has(n.name)) {
      const s = Dx(n.name, n.direction ?? "North"), i = n.direction === "North" || n.direction === "South";
      return Dl(s, i ? 2 : 1, i ? 1 : 2, (a, l, c) => {
        const h = mt / k, d = Mn(n, t);
        d.scale.set(h), d.x = 0, d.y = 0, a.addChild(d);
      });
    }
    if (n.name === "pipe") {
      const s = c0(n, t), i = Gx(s);
      return on(i, mt, mt, (r) => {
        const o = mt / k, a = Mn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (n.name === "pipe-to-ground") {
      const s = Vx(n.direction ?? "North");
      return on(s, mt, mt, (i) => {
        const r = mt / k, o = Mn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (er.has(n.name)) {
      const s = Hx(n.name, n.direction ?? "North");
      return on(s, mt, mt, (i) => {
        const r = mt / k, o = Mn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Jo.has(n.name)) {
      const s = jx(n.name);
      return on(s, mt, mt, (i) => {
        const r = mt / k, o = Mn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Re.has(n.name)) {
      const [s, i] = tn[n.name] ?? [
        1,
        1
      ], r = Ux(n.name);
      return Dl(r, s, i, (o, a, l) => {
        const c = `/fucktorio/pr-238/entity-frames/${n.name}.png`, h = te.has(c) ? ze.get(c) ?? null : null, d = 1.8, u = mt / n0;
        if (h) {
          const p = new De(h);
          p.scale.set(u * d);
          const f = a, g = l;
          p.x = -f * (d - 1) / 2, p.y = -g * (d - 1) / 2, o.addChild(p);
        } else {
          const f = {
            "assembling-machine-1": 5926530,
            "assembling-machine-2": 4874872,
            "assembling-machine-3": 3822186,
            "chemical-plant": 3832400,
            "oil-refinery": 5913226,
            "electric-furnace": 6969984,
            "steel-furnace": 8015920,
            "stone-furnace": 9068608,
            centrifuge: 3832448,
            lab: 4876880,
            "rocket-silo": 4868714,
            foundry: 9071152,
            biochamber: 4880954,
            biolab: 3828314,
            "electromagnetic-plant": 2775706,
            "cryogenic-plant": 4881034,
            recycler: 6969930,
            crusher: 5917242,
            beacon: 4874368,
            "storage-tank": 4876890,
            "electric-mining-drill": 8022576
          }[n.name] ?? 4872810;
          o.roundRect(2, 2, a - 4, l - 4, 3).fill({
            color: f,
            alpha: 0.5
          });
        }
      });
    }
    const e = `generic:${n.name}`;
    return on(e, mt, mt, (s) => {
      const i = mt / k, r = Mn(n, t);
      r.scale.set(i), r.x = 0, r.y = 0, s.addChild(r);
    });
  }
  function So(n, t, e, s = nr()) {
    const i = is(t);
    if (se.has(i)) return;
    const r = t.x ?? 0, o = t.y ?? 0, a = aa(t, s);
    let l = k / mt, c = k / mt;
    if (ee.has(t.name)) {
      const f = t.direction === "North" || t.direction === "South", g = f ? 2 : 1, m = f ? 1 : 2;
      l = g * k / (g * mt), c = m * k / (m * mt);
    } else if (Re.has(t.name)) {
      const [f, g] = tn[t.name] ?? [
        1,
        1
      ];
      l = f * k / (f * mt), c = g * k / (g * mt);
    }
    const h = r * k, d = o * k, u = new Gi({
      texture: a,
      x: h,
      y: d,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: l,
      scaleY: c
    });
    Re.has(t.name) ? n.machineContainer.addParticle(u) : n.beltContainer.addParticle(u);
    let p;
    if (t.carries && !Re.has(t.name)) {
      const f = Cd(t.carries), g = k * 0.35, m = (k - g) / 2;
      p = new Gi({
        texture: f,
        x: h + m,
        y: d + m,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: g / mt,
        scaleY: g / mt
      }), n.iconContainer.addParticle(p);
    }
    se.set(i, {
      entity: u,
      icon: p,
      revealAt: e,
      placedEntity: t
    });
  }
  const gs = /* @__PURE__ */ new Map();
  function u0(n, t, e, s, i, r, o) {
    const a = `${t},${e}`, l = gs.get(a);
    if (l && l.specKey === o) return null;
    const c = {
      name: "transport-belt",
      x: t,
      y: e,
      direction: s,
      recipe: null,
      carries: i,
      segment_id: null,
      io_type: null
    }, h = nr();
    Di(c, h);
    const d = aa(c, h), u = Sn(i), p = new Gi({
      texture: d,
      x: t * k,
      y: e * k,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: k / mt,
      scaleY: k / mt,
      tint: u
    });
    return n.ghostContainer.addParticle(p), l || gs.set(a, {
      particle: p,
      specKey: o
    }), p;
  }
  function Zl(n, t, e) {
    const s = `${t},${e}`, i = gs.get(s);
    i && (n.ghostContainer.removeParticle(i.particle), gs.delete(s));
  }
  function p0(n) {
    n.ghostContainer.removeParticles(), gs.clear();
  }
  function f0(n, t, e) {
    const s = [];
    for (const [i, r] of se.entries()) r.placedEntity.x !== t || r.placedEntity.y !== e || (Re.has(r.placedEntity.name) ? n.machineContainer.removeParticle(r.entity) : n.beltContainer.removeParticle(r.entity), r.icon && n.iconContainer.removeParticle(r.icon), se.delete(i), s.push(i));
    return s;
  }
  function m0(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const [s, i] of se.entries()) {
      if (i.placedEntity.name !== "pipe") continue;
      const r = aa(i.placedEntity, t);
      if (i.entity.texture === r) continue;
      const o = i.entity, a = new Gi({
        texture: r,
        x: o.x,
        y: o.y,
        alpha: o.alpha,
        anchorX: o.anchorX,
        anchorY: o.anchorY,
        scaleX: o.scaleX,
        scaleY: o.scaleY
      });
      n.beltContainer.removeParticle(o), n.beltContainer.addParticle(a), se.set(s, {
        ...i,
        entity: a
      }), e.set(o, a);
    }
    return e;
  }
  function $d(n, t) {
    for (const e of se.values()) {
      const s = Math.min(1, Math.max(0, (t - e.revealAt) / Id));
      e.entity.alpha = s, e.icon && (e.icon.alpha = s);
    }
  }
  function* Ql(n) {
    for (const t of se.values()) yield {
      particle: t.entity,
      iconParticle: t.icon,
      revealAt: t.revealAt
    };
  }
  function Bd(n) {
    const t = /* @__PURE__ */ new Map();
    let e = false;
    function s() {
      e || (e = true, n.add(r), sr());
    }
    function i() {
      e && (e = false, n.remove(r), Dn());
    }
    function r() {
      const c = performance.now();
      let h = false;
      for (const [d, u] of t) {
        const p = c - u.startTime, f = Math.min(1, p / u.duration), g = u.ease(f), m = u.startAlpha + (u.targetAlpha - u.startAlpha) * g;
        u.entityParticle.alpha = m, u.iconParticle && (u.iconParticle.alpha = m), f >= 1 ? t.delete(d) : h = true;
      }
      h || i(), Ze();
    }
    function o(c, h, d, u) {
      const p = t.get(c), f = performance.now();
      let g;
      if (p) {
        const y = f - p.startTime, b = Math.min(1, y / p.duration);
        g = p.startAlpha + (p.targetAlpha - p.startAlpha) * p.ease(b);
      } else g = h.alpha;
      if (Math.abs(g - u) < 1e-3) {
        t.delete(c), h.alpha = u, d && (d.alpha = u);
        return;
      }
      const m = u > g;
      t.set(c, {
        entityParticle: h,
        iconParticle: d,
        startAlpha: g,
        targetAlpha: u,
        startTime: f,
        duration: m ? i0 : r0,
        ease: m ? o0 : a0
      }), s();
    }
    function a(c) {
      t.clear();
      for (const h of se.values()) h.entity.alpha = c, h.icon && (h.icon.alpha = c);
      i(), Ze();
    }
    function l() {
      t.clear(), i();
    }
    return {
      animateTo: o,
      cancelAll: a,
      destroy: l
    };
  }
  function Od(n) {
    let t = 0;
    for (const i of n.values()) i > t && (t = i);
    const e = Math.max(t, s0), s = /* @__PURE__ */ new Map();
    for (const [i, r] of n) {
      const o = Hi + (1 - Hi) * (1 - r / e);
      s.set(i, o);
    }
    return s;
  }
  function Cn(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function g0(n, t, e) {
    t.clear(), t.layout = n;
    const s = nr();
    for (const l of n.entities) Di(l, s);
    const i = 0;
    for (const l of n.entities) So(t, l, i, s);
    d0(t, n, s), $d(t, Id + 1);
    const r = Md(n);
    if (!e) return y0();
    const o = Bd(e.ticker);
    function a(l) {
      const c = Od(l);
      for (const h of se.values()) {
        const d = Cn(h.placedEntity), u = c.get(d) ?? Hi;
        o.animateTo(d, h.entity, h.icon, u);
      }
    }
    return {
      highlightItem(l) {
        if (o.cancelAll(1), !!l) for (const c of se.values()) {
          const h = c.placedEntity, u = (h.carries ?? h.recipe ?? null) === l ? 1 : 0.15, p = Cn(h);
          o.animateTo(p, c.entity, c.icon, u);
        }
      },
      highlightBeltNetwork(l) {
        if (!l) {
          o.cancelAll(1);
          return;
        }
        const c = Cn(l), h = Pd(r, c);
        a(h);
      },
      clearHighlight() {
        for (const l of se.values()) {
          const c = Cn(l.placedEntity);
          o.animateTo(c, l.entity, l.icon, 1);
        }
      },
      chainKey(l) {
        return l.carries ?? l.recipe ?? null;
      }
    };
  }
  function y0() {
    function n() {
      for (const t of se.values()) t.entity.alpha = 1, t.icon && (t.icon.alpha = 1);
    }
    return {
      highlightItem(t) {
        if (n(), !!t) for (const e of se.values()) {
          const s = e.placedEntity, r = (s.carries ?? s.recipe ?? null) === t ? 1 : 0.15;
          e.entity.alpha = r, e.icon && (e.icon.alpha = r);
        }
      },
      highlightBeltNetwork() {
      },
      clearHighlight() {
        n();
      },
      chainKey(t) {
        return t.carries ?? t.recipe ?? null;
      }
    };
  }
  function x0(n, t) {
    const e = Md(n), s = Bd(t.ticker);
    function i(r) {
      const o = Cn(r), a = Pd(e, o), l = Od(a);
      for (const c of se.values()) {
        const h = Cn(c.placedEntity), d = l.get(h) ?? Hi;
        s.animateTo(h, c.entity, c.icon, d);
      }
    }
    return {
      highlightItem(r) {
        if (s.cancelAll(1), !!r) for (const o of se.values()) {
          const a = o.placedEntity, c = (a.carries ?? a.recipe ?? null) === r ? 1 : 0.15;
          o.entity.alpha = c, o.icon && (o.icon.alpha = c);
        }
      },
      highlightBeltNetwork(r) {
        if (!r) {
          for (const o of se.values()) {
            const a = Cn(o.placedEntity);
            s.animateTo(a, o.entity, o.icon, 1);
          }
          return;
        }
        i(r);
      },
      clearHighlight() {
        for (const r of se.values()) {
          const o = Cn(r.placedEntity);
          s.animateTo(o, r.entity, r.icon, 1);
        }
      },
      chainKey(r) {
        return r.carries ?? r.recipe ?? null;
      },
      destroy() {
        s.destroy();
      }
    };
  }
  const tc = 6, b0 = 18, _0 = 2, w0 = 26, v0 = 0, C0 = 0.6;
  function S0(n) {
    return n.carries ? un.has(n.name) || be.has(n.name) || Xs.has(n.name) : false;
  }
  function k0(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const l of t.external_inputs) e.set(l.item, {
      rate: l.rate,
      isFluid: !!l.is_fluid
    });
    if (e.size === 0) return [];
    const s = /* @__PURE__ */ new Map();
    for (const l of n.entities) {
      if (!S0(l)) continue;
      const c = l.carries;
      if (!e.has(c)) continue;
      const h = l.x ?? 0, d = l.y ?? 0, u = s.get(h);
      (!u || d < u.y) && s.set(h, {
        y: d,
        carries: c
      });
    }
    if (s.size === 0) return [];
    const i = Array.from(s.entries()).filter(([, l]) => l.y === v0).map(([l, c]) => ({
      x: l,
      ...c
    })).sort((l, c) => l.x - c.x), r = [];
    let o = null;
    for (const l of i) {
      const c = e.get(l.carries);
      o && o.item === l.carries && l.x === o.xMax + 1 ? (o.xMax = l.x, o.topY = Math.min(o.topY, l.y)) : (o && r.push(o), o = {
        item: l.carries,
        isFluid: c.isFluid,
        xMin: l.x,
        xMax: l.x,
        topY: l.y,
        rate: c.rate
      });
    }
    o && r.push(o);
    const a = /* @__PURE__ */ new Map();
    for (const l of r) a.set(l.item, (a.get(l.item) ?? 0) + 1);
    for (const l of r) {
      const c = a.get(l.item) ?? 1;
      c > 1 && (l.rate = l.rate / c);
    }
    return r;
  }
  function T0(n) {
    return `${n.toFixed(1)}/s`;
  }
  function A0(n) {
    const t = n.xMax - n.xMin + 1, e = Math.min(w0, b0 + (t - 1) * _0), s = new zt();
    s.eventMode = "none";
    const i = Cd(n.item), r = new De(i);
    r.width = e, r.height = e, r.x = 0, r.y = -e / 2, s.addChild(r);
    const o = new Le({
      fontFamily: "'JetBrains Mono','Consolas',monospace",
      fontSize: e,
      fontWeight: "bold",
      fill: 16777215,
      dropShadow: {
        color: 0,
        distance: 1,
        blur: 3,
        alpha: 0.85
      }
    }), a = new Ge({
      text: T0(n.rate),
      style: o
    });
    a.x = e + tc, a.y = -a.height / 2, s.addChild(a);
    const l = new Le({
      fontFamily: "'JetBrains Mono','Consolas',monospace",
      fontSize: e,
      fontWeight: "bold",
      fill: 16777215,
      dropShadow: {
        color: 0,
        distance: 1,
        blur: 3,
        alpha: 0.85
      }
    }), c = new Ge({
      text: ue(n.item),
      style: l
    });
    return c.alpha = C0, c.x = a.x + a.width + tc, c.y = -c.height / 2, s.addChild(c), s;
  }
  function E0(n, t, e) {
    if (n.removeChildren(), !e) return;
    const s = k0(t, e);
    if (s.length !== 0) for (const i of s) {
      const r = A0(i);
      r.rotation = -Math.PI / 2;
      const o = (i.xMin + i.xMax + 1) / 2 * k, a = i.topY * k - k * 0.5;
      r.x = o, r.y = a, n.addChild(r);
    }
  }
  function M0(n) {
    return ee.has(n.name) ? n.direction === "East" || n.direction === "West" ? [
      1,
      2
    ] : [
      2,
      1
    ] : tn[n.name] ?? [
      1,
      1
    ];
  }
  const Fr = 57504;
  function ec(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map();
    for (const v of s.entities) r.set(`${v.x ?? 0},${v.y ?? 0}`, v);
    let o = null, a = false, l = [];
    const c = new ut();
    e.addChild(c);
    const h = new ut();
    e.addChild(h);
    function d(v, w) {
      const C = n.getBoundingClientRect();
      return t.toWorld(v - C.left, w - C.top);
    }
    function u(v, w) {
      if (!o) return;
      const C = d(o.sx, o.sy), E = d(v, w), P = Math.min(C.x, E.x), T = Math.min(C.y, E.y), A = Math.abs(E.x - C.x), H = Math.abs(E.y - C.y);
      c.clear(), c.rect(P, T, A, H).fill({
        color: Fr,
        alpha: 0.18
      }), c.setStrokeStyle({
        width: 1,
        color: Fr,
        alpha: 0.8
      }), c.rect(P, T, A, H).stroke(), Ze();
    }
    function p(v) {
      if (h.clear(), v.length !== 0) {
        h.setStrokeStyle({
          width: 1.5,
          color: Fr,
          alpha: 0.9
        });
        for (const w of v) {
          const [C, E] = M0(w), P = (w.x ?? 0) * k + 1, T = (w.y ?? 0) * k + 1;
          h.rect(P, T, C * k - 2, E * k - 2).stroke();
        }
      }
    }
    function f(v, w) {
      if (!o) return [];
      const C = d(o.sx, o.sy), E = d(v, w), P = Math.min(Math.floor(C.x / k), Math.floor(E.x / k)), T = Math.max(Math.floor(C.x / k), Math.floor(E.x / k)), A = Math.min(Math.floor(C.y / k), Math.floor(E.y / k)), H = Math.max(Math.floor(C.y / k), Math.floor(E.y / k)), j = [];
      for (let D = P; D <= T; D++) for (let N = A; N <= H; N++) {
        const $ = r.get(`${D},${N}`);
        $ && j.push($);
      }
      return j;
    }
    const g = (v) => {
      v.button !== 0 || !v.shiftKey || (o = {
        sx: v.clientX,
        sy: v.clientY
      }, a = false);
    }, m = (v) => {
      if (!o) return;
      const w = v.clientX - o.sx, C = v.clientY - o.sy;
      !a && w * w + C * C > 36 && (a = true), a && u(v.clientX, v.clientY);
    }, y = (v) => {
      if (v.button === 0) {
        if (a) v.stopImmediatePropagation(), c.clear(), l = f(v.clientX, v.clientY), p(l), i(l), Ze();
        else if (o !== null) {
          const w = d(v.clientX, v.clientY), C = Math.floor(w.x / k), E = Math.floor(w.y / k);
          r.has(`${C},${E}`) && (l = [], h.clear(), i([]), Ze());
        }
        o = null, a = false;
      }
    };
    function b() {
      l = [], c.clear(), h.clear(), i([]), Ze();
    }
    const x = (v) => {
      v.preventDefault(), l.length > 0 && b();
    }, _ = (v) => {
      v.key === "Escape" && l.length > 0 && b();
    };
    return n.addEventListener("pointerdown", g, {
      capture: true
    }), n.addEventListener("pointermove", m, {
      capture: true
    }), n.addEventListener("pointerup", y, {
      capture: true
    }), n.addEventListener("contextmenu", x), window.addEventListener("keydown", _), {
      destroy() {
        n.removeEventListener("pointerdown", g, {
          capture: true
        }), n.removeEventListener("pointermove", m, {
          capture: true
        }), n.removeEventListener("pointerup", y, {
          capture: true
        }), n.removeEventListener("contextmenu", x), window.removeEventListener("keydown", _), c.destroy(), h.destroy();
      },
      clear: b,
      getSelected() {
        return [
          ...l
        ];
      },
      buildJson(v, w) {
        return JSON.stringify({
          params: v,
          selected: l.map((C) => ({
            x: C.x ?? 0,
            y: C.y ?? 0,
            name: C.name,
            direction: C.direction,
            carries: C.carries,
            recipe: C.recipe,
            rate: C.rate,
            io_type: C.io_type
          })),
          note: w
        }, null, 2);
      }
    };
  }
  const P0 = [
    "partitioned-per-consumer",
    "partitioned-decomposed"
  ], I0 = [
    "horizontal-stack"
  ], Nr = [
    "iron-plate",
    "copper-plate",
    "steel-plate",
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], ko = [
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], Fd = "iron-gear-wheel", Nd = 10, R0 = "assembling-machine-3";
  function L0() {
    const n = new URLSearchParams(window.location.search), t = n.get("item") ?? Fd, e = parseFloat(n.get("rate") ?? ""), s = isNaN(e) || e <= 0 ? Nd : e, i = n.get("machine"), r = n.get("in"), o = r ? r.split(",").filter((f) => f.length > 0) : ko, a = n.get("belt"), l = n.get("strategy"), c = l && P0.includes(l) ? l : null, h = n.get("row_layout"), d = h && I0.includes(h) ? h : null, u = n.get("ci"), p = u ? u.split(",").filter((f) => f.length > 0) : [];
    return {
      item: t,
      rate: s,
      machine: i,
      inputs: o,
      belt: a,
      strategy: c,
      rowLayout: d,
      customInputs: p
    };
  }
  function $0(n) {
    if (n.item === Fd && n.rate === Nd && n.machine === R0 && n.inputs.length === ko.length && n.inputs.every((s, i) => s === ko[i]) && !n.belt && !n.strategy && !n.rowLayout && n.customInputs.length === 0) {
      history.replaceState(null, "", window.location.pathname);
      return;
    }
    const e = new URLSearchParams();
    e.set("item", n.item), e.set("rate", String(n.rate)), e.set("machine", n.machine), e.set("in", n.inputs.join(",")), n.belt && e.set("belt", n.belt), n.strategy && e.set("strategy", n.strategy), n.rowLayout && e.set("row_layout", n.rowLayout), n.customInputs.length > 0 && e.set("ci", n.customInputs.join(",")), history.replaceState(null, "", "?" + e.toString());
  }
  function cn(n, t = 14) {
    const e = document.createElement("img");
    return e.src = `/fucktorio/pr-238/icons/${n}.png`, e.width = t, e.height = t, e.style.cssText = "image-rendering:pixelated", e.onerror = () => {
      e.style.display = "none";
    }, e;
  }
  function B0(n, t) {
    const e = document.createElement("option");
    return e.value = n, e.textContent = ue(n), n === t && (e.selected = true), e;
  }
  function Si(n, t, e) {
    const s = document.createElement("div");
    s.className = "sb-section";
    const i = document.createElement("div");
    i.className = "sb-section-header";
    const r = document.createElement("span");
    r.className = "sb-section-icon", r.innerHTML = n, i.appendChild(r);
    const o = document.createElement("span");
    o.textContent = t, i.appendChild(o);
    let a = null;
    e !== void 0 && (a = document.createElement("span"), a.className = "sb-section-count", a.textContent = e, i.appendChild(a)), s.appendChild(i);
    const l = document.createElement("div");
    return s.appendChild(l), {
      section: s,
      body: l,
      countEl: a
    };
  }
  function nc(n, t, e, s) {
    for (const i of t) {
      const r = document.createElement("div");
      r.className = `sb-machine-flow ${e}`, s && r.appendChild(document.createTextNode(s)), r.appendChild(cn(i.item, 13)), r.appendChild(document.createTextNode(ue(i.item)));
      const o = document.createElement("span");
      o.className = "flow-rate";
      const a = Ed(i.rate), l = a ? Ad(a.color) : "#f88";
      o.style.color = l, o.textContent = `${i.rate.toFixed(1)}/s`, r.appendChild(o), n.appendChild(r);
    }
  }
  const sc = /* @__PURE__ */ new Set([
    "water",
    "crude-oil",
    "petroleum-gas",
    "light-oil",
    "heavy-oil",
    "sulfuric-acid",
    "lubricant",
    "steam"
  ]), Wd = "fucktorio-recent-items", ic = 5;
  function Gd() {
    try {
      const n = localStorage.getItem(Wd);
      return n ? JSON.parse(n) : [];
    } catch {
      return [];
    }
  }
  function O0(n) {
    const t = Gd().filter((e) => e !== n);
    t.unshift(n), t.length > ic && (t.length = ic);
    try {
      localStorage.setItem(Wd, JSON.stringify(t));
    } catch {
    }
  }
  function F0(n, t, e) {
    let s = t, i = false, r = null;
    const o = document.createElement("div");
    o.className = "sb-item-picker";
    const a = document.createElement("div");
    a.className = "sb-picker-value";
    const l = document.createElement("span");
    l.className = "sb-picker-arrow", l.textContent = "\u25BE", o.appendChild(a), o.appendChild(l);
    const c = document.createElement("div");
    c.className = "sb-picker-dropdown", c.style.display = "none", o.appendChild(c);
    const h = document.createElement("input");
    h.type = "text", h.className = "sb-picker-search", h.placeholder = "Search items\u2026", c.appendChild(h);
    const d = document.createElement("div");
    d.className = "sb-picker-list", c.appendChild(d);
    function u() {
      if (a.innerHTML = "", s) {
        a.appendChild(cn(s, 14));
        const x = document.createElement("span");
        x.textContent = ue(s), a.appendChild(x);
      } else {
        const x = document.createElement("span");
        x.className = "sb-picker-placeholder", x.textContent = "Select item\u2026", a.appendChild(x);
      }
    }
    function p(x) {
      const _ = document.createElement("div");
      _.className = "sb-picker-item" + (x === s ? " selected" : ""), _.dataset.slug = x, _.appendChild(cn(x, 14));
      const v = document.createElement("span");
      return v.textContent = ue(x), _.appendChild(v), _.addEventListener("mousedown", (w) => {
        w.preventDefault(), g(x);
      }), _;
    }
    function f(x) {
      d.innerHTML = "", r = null;
      const _ = x.trim().toLowerCase(), v = _ ? n.filter((w) => w.includes(_) || ue(w).toLowerCase().includes(_)) : n;
      if (!_) {
        const w = Gd().filter((C) => n.includes(C));
        if (w.length > 0) {
          const C = document.createElement("div");
          C.className = "sb-picker-section-label", C.textContent = "Recent", d.appendChild(C);
          for (const P of w) d.appendChild(p(P));
          const E = document.createElement("div");
          E.className = "sb-picker-divider", d.appendChild(E);
        }
      }
      for (const w of v) d.appendChild(p(w));
      if (!_ && s) {
        const w = d.querySelector(`[data-slug="${s}"]`);
        w && w.scrollIntoView({
          block: "nearest"
        });
      }
    }
    function g(x) {
      s = x, O0(x), o.classList.remove("item-invalid"), u(), y(), e(x);
    }
    function m() {
      i = true, o.classList.add("open"), c.style.display = "", l.textContent = "\u25B4", h.value = "", f(""), requestAnimationFrame(() => h.focus());
    }
    function y() {
      i = false, o.classList.remove("open"), c.style.display = "none", l.textContent = "\u25BE", r = null;
    }
    function b(x) {
      const _ = d.querySelectorAll(".sb-picker-item");
      if (!_.length) return;
      const v = Array.from(_);
      let w = r ? v.indexOf(r) : -1;
      w = Math.max(0, Math.min(v.length - 1, w + x)), r == null ? void 0 : r.classList.remove("highlighted"), r = v[w], r.classList.add("highlighted"), r.scrollIntoView({
        block: "nearest"
      });
    }
    return o.addEventListener("mousedown", (x) => {
      c.contains(x.target) || (x.preventDefault(), i ? y() : m());
    }), h.addEventListener("input", () => f(h.value)), h.addEventListener("keydown", (x) => {
      x.key === "ArrowDown" ? (x.preventDefault(), b(1)) : x.key === "ArrowUp" ? (x.preventDefault(), b(-1)) : x.key === "Enter" ? (r == null ? void 0 : r.dataset.slug) && g(r.dataset.slug) : x.key === "Escape" && y();
    }), document.addEventListener("mousedown", (x) => {
      i && !o.contains(x.target) && y();
    }), u(), {
      el: o,
      getValue: () => s,
      setValue(x) {
        s = x, u();
      },
      setInvalid(x) {
        o.classList.toggle("item-invalid", x);
      }
    };
  }
  function N0(n, t, e) {
    n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "sidebar-inner";
    const { section: i, body: r } = Si('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="2"/></svg>', "Target"), o = t.allProducibleItems(), a = new Set(o);
    function l(I, G) {
      const V = document.createElement("div");
      V.className = "sb-field";
      const J = document.createElement("span");
      return J.className = "sb-field-label", J.textContent = I, V.appendChild(J), G.style.flex = "1", G.style.minWidth = "0", V.appendChild(G), V;
    }
    const c = F0(o, "", () => ht());
    c.el.style.cssText = "margin-bottom:6px", r.appendChild(c.el);
    const h = document.createElement("select");
    h.className = "sb-select", [
      "assembling-machine-1",
      "assembling-machine-2",
      "assembling-machine-3"
    ].forEach((I) => h.appendChild(B0(I, "assembling-machine-3"))), r.appendChild(l("Assembler", h));
    const d = document.createElement("select");
    d.className = "sb-select", [
      [
        "Auto",
        ""
      ],
      [
        "Yellow 15/s",
        "transport-belt"
      ],
      [
        "Red 30/s",
        "fast-transport-belt"
      ],
      [
        "Blue 45/s",
        "express-transport-belt"
      ]
    ].forEach(([I, G]) => {
      const V = document.createElement("option");
      V.value = G, V.textContent = I, d.appendChild(V);
    }), r.appendChild(l("Belt", d));
    const u = document.createElement("select");
    u.className = "sb-select", [
      [
        "Pooled (today)",
        "",
        false
      ],
      [
        "Partitioned per consumer",
        "partitioned-per-consumer",
        true
      ],
      [
        "Partitioned + decomposed",
        "partitioned-decomposed",
        true
      ]
    ].forEach(([I, G, V]) => {
      const J = document.createElement("option");
      J.value = G, J.textContent = I, V && (J.disabled = true, J.title = "Wired in Phase 1 of rfp-modular-production"), u.appendChild(J);
    }), r.appendChild(l("Strategy", u));
    const p = document.createElement("select");
    p.className = "sb-select", [
      [
        "Vertical split (today)",
        ""
      ],
      [
        "Horizontal stack (RFP)",
        "horizontal-stack"
      ]
    ].forEach(([I, G]) => {
      const V = document.createElement("option");
      V.value = G, V.textContent = I, p.appendChild(V);
    }), r.appendChild(l("Row layout", p));
    const f = document.createElement("div");
    f.className = "sb-field";
    const g = document.createElement("span");
    g.className = "sb-field-label", g.textContent = "Rate", f.appendChild(g);
    const m = document.createElement("input");
    m.type = "number", m.className = "sb-input", m.step = "0.5", m.min = "0.1", m.style.cssText = "flex:1;min-width:0", m.placeholder = "10", f.appendChild(m);
    const y = document.createElement("span");
    y.className = "sb-rate-suffix", y.textContent = "/s", f.appendChild(y), r.appendChild(f), s.appendChild(i);
    const { section: b, body: x } = Si('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="6" rx="1"/><line x1="5" y1="8" x2="11" y2="8"/></svg>', "Inputs"), _ = document.createElement("div");
    _.className = "sb-tags";
    const v = /* @__PURE__ */ new Map();
    Nr.forEach((I) => {
      const G = document.createElement("label");
      G.className = `sb-tag${sc.has(I) ? " fluid" : ""}`;
      const V = document.createElement("span");
      V.className = "sb-tag-check", V.textContent = "\u2713";
      const J = document.createElement("input");
      J.type = "checkbox", J.value = I, J.style.display = "none", v.set(I, J), G.appendChild(V), G.appendChild(cn(I, 14)), G.appendChild(document.createTextNode(ue(I))), G.appendChild(J), J.addEventListener("change", () => {
        G.classList.toggle("active", J.checked);
      }), _.appendChild(G);
    }), x.appendChild(_);
    let w = [];
    const C = document.createElement("div");
    C.className = "sb-tags sb-custom-tags", x.appendChild(C);
    const E = document.createElement("datalist");
    E.id = "fucktorio-custom-inputs-datalist";
    const P = new Set(Nr);
    o.filter((I) => !P.has(I)).forEach((I) => {
      const G = document.createElement("option");
      G.value = I, E.appendChild(G);
    }), x.appendChild(E);
    const T = document.createElement("input");
    T.type = "text", T.className = "sb-input sb-custom-input-field", T.setAttribute("list", "fucktorio-custom-inputs-datalist"), T.autocomplete = "off", T.placeholder = "+ add input\u2026", x.appendChild(T);
    function A(I) {
      const G = document.createElement("div");
      G.className = `sb-tag sb-custom-tag active${sc.has(I) ? " fluid" : ""}`, G.dataset.item = I, G.appendChild(cn(I, 14)), G.appendChild(document.createTextNode(ue(I)));
      const V = document.createElement("span");
      V.className = "sb-tag-remove", V.textContent = "\xD7", V.addEventListener("click", (J) => {
        J.stopPropagation(), w = w.filter((xt) => xt !== I), G.remove(), ht();
      }), G.appendChild(V), C.appendChild(G);
    }
    function H(I) {
      const G = I.trim();
      !G || !a.has(G) || P.has(G) || w.includes(G) || (w.push(G), A(G), T.value = "", ht());
    }
    T.addEventListener("keydown", (I) => {
      I.key === "Enter" && H(T.value);
    }), T.addEventListener("change", () => {
      H(T.value);
    }), s.appendChild(b);
    const { section: j, body: D, countEl: N } = Si('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>', "Solver", ""), $ = document.createElement("div");
    D.appendChild($), s.appendChild(j);
    const W = document.createElement("div");
    W.className = "sb-actions", W.style.display = "none";
    const X = document.createElement("button");
    X.className = "sb-btn sb-btn-secondary", X.textContent = "Copy Blueprint", X.style.flex = "1", W.appendChild(X);
    const U = document.createElement("div");
    U.className = "sb-copy-status", W.appendChild(U), D.appendChild(W);
    const { section: Z, body: R, countEl: B } = Si('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.8" fill="currentColor" stroke="none"/></svg>', "Validation", "");
    Z.style.display = "none", s.appendChild(Z), n.appendChild(s);
    const O = L0();
    c.setValue(O.item), m.value = String(O.rate);
    const z = /* @__PURE__ */ new Set([
      "assembling-machine-1",
      "assembling-machine-2",
      "assembling-machine-3"
    ]);
    h.value = O.machine && z.has(O.machine) ? O.machine : "assembling-machine-3", v.forEach((I, G) => {
      I.checked = O.inputs.includes(G);
      const V = I.closest(".sb-tag");
      V && V.classList.toggle("active", I.checked);
    }), O.belt && (d.value = O.belt), O.strategy && (u.value = O.strategy), O.rowLayout && (p.value = O.rowLayout);
    for (const I of O.customInputs) a.has(I) && !P.has(I) && !w.includes(I) && (w.push(I), A(I));
    let Y = null, K = O.item, nt = null, ot = 0;
    function ht() {
      Y !== null && clearTimeout(Y), Y = setTimeout(() => {
        gt().catch((I) => console.error("runSolve failed:", I));
      }, 150);
    }
    async function gt() {
      var _a2;
      const I = c.getValue(), G = parseFloat(m.value), V = h.value, J = Nr.filter((ct) => {
        var _a3;
        return (_a3 = v.get(ct)) == null ? void 0 : _a3.checked;
      }), xt = [
        ...J,
        ...w
      ];
      if (!a.has(I)) {
        c.setInvalid(true);
        return;
      }
      if (c.setInvalid(false), isNaN(G) || G <= 0) return;
      if (I !== K) {
        const ct = t.defaultMachineForItem(I, V);
        z.has(ct) && (h.value = ct), K = I;
      }
      $0({
        item: I,
        rate: G,
        machine: h.value,
        inputs: J,
        belt: d.value || null,
        strategy: u.value || null,
        rowLayout: p.value || null,
        customInputs: w
      });
      const bt = ++ot;
      $.innerHTML = "", nt = null, W.style.display = "none";
      let wt;
      try {
        wt = await t.solve(I, G, xt, h.value);
      } catch (ct) {
        if (bt !== ot) return;
        e.renderGraph(null), N && (N.textContent = "error");
        const vt = document.createElement("div");
        vt.className = "sb-result-error", vt.textContent = String(ct), $.appendChild(vt);
        return;
      }
      if (bt !== ot) return;
      W0($, wt), e.renderGraph(wt);
      const Dt = wt.machines.reduce((ct, vt) => ct + Math.ceil(vt.count), 0);
      N && (N.textContent = `${Dt} machines`);
      const kt = /* @__PURE__ */ new Set();
      for (const ct of wt.machines) {
        for (const vt of ct.inputs) kt.add(vt.item);
        for (const vt of ct.outputs) kt.add(vt.item);
      }
      for (const ct of wt.external_inputs) kt.add(ct.item);
      for (const ct of wt.external_outputs) kt.add(ct.item);
      if (await _d(Array.from(kt)), bt !== ot) return;
      let Ft;
      try {
        const ct = d.value || void 0, vt = u.value || void 0, Jt = p.value || void 0, ce = e.startStreaming();
        Ft = await t.buildLayoutStreaming(wt, ct, vt, Jt, ce);
      } catch (ct) {
        if (bt !== ot) return;
        const vt = document.createElement("div");
        vt.className = "sb-result-error", vt.textContent = `Layout error: ${ct}`, $.appendChild(vt);
        return;
      }
      bt === ot && (nt = Ft, ld(wt.machines), e.renderLayout(Ft, wt), W.style.display = ((_a2 = Ft.warnings) == null ? void 0 : _a2.length) ? "none" : "flex");
    }
    return X.addEventListener("click", async () => {
      if (!nt) return;
      const I = await t.exportBlueprint(nt, c.getValue());
      await navigator.clipboard.writeText(I), U.textContent = "Copied!", setTimeout(() => {
        U.textContent = "";
      }, 2e3);
    }), m.addEventListener("input", ht), h.addEventListener("change", ht), d.addEventListener("change", ht), u.addEventListener("change", ht), p.addEventListener("change", ht), v.forEach((I) => I.addEventListener("change", ht)), gt().catch((I) => console.error("runSolve failed:", I)), {
      getParams() {
        const I = c.getValue(), G = parseFloat(m.value);
        return !I || isNaN(G) || G <= 0 ? null : {
          item: I,
          rate: G
        };
      },
      setParams(I, G) {
        c.setValue(I.item), m.value = String(I.rate), I.machine && z.has(I.machine) ? h.value = I.machine : h.value = "assembling-machine-3", I.inputs && v.forEach((V, J) => {
          V.checked = I.inputs.includes(J);
          const xt = V.closest(".sb-tag");
          xt && xt.classList.toggle("active", V.checked);
        }), I.belt ? d.value = I.belt : d.value = "", C.innerHTML = "", w = [];
        for (const V of I.customInputs ?? []) a.has(V) && !P.has(V) && !w.includes(V) && (w.push(V), A(V));
        K = I.item, (G == null ? void 0 : G.skipAutoSolve) || ht();
      },
      updateValidation(I, G) {
        if (R.innerHTML = "", I.length === 0) {
          Z.style.display = "none", B && (B.textContent = "");
          return;
        }
        Z.style.display = "";
        const V = I.filter((bt) => bt.severity === "Error").length, J = I.length - V;
        B && (V > 0 ? (B.textContent = `${V} error${V !== 1 ? "s" : ""}`, B.style.color = "#f66") : (B.textContent = `${J} warning${J !== 1 ? "s" : ""}`, B.style.color = "#fa0"));
        const xt = /* @__PURE__ */ new Map();
        for (const bt of I) {
          let wt = xt.get(bt.category);
          wt || (wt = [], xt.set(bt.category, wt)), wt.push(bt);
        }
        for (const [bt, wt] of xt) {
          const kt = wt.some((Rt) => Rt.severity === "Error") ? "#f44" : "#fa0", Ft = wt.find((Rt) => Rt.x != null && Rt.y != null), ct = document.createElement("div");
          ct.className = "sb-val-group";
          const vt = document.createElement("div");
          vt.className = "sb-val-group-header";
          const Jt = document.createElement("span");
          Jt.className = "sb-val-group-chevron", Jt.textContent = "\u25BE", Jt.addEventListener("click", (Rt) => {
            Rt.stopPropagation();
            const Zt = pe.style.display === "none";
            pe.style.display = Zt ? "" : "none", Jt.textContent = Zt ? "\u25BE" : "\u25B8";
          }), vt.appendChild(Jt);
          const ce = document.createElement("span");
          ce.className = "sb-val-group-dot", ce.style.background = kt, vt.appendChild(ce);
          const Te = document.createElement("span");
          Te.className = "sb-val-group-name", Te.textContent = bt, vt.appendChild(Te);
          const dt = document.createElement("span");
          dt.className = "sb-val-group-count", dt.textContent = String(wt.length), vt.appendChild(dt);
          const pe = document.createElement("div");
          pe.className = "sb-val-group-body", Ft && (vt.classList.add("clickable"), vt.addEventListener("click", () => {
            G(Ft.x, Ft.y);
          }));
          for (const Rt of wt) {
            const Zt = document.createElement("div"), Ht = Rt.x != null && Rt.y != null;
            Zt.className = "sb-val-issue" + (Ht ? " clickable" : "");
            const Qt = document.createElement("span");
            if (Qt.className = "sb-val-issue-msg", Qt.textContent = Rt.message, Zt.appendChild(Qt), Ht) {
              const M = document.createElement("span");
              M.className = "sb-val-issue-coord", M.textContent = `${Rt.x}, ${Rt.y}`, Zt.appendChild(M), Zt.addEventListener("click", (S) => {
                S.stopPropagation();
                const L = Zt.classList.contains("pinned");
                R.querySelectorAll(".sb-val-issue.pinned").forEach((q) => q.classList.remove("pinned")), L || Zt.classList.add("pinned"), G(Rt.x, Rt.y);
              });
            } else Zt.style.opacity = "0.6";
            pe.appendChild(Zt);
          }
          ct.appendChild(vt), ct.appendChild(pe), R.appendChild(ct);
        }
      }
    };
  }
  function W0(n, t) {
    if (t.external_inputs.length > 0) {
      const l = document.createElement("div");
      l.className = "sb-ext-section-title", l.textContent = "External inputs", n.appendChild(l);
      for (const h of t.external_inputs) {
        const d = document.createElement("div");
        d.className = "sb-ext-flow", d.appendChild(cn(h.item, 14)), d.appendChild(document.createTextNode(ue(h.item)));
        const u = document.createElement("span");
        u.className = "sb-ext-rate", u.textContent = `${h.rate.toFixed(1)}/s`, d.appendChild(u), n.appendChild(d);
      }
      const c = document.createElement("div");
      c.className = "sb-divider", n.appendChild(c);
    }
    const e = /* @__PURE__ */ new Map();
    for (const l of t.machines) {
      let c = e.get(l.entity);
      c || (c = [], e.set(l.entity, c)), c.push(l);
    }
    for (const [l, c] of e) {
      const h = c.reduce((m, y) => m + Math.ceil(y.count), 0), d = document.createElement("div");
      d.className = "sb-machine-group";
      const u = document.createElement("div");
      u.className = "sb-machine-group-header", u.appendChild(cn(l, 16));
      const p = document.createElement("span");
      p.className = "sb-machine-group-name", p.textContent = ue(l), u.appendChild(p);
      const f = document.createElement("span");
      f.className = "sb-machine-group-count", f.textContent = `\xD7${h}`, u.appendChild(f), d.appendChild(u);
      const g = document.createElement("div");
      g.className = "sb-machine-group-body";
      for (const m of c) {
        const y = document.createElement("div");
        y.className = "sb-machine-flow", y.style.cssText = "color:#6b7280;margin-bottom:2px", y.appendChild(document.createTextNode("\u2192 ")), y.appendChild(cn(m.recipe, 13)), y.appendChild(document.createTextNode(ue(m.recipe))), g.appendChild(y), nc(g, m.inputs, "flow-in", "\u25B6 "), nc(g, m.outputs, "flow-out", "\u25C0 ");
      }
      d.appendChild(g), n.appendChild(d);
    }
    const s = document.createElement("div");
    s.className = "sb-status", s.style.cssText = "margin-top:6px";
    const i = t.machines.reduce((l, c) => l + Math.ceil(c.count), 0), r = t.dependency_order.length, o = document.createElement("span");
    o.textContent = `${i} machines`, s.appendChild(o);
    const a = document.createElement("span");
    if (a.textContent = `depth ${r}`, s.appendChild(a), n.appendChild(s), t.external_outputs.length > 0) for (const l of t.external_outputs) {
      const c = document.createElement("div");
      c.style.cssText = "display:flex;align-items:center;gap:4px;padding:2px 0;font-size:11px;color:#b5cea8", c.appendChild(cn(l.item, 13)), c.appendChild(document.createTextNode(ue(l.item)));
      const h = Ed(l.rate);
      if (h) {
        const d = Ad(h.color);
        c.appendChild(document.createTextNode(`${l.rate.toFixed(1)}/s`));
        const u = document.createElement("span");
        u.className = "sb-belt-chip", u.style.borderColor = d, u.style.color = d, u.textContent = h.name, c.appendChild(u);
      } else {
        c.appendChild(document.createTextNode(`${l.rate.toFixed(1)}/s`));
        const d = document.createElement("span");
        d.className = "sb-belt-overflow", d.textContent = "\u26A0 overflow", c.appendChild(d);
      }
      n.appendChild(c);
    }
  }
  let Ie = null, la = 0;
  const Vn = /* @__PURE__ */ new Map();
  let oe = null;
  const To = "fucktorio:sat-cache:v1";
  let ln = new Uint8Array(0);
  function G0() {
    try {
      const n = localStorage.getItem(To);
      if (!n) return new Uint8Array(0);
      const t = atob(n), e = new Uint8Array(t.length);
      for (let s = 0; s < t.length; s++) e[s] = t.charCodeAt(s);
      return e;
    } catch (n) {
      return console.warn("[engine] could not read SAT cache from localStorage", n), new Uint8Array(0);
    }
  }
  function z0(n) {
    let t = "";
    for (let i = 0; i < n.length; i += 8192) t += String.fromCharCode.apply(null, Array.from(n.subarray(i, i + 8192)));
    const s = btoa(t);
    try {
      localStorage.setItem(To, s);
    } catch (i) {
      if (i instanceof DOMException && (i.name === "QuotaExceededError" || i.code === 22)) {
        console.warn("[engine] SAT cache quota exceeded \u2014 clearing");
        try {
          localStorage.removeItem(To);
        } catch {
        }
        ln = new Uint8Array(0);
      } else console.warn("[engine] failed to persist SAT cache", i);
    }
  }
  function D0(n) {
    const t = new Uint8Array(ln.length + n.length);
    t.set(ln, 0), t.set(n, ln.length), ln = t, z0(ln);
  }
  let Ao = [], zd = [], Dd = /* @__PURE__ */ new Map(), Eo = /* @__PURE__ */ new Set(), Mo = 0;
  function pn(n) {
    Mo += n;
    for (const t of Eo) t(Mo);
  }
  function H0(n) {
    return Eo.add(n), n(Mo), () => Eo.delete(n);
  }
  function Ee(n, t) {
    if (!Ie) throw new Error("Engine not initialized \u2014 call initEngine() first");
    const e = ++la;
    return pn(1), new Promise((s, i) => {
      Vn.set(e, {
        resolve: (r) => {
          pn(-1), oe === e && (oe = null), s(r);
        },
        reject: (r) => {
          pn(-1), oe === e && (oe = null), i(r);
        },
        onEvent: t
      }), Ie.postMessage({
        id: e,
        ...n
      });
    });
  }
  async function Hd() {
    if (Ie) return;
    if (Ie = new Worker(new URL("/fucktorio/pr-238/assets/engine.worker-B1RhJAUh.js", import.meta.url), {
      type: "module",
      name: "fucktorio-engine"
    }), Ie.onmessage = (t) => {
      const { id: e } = t.data, s = Vn.get(e);
      if (s) {
        if ("streamEvents" in t.data) {
          if (globalThis.__TRACE_LOGS === true) {
            const i = {};
            for (const r of t.data.streamEvents) {
              const o = r.phase ?? "?";
              i[o] = (i[o] ?? 0) + 1;
            }
            console.log(`[main  t=${performance.now().toFixed(0)}ms] arrived ${t.data.streamEvents.length}:`, i);
          }
          if (s.onEvent) for (const i of t.data.streamEvents) s.onEvent(i);
          return;
        }
        Vn.delete(e), t.data.ok ? s.resolve(t.data.result) : s.reject(new Error(t.data.error));
      }
    }, Ie.onerror = (t) => {
      console.error("[engine.worker] error", t);
    }, await Ee({
      method: "init"
    }), ln = G0(), ln.length > 0) try {
      const t = await Ee({
        method: "seedZoneCache",
        bytes: ln
      });
      globalThis.__TRACE_LOGS === true && console.log(`[engine] seeded ${t} SAT zone records from localStorage`);
    } catch (t) {
      console.warn("[engine] seedZoneCache failed; persistence disabled this session", t);
    }
    Ao = await Ee({
      method: "allProducibleItems"
    }), zd = await Ee({
      method: "allProducerMachines"
    });
    const n = await Ee({
      method: "defaultMachinesForItems",
      items: Ao,
      fallback: "assembling-machine-3"
    });
    Dd = new Map(n);
  }
  async function U0(n, t, e, s) {
    return oe !== null && await ca(), Ee({
      method: "solve",
      targetItem: n,
      targetRate: t,
      availableInputs: e,
      machineEntity: s
    });
  }
  function j0() {
    return Ao;
  }
  function V0() {
    return zd;
  }
  function Y0(n, t, e, s) {
    return Ee({
      method: "layout",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null
    });
  }
  function X0(n, t, e, s) {
    return Ee({
      method: "layoutTraced",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null
    });
  }
  async function ca() {
    if (!Ie) return;
    Ie.terminate(), Ie = null;
    const n = new Error("Engine superseded by a newer request");
    for (const [, t] of Vn) t.reject(n);
    Vn.clear(), oe = null, await Hd();
  }
  async function q0(n, t, e, s, i) {
    oe !== null && await ca();
    const r = ++la;
    return oe = r, pn(1), new Promise((o, a) => {
      Vn.set(r, {
        resolve: (c) => {
          pn(-1), oe === r && (oe = null), o(c);
        },
        reject: (c) => {
          pn(-1), oe === r && (oe = null), a(c);
        },
        onEvent: i
      });
      const l = globalThis.__TRACE_LOGS === true;
      Ie.postMessage({
        id: r,
        method: "layoutStreaming",
        result: n,
        maxBeltTier: t ?? null,
        strategy: e ?? null,
        rowLayout: s ?? null,
        traceLogs: l
      });
    });
  }
  function K0(n, t) {
    return Ee({
      method: "exportBlueprint",
      layout: n,
      label: t
    });
  }
  function J0(n, t) {
    return Dd.get(n) ?? t;
  }
  function Z0(n, t) {
    return Ee({
      method: "validateLayout",
      layout: n,
      solverResult: t
    });
  }
  function Q0(n, t) {
    return Ee({
      method: "solveFixture",
      fixtureJson: n,
      pinsJson: JSON.stringify(t)
    });
  }
  function tb(n) {
    return Ee({
      method: "parseBlueprint",
      bp: n
    });
  }
  async function Ud(n, t, e, s, i = 0) {
    if (oe !== null && await ca(), !Ie) throw new Error("Engine not initialized");
    const r = ++la;
    return oe = r, pn(1), new Promise((o, a) => {
      Vn.set(r, {
        resolve: (l) => {
          pn(-1), oe === r && (oe = null), o(l);
        },
        reject: (l) => {
          pn(-1), oe === r && (oe = null), a(l);
        },
        onEvent: (l) => {
          const c = l;
          if (c.phase === "SatImprovement" && c.data) s(c.data);
          else if (c.phase === "SatOptimumProven" && c.data) {
            const h = c.data, d = h.record_bytes instanceof Uint8Array ? h.record_bytes : new Uint8Array(h.record_bytes);
            D0(d);
          }
        }
      }), Ie.postMessage({
        id: r,
        method: "improveRegionStreaming",
        layout: n,
        regionId: t,
        budgetMs: e,
        maxIters: i
      });
    });
  }
  async function eb(n, t) {
    let e = n;
    const s = new Set((e.regions ?? []).filter((i) => i.kind === "crossing_zone").map((i) => i.id));
    for (; s.size > 0; ) {
      let i = 0;
      const r = [
        ...s
      ].sort((o, a) => o - a);
      for (const o of r) {
        if (!s.has(o)) continue;
        let a = false;
        e = await Ud(e, o, t.perRegionBudgetMs, (l) => {
          var _a2;
          l.iter > 0 && (a = true), (_a2 = t.onImprovement) == null ? void 0 : _a2.call(t, l);
        }, 1), a ? i += 1 : s.delete(o);
      }
      if (i === 0) break;
    }
    return e;
  }
  function nb() {
    return {
      solve: U0,
      allProducibleItems: j0,
      allProducerMachines: V0,
      buildLayout: Y0,
      buildLayoutTraced: X0,
      buildLayoutStreaming: q0,
      exportBlueprint: K0,
      defaultMachineForItem: J0,
      validateLayout: Z0,
      solveFixture: Q0,
      improveRegion: Ud,
      optimizeAllRegions: eb
    };
  }
  const sb = `
  .corpus-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    color: #e0e0e0;
    font-family: sans-serif;
    font-size: 13px;
    box-sizing: border-box;
    overflow: hidden;
  }
  .corpus-load {
    padding: 12px;
    border-bottom: 1px solid #333;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .corpus-load h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #c8c8c8;
  }
  .corpus-load p {
    margin: 0;
    color: #888;
    font-size: 11px;
    line-height: 1.4;
  }
  .corpus-load-btn {
    background: #0e639c;
    color: #fff;
    border: none;
    border-radius: 3px;
    padding: 6px 10px;
    font-size: 12px;
    cursor: pointer;
    text-align: center;
  }
  .corpus-load-btn:hover { background: #1177bb; }
  .corpus-paste {
    padding: 8px 12px;
    border-bottom: 1px solid #333;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .corpus-paste label {
    font-size: 11px;
    color: #888;
  }
  .corpus-paste textarea {
    background: #252526;
    color: #9cdcfe;
    border: 1px solid #444;
    border-radius: 3px;
    padding: 4px 6px;
    font-family: monospace;
    font-size: 10px;
    resize: none;
    height: 48px;
  }
  .corpus-paste textarea::placeholder { color: #555; }
  .corpus-paste textarea.error { border-color: #c44; }
  .corpus-paste-error {
    color: #f66;
    font-size: 10px;
    font-family: monospace;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .corpus-filters {
    padding: 8px 12px;
    border-bottom: 1px solid #333;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .corpus-filter-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  .corpus-filter-row label {
    color: #aaa;
    white-space: nowrap;
  }
  .corpus-filter-row input[type="checkbox"] {
    accent-color: #569cd6;
  }
  .corpus-filter-row select,
  .corpus-filter-row input[type="text"] {
    flex: 1;
    background: #252526;
    color: #e0e0e0;
    border: 1px solid #444;
    border-radius: 3px;
    padding: 3px 5px;
    font-size: 12px;
  }
  .corpus-count {
    font-size: 11px;
    color: #666;
    padding: 0 12px 4px;
  }
  .corpus-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .corpus-item {
    padding: 7px 12px;
    cursor: pointer;
    border-bottom: 1px solid #2a2a2a;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .corpus-item:hover { background: #2a2d2e; }
  .corpus-item.selected { background: #094771; }
  .corpus-item-name {
    font-family: monospace;
    font-size: 11px;
    color: #9cdcfe;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .corpus-item-meta {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .corpus-badge {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 2px;
    font-weight: 600;
    letter-spacing: 0.03em;
  }
  .corpus-badge-bus { background: #1a4a1a; color: #6a9; }
  .corpus-badge-product { background: #2a2a3a; color: #9cdcfe; }
  .corpus-badge-machines { color: #888; font-size: 10px; }
  .corpus-stats {
    padding: 8px 12px;
    border-top: 1px solid #333;
    font-family: monospace;
    font-size: 11px;
    background: #252526;
    display: none;
  }
  .corpus-stats.visible { display: block; }
  .corpus-stats-title {
    color: #9cdcfe;
    font-weight: 600;
    margin-bottom: 4px;
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .corpus-stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2px 8px;
  }
  .corpus-stats-row {
    display: flex;
    justify-content: space-between;
  }
  .corpus-stats-key { color: #888; }
  .corpus-stats-val { color: #b5cea8; }
  .corpus-empty {
    padding: 24px 12px;
    color: #555;
    font-size: 12px;
    text-align: center;
    line-height: 1.6;
  }
`;
  function ib() {
    if (document.getElementById("fucktorio-corpus-style")) return;
    const n = document.createElement("style");
    n.id = "fucktorio-corpus-style", n.textContent = sb, document.head.appendChild(n);
  }
  function rb(n, t) {
    ib(), n.innerHTML = "";
    const e = document.createElement("div");
    e.className = "corpus-panel", n.appendChild(e);
    let s = [], i = [], r = -1, o = false, a = "", l = "";
    const c = document.createElement("div");
    c.className = "corpus-load";
    const h = document.createElement("h2");
    h.textContent = "Corpus Browser", c.appendChild(h);
    const d = document.createElement("p");
    d.textContent = "Load corpus.json generated by scripts/analysis/mine_corpus.py", c.appendChild(d);
    const u = document.createElement("input");
    u.type = "file", u.accept = ".json", u.style.display = "none";
    const p = document.createElement("button");
    p.className = "corpus-load-btn", p.textContent = "Load corpus.json\u2026", p.onclick = () => u.click(), c.appendChild(u), c.appendChild(p), e.appendChild(c);
    const f = document.createElement("div");
    f.className = "corpus-paste";
    const g = document.createElement("label");
    g.textContent = "Or paste a blueprint string directly (parsed in-browser via WASM)", f.appendChild(g);
    const m = document.createElement("textarea");
    m.placeholder = "0eJyt... paste Factorio blueprint string", f.appendChild(m);
    const y = document.createElement("div");
    y.className = "corpus-paste-error", y.style.display = "none", f.appendChild(y), e.appendChild(f);
    let b = 0;
    m.addEventListener("input", () => {
      const R = m.value.trim(), B = ++b;
      if (!R) {
        m.classList.remove("error"), y.style.display = "none";
        return;
      }
      tb(R).then((O) => {
        B === b && (m.classList.remove("error"), y.style.display = "none", t(O));
      }).catch((O) => {
        B === b && (m.classList.add("error"), y.textContent = String(O), y.style.display = "block");
      });
    });
    const x = document.createElement("div");
    x.className = "corpus-filters", x.style.display = "none";
    const _ = document.createElement("div");
    _.className = "corpus-filter-row";
    const v = document.createElement("label");
    v.textContent = "Search";
    const w = document.createElement("input");
    w.type = "text", w.placeholder = "filter by name\u2026", _.appendChild(v), _.appendChild(w), x.appendChild(_);
    const C = document.createElement("div");
    C.className = "corpus-filter-row";
    const E = document.createElement("label");
    E.textContent = "Product";
    const P = document.createElement("select");
    C.appendChild(E), C.appendChild(P), x.appendChild(C);
    const T = document.createElement("div");
    T.className = "corpus-filter-row";
    const A = document.createElement("input");
    A.type = "checkbox";
    const H = document.createElement("label");
    H.style.display = "flex", H.style.alignItems = "center", H.style.gap = "5px", H.style.cursor = "pointer", H.appendChild(A), H.appendChild(document.createTextNode("Bus layouts only")), T.appendChild(H), x.appendChild(T), e.appendChild(x);
    const j = document.createElement("div");
    j.className = "corpus-count", j.style.display = "none", e.appendChild(j);
    const D = document.createElement("div");
    D.className = "corpus-list", e.appendChild(D);
    const N = document.createElement("div");
    N.className = "corpus-stats", e.appendChild(N);
    function $() {
      i = s.filter((R) => !(o && !R.stats.is_bus_layout || a && a !== "__all__" && R.stats.final_product !== a || l && !R.name.toLowerCase().includes(l.toLowerCase()))), r = -1, W();
    }
    function W() {
      if (D.innerHTML = "", j.textContent = `${i.length} of ${s.length} blueprint(s)`, i.length === 0) {
        const R = document.createElement("div");
        R.className = "corpus-empty", R.textContent = s.length === 0 ? "No corpus loaded yet." : "No blueprints match the current filters.", D.appendChild(R), N.classList.remove("visible");
        return;
      }
      for (let R = 0; R < i.length; R++) {
        const B = i[R], O = document.createElement("div");
        O.className = "corpus-item" + (R === r ? " selected" : "");
        const z = document.createElement("div");
        z.className = "corpus-item-name", z.textContent = B.name, z.title = B.name, O.appendChild(z);
        const Y = document.createElement("div");
        if (Y.className = "corpus-item-meta", B.stats.is_bus_layout) {
          const ot = document.createElement("span");
          ot.className = "corpus-badge corpus-badge-bus", ot.textContent = "BUS", Y.appendChild(ot);
        }
        if (B.stats.final_product) {
          const ot = document.createElement("span");
          ot.className = "corpus-badge corpus-badge-product", ot.textContent = B.stats.final_product, Y.appendChild(ot);
        }
        const K = document.createElement("span");
        K.className = "corpus-badge corpus-badge-machines", K.textContent = `${B.stats.machine_count}m`, Y.appendChild(K), O.appendChild(Y);
        const nt = R;
        O.addEventListener("click", () => X(nt)), D.appendChild(O);
      }
    }
    function X(R) {
      r = R;
      const B = i[R];
      W(), t(B.layout), U(B);
    }
    function U(R) {
      N.innerHTML = "", N.classList.add("visible");
      const B = document.createElement("div");
      B.className = "corpus-stats-title", B.textContent = R.name, B.title = R.name, N.appendChild(B);
      const O = document.createElement("div");
      O.className = "corpus-stats-grid";
      const z = R.stats, Y = [
        [
          "machines",
          String(z.machine_count)
        ],
        [
          "recipes",
          String(z.recipe_count)
        ],
        [
          "is_bus",
          z.is_bus_layout ? "yes" : "no"
        ],
        [
          "density",
          z.density.toFixed(2)
        ]
      ];
      z.is_bus_layout && Y.push([
        "bus_lanes",
        String(z.bus_lane_count)
      ], [
        "bus_pitch",
        z.bus_pitch.toFixed(1)
      ], [
        "row_pitch",
        z.row_pitch.toFixed(1)
      ], [
        "rows",
        String(z.row_count)
      ]), Y.push([
        "bbox",
        `${z.bbox_width}\xD7${z.bbox_height}`
      ], [
        "belt_tiles",
        String(z.belt_tiles)
      ]), z.pipe_tiles > 0 && Y.push([
        "pipe_tiles",
        String(z.pipe_tiles)
      ]);
      for (const [K, nt] of Y) {
        const ot = document.createElement("div");
        ot.className = "corpus-stats-row";
        const ht = document.createElement("span");
        ht.className = "corpus-stats-key", ht.textContent = K;
        const gt = document.createElement("span");
        gt.className = "corpus-stats-val", gt.textContent = nt, ot.appendChild(ht), ot.appendChild(gt), O.appendChild(ot);
      }
      N.appendChild(O);
    }
    function Z() {
      const R = new Set(s.map((O) => O.stats.final_product).filter(Boolean));
      P.innerHTML = "";
      const B = document.createElement("option");
      B.value = "__all__", B.textContent = "All products", P.appendChild(B);
      for (const O of Array.from(R).sort()) {
        const z = document.createElement("option");
        z.value = O, z.textContent = O, P.appendChild(z);
      }
    }
    u.addEventListener("change", () => {
      var _a2;
      const R = (_a2 = u.files) == null ? void 0 : _a2[0];
      if (!R) return;
      const B = new FileReader();
      B.onload = (O) => {
        var _a3;
        try {
          s = JSON.parse((_a3 = O.target) == null ? void 0 : _a3.result).blueprints ?? [], i = s, r = -1, p.textContent = `Reload corpus.json (${s.length} blueprints)`, x.style.display = "", j.style.display = "", N.classList.remove("visible"), Z(), $();
        } catch (z) {
          alert(`Failed to parse corpus.json: ${z}`);
        }
      }, B.readAsText(R);
    }), w.addEventListener("input", () => {
      l = w.value, $();
    }), P.addEventListener("change", () => {
      a = P.value, $();
    }), A.addEventListener("change", () => {
      o = A.checked, $();
    }), W();
  }
  const ob = [
    {
      label: "Iron Gear Wheel",
      item: "iron-gear-wheel",
      rate: 10,
      inputs: [
        "iron-plate"
      ],
      machine: "assembling-machine-2",
      tier: 1,
      status: "solved",
      desc: "1 recipe, 1 solid input"
    },
    {
      label: "Electronic Circuit",
      item: "electronic-circuit",
      rate: 10,
      inputs: [
        "iron-plate",
        "copper-plate"
      ],
      machine: "assembling-machine-2",
      tier: 2,
      status: "solved",
      desc: "2 recipes, 2 solid inputs"
    },
    {
      label: "Electronic Circuit (ores)",
      item: "electronic-circuit",
      rate: 10,
      inputs: [
        "iron-ore",
        "copper-ore"
      ],
      machine: "assembling-machine-2",
      tier: 2,
      status: "solved",
      desc: "From ores \u2014 smelting included"
    },
    {
      label: "Plastic Bar",
      item: "plastic-bar",
      rate: 10,
      inputs: [
        "coal",
        "petroleum-gas"
      ],
      machine: "chemical-plant",
      tier: 3,
      status: "solved",
      desc: "1 recipe, fluid + solid input"
    },
    {
      label: "Advanced Circuit",
      item: "advanced-circuit",
      rate: 10,
      inputs: [
        "iron-plate",
        "copper-plate",
        "plastic-bar"
      ],
      machine: "assembling-machine-2",
      tier: 4,
      status: "partial",
      desc: "5+ recipes, mixed solid/fluid"
    },
    {
      label: "Advanced Circuit (ores, T1)",
      item: "advanced-circuit",
      rate: 5,
      inputs: [
        "iron-plate",
        "copper-plate",
        "coal",
        "water",
        "crude-oil",
        "iron-ore",
        "copper-ore"
      ],
      machine: "assembling-machine-1",
      beltTier: "transport-belt",
      tier: 4,
      status: "partial",
      desc: "Full stack from raw ores, tier-1 machines + yellow belts"
    }
  ], ab = `
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@300;400;500;600;700&family=Space+Grotesk:wght@400;500;600;700&display=swap');

.fucktorio-landing {
  position: fixed;
  inset: 0;
  background: #0c0c0c;
  color: #d4d4d4;
  font-family: 'JetBrains Mono', monospace;
  display: flex;
  flex-direction: column;
  align-items: center;
  overflow-y: auto;
  z-index: 2000;
}

.fucktorio-landing::before {
  content: '';
  position: fixed;
  inset: 0;
  background-image:
    linear-gradient(rgba(56,189,248,0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(56,189,248,0.03) 1px, transparent 1px);
  background-size: 24px 24px;
  pointer-events: none;
  z-index: 0;
}

.fucktorio-landing-inner {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 900px;
  padding: 60px 32px 80px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

/* Header */

.fucktorio-landing-header {
  text-align: center;
  margin-bottom: 56px;
}

.fucktorio-landing-title {
  font-family: 'Space Grotesk', sans-serif;
  font-size: 52px;
  font-weight: 700;
  color: #f0f0f0;
  letter-spacing: -2px;
  margin: 0 0 8px;
  line-height: 1;
}

.fucktorio-landing-title span {
  background: linear-gradient(135deg, #38bdf8 0%, #818cf8 50%, #c084fc 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.fucktorio-landing-subtitle {
  font-size: 13px;
  font-weight: 300;
  color: #6b7280;
  letter-spacing: 2.5px;
  text-transform: uppercase;
  margin: 0;
}

/* Ladder */

.fucktorio-landing-ladder {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 48px;
}

.fucktorio-landing-ladder-header {
  display: grid;
  grid-template-columns: 64px 1fr 100px 80px;
  padding: 0 16px 10px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #4b5563;
  border-bottom: 1px solid #1f2937;
  margin-bottom: 2px;
}

/* Card */

.fucktorio-landing-card {
  display: grid;
  grid-template-columns: 64px 1fr 100px 80px;
  align-items: center;
  padding: 14px 16px;
  background: rgba(255,255,255,0.02);
  border: 1px solid rgba(255,255,255,0.04);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  overflow: hidden;
}

.fucktorio-landing-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: transparent;
  transition: background 0.2s ease;
}

.fucktorio-landing-card:hover {
  background: rgba(255,255,255,0.05);
  border-color: rgba(255,255,255,0.08);
}

.fucktorio-landing-card.solved:hover::before { background: #34d399; }
.fucktorio-landing-card.partial:hover::before { background: #fbbf24; }
.fucktorio-landing-card.wip { opacity: 0.4; cursor: default; }
.fucktorio-landing-card.wip:hover { background: rgba(255,255,255,0.02); border-color: rgba(255,255,255,0.04); }
.fucktorio-landing-card.loading { pointer-events: none; }

.fucktorio-landing-tier {
  font-size: 11px;
  font-weight: 600;
  color: #4b5563;
}
.fucktorio-landing-tier span {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 1.5px solid #374151;
  color: #6b7280;
  font-size: 12px;
}

.fucktorio-landing-card-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.fucktorio-landing-card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
  color: #e5e7eb;
}

.fucktorio-landing-card-icon {
  width: 22px;
  height: 22px;
  image-rendering: pixelated;
  flex-shrink: 0;
}

.fucktorio-landing-card-rate {
  font-size: 11px;
  color: #6b7280;
  font-weight: 300;
}

.fucktorio-landing-card-desc {
  font-size: 11px;
  color: #4b5563;
  font-weight: 300;
}

.fucktorio-landing-status {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  padding: 3px 8px;
  border-radius: 3px;
  text-align: center;
  justify-self: center;
}
.fucktorio-landing-status.solved { background: rgba(52,211,153,0.12); color: #34d399; }
.fucktorio-landing-status.partial { background: rgba(251,191,36,0.12); color: #fbbf24; }
.fucktorio-landing-status.wip { background: rgba(107,114,128,0.12); color: #6b7280; }

.fucktorio-landing-entities {
  font-size: 11px;
  color: #4b5563;
  text-align: right;
  font-weight: 300;
}

/* Footer */

.fucktorio-landing-footer {
  margin-top: 16px;
  text-align: center;
}

.fucktorio-landing-launch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: rgba(255,255,255,0.04);
  color: #9ca3af;
  border: 1px solid rgba(255,255,255,0.08);
  padding: 12px 28px;
  border-radius: 6px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  letter-spacing: 0.3px;
}
.fucktorio-landing-launch:hover {
  background: rgba(255,255,255,0.08);
  color: #e5e7eb;
  border-color: rgba(255,255,255,0.15);
}
.fucktorio-landing-launch svg {
  width: 16px;
  height: 16px;
}

/* Modal */

.fucktorio-preview-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.75);
  backdrop-filter: blur(8px);
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fucktorio-fadeIn 0.2s ease forwards;
}

@keyframes fucktorio-fadeIn { to { opacity: 1; } }

.fucktorio-preview-modal {
  background: #141414;
  border: 1px solid #2a2a2a;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  width: 75vw;
  max-width: 1000px;
  height: 70vh;
  max-height: 700px;
  box-shadow: 0 25px 60px rgba(0,0,0,0.5);
  animation: fucktorio-modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  transform: scale(0.95) translateY(10px);
  opacity: 0;
}

@keyframes fucktorio-modalIn { to { transform: scale(1) translateY(0); opacity: 1; } }

.fucktorio-preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid #1f2937;
  background: rgba(255,255,255,0.02);
  flex-shrink: 0;
}

.fucktorio-preview-title {
  font-size: 13px;
  font-weight: 500;
  color: #9ca3af;
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: 'JetBrains Mono', monospace;
}

.fucktorio-preview-title img {
  width: 18px;
  height: 18px;
  image-rendering: pixelated;
}

.fucktorio-preview-stats {
  display: flex;
  gap: 16px;
  font-size: 11px;
  color: #4b5563;
  font-family: 'JetBrains Mono', monospace;
}

.fucktorio-preview-stats span { color: #6b7280; }

.fucktorio-preview-close {
  background: none;
  border: 1px solid #333;
  color: #6b7280;
  width: 28px;
  height: 28px;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  transition: all 0.15s ease;
  font-family: 'JetBrains Mono', monospace;
}
.fucktorio-preview-close:hover {
  background: rgba(255,255,255,0.06);
  color: #e5e7eb;
  border-color: #555;
}

.fucktorio-preview-canvas {
  flex: 1;
  position: relative;
  overflow: hidden;
  background: #111;
}

.fucktorio-preview-canvas canvas {
  display: block;
}

.fucktorio-preview-badge {
  position: absolute;
  bottom: 12px;
  right: 12px;
  background: rgba(0,0,0,0.75);
  color: #6b7280;
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  pointer-events: none;
  z-index: 1;
}

.fucktorio-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #1f2937;
  border-top-color: #38bdf8;
  border-radius: 50%;
  animation: fucktorio-spin 0.6s linear infinite;
  display: inline-block;
}

@keyframes fucktorio-spin { to { transform: rotate(360deg); } }
`;
  function lb() {
    if (document.getElementById("fucktorio-landing-style")) return;
    const n = document.createElement("style");
    n.id = "fucktorio-landing-style", n.textContent = ab, document.head.appendChild(n);
  }
  function cb(n, t, e) {
    lb(), n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "fucktorio-landing", n.appendChild(s);
    const i = document.createElement("div");
    i.className = "fucktorio-landing-inner", s.appendChild(i);
    const r = document.createElement("div");
    r.className = "fucktorio-landing-header", i.appendChild(r);
    const o = document.createElement("h1");
    o.className = "fucktorio-landing-title", o.innerHTML = "Fuck<span>torio</span>", r.appendChild(o);
    const a = document.createElement("p");
    a.className = "fucktorio-landing-subtitle", a.textContent = "Automated Factory Blueprint Generator", r.appendChild(a);
    const l = document.createElement("div");
    l.className = "fucktorio-landing-ladder", i.appendChild(l);
    const c = document.createElement("div");
    c.className = "fucktorio-landing-ladder-header", c.innerHTML = "<span>Tier</span><span>Recipe</span><span>Status</span><span>Entities</span>", l.appendChild(c);
    for (const u of ob) {
      const p = document.createElement("div");
      p.className = `fucktorio-landing-card ${u.status}`;
      const f = document.createElement("div");
      f.className = "fucktorio-landing-tier", f.innerHTML = `<span>${u.tier}</span>`, p.appendChild(f);
      const g = document.createElement("div");
      g.className = "fucktorio-landing-card-body";
      const m = document.createElement("div");
      m.className = "fucktorio-landing-card-title";
      const y = document.createElement("img");
      y.src = `/fucktorio/pr-238/icons/${u.item}.png`, y.className = "fucktorio-landing-card-icon", y.onerror = () => {
        y.style.display = "none";
      }, m.appendChild(y), m.appendChild(document.createTextNode(u.label));
      const b = document.createElement("span");
      b.className = "fucktorio-landing-card-rate", b.textContent = `${u.rate}/s`, m.appendChild(b), g.appendChild(m);
      const x = document.createElement("div");
      x.className = "fucktorio-landing-card-desc", x.textContent = u.desc, g.appendChild(x), p.appendChild(g);
      const _ = document.createElement("div");
      _.className = `fucktorio-landing-status ${u.status}`, _.textContent = u.status === "solved" ? "Solved" : u.status === "partial" ? "Partial" : "WIP", p.appendChild(_);
      const v = document.createElement("div");
      v.className = "fucktorio-landing-entities", v.textContent = "\u2014", p.appendChild(v), u.status !== "wip" && p.addEventListener("click", () => {
        hb(t, u, p, v);
      }), l.appendChild(p);
    }
    const h = document.createElement("div");
    h.className = "fucktorio-landing-footer", i.appendChild(h);
    const d = document.createElement("button");
    d.className = "fucktorio-landing-launch", d.innerHTML = '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>Open Generator', d.addEventListener("click", () => {
      s.style.transition = "opacity 0.3s ease", s.style.opacity = "0", setTimeout(() => {
        s.remove(), e.onOpenGenerator();
      }, 300);
    }), h.appendChild(d);
  }
  function hb(n, t, e, s) {
    e.classList.contains("loading") || (e.classList.add("loading"), s.innerHTML = '<span class="fucktorio-spinner"></span>', (async () => {
      let i, r;
      try {
        const o = n.defaultMachineForItem(t.item, t.machine);
        i = await n.solve(t.item, t.rate, t.inputs, o), r = await n.buildLayout(i, t.beltTier);
      } catch (o) {
        e.classList.remove("loading"), s.textContent = "error", console.error("Landing solve/layout failed:", o);
        return;
      }
      s.textContent = String(r.entities.length), e.classList.remove("loading"), ld(i.machines.map((o) => ({
        recipe: o.recipe,
        count: o.count,
        inputs: o.inputs.map((a) => ({
          item: a.item,
          rate: a.rate
        })),
        outputs: o.outputs.map((a) => ({
          item: a.item,
          rate: a.rate
        }))
      }))), db(t, r, i).catch((o) => {
        console.error("Modal init failed:", o);
      });
    })());
  }
  async function db(n, t, e) {
    const s = document.createElement("div");
    s.className = "fucktorio-preview-backdrop", document.body.appendChild(s);
    const i = (E) => {
      E.key === "Escape" && a();
    };
    let r = false, o = null;
    function a() {
      r || (r = true, document.removeEventListener("keydown", i), o && o.destroy(true), s.remove());
    }
    document.addEventListener("keydown", i), s.addEventListener("click", (E) => {
      E.target === s && a();
    });
    const l = document.createElement("div");
    l.className = "fucktorio-preview-modal", s.appendChild(l);
    const c = document.createElement("div");
    c.className = "fucktorio-preview-header";
    const h = document.createElement("div");
    h.className = "fucktorio-preview-title";
    const d = document.createElement("img");
    d.src = `/fucktorio/pr-238/icons/${n.item}.png`, d.onerror = () => {
      d.style.display = "none";
    }, h.appendChild(d), h.appendChild(document.createTextNode(` ${n.label} \u2014 ${n.rate}/s`)), c.appendChild(h);
    const u = document.createElement("div");
    u.className = "fucktorio-preview-stats";
    const p = `${t.width ?? 0}\xD7${t.height ?? 0}`, f = e.machines.reduce((E, P) => E + Math.ceil(P.count), 0);
    u.innerHTML = `<span>${f} machines</span><span>${p} tiles</span>`, c.appendChild(u);
    const g = document.createElement("button");
    g.className = "fucktorio-preview-close", g.textContent = "\xD7", g.addEventListener("click", a), c.appendChild(g), l.appendChild(c);
    const m = document.createElement("div");
    m.className = "fucktorio-preview-canvas", l.appendChild(m);
    const y = document.createElement("div");
    y.className = "fucktorio-preview-badge", y.textContent = `0 / ${t.entities.length}`, m.appendChild(y), o = new No(), await o.init({
      resizeTo: m,
      background: 1118481,
      antialias: true
    }), m.insertBefore(o.canvas, y), o.canvas.addEventListener("contextmenu", (E) => E.preventDefault());
    const b = (t.width ?? 20) * k, x = (t.height ?? 20) * k, _ = Math.max(b, x, 600) + 200, v = new id({
      screenWidth: m.clientWidth,
      screenHeight: m.clientHeight,
      worldWidth: _,
      worldHeight: _,
      events: o.renderer.events
    });
    v.drag({
      mouseButtons: "left"
    }).pinch().wheel().decelerate(), o.stage.addChild(v);
    const w = new zt();
    v.addChild(w), v.fit(true, b * 1.15, x * 1.2), v.moveCenter(b / 2, x / 2);
    const { renderLayoutAnimated: C } = await Gn(async () => {
      const { renderLayoutAnimated: E } = await import("./animated-CIfYM6SX.js");
      return {
        renderLayoutAnimated: E
      };
    }, []);
    C(t, w, y, () => {
    });
  }
  var ve = Uint8Array, ls = Uint16Array, ub = Int32Array, jd = new ve([
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    1,
    1,
    1,
    1,
    2,
    2,
    2,
    2,
    3,
    3,
    3,
    3,
    4,
    4,
    4,
    4,
    5,
    5,
    5,
    5,
    0,
    0,
    0,
    0
  ]), Vd = new ve([
    0,
    0,
    0,
    0,
    1,
    1,
    2,
    2,
    3,
    3,
    4,
    4,
    5,
    5,
    6,
    6,
    7,
    7,
    8,
    8,
    9,
    9,
    10,
    10,
    11,
    11,
    12,
    12,
    13,
    13,
    0,
    0
  ]), pb = new ve([
    16,
    17,
    18,
    0,
    8,
    7,
    9,
    6,
    10,
    5,
    11,
    4,
    12,
    3,
    13,
    2,
    14,
    1,
    15
  ]), Yd = function(n, t) {
    for (var e = new ls(31), s = 0; s < 31; ++s) e[s] = t += 1 << n[s - 1];
    for (var i = new ub(e[30]), s = 1; s < 30; ++s) for (var r = e[s]; r < e[s + 1]; ++r) i[r] = r - e[s] << 5 | s;
    return {
      b: e,
      r: i
    };
  }, Xd = Yd(jd, 2), qd = Xd.b, fb = Xd.r;
  qd[28] = 258, fb[258] = 28;
  var mb = Yd(Vd, 0), gb = mb.b, Po = new ls(32768);
  for (var Wt = 0; Wt < 32768; ++Wt) {
    var xn = (Wt & 43690) >> 1 | (Wt & 21845) << 1;
    xn = (xn & 52428) >> 2 | (xn & 13107) << 2, xn = (xn & 61680) >> 4 | (xn & 3855) << 4, Po[Wt] = ((xn & 65280) >> 8 | (xn & 255) << 8) >> 1;
  }
  var Ds = (function(n, t, e) {
    for (var s = n.length, i = 0, r = new ls(t); i < s; ++i) n[i] && ++r[n[i] - 1];
    var o = new ls(t);
    for (i = 1; i < t; ++i) o[i] = o[i - 1] + r[i - 1] << 1;
    var a;
    if (e) {
      a = new ls(1 << t);
      var l = 15 - t;
      for (i = 0; i < s; ++i) if (n[i]) for (var c = i << 4 | n[i], h = t - n[i], d = o[n[i] - 1]++ << h, u = d | (1 << h) - 1; d <= u; ++d) a[Po[d] >> l] = c;
    } else for (a = new ls(s), i = 0; i < s; ++i) n[i] && (a[i] = Po[o[n[i] - 1]++] >> 15 - n[i]);
    return a;
  }), ni = new ve(288);
  for (var Wt = 0; Wt < 144; ++Wt) ni[Wt] = 8;
  for (var Wt = 144; Wt < 256; ++Wt) ni[Wt] = 9;
  for (var Wt = 256; Wt < 280; ++Wt) ni[Wt] = 7;
  for (var Wt = 280; Wt < 288; ++Wt) ni[Wt] = 8;
  var Kd = new ve(32);
  for (var Wt = 0; Wt < 32; ++Wt) Kd[Wt] = 5;
  var yb = Ds(ni, 9, 1), xb = Ds(Kd, 5, 1), Wr = function(n) {
    for (var t = n[0], e = 1; e < n.length; ++e) n[e] > t && (t = n[e]);
    return t;
  }, Fe = function(n, t, e) {
    var s = t / 8 | 0;
    return (n[s] | n[s + 1] << 8) >> (t & 7) & e;
  }, Gr = function(n, t) {
    var e = t / 8 | 0;
    return (n[e] | n[e + 1] << 8 | n[e + 2] << 16) >> (t & 7);
  }, bb = function(n) {
    return (n + 7) / 8 | 0;
  }, _b = function(n, t, e) {
    return (e == null || e > n.length) && (e = n.length), new ve(n.subarray(t, e));
  }, wb = [
    "unexpected EOF",
    "invalid block type",
    "invalid length/literal",
    "invalid distance",
    "stream finished",
    "no stream handler",
    ,
    "no callback",
    "invalid UTF-8 data",
    "extra field too long",
    "date not in range 1980-2099",
    "filename too long",
    "stream finishing",
    "invalid zip data"
  ], Ne = function(n, t, e) {
    var s = new Error(t || wb[n]);
    if (s.code = n, Error.captureStackTrace && Error.captureStackTrace(s, Ne), !e) throw s;
    return s;
  }, vb = function(n, t, e, s) {
    var i = n.length, r = 0;
    if (!i || t.f && !t.l) return e || new ve(0);
    var o = !e, a = o || t.i != 2, l = t.i;
    o && (e = new ve(i * 3));
    var c = function(I) {
      var G = e.length;
      if (I > G) {
        var V = new ve(Math.max(G * 2, I));
        V.set(e), e = V;
      }
    }, h = t.f || 0, d = t.p || 0, u = t.b || 0, p = t.l, f = t.d, g = t.m, m = t.n, y = i * 8;
    do {
      if (!p) {
        h = Fe(n, d, 1);
        var b = Fe(n, d + 1, 3);
        if (d += 3, b) if (b == 1) p = yb, f = xb, g = 9, m = 5;
        else if (b == 2) {
          var w = Fe(n, d, 31) + 257, C = Fe(n, d + 10, 15) + 4, E = w + Fe(n, d + 5, 31) + 1;
          d += 14;
          for (var P = new ve(E), T = new ve(19), A = 0; A < C; ++A) T[pb[A]] = Fe(n, d + A * 3, 7);
          d += C * 3;
          for (var H = Wr(T), j = (1 << H) - 1, D = Ds(T, H, 1), A = 0; A < E; ) {
            var N = D[Fe(n, d, j)];
            d += N & 15;
            var x = N >> 4;
            if (x < 16) P[A++] = x;
            else {
              var $ = 0, W = 0;
              for (x == 16 ? (W = 3 + Fe(n, d, 3), d += 2, $ = P[A - 1]) : x == 17 ? (W = 3 + Fe(n, d, 7), d += 3) : x == 18 && (W = 11 + Fe(n, d, 127), d += 7); W--; ) P[A++] = $;
            }
          }
          var X = P.subarray(0, w), U = P.subarray(w);
          g = Wr(X), m = Wr(U), p = Ds(X, g, 1), f = Ds(U, m, 1);
        } else Ne(1);
        else {
          var x = bb(d) + 4, _ = n[x - 4] | n[x - 3] << 8, v = x + _;
          if (v > i) {
            l && Ne(0);
            break;
          }
          a && c(u + _), e.set(n.subarray(x, v), u), t.b = u += _, t.p = d = v * 8, t.f = h;
          continue;
        }
        if (d > y) {
          l && Ne(0);
          break;
        }
      }
      a && c(u + 131072);
      for (var Z = (1 << g) - 1, R = (1 << m) - 1, B = d; ; B = d) {
        var $ = p[Gr(n, d) & Z], O = $ >> 4;
        if (d += $ & 15, d > y) {
          l && Ne(0);
          break;
        }
        if ($ || Ne(2), O < 256) e[u++] = O;
        else if (O == 256) {
          B = d, p = null;
          break;
        } else {
          var z = O - 254;
          if (O > 264) {
            var A = O - 257, Y = jd[A];
            z = Fe(n, d, (1 << Y) - 1) + qd[A], d += Y;
          }
          var K = f[Gr(n, d) & R], nt = K >> 4;
          K || Ne(3), d += K & 15;
          var U = gb[nt];
          if (nt > 3) {
            var Y = Vd[nt];
            U += Gr(n, d) & (1 << Y) - 1, d += Y;
          }
          if (d > y) {
            l && Ne(0);
            break;
          }
          a && c(u + 131072);
          var ot = u + z;
          if (u < U) {
            var ht = r - U, gt = Math.min(U, ot);
            for (ht + u < 0 && Ne(3); u < gt; ++u) e[u] = s[ht + u];
          }
          for (; u < ot; ++u) e[u] = e[u - U];
        }
      }
      t.l = p, t.p = B, t.b = u, t.f = h, p && (h = 1, t.m = g, t.d = f, t.n = m);
    } while (!h);
    return u != e.length && o ? _b(e, 0, u) : e.subarray(0, u);
  }, Cb = new ve(0), Sb = function(n) {
    (n[0] != 31 || n[1] != 139 || n[2] != 8) && Ne(6, "invalid gzip data");
    var t = n[3], e = 10;
    t & 4 && (e += (n[10] | n[11] << 8) + 2);
    for (var s = (t >> 3 & 1) + (t >> 4 & 1); s > 0; s -= !n[e++]) ;
    return e + (t & 2);
  }, kb = function(n) {
    var t = n.length;
    return (n[t - 4] | n[t - 3] << 8 | n[t - 2] << 16 | n[t - 1] << 24) >>> 0;
  };
  function Tb(n, t) {
    var e = Sb(n);
    return e + 8 > n.length && Ne(6, "invalid gzip data"), vb(n.subarray(e, -8), {
      i: 2
    }, new ve(kb(n)), t);
  }
  var Ab = typeof TextDecoder < "u" && new TextDecoder(), Eb = 0;
  try {
    Ab.decode(Cb, {
      stream: true
    }), Eb = 1;
  } catch {
  }
  const zr = "fls1";
  async function Jd(n) {
    const t = typeof n == "string" ? n : new TextDecoder().decode(n);
    if (!t.startsWith(zr)) throw new Error(`Not a layout snapshot: expected "${zr}" prefix, got "${t.slice(0, 4)}"`);
    const e = t.slice(zr.length), s = Uint8Array.from(atob(e), (o) => o.charCodeAt(0)), i = Tb(s), r = new TextDecoder().decode(i);
    return JSON.parse(r);
  }
  function Mb(n) {
    return new Promise((t, e) => {
      const s = new FileReader();
      s.onload = () => t(s.result), s.onerror = () => e(new Error("Failed to read file")), s.readAsText(n);
    });
  }
  function Pb(n, t) {
    n.addEventListener("dragover", (e) => {
      e.preventDefault(), e.stopPropagation(), n.style.outline = "2px dashed #569cd6";
    }), n.addEventListener("dragleave", () => {
      n.style.outline = "none";
    }), n.addEventListener("drop", async (e) => {
      var _a2;
      e.preventDefault(), e.stopPropagation(), n.style.outline = "none";
      const s = (_a2 = e.dataTransfer) == null ? void 0 : _a2.files[0];
      if (s) {
        if (!s.name.endsWith(".fls")) {
          alert("Expected a .fls snapshot file");
          return;
        }
        try {
          const i = await Mb(s), r = await Jd(i);
          t(r);
        } catch (i) {
          alert(`Failed to load snapshot: ${i}`);
        }
      }
    });
  }
  function Ib(n, t, e) {
    const s = document.createElement("div");
    s.style.cssText = "background:rgba(0,40,80,0.85);color:#e0e0e0;font:11px monospace;padding:6px 10px;border-bottom:1px solid #569cd6;display:flex;align-items:center;gap:8px;flex-wrap:wrap;z-index:20";
    const { params: i, context: r, trace: o, validation: a } = t;
    let c = `<span style="color:#569cd6;font-weight:bold">${r.test_name ?? r.label ?? "snapshot"}</span>`;
    r.git_sha && (c += ` <span style="color:#888">(git: ${r.git_sha})</span>`), c += ` <span style="color:#aaa">${t.created_at}</span>`;
    let h = `${i.item} @ ${i.rate}/s`;
    h += ` \xB7 ${i.machine}`, i.belt_tier && (h += ` \xB7 ${i.belt_tier}`), i.inputs.length && (h += ` \xB7 from ${i.inputs.join(", ")}`);
    const d = document.createElement("span");
    d.innerHTML = c, s.appendChild(d);
    const u = document.createElement("span");
    if (u.style.cssText = "color:#888;margin-left:8px", u.textContent = h, s.appendChild(u), !o.complete) {
      const b = document.createElement("span");
      b.style.cssText = "color:#ff6b6b;margin-left:8px", b.textContent = "\u26A0 Incomplete trace", s.appendChild(b);
    }
    if (a.truncated) {
      const b = document.createElement("span");
      b.style.cssText = "color:#ff6b6b;margin-left:4px", b.textContent = "\u26A0 Validation truncated", s.appendChild(b);
    }
    const p = a.issues.filter((b) => b.severity === "Error").length, f = a.issues.length - p;
    if (a.issues.length > 0) {
      const b = document.createElement("span");
      b.style.cssText = "margin-left:8px", b.innerHTML = `<span style="color:#f66">${p} errors</span> <span style="color:#fa0">${f} warnings</span>`, s.appendChild(b);
    }
    const g = document.createElement("span");
    g.style.cssText = "flex:1", s.appendChild(g);
    const m = document.createElement("button");
    m.textContent = "Re-solve", m.title = "Not yet implemented", m.disabled = true, m.style.cssText = "background:#222;border:1px solid #444;color:#666;padding:2px 8px;border-radius:3px;font:11px monospace;cursor:not-allowed", s.appendChild(m);
    const y = document.createElement("button");
    return y.textContent = "Clear", y.style.cssText = "background:#333;border:1px solid #666;color:#ccc;padding:2px 8px;border-radius:3px;cursor:pointer;font:11px monospace", y.addEventListener("click", () => e.onClear()), s.appendChild(y), n.insertBefore(s, n.firstChild), s;
  }
  function rc(n, t, e, s, i, r, o, a) {
    const l = s - t, c = i - e, h = Math.sqrt(l * l + c * c);
    if (h === 0) return;
    const d = l / h, u = c / h;
    let p = 0;
    for (; p < h; ) {
      const f = Math.min(p + r, h);
      n.moveTo(t + d * p, e + u * p).lineTo(t + d * f, e + u * f).stroke(a), p = f + o;
    }
  }
  function Rb(n, t, e, s, i) {
    const r = new zt(), o = n.find((h) => h.phase === "LanesPlanned");
    if (o) for (const h of o.data.lanes) {
      const d = new ut(), u = h.x * k;
      d.rect(u, 0, k, e * k).fill({
        color: h.is_fluid ? 4500223 : 4521864,
        alpha: 0.04
      }), d.eventMode = "static", d.on("pointerenter", () => i(`Lane: ${h.item} @ x=${h.x} (${h.rate.toFixed(1)}/s${h.is_fluid ? " fluid" : ""})`)), d.on("pointerleave", () => i(null)), r.addChild(d);
    }
    const a = n.find((h) => h.phase === "RowsPlaced");
    if (a) for (const h of a.data.rows) {
      const d = new ut(), u = h.y_end * k;
      d.moveTo(0, u).lineTo(t * k, u).stroke({
        width: 1,
        color: 6982234,
        alpha: 0.3
      }), d.eventMode = "static", d.on("pointerenter", () => i(`Row ${h.index}: ${h.recipe} (${h.machine_count}\xD7 ${h.machine})`)), d.on("pointerleave", () => i(null)), r.addChild(d);
    }
    for (const h of n) {
      if (h.phase !== "BalancerStamped") continue;
      const d = h.data, u = (d.y_end - d.y_start) * k;
      if (u <= 0) continue;
      const p = new ut();
      p.rect(0, d.y_start * k, t * k, u).fill({
        color: 11158783,
        alpha: 0.05
      }).stroke({
        width: 1,
        color: 11158783,
        alpha: 0.4
      }), p.eventMode = "static", p.on("pointerenter", () => i(`Balancer: ${d.item} ${d.shape[0]}\u2192${d.shape[1]} (template: ${d.template_found})`)), p.on("pointerleave", () => i(null)), r.addChild(p);
    }
    for (const h of n) {
      if (h.phase !== "TapoffRouted") continue;
      const d = h.data, u = new ut();
      u.moveTo(d.from_x * k + k / 2, d.from_y * k + k / 2).lineTo(d.to_x * k + k / 2, d.to_y * k + k / 2).stroke({
        width: 2,
        color: 8978244,
        alpha: 0.5
      }), u.eventMode = "static", u.on("pointerenter", () => i(`Tap-off: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) len=${d.path_len}`)), u.on("pointerleave", () => i(null)), r.addChild(u);
    }
    for (const h of n) {
      if (h.phase !== "MergerBlockPlaced") continue;
      const d = h.data, u = new ut();
      u.rect(0, d.block_y * k, t * k, d.block_height * k).fill({
        color: 16763972,
        alpha: 0.05
      }).stroke({
        width: 1,
        color: 16763972,
        alpha: 0.4
      }), u.eventMode = "static", u.on("pointerenter", () => i(`Merger: ${d.item} (${d.lanes} lanes, y=${d.block_y}..${d.block_y + d.block_height})`)), u.on("pointerleave", () => i(null)), r.addChild(u);
    }
    for (const h of n) {
      if (h.phase !== "RouteFailure") continue;
      const d = h.data, u = d.from_x * k + k / 2, p = d.from_y * k + k / 2, f = 3, g = new ut();
      g.label = "RouteFailure", g.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), g.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), rc(g, u, p, d.to_x * k + k / 2, d.to_y * k + k / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), g.eventMode = "static", g.on("pointerenter", () => i(`Route failed: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) [${d.spec_key}]`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    const l = Lb;
    let c = 0;
    for (const h of n) {
      if (h.phase !== "GhostSpecRouted") continue;
      const d = h.data, u = l[c % l.length];
      c++;
      const p = new ut();
      if (d.tiles && d.tiles.length > 1) {
        p.setStrokeStyle({
          width: 3,
          color: u,
          alpha: 0.7
        }), p.moveTo(d.tiles[0][0] * k + k / 2, d.tiles[0][1] * k + k / 2);
        for (let f = 1; f < d.tiles.length; f++) p.lineTo(d.tiles[f][0] * k + k / 2, d.tiles[f][1] * k + k / 2);
        p.stroke();
      }
      p.eventMode = "static", p.on("pointerenter", () => i(`Ghost path: ${d.spec_key} len=${d.path_len} crossings=${d.crossings} turns=${d.turns}`)), p.on("pointerleave", () => i(null)), r.addChild(p);
    }
    for (const h of n) {
      if (h.phase !== "GhostSpecFailed") continue;
      const d = h.data, u = d.from_x * k + k / 2, p = d.from_y * k + k / 2, f = 4, g = new ut();
      g.label = "RouteFailure", g.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), g.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), rc(g, u, p, d.to_x * k + k / 2, d.to_y * k + k / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), g.eventMode = "static", g.on("pointerenter", () => i(`Ghost failed: ${d.spec_key} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y})`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    for (const h of n) {
      if (h.phase !== "GhostClusterSolved" && h.phase !== "GhostClusterFailed") continue;
      const d = h.phase === "GhostClusterFailed", u = d ? null : h.data, p = d ? h.data : null, f = u ?? p, g = d ? 16729156 : 4500223, m = new ut();
      m.rect(f.zone_x * k, f.zone_y * k, f.zone_w * k, f.zone_h * k).fill({
        color: g,
        alpha: d ? 0.15 : 0.08
      }).stroke({
        width: d ? 2 : 1,
        color: g,
        alpha: d ? 0.9 : 0.6
      }), m.eventMode = "static";
      const y = u ? ` vars=${u.variables} clauses=${u.clauses} ${(u.solve_time_us / 1e3).toFixed(1)}ms` : "";
      m.on("pointerenter", () => i(`Cluster #${f.cluster_id}: ${f.zone_w}x${f.zone_h} @ (${f.zone_x},${f.zone_y}) ${f.boundary_count} ports${y}${d ? " FAILED" : ""}`)), m.on("pointerleave", () => i(null)), r.addChild(m);
    }
    return s.addChild(r), r;
  }
  const Lb = [
    5676246,
    6996096,
    13672512,
    11567312
  ], $b = {
    Error: 16729156,
    Warning: 16755200
  }, Bb = 0.85;
  function Ob(n, t, e) {
    const s = new zt(), i = /* @__PURE__ */ new Map();
    for (const r of n) {
      if (r.x == null || r.y == null) continue;
      const o = $b[r.severity] ?? 4500223, a = new ut();
      a.rect(r.x * k, r.y * k, k, k).stroke({
        width: 2,
        color: o,
        alpha: Bb
      }), a.eventMode = "static", a.on("pointerenter", () => e(`[${r.severity}] ${r.category}: ${r.message}`)), a.on("pointerleave", () => e(null)), s.addChild(a);
      const l = `${r.x},${r.y}`, c = i.get(l);
      c ? c.push(a) : i.set(l, [
        a
      ]);
    }
    return t.addChild(s), {
      layer: s,
      circleMap: i
    };
  }
  function Fb(n) {
    const t = n.point.direction;
    return t === "East" || t === "West" ? "horizontal" : "vertical";
  }
  function Nb(n) {
    const t = new Set(n.map(Fb));
    return t.size === 1 ? [
      ...t
    ][0] : "mixed";
  }
  function Wb(n) {
    const t = n.ports ?? [];
    if (t.length === 0) return {
      cls: "no-ports",
      summary: "Region has no boundary ports \u2014 degenerate region with no flow.",
      items: /* @__PURE__ */ new Map()
    };
    const e = /* @__PURE__ */ new Map();
    for (const a of t) {
      const l = a.item ?? "?";
      e.has(l) || e.set(l, {
        name: l,
        axis: "horizontal",
        inputs: [],
        outputs: []
      });
      const c = e.get(l);
      a.io === "Input" ? c.inputs.push(a) : c.outputs.push(a);
    }
    for (const a of e.values()) a.axis = Nb([
      ...a.inputs,
      ...a.outputs
    ]);
    const s = [];
    for (const a of e.values()) (a.inputs.length === 0 || a.outputs.length === 0) && s.push(a.name);
    if (s.length > 0) return {
      cls: "unbalanced",
      summary: `${s.length} item(s) have unbalanced ports (missing input or output): ${s.slice(0, 3).join(", ")}${s.length > 3 ? "\u2026" : ""}. The SAT solver would normally filter these out before solving.`,
      items: e
    };
    const i = [
      ...e.values()
    ];
    if (i.length === 1) {
      const a = i[0];
      return a.inputs.length === 1 && a.outputs.length === 1 ? {
        cls: "single-item",
        summary: `Single-item passthrough: ${a.name} (${a.axis}). 1 input \u2192 1 output. Trivial routing, no crossing needed.`,
        items: e
      } : {
        cls: "same-direction",
        summary: `Same-direction, single item: ${a.name} with ${a.inputs.length} inputs / ${a.outputs.length} outputs on the ${a.axis} axis. Could be a merge point.`,
        items: e
      };
    }
    if (i.length === 2) {
      const [a, l] = i, c = [
        a.axis,
        l.axis
      ];
      if (a.inputs.length === 1 && a.outputs.length === 1 && l.inputs.length === 1 && l.outputs.length === 1 && c.includes("horizontal") && c.includes("vertical")) {
        const u = a.axis === "horizontal" ? a : l, p = a.axis === "vertical" ? a : l;
        return {
          cls: "perpendicular",
          summary: `Perpendicular crossing (T1): ${u.name} (horizontal) crosses ${p.name} (vertical). A UG bridge in 3 tiles would route this deterministically \u2014 no SAT needed.`,
          items: e
        };
      }
      if (a.axis === l.axis) return {
        cls: "same-direction",
        summary: `Same-direction overlap (T3): ${a.name} and ${l.name} both on ${a.axis} axis. One needs to go underground past the other.`,
        items: e
      };
      const h = i.filter((u) => u.axis === "horizontal"), d = i.filter((u) => u.axis === "vertical");
      return h.length === 1 && d.length === 1 ? {
        cls: "complex",
        summary: `2-item crossing with multiple ports per item \u2014 the horizontal spec has ${h[0].inputs.length}/${h[0].outputs.length} in/out, the vertical has ${d[0].inputs.length}/${d[0].outputs.length}. Not a simple T1 crossing.`,
        items: e
      } : {
        cls: "complex",
        summary: "2-item mixed-axis region that doesn't match T1 or T3.",
        items: e
      };
    }
    const r = i.filter((a) => a.axis === "horizontal"), o = i.filter((a) => a.axis === "vertical");
    if (r.length === 1 && o.length === i.length - 1) {
      const a = r[0];
      if (o.every((c) => c.inputs.length === 1 && c.outputs.length === 1) && a.inputs.length === 1 && a.outputs.length === 1) return {
        cls: "corridor",
        summary: `Corridor run (T2): horizontal ${a.name} crosses ${o.length} vertical trunks. A single long UG bridge would route this in ~${o.length + 1} tiles.`,
        items: e
      };
    }
    return o.length === 1 && r.length === i.length - 1 ? {
      cls: "corridor",
      summary: `Corridor run (T2, rotated): vertical ${o[0].name} crosses ${r.length} horizontal specs.`,
      items: e
    } : {
      cls: "complex",
      summary: `Multi-path cluster (T4): ${i.length} items (${r.length} horizontal, ${o.length} vertical). No simple template matches \u2014 this is SAT territory.`,
      items: e
    };
  }
  function Gb(n) {
    switch (n) {
      case "corridor_template":
        return 4029365;
      case "junction_template":
        return 4892271;
      case "crossing_zone":
        return 3842122;
      case "unresolved":
        return 13647936;
    }
  }
  function zb(n) {
    switch (n) {
      case "perpendicular":
        return 4892271;
      case "corridor":
        return 4029365;
      case "same-direction":
        return 13672512;
      case "single-item":
        return 7385312;
      case "complex":
        return 13647936;
      case "unbalanced":
        return 11579568;
      case "no-ports":
        return 5263440;
      case "unknown":
        return 16711935;
    }
  }
  const Db = 0.35;
  function Hb(n, t, e, s, i) {
    const r = k * 0.45;
    n.setStrokeStyle({
      width: 3,
      color: i,
      alpha: Db
    });
    let o = 0, a = -1;
    switch (s) {
      case "East":
        o = 1, a = 0;
        break;
      case "South":
        o = 0, a = 1;
        break;
      case "West":
        o = -1, a = 0;
        break;
    }
    const l = t + o * r, c = e + a * r, h = t - o * r, d = e - a * r;
    n.moveTo(h, d).lineTo(l, c).stroke();
    const u = r * 0.55, p = -a * u, f = o * u;
    n.moveTo(l - o * u + p, c - a * u + f).lineTo(l, c).lineTo(l - o * u - p, c - a * u - f).stroke();
  }
  function Dr(n) {
    return [
      n.point.x,
      n.point.y
    ];
  }
  function Ub(n, t, e, s, i, r, o = 2, a = 6, l = 4, c = 0.9) {
    const h = s - t, d = i - e, u = Math.hypot(h, d);
    if (u < 0.5) return;
    const p = h / u, f = d / u;
    n.setStrokeStyle({
      width: o,
      color: r,
      alpha: c
    });
    let g = 0;
    for (; g < u; ) {
      const m = g, y = Math.min(g + a, u);
      n.moveTo(t + p * m, e + f * m).lineTo(t + p * y, e + f * y).stroke(), g = y + l;
    }
  }
  function jb(n) {
    const t = /* @__PURE__ */ new Map();
    for (const s of n) {
      const i = s.item ?? "?";
      let r = t.get(i);
      r || (r = {
        inputs: [],
        outputs: []
      }, t.set(i, r)), s.io === "Input" ? r.inputs.push(s) : r.outputs.push(s);
    }
    const e = [];
    for (const [s, { inputs: i, outputs: r }] of t) {
      const o = Math.min(i.length, r.length);
      for (let a = 0; a < o; a++) e.push({
        item: s,
        inPort: i[a],
        outPort: r[a]
      });
    }
    return e;
  }
  function Vb(n) {
    const t = new zt(), e = n.regions ?? [], s = [];
    if (e.length === 0) return {
      layer: t,
      items: s,
      hitTest: () => null
    };
    for (const r of e) {
      const o = Wb(r), a = Gb(r.kind), l = zb(o.cls), c = r.x * k, h = r.y * k, d = r.width * k, u = r.height * k;
      s.push({
        region: r,
        classification: o,
        bboxPixels: {
          x: c,
          y: h,
          w: d,
          h: u
        }
      });
      const p = new ut(), f = r.kind === "crossing_zone" ? 0.06 : 0.14;
      p.rect(c, h, d, u).fill({
        color: a,
        alpha: f
      }), p.setStrokeStyle({
        width: 1,
        color: 0,
        alpha: 0.55
      }), p.rect(c - 1, h - 1, d + 2, u + 2).stroke(), p.setStrokeStyle({
        width: 2,
        color: l,
        alpha: 0.85
      }), p.rect(c, h, d, u).stroke(), t.addChild(p);
      const g = r.ports ?? [], m = jb(g);
      for (const { item: y, inPort: b, outPort: x } of m) {
        const [_, v] = Dr(b), [w, C] = Dr(x), E = _ * k + k / 2, P = v * k + k / 2, T = w * k + k / 2, A = C * k + k / 2, H = Sn(y), j = new ut();
        Ub(j, E, P, T, A, H), t.addChild(j);
      }
      for (const y of g) {
        const [b, x] = Dr(y), _ = b * k + k / 2, v = x * k + k / 2, w = new ut(), C = y.item ? Sn(y.item) : 8947848;
        Hb(w, _, v, y.point.direction, C), t.addChild(w);
      }
    }
    return {
      layer: t,
      items: s,
      hitTest: (r, o) => {
        let a = null, l = 1 / 0;
        for (const c of s) {
          const h = c.bboxPixels;
          if (r >= h.x && r < h.x + h.w && o >= h.y && o < h.y + h.h) {
            const d = h.w * h.h;
            d < l && (l = d, a = c);
          }
        }
        return a;
      }
    };
  }
  const Yb = /* @__PURE__ */ new Set([
    "JunctionGrowthStarted",
    "JunctionGrowthIteration",
    "JunctionStrategyAttempt",
    "SatInvocation",
    "JunctionSolved",
    "JunctionGrowthCapped",
    "RegionWalkerVeto"
  ]);
  function Xb(n) {
    return Yb.has(n.phase);
  }
  function qb(n) {
    const t = n.data;
    return typeof t.seed_x == "number" && typeof t.seed_y == "number" ? [
      t.seed_x,
      t.seed_y
    ] : [
      t.tile_x,
      t.tile_y
    ];
  }
  function Kb(n, t) {
    return `${n},${t}`;
  }
  function Jb(n) {
    const t = /* @__PURE__ */ new Map(), e = (a, l) => {
      const c = Kb(a, l);
      let h = t.get(c);
      return h || (h = {
        seed: {
          x: a,
          y: l
        },
        participating: [],
        nearbyStamped: [],
        iters: /* @__PURE__ */ new Map(),
        iterOrder: [],
        outcome: {
          kind: "Open"
        },
        order: t.size
      }, t.set(c, h)), h;
    }, s = (a, l) => `${a}|${l}`, i = (a, l, c) => {
      const h = s(l, c);
      let d = a.iters.get(h);
      return d || (d = {
        iter: l,
        variant: c,
        bbox: {
          x: 0,
          y: 0,
          w: 0,
          h: 0
        },
        tiles: [],
        forbidden: [],
        boundaries: [],
        participating: [],
        encountered: [],
        attempts: [],
        sat: null,
        veto: null
      }, a.iters.set(h, d), a.iterOrder.push(h)), d;
    };
    for (const a of n) {
      if (!Xb(a)) continue;
      const [l, c] = qb(a), h = e(l, c);
      switch (a.phase) {
        case "JunctionGrowthStarted": {
          h.participating = a.data.participating, h.nearbyStamped = a.data.nearby_stamped;
          break;
        }
        case "JunctionGrowthIteration": {
          const d = i(h, a.data.iter, a.data.variant);
          d.bbox = {
            x: a.data.bbox_x,
            y: a.data.bbox_y,
            w: a.data.bbox_w,
            h: a.data.bbox_h
          }, d.tiles = a.data.tiles, d.forbidden = a.data.forbidden_tiles, d.boundaries = a.data.boundaries, d.participating = a.data.participating, d.encountered = a.data.encountered;
          break;
        }
        case "JunctionStrategyAttempt": {
          i(h, a.data.iter, a.data.variant).attempts.push({
            strategy: a.data.strategy,
            outcome: a.data.outcome,
            detail: a.data.detail,
            elapsedUs: a.data.elapsed_us
          });
          break;
        }
        case "SatInvocation": {
          const d = i(h, a.data.iter, a.data.variant), { seed_x: u, seed_y: p, iter: f, variant: g, ...m } = a.data;
          d.sat = m;
          break;
        }
        case "RegionWalkerVeto": {
          const d = i(h, a.data.growth_iter, a.data.variant), { tile_x: u, tile_y: p, growth_iter: f, variant: g, ...m } = a.data;
          d.veto = m;
          break;
        }
        case "JunctionSolved": {
          h.outcome = {
            kind: "Solved",
            strategy: a.data.strategy,
            growthIter: a.data.growth_iter,
            regionTiles: a.data.region_tiles
          };
          break;
        }
        case "JunctionGrowthCapped": {
          h.outcome = {
            kind: "Capped",
            iters: a.data.iters,
            regionTiles: a.data.region_tiles,
            reason: a.data.reason
          };
          break;
        }
      }
    }
    const r = [], o = Array.from(t.values()).sort((a, l) => a.order - l.order);
    for (const a of o) {
      const l = a.iterOrder.map((h) => a.iters.get(h));
      let c = Math.max(0, l.length - 1);
      if (a.outcome.kind === "Solved") {
        const h = a.outcome.growthIter, d = l.findIndex((u) => u.iter === h && u.attempts.some((p) => p.outcome === "Solved"));
        if (d >= 0) c = d;
        else {
          const u = l.findIndex((p) => p.iter === h);
          u >= 0 && (c = u);
        }
      }
      r.push({
        seed: a.seed,
        participating: a.participating,
        nearbyStamped: a.nearbyStamped,
        iterations: l,
        outcome: a.outcome,
        defaultIterIndex: c
      });
    }
    return r;
  }
  function oc(n) {
    return n.iterations.length === 0 ? null : n.iterations[n.defaultIterIndex] ?? n.iterations[n.iterations.length - 1];
  }
  function Zb(n) {
    return n ? `${n.entity_name}@(${n.entity_x},${n.entity_y}) ${n.direction}` : "";
  }
  function Qb(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function t_(n, t, e) {
    const s = t.x - e, i = t.x + t.w + e, r = t.y - e, o = t.y + t.h + e, a = [];
    for (const l of n) {
      if (l.phase !== "GhostSpecRouted") continue;
      const c = l.data, h = c.tiles;
      if (!h || h.length === 0) continue;
      let d = false;
      for (const [u, p] of h) if (u >= s && u < i && p >= r && p < o) {
        d = true;
        break;
      }
      d && a.push({
        item: Qb(c.spec_key),
        specKey: c.spec_key,
        tiles: h
      });
    }
    return a;
  }
  const ac = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, e_ = new Le({
    fontFamily: "monospace",
    fontSize: 10,
    fill: 16777215,
    dropShadow: {
      color: 0,
      distance: 1,
      blur: 2,
      alpha: 0.8
    }
  }), n_ = new Le({
    fontFamily: "monospace",
    fontSize: 9,
    fill: 16777215,
    dropShadow: {
      color: 0,
      distance: 1,
      blur: 2,
      alpha: 0.9
    }
  });
  function s_(n) {
    const t = new zt(), e = [], s = [];
    for (const o of n) {
      if (o.outcome.kind !== "Solved") continue;
      const a = oc(o);
      if (!a) continue;
      const l = a.bbox;
      l.w <= 0 || l.h <= 0 || s.push({
        x: l.x,
        y: l.y,
        w: l.w,
        h: l.h
      });
    }
    const i = (o, a) => {
      for (const l of s) if (o >= l.x && o < l.x + l.w && a >= l.y && a < l.y + l.h) return true;
      return false;
    };
    for (const o of n) {
      const a = oc(o);
      if (!a) continue;
      const l = a.bbox;
      if (l.w <= 0 || l.h <= 0 || o.outcome.kind === "Capped" && i(o.seed.x, o.seed.y)) continue;
      const c = l.x * k, h = l.y * k, d = l.w * k, u = l.h * k, p = ac[o.outcome.kind] ?? ac.Open, f = new ut();
      f.rect(c, h, d, u).fill({
        color: p,
        alpha: 0.14
      }), f.setStrokeStyle({
        width: 1,
        color: 0,
        alpha: 0.55
      }), f.rect(c - 1, h - 1, d + 2, u + 2).stroke(), f.setStrokeStyle({
        width: 2,
        color: p,
        alpha: 0.85
      }), f.rect(c, h, d, u).stroke(), t.addChild(f);
      const g = new Ge({
        text: `Junction (${o.seed.x},${o.seed.y})`,
        style: e_
      });
      g.x = c + 3, g.y = h + 2, t.addChild(g);
      const m = new Ge({
        text: i_(o),
        style: n_
      });
      m.x = c + 3, m.y = h + u - m.height - 2, t.addChild(m), e.push({
        cluster: o,
        pxX: c,
        pxY: h,
        pxW: d,
        pxH: u
      });
    }
    return {
      layer: t,
      hitTest: (o, a) => {
        let l = null, c = Number.POSITIVE_INFINITY;
        for (const h of e) {
          if (o < h.pxX || a < h.pxY || o >= h.pxX + h.pxW || a >= h.pxY + h.pxH) continue;
          const d = h.pxW * h.pxH;
          d < c && (l = h.cluster, c = d);
        }
        return l;
      }
    };
  }
  function i_(n) {
    const t = n.iterations.length;
    switch (n.outcome.kind) {
      case "Solved": {
        const e = n.iterations.filter((i) => i.veto !== null).length, s = e > 0 ? ` \xB7 ${e} veto${e === 1 ? "" : "es"}` : "";
        return `Solved @ iter ${n.outcome.growthIter} \xB7 ${t} iter${t === 1 ? "" : "s"}${s}`;
      }
      case "Capped":
        return `Capped \xB7 ${n.outcome.iters} iter${n.outcome.iters === 1 ? "" : "s"}`;
      case "Open":
        return `Open \xB7 ${t} iter${t === 1 ? "" : "s"}`;
    }
  }
  const lc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, r_ = 0.04, o_ = 9060416, a_ = 0.55, l_ = 4243680, c_ = 5592405, h_ = 0.55, d_ = 16777215, cc = 0.85, u_ = 16777215, hc = new Le({
    fontFamily: "monospace",
    fontSize: 7,
    fontWeight: "700",
    fill: d_
  }), dc = /* @__PURE__ */ new Map();
  function p_(n) {
    let t = dc.get(n);
    if (!t) {
      const s = `/fucktorio/pr-238/icons/${n}.png`;
      t = ze.load(s).catch(() => null), dc.set(n, t);
    }
    return t;
  }
  function f_() {
    const n = new zt();
    n.label = "sat-zone-overlay";
    let t = 0;
    function e() {
      for (; n.children.length > 0; ) {
        const r = n.children[0];
        n.removeChild(r), r.destroy({
          children: true
        });
      }
    }
    function s(r) {
      var _a2;
      t += 1;
      const o = t;
      if (e(), !r) return;
      const { cluster: a, iter: l } = r, c = l.bbox, h = lc[a.outcome.kind] ?? lc.Open, d = new ut();
      d.rect(c.x * k, c.y * k, c.w * k, c.h * k).fill({
        color: h,
        alpha: r_
      }), n.addChild(d);
      const u = m_(c.x * k, c.y * k, c.w * k, c.h * k, {
        dashLen: k * 0.45,
        gapLen: k * 0.25,
        width: 3,
        color: h,
        alpha: 0.95
      });
      n.addChild(u);
      const p = new ut();
      p.setStrokeStyle({
        width: 1,
        color: o_,
        alpha: a_
      });
      for (const [m, y] of l.forbidden) g_(p, m * k, y * k, k);
      n.addChild(p);
      const f = new ut();
      f.circle((a.seed.x + 0.5) * k, (a.seed.y + 0.5) * k, k * 0.42).stroke({
        width: 3,
        color: l_,
        alpha: 0.95
      }), n.addChild(f);
      const g = ((_a2 = l.sat) == null ? void 0 : _a2.boundaries) ?? l.boundaries;
      for (const m of g) x_(n, m, o, () => t);
    }
    function i() {
      e(), n.destroy({
        children: true
      });
    }
    return {
      layer: n,
      update: s,
      destroy: i
    };
  }
  function m_(n, t, e, s, i) {
    const r = new ut();
    r.setStrokeStyle({
      width: i.width,
      color: i.color,
      alpha: i.alpha
    });
    const o = [
      [
        n,
        t,
        n + e,
        t
      ],
      [
        n + e,
        t,
        n + e,
        t + s
      ],
      [
        n + e,
        t + s,
        n,
        t + s
      ],
      [
        n,
        t + s,
        n,
        t
      ]
    ];
    for (const [a, l, c, h] of o) {
      const d = c - a, u = h - l, p = Math.hypot(d, u), f = d / p, g = u / p, m = i.dashLen + i.gapLen;
      for (let y = 0; y < p; y += m) {
        const b = Math.min(y + i.dashLen, p);
        r.moveTo(a + f * y, l + g * y).lineTo(a + f * b, l + g * b).stroke();
      }
    }
    return r;
  }
  function g_(n, t, e, s) {
    const i = s;
    n.moveTo(t, e + i).lineTo(t + i, e).stroke(), n.moveTo(t + i / 3, e + i).lineTo(t + i, e + 2 * i / 3).stroke(), n.moveTo(t, e + 2 * i / 3).lineTo(t + 2 * i / 3, e).stroke();
  }
  function y_(n, t) {
    return (n ? {
      North: "bottom",
      East: "left",
      South: "top",
      West: "right"
    } : {
      North: "top",
      East: "right",
      South: "bottom",
      West: "left"
    })[t] ?? "top";
  }
  function x_(n, t, e, s) {
    const i = t.x * k, r = t.y * k, o = k / 3, a = y_(t.is_input, t.direction), l = a === "top" || a === "bottom";
    let c = i, h = r, d = k, u = k;
    a === "top" ? u = o : a === "bottom" ? (h = r + k - o, u = o) : (a === "left" || (c = i + k - o), d = o);
    const p = new ut();
    p.rect(c, h, d, u).fill({
      color: c_,
      alpha: h_
    }), t.interior && (p.setStrokeStyle({
      width: 1,
      color: u_,
      alpha: 0.5
    }), p.rect(c, h, d, u).stroke()), n.addChild(p);
    const f = b_(t.direction), [g, m, y, b] = l ? [
      c + d / 6,
      h + u / 2,
      c + d * 5 / 6,
      h + u / 2
    ] : [
      c + d / 2,
      h + u / 6,
      c + d / 2,
      h + u * 5 / 6
    ], x = new Ge({
      text: f,
      style: hc
    });
    x.anchor.set(0.5), x.x = g, x.y = m, x.alpha = cc, n.addChild(x);
    const _ = new Ge({
      text: f,
      style: hc
    });
    _.anchor.set(0.5), _.x = y, _.y = b, _.alpha = cc, n.addChild(_);
    const v = c + d / 2, w = h + u / 2;
    p_(t.item).then((C) => {
      if (e !== s() || !C) return;
      const E = new De(C);
      E.anchor.set(0.5), E.x = v, E.y = w;
      const P = o * 0.95;
      E.width = P, E.height = P, n.addChild(E);
    });
  }
  function b_(n) {
    switch (n) {
      case "North":
        return "\u25B2";
      case "East":
        return "\u25B6";
      case "South":
        return "\u25BC";
      case "West":
        return "\u25C0";
      default:
        return "?";
    }
  }
  const __ = 4247776, w_ = 0.18;
  function v_(n) {
    if (!n || n.length === 0) return null;
    const t = /* @__PURE__ */ new Set();
    for (const i of n) if (i.phase === "GhostSpecRouted") for (const [r, o] of i.data.tiles) t.add(`${r},${o}`);
    if (t.size === 0) return null;
    const e = new zt();
    e.label = "ghost-tiles-overlay";
    const s = new ut();
    for (const i of t) {
      const [r, o] = i.split(","), a = Number(r), l = Number(o);
      s.rect(a * k, l * k, k, k).fill({
        color: __,
        alpha: w_
      });
    }
    return e.addChild(s), e;
  }
  function C_(n, t, e) {
    const s = document.createElement("div");
    s.className = "jd-inline";
    const i = document.createElement("div");
    i.className = "jd-inline-head";
    const r = document.createElement("span");
    r.className = "jd-title";
    const o = document.createElement("span");
    o.className = "jd-status-pill";
    const a = document.createElement("button");
    a.className = "jd-inline-btn jd-inline-details-btn", a.textContent = "\u2139", a.title = "Show details (i)";
    const l = document.createElement("button");
    l.className = "jd-inline-btn jd-inline-copy-btn", l.textContent = "\u29C9", l.title = "Copy debug dump to clipboard";
    const c = document.createElement("button");
    c.className = "jd-inline-btn jd-inline-fixture-btn", c.textContent = "\u26AB", c.title = "Copy as SAT-fixture JSON";
    const h = document.createElement("button");
    h.className = "jd-inline-btn jd-inline-edit-btn", h.textContent = "\u270E", h.title = "Edit this SAT zone (Phase F)";
    const d = document.createElement("span");
    d.className = "jd-close", d.textContent = "\xD7", d.title = "Deselect (Esc)", i.append(r, o, l, c, h, a, d);
    const u = document.createElement("div");
    u.className = "jd-stepper";
    const p = document.createElement("button");
    p.className = "jd-step-btn", p.textContent = "\u25C0", p.title = "previous iteration (\u2190)";
    const f = document.createElement("span");
    f.className = "jd-step-label";
    const g = document.createElement("button");
    g.className = "jd-step-btn", g.textContent = "\u25B6", g.title = "next iteration (\u2192)";
    const m = document.createElement("button");
    m.className = "jd-step-btn jd-terminal-btn", m.textContent = "\u21BA", m.title = "jump to default (terminal) iteration", u.append(p, f, g, m);
    const y = document.createElement("div");
    y.className = "jd-inline-summary", s.append(i, u, y), n.append(s);
    const b = document.createElement("div");
    b.className = "jd-modal-backdrop";
    const x = document.createElement("div");
    x.className = "jd-modal";
    const _ = document.createElement("div");
    _.className = "jd-titlebar";
    const v = document.createElement("span");
    v.className = "jd-title", v.textContent = "Junction details";
    const w = document.createElement("span");
    w.className = "jd-status-pill";
    const C = document.createElement("span");
    C.className = "jd-close", C.textContent = "\xD7", C.title = "Close details (Esc)", _.append(v, w, C);
    const E = document.createElement("div");
    E.className = "jd-detail";
    const P = document.createElement("div");
    P.className = "jd-footer", P.textContent = "Esc close \xB7 \u2190/\u2192 step all \xB7 w/s iter \xB7 a/d variant \xB7 Home/End first/last", x.append(_, E, P), n.append(b, x);
    let T = null, A = 0, H = null, j = false;
    function D(I, G) {
      T = I, H = G ?? null, A = I.defaultIterIndex, s.classList.add("jd-open"), K(), Y(), R();
    }
    function N() {
      T && (W(), T = null, H = null, s.classList.remove("jd-open"), e.onChange(null));
    }
    function $() {
      !T || j || (j = true, b.classList.add("jd-open"), x.classList.add("jd-open"), nt());
    }
    function W() {
      j && (j = false, b.classList.remove("jd-open"), x.classList.remove("jd-open"));
    }
    function X() {
      j ? W() : $();
    }
    function U() {
      return T !== null;
    }
    function Z(I) {
      if (!T) return;
      const G = Math.max(0, Math.min(T.iterations.length - 1, I));
      G !== A && (A = G, K(), Y(), R());
    }
    function R() {
      if (!T) return;
      const I = T.iterations[A];
      if (!I) return;
      const G = t.toScreen(I.bbox.x * k, I.bbox.y * k), V = t.toScreen((I.bbox.x + I.bbox.w) * k, (I.bbox.y + I.bbox.h) * k), J = n.getBoundingClientRect();
      if (!(V.x < 0 || G.x > J.width || V.y < 0 || G.y > J.height)) return;
      const bt = (I.bbox.x + I.bbox.w / 2) * k, wt = (I.bbox.y + I.bbox.h / 2) * k;
      t.moveCenter(bt, wt);
    }
    function B() {
      const I = /* @__PURE__ */ new Map(), G = T;
      if (!G) return I;
      for (let V = 0; V < G.iterations.length; V++) {
        const J = G.iterations[V], xt = I.get(J.iter) ?? [];
        xt.push(V), I.set(J.iter, xt);
      }
      return I;
    }
    function O(I) {
      if (!T) return;
      const G = B(), V = Array.from(G.keys()).sort((kt, Ft) => kt - Ft), J = T.iterations[A].iter, xt = V.indexOf(J), bt = V[Math.max(0, Math.min(V.length - 1, xt + I))], wt = G.get(bt) ?? [], Dt = wt.find((kt) => T.iterations[kt].variant === "");
      Z(Dt ?? wt[0] ?? A);
    }
    function z(I) {
      if (!T) return;
      const G = B(), V = T.iterations[A].iter, J = G.get(V) ?? [];
      if (J.length <= 1) return;
      const bt = (J.indexOf(A) + I + J.length) % J.length;
      Z(J[bt]);
    }
    function Y() {
      if (!T) return;
      const I = T.iterations[A];
      if (!I) return;
      const G = n.getBoundingClientRect(), V = s.offsetWidth || 200, J = s.offsetHeight || 70, xt = (I.bbox.x + I.bbox.w) * k, bt = (I.bbox.y + I.bbox.h) * k, wt = I.bbox.y * k, Dt = t.toScreen(xt, bt), kt = t.toScreen(xt, wt);
      let Ft = Dt.x - V, ct = Dt.y;
      ct + J > G.height - 4 && (ct = kt.y - J), Ft = Math.max(4, Math.min(Ft, G.width - V - 4)), ct = Math.max(4, Math.min(ct, G.height - J - 4)), s.style.left = `${Ft}px`, s.style.top = `${ct}px`;
    }
    t.on("moved", Y), t.on("zoomed", Y), window.addEventListener("resize", Y);
    function K() {
      if (!T) return;
      const I = T, G = I.iterations[A];
      r.textContent = `Junction (${I.seed.x},${I.seed.y})`, o.className = `jd-status-pill jd-${I.outcome.kind.toLowerCase()}`, o.textContent = uc(I);
      const V = I.iterations.length, J = G && G.variant ? ` \xB7 ${G.variant}` : "";
      f.textContent = `iter ${G ? G.iter : "-"}${J} \xB7 ${A + 1}/${V}`, p.disabled = A <= 0, g.disabled = A >= V - 1, m.disabled = A === I.defaultIterIndex, y.innerHTML = "";
      for (const xt of k_(I, G)) {
        const bt = document.createElement("div");
        bt.className = `jd-inline-summary-row jd-inline-summary-row--${xt.tone}`, bt.textContent = xt.text, y.appendChild(bt);
      }
      j && nt(), G && e.onChange({
        cluster: I,
        iter: G,
        trace: H
      });
    }
    function nt() {
      if (!T) return;
      const I = T, G = I.iterations[A];
      w.className = `jd-status-pill jd-${I.outcome.kind.toLowerCase()}`, w.textContent = uc(I), v.textContent = `Junction (${I.seed.x},${I.seed.y})`, A_(E, I, G);
    }
    d.addEventListener("click", N), a.addEventListener("click", X), l.addEventListener("click", gt), c.addEventListener("click", () => {
      if (!T) return;
      const I = Io(T, T.iterations[A], H);
      B_(c, I);
    }), h.addEventListener("click", () => {
      if (!T || !e.onEditRequested) return;
      const I = T.iterations[A];
      I && e.onEditRequested({
        cluster: T,
        iter: I,
        trace: H
      });
    }), C.addEventListener("click", W), b.addEventListener("click", W);
    function ot() {
      if (!T) return null;
      const I = T.iterations[A];
      return I ? {
        cluster: T,
        iter: I,
        trace: H
      } : null;
    }
    function ht(I) {
      s.classList.toggle("jd-edit-mode", I), c.disabled = I, l.disabled = I, h.disabled = I;
    }
    function gt() {
      var _a2;
      if (!T) return;
      const I = S_(T, A), G = JSON.stringify(I, (J, xt) => typeof xt == "bigint" ? String(xt) : xt, 2), V = (J) => {
        const xt = l.textContent;
        l.textContent = J ? "\u2713" : "!", l.classList.add("jd-inline-btn--flash"), window.setTimeout(() => {
          l.textContent = xt, l.classList.remove("jd-inline-btn--flash");
        }, 900);
      };
      if ((_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText) navigator.clipboard.writeText(G).then(() => V(true), () => V(false));
      else {
        const J = document.createElement("textarea");
        J.value = G, J.style.position = "fixed", J.style.opacity = "0", document.body.appendChild(J), J.select();
        try {
          document.execCommand("copy"), V(true);
        } catch {
          V(false);
        }
        document.body.removeChild(J);
      }
    }
    return p.addEventListener("click", () => Z(A - 1)), g.addEventListener("click", () => Z(A + 1)), m.addEventListener("click", () => {
      T && Z(T.defaultIterIndex);
    }), document.addEventListener("keydown", (I) => {
      var _a2, _b2;
      if (!U()) return;
      const G = (_b2 = (_a2 = I.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (G === "INPUT" || G === "TEXTAREA" || G === "SELECT") return;
      const V = I.key, J = () => {
        I.stopImmediatePropagation(), I.preventDefault();
      };
      V === "Escape" ? (j ? W() : N(), J()) : V === "ArrowLeft" ? (Z(A - 1), J()) : V === "ArrowRight" ? (Z(A + 1), J()) : V === "Home" ? (Z(0), J()) : V === "End" && T ? (Z(T.iterations.length - 1), J()) : V === "w" || V === "W" ? (O(-1), J()) : V === "s" || V === "S" ? (O(1), J()) : V === "a" || V === "A" ? (z(-1), J()) : V === "d" || V === "D" ? (z(1), J()) : (V === "i" || V === "I") && (X(), J());
    }, {
      capture: true
    }), {
      open: D,
      close: N,
      isOpen: U,
      inlineEl: s,
      getSelection: ot,
      setEditMode: ht
    };
  }
  function S_(n, t) {
    const e = (r) => r ? `${r.entity_name}@(${r.entity_x},${r.entity_y}) ${r.direction}` : void 0, s = (r) => {
      const o = {
        x: r.x,
        y: r.y,
        dir: r.direction,
        item: r.item,
        in: r.is_input
      };
      r.interior && (o.interior = true), r.spec_key && (o.spec = r.spec_key);
      const a = e(r.external_feeder);
      return a && (o.feeder = a), o;
    }, i = n.iterations.map((r, o) => {
      var _a2;
      const a = ((_a2 = r.sat) == null ? void 0 : _a2.boundaries) ?? r.boundaries, l = {
        idx: o,
        iter: r.iter,
        bbox: r.bbox,
        boundaries: a.map(s),
        attempts: r.attempts.map((c) => ({
          strategy: c.strategy,
          outcome: c.outcome,
          ...c.detail ? {
            detail: c.detail
          } : {}
        }))
      };
      return r.variant && (l.variant = r.variant), r.veto && (l.veto = r.veto), r.sat && (l.sat = {
        satisfied: r.sat.satisfied,
        vars: r.sat.variables,
        clauses: r.sat.clauses,
        solveUs: r.sat.solve_time_us
      }), l;
    });
    return {
      url: window.location.href,
      ts: (/* @__PURE__ */ new Date()).toISOString(),
      currentIterIndex: t,
      seed: n.seed,
      outcome: n.outcome,
      participating: n.participating.map((r) => ({
        key: r.key,
        item: r.item,
        start: [
          r.initial_tile_x,
          r.initial_tile_y
        ]
      })),
      iterations: i
    };
  }
  function uc(n) {
    switch (n.outcome.kind) {
      case "Solved":
        return `Solved \xB7 ${n.outcome.regionTiles}t`;
      case "Capped":
        return `Capped \xB7 ${n.outcome.iters} iter`;
      case "Open":
        return "Open";
    }
  }
  function k_(n, t) {
    if (t) {
      if (t.veto) return [
        {
          text: `veto \xB7 ${ki(t.veto.broken_segment, 22)} @ (${t.veto.break_tile_x},${t.veto.break_tile_y})`,
          tone: "warn"
        }
      ];
      const e = t.attempts.find((i) => i.outcome === "Solved");
      if (e) return [
        {
          text: `${e.strategy} ok \xB7 ${e.elapsedUs}\xB5s`,
          tone: "ok"
        }
      ];
      const s = [
        ...t.attempts
      ].reverse().find((i) => i.outcome !== "Solved");
      if (s) {
        const i = s.detail ? ` \xB7 ${ki(s.detail, 28)}` : "";
        return [
          {
            text: `${s.strategy} \u2192 ${ki(s.outcome, 12)}${i}`,
            tone: "fail"
          }
        ];
      }
    }
    switch (n.outcome.kind) {
      case "Solved":
        return [
          {
            text: `solved @ iter ${n.outcome.growthIter}`,
            tone: "ok"
          }
        ];
      case "Capped":
        return [
          {
            text: `cap: ${ki(n.outcome.reason, 32)}`,
            tone: "fail"
          }
        ];
      case "Open":
        return [
          {
            text: "open \u2014 never terminated",
            tone: "warn"
          }
        ];
    }
  }
  function ki(n, t) {
    return n.length <= t ? n : `${n.slice(0, t - 1)}\u2026`;
  }
  function Zd(n) {
    var _a2;
    return ((_a2 = n.sat) == null ? void 0 : _a2.boundaries) ?? n.boundaries;
  }
  function T_(n) {
    switch (n) {
      case "North":
        return "\u2191";
      case "East":
        return "\u2192";
      case "South":
        return "\u2193";
      case "West":
        return "\u2190";
      default:
        return "?";
    }
  }
  function A_(n, t, e) {
    n.innerHTML = "", n.appendChild(E_(t)), n.appendChild(M_(t, e)), e && (n.appendChild(P_(e)), n.appendChild(I_(e)), n.appendChild(R_(e)), e.veto && n.appendChild(L_(e))), t.nearbyStamped.length > 0 && n.appendChild($_(t));
  }
  function Xn(n, t = true) {
    const e = document.createElement("details");
    t && (e.open = true);
    const s = document.createElement("summary");
    s.textContent = n;
    const i = document.createElement("div");
    return i.className = "jd-sec-body", e.append(s, i), {
      details: e,
      bodyEl: i
    };
  }
  function E_(n) {
    const { details: t, bodyEl: e } = Xn("Summary"), s = document.createElement("div");
    s.className = "jd-kv-grid";
    const i = [
      [
        "seed",
        `(${n.seed.x}, ${n.seed.y})`
      ],
      [
        "iterations",
        String(n.iterations.length)
      ],
      [
        "outcome",
        n.outcome.kind
      ]
    ];
    n.outcome.kind === "Solved" ? i.push([
      "strategy",
      n.outcome.strategy
    ], [
      "solved at iter",
      String(n.outcome.growthIter)
    ], [
      "region tiles",
      String(n.outcome.regionTiles)
    ]) : n.outcome.kind === "Capped" && i.push([
      "iters attempted",
      String(n.outcome.iters)
    ], [
      "region tiles",
      String(n.outcome.regionTiles)
    ], [
      "reason",
      n.outcome.reason
    ]);
    for (const [r, o] of i) {
      const a = document.createElement("span");
      a.textContent = r;
      const l = document.createElement("span");
      l.textContent = o, s.append(a, l);
    }
    return e.appendChild(s), t;
  }
  function M_(n, t) {
    const { details: e, bodyEl: s } = Xn("Participating specs");
    if (n.participating.length === 0) {
      const r = document.createElement("div");
      return r.className = "jd-row jd-row--dim", r.textContent = "(none reported)", s.appendChild(r), e;
    }
    const i = new Set((t == null ? void 0 : t.participating) ?? []);
    for (const r of n.participating) {
      const o = document.createElement("div");
      o.className = "jd-row", t && !i.has(r.key) && o.classList.add("jd-spec-drop"), o.textContent = `${r.key} \xB7 ${r.item} \xB7 start=(${r.initial_tile_x},${r.initial_tile_y}) \xB7 path_len=${r.path_len} \xB7 frontier=[${r.initial_start}..${r.initial_end}]`, s.appendChild(o);
    }
    if (t && t.encountered.length > 0) {
      const r = document.createElement("div");
      r.className = "jd-row jd-row--dim", r.textContent = `encountered (non-participating): ${t.encountered.join(", ")}`, s.appendChild(r);
    }
    return e;
  }
  function P_(n) {
    const t = Zd(n), e = n.sat ? " (as fed to SAT)" : " (spec perimeter)", { details: s, bodyEl: i } = Xn(`Boundaries${e}`);
    if (t.length === 0) {
      const r = document.createElement("div");
      return r.className = "jd-row jd-row--dim", r.textContent = "(none)", i.appendChild(r), s;
    }
    for (const r of t) {
      const o = document.createElement("div");
      o.className = "jd-row";
      const a = r.is_input ? "IN " : "OUT", l = r.interior ? " (interior)" : "", c = r.external_feeder ? ` \u2190 ${Zb(r.external_feeder)}` : "";
      o.style.color = r.is_input ? "#9f9" : "#f99";
      const h = r.spec_key ? ` \xB7 ${r.spec_key}` : "";
      o.textContent = `${a} (${r.x},${r.y}) ${T_(r.direction)} ${r.direction} \xB7 ${r.item}${l}${h}${c}`, i.appendChild(o);
    }
    return s;
  }
  function I_(n) {
    const { details: t, bodyEl: e } = Xn("Strategy attempts");
    if (n.attempts.length === 0) {
      const s = document.createElement("div");
      return s.className = "jd-row jd-row--dim", s.textContent = "(no attempts recorded)", e.appendChild(s), t;
    }
    for (const s of n.attempts) {
      const i = document.createElement("div");
      i.className = "jd-row";
      const r = s.outcome !== "Solved";
      i.classList.add(r ? "jd-row--fail" : "jd-row--pass");
      const o = s.detail ? `  ${s.detail}` : "";
      i.textContent = `${s.strategy} \u2192 ${s.outcome}${o}  \xB7 ${s.elapsedUs}\xB5s`, e.appendChild(i);
    }
    return t;
  }
  function R_(n) {
    const { details: t, bodyEl: e } = Xn("SAT", !!n.sat);
    if (!n.sat) {
      const o = document.createElement("div");
      return o.className = "jd-row jd-row--dim", o.textContent = "(SAT not invoked this iteration)", e.appendChild(o), t;
    }
    const s = n.sat, i = document.createElement("div");
    i.className = "jd-kv-grid";
    const r = [
      [
        "satisfied",
        String(s.satisfied)
      ],
      [
        "zone",
        `(${s.zone_x},${s.zone_y}) ${s.zone_w}\xD7${s.zone_h}`
      ],
      [
        "belt tier",
        s.belt_tier
      ],
      [
        "max reach",
        String(s.max_reach)
      ],
      [
        "vars",
        String(s.variables)
      ],
      [
        "clauses",
        String(s.clauses)
      ],
      [
        "solve time",
        `${s.solve_time_us}\xB5s`
      ],
      [
        "entities placed",
        String(s.entities_raw)
      ],
      [
        "forced empty",
        String(s.forced_empty.length)
      ],
      [
        "boundaries",
        String(s.boundaries.length)
      ]
    ];
    for (const [o, a] of r) {
      const l = document.createElement("span");
      l.textContent = o;
      const c = document.createElement("span");
      c.textContent = a, i.append(l, c);
    }
    return e.appendChild(i), t;
  }
  function L_(n) {
    const { details: t, bodyEl: e } = Xn("Walker veto");
    if (!n.veto) return t;
    const s = n.veto, i = document.createElement("div");
    i.className = "jd-kv-grid";
    const r = [
      [
        "strategy",
        s.strategy
      ],
      [
        "broken segment",
        s.broken_segment
      ],
      [
        "break tile",
        `(${s.break_tile_x},${s.break_tile_y})`
      ],
      [
        "break count",
        String(s.break_count)
      ]
    ];
    for (const [o, a] of r) {
      const l = document.createElement("span");
      l.textContent = o;
      const c = document.createElement("span");
      c.textContent = a, i.append(l, c);
    }
    return e.appendChild(i), t;
  }
  function $_(n) {
    const { details: t, bodyEl: e } = Xn("Nearby stamped", false);
    for (const s of n.nearbyStamped) {
      const i = document.createElement("div");
      i.className = "jd-row";
      const r = s.carries ? ` carries=${s.carries}` : "", o = s.segment_id ? ` \xB7 seg=${s.segment_id}` : "";
      i.textContent = `(${s.x},${s.y}) ${s.name} ${s.direction}${r}${o}${s.feeds_seed_area ? "  \u26A0 feeds seed" : ""}`, e.appendChild(i);
    }
    return t;
  }
  const Qd = {
    "transport-belt": 4,
    "fast-transport-belt": 6,
    "express-transport-belt": 8
  };
  function Io(n, t, e, s) {
    var _a2, _b2;
    const i = n.seed, r = (t == null ? void 0 : t.bbox) ?? {
      x: i.x,
      y: i.y,
      w: 1,
      h: 1
    }, o = (t == null ? void 0 : t.iter) ?? 0, a = ((_a2 = t == null ? void 0 : t.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", l = ((_b2 = t == null ? void 0 : t.sat) == null ? void 0 : _b2.max_reach) ?? Qd[a] ?? 4, c = Zd(t ?? {
      boundaries: [],
      sat: null
    }).map((p) => ({
      x: p.x,
      y: p.y,
      dir: p.direction,
      item: p.item,
      in: p.is_input,
      ...p.interior ? {
        interior: true
      } : {}
    })), h = t && e ? t_(e, r, 2).map((p) => ({
      item: p.item,
      spec_key: p.specKey,
      tiles: p.tiles
    })) : [], d = {
      mode: "solve"
    };
    (s == null ? void 0 : s.maxCost) !== void 0 && (d.max_cost = s.maxCost);
    const u = {
      version: 1,
      name: (s == null ? void 0 : s.name) ?? `fixture_${i.x}_${i.y}_iter${o}`,
      notes: "",
      source_url: window.location.href,
      seed: [
        i.x,
        i.y
      ],
      bbox: {
        x: r.x,
        y: r.y,
        w: r.w,
        h: r.h
      },
      forbidden: (t == null ? void 0 : t.forbidden) ?? [],
      belt_tier: a,
      max_reach: l,
      boundaries: c,
      expected: d,
      ...h.length > 0 ? {
        context: {
          ghost_paths: h
        }
      } : {},
      ...(s == null ? void 0 : s.paintedEntities) ? {
        painted: {
          entities: s.paintedEntities
        }
      } : {}
    };
    return JSON.stringify(u, null, 2);
  }
  function B_(n, t) {
    var _a2;
    const e = n.textContent ?? "", s = () => {
      n.textContent = "\u2713", n.style.color = "#9f9", setTimeout(() => {
        n.textContent = e, n.style.color = "";
      }, 1200);
    }, i = () => {
      prompt("Copy fixture JSON (Ctrl+A, Ctrl+C):", t);
    };
    ((_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText) ? navigator.clipboard.writeText(t).then(s, i) : i();
  }
  function O_(n) {
    return n.x === void 0 || n.y === void 0 || n.direction === void 0 ? null : n;
  }
  const Ti = {
    North: [
      0,
      -1
    ],
    East: [
      1,
      0
    ],
    South: [
      0,
      1
    ],
    West: [
      -1,
      0
    ]
  }, pc = {
    North: "East",
    East: "South",
    South: "West",
    West: "North"
  }, Ai = {
    "transport-belt": "transport-belt",
    "fast-transport-belt": "fast-transport-belt",
    "express-transport-belt": "express-transport-belt"
  }, fc = {
    "transport-belt": "underground-belt",
    "fast-transport-belt": "fast-underground-belt",
    "express-transport-belt": "express-underground-belt"
  };
  function F_(n) {
    const { viewport: t, canvas: e, engine: s, jd: i, satZoneOverlayLayer: r } = n;
    let o = null, a = null, l = null, c = null, h = null, d = null, u = false, p = null, f = [], g = [], m = [], y = [], b = "belt", x = "East", _ = 0, v = null, w = "idle", C = 0, E = null, P = null;
    function T(M, S) {
      return `${M},${S}`;
    }
    function A(M, S) {
      if (!p) return false;
      const L = p.bbox;
      return M >= L.x && M < L.x + L.w && S >= L.y && S < L.y + L.h;
    }
    function H(M, S) {
      return f.find((L) => L.x === M && L.y === S);
    }
    function j() {
      if (!p || p.items.length === 0) return null;
      const M = Math.max(0, Math.min(_, p.items.length - 1));
      return p.items[M] ?? null;
    }
    function D(M, S, L) {
      if (!p) return null;
      const [q, Q] = Ti[L], rt = M - q, pt = S - Q, et = H(rt, pt);
      if (et && et.direction === L && (Ai[et.name] === et.name || et.io_type === "output")) return et.carries ?? null;
      const ft = p.boundaries.find(($t) => $t.x === M && $t.y === S && $t.isInput && $t.dir === L);
      return ft ? ft.item : null;
    }
    function N(M, S) {
      return D(M.x, M.y, S) ?? j();
    }
    function $(M, S) {
      if (!p) return null;
      const [L, q] = Ti[S];
      for (let Q = 1; Q <= p.maxReach + 1; Q++) {
        const rt = M.x - L * Q, pt = M.y - q * Q, et = H(rt, pt);
        if (et && et.io_type === "input" && et.direction === S) return et.carries ?? null;
      }
      return null;
    }
    function W() {
      g.push(f.map((M) => ({
        ...M
      }))), g.length > 64 && g.shift(), m.length = 0;
    }
    function X(M, S) {
      W(), f = M, Z(), nt();
    }
    function U() {
      if (!p) return [];
      const M = Ai[p.beltTier] ?? "transport-belt", S = [];
      for (const L of p.boundaries) {
        if (!L.isInput) continue;
        const [q, Q] = Ti[L.dir];
        S.push({
          name: M,
          x: L.x - q,
          y: L.y - Q,
          direction: L.dir,
          carries: L.item
        });
      }
      return S;
    }
    function Z() {
      const M = U();
      o && zs({
        entities: f
      }, o, void 0, void 0, void 0, M), a && zs({
        entities: y
      }, a, void 0, void 0, void 0, M), R();
    }
    function R() {
      if (c) {
        c.removeChildren();
        for (const M of f) {
          const S = M.carries;
          if (!S) continue;
          const L = `/fucktorio/pr-238/icons/${S}.png`, q = ze.get(L);
          if (!q) continue;
          const Q = new De(q), rt = k * 0.55;
          Q.width = rt, Q.height = rt, Q.x = M.x * k + (k - rt) / 2, Q.y = M.y * k + (k - rt) / 2, Q.alpha = 0.85, c.addChild(Q);
        }
      }
    }
    function B(M, S) {
      l && (zs({
        entities: M
      }, l, void 0, void 0, void 0, U()), l.alpha = S ? 0.5 : 0.45, l.tint = S ? 16733525 : 16777215);
    }
    function O() {
      l && (l.removeChildren(), l.tint = 16777215);
    }
    function z(M, S = "") {
      if (w = M, !d) return;
      d.classList.remove("ok", "solving", "invalid", "idle"), d.classList.add(M === "valid" ? "ok" : M);
      const L = M === "valid" ? "\u25CF" : M === "solving" ? "\u25D4" : M === "invalid" ? "\u25CF" : "\u25CB";
      d.textContent = L;
      let q = "";
      M === "valid" ? q = P !== null ? `valid \xB7 cost ${P} / yours ${Y(f)}` : "valid" : M === "solving" ? q = "solving\u2026" : M === "invalid" ? q = "invalid" : q = "no edits yet", d.title = S ? `${q}
${S}` : q, pe();
    }
    function Y(M) {
      let S = 0;
      for (const L of M) Ai[L.name] === L.name ? S += 1 : fc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] === L.name && (S += 5);
      return S;
    }
    function K() {
      if (!p) return "no zone";
      const M = /* @__PURE__ */ new Set();
      for (const S of f) {
        const L = T(S.x, S.y);
        if (M.has(L)) return `duplicate entity at (${S.x},${S.y})`;
        if (M.add(L), !A(S.x, S.y)) return `entity at (${S.x},${S.y}) outside bbox`;
        if (p.forbidden.has(L)) return `entity at (${S.x},${S.y}) on forbidden tile`;
      }
      for (const S of f) {
        if (S.io_type !== "input") continue;
        const [L, q] = Ti[S.direction];
        let Q = false;
        for (let rt = 1; rt <= p.maxReach + 1; rt++) {
          const pt = S.x + L * rt, et = S.y + q * rt, ft = H(pt, et);
          if (ft) {
            if (ft.io_type === "output" && ft.direction === S.direction && ft.carries === S.carries) {
              Q = true;
              break;
            }
            if (ft.io_type === "input" && ft.carries === S.carries) return `UG-in at (${S.x},${S.y}) blocked by another UG-in at (${pt},${et})`;
          }
        }
        if (!Q) return `UG-in at (${S.x},${S.y}) has no matching UG-out within reach ${p.maxReach}`;
      }
      return null;
    }
    function nt() {
      if (!u || !p) return;
      const M = K();
      if (M) {
        y = [], P = null, Z(), z("invalid", M);
        return;
      }
      z("solving"), E !== null && window.clearTimeout(E);
      const S = ++C;
      E = window.setTimeout(() => {
        ot(S);
      }, 300);
    }
    async function ot(M) {
      if (p) try {
        const S = await s.solveFixture(p.fixtureJson, f);
        if (M !== C || !u) return;
        if (!S) {
          y = [], P = null, Z(), z("invalid", "SAT cannot complete this layout");
          return;
        }
        const L = new Set(f.map((Q) => T(Q.x, Q.y))), q = [];
        for (const Q of S.entities) {
          const rt = O_(Q);
          rt && !L.has(T(rt.x, rt.y)) && q.push(rt);
        }
        y = q, P = S.cost, Z(), z("valid");
      } catch (S) {
        if (M !== C) return;
        y = [], Z(), z("invalid", `solver error: ${S instanceof Error ? S.message : String(S)}`);
      }
    }
    function ht(M) {
      const S = e.getBoundingClientRect(), L = t.toWorld(M.clientX - S.left, M.clientY - S.top);
      return {
        x: Math.floor(L.x / k),
        y: Math.floor(L.y / k)
      };
    }
    function gt(M, S, L) {
      const q = [], Q = S.x === M.x ? 0 : S.x > M.x ? 1 : -1, rt = S.y === M.y ? 0 : S.y > M.y ? 1 : -1;
      if (L) {
        for (let et = M.y; et !== S.y + rt && rt !== 0; et += rt) q.push({
          x: M.x,
          y: et
        });
        rt === 0 && q.push({
          x: M.x,
          y: M.y
        });
        for (let et = M.x + Q; Q !== 0 && et !== S.x + Q; et += Q) q.push({
          x: et,
          y: S.y
        });
      } else {
        for (let et = M.x; et !== S.x + Q && Q !== 0; et += Q) q.push({
          x: et,
          y: M.y
        });
        Q === 0 && q.push({
          x: M.x,
          y: M.y
        });
        for (let et = M.y + rt; rt !== 0 && et !== S.y + rt; et += rt) q.push({
          x: S.x,
          y: et
        });
      }
      const pt = [];
      for (const et of q) {
        const ft = pt[pt.length - 1];
        (!ft || ft.x !== et.x || ft.y !== et.y) && pt.push(et);
      }
      return pt;
    }
    function I(M, S) {
      return M.x === S.x && M.y === S.y - 1 ? "South" : M.x === S.x && M.y === S.y + 1 ? "North" : M.y === S.y && M.x === S.x - 1 ? "East" : M.y === S.y && M.x === S.x + 1 ? "West" : null;
    }
    function G(M) {
      if (!p || M.length === 0) return null;
      for (const ft of M) if (!A(ft.x, ft.y)) return null;
      const S = (ft) => !p.forbidden.has(T(ft.x, ft.y)), L = M[0], q = M[M.length - 1];
      if (!L || !q || !S(L) || !S(q)) return null;
      const Q = [], rt = M.length > 1 ? I(M[0], M[1]) ?? x : x, pt = N(M[0], rt);
      let et = 0;
      for (; et < M.length; ) {
        const ft = M[et], $t = M[et + 1] ?? null, he = $t ? I(ft, $t) : et > 0 ? I(M[et - 1], ft) : x;
        if (!he) return null;
        let Ut = et + 1;
        for (; Ut < M.length && S(M[Ut]); ) Ut++;
        if (Ut === M.length) {
          Q.push(V(ft, he, pt)), et++;
          continue;
        }
        let F = Ut;
        for (; F < M.length && !S(M[F]); ) F++;
        if (F === M.length) return null;
        const tt = M[Ut - 1], st = M[F];
        if (Math.abs(st.x - tt.x) + Math.abs(st.y - tt.y) > p.maxReach + 1) return null;
        for (let Pt = et; Pt < Ut - 1; Pt++) {
          const It = M[Pt], Tt = I(It, M[Pt + 1]);
          if (!Tt) return null;
          Q.push(V(It, Tt, pt));
        }
        const at = I(tt, st);
        if (!at) return null;
        Q.push(J(tt, at, "input", pt)), Q.push(J(st, at, "output", pt)), et = F + 1;
      }
      return Q;
    }
    function V(M, S, L) {
      return {
        name: Ai[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "transport-belt",
        x: M.x,
        y: M.y,
        direction: S,
        carries: L ?? void 0
      };
    }
    function J(M, S, L, q) {
      return {
        name: fc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "underground-belt",
        x: M.x,
        y: M.y,
        direction: S,
        io_type: L,
        carries: q ?? void 0
      };
    }
    function xt(M) {
      const S = new Set(M.map((q) => T(q.x, q.y))), L = f.filter((q) => !S.has(T(q.x, q.y))).concat(M);
      X(L);
    }
    function bt(M) {
      if (!p || !A(M.x, M.y) || !f.some((L) => L.x === M.x && L.y === M.y)) return;
      const S = f.filter((L) => !(L.x === M.x && L.y === M.y));
      X(S);
    }
    function wt(M) {
      if (!p) return;
      const S = p.boundaries.find((q) => q.x === M.x && q.y === M.y && q.isInput);
      if (!S) return;
      const L = p.items.indexOf(S.item);
      L >= 0 && L !== _ && (_ = L, dt());
    }
    function Dt(M) {
      if (!u || !p || M.button !== 0) return;
      const S = ht(M);
      if (!S || !A(S.x, S.y)) return;
      if (wt(S), b === "erase") {
        bt(S), M.stopPropagation(), M.preventDefault();
        return;
      }
      if (b === "ug-in" || b === "ug-out") {
        const Q = b === "ug-in" ? D(S.x, S.y, x) ?? j() : $(S, x) ?? D(S.x, S.y, x) ?? j(), rt = b === "ug-in" ? J(S, x, "input", Q) : J(S, x, "output", Q), pt = f.filter((et) => !(et.x === S.x && et.y === S.y)).concat(rt);
        X(pt), M.stopPropagation(), M.preventDefault();
        return;
      }
      v = {
        startX: S.x,
        startY: S.y,
        bendVerticalFirst: false
      };
      const L = gt({
        x: S.x,
        y: S.y
      }, S, false), q = G(L);
      B(q ?? [], q === null), M.stopPropagation(), M.preventDefault();
    }
    function kt(M) {
      if (!u || !v || !p) return;
      const S = ht(M);
      if (!S) return;
      const L = gt({
        x: v.startX,
        y: v.startY
      }, S, v.bendVerticalFirst), q = G(L);
      B(q ?? [], q === null);
    }
    function Ft(M) {
      if (!u || !v || !p) {
        v = null, O();
        return;
      }
      const S = ht(M);
      if (!S) {
        v = null, O();
        return;
      }
      const L = gt({
        x: v.startX,
        y: v.startY
      }, S, v.bendVerticalFirst), q = G(L);
      if (v = null, O(), !q) {
        z("invalid", "drag rejected: out of bounds, on obstacle, or UG too long");
        return;
      }
      xt(q), M.stopPropagation(), M.preventDefault();
    }
    function ct(M) {
      var _a2, _b2;
      if (!u) return;
      const S = (_b2 = (_a2 = M.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (S === "INPUT" || S === "TEXTAREA" || S === "SELECT") return;
      const L = () => {
        M.stopImmediatePropagation(), M.preventDefault();
      };
      if (M.key === "Escape") {
        Ht(), L();
        return;
      }
      if (M.key === "1") {
        vt("belt"), L();
        return;
      }
      if (M.key === "2") {
        vt("ug-in"), L();
        return;
      }
      if (M.key === "3") {
        vt("ug-out"), L();
        return;
      }
      if (M.key === "0") {
        vt("erase"), L();
        return;
      }
      if (M.key === "r" || M.key === "R") {
        v ? v.bendVerticalFirst = !v.bendVerticalFirst : (x = pc[x], dt()), L();
        return;
      }
      if (M.key === "[" && p) {
        _ = (_ - 1 + p.items.length) % p.items.length, dt(), L();
        return;
      }
      if (M.key === "]" && p) {
        _ = (_ + 1) % p.items.length, dt(), L();
        return;
      }
      if ((M.key === "Enter" || M.key === "a" || M.key === "A") && w === "valid" && y.length > 0) {
        Te(), L();
        return;
      }
      if ((M.ctrlKey || M.metaKey) && (M.key === "z" || M.key === "Z")) {
        M.shiftKey ? ce() : Jt(), L();
        return;
      }
    }
    function vt(M) {
      b = M, dt();
    }
    function Jt() {
      g.length !== 0 && (m.push(f), f = g.pop(), Z(), nt());
    }
    function ce() {
      m.length !== 0 && (g.push(f), f = m.pop(), Z(), nt());
    }
    function Te() {
      if (y.length === 0) return;
      const M = f.concat(y.map((S) => ({
        ...S
      })));
      y = [], X(M);
    }
    function dt() {
      if (!h) return;
      h.innerHTML = "";
      const M = [
        [
          "belt",
          "B",
          "Belt (1)"
        ],
        [
          "ug-in",
          "\u21A7",
          "UG-in (2)"
        ],
        [
          "ug-out",
          "\u21A5",
          "UG-out (3)"
        ],
        [
          "erase",
          "\u2715",
          "Erase (0)"
        ]
      ];
      for (const [ft, $t, he] of M) {
        const Ut = document.createElement("button");
        Ut.className = "se-tool" + (b === ft ? " se-tool-active" : ""), Ut.textContent = $t, Ut.title = he, Ut.addEventListener("click", () => vt(ft)), h.appendChild(Ut);
      }
      const S = document.createElement("button");
      S.className = "se-dir";
      const L = {
        North: "\u2191",
        East: "\u2192",
        South: "\u2193",
        West: "\u2190"
      };
      if (S.textContent = L[x], S.title = "Brush direction (R rotates)", S.addEventListener("click", () => {
        x = pc[x], dt();
      }), h.appendChild(S), p && p.items.length > 1) {
        const ft = document.createElement("select");
        ft.className = "se-item";
        for (const [$t, he] of p.items.entries()) {
          const Ut = document.createElement("option");
          Ut.value = String($t), Ut.textContent = he, $t === _ && (Ut.selected = true), ft.appendChild(Ut);
        }
        ft.addEventListener("change", () => {
          _ = Number(ft.value) | 0;
        }), h.appendChild(ft);
      } else if (p && p.items.length === 1) {
        const ft = document.createElement("span");
        ft.className = "se-item-label", ft.textContent = p.items[0], h.appendChild(ft);
      }
      const q = document.createElement("span");
      q.style.flex = "1", h.appendChild(q);
      const Q = document.createElement("button");
      Q.className = "se-accept", Q.textContent = "Accept", Q.title = "Promote ghost into painted layer (Enter)", Q.addEventListener("click", Te), Q.disabled = !(w === "valid" && y.length > 0), h.appendChild(Q);
      const rt = document.createElement("button");
      rt.className = "se-revert", rt.textContent = "Revert", rt.title = "Discard all painted edits", rt.addEventListener("click", () => {
        X([]);
      }), h.appendChild(rt);
      const pt = document.createElement("button");
      pt.className = "se-export", pt.textContent = "Export", pt.title = "Save fixture JSON (clipboard + download)", pt.addEventListener("click", Rt), pt.disabled = w !== "valid", h.appendChild(pt);
      const et = document.createElement("button");
      et.className = "se-done", et.textContent = "Done", et.title = "Exit edit mode (Esc)", et.addEventListener("click", Ht), h.appendChild(et);
    }
    function pe() {
      if (!h) return;
      const M = h.querySelector(".se-accept");
      M && (M.disabled = !(w === "valid" && y.length > 0));
      const S = h.querySelector(".se-export");
      S && (S.disabled = w !== "valid");
    }
    function Rt() {
      var _a2;
      if (!p || w !== "valid") return;
      const M = P ?? Y(f), S = Io(p.selection.cluster, p.selection.iter, p.selection.trace, {
        maxCost: M,
        paintedEntities: f
      }), L = (p.selection.cluster.seed ? `fixture_${p.selection.cluster.seed.x}_${p.selection.cluster.seed.y}_painted` : "fixture_painted") + ".json";
      (_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText(S).catch(() => {
      });
      const q = new Blob([
        S
      ], {
        type: "application/json"
      }), Q = URL.createObjectURL(q), rt = document.createElement("a");
      rt.href = Q, rt.download = L, document.body.appendChild(rt), rt.click(), document.body.removeChild(rt), URL.revokeObjectURL(Q);
    }
    function Zt(M) {
      var _a2, _b2, _c2, _d2;
      u && Ht();
      const S = M.iter, L = ((_a2 = S.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", q = ((_b2 = S.sat) == null ? void 0 : _b2.max_reach) ?? Qd[L] ?? 4, Q = ((_c2 = S.sat) == null ? void 0 : _c2.boundaries) ?? S.boundaries, rt = Array.from(new Set(Q.map((et) => et.item))), pt = Q.map((et) => ({
        x: et.x,
        y: et.y,
        item: et.item,
        isInput: et.is_input,
        dir: et.direction
      }));
      p = {
        bbox: {
          x: S.bbox.x,
          y: S.bbox.y,
          w: S.bbox.w,
          h: S.bbox.h
        },
        forbidden: new Set((S.forbidden ?? []).map((et) => `${et[0]},${et[1]}`)),
        beltTier: L,
        maxReach: q,
        items: rt,
        boundaries: pt,
        fixtureJson: Io(M.cluster, M.iter, M.trace),
        selection: M
      }, f = [], g = [], m = [], y = [], b = "belt", x = "East", _ = 0, v = null, P = null, o = new zt(), a = new zt(), a.alpha = 0.55, l = new zt(), c = new zt(), t.addChild(o), t.addChild(a), t.addChild(c), t.addChild(l), t.setChildIndex(r, t.children.length - 1), h = document.createElement("div"), h.className = "se-toolbar", i.inlineEl.appendChild(h), d = document.createElement("span"), d.className = "se-status", (_d2 = i.inlineEl.querySelector(".jd-inline-head")) == null ? void 0 : _d2.appendChild(d), i.setEditMode(true), t.plugins.pause("drag"), e.addEventListener("pointerdown", Dt, {
        capture: true
      }), e.addEventListener("pointerup", Ft, {
        capture: true
      }), e.addEventListener("pointermove", kt, {
        capture: true
      }), document.addEventListener("keydown", ct, {
        capture: true
      }), u = true, dt(), z("idle");
    }
    function Ht() {
      u && (u = false, E !== null && (window.clearTimeout(E), E = null), C++, o && (t.removeChild(o), o.destroy({
        children: true
      }), o = null), a && (t.removeChild(a), a.destroy({
        children: true
      }), a = null), l && (t.removeChild(l), l.destroy({
        children: true
      }), l = null), c && (t.removeChild(c), c.destroy({
        children: true
      }), c = null), h && (h.remove(), h = null), d && (d.remove(), d = null), e.removeEventListener("pointerdown", Dt, {
        capture: true
      }), e.removeEventListener("pointerup", Ft, {
        capture: true
      }), e.removeEventListener("pointermove", kt, {
        capture: true
      }), document.removeEventListener("keydown", ct, {
        capture: true
      }), t.plugins.resume("drag"), i.setEditMode(false), p = null, f = [], g = [], m = [], y = []);
    }
    function Qt() {
      return u;
    }
    return {
      enter: Zt,
      exit: Ht,
      isActive: Qt
    };
  }
  let ds = {
    master: false,
    stepThrough: true,
    satZones: false,
    soloRegions: false,
    ghostTiles: false,
    itemColors: true,
    traceOverlay: false
  };
  const N_ = [];
  function W_() {
    const n = new URLSearchParams(window.location.search).get("debug") === "1", t = localStorage.getItem("fk-debug") === "1", e = localStorage.getItem("fk-sat-zones") === "1", s = localStorage.getItem("fk-ghost-tiles") === "1", i = localStorage.getItem("fk-item-colors"), r = localStorage.getItem("fk-trace-overlay") === "1";
    ds = {
      ...ds,
      master: n || t,
      satZones: e,
      ghostTiles: s,
      itemColors: i === null ? true : i === "1",
      traceOverlay: r
    };
  }
  function G_() {
    return ds;
  }
  function ts(n) {
    ds = {
      ...ds,
      ...n
    }, "master" in n && localStorage.setItem("fk-debug", n.master ? "1" : "0"), "satZones" in n && localStorage.setItem("fk-sat-zones", n.satZones ? "1" : "0"), "ghostTiles" in n && localStorage.setItem("fk-ghost-tiles", n.ghostTiles ? "1" : "0"), "itemColors" in n && localStorage.setItem("fk-item-colors", n.itemColors ? "1" : "0"), "traceOverlay" in n && localStorage.setItem("fk-trace-overlay", n.traceOverlay ? "1" : "0");
    for (const t of N_) t(ds);
  }
  function es(n, t, e = false) {
    const s = document.createElement("input");
    s.type = "checkbox", s.checked = e;
    const i = document.createElement("div");
    i.className = "overlay-toggle";
    const r = document.createElement("label");
    return r.appendChild(s), r.appendChild(document.createTextNode(t)), i.appendChild(r), n.appendChild(i), s;
  }
  function z_(n) {
    n.style.position = "relative";
    const t = document.createElement("div");
    t.className = "overlay-panel";
    const e = G_(), s = es(t, "Debug", e.master), i = es(t, "Item colours", e.itemColors), r = document.createElement("div");
    r.className = "overlay-sub-panel", r.style.display = e.master ? "flex" : "none";
    const o = es(r, "SAT Zones", e.satZones), a = es(r, "Ghost tiles", e.ghostTiles), l = es(r, "Trace overlay", e.traceOverlay), c = es(r, "Solo regions", e.soloRegions);
    return t.appendChild(r), n.appendChild(t), s.addEventListener("change", () => {
      r.style.display = s.checked ? "flex" : "none", ts({
        master: s.checked
      });
    }), o.addEventListener("change", () => {
      ts({
        satZones: o.checked
      });
    }), a.addEventListener("change", () => {
      ts({
        ghostTiles: a.checked
      });
    }), l.addEventListener("change", () => {
      ts({
        traceOverlay: l.checked
      });
    }), i.addEventListener("change", () => {
      ts({
        itemColors: i.checked
      });
    }), {
      setDebugEnabled(h) {
        s.checked = h, r.style.display = h ? "flex" : "none", ts({
          master: h
        });
      },
      debugCb: s,
      colorCb: i,
      regionsCb: o,
      soloRegionsCb: c,
      ghostTilesCb: a,
      traceOverlayCb: l
    };
  }
  const D_ = {
    North: "\u2191",
    East: "\u2192",
    South: "\u2193",
    West: "\u2190"
  }, mc = {
    N: "\u2191",
    E: "\u2192",
    S: "\u2193",
    W: "\u2190"
  };
  function nn(n = 16) {
    const t = document.createElement("img");
    return t.width = n, t.height = n, t.style.cssText = "vertical-align:middle;margin-right:3px;image-rendering:pixelated", t.addEventListener("error", () => {
      t.style.display = "none";
    }), t;
  }
  function ns(n, t) {
    n.style.display = "", n.src = `/fucktorio/pr-238/icons/${t}.png`;
  }
  function Hr(n, t) {
    const e = [];
    return {
      get(s) {
        for (; e.length <= s; ) {
          const i = t();
          n.appendChild(i), e.push(i);
        }
        return e[s].style.display = "", e[s];
      },
      trim(s) {
        for (let i = s; i < e.length; i++) e[i].style.display = "none";
      },
      get length() {
        return e.length;
      }
    };
  }
  function H_(n) {
    const t = document.createElement("div");
    t.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:240px;line-height:1.5", document.body.appendChild(t);
    const e = document.createElement("div");
    t.appendChild(e);
    const s = document.createElement("span");
    s.style.color = "#888", s.style.display = "none", t.appendChild(s);
    const i = document.createElement("div");
    t.appendChild(i);
    const r = document.createElement("div"), o = nn(16), a = document.createElement("b");
    r.append(o, a), r.style.display = "none", i.appendChild(r);
    const l = document.createElement("div");
    l.style.display = "none", i.appendChild(l);
    const c = document.createElement("div"), h = nn(16), d = document.createElement("span");
    c.append(h, d), c.style.display = "none", i.appendChild(c);
    const u = document.createElement("div");
    u.style.color = "#b5cea8", u.style.display = "none", i.appendChild(u);
    const p = document.createElement("div");
    p.style.display = "none", i.appendChild(p);
    const f = document.createElement("div"), g = nn(16), m = document.createElement("span");
    f.append(g, m), f.style.display = "none", i.appendChild(f);
    function y() {
      const S = document.createElement("div");
      S.style.color = "#aaa";
      const L = document.createElement("span"), q = nn(14), Q = document.createElement("span");
      return S.append(L, q, Q), S;
    }
    const b = Hr(i, y), x = document.createElement("div");
    x.style.color = "#9cdcfe", x.style.display = "none", i.appendChild(x);
    const _ = document.createElement("div");
    _.style.display = "none", t.appendChild(_);
    const v = document.createElement("div");
    v.style.display = "none", t.appendChild(v);
    const w = document.createElement("div");
    w.style.display = "none", t.appendChild(w);
    const C = document.createElement("div");
    C.style.cssText = "position:absolute;top:8px;right:8px;background:#141414;color:#e0e0e0;border:1px solid #888;padding:8px 10px;font:12px monospace;border-radius:4px;display:none;z-index:20;min-width:220px;max-width:340px;line-height:1.55;box-shadow:0 4px 14px rgba(0,0,0,0.5)", n.appendChild(C);
    const E = document.createElement("div");
    E.style.cssText = "display:flex;justify-content:space-between;align-items:center;margin-bottom:4px";
    const P = document.createElement("span");
    P.style.cssText = "color:#8af;font-weight:bold", P.textContent = "pinned";
    const T = document.createElement("span");
    T.style.color = "#888", E.append(P, T), C.appendChild(E);
    const A = document.createElement("div");
    C.appendChild(A);
    const H = document.createElement("div"), j = nn(16), D = document.createElement("b");
    H.append(j, D), H.style.display = "none", A.appendChild(H);
    const N = document.createElement("span");
    N.style.color = "#888", N.textContent = "no entity at tile", N.style.display = "none", A.appendChild(N);
    const $ = document.createElement("div");
    $.style.display = "none", A.appendChild($);
    const W = document.createElement("div"), X = nn(16), U = document.createElement("span");
    W.append(X, U), W.style.display = "none", A.appendChild(W);
    const Z = document.createElement("div");
    Z.style.color = "#b5cea8", Z.style.display = "none", A.appendChild(Z);
    const R = document.createElement("div");
    R.style.display = "none", A.appendChild(R);
    const B = document.createElement("div"), O = nn(16), z = document.createElement("span");
    B.append(O, z), B.style.display = "none", A.appendChild(B);
    const Y = Hr(A, y), K = document.createElement("div");
    K.style.color = "#9cdcfe", K.style.display = "none", A.appendChild(K);
    const nt = document.createElement("div");
    nt.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", nt.style.display = "none";
    const ot = document.createElement("span"), ht = document.createElement("span");
    ht.style.color = "#888", nt.append(ot, ht), C.appendChild(nt);
    const gt = document.createElement("div");
    gt.style.marginTop = "4px", gt.style.display = "none", C.appendChild(gt);
    const I = document.createElement("div");
    I.style.display = "none", C.appendChild(I);
    const G = document.createElement("div");
    G.style.marginTop = "4px", I.appendChild(G);
    function V() {
      const S = document.createElement("div");
      return S.style.marginLeft = "4px", S;
    }
    const J = Hr(I, V), xt = document.createElement("div");
    xt.style.cssText = "color:#555;margin-top:6px;font-size:10px", xt.textContent = "click elsewhere or press Esc to unpin", C.appendChild(xt), document.addEventListener("mousemove", (S) => {
      t.style.left = S.clientX + 14 + "px", t.style.top = S.clientY - 10 + "px";
    });
    let bt = null, wt = null, Dt = null, kt = null, Ft = null, ct = null;
    const vt = /* @__PURE__ */ new Set();
    function Jt() {
      const S = ct ? {
        x: ct.x,
        y: ct.y
      } : null;
      for (const L of vt) L(S);
    }
    function ce(S, L) {
      ns(L.headerIcon, S.name), L.headerName.textContent = ue(S.name), L.header.style.display = "", S.direction && S.name !== "pipe" ? (L.dirRow.textContent = `${D_[S.direction] ?? ""} ${S.direction}`, L.dirRow.style.display = "") : L.dirRow.style.display = "none", S.carries ? (ns(L.carriesIcon, S.carries), L.carriesName.textContent = " " + ue(S.carries), L.carriesRow.style.display = "") : L.carriesRow.style.display = "none", S.rate != null ? (L.rateRow.textContent = `${S.rate.toFixed(1)}/s`, L.rateRow.style.display = "") : L.rateRow.style.display = "none", S.io_type ? (L.ioRow.textContent = `io: ${S.io_type}`, L.ioRow.style.display = "") : L.ioRow.style.display = "none";
      let q = 0;
      if (S.recipe) {
        ns(L.recipeIcon, S.recipe), L.recipeName.textContent = " " + ue(S.recipe), L.recipeRow.style.display = "";
        const Q = Px(S.recipe);
        if (Q) {
          const rt = [
            ...Q.inputs.map((pt) => ({
              arrow: "\u25B6",
              item: pt.item,
              rate: pt.rate
            })),
            ...Q.outputs.map((pt) => ({
              arrow: "\u25C0",
              item: pt.item,
              rate: pt.rate
            }))
          ];
          for (const pt of rt) {
            const et = L.flowPool.get(q++), [ft, $t, he] = et.children;
            ft.textContent = `${pt.arrow} `, ns($t, pt.item), he.textContent = `${ue(pt.item)} ${pt.rate.toFixed(1)}/s`;
          }
        }
      } else L.recipeRow.style.display = "none";
      return L.flowPool.trim(q), S.segment_id ? (L.segmentRow.textContent = S.segment_id, L.segmentRow.style.display = "") : L.segmentRow.style.display = "none", q;
    }
    function Te(S) {
      if (S.ghosts.length === 0) return _.style.display = "none", false;
      if (_.style.display = "", S.ghosts.length === 1) {
        const L = S.ghosts[0], q = L.direction ? mc[L.direction] : "";
        _.textContent = "";
        const Q = document.createElement("span");
        Q.style.color = "#8af", Q.textContent = "ghost ";
        const rt = nn(12);
        ns(rt, L.item);
        const pt = document.createTextNode(`${L.item} ${q}`);
        _.append(Q, rt, pt);
      } else {
        _.textContent = "";
        const L = document.createElement("span");
        L.style.color = "#8af", L.textContent = `${S.ghosts.length} ghosts crossing`, _.appendChild(L);
      }
      return true;
    }
    function dt(S) {
      if (!S.axis) return v.style.display = "none", false;
      const { vert: L, horiz: q } = S.axis;
      if (L === 0 && q === 0) return v.style.display = "none", false;
      const Q = L >= 2 || q >= 2, rt = L >= 1 && q >= 1, pt = Q ? "#ff6060" : rt ? "#60b0ff" : "#888";
      return v.style.display = "", v.style.color = pt, v.textContent = `axis V${L} H${q}`, true;
    }
    function pe(S) {
      if (!S.junction) return w.style.display = "none", false;
      const L = S.junction, q = L.outcome === "Solved" ? "#80d080" : L.outcome === "Capped" ? "#e0b060" : "#c06060";
      return w.style.display = "", w.style.color = q, w.textContent = `junction seed (${L.seedX},${L.seedY}) \xB7 ${L.outcome}`, true;
    }
    function Rt(S) {
      if (S.ghosts.length === 0) {
        I.style.display = "none";
        return;
      }
      I.style.display = "", S.ghosts.length >= 2 ? (G.style.color = "#ffa060", G.textContent = `\u26A0 ${S.ghosts.length} ghost specs at this tile`) : (G.style.color = "#8af", G.textContent = "ghost");
      let L = 0;
      for (const q of S.ghosts) {
        const Q = q.direction ? mc[q.direction] : "\xB7", rt = J.get(L++);
        rt.textContent = "";
        const pt = document.createTextNode(`${Q} `), et = nn(14);
        ns(et, q.item);
        const ft = document.createTextNode(q.item);
        if (rt.append(pt, et, ft), q.isStart) {
          const $t = document.createElement("span");
          $t.style.color = "#80d080", $t.textContent = " start", rt.appendChild($t);
        } else if (q.isEnd) {
          const $t = document.createElement("span");
          $t.style.color = "#d08080", $t.textContent = " end", rt.appendChild($t);
        }
      }
      J.trim(L);
    }
    function Zt() {
      if (wt !== null) {
        e.innerHTML = wt, e.style.display = "", i.style.display = "none", _.style.display = "none", v.style.display = "none", w.style.display = "none", kt ? (s.textContent = `(${kt.x}, ${kt.y})`, s.style.display = "", s.style.display = "block") : s.style.display = "none", t.style.display = "block";
        return;
      }
      e.style.display = "none", e.innerHTML = "", i.style.display = "";
      let S = false;
      if (Dt ? (S = true, ce(Dt, {
        header: r,
        headerIcon: o,
        headerName: a,
        dirRow: l,
        carriesRow: c,
        carriesIcon: h,
        carriesName: d,
        rateRow: u,
        ioRow: p,
        recipeRow: f,
        recipeIcon: g,
        recipeName: m,
        flowPool: b,
        segmentRow: x
      })) : (r.style.display = "none", l.style.display = "none", c.style.display = "none", u.style.display = "none", p.style.display = "none", f.style.display = "none", b.trim(0), x.style.display = "none"), kt) {
        const L = Ft == null ? void 0 : Ft.lookup(kt.x, kt.y);
        L ? (Te(L) && (S = true), dt(L) && (S = true), pe(L) && (S = true)) : (_.style.display = "none", v.style.display = "none", w.style.display = "none"), s.textContent = `(${kt.x}, ${kt.y})`, s.style.display = "block", S = true;
      } else s.style.display = "none", _.style.display = "none", v.style.display = "none", w.style.display = "none";
      if (!S) {
        t.style.display = "none", bt && bt.clearHighlight();
        return;
      }
      t.style.display = "block", Dt && bt ? bt.highlightBeltNetwork(Dt) : bt && bt.clearHighlight();
    }
    function Ht() {
      if (!ct) {
        C.style.display = "none";
        return;
      }
      const { entity: S, x: L, y: q } = ct, Q = Ft == null ? void 0 : Ft.lookup(L, q);
      if (T.textContent = `(${L}, ${q})`, S ? (N.style.display = "none", ce(S, {
        header: H,
        headerIcon: j,
        headerName: D,
        dirRow: $,
        carriesRow: W,
        carriesIcon: X,
        carriesName: U,
        rateRow: Z,
        ioRow: R,
        recipeRow: B,
        recipeIcon: O,
        recipeName: z,
        flowPool: Y,
        segmentRow: K
      })) : (H.style.display = "none", $.style.display = "none", W.style.display = "none", Z.style.display = "none", R.style.display = "none", B.style.display = "none", Y.trim(0), K.style.display = "none", N.style.display = ""), Q) {
        if (Q.junction) {
          const rt = Q.junction.outcome === "Solved" ? "#80d080" : Q.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
          ot.style.color = rt, ot.textContent = `junction seed (${Q.junction.seedX},${Q.junction.seedY})`, ht.textContent = ` \xB7 ${Q.junction.outcome}`, nt.style.display = "";
        } else nt.style.display = "none";
        if (Q.axis) {
          const { vert: rt, horiz: pt } = Q.axis;
          if (rt > 0 || pt > 0) {
            const et = rt >= 2 || pt >= 2, ft = rt >= 1 && pt >= 1, $t = et ? " same-axis conflict" : ft ? " perpendicular crossing" : "", he = et ? "#ff6060" : ft ? "#60b0ff" : "#bbb";
            gt.style.color = he, gt.textContent = `axis: V=${rt} H=${pt}${$t}`, gt.style.display = "";
          } else gt.style.display = "none";
        } else gt.style.display = "none";
        Rt(Q);
      } else nt.style.display = "none", gt.style.display = "none", I.style.display = "none";
      C.style.display = "block";
    }
    function Qt() {
      Zt(), Ht();
    }
    function M(S, L, q) {
      Dt = S, L !== void 0 && q !== void 0 ? kt = {
        x: L,
        y: q
      } : S && (kt = {
        x: S.x ?? 0,
        y: S.y ?? 0
      }), Qt();
    }
    return document.addEventListener("keydown", (S) => {
      S.key === "Escape" && ct && (ct = null, Jt(), Qt());
    }), {
      onHover: M,
      setHighlightController(S) {
        bt = S;
      },
      setTooltipOverride(S) {
        wt = S, Qt();
      },
      setCursorTile(S, L) {
        S === null || L === void 0 ? kt = null : kt = {
          x: S,
          y: L
        }, Qt();
      },
      setTileContext(S) {
        Ft = S, Qt();
      },
      pinTile(S, L, q) {
        ct = {
          entity: S,
          x: L,
          y: q
        }, Jt(), Qt();
      },
      clearPin() {
        ct = null, Jt(), Qt();
      },
      getPinnedTile() {
        return ct ? {
          x: ct.x,
          y: ct.y
        } : null;
      },
      onPinChange(S) {
        return vt.add(S), () => vt.delete(S);
      }
    };
  }
  function U_(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function gc(n, t, e, s) {
    return e > n ? "E" : e < n ? "W" : s > t ? "S" : s < t ? "N" : null;
  }
  const j_ = {
    ghosts: [],
    axis: null,
    junction: null
  };
  function V_(n) {
    var _a2;
    if (!n || n.length === 0) return {
      lookup: () => j_
    };
    const t = /* @__PURE__ */ new Map(), e = /* @__PURE__ */ new Map(), s = /* @__PURE__ */ new Map();
    for (const i of n) if (i.phase === "GhostSpecRouted") {
      const { spec_key: r, tiles: o } = i.data, a = U_(r);
      if (!o || o.length === 0) continue;
      for (let l = 0; l < o.length; l++) {
        const [c, h] = o[l];
        let d = null;
        l < o.length - 1 ? d = gc(c, h, o[l + 1][0], o[l + 1][1]) : l > 0 && (d = gc(o[l - 1][0], o[l - 1][1], c, h));
        const u = `${c},${h}`, p = t.get(u), f = {
          item: a,
          specKey: r,
          direction: d,
          isStart: l === 0,
          isEnd: l === o.length - 1
        };
        p ? p.push(f) : t.set(u, [
          f
        ]);
      }
    } else if (i.phase === "GhostAxisOccupancy") for (const r of i.data.tiles) e.set(`${r.x},${r.y}`, {
      vert: r.vert_count,
      horiz: r.horiz_count
    });
    else if (i.phase === "JunctionSolved" || i.phase === "JunctionGrowthCapped") {
      const r = i.data, o = i.phase === "JunctionSolved" ? "Solved" : "Capped";
      s.set(`${r.tile_x},${r.tile_y}`, {
        seedX: r.tile_x,
        seedY: r.tile_y,
        outcome: o
      });
    } else if (i.phase === "JunctionGrowthIteration") {
      const r = i.data, o = `${r.seed_x},${r.seed_y}`;
      for (const [a, l] of r.tiles) {
        const c = `${a},${l}`;
        (!s.has(c) || s.get(c).seedX === r.seed_x) && s.set(c, {
          seedX: r.seed_x,
          seedY: r.seed_y,
          outcome: ((_a2 = s.get(o)) == null ? void 0 : _a2.outcome) ?? "Open"
        });
      }
    }
    return {
      lookup(i, r) {
        const o = `${i},${r}`;
        return {
          ghosts: t.get(o) ?? [],
          axis: e.get(o) ?? null,
          junction: s.get(o) ?? null
        };
      }
    };
  }
  function Y_(n) {
    let t = null;
    function e() {
      t && (t.remove(), t = null);
      const { sidebarEl: i } = n;
      i == null ? void 0 : i.querySelectorAll("input,select,button").forEach((r) => {
        r.disabled = false;
      });
    }
    function s(i) {
      var _a2;
      const r = {
        ...i.layout,
        trace: i.trace.events
      };
      (i.trace.events.length > 0 || i.validation.issues.length > 0) && n.onDebugEnable(), n.renderLayoutOnCanvas(r), i.validation.issues.length > 0 && (n.setCachedValidationIssues(i.validation.issues), n.updateValidationOverlay()), (_a2 = n.getSidebarCtrl()) == null ? void 0 : _a2.setParams(i.params, {
        skipAutoSolve: true
      }), e();
      const o = {
        onClear: () => n.onClear()
      }, { sidebarEl: a } = n;
      a && (t = Ib(a, i, o), a.querySelectorAll("input,select,button").forEach((l) => {
        l.closest("[data-snapshot-keep]") || (l.disabled = true);
      }));
    }
    return {
      load: s,
      clear: e
    };
  }
  const X_ = 2200, yc = 180, Ur = 200, q_ = 8, K_ = [
    "rows_placed",
    "lanes_planned",
    "bus_routed",
    "poles_placed"
  ];
  function Ui(n) {
    return `${n.x ?? 0},${n.y ?? 0},${n.name},${n.recipe ?? ""}`;
  }
  function J_(n) {
    const t = /* @__PURE__ */ new Map(), e = n.trace;
    if (!Array.isArray(e)) return t;
    for (const s of e) {
      const i = s;
      i.phase === "PhaseSnapshot" && i.data && t.set(i.data.phase, i.data);
    }
    return t;
  }
  function Z_(n) {
    const t = J_(n), e = [], s = /* @__PURE__ */ new Set();
    for (const r of K_) {
      const o = t.get(r);
      if (!o) {
        e.push({
          phase: r,
          entities: []
        });
        continue;
      }
      const a = [];
      for (const l of o.entities) {
        const c = Ui(l);
        s.has(c) || (s.add(c), a.push(l));
      }
      e.push({
        phase: r,
        entities: a
      });
    }
    const i = [];
    for (const r of n.entities) {
      const o = Ui(r);
      s.has(o) || (s.add(o), i.push(r));
    }
    e.push({
      phase: "final",
      entities: i
    });
    for (const r of e) r.entities.sort((o, a) => {
      const l = o.y ?? 0, c = a.y ?? 0;
      return l !== c ? l - c : (o.x ?? 0) - (a.x ?? 0);
    });
    return e;
  }
  function Q_(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map(), o = zs(n, t, e, s, (w, C) => {
      r.set(Ui(w), C);
    }), a = /* @__PURE__ */ new Set();
    for (const w of r.values()) for (const C of w) a.add(C);
    const l = [];
    for (const w of t.children) {
      const C = w;
      a.has(C) || l.push(C);
    }
    for (const w of l) w.alpha = 0;
    for (const w of r.values()) for (const C of w) C.alpha = 0;
    const c = Z_(n), h = c.reduce((w, C) => w + (C.entities.length > 0 ? 1 : 0), 0);
    if (h === 0) {
      for (const w of l) w.alpha = 1;
      for (const w of r.values()) for (const C of w) C.alpha = 1;
      return {
        controller: o,
        handle: {
          cancel: () => {
          },
          finish: () => {
          },
          isDone: () => true
        }
      };
    }
    const u = Math.max(0, X_ - yc * h) / h, p = [], f = /* @__PURE__ */ new Map();
    let g = 0;
    for (const w of c) {
      if (w.entities.length === 0) continue;
      f.set(w.phase, g);
      const C = Math.min(q_, u / w.entities.length);
      w.entities.forEach((P, T) => {
        const A = r.get(Ui(P));
        !A || A.length === 0 || p.push({
          graphics: A,
          revealStartMs: g + T * C
        });
      });
      const E = (w.entities.length - 1) * C;
      g += E + Ur + yc;
    }
    if (l.length > 0) {
      const w = f.get("bus_routed") ?? f.get("rows_placed") ?? 0;
      for (const C of l) p.push({
        graphics: [
          C
        ],
        revealStartMs: w
      });
    }
    p.sort((w, C) => w.revealStartMs - C.revealStartMs);
    const m = performance.now();
    let y = 0, b = false, x = p.length === 0;
    const _ = () => {
      if (b || x) return;
      const w = performance.now() - m;
      for (let C = y; C < p.length; C++) {
        const E = p[C];
        if (E.revealStartMs > w) break;
        const P = Math.min(1, (w - E.revealStartMs) / Ur);
        for (const T of E.graphics) T.alpha = P;
      }
      for (; y < p.length; ) {
        const C = p[y];
        if (w - C.revealStartMs < Ur) break;
        for (const E of C.graphics) E.alpha = 1;
        y++;
      }
      y >= p.length && (x = true, i.ticker.remove(_), Dn());
    };
    return x || (i.ticker.add(_), sr()), {
      controller: o,
      handle: {
        cancel() {
          b || x || (b = true, i.ticker.remove(_), Dn(), Ze());
        },
        finish() {
          if (!(b || x)) {
            for (const w of p) for (const C of w.graphics) C.alpha = 1;
            x = true, i.ticker.remove(_), Dn(), Ze();
          }
        },
        isDone() {
          return x || b;
        }
      }
    };
  }
  const tw = 4243680;
  function ew(n, t, e, s = 240) {
    const i = new ut();
    t.addChild(i);
    const r = e.x * k, o = e.y * k, a = e.w * k, l = e.h * k;
    let c = 0;
    const h = () => {
      c += n.ticker.deltaMS;
      const d = Math.max(0, (s - c) / s);
      if (d <= 0) {
        n.ticker.remove(h), i.destroy(), Dn();
        return;
      }
      i.clear(), i.rect(r, o, a, l).fill({
        color: tw,
        alpha: 0.55 * d
      });
    };
    n.ticker.add(h), sr();
  }
  const Ei = 150, xc = 80, nw = 4, sw = 300, bn = 4, _n = 900, iw = 6, rw = 250, Is = 800, Mi = 250;
  function Ue(n, t, e) {
    return n <= 1 ? t : Math.min(t, e / n);
  }
  function bc(n, t) {
    return `${n},${t}`;
  }
  function ow(n) {
    return n.split(":")[1] ?? "";
  }
  function aw(n, t) {
    return n > 0 ? "East" : n < 0 ? "West" : t > 0 ? "South" : "North";
  }
  function lw(n, t, e, s) {
    const i = Rd();
    i.attachTo(n);
    const r = new ut();
    n.addChild(r);
    const o = /* @__PURE__ */ new Map(), a = /* @__PURE__ */ new Set(), l = /* @__PURE__ */ new Map(), c = nr();
    let h = false, d = false, u = false, p = 0;
    const f = () => u ? p : performance.now();
    let g = null, m = 0;
    const y = /* @__PURE__ */ new Map();
    let b = null, x = null;
    const _ = [], v = globalThis.__TRACE_LOGS === true, w = (R, B) => {
      globalThis.__ANIM_LOGS && console.log(`[anim t=${f().toFixed(0)}ms] ${R}`, B);
    };
    function C(R, B) {
      const z = !y.get(R), Y = {
        id: R,
        virtualMs: B
      };
      y.set(R, Y), z && (b == null ? void 0 : b(Y));
    }
    function E(R, B) {
      g === null && (g = R);
      const O = R + (B ? Ei : xc);
      O > m && (m = O);
    }
    function P(R, B) {
      if (R.length === 0) return;
      const O = f();
      for (const Y of R) Di(Y, c);
      const z = [
        ...R
      ].sort((Y, K) => {
        const nt = (Y.y ?? 0) - (K.y ?? 0);
        return nt !== 0 ? nt : (Y.x ?? 0) - (K.x ?? 0);
      });
      z.forEach((Y, K) => {
        const nt = O + K * B, ot = is(Y);
        if (a.has(ot)) return;
        a.add(ot), So(i, Y, nt, c), l.has(ot) || l.set(ot, nt), Zl(i, Y.x ?? 0, Y.y ?? 0);
        const ht = o.get(bc(Y.x ?? 0, Y.y ?? 0));
        ht && ht.fadeOutStartMs === null && (ht.fadeOutStartMs = nt, E(nt, false));
      }), E(O + (z.length - 1) * B, true);
    }
    function T(R, B, O, z, Y, K) {
      const nt = bc(R, B), ot = o.get(nt);
      ot && ot.specKey === Y || (u0(i, R, B, O, z, K, Y), ot || o.set(nt, {
        specKey: Y,
        fadeStartMs: K,
        fadeOutStartMs: null
      }), E(K, true));
    }
    function A(R) {
      const B = Ue(R.length, bn, _n);
      w("rows_placed", {
        count: R.length,
        stagger_ms: B,
        span_ms: R.length * B
      }), P(R, B);
    }
    function H(R) {
      const B = f(), O = ow(R.spec_key), z = R.tiles;
      if (z.length === 0) return;
      const Y = Ue(z.length, nw, sw);
      w("ghost_routed", {
        spec_key: R.spec_key,
        item: O,
        tile_count: z.length,
        span_ms: z.length * Y
      });
      for (let K = 0; K < z.length; K++) {
        const [nt, ot] = z[K];
        let ht = 0, gt = 0;
        K < z.length - 1 ? (ht = z[K + 1][0] - nt, gt = z[K + 1][1] - ot) : K > 0 && (ht = nt - z[K - 1][0], gt = ot - z[K - 1][1]), T(nt, ot, aw(ht, gt), O, R.spec_key, B + K * Y);
      }
    }
    function j(R) {
      const B = R.entities.length, O = Ue(B, bn, _n);
      w("committed", {
        source: "spec",
        count: B,
        span_ms: B * O
      }), P(R.entities, O);
    }
    function D(R) {
      const B = f(), O = R.zone_x, z = R.zone_y, Y = R.zone_x + R.zone_w - 1, K = R.zone_y + R.zone_h - 1;
      for (const [ht, gt] of o.entries()) {
        const [I, G] = ht.split(",").map(Number);
        I < O || I > Y || G < z || G > K || gt.fadeOutStartMs === null && (gt.fadeOutStartMs = B, E(B, false), Zl(i, I, G));
      }
      for (const ht of _) ht.clusterId === R.cluster_id && (ht.cleared = true);
      for (let ht = R.zone_y; ht <= K; ht++) for (let gt = R.zone_x; gt <= Y; gt++) {
        const I = f0(i, gt, ht);
        for (const G of I) a.delete(G);
      }
      const nt = R.entities.length, ot = Ue(nt, iw, rw);
      w("junction", {
        cluster_id: R.cluster_id,
        zone: `${R.zone_x},${R.zone_y}+${R.zone_w}x${R.zone_h}`,
        count: nt,
        span_ms: nt * ot
      }), P(R.entities, ot);
    }
    function N(R) {
      if (d = true, R.phase === "rows_placed") {
        A(R.entities);
        return;
      }
      if (R.phase !== "lanes_planned") {
        if (R.phase === "bus_routed") {
          const B = R.entities.filter((O) => !a.has(is(O)));
          if (B.length > 0) {
            const O = Ue(B.length, bn, _n);
            P(B, O);
          }
          return;
        }
        if (R.phase === "poles_placed") {
          const B = R.entities.filter((O) => !a.has(is(O)));
          if (B.length > 0) {
            const O = Ue(B.length, bn, _n);
            P(B, O);
          }
          return;
        }
      }
    }
    function $(R) {
      const B = f();
      w("cluster_outline", {
        cluster_id: R.cluster_id,
        zone: `${R.zone_x},${R.zone_y}+${R.zone_w}x${R.zone_h}`,
        lifetime_ms: Is,
        fade_ms: Mi
      }), _.push({
        clusterId: R.cluster_id,
        x: R.zone_x,
        y: R.zone_y,
        w: R.zone_w,
        h: R.zone_h,
        startMs: B,
        cleared: false
      }), E(B, true);
    }
    const W = () => {
      if (h) return;
      const R = f();
      $d(i, R);
      for (const [B, O] of o.entries()) if (O.fadeOutStartMs !== null && R >= O.fadeOutStartMs) {
        const [z, Y] = B.split(",").map(Number);
        (R - O.fadeOutStartMs) / xc >= 1 && o.delete(B);
      }
      for (const B of i.ghostContainer.particleChildren) B.alpha < 0.5 && (B.alpha = Math.min(0.5, B.alpha + 16 / Ei));
      r.clear();
      for (let B = _.length - 1; B >= 0; B--) {
        const O = _[B], z = R - O.startMs;
        if (!(z < 0)) if (O.cleared || z >= Is) {
          const Y = O.cleared ? Math.max(z, Is - Mi) : z;
          if (Y >= Is) {
            _.splice(B, 1);
            continue;
          }
          const K = Math.max(0, 1 - (Y - (Is - Mi)) / Mi);
          r.rect(O.x * k, O.y * k, O.w * k, O.h * k), r.stroke({
            width: 2,
            color: 4508927,
            alpha: 0.9 * K
          });
        } else r.rect(O.x * k, O.y * k, O.w * k, O.h * k), r.stroke({
          width: 2,
          color: 4508927,
          alpha: 0.9
        });
      }
    };
    t.ticker.add(W), sr();
    let X = true;
    w("streaming_start", {});
    function U(R, B) {
      if (h || u) return;
      b = B ?? null, v && console.log(`[stream t=${f().toFixed(0)}] ${R.phase}`, "data" in R ? R.data : void 0);
      const O = f();
      switch (g === null && (g = O), R.phase) {
        case "PhaseSnapshot": {
          const z = R.data;
          z.phase === "rows_placed" && C("machines", O), z.phase === "poles_placed" && C("poles", O), N(z);
          break;
        }
        case "GhostSpecRouted":
          C("ghost_routes", O), H(R.data);
          break;
        case "GhostSpecCommitted":
          C("committed_routes", O), j(R.data);
          break;
        case "JunctionCommitted":
          C("junctions", O), D(R.data);
          break;
        case "GhostClusterSolved":
          $(R.data);
          break;
        case "TrunkBeltCommitted": {
          const z = R.data, Y = z.entities.length, K = Ue(Y, bn, _n);
          w("committed", {
            source: "trunk",
            count: Y,
            span_ms: Y * K
          }), P(z.entities, K);
          break;
        }
        case "BalancerCommitted": {
          const z = R.data, Y = z.entities.length, K = Ue(Y, bn, _n);
          w("committed", {
            source: "balancer",
            count: Y,
            span_ms: Y * K
          }), P(z.entities, K);
          break;
        }
        case "OutputMergerCommitted": {
          const z = R.data, Y = z.entities.length, K = Ue(Y, bn, _n);
          w("committed", {
            source: "merger",
            count: Y,
            span_ms: Y * K
          }), P(z.entities, K);
          break;
        }
        case "PolesCommitted": {
          const z = R.data, Y = z.entities.length, K = Ue(Y, bn, _n);
          w("committed", {
            source: "poles",
            count: Y,
            span_ms: Y * K
          }), P(z.entities, K);
          break;
        }
      }
      b = null;
    }
    return {
      onEvent: U,
      hasCommittedEntities: () => d,
      cancel() {
        h || (h = true, t.ticker.remove(W), X && (Dn(), X = false), i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), x = null, Ze());
      },
      finish(R) {
        t.ticker.remove(W), X && (Dn(), X = false), p0(i), w("streaming_finish", {
          entity_count: i.count(),
          latest_fade_end_ms: m
        });
        const B = g ?? 0, O = [];
        for (const { particle: K, iconParticle: nt, revealAt: ot } of Ql()) O.push({
          kind: "particle",
          particle: K,
          iconParticle: nt,
          revealAt: ot
        });
        const z = R.entities.filter((K) => !a.has(is(K)));
        if (z.length > 0) {
          for (const nt of z) Di(nt, c);
          for (const nt of z) So(i, nt, B, c), a.add(is(nt));
          const K = new Set(O.map((nt) => nt.particle));
          for (const { particle: nt, iconParticle: ot, revealAt: ht } of Ql()) K.has(nt) || O.push({
            kind: "particle",
            particle: nt,
            iconParticle: ot,
            revealAt: ht
          });
        }
        const Y = m0(i, c);
        if (Y.size > 0) for (const K of O) {
          const nt = Y.get(K.particle);
          nt && (K.particle = nt);
        }
        return x = O, u = true, p = m, Z(p), x0(R, t);
      },
      seekTo(R) {
        if (h || x === null) return;
        const B = g ?? 0, O = Math.max(m, B);
        p = Math.min(O, Math.max(B, R)), w("scrub", {
          virtualMs: p
        }), Z(p);
      },
      getTimeRange() {
        return {
          firstMs: g ?? 0,
          lastMs: m
        };
      },
      getMilestones() {
        return Array.from(y.values()).sort((R, B) => R.virtualMs - B.virtualMs);
      }
    };
    function Z(R) {
      if (x !== null) {
        for (const B of x) {
          const O = R - B.revealAt, z = O <= 0 ? 0 : O >= Ei ? 1 : O / Ei;
          B.particle.alpha = z, B.iconParticle && (B.iconParticle.alpha = z);
        }
        Ze();
      }
    }
  }
  const cw = 0.03, hw = 200, jr = [
    "machines",
    "ghost_routes",
    "committed_routes",
    "junctions",
    "poles",
    "optimizing"
  ], dw = {
    machines: "Machines",
    ghost_routes: "Belt routes",
    committed_routes: "Belts placed",
    junctions: "Crossings",
    poles: "Power poles",
    optimizing: "Optimizing"
  };
  function uw(n, t) {
    const e = document.createElement("div");
    e.className = "timeline-scrubber";
    const s = document.createElement("div");
    s.className = "ts-chips", e.appendChild(s);
    const i = document.createElement("div");
    i.className = "ts-bar", e.appendChild(i);
    const r = document.createElement("div");
    r.className = "ts-track", i.appendChild(r);
    const o = document.createElement("div");
    o.className = "ts-fill", i.appendChild(o);
    const a = document.createElement("div");
    a.className = "ts-thumb", i.appendChild(a), n.appendChild(e);
    const l = /* @__PURE__ */ new Map();
    for (const D of jr) {
      const N = document.createElement("div");
      N.className = "ts-chip", N.dataset.milestone = D, N.textContent = dw[D], D === "optimizing" && (N.style.display = "none"), s.appendChild(N), l.set(D, N);
    }
    let c = false, h = null, d = [];
    const u = /* @__PURE__ */ new Set();
    let p = null;
    function f(D) {
      o.style.width = `${D * 100}%`, a.style.left = `${D * 100}%`;
    }
    function g(D) {
      var _a2, _b2;
      p !== D && (p && ((_a2 = l.get(p)) == null ? void 0 : _a2.classList.remove("ts-chip--active")), D && ((_b2 = l.get(D)) == null ? void 0 : _b2.classList.add("ts-chip--active")), p = D);
    }
    function m(D, N) {
      var _a2;
      if (c) return;
      const $ = D.id;
      u.add($), (_a2 = l.get($)) == null ? void 0 : _a2.classList.add("ts-chip--reached"), g($);
      const W = Math.max(1, N.lastMs - N.firstMs), X = (D.virtualMs - N.firstMs) / W;
      f(Math.min(1, Math.max(0, X))), e.classList.add("ts-visible");
    }
    function y(D) {
      return h ? h.firstMs + D * (h.lastMs - h.firstMs) : 0;
    }
    function b(D) {
      for (const N of d) if (Math.abs(D - N.frac) < cw) return {
        frac: N.frac,
        snapped: true
      };
      return {
        frac: D,
        snapped: false
      };
    }
    function x(D) {
      if (!h) return;
      const N = i.getBoundingClientRect(), $ = (D - N.left) / N.width, W = Math.min(1, Math.max(0, $)), { frac: X, snapped: U } = b(W);
      f(X), U ? a.classList.add("ts-thumb--snapped") : a.classList.remove("ts-thumb--snapped"), t(y(X));
    }
    let _ = null, v = null;
    function w(D) {
      if (!c || !h) return;
      D.preventDefault();
      try {
        i.setPointerCapture(D.pointerId);
      } catch {
      }
      const N = (W) => x(W.clientX), $ = (W) => {
        _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), _ = null, v = null, a.classList.remove("ts-thumb--snapped");
      };
      _ = N, v = $, document.addEventListener("pointermove", N), document.addEventListener("pointerup", $, {
        once: true
      }), x(D.clientX);
    }
    i.addEventListener("pointerdown", w);
    function C(D, N) {
      const $ = D.lastMs - D.firstMs;
      if ($ < hw || N.length === 0) {
        A();
        return;
      }
      c = true, h = D, e.classList.add("ts-scrub-mode"), e.classList.add("ts-visible"), d = N.map((W) => ({
        id: W.id,
        frac: (W.virtualMs - D.firstMs) / $
      })), s.style.justifyContent = "flex-start", s.style.position = "relative";
      for (const W of l.values()) W.style.position = "absolute", W.style.transform = "translateX(-50%)";
      for (const W of jr) {
        const X = l.get(W);
        if (!X) continue;
        const U = d.find((Z) => Z.id === W);
        U ? (X.style.left = `${U.frac * 100}%`, X.style.display = "", X.classList.add("ts-chip--reached")) : X.style.display = "none";
      }
      P(), requestAnimationFrame(T), f(1), g(null);
    }
    let E = null;
    function P() {
      if (E && E.remove(), !h) return;
      const D = document.createElement("div");
      D.className = "ts-ticks";
      for (const N of d) {
        const $ = document.createElement("div");
        $.className = "ts-tick", $.style.left = `${N.frac * 100}%`, D.appendChild($);
      }
      i.appendChild(D), E = D;
    }
    function T() {
      if (!c) return;
      const D = 6, N = s.clientWidth;
      if (N <= 0) return;
      const $ = jr.map((X) => {
        var _a2;
        const U = l.get(X);
        if (!U || U.style.display === "none") return null;
        const Z = X === "optimizing" ? 1 : ((_a2 = d.find((R) => R.id === X)) == null ? void 0 : _a2.frac) ?? 0;
        return {
          el: U,
          originalFrac: Z
        };
      }).filter((X) => X !== null);
      let W = -1 / 0;
      for (const { el: X, originalFrac: U } of $) {
        const Z = X.offsetWidth / 2, R = U * N, B = W + Z + D, O = Math.max(R, B);
        X.style.left = `${O / N * 100}%`, W = O + Z;
      }
    }
    function A() {
      c = false, h = null, d = [], u.clear(), p = null, E && (E.remove(), E = null), e.classList.remove("ts-visible", "ts-scrub-mode"), o.style.width = "0", a.style.left = "0", a.classList.remove("ts-thumb--snapped"), s.style.justifyContent = "space-between", s.style.position = "";
      for (const [D, N] of l) N.style.position = "", N.style.transform = "", N.style.left = "", N.style.display = D === "optimizing" ? "none" : "", N.classList.remove("ts-chip--reached", "ts-chip--active", "ts-chip--in-progress");
    }
    function H(D) {
      const N = l.get("optimizing");
      if (N) {
        if (N.classList.remove("ts-chip--in-progress", "ts-chip--reached"), D === "idle") {
          N.style.display = "none";
          return;
        }
        N.style.display = "", e.classList.add("ts-visible"), c && (N.style.position = "absolute", N.style.transform = "translateX(-50%)", N.style.left = "100%", requestAnimationFrame(T)), D === "active" ? N.classList.add("ts-chip--in-progress") : D === "done" && N.classList.add("ts-chip--reached");
      }
    }
    function j() {
      _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), i.removeEventListener("pointerdown", w), e.remove();
    }
    return {
      noteMilestone: m,
      arm: C,
      markOptimizeState: H,
      reset: A,
      destroy: j
    };
  }
  const pw = `
.fucktorio-busy {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(20, 20, 20, 0.82);
  color: #d4d4d4;
  padding: 6px 12px;
  border: 1px solid #2a2a2a;
  border-radius: 4px;
  font: 11px 'JetBrains Mono', 'Consolas', monospace;
  letter-spacing: 0.5px;
  pointer-events: none;
  z-index: 20;
  opacity: 0;
  transition: opacity 0.15s ease;
}
.fucktorio-busy.visible { opacity: 1; }
.fucktorio-busy-spin {
  width: 12px;
  height: 12px;
  border: 2px solid #2a2a2a;
  border-top-color: #569cd6;
  border-radius: 50%;
  animation: fucktorio-busy-spin 0.7s linear infinite;
  display: inline-block;
}
@keyframes fucktorio-busy-spin { to { transform: rotate(360deg); } }
`;
  function fw() {
    if (document.getElementById("fucktorio-busy-style")) return;
    const n = document.createElement("style");
    n.id = "fucktorio-busy-style", n.textContent = pw, document.head.appendChild(n);
  }
  const mw = 120;
  function gw(n) {
    fw();
    const t = document.createElement("div");
    t.className = "fucktorio-busy";
    const e = document.createElement("span");
    e.className = "fucktorio-busy-spin", t.appendChild(e);
    const s = document.createElement("span");
    s.textContent = "computing\u2026", t.appendChild(s), n.appendChild(t);
    let i = null;
    H0((r) => {
      r > 0 ? i === null && !t.classList.contains("visible") && (i = setTimeout(() => {
        t.classList.add("visible"), i = null;
      }, mw)) : (i !== null && (clearTimeout(i), i = null), t.classList.remove("visible"));
    });
  }
  function je(n, t) {
    return n.filter((e) => e.phase === t);
  }
  const In = "color:#9cdcfe;font-weight:bold", ae = "color:#888", ye = "color:#e0e0e0", ss = "color:#6a6", Pi = "color:#ffaa00", Vr = "color:#f66", Yr = "color:#c586c0";
  function yw(n) {
    var _a2;
    const t = Array.isArray(n.trace) ? n.trace : [];
    if (t.length === 0) return;
    const e = je(t, "PhaseTime"), s = je(t, "SatInvocation"), i = je(t, "JunctionSolved"), r = je(t, "JunctionGrowthCapped"), o = je(t, "GhostClusterSolved"), a = je(t, "GhostRoutingComplete"), l = je(t, "RegionWalkerVeto"), c = je(t, "JunctionGrowthIteration"), h = je(t, "NegotiateComplete"), d = je(t, "ValidationCompleted"), u = e.reduce((b, x) => b + x.data.duration_ms, 0), p = s.reduce((b, x) => b + x.data.solve_time_us, 0), f = s.filter((b) => b.data.satisfied).length, g = ((_a2 = n.entities) == null ? void 0 : _a2.length) ?? 0, m = r.length > 0 ? Pi : ss, y = r.length > 0 ? ` \xB7 ${r.length} capped` : "";
    if (console.log(`%c\u25B6 layout %c${n.width}\xD7${n.height}  %c${g} entities  %c${u}ms  %cSAT ${Math.round(p / 1e3)}ms (${s.length}\xD7)%c${y}`, In, ye, ae, ye, Yr, m), console.groupCollapsed("%c  \u21B3 breakdown", ae), e.length > 0) {
      const b = [
        ...e
      ].sort((x, _) => _.data.duration_ms - x.data.duration_ms);
      console.log(`%cphases%c ${u}ms total`, In, ae);
      for (const x of b) {
        const _ = x.data, v = u > 0 ? _.duration_ms / u * 100 : 0, w = Math.max(1, Math.round(v / 100 * 24)), C = "\u2588".repeat(w);
        console.log(`  %c${_.phase.padEnd(18)}%c ${String(_.duration_ms).padStart(5)}ms  %c${C}%c ${v.toFixed(1)}%`, ae, ye, Yr, ae);
      }
    }
    if (s.length > 0) {
      const b = p / 1e3, x = u > 0 ? b / u * 100 : 0, _ = p / s.length, v = [
        ...s
      ].sort((C, E) => E.data.solve_time_us - C.data.solve_time_us)[0], w = [
        ...s
      ].sort((C, E) => E.data.zone_w * E.data.zone_h - C.data.zone_w * C.data.zone_h)[0];
      console.log(`%cSAT%c ${s.length} invocations \xB7 ${b.toFixed(1)}ms (%c${x.toFixed(1)}%%%c of total)`, In, ye, Yr, ye), console.log(`  %csatisfied%c ${f}  %cunsat%c ${s.length - f}  %cavg%c ${(_ / 1e3).toFixed(2)}ms`, ae, ss, ae, Vr, ae, ye), v && console.log(`  %cslowest call%c ${(v.data.solve_time_us / 1e3).toFixed(1)}ms \u2014 %c${v.data.zone_w}\xD7${v.data.zone_h} @ (${v.data.zone_x},${v.data.zone_y}), ${v.data.variables} vars, ${v.data.clauses} clauses`, ae, ye, ae), w && w !== v && console.log(`  %cbiggest zone%c ${w.data.zone_w}\xD7${w.data.zone_h} @ (${w.data.zone_x},${w.data.zone_y}) \u2014 ${w.data.variables} vars`, ae, ye);
    }
    if (o.length > 0 || i.length > 0 || r.length > 0) {
      if (console.log("%cjunctions", In), console.log(`  %cclusters%c ${o.length}  %csolved%c ${i.length}  %ccapped%c ${r.length}  %cvetoes%c ${l.length}`, ae, ye, ae, ss, ae, r.length > 0 ? Pi : ye, ae, ye), c.length > 0) {
        const b = /* @__PURE__ */ new Map();
        for (const _ of c) {
          const v = `${_.data.seed_x},${_.data.seed_y}`;
          b.set(v, Math.max(b.get(v) ?? 0, _.data.iter));
        }
        const x = [
          ...b.entries()
        ].sort((_, v) => v[1] - _[1])[0];
        x && x[1] > 0 && console.log(`  %chardest%c junction at (${x[0]}) needed ${x[1] + 1} growth iters`, ae, ye);
      }
      if (r.length > 0) for (const b of r) console.log(`    %c\u26A0 capped at (${b.data.tile_x},${b.data.tile_y})%c \u2014 ${b.data.reason}, ${b.data.region_tiles} tiles after ${b.data.iters} iters`, Pi, ae);
    }
    if (a.length > 0) {
      const b = a[0].data, x = b.unroutable_count > 0 ? Vr : ss;
      console.log(`%cghost router%c ${b.entity_count} routed entities, ${b.cluster_count} clusters, max cluster ${b.max_cluster_tiles} tiles  %c${b.unroutable_count} unroutable`, In, ye, x);
    }
    if (h.length > 0) {
      const b = h[0].data;
      console.log(`%cA* negotiate%c ${b.specs} specs, ${b.iterations} iters, ${b.duration_ms}ms`, In, ye);
    }
    if (d.length > 0) {
      const b = d[0].data, x = b.error_count > 0 ? Vr : ss, _ = b.warning_count > 0 ? Pi : ss;
      console.log(`%cvalidation  %c${b.error_count} errors  %c${b.warning_count} warnings`, In, x, _);
    }
    console.groupEnd();
  }
  const xw = [
    "assembling-machine-1",
    "assembling-machine-2",
    "assembling-machine-3",
    "electric-furnace",
    "steel-furnace",
    "stone-furnace",
    "chemical-plant",
    "oil-refinery",
    "centrifuge",
    "lab",
    "rocket-silo",
    "foundry",
    "electromagnetic-plant",
    "cryogenic-plant",
    "biochamber",
    "biolab",
    "recycler",
    "crusher",
    "beacon",
    "storage-tank",
    "electric-mining-drill"
  ];
  async function bw() {
    await Hd();
    const n = nb();
    await Bx(xw);
    const t = document.getElementById("app"), e = window.location.hash, s = new URLSearchParams(window.location.search), i = s.has("item") || s.has("rate") || s.has("machine") || s.has("in") || s.has("belt");
    if (!(e.startsWith("#/layout") || s.has("generator") || i)) {
      const o = document.createElement("div");
      t.appendChild(o), cb(o, n, {
        onOpenGenerator: () => {
          o.remove(), _c(n), window.history.replaceState({}, "", "#/layout");
        }
      });
      return;
    }
    _c(n);
  }
  async function _c(n) {
    const t = document.getElementById("canvas-container");
    if (!t) throw new Error("Missing #canvas-container element");
    const e = document.getElementById("app");
    e.style.display = "flex";
    const s = document.getElementById("sidebar");
    s && (s.style.display = ""), t.style.display = "";
    const { app: i, viewport: r, requestRender: o, beginAnimating: a, endAnimating: l } = await Yx(t), c = Xx(r);
    let h = false;
    Ps(r, null), W_();
    const d = z_(t), { debugCb: u, colorCb: p, regionsCb: f, soloRegionsCb: g, ghostTilesCb: m, traceOverlayCb: y } = d;
    _i(p.checked);
    const b = () => {
      globalThis.__ANIM_LOGS = u.checked;
    };
    u.addEventListener("change", b), b();
    const x = H_(t), _ = f_();
    let v = null;
    const w = C_(t, r, {
      onChange: (F) => {
        if (_.update(F), F) {
          P.alpha = C.isActive() ? 0.2 : 0.35;
          const tt = F.iter.bbox;
          v = {
            bboxX: tt.x,
            bboxY: tt.y,
            bboxW: tt.w,
            bboxH: tt.h
          };
        } else P.alpha = 1, v = null, C.isActive() && C.exit();
        o();
      },
      onEditRequested: (F) => {
        P.alpha = 0.2, C.enter(F), o();
      }
    }), C = F_({
      viewport: r,
      canvas: i.canvas,
      engine: n,
      jd: w,
      satZoneOverlayLayer: _.layer
    });
    Pb(t, (F) => M.load(F));
    function E(F) {
      P.removeChildren();
      const tt = Rd();
      return tt.attachTo(P), g0(F, tt, i);
    }
    const P = new zt();
    P.isRenderGroup = true, P.eventMode = "none", r.addChild(P);
    const T = new zt();
    T.eventMode = "none", r.addChild(T), r.addChild(_.layer);
    const A = new ut();
    A.label = "pin-highlight", r.addChild(A), x.onPinChange((F) => {
      if (A.clear(), F) {
        const tt = F.x * k, st = F.y * k;
        A.setStrokeStyle({
          width: 2,
          color: 8440063,
          alpha: 0.95
        }), A.rect(tt - 2, st - 2, k + 4, k + 4).stroke();
      }
      o();
    }), r.moveCenter(Ke / 2, Ke / 2);
    const H = (F) => {
    };
    let j = null;
    function D(F) {
      j = F, x.onHover(F, F == null ? void 0 : F.x, F == null ? void 0 : F.y);
    }
    let N = /* @__PURE__ */ new Map();
    function $(F) {
      const tt = /* @__PURE__ */ new Map();
      for (const st of F.entities) {
        const lt = st.x ?? 0, at = st.y ?? 0, Pt = tn[st.name];
        if (Pt) {
          const [It, Tt] = Pt;
          for (let Lt = 0; Lt < Tt; Lt++) for (let fe = 0; fe < It; fe++) tt.set(`${lt + fe},${at + Lt}`, st);
        } else if (ee.has(st.name)) {
          tt.set(`${lt},${at}`, st);
          const [It, Tt] = Zo(st.direction);
          tt.set(`${lt + It},${at + Tt}`, st);
        } else tt.set(`${lt},${at}`, st);
      }
      N = tt;
    }
    function W(F) {
      return {
        highlightItem: (tt) => {
          F.highlightItem(tt), o();
        },
        highlightBeltNetwork: (tt) => {
          F.highlightBeltNetwork(tt), o();
        },
        clearHighlight: () => {
          F.clearHighlight(), o();
        },
        chainKey: F.chainKey
      };
    }
    let X = false, U = null, Z = null, R = false, B = null;
    const O = {
      update() {
      },
      getPhaseIndex() {
        return -1;
      },
      reset() {
      }
    };
    function z() {
      var _a2;
      Z && (P.removeChild(Z), Z.destroy(), Z = null);
      const F = O.getPhaseIndex();
      if (u.checked && F >= 0, R && (B == null ? void 0 : B.cancel(), B = null, R = false, dt)) {
        const st = zs(dt, P, D, H);
        x.setHighlightController(W(st)), o();
      }
      if (!u.checked || !y.checked || !((_a2 = dt == null ? void 0 : dt.trace) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      const tt = dt.trace;
      Z = Rb(tt, dt.width ?? 0, dt.height ?? 0, P, (st) => {
        x.setTooltipOverride(st ? `<span style="color:#8af">TRACE</span> ${st}` : null);
      }), o();
    }
    let Y = null, K = null, nt = null;
    const ot = document.createElement("div");
    ot.className = "validation-badge", ot.style.display = "none", t.appendChild(ot);
    function ht(F) {
      if (!F || F.length === 0) {
        ot.style.display = "none";
        return;
      }
      const tt = F.filter((at) => at.severity === "Error").length, st = F.length - tt;
      let lt;
      tt > 0 && st > 0 ? lt = `\u26A0 ${tt} error${tt > 1 ? "s" : ""}, ${st} warning${st > 1 ? "s" : ""}` : tt > 0 ? lt = `\u26A0 ${tt} error${tt > 1 ? "s" : ""}` : lt = `\u26A0 ${st} warning${st > 1 ? "s" : ""}`, ot.textContent = lt, ot.classList.toggle("has-errors", tt > 0), ot.style.display = "block";
    }
    let gt = null, I = null, G = null, V = null, J = null;
    const xt = 1;
    function bt(F, tt) {
      const st = F * k + k / 2, lt = tt * k + k / 2;
      r.scale.x < xt && r.setZoom(xt, false), r.moveCenter(st, lt);
    }
    function wt(F) {
      var _a2, _b2;
      const tt = [];
      for (const st of F.regions ?? []) {
        if (st.kind !== "unresolved") continue;
        const lt = st.x + Math.floor(st.width / 2), at = st.y + Math.floor(st.height / 2), Pt = ((_b2 = (_a2 = st.ports) == null ? void 0 : _a2.find((It) => It.item)) == null ? void 0 : _b2.item) ?? "unknown";
        tt.push({
          severity: "Warning",
          category: `ghost-router \xB7 ${Pt}`,
          message: `unresolved crossing at (${lt}, ${at})`,
          x: lt,
          y: at
        });
      }
      for (const st of F.warnings ?? []) /^ghost router:.*unresolved crossings/i.test(st) || tt.push({
        severity: "Warning",
        category: "layout",
        message: st,
        x: void 0,
        y: void 0
      });
      return tt;
    }
    function Dt() {
      if (Y && (P.removeChild(Y), Y.destroy(), Y = null), dt && !K && nt !== dt) {
        const lt = dt;
        nt = lt, n.validateLayout(lt, pe).then((at) => {
          dt === lt && (K = at, nt = null, Dt(), $t(lt));
        }).catch(() => {
          dt === lt && (K = [], nt = null, Dt(), $t(lt));
        });
      }
      const F = dt ? wt(dt) : [], tt = [
        ...K ?? [],
        ...F
      ];
      if (Ut == null ? void 0 : Ut.updateValidation(tt, bt), ht(tt), !dt || tt.length === 0) {
        o();
        return;
      }
      Y = Ob(tt, P, (lt) => {
        x.setTooltipOverride(lt ? `<span style="color:#f44">VALIDATION</span> ${lt}` : null);
      }).layer, o();
    }
    function kt() {
      if (J && (r.removeChild(J), J.destroy({
        children: true
      }), J = null), !u.checked || !m.checked || !dt) {
        o();
        return;
      }
      const F = v_(dt.trace);
      if (!F) {
        o();
        return;
      }
      J = F, r.addChildAt(J, 0), o();
    }
    function Ft() {
      var _a2;
      if (gt && (P.removeChild(gt), gt.destroy(), gt = null), G && (P.removeChild(G), G.destroy(), G = null), I = null, V = null, !u.checked || !(f == null ? void 0 : f.checked) || !dt) {
        o();
        return;
      }
      if (dt.regions && dt.regions.length > 0) {
        const F = Vb(dt);
        gt = F.layer, I = F.hitTest, P.addChild(gt);
      }
      if ((_a2 = dt.trace) == null ? void 0 : _a2.length) {
        const F = Jb(dt.trace);
        if (F.length > 0) {
          const tt = s_(F);
          G = tt.layer, V = tt.hitTest, P.addChild(G);
        }
      }
      o();
    }
    const ct = document.createElement("div");
    ct.style.cssText = "position:absolute;bottom:34px;left:8px;background:rgba(0,0,0,0.8);color:#e0e0e0;font:11px monospace;padding:6px 8px;border-radius:3px;border:1px solid #00e0a0;z-index:10;display:none;min-width:200px", t.appendChild(ct);
    const vt = document.createElement("div");
    vt.style.cssText = "color:#00e0a0;margin-bottom:4px", ct.appendChild(vt);
    const Jt = document.createElement("textarea");
    Jt.placeholder = "Add a note\u2026", Jt.rows = 2, Jt.style.cssText = "width:100%;box-sizing:border-box;background:#2a2a2a;color:#e0e0e0;border:1px solid #555;border-radius:2px;font:11px monospace;resize:vertical;margin-bottom:4px", ct.appendChild(Jt);
    const ce = document.createElement("div");
    ce.style.cssText = "color:#777", ce.textContent = "Ctrl+C to copy JSON", ct.appendChild(ce);
    let Te = false, dt = null, pe = null, Rt = null, Zt = null, Ht = null;
    const Qt = uw(t, (F) => Ht == null ? void 0 : Ht.seekTo(F));
    gw(t);
    const M = Y_({
      sidebarEl: document.getElementById("sidebar"),
      getSidebarCtrl: () => Ut,
      renderLayoutOnCanvas: ft,
      setCachedValidationIssues: (F) => {
        K = F;
      },
      updateValidationOverlay: Dt,
      panToTile: bt,
      onDebugEnable: () => d.setDebugEnabled(true),
      onClear: () => {
        M.clear(), P.removeChildren(), T.removeChildren(), x.clearPin(), x.setTileContext(null), dt = null, pe = null, N = /* @__PURE__ */ new Map(), K = null, Ps(r, null), r.moveCenter(Ke / 2, Ke / 2), ht(null), Ut == null ? void 0 : Ut.updateValidation([], bt), w.close();
      }
    });
    function S(F) {
      F.length === 0 ? (ct.style.display = "none", Jt.value = "") : (vt.textContent = `${F.length} entit${F.length === 1 ? "y" : "ies"} selected`, ct.style.display = "block");
    }
    async function L(F) {
      if (Te || !(F.regions ?? []).some((Bt) => Bt.kind === "crossing_zone")) return;
      Te = true, Qt.markOptimizeState("active"), Rt && (Rt.destroy(), Rt = null);
      const st = (Bt) => Bt === "transport-belt" || Bt === "fast-transport-belt" || Bt === "express-transport-belt" || Bt === "underground-belt" || Bt === "fast-underground-belt" || Bt === "express-underground-belt", lt = [], at = 130, Pt = 90, It = 520;
      let Tt = 0, Lt = -1, fe = 0, ge = false, Tn = false, qn = null;
      const tu = (Bt) => {
        if (!dt) return;
        const He = Bt.zone_x, Be = Bt.zone_y, Kn = He + Bt.zone_w, ir = Be + Bt.zone_h, nu = (ws) => {
          const da = ws.x ?? 0, ua = ws.y ?? 0;
          return da >= He && da < Kn && ua >= Be && ua < ir;
        };
        dt.entities = dt.entities.filter((ws) => !(nu(ws) && st(ws.name))).concat(Bt.entities), ew(i, r, {
          x: He,
          y: Be,
          w: Bt.zone_w,
          h: Bt.zone_h
        }), E(dt), o();
      }, eu = () => ge && lt.length === 0, ha = (Bt) => {
        if (!Tn) {
          for (; lt.length > 0; ) {
            const He = lt[0], Be = He.imp.region_id === Lt, Kn = Be ? Math.min(It, fe * Pt) : 0, ir = at + Kn;
            if (Bt - Tt < ir) break;
            lt.shift(), tu(He.imp), Be ? fe += 1 : (fe = 1, Lt = He.imp.region_id), Tt = Bt;
            break;
          }
          if (eu()) {
            qn = null;
            return;
          }
          qn = requestAnimationFrame(ha);
        }
      };
      qn = requestAnimationFrame(ha);
      try {
        const Bt = await n.optimizeAllRegions(F, {
          perRegionBudgetMs: 800,
          onImprovement: (Be) => {
            Be.iter !== 0 && lt.push({
              imp: Be
            });
          }
        });
        ge = true, await new Promise((Be) => {
          const Kn = () => {
            lt.length === 0 ? Be() : requestAnimationFrame(Kn);
          };
          Kn();
        }), dt = Bt, $(Bt), window.__layout = Bt;
        const He = E(Bt);
        x.setHighlightController(W(He)), o();
      } catch (Bt) {
        (Bt instanceof Error ? Bt.message : String(Bt)).includes("superseded") || console.error("[auto-optimize] failed", Bt);
      } finally {
        Tn = true, ge = true, qn !== null && cancelAnimationFrame(qn), Te = false, Qt.markOptimizeState("done"), dt && (Rt = ec(i.canvas, r, P, dt, S));
      }
    }
    i.canvas.addEventListener("pointermove", (F) => {
      const tt = i.canvas.getBoundingClientRect(), st = F.clientX - tt.left, lt = F.clientY - tt.top, at = r.toWorld(st, lt), Pt = Math.floor(at.x / k), It = Math.floor(at.y / k);
      x.setCursorTile(Pt, It);
      const Tt = N.get(`${Pt},${It}`) ?? null;
      Tt !== j && D(Tt), j || x.onHover(null, Pt, It);
    }), i.canvas.addEventListener("pointerleave", () => {
      x.setCursorTile(null), j && D(null);
    });
    const q = 4;
    let Q = null;
    i.canvas.addEventListener("pointerdown", (F) => {
      if (F.button !== 0 || F.shiftKey || F.altKey || F.ctrlKey || F.metaKey) {
        Q = null;
        return;
      }
      Q = {
        x: F.clientX,
        y: F.clientY,
        shifted: false
      };
    }), i.canvas.addEventListener("pointerup", (F) => {
      if (!Q) return;
      const tt = F.clientX - Q.x, st = F.clientY - Q.y;
      if (Q = null, Math.hypot(tt, st) > q || F.button !== 0 || F.shiftKey || F.altKey || F.ctrlKey || F.metaKey) return;
      const lt = i.canvas.getBoundingClientRect(), at = r.toWorld(F.clientX - lt.left, F.clientY - lt.top), Pt = Math.floor(at.x / k), It = Math.floor(at.y / k);
      if (!f.checked) {
        const ge = j && j.x === Pt && j.y === It ? j : null;
        if (!ge) return;
        x.pinTile(ge, Pt, It);
        return;
      }
      const Tt = (V == null ? void 0 : V(at.x, at.y)) ?? null;
      if (Tt) {
        w.open(Tt, dt == null ? void 0 : dt.trace);
        return;
      }
      if (v) {
        const ge = at.x / k, Tn = at.y / k;
        if (!(ge >= v.bboxX && Tn >= v.bboxY && ge < v.bboxX + v.bboxW && Tn < v.bboxY + v.bboxH)) {
          w.close();
          return;
        }
      }
      const Lt = (I == null ? void 0 : I(at.x, at.y)) ?? null;
      if (Lt) {
        const ge = (Lt.region.x + Lt.region.width / 2) * k, Tn = (Lt.region.y + Lt.region.height / 2) * k;
        r.moveCenter(ge, Tn);
      }
      const fe = x.getPinnedTile();
      if (fe && fe.x === Pt && fe.y === It) x.clearPin();
      else {
        const ge = j && j.x === Pt && j.y === It ? j : null;
        if (!ge) return;
        x.pinTile(ge, Pt, It);
      }
    }), document.addEventListener("keydown", (F) => {
      F.key === "Shift" && r.plugins.pause("drag");
    }), document.addEventListener("keyup", (F) => {
      F.key === "Shift" && r.plugins.resume("drag");
    }), window.addEventListener("blur", () => r.plugins.resume("drag"));
    function rt(F) {
      Zt == null ? void 0 : Zt.cancel(), Zt = null, Ht == null ? void 0 : Ht.cancel(), Ht = null, Qt.reset(), P.removeChildren(), T.removeChildren(), pe = F, Ps(r, F), F || r.moveCenter(Ke / 2, Ke / 2);
    }
    function pt() {
      Ht == null ? void 0 : Ht.cancel(), Qt.reset(), Ps(r, null), Ht = lw(P, i);
      let F = false, tt = null;
      return (st) => {
        if (st.phase === "PhaseSnapshot") {
          const at = st.data;
          at.width > 0 && at.height > 0 && (F || (r.fit(true, at.width * k * 1.15, at.height * k * 1.25), r.moveCenter(at.width * k / 2, at.height * k / 2), F = true), tt ? tt.resize(at.width + 2, at.height + 2) : tt = et(at.width + 2, at.height + 2));
        }
        Ht == null ? void 0 : Ht.onEvent(st, (at) => {
          Ht && Qt.noteMilestone(at, Ht.getTimeRange());
        });
      };
    }
    function et(F, tt) {
      let st = F, lt = tt, at = false;
      if (h) return Pn(c, st, lt), o(), at = true, {
        cancel: () => {
        },
        resize(Lt, fe) {
          Pn(c, Lt, fe), o();
        }
      };
      const Pt = 250, It = performance.now();
      c.alpha = 1, Pn(c, st, lt, 0);
      const Tt = () => {
        if (at) return;
        const Lt = Math.min(1, (performance.now() - It) / Pt);
        Pn(c, st, lt, Lt), o(), Lt >= 1 && (at = true, h = true, i.ticker.remove(Tt), l());
      };
      return i.ticker.add(Tt), a(), {
        cancel() {
          at || (at = true, h = true, i.ticker.remove(Tt), l(), Pn(c, st, lt), o());
        },
        resize(Lt, fe) {
          st = Lt, lt = fe, at && (Pn(c, st, lt), o());
        }
      };
    }
    function ft(F, tt) {
      _d(Ox(F.entities)), dt = F, tt && (pe = tt), $(F), window.__layout = F, yw(F), R = false, B == null ? void 0 : B.cancel(), B = null, Rt && (Rt.destroy(), Rt = null), Zt == null ? void 0 : Zt.cancel(), Zt = null, ct.style.display = "none", Jt.value = "", K = null, Ps(r, null);
      let st;
      if (Ht == null ? void 0 : Ht.hasCommittedEntities()) st = Ht.finish(F), Qt.arm(Ht.getTimeRange(), Ht.getMilestones());
      else if (Ht == null ? void 0 : Ht.cancel(), Ht = null, Qt.reset(), (Array.isArray(F.trace) ? F.trace : []).some((Tt) => Tt.phase === "PhaseSnapshot")) {
        const Tt = Q_(F, P, D, H, i);
        st = Tt.controller, Zt = Tt.handle;
      } else st = E(F);
      x.setHighlightController(W(st)), x.setTileContext(V_(F.trace)), x.clearPin(), Rt = ec(i.canvas, r, P, F, S), z(), Dt(), Ft(), kt(), E0(T, F, pe);
      const lt = F.width ?? 0, at = F.height ?? 0;
      if (Pn(c, lt + 2, at + 2), lt > 0 && at > 0) {
        const Pt = lt * 32, It = at * 32, Tt = 192;
        r.fit(true, Pt * 1.1, (It + Tt) * 1.2), r.moveCenter(Pt / 2, (It - Tt) / 2);
      }
      X && (P.alpha = 0.12), o(), requestAnimationFrame(() => {
        dt === F && $t(F);
      });
    }
    function $t(F) {
      dt !== F || nt === F || K === null || [
        ...K,
        ...wt(F)
      ].length > 0 || L(F);
    }
    document.addEventListener("keydown", (F) => {
      if (F.ctrlKey) {
        if (F.key === "c") {
          if (!Rt || Rt.getSelected().length === 0) return;
          F.preventDefault();
          const tt = (Ut == null ? void 0 : Ut.getParams()) ?? null, st = Rt.buildJson(tt, Jt.value.trim());
          navigator.clipboard.writeText(st).catch(() => {
          }), ce.textContent = "Copied!", setTimeout(() => {
            ce.textContent = "Ctrl+C to copy JSON";
          }, 2e3);
        } else if (F.key === "o") {
          F.preventDefault();
          const tt = document.createElement("input");
          tt.type = "file", tt.accept = ".fls", tt.addEventListener("change", async () => {
            var _a2;
            const st = (_a2 = tt.files) == null ? void 0 : _a2[0];
            if (st) try {
              const lt = await st.text(), at = await Jd(lt);
              M.load(at);
            } catch (lt) {
              alert(`Failed to load snapshot: ${lt}`);
            }
          }), tt.click();
        }
      }
    });
    const he = document.getElementById("sidebar");
    let Ut = null;
    if (he) {
      let F = function(Tt) {
        const Lt = document.createElement("button");
        return Lt.textContent = Tt, Lt.style.cssText = "flex:1;padding:10px 4px;background:none;border:none;border-bottom:2px solid transparent;color:#777;font:12px 'JetBrains Mono','Consolas',monospace;cursor:pointer;letter-spacing:0.5px;transition:all 0.15s", Lt;
      }, tt = function(Tt) {
        const Lt = Tt === "generate";
        Pt.style.display = Lt ? "flex" : "none", It.style.display = Lt ? "none" : "flex", lt.style.borderBottomColor = Lt ? "#569cd6" : "transparent", lt.style.color = Lt ? "#d4d4d4" : "#777", at.style.borderBottomColor = Lt ? "transparent" : "#569cd6", at.style.color = Lt ? "#777" : "#d4d4d4";
      };
      const st = document.createElement("div");
      st.style.cssText = "display:flex;border-bottom:1px solid #2a2a2a;background:#141414;flex-shrink:0";
      const lt = F("Generate"), at = F("Corpus");
      st.appendChild(lt), st.appendChild(at);
      const Pt = document.createElement("div");
      Pt.style.cssText = "flex:1;overflow:hidden;display:flex;flex-direction:column;";
      const It = document.createElement("div");
      It.style.cssText = "flex:1;overflow:hidden;display:none;flex-direction:column;", he.style.cssText += ";display:flex;flex-direction:column;padding:0;overflow:hidden;", he.appendChild(st), he.appendChild(Pt), he.appendChild(It), lt.onclick = () => tt("generate"), at.onclick = () => tt("corpus"), tt("generate"), Ut = N0(Pt, n, {
        renderGraph: rt,
        renderLayout: ft,
        startStreaming: pt
      }), u.addEventListener("change", () => {
        z(), Dt(), Ft(), kt();
      }), m.addEventListener("change", () => {
        kt();
      }), y.addEventListener("change", () => {
        z();
      }), p.addEventListener("change", () => {
        _i(p.checked), dt && ft(dt);
      }), f.addEventListener("change", () => {
        Ft();
      }), g.addEventListener("change", () => {
        const Tt = () => o();
        g.checked ? (X = true, U = {
          colorChecked: p.checked,
          regionsChecked: f.checked,
          entityAlpha: P.alpha
        }, f.checked || (f.checked = true, Ft()), p.checked && (p.checked = false, _i(false), dt && ft(dt)), P.alpha = 0.12, Ft(), Tt()) : (X = false, U && (P.alpha = U.entityAlpha, f.checked !== U.regionsChecked && (f.checked = U.regionsChecked, Ft()), p.checked !== U.colorChecked && (p.checked = U.colorChecked, _i(p.checked), dt && ft(dt)), U = null), Tt());
      }), rb(It, ft);
    }
  }
  bw().catch((n) => {
    console.error("Failed to initialize app:", n);
  });
})();
export {
  Iu as $,
  Os as A,
  jn as B,
  rn as C,
  Bc as D,
  Qe as E,
  it as F,
  ti as G,
  ut as H,
  fs as I,
  Dg as J,
  Bm as K,
  Ki as L,
  _t as M,
  Tm as N,
  fn as O,
  Ji as P,
  jc as Q,
  Nt as R,
  Xi as S,
  Ct as T,
  Mt as U,
  uh as V,
  so as W,
  Ot as X,
  Nn as Y,
  Hs as Z,
  At as _,
  __tla,
  Oi as a,
  zt as a0,
  If as a1,
  _s as a2,
  ja as a3,
  Ii as a4,
  Bo as a5,
  ch as a6,
  ke as a7,
  Fo as a8,
  am as a9,
  lm as aA,
  hm as aB,
  ym as aC,
  mm as aD,
  xm as aE,
  cm as aa,
  gm as ab,
  pm as ac,
  yu as ad,
  bm as ae,
  _e as af,
  Lc as ag,
  Se as ah,
  ji as ai,
  Ea as aj,
  De as ak,
  by as al,
  Xu as am,
  Pa as an,
  fr as ao,
  Ia as ap,
  qu as aq,
  Oc as ar,
  Zi as as,
  Gh as at,
  ph as au,
  Qs as av,
  qp as aw,
  Up as ax,
  Sh as ay,
  Ff as az,
  Oo as b,
  Hn as c,
  Et as d,
  yo as e,
  xo as f,
  yi as g,
  Gt as h,
  Le as i,
  Uh as j,
  jh as k,
  Pe as l,
  ww as m,
  ly as n,
  qg as o,
  te as p,
  St as q,
  zs as r,
  Bi as s,
  $f as t,
  Tp as u,
  qt as v,
  Vt as w,
  re as x,
  Kt as y,
  jt as z
};
