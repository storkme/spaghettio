const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/browserAll-i-drEUnr.js","assets/webworkerAll-Ds81m8HJ.js","assets/Filter-Dv8SmpOO.js","assets/WebGPURenderer-CdejW-a0.js","assets/BufferResource-E3Pb1RH3.js","assets/RenderTargetSystem-C5lAMLAy.js","assets/WebGLRenderer-BTBXxSvE.js","assets/CanvasRenderer-CCJ0mBY9.js"])))=>i.map(i=>d[i]);
let yh, ea, Ji, Ut, om, rn, Zp, fi, Zn, Cs, le, ee, Kt, qs, ah, vt, ht, ft, Ht, ur, S, pe, fy, ag, fr, Zm, yn, mr, ir, Pt, Fh, vo, Gt, Yn, ni, Tt, sp, sm, Ps, ud, Wh, pi, vf, yf, Kh, cm, $m, Om, Hm, zm, Um, dl, Vi, Zo, Bh, Me, ta, Lm, Bm, Dm, Wm, Hu, jm, ve, rh, ke, cr, Xa, Ye, jy, _p, Ka, Rr, Ja, wp, lh, gr, Fn, _r, gv, Zi, Qo, Mt, Ct, Oo, Kn, cn, Bo, Ii, Xt, Be, gd, yd, ti, Re, mv, $y, te, wy, ne, Jt, St;
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
  const Mu = "modulepreload", Pu = function(n) {
    return "/spaghettio/" + n;
  }, La = {}, Pn = function(t, e, s) {
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
        if (c = Pu(c), c in La) return;
        La[c] = true;
        const h = c.endsWith(".css"), d = h ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${c}"]${d}`)) return;
        const u = document.createElement("link");
        if (u.rel = h ? "stylesheet" : Mu, h || (u.as = "script"), u.crossOrigin = "", u.href = c, l && u.setAttribute("nonce", l), document.head.appendChild(u), h) return new Promise((p, f) => {
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
  ht = ((n) => (n.Application = "application", n.WebGLPipes = "webgl-pipes", n.WebGLPipesAdaptor = "webgl-pipes-adaptor", n.WebGLSystem = "webgl-system", n.WebGPUPipes = "webgpu-pipes", n.WebGPUPipesAdaptor = "webgpu-pipes-adaptor", n.WebGPUSystem = "webgpu-system", n.CanvasSystem = "canvas-system", n.CanvasPipesAdaptor = "canvas-pipes-adaptor", n.CanvasPipes = "canvas-pipes", n.Asset = "asset", n.LoadParser = "load-parser", n.ResolveParser = "resolve-parser", n.CacheParser = "cache-parser", n.DetectionParser = "detection-parser", n.MaskEffect = "mask-effect", n.BlendMode = "blend-mode", n.TextureSource = "texture-source", n.Environment = "environment", n.ShapeBuilder = "shape-builder", n.Batcher = "batcher", n))(ht || {});
  let po, yi, Iu, Ru;
  po = (n) => {
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
  yi = (n, t) => po(n).priority ?? t;
  Gt = {
    _addHandlers: {},
    _removeHandlers: {},
    _queue: {},
    remove(...n) {
      return n.map(po).forEach((t) => {
        t.type.forEach((e) => {
          var _a2, _b2;
          return (_b2 = (_a2 = this._removeHandlers)[e]) == null ? void 0 : _b2.call(_a2, t);
        });
      }), this;
    },
    add(...n) {
      return n.map(po).forEach((t) => {
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
        }), t.sort((r, o) => yi(o.value, e) - yi(r.value, e)));
      }, (s) => {
        const i = t.findIndex((r) => r.name === s.name);
        i !== -1 && t.splice(i, 1);
      });
    },
    handleByList(n, t, e = -1) {
      return this.handle(n, (s) => {
        t.includes(s.ref) || (t.push(s.ref), t.sort((i, r) => yi(r, e) - yi(i, e)));
      }, (s) => {
        const i = t.indexOf(s.ref);
        i !== -1 && t.splice(i, 1);
      });
    },
    mixin(n, ...t) {
      for (const e of t) Object.defineProperties(n.prototype, Object.getOwnPropertyDescriptors(e));
    }
  };
  Iu = {
    extension: {
      type: ht.Environment,
      name: "browser",
      priority: -1
    },
    test: () => true,
    load: async () => {
      await Pn(() => import("./browserAll-i-drEUnr.js"), __vite__mapDeps([0,1,2]));
    }
  };
  Ru = {
    extension: {
      type: ht.Environment,
      name: "webworker",
      priority: 0
    },
    test: () => typeof self < "u" && self.WorkerGlobalScope !== void 0,
    load: async () => {
      await Pn(() => import("./webworkerAll-Ds81m8HJ.js"), __vite__mapDeps([1,2]));
    }
  };
  class ue {
    constructor(t, e, s) {
      this._x = e || 0, this._y = s || 0, this._observer = t;
    }
    clone(t) {
      return new ue(t ?? this._observer, this._x, this._y);
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
  function Yc(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var Cr = {
    exports: {}
  }, $a;
  function Lu() {
    return $a || ($a = 1, (function(n) {
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
        var m = e ? e + c : c;
        if (!this._events[m]) return false;
        var g = this._events[m], y = arguments.length, w, x;
        if (g.fn) {
          switch (g.once && this.removeListener(c, g.fn, void 0, true), y) {
            case 1:
              return g.fn.call(g.context), true;
            case 2:
              return g.fn.call(g.context, h), true;
            case 3:
              return g.fn.call(g.context, h, d), true;
            case 4:
              return g.fn.call(g.context, h, d, u), true;
            case 5:
              return g.fn.call(g.context, h, d, u, p), true;
            case 6:
              return g.fn.call(g.context, h, d, u, p, f), true;
          }
          for (x = 1, w = new Array(y - 1); x < y; x++) w[x - 1] = arguments[x];
          g.fn.apply(g.context, w);
        } else {
          var _ = g.length, v;
          for (x = 0; x < _; x++) switch (g[x].once && this.removeListener(c, g[x].fn, void 0, true), y) {
            case 1:
              g[x].fn.call(g[x].context);
              break;
            case 2:
              g[x].fn.call(g[x].context, h);
              break;
            case 3:
              g[x].fn.call(g[x].context, h, d);
              break;
            case 4:
              g[x].fn.call(g[x].context, h, d, u);
              break;
            default:
              if (!w) for (v = 1, w = new Array(y - 1); v < y; v++) w[v - 1] = arguments[v];
              g[x].fn.apply(g[x].context, w);
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
          for (var m = 0, g = [], y = f.length; m < y; m++) (f[m].fn !== h || u && !f[m].once || d && f[m].context !== d) && g.push(f[m]);
          g.length ? this._events[p] = g.length === 1 ? g[0] : g : o(this, p);
        }
        return this;
      }, a.prototype.removeAllListeners = function(c) {
        var h;
        return c ? (h = e ? e + c : c, this._events[h] && o(this, h)) : (this._events = new s(), this._eventsCount = 0), this;
      }, a.prototype.off = a.prototype.removeListener, a.prototype.addListener = a.prototype.on, a.prefixed = e, a.EventEmitter = a, n.exports = a;
    })(Cr)), Cr.exports;
  }
  var $u = Lu();
  let Bu, Ou, Nu;
  rn = Yc($u);
  Bu = Math.PI * 2;
  Ou = 180 / Math.PI;
  Nu = Math.PI / 180;
  Tt = class {
    constructor(t = 0, e = 0) {
      this.x = 0, this.y = 0, this.x = t, this.y = e;
    }
    clone() {
      return new Tt(this.x, this.y);
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
      return Sr.x = 0, Sr.y = 0, Sr;
    }
  };
  const Sr = new Tt();
  vt = class {
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
      e = e || new Tt();
      const s = t.x, i = t.y;
      return e.x = this.a * s + this.c * i + this.tx, e.y = this.b * s + this.d * i + this.ty, e;
    }
    applyInverse(t, e) {
      e = e || new Tt();
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
      return c < 1e-5 || Math.abs(Bu - c) < 1e-5 ? (t.rotation = l, t.skew.x = t.skew.y = 0) : (t.rotation = 0, t.skew.x = a, t.skew.y = l), t.scale.x = Math.sqrt(e * e + s * s), t.scale.y = Math.sqrt(i * i + r * r), t.position.x = this.tx + (o.x * e + o.y * i), t.position.y = this.ty + (o.x * s + o.y * r), t;
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
      const t = new vt();
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
      return Wu.identity();
    }
    static get shared() {
      return Fu.identity();
    }
  };
  const Fu = new vt(), Wu = new vt(), zn = [
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
  ], Dn = [
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
  ], Hn = [
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
  ], Un = [
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
  ], fo = [], Xc = [], xi = Math.sign;
  function Gu() {
    for (let n = 0; n < 16; n++) {
      const t = [];
      fo.push(t);
      for (let e = 0; e < 16; e++) {
        const s = xi(zn[n] * zn[e] + Hn[n] * Dn[e]), i = xi(Dn[n] * zn[e] + Un[n] * Dn[e]), r = xi(zn[n] * Hn[e] + Hn[n] * Un[e]), o = xi(Dn[n] * Hn[e] + Un[n] * Un[e]);
        for (let a = 0; a < 16; a++) if (zn[a] === s && Dn[a] === i && Hn[a] === r && Un[a] === o) {
          t.push(a);
          break;
        }
      }
    }
    for (let n = 0; n < 16; n++) {
      const t = new vt();
      t.set(zn[n], Dn[n], Hn[n], Un[n], 0, 0), Xc.push(t);
    }
  }
  Gu();
  let bi;
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
    uX: (n) => zn[n],
    uY: (n) => Dn[n],
    vX: (n) => Hn[n],
    vY: (n) => Un[n],
    inv: (n) => n & 8 ? n & 15 : -n & 7,
    add: (n, t) => fo[n][t],
    sub: (n, t) => fo[n][St.inv(t)],
    rotate180: (n) => n ^ 4,
    isVertical: (n) => (n & 3) === 2,
    byDirection: (n, t) => Math.abs(n) * 2 <= Math.abs(t) ? t >= 0 ? St.S : St.N : Math.abs(t) * 2 <= Math.abs(n) ? n > 0 ? St.E : St.W : t > 0 ? n > 0 ? St.SE : St.SW : n > 0 ? St.NE : St.NW,
    matrixAppendRotationInv: (n, t, e = 0, s = 0, i = 0, r = 0) => {
      const o = Xc[St.inv(t)], a = o.a, l = o.b, c = o.c, h = o.d, d = e - Math.min(0, a * i, c * r, a * i + c * r), u = s - Math.min(0, l * i, h * r, l * i + h * r), p = n.a, f = n.b, m = n.c, g = n.d;
      n.a = a * p + l * m, n.b = a * f + l * g, n.c = c * p + h * m, n.d = c * f + h * g, n.tx = d * p + u * m + n.tx, n.ty = d * f + u * g + n.ty;
    },
    transformRectCoords: (n, t, e, s) => {
      const { x: i, y: r, width: o, height: a } = n, { x: l, y: c, width: h, height: d } = t;
      return e === St.E ? (s.set(i + l, r + c, o, a), s) : e === St.S ? s.set(h - r - a + l, i + c, a, o) : e === St.W ? s.set(h - i - o + l, d - r - a + c, o, a) : e === St.N ? s.set(r + l, d - i - o + c, a, o) : s.set(i + l, r + c, o, a);
    }
  };
  bi = [
    new Tt(),
    new Tt(),
    new Tt(),
    new Tt()
  ];
  Ht = class {
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
      return new Ht(0, 0, 0, 0);
    }
    clone() {
      return new Ht(this.x, this.y, this.width, this.height);
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
      const a = this.x, l = this.y, c = s * (1 - i), h = s - c, d = a - c, u = a + r + c, p = l - c, f = l + o + c, m = a + h, g = a + r - h, y = l + h, w = l + o - h;
      return t >= d && t <= u && e >= p && e <= f && !(t > m && t < g && e > y && e < w);
    }
    intersects(t, e) {
      if (!e) {
        const T = this.x < t.x ? t.x : this.x;
        if ((this.right > t.right ? t.right : this.right) <= T) return false;
        const A = this.y < t.y ? t.y : this.y;
        return (this.bottom > t.bottom ? t.bottom : this.bottom) > A;
      }
      const s = this.left, i = this.right, r = this.top, o = this.bottom;
      if (i <= s || o <= r) return false;
      const a = bi[0].set(t.left, t.top), l = bi[1].set(t.left, t.bottom), c = bi[2].set(t.right, t.top), h = bi[3].set(t.right, t.bottom);
      if (c.x <= a.x || l.y <= a.y) return false;
      const d = Math.sign(e.a * e.d - e.b * e.c);
      if (d === 0 || (e.apply(a, a), e.apply(l, l), e.apply(c, c), e.apply(h, h), Math.max(a.x, l.x, c.x, h.x) <= s || Math.min(a.x, l.x, c.x, h.x) >= i || Math.max(a.y, l.y, c.y, h.y) <= r || Math.min(a.y, l.y, c.y, h.y) >= o)) return false;
      const u = d * (l.y - a.y), p = d * (a.x - l.x), f = u * s + p * r, m = u * i + p * r, g = u * s + p * o, y = u * i + p * o;
      if (Math.max(f, m, g, y) <= u * a.x + p * a.y || Math.min(f, m, g, y) >= u * h.x + p * h.y) return false;
      const w = d * (a.y - c.y), x = d * (c.x - a.x), _ = w * s + x * r, v = w * i + x * r, b = w * s + x * o, C = w * i + x * o;
      return !(Math.max(_, v, b, C) <= w * a.x + x * a.y || Math.min(_, v, b, C) >= w * h.x + x * h.y);
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
      return t || (t = new Ht()), t.copyFrom(this), t;
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
  const Tr = {
    default: -1
  };
  ee = function(n = "default") {
    return Tr[n] === void 0 && (Tr[n] = -1), ++Tr[n];
  };
  let Ba, zu, ps;
  Ba = /* @__PURE__ */ new Set();
  te = "8.0.0";
  zu = "8.3.4";
  ps = {
    quiet: false,
    noColor: false
  };
  Mt = ((n, t, e = 3) => {
    if (ps.quiet || Ba.has(t)) return;
    let s = new Error().stack;
    const i = `${t}
Deprecated since v${n}`, r = typeof console.groupCollapsed == "function" && !ps.noColor;
    typeof s > "u" ? console.warn("PixiJS Deprecation Warning: ", i) : (s = s.split(`
`).splice(e).join(`
`), r ? (console.groupCollapsed("%cPixiJS Deprecation Warning: %c%s", "color:#614108;background:#fffbe6", "font-weight:normal;color:#614108;background:#fffbe6", i), console.warn(s), console.groupEnd()) : (console.warn("PixiJS Deprecation Warning: ", i), console.warn(s))), Ba.add(t);
  });
  Object.defineProperties(Mt, {
    quiet: {
      get: () => ps.quiet,
      set: (n) => {
        ps.quiet = n;
      },
      enumerable: true,
      configurable: false
    },
    noColor: {
      get: () => ps.noColor,
      set: (n) => {
        ps.noColor = n;
      },
      enumerable: true,
      configurable: false
    }
  });
  const qc = () => {
  };
  function ws(n) {
    return n += n === 0 ? 1 : 0, --n, n |= n >>> 1, n |= n >>> 2, n |= n >>> 4, n |= n >>> 8, n |= n >>> 16, n + 1;
  }
  function Oa(n) {
    return !(n & n - 1) && !!n;
  }
  function Kc(n) {
    const t = {};
    for (const e in n) n[e] !== void 0 && (t[e] = n[e]);
    return t;
  }
  const Na = /* @__PURE__ */ Object.create(null);
  function Du(n) {
    const t = Na[n];
    return t === void 0 && (Na[n] = ee("resource")), t;
  }
  const Jc = class Zc extends rn {
    constructor(t = {}) {
      super(), this._resourceType = "textureSampler", this._touched = 0, this._maxAnisotropy = 1, this.destroyed = false, t = {
        ...Zc.defaultOptions,
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
      Mt(te, "TextureStyle.wrapMode is now TextureStyle.addressMode"), this.addressMode = t;
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
      return this._sharedResourceId = Du(t), this._resourceId;
    }
    destroy() {
      this.destroyed = true, this.emit("destroy", this), this.emit("change", this), this.removeAllListeners();
    }
  };
  Jc.defaultOptions = {
    addressMode: "clamp-to-edge",
    scaleMode: "linear"
  };
  Kn = Jc;
  const Qc = class th extends rn {
    constructor(t = {}) {
      super(), this.options = t, this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.uid = ee("textureSource"), this._resourceType = "textureSource", this._resourceId = ee("resource"), this.uploadMethodId = "unknown", this._resolution = 1, this.pixelWidth = 1, this.pixelHeight = 1, this.width = 1, this.height = 1, this.sampleCount = 1, this.mipLevelCount = 1, this.autoGenerateMipmaps = false, this.format = "rgba8unorm", this.dimension = "2d", this.viewDimension = "2d", this.arrayLayerCount = 1, this.antialias = false, this._touched = 0, this._batchTick = -1, this._textureBindLocation = -1, t = {
        ...th.defaultOptions,
        ...t
      }, this.label = t.label ?? "", this.resource = t.resource, this.autoGarbageCollect = t.autoGarbageCollect, this._resolution = t.resolution, t.width ? this.pixelWidth = t.width * this._resolution : this.pixelWidth = this.resource ? this.resourceWidth ?? 1 : 1, t.height ? this.pixelHeight = t.height * this._resolution : this.pixelHeight = this.resource ? this.resourceHeight ?? 1 : 1, this.width = this.pixelWidth / this._resolution, this.height = this.pixelHeight / this._resolution, this.format = t.format, this.dimension = t.dimensions, this.viewDimension = t.viewDimension ?? t.dimensions, this.arrayLayerCount = t.arrayLayerCount, this.mipLevelCount = t.mipLevelCount, this.autoGenerateMipmaps = t.autoGenerateMipmaps, this.sampleCount = t.sampleCount, this.antialias = t.antialias, this.alphaMode = t.alphaMode, this.style = new Kn(Kc(t)), this.destroyed = false, this._refreshPOT();
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
      this._resourceId = ee("resource"), this.emit("change", this), this.emit("unload", this);
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
      return this.width = i / s, this.height = r / s, this._resolution = s, this.pixelWidth === i && this.pixelHeight === r ? false : (this._refreshPOT(), this.pixelWidth = i, this.pixelHeight = r, this.emit("resize", this), this._resourceId = ee("resource"), this.emit("change", this), true);
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
      this.isPowerOfTwo = Oa(this.pixelWidth) && Oa(this.pixelHeight);
    }
    static test(t) {
      throw new Error("Unimplemented");
    }
  };
  Qc.defaultOptions = {
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
  Me = Qc;
  class qo extends Me {
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
  qo.extension = ht.TextureSource;
  const Fa = new vt();
  Hu = class {
    constructor(t, e) {
      this.mapCoord = new vt(), this.uClampFrame = new Float32Array(4), this.uClampOffset = new Float32Array(2), this._textureID = -1, this._updateID = 0, this.clampOffset = 0, typeof e > "u" ? this.clampMargin = t.width < 10 ? 0 : 0.5 : this.clampMargin = e, this.isSimple = false, this.texture = t;
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
      i && (Fa.set(s.width / i.width, 0, 0, s.height / i.height, -i.x / i.width, -i.y / i.height), this.mapCoord.append(Fa));
      const r = t.source, o = this.uClampFrame, a = this.clampMargin / r._resolution, l = this.clampOffset / r._resolution;
      return o[0] = (t.frame.x + a + l) / r.width, o[1] = (t.frame.y + a + l) / r.height, o[2] = (t.frame.x + t.frame.width - a + l) / r.width, o[3] = (t.frame.y + t.frame.height - a + l) / r.height, this.uClampOffset[0] = this.clampOffset / r.pixelWidth, this.uClampOffset[1] = this.clampOffset / r.pixelHeight, this.isSimple = t.frame.width === r.width && t.frame.height === r.height && t.rotate === 0, true;
    }
  };
  Ct = class extends rn {
    constructor({ source: t, label: e, frame: s, orig: i, trim: r, defaultAnchor: o, defaultBorders: a, rotate: l, dynamic: c } = {}) {
      if (super(), this.uid = ee("texture"), this.uvs = {
        x0: 0,
        y0: 0,
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
        x3: 0,
        y3: 0
      }, this.frame = new Ht(), this.noFrame = false, this.dynamic = false, this.isTexture = true, this.label = e, this.source = (t == null ? void 0 : t.source) ?? new Me(), this.noFrame = !s, s) this.frame.copyFrom(s);
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
      return this._textureMatrix || (this._textureMatrix = new Hu(this)), this._textureMatrix;
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
      return Mt(te, "Texture.baseTexture is now Texture.source"), this._source;
    }
  };
  Ct.EMPTY = new Ct({
    label: "EMPTY",
    source: new Me({
      label: "EMPTY"
    })
  });
  Ct.EMPTY.destroy = qc;
  Ct.WHITE = new Ct({
    source: new qo({
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
  Ct.WHITE.destroy = qc;
  function eh(n, t, e) {
    const { width: s, height: i } = e.orig, r = e.trim;
    if (r) {
      const o = r.width, a = r.height;
      n.minX = r.x - t._x * s, n.maxX = n.minX + o, n.minY = r.y - t._y * i, n.maxY = n.minY + a;
    } else n.minX = -t._x * s, n.maxX = n.minX + s, n.minY = -t._y * i, n.maxY = n.minY + i;
  }
  const Wa = new vt();
  ke = class {
    constructor(t = 1 / 0, e = 1 / 0, s = -1 / 0, i = -1 / 0) {
      this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = Wa, this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
    }
    isEmpty() {
      return this.minX > this.maxX || this.minY > this.maxY;
    }
    get rectangle() {
      this._rectangle || (this._rectangle = new Ht());
      const t = this._rectangle;
      return this.minX > this.maxX || this.minY > this.maxY ? (t.x = 0, t.y = 0, t.width = 0, t.height = 0) : t.copyFromBounds(this), t;
    }
    clear() {
      return this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = Wa, this;
    }
    set(t, e, s, i) {
      this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
    }
    addFrame(t, e, s, i, r) {
      r || (r = this.matrix);
      const o = r.a, a = r.b, l = r.c, c = r.d, h = r.tx, d = r.ty;
      let u = this.minX, p = this.minY, f = this.maxX, m = this.maxY, g = o * t + l * e + h, y = a * t + c * e + d;
      g < u && (u = g), y < p && (p = y), g > f && (f = g), y > m && (m = y), g = o * s + l * e + h, y = a * s + c * e + d, g < u && (u = g), y < p && (p = y), g > f && (f = g), y > m && (m = y), g = o * t + l * i + h, y = a * t + c * i + d, g < u && (u = g), y < p && (p = y), g > f && (f = g), y > m && (m = y), g = o * s + l * i + h, y = a * s + c * i + d, g < u && (u = g), y < p && (p = y), g > f && (f = g), y > m && (m = y), this.minX = u, this.minY = p, this.maxX = f, this.maxY = m;
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
      return new ke(this.minX, this.minY, this.maxX, this.maxY);
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
      for (let m = e; m < s; m += 2) {
        const g = t[m], y = t[m + 1], w = c * g + d * y + p, x = h * g + u * y + f;
        r = w < r ? w : r, o = x < o ? x : o, a = w > a ? w : a, l = x > l ? x : l;
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
  var Uu = {
    grad: 0.9,
    turn: 360,
    rad: 360 / (2 * Math.PI)
  }, on = function(n) {
    return typeof n == "string" ? n.length > 0 : typeof n == "number";
  }, ae = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = Math.pow(10, t)), Math.round(e * n) / e + 0;
  }, Ie = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = 1), n > e ? e : n > t ? n : t;
  }, nh = function(n) {
    return (n = isFinite(n) ? n % 360 : 0) > 0 ? n : n + 360;
  }, Ga = function(n) {
    return {
      r: Ie(n.r, 0, 255),
      g: Ie(n.g, 0, 255),
      b: Ie(n.b, 0, 255),
      a: Ie(n.a)
    };
  }, Er = function(n) {
    return {
      r: ae(n.r),
      g: ae(n.g),
      b: ae(n.b),
      a: ae(n.a, 3)
    };
  }, ju = /^#([0-9a-f]{3,8})$/i, _i = function(n) {
    var t = n.toString(16);
    return t.length < 2 ? "0" + t : t;
  }, sh = function(n) {
    var t = n.r, e = n.g, s = n.b, i = n.a, r = Math.max(t, e, s), o = r - Math.min(t, e, s), a = o ? r === t ? (e - s) / o : r === e ? 2 + (s - t) / o : 4 + (t - e) / o : 0;
    return {
      h: 60 * (a < 0 ? a + 6 : a),
      s: r ? o / r * 100 : 0,
      v: r / 255 * 100,
      a: i
    };
  }, ih = function(n) {
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
  }, za = function(n) {
    return {
      h: nh(n.h),
      s: Ie(n.s, 0, 100),
      l: Ie(n.l, 0, 100),
      a: Ie(n.a)
    };
  }, Da = function(n) {
    return {
      h: ae(n.h),
      s: ae(n.s),
      l: ae(n.l),
      a: ae(n.a, 3)
    };
  }, Ha = function(n) {
    return ih((e = (t = n).s, {
      h: t.h,
      s: (e *= ((s = t.l) < 50 ? s : 100 - s) / 100) > 0 ? 2 * e / (s + e) * 100 : 0,
      v: s + e,
      a: t.a
    }));
    var t, e, s;
  }, Xs = function(n) {
    return {
      h: (t = sh(n)).h,
      s: (i = (200 - (e = t.s)) * (s = t.v) / 100) > 0 && i < 200 ? e * s / 100 / (i <= 100 ? i : 200 - i) * 100 : 0,
      l: i / 2,
      a: t.a
    };
    var t, e, s, i;
  }, Vu = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s*,\s*([+-]?\d*\.?\d+)%\s*,\s*([+-]?\d*\.?\d+)%\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Yu = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s+([+-]?\d*\.?\d+)%\s+([+-]?\d*\.?\d+)%\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Xu = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, qu = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, mo = {
    string: [
      [
        function(n) {
          var t = ju.exec(n);
          return t ? (n = t[1]).length <= 4 ? {
            r: parseInt(n[0] + n[0], 16),
            g: parseInt(n[1] + n[1], 16),
            b: parseInt(n[2] + n[2], 16),
            a: n.length === 4 ? ae(parseInt(n[3] + n[3], 16) / 255, 2) : 1
          } : n.length === 6 || n.length === 8 ? {
            r: parseInt(n.substr(0, 2), 16),
            g: parseInt(n.substr(2, 2), 16),
            b: parseInt(n.substr(4, 2), 16),
            a: n.length === 8 ? ae(parseInt(n.substr(6, 2), 16) / 255, 2) : 1
          } : null : null;
        },
        "hex"
      ],
      [
        function(n) {
          var t = Xu.exec(n) || qu.exec(n);
          return t ? t[2] !== t[4] || t[4] !== t[6] ? null : Ga({
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
          var t = Vu.exec(n) || Yu.exec(n);
          if (!t) return null;
          var e, s, i = za({
            h: (e = t[1], s = t[2], s === void 0 && (s = "deg"), Number(e) * (Uu[s] || 1)),
            s: Number(t[3]),
            l: Number(t[4]),
            a: t[5] === void 0 ? 1 : Number(t[5]) / (t[6] ? 100 : 1)
          });
          return Ha(i);
        },
        "hsl"
      ]
    ],
    object: [
      [
        function(n) {
          var t = n.r, e = n.g, s = n.b, i = n.a, r = i === void 0 ? 1 : i;
          return on(t) && on(e) && on(s) ? Ga({
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
          if (!on(t) || !on(e) || !on(s)) return null;
          var o = za({
            h: Number(t),
            s: Number(e),
            l: Number(s),
            a: Number(r)
          });
          return Ha(o);
        },
        "hsl"
      ],
      [
        function(n) {
          var t = n.h, e = n.s, s = n.v, i = n.a, r = i === void 0 ? 1 : i;
          if (!on(t) || !on(e) || !on(s)) return null;
          var o = (function(a) {
            return {
              h: nh(a.h),
              s: Ie(a.s, 0, 100),
              v: Ie(a.v, 0, 100),
              a: Ie(a.a)
            };
          })({
            h: Number(t),
            s: Number(e),
            v: Number(s),
            a: Number(r)
          });
          return ih(o);
        },
        "hsv"
      ]
    ]
  }, Ua = function(n, t) {
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
  }, Ku = function(n) {
    return typeof n == "string" ? Ua(n.trim(), mo.string) : typeof n == "object" && n !== null ? Ua(n, mo.object) : [
      null,
      void 0
    ];
  }, Ar = function(n, t) {
    var e = Xs(n);
    return {
      h: e.h,
      s: Ie(e.s + 100 * t, 0, 100),
      l: e.l,
      a: e.a
    };
  }, kr = function(n) {
    return (299 * n.r + 587 * n.g + 114 * n.b) / 1e3 / 255;
  }, ja = function(n, t) {
    var e = Xs(n);
    return {
      h: e.h,
      s: e.s,
      l: Ie(e.l + 100 * t, 0, 100),
      a: e.a
    };
  }, go = (function() {
    function n(t) {
      this.parsed = Ku(t)[0], this.rgba = this.parsed || {
        r: 0,
        g: 0,
        b: 0,
        a: 1
      };
    }
    return n.prototype.isValid = function() {
      return this.parsed !== null;
    }, n.prototype.brightness = function() {
      return ae(kr(this.rgba), 2);
    }, n.prototype.isDark = function() {
      return kr(this.rgba) < 0.5;
    }, n.prototype.isLight = function() {
      return kr(this.rgba) >= 0.5;
    }, n.prototype.toHex = function() {
      return t = Er(this.rgba), e = t.r, s = t.g, i = t.b, o = (r = t.a) < 1 ? _i(ae(255 * r)) : "", "#" + _i(e) + _i(s) + _i(i) + o;
      var t, e, s, i, r, o;
    }, n.prototype.toRgb = function() {
      return Er(this.rgba);
    }, n.prototype.toRgbString = function() {
      return t = Er(this.rgba), e = t.r, s = t.g, i = t.b, (r = t.a) < 1 ? "rgba(" + e + ", " + s + ", " + i + ", " + r + ")" : "rgb(" + e + ", " + s + ", " + i + ")";
      var t, e, s, i, r;
    }, n.prototype.toHsl = function() {
      return Da(Xs(this.rgba));
    }, n.prototype.toHslString = function() {
      return t = Da(Xs(this.rgba)), e = t.h, s = t.s, i = t.l, (r = t.a) < 1 ? "hsla(" + e + ", " + s + "%, " + i + "%, " + r + ")" : "hsl(" + e + ", " + s + "%, " + i + "%)";
      var t, e, s, i, r;
    }, n.prototype.toHsv = function() {
      return t = sh(this.rgba), {
        h: ae(t.h),
        s: ae(t.s),
        v: ae(t.v),
        a: ae(t.a, 3)
      };
      var t;
    }, n.prototype.invert = function() {
      return Je({
        r: 255 - (t = this.rgba).r,
        g: 255 - t.g,
        b: 255 - t.b,
        a: t.a
      });
      var t;
    }, n.prototype.saturate = function(t) {
      return t === void 0 && (t = 0.1), Je(Ar(this.rgba, t));
    }, n.prototype.desaturate = function(t) {
      return t === void 0 && (t = 0.1), Je(Ar(this.rgba, -t));
    }, n.prototype.grayscale = function() {
      return Je(Ar(this.rgba, -1));
    }, n.prototype.lighten = function(t) {
      return t === void 0 && (t = 0.1), Je(ja(this.rgba, t));
    }, n.prototype.darken = function(t) {
      return t === void 0 && (t = 0.1), Je(ja(this.rgba, -t));
    }, n.prototype.rotate = function(t) {
      return t === void 0 && (t = 15), this.hue(this.hue() + t);
    }, n.prototype.alpha = function(t) {
      return typeof t == "number" ? Je({
        r: (e = this.rgba).r,
        g: e.g,
        b: e.b,
        a: t
      }) : ae(this.rgba.a, 3);
      var e;
    }, n.prototype.hue = function(t) {
      var e = Xs(this.rgba);
      return typeof t == "number" ? Je({
        h: t,
        s: e.s,
        l: e.l,
        a: e.a
      }) : ae(e.h);
    }, n.prototype.isEqual = function(t) {
      return this.toHex() === Je(t).toHex();
    }, n;
  })(), Je = function(n) {
    return n instanceof go ? n : new go(n);
  }, Va = [], Ju = function(n) {
    n.forEach(function(t) {
      Va.indexOf(t) < 0 && (t(go, mo), Va.push(t));
    });
  };
  function Zu(n, t) {
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
          var m = (a = h, l = r[f], Math.pow(a.r - l.r, 2) + Math.pow(a.g - l.g, 2) + Math.pow(a.b - l.b, 2));
          m < d && (d = m, u = f);
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
  Ju([
    Zu
  ]);
  const vs = class Us {
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
      if (t instanceof Us) this._value = this._cloneSource(t._value), this._int = t._int, this._components.set(t._components);
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
      const [e, s, i, r] = Us._temp.setValue(t)._components;
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
          const a = Us.HEX_PATTERN.exec(t);
          a && (t = `#${a[2]}`);
        }
        const o = Je(t);
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
      return typeof t == "number" || typeof t == "string" || t instanceof Number || t instanceof Us || Array.isArray(t) || t instanceof Uint8Array || t instanceof Uint8ClampedArray || t instanceof Float32Array || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 && t.a !== void 0;
    }
  };
  vs.shared = new vs();
  vs._temp = new vs();
  vs.HEX_PATTERN = /^(#|0x)?(([a-f0-9]{3}){1,2}([a-f0-9]{2})?)$/i;
  Xt = vs;
  const Qu = {
    cullArea: null,
    cullable: false,
    cullableChildren: true
  };
  let Mr = 0;
  const Ya = 500;
  Jt = function(...n) {
    Mr !== Ya && (Mr++, Mr === Ya ? console.warn("PixiJS Warning: too many warnings, no more warnings will be reported to the console by PixiJS.") : console.warn("PixiJS Warning: ", ...n));
  };
  pi = {
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
  class tp {
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
  class ep {
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
      return this._poolsByClass.has(t) || this._poolsByClass.set(t, new tp(t)), this._poolsByClass.get(t);
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
  ve = new ep();
  pi.register(ve);
  const np = {
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
      Mt("v8.6.0", "cacheAsBitmap is deprecated, use cacheAsTexture instead."), this.cacheAsTexture(n);
    }
  };
  sp = function(n, t, e) {
    const s = n.length;
    let i;
    if (t >= s || e === 0) return;
    e = t + e > s ? s - t : e;
    const r = s - e;
    for (i = t; i < r; ++i) n[i] = n[i + e];
    n.length = r;
  };
  const ip = {
    allowChildren: true,
    removeChildren(n = 0, t) {
      var _a2;
      const e = t ?? this.children.length, s = e - n, i = [];
      if (s > 0 && s <= e) {
        for (let o = e - 1; o >= n; o--) {
          const a = this.children[o];
          a && (i.push(a), a.parent = null);
        }
        sp(this.children, n, e);
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
      this.allowChildren || Mt(te, "addChildAt: Only Containers will be allowed to add children in v8.0.0");
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
  }, rp = {
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
  Xa = class {
    constructor() {
      this.pipe = "filter", this.priority = 1;
    }
    destroy() {
      for (let t = 0; t < this.filters.length; t++) this.filters[t].destroy();
      this.filters = null, this.filterArea = null;
    }
  };
  class op {
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
        if (s.test(t)) return ve.get(s.maskClass, t);
      }
      return t;
    }
    returnMaskEffect(t) {
      ve.return(t);
    }
  }
  const yo = new op();
  Gt.handleByList(ht.MaskEffect, yo._effectClasses);
  const ap = {
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
      (t == null ? void 0 : t.mask) !== n && (t && (this.removeEffect(t), yo.returnMaskEffect(t), this._maskEffect = null), n != null && (this._maskEffect = yo.getMaskEffect(n), this.addEffect(this._maskEffect)));
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
      const t = this._filterEffect || (this._filterEffect = new Xa());
      n = n;
      const e = (n == null ? void 0 : n.length) > 0, s = ((_a2 = t.filters) == null ? void 0 : _a2.length) > 0, i = e !== s;
      n = Array.isArray(n) ? n.slice(0) : n, t.filters = Object.freeze(n), i && (e ? this.addEffect(t) : (this.removeEffect(t), t.filters = n ?? null));
    },
    get filters() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filters;
    },
    set filterArea(n) {
      this._filterEffect || (this._filterEffect = new Xa()), this._filterEffect.filterArea = n;
    },
    get filterArea() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filterArea;
    }
  }, lp = {
    label: null,
    get name() {
      return Mt(te, "Container.name property has been removed, use Container.label instead"), this.label;
    },
    set name(n) {
      Mt(te, "Container.name property has been removed, use Container.label instead"), this.label = n;
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
  }, xe = ve.getPool(vt), fn = ve.getPool(ke), cp = new vt(), hp = {
    getFastGlobalBounds(n, t) {
      t || (t = new ke()), t.clear(), this._getGlobalBoundsRecursive(!!n, t, this.parentRenderLayer), t.isValid || t.set(0, 0, 0, 0);
      const e = this.renderGroup || this.parentRenderGroup;
      return t.applyMatrix(e.worldTransform), t;
    },
    _getGlobalBoundsRecursive(n, t, e) {
      let s = t;
      if (n && this.parentRenderLayer && this.parentRenderLayer !== e || this.localDisplayStatus !== 7 || !this.measurable) return;
      const i = !!this.effects.length;
      if ((this.renderGroup || i) && (s = fn.get().clear()), this.boundsArea) t.addRect(this.boundsArea, this.worldTransform);
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
        r && s.applyMatrix(o.worldTransform.copyTo(cp).invert()), t.addBounds(s), fn.return(s);
      } else this.renderGroup && (t.addBounds(s, this.relativeGroupTransform), fn.return(s));
    }
  };
  rh = function(n, t, e) {
    e.clear();
    let s, i;
    return n.parent ? t ? s = n.parent.worldTransform : (i = xe.get().identity(), s = Ko(n, i)) : s = vt.IDENTITY, oh(n, e, s, t), i && xe.return(i), e.isValid || e.set(0, 0, 0, 0), e;
  };
  function oh(n, t, e, s) {
    var _a2, _b2;
    if (!n.visible || !n.measurable) return;
    let i;
    s ? i = n.worldTransform : (n.updateLocalTransform(), i = xe.get(), i.appendFrom(n.localTransform, e));
    const r = t, o = !!n.effects.length;
    if (o && (t = fn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, i);
    else {
      const a = n.bounds;
      a && !a.isEmpty() && (t.matrix = i, t.addBounds(a));
      for (let l = 0; l < n.children.length; l++) oh(n.children[l], t, i, s);
    }
    if (o) {
      for (let a = 0; a < n.effects.length; a++) (_b2 = (_a2 = n.effects[a]).addBounds) == null ? void 0 : _b2.call(_a2, t);
      r.addBounds(t, vt.IDENTITY), fn.return(t);
    }
    s || xe.return(i);
  }
  function Ko(n, t) {
    const e = n.parent;
    return e && (Ko(e, t), e.updateLocalTransform(), t.append(e.localTransform)), t;
  }
  ah = function(n, t) {
    if (n === 16777215 || !t) return t;
    if (t === 16777215 || !n) return n;
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = t >> 16 & 255, o = t >> 8 & 255, a = t & 255, l = e * r / 255 | 0, c = s * o / 255 | 0, h = i * a / 255 | 0;
    return (l << 16) + (c << 8) + h;
  };
  const qa = 16777215;
  Ka = function(n, t) {
    return n === qa ? t : t === qa ? n : ah(n, t);
  };
  qs = function(n) {
    return ((n & 255) << 16) + (n & 65280) + (n >> 16 & 255);
  };
  const dp = {
    getGlobalAlpha(n) {
      if (n) return this.renderGroup ? this.renderGroup.worldAlpha : this.parentRenderGroup ? this.parentRenderGroup.worldAlpha * this.alpha : this.alpha;
      let t = this.alpha, e = this.parent;
      for (; e; ) t *= e.alpha, e = e.parent;
      return t;
    },
    getGlobalTransform(n = new vt(), t) {
      if (t) return n.copyFrom(this.worldTransform);
      this.updateLocalTransform();
      const e = Ko(this, xe.get().identity());
      return n.appendFrom(this.localTransform, e), xe.return(e), n;
    },
    getGlobalTint(n) {
      if (n) return this.renderGroup ? qs(this.renderGroup.worldColor) : this.parentRenderGroup ? qs(Ka(this.localColor, this.parentRenderGroup.worldColor)) : this.tint;
      let t = this.localColor, e = this.parent;
      for (; e; ) t = Ka(t, e.localColor), e = e.parent;
      return qs(t);
    }
  };
  lh = function(n, t, e) {
    return t.clear(), e || (e = vt.IDENTITY), ch(n, t, e, n, true), t.isValid || t.set(0, 0, 0, 0), t;
  };
  function ch(n, t, e, s, i) {
    var _a2, _b2;
    let r;
    if (i) r = xe.get(), r = e.copyTo(r);
    else {
      if (!n.visible || !n.measurable) return;
      n.updateLocalTransform();
      const l = n.localTransform;
      r = xe.get(), r.appendFrom(l, e);
    }
    const o = t, a = !!n.effects.length;
    if (a && (t = fn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, r);
    else {
      n.renderPipeId && (t.matrix = r, t.addBounds(n.bounds));
      const l = n.children;
      for (let c = 0; c < l.length; c++) ch(l[c], t, r, s, false);
    }
    if (a) {
      for (let l = 0; l < n.effects.length; l++) (_b2 = (_a2 = n.effects[l]).addLocalBounds) == null ? void 0 : _b2.call(_a2, t, s);
      o.addBounds(t, vt.IDENTITY), fn.return(t);
    }
    xe.return(r);
  }
  function hh(n, t) {
    const e = n.children;
    for (let s = 0; s < e.length; s++) {
      const i = e[s], r = i.uid, o = (i._didViewChangeTick & 65535) << 16 | i._didContainerChangeTick & 65535, a = t.index;
      (t.data[a] !== r || t.data[a + 1] !== o) && (t.data[t.index] = r, t.data[t.index + 1] = o, t.didChange = true), t.index = a + 2, i.children.length && hh(i, t);
    }
    return t.didChange;
  }
  const up = new vt(), pp = {
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
        localBounds: new ke()
      });
      const n = this._localBoundsCacheData;
      return n.index = 1, n.didChange = false, n.data[0] !== this._didViewChangeTick && (n.didChange = true, n.data[0] = this._didViewChangeTick), hh(this, n), n.didChange && lh(this, n.localBounds, up), n.localBounds;
    },
    getBounds(n, t) {
      return rh(this, n, t || new ke());
    }
  }, fp = {
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
  }, mp = {
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
      this.sortDirty && (this.sortDirty = false, this.children.sort(gp));
    }
  };
  function gp(n, t) {
    return n._zIndex - t._zIndex;
  }
  const yp = {
    getGlobalPosition(n = new Tt(), t = false) {
      return this.parent ? this.parent.toGlobal(this._position, n, t) : (n.x = this._position.x, n.y = this._position.y), n;
    },
    toGlobal(n, t, e = false) {
      const s = this.getGlobalTransform(xe.get(), e);
      return t = s.apply(n, t), xe.return(s), t;
    },
    toLocal(n, t, e, s) {
      t && (n = t.toGlobal(n, e, s));
      const i = this.getGlobalTransform(xe.get(), s);
      return e = i.applyInverse(n, e), xe.return(i), e;
    }
  };
  class Jo {
    constructor() {
      this.uid = ee("instructionSet"), this.instructions = [], this.instructionSize = 0, this.renderables = [], this.gcTick = 0;
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
  let xp = 0;
  class bp {
    constructor(t) {
      this._poolKeyHash = /* @__PURE__ */ Object.create(null), this._texturePool = {}, this.textureOptions = t || {}, this.enableFullScreen = false, this.textureStyle = new Kn(this.textureOptions);
    }
    createTexture(t, e, s, i) {
      const r = new Me({
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
        label: `texturePool_${xp++}`
      });
    }
    getOptimalTexture(t, e, s = 1, i, r = false) {
      let o = Math.ceil(t * s - 1e-6), a = Math.ceil(e * s - 1e-6);
      o = ws(o), a = ws(a);
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
  cr = new bp();
  pi.register(cr);
  _p = class {
    constructor() {
      this.renderPipeId = "renderGroup", this.root = null, this.canBundle = false, this.renderGroupParent = null, this.renderGroupChildren = [], this.worldTransform = new vt(), this.worldColorAlpha = 4294967295, this.worldColor = 16777215, this.worldAlpha = 1, this.childrenToUpdate = /* @__PURE__ */ Object.create(null), this.updateTick = 0, this.gcTick = 0, this.childrenRenderablesToUpdate = {
        list: [],
        index: 0
      }, this.structureDidChange = true, this.instructionSet = new Jo(), this._onRenderContainers = [], this.textureNeedsUpdate = true, this.isCachedAsTexture = false, this._matrixDirty = 7;
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
      this.isCachedAsTexture = false, this.texture && (cr.returnTexture(this.texture, true), this.texture = null);
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
      return (this._matrixDirty & 1) === 0 ? this._inverseWorldTransform : (this._matrixDirty &= -2, this._inverseWorldTransform || (this._inverseWorldTransform = new vt()), this._inverseWorldTransform.copyFrom(this.worldTransform).invert());
    }
    get textureOffsetInverseTransform() {
      return (this._matrixDirty & 2) === 0 ? this._textureOffsetInverseTransform : (this._matrixDirty &= -3, this._textureOffsetInverseTransform || (this._textureOffsetInverseTransform = new vt()), this._textureOffsetInverseTransform.copyFrom(this.inverseWorldTransform).translate(-this._textureBounds.x, -this._textureBounds.y));
    }
    get inverseParentTextureTransform() {
      if ((this._matrixDirty & 4) === 0) return this._inverseParentTextureTransform;
      this._matrixDirty &= -5;
      const t = this._parentCacheAsTextureRenderGroup;
      return t ? (this._inverseParentTextureTransform || (this._inverseParentTextureTransform = new vt()), this._inverseParentTextureTransform.copyFrom(this.worldTransform).prepend(t.inverseWorldTransform).translate(-t._textureBounds.x, -t._textureBounds.y)) : this.worldTransform;
    }
    get cacheToLocalTransform() {
      return this.isCachedAsTexture ? this.textureOffsetInverseTransform : this._parentCacheAsTextureRenderGroup ? this._parentCacheAsTextureRenderGroup.textureOffsetInverseTransform : null;
    }
  };
  function xo(n, t, e = {}) {
    for (const s in t) !e[s] && t[s] !== void 0 && (n[s] = t[s]);
  }
  let Pr, wi, Ir, vi;
  Pr = new ue(null);
  wi = new ue(null);
  Ir = new ue(null, 1, 1);
  vi = new ue(null);
  Ja = 1;
  wp = 2;
  Rr = 4;
  Ut = class extends rn {
    constructor(t = {}) {
      var _a2, _b2;
      super(), this.uid = ee("renderable"), this._updateFlags = 15, this.renderGroup = null, this.parentRenderGroup = null, this.parentRenderGroupIndex = 0, this.didChange = false, this.didViewUpdate = false, this.relativeRenderGroupDepth = 0, this.children = [], this.parent = null, this.includeInBuild = true, this.measurable = true, this.isSimple = true, this.parentRenderLayer = null, this.updateTick = -1, this.localTransform = new vt(), this.relativeGroupTransform = new vt(), this.groupTransform = this.relativeGroupTransform, this.destroyed = false, this._position = new ue(this, 0, 0), this._scale = Ir, this._pivot = wi, this._origin = vi, this._skew = Pr, this._cx = 1, this._sx = 0, this._cy = 0, this._sy = 1, this._rotation = 0, this.localColor = 16777215, this.localAlpha = 1, this.groupAlpha = 1, this.groupColor = 16777215, this.groupColorAlpha = 4294967295, this.localBlendMode = "inherit", this.groupBlendMode = "normal", this.localDisplayStatus = 7, this.globalDisplayStatus = 7, this._didContainerChangeTick = 0, this._didViewChangeTick = 0, this._didLocalTransformChangeId = -1, this.effects = [], xo(this, t, {
        children: true,
        parent: true,
        effects: true
      }), (_a2 = t.children) == null ? void 0 : _a2.forEach((e) => this.addChild(e)), (_b2 = t.parent) == null ? void 0 : _b2.addChild(this);
    }
    static mixin(t) {
      Mt("8.8.0", "Container.mixin is deprecated, please use extensions.mixin instead."), Gt.mixin(Ut, t);
    }
    set _didChangeId(t) {
      this._didViewChangeTick = t >> 12 & 4095, this._didContainerChangeTick = t & 4095;
    }
    get _didChangeId() {
      return this._didContainerChangeTick & 4095 | (this._didViewChangeTick & 4095) << 12;
    }
    addChild(...t) {
      if (this.allowChildren || Mt(te, "addChild: Only Containers will be allowed to add children in v8.0.0"), t.length > 1) {
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
      t == null ? void 0 : t.removeChild(this), this.renderGroup = ve.get(_p, this), this.groupTransform = vt.IDENTITY, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
    }
    disableRenderGroup() {
      if (!this.renderGroup) return;
      const t = this.parentRenderGroup;
      t == null ? void 0 : t.removeChild(this), ve.return(this.renderGroup), this.renderGroup = null, this.groupTransform = this.relativeGroupTransform, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
    }
    _updateIsSimple() {
      this.isSimple = !this.renderGroup && this.effects.length === 0;
    }
    get worldTransform() {
      return this._worldTransform || (this._worldTransform = new vt()), this.renderGroup ? this._worldTransform.copyFrom(this.renderGroup.worldTransform) : this.parentRenderGroup && this._worldTransform.appendFrom(this.relativeGroupTransform, this.parentRenderGroup.worldTransform), this._worldTransform;
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
      return this.rotation * Ou;
    }
    set angle(t) {
      this.rotation = t * Nu;
    }
    get pivot() {
      return this._pivot === wi && (this._pivot = new ue(this, 0, 0)), this._pivot;
    }
    set pivot(t) {
      this._pivot === wi && (this._pivot = new ue(this, 0, 0), this._origin !== vi && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._pivot.set(t) : this._pivot.copyFrom(t);
    }
    get skew() {
      return this._skew === Pr && (this._skew = new ue(this, 0, 0)), this._skew;
    }
    set skew(t) {
      this._skew === Pr && (this._skew = new ue(this, 0, 0)), this._skew.copyFrom(t);
    }
    get scale() {
      return this._scale === Ir && (this._scale = new ue(this, 1, 1)), this._scale;
    }
    set scale(t) {
      this._scale === Ir && (this._scale = new ue(this, 0, 0)), typeof t == "string" && (t = parseFloat(t)), typeof t == "number" ? this._scale.set(t) : this._scale.copyFrom(t);
    }
    get origin() {
      return this._origin === vi && (this._origin = new ue(this, 0, 0)), this._origin;
    }
    set origin(t) {
      this._origin === vi && (this._origin = new ue(this, 0, 0), this._pivot !== wi && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._origin.set(t) : this._origin.copyFrom(t);
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
      t !== this.localAlpha && (this.localAlpha = t, this._updateFlags |= Ja, this._onUpdate());
    }
    get alpha() {
      return this.localAlpha;
    }
    set tint(t) {
      const s = Xt.shared.setValue(t ?? 16777215).toBgrNumber();
      s !== this.localColor && (this.localColor = s, this._updateFlags |= Ja, this._onUpdate());
    }
    get tint() {
      return qs(this.localColor);
    }
    set blendMode(t) {
      this.localBlendMode !== t && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= wp, this.localBlendMode = t, this._onUpdate());
    }
    get blendMode() {
      return this.localBlendMode;
    }
    get visible() {
      return !!(this.localDisplayStatus & 2);
    }
    set visible(t) {
      const e = t ? 2 : 0;
      (this.localDisplayStatus & 2) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Rr, this.localDisplayStatus ^= 2, this._onUpdate(), this.emit("visibleChanged", t));
    }
    get culled() {
      return !(this.localDisplayStatus & 4);
    }
    set culled(t) {
      const e = t ? 0 : 4;
      (this.localDisplayStatus & 4) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Rr, this.localDisplayStatus ^= 4, this._onUpdate());
    }
    get renderable() {
      return !!(this.localDisplayStatus & 1);
    }
    set renderable(t) {
      const e = t ? 1 : 0;
      (this.localDisplayStatus & 1) !== e && (this._updateFlags |= Rr, this.localDisplayStatus ^= 1, this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._onUpdate());
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
  Gt.mixin(Ut, ip, hp, yp, fp, pp, ap, lp, mp, Qu, np, dp, rp);
  class hr extends Ut {
    constructor(t) {
      super(t), this.canBundle = true, this.allowChildren = false, this._roundPixels = 0, this._lastUsed = -1, this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this._bounds = new ke(0, 1, 0, 0), this._boundsDirty = true, this.autoGarbageCollect = t.autoGarbageCollect ?? true;
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
  Ye = class extends hr {
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
      }, this._anchor = new ue({
        _onUpdate: () => {
          this.onViewUpdate();
        }
      }), s ? this.anchor = s : e.defaultAnchor && (this.anchor = e.defaultAnchor), this.texture = e, this.allowChildren = false, this.roundPixels = i ?? false, r !== void 0 && (this.width = r), o !== void 0 && (this.height = o);
    }
    static from(t, e = false) {
      return t instanceof Ct ? new Ye(t) : new Ye(Ct.from(t, e));
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
      return eh(this._visualBounds, this._anchor, this._texture), this._visualBounds;
    }
    get sourceBounds() {
      return Mt("8.6.1", "Sprite.sourceBounds is deprecated, use visualBounds instead."), this.visualBounds;
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
  const vp = new ke();
  function dh(n, t, e) {
    const s = vp;
    n.measurable = true, rh(n, e, s), t.addBoundsMask(s), n.measurable = false;
  }
  function uh(n, t, e) {
    const s = fn.get();
    n.measurable = true;
    const i = xe.get().identity(), r = ph(n, e, i);
    lh(n, s, r), n.measurable = false, t.addBoundsMask(s), xe.return(i), fn.return(s);
  }
  function ph(n, t, e) {
    return n ? (n !== t && (ph(n.parent, t, e), n.updateLocalTransform(), e.append(n.localTransform)), e) : (Jt("Mask bounds, renderable is not inside the root container"), e);
  }
  class fh {
    constructor(t) {
      this.priority = 0, this.inverse = false, this.pipe = "alphaMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t, this.renderMaskToTexture = !(t instanceof Ye), this.mask.renderable = this.renderMaskToTexture, this.mask.includeInBuild = !this.renderMaskToTexture, this.mask.measurable = false;
    }
    reset() {
      this.mask !== null && (this.mask.measurable = true, this.mask = null);
    }
    addBounds(t, e) {
      this.inverse || dh(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      uh(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof Ye;
    }
  }
  fh.extension = ht.MaskEffect;
  class mh {
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
  mh.extension = ht.MaskEffect;
  class gh {
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
      dh(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      uh(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof Ut;
    }
  }
  gh.extension = ht.MaskEffect;
  const Cp = {
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
  let Za = Cp;
  Pt = {
    get() {
      return Za;
    },
    set(n) {
      Za = n;
    }
  };
  yh = class extends Me {
    constructor(t) {
      t.resource || (t.resource = Pt.get().createCanvas()), t.width || (t.width = t.resource.width, t.autoDensity || (t.width /= t.resolution)), t.height || (t.height = t.resource.height, t.autoDensity || (t.height /= t.resolution)), super(t), this.uploadMethodId = "image", this.autoDensity = t.autoDensity, this.resizeCanvas(), this.transparent = !!t.transparent;
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
  yh.extension = ht.TextureSource;
  Cs = class extends Me {
    constructor(t) {
      super(t), this.uploadMethodId = "image", this.autoGarbageCollect = true;
    }
    static test(t) {
      return globalThis.HTMLImageElement && t instanceof HTMLImageElement || typeof ImageBitmap < "u" && t instanceof ImageBitmap || globalThis.VideoFrame && t instanceof VideoFrame;
    }
  };
  Cs.extension = ht.TextureSource;
  ni = ((n) => (n[n.INTERACTION = 50] = "INTERACTION", n[n.HIGH = 25] = "HIGH", n[n.NORMAL = 0] = "NORMAL", n[n.LOW = -25] = "LOW", n[n.UTILITY = -50] = "UTILITY", n))(ni || {});
  class Lr {
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
  const xh = class Se {
    constructor() {
      this.autoStart = false, this.deltaTime = 1, this.lastTime = -1, this.speed = 1, this.started = false, this._requestId = null, this._maxElapsedMS = 100, this._minElapsedMS = 0, this._protected = false, this._lastFrame = -1, this._head = new Lr(null, null, 1 / 0), this.deltaMS = 1 / Se.targetFPMS, this.elapsedMS = 1 / Se.targetFPMS, this._tick = (t) => {
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
    add(t, e, s = ni.NORMAL) {
      return this._addListener(new Lr(t, e, s));
    }
    addOnce(t, e, s = ni.NORMAL) {
      return this._addListener(new Lr(t, e, s, true));
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
        this.deltaMS = e, this.deltaTime = this.deltaMS * Se.targetFPMS;
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
      const e = Math.min(Math.max(0, t) / 1e3, Se.targetFPMS);
      this._maxElapsedMS = 1 / e, this._minElapsedMS && t > this.maxFPS && (this.maxFPS = t);
    }
    get maxFPS() {
      return this._minElapsedMS ? Math.round(1e3 / this._minElapsedMS) : 0;
    }
    set maxFPS(t) {
      t === 0 ? this._minElapsedMS = 0 : (t < this.minFPS && (this.minFPS = t), this._minElapsedMS = 1 / (t / 1e3));
    }
    static get shared() {
      if (!Se._shared) {
        const t = Se._shared = new Se();
        t.autoStart = true, t._protected = true;
      }
      return Se._shared;
    }
    static get system() {
      if (!Se._system) {
        const t = Se._system = new Se();
        t.autoStart = true, t._protected = true;
      }
      return Se._system;
    }
  };
  xh.targetFPMS = 0.06;
  let $r;
  Yn = xh;
  async function bh() {
    return $r ?? ($r = (async () => {
      var _a2;
      const t = Pt.get().createCanvas(1, 1).getContext("webgl");
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
    })()), $r;
  }
  const dr = class _h extends Me {
    constructor(t) {
      super(t), this.isReady = false, this.uploadMethodId = "video", t = {
        ..._h.defaultOptions,
        ...t
      }, this._autoUpdate = true, this._isConnectedToTicker = false, this._updateFPS = t.updateFPS || 0, this._msToNextUpdate = 0, this.autoPlay = t.autoPlay !== false, this.alphaMode = t.alphaMode ?? "premultiply-alpha-on-upload", this._videoFrameRequestCallback = this._videoFrameRequestCallback.bind(this), this._videoFrameRequestCallbackHandle = null, this._load = null, this._resolve = null, this._reject = null, this._onCanPlay = this._onCanPlay.bind(this), this._onCanPlayThrough = this._onCanPlayThrough.bind(this), this._onError = this._onError.bind(this), this._onPlayStart = this._onPlayStart.bind(this), this._onPlayStop = this._onPlayStop.bind(this), this._onSeeked = this._onSeeked.bind(this), t.autoLoad !== false && this.load();
    }
    updateFrame() {
      if (!this.destroyed) {
        if (this._updateFPS) {
          const t = Yn.shared.elapsedMS * this.resource.playbackRate;
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
      return (t.readyState === t.HAVE_ENOUGH_DATA || t.readyState === t.HAVE_FUTURE_DATA) && t.width && t.height && (t.complete = true), t.addEventListener("play", this._onPlayStart), t.addEventListener("pause", this._onPlayStop), t.addEventListener("seeked", this._onSeeked), this._isSourceReady() ? this._mediaReady() : (e.preload || t.addEventListener("canplay", this._onCanPlay), t.addEventListener("canplaythrough", this._onCanPlayThrough), t.addEventListener("error", this._onError, true)), this.alphaMode = await bh(), this._load = new Promise((s, i) => {
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
      this._autoUpdate && this._isSourcePlaying() ? !this._updateFPS && this.resource.requestVideoFrameCallback ? (this._isConnectedToTicker && (Yn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0), this._videoFrameRequestCallbackHandle === null && (this._videoFrameRequestCallbackHandle = this.resource.requestVideoFrameCallback(this._videoFrameRequestCallback))) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker || (Yn.shared.add(this.updateFrame, this), this._isConnectedToTicker = true, this._msToNextUpdate = 0)) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker && (Yn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0));
    }
    static test(t) {
      return globalThis.HTMLVideoElement && t instanceof HTMLVideoElement;
    }
  };
  dr.extension = ht.TextureSource;
  dr.defaultOptions = {
    ...Me.defaultOptions,
    autoLoad: true,
    autoPlay: true,
    updateFPS: 0,
    crossorigin: true,
    loop: false,
    muted: true,
    playsinline: true,
    preload: false
  };
  dr.MIME_TYPES = {
    ogv: "video/ogg",
    mov: "video/quicktime",
    m4v: "video/mp4"
  };
  let Ks = dr;
  const He = (n, t, e = false) => (Array.isArray(n) || (n = [
    n
  ]), t ? n.map((s) => typeof s == "string" || e ? t(s) : s) : n);
  class Sp {
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
      return e || Jt(`[Assets] Asset id ${t} was not found in the Cache`), e;
    }
    set(t, e) {
      const s = He(t);
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
        this._cache.has(l) && this._cache.get(l) !== c && Jt("[Cache] already has key:", l), this._cache.set(l, r.get(l));
      });
    }
    remove(t) {
      if (!this._cacheMap.has(t)) {
        Jt(`[Assets] Asset id ${t} was not found in the Cache`);
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
  let bo;
  ne = new Sp();
  bo = [];
  Gt.handleByList(ht.TextureSource, bo);
  function wh(n = {}) {
    const t = n && n.resource, e = t ? n.resource : n, s = t ? n : {
      resource: n
    };
    for (let i = 0; i < bo.length; i++) {
      const r = bo[i];
      if (r.test(e)) return new r(s);
    }
    throw new Error(`Could not find a source type for resource: ${s.resource}`);
  }
  function Tp(n = {}, t = false) {
    const e = n && n.resource, s = e ? n.resource : n, i = e ? n : {
      resource: n
    };
    if (!t && ne.has(s)) return ne.get(s);
    const r = new Ct({
      source: wh(i)
    });
    return r.on("destroy", () => {
      ne.has(s) && ne.remove(s);
    }), t || ne.set(s, r), r;
  }
  function Ep(n, t = false) {
    return typeof n == "string" ? ne.get(n) : n instanceof Me ? new Ct({
      source: n
    }) : Tp(n, t);
  }
  Ct.from = Ep;
  Me.from = wh;
  Gt.add(fh, mh, gh, Ks, Cs, yh, qo);
  var Ln = ((n) => (n[n.Low = 0] = "Low", n[n.Normal = 1] = "Normal", n[n.High = 2] = "High", n))(Ln || {});
  function Ge(n) {
    if (typeof n != "string") throw new TypeError(`Path must be a string. Received ${JSON.stringify(n)}`);
  }
  function $s(n) {
    return n.split("?")[0].split("#")[0];
  }
  function Ap(n) {
    return n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  function kp(n, t, e) {
    return n.replace(new RegExp(Ap(t), "g"), e);
  }
  function Mp(n, t) {
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
  const Ae = {
    toPosix(n) {
      return kp(n, "\\", "/");
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
      Ge(n), n = this.toPosix(n);
      const t = /^file:\/\/\//.exec(n);
      if (t) return t[0];
      const e = /^[^/:]+:\/{0,2}/.exec(n);
      return e ? e[0] : "";
    },
    toAbsolute(n, t, e) {
      if (Ge(n), this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      const s = $s(this.toPosix(t ?? Pt.get().getBaseUrl())), i = $s(this.toPosix(e ?? this.rootname(s)));
      return n = this.toPosix(n), n.startsWith("/") ? Ae.join(i, n.slice(1)) : this.isAbsolute(n) ? n : this.join(s, n);
    },
    normalize(n) {
      if (Ge(n), n.length === 0) return ".";
      if (this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      n = this.toPosix(n);
      let t = "";
      const e = n.startsWith("/");
      this.hasProtocol(n) && (t = this.rootname(n), n = n.slice(t.length));
      const s = n.endsWith("/");
      return n = Mp(n), n.length > 0 && s && (n += "/"), e ? `/${n}` : t + n;
    },
    isAbsolute(n) {
      return Ge(n), n = this.toPosix(n), this.hasProtocol(n) ? true : n.startsWith("/");
    },
    join(...n) {
      if (n.length === 0) return ".";
      let t;
      for (let e = 0; e < n.length; ++e) {
        const s = n[e];
        if (Ge(s), s.length > 0) if (t === void 0) t = s;
        else {
          const i = n[e - 1] ?? "";
          this.joinExtensions.includes(this.extname(i).toLowerCase()) ? t += `/../${s}` : t += `/${s}`;
        }
      }
      return t === void 0 ? "." : this.normalize(t);
    },
    dirname(n) {
      if (Ge(n), n.length === 0) return ".";
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
      Ge(n), n = this.toPosix(n);
      let t = "";
      if (n.startsWith("/") ? t = "/" : t = this.getProtocol(n), this.isUrl(n)) {
        const e = n.indexOf("/", t.length);
        e !== -1 ? t = n.slice(0, e) : t = n, t.endsWith("/") || (t += "/");
      }
      return t;
    },
    basename(n, t) {
      Ge(n), t && Ge(t), n = $s(this.toPosix(n));
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
      Ge(n), n = $s(this.toPosix(n));
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
      Ge(n);
      const t = {
        root: "",
        dir: "",
        base: "",
        ext: "",
        name: ""
      };
      if (n.length === 0) return t;
      n = $s(this.toPosix(n));
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
  function vh(n, t, e, s, i) {
    const r = t[e];
    for (let o = 0; o < r.length; o++) {
      const a = r[o];
      e < t.length - 1 ? vh(n.replace(s[e], a), t, e + 1, s, i) : i.push(n.replace(s[e], a));
    }
  }
  function Pp(n) {
    const t = /\{(.*?)\}/g, e = n.match(t), s = [];
    if (e) {
      const i = [];
      e.forEach((r) => {
        const o = r.substring(1, r.length - 1).split(",");
        i.push(o);
      }), vh(n, i, 0, e, s);
    } else s.push(n);
    return s;
  }
  const Ki = (n) => !Array.isArray(n);
  class As {
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
      return He(e || s, (r) => typeof r == "string" ? r : Array.isArray(r) ? r.map((o) => (o == null ? void 0 : o.src) ?? o) : (r == null ? void 0 : r.src) ? r.src : r, true);
    }
    removeAlias(t, e) {
      this._assetMap[t] && (e && e !== this._resolverHash[t] || (delete this._resolverHash[t], delete this._assetMap[t]));
    }
    addManifest(t) {
      this._manifest && Jt("[Resolver] Manifest already exists, this will be overwritten"), this._manifest = t, t.bundles.forEach((e) => {
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
        this.hasKey(r) && Jt(`[Resolver] already has key: ${r} overwriting`);
      }, He(e).forEach((r) => {
        const { src: o } = r;
        let { data: a, format: l, loadParser: c, parser: h } = r;
        const d = He(o).map((m) => typeof m == "string" ? Pp(m) : Array.isArray(m) ? m : [
          m
        ]), u = this.getAlias(r);
        Array.isArray(u) ? u.forEach(s) : s(u);
        const p = [], f = (m) => {
          const g = this._parsers.find((y) => y.test(m));
          return {
            src: m,
            ...g == null ? void 0 : g.parse(m)
          };
        };
        d.forEach((m) => {
          m.forEach((g) => {
            let y = {};
            if (typeof g != "object" ? y = f(g) : (a = g.data ?? a, l = g.format ?? l, (g.loadParser || g.parser) && (c = g.loadParser ?? c, h = g.parser ?? h), y = {
              ...f(g.src),
              ...g
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
        }), u.forEach((m) => {
          this._assetMap[m] = p;
        });
      });
    }
    resolveBundle(t) {
      const e = Ki(t);
      t = He(t);
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
      const e = Ki(t);
      t = He(t);
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
      return (this._basePath || this._rootPath) && (t.src = Ae.toAbsolute(t.src, this._basePath, this._rootPath)), t.alias = s ?? t.alias ?? [
        t.src
      ], t.src = this._appendDefaultSearchParams(t.src), t.data = {
        ...i || {},
        ...t.data
      }, t.loadParser = r ?? t.loadParser, t.parser = o ?? t.parser, t.format = a ?? t.format ?? Ip(t.src), l !== void 0 && (t.progressSize = l), t;
    }
  }
  As.RETINA_PREFIX = /@([0-9\.]+)x/;
  function Ip(n) {
    return n.split(".").pop().split("?").shift().split("#").shift();
  }
  const _o = (n, t) => {
    const e = t.split("?")[1];
    return e && (n += `?${e}`), n;
  }, Ch = class js {
    constructor(t, e) {
      this.linkedSheets = [];
      let s = t;
      (t == null ? void 0 : t.source) instanceof Me && (s = {
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
        this._callback = t, this._batchIndex = 0, this._frameKeys.length <= js.BATCH_SIZE ? (this._processFrames(0), this._processAnimations(), this._parseComplete()) : this._nextBatch();
      });
    }
    parseSync() {
      return this._processFrames(0, true), this._processAnimations(), this.textures;
    }
    _processFrames(t, e = false) {
      let s = t;
      const i = e ? 1 / 0 : js.BATCH_SIZE;
      for (; s - t < i && s < this._frameKeys.length; ) {
        const r = this._frameKeys[s], o = this._frames[r], a = o.frame;
        if (a) {
          let l = null, c = null;
          const h = o.trimmed !== false && o.sourceSize ? o.sourceSize : o.frame, d = new Ht(0, 0, Math.floor(h.w) / this.resolution, Math.floor(h.h) / this.resolution);
          o.rotated ? l = new Ht(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.h) / this.resolution, Math.floor(a.w) / this.resolution) : l = new Ht(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution), o.trimmed !== false && o.spriteSourceSize && (c = new Ht(Math.floor(o.spriteSourceSize.x) / this.resolution, Math.floor(o.spriteSourceSize.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution)), this.textures[r] = new Ct({
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
      this._processFrames(this._batchIndex * js.BATCH_SIZE), this._batchIndex++, setTimeout(() => {
        this._batchIndex * js.BATCH_SIZE < this._frameKeys.length ? this._nextBatch() : (this._processAnimations(), this._parseComplete());
      }, 0);
    }
    destroy(t = false) {
      var _a2;
      for (const e in this.textures) this.textures[e].destroy();
      this._frames = null, this._frameKeys = null, this.data = null, this.textures = null, t && ((_a2 = this._texture) == null ? void 0 : _a2.destroy(), this.textureSource.destroy()), this._texture = null, this.textureSource = null, this.linkedSheets = [];
    }
  };
  Ch.BATCH_SIZE = 1e3;
  let Qa = Ch;
  const Rp = [
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
  function Sh(n, t, e) {
    const s = {};
    if (n.forEach((i) => {
      s[i] = t;
    }), Object.keys(t.textures).forEach((i) => {
      s[`${t.cachePrefix}${i}`] = t.textures[i];
    }), !e) {
      const i = Ae.dirname(n[0]);
      t.linkedSheets.forEach((r, o) => {
        const a = Sh([
          `${i}/${t.data.meta.related_multi_packs[o]}`
        ], r, true);
        Object.assign(s, a);
      });
    }
    return s;
  }
  const Lp = {
    extension: ht.Asset,
    cache: {
      test: (n) => n instanceof Qa,
      getCacheableAssets: (n, t) => Sh(n, t, false)
    },
    resolver: {
      extension: {
        type: ht.ResolveParser,
        name: "resolveSpritesheet"
      },
      test: (n) => {
        const e = n.split("?")[0].split("."), s = e.pop(), i = e.pop();
        return s === "json" && Rp.includes(i);
      },
      parse: (n) => {
        var _a2;
        const t = n.split(".");
        return {
          resolution: parseFloat(((_a2 = As.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
          format: t[t.length - 2],
          src: n
        };
      }
    },
    loader: {
      name: "spritesheetLoader",
      id: "spritesheet",
      extension: {
        type: ht.LoadParser,
        priority: Ln.Normal,
        name: "spritesheetLoader"
      },
      async testParse(n, t) {
        return Ae.extname(t.src).toLowerCase() === ".json" && !!n.frames;
      },
      async parse(n, t, e) {
        var _a2, _b2;
        const { texture: s, imageFilename: i, textureOptions: r, cachePrefix: o } = (t == null ? void 0 : t.data) ?? {};
        let a = Ae.dirname(t.src);
        a && a.lastIndexOf("/") !== a.length - 1 && (a += "/");
        let l;
        if (s instanceof Ct) l = s;
        else {
          const d = _o(a + (i ?? n.meta.image), t.src);
          l = (await e.load([
            {
              src: d,
              data: r
            }
          ]))[d];
        }
        const c = new Qa({
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
            ((_b2 = t.data) == null ? void 0 : _b2.ignoreMultiPack) || (f = _o(f, t.src), d.push(e.load({
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
  Gt.add(Lp);
  const Br = /* @__PURE__ */ Object.create(null), tl = /* @__PURE__ */ Object.create(null);
  Zo = function(n, t) {
    let e = tl[n];
    return e === void 0 && (Br[t] === void 0 && (Br[t] = 1), tl[n] = e = Br[t]++), e;
  };
  let Ci;
  function Th() {
    return (!Ci || (Ci == null ? void 0 : Ci.isContextLost())) && (Ci = Pt.get().createCanvas().getContext("webgl", {})), Ci;
  }
  let Si;
  function $p() {
    if (!Si) {
      Si = "mediump";
      const n = Th();
      n && n.getShaderPrecisionFormat && (Si = n.getShaderPrecisionFormat(n.FRAGMENT_SHADER, n.HIGH_FLOAT).precision ? "highp" : "mediump");
    }
    return Si;
  }
  function Bp(n, t, e) {
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
  function Op(n, t, e) {
    const s = e ? t.maxSupportedFragmentPrecision : t.maxSupportedVertexPrecision;
    if (n.substring(0, 9) !== "precision") {
      let i = e ? t.requestedFragmentPrecision : t.requestedVertexPrecision;
      return i === "highp" && s !== "highp" && (i = "mediump"), `precision ${i} float;
${n}`;
    } else if (s !== "highp" && n.substring(0, 15) === "precision highp") return n.replace("precision highp", "precision mediump");
    return n;
  }
  function Np(n, t) {
    return t ? `#version 300 es
${n}` : n;
  }
  const Fp = {}, Wp = {};
  function Gp(n, { name: t = "pixi-program" }, e = true) {
    t = t.replace(/\s+/g, "-"), t += e ? "-fragment" : "-vertex";
    const s = e ? Fp : Wp;
    return s[t] ? (s[t]++, t += `-${s[t]}`) : s[t] = 1, n.indexOf("#define SHADER_NAME") !== -1 ? n : `${`#define SHADER_NAME ${t}`}
${n}`;
  }
  function zp(n, t) {
    return t ? n.replace("#version 300 es", "") : n;
  }
  const Or = {
    stripVersion: zp,
    ensurePrecision: Op,
    addProgramDefines: Bp,
    setProgramName: Gp,
    insertVersion: Np
  }, Bs = /* @__PURE__ */ Object.create(null), Eh = class wo {
    constructor(t) {
      t = {
        ...wo.defaultOptions,
        ...t
      };
      const e = t.fragment.indexOf("#version 300 es") !== -1, s = {
        stripVersion: e,
        ensurePrecision: {
          requestedFragmentPrecision: t.preferredFragmentPrecision,
          requestedVertexPrecision: t.preferredVertexPrecision,
          maxSupportedVertexPrecision: "highp",
          maxSupportedFragmentPrecision: $p()
        },
        setProgramName: {
          name: t.name
        },
        addProgramDefines: e,
        insertVersion: e
      };
      let i = t.fragment, r = t.vertex;
      Object.keys(Or).forEach((o) => {
        const a = s[o];
        i = Or[o](i, a, true), r = Or[o](r, a, false);
      }), this.fragment = i, this.vertex = r, this.transformFeedbackVaryings = t.transformFeedbackVaryings, this._key = Zo(`${this.vertex}:${this.fragment}`, "gl-program");
    }
    destroy() {
      this.fragment = null, this.vertex = null, this._attributeData = null, this._uniformData = null, this._uniformBlockData = null, this.transformFeedbackVaryings = null, Bs[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex}:${t.fragment}`;
      return Bs[e] || (Bs[e] = new wo(t), Bs[e]._cacheKey = e), Bs[e];
    }
  };
  Eh.defaultOptions = {
    preferredVertexPrecision: "highp",
    preferredFragmentPrecision: "mediump"
  };
  Qo = Eh;
  const el = {
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
  Ji = function(n) {
    return el[n] ?? el.float32;
  };
  const Dp = {
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
  }, nl = /@location\((\d+)\)\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_<>]+)(?:,|\s|\)|$)/g;
  function sl(n, t) {
    let e;
    for (; (e = nl.exec(n)) !== null; ) {
      const s = Dp[e[3]] ?? "float32";
      t[e[2]] = {
        location: parseInt(e[1], 10),
        format: s,
        stride: Ji(s).stride,
        offset: 0,
        instance: false,
        start: 0
      };
    }
    nl.lastIndex = 0;
  }
  function Hp(n) {
    return n.replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  }
  function Up({ source: n, entryPoint: t }) {
    const e = {}, s = Hp(n), i = s.indexOf(`fn ${t}(`);
    if (i === -1) return e;
    const r = s.indexOf("->", i);
    if (r === -1) return e;
    const o = s.substring(i, r);
    if (sl(o, e), Object.keys(e).length === 0) {
      const a = o.match(/\(\s*\w+\s*:\s*(\w+)/);
      if (a) {
        const l = a[1], c = new RegExp(`struct\\s+${l}\\s*\\{([^}]+)\\}`, "s"), h = s.match(c);
        h && sl(h[1], e);
      }
    }
    return e;
  }
  function Nr(n) {
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
      const u = d.match(l)[1], p = d.match(a).reduce((f, m) => {
        const [g, y] = m.split(":");
        return f[g.trim()] = y.trim(), f;
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
  var jn = ((n) => (n[n.VERTEX = 1] = "VERTEX", n[n.FRAGMENT = 2] = "FRAGMENT", n[n.COMPUTE = 4] = "COMPUTE", n))(jn || {});
  function jp({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = []), s.isUniform ? t[s.group].push({
        binding: s.binding,
        visibility: jn.VERTEX | jn.FRAGMENT,
        buffer: {
          type: "uniform"
        }
      }) : s.type === "sampler" ? t[s.group].push({
        binding: s.binding,
        visibility: jn.FRAGMENT,
        sampler: {
          type: "filtering"
        }
      }) : s.type === "texture_2d" || s.type.startsWith("texture_2d<") ? t[s.group].push({
        binding: s.binding,
        visibility: jn.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d",
          multisampled: false
        }
      }) : s.type === "texture_2d_array" || s.type.startsWith("texture_2d_array<") ? t[s.group].push({
        binding: s.binding,
        visibility: jn.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d-array",
          multisampled: false
        }
      }) : (s.type === "texture_cube" || s.type.startsWith("texture_cube<")) && t[s.group].push({
        binding: s.binding,
        visibility: jn.FRAGMENT,
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
  function Vp({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = {}), t[s.group][s.name] = s.binding;
    }
    return t;
  }
  function Yp(n, t) {
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
  const Os = /* @__PURE__ */ Object.create(null);
  fi = class {
    constructor(t) {
      var _a2, _b2;
      this._layoutKey = 0, this._attributeLocationsKey = 0;
      const { fragment: e, vertex: s, layout: i, gpuLayout: r, name: o } = t;
      if (this.name = o, this.fragment = e, this.vertex = s, e.source === s.source) {
        const a = Nr(e.source);
        this.structsAndGroups = a;
      } else {
        const a = Nr(s.source), l = Nr(e.source);
        this.structsAndGroups = Yp(a, l);
      }
      this.layout = i ?? Vp(this.structsAndGroups), this.gpuLayout = r ?? jp(this.structsAndGroups), this.autoAssignGlobalUniforms = ((_a2 = this.layout[0]) == null ? void 0 : _a2.globalUniforms) !== void 0, this.autoAssignLocalUniforms = ((_b2 = this.layout[1]) == null ? void 0 : _b2.localUniforms) !== void 0, this._generateProgramKey();
    }
    _generateProgramKey() {
      const { vertex: t, fragment: e } = this, s = t.source + e.source + t.entryPoint + e.entryPoint;
      this._layoutKey = Zo(s, "program");
    }
    get attributeData() {
      return this._attributeData ?? (this._attributeData = Up(this.vertex)), this._attributeData;
    }
    destroy() {
      this.gpuLayout = null, this.layout = null, this.structsAndGroups = null, this.fragment = null, this.vertex = null, Os[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex.source}:${t.fragment.source}:${t.fragment.entryPoint}:${t.vertex.entryPoint}`;
      return Os[e] || (Os[e] = new fi(t), Os[e]._cacheKey = e), Os[e];
    }
  };
  const Ah = [
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
  ], Xp = Ah.reduce((n, t) => (n[t] = true, n), {});
  function qp(n, t) {
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
  const kh = class Mh {
    constructor(t, e) {
      this._touched = 0, this.uid = ee("uniform"), this._resourceType = "uniformGroup", this._resourceId = ee("resource"), this.isUniformGroup = true, this._dirtyId = 0, this.destroyed = false, e = {
        ...Mh.defaultOptions,
        ...e
      }, this.uniformStructures = t;
      const s = {};
      for (const i in t) {
        const r = t[i];
        if (r.name = i, r.size = r.size ?? 1, !Xp[r.type]) {
          const o = r.type.match(/^array<(\w+(?:<\w+>)?),\s*(\d+)>$/);
          if (o) {
            const [, a, l] = o;
            throw new Error(`Uniform type ${r.type} is not supported. Use type: '${a}', size: ${l} instead.`);
          }
          throw new Error(`Uniform type ${r.type} is not supported. Supported uniform types are: ${Ah.join(", ")}`);
        }
        r.value ?? (r.value = qp(r.type, r.size)), s[i] = r.value;
      }
      this.uniforms = s, this._dirtyId = 1, this.ubo = e.ubo, this.isStatic = e.isStatic, this._signature = Zo(Object.keys(s).map((i) => `${i}-${t[i].type}`).join("-"), "uniform-group");
    }
    update() {
      this._dirtyId++;
    }
  };
  kh.defaultOptions = {
    ubo: false,
    isStatic: false
  };
  ta = kh;
  Vi = class {
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
  vo = ((n) => (n[n.WEBGL = 1] = "WEBGL", n[n.WEBGPU = 2] = "WEBGPU", n[n.CANVAS = 4] = "CANVAS", n[n.BOTH = 3] = "BOTH", n))(vo || {});
  ur = class extends rn {
    constructor(t) {
      super(), this.uid = ee("shader"), this._uniformBindMap = /* @__PURE__ */ Object.create(null), this._ownedBindGroups = [], this._destroyed = false;
      let { gpuProgram: e, glProgram: s, groups: i, resources: r, compatibleRenderers: o, groupMap: a } = t;
      this.gpuProgram = e, this.glProgram = s, o === void 0 && (o = 0, e && (o |= vo.WEBGPU), s && (o |= vo.WEBGL)), this.compatibleRenderers = o;
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
        for (const h in r) l[h] || (i[99] || (i[99] = new Vi(), this._ownedBindGroups.push(i[99])), l[h] = {
          group: 99,
          binding: c,
          name: h
        }, a[99] = a[99] || {}, a[99][c] = h, c++);
        for (const h in r) {
          const d = h;
          let u = r[h];
          !u.source && !u._resourceType && (u = new ta(u));
          const p = l[d];
          p && (i[p.group] || (i[p.group] = new Vi(), this._ownedBindGroups.push(i[p.group])), i[p.group].setResource(u, p.binding));
        }
      }
      this.groups = i, this._uniformBindMap = a, this.resources = this._buildResourceAccessor(i, l);
    }
    addResource(t, e, s) {
      var i, r;
      (i = this._uniformBindMap)[e] || (i[e] = {}), (r = this._uniformBindMap[e])[s] || (r[s] = t), this.groups[e] || (this.groups[e] = new Vi(), this._ownedBindGroups.push(this.groups[e]));
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
      return e && (r = fi.from(e)), s && (o = Qo.from(s)), new ur({
        gpuProgram: r,
        glProgram: o,
        ...i
      });
    }
  };
  const Kp = {
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
  }, Fr = 0, Wr = 1, Gr = 2, zr = 3, Dr = 4, Hr = 5, Co = class Ph {
    constructor() {
      this.data = 0, this.blendMode = "normal", this.polygonOffset = 0, this.blend = true, this.depthMask = true;
    }
    get blend() {
      return !!(this.data & 1 << Fr);
    }
    set blend(t) {
      !!(this.data & 1 << Fr) !== t && (this.data ^= 1 << Fr);
    }
    get offsets() {
      return !!(this.data & 1 << Wr);
    }
    set offsets(t) {
      !!(this.data & 1 << Wr) !== t && (this.data ^= 1 << Wr);
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
      return !!(this.data & 1 << Gr);
    }
    set culling(t) {
      !!(this.data & 1 << Gr) !== t && (this.data ^= 1 << Gr);
    }
    get depthTest() {
      return !!(this.data & 1 << zr);
    }
    set depthTest(t) {
      !!(this.data & 1 << zr) !== t && (this.data ^= 1 << zr);
    }
    get depthMask() {
      return !!(this.data & 1 << Hr);
    }
    set depthMask(t) {
      !!(this.data & 1 << Hr) !== t && (this.data ^= 1 << Hr);
    }
    get clockwiseFrontFace() {
      return !!(this.data & 1 << Dr);
    }
    set clockwiseFrontFace(t) {
      !!(this.data & 1 << Dr) !== t && (this.data ^= 1 << Dr);
    }
    get blendMode() {
      return this._blendMode;
    }
    set blendMode(t) {
      this.blend = t !== "none", this._blendMode = t, this._blendModeId = Kp[t] || 0;
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
      const t = new Ph();
      return t.depthTest = false, t.blend = true, t;
    }
  };
  Co.default2d = Co.for2d();
  Zi = Co;
  const So = [];
  Gt.handleByNamedList(ht.Environment, So);
  async function Jp(n) {
    if (!n) for (let t = 0; t < So.length; t++) {
      const e = So[t];
      if (e.value.test()) {
        await e.value.load();
        return;
      }
    }
  }
  let Ns;
  Zp = function() {
    if (typeof Ns == "boolean") return Ns;
    try {
      Ns = new Function("param1", "param2", "param3", "return param1[param2] === param3;")({
        a: "b"
      }, "a", "b") === true;
    } catch {
      Ns = false;
    }
    return Ns;
  };
  function il(n, t, e = 2) {
    const s = t && t.length, i = s ? t[0] * e : n.length;
    let r = Ih(n, 0, i, e, true);
    const o = [];
    if (!r || r.next === r.prev) return o;
    let a, l, c;
    if (s && (r = sf(n, t, r, e)), n.length > 80 * e) {
      a = n[0], l = n[1];
      let h = a, d = l;
      for (let u = e; u < i; u += e) {
        const p = n[u], f = n[u + 1];
        p < a && (a = p), f < l && (l = f), p > h && (h = p), f > d && (d = f);
      }
      c = Math.max(h - a, d - l), c = c !== 0 ? 32767 / c : 0;
    }
    return si(r, o, e, a, l, c, 0), o;
  }
  function Ih(n, t, e, s, i) {
    let r;
    if (i === mf(n, t, e, s) > 0) for (let o = t; o < e; o += s) r = rl(o / s | 0, n[o], n[o + 1], r);
    else for (let o = e - s; o >= t; o -= s) r = rl(o / s | 0, n[o], n[o + 1], r);
    return r && Ss(r, r.next) && (ri(r), r = r.next), r;
  }
  function Jn(n, t) {
    if (!n) return n;
    t || (t = n);
    let e = n, s;
    do
      if (s = false, !e.steiner && (Ss(e, e.next) || Zt(e.prev, e, e.next) === 0)) {
        if (ri(e), e = t = e.prev, e === e.next) break;
        s = true;
      } else e = e.next;
    while (s || e !== t);
    return t;
  }
  function si(n, t, e, s, i, r, o) {
    if (!n) return;
    !o && r && cf(n, s, i, r);
    let a = n;
    for (; n.prev !== n.next; ) {
      const l = n.prev, c = n.next;
      if (r ? tf(n, s, i, r) : Qp(n)) {
        t.push(l.i, n.i, c.i), ri(n), n = c.next, a = c.next;
        continue;
      }
      if (n = c, n === a) {
        o ? o === 1 ? (n = ef(Jn(n), t), si(n, t, e, s, i, r, 2)) : o === 2 && nf(n, t, e, s, i, r) : si(Jn(n), t, e, s, i, r, 1);
        break;
      }
    }
  }
  function Qp(n) {
    const t = n.prev, e = n, s = n.next;
    if (Zt(t, e, s) >= 0) return false;
    const i = t.x, r = e.x, o = s.x, a = t.y, l = e.y, c = s.y, h = Math.min(i, r, o), d = Math.min(a, l, c), u = Math.max(i, r, o), p = Math.max(a, l, c);
    let f = s.next;
    for (; f !== t; ) {
      if (f.x >= h && f.x <= u && f.y >= d && f.y <= p && Vs(i, a, r, l, o, c, f.x, f.y) && Zt(f.prev, f, f.next) >= 0) return false;
      f = f.next;
    }
    return true;
  }
  function tf(n, t, e, s) {
    const i = n.prev, r = n, o = n.next;
    if (Zt(i, r, o) >= 0) return false;
    const a = i.x, l = r.x, c = o.x, h = i.y, d = r.y, u = o.y, p = Math.min(a, l, c), f = Math.min(h, d, u), m = Math.max(a, l, c), g = Math.max(h, d, u), y = To(p, f, t, e, s), w = To(m, g, t, e, s);
    let x = n.prevZ, _ = n.nextZ;
    for (; x && x.z >= y && _ && _.z <= w; ) {
      if (x.x >= p && x.x <= m && x.y >= f && x.y <= g && x !== i && x !== o && Vs(a, h, l, d, c, u, x.x, x.y) && Zt(x.prev, x, x.next) >= 0 || (x = x.prevZ, _.x >= p && _.x <= m && _.y >= f && _.y <= g && _ !== i && _ !== o && Vs(a, h, l, d, c, u, _.x, _.y) && Zt(_.prev, _, _.next) >= 0)) return false;
      _ = _.nextZ;
    }
    for (; x && x.z >= y; ) {
      if (x.x >= p && x.x <= m && x.y >= f && x.y <= g && x !== i && x !== o && Vs(a, h, l, d, c, u, x.x, x.y) && Zt(x.prev, x, x.next) >= 0) return false;
      x = x.prevZ;
    }
    for (; _ && _.z <= w; ) {
      if (_.x >= p && _.x <= m && _.y >= f && _.y <= g && _ !== i && _ !== o && Vs(a, h, l, d, c, u, _.x, _.y) && Zt(_.prev, _, _.next) >= 0) return false;
      _ = _.nextZ;
    }
    return true;
  }
  function ef(n, t) {
    let e = n;
    do {
      const s = e.prev, i = e.next.next;
      !Ss(s, i) && Lh(s, e, e.next, i) && ii(s, i) && ii(i, s) && (t.push(s.i, e.i, i.i), ri(e), ri(e.next), e = n = i), e = e.next;
    } while (e !== n);
    return Jn(e);
  }
  function nf(n, t, e, s, i, r) {
    let o = n;
    do {
      let a = o.next.next;
      for (; a !== o.prev; ) {
        if (o.i !== a.i && uf(o, a)) {
          let l = $h(o, a);
          o = Jn(o, o.next), l = Jn(l, l.next), si(o, t, e, s, i, r, 0), si(l, t, e, s, i, r, 0);
          return;
        }
        a = a.next;
      }
      o = o.next;
    } while (o !== n);
  }
  function sf(n, t, e, s) {
    const i = [];
    for (let r = 0, o = t.length; r < o; r++) {
      const a = t[r] * s, l = r < o - 1 ? t[r + 1] * s : n.length, c = Ih(n, a, l, s, false);
      c === c.next && (c.steiner = true), i.push(df(c));
    }
    i.sort(rf);
    for (let r = 0; r < i.length; r++) e = of(i[r], e);
    return e;
  }
  function rf(n, t) {
    let e = n.x - t.x;
    if (e === 0 && (e = n.y - t.y, e === 0)) {
      const s = (n.next.y - n.y) / (n.next.x - n.x), i = (t.next.y - t.y) / (t.next.x - t.x);
      e = s - i;
    }
    return e;
  }
  function of(n, t) {
    const e = af(n, t);
    if (!e) return t;
    const s = $h(e, n);
    return Jn(s, s.next), Jn(e, e.next);
  }
  function af(n, t) {
    let e = t;
    const s = n.x, i = n.y;
    let r = -1 / 0, o;
    if (Ss(n, e)) return e;
    do {
      if (Ss(n, e.next)) return e.next;
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
      if (s >= e.x && e.x >= l && s !== e.x && Rh(i < c ? s : r, i, l, c, i < c ? r : s, i, e.x, e.y)) {
        const d = Math.abs(i - e.y) / (s - e.x);
        ii(e, n) && (d < h || d === h && (e.x > o.x || e.x === o.x && lf(o, e))) && (o = e, h = d);
      }
      e = e.next;
    } while (e !== a);
    return o;
  }
  function lf(n, t) {
    return Zt(n.prev, n, t.prev) < 0 && Zt(t.next, n, n.next) < 0;
  }
  function cf(n, t, e, s) {
    let i = n;
    do
      i.z === 0 && (i.z = To(i.x, i.y, t, e, s)), i.prevZ = i.prev, i.nextZ = i.next, i = i.next;
    while (i !== n);
    i.prevZ.nextZ = null, i.prevZ = null, hf(i);
  }
  function hf(n) {
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
  function To(n, t, e, s, i) {
    return n = (n - e) * i | 0, t = (t - s) * i | 0, n = (n | n << 8) & 16711935, n = (n | n << 4) & 252645135, n = (n | n << 2) & 858993459, n = (n | n << 1) & 1431655765, t = (t | t << 8) & 16711935, t = (t | t << 4) & 252645135, t = (t | t << 2) & 858993459, t = (t | t << 1) & 1431655765, n | t << 1;
  }
  function df(n) {
    let t = n, e = n;
    do
      (t.x < e.x || t.x === e.x && t.y < e.y) && (e = t), t = t.next;
    while (t !== n);
    return e;
  }
  function Rh(n, t, e, s, i, r, o, a) {
    return (i - o) * (t - a) >= (n - o) * (r - a) && (n - o) * (s - a) >= (e - o) * (t - a) && (e - o) * (r - a) >= (i - o) * (s - a);
  }
  function Vs(n, t, e, s, i, r, o, a) {
    return !(n === o && t === a) && Rh(n, t, e, s, i, r, o, a);
  }
  function uf(n, t) {
    return n.next.i !== t.i && n.prev.i !== t.i && !pf(n, t) && (ii(n, t) && ii(t, n) && ff(n, t) && (Zt(n.prev, n, t.prev) || Zt(n, t.prev, t)) || Ss(n, t) && Zt(n.prev, n, n.next) > 0 && Zt(t.prev, t, t.next) > 0);
  }
  function Zt(n, t, e) {
    return (t.y - n.y) * (e.x - t.x) - (t.x - n.x) * (e.y - t.y);
  }
  function Ss(n, t) {
    return n.x === t.x && n.y === t.y;
  }
  function Lh(n, t, e, s) {
    const i = Ei(Zt(n, t, e)), r = Ei(Zt(n, t, s)), o = Ei(Zt(e, s, n)), a = Ei(Zt(e, s, t));
    return !!(i !== r && o !== a || i === 0 && Ti(n, e, t) || r === 0 && Ti(n, s, t) || o === 0 && Ti(e, n, s) || a === 0 && Ti(e, t, s));
  }
  function Ti(n, t, e) {
    return t.x <= Math.max(n.x, e.x) && t.x >= Math.min(n.x, e.x) && t.y <= Math.max(n.y, e.y) && t.y >= Math.min(n.y, e.y);
  }
  function Ei(n) {
    return n > 0 ? 1 : n < 0 ? -1 : 0;
  }
  function pf(n, t) {
    let e = n;
    do {
      if (e.i !== n.i && e.next.i !== n.i && e.i !== t.i && e.next.i !== t.i && Lh(e, e.next, n, t)) return true;
      e = e.next;
    } while (e !== n);
    return false;
  }
  function ii(n, t) {
    return Zt(n.prev, n, n.next) < 0 ? Zt(n, t, n.next) >= 0 && Zt(n, n.prev, t) >= 0 : Zt(n, t, n.prev) < 0 || Zt(n, n.next, t) < 0;
  }
  function ff(n, t) {
    let e = n, s = false;
    const i = (n.x + t.x) / 2, r = (n.y + t.y) / 2;
    do
      e.y > r != e.next.y > r && e.next.y !== e.y && i < (e.next.x - e.x) * (r - e.y) / (e.next.y - e.y) + e.x && (s = !s), e = e.next;
    while (e !== n);
    return s;
  }
  function $h(n, t) {
    const e = Eo(n.i, n.x, n.y), s = Eo(t.i, t.x, t.y), i = n.next, r = t.prev;
    return n.next = t, t.prev = n, e.next = i, i.prev = e, s.next = e, e.prev = s, r.next = s, s.prev = r, s;
  }
  function rl(n, t, e, s) {
    const i = Eo(n, t, e);
    return s ? (i.next = s.next, i.prev = s, s.next.prev = i, s.next = i) : (i.prev = i, i.next = i), i;
  }
  function ri(n) {
    n.next.prev = n.prev, n.prev.next = n.next, n.prevZ && (n.prevZ.nextZ = n.nextZ), n.nextZ && (n.nextZ.prevZ = n.prevZ);
  }
  function Eo(n, t, e) {
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
  function mf(n, t, e, s) {
    let i = 0;
    for (let r = t, o = e - s; r < e; r += s) i += (n[o] - n[r]) * (n[r + 1] + n[o + 1]), o = r;
    return i;
  }
  const gf = il.default || il;
  Bh = ((n) => (n[n.NONE = 0] = "NONE", n[n.COLOR = 16384] = "COLOR", n[n.STENCIL = 1024] = "STENCIL", n[n.DEPTH = 256] = "DEPTH", n[n.COLOR_DEPTH = 16640] = "COLOR_DEPTH", n[n.COLOR_STENCIL = 17408] = "COLOR_STENCIL", n[n.DEPTH_STENCIL = 1280] = "DEPTH_STENCIL", n[n.ALL = 17664] = "ALL", n))(Bh || {});
  yf = class {
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
  const xf = [
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
  ], Oh = class Nh extends rn {
    constructor(t) {
      super(), this.tick = 0, this.uid = ee("renderer"), this.runners = /* @__PURE__ */ Object.create(null), this.renderPipes = /* @__PURE__ */ Object.create(null), this._initOptions = {}, this._systemsHash = /* @__PURE__ */ Object.create(null), this.type = t.type, this.name = t.name, this.config = t;
      const e = [
        ...xf,
        ...this.config.runners ?? []
      ];
      this._addRunners(...e), this._unsafeEvalCheck();
    }
    async init(t = {}) {
      const e = t.skipExtensionImports === true ? true : t.manageImports === false;
      await Jp(e), this._addSystems(this.config.systems), this._addPipes(this.config.renderPipes, this.config.renderPipeAdaptors);
      for (const s in this._systemsHash) t = {
        ...this._systemsHash[s].constructor.defaultOptions,
        ...t
      };
      t = {
        ...Nh.defaultOptions,
        ...t
      }, this._roundPixels = t.roundPixels ? 1 : 0;
      for (let s = 0; s < this.runners.init.items.length; s++) await this.runners.init.items[s].init(t);
      this._initOptions = t;
    }
    render(t, e) {
      this.tick++;
      let s = t;
      if (s instanceof Ut && (s = {
        container: s
      }, e && (Mt(te, "passing a second argument is deprecated, please use render options instead"), s.target = e.renderTexture)), s.target || (s.target = this.view.renderTarget), s.target === this.view.renderTarget && (this._lastObjectRendered = s.container, s.clearColor ?? (s.clearColor = this.background.colorRgba), s.clear ?? (s.clear = this.background.clearBeforeRender)), s.clearColor) {
        const i = Array.isArray(s.clearColor) && s.clearColor.length === 4;
        s.clearColor = i ? s.clearColor : Xt.shared.setValue(s.clearColor).toArray();
      }
      s.transform || (s.container.updateLocalTransform(), s.transform = s.container.localTransform), s.container.visible && (s.container.enableRenderGroup(), this.runners.prerender.emit(s), this.runners.renderStart.emit(s), this.runners.render.emit(s), this.runners.renderEnd.emit(s), this.runners.postrender.emit(s));
    }
    resize(t, e, s) {
      const i = this.view.resolution;
      this.view.resize(t, e, s), this.emit("resize", this.view.screen.width, this.view.screen.height, this.view.resolution), s !== void 0 && s !== i && this.runners.resolutionChange.emit(s);
    }
    clear(t = {}) {
      const e = this;
      t.target || (t.target = e.renderTarget.renderTarget), t.clearColor || (t.clearColor = this.background.colorRgba), t.clear ?? (t.clear = Bh.ALL);
      const { clear: s, clearColor: i, target: r, mipLevel: o, layer: a } = t;
      Xt.shared.setValue(i ?? this.background.colorRgba), e.renderTarget.clear(r, s, Xt.shared.toArray(), o ?? 0, a ?? 0);
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
        this.runners[e] = new yf(e);
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
      this.runners.destroy.items.reverse(), this.runners.destroy.emit(t), (t === true || typeof t == "object" && t.releaseGlobalResources) && pi.release(), Object.values(this.runners).forEach((e) => {
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
      if (!Zp()) throw new Error("Current environment does not allow unsafe-eval, please use pixi.js/unsafe-eval module to enable support.");
    }
    resetState() {
      this.runners.resetState.emit();
    }
  };
  Oh.defaultOptions = {
    resolution: 1,
    failIfMajorPerformanceCaveat: false,
    roundPixels: false
  };
  let Ai;
  Fh = Oh;
  function bf(n) {
    return Ai !== void 0 || (Ai = (() => {
      var _a2;
      const t = {
        stencil: true,
        failIfMajorPerformanceCaveat: n ?? Fh.defaultOptions.failIfMajorPerformanceCaveat
      };
      try {
        if (!Pt.get().getWebGLRenderingContext()) return false;
        let s = Pt.get().createCanvas().getContext("webgl", t);
        const i = !!((_a2 = s == null ? void 0 : s.getContextAttributes()) == null ? void 0 : _a2.stencil);
        if (s) {
          const r = s.getExtension("WEBGL_lose_context");
          r && r.loseContext();
        }
        return s = null, i;
      } catch {
        return false;
      }
    })()), Ai;
  }
  let ki;
  async function _f(n = {}) {
    return ki !== void 0 || (ki = await (async () => {
      const t = Pt.get().getNavigator().gpu;
      if (!t) return false;
      try {
        return await (await t.requestAdapter(n)).requestDevice(), true;
      } catch {
        return false;
      }
    })()), ki;
  }
  const ol = [
    "webgl",
    "webgpu",
    "canvas"
  ];
  async function wf(n) {
    let t = [];
    n.preference ? (t.push(n.preference), ol.forEach((r) => {
      r !== n.preference && t.push(r);
    })) : t = ol.slice();
    let e, s = {};
    for (let r = 0; r < t.length; r++) {
      const o = t[r];
      if (o === "webgpu" && await _f()) {
        const { WebGPURenderer: a } = await Pn(async () => {
          const { WebGPURenderer: l } = await import("./WebGPURenderer-CdejW-a0.js");
          return {
            WebGPURenderer: l
          };
        }, __vite__mapDeps([3,4,5,2]));
        e = a, s = {
          ...n,
          ...n.webgpu
        };
        break;
      } else if (o === "webgl" && bf(n.failIfMajorPerformanceCaveat ?? Fh.defaultOptions.failIfMajorPerformanceCaveat)) {
        const { WebGLRenderer: a } = await Pn(async () => {
          const { WebGLRenderer: l } = await import("./WebGLRenderer-BTBXxSvE.js");
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
        const { CanvasRenderer: a } = await Pn(async () => {
          const { CanvasRenderer: l } = await import("./CanvasRenderer-CCJ0mBY9.js");
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
  Wh = "8.17.1";
  class Gh {
    static init() {
      var _a2;
      (_a2 = globalThis.__PIXI_APP_INIT__) == null ? void 0 : _a2.call(globalThis, this, Wh);
    }
    static destroy() {
    }
  }
  Gh.extension = ht.Application;
  vf = class {
    constructor(t) {
      this._renderer = t;
    }
    init() {
      var _a2;
      (_a2 = globalThis.__PIXI_RENDERER_INIT__) == null ? void 0 : _a2.call(globalThis, this._renderer, Wh);
    }
    destroy() {
      this._renderer = null;
    }
  };
  vf.extension = {
    type: [
      ht.WebGLSystem,
      ht.WebGPUSystem
    ],
    name: "initHook",
    priority: -10
  };
  class zh {
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
  zh.extension = ht.Application;
  class Dh {
    static init(t) {
      t = Object.assign({
        autoStart: true,
        sharedTicker: false
      }, t), Object.defineProperty(this, "ticker", {
        configurable: true,
        set(e) {
          this._ticker && this._ticker.remove(this.render, this), this._ticker = e, e && e.add(this.render, this, ni.LOW);
        },
        get() {
          return this._ticker;
        }
      }), this.stop = () => {
        this._ticker.stop();
      }, this.start = () => {
        this._ticker.start();
      }, this._ticker = null, this.ticker = t.sharedTicker ? Yn.shared : new Yn(), t.autoStart && this.start();
    }
    static destroy() {
      if (this._ticker) {
        const t = this._ticker;
        this.ticker = null, t.destroy();
      }
    }
  }
  Dh.extension = ht.Application;
  Gt.add(zh);
  Gt.add(Dh);
  const Hh = class Ao {
    constructor(...t) {
      this.stage = new Ut(), t[0] !== void 0 && Mt(te, "Application constructor options are deprecated, please use Application.init() instead.");
    }
    async init(t) {
      t = {
        ...t
      }, this.stage || (this.stage = new Ut()), this.renderer = await wf(t), Ao._plugins.forEach((e) => {
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
      return Mt(te, "Application.view is deprecated, please use Application.canvas instead."), this.renderer.canvas;
    }
    get screen() {
      return this.renderer.screen;
    }
    destroy(t = false, e = false) {
      const s = Ao._plugins.slice(0);
      s.reverse(), s.forEach((i) => {
        i.destroy.call(this);
      }), this.stage.destroy(e), this.stage = null, this.renderer.destroy(t), this.renderer = null;
    }
  };
  Hh._plugins = [];
  ea = Hh;
  Gt.handleByList(ht.Application, ea._plugins);
  Gt.add(Gh);
  const Ur = {
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
        for (const m in p) {
          const g = p[m].split("="), y = g[0], w = g[1].replace(/"/gm, ""), x = parseFloat(w), _ = isNaN(x) ? w : x;
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
  }, al = {
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
  }, ll = {
    test(n) {
      return typeof n == "string" && n.match(/<font(\s|>)/) ? al.test(Pt.get().parseXML(n)) : false;
    },
    parse(n) {
      return al.parse(Pt.get().parseXML(n));
    }
  }, Cf = [
    ".xml",
    ".fnt"
  ], Sf = {
    extension: {
      type: ht.CacheParser,
      name: "cacheBitmapFont"
    },
    test: (n) => !!(n == null ? void 0 : n.pages) && !!(n == null ? void 0 : n.chars) && typeof (n == null ? void 0 : n.fontFamily) == "string" && n.fontFamily !== "",
    getCacheableAssets(n, t) {
      const e = {};
      return n.forEach((s) => {
        e[s] = t, e[`${s}-bitmap`] = t;
      }), e[`${t.fontFamily}-bitmap`] = t, e;
    }
  }, Tf = {
    extension: {
      type: ht.LoadParser,
      priority: Ln.Normal
    },
    name: "loadBitmapFont",
    id: "bitmap-font",
    test(n) {
      return Cf.includes(Ae.extname(n).toLowerCase());
    },
    async testParse(n) {
      return Ur.test(n) || ll.test(n);
    },
    async parse(n, t, e) {
      const s = Ur.test(n) ? Ur.parse(n) : ll.parse(n), { src: i } = t, { pages: r } = s, o = [], a = s.distanceField ? {
        scaleMode: "linear",
        alphaMode: "premultiply-alpha-on-upload",
        autoGenerateMipmaps: false,
        resolution: 1
      } : {};
      for (let u = 0; u < r.length; ++u) {
        const p = r[u].file;
        let f = Ae.join(Ae.dirname(i), p);
        f = _o(f, i), o.push({
          src: f,
          data: a
        });
      }
      const [l, { BitmapFont: c }] = await Promise.all([
        e.load(o),
        Pn(() => import("./BitmapFont-CptZlvxu.js"), [])
      ]), h = o.map((u) => l[u.src]);
      return new c({
        data: s,
        textures: h
      }, i);
    },
    async load(n, t) {
      return await (await Pt.get().fetch(n)).text();
    },
    async unload(n, t, e) {
      await Promise.all(n.pages.map((s) => e.unload(s.texture.source._sourceOrigin))), n.destroy();
    }
  };
  class Ef {
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
  const Af = {
    extension: {
      type: ht.CacheParser,
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
  async function Uh(n) {
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
  const kf = {
    extension: {
      type: ht.DetectionParser,
      priority: 1
    },
    test: async () => Uh("data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A="),
    add: async (n) => [
      ...n,
      "avif"
    ],
    remove: async (n) => n.filter((t) => t !== "avif")
  }, cl = [
    "png",
    "jpg",
    "jpeg"
  ], Mf = {
    extension: {
      type: ht.DetectionParser,
      priority: -1
    },
    test: () => Promise.resolve(true),
    add: async (n) => [
      ...n,
      ...cl
    ],
    remove: async (n) => n.filter((t) => !cl.includes(t))
  }, Pf = "WorkerGlobalScope" in globalThis && globalThis instanceof globalThis.WorkerGlobalScope;
  function pr(n) {
    return Pf ? false : document.createElement("video").canPlayType(n) !== "";
  }
  const If = {
    extension: {
      type: ht.DetectionParser,
      priority: 0
    },
    test: async () => pr("video/mp4"),
    add: async (n) => [
      ...n,
      "mp4",
      "m4v"
    ],
    remove: async (n) => n.filter((t) => t !== "mp4" && t !== "m4v")
  }, Rf = {
    extension: {
      type: ht.DetectionParser,
      priority: 0
    },
    test: async () => pr("video/ogg"),
    add: async (n) => [
      ...n,
      "ogv"
    ],
    remove: async (n) => n.filter((t) => t !== "ogv")
  }, Lf = {
    extension: {
      type: ht.DetectionParser,
      priority: 0
    },
    test: async () => pr("video/webm"),
    add: async (n) => [
      ...n,
      "webm"
    ],
    remove: async (n) => n.filter((t) => t !== "webm")
  }, $f = {
    extension: {
      type: ht.DetectionParser,
      priority: 0
    },
    test: async () => Uh("data:image/webp;base64,UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAAAfQ//73v/+BiOh/AAA="),
    add: async (n) => [
      ...n,
      "webp"
    ],
    remove: async (n) => n.filter((t) => t !== "webp")
  }, jh = class Yi {
    constructor() {
      this.loadOptions = {
        ...Yi.defaultOptions
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
        if ((e.parser || e.loadParser) && (r = this._parserHash[e.parser || e.loadParser], e.loadParser && Jt(`[Assets] "loadParser" is deprecated, use "parser" instead for ${t}`), r || Jt(`[Assets] specified load parser "${e.parser || e.loadParser}" not found while loading ${t}`)), !r) {
          for (let o = 0; o < this.parsers.length; o++) {
            const a = this.parsers[o];
            if (a.load && ((_a2 = a.test) == null ? void 0 : _a2.call(a, t, e, this))) {
              r = a;
              break;
            }
          }
          if (!r) return Jt(`[Assets] ${t} could not be loaded as we don't know how to parse it, ensure the correct parser has been added`), null;
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
        ...Yi.defaultOptions,
        ...this.loadOptions,
        onProgress: e
      } : {
        ...Yi.defaultOptions,
        ...this.loadOptions,
        ...e || {}
      }, { onProgress: i, onError: r, strategy: o, retryCount: a, retryDelay: l } = s;
      let c = 0;
      const h = {}, d = Ki(t), u = He(t, (m) => ({
        alias: [
          m
        ],
        src: m,
        data: {}
      })), p = u.reduce((m, g) => m + (g.progressSize || 1), 0), f = u.map(async (m) => {
        const g = Ae.toAbsolute(m.src);
        h[m.src] || (await this._loadAssetWithRetry(g, m, {
          onProgress: i,
          onError: r,
          strategy: o,
          retryCount: a,
          retryDelay: l
        }, h), c += m.progressSize || 1, i && i(c / p));
      });
      return await Promise.all(f), d ? h[u[0].src] : h;
    }
    async unload(t) {
      const s = He(t, (i) => ({
        alias: [
          i
        ],
        src: i
      })).map(async (i) => {
        var _a2, _b2;
        const r = Ae.toAbsolute(i.src), o = this.promiseCache[r];
        if (o) {
          const a = await o.promise;
          delete this.promiseCache[r], await ((_b2 = (_a2 = o.parser) == null ? void 0 : _a2.unload) == null ? void 0 : _b2.call(_a2, a, i, this));
        }
      });
      await Promise.all(s);
    }
    _validateParsers() {
      this._parsersValidated = true, this._parserHash = this._parsers.filter((t) => t.name || t.id).reduce((t, e) => (!e.name && !e.id ? Jt("[Assets] parser should have an id") : (t[e.name] || t[e.id]) && Jt(`[Assets] parser id conflict "${e.id}"`), t[e.name] = e, e.id && (t[e.id] = e), t), {});
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
  jh.defaultOptions = {
    onProgress: void 0,
    onError: void 0,
    strategy: "throw",
    retryCount: 3,
    retryDelay: 250
  };
  let Bf = jh;
  function ks(n, t) {
    if (Array.isArray(t)) {
      for (const e of t) if (n.startsWith(`data:${e}`)) return true;
      return false;
    }
    return n.startsWith(`data:${t}`);
  }
  function Ms(n, t) {
    const e = n.split("?")[0], s = Ae.extname(e).toLowerCase();
    return Array.isArray(t) ? t.includes(s) : s === t;
  }
  const Of = ".json", Nf = "application/json", Ff = {
    extension: {
      type: ht.LoadParser,
      priority: Ln.Low
    },
    name: "loadJson",
    id: "json",
    test(n) {
      return ks(n, Nf) || Ms(n, Of);
    },
    async load(n) {
      return await (await Pt.get().fetch(n)).json();
    }
  }, Wf = ".txt", Gf = "text/plain", zf = {
    name: "loadTxt",
    id: "text",
    extension: {
      type: ht.LoadParser,
      priority: Ln.Low,
      name: "loadTxt"
    },
    test(n) {
      return ks(n, Gf) || Ms(n, Wf);
    },
    async load(n) {
      return await (await Pt.get().fetch(n)).text();
    }
  }, Df = [
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
  ], Hf = [
    ".ttf",
    ".otf",
    ".woff",
    ".woff2"
  ], Uf = [
    "font/ttf",
    "font/otf",
    "font/woff",
    "font/woff2"
  ], jf = /^(--|-?[A-Z_])[0-9A-Z_-]*$/i;
  function Vf(n) {
    const t = Ae.extname(n), i = Ae.basename(n, t).replace(/(-|_)/g, " ").toLowerCase().split(" ").map((a) => a.charAt(0).toUpperCase() + a.slice(1));
    let r = i.length > 0;
    for (const a of i) if (!a.match(jf)) {
      r = false;
      break;
    }
    let o = i.join(" ");
    return r || (o = `"${o.replace(/[\\"]/g, "\\$&")}"`), o;
  }
  const Yf = /^[0-9A-Za-z%:/?#\[\]@!\$&'()\*\+,;=\-._~]*$/;
  function Xf(n) {
    return Yf.test(n) ? n : encodeURI(n);
  }
  const qf = {
    extension: {
      type: ht.LoadParser,
      priority: Ln.Low
    },
    name: "loadWebFont",
    id: "web-font",
    test(n) {
      return ks(n, Uf) || Ms(n, Hf);
    },
    async load(n, t) {
      var _a2, _b2, _c2;
      const e = Pt.get().getFontFaceSet();
      if (e) {
        const s = [], i = ((_a2 = t.data) == null ? void 0 : _a2.family) ?? Vf(n), r = ((_c2 = (_b2 = t.data) == null ? void 0 : _b2.weights) == null ? void 0 : _c2.filter((a) => Df.includes(a))) ?? [
          "normal"
        ], o = t.data ?? {};
        for (let a = 0; a < r.length; a++) {
          const l = r[a], c = new FontFace(i, `url('${Xf(n)}')`, {
            ...o,
            weight: l
          });
          await c.load(), e.add(c), s.push(c);
        }
        return ne.has(`${i}-and-url`) ? ne.get(`${i}-and-url`).entries.push({
          url: n,
          faces: s
        }) : ne.set(`${i}-and-url`, {
          entries: [
            {
              url: n,
              faces: s
            }
          ]
        }), s.length === 1 ? s[0] : s;
      }
      return Jt("[loadWebFont] FontFace API is not supported. Skipping loading font"), null;
    },
    unload(n) {
      const t = Array.isArray(n) ? n : [
        n
      ], e = t[0].family, s = ne.get(`${e}-and-url`), i = s.entries.find((r) => r.faces.some((o) => t.indexOf(o) !== -1));
      i.faces = i.faces.filter((r) => t.indexOf(r) === -1), i.faces.length === 0 && (s.entries = s.entries.filter((r) => r !== i)), t.forEach((r) => {
        Pt.get().getFontFaceSet().delete(r);
      }), s.entries.length === 0 && ne.remove(`${e}-and-url`);
    }
  };
  var jr, hl;
  function Kf() {
    if (hl) return jr;
    hl = 1, jr = e;
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
    return jr;
  }
  var Jf = Kf();
  const Zf = Yc(Jf);
  function Qf(n, t) {
    const e = Zf(n), s = [];
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
          Jt(`Unknown SVG path command: ${c}`);
      }
      c !== "Z" && c !== "z" && i === null && (i = {
        startX: r,
        startY: o
      }, s.push(i));
    }
    return t;
  }
  class na {
    constructor(t = 0, e = 0, s = 0) {
      this.type = "circle", this.x = t, this.y = e, this.radius = s;
    }
    clone() {
      return new na(this.x, this.y, this.radius);
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
      return t || (t = new Ht()), t.x = this.x - this.radius, t.y = this.y - this.radius, t.width = this.radius * 2, t.height = this.radius * 2, t;
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
  class sa {
    constructor(t = 0, e = 0, s = 0, i = 0) {
      this.type = "ellipse", this.x = t, this.y = e, this.halfWidth = s, this.halfHeight = i;
    }
    clone() {
      return new sa(this.x, this.y, this.halfWidth, this.halfHeight);
    }
    contains(t, e) {
      if (this.halfWidth <= 0 || this.halfHeight <= 0) return false;
      let s = (t - this.x) / this.halfWidth, i = (e - this.y) / this.halfHeight;
      return s *= s, i *= i, s + i <= 1;
    }
    strokeContains(t, e, s, i = 0.5) {
      const { halfWidth: r, halfHeight: o } = this;
      if (r <= 0 || o <= 0) return false;
      const a = s * (1 - i), l = s - a, c = r - l, h = o - l, d = r + a, u = o + a, p = t - this.x, f = e - this.y, m = p * p / (c * c) + f * f / (h * h), g = p * p / (d * d) + f * f / (u * u);
      return m > 1 && g <= 1;
    }
    getBounds(t) {
      return t || (t = new Ht()), t.x = this.x - this.halfWidth, t.y = this.y - this.halfHeight, t.width = this.halfWidth * 2, t.height = this.halfHeight * 2, t;
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
  function tm(n, t, e, s, i, r) {
    const o = n - e, a = t - s, l = i - e, c = r - s, h = o * l + a * c, d = l * l + c * c;
    let u = -1;
    d !== 0 && (u = h / d);
    let p, f;
    u < 0 ? (p = e, f = s) : u > 1 ? (p = i, f = r) : (p = e + u * l, f = s + u * c);
    const m = n - p, g = t - f;
    return m * m + g * g;
  }
  let em, nm;
  class Js {
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
      const e = this.getBounds(em), s = t.getBounds(nm);
      if (!e.containsRect(s)) return false;
      const i = t.points;
      for (let r = 0; r < i.length; r += 2) {
        const o = i[r], a = i[r + 1];
        if (!this.contains(o, a)) return false;
      }
      return true;
    }
    clone() {
      const t = this.points.slice(), e = new Js(t);
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
        const d = l[h], u = l[h + 1], p = l[(h + 2) % l.length], f = l[(h + 3) % l.length], m = tm(t, e, d, u, p, f), g = Math.sign((p - d) * (e - u) - (f - u) * (t - d));
        if (m <= (g < 0 ? a : o)) return true;
      }
      return false;
    }
    getBounds(t) {
      t || (t = new Ht());
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
      return Mt("8.11.0", "Polygon.lastX is deprecated, please use Polygon.lastX instead."), this.points[this.points.length - 2];
    }
    get y() {
      return Mt("8.11.0", "Polygon.y is deprecated, please use Polygon.lastY instead."), this.points[this.points.length - 1];
    }
    get startX() {
      return this.points[0];
    }
    get startY() {
      return this.points[1];
    }
  }
  const Mi = (n, t, e, s, i, r, o) => {
    const a = n - e, l = t - s, c = Math.sqrt(a * a + l * l);
    return c >= i - r && c <= i + o;
  };
  class ia {
    constructor(t = 0, e = 0, s = 0, i = 0, r = 20) {
      this.type = "roundedRectangle", this.x = t, this.y = e, this.width = s, this.height = i, this.radius = r;
    }
    getBounds(t) {
      return t || (t = new Ht()), t.x = this.x, t.y = this.y, t.width = this.width, t.height = this.height, t;
    }
    clone() {
      return new ia(this.x, this.y, this.width, this.height, this.radius);
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
      const { x: r, y: o, width: a, height: l, radius: c } = this, h = s * (1 - i), d = s - h, u = r + c, p = o + c, f = a - c * 2, m = l - c * 2, g = r + a, y = o + l;
      return (t >= r - h && t <= r + d || t >= g - d && t <= g + h) && e >= p && e <= p + m || (e >= o - h && e <= o + d || e >= y - d && e <= y + h) && t >= u && t <= u + f ? true : t < u && e < p && Mi(t, e, u, p, c, d, h) || t > g - c && e < p && Mi(t, e, g - c, p, c, d, h) || t > g - c && e > y - c && Mi(t, e, g - c, y - c, c, d, h) || t < u && e > y - c && Mi(t, e, u, y - c, c, d, h);
    }
    toString() {
      return `[pixi.js/math:RoundedRectangle x=${this.x} y=${this.y}width=${this.width} height=${this.height} radius=${this.radius}]`;
    }
  }
  const Vh = {};
  sm = function(n, t, e) {
    let s = 2166136261;
    for (let i = 0; i < t; i++) s ^= n[i].uid, s = Math.imul(s, 16777619), s >>>= 0;
    return Vh[s] || im(n, t, s, e);
  };
  function im(n, t, e, s) {
    const i = {};
    let r = 0;
    for (let a = 0; a < s; a++) {
      const l = a < t ? n[a] : Ct.EMPTY.source;
      i[r++] = l.source, i[r++] = l.style;
    }
    const o = new Vi(i);
    return Vh[e] = o, o;
  }
  class fs {
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
  dl = function(n, t, e, s) {
    if (e ?? (e = 0), s ?? (s = Math.min(n.byteLength - e, t.byteLength)), !(e & 7) && !(s & 7)) {
      const i = s / 8;
      new Float64Array(t, 0, i).set(new Float64Array(n, e, i));
    } else if (!(e & 3) && !(s & 3)) {
      const i = s / 4;
      new Float32Array(t, 0, i).set(new Float32Array(n, e, i));
    } else new Uint8Array(t).set(new Uint8Array(n, e, s));
  };
  const rm = {
    normal: "normal-npm",
    add: "add-npm",
    screen: "screen-npm"
  };
  om = ((n) => (n[n.DISABLED = 0] = "DISABLED", n[n.RENDERING_MASK_ADD = 1] = "RENDERING_MASK_ADD", n[n.MASK_ACTIVE = 2] = "MASK_ACTIVE", n[n.INVERSE_MASK_ACTIVE = 3] = "INVERSE_MASK_ACTIVE", n[n.RENDERING_MASK_REMOVE = 4] = "RENDERING_MASK_REMOVE", n[n.NONE = 5] = "NONE", n))(om || {});
  function ko(n, t) {
    return t.alphaMode === "no-premultiply-alpha" && rm[n] || n;
  }
  const am = [
    "precision mediump float;",
    "void main(void){",
    "float test = 0.1;",
    "%forloop%",
    "gl_FragColor = vec4(0.0);",
    "}"
  ].join(`
`);
  function lm(n) {
    let t = "";
    for (let e = 0; e < n; ++e) e > 0 && (t += `
else `), e < n - 1 && (t += `if(test == ${e}.0){}`);
    return t;
  }
  cm = function(n, t) {
    if (n === 0) throw new Error("Invalid value of `0` passed to `checkMaxIfStatementsInShader`");
    const e = t.createShader(t.FRAGMENT_SHADER);
    try {
      for (; ; ) {
        const s = am.replace(/%forloop%/gi, lm(n));
        if (t.shaderSource(e, s), t.compileShader(e), !t.getShaderParameter(e, t.COMPILE_STATUS)) n = n / 2 | 0;
        else break;
      }
    } finally {
      t.deleteShader(e);
    }
    return n;
  };
  let rs = null;
  function hm() {
    var _a2;
    if (rs) return rs;
    const n = Th();
    return rs = n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS), rs = cm(rs, n), (_a2 = n.getExtension("WEBGL_lose_context")) == null ? void 0 : _a2.loseContext(), rs;
  }
  class dm {
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
  class um {
    constructor() {
      this.renderPipeId = "batch", this.action = "startBatch", this.start = 0, this.size = 0, this.textures = new dm(), this.blendMode = "normal", this.topology = "triangle-strip", this.canBundle = true;
    }
    destroy() {
      this.textures = null, this.gpuBindGroup = null, this.bindGroup = null, this.batcher = null, this.elements = null;
    }
  }
  const Zs = [];
  let Qi = 0;
  pi.register({
    clear: () => {
      if (Zs.length > 0) for (const n of Zs) n && n.destroy();
      Zs.length = 0, Qi = 0;
    }
  });
  function ul() {
    return Qi > 0 ? Zs[--Qi] : new um();
  }
  function pl(n) {
    n.elements = null, Zs[Qi++] = n;
  }
  let Fs = 0;
  const Yh = class Xh {
    constructor(t) {
      this.uid = ee("batcher"), this.dirty = true, this.batchIndex = 0, this.batches = [], this._elements = [], t = {
        ...Xh.defaultOptions,
        ...t
      }, t.maxTextures || (Mt("v8.8.0", "maxTextures is a required option for Batcher now, please pass it in the options"), t.maxTextures = hm());
      const { maxTextures: e, attributesInitialSize: s, indicesInitialSize: i } = t;
      this.attributeBuffer = new fs(s * 4), this.indexBuffer = new Uint16Array(i), this.maxTextures = e;
    }
    begin() {
      this.elementSize = 0, this.elementStart = 0, this.indexSize = 0, this.attributeSize = 0;
      for (let t = 0; t < this.batchIndex; t++) pl(this.batches[t]);
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
      let s = ul(), i = s.textures;
      i.clear();
      const r = e[this.elementStart];
      let o = ko(r.blendMode, r.texture._source), a = r.topology;
      this.attributeSize * 4 > this.attributeBuffer.size && this._resizeAttributeBuffer(this.attributeSize * 4), this.indexSize > this.indexBuffer.length && this._resizeIndexBuffer(this.indexSize);
      const l = this.attributeBuffer.float32View, c = this.attributeBuffer.uint32View, h = this.indexBuffer;
      let d = this._batchIndexSize, u = this._batchIndexStart, p = "startBatch", f = [];
      const m = this.maxTextures;
      for (let g = this.elementStart; g < this.elementSize; ++g) {
        const y = e[g];
        e[g] = null;
        const x = y.texture._source, _ = ko(y.blendMode, x), v = o !== _ || a !== y.topology;
        if (x._batchTick === Fs && !v) {
          y._textureId = x._textureBindLocation, d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize)), y._batch = s, f.push(y);
          continue;
        }
        x._batchTick = Fs, (i.count >= m || v) && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), p = "renderBatch", u = d, o = _, a = y.topology, s = ul(), i = s.textures, i.clear(), f = [], ++Fs), y._textureId = x._textureBindLocation = i.count, i.ids[x.uid] = i.count, i.textures[i.count++] = x, y._batch = s, f.push(y), d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize));
      }
      i.count > 0 && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), u = d, ++Fs), this.elementStart = this.elementSize, this._batchIndexStart = u, this._batchIndexSize = d;
    }
    _finishBatch(t, e, s, i, r, o, a, l, c) {
      t.gpuBindGroup = null, t.bindGroup = null, t.action = l, t.batcher = this, t.textures = i, t.blendMode = r, t.topology = o, t.start = e, t.size = s, t.elements = c, ++Fs, this.batches[this.batchIndex++] = t, a.add(t);
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
      const e = Math.max(t, this.attributeBuffer.size * 2), s = new fs(e);
      dl(this.attributeBuffer.rawBinaryData, s.rawBinaryData), this.attributeBuffer = s;
    }
    _resizeIndexBuffer(t) {
      const e = this.indexBuffer;
      let s = Math.max(t, e.length * 1.5);
      s += s % 2;
      const i = s > 65535 ? new Uint32Array(s) : new Uint16Array(s);
      if (i.BYTES_PER_ELEMENT !== e.BYTES_PER_ELEMENT) for (let r = 0; r < e.length; r++) i[r] = e[r];
      else dl(e.buffer, i.buffer);
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
        for (let e = 0; e < this.batchIndex; e++) pl(this.batches[e]);
        this.batches = null, this.geometry.destroy(true), this.geometry = null, t.shader && ((_a2 = this.shader) == null ? void 0 : _a2.destroy(), this.shader = null);
        for (let e = 0; e < this._elements.length; e++) this._elements[e] && (this._elements[e]._batch = null);
        this._elements = null, this.indexBuffer = null, this.attributeBuffer.destroy(), this.attributeBuffer = null;
      }
    }
  };
  Yh.defaultOptions = {
    maxTextures: null,
    attributesInitialSize: 4,
    indicesInitialSize: 6
  };
  let pm = Yh;
  le = ((n) => (n[n.MAP_READ = 1] = "MAP_READ", n[n.MAP_WRITE = 2] = "MAP_WRITE", n[n.COPY_SRC = 4] = "COPY_SRC", n[n.COPY_DST = 8] = "COPY_DST", n[n.INDEX = 16] = "INDEX", n[n.VERTEX = 32] = "VERTEX", n[n.UNIFORM = 64] = "UNIFORM", n[n.STORAGE = 128] = "STORAGE", n[n.INDIRECT = 256] = "INDIRECT", n[n.QUERY_RESOLVE = 512] = "QUERY_RESOLVE", n[n.STATIC = 1024] = "STATIC", n))(le || {});
  Zn = class extends rn {
    constructor(t) {
      let { data: e, size: s } = t;
      const { usage: i, label: r, shrinkToFit: o } = t;
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.autoGarbageCollect = true, this.uid = ee("buffer"), this._resourceType = "buffer", this._resourceId = ee("resource"), this._touched = 0, this._updateID = 1, this._dataInt32 = null, this.shrinkToFit = true, this.destroyed = false, e instanceof Array && (e = new Float32Array(e)), this._data = e, s ?? (s = e == null ? void 0 : e.byteLength);
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
      return !!(this.descriptor.usage & le.STATIC);
    }
    set static(t) {
      t ? this.descriptor.usage |= le.STATIC : this.descriptor.usage &= ~le.STATIC;
    }
    setDataWithSize(t, e, s) {
      if (this._updateID++, this._updateSize = e * t.BYTES_PER_ELEMENT, this._data === t) {
        s && this.emit("update", this);
        return;
      }
      const i = this._data;
      if (this._data = t, this._dataInt32 = null, !i || i.length !== t.length) {
        !this.shrinkToFit && i && t.byteLength < i.byteLength ? s && this.emit("update", this) : (this.descriptor.size = t.byteLength, this._resourceId = ee("resource"), this.emit("change", this));
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
  function qh(n, t) {
    if (!(n instanceof Zn)) {
      let e = t ? le.INDEX : le.VERTEX;
      n instanceof Array && (t ? (n = new Uint32Array(n), e = le.INDEX | le.COPY_DST) : (n = new Float32Array(n), e = le.VERTEX | le.COPY_DST)), n = new Zn({
        data: n,
        label: t ? "index-mesh-buffer" : "vertex-mesh-buffer",
        usage: e
      });
    }
    return n;
  }
  function fm(n, t, e) {
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
  function mm(n) {
    return (n instanceof Zn || Array.isArray(n) || n.BYTES_PER_ELEMENT) && (n = {
      buffer: n
    }), n.buffer = qh(n.buffer, false), n;
  }
  Kh = class extends rn {
    constructor(t = {}) {
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ee("geometry"), this._layoutKey = 0, this.instanceCount = 1, this._bounds = new ke(), this._boundsDirty = true;
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
      const s = mm(e);
      this.buffers.indexOf(s.buffer) === -1 && (this.buffers.push(s.buffer), s.buffer.on("update", this.onBufferUpdate, this), s.buffer.on("change", this.onBufferUpdate, this)), this.attributes[t] = s;
    }
    addIndex(t) {
      this.indexBuffer = qh(t, true), this.buffers.push(this.indexBuffer);
    }
    get bounds() {
      return this._boundsDirty ? (this._boundsDirty = false, fm(this, "aPosition", this._bounds)) : this._bounds;
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
  const gm = new Float32Array(1), ym = new Uint32Array(1);
  class xm extends Kh {
    constructor() {
      const e = new Zn({
        data: gm,
        label: "attribute-batch-buffer",
        usage: le.VERTEX | le.COPY_DST,
        shrinkToFit: false
      }), s = new Zn({
        data: ym,
        label: "index-batch-buffer",
        usage: le.INDEX | le.COPY_DST,
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
  function fl(n, t, e) {
    if (n) for (const s in n) {
      const i = s.toLocaleLowerCase(), r = t[i];
      if (r) {
        let o = n[s];
        s === "header" && (o = o.replace(/@in\s+[^;]+;\s*/g, "").replace(/@out\s+[^;]+;\s*/g, "")), e && r.push(`//----${e}----//`), r.push(o);
      } else Jt(`${s} placement hook does not exist in shader`);
    }
  }
  const bm = /\{\{(.*?)\}\}/g;
  function ml(n) {
    var _a2;
    const t = {};
    return (((_a2 = n.match(bm)) == null ? void 0 : _a2.map((s) => s.replace(/[{()}]/g, ""))) ?? []).forEach((s) => {
      t[s] = [];
    }), t;
  }
  function gl(n, t) {
    let e;
    const s = /@in\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function yl(n, t, e = false) {
    const s = [];
    gl(t, s), n.forEach((a) => {
      a.header && gl(a.header, s);
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
  function xl(n, t) {
    let e;
    const s = /@out\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function _m(n) {
    const e = /\b(\w+)\s*:/g.exec(n);
    return e ? e[1] : "";
  }
  function wm(n) {
    const t = /@.*?\s+/g;
    return n.replace(t, "");
  }
  function vm(n, t) {
    const e = [];
    xl(t, e), n.forEach((l) => {
      l.header && xl(l.header, e);
    });
    let s = 0;
    const i = e.sort().map((l) => l.indexOf("builtin") > -1 ? l : `@location(${s++}) ${l}`).join(`,
`), r = e.sort().map((l) => `       var ${wm(l)};`).join(`
`), o = `return VSOutput(
            ${e.sort().map((l) => ` ${_m(l)}`).join(`,
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
  function bl(n, t) {
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
  const An = /* @__PURE__ */ Object.create(null), Vr = /* @__PURE__ */ new Map();
  let Cm = 0;
  function Sm({ template: n, bits: t }) {
    const e = Jh(n, t);
    if (An[e]) return An[e];
    const { vertex: s, fragment: i } = Em(n, t);
    return An[e] = Zh(s, i, t), An[e];
  }
  function Tm({ template: n, bits: t }) {
    const e = Jh(n, t);
    return An[e] || (An[e] = Zh(n.vertex, n.fragment, t)), An[e];
  }
  function Em(n, t) {
    const e = t.map((o) => o.vertex).filter((o) => !!o), s = t.map((o) => o.fragment).filter((o) => !!o);
    let i = yl(e, n.vertex, true);
    i = vm(e, i);
    const r = yl(s, n.fragment, true);
    return {
      vertex: i,
      fragment: r
    };
  }
  function Jh(n, t) {
    return t.map((e) => (Vr.has(e) || Vr.set(e, Cm++), Vr.get(e))).sort((e, s) => e - s).join("-") + n.vertex + n.fragment;
  }
  function Zh(n, t, e) {
    const s = ml(n), i = ml(t);
    return e.forEach((r) => {
      fl(r.vertex, s, r.name), fl(r.fragment, i, r.name);
    }), {
      vertex: bl(n, s),
      fragment: bl(t, i)
    };
  }
  const Am = `
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
`, km = `
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
`, Mm = `
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
`, Pm = `

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
`, Im = {
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
  }, Rm = {
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
  Lm = function({ bits: n, name: t }) {
    const e = Sm({
      template: {
        fragment: km,
        vertex: Am
      },
      bits: [
        Im,
        ...n
      ]
    });
    return fi.from({
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
  $m = function({ bits: n, name: t }) {
    return new Qo({
      name: t,
      ...Tm({
        template: {
          vertex: Mm,
          fragment: Pm
        },
        bits: [
          Rm,
          ...n
        ]
      })
    });
  };
  let Yr;
  Bm = {
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
  Om = {
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
  Yr = {};
  function Nm(n) {
    const t = [];
    if (n === 1) t.push("@group(1) @binding(0) var textureSource1: texture_2d<f32>;"), t.push("@group(1) @binding(1) var textureSampler1: sampler;");
    else {
      let e = 0;
      for (let s = 0; s < n; s++) t.push(`@group(1) @binding(${e++}) var textureSource${s + 1}: texture_2d<f32>;`), t.push(`@group(1) @binding(${e++}) var textureSampler${s + 1}: sampler;`);
    }
    return t.join(`
`);
  }
  function Fm(n) {
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
  Wm = function(n) {
    return Yr[n] || (Yr[n] = {
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

                ${Nm(n)}
            `,
        main: `
                var uvDx = dpdx(vUV);
                var uvDy = dpdy(vUV);

                ${Fm(n)}
            `
      }
    }), Yr[n];
  };
  const Xr = {};
  function Gm(n) {
    const t = [];
    for (let e = 0; e < n; e++) e > 0 && t.push("else"), e < n - 1 && t.push(`if(vTextureId < ${e}.5)`), t.push("{"), t.push(`	outColor = texture(uTextures[${e}], vUV);`), t.push("}");
    return t.join(`
`);
  }
  zm = function(n) {
    return Xr[n] || (Xr[n] = {
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

                ${Gm(n)}
            `
      }
    }), Xr[n];
  };
  let _l;
  Dm = {
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
  Hm = {
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
  _l = {};
  Um = function(n) {
    let t = _l[n];
    if (t) return t;
    const e = new Int32Array(n);
    for (let s = 0; s < n; s++) e[s] = s;
    return t = _l[n] = new ta({
      uTextures: {
        value: e,
        type: "i32",
        size: n
      }
    }, {
      isStatic: true
    }), t;
  };
  class wl extends ur {
    constructor(t) {
      const e = $m({
        name: "batch",
        bits: [
          Om,
          zm(t),
          Hm
        ]
      }), s = Lm({
        name: "batch",
        bits: [
          Bm,
          Wm(t),
          Dm
        ]
      });
      super({
        glProgram: e,
        gpuProgram: s,
        resources: {
          batchSamplers: Um(t)
        }
      }), this.maxTextures = t;
    }
  }
  let Ws = null;
  const Qh = class td extends pm {
    constructor(t) {
      super(t), this.geometry = new xm(), this.name = td.extension.name, this.vertexSize = 6, Ws ?? (Ws = new wl(t.maxTextures)), this.shader = Ws;
    }
    packAttributes(t, e, s, i, r) {
      const o = r << 16 | t.roundPixels & 65535, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, { positions: f, uvs: m } = t, g = t.color, y = t.attributeOffset, w = y + t.attributeSize;
      for (let x = y; x < w; x++) {
        const _ = x * 2, v = f[_], b = f[_ + 1];
        e[i++] = l * v + h * b + u, e[i++] = d * b + c * v + p, e[i++] = m[_], e[i++] = m[_ + 1], s[i++] = g, s[i++] = o;
      }
    }
    packQuadAttributes(t, e, s, i, r) {
      const o = t.texture, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, f = t.bounds, m = f.maxX, g = f.minX, y = f.maxY, w = f.minY, x = o.uvs, _ = t.color, v = r << 16 | t.roundPixels & 65535;
      e[i + 0] = l * g + h * w + u, e[i + 1] = d * w + c * g + p, e[i + 2] = x.x0, e[i + 3] = x.y0, s[i + 4] = _, s[i + 5] = v, e[i + 6] = l * m + h * w + u, e[i + 7] = d * w + c * m + p, e[i + 8] = x.x1, e[i + 9] = x.y1, s[i + 10] = _, s[i + 11] = v, e[i + 12] = l * m + h * y + u, e[i + 13] = d * y + c * m + p, e[i + 14] = x.x2, e[i + 15] = x.y2, s[i + 16] = _, s[i + 17] = v, e[i + 18] = l * g + h * y + u, e[i + 19] = d * y + c * g + p, e[i + 20] = x.x3, e[i + 21] = x.y3, s[i + 22] = _, s[i + 23] = v;
    }
    _updateMaxTextures(t) {
      this.shader.maxTextures !== t && (Ws = new wl(t), this.shader = Ws);
    }
    destroy() {
      this.shader = null, super.destroy();
    }
  };
  Qh.extension = {
    type: [
      ht.Batcher
    ],
    name: "default"
  };
  jm = Qh;
  Ps = class {
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
  function Vm(n, t, e, s, i, r, o, a = null) {
    let l = 0;
    e *= t, i *= r;
    const c = a.a, h = a.b, d = a.c, u = a.d, p = a.tx, f = a.ty;
    for (; l < o; ) {
      const m = n[e], g = n[e + 1];
      s[i] = c * m + d * g + p, s[i + 1] = h * m + u * g + f, i += r, e += t, l++;
    }
  }
  function Ym(n, t, e, s) {
    let i = 0;
    for (t *= e; i < s; ) n[t] = 0, n[t + 1] = 0, t += e, i++;
  }
  function ed(n, t, e, s, i) {
    const r = t.a, o = t.b, a = t.c, l = t.d, c = t.tx, h = t.ty;
    e || (e = 0), s || (s = 2), i || (i = n.length / s - e);
    let d = e * s;
    for (let u = 0; u < i; u++) {
      const p = n[d], f = n[d + 1];
      n[d] = r * p + a * f + c, n[d + 1] = o * p + l * f + h, d += s;
    }
  }
  const Xm = new vt();
  class ra {
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
      return s ? ah(e, s.groupColor) + (this.alpha * s.groupAlpha * 255 << 24) : e + (this.alpha * 255 << 24);
    }
    get transform() {
      var _a2;
      return ((_a2 = this.renderable) == null ? void 0 : _a2.groupTransform) || Xm;
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
  const oi = {
    extension: {
      type: ht.ShapeBuilder,
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
        const _ = n, v = _.width / 2, b = _.height / 2;
        e = _.x + v, s = _.y + b, o = a = Math.max(0, Math.min(_.radius, Math.min(v, b))), i = v - o, r = b - a;
      }
      if (i < 0 || r < 0) return false;
      const l = Math.ceil(2.3 * Math.sqrt(o + a)), c = l * 8 + (i ? 4 : 0) + (r ? 4 : 0);
      if (c === 0) return false;
      if (l === 0) return t[0] = t[6] = e + i, t[1] = t[3] = s + r, t[2] = t[4] = e - i, t[5] = t[7] = s - r, true;
      let h = 0, d = l * 4 + (i ? 2 : 0) + 2, u = d, p = c, f = i + o, m = r, g = e + f, y = e - f, w = s + m;
      if (t[h++] = g, t[h++] = w, t[--d] = w, t[--d] = y, r) {
        const _ = s - m;
        t[u++] = y, t[u++] = _, t[--p] = _, t[--p] = g;
      }
      for (let _ = 1; _ < l; _++) {
        const v = Math.PI / 2 * (_ / l), b = i + Math.cos(v) * o, C = r + Math.sin(v) * a, T = e + b, L = e - b, A = s + C, E = s - C;
        t[h++] = T, t[h++] = A, t[--d] = A, t[--d] = L, t[u++] = L, t[u++] = E, t[--p] = E, t[--p] = T;
      }
      f = i, m = r + a, g = e + f, y = e - f, w = s + m;
      const x = s - m;
      return t[h++] = g, t[h++] = w, t[--p] = x, t[--p] = g, i && (t[h++] = y, t[h++] = w, t[--p] = x, t[--p] = y), true;
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
  }, qm = {
    ...oi,
    extension: {
      ...oi.extension,
      name: "ellipse"
    }
  }, Km = {
    ...oi,
    extension: {
      ...oi.extension,
      name: "roundedRectangle"
    }
  }, nd = 1e-4, vl = 1e-4;
  function Jm(n) {
    const t = n.length;
    if (t < 6) return 1;
    let e = 0;
    for (let s = 0, i = n[t - 2], r = n[t - 1]; s < t; s += 2) {
      const o = n[s], a = n[s + 1];
      e += (o - i) * (a + r), i = o, r = a;
    }
    return e < 0 ? -1 : 1;
  }
  function Cl(n, t, e, s, i, r, o, a) {
    const l = n - e * i, c = t - s * i, h = n + e * r, d = t + s * r;
    let u, p;
    o ? (u = s, p = -e) : (u = -s, p = e);
    const f = l + u, m = c + p, g = h + u, y = d + p;
    return a.push(f, m), a.push(g, y), 2;
  }
  function On(n, t, e, s, i, r, o, a) {
    const l = e - n, c = s - t;
    let h = Math.atan2(l, c), d = Math.atan2(i - n, r - t);
    a && h < d ? h += Math.PI * 2 : !a && h > d && (d += Math.PI * 2);
    let u = h;
    const p = d - h, f = Math.abs(p), m = Math.sqrt(l * l + c * c), g = (15 * f * Math.sqrt(m) / Math.PI >> 0) + 1, y = p / g;
    if (u += y, a) {
      o.push(n, t), o.push(e, s);
      for (let w = 1, x = u; w < g; w++, x += y) o.push(n, t), o.push(n + Math.sin(x) * m, t + Math.cos(x) * m);
      o.push(n, t), o.push(i, r);
    } else {
      o.push(e, s), o.push(n, t);
      for (let w = 1, x = u; w < g; w++, x += y) o.push(n + Math.sin(x) * m, t + Math.cos(x) * m), o.push(n, t);
      o.push(i, r), o.push(n, t);
    }
    return g * 2;
  }
  Zm = function(n, t, e, s, i, r) {
    const o = nd;
    if (n.length === 0) return;
    const a = t;
    let l = a.alignment;
    if (t.alignment !== 0.5) {
      let X = Jm(n);
      l = (l - 0.5) * X + 0.5;
    }
    const c = new Tt(n[0], n[1]), h = new Tt(n[n.length - 2], n[n.length - 1]), d = s, u = Math.abs(c.x - h.x) < o && Math.abs(c.y - h.y) < o;
    if (d) {
      n = n.slice(), u && (n.pop(), n.pop(), h.set(n[n.length - 2], n[n.length - 1]));
      const X = (c.x + h.x) * 0.5, Q = (h.y + c.y) * 0.5;
      n.unshift(X, Q), n.push(X, Q);
    }
    const p = i, f = n.length / 2;
    let m = n.length;
    const g = p.length / 2, y = a.width / 2, w = y * y, x = a.miterLimit * a.miterLimit;
    let _ = n[0], v = n[1], b = n[2], C = n[3], T = 0, L = 0, A = -(v - C), E = _ - b, I = 0, V = 0, G = Math.sqrt(A * A + E * E);
    A /= G, E /= G, A *= y, E *= y;
    const F = l, $ = (1 - F) * 2, z = F * 2;
    d || (a.cap === "round" ? m += On(_ - A * ($ - z) * 0.5, v - E * ($ - z) * 0.5, _ - A * $, v - E * $, _ + A * z, v + E * z, p, true) + 2 : a.cap === "square" && (m += Cl(_, v, A, E, $, z, true, p))), p.push(_ - A * $, v - E * $), p.push(_ + A * z, v + E * z);
    for (let X = 1; X < f - 1; ++X) {
      _ = n[(X - 1) * 2], v = n[(X - 1) * 2 + 1], b = n[X * 2], C = n[X * 2 + 1], T = n[(X + 1) * 2], L = n[(X + 1) * 2 + 1], A = -(v - C), E = _ - b, G = Math.sqrt(A * A + E * E), A /= G, E /= G, A *= y, E *= y, I = -(C - L), V = b - T, G = Math.sqrt(I * I + V * V), I /= G, V /= G, I *= y, V *= y;
      const Q = b - _, M = v - C, O = b - T, N = L - C, D = Q * O + M * N, K = M * O - N * Q, tt = K < 0;
      if (Math.abs(K) < 1e-3 * Math.abs(D)) {
        p.push(b - A * $, C - E * $), p.push(b + A * z, C + E * z), D >= 0 && (a.join === "round" ? m += On(b, C, b - A * $, C - E * $, b - I * $, C - V * $, p, false) + 4 : m += 2, p.push(b - I * z, C - V * z), p.push(b + I * $, C + V * $));
        continue;
      }
      const ot = (-A + _) * (-E + C) - (-A + b) * (-E + v), at = (-I + T) * (-V + C) - (-I + b) * (-V + L), mt = (Q * at - O * ot) / K, bt = (N * ot - M * at) / K, Y = (mt - b) * (mt - b) + (bt - C) * (bt - C), et = b + (mt - b) * $, lt = C + (bt - C) * $, ct = b - (mt - b) * z, _t = C - (bt - C) * z, Et = Math.min(Q * Q + M * M, O * O + N * N), jt = tt ? $ : z, U = Et + jt * jt * w;
      Y <= U ? a.join === "bevel" || Y / w > x ? (tt ? (p.push(et, lt), p.push(b + A * z, C + E * z), p.push(et, lt), p.push(b + I * z, C + V * z)) : (p.push(b - A * $, C - E * $), p.push(ct, _t), p.push(b - I * $, C - V * $), p.push(ct, _t)), m += 2) : a.join === "round" ? tt ? (p.push(et, lt), p.push(b + A * z, C + E * z), m += On(b, C, b + A * z, C + E * z, b + I * z, C + V * z, p, true) + 4, p.push(et, lt), p.push(b + I * z, C + V * z)) : (p.push(b - A * $, C - E * $), p.push(ct, _t), m += On(b, C, b - A * $, C - E * $, b - I * $, C - V * $, p, false) + 4, p.push(b - I * $, C - V * $), p.push(ct, _t)) : (p.push(et, lt), p.push(ct, _t)) : (p.push(b - A * $, C - E * $), p.push(b + A * z, C + E * z), a.join === "round" ? tt ? m += On(b, C, b + A * z, C + E * z, b + I * z, C + V * z, p, true) + 2 : m += On(b, C, b - A * $, C - E * $, b - I * $, C - V * $, p, false) + 2 : a.join === "miter" && Y / w <= x && (tt ? (p.push(ct, _t), p.push(ct, _t)) : (p.push(et, lt), p.push(et, lt)), m += 2), p.push(b - I * $, C - V * $), p.push(b + I * z, C + V * z), m += 2);
    }
    _ = n[(f - 2) * 2], v = n[(f - 2) * 2 + 1], b = n[(f - 1) * 2], C = n[(f - 1) * 2 + 1], A = -(v - C), E = _ - b, G = Math.sqrt(A * A + E * E), A /= G, E /= G, A *= y, E *= y, p.push(b - A * $, C - E * $), p.push(b + A * z, C + E * z), d || (a.cap === "round" ? m += On(b - A * ($ - z) * 0.5, C - E * ($ - z) * 0.5, b - A * $, C - E * $, b + A * z, C + E * z, p, false) + 2 : a.cap === "square" && (m += Cl(b, C, A, E, $, z, false, p)));
    const J = vl * vl;
    for (let X = g; X < m + g - 2; ++X) _ = p[X * 2], v = p[X * 2 + 1], b = p[(X + 1) * 2], C = p[(X + 1) * 2 + 1], T = p[(X + 2) * 2], L = p[(X + 2) * 2 + 1], !(Math.abs(_ * (C - L) + b * (L - v) + T * (v - C)) < J) && r.push(X, X + 1, X + 2);
  };
  function Qm(n, t, e, s) {
    const i = nd;
    if (n.length === 0) return;
    const r = n[0], o = n[1], a = n[n.length - 2], l = n[n.length - 1], c = t || Math.abs(r - a) < i && Math.abs(o - l) < i, h = e, d = n.length / 2, u = h.length / 2;
    for (let p = 0; p < d; p++) h.push(n[p * 2]), h.push(n[p * 2 + 1]);
    for (let p = 0; p < d - 1; p++) s.push(u + p, u + p + 1);
    c && s.push(u + d - 1, u);
  }
  function sd(n, t, e, s, i, r, o) {
    const a = gf(n, t, 2);
    if (!a) return;
    for (let c = 0; c < a.length; c += 3) r[o++] = a[c] + i, r[o++] = a[c + 1] + i, r[o++] = a[c + 2] + i;
    let l = i * s;
    for (let c = 0; c < n.length; c += 2) e[l] = n[c], e[l + 1] = n[c + 1], l += s;
  }
  const tg = [], eg = {
    extension: {
      type: ht.ShapeBuilder,
      name: "polygon"
    },
    build(n, t) {
      for (let e = 0; e < n.points.length; e++) t[e] = n.points[e];
      return true;
    },
    triangulate(n, t, e, s, i, r) {
      sd(n, tg, t, e, s, i, r);
    }
  }, ng = {
    extension: {
      type: ht.ShapeBuilder,
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
  }, sg = {
    extension: {
      type: ht.ShapeBuilder,
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
  }, Sl = [
    {
      offset: 0,
      color: "white"
    },
    {
      offset: 1,
      color: "black"
    }
  ], oa = class Mo {
    constructor(...t) {
      this.uid = ee("fillGradient"), this._tick = 0, this.type = "linear", this.colorStops = [];
      let e = ig(t);
      e = {
        ...e.type === "radial" ? Mo.defaultRadialOptions : Mo.defaultLinearOptions,
        ...Kc(e)
      }, this._textureSize = e.textureSize, this._wrapMode = e.wrapMode, e.type === "radial" ? (this.center = e.center, this.outerCenter = e.outerCenter ?? this.center, this.innerRadius = e.innerRadius, this.outerRadius = e.outerRadius, this.scale = e.scale, this.rotation = e.rotation) : (this.start = e.start, this.end = e.end), this.textureSpace = e.textureSpace, this.type = e.type, e.colorStops.forEach((i) => {
        this.addColorStop(i.offset, i.color);
      });
    }
    addColorStop(t, e) {
      return this.colorStops.push({
        offset: t,
        color: Xt.shared.setValue(e).toHexa()
      }), this;
    }
    buildLinearGradient() {
      if (this.texture) return;
      let { x: t, y: e } = this.start, { x: s, y: i } = this.end, r = s - t, o = i - e;
      const a = r < 0 || o < 0;
      if (this._wrapMode === "clamp-to-edge") {
        if (r < 0) {
          const g = t;
          t = s, s = g, r *= -1;
        }
        if (o < 0) {
          const g = e;
          e = i, i = g, o *= -1;
        }
      }
      const l = this.colorStops.length ? this.colorStops : Sl, c = this._textureSize, { canvas: h, context: d } = El(c, 1), u = a ? d.createLinearGradient(this._textureSize, 0, 0, 0) : d.createLinearGradient(0, 0, this._textureSize, 0);
      Tl(u, l), d.fillStyle = u, d.fillRect(0, 0, c, 1), this.texture = new Ct({
        source: new Cs({
          resource: h,
          addressMode: this._wrapMode
        })
      });
      const p = Math.sqrt(r * r + o * o), f = Math.atan2(o, r), m = new vt();
      m.scale(p / c, 1), m.rotate(f), m.translate(t, e), this.textureSpace === "local" && m.scale(c, c), this.transform = m;
    }
    buildGradient() {
      this.texture || this._tick++, this.type === "linear" ? this.buildLinearGradient() : this.buildRadialGradient();
    }
    buildRadialGradient() {
      if (this.texture) return;
      const t = this.colorStops.length ? this.colorStops : Sl, e = this._textureSize, { canvas: s, context: i } = El(e, e), { x: r, y: o } = this.center, { x: a, y: l } = this.outerCenter, c = this.innerRadius, h = this.outerRadius, d = a - h, u = l - h, p = e / (h * 2), f = (r - d) * p, m = (o - u) * p, g = i.createRadialGradient(f, m, c * p, (a - d) * p, (l - u) * p, h * p);
      Tl(g, t), i.fillStyle = t[t.length - 1].color, i.fillRect(0, 0, e, e), i.fillStyle = g, i.translate(f, m), i.rotate(this.rotation), i.scale(1, this.scale), i.translate(-f, -m), i.fillRect(0, 0, e, e), this.texture = new Ct({
        source: new Cs({
          resource: s,
          addressMode: this._wrapMode
        })
      });
      const y = new vt();
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
  oa.defaultLinearOptions = {
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
  oa.defaultRadialOptions = {
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
  yn = oa;
  function Tl(n, t) {
    for (let e = 0; e < t.length; e++) {
      const s = t[e];
      n.addColorStop(s.offset, s.color);
    }
  }
  function El(n, t) {
    const e = Pt.get().createCanvas(n, t), s = e.getContext("2d");
    return {
      canvas: e,
      context: s
    };
  }
  function ig(n) {
    let t = n[0] ?? {};
    return (typeof t == "number" || n[1]) && (Mt("8.5.2", "use options object instead"), t = {
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
      textureSize: n[5] ?? yn.defaultLinearOptions.textureSize
    }), t;
  }
  const rg = new vt(), og = new Ht();
  ag = function(n, t, e, s) {
    const i = t.matrix ? n.copyFrom(t.matrix).invert() : n.identity();
    if (t.textureSpace === "local") {
      const o = e.getBounds(og);
      t.width && o.pad(t.width);
      const { x: a, y: l } = o, c = 1 / o.width, h = 1 / o.height, d = -a * c, u = -l * h, p = i.a, f = i.b, m = i.c, g = i.d;
      i.a *= c, i.b *= c, i.c *= h, i.d *= h, i.tx = d * p + u * m + i.tx, i.ty = d * f + u * g + i.ty;
    } else i.translate(t.texture.frame.x, t.texture.frame.y), i.scale(1 / t.texture.source.width, 1 / t.texture.source.height);
    const r = t.texture.source.style;
    return !(t.fill instanceof yn) && r.addressMode === "clamp-to-edge" && (r.addressMode = "repeat", r.update()), s && i.append(rg.copyFrom(s).invert()), i;
  };
  fr = {};
  Gt.handleByMap(ht.ShapeBuilder, fr);
  Gt.add(ng, eg, sg, oi, qm, Km);
  const lg = new Ht(), cg = new vt();
  function hg(n, t) {
    const { geometryData: e, batches: s } = t;
    s.length = 0, e.indices.length = 0, e.vertices.length = 0, e.uvs.length = 0;
    for (let i = 0; i < n.instructions.length; i++) {
      const r = n.instructions[i];
      if (r.action === "texture") dg(r.data, s, e);
      else if (r.action === "fill" || r.action === "stroke") {
        const o = r.action === "stroke", a = r.data.path.shapePath, l = r.data.style, c = r.data.hole;
        o && c && Al(c.shapePath, l, true, s, e), c && (a.shapePrimitives[a.shapePrimitives.length - 1].holes = c.shapePath.shapePrimitives), Al(a, l, o, s, e);
      }
    }
  }
  function dg(n, t, e) {
    const s = [], i = fr.rectangle, r = lg;
    r.x = n.dx, r.y = n.dy, r.width = n.dw, r.height = n.dh;
    const o = n.transform;
    if (!i.build(r, s)) return;
    const { vertices: a, uvs: l, indices: c } = e, h = c.length, d = a.length / 2;
    o && ed(s, o), i.triangulate(s, a, 2, d, c, h);
    const u = n.image, p = u.uvs;
    l.push(p.x0, p.y0, p.x1, p.y1, p.x3, p.y3, p.x2, p.y2);
    const f = ve.get(ra);
    f.indexOffset = h, f.indexSize = c.length - h, f.attributeOffset = d, f.attributeSize = a.length / 2 - d, f.baseColor = n.style, f.alpha = n.alpha, f.texture = u, f.geometryData = e, t.push(f);
  }
  function Al(n, t, e, s, i) {
    const { vertices: r, uvs: o, indices: a } = i;
    n.shapePrimitives.forEach(({ shape: l, transform: c, holes: h }) => {
      const d = [], u = fr[l.type];
      if (!u.build(l, d)) return;
      const p = a.length, f = r.length / 2;
      let m = "triangle-list";
      if (c && ed(d, c), e) {
        const x = l.closePath ?? true, _ = t;
        _.pixelLine ? (Qm(d, x, r, a), m = "line-list") : Zm(d, _, false, x, r, a);
      } else if (h) {
        const x = [], _ = d.slice();
        ug(h).forEach((b) => {
          x.push(_.length / 2), _.push(...b);
        }), sd(_, x, r, 2, f, a, p);
      } else u.triangulate(d, r, 2, f, a, p);
      const g = o.length / 2, y = t.texture;
      if (y !== Ct.WHITE) {
        const x = ag(cg, t, l, c);
        Vm(r, 2, f, o, g, 2, r.length / 2 - f, x);
      } else Ym(o, g, 2, r.length / 2 - f);
      const w = ve.get(ra);
      w.indexOffset = p, w.indexSize = a.length - p, w.attributeOffset = f, w.attributeSize = r.length / 2 - f, w.baseColor = t.color, w.alpha = t.alpha, w.texture = y, w.geometryData = i, w.topology = m, s.push(w);
    });
  }
  function ug(n) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e].shape, i = [];
      fr[s.type].build(s, i) && t.push(i);
    }
    return t;
  }
  class pg {
    constructor() {
      this.batches = [], this.geometryData = {
        vertices: [],
        uvs: [],
        indices: []
      };
    }
    reset() {
      this.batches && this.batches.forEach((t) => {
        ve.return(t);
      }), this.graphicsData && ve.return(this.graphicsData), this.isBatchable = false, this.context = null, this.batches.length = 0, this.geometryData.indices.length = 0, this.geometryData.vertices.length = 0, this.geometryData.uvs.length = 0, this.graphicsData = null;
    }
    destroy() {
      this.reset(), this.batches = null, this.geometryData = null;
    }
  }
  class fg {
    constructor() {
      this.instructions = new Jo();
    }
    init(t) {
      const e = t.maxTextures;
      this.batcher ? this.batcher._updateMaxTextures(e) : this.batcher = new jm({
        maxTextures: e
      }), this.instructions.reset();
    }
    get geometry() {
      return Mt(zu, "GraphicsContextRenderData#geometry is deprecated, please use batcher.geometry instead."), this.batcher.geometry;
    }
    destroy() {
      this.batcher.destroy(), this.instructions.destroy(), this.batcher = null, this.instructions = null;
    }
  }
  const aa = class Po {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Ps({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      Po.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? Po.defaultOptions.bezierSmoothness;
    }
    getContextRenderData(t) {
      return t._gpuData[this._renderer.uid].graphicsData || this._initContextRenderData(t);
    }
    updateGpuContext(t) {
      const e = !!t._gpuData[this._renderer.uid], s = t._gpuData[this._renderer.uid] || this._initContext(t);
      if (t.dirty || !e) {
        e && s.reset(), hg(t, s);
        const i = t.batchMode;
        t.customShader || i === "no-batch" ? s.isBatchable = false : i === "auto" ? s.isBatchable = s.geometryData.vertices.length < 400 : s.isBatchable = true, t.dirty = false;
      }
      return s;
    }
    getGpuContext(t) {
      return t._gpuData[this._renderer.uid] || this._initContext(t);
    }
    _initContextRenderData(t) {
      const e = ve.get(fg, {
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
        u.bindGroup = sm(u.textures.textures, u.textures.count, this._renderer.limits.maxBatchableTextures);
      }
      return e;
    }
    _initContext(t) {
      const e = new pg();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  aa.extension = {
    type: [
      ht.WebGLSystem,
      ht.WebGPUSystem
    ],
    name: "graphicsContext"
  };
  aa.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let la = aa;
  const mg = 8, Pi = 11920929e-14, gg = 1;
  function id(n, t, e, s, i, r, o, a, l, c) {
    const d = Math.min(0.99, Math.max(0, c ?? la.defaultOptions.bezierSmoothness));
    let u = (gg - d) / 1;
    return u *= u, yg(t, e, s, i, r, o, a, l, n, u), n;
  }
  function yg(n, t, e, s, i, r, o, a, l, c) {
    Io(n, t, e, s, i, r, o, a, l, c, 0), l.push(o, a);
  }
  function Io(n, t, e, s, i, r, o, a, l, c, h) {
    if (h > mg) return;
    const d = (n + e) / 2, u = (t + s) / 2, p = (e + i) / 2, f = (s + r) / 2, m = (i + o) / 2, g = (r + a) / 2, y = (d + p) / 2, w = (u + f) / 2, x = (p + m) / 2, _ = (f + g) / 2, v = (y + x) / 2, b = (w + _) / 2;
    if (h > 0) {
      let C = o - n, T = a - t;
      const L = Math.abs((e - o) * T - (s - a) * C), A = Math.abs((i - o) * T - (r - a) * C);
      if (L > Pi && A > Pi) {
        if ((L + A) * (L + A) <= c * (C * C + T * T)) {
          l.push(v, b);
          return;
        }
      } else if (L > Pi) {
        if (L * L <= c * (C * C + T * T)) {
          l.push(v, b);
          return;
        }
      } else if (A > Pi) {
        if (A * A <= c * (C * C + T * T)) {
          l.push(v, b);
          return;
        }
      } else if (C = v - (n + o) / 2, T = b - (t + a) / 2, C * C + T * T <= c) {
        l.push(v, b);
        return;
      }
    }
    Io(n, t, d, u, y, w, v, b, l, c, h + 1), Io(v, b, x, _, m, g, o, a, l, c, h + 1);
  }
  const xg = 8, bg = 11920929e-14, _g = 1;
  function wg(n, t, e, s, i, r, o, a) {
    const c = Math.min(0.99, Math.max(0, a ?? la.defaultOptions.bezierSmoothness));
    let h = (_g - c) / 1;
    return h *= h, vg(t, e, s, i, r, o, n, h), n;
  }
  function vg(n, t, e, s, i, r, o, a) {
    Ro(o, n, t, e, s, i, r, a, 0), o.push(i, r);
  }
  function Ro(n, t, e, s, i, r, o, a, l) {
    if (l > xg) return;
    const c = (t + s) / 2, h = (e + i) / 2, d = (s + r) / 2, u = (i + o) / 2, p = (c + d) / 2, f = (h + u) / 2;
    let m = r - t, g = o - e;
    const y = Math.abs((s - r) * g - (i - o) * m);
    if (y > bg) {
      if (y * y <= a * (m * m + g * g)) {
        n.push(p, f);
        return;
      }
    } else if (m = p - (t + r) / 2, g = f - (e + o) / 2, m * m + g * g <= a) {
      n.push(p, f);
      return;
    }
    Ro(n, t, e, c, h, p, f, a, l + 1), Ro(n, p, f, d, u, r, o, a, l + 1);
  }
  function rd(n, t, e, s, i, r, o, a) {
    let l = Math.abs(i - r);
    (!o && i > r || o && r > i) && (l = 2 * Math.PI - l), a || (a = Math.max(6, Math.floor(6 * Math.pow(s, 1 / 3) * (l / Math.PI)))), a = Math.max(a, 3);
    let c = l / a, h = i;
    c *= o ? -1 : 1;
    for (let d = 0; d < a + 1; d++) {
      const u = Math.cos(h), p = Math.sin(h), f = t + u * s, m = e + p * s;
      n.push(f, m), h += c;
    }
  }
  function Cg(n, t, e, s, i, r) {
    const o = n[n.length - 2], l = n[n.length - 1] - e, c = o - t, h = i - e, d = s - t, u = Math.abs(l * d - c * h);
    if (u < 1e-8 || r === 0) {
      (n[n.length - 2] !== t || n[n.length - 1] !== e) && n.push(t, e);
      return;
    }
    const p = l * l + c * c, f = h * h + d * d, m = l * h + c * d, g = r * Math.sqrt(p) / u, y = r * Math.sqrt(f) / u, w = g * m / p, x = y * m / f, _ = g * d + y * c, v = g * h + y * l, b = c * (y + w), C = l * (y + w), T = d * (g + x), L = h * (g + x), A = Math.atan2(C - v, b - _), E = Math.atan2(L - v, T - _);
    rd(n, _ + t, v + e, r, A, E, c * h > d * l);
  }
  const Qs = Math.PI * 2, qr = {
    centerX: 0,
    centerY: 0,
    ang1: 0,
    ang2: 0
  }, Kr = ({ x: n, y: t }, e, s, i, r, o, a, l) => {
    n *= e, t *= s;
    const c = i * n - r * t, h = r * n + i * t;
    return l.x = c + o, l.y = h + a, l;
  };
  function Sg(n, t) {
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
  const kl = (n, t, e, s) => {
    const i = n * s - t * e < 0 ? -1 : 1;
    let r = n * e + t * s;
    return r > 1 && (r = 1), r < -1 && (r = -1), i * Math.acos(r);
  }, Tg = (n, t, e, s, i, r, o, a, l, c, h, d, u) => {
    const p = Math.pow(i, 2), f = Math.pow(r, 2), m = Math.pow(h, 2), g = Math.pow(d, 2);
    let y = p * f - p * g - f * m;
    y < 0 && (y = 0), y /= p * g + f * m, y = Math.sqrt(y) * (o === a ? -1 : 1);
    const w = y * i / r * d, x = y * -r / i * h, _ = c * w - l * x + (n + e) / 2, v = l * w + c * x + (t + s) / 2, b = (h - w) / i, C = (d - x) / r, T = (-h - w) / i, L = (-d - x) / r, A = kl(1, 0, b, C);
    let E = kl(b, C, T, L);
    a === 0 && E > 0 && (E -= Qs), a === 1 && E < 0 && (E += Qs), u.centerX = _, u.centerY = v, u.ang1 = A, u.ang2 = E;
  };
  function Eg(n, t, e, s, i, r, o, a = 0, l = 0, c = 0) {
    if (r === 0 || o === 0) return;
    const h = Math.sin(a * Qs / 360), d = Math.cos(a * Qs / 360), u = d * (t - s) / 2 + h * (e - i) / 2, p = -h * (t - s) / 2 + d * (e - i) / 2;
    if (u === 0 && p === 0) return;
    r = Math.abs(r), o = Math.abs(o);
    const f = Math.pow(u, 2) / Math.pow(r, 2) + Math.pow(p, 2) / Math.pow(o, 2);
    f > 1 && (r *= Math.sqrt(f), o *= Math.sqrt(f)), Tg(t, e, s, i, r, o, l, c, h, d, u, p, qr);
    let { ang1: m, ang2: g } = qr;
    const { centerX: y, centerY: w } = qr;
    let x = Math.abs(g) / (Qs / 4);
    Math.abs(1 - x) < 1e-7 && (x = 1);
    const _ = Math.max(Math.ceil(x), 1);
    g /= _;
    let v = n[n.length - 2], b = n[n.length - 1];
    const C = {
      x: 0,
      y: 0
    };
    for (let T = 0; T < _; T++) {
      const L = Sg(m, g), { x: A, y: E } = Kr(L[0], r, o, d, h, y, w, C), { x: I, y: V } = Kr(L[1], r, o, d, h, y, w, C), { x: G, y: F } = Kr(L[2], r, o, d, h, y, w, C);
      id(n, v, b, A, E, I, V, G, F), v = G, b = F, m += g;
    }
  }
  function Ag(n, t, e) {
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
      const m = u / 2;
      let g, y = Math.abs(Math.cos(m) * l / Math.sin(m));
      y > Math.min(h.len / 2, d.len / 2) ? (y = Math.min(h.len / 2, d.len / 2), g = Math.abs(y * Math.sin(m) / Math.cos(m))) : g = l;
      const w = a.x + d.nx * y + -d.ny * g * p, x = a.y + d.ny * y + d.nx * g * p, _ = Math.atan2(h.ny, h.nx) + Math.PI / 2 * p, v = Math.atan2(d.ny, d.nx) - Math.PI / 2 * p;
      o === 0 && n.moveTo(w + Math.cos(_) * g, x + Math.sin(_) * g), n.arc(w, x, g, _, v, f), r = a;
    }
  }
  function kg(n, t, e, s) {
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
        const g = Math.min(u / 2, c);
        p = r(l, h, g / u);
      }
      const f = i(d, l);
      let m;
      if (f < 1e-4) m = l;
      else {
        const g = Math.min(f / 2, c);
        m = r(l, d, g / f);
      }
      a === 0 ? n.moveTo(p.x, p.y) : n.lineTo(p.x, p.y), n.quadraticCurveTo(l.x, l.y, m.x, m.y, s);
    }
  }
  const Mg = new Ht();
  class Pg {
    constructor(t) {
      this.shapePrimitives = [], this._currentPoly = null, this._bounds = new ke(), this._graphicsPath2D = t, this.signed = t.checkForHoles;
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
      return rd(a, t, e, s, i, r, o), this;
    }
    arcTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly.points;
      return Cg(o, t, e, s, i, r), this;
    }
    arcToSvg(t, e, s, i, r, o, a) {
      const l = this._currentPoly.points;
      return Eg(l, this._currentPoly.lastX, this._currentPoly.lastY, o, a, t, e, s, i, r), this;
    }
    bezierCurveTo(t, e, s, i, r, o, a) {
      this._ensurePoly();
      const l = this._currentPoly;
      return id(this._currentPoly.points, l.lastX, l.lastY, t, e, s, i, r, o, a), this;
    }
    quadraticCurveTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly;
      return wg(this._currentPoly.points, o.lastX, o.lastY, t, e, s, i, r), this;
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
      return this.drawShape(new Ht(t, e, s, i), r), this;
    }
    circle(t, e, s, i) {
      return this.drawShape(new na(t, e, s), i), this;
    }
    poly(t, e, s) {
      const i = new Js(t);
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
        const p = u * h + c, f = t + s * Math.cos(p), m = e + s * Math.sin(p), g = p + Math.PI + d, y = p - Math.PI - d, w = f + r * Math.cos(g), x = m + r * Math.sin(g), _ = f + r * Math.cos(y), v = m + r * Math.sin(y);
        u === 0 ? this.moveTo(w, x) : this.lineTo(w, x), this.quadraticCurveTo(f, m, _, v, a);
      }
      return this.closePath();
    }
    roundShape(t, e, s = false, i) {
      return t.length < 3 ? this : (s ? kg(this, t, e, i) : Ag(this, t, e), this.closePath());
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
      return this.drawShape(new sa(t, e, s, i), r), this;
    }
    roundRect(t, e, s, i, r, o) {
      return this.drawShape(new ia(t, e, s, i, r), o), this;
    }
    drawShape(t, e) {
      return this.endPoly(), this.shapePrimitives.push({
        shape: t,
        transform: e
      }), this;
    }
    startPoly(t, e) {
      let s = this._currentPoly;
      return s && this.endPoly(), s = new Js(), s.points.push(t, e), this._currentPoly = s, this;
    }
    endPoly(t = false) {
      const e = this._currentPoly;
      return e && e.points.length > 2 && (e.closePath = t, this.shapePrimitives.push({
        shape: e
      })), this._currentPoly = null, this;
    }
    _ensurePoly(t = true) {
      if (!this._currentPoly && (this._currentPoly = new Js(), t)) {
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
        const i = e[s], r = i.shape.getBounds(Mg);
        i.transform ? t.addRect(r, i.transform) : t.addRect(r);
      }
      return t;
    }
  }
  class mn {
    constructor(t, e = false) {
      this.instructions = [], this.uid = ee("graphicsPath"), this._dirty = true, this.checkForHoles = e, typeof t == "string" ? Qf(t, this) : this.instructions = (t == null ? void 0 : t.slice()) ?? [];
    }
    get shapePath() {
      return this._shapePath || (this._shapePath = new Pg(this)), this._dirty && (this._dirty = false, this._shapePath.buildPath()), this._shapePath;
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
      const o = this.instructions[this.instructions.length - 1], a = this.getLastPoint(Tt.shared);
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
      const i = this.instructions[this.instructions.length - 1], r = this.getLastPoint(Tt.shared);
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
      const e = new mn();
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
      let l = 0, c = 0, h = 0, d = 0, u = 0, p = 0, f = 0, m = 0;
      for (let g = 0; g < this.instructions.length; g++) {
        const y = this.instructions[g], w = y.data;
        switch (y.action) {
          case "moveTo":
          case "lineTo":
            l = w[0], c = w[1], w[0] = e * l + i * c + o, w[1] = s * l + r * c + a;
            break;
          case "bezierCurveTo":
            h = w[0], d = w[1], u = w[2], p = w[3], l = w[4], c = w[5], w[0] = e * h + i * d + o, w[1] = s * h + r * d + a, w[2] = e * u + i * p + o, w[3] = s * u + r * p + a, w[4] = e * l + i * c + o, w[5] = s * l + r * c + a;
            break;
          case "quadraticCurveTo":
            h = w[0], d = w[1], l = w[2], c = w[3], w[0] = e * h + i * d + o, w[1] = s * h + r * d + a, w[2] = e * l + i * c + o, w[3] = s * l + r * c + a;
            break;
          case "arcToSvg":
            l = w[5], c = w[6], f = w[0], m = w[1], w[0] = e * f + i * m, w[1] = s * f + r * m, w[5] = e * l + i * c + o, w[6] = s * l + r * c + a;
            break;
          case "circle":
            w[4] = Gs(w[3], t);
            break;
          case "rect":
            w[4] = Gs(w[4], t);
            break;
          case "ellipse":
            w[8] = Gs(w[8], t);
            break;
          case "roundRect":
            w[5] = Gs(w[5], t);
            break;
          case "addPath":
            w[0].transform(t);
            break;
          case "poly":
            w[2] = Gs(w[2], t);
            break;
          default:
            Jt("unknown transform action", y.action);
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
  function Gs(n, t) {
    return n ? n.prepend(t) : t.clone();
  }
  function Qt(n, t, e) {
    const s = n.getAttribute(t);
    return s ? Number(s) : e;
  }
  function Ig(n, t) {
    const e = n.querySelectorAll("defs");
    for (let s = 0; s < e.length; s++) {
      const i = e[s];
      for (let r = 0; r < i.children.length; r++) {
        const o = i.children[r];
        switch (o.nodeName.toLowerCase()) {
          case "lineargradient":
            t.defs[o.id] = Rg(o);
            break;
          case "radialgradient":
            t.defs[o.id] = Lg();
            break;
        }
      }
    }
  }
  function Rg(n) {
    const t = Qt(n, "x1", 0), e = Qt(n, "y1", 0), s = Qt(n, "x2", 1), i = Qt(n, "y2", 0), r = n.getAttribute("gradientUnits") || "objectBoundingBox", o = new yn(t, e, s, i, r === "objectBoundingBox" ? "local" : "global");
    for (let a = 0; a < n.children.length; a++) {
      const l = n.children[a], c = Qt(l, "offset", 0), h = Xt.shared.setValue(l.getAttribute("stop-color")).toNumber();
      o.addColorStop(c, h);
    }
    return o;
  }
  function Lg(n) {
    return Jt("[SVG Parser] Radial gradients are not yet supported"), new yn(0, 0, 1, 0);
  }
  function Ml(n) {
    const t = n.match(/url\s*\(\s*['"]?\s*#([^'"\s)]+)\s*['"]?\s*\)/i);
    return t ? t[1] : "";
  }
  const Pl = {
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
  function od(n, t) {
    const e = n.getAttribute("style"), s = {}, i = {}, r = {
      strokeStyle: s,
      fillStyle: i,
      useFill: false,
      useStroke: false
    };
    for (const o in Pl) {
      const a = n.getAttribute(o);
      a && Il(t, r, o, a.trim());
    }
    if (e) {
      const o = e.split(";");
      for (let a = 0; a < o.length; a++) {
        const l = o[a].trim(), [c, h] = l.split(":");
        Pl[c] && Il(t, r, c, h.trim());
      }
    }
    return {
      strokeStyle: r.useStroke ? s : null,
      fillStyle: r.useFill ? i : null,
      useFill: r.useFill,
      useStroke: r.useStroke
    };
  }
  function Il(n, t, e, s) {
    switch (e) {
      case "stroke":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Ml(s);
            t.strokeStyle.fill = n.defs[i];
          } else t.strokeStyle.color = Xt.shared.setValue(s).toNumber();
          t.useStroke = true;
        }
        break;
      case "stroke-width":
        t.strokeStyle.width = Number(s);
        break;
      case "fill":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Ml(s);
            t.fillStyle.fill = n.defs[i];
          } else t.fillStyle.color = Xt.shared.setValue(s).toNumber();
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
  function $g(n) {
    if (n.length <= 2) return true;
    const t = n.map((a) => a.area).sort((a, l) => l - a), [e, s] = t, i = t[t.length - 1], r = e / s, o = s / i;
    return !(r > 3 && o < 2);
  }
  function Bg(n) {
    return n.split(/(?=[Mm])/).filter((s) => s.trim().length > 0);
  }
  function Og(n) {
    const t = n.match(/[-+]?[0-9]*\.?[0-9]+/g);
    if (!t || t.length < 4) return 0;
    const e = t.map(Number), s = [], i = [];
    for (let h = 0; h < e.length; h += 2) h + 1 < e.length && (s.push(e[h]), i.push(e[h + 1]));
    if (s.length === 0 || i.length === 0) return 0;
    const r = Math.min(...s), o = Math.max(...s), a = Math.min(...i), l = Math.max(...i);
    return (o - r) * (l - a);
  }
  function Rl(n, t) {
    const e = new mn(n, false);
    for (const s of e.instructions) t.instructions.push(s);
  }
  function Ng(n, t) {
    if (typeof n == "string") {
      const o = document.createElement("div");
      o.innerHTML = n.trim(), n = o.querySelector("svg");
    }
    const e = {
      context: t,
      defs: {},
      path: new mn()
    };
    Ig(n, e);
    const s = n.children, { fillStyle: i, strokeStyle: r } = od(n, e);
    for (let o = 0; o < s.length; o++) {
      const a = s[o];
      a.nodeName.toLowerCase() !== "defs" && ad(a, e, i, r);
    }
    return t;
  }
  function ad(n, t, e, s) {
    const i = n.children, { fillStyle: r, strokeStyle: o } = od(n, t);
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
    let l, c, h, d, u, p, f, m, g, y, w, x, _, v, b, C, T;
    switch (n.nodeName.toLowerCase()) {
      case "path": {
        v = n.getAttribute("d");
        const L = n.getAttribute("fill-rule"), A = Bg(v), E = L === "evenodd", I = A.length > 1;
        if (E && I) {
          const G = A.map(($) => ({
            path: $,
            area: Og($)
          }));
          if (G.sort(($, z) => z.area - $.area), A.length > 3 || !$g(G)) for (let $ = 0; $ < G.length; $++) {
            const z = G[$], J = $ === 0;
            t.context.beginPath();
            const X = new mn(void 0, true);
            Rl(z.path, X), t.context.path(X), J ? (e && t.context.fill(e), s && t.context.stroke(s)) : t.context.cut();
          }
          else for (let $ = 0; $ < G.length; $++) {
            const z = G[$], J = $ % 2 === 1;
            t.context.beginPath();
            const X = new mn(void 0, true);
            Rl(z.path, X), t.context.path(X), J ? t.context.cut() : (e && t.context.fill(e), s && t.context.stroke(s));
          }
        } else {
          const G = L ? L === "evenodd" : true;
          b = new mn(v, G), t.context.path(b), e && t.context.fill(e), s && t.context.stroke(s);
        }
        break;
      }
      case "circle":
        f = Qt(n, "cx", 0), m = Qt(n, "cy", 0), g = Qt(n, "r", 0), t.context.ellipse(f, m, g, g), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "rect":
        l = Qt(n, "x", 0), c = Qt(n, "y", 0), C = Qt(n, "width", 0), T = Qt(n, "height", 0), y = Qt(n, "rx", 0), w = Qt(n, "ry", 0), y || w ? t.context.roundRect(l, c, C, T, y || w) : t.context.rect(l, c, C, T), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "ellipse":
        f = Qt(n, "cx", 0), m = Qt(n, "cy", 0), y = Qt(n, "rx", 0), w = Qt(n, "ry", 0), t.context.beginPath(), t.context.ellipse(f, m, y, w), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "line":
        h = Qt(n, "x1", 0), d = Qt(n, "y1", 0), u = Qt(n, "x2", 0), p = Qt(n, "y2", 0), t.context.beginPath(), t.context.moveTo(h, d), t.context.lineTo(u, p), s && t.context.stroke(s);
        break;
      case "polygon":
        _ = n.getAttribute("points"), x = _.match(/-?\d+/g).map((L) => parseInt(L, 10)), t.context.poly(x, true), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "polyline":
        _ = n.getAttribute("points"), x = _.match(/-?\d+/g).map((L) => parseInt(L, 10)), t.context.poly(x, false), s && t.context.stroke(s);
        break;
      case "g":
      case "svg":
        break;
      default: {
        Jt(`[SVG parser] <${n.nodeName}> elements unsupported`);
        break;
      }
    }
    a && (e = null);
    for (let L = 0; L < i.length; L++) ad(i[L], t, e, s);
  }
  const Ll = {
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
  mr = class {
    constructor(t, e) {
      this.uid = ee("fillPattern"), this._tick = 0, this.transform = new vt(), this.texture = t, this.transform.scale(1 / t.frame.width, 1 / t.frame.height), e && (t.source.style.addressModeU = Ll[e].addressModeU, t.source.style.addressModeV = Ll[e].addressModeV);
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
  function Fg(n) {
    return Xt.isColorLike(n);
  }
  function $l(n) {
    return n instanceof mr;
  }
  function Bl(n) {
    return n instanceof yn;
  }
  function Wg(n) {
    return n instanceof Ct;
  }
  function Gg(n, t, e) {
    const s = Xt.shared.setValue(t ?? 0);
    return n.color = s.toNumber(), n.alpha = s.alpha === 1 ? e.alpha : s.alpha, n.texture = Ct.WHITE, {
      ...e,
      ...n
    };
  }
  function zg(n, t, e) {
    return n.texture = t, {
      ...e,
      ...n
    };
  }
  function Ol(n, t, e) {
    return n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, {
      ...e,
      ...n
    };
  }
  function Nl(n, t, e) {
    return t.buildGradient(), n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, n.textureSpace = t.textureSpace, {
      ...e,
      ...n
    };
  }
  function Dg(n, t) {
    const e = {
      ...t,
      ...n
    }, s = Xt.shared.setValue(e.color);
    return e.alpha *= s.alpha, e.color = s.toNumber(), e;
  }
  function Xn(n, t) {
    if (n == null) return null;
    const e = {}, s = n;
    return Fg(n) ? Gg(e, n, t) : Wg(n) ? zg(e, n, t) : $l(n) ? Ol(e, n, t) : Bl(n) ? Nl(e, n, t) : s.fill && $l(s.fill) ? Ol(s, s.fill, t) : s.fill && Bl(s.fill) ? Nl(s, s.fill, t) : Dg(s, t);
  }
  function tr(n, t) {
    const { width: e, alignment: s, miterLimit: i, cap: r, join: o, pixelLine: a, ...l } = t, c = Xn(n, l);
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
  function Hg(n, t) {
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
        const h = (c - 2 + a) % a, d = (c + 2) % a, u = o[h], p = o[h + 1], f = o[c], m = o[c + 1], g = o[d], y = o[d + 1], w = u - f, x = p - m, _ = g - f, v = y - m, b = w * w + x * x, C = _ * _ + v * v;
        if (b < 1e-12 || C < 1e-12) continue;
        let A = (w * _ + x * v) / Math.sqrt(b * C);
        A < -1 ? A = -1 : A > 1 && (A = 1);
        const E = Math.sqrt((1 - A) * 0.5);
        if (E < 1e-6) continue;
        const I = Math.min(1 / E, t);
        I > e && (e = I);
      }
    }
    return e;
  }
  const Ug = new Tt(), Fl = new vt(), ca = class Ze extends rn {
    constructor() {
      super(...arguments), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ee("graphicsContext"), this.dirty = true, this.batchMode = "auto", this.instructions = [], this.destroyed = false, this._activePath = new mn(), this._transform = new vt(), this._fillStyle = {
        ...Ze.defaultFillStyle
      }, this._strokeStyle = {
        ...Ze.defaultStrokeStyle
      }, this._stateStack = [], this._tick = 0, this._bounds = new ke(), this._boundsDirty = true;
    }
    clone() {
      const t = new Ze();
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
      this._fillStyle = Xn(t, Ze.defaultFillStyle);
    }
    get strokeStyle() {
      return this._strokeStyle;
    }
    set strokeStyle(t) {
      this._strokeStyle = tr(t, Ze.defaultStrokeStyle);
    }
    setFillStyle(t) {
      return this._fillStyle = Xn(t, Ze.defaultFillStyle), this;
    }
    setStrokeStyle(t) {
      return this._strokeStyle = Xn(t, Ze.defaultStrokeStyle), this;
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
          style: e || e === 0 ? Xt.shared.setValue(e).toNumber() : 16777215
        }
      }), this.onUpdate(), this;
    }
    beginPath() {
      return this._activePath = new mn(), this;
    }
    fill(t, e) {
      let s;
      const i = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (i == null ? void 0 : i.action) === "stroke" ? s = i.data.path : s = this._activePath.clone(), s ? (t != null && (e !== void 0 && typeof t == "number" && (Mt(te, "GraphicsContext.fill(color, alpha) is deprecated, use GraphicsContext.fill({ color, alpha }) instead"), t = {
        color: t,
        alpha: e
      }), this._fillStyle = Xn(t, Ze.defaultFillStyle)), this.instructions.push({
        action: "fill",
        data: {
          style: this.fillStyle,
          path: s
        }
      }), this.onUpdate(), this._initNextPathLocation(), this._tick = 0, this) : this;
    }
    _initNextPathLocation() {
      const { x: t, y: e } = this._activePath.getLastPoint(Tt.shared);
      this._activePath.clear(), this._activePath.moveTo(t, e);
    }
    stroke(t) {
      let e;
      const s = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (s == null ? void 0 : s.action) === "fill" ? e = s.data.path : e = this._activePath.clone(), e ? (t != null && (this._strokeStyle = tr(t, Ze.defaultStrokeStyle)), this.instructions.push({
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
      return this._tick++, Ng(t, this), this;
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
      return t instanceof vt ? (this._transform.set(t.a, t.b, t.c, t.d, t.tx, t.ty), this) : (this._transform.set(t, e, s, i, r, o), this);
    }
    transform(t, e, s, i, r, o) {
      return t instanceof vt ? (this._transform.append(t), this) : (Fl.set(t, e, s, i, r, o), this._transform.append(Fl), this);
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
          r.style.join === "miter" && (a *= Hg(r.path, r.style.miterLimit));
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
          const u = c[h].transform, p = u ? u.applyInverse(t, Ug) : t;
          if (r.action === "fill") s = d.contains(p.x, p.y);
          else {
            const m = l;
            s = d.strokeContains(p.x, p.y, m.width, m.alignment);
          }
          const f = o.hole;
          if (f) {
            const m = (_a2 = f.shapePath) == null ? void 0 : _a2.shapePrimitives;
            if (m) for (let g = 0; g < m.length; g++) m[g].shape.contains(p.x, p.y) && (s = false);
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
  ca.defaultFillStyle = {
    color: 16777215,
    alpha: 1,
    texture: Ct.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local"
  };
  ca.defaultStrokeStyle = {
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
  let Pe = ca;
  function ha(n, t = 1) {
    var _a2;
    const e = (_a2 = As.RETINA_PREFIX) == null ? void 0 : _a2.exec(n);
    return e ? parseFloat(e[1]) : t;
  }
  function da(n, t, e) {
    n.label = e, n._sourceOrigin = e;
    const s = new Ct({
      source: n,
      label: e
    }), i = () => {
      delete t.promiseCache[e], ne.has(e) && ne.remove(e);
    };
    return s.source.once("destroy", () => {
      t.promiseCache[e] && (Jt("[Assets] A TextureSource managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the TextureSource."), i());
    }), s.once("destroy", () => {
      n.destroyed || (Jt("[Assets] A Texture managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the Texture."), i());
    }), s;
  }
  const jg = ".svg", Vg = "image/svg+xml", Yg = {
    extension: {
      type: ht.LoadParser,
      priority: Ln.Low,
      name: "loadSVG"
    },
    name: "loadSVG",
    id: "svg",
    config: {
      crossOrigin: "anonymous",
      parseAsGraphicsContext: false
    },
    test(n) {
      return ks(n, Vg) || Ms(n, jg);
    },
    async load(n, t, e) {
      var _a2;
      return ((_a2 = t.data) == null ? void 0 : _a2.parseAsGraphicsContext) ?? this.config.parseAsGraphicsContext ? qg(n) : Xg(n, t, e, this.config.crossOrigin);
    },
    unload(n) {
      n.destroy(true);
    }
  };
  async function Xg(n, t, e, s) {
    var _a2, _b2, _c2;
    const i = await Pt.get().fetch(n), r = Pt.get().createImage();
    r.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(await i.text())}`, r.crossOrigin = s, await r.decode();
    const o = ((_a2 = t.data) == null ? void 0 : _a2.width) ?? r.width, a = ((_b2 = t.data) == null ? void 0 : _b2.height) ?? r.height, l = ((_c2 = t.data) == null ? void 0 : _c2.resolution) || ha(n), c = Math.ceil(o * l), h = Math.ceil(a * l), d = Pt.get().createCanvas(c, h), u = d.getContext("2d");
    u.imageSmoothingEnabled = true, u.imageSmoothingQuality = "high", u.drawImage(r, 0, 0, o * l, a * l);
    const { parseAsGraphicsContext: p, ...f } = t.data ?? {}, m = new Cs({
      resource: d,
      alphaMode: "premultiply-alpha-on-upload",
      resolution: l,
      ...f
    });
    return da(m, e, n);
  }
  async function qg(n) {
    const e = await (await Pt.get().fetch(n)).text(), s = new Pe();
    return s.svg(e), s;
  }
  const Kg = `(function () {
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
  let ys = null, Lo = class {
    constructor() {
      ys || (ys = URL.createObjectURL(new Blob([
        Kg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(ys);
    }
  };
  Lo.revokeObjectURL = function() {
    ys && (URL.revokeObjectURL(ys), ys = null);
  };
  const Jg = `(function () {
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
  let xs = null;
  class ld {
    constructor() {
      xs || (xs = URL.createObjectURL(new Blob([
        Jg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(xs);
    }
  }
  ld.revokeObjectURL = function() {
    xs && (URL.revokeObjectURL(xs), xs = null);
  };
  let Wl = 0, Jr;
  class Zg {
    constructor() {
      this._initialized = false, this._createdWorkers = 0, this._workerPool = [], this._queue = [], this._resolveHash = {};
    }
    isImageBitmapSupported() {
      return this._isImageBitmapSupported !== void 0 ? this._isImageBitmapSupported : (this._isImageBitmapSupported = new Promise((t) => {
        const { worker: e } = new Lo();
        e.addEventListener("message", (s) => {
          e.terminate(), Lo.revokeObjectURL(), t(s.data);
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
      Jr === void 0 && (Jr = navigator.hardwareConcurrency || 4);
      let t = this._workerPool.pop();
      return !t && this._createdWorkers < Jr && (this._createdWorkers++, t = new ld().worker, t.addEventListener("message", (e) => {
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
      this._resolveHash[Wl] = {
        resolve: e.resolve,
        reject: e.reject
      }, t.postMessage({
        data: e.arguments,
        uuid: Wl++,
        id: s
      });
    }
    reset() {
      this._workerPool.forEach((t) => t.terminate()), this._workerPool.length = 0, Object.values(this._resolveHash).forEach(({ reject: t }) => {
        t == null ? void 0 : t(new Error("WorkerManager has been reset before completion"));
      }), this._resolveHash = {}, this._queue.length = 0, this._initialized = false, this._createdWorkers = 0;
    }
  }
  const Gl = new Zg(), Qg = [
    ".jpeg",
    ".jpg",
    ".png",
    ".webp",
    ".avif"
  ], ty = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/avif"
  ];
  async function ey(n, t) {
    var _a2;
    const e = await Pt.get().fetch(n);
    if (!e.ok) throw new Error(`[loadImageBitmap] Failed to fetch ${n}: ${e.status} ${e.statusText}`);
    const s = await e.blob();
    return ((_a2 = t == null ? void 0 : t.data) == null ? void 0 : _a2.alphaMode) === "premultiplied-alpha" ? createImageBitmap(s, {
      premultiplyAlpha: "none"
    }) : createImageBitmap(s);
  }
  const cd = {
    name: "loadTextures",
    id: "texture",
    extension: {
      type: ht.LoadParser,
      priority: Ln.High,
      name: "loadTextures"
    },
    config: {
      preferWorkers: true,
      preferCreateImageBitmap: true,
      crossOrigin: "anonymous"
    },
    test(n) {
      return ks(n, ty) || Ms(n, Qg);
    },
    async load(n, t, e) {
      var _a2;
      let s = null;
      globalThis.createImageBitmap && this.config.preferCreateImageBitmap ? this.config.preferWorkers && await Gl.isImageBitmapSupported() ? s = await Gl.loadImageBitmap(n, t) : s = await ey(n, t) : s = await new Promise((r, o) => {
        s = Pt.get().createImage(), s.crossOrigin = this.config.crossOrigin, s.src = n, s.complete ? r(s) : (s.onload = () => {
          r(s);
        }, s.onerror = o);
      });
      const i = new Cs({
        resource: s,
        alphaMode: "premultiply-alpha-on-upload",
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || ha(n),
        ...t.data
      });
      return da(i, e, n);
    },
    unload(n) {
      n.destroy(true);
    }
  }, ny = [
    ".mp4",
    ".m4v",
    ".webm",
    ".ogg",
    ".ogv",
    ".h264",
    ".avi",
    ".mov"
  ];
  let Zr, Qr;
  function sy(n, t, e) {
    e === void 0 && !t.startsWith("data:") ? n.crossOrigin = ry(t) : e !== false && (n.crossOrigin = typeof e == "string" ? e : "anonymous");
  }
  function iy(n) {
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
  function ry(n, t = globalThis.location) {
    if (n.startsWith("data:")) return "";
    t || (t = globalThis.location);
    const e = new URL(n, document.baseURI);
    return e.hostname !== t.hostname || e.port !== t.port || e.protocol !== t.protocol ? "anonymous" : "";
  }
  function oy() {
    const n = [], t = [];
    for (const e of ny) {
      const s = Ks.MIME_TYPES[e.substring(1)] || `video/${e.substring(1)}`;
      pr(s) && (n.push(e), t.includes(s) || t.push(s));
    }
    return {
      validVideoExtensions: n,
      validVideoMime: t
    };
  }
  const ay = {
    name: "loadVideo",
    id: "video",
    extension: {
      type: ht.LoadParser,
      name: "loadVideo"
    },
    test(n) {
      if (!Zr || !Qr) {
        const { validVideoExtensions: s, validVideoMime: i } = oy();
        Zr = s, Qr = i;
      }
      const t = ks(n, Qr), e = Ms(n, Zr);
      return t || e;
    },
    async load(n, t, e) {
      var _a2, _b2;
      const s = {
        ...Ks.defaultOptions,
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || ha(n),
        alphaMode: ((_b2 = t.data) == null ? void 0 : _b2.alphaMode) || await bh(),
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
      }), s.muted === true && (i.muted = true), sy(i, n, s.crossorigin);
      const o = document.createElement("source");
      let a;
      if (s.mime) a = s.mime;
      else if (n.startsWith("data:")) a = n.slice(5, n.indexOf(";"));
      else if (!n.startsWith("blob:")) {
        const l = n.split("?")[0].slice(n.lastIndexOf(".") + 1).toLowerCase();
        a = Ks.MIME_TYPES[l] || `video/${l}`;
      }
      return o.src = n, a && (o.type = a), new Promise((l, c) => {
        s.preload && !s.autoPlay && i.load(), i.addEventListener("canplay", h), i.addEventListener("error", d), o.addEventListener("error", d), i.appendChild(o);
        async function h() {
          const p = new Ks({
            ...s,
            resource: i
          });
          u(), t.data.preload && await iy(i), l(da(p, e, n));
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
  }, hd = {
    extension: {
      type: ht.ResolveParser,
      name: "resolveTexture"
    },
    test: cd.test,
    parse: (n) => {
      var _a2;
      return {
        resolution: parseFloat(((_a2 = As.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
        format: n.split(".").pop(),
        src: n
      };
    }
  }, ly = {
    extension: {
      type: ht.ResolveParser,
      priority: -2,
      name: "resolveJson"
    },
    test: (n) => As.RETINA_PREFIX.test(n) && n.endsWith(".json"),
    parse: hd.parse
  };
  class cy {
    constructor() {
      this._detections = [], this._initialized = false, this.resolver = new As(), this.loader = new Bf(), this.cache = ne, this._backgroundLoader = new Ef(this.loader), this._backgroundLoader.active = true, this.reset();
    }
    async init(t = {}) {
      var _a2, _b2;
      if (this._initialized) {
        Jt("[Assets]AssetManager already initialized, did you load before calling this Assets.init()?");
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
      const s = Ki(t), i = He(t).map((a) => {
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
        const p = i[d], f = Object.values(p), g = [
          ...new Set(f.flat())
        ].reduce((y, w) => y + (w.progressSize || 1), 0);
        return l.push(0), a += g, this._mapLoadToResolve(p, (y) => {
          l[u] = y * g, c();
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
      if (typeof t == "string") return ne.get(t);
      const e = {};
      for (let s = 0; s < t.length; s++) e[s] = ne.get(t[s]);
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
        }), ne.set(l, a);
      }), r;
    }
    async unload(t) {
      this._initialized || await this.init();
      const e = He(t).map((i) => typeof i != "string" ? i.src : i), s = this.resolver.resolve(e);
      await this._unloadFromResolved(s);
    }
    async unloadBundle(t) {
      this._initialized || await this.init(), t = He(t);
      const e = this.resolver.resolveBundle(t), s = Object.keys(e).map((i) => this._unloadFromResolved(e[i]));
      await Promise.all(s);
    }
    async _unloadFromResolved(t) {
      const e = Object.values(t);
      e.forEach((s) => {
        ne.remove(s.src);
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
  const je = new cy();
  Gt.handleByList(ht.LoadParser, je.loader.parsers).handleByList(ht.ResolveParser, je.resolver.parsers).handleByList(ht.CacheParser, je.cache.parsers).handleByList(ht.DetectionParser, je.detections);
  Gt.add(Af, Mf, kf, $f, If, Rf, Lf, Ff, zf, qf, Yg, cd, ay, Tf, Sf, hd, ly);
  const zl = {
    loader: ht.LoadParser,
    resolver: ht.ResolveParser,
    cache: ht.CacheParser,
    detection: ht.DetectionParser
  };
  Gt.handle(ht.Asset, (n) => {
    const t = n.ref;
    Object.entries(zl).filter(([e]) => !!t[e]).forEach(([e, s]) => Gt.add(Object.assign(t[e], {
      extension: t[e].extension ?? s
    })));
  }, (n) => {
    const t = n.ref;
    Object.keys(zl).filter((e) => !!t[e]).forEach((e) => Gt.remove(t[e]));
  });
  class hy {
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
  class dy {
    constructor() {
      this.instructions = new Jo();
    }
    init() {
      this.instructions.reset();
    }
    destroy() {
      this.instructions.destroy(), this.instructions = null;
    }
  }
  const ua = class $o {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Ps({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      $o.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? $o.defaultOptions.bezierSmoothness;
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
      const e = new dy(), s = this.getGpuContext(t);
      return s.graphicsData = e, e.init(), e;
    }
    _initContext(t) {
      const e = new hy();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  ua.extension = {
    type: [
      ht.CanvasSystem
    ],
    name: "graphicsContext"
  };
  ua.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let uy = ua;
  class dd {
    constructor(t, e) {
      this.state = Zi.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Ps({
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
  dd.extension = {
    type: [
      ht.CanvasPipes
    ],
    name: "graphics"
  };
  ud = function(n, t, e) {
    const s = (n >> 24 & 255) / 255;
    t[e++] = (n & 255) / 255 * s, t[e++] = (n >> 8 & 255) / 255 * s, t[e++] = (n >> 16 & 255) / 255 * s, t[e++] = s;
  };
  class py {
    constructor() {
      this.batches = [], this.batched = false;
    }
    destroy() {
      this.batches.forEach((t) => {
        ve.return(t);
      }), this.batches.length = 0;
    }
  }
  class pd {
    constructor(t, e) {
      this.state = Zi.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Ps({
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
      o.uTransformMatrix = t.groupTransform, o.uRound = e._roundPixels | t._roundPixels, ud(t.groupColorAlpha, o.uColor, 0), this._adaptor.execute(this, t);
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
      const e = new py();
      return t._gpuData[this.renderer.uid] = e, this._managedGraphics.add(t), e;
    }
    _updateBatchesForRenderable(t, e) {
      const s = t.context, r = this.renderer.graphicsContext.getGpuContext(s), o = this.renderer._roundPixels | t._roundPixels;
      e.batches = r.batches.map((a) => {
        const l = ve.get(ra);
        return a.copyTo(l), l.renderable = t, l.roundPixels = o, l;
      });
    }
    destroy() {
      this._managedGraphics.destroy(), this.renderer = null, this._adaptor.destroy(), this._adaptor = null, this.state = null;
    }
  }
  pd.extension = {
    type: [
      ht.WebGLPipes,
      ht.WebGPUPipes
    ],
    name: "graphics"
  };
  Gt.add(dd);
  Gt.add(pd);
  Gt.add(uy);
  Gt.add(la);
  ft = class extends hr {
    constructor(t) {
      t instanceof Pe && (t = {
        context: t
      });
      const { context: e, roundPixels: s, ...i } = t || {};
      super({
        label: "Graphics",
        ...i
      }), this.renderPipeId = "graphics", e ? this.context = e : (this.context = this._ownedContext = new Pe(), this.context.autoGarbageCollect = this.autoGarbageCollect), this.didViewUpdate = true, this.allowChildren = false, this.roundPixels = s ?? false;
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
      return t ? new ft(this._context.clone()) : (this._ownedContext = null, new ft(this._context));
    }
    lineStyle(t, e, s) {
      Mt(te, "Graphics#lineStyle is no longer needed. Use Graphics#setStrokeStyle to set the stroke style.");
      const i = {};
      return t && (i.width = t), e && (i.color = e), s && (i.alpha = s), this.context.strokeStyle = i, this;
    }
    beginFill(t, e) {
      Mt(te, "Graphics#beginFill is no longer needed. Use Graphics#fill to fill the shape with the desired style.");
      const s = {};
      return t !== void 0 && (s.color = t), e !== void 0 && (s.alpha = e), this.context.fillStyle = s, this;
    }
    endFill() {
      Mt(te, "Graphics#endFill is no longer needed. Use Graphics#fill to fill the shape with the desired style."), this.context.fill();
      const t = this.context.strokeStyle;
      return (t.width !== Pe.defaultStrokeStyle.width || t.color !== Pe.defaultStrokeStyle.color || t.alpha !== Pe.defaultStrokeStyle.alpha) && this.context.stroke(), this;
    }
    drawCircle(...t) {
      return Mt(te, "Graphics#drawCircle has been renamed to Graphics#circle"), this._callContextMethod("circle", t);
    }
    drawEllipse(...t) {
      return Mt(te, "Graphics#drawEllipse has been renamed to Graphics#ellipse"), this._callContextMethod("ellipse", t);
    }
    drawPolygon(...t) {
      return Mt(te, "Graphics#drawPolygon has been renamed to Graphics#poly"), this._callContextMethod("poly", t);
    }
    drawRect(...t) {
      return Mt(te, "Graphics#drawRect has been renamed to Graphics#rect"), this._callContextMethod("rect", t);
    }
    drawRoundedRect(...t) {
      return Mt(te, "Graphics#drawRoundedRect has been renamed to Graphics#roundRect"), this._callContextMethod("roundRect", t);
    }
    drawStar(...t) {
      return Mt(te, "Graphics#drawStar has been renamed to Graphics#star"), this._callContextMethod("star", t);
    }
  };
  let os;
  function Dl(n) {
    const t = Pt.get().createCanvas(6, 1), e = t.getContext("2d");
    return e.fillStyle = n, e.fillRect(0, 0, 6, 1), t;
  }
  fy = function() {
    if (os !== void 0) return os;
    try {
      const n = Dl("#ff00ff"), t = Dl("#ffff00"), s = Pt.get().createCanvas(6, 1).getContext("2d");
      s.globalCompositeOperation = "multiply", s.drawImage(n, 0, 0), s.drawImage(t, 2, 0);
      const i = s.getImageData(2, 0, 1, 1);
      if (!i) os = false;
      else {
        const r = i.data;
        os = r[0] === 255 && r[1] === 0 && r[2] === 0;
      }
    } catch {
      os = false;
    }
    return os;
  };
  Kt = {
    canvas: null,
    convertTintToImage: false,
    cacheStepsPerColorChannel: 8,
    canUseMultiply: fy(),
    tintMethod: null,
    _canvasSourceCache: /* @__PURE__ */ new WeakMap(),
    _unpremultipliedCache: /* @__PURE__ */ new WeakMap(),
    getCanvasSource: (n) => {
      const t = n.source, e = t == null ? void 0 : t.resource;
      if (!e) return null;
      const s = t.alphaMode === "premultiplied-alpha", i = t.resourceWidth ?? t.pixelWidth, r = t.resourceHeight ?? t.pixelHeight, o = i !== t.pixelWidth || r !== t.pixelHeight;
      if (s) {
        if ((e instanceof HTMLCanvasElement || typeof OffscreenCanvas < "u" && e instanceof OffscreenCanvas) && !o) return e;
        const a = Kt._unpremultipliedCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
      }
      if (e instanceof Uint8Array || e instanceof Uint8ClampedArray || e instanceof Int8Array || e instanceof Uint16Array || e instanceof Int16Array || e instanceof Uint32Array || e instanceof Int32Array || e instanceof Float32Array || e instanceof ArrayBuffer) {
        const a = Kt._canvasSourceCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
        const l = Pt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d"), h = c.createImageData(t.pixelWidth, t.pixelHeight), d = h.data, u = e instanceof ArrayBuffer ? new Uint8Array(e) : new Uint8Array(e.buffer, e.byteOffset, e.byteLength);
        if (t.format === "bgra8unorm") for (let p = 0; p < d.length && p + 3 < u.length; p += 4) d[p] = u[p + 2], d[p + 1] = u[p + 1], d[p + 2] = u[p], d[p + 3] = u[p + 3];
        else d.set(u.subarray(0, d.length));
        return c.putImageData(h, 0, 0), Kt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      if (s) {
        const a = Pt.get().createCanvas(t.pixelWidth, t.pixelHeight), l = a.getContext("2d", {
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
        return l.putImageData(c, 0, 0), Kt._unpremultipliedCache.set(t, {
          canvas: a,
          resourceId: t._resourceId
        }), a;
      }
      if (o) {
        const a = Kt._canvasSourceCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
        const l = Pt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d");
        return l.width = t.pixelWidth, l.height = t.pixelHeight, c.drawImage(e, 0, 0), Kt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      return e;
    },
    getTintedCanvas: (n, t) => {
      const e = n.texture, s = Xt.shared.setValue(t).toHex(), i = e.tintCache || (e.tintCache = {}), r = i[s], o = e.source._resourceId;
      if ((r == null ? void 0 : r.tintId) === o) return r;
      const a = r && "getContext" in r ? r : Pt.get().createCanvas();
      return Kt.tintMethod(e, t, a), a.tintId = o, i[s] = a, i[s];
    },
    getTintedPattern: (n, t) => {
      const e = Xt.shared.setValue(t).toHex(), s = n.patternCache || (n.patternCache = {}), i = n.source._resourceId;
      let r = s[e];
      return (r == null ? void 0 : r.tintId) === i || (Kt.canvas || (Kt.canvas = Pt.get().createCanvas()), Kt.tintMethod(n, t, Kt.canvas), r = Kt.canvas.getContext("2d").createPattern(Kt.canvas, "repeat"), r.tintId = i, s[e] = r), r;
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
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.fillStyle = Xt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "multiply";
      const h = Kt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && Kt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.globalCompositeOperation = "destination-atop", s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
    },
    tintWithOverlay: (n, t, e) => {
      const s = e.getContext("2d"), i = n.frame.clone(), r = n.source._resolution ?? n.source.resolution ?? 1, o = n.rotate;
      i.x *= r, i.y *= r, i.width *= r, i.height *= r;
      const a = St.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy", s.fillStyle = Xt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "destination-atop";
      const h = Kt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && Kt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
    },
    tintWithPerPixel: (n, t, e) => {
      const s = e.getContext("2d"), i = n.frame.clone(), r = n.source._resolution ?? n.source.resolution ?? 1, o = n.rotate;
      i.x *= r, i.y *= r, i.width *= r, i.height *= r;
      const a = St.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy";
      const h = Kt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && Kt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
      const d = t >> 16 & 255, u = t >> 8 & 255, p = t & 255, f = s.getImageData(0, 0, l, c), m = f.data;
      for (let g = 0; g < m.length; g += 4) m[g] = m[g] * d / 255, m[g + 1] = m[g + 1] * u / 255, m[g + 2] = m[g + 2] * p / 255;
      s.putImageData(f, 0, 0);
    },
    _applyInverseRotation: (n, t, e, s) => {
      const i = St.inv(t), r = St.uX(i), o = St.uY(i), a = St.vX(i), l = St.vY(i), c = -Math.min(0, r * e, a * s, r * e + a * s), h = -Math.min(0, o * e, l * s, o * e + l * s);
      n.transform(r, o, a, l, c, h);
    }
  };
  Kt.tintMethod = Kt.canUseMultiply ? Kt.tintWithMultiply : Kt.tintWithPerPixel;
  class my extends hr {
    constructor(t, e) {
      const { text: s, resolution: i, style: r, anchor: o, width: a, height: l, roundPixels: c, ...h } = t;
      super({
        ...h
      }), this.batched = true, this._resolution = null, this._autoResolution = true, this._didTextUpdate = true, this._styleClass = e, this.text = s ?? "", this.style = r, this.resolution = i ?? null, this.allowChildren = false, this._anchor = new ue({
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
  function gy(n, t) {
    let e = n[0] ?? {};
    return (typeof e == "string" || n[1]) && (Mt(te, `use new ${t}({ text: "hi!", style }) instead`), e = {
      text: e,
      style: n[1]
    }), e;
  }
  class yy {
    constructor(t) {
      this._canvasPool = /* @__PURE__ */ Object.create(null), this.canvasOptions = t || {}, this.enableFullScreen = false;
    }
    _createCanvasAndContext(t, e) {
      const s = Pt.get().createCanvas();
      s.width = t, s.height = e;
      const i = s.getContext("2d");
      return {
        canvas: s,
        context: i
      };
    }
    getOptimalCanvasAndContext(t, e, s = 1) {
      t = Math.ceil(t * s - 1e-6), e = Math.ceil(e * s - 1e-6), t = ws(t), e = ws(e);
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
  Bo = new yy();
  pi.register(Bo);
  let Nn = null, dn = null;
  function xy(n, t) {
    Nn || (Nn = Pt.get().createCanvas(256, 128), dn = Nn.getContext("2d", {
      willReadFrequently: true
    }), dn.globalCompositeOperation = "copy", dn.globalAlpha = 1), (Nn.width < n || Nn.height < t) && (Nn.width = ws(n), Nn.height = ws(t));
  }
  function Hl(n, t, e) {
    for (let s = 0, i = 4 * e * t; s < t; ++s, i += 4) if (n[i + 3] !== 0) return false;
    return true;
  }
  function Ul(n, t, e, s, i) {
    const r = 4 * t;
    for (let o = s, a = s * r + 4 * e; o <= i; ++o, a += r) if (n[a + 3] !== 0) return false;
    return true;
  }
  function by(...n) {
    let t = n[0];
    t.canvas || (t = {
      canvas: n[0],
      resolution: n[1]
    });
    const { canvas: e } = t, s = Math.min(t.resolution ?? 1, 1), i = t.width ?? e.width, r = t.height ?? e.height;
    let o = t.output;
    if (xy(i, r), !dn) throw new TypeError("Failed to get canvas 2D context");
    dn.drawImage(e, 0, 0, i, r, 0, 0, i * s, r * s);
    const l = dn.getImageData(0, 0, i, r).data;
    let c = 0, h = 0, d = i - 1, u = r - 1;
    for (; h < r && Hl(l, i, h); ) ++h;
    if (h === r) return Ht.EMPTY;
    for (; Hl(l, i, u); ) --u;
    for (; Ul(l, i, c, h, u); ) ++c;
    for (; Ul(l, i, d, h, u); ) --d;
    return ++d, ++u, dn.globalCompositeOperation = "source-over", dn.strokeRect(c, h, d - c, u - h), dn.globalCompositeOperation = "copy", o ?? (o = new Ht()), o.set(c / s, h / s, (d - c) / s, (u - h) / s), o;
  }
  class _y {
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
  wy = function(n = 1e3, t = 0, e = false) {
    if (isNaN(n) || n < 0) throw new TypeError("Invalid max value");
    if (isNaN(t) || t < 0) throw new TypeError("Invalid ttl value");
    if (typeof e != "boolean") throw new TypeError("Invalid resetTtl value");
    return new _y(n, t, e);
  };
  function fd(n) {
    return !!n.tagStyles && Object.keys(n.tagStyles).length > 0;
  }
  function md(n) {
    return n.includes("<");
  }
  function vy(n, t) {
    return n.clone().assign(t);
  }
  function Cy(n, t) {
    const e = [], s = t.tagStyles;
    if (!fd(t) || !md(n)) return e.push({
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
            const u = i[i.length - 1], p = vy(u, s[d]);
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
  const Sy = [
    10,
    13
  ], Ty = new Set(Sy), Ey = [
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
  ], Ay = new Set(Ey), ky = [
    9,
    32
  ], My = new Set(ky), Py = [
    45,
    8208,
    8211,
    8212,
    173
  ], Iy = new Set(Py), Ry = /(\r\n|\r|\n)/, Ly = /(?:\r\n|\r|\n)/;
  function er(n) {
    return typeof n != "string" ? false : Ty.has(n.charCodeAt(0));
  }
  Re = function(n, t) {
    return typeof n != "string" ? false : Ay.has(n.charCodeAt(0));
  };
  mv = function(n) {
    return typeof n != "string" ? false : My.has(n.charCodeAt(0));
  };
  $y = function(n) {
    return typeof n != "string" ? false : Iy.has(n.charCodeAt(0));
  };
  gd = function(n) {
    return n === "normal" || n === "pre-line";
  };
  yd = function(n) {
    return n === "normal";
  };
  function ln(n) {
    if (typeof n != "string") return "";
    let t = n.length - 1;
    for (; t >= 0 && Re(n[t]); ) t--;
    return t < n.length - 1 ? n.slice(0, t + 1) : n;
  }
  function xd(n) {
    const t = [], e = [];
    if (typeof n != "string") return t;
    for (let s = 0; s < n.length; s++) {
      const i = n[s], r = n[s + 1];
      if (Re(i) || er(i)) {
        e.length > 0 && (t.push(e.join("")), e.length = 0), i === "\r" && r === `
` ? (t.push(`\r
`), s++) : t.push(i);
        continue;
      }
      e.push(i), $y(i) && r && !Re(r) && !er(r) && (t.push(e.join("")), e.length = 0);
    }
    return e.length > 0 && t.push(e.join("")), t;
  }
  function bd(n, t, e, s) {
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
  const By = /\r\n|\r|\n/g;
  function Oy(n, t, e, s, i, r, o, a) {
    var _a2, _b2;
    const l = Cy(n, t);
    if (yd(t.whiteSpace)) for (let F = 0; F < l.length; F++) {
      const $ = l[F];
      l[F] = {
        text: $.text.replace(By, " "),
        style: $.style
      };
    }
    const h = [];
    let d = [];
    for (const F of l) {
      const $ = F.text.split(Ry);
      for (let z = 0; z < $.length; z++) {
        const J = $[z];
        J === `\r
` || J === "\r" || J === `
` ? (h.push(d), d = []) : J.length > 0 && d.push({
          text: J,
          style: F.style
        });
      }
    }
    (d.length > 0 || h.length === 0) && h.push(d);
    const u = e ? Ny(h, t, s, i, o, a) : h, p = [], f = [], m = [], g = [], y = [];
    let w = 0;
    const x = t._fontString, _ = r(x);
    _.fontSize === 0 && (_.fontSize = t.fontSize, _.ascent = t.fontSize);
    let v = "", b = !!t.dropShadow, C = ((_a2 = t._stroke) == null ? void 0 : _a2.width) || 0;
    for (const F of u) {
      let $ = 0, z = _.ascent, J = _.descent, X = "";
      for (const M of F) {
        const O = M.style._fontString, N = r(O);
        O !== v && (s.font = O, v = O);
        const D = i(M.text, M.style.letterSpacing, s);
        $ += D, z = Math.max(z, N.ascent), J = Math.max(J, N.descent), X += M.text;
        const K = ((_b2 = M.style._stroke) == null ? void 0 : _b2.width) || 0;
        K > C && (C = K), !b && M.style.dropShadow && (b = true);
      }
      F.length === 0 && (z = _.ascent, J = _.descent), p.push($), f.push(z), m.push(J), y.push(X);
      const Q = t.lineHeight || z + J;
      g.push(Q + t.leading), w = Math.max(w, $);
    }
    const T = C, E = (e && t.align !== "left" ? Math.max(w, t.wordWrapWidth) : w) + T + (t.dropShadow ? t.dropShadow.distance : 0);
    let I = 0;
    for (let F = 0; F < g.length; F++) I += g[F];
    I = Math.max(I, g[0] + T);
    const V = I + (t.dropShadow ? t.dropShadow.distance : 0), G = t.lineHeight || _.fontSize;
    return {
      width: E,
      height: V,
      lines: y,
      lineWidths: p,
      lineHeight: G + t.leading,
      maxLineWidth: w,
      fontProperties: _,
      runsByLine: u,
      lineAscents: f,
      lineDescents: m,
      lineHeights: g,
      hasDropShadow: b
    };
  }
  function Ny(n, t, e, s, i, r) {
    var _a2;
    const { letterSpacing: o, whiteSpace: a, wordWrapWidth: l, breakWords: c } = t, h = gd(a), d = l + o, u = {};
    let p = "";
    const f = (g, y) => {
      const w = `${g}|${y.styleKey}`;
      let x = u[w];
      if (x === void 0) {
        const _ = y._fontString;
        _ !== p && (e.font = _, p = _), x = s(g, y.letterSpacing, e) + y.letterSpacing, u[w] = x;
      }
      return x;
    }, m = [];
    for (const g of n) {
      const y = Fy(g), w = m.length, x = (E) => {
        let I = 0, V = E;
        do {
          const { token: G, style: F } = y[V];
          I += f(G, F), V++;
        } while (V < y.length && y[V].continuesFromPrevious);
        return I;
      }, _ = (E) => {
        const I = [];
        let V = E;
        do
          I.push({
            token: y[V].token,
            style: y[V].style
          }), V++;
        while (V < y.length && y[V].continuesFromPrevious);
        return I;
      };
      let v = [], b = 0, C = !h, T = null;
      const L = () => {
        T && T.text.length > 0 && v.push(T), T = null;
      }, A = () => {
        if (L(), v.length > 0) {
          const E = v[v.length - 1];
          E.text = ln(E.text), E.text.length === 0 && v.pop();
        }
        m.push(v), v = [], b = 0, C = false;
      };
      for (let E = 0; E < y.length; E++) {
        const { token: I, style: V, continuesFromPrevious: G } = y[E], F = f(I, V);
        if (h) {
          const J = Re(I), X = (T == null ? void 0 : T.text[T.text.length - 1]) ?? ((_a2 = v[v.length - 1]) == null ? void 0 : _a2.text.slice(-1)) ?? "", Q = X ? Re(X) : false;
          if (J && Q) continue;
        }
        const $ = !G, z = $ ? x(E) : F;
        if (z > d && $) if (b > 0 && A(), c) {
          const J = _(E);
          for (let X = 0; X < J.length; X++) {
            const Q = J[X].token, M = J[X].style, O = bd(Q, c, r, i);
            for (const N of O) {
              const D = f(N, M);
              D + b > d && A(), !T || T.style !== M ? (L(), T = {
                text: N,
                style: M
              }) : T.text += N, b += D;
            }
          }
          E += J.length - 1;
        } else {
          const J = _(E);
          L(), m.push(J.map((X) => ({
            text: X.token,
            style: X.style
          }))), C = false, E += J.length - 1;
        }
        else if (z + b > d && $) {
          if (Re(I)) {
            C = false;
            continue;
          }
          A(), T = {
            text: I,
            style: V
          }, b = F;
        } else if (G && !c) !T || T.style !== V ? (L(), T = {
          text: I,
          style: V
        }) : T.text += I, b += F;
        else {
          const J = Re(I);
          if (b === 0 && J && !C) continue;
          !T || T.style !== V ? (L(), T = {
            text: I,
            style: V
          }) : T.text += I, b += F;
        }
      }
      if (L(), v.length > 0) {
        const E = v[v.length - 1];
        E.text = ln(E.text), E.text.length === 0 && v.pop();
      }
      (v.length > 0 || m.length === w) && m.push(v);
    }
    return m;
  }
  function Fy(n) {
    const t = [];
    let e = false;
    for (const s of n) {
      const i = xd(s.text);
      let r = true;
      for (const o of i) {
        const a = Re(o) || er(o), l = r && e && !a;
        t.push({
          token: o,
          style: s.style,
          continuesFromPrevious: l
        }), e = !a, r = false;
      }
    }
    return t;
  }
  const Wy = {
    willReadFrequently: true
  };
  function jl(n, t, e, s, i) {
    let r = e[n];
    return typeof r != "number" && (r = i(n, t, s) + t, e[n] = r), r;
  }
  function Gy(n, t, e, s, i, r, o) {
    const a = e.getContext("2d", Wy);
    a.font = t._fontString;
    let l = 0, c = "";
    const h = [], d = /* @__PURE__ */ Object.create(null), { letterSpacing: u, whiteSpace: p } = t, f = gd(p), m = yd(p);
    let g = !f;
    const y = t.wordWrapWidth + u, w = xd(n);
    for (let _ = 0; _ < w.length; _++) {
      let v = w[_];
      if (er(v)) {
        if (!m) {
          h.push(ln(c)), g = !f, c = "", l = 0;
          continue;
        }
        v = " ";
      }
      if (f) {
        const C = Re(v), T = Re(c[c.length - 1]);
        if (C && T) continue;
      }
      const b = jl(v, u, d, a, s);
      if (b > y) if (c !== "" && (h.push(ln(c)), c = "", l = 0), i(v, t.breakWords)) {
        const C = bd(v, t.breakWords, o, r);
        for (const T of C) {
          const L = jl(T, u, d, a, s);
          L + l > y && (h.push(ln(c)), g = false, c = "", l = 0), c += T, l += L;
        }
      } else c.length > 0 && (h.push(ln(c)), c = "", l = 0), h.push(ln(v)), g = false, c = "", l = 0;
      else b + l > y && (g = false, h.push(ln(c)), c = "", l = 0), (c.length > 0 || !Re(v) || g) && (c += v, l += b);
    }
    const x = ln(c);
    return x.length > 0 && h.push(x), h.join(`
`);
  }
  const Vl = {
    willReadFrequently: true
  }, xn = class xt {
    static get experimentalLetterSpacingSupported() {
      let t = xt._experimentalLetterSpacingSupported;
      if (t === void 0) {
        const e = Pt.get().getCanvasRenderingContext2D().prototype;
        t = xt._experimentalLetterSpacingSupported = "letterSpacing" in e || "textLetterSpacing" in e;
      }
      return t;
    }
    constructor(t, e, s, i, r, o, a, l, c, h) {
      this.text = t, this.style = e, this.width = s, this.height = i, this.lines = r, this.lineWidths = o, this.lineHeight = a, this.maxLineWidth = l, this.fontProperties = c, h && (this.runsByLine = h.runsByLine, this.lineAscents = h.lineAscents, this.lineDescents = h.lineDescents, this.lineHeights = h.lineHeights, this.hasDropShadow = h.hasDropShadow);
    }
    static measureText(t = " ", e, s = xt._canvas, i = e.wordWrap) {
      var _a2;
      const r = `${t}-${e.styleKey}-wordWrap-${i}`;
      if (xt._measurementCache.has(r)) return xt._measurementCache.get(r);
      if (fd(e) && md(t)) {
        const v = Oy(t, e, i, xt._context, xt._measureText, xt.measureFont, xt.canBreakChars, xt.wordWrapSplit), b = new xt(t, e, v.width, v.height, v.lines, v.lineWidths, v.lineHeight, v.maxLineWidth, v.fontProperties, {
          runsByLine: v.runsByLine,
          lineAscents: v.lineAscents,
          lineDescents: v.lineDescents,
          lineHeights: v.lineHeights,
          hasDropShadow: v.hasDropShadow
        });
        return xt._measurementCache.set(r, b), b;
      }
      const a = e._fontString, l = xt.measureFont(a);
      l.fontSize === 0 && (l.fontSize = e.fontSize, l.ascent = e.fontSize, l.descent = 0);
      const c = xt._context;
      c.font = a;
      const d = (i ? xt._wordWrap(t, e, s) : t).split(Ly), u = new Array(d.length);
      let p = 0;
      for (let v = 0; v < d.length; v++) {
        const b = xt._measureText(d[v], e.letterSpacing, c);
        u[v] = b, p = Math.max(p, b);
      }
      const f = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0, m = e.lineHeight || l.fontSize, g = xt._getAlignWidth(p, e, i), y = xt._adjustWidthForStyle(g, e), w = Math.max(m, l.fontSize + f) + (d.length - 1) * (m + e.leading), x = xt._adjustHeightForStyle(w, e), _ = new xt(t, e, y, x, d, u, m + e.leading, p, l);
      return xt._measurementCache.set(r, _), _;
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
      xt.experimentalLetterSpacingSupported && (xt.experimentalLetterSpacing ? (s.letterSpacing = `${e}px`, s.textLetterSpacing = `${e}px`, i = true) : (s.letterSpacing = "0px", s.textLetterSpacing = "0px"));
      const r = s.measureText(t);
      let o = r.width;
      const a = -(r.actualBoundingBoxLeft ?? 0);
      let c = (r.actualBoundingBoxRight ?? 0) - a;
      if (o > 0) if (i) o -= e, c -= e;
      else {
        const h = (xt.graphemeSegmenter(t).length - 1) * e;
        o += h, c += h;
      }
      return Math.max(o, c);
    }
    static _wordWrap(t, e, s = xt._canvas) {
      return Gy(t, e, s, xt._measureText, xt.canBreakWords, xt.canBreakChars, xt.wordWrapSplit);
    }
    static isBreakingSpace(t, e) {
      return Re(t);
    }
    static canBreakWords(t, e) {
      return e;
    }
    static canBreakChars(t, e, s, i, r) {
      return true;
    }
    static wordWrapSplit(t) {
      return xt.graphemeSegmenter(t);
    }
    static measureFont(t) {
      if (xt._fonts[t]) return xt._fonts[t];
      const e = xt._context;
      e.font = t;
      const s = e.measureText(xt.METRICS_STRING + xt.BASELINE_SYMBOL), i = s.actualBoundingBoxAscent ?? 0, r = s.actualBoundingBoxDescent ?? 0, o = {
        ascent: i,
        descent: r,
        fontSize: i + r
      };
      return xt._fonts[t] = o, o;
    }
    static clearMetrics(t = "") {
      t ? delete xt._fonts[t] : xt._fonts = {};
    }
    static get _canvas() {
      var _a2;
      if (!xt.__canvas) {
        let t;
        try {
          const e = new OffscreenCanvas(0, 0);
          if ((_a2 = e.getContext("2d", Vl)) == null ? void 0 : _a2.measureText) return xt.__canvas = e, e;
          t = Pt.get().createCanvas();
        } catch {
          t = Pt.get().createCanvas();
        }
        t.width = t.height = 10, xt.__canvas = t;
      }
      return xt.__canvas;
    }
    static get _context() {
      return xt.__context || (xt.__context = xt._canvas.getContext("2d", Vl)), xt.__context;
    }
  };
  xn.METRICS_STRING = "|\xC9q\xC5";
  xn.BASELINE_SYMBOL = "M";
  xn.BASELINE_MULTIPLIER = 1.4;
  xn.HEIGHT_MULTIPLIER = 2;
  xn.graphemeSegmenter = (() => {
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
  xn.experimentalLetterSpacing = false;
  xn._fonts = {};
  xn._measurementCache = wy(1e3);
  cn = xn;
  const zy = [
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui"
  ];
  Oo = function(n) {
    const t = typeof n.fontSize == "number" ? `${n.fontSize}px` : n.fontSize;
    let e = n.fontFamily;
    Array.isArray(n.fontFamily) || (e = n.fontFamily.split(","));
    for (let s = e.length - 1; s >= 0; s--) {
      let i = e[s].trim();
      !/([\"\'])[^\'\"]+\1/.test(i) && !zy.includes(i) && (i = `"${i}"`), e[s] = i;
    }
    return `${n.fontStyle} ${n.fontVariant} ${n.fontWeight} ${t} ${e.join(",")}`;
  };
  const Yl = 1e5;
  Ii = function(n, t, e, s = 0, i = 0, r = 0) {
    if (n.texture === Ct.WHITE && !n.fill) return Xt.shared.setValue(n.color).setAlpha(n.alpha ?? 1).toHexa();
    if (n.fill) {
      if (n.fill instanceof mr) {
        const o = n.fill, a = t.createPattern(o.texture.source.resource, "repeat"), l = o.transform.copyTo(vt.shared);
        return l.scale(o.texture.source.pixelWidth, o.texture.source.pixelHeight), a.setTransform(l), a;
      } else if (n.fill instanceof yn) {
        const o = n.fill, a = o.type === "linear", l = o.textureSpace === "local";
        let c = 1, h = 1;
        l && e && (c = e.width + s, h = e.height + s);
        let d, u = false;
        if (a) {
          const { start: p, end: f } = o;
          d = t.createLinearGradient(p.x * c + i, p.y * h + r, f.x * c + i, f.y * h + r), u = Math.abs(f.x - p.x) < Math.abs((f.y - p.y) * 0.1);
        } else {
          const { center: p, innerRadius: f, outerCenter: m, outerRadius: g } = o;
          d = t.createRadialGradient(p.x * c + i, p.y * h + r, f * c, m.x * c + i, m.y * h + r, g * c);
        }
        if (u && l && e) {
          const p = e.lineHeight / h;
          for (let f = 0; f < e.lines.length; f++) {
            const m = (f * e.lineHeight + s / 2) / h;
            o.colorStops.forEach((g) => {
              let y = m + g.offset * p;
              y = Math.max(0, Math.min(1, y)), d.addColorStop(Math.floor(y * Yl) / Yl, Xt.shared.setValue(g.color).toHex());
            });
          }
        } else o.colorStops.forEach((p) => {
          d.addColorStop(p.offset, Xt.shared.setValue(p.color).toHex());
        });
        return d;
      }
    } else {
      const o = t.createPattern(n.texture.source.resource, "repeat"), a = n.matrix.copyTo(vt.shared);
      return a.scale(n.texture.source.pixelWidth, n.texture.source.pixelHeight), o.setTransform(a), o;
    }
    return Jt("FillStyle not recognised", n), "red";
  };
  const Xl = new Ht();
  function as(n) {
    let t = 0;
    for (let e = 0; e < n.length; e++) n.charCodeAt(e) === 32 && t++;
    return t;
  }
  class Dy {
    getCanvasAndContext(t) {
      const { text: e, style: s, resolution: i = 1 } = t, r = s._getFinalPadding(), o = cn.measureText(e || " ", s), a = Math.ceil(Math.ceil(Math.max(1, o.width) + r * 2) * i), l = Math.ceil(Math.ceil(Math.max(1, o.height) + r * 2) * i), c = Bo.getOptimalCanvasAndContext(a, l);
      this._renderTextToCanvas(s, r, i, c, o);
      const h = s.trim ? by({
        canvas: c.canvas,
        width: a,
        height: l,
        resolution: 1,
        output: Xl
      }) : Xl.set(0, 0, a, l);
      return {
        canvasAndContext: c,
        frame: h
      };
    }
    returnCanvasAndContext(t) {
      Bo.returnCanvasAndContext(t);
    }
    _renderTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      if (r.runsByLine && r.runsByLine.length > 0) {
        this._renderTaggedTextToCanvas(r, t, e, s, i);
        return;
      }
      const { canvas: o, context: a } = i, l = Oo(t), c = r.lines, h = r.lineHeight, d = r.lineWidths, u = r.maxLineWidth, p = r.fontProperties, f = o.height;
      if (a.resetTransform(), a.scale(s, s), a.textBaseline = t.textBaseline, (_a2 = t._stroke) == null ? void 0 : _a2.width) {
        const b = t._stroke;
        a.lineWidth = b.width, a.miterLimit = b.miterLimit, a.lineJoin = b.join, a.lineCap = b.cap;
      }
      a.font = l;
      let m, g;
      const y = t.dropShadow ? 2 : 1, w = t.wordWrap ? Math.max(t.wordWrapWidth, u) : u, _ = (((_b2 = t._stroke) == null ? void 0 : _b2.width) ?? 0) / 2;
      let v = (h - p.fontSize) / 2;
      h - p.fontSize < 0 && (v = 0);
      for (let b = 0; b < y; ++b) {
        const C = t.dropShadow && b === 0, T = C ? Math.ceil(Math.max(1, f) + e * 2) : 0, L = T * s;
        if (C) this._setupDropShadow(a, t, s, L);
        else {
          const A = t._gradientBounds, E = t._gradientOffset;
          if (A) {
            const I = {
              width: A.width,
              height: A.height,
              lineHeight: A.height,
              lines: r.lines
            };
            this._setFillAndStrokeStyles(a, t, I, e, _, (E == null ? void 0 : E.x) ?? 0, (E == null ? void 0 : E.y) ?? 0);
          } else E ? this._setFillAndStrokeStyles(a, t, r, e, _, E.x, E.y) : this._setFillAndStrokeStyles(a, t, r, e, _);
          a.shadowColor = "rgba(0,0,0,0)";
        }
        for (let A = 0; A < c.length; A++) {
          m = _, g = _ + A * h + p.ascent + v, m += this._getAlignmentOffset(d[A], w, t.align);
          let E = 0;
          if (t.align === "justify" && t.wordWrap && A < c.length - 1) {
            const I = as(c[A]);
            I > 0 && (E = (w - d[A]) / I);
          }
          ((_c2 = t._stroke) == null ? void 0 : _c2.width) && this._drawLetterSpacing(c[A], t, i, m + e, g + e - T, true, E), t._fill !== void 0 && this._drawLetterSpacing(c[A], t, i, m + e, g + e - T, false, E);
        }
      }
    }
    _renderTaggedTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      const { canvas: o, context: a } = r, { runsByLine: l, lineWidths: c, maxLineWidth: h, lineAscents: d, lineHeights: u, hasDropShadow: p } = t, f = o.height;
      a.resetTransform(), a.scale(i, i), a.textBaseline = e.textBaseline;
      const m = p ? 2 : 1, g = e.wordWrap ? Math.max(e.wordWrapWidth, h) : h;
      let y = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0;
      for (const _ of l) for (const v of _) {
        const b = ((_b2 = v.style._stroke) == null ? void 0 : _b2.width) ?? 0;
        b > y && (y = b);
      }
      const w = y / 2, x = [];
      for (let _ = 0; _ < l.length; _++) {
        const v = l[_], b = [];
        for (const C of v) {
          const T = Oo(C.style);
          a.font = T, b.push({
            width: cn._measureText(C.text, C.style.letterSpacing, a),
            font: T
          });
        }
        x.push(b);
      }
      for (let _ = 0; _ < m; ++_) {
        const v = p && _ === 0, b = v ? Math.ceil(Math.max(1, f) + s * 2) : 0, C = b * i;
        v || (a.shadowColor = "rgba(0,0,0,0)");
        let T = w;
        for (let L = 0; L < l.length; L++) {
          const A = l[L], E = c[L], I = d[L], V = u[L], G = x[L];
          let F = w;
          F += this._getAlignmentOffset(E, g, e.align);
          let $ = 0;
          if (e.align === "justify" && e.wordWrap && L < l.length - 1) {
            let X = 0;
            for (const Q of A) X += as(Q.text);
            X > 0 && ($ = (g - E) / X);
          }
          const z = T + I;
          let J = F + s;
          for (let X = 0; X < A.length; X++) {
            const Q = A[X], { width: M, font: O } = G[X];
            if (a.font = O, a.textBaseline = Q.style.textBaseline, (_c2 = Q.style._stroke) == null ? void 0 : _c2.width) {
              const D = Q.style._stroke;
              if (a.lineWidth = D.width, a.miterLimit = D.miterLimit, a.lineJoin = D.join, a.lineCap = D.cap, v) if (Q.style.dropShadow) this._setupDropShadow(a, Q.style, i, C);
              else {
                const K = as(Q.text);
                J += M + K * $;
                continue;
              }
              else {
                const K = cn.measureFont(O), tt = Q.style.lineHeight || K.fontSize, ot = {
                  width: M,
                  height: tt,
                  lineHeight: tt,
                  lines: [
                    Q.text
                  ]
                };
                a.strokeStyle = Ii(D, a, ot, s * 2, J - s, T);
              }
              this._drawLetterSpacing(Q.text, Q.style, r, J, z + s - b, true, $);
            }
            const N = as(Q.text);
            J += M + N * $;
          }
          J = F + s;
          for (let X = 0; X < A.length; X++) {
            const Q = A[X], { width: M, font: O } = G[X];
            if (a.font = O, a.textBaseline = Q.style.textBaseline, Q.style._fill !== void 0) {
              if (v) if (Q.style.dropShadow) this._setupDropShadow(a, Q.style, i, C);
              else {
                const D = as(Q.text);
                J += M + D * $;
                continue;
              }
              else {
                const D = cn.measureFont(O), K = Q.style.lineHeight || D.fontSize, tt = {
                  width: M,
                  height: K,
                  lineHeight: K,
                  lines: [
                    Q.text
                  ]
                };
                a.fillStyle = Ii(Q.style._fill, a, tt, s * 2, J - s, T);
              }
              this._drawLetterSpacing(Q.text, Q.style, r, J, z + s - b, false, $);
            }
            const N = as(Q.text);
            J += M + N * $;
          }
          T += V;
        }
      }
    }
    _setFillAndStrokeStyles(t, e, s, i, r, o = 0, a = 0) {
      var _a2;
      if (t.fillStyle = e._fill ? Ii(e._fill, t, s, i * 2, o, a) : null, (_a2 = e._stroke) == null ? void 0 : _a2.width) {
        const l = r + i * 2;
        t.strokeStyle = Ii(e._stroke, t, s, l, o, a);
      }
    }
    _setupDropShadow(t, e, s, i) {
      t.fillStyle = "black", t.strokeStyle = "black";
      const r = e.dropShadow, o = r.color, a = r.alpha;
      t.shadowColor = Xt.shared.setValue(o).setAlpha(a).toRgbaString();
      const l = r.blur * s, c = r.distance * s;
      t.shadowBlur = l, t.shadowOffsetX = Math.cos(r.angle) * c, t.shadowOffsetY = Math.sin(r.angle) * c + i;
    }
    _getAlignmentOffset(t, e, s) {
      return s === "right" ? e - t : s === "center" ? (e - t) / 2 : 0;
    }
    _drawLetterSpacing(t, e, s, i, r, o = false, a = 0) {
      const { context: l } = s, c = e.letterSpacing;
      let h = false;
      if (cn.experimentalLetterSpacingSupported && (cn.experimentalLetterSpacing ? (l.letterSpacing = `${c}px`, l.textLetterSpacing = `${c}px`, h = true) : (l.letterSpacing = "0px", l.textLetterSpacing = "0px")), (c === 0 || h) && a === 0) {
        o ? l.strokeText(t, i, r) : l.fillText(t, i, r);
        return;
      }
      if (a !== 0 && (c === 0 || h)) {
        const m = t.split(" ");
        let g = i;
        const y = l.measureText(" ").width;
        for (let w = 0; w < m.length; w++) o ? l.strokeText(m[w], g, r) : l.fillText(m[w], g, r), g += l.measureText(m[w]).width + y + a;
        return;
      }
      let d = i;
      const u = cn.graphemeSegmenter(t);
      let p = l.measureText(t).width, f = 0;
      for (let m = 0; m < u.length; ++m) {
        const g = u[m];
        o ? l.strokeText(g, d, r) : l.fillText(g, d, r);
        let y = "";
        for (let w = m + 1; w < u.length; ++w) y += u[w];
        f = l.measureText(y).width, d += p - f + c, g === " " && (d += a), p = f;
      }
    }
  }
  const ms = new Dy(), pa = class Vn extends rn {
    constructor(t = {}) {
      super(), this.uid = ee("textStyle"), this._tick = 0, this._cachedFontString = null, Hy(t), t instanceof Vn && (t = t._toObject());
      const i = {
        ...Vn.defaultTextStyle,
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
        ...Vn.defaultDropShadow,
        ...t
      }) : this._dropShadow = t ? this._createProxy({
        ...Vn.defaultDropShadow
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
        ...Pe.defaultFillStyle,
        ...t
      }, () => {
        this._fill = Xn({
          ...this._originalFill
        }, Pe.defaultFillStyle);
      })), this._fill = Xn(t === 0 ? "black" : t, Pe.defaultFillStyle), this.update());
    }
    get stroke() {
      return this._originalStroke;
    }
    set stroke(t) {
      t !== this._originalStroke && (this._originalStroke = t, this._isFillStyle(t) && (this._originalStroke = this._createProxy({
        ...Pe.defaultStrokeStyle,
        ...t
      }, () => {
        this._stroke = tr({
          ...this._originalStroke
        }, Pe.defaultStrokeStyle);
      })), this._stroke = tr(t, Pe.defaultStrokeStyle), this.update());
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
      const t = Vn.defaultTextStyle;
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
      return this._cachedFontString === null && (this._cachedFontString = Oo(this)), this._cachedFontString;
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
      return new Vn(this._toObject());
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
      return (t ?? null) !== null && !(Xt.isColorLike(t) || t instanceof yn || t instanceof mr);
    }
  };
  pa.defaultDropShadow = {
    alpha: 1,
    angle: Math.PI / 6,
    blur: 0,
    color: "black",
    distance: 5
  };
  pa.defaultTextStyle = {
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
  Be = pa;
  function Hy(n) {
    const t = n;
    if (typeof t.dropShadow == "boolean" && t.dropShadow) {
      const e = Be.defaultDropShadow;
      n.dropShadow = {
        alpha: t.dropShadowAlpha ?? e.alpha,
        angle: t.dropShadowAngle ?? e.angle,
        blur: t.dropShadowBlur ?? e.blur,
        color: t.dropShadowColor ?? e.color,
        distance: t.dropShadowDistance ?? e.distance
      };
    }
    if (t.strokeThickness !== void 0) {
      Mt(te, "strokeThickness is now a part of stroke");
      const e = t.stroke;
      let s = {};
      if (Xt.isColorLike(e)) s.color = e;
      else if (e instanceof yn || e instanceof mr) s.fill = e;
      else if (Object.hasOwnProperty.call(e, "color") || Object.hasOwnProperty.call(e, "fill")) s = e;
      else throw new Error("Invalid stroke value.");
      n.stroke = {
        ...s,
        width: t.strokeThickness
      };
    }
    if (Array.isArray(t.fillGradientStops)) {
      if (Mt(te, "gradient fill is now a fill pattern: `new FillGradient(...)`"), !Array.isArray(t.fill) || t.fill.length === 0) throw new Error("Invalid fill value. Expected an array of colors for gradient fill.");
      t.fill.length !== t.fillGradientStops.length && Jt("The number of fill colors must match the number of fill gradient stops.");
      const e = new yn({
        start: {
          x: 0,
          y: 0
        },
        end: {
          x: 0,
          y: 1
        },
        textureSpace: "local"
      }), s = t.fillGradientStops.slice(), i = t.fill.map((r) => Xt.shared.setValue(r).toNumber());
      s.forEach((r, o) => {
        e.addColorStop(r, i[o]);
      }), n.fill = {
        fill: e
      };
    }
  }
  function Uy(n, t) {
    const { texture: e, bounds: s } = n, i = t._style._getFinalPadding();
    eh(s, t._anchor, e);
    const r = t._anchor._x * i * 2, o = t._anchor._y * i * 2;
    s.minX -= i - r, s.minY -= i - o, s.maxX -= i - r, s.maxY -= i - o;
  }
  jy = class {
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
  class Vy extends jy {
  }
  class _d {
    constructor(t) {
      this._renderer = t, t.runners.resolutionChange.add(this), this._managedTexts = new Ps({
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
        (s.currentKey !== t.styleKey || t._resolution !== i) && this._updateGpuText(t), t._didTextUpdate = false, Uy(s, t);
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
      const e = new Vy();
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
  _d.extension = {
    type: [
      ht.WebGLPipes,
      ht.WebGPUPipes,
      ht.CanvasPipes
    ],
    name: "text"
  };
  const Yy = new ke();
  function Xy(n, t, e, s, i = false) {
    const r = Yy;
    r.minX = 0, r.minY = 0, r.maxX = n.width / s | 0, r.maxY = n.height / s | 0;
    const o = cr.getOptimalTexture(r.width, r.height, s, false, i);
    return o.source.uploadMethodId = "image", o.source.resource = n, o.source.alphaMode = "premultiply-alpha-on-upload", o.frame.width = t / s, o.frame.height = e / s, o.source.emit("update", o.source), o.updateUvs(), o;
  }
  class wd {
    constructor(t, e) {
      this._activeTextures = {}, this._renderer = t, this._retainCanvasContext = e;
    }
    getTexture(t, e, s, i) {
      typeof t == "string" && (Mt("8.0.0", "CanvasTextSystem.getTexture: Use object TextOptions instead of separate arguments"), t = {
        text: t,
        style: s,
        resolution: e
      }), t.style instanceof Be || (t.style = new Be(t.style)), t.textureStyle instanceof Kn || (t.textureStyle = new Kn(t.textureStyle)), typeof t.text != "string" && (t.text = t.text.toString());
      const { text: r, style: o, textureStyle: a, autoGenerateMipmaps: l } = t, c = t.resolution ?? this._renderer.resolution, { frame: h, canvasAndContext: d } = ms.getCanvasAndContext({
        text: r,
        style: o,
        resolution: c
      }), u = Xy(d.canvas, h.width, h.height, c, l);
      if (a && (u.source.style = a), o.trim && (h.pad(o.padding), u.frame.copyFrom(h), u.frame.scale(1 / c), u.updateUvs()), o.filters) {
        const p = this._applyFilters(u, o.filters);
        return this.returnTexture(u), ms.returnCanvasAndContext(d), p;
      }
      return this._renderer.texture.initSource(u._source), this._retainCanvasContext || ms.returnCanvasAndContext(d), u;
    }
    returnTexture(t) {
      const e = t.source, s = e.resource;
      if (this._retainCanvasContext && (s == null ? void 0 : s.getContext)) {
        const i = s.getContext("2d");
        i && ms.returnCanvasAndContext({
          canvas: s,
          context: i
        });
      }
      e.resource = null, e.uploadMethodId = "unknown", e.alphaMode = "no-premultiply-alpha", cr.returnTexture(t, true);
    }
    renderTextToCanvas() {
      Mt("8.10.0", "CanvasTextSystem.renderTextToCanvas: no longer supported, use CanvasTextSystem.getTexture instead");
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
  class vd extends wd {
    constructor(t) {
      super(t, true);
    }
  }
  vd.extension = {
    type: [
      ht.CanvasSystem
    ],
    name: "canvasText"
  };
  class Cd extends wd {
    constructor(t) {
      super(t, false);
    }
  }
  Cd.extension = {
    type: [
      ht.WebGLSystem,
      ht.WebGPUSystem
    ],
    name: "canvasText"
  };
  Gt.add(vd);
  Gt.add(Cd);
  Gt.add(_d);
  class Ue extends my {
    constructor(...t) {
      const e = gy(t, "Text");
      super(e, Be), this.renderPipeId = "text", e.textureStyle && (this.textureStyle = e.textureStyle instanceof Kn ? e.textureStyle : new Kn(e.textureStyle)), this.autoGenerateMipmaps = e.autoGenerateMipmaps ?? Me.defaultOptions.autoGenerateMipmaps;
    }
    updateBounds() {
      const t = this._bounds, e = this._anchor;
      let s = 0, i = 0;
      if (this._style.trim) {
        const { frame: r, canvasAndContext: o } = ms.getCanvasAndContext({
          text: this.text,
          style: this._style,
          resolution: 1
        });
        ms.returnCanvasAndContext(o), s = r.width, i = r.height;
      } else {
        const r = cn.measureText(this._text, this._style);
        s = r.width, i = r.height;
      }
      t.minX = -e._x * s, t.maxX = t.minX + s, t.minY = -e._y * i, t.maxY = t.minY + i;
    }
  }
  gr = class extends Ct {
    static create(t) {
      const { dynamic: e, ...s } = t;
      return new gr({
        source: new Me(s),
        dynamic: e ?? false
      });
    }
    resize(t, e, s) {
      return this.source.resize(t, e, s), this;
    }
  };
  class qy {
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
        const m = p & 16777215, g = ((m & 255) << 16) + (m & 65280) + (m >> 16 & 255);
        let y = u.source.resource;
        g !== 16777215 && (y = Kt.getTintedCanvas({
          texture: u
        }, g));
        const w = u.frame, x = u.source.resolution, _ = w.x * x, v = w.y * x, b = w.width * x, C = w.height * x;
        i.globalAlpha = f;
        const T = -d.anchorX * w.width, L = -d.anchorY * w.height;
        d.rotation !== 0 || d.scaleX !== 1 || d.scaleY !== 1 ? (i.save(), i.translate(d.x, d.y), i.rotate(d.rotation), i.scale(d.scaleX, d.scaleY), i.drawImage(y, _, v, b, C, T, L, w.width, w.height), i.restore()) : i.drawImage(y, _, v, b, C, d.x + T, d.y + L, w.width, w.height);
      }
      i.restore();
    }
  }
  function ql(n, t = null) {
    const e = n * 6;
    if (e > 65535 ? t || (t = new Uint32Array(e)) : t || (t = new Uint16Array(e)), t.length !== e) throw new Error(`Out buffer length is incorrect, got ${t.length} and expected ${e}`);
    for (let s = 0, i = 0; s < e; s += 6, i += 4) t[s + 0] = i + 0, t[s + 1] = i + 1, t[s + 2] = i + 2, t[s + 3] = i + 0, t[s + 4] = i + 2, t[s + 5] = i + 3;
    return t;
  }
  function Ky(n) {
    return {
      dynamicUpdate: Kl(n, true),
      staticUpdate: Kl(n, false)
    };
  }
  function Kl(n, t) {
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
      const a = Ji(o.format);
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
  class Jy {
    constructor(t) {
      this._size = 0, this._generateParticleUpdateCache = {};
      const e = this._size = t.size ?? 1e3, s = t.properties;
      let i = 0, r = 0;
      for (const h in s) {
        const d = s[h], u = Ji(d.format);
        d.dynamic ? r += u.stride : i += u.stride;
      }
      this._dynamicStride = r / 4, this._staticStride = i / 4, this.staticAttributeBuffer = new fs(e * 4 * i), this.dynamicAttributeBuffer = new fs(e * 4 * r), this.indexBuffer = ql(e);
      const o = new Kh();
      let a = 0, l = 0;
      this._staticBuffer = new Zn({
        data: new Float32Array(1),
        label: "static-particle-buffer",
        shrinkToFit: false,
        usage: le.VERTEX | le.COPY_DST
      }), this._dynamicBuffer = new Zn({
        data: new Float32Array(1),
        label: "dynamic-particle-buffer",
        shrinkToFit: false,
        usage: le.VERTEX | le.COPY_DST
      });
      for (const h in s) {
        const d = s[h], u = Ji(d.format);
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
      const e = Zy(t);
      return this._generateParticleUpdateCache[e] ? this._generateParticleUpdateCache[e] : (this._generateParticleUpdateCache[e] = this.generateParticleUpdate(t), this._generateParticleUpdateCache[e]);
    }
    generateParticleUpdate(t) {
      return Ky(t);
    }
    update(t, e) {
      t.length > this._size && (e = true, this._size = Math.max(t.length, this._size * 1.5 | 0), this.staticAttributeBuffer = new fs(this._size * this._staticStride * 4 * 4), this.dynamicAttributeBuffer = new fs(this._size * this._dynamicStride * 4 * 4), this.indexBuffer = ql(this._size), this.geometry.indexBuffer.setDataWithSize(this.indexBuffer, this.indexBuffer.byteLength, true));
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
  function Zy(n) {
    const t = [];
    for (const e in n) {
      const s = n[e];
      t.push(e, s.code, s.dynamic ? "d" : "s");
    }
    return t.join("_");
  }
  var Qy = `varying vec2 vUV;
varying vec4 vColor;

uniform sampler2D uTexture;

void main(void){
    vec4 color = texture2D(uTexture, vUV) * vColor;
    gl_FragColor = color;
}`, tx = `attribute vec2 aVertex;
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
`, Jl = `
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
  class ex extends ur {
    constructor() {
      const t = Qo.from({
        vertex: tx,
        fragment: Qy
      }), e = fi.from({
        fragment: {
          source: Jl,
          entryPoint: "mainFragment"
        },
        vertex: {
          source: Jl,
          entryPoint: "mainVertex"
        }
      });
      super({
        glProgram: t,
        gpuProgram: e,
        resources: {
          uTexture: Ct.WHITE.source,
          uSampler: new Kn({}),
          uniforms: {
            uTranslationMatrix: {
              value: new vt(),
              type: "mat3x3<f32>"
            },
            uColor: {
              value: new Xt(16777215),
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
  class yr {
    constructor(t, e) {
      this.state = Zi.for2d(), this.localUniforms = new ta({
        uTranslationMatrix: {
          value: new vt(),
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
      }), this.renderer = t, this.adaptor = e, this.defaultShader = new ex(), this.state = Zi.for2d(), this._managedContainers = new Ps({
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
      return t._gpuData[this.renderer.uid] = new Jy({
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
      i.update(e, t._childrenDirty), t._childrenDirty = false, r.blendMode = ko(t.blendMode, t.texture._source);
      const o = this.localUniforms.uniforms, a = o.uTranslationMatrix;
      t.worldTransform.copyTo(a);
      const l = s.globalUniforms.globalUniformData;
      a.tx -= l.offset.x, a.ty -= l.offset.y, a.prepend(l.projectionMatrix), o.uResolution = l.resolution, o.uRound = s._roundPixels | t._roundPixels, ud(t.groupColorAlpha, o.uColor, 0), this.adaptor.execute(this, t);
    }
    destroy() {
      this._managedContainers.destroy(), this.renderer = null, this.defaultShader && (this.defaultShader.destroy(), this.defaultShader = null);
    }
  }
  yr.extension = {
    type: [
      ht.CanvasPipes
    ],
    name: "particle"
  };
  class Sd extends yr {
    constructor(t) {
      super(t, new qy());
    }
  }
  Sd.extension = {
    type: [
      ht.CanvasPipes
    ],
    name: "particle"
  };
  class nx {
    execute(t, e) {
      const s = t.state, i = t.renderer, r = e.shader || t.defaultShader;
      r.resources.uTexture = e.texture._source, r.resources.uniforms = t.localUniforms;
      const o = i.gl, a = t.getBuffers(e);
      i.shader.bind(r), i.state.set(s), i.geometry.bind(a.geometry, r.glProgram);
      const c = a.geometry.indexBuffer.data.BYTES_PER_ELEMENT === 2 ? o.UNSIGNED_SHORT : o.UNSIGNED_INT;
      o.drawElements(o.TRIANGLES, e.particleChildren.length * 6, c, 0);
    }
  }
  class Td extends yr {
    constructor(t) {
      super(t, new nx());
    }
  }
  Td.extension = {
    type: [
      ht.WebGLPipes
    ],
    name: "particle"
  };
  class sx {
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
  class Ed extends yr {
    constructor(t) {
      super(t, new sx());
    }
  }
  Ed.extension = {
    type: [
      ht.WebGPUPipes
    ],
    name: "particle"
  };
  const Ad = class No {
    constructor(t) {
      if (t instanceof Ct) this.texture = t, xo(this, No.defaultOptions, {});
      else {
        const e = {
          ...No.defaultOptions,
          ...t
        };
        xo(this, e, {});
      }
    }
    get alpha() {
      return this._alpha;
    }
    set alpha(t) {
      this._alpha = Math.min(Math.max(t, 0), 1), this._updateColor();
    }
    get tint() {
      return qs(this._tint);
    }
    set tint(t) {
      this._tint = Xt.shared.setValue(t ?? 16777215).toBgrNumber(), this._updateColor();
    }
    _updateColor() {
      this.color = this._tint + ((this._alpha * 255 | 0) << 24);
    }
  };
  Ad.defaultOptions = {
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
  let nr = Ad;
  const Zl = {
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
  Gt.add(Td);
  Gt.add(Ed);
  Gt.add(Sd);
  const ix = new ke(0, 0, 0, 0), kd = class Fo extends hr {
    constructor(t = {}) {
      t = {
        ...Fo.defaultOptions,
        ...t,
        dynamicProperties: {
          ...Fo.defaultOptions.dynamicProperties,
          ...t == null ? void 0 : t.dynamicProperties
        }
      };
      const { dynamicProperties: e, shader: s, roundPixels: i, texture: r, particles: o, ...a } = t;
      super({
        label: "ParticleContainer",
        ...a
      }), this.renderPipeId = "particle", this.batched = false, this._childrenDirty = false, this.texture = r || null, this.shader = s, this._properties = {};
      for (const l in Zl) {
        const c = Zl[l], h = e[l];
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
      return ix;
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
  kd.defaultOptions = {
    dynamicProperties: {
      vertex: false,
      position: true,
      rotation: false,
      uvs: false,
      color: false
    },
    roundPixels: false
  };
  let rx = kd;
  Gt.add(Iu, Ru);
  var ox = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
  function ax(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var Md = {
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
    }).call(ox);
  })(Md);
  var lx = Md.exports;
  const Ql = ax(lx);
  function xr(n, t) {
    if (n) {
      if (typeof n == "function") return n;
      if (typeof n == "string") return Ql[n];
    } else return Ql[t];
  }
  class cx {
    constructor(t) {
      this.viewport = t, this.touches = [], this.addListeners();
    }
    addListeners() {
      this.viewport.eventMode = "static", this.viewport.forceHitArea || (this.viewport.hitArea = new Ht(0, 0, this.viewport.worldWidth, this.viewport.worldHeight)), this.viewport.on("pointerdown", this.down, this), this.viewport.options.allowPreserveDragOutside ? this.viewport.on("globalpointermove", this.move, this) : this.viewport.on("pointermove", this.move, this), this.viewport.on("pointerup", this.up, this), this.viewport.on("pointerupoutside", this.up, this), this.viewport.on("pointercancel", this.up, this), this.viewport.options.allowPreserveDragOutside || this.viewport.on("pointerleave", this.up, this), this.wheelFunction = (t) => this.handleWheel(t), this.viewport.options.events.domElement.addEventListener("wheel", this.wheelFunction, {
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
      const e = new Tt();
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
  const zs = [
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
  class hx {
    constructor(t) {
      this.viewport = t, this.list = [], this.plugins = {};
    }
    add(t, e, s = zs.length) {
      const i = this.plugins[t];
      i && i.destroy(), this.plugins[t] = e;
      const r = zs.indexOf(t);
      r !== -1 && zs.splice(r, 1), zs.splice(s, 0, t), this.sort();
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
      for (const t of zs) this.plugins[t] && this.list.push(this.plugins[t]);
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
  class Oe {
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
  const dx = {
    removeOnInterrupt: false,
    ease: "linear",
    time: 1e3
  };
  class ux extends Oe {
    constructor(t, e = {}) {
      super(t), this.startWidth = null, this.startHeight = null, this.deltaWidth = null, this.deltaHeight = null, this.width = null, this.height = null, this.time = 0, this.options = Object.assign({}, dx, e), this.options.ease = xr(this.options.ease), this.setupPosition(), this.setupZoom(), this.time = 0;
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
      const e = new Tt(this.parent.scale.x, this.parent.scale.y);
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
          const i = this.startX, r = this.startY, o = this.deltaX, a = this.deltaY, l = new Tt(this.parent.x, this.parent.y);
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
  const px = {
    sides: "all",
    friction: 0.5,
    time: 150,
    ease: "easeInOutSine",
    underflow: "center",
    bounceBox: null
  };
  class fx extends Oe {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, px, e), this.ease = xr(this.options.ease, "easeInOutSine"), this.options.sides ? this.options.sides === "all" ? this.top = this.bottom = this.left = this.right = true : this.options.sides === "horizontal" ? (this.right = this.left = true, this.top = this.bottom = false) : this.options.sides === "vertical" ? (this.left = this.right = false, this.top = this.bottom = true) : (this.top = this.options.sides.indexOf("top") !== -1, this.bottom = this.options.sides.indexOf("bottom") !== -1, this.left = this.options.sides.indexOf("left") !== -1, this.right = this.options.sides.indexOf("right") !== -1) : this.left = this.top = this.right = this.bottom = false;
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
          topLeft: new Tt(e * this.parent.scale.x, s * this.parent.scale.y),
          bottomRight: new Tt(i * this.parent.scale.x - this.parent.screenWidth, r * this.parent.scale.y - this.parent.screenHeight)
        };
      }
      return {
        left: this.parent.left < 0,
        right: this.parent.right > this.parent.worldWidth,
        top: this.parent.top < 0,
        bottom: this.parent.bottom > this.parent.worldHeight,
        topLeft: new Tt(0, 0),
        bottomRight: new Tt(this.parent.worldWidth * this.parent.scale.x - this.parent.screenWidth, this.parent.worldHeight * this.parent.scale.y - this.parent.screenHeight)
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
  const mx = {
    left: false,
    right: false,
    top: false,
    bottom: false,
    direction: null,
    underflow: "center"
  };
  class gx extends Oe {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, mx, e), this.options.direction && (this.options.left = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.right = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.top = this.options.direction === "y" || this.options.direction === "all" ? true : null, this.options.bottom = this.options.direction === "y" || this.options.direction === "all" ? true : null), this.parseUnderflow(), this.last = {
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
      const t = new Tt(this.parent.x, this.parent.y), e = this.parent.plugins.decelerate || {};
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
  const yx = {
    minWidth: null,
    minHeight: null,
    maxWidth: null,
    maxHeight: null,
    minScale: null,
    maxScale: null
  };
  class xx extends Oe {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, yx, e), this.clamp();
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
  const bx = {
    friction: 0.98,
    bounce: 0.8,
    minSpeed: 0.01
  }, _n = 16;
  class _x extends Oe {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, bx, e), this.saved = [], this.timeSinceRelease = 0, this.reset(), this.parent.on("moved", (s) => this.handleMoved(s));
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
        this.parent.x += this.x * _n / o * (Math.pow(r, i / _n) - Math.pow(r, s / _n)), this.x *= Math.pow(this.percentChangeX, t / _n);
      }
      if (this.y) {
        const r = this.percentChangeY, o = Math.log(r);
        this.parent.y += this.y * _n / o * (Math.pow(r, i / _n) - Math.pow(r, s / _n)), this.y *= Math.pow(this.percentChangeY, t / _n);
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
  const wx = {
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
  class vx extends Oe {
    constructor(t, e = {}) {
      super(t), this.windowEventHandlers = [], this.options = Object.assign({}, wx, e), this.moved = false, this.reverse = this.options.reverse ? 1 : -1, this.xDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "x", this.yDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "y", this.keyIsPressed = false, this.parseUnderflow(), this.mouseButtons(this.options.mouseButtons), this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
            screen: new Tt(this.last.x, this.last.y),
            world: this.parent.toWorld(new Tt(this.last.x, this.last.y)),
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
        const s = new Tt(this.last.x, this.last.y);
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
  const Cx = {
    speed: 0,
    acceleration: null,
    radius: null
  };
  class Sx extends Oe {
    constructor(t, e, s = {}) {
      super(t), this.target = e, this.options = Object.assign({}, Cx, s), this.velocity = {
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
  const Tx = {
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
  class Ex extends Oe {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Tx, e), this.reverse = this.options.reverse ? 1 : -1, this.radiusSquared = typeof this.options.radius == "number" ? Math.pow(this.options.radius, 2) : null, this.resize();
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
  const Ax = {
    noDrag: false,
    percent: 1,
    center: null,
    factor: 1,
    axis: "all"
  }, kx = new Tt();
  class Mx extends Oe {
    constructor(t, e = {}) {
      super(t), this.active = false, this.pinching = false, this.moved = false, this.options = Object.assign({}, Ax, e);
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
      const { x: e, y: s } = (this.parent.parent || this.parent).toLocal(t.global, void 0, kx), i = this.parent.input.touches;
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
          const c = new Tt(r.last.x + (o.last.x - r.last.x) / 2, r.last.y + (o.last.y - r.last.y) / 2);
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
  const Px = {
    topLeft: false,
    friction: 0.8,
    time: 1e3,
    ease: "easeInOutSine",
    interrupt: true,
    removeOnComplete: false,
    removeOnInterrupt: false,
    forceStart: false
  };
  class Ix extends Oe {
    constructor(t, e, s, i = {}) {
      super(t), this.options = Object.assign({}, Px, i), this.ease = xr(i.ease, "easeInOutSine"), this.x = e, this.y = s, this.options.forceStart && this.snapStart();
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
  const Rx = {
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
  class Lx extends Oe {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Rx, e), this.ease = xr(this.options.ease), this.xIndependent = false, this.yIndependent = false, this.xScale = 0, this.yScale = 0, this.options.width > 0 && (this.xScale = t.screenWidth / this.options.width, this.xIndependent = true), this.options.height > 0 && (this.yScale = t.screenHeight / this.options.height, this.yIndependent = true), this.xScale = this.xIndependent ? this.xScale : this.yScale, this.yScale = this.yIndependent ? this.yScale : this.xScale, this.options.time === 0 ? (t.container.scale.x = this.xScale, t.container.scale.y = this.yScale, this.options.removeOnComplete && this.parent.plugins.remove("snap-zoom")) : e.forceStart && this.createSnapping();
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
  const $x = {
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
  class Bx extends Oe {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, $x, e), this.keyIsPressed = false, this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
  const Ox = {
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
    ticker: Yn.shared,
    allowPreserveDragOutside: false
  };
  class Pd extends Ut {
    constructor(t) {
      super(), this._disableOnContextMenu = (e) => e.preventDefault(), this.options = {
        ...Ox,
        ...t
      }, this.screenWidth = this.options.screenWidth, this.screenHeight = this.options.screenHeight, this._worldWidth = this.options.worldWidth, this._worldHeight = this.options.worldHeight, this.forceHitArea = this.options.forceHitArea, this.threshold = this.options.threshold, this.options.disableOnContextMenu && this.options.events.domElement.addEventListener("contextmenu", this._disableOnContextMenu), this.options.noTicker || (this.tickerFunction = () => this.update(this.options.ticker.elapsedMS), this.options.ticker.add(this.tickerFunction)), this.input = new cx(this), this.plugins = new hx(this);
    }
    destroy(t) {
      var e;
      !this.options.noTicker && this.tickerFunction && this.options.ticker.remove(this.tickerFunction), this.options.disableOnContextMenu && ((e = this.options.events.domElement) == null || e.removeEventListener("contextmenu", this._disableOnContextMenu)), this.input.destroy(), super.destroy(t);
    }
    update(t) {
      this.pause || (this.plugins.update(t), this.lastViewport && (this.lastViewport.x !== this.x || this.lastViewport.y !== this.y ? this.moving = true : this.moving && (this.emit("moved-end", this), this.moving = false), this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y ? this.zooming = true : this.zooming && (this.emit("zoomed-end", this), this.zooming = false)), this.forceHitArea || (this._hitAreaDefault = new Ht(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight), this.hitArea = this._hitAreaDefault), this._dirty = this._dirty || !this.lastViewport || this.lastViewport.x !== this.x || this.lastViewport.y !== this.y || this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y, this.lastViewport = {
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
      return new Ht(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight);
    }
    toWorld(t, e) {
      return arguments.length === 2 ? this.toLocal(new Tt(t, e)) : this.toLocal(t);
    }
    toScreen(t, e) {
      return arguments.length === 2 ? this.toGlobal(new Tt(t, e)) : this.toGlobal(t);
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
      return new Tt(this.worldScreenWidth / 2 - this.x / this.scale.x, this.worldScreenHeight / 2 - this.y / this.scale.y);
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
      return new Tt(-this.x / this.scale.x, -this.y / this.scale.y);
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
      return this.plugins.add("snap-zoom", new Lx(this, t)), this;
    }
    OOB() {
      return {
        left: this.left < 0,
        right: this.right > this.worldWidth,
        top: this.top < 0,
        bottom: this.bottom > this.worldHeight,
        cornerPoint: new Tt(this.worldWidth * this.scale.x - this.screenWidth, this.worldHeight * this.scale.y - this.screenHeight)
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
      t ? (this._forceHitArea = t, this.hitArea = t) : (this._forceHitArea = null, this.hitArea = new Ht(0, 0, this.worldWidth, this.worldHeight));
    }
    drag(t) {
      return this.plugins.add("drag", new vx(this, t)), this;
    }
    clamp(t) {
      return this.plugins.add("clamp", new gx(this, t)), this;
    }
    decelerate(t) {
      return this.plugins.add("decelerate", new _x(this, t)), this;
    }
    bounce(t) {
      return this.plugins.add("bounce", new fx(this, t)), this;
    }
    pinch(t) {
      return this.plugins.add("pinch", new Mx(this, t)), this;
    }
    snap(t, e, s) {
      return this.plugins.add("snap", new Ix(this, t, e, s)), this;
    }
    follow(t, e) {
      return this.plugins.add("follow", new Sx(this, t, e)), this;
    }
    wheel(t) {
      return this.plugins.add("wheel", new Bx(this, t)), this;
    }
    animate(t) {
      return this.plugins.add("animate", new ux(this, t)), this;
    }
    clampZoom(t) {
      return this.plugins.add("clamp-zoom", new xx(this, t)), this;
    }
    mouseEdges(t) {
      return this.plugins.add("mouse-edges", new Ex(this, t)), this;
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
  const Nx = 32, Wo = /* @__PURE__ */ new Set([
    "transport-belt",
    "fast-transport-belt",
    "express-transport-belt"
  ]), Ri = /* @__PURE__ */ new Set([
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt"
  ]), to = /* @__PURE__ */ new Set([
    "splitter",
    "fast-splitter",
    "express-splitter"
  ]), Fx = /* @__PURE__ */ new Set([
    "inserter",
    "fast-inserter",
    "long-handed-inserter"
  ]), Wx = /* @__PURE__ */ new Set([
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
  ]), tc = {
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
  function Qe(n, t) {
    return `${n},${t}`;
  }
  function sr(n) {
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
  function Li(n, t, e, s, i, r) {
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
  const Gx = 9;
  function zx(n) {
    const t = {
      nodes: /* @__PURE__ */ new Map(),
      outEdges: /* @__PURE__ */ new Map(),
      inEdges: /* @__PURE__ */ new Map(),
      tileToAnchor: /* @__PURE__ */ new Map(),
      entityMap: /* @__PURE__ */ new Map()
    };
    for (const e of n.entities) t.entityMap.set(Qe(e.x ?? 0, e.y ?? 0), e);
    for (const e of n.entities) {
      if (!Wo.has(e.name) && !Ri.has(e.name) && !to.has(e.name)) continue;
      const s = e.x ?? 0, i = e.y ?? 0, r = Qe(s, i);
      if (t.nodes.set(r, e), t.tileToAnchor.set(r, r), to.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South", a = s + (o ? 1 : 0), l = i + (o ? 0 : 1);
        t.tileToAnchor.set(Qe(a, l), r);
      }
    }
    for (const [e, s] of t.nodes) {
      const i = s.x ?? 0, r = s.y ?? 0, o = s.direction ?? "North", [a, l] = sr(o);
      if (Wo.has(s.name)) {
        const c = t.tileToAnchor.get(Qe(i + a, r + l));
        if (c !== void 0 && c !== e) {
          const h = t.nodes.get(c), [d, u] = sr(h.direction), p = a * u - l * d;
          Li(t, e, c, "both", p > 0, false);
        }
      } else if (Ri.has(s.name)) if (s.io_type === "input") for (let c = 1; c <= Gx; c++) {
        const h = t.entityMap.get(Qe(i + a * c, r + l * c));
        if (h) {
          if (Ri.has(h.name) && h.name === s.name && h.io_type === "input" && h.direction === o) break;
          if (Ri.has(h.name) && h.name === s.name && h.io_type === "output" && h.direction === o) {
            const d = t.tileToAnchor.get(Qe(h.x ?? 0, h.y ?? 0));
            d !== void 0 && Li(t, e, d, "both", false, false);
            break;
          }
        }
      }
      else {
        const c = t.tileToAnchor.get(Qe(i + a, r + l));
        c !== void 0 && c !== e && Li(t, e, c, "both", false, false);
      }
      else if (to.has(s.name)) {
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
          const f = t.tileToAnchor.get(Qe(u, p));
          f !== void 0 && f !== e && Li(t, e, f, "both", false, true);
        }
      }
    }
    return t;
  }
  function Dx(n, t) {
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
  function Hx(n, t) {
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
        const c = Qe(r + a, o + l), h = t.get(c);
        h && Fx.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  function Ux(n, t) {
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
        const c = Qe(r + a, o + l), h = t.get(c);
        h && Wx.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  const Go = {
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
  function ec(n, t) {
    const e = t.nodes.get(n);
    if (!e || !Wo.has(e.name)) return null;
    const s = e.direction ?? "North";
    for (const i of t.inEdges.get(n) ?? []) {
      const r = t.nodes.get(i.from);
      if (!r) continue;
      const o = r.direction ?? "North";
      if (`${o}_${s}` in Go) return {
        inDir: o,
        outDir: s
      };
    }
    return null;
  }
  function jx(n, t, e, s, i, r, o) {
    const a = s - t, l = i - e, c = Math.sqrt(a * a + l * l);
    if (c === 0) return;
    const h = a / c, d = l / c;
    let u = 0, p = true;
    for (; u < c; ) {
      const f = Math.min(p ? r : o, c - u);
      p && n.moveTo(t + h * u, e + d * u).lineTo(t + h * (u + f), e + d * (u + f)).stroke(), u += f, p = !p;
    }
  }
  function Vx(n, t, e, s, i, r, o, a, l) {
    let c = o ? i - r : r - i;
    c < 0 && (c += 2 * Math.PI);
    let h = 0, d = true;
    for (; h < c; ) {
      const u = Math.min(d ? a : l, c - h);
      if (d) {
        const p = o ? i - h : i + h, f = o ? p - u : p + u, m = t + s * Math.cos(p), g = e + s * Math.sin(p);
        n.moveTo(m, g).arc(t, e, s, p, f, o).stroke();
      }
      h += u, d = !d;
    }
  }
  function Yx(n, t, e, s, i) {
    const r = Nx, o = r / 2;
    for (const l of e) {
      if (t.has(l)) continue;
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = tc[c.name] ?? 14733424, [p, f] = sr(c.direction), m = h + r / 2, g = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.05
      }), n.setStrokeStyle({
        width: 1.5,
        color: u,
        alpha: 0.28,
        cap: "round"
      });
      const y = ec(l, i);
      if (y) {
        const w = Go[`${y.inDir}_${y.outDir}`], x = w.cx(h, d, r), _ = w.cy(h, d, r);
        Vx(n, x, _, o, w.startAngle, w.endAngle, w.anticlockwise, 5 / o, 3 / o);
      } else jx(n, m - p * r * 0.45, g - f * r * 0.45, m + p * r * 0.45, g + f * r * 0.45, 5, 3);
    }
    for (const l of t) {
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = tc[c.name] ?? 14733424, [p, f] = sr(c.direction), m = h + r / 2, g = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.2
      }), n.setStrokeStyle({
        width: 2,
        color: u,
        alpha: 0.85,
        cap: "round"
      });
      const y = ec(l, i);
      if (y) {
        const w = Go[`${y.inDir}_${y.outDir}`], x = w.cx(h, d, r), _ = w.cy(h, d, r), v = x + o * Math.cos(w.startAngle), b = _ + o * Math.sin(w.startAngle);
        n.moveTo(v, b).arc(x, _, o, w.startAngle, w.endAngle, w.anticlockwise).stroke();
      } else n.moveTo(m - p * r * 0.45, g - f * r * 0.45).lineTo(m + p * r * 0.45, g + f * r * 0.45).stroke();
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
  let Xx, qx, $n, Id, Rd, Kx, nc, sc, Jx, Zx, Qx;
  S = 32;
  Xx = {
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
  };
  qx = 4872810;
  $n = {
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
  };
  Id = {
    inserter: 6983230,
    "fast-inserter": 4886736,
    "long-handed-inserter": 13647936
  };
  Rd = 9079434;
  Kx = 6974058;
  nc = 2039583;
  sc = 12623920;
  Jx = 2762e3;
  Zx = 0.35;
  Qx = {
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
  function t0(n, t) {
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = 0.21 * e + 0.72 * s + 0.07 * i, o = Math.round(r + (e - r) * t), a = Math.round(r + (s - r) * t), l = Math.round(r + (i - r) * t);
    return o << 16 | a << 8 | l;
  }
  const ic = Object.fromEntries(Object.entries(Qx).map(([n, t]) => [
    n,
    t0(t, Zx)
  ]));
  function e0(n, t, e) {
    const s = t * Math.min(e, 1 - e), i = (r) => {
      const o = (r + n * 12) % 12;
      return Math.round((e - s * Math.max(-1, Math.min(o - 3, 9 - o, 1))) * 255);
    };
    return i(0) << 16 | i(8) << 8 | i(4);
  }
  let Ld = true;
  function $i(n) {
    Ld = n;
  }
  let zo = /* @__PURE__ */ new Map();
  function n0(n) {
    return zo.get(n);
  }
  function $d(n) {
    zo = /* @__PURE__ */ new Map();
    for (const t of n) zo.set(t.recipe, {
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
  function Rn(n) {
    if (!Ld) return 7829367;
    if (!n) return 6710886;
    if (n in ic) return ic[n];
    let t = 0;
    for (let s = 0; s < n.length; s++) t = (t << 5) - t + n.charCodeAt(s) | 0;
    const e = Math.abs(t) % 30 * 12;
    return e0(e / 360, 0.2, 0.48);
  }
  const Ne = {
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
  function fe(n) {
    return n.split("-").map((t) => t.charAt(0).toUpperCase() + t.slice(1)).join(" ");
  }
  let $e, br, sn, ie, ai, fa;
  $e = new Set(Object.keys(Ne));
  br = new Set(Object.keys(Id));
  sn = new Set(Object.keys($n).filter((n) => !n.includes("underground") && !n.includes("splitter")));
  pe = new Set(Object.keys($n).filter((n) => n.includes("underground")));
  ie = new Set(Object.keys($n).filter((n) => n.includes("splitter")));
  ai = /* @__PURE__ */ new Set([
    "pipe",
    "pipe-to-ground"
  ]);
  fa = /* @__PURE__ */ new Set([
    "medium-electric-pole",
    "small-electric-pole"
  ]);
  function mi(n) {
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
  function In(n) {
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
  function ma(n) {
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
  function Bd(n, t) {
    const e = n.direction ?? "North", [s, i] = In(e);
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
      if (!d || !(sn.has(d.name) || pe.has(d.name) && d.io_type === "output" || ie.has(d.name))) continue;
      const [p, f] = In(d.direction), m = ie.has(d.name) ? c : d.x ?? 0, g = ie.has(d.name) ? h : d.y ?? 0;
      if (!(m + p !== (n.x ?? 0) || g + f !== (n.y ?? 0))) if (d.direction === e) r = true;
      else {
        const y = p * i - f * s;
        y !== 0 && (o = {
          turn: y > 0 ? "cw" : "ccw"
        });
      }
    }
    return o && !r ? o : null;
  }
  function rc(n, t) {
    const e = Math.round((n >> 16 & 255) * t), s = Math.round((n >> 8 & 255) * t), i = Math.round((n & 255) * t);
    return e << 16 | s << 8 | i;
  }
  const li = 3, Od = 3815994, ga = 5592405, ya = 0.9, Bi = S * (1 - ya) / 2, Nd = S * ya;
  function xa(n, t) {
    const e = new ft(), s = S, i = Nd, [, r] = $n[n.name] ?? [
      11046960,
      14733424
    ], o = Rn(n.carries);
    if (t) s0(e, s, r, n.direction, t, o);
    else {
      e.rect(Bi, Bi, i, i).fill(Od), e.setStrokeStyle({
        width: 1,
        color: ga,
        alignment: 0
      }), e.rect(Bi, Bi, i, i).stroke();
      const a = new ft();
      a.x = s / 2, a.y = s / 2, a.rotation = mi(n.direction), a.rect(-i / 2, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(1, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(-1, -i / 2, 2, i).fill(657930), i0(a, i, r), e.addChild(a);
    }
    return e;
  }
  function s0(n, t, e, s, i, r) {
    const o = new ft();
    o.x = t / 2, o.y = t / 2, o.rotation = mi(s), o.scale.set(ya);
    const a = t / 2, c = (i.turn === "cw" ? 1 : -1) * a, h = -a, d = i.turn === "ccw" ? 0 : Math.PI / 2, u = i.turn === "ccw" ? Math.PI / 2 : Math.PI;
    o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().fill(Od), o.setStrokeStyle({
      width: 1,
      color: ga,
      alignment: 0
    }), o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().stroke();
    const p = t * 0.5, f = 1.5;
    o.moveTo(c, h).arc(c, h, p - f, d, u, false).closePath().fill({
      color: r,
      alpha: 0.45
    });
    const m = Math.cos(d), g = Math.sin(d), y = Math.cos(u), w = Math.sin(u);
    o.moveTo(c + (p + f) * m, h + (p + f) * g).lineTo(c + t * m, h + t * g).arc(c, h, t, d, u, false).lineTo(c + (p + f) * y, h + (p + f) * w).arc(c, h, p + f, u, d, true).closePath().fill({
      color: r,
      alpha: 0.45
    });
    const x = t * 0.22, _ = Math.max(1, t * 0.07), v = t * 0.5, b = x / v, C = v + x, T = v - x;
    o.setStrokeStyle({
      width: _,
      color: e,
      cap: "round",
      join: "round"
    });
    const L = Math.PI / 2, A = i.turn === "cw" ? Math.PI : 0;
    for (const E of [
      0.6
    ]) {
      const I = L + E * (A - L), V = i.turn === "cw" ? I - b : I + b, G = c + v * Math.cos(I), F = h + v * Math.sin(I), $ = c + C * Math.cos(V), z = h + C * Math.sin(V), J = c + T * Math.cos(V), X = h + T * Math.sin(V);
      o.moveTo($, z).lineTo(G, F).lineTo(J, X).stroke();
    }
    n.addChild(o);
  }
  function i0(n, t, e) {
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
  function Fd(n) {
    const t = new ft(), e = S, [, s] = $n[n.name] ?? [
      11046960,
      14733424
    ], i = n.io_type === "input", r = e / 2, o = i ? 1 : -1, a = new ft();
    a.x = r, a.y = r, a.rotation = mi(n.direction);
    const l = Rn(n.carries), c = Nd / 2, h = e * 0.25, d = o * r, u = 0;
    a.moveTo(-c, d).lineTo(c, d).lineTo(h, u).lineTo(-h, u).closePath().fill({
      color: l,
      alpha: 0.7
    }), a.setStrokeStyle({
      width: 1,
      color: ga,
      alpha: 0.8
    }), a.moveTo(-c, d).lineTo(-h, u).lineTo(h, u).lineTo(c, d).stroke();
    const p = e * 0.38, f = e * 0.3, m = o * e * 0.22, g = m - f / 2, y = m + f / 2;
    return a.moveTo(0, g).lineTo(p / 2, y).lineTo(-p / 2, y).closePath().fill(s), t.addChild(a), t;
  }
  function Wd(n) {
    const t = new ft(), [e, s] = $n[n.name] ?? [
      11046960,
      14733424
    ], i = n.direction === "North" || n.direction === "South", r = i ? S * 2 - 1 : S - 1, o = i ? S - 1 : S * 2 - 1, a = i ? r / 2 : o / 2, l = Math.max(2, Math.min(r, o) * 0.18);
    t.roundRect(0, 0, r, o, li).fill(e), t.roundRect(0, 0, r, o, li).fill({
      color: Rn(n.carries),
      alpha: 0.3
    }), i ? t.rect(a - l / 2, 0, l, o).fill(rc(e, 0.5)) : t.rect(0, a - l / 2, r, l).fill(rc(e, 0.5));
    const c = mi(n.direction), h = a * 0.25, d = Math.max(1, a * 0.12);
    for (let u = 0; u < 2; u++) {
      const p = i ? a * u + a / 2 : r / 2, f = i ? o / 2 : a * u + a / 2, m = new ft();
      m.x = p, m.y = f, m.rotation = c, m.setStrokeStyle({
        width: d,
        color: s,
        cap: "round"
      }), m.moveTo(-h, h * 0.5).lineTo(0, -h * 0.5).lineTo(h, h * 0.5).stroke(), t.addChild(m);
    }
    return t;
  }
  function Gd(n) {
    const t = new ft(), e = S - 1, s = n.carries ? Rn(n.carries) : Id[n.name] ?? 6983230;
    t.roundRect(0, 0, e, e, li).fill(2767402);
    const i = new ft();
    i.x = e / 2, i.y = e / 2, i.rotation = mi(n.direction), i.circle(0, e * 0.2, e * 0.15).fill(4473924);
    const r = Math.max(1.5, e * 0.12);
    i.setStrokeStyle({
      width: r,
      color: s,
      cap: "round"
    }), i.moveTo(0, e * 0.2).lineTo(0, -e * 0.35).stroke();
    const o = -e * 0.35, a = e * 0.18;
    return i.moveTo(-a, o - a * 0.6).lineTo(0, o).lineTo(a, o - a * 0.6).stroke(), t.addChild(i), t;
  }
  const ba = 1, _a = 2, wa = 4, va = 8;
  function r0(n) {
    const [t, e] = In(n.direction);
    return [
      -t,
      -e
    ];
  }
  function zd(n, t, e) {
    if (n.name === "pipe") return true;
    if (n.name === "pipe-to-ground") {
      const [s, i] = r0(n);
      return -t === s && -e === i;
    }
    return false;
  }
  function Dd(n, t) {
    const e = new ft(), s = S - 1, i = n.name === "pipe-to-ground", r = i ? Kx : Rd;
    e.roundRect(0, 0, s, s, li).fill(nc);
    const o = s / 2, a = s / 2, l = Math.max(2, s * 0.4);
    if (i) {
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      });
      const [c, h] = In(n.direction);
      e.moveTo(o, a).lineTo(o - c * s / 2, a - h * s / 2).stroke(), e.circle(o, a, l * 0.4).fill(r), e.circle(o, a, l * 0.25).fill(nc);
    } else if (t === 0) e.circle(o, a, l * 0.4).fill(r);
    else {
      const c = !!(t & ba), h = !!(t & _a), d = !!(t & wa), u = !!(t & va), p = (c ? 1 : 0) + (h ? 1 : 0) + (d ? 1 : 0) + (u ? 1 : 0);
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      }), p === 1 ? (c ? e.moveTo(o, a).lineTo(o, 0).stroke() : h ? e.moveTo(o, a).lineTo(s, a).stroke() : d ? e.moveTo(o, a).lineTo(o, s).stroke() : e.moveTo(o, a).lineTo(0, a).stroke(), e.circle(o, a, l * 0.4).fill(r)) : c && d && !h && !u ? e.moveTo(o, 0).lineTo(o, s).stroke() : h && u && !c && !d ? e.moveTo(0, a).lineTo(s, a).stroke() : p === 2 ? c && h ? e.moveTo(o, 0).quadraticCurveTo(o, a, s, a).stroke() : h && d ? e.moveTo(s, a).quadraticCurveTo(o, a, o, s).stroke() : d && u ? e.moveTo(o, s).quadraticCurveTo(o, a, 0, a).stroke() : e.moveTo(0, a).quadraticCurveTo(o, a, o, 0).stroke() : p === 3 ? u ? d ? h ? (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, s).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(0, a).stroke()) : (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, 0).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(s, a).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(0, a).lineTo(s, a).stroke());
    }
    return e;
  }
  function Hd() {
    const n = new ft(), t = S - 1;
    n.roundRect(0, 0, t, t, li).fill(Jx);
    const e = t / 2, s = t / 2, i = t * 0.38, r = Math.max(1.5, t * 0.2);
    return n.rect(e - r / 2, s - i, r, i * 2).fill(sc), n.rect(e - i, s - r / 2, i * 2, r).fill(sc), n.circle(e, s, r * 0.6).fill(14729280), n;
  }
  function Ud(n) {
    const t = new ft(), [e, s] = Ne[n.name] ?? [
      1,
      1
    ], i = e * S - 1, r = s * S - 1;
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
      const f = Math.sqrt((u - h) ** 2 + (p - d) ** 2), m = (u - h) / f, g = (p - d) / f;
      let y = 0;
      for (; y < f; ) {
        const w = Math.min(y + o, f);
        t.moveTo(h + m * y, d + g * y).lineTo(h + m * w, d + g * w).stroke(), y = w + a;
      }
    }
    const l = 1.8, c = oc(`/spaghettio/entity-frames/${n.name}.png`);
    if (c) {
      const h = new Ye(c), d = S / o0;
      h.scale.set(d * l), h.x = -i * (l - 1) / 2, h.y = -r * (l - 1) / 2, t.addChild(h);
    } else {
      const h = oc(`/spaghettio/icons/${n.name}.png`);
      if (h) {
        const d = new Ye(h), u = Math.min(i, r) * 0.8 * l;
        d.width = u, d.height = u, d.x = (i - u) / 2, d.y = (r - u) / 2, t.addChild(d);
      } else {
        const d = Xx[n.name] ?? qx;
        t.roundRect(2, 2, i - 4, r - 4, 3).fill({
          color: d,
          alpha: 0.5
        });
      }
    }
    return t;
  }
  function jd() {
    const n = new ft(), t = S - 1;
    return n.rect(0, 0, t, t).fill(4872810), n.setStrokeStyle({
      width: 1,
      color: 0,
      alpha: 0.4
    }), n.rect(0, 0, t, t).stroke(), n;
  }
  const o0 = 64;
  async function a0(n) {
    const t = "/spaghettio/", e = [
      ...n.map((s) => `${t}icons/${s}.png`),
      ...n.map((s) => `${t}entity-frames/${s}.png`)
    ];
    await Promise.allSettled(e.map((s) => je.load(s)));
  }
  async function Vd(n) {
    const t = "/spaghettio/";
    await Promise.allSettled(n.map((e) => je.load(`${t}icons/${e}.png`)));
  }
  function l0(n) {
    const t = /* @__PURE__ */ new Set();
    for (const e of n) e.carries && t.add(e.carries);
    return Array.from(t);
  }
  function oc(n) {
    return ne.has(n) ? je.get(n) ?? null : null;
  }
  const c0 = {
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
  function Yd(n) {
    const t = c0[n.name];
    if (!t) return [];
    const e = n.mirror ?? false;
    return t.filter(([, , , s]) => s === "always" || s === "default" && !e || s === "mirror" && e);
  }
  function Xd(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of Yd(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  _r = function() {
    return {
      tileMap: /* @__PURE__ */ new Map(),
      machineByTile: /* @__PURE__ */ new Map()
    };
  };
  ir = function(n, t) {
    const e = n.x ?? 0, s = n.y ?? 0;
    if (t.tileMap.set(`${e},${s}`, n), ie.has(n.name)) {
      const [i, r] = ma(n.direction);
      t.tileMap.set(`${e + i},${s + r}`, n);
    }
    if ($e.has(n.name)) {
      const [i, r] = Ne[n.name] ?? [
        1,
        1
      ];
      for (let o = 0; o < r; o++) for (let a = 0; a < i; a++) t.machineByTile.set(`${e + a},${s + o}`, n);
    }
  };
  Fn = function(n, t) {
    let e;
    if (sn.has(n.name)) e = xa(n, Bd(n, t.tileMap));
    else if (pe.has(n.name)) e = Fd(n);
    else if (ie.has(n.name)) e = Wd(n);
    else if (br.has(n.name)) e = Gd(n);
    else if (ai.has(n.name)) {
      let s = 0;
      if (n.name === "pipe") {
        const i = n.x ?? 0, r = n.y ?? 0;
        for (const [o, a, l] of [
          [
            0,
            -1,
            ba
          ],
          [
            1,
            0,
            _a
          ],
          [
            0,
            1,
            wa
          ],
          [
            -1,
            0,
            va
          ]
        ]) {
          const c = `${i + o},${r + a}`, h = t.tileMap.get(c);
          if (h && zd(h, o, a)) {
            s |= l;
            continue;
          }
          const d = t.machineByTile.get(c);
          d && Xd(i, r, d) && (s |= l);
        }
      }
      e = Dd(n, s);
    } else fa.has(n.name) ? e = Hd() : $e.has(n.name) ? e = Ud(n) : e = jd();
    return e.x = (n.x ?? 0) * S, e.y = (n.y ?? 0) * S, e;
  };
  gv = function(n, t, e = 8) {
    if (!pe.has(n.name) || n.io_type !== "input") return null;
    const [s, i] = In(n.direction), r = n.x ?? 0, o = n.y ?? 0;
    for (let a = 1; a <= e; a++) {
      const l = t.get(`${r + s * a},${o + i * a}`);
      if (l) {
        if (pe.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "input") return null;
        if (pe.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "output") {
          const [c] = $n[n.name] ?? [
            11046960,
            14733424
          ], h = new ft(), d = Math.abs(s) > 0;
          for (let u = 1; u < a; u++) {
            const p = (r + s * u) * S, f = (o + i * u) * S;
            d ? h.rect(p, f + S * 0.25, S, S * 0.5).fill({
              color: c,
              alpha: 0.25
            }) : h.rect(p + S * 0.25, f, S * 0.5, S).fill({
              color: c,
              alpha: 0.25
            });
          }
          return h;
        }
      }
    }
    return null;
  };
  ti = function(n, t, e, s, i, r) {
    t.removeChildren();
    const o = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (o.set(`${f.x ?? 0},${f.y ?? 0}`, f), ie.has(f.name)) {
      const [m, g] = ma(f.direction);
      o.set(`${(f.x ?? 0) + m},${(f.y ?? 0) + g}`, f);
    }
    if (r) for (const f of r) {
      const m = `${f.x ?? 0},${f.y ?? 0}`;
      o.has(m) || o.set(m, f);
    }
    const a = /* @__PURE__ */ new Map();
    for (const f of n.entities) if ($e.has(f.name)) {
      const [m, g] = Ne[f.name] ?? [
        1,
        1
      ], y = f.x ?? 0, w = f.y ?? 0;
      for (let x = 0; x < g; x++) for (let _ = 0; _ < m; _++) a.set(`${y + _},${w + x}`, f);
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const g of n.entities) pe.has(g.name) && f.set(`${g.x ?? 0},${g.y ?? 0}`, g);
      const m = 8;
      for (const g of n.entities) {
        if (!pe.has(g.name) || g.io_type !== "input") continue;
        const [y, w] = In(g.direction), x = g.x ?? 0, _ = g.y ?? 0;
        for (let v = 1; v <= m; v++) {
          const b = f.get(`${x + y * v},${_ + w * v}`);
          if (b) {
            if (pe.has(b.name) && b.name === g.name && b.direction === g.direction && b.io_type === "input") break;
            if (pe.has(b.name) && b.name === g.name && b.direction === g.direction && b.io_type === "output") {
              const [C] = $n[g.name] ?? [
                11046960,
                14733424
              ], T = new ft(), L = Math.abs(y) > 0;
              for (let A = 1; A < v; A++) {
                const E = (x + y * A) * S, I = (_ + w * A) * S;
                L ? T.rect(E, I + S * 0.25, S, S * 0.5).fill({
                  color: C,
                  alpha: 0.25
                }) : T.rect(E + S * 0.25, I, S * 0.5, S).fill({
                  color: C,
                  alpha: 0.25
                });
              }
              t.addChild(T);
              break;
            }
          }
        }
      }
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const g of n.entities) g.name === "pipe-to-ground" && f.set(`${g.x ?? 0},${g.y ?? 0}`, g);
      const m = 10;
      for (const g of n.entities) {
        if (g.name !== "pipe-to-ground" || g.io_type !== "input") continue;
        const [y, w] = In(g.direction), x = g.x ?? 0, _ = g.y ?? 0;
        for (let v = 2; v <= m; v++) {
          const b = f.get(`${x + y * v},${_ + w * v}`);
          if (!b) continue;
          const [C, T] = In(b.direction);
          if (b.io_type !== "output" || C !== -y || T !== -w) break;
          const L = new ft();
          L.setStrokeStyle({
            width: 2,
            color: Rd,
            alpha: 0.55,
            cap: "round"
          });
          const A = (x + 0.5 + y * 0.5) * S, E = (_ + 0.5 + w * 0.5) * S, I = (v - 1) * S, V = 5, G = 3;
          let F = 0;
          for (; F < I; ) {
            const $ = Math.min(F + V, I);
            L.moveTo(A + y * F, E + w * F).lineTo(A + y * $, E + w * $).stroke(), F = $ + G;
          }
          t.addChild(L);
          break;
        }
      }
    }
    const l = /* @__PURE__ */ new Map(), c = [], h = /* @__PURE__ */ new Map();
    for (const f of n.entities) {
      let m;
      if (sn.has(f.name)) m = xa(f, Bd(f, o));
      else if (pe.has(f.name)) m = Fd(f);
      else if (ie.has(f.name)) m = Wd(f);
      else if (br.has(f.name)) m = Gd(f);
      else if (ai.has(f.name)) {
        let y = 0;
        if (f.name === "pipe") {
          const w = f.x ?? 0, x = f.y ?? 0;
          for (const [_, v, b] of [
            [
              0,
              -1,
              ba
            ],
            [
              1,
              0,
              _a
            ],
            [
              0,
              1,
              wa
            ],
            [
              -1,
              0,
              va
            ]
          ]) {
            if (v === -1 && x + v < 0) {
              y |= b;
              continue;
            }
            const C = `${w + _},${x + v}`, T = o.get(C);
            if (T && zd(T, _, v)) {
              y |= b;
              continue;
            }
            const L = a.get(C);
            L && Xd(w, x, L) && (y |= b);
          }
        }
        m = Dd(f, y);
      } else fa.has(f.name) ? m = Hd() : $e.has(f.name) ? m = Ud(f) : m = jd();
      m.x = (f.x ?? 0) * S, m.y = (f.y ?? 0) * S, s && (m.eventMode = "static", m.cursor = "pointer", m.on("click", () => s(f)));
      const g = ac(f);
      g && (l.has(g) || l.set(g, []), l.get(g).push(m)), h.set(m, `${f.x ?? 0},${f.y ?? 0}`), c.push(m), t.addChild(m), i == null ? void 0 : i(f, [
        m
      ]);
    }
    const d = zx(n);
    let u = null;
    function p() {
      u && (t.removeChild(u), u.destroy(), u = null);
      for (const f of c) f.alpha = 1;
    }
    return {
      highlightItem(f) {
        if (p(), !f) return;
        const m = l.get(f);
        if (!m || m.length === 0) return;
        const g = new Set(m);
        for (const y of c) y.alpha = g.has(y) ? 1 : 0.15;
      },
      highlightBeltNetwork(f) {
        if (p(), !f) return;
        const m = `${f.x ?? 0},${f.y ?? 0}`, g = d.tileToAnchor.get(m) ?? m;
        if (!d.nodes.has(g)) return;
        const { downstream: y, upstream: w } = Dx(g, d), x = /* @__PURE__ */ new Set([
          ...y,
          ...w
        ]), _ = Hx(x, d.entityMap), v = Ux(_, d.entityMap);
        for (const b of c) {
          const C = h.get(b);
          if (!C) {
            b.alpha = 0.15;
            continue;
          }
          x.has(C) ? b.alpha = 0.5 : _.has(C) ? b.alpha = 0.9 : v.has(C) ? b.alpha = 0.75 : b.alpha = 0.15;
        }
        u = new ft(), Yx(u, y, w, g, d), t.addChild(u);
      },
      clearHighlight() {
        p();
      },
      chainKey: ac
    };
  };
  function ac(n) {
    return n.carries ? n.carries : n.recipe ? n.recipe : null;
  }
  const ci = 4096, gt = 128, tn = ci / gt;
  let kn = null;
  const hi = /* @__PURE__ */ new Map();
  let nn = 0, di = null;
  function h0(n) {
    di = n;
  }
  function hn(n, t, e, s) {
    const i = hi.get(n);
    if (i) return i;
    if (!di) return console.warn("[atlas] getEntityTexture called before initAtlas; returning blank texture"), Ct.EMPTY;
    kn || (kn = gr.create({
      width: ci,
      height: ci
    })), nn >= tn * tn && (console.warn("[atlas] atlas is full \u2014 variant will reuse slot 0:", n), nn = 0);
    const r = nn % tn, o = Math.floor(nn / tn), a = r * gt, l = o * gt;
    nn++;
    const c = new ft();
    s(c);
    const h = new vt(1, 0, 0, 1, a, l);
    di.render({
      container: c,
      target: kn,
      transform: h,
      clear: false
    }), c.destroy({
      children: true
    });
    const d = new Ht(a, l, gt, gt), u = new Ct({
      source: kn.source,
      frame: d
    });
    return hi.set(n, u), u;
  }
  function lc(n, t, e, s) {
    const i = hi.get(n);
    if (i) return i;
    if (!di) return console.warn("[atlas] getMultiCellTexture called before initAtlas; returning blank texture"), Ct.EMPTY;
    kn || (kn = gr.create({
      width: ci,
      height: ci
    }));
    const r = nn % tn;
    r + t > tn && (nn += tn - r);
    const a = nn % tn, l = Math.floor(nn / tn), c = a * gt, h = l * gt;
    nn = (l + e) * tn;
    const d = t * gt, u = e * gt, p = new ft();
    s(p, d, u);
    const f = new vt(1, 0, 0, 1, c, h);
    di.render({
      container: p,
      target: kn,
      transform: f,
      clear: false
    }), p.destroy({
      children: true
    });
    const m = new Ht(c, h, d, u), g = new Ct({
      source: kn.source,
      frame: m
    });
    return hi.set(n, g), g;
  }
  function qd(n) {
    const t = `icon:${n}`, e = hi.get(t);
    if (e) return e;
    const s = `/spaghettio/icons/${n}.png`;
    if (ne.has(s)) {
      const r = je.get(s);
      if (r) return hn(t, gt, gt, (a) => {
        const c = gt - 16;
        a.rect(8, 8, c, c).fill({
          texture: r
        });
      });
    }
    const i = Rn(n);
    return hn(t, gt, gt, (r) => {
      const o = gt / 2, a = gt / 2;
      r.circle(o, a, 7).fill({
        color: i,
        alpha: 0.85
      });
    });
  }
  function d0(n, t, e = "straight") {
    return `belt:${n}:${t}:${e}`;
  }
  function u0(n) {
    return `pipe:${n}`;
  }
  function p0(n, t, e) {
    return `ugbelt:${n}:${t}:${e}`;
  }
  function f0(n, t) {
    return `splitter:${n}:${t}`;
  }
  function m0(n, t) {
    return `inserter:${n}:${t}`;
  }
  function g0(n) {
    return `machine:${n}`;
  }
  function y0(n) {
    return `pole:${n}`;
  }
  function x0(n) {
    return `ptg:${n}`;
  }
  const en = 3200;
  let Kd = null, Jd = null, Zd = null;
  function Ve() {
    Kd == null ? void 0 : Kd();
  }
  function wr() {
    Jd == null ? void 0 : Jd();
  }
  function qn() {
    Zd == null ? void 0 : Zd();
  }
  async function b0(n) {
    const t = new ea();
    await t.init({
      resizeTo: n,
      background: 1973790,
      antialias: true,
      autoStart: false,
      sharedTicker: false
    }), h0(t.renderer), t.ticker.add(() => t.render(), null, ni.LOW), n.appendChild(t.canvas), t.canvas.addEventListener("contextmenu", (h) => h.preventDefault());
    const e = new Pd({
      screenWidth: n.clientWidth,
      screenHeight: n.clientHeight,
      worldWidth: en,
      worldHeight: en,
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
    Kd = r, Jd = o, Zd = a, e.on("moved", r), e.on("zoomed", r);
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
      e.resize(n.clientWidth, n.clientHeight, en, en), r();
    }), r(), {
      app: t,
      viewport: e,
      requestRender: r,
      beginAnimating: o,
      endAnimating: a
    };
  }
  const Oi = 32, cc = 2763306, hc = 3815994;
  function _0(n) {
    const t = new ft();
    return n.addChildAt(t, 0), t;
  }
  function Wn(n, t, e, s = 1) {
    if (n.clear(), t <= 0 || e <= 0) return;
    const i = Math.max(0, Math.min(1, s));
    if (i === 0) return;
    const r = t * Oi, a = e * Oi * i;
    for (let l = 0; l <= t; l++) {
      const c = l * Oi, h = l % 10 === 0;
      n.moveTo(c, 0).lineTo(c, a).stroke({
        width: h ? 1.5 : 1,
        color: h ? hc : cc
      });
    }
    for (let l = 0; l <= e; l++) {
      const c = l * Oi;
      if (c > a) break;
      const h = l % 10 === 0;
      n.moveTo(0, c).lineTo(r, c).stroke({
        width: h ? 1.5 : 1,
        color: h ? hc : cc
      });
    }
  }
  const Xi = {
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
  }, w0 = 8947848;
  function Qd(n) {
    const t = n >> 16 & 255, e = n >> 8 & 255, s = n & 255;
    return `rgb(${t},${e},${s})`;
  }
  const v0 = [
    {
      name: "transport-belt",
      color: Xi["transport-belt"],
      throughput: 15
    },
    {
      name: "fast-transport-belt",
      color: Xi["fast-transport-belt"],
      throughput: 30
    },
    {
      name: "express-transport-belt",
      color: Xi["express-transport-belt"],
      throughput: 45
    }
  ];
  function tu(n) {
    for (const t of v0) if (n <= t.throughput) return t;
    return null;
  }
  const C0 = 240, dc = 80, Ca = 180, S0 = 60, uc = 100, pc = 100, fc = "production-graph";
  function T0(n) {
    return n <= 15 ? 13153632 : n <= 30 ? 14700624 : n <= 45 ? 5284064 : 16711816;
  }
  const mc = new Be({
    fontSize: 13,
    fontWeight: "bold",
    fill: 14737632,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: Ca - 12
  }), gc = new Be({
    fontSize: 11,
    fill: 10280190,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: Ca - 12
  }), yc = new Be({
    fontSize: 10,
    fill: 16777215,
    fontFamily: "sans-serif"
  });
  function Ds(n, t) {
    const e = n.getChildByName(fc);
    e && (e.destroy({
      children: true
    }), n.removeChild(e));
    const s = new Ut();
    if (s.label = fc, n.addChild(s), !t || t.machines.length === 0) return s;
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
    for (const [x, _] of l) _.forEach((v, b) => {
      u.set(v.recipe, {
        x: uc + (x + a) * C0,
        y: pc + b * dc,
        w: Ca,
        h: S0,
        machine: v
      });
    });
    const p = /* @__PURE__ */ new Map();
    c.forEach((x, _) => {
      p.set(x, {
        x: uc,
        y: pc + _ * dc,
        w: 140,
        h: 40
      });
    });
    const f = new ft();
    s.addChild(f);
    for (const x of t.machines) {
      const _ = u.get(x.recipe);
      if (_) for (const v of x.inputs) {
        const b = h.get(v.item), C = b ? u.get(b.recipe) : p.get(v.item);
        if (!C) continue;
        const T = T0(v.rate), L = C.x + C.w, A = C.y + C.h * 2 / 3, E = _.x, I = _.y + _.h / 3, V = (L + E) / 2;
        f.moveTo(L, A).lineTo(V, A).lineTo(V, I).lineTo(E, I).stroke({
          color: T,
          width: 2,
          alpha: 0.85
        });
        const G = `${v.rate.toFixed(1)}/s ${v.item}`, F = (L + V) / 2, $ = A - 14, z = cn.measureText(G, yc), J = new ft();
        J.rect(F - 2, $ - 1, z.width + 4, z.height + 2).fill({
          color: 1973790,
          alpha: 0.7
        }), s.addChild(J);
        const X = new Ue({
          text: G,
          style: yc
        });
        X.position.set(F, $), s.addChild(X);
      }
    }
    for (const x of u.values()) {
      const _ = x.machine, v = Xi[_.entity] ?? w0, b = new ft();
      b.rect(x.x, x.y, x.w, x.h).fill({
        color: v,
        alpha: 0.6
      }).stroke({
        color: v,
        width: 2
      }), s.addChild(b);
      const C = new Ue({
        text: `${_.count.toFixed(1)} \xD7 ${_.entity}`,
        style: mc
      });
      C.position.set(x.x + 6, x.y + 6), s.addChild(C);
      const T = new Ue({
        text: _.recipe,
        style: gc
      });
      T.position.set(x.x + 6, x.y + 24), s.addChild(T);
    }
    for (const [x, _] of p) {
      const v = d.get(x), b = v !== void 0 ? `${v.toFixed(1)}/s` : "", C = new ft();
      C.rect(_.x, _.y, _.w, _.h).fill({
        color: 2763306,
        alpha: 0.8
      }).stroke({
        color: 8947848,
        width: 1.5
      }), s.addChild(C);
      const T = new Ue({
        text: b,
        style: mc
      });
      T.position.set(_.x + 6, _.y + 4), s.addChild(T);
      const L = new Ue({
        text: x,
        style: gc
      });
      L.position.set(_.x + 6, _.y + 20), s.addChild(L);
    }
    let m = 1 / 0, g = -1 / 0, y = 1 / 0, w = -1 / 0;
    for (const x of [
      ...u.values(),
      ...p.values()
    ]) x.x < m && (m = x.x), x.x + x.w > g && (g = x.x + x.w), x.y < y && (y = x.y), x.y + x.h > w && (w = x.y + x.h);
    return n.moveCenter((m + g) / 2, (y + w) / 2), s;
  }
  function re(n, t) {
    return `${n},${t}`;
  }
  function we(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function wn(n) {
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
  function me(n, t, e) {
    if (t === e) return;
    let s = n.get(t);
    s || (s = [], n.set(t, s)), s.includes(e) || s.push(e);
    let i = n.get(e);
    i || (i = [], n.set(e, i)), i.includes(t) || i.push(t);
  }
  function E0(n) {
    const t = /* @__PURE__ */ new Map();
    for (const e of n.entities) {
      const s = e.x ?? 0, i = e.y ?? 0, r = we(e);
      if (t.set(re(s, i), r), ie.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South";
        t.set(re(s + (o ? 1 : 0), i + (o ? 0 : 1)), r);
      }
      if ($e.has(e.name)) {
        const [o, a] = Ne[e.name] ?? [
          1,
          1
        ];
        for (let l = 0; l < a; l++) for (let c = 0; c < o; c++) c === 0 && l === 0 || t.set(re(s + c, i + l), r);
      }
    }
    return t;
  }
  const A0 = 9;
  function eu(n) {
    const t = /* @__PURE__ */ new Map(), e = E0(n);
    for (const o of n.entities) {
      const a = we(o);
      t.has(a) || t.set(a, []);
    }
    const s = /* @__PURE__ */ new Map();
    for (const o of n.entities) s.set(re(o.x ?? 0, o.y ?? 0), o);
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
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = wn(o.direction);
      if (sn.has(o.name)) {
        const u = e.get(re(a + h, l + d));
        u && me(t, c, u);
        const p = e.get(re(a - h, l - d));
        p && me(t, c, p);
        for (const [f, m] of i) {
          if (f === h && m === d || f === -h && m === -d) continue;
          const g = s.get(re(a + f, l + m));
          if (!g || !sn.has(g.name) && !pe.has(g.name) && !ie.has(g.name)) continue;
          const [y, w] = wn(g.direction), x = ie.has(g.name) ? a + f : g.x ?? 0, _ = ie.has(g.name) ? l + m : g.y ?? 0;
          x + y === a && _ + w === l && me(t, c, we(g));
        }
      } else if (pe.has(o.name)) {
        if (o.io_type === "input") for (let u = 1; u <= A0; u++) {
          const p = s.get(re(a + h * u, l + d * u));
          if (p) {
            if (pe.has(p.name) && p.name === o.name && p.io_type === "input" && p.direction === o.direction) break;
            if (pe.has(p.name) && p.name === o.name && p.io_type === "output" && p.direction === o.direction) {
              me(t, c, we(p));
              break;
            }
          }
        }
        else {
          const u = e.get(re(a + h, l + d));
          u && me(t, c, u);
        }
        for (const [u, p] of i) {
          const f = s.get(re(a + u, l + p));
          if (!f || !sn.has(f.name) && !ie.has(f.name)) continue;
          const [m, g] = wn(f.direction);
          (f.x ?? 0) + m === a && (f.y ?? 0) + g === l && me(t, c, we(f));
        }
      } else if (ie.has(o.name)) {
        const u = o.direction === "North" || o.direction === "South", [p, f] = u ? [
          1,
          0
        ] : [
          0,
          1
        ];
        for (const [m, g] of [
          [
            0,
            0
          ],
          [
            p,
            f
          ]
        ]) {
          const y = e.get(re(a + m + h, l + g + d));
          y && y !== c && me(t, c, y);
          const w = e.get(re(a + m - h, l + g - d));
          w && w !== c && me(t, c, w);
        }
      }
    }
    for (const o of n.entities) {
      if (!br.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = wn(o.direction), p = o.name === "long-handed-inserter" ? 2 : 1, f = e.get(re(a - h * p, l - d * p)), m = e.get(re(a + h * p, l + d * p));
      f && me(t, c, f), m && me(t, c, m);
    }
    const r = 10;
    for (const o of n.entities) {
      if (!ai.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o);
      if (o.name === "pipe") for (const [h, d] of i) {
        const u = s.get(re(a + h, l + d));
        if (u) {
          if (u.name === "pipe") me(t, c, we(u));
          else if (u.name === "pipe-to-ground") {
            const [p, f] = wn(u.direction);
            h === p && d === f && me(t, c, we(u));
          } else if ($e.has(u.name)) {
            const p = e.get(re(u.x ?? 0, u.y ?? 0));
            p && me(t, c, p);
          }
        }
      }
      else if (o.name === "pipe-to-ground") {
        if (o.io_type === "input") {
          const [p, f] = wn(o.direction);
          for (let m = 2; m <= r; m++) {
            const g = s.get(re(a + p * m, l + f * m));
            if (!g || g.name !== "pipe-to-ground") continue;
            const [y, w] = wn(g.direction);
            if (g.io_type === "output" && y === -p && w === -f) {
              me(t, c, we(g));
              break;
            }
            break;
          }
        }
        const [h, d] = wn(o.direction), u = s.get(re(a - h, l - d));
        u && u.name === "pipe" && me(t, c, we(u));
      }
    }
    for (const o of n.entities) {
      if (!$e.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = Ne[o.name] ?? [
        1,
        1
      ];
      for (let u = 0; u < d; u++) for (let p = 0; p < h; p++) for (const [f, m] of i) {
        const g = a + p + f, y = l + u + m;
        if (g >= a && g < a + h && y >= l && y < l + d) continue;
        const w = s.get(re(g, y));
        w && ai.has(w.name) && me(t, c, we(w));
      }
    }
    return t;
  }
  function nu(n, t) {
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
  const su = 150, k0 = 64, rr = 0.2, M0 = 5, P0 = 100, I0 = 200, R0 = (n) => 1 - Math.pow(1 - n, 3), L0 = (n) => n;
  function Ni() {
    return new rx({
      dynamicProperties: {
        color: true,
        position: false,
        rotation: false,
        vertex: false,
        uvs: false
      }
    });
  }
  function iu() {
    const n = Ni(), t = new Ut(), e = Ni(), s = Ni(), i = Ni();
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
        n.removeParticles(), t.removeChildren(), e.removeParticles(), s.removeParticles(), i.removeParticles(), oe.clear(), Ts.clear();
      },
      count() {
        return n.particleChildren.length + e.particleChildren.length + s.particleChildren.length + i.particleChildren.length;
      }
    };
  }
  const oe = /* @__PURE__ */ new Map();
  function us(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function $0(n, t) {
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
      if (!u || !(sn.has(u.name) || pe.has(u.name) && u.io_type === "output" || ie.has(u.name))) continue;
      const [f, m] = s[u.direction ?? "North"] ?? [
        0,
        -1
      ], g = ie.has(u.name) ? h : u.x ?? 0, y = ie.has(u.name) ? d : u.y ?? 0;
      if (!(g + f !== (n.x ?? 0) || y + m !== (n.y ?? 0))) if (u.direction === e) o = true;
      else {
        const w = f * r - m * i;
        w !== 0 && (a = w > 0 ? "cw" : "ccw");
      }
    }
    return a && !o ? a === "cw" ? "corner-cw" : "corner-ccw" : "straight";
  }
  function B0(n, t) {
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
      d && ru(e, s, d) && (r |= l);
    }
    return r;
  }
  function ru(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of Yd(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  const O0 = 9079434;
  function N0(n, t, e) {
    const s = n.pipeStubLayer;
    s.removeChildren();
    const i = Math.max(2, (S - 1) * 0.4);
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
        if (!d || !ru(o, a, d)) continue;
        const u = o * S + S / 2, p = a * S + S / 2, f = S * 1.5, m = u + l * f, g = p + c * f, y = new ft();
        y.moveTo(u, p).lineTo(m, g).stroke({
          width: i,
          color: O0,
          cap: "round"
        }), s.addChild(y);
      }
    }
  }
  function Sa(n, t) {
    if (sn.has(n.name)) {
      const s = $0(n, t.tileMap), i = d0(n.name, n.direction ?? "North", s);
      return hn(i, gt, gt, (r) => {
        const o = gt / S;
        let a = null;
        s === "corner-cw" ? a = {
          turn: "cw"
        } : s === "corner-ccw" && (a = {
          turn: "ccw"
        });
        const l = xa(n, a);
        l.scale.set(o), r.addChild(l);
      });
    }
    if (pe.has(n.name)) {
      const s = n.io_type ?? "input", i = p0(n.name, n.direction ?? "North", s);
      return hn(i, gt, gt, (r) => {
        const o = gt / S, a = Fn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (ie.has(n.name)) {
      const s = f0(n.name, n.direction ?? "North"), i = n.direction === "North" || n.direction === "South";
      return lc(s, i ? 2 : 1, i ? 1 : 2, (a, l, c) => {
        const h = gt / S, d = Fn(n, t);
        d.scale.set(h), d.x = 0, d.y = 0, a.addChild(d);
      });
    }
    if (n.name === "pipe") {
      const s = B0(n, t), i = u0(s);
      return hn(i, gt, gt, (r) => {
        const o = gt / S, a = Fn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (n.name === "pipe-to-ground") {
      const s = x0(n.direction ?? "North");
      return hn(s, gt, gt, (i) => {
        const r = gt / S, o = Fn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (br.has(n.name)) {
      const s = m0(n.name, n.direction ?? "North");
      return hn(s, gt, gt, (i) => {
        const r = gt / S, o = Fn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (fa.has(n.name)) {
      const s = y0(n.name);
      return hn(s, gt, gt, (i) => {
        const r = gt / S, o = Fn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if ($e.has(n.name)) {
      const [s, i] = Ne[n.name] ?? [
        1,
        1
      ], r = g0(n.name);
      return lc(r, s, i, (o, a, l) => {
        const c = `/spaghettio/entity-frames/${n.name}.png`, h = ne.has(c) ? je.get(c) ?? null : null, d = 1.8, u = gt / k0;
        if (h) {
          const p = new Ye(h);
          p.scale.set(u * d);
          const f = a, m = l;
          p.x = -f * (d - 1) / 2, p.y = -m * (d - 1) / 2, o.addChild(p);
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
    return hn(e, gt, gt, (s) => {
      const i = gt / S, r = Fn(n, t);
      r.scale.set(i), r.x = 0, r.y = 0, s.addChild(r);
    });
  }
  function Do(n, t, e, s = _r()) {
    const i = us(t);
    if (oe.has(i)) return;
    const r = t.x ?? 0, o = t.y ?? 0, a = Sa(t, s);
    let l = S / gt, c = S / gt;
    if (ie.has(t.name)) {
      const f = t.direction === "North" || t.direction === "South", m = f ? 2 : 1, g = f ? 1 : 2;
      l = m * S / (m * gt), c = g * S / (g * gt);
    } else if ($e.has(t.name)) {
      const [f, m] = Ne[t.name] ?? [
        1,
        1
      ];
      l = f * S / (f * gt), c = m * S / (m * gt);
    }
    const h = r * S, d = o * S, u = new nr({
      texture: a,
      x: h,
      y: d,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: l,
      scaleY: c
    });
    $e.has(t.name) ? n.machineContainer.addParticle(u) : n.beltContainer.addParticle(u);
    let p;
    if (t.carries && !$e.has(t.name)) {
      const f = qd(t.carries), m = S * 0.35, g = (S - m) / 2;
      p = new nr({
        texture: f,
        x: h + g,
        y: d + g,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: m / gt,
        scaleY: m / gt
      }), n.iconContainer.addParticle(p);
    }
    oe.set(i, {
      entity: u,
      icon: p,
      revealAt: e,
      placedEntity: t
    });
  }
  const Ts = /* @__PURE__ */ new Map();
  function F0(n, t, e, s, i, r, o) {
    const a = `${t},${e}`, l = Ts.get(a);
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
    }, h = _r();
    ir(c, h);
    const d = Sa(c, h), u = Rn(i), p = new nr({
      texture: d,
      x: t * S,
      y: e * S,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: S / gt,
      scaleY: S / gt,
      tint: u
    });
    return n.ghostContainer.addParticle(p), l || Ts.set(a, {
      particle: p,
      specKey: o
    }), p;
  }
  function xc(n, t, e) {
    const s = `${t},${e}`, i = Ts.get(s);
    i && (n.ghostContainer.removeParticle(i.particle), Ts.delete(s));
  }
  function W0(n) {
    n.ghostContainer.removeParticles(), Ts.clear();
  }
  function G0(n, t, e) {
    const s = [];
    for (const [i, r] of oe.entries()) r.placedEntity.x !== t || r.placedEntity.y !== e || ($e.has(r.placedEntity.name) ? n.machineContainer.removeParticle(r.entity) : n.beltContainer.removeParticle(r.entity), r.icon && n.iconContainer.removeParticle(r.icon), oe.delete(i), s.push(i));
    return s;
  }
  function z0(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const [s, i] of oe.entries()) {
      const r = i.placedEntity.name;
      if (r !== "pipe" && !sn.has(r)) continue;
      const o = Sa(i.placedEntity, t);
      if (i.entity.texture === o) continue;
      const a = i.entity, l = new nr({
        texture: o,
        x: a.x,
        y: a.y,
        alpha: a.alpha,
        anchorX: a.anchorX,
        anchorY: a.anchorY,
        scaleX: a.scaleX,
        scaleY: a.scaleY
      });
      n.beltContainer.removeParticle(a), n.beltContainer.addParticle(l), oe.set(s, {
        ...i,
        entity: l
      }), e.set(a, l);
    }
    return e;
  }
  function ou(n, t) {
    for (const e of oe.values()) {
      const s = Math.min(1, Math.max(0, (t - e.revealAt) / su));
      e.entity.alpha = s, e.icon && (e.icon.alpha = s);
    }
  }
  function* bc(n) {
    for (const t of oe.values()) yield {
      particle: t.entity,
      iconParticle: t.icon,
      revealAt: t.revealAt
    };
  }
  function au(n) {
    const t = /* @__PURE__ */ new Map();
    let e = false;
    function s() {
      e || (e = true, n.add(r), wr());
    }
    function i() {
      e && (e = false, n.remove(r), qn());
    }
    function r() {
      const c = performance.now();
      let h = false;
      for (const [d, u] of t) {
        const p = c - u.startTime, f = Math.min(1, p / u.duration), m = u.ease(f), g = u.startAlpha + (u.targetAlpha - u.startAlpha) * m;
        u.entityParticle.alpha = g, u.iconParticle && (u.iconParticle.alpha = g), f >= 1 ? t.delete(d) : h = true;
      }
      h || i(), Ve();
    }
    function o(c, h, d, u) {
      const p = t.get(c), f = performance.now();
      let m;
      if (p) {
        const y = f - p.startTime, w = Math.min(1, y / p.duration);
        m = p.startAlpha + (p.targetAlpha - p.startAlpha) * p.ease(w);
      } else m = h.alpha;
      if (Math.abs(m - u) < 1e-3) {
        t.delete(c), h.alpha = u, d && (d.alpha = u);
        return;
      }
      const g = u > m;
      t.set(c, {
        entityParticle: h,
        iconParticle: d,
        startAlpha: m,
        targetAlpha: u,
        startTime: f,
        duration: g ? P0 : I0,
        ease: g ? R0 : L0
      }), s();
    }
    function a(c) {
      t.clear();
      for (const h of oe.values()) h.entity.alpha = c, h.icon && (h.icon.alpha = c);
      i(), Ve();
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
  function lu(n) {
    let t = 0;
    for (const i of n.values()) i > t && (t = i);
    const e = Math.max(t, M0), s = /* @__PURE__ */ new Map();
    for (const [i, r] of n) {
      const o = rr + (1 - rr) * (1 - r / e);
      s.set(i, o);
    }
    return s;
  }
  function Mn(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function D0(n, t, e) {
    t.clear(), t.layout = n;
    const s = _r();
    for (const l of n.entities) ir(l, s);
    const i = 0;
    for (const l of n.entities) Do(t, l, i, s);
    N0(t, n, s), ou(t, su + 1);
    const r = eu(n);
    if (!e) return H0();
    const o = au(e.ticker);
    function a(l) {
      const c = lu(l);
      for (const h of oe.values()) {
        const d = Mn(h.placedEntity), u = c.get(d) ?? rr;
        o.animateTo(d, h.entity, h.icon, u);
      }
    }
    return {
      highlightItem(l) {
        if (o.cancelAll(1), !!l) for (const c of oe.values()) {
          const h = c.placedEntity, u = (h.carries ?? h.recipe ?? null) === l ? 1 : 0.15, p = Mn(h);
          o.animateTo(p, c.entity, c.icon, u);
        }
      },
      highlightBeltNetwork(l) {
        if (!l) {
          o.cancelAll(1);
          return;
        }
        const c = Mn(l), h = nu(r, c);
        a(h);
      },
      clearHighlight() {
        for (const l of oe.values()) {
          const c = Mn(l.placedEntity);
          o.animateTo(c, l.entity, l.icon, 1);
        }
      },
      chainKey(l) {
        return l.carries ?? l.recipe ?? null;
      }
    };
  }
  function H0() {
    function n() {
      for (const t of oe.values()) t.entity.alpha = 1, t.icon && (t.icon.alpha = 1);
    }
    return {
      highlightItem(t) {
        if (n(), !!t) for (const e of oe.values()) {
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
  function U0(n, t) {
    const e = eu(n), s = au(t.ticker);
    function i(r) {
      const o = Mn(r), a = nu(e, o), l = lu(a);
      for (const c of oe.values()) {
        const h = Mn(c.placedEntity), d = l.get(h) ?? rr;
        s.animateTo(h, c.entity, c.icon, d);
      }
    }
    return {
      highlightItem(r) {
        if (s.cancelAll(1), !!r) for (const o of oe.values()) {
          const a = o.placedEntity, c = (a.carries ?? a.recipe ?? null) === r ? 1 : 0.15;
          o.entity.alpha = c, o.icon && (o.icon.alpha = c);
        }
      },
      highlightBeltNetwork(r) {
        if (!r) {
          for (const o of oe.values()) {
            const a = Mn(o.placedEntity);
            s.animateTo(a, o.entity, o.icon, 1);
          }
          return;
        }
        i(r);
      },
      clearHighlight() {
        for (const r of oe.values()) {
          const o = Mn(r.placedEntity);
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
  const _c = 6, j0 = 18, V0 = 2, Y0 = 26, X0 = -Math.PI / 3, wc = 2, q0 = 16777215, K0 = 0.55, vc = 4, J0 = 0, Z0 = 0.6;
  function Q0(n) {
    return n.carries ? sn.has(n.name) || pe.has(n.name) || ai.has(n.name) : false;
  }
  function tb(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const l of t.external_inputs) e.set(l.item, {
      rate: l.rate,
      isFluid: !!l.is_fluid
    });
    if (e.size === 0) return [];
    const s = /* @__PURE__ */ new Map();
    for (const l of n.entities) {
      if (!Q0(l)) continue;
      const c = l.carries;
      if (!e.has(c)) continue;
      const h = l.x ?? 0, d = l.y ?? 0, u = s.get(h);
      (!u || d < u.y) && s.set(h, {
        y: d,
        carries: c
      });
    }
    if (s.size === 0) return [];
    const i = Array.from(s.entries()).filter(([, l]) => l.y === J0).map(([l, c]) => ({
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
  function eb(n) {
    return `${n.toFixed(1)}/s`;
  }
  function nb(n) {
    const t = n.xMax - n.xMin + 1, e = Math.min(Y0, j0 + (t - 1) * V0), s = new Ut();
    s.eventMode = "none";
    const i = qd(n.item), r = new Ye(i);
    r.width = e, r.height = e, r.x = 0, r.y = -e / 2, s.addChild(r);
    const o = new Be({
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
    }), a = new Ue({
      text: eb(n.rate),
      style: o
    });
    a.x = e + _c, a.y = -a.height / 2, s.addChild(a);
    const l = new Be({
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
    }), c = new Ue({
      text: fe(n.item),
      style: l
    });
    return c.alpha = Z0, c.x = a.x + a.width + _c, c.y = -c.height / 2, s.addChild(c), s;
  }
  function sb(n, t, e) {
    if (n.removeChildren(), !e) return;
    const s = tb(t, e);
    if (s.length !== 0) for (const i of s) {
      const r = i.topY * S - wc, o = i.xMin * S + vc, a = (i.xMax - i.xMin + 1) * S - 2 * vc;
      if (a > 0) {
        const d = new ft();
        d.rect(o, r, a, wc).fill({
          color: q0,
          alpha: K0
        }), n.addChild(d);
      }
      const l = nb(i);
      l.rotation = X0;
      const c = (i.xMin + i.xMax + 1) / 2 * S, h = i.topY * S - S * 0.5;
      l.x = c, l.y = h, n.addChild(l);
    }
  }
  function ib(n) {
    return ie.has(n.name) ? n.direction === "East" || n.direction === "West" ? [
      1,
      2
    ] : [
      2,
      1
    ] : Ne[n.name] ?? [
      1,
      1
    ];
  }
  const eo = 57504;
  function Cc(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map();
    for (const v of s.entities) r.set(`${v.x ?? 0},${v.y ?? 0}`, v);
    let o = null, a = false, l = [];
    const c = new ft();
    e.addChild(c);
    const h = new ft();
    e.addChild(h);
    function d(v, b) {
      const C = n.getBoundingClientRect();
      return t.toWorld(v - C.left, b - C.top);
    }
    function u(v, b) {
      if (!o) return;
      const C = d(o.sx, o.sy), T = d(v, b), L = Math.min(C.x, T.x), A = Math.min(C.y, T.y), E = Math.abs(T.x - C.x), I = Math.abs(T.y - C.y);
      c.clear(), c.rect(L, A, E, I).fill({
        color: eo,
        alpha: 0.18
      }), c.setStrokeStyle({
        width: 1,
        color: eo,
        alpha: 0.8
      }), c.rect(L, A, E, I).stroke(), Ve();
    }
    function p(v) {
      if (h.clear(), v.length !== 0) {
        h.setStrokeStyle({
          width: 1.5,
          color: eo,
          alpha: 0.9
        });
        for (const b of v) {
          const [C, T] = ib(b), L = (b.x ?? 0) * S + 1, A = (b.y ?? 0) * S + 1;
          h.rect(L, A, C * S - 2, T * S - 2).stroke();
        }
      }
    }
    function f(v, b) {
      if (!o) return [];
      const C = d(o.sx, o.sy), T = d(v, b), L = Math.min(Math.floor(C.x / S), Math.floor(T.x / S)), A = Math.max(Math.floor(C.x / S), Math.floor(T.x / S)), E = Math.min(Math.floor(C.y / S), Math.floor(T.y / S)), I = Math.max(Math.floor(C.y / S), Math.floor(T.y / S)), V = [];
      for (let G = L; G <= A; G++) for (let F = E; F <= I; F++) {
        const $ = r.get(`${G},${F}`);
        $ && V.push($);
      }
      return V;
    }
    const m = (v) => {
      v.button !== 0 || !v.shiftKey || (o = {
        sx: v.clientX,
        sy: v.clientY
      }, a = false);
    }, g = (v) => {
      if (!o) return;
      const b = v.clientX - o.sx, C = v.clientY - o.sy;
      !a && b * b + C * C > 36 && (a = true), a && u(v.clientX, v.clientY);
    }, y = (v) => {
      if (v.button === 0) {
        if (a) v.stopImmediatePropagation(), c.clear(), l = f(v.clientX, v.clientY), p(l), i(l), Ve();
        else if (o !== null) {
          const b = d(v.clientX, v.clientY), C = Math.floor(b.x / S), T = Math.floor(b.y / S);
          r.has(`${C},${T}`) && (l = [], h.clear(), i([]), Ve());
        }
        o = null, a = false;
      }
    };
    function w() {
      l = [], c.clear(), h.clear(), i([]), Ve();
    }
    const x = (v) => {
      v.preventDefault(), l.length > 0 && w();
    }, _ = (v) => {
      v.key === "Escape" && l.length > 0 && w();
    };
    return n.addEventListener("pointerdown", m, {
      capture: true
    }), n.addEventListener("pointermove", g, {
      capture: true
    }), n.addEventListener("pointerup", y, {
      capture: true
    }), n.addEventListener("contextmenu", x), window.addEventListener("keydown", _), {
      destroy() {
        n.removeEventListener("pointerdown", m, {
          capture: true
        }), n.removeEventListener("pointermove", g, {
          capture: true
        }), n.removeEventListener("pointerup", y, {
          capture: true
        }), n.removeEventListener("contextmenu", x), window.removeEventListener("keydown", _), c.destroy(), h.destroy();
      },
      clear: w,
      getSelected() {
        return [
          ...l
        ];
      },
      buildJson(v, b) {
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
          note: b
        }, null, 2);
      }
    };
  }
  const rb = {
    accumulator: "acc",
    "active-provider-chest": "apc",
    "advanced-circuit": "acd",
    "agricultural-science-pack": "aspg",
    "agricultural-tower": "atg",
    ammonia: "amm",
    "ammoniacal-solution": "asm",
    "arithmetic-combinator": "acr",
    "artificial-jellynut-soil": "ajs",
    "artificial-yumako-soil": "ays",
    "artillery-shell": "asr",
    "artillery-turret": "atr",
    "artillery-wagon": "awr",
    "assembling-machine-1": "am1",
    "assembling-machine-2": "am2",
    "assembling-machine-3": "am3",
    "asteroid-collector": "acs",
    "atomic-bomb": "abt",
    "automation-science-pack": "aspu",
    barrel: "bar",
    battery: "bat",
    "battery-equipment": "beaq",
    "battery-mk2-equipment": "bmeakqt2",
    "battery-mk3-equipment": "bmeakqt3",
    beacon: "beac",
    "belt-immunity-equipment": "bie",
    "big-electric-pole": "bep",
    "big-mining-drill": "bmdi",
    biochamber: "bioc",
    bioflux: "biof",
    biolab: "biol",
    "biter-egg": "bei",
    boiler: "boi",
    "buffer-chest": "bcu",
    "bulk-inserter": "biunl",
    "burner-inserter": "biunr",
    "burner-mining-drill": "bmdu",
    calcite: "cal",
    "cannon-shell": "csa",
    "captive-biter-spawner": "cbs",
    "capture-robot-rocket": "crr",
    car: "car",
    carbon: "carb",
    "carbon-fiber": "cfa",
    "cargo-bay": "cba",
    "cargo-landing-pad": "clp",
    "cargo-wagon": "cwa",
    centrifuge: "cen",
    "chemical-plant": "cph",
    "chemical-science-pack": "csph",
    "cliff-explosives": "cel",
    "cluster-grenade": "cgl",
    coal: "coa",
    "combat-shotgun": "cso",
    concrete: "con",
    "constant-combinator": "ccoo",
    "construction-robot": "cro",
    "copper-bacteria": "cbo",
    "copper-cable": "ccoa",
    "copper-ore": "coo",
    "copper-plate": "cpo",
    "crude-oil": "cor",
    "crude-oil-barrel": "cob",
    crusher: "cru",
    "cryogenic-plant": "cpr",
    "cryogenic-science-pack": "cspr",
    "decider-combinator": "dceo",
    "defender-capsule": "dceaf",
    "depleted-uranium-fuel-cell": "duf",
    "destroyer-capsule": "dceas",
    "discharge-defense-equipment": "dde",
    "display-panel": "dpi",
    "distractor-capsule": "dci",
    "efficiency-module": "emf",
    "efficiency-module-2": "em2",
    "efficiency-module-3": "em3",
    "electric-engine-unit": "eeu",
    "electric-furnace": "efl",
    "electric-mining-drill": "emd",
    electrolyte: "ele",
    "electromagnetic-plant": "epl",
    "electromagnetic-science-pack": "esp",
    "electronic-circuit": "ecl",
    "energy-shield-equipment": "ese",
    "energy-shield-mk2-equipment": "esm",
    "engine-unit": "eun",
    "exoskeleton-equipment": "eex",
    "explosive-cannon-shell": "ecs",
    "explosive-rocket": "erx",
    "explosive-uranium-cannon-shell": "euc",
    explosives: "exp",
    "express-loader": "elx",
    "express-splitter": "esx",
    "express-transport-belt": "etb",
    "express-underground-belt": "eub",
    "fast-inserter": "fia",
    "fast-loader": "flao",
    "fast-splitter": "fsa",
    "fast-transport-belt": "ftb",
    "fast-underground-belt": "fub",
    "firearm-magazine": "fmi",
    "fission-reactor-equipment": "frei",
    flamethrower: "flam",
    "flamethrower-ammo": "fal",
    "flamethrower-turret": "ftl",
    "fluid-wagon": "fwl",
    fluorine: "flu",
    "fluoroketone-cold": "fcl",
    "fluoroketone-cold-barrel": "fcb",
    "fluoroketone-hot": "fhl",
    "fluoroketone-hot-barrel": "fhb",
    "flying-robot-frame": "frf",
    foundation: "founda",
    foundry: "foundr",
    "fusion-generator": "fgu",
    "fusion-power-cell": "fpc",
    "fusion-reactor": "fru",
    "fusion-reactor-equipment": "freu",
    gate: "gat",
    grenade: "gre",
    "gun-turret": "gtu",
    "hazard-concrete": "hca",
    "heat-exchanger": "hee",
    "heat-interface": "hie",
    "heat-pipe": "hpe",
    "heating-tower": "hte",
    "heavy-armor": "hae",
    "heavy-oil": "hoe",
    "heavy-oil-barrel": "hob",
    "holmium-ore": "hoo",
    "holmium-plate": "hpo",
    "holmium-solution": "hso",
    ice: "ice",
    "ice-platform": "ipc",
    "infinity-chest": "icn",
    "infinity-pipe": "ipn",
    inserter: "ins",
    "iron-bacteria": "ibr",
    "iron-chest": "icr",
    "iron-gear-wheel": "igw",
    "iron-ore": "ior",
    "iron-plate": "ipr",
    "iron-stick": "isr",
    jelly: "jelly",
    jellynut: "jellyn",
    "jellynut-seed": "jse",
    lab: "lab",
    "land-mine": "lma",
    landfill: "lan",
    "laser-turret": "lta",
    lava: "lav",
    "light-armor": "lai",
    "light-oil": "loi",
    "light-oil-barrel": "lob",
    "lightning-collector": "lci",
    "lightning-rod": "lri",
    lithium: "lit",
    "lithium-brine": "lbi",
    "lithium-plate": "lpi",
    loader: "loa",
    locomotive: "loc",
    "logistic-robot": "lro",
    "logistic-science-pack": "lsp",
    "long-handed-inserter": "lhi",
    "low-density-structure": "lds",
    lubricant: "lub",
    "lubricant-barrel": "lbu",
    "mech-armor": "mae",
    "medium-electric-pole": "mep",
    "metallurgic-science-pack": "mspe",
    "military-science-pack": "mspi",
    "modular-armor": "mao",
    "molten-copper": "mco",
    "molten-iron": "mio",
    "night-vision-equipment": "nve",
    "nuclear-fuel": "nfu",
    "nuclear-reactor": "nru",
    nutrients: "nut",
    "offshore-pump": "opf",
    "oil-refinery": "ori",
    "overgrowth-jellynut-soil": "ojs",
    "overgrowth-yumako-soil": "oys",
    "passive-provider-chest": "ppc",
    "pentapod-egg": "pee",
    "personal-laser-defense-equipment": "pld",
    "personal-roboport-equipment": "pre",
    "personal-roboport-mk2-equipment": "prme",
    "petroleum-gas": "pge",
    "petroleum-gas-barrel": "pgb",
    "piercing-rounds-magazine": "prmi",
    "piercing-shotgun-shell": "pss",
    pipe: "pip",
    "pipe-to-ground": "ptg",
    pistol: "pis",
    "plastic-bar": "pbl",
    "poison-capsule": "pco",
    "power-armor": "pao",
    "power-armor-mk2": "pam",
    "power-switch": "pso",
    "processing-unit": "pur",
    "production-science-pack": "psprcaoicd",
    "productivity-module": "pmr",
    "productivity-module-2": "pm2",
    "productivity-module-3": "pm3",
    "programmable-speaker": "psr",
    "promethium-asteroid-chunk": "pac",
    "promethium-science-pack": "psprcaoicm",
    pump: "pump",
    pumpjack: "pumpj",
    "quality-module": "qmu",
    "quality-module-2": "qm2",
    "quality-module-3": "qm3",
    "quantum-processor": "qpu",
    radar: "rad",
    rail: "rail",
    "rail-chain-signal": "rcs",
    "rail-ramp": "rra",
    "rail-signal": "rsai",
    "rail-support": "rsau",
    railgun: "railg",
    "railgun-ammo": "raa",
    "railgun-turret": "rta",
    "raw-fish": "rfa",
    recycler: "rec",
    "refined-concrete": "rceo",
    "refined-hazard-concrete": "rhc",
    "repair-pack": "rpe",
    "requester-chest": "rceh",
    roboport: "rob",
    rocket: "roc",
    "rocket-fuel": "rfo",
    "rocket-launcher": "rlo",
    "rocket-part": "rpo",
    "rocket-silo": "rso",
    "rocket-turret": "rto",
    "selector-combinator": "sce",
    shotgun: "sho",
    "shotgun-shell": "ssh",
    "slowdown-capsule": "scl",
    "small-electric-pole": "sep",
    "small-lamp": "slm",
    "solar-panel": "spoa",
    "solar-panel-equipment": "spe",
    "solid-fuel": "sfo",
    "space-platform-foundation": "spf",
    "space-platform-starter-pack": "sps",
    "space-science-pack": "ssp",
    "speed-module": "smp",
    "speed-module-2": "sm2",
    "speed-module-3": "sm3",
    spidertron: "spi",
    splitter: "spl",
    spoilage: "spoi",
    "stack-inserter": "sit",
    steam: "ste",
    "steam-engine": "set",
    "steam-turbine": "sttu",
    "steel-chest": "scthe",
    "steel-furnace": "sftue",
    "steel-plate": "spt",
    stone: "sto",
    "stone-brick": "sbt",
    "stone-furnace": "sftuo",
    "stone-wall": "swt",
    "storage-chest": "sctho",
    "storage-tank": "stta",
    "submachine-gun": "sgu",
    substation: "sub",
    sulfur: "sul",
    "sulfuric-acid": "sau",
    "sulfuric-acid-barrel": "sab",
    supercapacitor: "superca",
    superconductor: "superco",
    tank: "tan",
    "tesla-ammo": "tae",
    "tesla-turret": "tte",
    teslagun: "tes",
    thruster: "thr",
    "thruster-fuel": "tfh",
    "thruster-oxidizer": "toh",
    "toolbelt-equipment": "teo",
    "train-stop": "tsrt",
    "transport-belt": "tbr",
    "tree-seed": "tsre",
    "tungsten-carbide": "tcu",
    "tungsten-ore": "tou",
    "tungsten-plate": "tpu",
    "turbo-loader": "tlu",
    "turbo-splitter": "tsu",
    "turbo-transport-belt": "ttb",
    "turbo-underground-belt": "tub",
    "underground-belt": "ubn",
    "uranium-235": "u2r3a5",
    "uranium-238": "u2r3a8",
    "uranium-cannon-shell": "ucs",
    "uranium-fuel-cell": "ufc",
    "uranium-ore": "uor",
    "uranium-rounds-magazine": "urm",
    "utility-science-pack": "usp",
    water: "wat",
    "water-barrel": "wba",
    wood: "woo",
    "wooden-chest": "wco",
    yumako: "yum",
    "yumako-mash": "ymu",
    "yumako-seed": "ysu"
  }, ob = {
    codes: rb
  }, cu = ob, ab = new Map(Object.entries(cu.codes)), lb = new Map(Object.entries(cu.codes).map(([n, t]) => [
    t,
    n
  ]));
  function cb(n) {
    return ab.get(n) ?? null;
  }
  function hb(n) {
    return lb.get(n) ?? null;
  }
  const db = [
    "partitioned-per-consumer",
    "partitioned-decomposed"
  ], ub = [
    "horizontal-stack"
  ], pb = [
    "regular",
    "fast"
  ], no = [
    "iron-plate",
    "copper-plate",
    "steel-plate",
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], Es = [
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], Ta = "iron-gear-wheel", Ea = 10, bs = {
    crafting: "assembling-machine-3",
    smelting: "electric-furnace"
  }, ui = {
    crafting: "craft",
    smelting: "smelt"
  }, fb = "machine", or = "#/l/", Ys = "_", ar = ",", mb = {
    pd: "partitioned-decomposed"
  }, Sc = {
    "partitioned-decomposed": "pd"
  }, gb = {
    hs: "horizontal-stack"
  }, Tc = {
    "horizontal-stack": "hs"
  }, yb = {
    r: "regular",
    f: "fast"
  }, Ec = {
    regular: "r",
    fast: "f"
  };
  function ls(n) {
    return cb(n) ?? n;
  }
  function cs(n) {
    return hb(n);
  }
  function xb() {
    const n = window.location.hash;
    if (!n.startsWith(or)) return null;
    const t = n.slice(or.length), e = t.indexOf("?"), s = e >= 0 ? t.slice(0, e) : t, i = e >= 0 ? t.slice(e + 1) : "", r = s.split("/"), o = (E) => {
      const I = r[E];
      return I === void 0 || I === "" || I === Ys ? null : I;
    }, a = o(0);
    let l;
    if (a) {
      const E = cs(a);
      if (E === null) return null;
      l = E;
    } else l = Ta;
    const c = o(1), h = c !== null ? parseFloat(c) : NaN, d = !isNaN(h) && h > 0 ? h : Ea, u = o(2), p = {};
    if (u) {
      const E = cs(u);
      if (E === null) return null;
      p.crafting = E;
    }
    const f = o(3);
    let m;
    if (f) {
      const E = f.split(ar).filter((V) => V.length > 0), I = [];
      for (const V of E) {
        const G = cs(V);
        if (G === null) return null;
        I.push(G);
      }
      m = I;
    } else m = Es;
    const g = o(4);
    let y = null;
    if (g && (y = cs(g), y === null)) return null;
    const w = new URLSearchParams(i), x = w.get("s"), _ = x ? mb[x] ?? null : null, v = w.get("rl"), b = v ? gb[v] ?? null : null, C = w.get("it"), T = C ? yb[C] ?? null : null, L = w.get("ci");
    let A = [];
    if (L) {
      const E = L.split(ar).filter((I) => I.length > 0);
      for (const I of E) {
        const V = cs(I);
        if (V === null) return null;
        A.push(V);
      }
    }
    for (const [E, I] of Object.entries(ui)) {
      if (E === "crafting") continue;
      const V = w.get(I);
      if (!V) continue;
      const G = cs(V);
      if (G === null) return null;
      p[E] = G;
    }
    return {
      item: l,
      rate: d,
      machines: p,
      inputs: m,
      belt: y,
      strategy: _,
      rowLayout: b,
      inserterTier: T,
      customInputs: A
    };
  }
  function bb() {
    const n = new URLSearchParams(window.location.search), t = n.get("item") ?? Ta, e = parseFloat(n.get("rate") ?? ""), s = isNaN(e) || e <= 0 ? Ea : e, i = {};
    for (const [y, w] of Object.entries(ui)) {
      const x = n.get(w);
      x && (i[y] = x);
    }
    const r = n.get(fb);
    r && !i.crafting && (i.crafting = r);
    const o = n.get("in"), a = o ? o.split(",").filter((y) => y.length > 0) : Es, l = n.get("belt"), c = n.get("strategy");
    let h = c && db.includes(c) ? c : null;
    h === "partitioned-per-consumer" && (h = "partitioned-decomposed");
    const d = n.get("row_layout"), u = d && ub.includes(d) ? d : null, p = n.get("inserter_tier"), f = p && pb.includes(p) ? p : null, m = n.get("ci"), g = m ? m.split(",").filter((y) => y.length > 0) : [];
    return {
      item: t,
      rate: s,
      machines: i,
      inputs: a,
      belt: l,
      strategy: h,
      rowLayout: u,
      inserterTier: f,
      customInputs: g
    };
  }
  function _b() {
    return xb() ?? bb();
  }
  function wb() {
    if (window.location.hash.startsWith(or)) return true;
    const n = new URLSearchParams(window.location.search);
    return n.has("item") || n.has("rate") || n.has("machine") || n.has("in") || n.has("belt");
  }
  function vb(n) {
    for (const [t, e] of Object.entries(ui)) {
      const s = n[t];
      if (s && s !== bs[t]) return false;
    }
    for (const t of Object.keys(n)) if (!(t in ui)) return false;
    return true;
  }
  function Cb(n) {
    const t = ls(n.item), e = String(n.rate), s = n.machines.crafting, i = !s || s === bs.crafting ? Ys : ls(s), r = n.inputs.length === Es.length && n.inputs.every((u, p) => u === Es[p]), o = n.inputs.length === 0 || r ? Ys : n.inputs.map(ls).join(ar), a = n.belt ? ls(n.belt) : Ys, l = new URLSearchParams();
    n.strategy && Sc[n.strategy] && l.set("s", Sc[n.strategy]), n.rowLayout && Tc[n.rowLayout] && l.set("rl", Tc[n.rowLayout]), n.inserterTier && Ec[n.inserterTier] && l.set("it", Ec[n.inserterTier]), n.customInputs.length > 0 && l.set("ci", n.customInputs.map(ls).join(ar));
    for (const [u, p] of Object.entries(ui)) {
      if (u === "crafting") continue;
      const f = n.machines[u];
      f && f !== bs[u] && l.set(p, ls(f));
    }
    const c = [
      t,
      e,
      i,
      o,
      a
    ], h = l.toString();
    if (h.length === 0) for (; c.length > 2 && c[c.length - 1] === Ys; ) c.pop();
    let d = `${or}${c.join("/")}`;
    return h.length > 0 && (d += `?${h}`), d;
  }
  function Sb(n) {
    const t = n.item === Ta && n.rate === Ea && vb(n.machines) && n.inputs.length === Es.length && n.inputs.every((s, i) => s === Es[i]) && !n.belt && !n.strategy && !n.rowLayout && !n.inserterTier && n.customInputs.length === 0, e = window.location.pathname;
    if (t) {
      history.replaceState(null, "", e);
      return;
    }
    history.replaceState(null, "", e + Cb(n));
  }
  const so = "[INCOMPATIBLE_MACHINE]";
  function pn(n, t = 14) {
    const e = document.createElement("img");
    return e.src = `/spaghettio/icons/${n}.png`, e.width = t, e.height = t, e.style.cssText = "image-rendering:pixelated", e.onerror = () => {
      e.style.display = "none";
    }, e;
  }
  function Tb(n, t) {
    const e = document.createElement("option");
    return e.value = n, e.textContent = fe(n), n === t && (e.selected = true), e;
  }
  function Fi(n, t, e) {
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
  function Ac(n, t, e, s) {
    for (const i of t) {
      const r = document.createElement("div");
      r.className = `sb-machine-flow ${e}`, s && r.appendChild(document.createTextNode(s)), r.appendChild(pn(i.item, 13)), r.appendChild(document.createTextNode(fe(i.item)));
      const o = document.createElement("span");
      o.className = "flow-rate";
      const a = tu(i.rate), l = a ? Qd(a.color) : "#f88";
      o.style.color = l, o.textContent = `${i.rate.toFixed(1)}/s`, r.appendChild(o), n.appendChild(r);
    }
  }
  const kc = /* @__PURE__ */ new Set([
    "water",
    "crude-oil",
    "petroleum-gas",
    "light-oil",
    "heavy-oil",
    "sulfuric-acid",
    "lubricant",
    "steam"
  ]), hu = "spaghettio-recent-items", Mc = 5;
  function du() {
    try {
      const n = localStorage.getItem(hu);
      return n ? JSON.parse(n) : [];
    } catch {
      return [];
    }
  }
  function Eb(n) {
    const t = du().filter((e) => e !== n);
    t.unshift(n), t.length > Mc && (t.length = Mc);
    try {
      localStorage.setItem(hu, JSON.stringify(t));
    } catch {
    }
  }
  function Ab(n, t, e) {
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
        a.appendChild(pn(s, 14));
        const x = document.createElement("span");
        x.textContent = fe(s), a.appendChild(x);
      } else {
        const x = document.createElement("span");
        x.className = "sb-picker-placeholder", x.textContent = "Select item\u2026", a.appendChild(x);
      }
    }
    function p(x) {
      const _ = document.createElement("div");
      _.className = "sb-picker-item" + (x === s ? " selected" : ""), _.dataset.slug = x, _.appendChild(pn(x, 14));
      const v = document.createElement("span");
      return v.textContent = fe(x), _.appendChild(v), _.addEventListener("mousedown", (b) => {
        b.preventDefault(), m(x);
      }), _;
    }
    function f(x) {
      d.innerHTML = "", r = null;
      const _ = x.trim().toLowerCase(), v = _ ? n.filter((b) => b.includes(_) || fe(b).toLowerCase().includes(_)) : n;
      if (!_) {
        const b = du().filter((C) => n.includes(C));
        if (b.length > 0) {
          const C = document.createElement("div");
          C.className = "sb-picker-section-label", C.textContent = "Recent", d.appendChild(C);
          for (const L of b) d.appendChild(p(L));
          const T = document.createElement("div");
          T.className = "sb-picker-divider", d.appendChild(T);
        }
      }
      for (const b of v) d.appendChild(p(b));
      if (!_ && s) {
        const b = d.querySelector(`[data-slug="${s}"]`);
        b && b.scrollIntoView({
          block: "nearest"
        });
      }
    }
    function m(x) {
      s = x, Eb(x), o.classList.remove("item-invalid"), u(), y(), e(x);
    }
    function g() {
      i = true, o.classList.add("open"), c.style.display = "", l.textContent = "\u25B4", h.value = "", f(""), requestAnimationFrame(() => h.focus());
    }
    function y() {
      i = false, o.classList.remove("open"), c.style.display = "none", l.textContent = "\u25BE", r = null;
    }
    function w(x) {
      const _ = d.querySelectorAll(".sb-picker-item");
      if (!_.length) return;
      const v = Array.from(_);
      let b = r ? v.indexOf(r) : -1;
      b = Math.max(0, Math.min(v.length - 1, b + x)), r == null ? void 0 : r.classList.remove("highlighted"), r = v[b], r.classList.add("highlighted"), r.scrollIntoView({
        block: "nearest"
      });
    }
    return o.addEventListener("mousedown", (x) => {
      c.contains(x.target) || (x.preventDefault(), i ? y() : g());
    }), h.addEventListener("input", () => f(h.value)), h.addEventListener("keydown", (x) => {
      x.key === "ArrowDown" ? (x.preventDefault(), w(1)) : x.key === "ArrowUp" ? (x.preventDefault(), w(-1)) : x.key === "Enter" ? (r == null ? void 0 : r.dataset.slug) && m(r.dataset.slug) : x.key === "Escape" && y();
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
  function kb(n, t, e) {
    var _a2;
    n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "sidebar-inner";
    const { section: i, body: r } = Fi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="2"/></svg>', "Target"), o = t.allProducibleItems(), a = new Set(o);
    function l(U, Z) {
      const it = document.createElement("div");
      it.className = "sb-field";
      const pt = document.createElement("span");
      return pt.className = "sb-field-label", pt.textContent = U, it.appendChild(pt), Z.style.flex = "1", Z.style.minWidth = "0", it.appendChild(Z), it;
    }
    const c = Ab(o, "", () => Et());
    c.el.style.cssText = "margin-bottom:6px", r.appendChild(c.el);
    const h = [
      {
        category: "crafting",
        label: "Assembler",
        options: [
          {
            value: "assembling-machine-1"
          },
          {
            value: "assembling-machine-2"
          },
          {
            value: "assembling-machine-3"
          }
        ]
      },
      {
        category: "smelting",
        label: "Furnace",
        options: [
          {
            value: "electric-furnace"
          },
          {
            value: "stone-furnace",
            disabled: true,
            title: "Requires fuel routing \u2014 coming later"
          }
        ]
      }
    ], d = [
      {
        label: "Foundry",
        machine: "foundry"
      },
      {
        label: "EM Plant",
        machine: "electromagnetic-plant"
      },
      {
        label: "Chemical Plant",
        machine: "chemical-plant"
      },
      {
        label: "Oil Refinery",
        machine: "oil-refinery"
      },
      {
        label: "Cryogenic Plant",
        machine: "cryogenic-plant"
      },
      {
        label: "Biochamber",
        machine: "biochamber"
      }
    ], u = /* @__PURE__ */ new Map();
    for (const U of h) {
      const Z = document.createElement("select");
      Z.className = "sb-select", Z.dataset.cat = U.category;
      const it = bs[U.category] ?? "";
      for (const pt of U.options) {
        const It = Tb(pt.value, it);
        pt.disabled && (It.disabled = true), pt.title && (It.title = pt.title), Z.appendChild(It);
      }
      r.appendChild(l(U.label, Z)), u.set(U.category, Z);
    }
    const p = u.get("crafting");
    for (const U of d) {
      const Z = document.createElement("span");
      Z.className = "sb-machine-readonly", Z.textContent = fe(U.machine), r.appendChild(l(U.label, Z));
    }
    function f() {
      const U = {};
      for (const [Z, it] of u) it.value && (U[Z] = it.value);
      return U;
    }
    const m = document.createElement("select");
    m.className = "sb-select", [
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
    ].forEach(([U, Z]) => {
      const it = document.createElement("option");
      it.value = Z, it.textContent = U, m.appendChild(it);
    }), r.appendChild(l("Belt", m));
    const g = document.createElement("select");
    g.className = "sb-select", [
      [
        "Stack (default)",
        ""
      ],
      [
        "Fast",
        "fast"
      ],
      [
        "Regular",
        "regular"
      ]
    ].forEach(([U, Z]) => {
      const it = document.createElement("option");
      it.value = Z, it.textContent = U, g.appendChild(it);
    }), r.appendChild(l("Inserter tier", g));
    const y = document.createElement("select");
    y.className = "sb-select", [
      [
        "Pooled (default)",
        ""
      ],
      [
        "Partitioned + decomposed",
        "partitioned-decomposed"
      ]
    ].forEach(([U, Z]) => {
      const it = document.createElement("option");
      it.value = Z, it.textContent = U, y.appendChild(it);
    }), r.appendChild(l("Strategy", y));
    const w = document.createElement("select");
    w.className = "sb-select", [
      [
        "Vertical split (today)",
        ""
      ],
      [
        "Horizontal stack (RFP)",
        "horizontal-stack"
      ]
    ].forEach(([U, Z]) => {
      const it = document.createElement("option");
      it.value = Z, it.textContent = U, w.appendChild(it);
    }), r.appendChild(l("Row layout", w));
    const x = document.createElement("div");
    x.className = "sb-field";
    const _ = document.createElement("span");
    _.className = "sb-field-label", _.textContent = "Rate", x.appendChild(_);
    const v = document.createElement("input");
    v.type = "number", v.className = "sb-input", v.step = "0.5", v.min = "0.1", v.style.cssText = "flex:1;min-width:0", v.placeholder = "10", x.appendChild(v);
    const b = document.createElement("span");
    b.className = "sb-rate-suffix", b.textContent = "/s", x.appendChild(b), r.appendChild(x), s.appendChild(i);
    const { section: C, body: T } = Fi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="6" rx="1"/><line x1="5" y1="8" x2="11" y2="8"/></svg>', "Inputs"), L = document.createElement("div");
    L.className = "sb-tags";
    const A = /* @__PURE__ */ new Map();
    no.forEach((U) => {
      const Z = document.createElement("label");
      Z.className = `sb-tag${kc.has(U) ? " fluid" : ""}`;
      const it = document.createElement("span");
      it.className = "sb-tag-check", it.textContent = "\u2713";
      const pt = document.createElement("input");
      pt.type = "checkbox", pt.value = U, pt.style.display = "none", A.set(U, pt), Z.appendChild(it), Z.appendChild(pn(U, 14)), Z.appendChild(document.createTextNode(fe(U))), Z.appendChild(pt), pt.addEventListener("change", () => {
        Z.classList.toggle("active", pt.checked);
      }), L.appendChild(Z);
    }), T.appendChild(L);
    let E = [];
    const I = document.createElement("div");
    I.className = "sb-tags sb-custom-tags", T.appendChild(I);
    const V = document.createElement("datalist");
    V.id = "spaghettio-custom-inputs-datalist";
    const G = new Set(no);
    o.filter((U) => !G.has(U)).forEach((U) => {
      const Z = document.createElement("option");
      Z.value = U, V.appendChild(Z);
    }), T.appendChild(V);
    const F = document.createElement("input");
    F.type = "text", F.className = "sb-input sb-custom-input-field", F.setAttribute("list", "spaghettio-custom-inputs-datalist"), F.autocomplete = "off", F.placeholder = "+ add input\u2026", T.appendChild(F);
    function $(U) {
      const Z = document.createElement("div");
      Z.className = `sb-tag sb-custom-tag active${kc.has(U) ? " fluid" : ""}`, Z.dataset.item = U, Z.appendChild(pn(U, 14)), Z.appendChild(document.createTextNode(fe(U)));
      const it = document.createElement("span");
      it.className = "sb-tag-remove", it.textContent = "\xD7", it.addEventListener("click", (pt) => {
        pt.stopPropagation(), E = E.filter((It) => It !== U), Z.remove(), Et();
      }), Z.appendChild(it), I.appendChild(Z);
    }
    function z(U) {
      const Z = U.trim();
      !Z || !a.has(Z) || G.has(Z) || E.includes(Z) || (E.push(Z), $(Z), F.value = "", Et());
    }
    F.addEventListener("keydown", (U) => {
      U.key === "Enter" && z(F.value);
    }), F.addEventListener("change", () => {
      z(F.value);
    }), s.appendChild(C);
    const { section: J, body: X, countEl: Q } = Fi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>', "Solver", ""), M = document.createElement("div");
    X.appendChild(M), s.appendChild(J);
    const O = document.createElement("div");
    O.className = "sb-actions", O.style.display = "none";
    const N = document.createElement("button");
    N.className = "sb-btn sb-btn-secondary", N.textContent = "Copy Blueprint", N.style.flex = "1", O.appendChild(N);
    const D = document.createElement("div");
    D.className = "sb-copy-status", O.appendChild(D), X.appendChild(O);
    const { section: K, body: tt, countEl: ot } = Fi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.8" fill="currentColor" stroke="none"/></svg>', "Validation", "");
    K.style.display = "none", s.appendChild(K), n.appendChild(s);
    const at = _b();
    c.setValue(at.item), v.value = String(at.rate);
    const mt = /* @__PURE__ */ new Set([
      "assembling-machine-1",
      "assembling-machine-2",
      "assembling-machine-3"
    ]);
    for (const [U, Z] of u) {
      const it = at.machines[U], pt = new Set(Array.from(Z.options).filter((It) => !It.disabled).map((It) => It.value));
      Z.value = it && pt.has(it) ? it : bs[U] ?? ((_a2 = Z.options[0]) == null ? void 0 : _a2.value) ?? "";
    }
    A.forEach((U, Z) => {
      U.checked = at.inputs.includes(Z);
      const it = U.closest(".sb-tag");
      it && it.classList.toggle("active", U.checked);
    }), at.belt && (m.value = at.belt), at.strategy && (y.value = at.strategy), at.rowLayout && (w.value = at.rowLayout), at.inserterTier && (g.value = at.inserterTier);
    for (const U of at.customInputs) a.has(U) && !G.has(U) && !E.includes(U) && (E.push(U), $(U));
    const bt = document.createElement("div");
    bt.className = "sb-config-error", bt.style.display = "none", M.before(bt);
    function Y(U) {
      U ? (bt.textContent = U, bt.style.display = "") : (bt.textContent = "", bt.style.display = "none");
    }
    let et = null, lt = at.item, ct = null, _t = 0;
    function Et() {
      et !== null && clearTimeout(et), et = setTimeout(() => {
        jt().catch((U) => console.error("runSolve failed:", U));
      }, 150);
    }
    async function jt() {
      var _a3;
      const U = c.getValue(), Z = parseFloat(v.value), it = no.filter((At) => {
        var _a4;
        return (_a4 = A.get(At)) == null ? void 0 : _a4.checked;
      }), pt = [
        ...it,
        ...E
      ];
      if (!a.has(U)) {
        c.setInvalid(true);
        return;
      }
      if (c.setInvalid(false), isNaN(Z) || Z <= 0) return;
      if (U !== lt) {
        const At = t.defaultMachineForItem(U, p.value);
        mt.has(At) && (p.value = At), lt = U;
      }
      const It = f();
      Sb({
        item: U,
        rate: Z,
        machines: It,
        inputs: it,
        belt: m.value || null,
        strategy: y.value || null,
        rowLayout: w.value || null,
        inserterTier: g.value || null,
        customInputs: E
      });
      const zt = ++_t;
      M.innerHTML = "", Y(null), ct = null, O.style.display = "none";
      let Dt;
      try {
        Dt = await t.solve(U, Z, pt, It, It.crafting ?? bs.crafting);
      } catch (At) {
        if (zt !== _t) return;
        e.renderGraph(null), Q && (Q.textContent = "error");
        const Rt = String(At instanceof Error ? At.message : At);
        if (Rt.includes(so)) {
          const Lt = Rt.indexOf(so), he = Rt.slice(Lt + so.length).trim();
          Y(he);
        } else {
          const Lt = document.createElement("div");
          Lt.className = "sb-result-error", Lt.textContent = Rt, M.appendChild(Lt);
        }
        return;
      }
      if (zt !== _t) return;
      Mb(M, Dt), e.renderGraph(Dt);
      const Vt = Dt.machines.reduce((At, Rt) => At + Math.ceil(Rt.count), 0);
      Q && (Q.textContent = `${Vt} machines`);
      const se = /* @__PURE__ */ new Set();
      for (const At of Dt.machines) {
        for (const Rt of At.inputs) se.add(Rt.item);
        for (const Rt of At.outputs) se.add(Rt.item);
      }
      for (const At of Dt.external_inputs) se.add(At.item);
      for (const At of Dt.external_outputs) se.add(At.item);
      if (await Vd(Array.from(se)), zt !== _t) return;
      let Ce;
      try {
        const At = m.value || void 0, Rt = y.value || void 0, Lt = w.value || void 0, he = g.value || void 0, k = e.startStreaming();
        Ce = await t.buildLayoutStreaming(Dt, At, Rt, Lt, he, k);
      } catch (At) {
        if (zt !== _t) return;
        const Rt = document.createElement("div");
        Rt.className = "sb-result-error", Rt.textContent = `Layout error: ${At}`, M.appendChild(Rt);
        return;
      }
      zt === _t && (ct = Ce, $d(Dt.machines), e.renderLayout(Ce, Dt), O.style.display = ((_a3 = Ce.warnings) == null ? void 0 : _a3.length) ? "none" : "flex");
    }
    N.addEventListener("click", async () => {
      if (!ct) return;
      const U = await t.exportBlueprint(ct, c.getValue());
      await navigator.clipboard.writeText(U), D.textContent = "Copied!", setTimeout(() => {
        D.textContent = "";
      }, 2e3);
    }), v.addEventListener("input", Et);
    for (const U of u.values()) U.addEventListener("change", Et);
    return m.addEventListener("change", Et), y.addEventListener("change", Et), w.addEventListener("change", Et), g.addEventListener("change", Et), A.forEach((U) => U.addEventListener("change", Et)), jt().catch((U) => console.error("runSolve failed:", U)), {
      getParams() {
        const U = c.getValue(), Z = parseFloat(v.value);
        return !U || isNaN(Z) || Z <= 0 ? null : {
          item: U,
          rate: Z
        };
      },
      setParams(U, Z) {
        c.setValue(U.item), v.value = String(U.rate), U.machine && mt.has(U.machine) ? p.value = U.machine : p.value = "assembling-machine-3", U.inputs && A.forEach((it, pt) => {
          it.checked = U.inputs.includes(pt);
          const It = it.closest(".sb-tag");
          It && It.classList.toggle("active", it.checked);
        }), U.belt ? m.value = U.belt : m.value = "", I.innerHTML = "", E = [];
        for (const it of U.customInputs ?? []) a.has(it) && !G.has(it) && !E.includes(it) && (E.push(it), $(it));
        lt = U.item, (Z == null ? void 0 : Z.skipAutoSolve) || Et();
      },
      updateValidation(U, Z, it) {
        if (tt.innerHTML = "", U.length === 0) {
          K.style.display = "none", ot && (ot.textContent = "");
          return;
        }
        K.style.display = "";
        const pt = U.filter((Dt) => Dt.severity === "Error").length, It = U.length - pt;
        ot && (pt > 0 ? (ot.textContent = `${pt} error${pt !== 1 ? "s" : ""}`, ot.style.color = "#f66") : (ot.textContent = `${It} warning${It !== 1 ? "s" : ""}`, ot.style.color = "#fa0"));
        const zt = /* @__PURE__ */ new Map();
        for (const Dt of U) {
          let Vt = zt.get(Dt.category);
          Vt || (Vt = [], zt.set(Dt.category, Vt)), Vt.push(Dt);
        }
        for (const [Dt, Vt] of zt) {
          const Ce = Vt.some((R) => R.severity === "Error") ? "#f44" : "#fa0", At = Vt.find((R) => R.x != null && R.y != null), Rt = document.createElement("div");
          Rt.className = "sb-val-group";
          const Lt = document.createElement("div");
          Lt.className = "sb-val-group-header";
          const he = document.createElement("span");
          he.className = "sb-val-group-chevron", he.textContent = "\u25BE", he.addEventListener("click", (R) => {
            R.stopPropagation();
            const B = j.style.display === "none";
            j.style.display = B ? "" : "none", he.textContent = B ? "\u25BE" : "\u25B8";
          }), Lt.appendChild(he);
          const k = document.createElement("span");
          k.className = "sb-val-group-dot", k.style.background = Ce, Lt.appendChild(k);
          const P = document.createElement("span");
          P.className = "sb-val-group-name", P.textContent = Dt, Lt.appendChild(P);
          const q = document.createElement("span");
          q.className = "sb-val-group-count", q.textContent = String(Vt.length), Lt.appendChild(q);
          const j = document.createElement("div");
          if (j.className = "sb-val-group-body", it) {
            const R = Vt.filter((B) => B.detail != null);
            if (R.length > 0) {
              const B = /* @__PURE__ */ new Map();
              for (const H of R) {
                const st = it(H) ?? "unattributed";
                B.set(st, (B.get(st) ?? 0) + 1);
              }
              const nt = document.createElement("div");
              nt.className = "sb-val-cause-rollup", nt.style.cssText = "font-size:11px;color:#b90;padding:1px 0 2px 22px", nt.textContent = [
                ...B.entries()
              ].sort((H, st) => st[1] - H[1]).map(([H, st]) => `${st} \xD7 ${H}`).join(" \xB7 "), j.appendChild(nt);
            }
          }
          At && (Lt.classList.add("clickable"), Lt.addEventListener("click", () => {
            Z(At.x, At.y);
          }));
          for (const R of Vt) {
            const B = document.createElement("div"), nt = R.x != null && R.y != null;
            B.className = "sb-val-issue" + (nt ? " clickable" : "");
            const H = document.createElement("span");
            if (H.className = "sb-val-issue-msg", H.textContent = R.message, B.appendChild(H), nt) {
              const st = document.createElement("span");
              st.className = "sb-val-issue-coord", st.textContent = `${R.x}, ${R.y}`, B.appendChild(st), B.addEventListener("click", (wt) => {
                wt.stopPropagation();
                const Ot = B.classList.contains("pinned");
                tt.querySelectorAll(".sb-val-issue.pinned").forEach((Bt) => Bt.classList.remove("pinned")), Ot || B.classList.add("pinned"), Z(R.x, R.y);
              });
            } else B.style.opacity = "0.6";
            j.appendChild(B);
          }
          Rt.appendChild(Lt), Rt.appendChild(j), tt.appendChild(Rt);
        }
      }
    };
  }
  function Mb(n, t) {
    if (t.external_inputs.length > 0) {
      const l = document.createElement("div");
      l.className = "sb-ext-section-title", l.textContent = "External inputs", n.appendChild(l);
      for (const h of t.external_inputs) {
        const d = document.createElement("div");
        d.className = "sb-ext-flow", d.appendChild(pn(h.item, 14)), d.appendChild(document.createTextNode(fe(h.item)));
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
      const h = c.reduce((g, y) => g + Math.ceil(y.count), 0), d = document.createElement("div");
      d.className = "sb-machine-group";
      const u = document.createElement("div");
      u.className = "sb-machine-group-header", u.appendChild(pn(l, 16));
      const p = document.createElement("span");
      p.className = "sb-machine-group-name", p.textContent = fe(l), u.appendChild(p);
      const f = document.createElement("span");
      f.className = "sb-machine-group-count", f.textContent = `\xD7${h}`, u.appendChild(f), d.appendChild(u);
      const m = document.createElement("div");
      m.className = "sb-machine-group-body";
      for (const g of c) {
        const y = document.createElement("div");
        y.className = "sb-machine-flow", y.style.cssText = "color:#6b7280;margin-bottom:2px", y.appendChild(document.createTextNode("\u2192 ")), y.appendChild(pn(g.recipe, 13)), y.appendChild(document.createTextNode(fe(g.recipe))), m.appendChild(y), Ac(m, g.inputs, "flow-in", "\u25B6 "), Ac(m, g.outputs, "flow-out", "\u25C0 ");
      }
      d.appendChild(m), n.appendChild(d);
    }
    const s = document.createElement("div");
    s.className = "sb-status", s.style.cssText = "margin-top:6px";
    const i = t.machines.reduce((l, c) => l + Math.ceil(c.count), 0), r = t.dependency_order.length, o = document.createElement("span");
    o.textContent = `${i} machines`, s.appendChild(o);
    const a = document.createElement("span");
    if (a.textContent = `depth ${r}`, s.appendChild(a), n.appendChild(s), t.external_outputs.length > 0) for (const l of t.external_outputs) {
      const c = document.createElement("div");
      c.style.cssText = "display:flex;align-items:center;gap:4px;padding:2px 0;font-size:11px;color:#b5cea8", c.appendChild(pn(l.item, 13)), c.appendChild(document.createTextNode(fe(l.item)));
      const h = tu(l.rate);
      if (h) {
        const d = Qd(h.color);
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
  let Le = null, Aa = 0;
  const Qn = /* @__PURE__ */ new Map();
  let ce = null;
  const Ho = "spaghettio:sat-cache:v1";
  let un = new Uint8Array(0);
  function Pb() {
    try {
      const n = localStorage.getItem(Ho);
      if (!n) return new Uint8Array(0);
      const t = atob(n), e = new Uint8Array(t.length);
      for (let s = 0; s < t.length; s++) e[s] = t.charCodeAt(s);
      return e;
    } catch (n) {
      return console.warn("[engine] could not read SAT cache from localStorage", n), new Uint8Array(0);
    }
  }
  function Ib(n) {
    let t = "";
    for (let i = 0; i < n.length; i += 8192) t += String.fromCharCode.apply(null, Array.from(n.subarray(i, i + 8192)));
    const s = btoa(t);
    try {
      localStorage.setItem(Ho, s);
    } catch (i) {
      if (i instanceof DOMException && (i.name === "QuotaExceededError" || i.code === 22)) {
        console.warn("[engine] SAT cache quota exceeded \u2014 clearing");
        try {
          localStorage.removeItem(Ho);
        } catch {
        }
        un = new Uint8Array(0);
      } else console.warn("[engine] failed to persist SAT cache", i);
    }
  }
  function Rb(n) {
    const t = new Uint8Array(un.length + n.length);
    t.set(un, 0), t.set(n, un.length), un = t, Ib(un);
  }
  let Uo = [], uu = [], pu = /* @__PURE__ */ new Map(), jo = /* @__PURE__ */ new Set(), Vo = 0;
  function gn(n) {
    Vo += n;
    for (const t of jo) t(Vo);
  }
  function Lb(n) {
    return jo.add(n), n(Vo), () => jo.delete(n);
  }
  function Te(n, t) {
    if (!Le) throw new Error("Engine not initialized \u2014 call initEngine() first");
    const e = ++Aa;
    return gn(1), new Promise((s, i) => {
      Qn.set(e, {
        resolve: (r) => {
          gn(-1), ce === e && (ce = null), s(r);
        },
        reject: (r) => {
          gn(-1), ce === e && (ce = null), i(r);
        },
        onEvent: t
      }), Le.postMessage({
        id: e,
        ...n
      });
    });
  }
  async function fu() {
    if (Le) return;
    if (Le = new Worker(new URL("/spaghettio/assets/engine.worker-DGt5LJoM.js", import.meta.url), {
      type: "module",
      name: "spaghettio-engine"
    }), Le.onmessage = (t) => {
      const { id: e } = t.data, s = Qn.get(e);
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
        Qn.delete(e), t.data.ok ? s.resolve(t.data.result) : s.reject(new Error(t.data.error));
      }
    }, Le.onerror = (t) => {
      console.error("[engine.worker] error", t);
    }, await Te({
      method: "init"
    }), un = Pb(), un.length > 0) try {
      const t = await Te({
        method: "seedZoneCache",
        bytes: un
      });
      globalThis.__TRACE_LOGS === true && console.log(`[engine] seeded ${t} SAT zone records from localStorage`);
    } catch (t) {
      console.warn("[engine] seedZoneCache failed; persistence disabled this session", t);
    }
    Uo = await Te({
      method: "allProducibleItems"
    }), uu = await Te({
      method: "allProducerMachines"
    });
    const n = await Te({
      method: "defaultMachinesForItems",
      items: Uo,
      fallback: "assembling-machine-3"
    });
    pu = new Map(n);
  }
  async function $b(n, t, e, s, i) {
    return ce !== null && await ka(), Te({
      method: "solve",
      targetItem: n,
      targetRate: t,
      availableInputs: e,
      palette: s,
      defaultMachine: i
    });
  }
  function Bb() {
    return Uo;
  }
  function Ob() {
    return uu;
  }
  function Nb(n, t, e, s, i) {
    return Te({
      method: "layout",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null
    });
  }
  function Fb(n, t, e, s, i) {
    return Te({
      method: "layoutTraced",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null
    });
  }
  async function ka() {
    if (!Le) return;
    Le.terminate(), Le = null;
    const n = new Error("Engine superseded by a newer request");
    for (const [, t] of Qn) t.reject(n);
    Qn.clear(), ce = null, await fu();
  }
  async function Wb(n, t, e, s, i, r) {
    ce !== null && await ka();
    const o = ++Aa;
    return ce = o, gn(1), new Promise((a, l) => {
      Qn.set(o, {
        resolve: (h) => {
          gn(-1), ce === o && (ce = null), a(h);
        },
        reject: (h) => {
          gn(-1), ce === o && (ce = null), l(h);
        },
        onEvent: r
      });
      const c = globalThis.__TRACE_LOGS === true;
      Le.postMessage({
        id: o,
        method: "layoutStreaming",
        result: n,
        maxBeltTier: t ?? null,
        strategy: e ?? null,
        rowLayout: s ?? null,
        maxInserterTier: i ?? null,
        traceLogs: c
      });
    });
  }
  function Gb(n, t) {
    return Te({
      method: "exportBlueprint",
      layout: n,
      label: t
    });
  }
  function zb(n, t) {
    return pu.get(n) ?? t;
  }
  function Db(n, t) {
    return Te({
      method: "validateLayout",
      layout: n,
      solverResult: t
    });
  }
  function Hb(n, t) {
    return Te({
      method: "solveFixture",
      fixtureJson: n,
      pinsJson: JSON.stringify(t)
    });
  }
  function Ub(n) {
    return Te({
      method: "parseBlueprint",
      bp: n
    });
  }
  async function mu(n, t, e, s, i = 0) {
    if (ce !== null && await ka(), !Le) throw new Error("Engine not initialized");
    const r = ++Aa;
    return ce = r, gn(1), new Promise((o, a) => {
      Qn.set(r, {
        resolve: (l) => {
          gn(-1), ce === r && (ce = null), o(l);
        },
        reject: (l) => {
          gn(-1), ce === r && (ce = null), a(l);
        },
        onEvent: (l) => {
          const c = l;
          if (c.phase === "SatImprovement" && c.data) s(c.data);
          else if (c.phase === "SatOptimumProven" && c.data) {
            const h = c.data, d = h.record_bytes instanceof Uint8Array ? h.record_bytes : new Uint8Array(h.record_bytes);
            Rb(d);
          }
        }
      }), Le.postMessage({
        id: r,
        method: "improveRegionStreaming",
        layout: n,
        regionId: t,
        budgetMs: e,
        maxIters: i
      });
    });
  }
  async function jb(n, t) {
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
        e = await mu(e, o, t.perRegionBudgetMs, (l) => {
          var _a2;
          l.iter > 0 && (a = true), (_a2 = t.onImprovement) == null ? void 0 : _a2.call(t, l);
        }, 1), a ? i += 1 : s.delete(o);
      }
      if (i === 0) break;
    }
    return e;
  }
  function Vb(n, t) {
    return Te({
      method: "balancerShowcase",
      maxInputs: n,
      maxOutputs: t
    });
  }
  function Yb() {
    return {
      solve: $b,
      allProducibleItems: Bb,
      allProducerMachines: Ob,
      buildLayout: Nb,
      buildLayoutTraced: Fb,
      buildLayoutStreaming: Wb,
      exportBlueprint: Gb,
      defaultMachineForItem: zb,
      validateLayout: Db,
      solveFixture: Hb,
      improveRegion: mu,
      optimizeAllRegions: jb,
      balancerShowcase: Vb
    };
  }
  const Xb = `
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
  function qb() {
    if (document.getElementById("spaghettio-corpus-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-corpus-style", n.textContent = Xb, document.head.appendChild(n);
  }
  function Kb(n, t) {
    qb(), n.innerHTML = "";
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
    const m = document.createElement("label");
    m.textContent = "Or paste a blueprint string directly (parsed in-browser via WASM)", f.appendChild(m);
    const g = document.createElement("textarea");
    g.placeholder = "0eJyt... paste Factorio blueprint string", f.appendChild(g);
    const y = document.createElement("div");
    y.className = "corpus-paste-error", y.style.display = "none", f.appendChild(y), e.appendChild(f);
    let w = 0;
    g.addEventListener("input", () => {
      const M = g.value.trim(), O = ++w;
      if (!M) {
        g.classList.remove("error"), y.style.display = "none";
        return;
      }
      Ub(M).then((N) => {
        O === w && (g.classList.remove("error"), y.style.display = "none", t(N));
      }).catch((N) => {
        O === w && (g.classList.add("error"), y.textContent = String(N), y.style.display = "block");
      });
    });
    const x = document.createElement("div");
    x.className = "corpus-filters", x.style.display = "none";
    const _ = document.createElement("div");
    _.className = "corpus-filter-row";
    const v = document.createElement("label");
    v.textContent = "Search";
    const b = document.createElement("input");
    b.type = "text", b.placeholder = "filter by name\u2026", _.appendChild(v), _.appendChild(b), x.appendChild(_);
    const C = document.createElement("div");
    C.className = "corpus-filter-row";
    const T = document.createElement("label");
    T.textContent = "Product";
    const L = document.createElement("select");
    C.appendChild(T), C.appendChild(L), x.appendChild(C);
    const A = document.createElement("div");
    A.className = "corpus-filter-row";
    const E = document.createElement("input");
    E.type = "checkbox";
    const I = document.createElement("label");
    I.style.display = "flex", I.style.alignItems = "center", I.style.gap = "5px", I.style.cursor = "pointer", I.appendChild(E), I.appendChild(document.createTextNode("Bus layouts only")), A.appendChild(I), x.appendChild(A), e.appendChild(x);
    const V = document.createElement("div");
    V.className = "corpus-count", V.style.display = "none", e.appendChild(V);
    const G = document.createElement("div");
    G.className = "corpus-list", e.appendChild(G);
    const F = document.createElement("div");
    F.className = "corpus-stats", e.appendChild(F);
    function $() {
      i = s.filter((M) => !(o && !M.stats.is_bus_layout || a && a !== "__all__" && M.stats.final_product !== a || l && !M.name.toLowerCase().includes(l.toLowerCase()))), r = -1, z();
    }
    function z() {
      if (G.innerHTML = "", V.textContent = `${i.length} of ${s.length} blueprint(s)`, i.length === 0) {
        const M = document.createElement("div");
        M.className = "corpus-empty", M.textContent = s.length === 0 ? "No corpus loaded yet." : "No blueprints match the current filters.", G.appendChild(M), F.classList.remove("visible");
        return;
      }
      for (let M = 0; M < i.length; M++) {
        const O = i[M], N = document.createElement("div");
        N.className = "corpus-item" + (M === r ? " selected" : "");
        const D = document.createElement("div");
        D.className = "corpus-item-name", D.textContent = O.name, D.title = O.name, N.appendChild(D);
        const K = document.createElement("div");
        if (K.className = "corpus-item-meta", O.stats.is_bus_layout) {
          const at = document.createElement("span");
          at.className = "corpus-badge corpus-badge-bus", at.textContent = "BUS", K.appendChild(at);
        }
        if (O.stats.final_product) {
          const at = document.createElement("span");
          at.className = "corpus-badge corpus-badge-product", at.textContent = O.stats.final_product, K.appendChild(at);
        }
        const tt = document.createElement("span");
        tt.className = "corpus-badge corpus-badge-machines", tt.textContent = `${O.stats.machine_count}m`, K.appendChild(tt), N.appendChild(K);
        const ot = M;
        N.addEventListener("click", () => J(ot)), G.appendChild(N);
      }
    }
    function J(M) {
      r = M;
      const O = i[M];
      z(), t(O.layout), X(O);
    }
    function X(M) {
      F.innerHTML = "", F.classList.add("visible");
      const O = document.createElement("div");
      O.className = "corpus-stats-title", O.textContent = M.name, O.title = M.name, F.appendChild(O);
      const N = document.createElement("div");
      N.className = "corpus-stats-grid";
      const D = M.stats, K = [
        [
          "machines",
          String(D.machine_count)
        ],
        [
          "recipes",
          String(D.recipe_count)
        ],
        [
          "is_bus",
          D.is_bus_layout ? "yes" : "no"
        ],
        [
          "density",
          D.density.toFixed(2)
        ]
      ];
      D.is_bus_layout && K.push([
        "bus_lanes",
        String(D.bus_lane_count)
      ], [
        "bus_pitch",
        D.bus_pitch.toFixed(1)
      ], [
        "row_pitch",
        D.row_pitch.toFixed(1)
      ], [
        "rows",
        String(D.row_count)
      ]), K.push([
        "bbox",
        `${D.bbox_width}\xD7${D.bbox_height}`
      ], [
        "belt_tiles",
        String(D.belt_tiles)
      ]), D.pipe_tiles > 0 && K.push([
        "pipe_tiles",
        String(D.pipe_tiles)
      ]);
      for (const [tt, ot] of K) {
        const at = document.createElement("div");
        at.className = "corpus-stats-row";
        const mt = document.createElement("span");
        mt.className = "corpus-stats-key", mt.textContent = tt;
        const bt = document.createElement("span");
        bt.className = "corpus-stats-val", bt.textContent = ot, at.appendChild(mt), at.appendChild(bt), N.appendChild(at);
      }
      F.appendChild(N);
    }
    function Q() {
      const M = new Set(s.map((N) => N.stats.final_product).filter(Boolean));
      L.innerHTML = "";
      const O = document.createElement("option");
      O.value = "__all__", O.textContent = "All products", L.appendChild(O);
      for (const N of Array.from(M).sort()) {
        const D = document.createElement("option");
        D.value = N, D.textContent = N, L.appendChild(D);
      }
    }
    u.addEventListener("change", () => {
      var _a2;
      const M = (_a2 = u.files) == null ? void 0 : _a2[0];
      if (!M) return;
      const O = new FileReader();
      O.onload = (N) => {
        var _a3;
        try {
          s = JSON.parse((_a3 = N.target) == null ? void 0 : _a3.result).blueprints ?? [], i = s, r = -1, p.textContent = `Reload corpus.json (${s.length} blueprints)`, x.style.display = "", V.style.display = "", F.classList.remove("visible"), Q(), $();
        } catch (D) {
          alert(`Failed to parse corpus.json: ${D}`);
        }
      }, O.readAsText(M);
    }), b.addEventListener("input", () => {
      l = b.value, $();
    }), L.addEventListener("change", () => {
      a = L.value, $();
    }), E.addEventListener("change", () => {
      o = E.checked, $();
    }), z();
  }
  const Jb = [
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
      label: "Advanced Circuit (ores, AM2)",
      item: "advanced-circuit",
      rate: 5,
      inputs: [
        "coal",
        "water",
        "crude-oil",
        "iron-ore",
        "copper-ore"
      ],
      machine: "assembling-machine-2",
      beltTier: "transport-belt",
      tier: 4,
      status: "solved",
      desc: "Full stack from raw ores, AM2 + yellow belts (0 errors / 0 warnings)"
    },
    {
      label: "Advanced Circuit (from plates)",
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
      desc: "5+ recipes, mixed solid/fluid \u2014 still has lane-throughput warnings"
    },
    {
      label: "Processing Unit",
      item: "processing-unit",
      rate: 2,
      inputs: [
        "iron-ore",
        "copper-ore",
        "stone",
        "coal",
        "water",
        "crude-oil"
      ],
      machine: "assembling-machine-3",
      beltTier: "fast-transport-belt",
      tier: 5,
      status: "wip",
      desc: "Deep chain, multiple fluids \u2014 scoped regression tests passing"
    }
  ], Zb = `
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@300;400;500;600;700&family=Space+Grotesk:wght@400;500;600;700&display=swap');

.spaghettio-landing {
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

.spaghettio-landing::before {
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

.spaghettio-landing-inner {
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

.spaghettio-landing-header {
  text-align: center;
  margin-bottom: 56px;
}

.spaghettio-landing-title {
  font-family: 'Space Grotesk', sans-serif;
  font-size: 52px;
  font-weight: 700;
  color: #f0f0f0;
  letter-spacing: -2px;
  margin: 0 0 8px;
  line-height: 1;
}

.spaghettio-landing-title span {
  background: linear-gradient(135deg, #38bdf8 0%, #818cf8 50%, #c084fc 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.spaghettio-landing-subtitle {
  font-size: 13px;
  font-weight: 300;
  color: #6b7280;
  letter-spacing: 2.5px;
  text-transform: uppercase;
  margin: 0;
}

/* Ladder */

.spaghettio-landing-ladder {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 48px;
}

.spaghettio-landing-ladder-header {
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

.spaghettio-landing-card {
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

.spaghettio-landing-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: transparent;
  transition: background 0.2s ease;
}

.spaghettio-landing-card:hover {
  background: rgba(255,255,255,0.05);
  border-color: rgba(255,255,255,0.08);
}

.spaghettio-landing-card.solved:hover::before { background: #34d399; }
.spaghettio-landing-card.partial:hover::before { background: #fbbf24; }
.spaghettio-landing-card.wip { opacity: 0.4; cursor: default; }
.spaghettio-landing-card.wip:hover { background: rgba(255,255,255,0.02); border-color: rgba(255,255,255,0.04); }
.spaghettio-landing-card.loading { pointer-events: none; }

.spaghettio-landing-tier {
  font-size: 11px;
  font-weight: 600;
  color: #4b5563;
}
.spaghettio-landing-tier span {
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

.spaghettio-landing-card-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.spaghettio-landing-card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
  color: #e5e7eb;
}

.spaghettio-landing-card-icon {
  width: 22px;
  height: 22px;
  image-rendering: pixelated;
  flex-shrink: 0;
}

.spaghettio-landing-card-rate {
  font-size: 11px;
  color: #6b7280;
  font-weight: 300;
}

.spaghettio-landing-card-desc {
  font-size: 11px;
  color: #4b5563;
  font-weight: 300;
}

.spaghettio-landing-status {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  padding: 3px 8px;
  border-radius: 3px;
  text-align: center;
  justify-self: center;
}
.spaghettio-landing-status.solved { background: rgba(52,211,153,0.12); color: #34d399; }
.spaghettio-landing-status.partial { background: rgba(251,191,36,0.12); color: #fbbf24; }
.spaghettio-landing-status.wip { background: rgba(107,114,128,0.12); color: #6b7280; }

.spaghettio-landing-entities {
  font-size: 11px;
  color: #4b5563;
  text-align: right;
  font-weight: 300;
}

/* Footer */

.spaghettio-landing-footer {
  margin-top: 16px;
  text-align: center;
}

.spaghettio-landing-launch {
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
.spaghettio-landing-launch:hover {
  background: rgba(255,255,255,0.08);
  color: #e5e7eb;
  border-color: rgba(255,255,255,0.15);
}
.spaghettio-landing-launch svg {
  width: 16px;
  height: 16px;
}

/* Modal */

.spaghettio-preview-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.75);
  backdrop-filter: blur(8px);
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: spaghettio-fadeIn 0.2s ease forwards;
}

@keyframes spaghettio-fadeIn { to { opacity: 1; } }

.spaghettio-preview-modal {
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
  animation: spaghettio-modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  transform: scale(0.95) translateY(10px);
  opacity: 0;
}

@keyframes spaghettio-modalIn { to { transform: scale(1) translateY(0); opacity: 1; } }

.spaghettio-preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid #1f2937;
  background: rgba(255,255,255,0.02);
  flex-shrink: 0;
}

.spaghettio-preview-title {
  font-size: 13px;
  font-weight: 500;
  color: #9ca3af;
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: 'JetBrains Mono', monospace;
}

.spaghettio-preview-title img {
  width: 18px;
  height: 18px;
  image-rendering: pixelated;
}

.spaghettio-preview-stats {
  display: flex;
  gap: 16px;
  font-size: 11px;
  color: #4b5563;
  font-family: 'JetBrains Mono', monospace;
}

.spaghettio-preview-stats span { color: #6b7280; }

.spaghettio-preview-close {
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
.spaghettio-preview-close:hover {
  background: rgba(255,255,255,0.06);
  color: #e5e7eb;
  border-color: #555;
}

.spaghettio-preview-canvas {
  flex: 1;
  position: relative;
  overflow: hidden;
  background: #111;
}

.spaghettio-preview-canvas canvas {
  display: block;
}

.spaghettio-preview-badge {
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

.spaghettio-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #1f2937;
  border-top-color: #38bdf8;
  border-radius: 50%;
  animation: spaghettio-spin 0.6s linear infinite;
  display: inline-block;
}

@keyframes spaghettio-spin { to { transform: rotate(360deg); } }
`;
  function Qb() {
    if (document.getElementById("spaghettio-landing-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-landing-style", n.textContent = Zb, document.head.appendChild(n);
  }
  function t_(n, t, e) {
    Qb(), n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "spaghettio-landing", n.appendChild(s);
    const i = document.createElement("div");
    i.className = "spaghettio-landing-inner", s.appendChild(i);
    const r = document.createElement("div");
    r.className = "spaghettio-landing-header", i.appendChild(r);
    const o = document.createElement("h1");
    o.className = "spaghettio-landing-title", o.innerHTML = "Spagh<span>ettio</span>", r.appendChild(o);
    const a = document.createElement("p");
    a.className = "spaghettio-landing-subtitle", a.textContent = "Automated Factory Blueprint Generator", r.appendChild(a);
    const l = document.createElement("div");
    l.className = "spaghettio-landing-ladder", i.appendChild(l);
    const c = document.createElement("div");
    c.className = "spaghettio-landing-ladder-header", c.innerHTML = "<span>Tier</span><span>Recipe</span><span>Status</span><span>Entities</span>", l.appendChild(c);
    for (const u of Jb) {
      const p = document.createElement("div");
      p.className = `spaghettio-landing-card ${u.status}`;
      const f = document.createElement("div");
      f.className = "spaghettio-landing-tier", f.innerHTML = `<span>${u.tier}</span>`, p.appendChild(f);
      const m = document.createElement("div");
      m.className = "spaghettio-landing-card-body";
      const g = document.createElement("div");
      g.className = "spaghettio-landing-card-title";
      const y = document.createElement("img");
      y.src = `/spaghettio/icons/${u.item}.png`, y.className = "spaghettio-landing-card-icon", y.onerror = () => {
        y.style.display = "none";
      }, g.appendChild(y), g.appendChild(document.createTextNode(u.label));
      const w = document.createElement("span");
      w.className = "spaghettio-landing-card-rate", w.textContent = `${u.rate}/s`, g.appendChild(w), m.appendChild(g);
      const x = document.createElement("div");
      x.className = "spaghettio-landing-card-desc", x.textContent = u.desc, m.appendChild(x), p.appendChild(m);
      const _ = document.createElement("div");
      _.className = `spaghettio-landing-status ${u.status}`, _.textContent = u.status === "solved" ? "Solved" : u.status === "partial" ? "Partial" : "WIP", p.appendChild(_);
      const v = document.createElement("div");
      v.className = "spaghettio-landing-entities", v.textContent = "\u2014", p.appendChild(v), u.status !== "wip" && p.addEventListener("click", () => {
        e_(t, u, p, v);
      }), l.appendChild(p);
    }
    const h = document.createElement("div");
    h.className = "spaghettio-landing-footer", i.appendChild(h);
    const d = document.createElement("button");
    d.className = "spaghettio-landing-launch", d.innerHTML = '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>Open Generator', d.addEventListener("click", () => {
      s.style.transition = "opacity 0.3s ease", s.style.opacity = "0", setTimeout(() => {
        s.remove(), e.onOpenGenerator();
      }, 300);
    }), h.appendChild(d);
  }
  function e_(n, t, e, s) {
    e.classList.contains("loading") || (e.classList.add("loading"), s.innerHTML = '<span class="spaghettio-spinner"></span>', (async () => {
      let i, r;
      try {
        const o = n.defaultMachineForItem(t.item, t.machine);
        i = await n.solve(t.item, t.rate, t.inputs, {}, o), r = await n.buildLayout(i, t.beltTier);
      } catch (o) {
        e.classList.remove("loading"), s.textContent = "error", console.error("Landing solve/layout failed:", o);
        return;
      }
      s.textContent = String(r.entities.length), e.classList.remove("loading"), $d(i.machines.map((o) => ({
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
      }))), n_(t, r, i).catch((o) => {
        console.error("Modal init failed:", o);
      });
    })());
  }
  async function n_(n, t, e) {
    const s = document.createElement("div");
    s.className = "spaghettio-preview-backdrop", document.body.appendChild(s);
    const i = (T) => {
      T.key === "Escape" && a();
    };
    let r = false, o = null;
    function a() {
      r || (r = true, document.removeEventListener("keydown", i), o && o.destroy(true), s.remove());
    }
    document.addEventListener("keydown", i), s.addEventListener("click", (T) => {
      T.target === s && a();
    });
    const l = document.createElement("div");
    l.className = "spaghettio-preview-modal", s.appendChild(l);
    const c = document.createElement("div");
    c.className = "spaghettio-preview-header";
    const h = document.createElement("div");
    h.className = "spaghettio-preview-title";
    const d = document.createElement("img");
    d.src = `/spaghettio/icons/${n.item}.png`, d.onerror = () => {
      d.style.display = "none";
    }, h.appendChild(d), h.appendChild(document.createTextNode(` ${n.label} \u2014 ${n.rate}/s`)), c.appendChild(h);
    const u = document.createElement("div");
    u.className = "spaghettio-preview-stats";
    const p = `${t.width ?? 0}\xD7${t.height ?? 0}`, f = e.machines.reduce((T, L) => T + Math.ceil(L.count), 0);
    u.innerHTML = `<span>${f} machines</span><span>${p} tiles</span>`, c.appendChild(u);
    const m = document.createElement("button");
    m.className = "spaghettio-preview-close", m.textContent = "\xD7", m.addEventListener("click", a), c.appendChild(m), l.appendChild(c);
    const g = document.createElement("div");
    g.className = "spaghettio-preview-canvas", l.appendChild(g);
    const y = document.createElement("div");
    y.className = "spaghettio-preview-badge", y.textContent = `0 / ${t.entities.length}`, g.appendChild(y), o = new ea(), await o.init({
      resizeTo: g,
      background: 1118481,
      antialias: true
    }), g.insertBefore(o.canvas, y), o.canvas.addEventListener("contextmenu", (T) => T.preventDefault());
    const w = (t.width ?? 20) * S, x = (t.height ?? 20) * S, _ = Math.max(w, x, 600) + 200, v = new Pd({
      screenWidth: g.clientWidth,
      screenHeight: g.clientHeight,
      worldWidth: _,
      worldHeight: _,
      events: o.renderer.events
    });
    v.drag({
      mouseButtons: "left"
    }).pinch().wheel().decelerate(), o.stage.addChild(v);
    const b = new Ut();
    v.addChild(b), v.fit(true, w * 1.15, x * 1.2), v.moveCenter(w / 2, x / 2);
    const { renderLayoutAnimated: C } = await Pn(async () => {
      const { renderLayoutAnimated: T } = await import("./animated-DzIPMOki.js");
      return {
        renderLayoutAnimated: T
      };
    }, []);
    C(t, b, y, () => {
    });
  }
  var Ee = Uint8Array, gs = Uint16Array, s_ = Int32Array, gu = new Ee([
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
  ]), yu = new Ee([
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
  ]), i_ = new Ee([
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
  ]), xu = function(n, t) {
    for (var e = new gs(31), s = 0; s < 31; ++s) e[s] = t += 1 << n[s - 1];
    for (var i = new s_(e[30]), s = 1; s < 30; ++s) for (var r = e[s]; r < e[s + 1]; ++r) i[r] = r - e[s] << 5 | s;
    return {
      b: e,
      r: i
    };
  }, bu = xu(gu, 2), _u = bu.b, r_ = bu.r;
  _u[28] = 258, r_[258] = 28;
  var o_ = xu(yu, 0), a_ = o_.b, Yo = new gs(32768);
  for (var Yt = 0; Yt < 32768; ++Yt) {
    var vn = (Yt & 43690) >> 1 | (Yt & 21845) << 1;
    vn = (vn & 52428) >> 2 | (vn & 13107) << 2, vn = (vn & 61680) >> 4 | (vn & 3855) << 4, Yo[Yt] = ((vn & 65280) >> 8 | (vn & 255) << 8) >> 1;
  }
  var ei = (function(n, t, e) {
    for (var s = n.length, i = 0, r = new gs(t); i < s; ++i) n[i] && ++r[n[i] - 1];
    var o = new gs(t);
    for (i = 1; i < t; ++i) o[i] = o[i - 1] + r[i - 1] << 1;
    var a;
    if (e) {
      a = new gs(1 << t);
      var l = 15 - t;
      for (i = 0; i < s; ++i) if (n[i]) for (var c = i << 4 | n[i], h = t - n[i], d = o[n[i] - 1]++ << h, u = d | (1 << h) - 1; d <= u; ++d) a[Yo[d] >> l] = c;
    } else for (a = new gs(s), i = 0; i < s; ++i) n[i] && (a[i] = Yo[o[n[i] - 1]++] >> 15 - n[i]);
    return a;
  }), gi = new Ee(288);
  for (var Yt = 0; Yt < 144; ++Yt) gi[Yt] = 8;
  for (var Yt = 144; Yt < 256; ++Yt) gi[Yt] = 9;
  for (var Yt = 256; Yt < 280; ++Yt) gi[Yt] = 7;
  for (var Yt = 280; Yt < 288; ++Yt) gi[Yt] = 8;
  var wu = new Ee(32);
  for (var Yt = 0; Yt < 32; ++Yt) wu[Yt] = 5;
  var l_ = ei(gi, 9, 1), c_ = ei(wu, 5, 1), io = function(n) {
    for (var t = n[0], e = 1; e < n.length; ++e) n[e] > t && (t = n[e]);
    return t;
  }, ze = function(n, t, e) {
    var s = t / 8 | 0;
    return (n[s] | n[s + 1] << 8) >> (t & 7) & e;
  }, ro = function(n, t) {
    var e = t / 8 | 0;
    return (n[e] | n[e + 1] << 8 | n[e + 2] << 16) >> (t & 7);
  }, h_ = function(n) {
    return (n + 7) / 8 | 0;
  }, d_ = function(n, t, e) {
    return (e == null || e > n.length) && (e = n.length), new Ee(n.subarray(t, e));
  }, u_ = [
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
  ], De = function(n, t, e) {
    var s = new Error(t || u_[n]);
    if (s.code = n, Error.captureStackTrace && Error.captureStackTrace(s, De), !e) throw s;
    return s;
  }, p_ = function(n, t, e, s) {
    var i = n.length, r = 0;
    if (!i || t.f && !t.l) return e || new Ee(0);
    var o = !e, a = o || t.i != 2, l = t.i;
    o && (e = new Ee(i * 3));
    var c = function(Y) {
      var et = e.length;
      if (Y > et) {
        var lt = new Ee(Math.max(et * 2, Y));
        lt.set(e), e = lt;
      }
    }, h = t.f || 0, d = t.p || 0, u = t.b || 0, p = t.l, f = t.d, m = t.m, g = t.n, y = i * 8;
    do {
      if (!p) {
        h = ze(n, d, 1);
        var w = ze(n, d + 1, 3);
        if (d += 3, w) if (w == 1) p = l_, f = c_, m = 9, g = 5;
        else if (w == 2) {
          var b = ze(n, d, 31) + 257, C = ze(n, d + 10, 15) + 4, T = b + ze(n, d + 5, 31) + 1;
          d += 14;
          for (var L = new Ee(T), A = new Ee(19), E = 0; E < C; ++E) A[i_[E]] = ze(n, d + E * 3, 7);
          d += C * 3;
          for (var I = io(A), V = (1 << I) - 1, G = ei(A, I, 1), E = 0; E < T; ) {
            var F = G[ze(n, d, V)];
            d += F & 15;
            var x = F >> 4;
            if (x < 16) L[E++] = x;
            else {
              var $ = 0, z = 0;
              for (x == 16 ? (z = 3 + ze(n, d, 3), d += 2, $ = L[E - 1]) : x == 17 ? (z = 3 + ze(n, d, 7), d += 3) : x == 18 && (z = 11 + ze(n, d, 127), d += 7); z--; ) L[E++] = $;
            }
          }
          var J = L.subarray(0, b), X = L.subarray(b);
          m = io(J), g = io(X), p = ei(J, m, 1), f = ei(X, g, 1);
        } else De(1);
        else {
          var x = h_(d) + 4, _ = n[x - 4] | n[x - 3] << 8, v = x + _;
          if (v > i) {
            l && De(0);
            break;
          }
          a && c(u + _), e.set(n.subarray(x, v), u), t.b = u += _, t.p = d = v * 8, t.f = h;
          continue;
        }
        if (d > y) {
          l && De(0);
          break;
        }
      }
      a && c(u + 131072);
      for (var Q = (1 << m) - 1, M = (1 << g) - 1, O = d; ; O = d) {
        var $ = p[ro(n, d) & Q], N = $ >> 4;
        if (d += $ & 15, d > y) {
          l && De(0);
          break;
        }
        if ($ || De(2), N < 256) e[u++] = N;
        else if (N == 256) {
          O = d, p = null;
          break;
        } else {
          var D = N - 254;
          if (N > 264) {
            var E = N - 257, K = gu[E];
            D = ze(n, d, (1 << K) - 1) + _u[E], d += K;
          }
          var tt = f[ro(n, d) & M], ot = tt >> 4;
          tt || De(3), d += tt & 15;
          var X = a_[ot];
          if (ot > 3) {
            var K = yu[ot];
            X += ro(n, d) & (1 << K) - 1, d += K;
          }
          if (d > y) {
            l && De(0);
            break;
          }
          a && c(u + 131072);
          var at = u + D;
          if (u < X) {
            var mt = r - X, bt = Math.min(X, at);
            for (mt + u < 0 && De(3); u < bt; ++u) e[u] = s[mt + u];
          }
          for (; u < at; ++u) e[u] = e[u - X];
        }
      }
      t.l = p, t.p = O, t.b = u, t.f = h, p && (h = 1, t.m = m, t.d = f, t.n = g);
    } while (!h);
    return u != e.length && o ? d_(e, 0, u) : e.subarray(0, u);
  }, f_ = new Ee(0), m_ = function(n) {
    (n[0] != 31 || n[1] != 139 || n[2] != 8) && De(6, "invalid gzip data");
    var t = n[3], e = 10;
    t & 4 && (e += (n[10] | n[11] << 8) + 2);
    for (var s = (t >> 3 & 1) + (t >> 4 & 1); s > 0; s -= !n[e++]) ;
    return e + (t & 2);
  }, g_ = function(n) {
    var t = n.length;
    return (n[t - 4] | n[t - 3] << 8 | n[t - 2] << 16 | n[t - 1] << 24) >>> 0;
  };
  function y_(n, t) {
    var e = m_(n);
    return e + 8 > n.length && De(6, "invalid gzip data"), p_(n.subarray(e, -8), {
      i: 2
    }, new Ee(g_(n)), t);
  }
  var x_ = typeof TextDecoder < "u" && new TextDecoder(), b_ = 0;
  try {
    x_.decode(f_, {
      stream: true
    }), b_ = 1;
  } catch {
  }
  const oo = "fls1";
  async function vu(n) {
    const t = typeof n == "string" ? n : new TextDecoder().decode(n);
    if (!t.startsWith(oo)) throw new Error(`Not a layout snapshot: expected "${oo}" prefix, got "${t.slice(0, 4)}"`);
    const e = t.slice(oo.length), s = Uint8Array.from(atob(e), (o) => o.charCodeAt(0)), i = y_(s), r = new TextDecoder().decode(i);
    return JSON.parse(r);
  }
  function __(n) {
    return new Promise((t, e) => {
      const s = new FileReader();
      s.onload = () => t(s.result), s.onerror = () => e(new Error("Failed to read file")), s.readAsText(n);
    });
  }
  function w_(n, t) {
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
          const i = await __(s), r = await vu(i);
          t(r);
        } catch (i) {
          alert(`Failed to load snapshot: ${i}`);
        }
      }
    });
  }
  function v_(n, t, e) {
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
      const w = document.createElement("span");
      w.style.cssText = "color:#ff6b6b;margin-left:8px", w.textContent = "\u26A0 Incomplete trace", s.appendChild(w);
    }
    if (a.truncated) {
      const w = document.createElement("span");
      w.style.cssText = "color:#ff6b6b;margin-left:4px", w.textContent = "\u26A0 Validation truncated", s.appendChild(w);
    }
    const p = a.issues.filter((w) => w.severity === "Error").length, f = a.issues.length - p;
    if (a.issues.length > 0) {
      const w = document.createElement("span");
      w.style.cssText = "margin-left:8px", w.innerHTML = `<span style="color:#f66">${p} errors</span> <span style="color:#fa0">${f} warnings</span>`, s.appendChild(w);
    }
    const m = document.createElement("span");
    m.style.cssText = "flex:1", s.appendChild(m);
    const g = document.createElement("button");
    g.textContent = "Re-solve", g.title = "Not yet implemented", g.disabled = true, g.style.cssText = "background:#222;border:1px solid #444;color:#666;padding:2px 8px;border-radius:3px;font:11px monospace;cursor:not-allowed", s.appendChild(g);
    const y = document.createElement("button");
    return y.textContent = "Clear", y.style.cssText = "background:#333;border:1px solid #666;color:#ccc;padding:2px 8px;border-radius:3px;cursor:pointer;font:11px monospace", y.addEventListener("click", () => e.onClear()), s.appendChild(y), n.insertBefore(s, n.firstChild), s;
  }
  function Pc(n, t, e, s, i, r, o, a) {
    const l = s - t, c = i - e, h = Math.sqrt(l * l + c * c);
    if (h === 0) return;
    const d = l / h, u = c / h;
    let p = 0;
    for (; p < h; ) {
      const f = Math.min(p + r, h);
      n.moveTo(t + d * p, e + u * p).lineTo(t + d * f, e + u * f).stroke(a), p = f + o;
    }
  }
  function C_(n, t, e, s, i) {
    const r = new Ut(), o = n.find((h) => h.phase === "LanesPlanned");
    if (o) for (const h of o.data.lanes) {
      const d = new ft(), u = h.x * S;
      d.rect(u, 0, S, e * S).fill({
        color: h.is_fluid ? 4500223 : 4521864,
        alpha: 0.04
      }), d.eventMode = "static", d.on("pointerenter", () => i(`Lane: ${h.item} @ x=${h.x} (${h.rate.toFixed(1)}/s${h.is_fluid ? " fluid" : ""})`)), d.on("pointerleave", () => i(null)), r.addChild(d);
    }
    const a = n.find((h) => h.phase === "RowsPlaced");
    if (a) for (const h of a.data.rows) {
      const d = new ft(), u = h.y_end * S;
      d.moveTo(0, u).lineTo(t * S, u).stroke({
        width: 1,
        color: 6982234,
        alpha: 0.3
      }), d.eventMode = "static", d.on("pointerenter", () => i(`Row ${h.index}: ${h.recipe} (${h.machine_count}\xD7 ${h.machine})`)), d.on("pointerleave", () => i(null)), r.addChild(d);
    }
    for (const h of n) {
      if (h.phase !== "BalancerStamped") continue;
      const d = h.data, u = (d.y_end - d.y_start) * S;
      if (u <= 0) continue;
      const p = new ft();
      p.rect(0, d.y_start * S, t * S, u).fill({
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
      const d = h.data, u = new ft();
      u.moveTo(d.from_x * S + S / 2, d.from_y * S + S / 2).lineTo(d.to_x * S + S / 2, d.to_y * S + S / 2).stroke({
        width: 2,
        color: 8978244,
        alpha: 0.5
      }), u.eventMode = "static", u.on("pointerenter", () => i(`Tap-off: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) len=${d.path_len}`)), u.on("pointerleave", () => i(null)), r.addChild(u);
    }
    for (const h of n) {
      if (h.phase !== "MergerBlockPlaced") continue;
      const d = h.data, u = new ft();
      u.rect(0, d.block_y * S, t * S, d.block_height * S).fill({
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
      const d = h.data, u = d.from_x * S + S / 2, p = d.from_y * S + S / 2, f = 3, m = new ft();
      m.label = "RouteFailure", m.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), m.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), Pc(m, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), m.eventMode = "static", m.on("pointerenter", () => i(`Route failed: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) [${d.spec_key}]`)), m.on("pointerleave", () => i(null)), r.addChild(m);
    }
    const l = S_;
    let c = 0;
    for (const h of n) {
      if (h.phase !== "GhostSpecRouted") continue;
      const d = h.data, u = l[c % l.length];
      c++;
      const p = new ft();
      if (d.tiles && d.tiles.length > 1) {
        p.setStrokeStyle({
          width: 3,
          color: u,
          alpha: 0.7
        }), p.moveTo(d.tiles[0][0] * S + S / 2, d.tiles[0][1] * S + S / 2);
        for (let f = 1; f < d.tiles.length; f++) p.lineTo(d.tiles[f][0] * S + S / 2, d.tiles[f][1] * S + S / 2);
        p.stroke();
      }
      p.eventMode = "static", p.on("pointerenter", () => i(`Ghost path: ${d.spec_key} len=${d.path_len} crossings=${d.crossings} turns=${d.turns}`)), p.on("pointerleave", () => i(null)), r.addChild(p);
    }
    for (const h of n) {
      if (h.phase !== "GhostSpecFailed") continue;
      const d = h.data, u = d.from_x * S + S / 2, p = d.from_y * S + S / 2, f = 4, m = new ft();
      m.label = "RouteFailure", m.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), m.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), Pc(m, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), m.eventMode = "static", m.on("pointerenter", () => i(`Ghost failed: ${d.spec_key} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y})`)), m.on("pointerleave", () => i(null)), r.addChild(m);
    }
    for (const h of n) {
      if (h.phase !== "GhostClusterSolved" && h.phase !== "GhostClusterFailed") continue;
      const d = h.phase === "GhostClusterFailed", u = d ? null : h.data, p = d ? h.data : null, f = u ?? p, m = d ? 16729156 : 4500223, g = new ft();
      g.rect(f.zone_x * S, f.zone_y * S, f.zone_w * S, f.zone_h * S).fill({
        color: m,
        alpha: d ? 0.15 : 0.08
      }).stroke({
        width: d ? 2 : 1,
        color: m,
        alpha: d ? 0.9 : 0.6
      }), g.eventMode = "static";
      const y = u ? ` vars=${u.variables} clauses=${u.clauses} ${(u.solve_time_us / 1e3).toFixed(1)}ms` : "";
      g.on("pointerenter", () => i(`Cluster #${f.cluster_id}: ${f.zone_w}x${f.zone_h} @ (${f.zone_x},${f.zone_y}) ${f.boundary_count} ports${y}${d ? " FAILED" : ""}`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    return s.addChild(r), r;
  }
  const S_ = [
    5676246,
    6996096,
    13672512,
    11567312
  ], T_ = {
    Error: 16729156,
    Warning: 16755200
  }, E_ = 0.85;
  function A_(n, t, e) {
    const s = new Ut(), i = /* @__PURE__ */ new Map();
    for (const r of n) {
      if (r.x == null || r.y == null) continue;
      const o = T_[r.severity] ?? 4500223, a = new ft();
      a.rect(r.x * S, r.y * S, S, S).stroke({
        width: 2,
        color: o,
        alpha: E_
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
  function k_(n) {
    const t = Math.max(0, Math.min(1, n)), e = Math.round(64 + 96 * t);
    return {
      color: 255 << 16 | e << 8 | 32,
      alpha: 0.45 * (1 - t) + 0.12
    };
  }
  function M_(n, t, e) {
    const s = /* @__PURE__ */ new Map();
    for (const o of n) {
      if (o.x == null || o.y == null || !o.detail) continue;
      const { delivered: a, needed: l } = o.detail;
      if (!(l > 0)) continue;
      const c = Math.max(0, Math.min(1, a / l)), h = `${o.x},${o.y}`, d = s.get(h);
      (d === void 0 || c < d) && s.set(h, c);
    }
    if (s.size === 0) return null;
    const i = /* @__PURE__ */ new Map();
    for (const o of t) {
      const a = Ne[o.name];
      a && i.set(`${o.x},${o.y}`, a);
    }
    const r = new Ut();
    r.eventMode = "none";
    for (const [o, a] of s) {
      const [l, c] = o.split(",").map(Number), h = i.get(o), [d, u] = h ?? [
        1,
        1
      ], { color: p, alpha: f } = k_(a), m = new ft();
      m.rect(l * S, c * S, d * S, u * S).fill({
        color: p,
        alpha: f
      }), r.addChild(m);
    }
    return e.addChild(r), r;
  }
  function Ic(n) {
    const [t, e] = Ne[n.name] ?? [
      1,
      1
    ], s = n.x ?? 0, i = n.y ?? 0;
    return {
      cx: (s + t / 2) * S,
      cy: (i + e / 2) * S
    };
  }
  function P_(n, t, e) {
    if (!n || n.length === 0) return null;
    const s = new Ut();
    s.eventMode = "none";
    const i = new ft();
    for (const [r, o] of n) {
      const a = t[r], l = t[o];
      if (!a || !l) continue;
      const { cx: c, cy: h } = Ic(a), { cx: d, cy: u } = Ic(l);
      i.moveTo(c, h), i.lineTo(d, u);
    }
    return i.stroke({
      color: 16751164,
      width: 1.5,
      alpha: 0.85
    }), s.addChild(i), e.addChild(s), s;
  }
  function I_(n) {
    const t = n.point.direction;
    return t === "East" || t === "West" ? "horizontal" : "vertical";
  }
  function R_(n) {
    const t = new Set(n.map(I_));
    return t.size === 1 ? [
      ...t
    ][0] : "mixed";
  }
  function L_(n) {
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
    for (const a of e.values()) a.axis = R_([
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
  function $_(n) {
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
  function B_(n) {
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
  const O_ = 0.35;
  function N_(n, t, e, s, i) {
    const r = S * 0.45;
    n.setStrokeStyle({
      width: 3,
      color: i,
      alpha: O_
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
  function ao(n) {
    return [
      n.point.x,
      n.point.y
    ];
  }
  function F_(n, t, e, s, i, r, o = 2, a = 6, l = 4, c = 0.9) {
    const h = s - t, d = i - e, u = Math.hypot(h, d);
    if (u < 0.5) return;
    const p = h / u, f = d / u;
    n.setStrokeStyle({
      width: o,
      color: r,
      alpha: c
    });
    let m = 0;
    for (; m < u; ) {
      const g = m, y = Math.min(m + a, u);
      n.moveTo(t + p * g, e + f * g).lineTo(t + p * y, e + f * y).stroke(), m = y + l;
    }
  }
  function W_(n) {
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
  function G_(n) {
    const t = new Ut(), e = n.regions ?? [], s = [];
    if (e.length === 0) return {
      layer: t,
      items: s,
      hitTest: () => null
    };
    for (const r of e) {
      const o = L_(r), a = $_(r.kind), l = B_(o.cls), c = r.x * S, h = r.y * S, d = r.width * S, u = r.height * S;
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
      const p = new ft(), f = r.kind === "crossing_zone" ? 0.06 : 0.14;
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
      const m = r.ports ?? [], g = W_(m);
      for (const { item: y, inPort: w, outPort: x } of g) {
        const [_, v] = ao(w), [b, C] = ao(x), T = _ * S + S / 2, L = v * S + S / 2, A = b * S + S / 2, E = C * S + S / 2, I = Rn(y), V = new ft();
        F_(V, T, L, A, E, I), t.addChild(V);
      }
      for (const y of m) {
        const [w, x] = ao(y), _ = w * S + S / 2, v = x * S + S / 2, b = new ft(), C = y.item ? Rn(y.item) : 8947848;
        N_(b, _, v, y.point.direction, C), t.addChild(b);
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
  const z_ = /* @__PURE__ */ new Set([
    "JunctionGrowthStarted",
    "JunctionGrowthIteration",
    "JunctionStrategyAttempt",
    "SatInvocation",
    "JunctionSolved",
    "JunctionGrowthCapped",
    "RegionWalkerVeto"
  ]);
  function D_(n) {
    return z_.has(n.phase);
  }
  function H_(n) {
    const t = n.data;
    return typeof t.seed_x == "number" && typeof t.seed_y == "number" ? [
      t.seed_x,
      t.seed_y
    ] : [
      t.tile_x,
      t.tile_y
    ];
  }
  function U_(n, t) {
    return `${n},${t}`;
  }
  function j_(n) {
    const t = /* @__PURE__ */ new Map(), e = (a, l) => {
      const c = U_(a, l);
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
      if (!D_(a)) continue;
      const [l, c] = H_(a), h = e(l, c);
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
          const d = i(h, a.data.iter, a.data.variant), { seed_x: u, seed_y: p, iter: f, variant: m, ...g } = a.data;
          d.sat = g;
          break;
        }
        case "RegionWalkerVeto": {
          const d = i(h, a.data.growth_iter, a.data.variant), { tile_x: u, tile_y: p, growth_iter: f, variant: m, ...g } = a.data;
          d.veto = g;
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
  function Rc(n) {
    return n.iterations.length === 0 ? null : n.iterations[n.defaultIterIndex] ?? n.iterations[n.iterations.length - 1];
  }
  function V_(n) {
    return n ? `${n.entity_name}@(${n.entity_x},${n.entity_y}) ${n.direction}` : "";
  }
  function Y_(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function X_(n, t, e) {
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
        item: Y_(c.spec_key),
        specKey: c.spec_key,
        tiles: h
      });
    }
    return a;
  }
  const Lc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, q_ = new Be({
    fontFamily: "monospace",
    fontSize: 10,
    fill: 16777215,
    dropShadow: {
      color: 0,
      distance: 1,
      blur: 2,
      alpha: 0.8
    }
  }), K_ = new Be({
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
  function J_(n) {
    const t = new Ut(), e = [], s = [];
    for (const o of n) {
      if (o.outcome.kind !== "Solved") continue;
      const a = Rc(o);
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
      const a = Rc(o);
      if (!a) continue;
      const l = a.bbox;
      if (l.w <= 0 || l.h <= 0 || o.outcome.kind === "Capped" && i(o.seed.x, o.seed.y)) continue;
      const c = l.x * S, h = l.y * S, d = l.w * S, u = l.h * S, p = Lc[o.outcome.kind] ?? Lc.Open, f = new ft();
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
      const m = new Ue({
        text: `Junction (${o.seed.x},${o.seed.y})`,
        style: q_
      });
      m.x = c + 3, m.y = h + 2, t.addChild(m);
      const g = new Ue({
        text: Z_(o),
        style: K_
      });
      g.x = c + 3, g.y = h + u - g.height - 2, t.addChild(g), e.push({
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
  function Z_(n) {
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
  const $c = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, Q_ = 0.04, tw = 9060416, ew = 0.55, nw = 4243680, sw = 5592405, iw = 0.55, rw = 16777215, Bc = 0.85, ow = 16777215, Oc = new Be({
    fontFamily: "monospace",
    fontSize: 7,
    fontWeight: "700",
    fill: rw
  }), Nc = /* @__PURE__ */ new Map();
  function aw(n) {
    let t = Nc.get(n);
    if (!t) {
      const s = `/spaghettio/icons/${n}.png`;
      t = je.load(s).catch(() => null), Nc.set(n, t);
    }
    return t;
  }
  function lw() {
    const n = new Ut();
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
      const { cluster: a, iter: l } = r, c = l.bbox, h = $c[a.outcome.kind] ?? $c.Open, d = new ft();
      d.rect(c.x * S, c.y * S, c.w * S, c.h * S).fill({
        color: h,
        alpha: Q_
      }), n.addChild(d);
      const u = cw(c.x * S, c.y * S, c.w * S, c.h * S, {
        dashLen: S * 0.45,
        gapLen: S * 0.25,
        width: 3,
        color: h,
        alpha: 0.95
      });
      n.addChild(u);
      const p = new ft();
      p.setStrokeStyle({
        width: 1,
        color: tw,
        alpha: ew
      });
      for (const [g, y] of l.forbidden) hw(p, g * S, y * S, S);
      n.addChild(p);
      const f = new ft();
      f.circle((a.seed.x + 0.5) * S, (a.seed.y + 0.5) * S, S * 0.42).stroke({
        width: 3,
        color: nw,
        alpha: 0.95
      }), n.addChild(f);
      const m = ((_a2 = l.sat) == null ? void 0 : _a2.boundaries) ?? l.boundaries;
      for (const g of m) uw(n, g, o, () => t);
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
  function cw(n, t, e, s, i) {
    const r = new ft();
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
      const d = c - a, u = h - l, p = Math.hypot(d, u), f = d / p, m = u / p, g = i.dashLen + i.gapLen;
      for (let y = 0; y < p; y += g) {
        const w = Math.min(y + i.dashLen, p);
        r.moveTo(a + f * y, l + m * y).lineTo(a + f * w, l + m * w).stroke();
      }
    }
    return r;
  }
  function hw(n, t, e, s) {
    const i = s;
    n.moveTo(t, e + i).lineTo(t + i, e).stroke(), n.moveTo(t + i / 3, e + i).lineTo(t + i, e + 2 * i / 3).stroke(), n.moveTo(t, e + 2 * i / 3).lineTo(t + 2 * i / 3, e).stroke();
  }
  function dw(n, t) {
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
  function uw(n, t, e, s) {
    const i = t.x * S, r = t.y * S, o = S / 3, a = dw(t.is_input, t.direction), l = a === "top" || a === "bottom";
    let c = i, h = r, d = S, u = S;
    a === "top" ? u = o : a === "bottom" ? (h = r + S - o, u = o) : (a === "left" || (c = i + S - o), d = o);
    const p = new ft();
    p.rect(c, h, d, u).fill({
      color: sw,
      alpha: iw
    }), t.interior && (p.setStrokeStyle({
      width: 1,
      color: ow,
      alpha: 0.5
    }), p.rect(c, h, d, u).stroke()), n.addChild(p);
    const f = pw(t.direction), [m, g, y, w] = l ? [
      c + d / 6,
      h + u / 2,
      c + d * 5 / 6,
      h + u / 2
    ] : [
      c + d / 2,
      h + u / 6,
      c + d / 2,
      h + u * 5 / 6
    ], x = new Ue({
      text: f,
      style: Oc
    });
    x.anchor.set(0.5), x.x = m, x.y = g, x.alpha = Bc, n.addChild(x);
    const _ = new Ue({
      text: f,
      style: Oc
    });
    _.anchor.set(0.5), _.x = y, _.y = w, _.alpha = Bc, n.addChild(_);
    const v = c + d / 2, b = h + u / 2;
    aw(t.item).then((C) => {
      if (e !== s() || !C) return;
      const T = new Ye(C);
      T.anchor.set(0.5), T.x = v, T.y = b;
      const L = o * 0.95;
      T.width = L, T.height = L, n.addChild(T);
    });
  }
  function pw(n) {
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
  const fw = 4247776, mw = 0.18;
  function gw(n) {
    if (!n || n.length === 0) return null;
    const t = /* @__PURE__ */ new Set();
    for (const i of n) if (i.phase === "GhostSpecRouted") for (const [r, o] of i.data.tiles) t.add(`${r},${o}`);
    if (t.size === 0) return null;
    const e = new Ut();
    e.label = "ghost-tiles-overlay";
    const s = new ft();
    for (const i of t) {
      const [r, o] = i.split(","), a = Number(r), l = Number(o);
      s.rect(a * S, l * S, S, S).fill({
        color: fw,
        alpha: mw
      });
    }
    return e.addChild(s), e;
  }
  function yw(n, t, e) {
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
    const m = document.createElement("button");
    m.className = "jd-step-btn", m.textContent = "\u25B6", m.title = "next iteration (\u2192)";
    const g = document.createElement("button");
    g.className = "jd-step-btn jd-terminal-btn", g.textContent = "\u21BA", g.title = "jump to default (terminal) iteration", u.append(p, f, m, g);
    const y = document.createElement("div");
    y.className = "jd-inline-summary", s.append(i, u, y), n.append(s);
    const w = document.createElement("div");
    w.className = "jd-modal-backdrop";
    const x = document.createElement("div");
    x.className = "jd-modal";
    const _ = document.createElement("div");
    _.className = "jd-titlebar";
    const v = document.createElement("span");
    v.className = "jd-title", v.textContent = "Junction details";
    const b = document.createElement("span");
    b.className = "jd-status-pill";
    const C = document.createElement("span");
    C.className = "jd-close", C.textContent = "\xD7", C.title = "Close details (Esc)", _.append(v, b, C);
    const T = document.createElement("div");
    T.className = "jd-detail";
    const L = document.createElement("div");
    L.className = "jd-footer", L.textContent = "Esc close \xB7 \u2190/\u2192 step all \xB7 w/s iter \xB7 a/d variant \xB7 Home/End first/last", x.append(_, T, L), n.append(w, x);
    let A = null, E = 0, I = null, V = false;
    function G(Y, et) {
      A = Y, I = et ?? null, E = Y.defaultIterIndex, s.classList.add("jd-open"), tt(), K(), M();
    }
    function F() {
      A && (z(), A = null, I = null, s.classList.remove("jd-open"), e.onChange(null));
    }
    function $() {
      !A || V || (V = true, w.classList.add("jd-open"), x.classList.add("jd-open"), ot());
    }
    function z() {
      V && (V = false, w.classList.remove("jd-open"), x.classList.remove("jd-open"));
    }
    function J() {
      V ? z() : $();
    }
    function X() {
      return A !== null;
    }
    function Q(Y) {
      if (!A) return;
      const et = Math.max(0, Math.min(A.iterations.length - 1, Y));
      et !== E && (E = et, tt(), K(), M());
    }
    function M() {
      if (!A) return;
      const Y = A.iterations[E];
      if (!Y) return;
      const et = t.toScreen(Y.bbox.x * S, Y.bbox.y * S), lt = t.toScreen((Y.bbox.x + Y.bbox.w) * S, (Y.bbox.y + Y.bbox.h) * S), ct = n.getBoundingClientRect();
      if (!(lt.x < 0 || et.x > ct.width || lt.y < 0 || et.y > ct.height)) return;
      const Et = (Y.bbox.x + Y.bbox.w / 2) * S, jt = (Y.bbox.y + Y.bbox.h / 2) * S;
      t.moveCenter(Et, jt);
    }
    function O() {
      const Y = /* @__PURE__ */ new Map(), et = A;
      if (!et) return Y;
      for (let lt = 0; lt < et.iterations.length; lt++) {
        const ct = et.iterations[lt], _t = Y.get(ct.iter) ?? [];
        _t.push(lt), Y.set(ct.iter, _t);
      }
      return Y;
    }
    function N(Y) {
      if (!A) return;
      const et = O(), lt = Array.from(et.keys()).sort((Z, it) => Z - it), ct = A.iterations[E].iter, _t = lt.indexOf(ct), Et = lt[Math.max(0, Math.min(lt.length - 1, _t + Y))], jt = et.get(Et) ?? [], U = jt.find((Z) => A.iterations[Z].variant === "");
      Q(U ?? jt[0] ?? E);
    }
    function D(Y) {
      if (!A) return;
      const et = O(), lt = A.iterations[E].iter, ct = et.get(lt) ?? [];
      if (ct.length <= 1) return;
      const Et = (ct.indexOf(E) + Y + ct.length) % ct.length;
      Q(ct[Et]);
    }
    function K() {
      if (!A) return;
      const Y = A.iterations[E];
      if (!Y) return;
      const et = n.getBoundingClientRect(), lt = s.offsetWidth || 200, ct = s.offsetHeight || 70, _t = (Y.bbox.x + Y.bbox.w) * S, Et = (Y.bbox.y + Y.bbox.h) * S, jt = Y.bbox.y * S, U = t.toScreen(_t, Et), Z = t.toScreen(_t, jt);
      let it = U.x - lt, pt = U.y;
      pt + ct > et.height - 4 && (pt = Z.y - ct), it = Math.max(4, Math.min(it, et.width - lt - 4)), pt = Math.max(4, Math.min(pt, et.height - ct - 4)), s.style.left = `${it}px`, s.style.top = `${pt}px`;
    }
    t.on("moved", K), t.on("zoomed", K), window.addEventListener("resize", K);
    function tt() {
      if (!A) return;
      const Y = A, et = Y.iterations[E];
      r.textContent = `Junction (${Y.seed.x},${Y.seed.y})`, o.className = `jd-status-pill jd-${Y.outcome.kind.toLowerCase()}`, o.textContent = Fc(Y);
      const lt = Y.iterations.length, ct = et && et.variant ? ` \xB7 ${et.variant}` : "";
      f.textContent = `iter ${et ? et.iter : "-"}${ct} \xB7 ${E + 1}/${lt}`, p.disabled = E <= 0, m.disabled = E >= lt - 1, g.disabled = E === Y.defaultIterIndex, y.innerHTML = "";
      for (const _t of bw(Y, et)) {
        const Et = document.createElement("div");
        Et.className = `jd-inline-summary-row jd-inline-summary-row--${_t.tone}`, Et.textContent = _t.text, y.appendChild(Et);
      }
      V && ot(), et && e.onChange({
        cluster: Y,
        iter: et,
        trace: I
      });
    }
    function ot() {
      if (!A) return;
      const Y = A, et = Y.iterations[E];
      b.className = `jd-status-pill jd-${Y.outcome.kind.toLowerCase()}`, b.textContent = Fc(Y), v.textContent = `Junction (${Y.seed.x},${Y.seed.y})`, ww(T, Y, et);
    }
    d.addEventListener("click", F), a.addEventListener("click", J), l.addEventListener("click", bt), c.addEventListener("click", () => {
      if (!A) return;
      const Y = Xo(A, A.iterations[E], I);
      Mw(c, Y);
    }), h.addEventListener("click", () => {
      if (!A || !e.onEditRequested) return;
      const Y = A.iterations[E];
      Y && e.onEditRequested({
        cluster: A,
        iter: Y,
        trace: I
      });
    }), C.addEventListener("click", z), w.addEventListener("click", z);
    function at() {
      if (!A) return null;
      const Y = A.iterations[E];
      return Y ? {
        cluster: A,
        iter: Y,
        trace: I
      } : null;
    }
    function mt(Y) {
      s.classList.toggle("jd-edit-mode", Y), c.disabled = Y, l.disabled = Y, h.disabled = Y;
    }
    function bt() {
      var _a2;
      if (!A) return;
      const Y = xw(A, E), et = JSON.stringify(Y, (ct, _t) => typeof _t == "bigint" ? String(_t) : _t, 2), lt = (ct) => {
        const _t = l.textContent;
        l.textContent = ct ? "\u2713" : "!", l.classList.add("jd-inline-btn--flash"), window.setTimeout(() => {
          l.textContent = _t, l.classList.remove("jd-inline-btn--flash");
        }, 900);
      };
      if ((_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText) navigator.clipboard.writeText(et).then(() => lt(true), () => lt(false));
      else {
        const ct = document.createElement("textarea");
        ct.value = et, ct.style.position = "fixed", ct.style.opacity = "0", document.body.appendChild(ct), ct.select();
        try {
          document.execCommand("copy"), lt(true);
        } catch {
          lt(false);
        }
        document.body.removeChild(ct);
      }
    }
    return p.addEventListener("click", () => Q(E - 1)), m.addEventListener("click", () => Q(E + 1)), g.addEventListener("click", () => {
      A && Q(A.defaultIterIndex);
    }), document.addEventListener("keydown", (Y) => {
      var _a2, _b2;
      if (!X()) return;
      const et = (_b2 = (_a2 = Y.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (et === "INPUT" || et === "TEXTAREA" || et === "SELECT") return;
      const lt = Y.key, ct = () => {
        Y.stopImmediatePropagation(), Y.preventDefault();
      };
      lt === "Escape" ? (V ? z() : F(), ct()) : lt === "ArrowLeft" ? (Q(E - 1), ct()) : lt === "ArrowRight" ? (Q(E + 1), ct()) : lt === "Home" ? (Q(0), ct()) : lt === "End" && A ? (Q(A.iterations.length - 1), ct()) : lt === "w" || lt === "W" ? (N(-1), ct()) : lt === "s" || lt === "S" ? (N(1), ct()) : lt === "a" || lt === "A" ? (D(-1), ct()) : lt === "d" || lt === "D" ? (D(1), ct()) : (lt === "i" || lt === "I") && (J(), ct());
    }, {
      capture: true
    }), {
      open: G,
      close: F,
      isOpen: X,
      inlineEl: s,
      getSelection: at,
      setEditMode: mt
    };
  }
  function xw(n, t) {
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
  function Fc(n) {
    switch (n.outcome.kind) {
      case "Solved":
        return `Solved \xB7 ${n.outcome.regionTiles}t`;
      case "Capped":
        return `Capped \xB7 ${n.outcome.iters} iter`;
      case "Open":
        return "Open";
    }
  }
  function bw(n, t) {
    if (t) {
      if (t.veto) return [
        {
          text: `veto \xB7 ${Wi(t.veto.broken_segment, 22)} @ (${t.veto.break_tile_x},${t.veto.break_tile_y})`,
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
        const i = s.detail ? ` \xB7 ${Wi(s.detail, 28)}` : "";
        return [
          {
            text: `${s.strategy} \u2192 ${Wi(s.outcome, 12)}${i}`,
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
            text: `cap: ${Wi(n.outcome.reason, 32)}`,
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
  function Wi(n, t) {
    return n.length <= t ? n : `${n.slice(0, t - 1)}\u2026`;
  }
  function Cu(n) {
    var _a2;
    return ((_a2 = n.sat) == null ? void 0 : _a2.boundaries) ?? n.boundaries;
  }
  function _w(n) {
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
  function ww(n, t, e) {
    n.innerHTML = "", n.appendChild(vw(t)), n.appendChild(Cw(t, e)), e && (n.appendChild(Sw(e)), n.appendChild(Tw(e)), n.appendChild(Ew(e)), e.veto && n.appendChild(Aw(e))), t.nearbyStamped.length > 0 && n.appendChild(kw(t));
  }
  function ts(n, t = true) {
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
  function vw(n) {
    const { details: t, bodyEl: e } = ts("Summary"), s = document.createElement("div");
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
  function Cw(n, t) {
    const { details: e, bodyEl: s } = ts("Participating specs");
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
  function Sw(n) {
    const t = Cu(n), e = n.sat ? " (as fed to SAT)" : " (spec perimeter)", { details: s, bodyEl: i } = ts(`Boundaries${e}`);
    if (t.length === 0) {
      const r = document.createElement("div");
      return r.className = "jd-row jd-row--dim", r.textContent = "(none)", i.appendChild(r), s;
    }
    for (const r of t) {
      const o = document.createElement("div");
      o.className = "jd-row";
      const a = r.is_input ? "IN " : "OUT", l = r.interior ? " (interior)" : "", c = r.external_feeder ? ` \u2190 ${V_(r.external_feeder)}` : "";
      o.style.color = r.is_input ? "#9f9" : "#f99";
      const h = r.spec_key ? ` \xB7 ${r.spec_key}` : "";
      o.textContent = `${a} (${r.x},${r.y}) ${_w(r.direction)} ${r.direction} \xB7 ${r.item}${l}${h}${c}`, i.appendChild(o);
    }
    return s;
  }
  function Tw(n) {
    const { details: t, bodyEl: e } = ts("Strategy attempts");
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
  function Ew(n) {
    const { details: t, bodyEl: e } = ts("SAT", !!n.sat);
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
  function Aw(n) {
    const { details: t, bodyEl: e } = ts("Walker veto");
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
  function kw(n) {
    const { details: t, bodyEl: e } = ts("Nearby stamped", false);
    for (const s of n.nearbyStamped) {
      const i = document.createElement("div");
      i.className = "jd-row";
      const r = s.carries ? ` carries=${s.carries}` : "", o = s.segment_id ? ` \xB7 seg=${s.segment_id}` : "";
      i.textContent = `(${s.x},${s.y}) ${s.name} ${s.direction}${r}${o}${s.feeds_seed_area ? "  \u26A0 feeds seed" : ""}`, e.appendChild(i);
    }
    return t;
  }
  const Su = {
    "transport-belt": 4,
    "fast-transport-belt": 6,
    "express-transport-belt": 8
  };
  function Xo(n, t, e, s) {
    var _a2, _b2;
    const i = n.seed, r = (t == null ? void 0 : t.bbox) ?? {
      x: i.x,
      y: i.y,
      w: 1,
      h: 1
    }, o = (t == null ? void 0 : t.iter) ?? 0, a = ((_a2 = t == null ? void 0 : t.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", l = ((_b2 = t == null ? void 0 : t.sat) == null ? void 0 : _b2.max_reach) ?? Su[a] ?? 4, c = Cu(t ?? {
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
    })), h = t && e ? X_(e, r, 2).map((p) => ({
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
  function Mw(n, t) {
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
  function Pw(n) {
    return n.x === void 0 || n.y === void 0 || n.direction === void 0 ? null : n;
  }
  const Gi = {
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
  }, Wc = {
    North: "East",
    East: "South",
    South: "West",
    West: "North"
  }, zi = {
    "transport-belt": "transport-belt",
    "fast-transport-belt": "fast-transport-belt",
    "express-transport-belt": "express-transport-belt"
  }, Gc = {
    "transport-belt": "underground-belt",
    "fast-transport-belt": "fast-underground-belt",
    "express-transport-belt": "express-underground-belt"
  };
  function Iw(n) {
    const { viewport: t, canvas: e, engine: s, jd: i, satZoneOverlayLayer: r } = n;
    let o = null, a = null, l = null, c = null, h = null, d = null, u = false, p = null, f = [], m = [], g = [], y = [], w = "belt", x = "East", _ = 0, v = null, b = "idle", C = 0, T = null, L = null;
    function A(k, P) {
      return `${k},${P}`;
    }
    function E(k, P) {
      if (!p) return false;
      const q = p.bbox;
      return k >= q.x && k < q.x + q.w && P >= q.y && P < q.y + q.h;
    }
    function I(k, P) {
      return f.find((q) => q.x === k && q.y === P);
    }
    function V() {
      if (!p || p.items.length === 0) return null;
      const k = Math.max(0, Math.min(_, p.items.length - 1));
      return p.items[k] ?? null;
    }
    function G(k, P, q) {
      if (!p) return null;
      const [j, R] = Gi[q], B = k - j, nt = P - R, H = I(B, nt);
      if (H && H.direction === q && (zi[H.name] === H.name || H.io_type === "output")) return H.carries ?? null;
      const st = p.boundaries.find((wt) => wt.x === k && wt.y === P && wt.isInput && wt.dir === q);
      return st ? st.item : null;
    }
    function F(k, P) {
      return G(k.x, k.y, P) ?? V();
    }
    function $(k, P) {
      if (!p) return null;
      const [q, j] = Gi[P];
      for (let R = 1; R <= p.maxReach + 1; R++) {
        const B = k.x - q * R, nt = k.y - j * R, H = I(B, nt);
        if (H && H.io_type === "input" && H.direction === P) return H.carries ?? null;
      }
      return null;
    }
    function z() {
      m.push(f.map((k) => ({
        ...k
      }))), m.length > 64 && m.shift(), g.length = 0;
    }
    function J(k, P) {
      z(), f = k, Q(), ot();
    }
    function X() {
      if (!p) return [];
      const k = zi[p.beltTier] ?? "transport-belt", P = [];
      for (const q of p.boundaries) {
        if (!q.isInput) continue;
        const [j, R] = Gi[q.dir];
        P.push({
          name: k,
          x: q.x - j,
          y: q.y - R,
          direction: q.dir,
          carries: q.item
        });
      }
      return P;
    }
    function Q() {
      const k = X();
      o && ti({
        entities: f
      }, o, void 0, void 0, void 0, k), a && ti({
        entities: y
      }, a, void 0, void 0, void 0, k), M();
    }
    function M() {
      if (c) {
        c.removeChildren();
        for (const k of f) {
          const P = k.carries;
          if (!P) continue;
          const q = `/spaghettio/icons/${P}.png`, j = je.get(q);
          if (!j) continue;
          const R = new Ye(j), B = S * 0.55;
          R.width = B, R.height = B, R.x = k.x * S + (S - B) / 2, R.y = k.y * S + (S - B) / 2, R.alpha = 0.85, c.addChild(R);
        }
      }
    }
    function O(k, P) {
      l && (ti({
        entities: k
      }, l, void 0, void 0, void 0, X()), l.alpha = P ? 0.5 : 0.45, l.tint = P ? 16733525 : 16777215);
    }
    function N() {
      l && (l.removeChildren(), l.tint = 16777215);
    }
    function D(k, P = "") {
      if (b = k, !d) return;
      d.classList.remove("ok", "solving", "invalid", "idle"), d.classList.add(k === "valid" ? "ok" : k);
      const q = k === "valid" ? "\u25CF" : k === "solving" ? "\u25D4" : k === "invalid" ? "\u25CF" : "\u25CB";
      d.textContent = q;
      let j = "";
      k === "valid" ? j = L !== null ? `valid \xB7 cost ${L} / yours ${K(f)}` : "valid" : k === "solving" ? j = "solving\u2026" : k === "invalid" ? j = "invalid" : j = "no edits yet", d.title = P ? `${j}
${P}` : j, Ce();
    }
    function K(k) {
      let P = 0;
      for (const q of k) zi[q.name] === q.name ? P += 1 : Gc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] === q.name && (P += 5);
      return P;
    }
    function tt() {
      if (!p) return "no zone";
      const k = /* @__PURE__ */ new Set();
      for (const P of f) {
        const q = A(P.x, P.y);
        if (k.has(q)) return `duplicate entity at (${P.x},${P.y})`;
        if (k.add(q), !E(P.x, P.y)) return `entity at (${P.x},${P.y}) outside bbox`;
        if (p.forbidden.has(q)) return `entity at (${P.x},${P.y}) on forbidden tile`;
      }
      for (const P of f) {
        if (P.io_type !== "input") continue;
        const [q, j] = Gi[P.direction];
        let R = false;
        for (let B = 1; B <= p.maxReach + 1; B++) {
          const nt = P.x + q * B, H = P.y + j * B, st = I(nt, H);
          if (st) {
            if (st.io_type === "output" && st.direction === P.direction && st.carries === P.carries) {
              R = true;
              break;
            }
            if (st.io_type === "input" && st.carries === P.carries) return `UG-in at (${P.x},${P.y}) blocked by another UG-in at (${nt},${H})`;
          }
        }
        if (!R) return `UG-in at (${P.x},${P.y}) has no matching UG-out within reach ${p.maxReach}`;
      }
      return null;
    }
    function ot() {
      if (!u || !p) return;
      const k = tt();
      if (k) {
        y = [], L = null, Q(), D("invalid", k);
        return;
      }
      D("solving"), T !== null && window.clearTimeout(T);
      const P = ++C;
      T = window.setTimeout(() => {
        at(P);
      }, 300);
    }
    async function at(k) {
      if (p) try {
        const P = await s.solveFixture(p.fixtureJson, f);
        if (k !== C || !u) return;
        if (!P) {
          y = [], L = null, Q(), D("invalid", "SAT cannot complete this layout");
          return;
        }
        const q = new Set(f.map((R) => A(R.x, R.y))), j = [];
        for (const R of P.entities) {
          const B = Pw(R);
          B && !q.has(A(B.x, B.y)) && j.push(B);
        }
        y = j, L = P.cost, Q(), D("valid");
      } catch (P) {
        if (k !== C) return;
        y = [], Q(), D("invalid", `solver error: ${P instanceof Error ? P.message : String(P)}`);
      }
    }
    function mt(k) {
      const P = e.getBoundingClientRect(), q = t.toWorld(k.clientX - P.left, k.clientY - P.top);
      return {
        x: Math.floor(q.x / S),
        y: Math.floor(q.y / S)
      };
    }
    function bt(k, P, q) {
      const j = [], R = P.x === k.x ? 0 : P.x > k.x ? 1 : -1, B = P.y === k.y ? 0 : P.y > k.y ? 1 : -1;
      if (q) {
        for (let H = k.y; H !== P.y + B && B !== 0; H += B) j.push({
          x: k.x,
          y: H
        });
        B === 0 && j.push({
          x: k.x,
          y: k.y
        });
        for (let H = k.x + R; R !== 0 && H !== P.x + R; H += R) j.push({
          x: H,
          y: P.y
        });
      } else {
        for (let H = k.x; H !== P.x + R && R !== 0; H += R) j.push({
          x: H,
          y: k.y
        });
        R === 0 && j.push({
          x: k.x,
          y: k.y
        });
        for (let H = k.y + B; B !== 0 && H !== P.y + B; H += B) j.push({
          x: P.x,
          y: H
        });
      }
      const nt = [];
      for (const H of j) {
        const st = nt[nt.length - 1];
        (!st || st.x !== H.x || st.y !== H.y) && nt.push(H);
      }
      return nt;
    }
    function Y(k, P) {
      return k.x === P.x && k.y === P.y - 1 ? "South" : k.x === P.x && k.y === P.y + 1 ? "North" : k.y === P.y && k.x === P.x - 1 ? "East" : k.y === P.y && k.x === P.x + 1 ? "West" : null;
    }
    function et(k) {
      if (!p || k.length === 0) return null;
      for (const st of k) if (!E(st.x, st.y)) return null;
      const P = (st) => !p.forbidden.has(A(st.x, st.y)), q = k[0], j = k[k.length - 1];
      if (!q || !j || !P(q) || !P(j)) return null;
      const R = [], B = k.length > 1 ? Y(k[0], k[1]) ?? x : x, nt = F(k[0], B);
      let H = 0;
      for (; H < k.length; ) {
        const st = k[H], wt = k[H + 1] ?? null, Ot = wt ? Y(st, wt) : H > 0 ? Y(k[H - 1], st) : x;
        if (!Ot) return null;
        let Bt = H + 1;
        for (; Bt < k.length && P(k[Bt]); ) Bt++;
        if (Bt === k.length) {
          R.push(lt(st, Ot, nt)), H++;
          continue;
        }
        let qt = Bt;
        for (; qt < k.length && !P(k[qt]); ) qt++;
        if (qt === k.length) return null;
        const ge = k[Bt - 1], es = k[qt];
        if (Math.abs(es.x - ge.x) + Math.abs(es.y - ge.y) > p.maxReach + 1) return null;
        for (let Fe = H; Fe < Bt - 1; Fe++) {
          const ns = k[Fe], bn = Y(ns, k[Fe + 1]);
          if (!bn) return null;
          R.push(lt(ns, bn, nt));
        }
        const Is = Y(ge, es);
        if (!Is) return null;
        R.push(ct(ge, Is, "input", nt)), R.push(ct(es, Is, "output", nt)), H = qt + 1;
      }
      return R;
    }
    function lt(k, P, q) {
      return {
        name: zi[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "transport-belt",
        x: k.x,
        y: k.y,
        direction: P,
        carries: q ?? void 0
      };
    }
    function ct(k, P, q, j) {
      return {
        name: Gc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "underground-belt",
        x: k.x,
        y: k.y,
        direction: P,
        io_type: q,
        carries: j ?? void 0
      };
    }
    function _t(k) {
      const P = new Set(k.map((j) => A(j.x, j.y))), q = f.filter((j) => !P.has(A(j.x, j.y))).concat(k);
      J(q);
    }
    function Et(k) {
      if (!p || !E(k.x, k.y) || !f.some((q) => q.x === k.x && q.y === k.y)) return;
      const P = f.filter((q) => !(q.x === k.x && q.y === k.y));
      J(P);
    }
    function jt(k) {
      if (!p) return;
      const P = p.boundaries.find((j) => j.x === k.x && j.y === k.y && j.isInput);
      if (!P) return;
      const q = p.items.indexOf(P.item);
      q >= 0 && q !== _ && (_ = q, se());
    }
    function U(k) {
      if (!u || !p || k.button !== 0) return;
      const P = mt(k);
      if (!P || !E(P.x, P.y)) return;
      if (jt(P), w === "erase") {
        Et(P), k.stopPropagation(), k.preventDefault();
        return;
      }
      if (w === "ug-in" || w === "ug-out") {
        const R = w === "ug-in" ? G(P.x, P.y, x) ?? V() : $(P, x) ?? G(P.x, P.y, x) ?? V(), B = w === "ug-in" ? ct(P, x, "input", R) : ct(P, x, "output", R), nt = f.filter((H) => !(H.x === P.x && H.y === P.y)).concat(B);
        J(nt), k.stopPropagation(), k.preventDefault();
        return;
      }
      v = {
        startX: P.x,
        startY: P.y,
        bendVerticalFirst: false
      };
      const q = bt({
        x: P.x,
        y: P.y
      }, P, false), j = et(q);
      O(j ?? [], j === null), k.stopPropagation(), k.preventDefault();
    }
    function Z(k) {
      if (!u || !v || !p) return;
      const P = mt(k);
      if (!P) return;
      const q = bt({
        x: v.startX,
        y: v.startY
      }, P, v.bendVerticalFirst), j = et(q);
      O(j ?? [], j === null);
    }
    function it(k) {
      if (!u || !v || !p) {
        v = null, N();
        return;
      }
      const P = mt(k);
      if (!P) {
        v = null, N();
        return;
      }
      const q = bt({
        x: v.startX,
        y: v.startY
      }, P, v.bendVerticalFirst), j = et(q);
      if (v = null, N(), !j) {
        D("invalid", "drag rejected: out of bounds, on obstacle, or UG too long");
        return;
      }
      _t(j), k.stopPropagation(), k.preventDefault();
    }
    function pt(k) {
      var _a2, _b2;
      if (!u) return;
      const P = (_b2 = (_a2 = k.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (P === "INPUT" || P === "TEXTAREA" || P === "SELECT") return;
      const q = () => {
        k.stopImmediatePropagation(), k.preventDefault();
      };
      if (k.key === "Escape") {
        Lt(), q();
        return;
      }
      if (k.key === "1") {
        It("belt"), q();
        return;
      }
      if (k.key === "2") {
        It("ug-in"), q();
        return;
      }
      if (k.key === "3") {
        It("ug-out"), q();
        return;
      }
      if (k.key === "0") {
        It("erase"), q();
        return;
      }
      if (k.key === "r" || k.key === "R") {
        v ? v.bendVerticalFirst = !v.bendVerticalFirst : (x = Wc[x], se()), q();
        return;
      }
      if (k.key === "[" && p) {
        _ = (_ - 1 + p.items.length) % p.items.length, se(), q();
        return;
      }
      if (k.key === "]" && p) {
        _ = (_ + 1) % p.items.length, se(), q();
        return;
      }
      if ((k.key === "Enter" || k.key === "a" || k.key === "A") && b === "valid" && y.length > 0) {
        Vt(), q();
        return;
      }
      if ((k.ctrlKey || k.metaKey) && (k.key === "z" || k.key === "Z")) {
        k.shiftKey ? Dt() : zt(), q();
        return;
      }
    }
    function It(k) {
      w = k, se();
    }
    function zt() {
      m.length !== 0 && (g.push(f), f = m.pop(), Q(), ot());
    }
    function Dt() {
      g.length !== 0 && (m.push(f), f = g.pop(), Q(), ot());
    }
    function Vt() {
      if (y.length === 0) return;
      const k = f.concat(y.map((P) => ({
        ...P
      })));
      y = [], J(k);
    }
    function se() {
      if (!h) return;
      h.innerHTML = "";
      const k = [
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
      for (const [st, wt, Ot] of k) {
        const Bt = document.createElement("button");
        Bt.className = "se-tool" + (w === st ? " se-tool-active" : ""), Bt.textContent = wt, Bt.title = Ot, Bt.addEventListener("click", () => It(st)), h.appendChild(Bt);
      }
      const P = document.createElement("button");
      P.className = "se-dir";
      const q = {
        North: "\u2191",
        East: "\u2192",
        South: "\u2193",
        West: "\u2190"
      };
      if (P.textContent = q[x], P.title = "Brush direction (R rotates)", P.addEventListener("click", () => {
        x = Wc[x], se();
      }), h.appendChild(P), p && p.items.length > 1) {
        const st = document.createElement("select");
        st.className = "se-item";
        for (const [wt, Ot] of p.items.entries()) {
          const Bt = document.createElement("option");
          Bt.value = String(wt), Bt.textContent = Ot, wt === _ && (Bt.selected = true), st.appendChild(Bt);
        }
        st.addEventListener("change", () => {
          _ = Number(st.value) | 0;
        }), h.appendChild(st);
      } else if (p && p.items.length === 1) {
        const st = document.createElement("span");
        st.className = "se-item-label", st.textContent = p.items[0], h.appendChild(st);
      }
      const j = document.createElement("span");
      j.style.flex = "1", h.appendChild(j);
      const R = document.createElement("button");
      R.className = "se-accept", R.textContent = "Accept", R.title = "Promote ghost into painted layer (Enter)", R.addEventListener("click", Vt), R.disabled = !(b === "valid" && y.length > 0), h.appendChild(R);
      const B = document.createElement("button");
      B.className = "se-revert", B.textContent = "Revert", B.title = "Discard all painted edits", B.addEventListener("click", () => {
        J([]);
      }), h.appendChild(B);
      const nt = document.createElement("button");
      nt.className = "se-export", nt.textContent = "Export", nt.title = "Save fixture JSON (clipboard + download)", nt.addEventListener("click", At), nt.disabled = b !== "valid", h.appendChild(nt);
      const H = document.createElement("button");
      H.className = "se-done", H.textContent = "Done", H.title = "Exit edit mode (Esc)", H.addEventListener("click", Lt), h.appendChild(H);
    }
    function Ce() {
      if (!h) return;
      const k = h.querySelector(".se-accept");
      k && (k.disabled = !(b === "valid" && y.length > 0));
      const P = h.querySelector(".se-export");
      P && (P.disabled = b !== "valid");
    }
    function At() {
      var _a2;
      if (!p || b !== "valid") return;
      const k = L ?? K(f), P = Xo(p.selection.cluster, p.selection.iter, p.selection.trace, {
        maxCost: k,
        paintedEntities: f
      }), q = (p.selection.cluster.seed ? `fixture_${p.selection.cluster.seed.x}_${p.selection.cluster.seed.y}_painted` : "fixture_painted") + ".json";
      (_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText(P).catch(() => {
      });
      const j = new Blob([
        P
      ], {
        type: "application/json"
      }), R = URL.createObjectURL(j), B = document.createElement("a");
      B.href = R, B.download = q, document.body.appendChild(B), B.click(), document.body.removeChild(B), URL.revokeObjectURL(R);
    }
    function Rt(k) {
      var _a2, _b2, _c2, _d2;
      u && Lt();
      const P = k.iter, q = ((_a2 = P.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", j = ((_b2 = P.sat) == null ? void 0 : _b2.max_reach) ?? Su[q] ?? 4, R = ((_c2 = P.sat) == null ? void 0 : _c2.boundaries) ?? P.boundaries, B = Array.from(new Set(R.map((H) => H.item))), nt = R.map((H) => ({
        x: H.x,
        y: H.y,
        item: H.item,
        isInput: H.is_input,
        dir: H.direction
      }));
      p = {
        bbox: {
          x: P.bbox.x,
          y: P.bbox.y,
          w: P.bbox.w,
          h: P.bbox.h
        },
        forbidden: new Set((P.forbidden ?? []).map((H) => `${H[0]},${H[1]}`)),
        beltTier: q,
        maxReach: j,
        items: B,
        boundaries: nt,
        fixtureJson: Xo(k.cluster, k.iter, k.trace),
        selection: k
      }, f = [], m = [], g = [], y = [], w = "belt", x = "East", _ = 0, v = null, L = null, o = new Ut(), a = new Ut(), a.alpha = 0.55, l = new Ut(), c = new Ut(), t.addChild(o), t.addChild(a), t.addChild(c), t.addChild(l), t.setChildIndex(r, t.children.length - 1), h = document.createElement("div"), h.className = "se-toolbar", i.inlineEl.appendChild(h), d = document.createElement("span"), d.className = "se-status", (_d2 = i.inlineEl.querySelector(".jd-inline-head")) == null ? void 0 : _d2.appendChild(d), i.setEditMode(true), t.plugins.pause("drag"), e.addEventListener("pointerdown", U, {
        capture: true
      }), e.addEventListener("pointerup", it, {
        capture: true
      }), e.addEventListener("pointermove", Z, {
        capture: true
      }), document.addEventListener("keydown", pt, {
        capture: true
      }), u = true, se(), D("idle");
    }
    function Lt() {
      u && (u = false, T !== null && (window.clearTimeout(T), T = null), C++, o && (t.removeChild(o), o.destroy({
        children: true
      }), o = null), a && (t.removeChild(a), a.destroy({
        children: true
      }), a = null), l && (t.removeChild(l), l.destroy({
        children: true
      }), l = null), c && (t.removeChild(c), c.destroy({
        children: true
      }), c = null), h && (h.remove(), h = null), d && (d.remove(), d = null), e.removeEventListener("pointerdown", U, {
        capture: true
      }), e.removeEventListener("pointerup", it, {
        capture: true
      }), e.removeEventListener("pointermove", Z, {
        capture: true
      }), document.removeEventListener("keydown", pt, {
        capture: true
      }), t.plugins.resume("drag"), i.setEditMode(false), p = null, f = [], m = [], g = [], y = []);
    }
    function he() {
      return u;
    }
    return {
      enter: Rt,
      exit: Lt,
      isActive: he
    };
  }
  let _s = {
    master: false,
    stepThrough: true,
    satZones: false,
    soloRegions: false,
    ghostTiles: false,
    itemColors: true,
    traceOverlay: false,
    heatmap: false,
    powerWires: false
  };
  const qi = [];
  function Rw() {
    const n = new URLSearchParams(window.location.search).get("debug") === "1", t = localStorage.getItem("fk-debug") === "1", e = localStorage.getItem("fk-sat-zones") === "1", s = localStorage.getItem("fk-ghost-tiles") === "1", i = localStorage.getItem("fk-item-colors"), r = localStorage.getItem("fk-trace-overlay") === "1", o = localStorage.getItem("fk-heatmap") === "1", a = localStorage.getItem("fk-power-wires") === "1";
    _s = {
      ..._s,
      master: n || t,
      satZones: e,
      ghostTiles: s,
      itemColors: i === null ? true : i === "1",
      traceOverlay: r,
      heatmap: o,
      powerWires: a
    };
  }
  function Tu() {
    return _s;
  }
  function Cn(n) {
    _s = {
      ..._s,
      ...n
    }, "master" in n && localStorage.setItem("fk-debug", n.master ? "1" : "0"), "satZones" in n && localStorage.setItem("fk-sat-zones", n.satZones ? "1" : "0"), "ghostTiles" in n && localStorage.setItem("fk-ghost-tiles", n.ghostTiles ? "1" : "0"), "itemColors" in n && localStorage.setItem("fk-item-colors", n.itemColors ? "1" : "0"), "traceOverlay" in n && localStorage.setItem("fk-trace-overlay", n.traceOverlay ? "1" : "0"), "heatmap" in n && localStorage.setItem("fk-heatmap", n.heatmap ? "1" : "0"), "powerWires" in n && localStorage.setItem("fk-power-wires", n.powerWires ? "1" : "0");
    for (const t of qi) t(_s);
  }
  function Lw(n) {
    return qi.push(n), () => {
      const t = qi.indexOf(n);
      t >= 0 && qi.splice(t, 1);
    };
  }
  function Sn(n, t, e = false) {
    const s = document.createElement("input");
    s.type = "checkbox", s.checked = e;
    const i = document.createElement("div");
    i.className = "overlay-toggle";
    const r = document.createElement("label");
    return r.appendChild(s), r.appendChild(document.createTextNode(t)), i.appendChild(r), n.appendChild(i), s;
  }
  function $w(n) {
    n.style.position = "relative";
    const t = document.createElement("div");
    t.className = "overlay-panel";
    const e = Tu(), s = Sn(t, "Debug", e.master), i = Sn(t, "Item colours", e.itemColors), r = Sn(t, "Starvation heatmap", e.heatmap), o = Sn(t, "Power wires", e.powerWires), a = document.createElement("div");
    a.className = "overlay-sub-panel", a.style.display = e.master ? "flex" : "none";
    const l = Sn(a, "SAT Zones", e.satZones), c = Sn(a, "Ghost tiles", e.ghostTiles), h = Sn(a, "Trace overlay", e.traceOverlay), d = Sn(a, "Solo regions", e.soloRegions);
    return t.appendChild(a), n.appendChild(t), s.addEventListener("change", () => {
      a.style.display = s.checked ? "flex" : "none", Cn({
        master: s.checked
      });
    }), l.addEventListener("change", () => {
      Cn({
        satZones: l.checked
      });
    }), c.addEventListener("change", () => {
      Cn({
        ghostTiles: c.checked
      });
    }), h.addEventListener("change", () => {
      Cn({
        traceOverlay: h.checked
      });
    }), i.addEventListener("change", () => {
      Cn({
        itemColors: i.checked
      });
    }), r.addEventListener("change", () => {
      Cn({
        heatmap: r.checked
      });
    }), o.addEventListener("change", () => {
      Cn({
        powerWires: o.checked
      });
    }), {
      setDebugEnabled(u) {
        s.checked = u, a.style.display = u ? "flex" : "none", Cn({
          master: u
        });
      },
      debugCb: s,
      colorCb: i,
      heatmapCb: r,
      powerWiresCb: o,
      regionsCb: l,
      soloRegionsCb: d,
      ghostTilesCb: c,
      traceOverlayCb: h
    };
  }
  function Bw(n) {
    const t = document.createElement("div");
    t.className = "retry-panel", n.appendChild(t);
    let e = null;
    function s(r) {
      if (!(r == null ? void 0 : r.trace)) return null;
      for (const o of r.trace) if (o.phase === "LayoutRetried") return o.data;
      return null;
    }
    function i() {
      const r = Tu().master, o = s(e);
      if (!r || !o) {
        t.classList.remove("visible"), t.replaceChildren();
        return;
      }
      t.classList.add("visible"), t.replaceChildren();
      const a = document.createElement("div");
      a.className = "retry-panel-title";
      const l = o.gaps.length;
      a.textContent = `Layout retry: ${l} row${l === 1 ? "" : "s"} widened`, t.appendChild(a);
      const c = document.createElement("div");
      c.className = "retry-panel-summary", c.textContent = `${o.caps_before} junction cap${o.caps_before === 1 ? "" : "s"} before retry`, t.appendChild(c);
      for (let h = 0; h < o.gaps.length; h++) {
        const [d, u] = o.gaps[h], p = o.recipes[h] ?? "?", f = document.createElement("div");
        f.className = "retry-panel-row";
        const m = document.createElement("span");
        m.className = "recipe", m.textContent = p;
        const g = document.createElement("span");
        g.className = "gap", g.textContent = `+${u} tile${u === 1 ? "" : "s"}`, f.appendChild(document.createTextNode(`row ${d} (`)), f.appendChild(m), f.appendChild(document.createTextNode("): ")), f.appendChild(g), t.appendChild(f);
      }
    }
    return Lw(() => i()), {
      update(r) {
        e = r, i();
      }
    };
  }
  const Ow = {
    North: "\u2191",
    East: "\u2192",
    South: "\u2193",
    West: "\u2190"
  }, zc = {
    N: "\u2191",
    E: "\u2192",
    S: "\u2193",
    W: "\u2190"
  };
  function an(n = 16) {
    const t = document.createElement("img");
    return t.width = n, t.height = n, t.style.cssText = "vertical-align:middle;margin-right:3px;image-rendering:pixelated", t.addEventListener("error", () => {
      t.style.display = "none";
    }), t;
  }
  function hs(n, t) {
    n.style.display = "", n.src = `/spaghettio/icons/${t}.png`;
  }
  function lo(n, t) {
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
  function Nw(n) {
    const t = document.createElement("div");
    t.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:240px;line-height:1.5", document.body.appendChild(t);
    const e = document.createElement("div");
    t.appendChild(e);
    const s = document.createElement("span");
    s.style.color = "#888", s.style.display = "none", t.appendChild(s);
    const i = document.createElement("div");
    t.appendChild(i);
    const r = document.createElement("div"), o = an(16), a = document.createElement("b");
    r.append(o, a), r.style.display = "none", i.appendChild(r);
    const l = document.createElement("div");
    l.style.display = "none", i.appendChild(l);
    const c = document.createElement("div"), h = an(16), d = document.createElement("span");
    c.append(h, d), c.style.display = "none", i.appendChild(c);
    const u = document.createElement("div");
    u.style.color = "#b5cea8", u.style.display = "none", i.appendChild(u);
    const p = document.createElement("div");
    p.style.display = "none", i.appendChild(p);
    const f = document.createElement("div"), m = an(16), g = document.createElement("span");
    f.append(m, g), f.style.display = "none", i.appendChild(f);
    function y() {
      const R = document.createElement("div");
      R.style.color = "#aaa";
      const B = document.createElement("span"), nt = an(14), H = document.createElement("span");
      return R.append(B, nt, H), R;
    }
    const w = lo(i, y), x = document.createElement("div");
    x.style.color = "#9cdcfe", x.style.display = "none", i.appendChild(x);
    const _ = document.createElement("div");
    _.style.display = "none", t.appendChild(_);
    const v = document.createElement("div");
    v.style.display = "none", t.appendChild(v);
    const b = document.createElement("div");
    b.style.display = "none", t.appendChild(b);
    const C = document.createElement("div");
    C.style.display = "none", t.appendChild(C);
    const T = document.createElement("div");
    T.style.cssText = "position:absolute;top:8px;right:8px;background:#141414;color:#e0e0e0;border:1px solid #888;padding:8px 10px;font:12px monospace;border-radius:4px;display:none;z-index:20;min-width:220px;max-width:340px;line-height:1.55;box-shadow:0 4px 14px rgba(0,0,0,0.5)", n.appendChild(T);
    const L = document.createElement("div");
    L.style.cssText = "display:flex;justify-content:space-between;align-items:center;margin-bottom:4px";
    const A = document.createElement("span");
    A.style.cssText = "color:#8af;font-weight:bold", A.textContent = "pinned";
    const E = document.createElement("span");
    E.style.color = "#888", L.append(A, E), T.appendChild(L);
    const I = document.createElement("div");
    T.appendChild(I);
    const V = document.createElement("div"), G = an(16), F = document.createElement("b");
    V.append(G, F), V.style.display = "none", I.appendChild(V);
    const $ = document.createElement("span");
    $.style.color = "#888", $.textContent = "no entity at tile", $.style.display = "none", I.appendChild($);
    const z = document.createElement("div");
    z.style.display = "none", I.appendChild(z);
    const J = document.createElement("div"), X = an(16), Q = document.createElement("span");
    J.append(X, Q), J.style.display = "none", I.appendChild(J);
    const M = document.createElement("div");
    M.style.color = "#b5cea8", M.style.display = "none", I.appendChild(M);
    const O = document.createElement("div");
    O.style.display = "none", I.appendChild(O);
    const N = document.createElement("div"), D = an(16), K = document.createElement("span");
    N.append(D, K), N.style.display = "none", I.appendChild(N);
    const tt = lo(I, y), ot = document.createElement("div");
    ot.style.color = "#9cdcfe", ot.style.display = "none", I.appendChild(ot);
    const at = document.createElement("div");
    at.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", at.style.display = "none";
    const mt = document.createElement("span"), bt = document.createElement("span");
    bt.style.color = "#888", at.append(mt, bt), T.appendChild(at);
    const Y = document.createElement("div");
    Y.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", Y.style.display = "none", T.appendChild(Y);
    const et = document.createElement("div");
    et.style.marginTop = "4px", et.style.display = "none", T.appendChild(et);
    const lt = document.createElement("div");
    lt.style.display = "none", T.appendChild(lt);
    const ct = document.createElement("div");
    ct.style.marginTop = "4px", lt.appendChild(ct);
    function _t() {
      const R = document.createElement("div");
      return R.style.marginLeft = "4px", R;
    }
    const Et = lo(lt, _t), jt = document.createElement("div");
    jt.style.cssText = "color:#555;margin-top:6px;font-size:10px", jt.textContent = "click elsewhere or press Esc to unpin", T.appendChild(jt), document.addEventListener("mousemove", (R) => {
      t.style.left = R.clientX + 14 + "px", t.style.top = R.clientY - 10 + "px";
    });
    let U = null, Z = null, it = null, pt = null, It = null, zt = null;
    const Dt = /* @__PURE__ */ new Set();
    function Vt() {
      const R = zt ? {
        x: zt.x,
        y: zt.y
      } : null;
      for (const B of Dt) B(R);
    }
    function se(R, B) {
      hs(B.headerIcon, R.name), B.headerName.textContent = fe(R.name), B.header.style.display = "", R.direction && R.name !== "pipe" ? (B.dirRow.textContent = `${Ow[R.direction] ?? ""} ${R.direction}`, B.dirRow.style.display = "") : B.dirRow.style.display = "none", R.carries ? (hs(B.carriesIcon, R.carries), B.carriesName.textContent = " " + fe(R.carries), B.carriesRow.style.display = "") : B.carriesRow.style.display = "none", R.rate != null ? (B.rateRow.textContent = `${R.rate.toFixed(1)}/s`, B.rateRow.style.display = "") : B.rateRow.style.display = "none", R.io_type ? (B.ioRow.textContent = `io: ${R.io_type}`, B.ioRow.style.display = "") : B.ioRow.style.display = "none";
      let nt = 0;
      if (R.recipe) {
        hs(B.recipeIcon, R.recipe), B.recipeName.textContent = " " + fe(R.recipe), B.recipeRow.style.display = "";
        const H = n0(R.recipe);
        if (H) {
          const st = [
            ...H.inputs.map((wt) => ({
              arrow: "\u25B6",
              item: wt.item,
              rate: wt.rate
            })),
            ...H.outputs.map((wt) => ({
              arrow: "\u25C0",
              item: wt.item,
              rate: wt.rate
            }))
          ];
          for (const wt of st) {
            const Ot = B.flowPool.get(nt++), [Bt, qt, ge] = Ot.children;
            Bt.textContent = `${wt.arrow} `, hs(qt, wt.item), ge.textContent = `${fe(wt.item)} ${wt.rate.toFixed(1)}/s`;
          }
        }
      } else B.recipeRow.style.display = "none";
      return B.flowPool.trim(nt), R.segment_id ? (B.segmentRow.textContent = R.segment_id, B.segmentRow.style.display = "") : B.segmentRow.style.display = "none", nt;
    }
    function Ce(R) {
      if (R.ghosts.length === 0) return _.style.display = "none", false;
      if (_.style.display = "", R.ghosts.length === 1) {
        const B = R.ghosts[0], nt = B.direction ? zc[B.direction] : "";
        _.textContent = "";
        const H = document.createElement("span");
        H.style.color = "#8af", H.textContent = "ghost ";
        const st = an(12);
        hs(st, B.item);
        const wt = document.createTextNode(`${B.item} ${nt}`);
        _.append(H, st, wt);
      } else {
        _.textContent = "";
        const B = document.createElement("span");
        B.style.color = "#8af", B.textContent = `${R.ghosts.length} ghosts crossing`, _.appendChild(B);
      }
      return true;
    }
    function At(R) {
      if (!R.axis) return v.style.display = "none", false;
      const { vert: B, horiz: nt } = R.axis;
      if (B === 0 && nt === 0) return v.style.display = "none", false;
      const H = B >= 2 || nt >= 2, st = B >= 1 && nt >= 1, wt = H ? "#ff6060" : st ? "#60b0ff" : "#888";
      return v.style.display = "", v.style.color = wt, v.textContent = `axis V${B} H${nt}`, true;
    }
    function Rt(R) {
      if (!R.junction) return b.style.display = "none", false;
      const B = R.junction, nt = B.outcome === "Solved" ? "#80d080" : B.outcome === "Capped" ? "#e0b060" : "#c06060";
      return b.style.display = "", b.style.color = nt, b.textContent = `junction seed (${B.seedX},${B.seedY}) \xB7 ${B.outcome}`, true;
    }
    function Lt(R) {
      if (R.cappedSides.length === 0) return C.style.display = "none", false;
      const B = R.cappedSides.map((nt) => {
        const H = nt.required - nt.shortfall;
        return `\u26A0 ${nt.sideIsOutput ? "out" : "in"}: ${nt.placedCount}\xD7${nt.placedEntity} moves ${H.toFixed(2)}/s of ${nt.required.toFixed(2)}/s \xB7 ${nt.limit}`;
      });
      return C.style.display = "", C.style.color = "#ffa060", C.textContent = B.join(" | "), true;
    }
    function he(R) {
      if (R.ghosts.length === 0) {
        lt.style.display = "none";
        return;
      }
      lt.style.display = "", R.ghosts.length >= 2 ? (ct.style.color = "#ffa060", ct.textContent = `\u26A0 ${R.ghosts.length} ghost specs at this tile`) : (ct.style.color = "#8af", ct.textContent = "ghost");
      let B = 0;
      for (const nt of R.ghosts) {
        const H = nt.direction ? zc[nt.direction] : "\xB7", st = Et.get(B++);
        st.textContent = "";
        const wt = document.createTextNode(`${H} `), Ot = an(14);
        hs(Ot, nt.item);
        const Bt = document.createTextNode(nt.item);
        if (st.append(wt, Ot, Bt), nt.isStart) {
          const qt = document.createElement("span");
          qt.style.color = "#80d080", qt.textContent = " start", st.appendChild(qt);
        } else if (nt.isEnd) {
          const qt = document.createElement("span");
          qt.style.color = "#d08080", qt.textContent = " end", st.appendChild(qt);
        }
      }
      Et.trim(B);
    }
    function k() {
      if (Z !== null) {
        e.innerHTML = Z, e.style.display = "", i.style.display = "none", _.style.display = "none", v.style.display = "none", b.style.display = "none", pt ? (s.textContent = `(${pt.x}, ${pt.y})`, s.style.display = "", s.style.display = "block") : s.style.display = "none", t.style.display = "block";
        return;
      }
      e.style.display = "none", e.innerHTML = "", i.style.display = "";
      let R = false;
      if (it ? (R = true, se(it, {
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
        recipeIcon: m,
        recipeName: g,
        flowPool: w,
        segmentRow: x
      })) : (r.style.display = "none", l.style.display = "none", c.style.display = "none", u.style.display = "none", p.style.display = "none", f.style.display = "none", w.trim(0), x.style.display = "none"), pt) {
        const B = It == null ? void 0 : It.lookup(pt.x, pt.y);
        B ? (Ce(B) && (R = true), At(B) && (R = true), Rt(B) && (R = true), Lt(B) && (R = true)) : (_.style.display = "none", v.style.display = "none", b.style.display = "none", C.style.display = "none"), s.textContent = `(${pt.x}, ${pt.y})`, s.style.display = "block", R = true;
      } else s.style.display = "none", _.style.display = "none", v.style.display = "none", b.style.display = "none";
      if (!R) {
        t.style.display = "none", U && U.clearHighlight();
        return;
      }
      t.style.display = "block", it && U ? U.highlightBeltNetwork(it) : U && U.clearHighlight();
    }
    function P() {
      if (!zt) {
        T.style.display = "none";
        return;
      }
      const { entity: R, x: B, y: nt } = zt, H = It == null ? void 0 : It.lookup(B, nt);
      if (E.textContent = `(${B}, ${nt})`, R ? ($.style.display = "none", se(R, {
        header: V,
        headerIcon: G,
        headerName: F,
        dirRow: z,
        carriesRow: J,
        carriesIcon: X,
        carriesName: Q,
        rateRow: M,
        ioRow: O,
        recipeRow: N,
        recipeIcon: D,
        recipeName: K,
        flowPool: tt,
        segmentRow: ot
      })) : (V.style.display = "none", z.style.display = "none", J.style.display = "none", M.style.display = "none", O.style.display = "none", N.style.display = "none", tt.trim(0), ot.style.display = "none", $.style.display = ""), H) {
        if (H.junction) {
          const st = H.junction.outcome === "Solved" ? "#80d080" : H.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
          mt.style.color = st, mt.textContent = `junction seed (${H.junction.seedX},${H.junction.seedY})`, bt.textContent = ` \xB7 ${H.junction.outcome}`, at.style.display = "";
        } else at.style.display = "none";
        if (H.axis) {
          const { vert: st, horiz: wt } = H.axis;
          if (st > 0 || wt > 0) {
            const Ot = st >= 2 || wt >= 2, Bt = st >= 1 && wt >= 1, qt = Ot ? " same-axis conflict" : Bt ? " perpendicular crossing" : "", ge = Ot ? "#ff6060" : Bt ? "#60b0ff" : "#bbb";
            et.style.color = ge, et.textContent = `axis: V=${st} H=${wt}${qt}`, et.style.display = "";
          } else et.style.display = "none";
        } else et.style.display = "none";
        if (he(H), H.cappedSides.length > 0) {
          const st = {
            "tier-cap": "a faster inserter tier at the same slot count would cover this \u2014 max inserter tier is the binding constraint",
            "column-contest": "this side lost the shared inserter column to the other belt; that one column would have covered it",
            geometry: "the row shape offers no further usable slot (belt span / fixed tiles) \u2014 a template geometry limit"
          };
          Y.replaceChildren();
          const wt = document.createElement("div");
          wt.style.color = "#ffa060", wt.textContent = `\u26A0 ${H.cappedSides.length} under-provisioned inserter side${H.cappedSides.length > 1 ? "s" : ""}`, Y.appendChild(wt);
          for (const Ot of H.cappedSides) {
            const Bt = (Ot.required - Ot.shortfall).toFixed(2), qt = document.createElement("div");
            qt.style.cssText = "margin-top:2px;color:#ccc", qt.textContent = `${Ot.sideIsOutput ? "output" : "input"}: ${Ot.placedCount}\xD7${Ot.placedEntity} moves ${Bt}/s of ${Ot.required.toFixed(2)}/s needed (short ${Ot.shortfall.toFixed(2)}/s) \u2014 ${Ot.limit}: ${st[Ot.limit] ?? Ot.limit}`, Y.appendChild(qt);
          }
          Y.style.display = "";
        } else Y.style.display = "none";
      } else at.style.display = "none", et.style.display = "none", lt.style.display = "none", Y.style.display = "none";
      T.style.display = "block";
    }
    function q() {
      k(), P();
    }
    function j(R, B, nt) {
      it = R, B !== void 0 && nt !== void 0 ? pt = {
        x: B,
        y: nt
      } : R && (pt = {
        x: R.x ?? 0,
        y: R.y ?? 0
      }), q();
    }
    return document.addEventListener("keydown", (R) => {
      R.key === "Escape" && zt && (zt = null, Vt(), q());
    }), {
      onHover: j,
      setHighlightController(R) {
        U = R;
      },
      setTooltipOverride(R) {
        Z = R, q();
      },
      setCursorTile(R, B) {
        R === null || B === void 0 ? pt = null : pt = {
          x: R,
          y: B
        }, q();
      },
      setTileContext(R) {
        It = R, q();
      },
      pinTile(R, B, nt) {
        zt = {
          entity: R,
          x: B,
          y: nt
        }, Vt(), q();
      },
      clearPin() {
        zt = null, Vt(), q();
      },
      getPinnedTile() {
        return zt ? {
          x: zt.x,
          y: zt.y
        } : null;
      },
      onPinChange(R) {
        return Dt.add(R), () => Dt.delete(R);
      }
    };
  }
  function Fw(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function Dc(n, t, e, s) {
    return e > n ? "E" : e < n ? "W" : s > t ? "S" : s < t ? "N" : null;
  }
  const Ww = {
    ghosts: [],
    axis: null,
    junction: null,
    cappedSides: []
  };
  function Gw(n) {
    var _a2;
    if (!n || n.length === 0) return {
      lookup: () => Ww
    };
    const t = /* @__PURE__ */ new Map(), e = /* @__PURE__ */ new Map(), s = /* @__PURE__ */ new Map(), i = /* @__PURE__ */ new Map();
    for (const r of n) if (r.phase === "GhostSpecRouted") {
      const { spec_key: o, tiles: a } = r.data, l = Fw(o);
      if (!a || a.length === 0) continue;
      for (let c = 0; c < a.length; c++) {
        const [h, d] = a[c];
        let u = null;
        c < a.length - 1 ? u = Dc(h, d, a[c + 1][0], a[c + 1][1]) : c > 0 && (u = Dc(a[c - 1][0], a[c - 1][1], h, d));
        const p = `${h},${d}`, f = t.get(p), m = {
          item: l,
          specKey: o,
          direction: u,
          isStart: c === 0,
          isEnd: c === a.length - 1
        };
        f ? f.push(m) : t.set(p, [
          m
        ]);
      }
    } else if (r.phase === "GhostAxisOccupancy") for (const o of r.data.tiles) e.set(`${o.x},${o.y}`, {
      vert: o.vert_count,
      horiz: o.horiz_count
    });
    else if (r.phase === "JunctionSolved" || r.phase === "JunctionGrowthCapped") {
      const o = r.data, a = r.phase === "JunctionSolved" ? "Solved" : "Capped";
      s.set(`${o.tile_x},${o.tile_y}`, {
        seedX: o.tile_x,
        seedY: o.tile_y,
        outcome: a
      });
    } else if (r.phase === "InserterSideCapped") {
      const o = r.data, a = `${o.machine_x},${o.machine_y}`, l = {
        recipe: o.recipe,
        sideIsOutput: o.side_is_output,
        required: o.required,
        placedEntity: o.placed_entity,
        placedCount: o.placed_count,
        shortfall: o.shortfall,
        limit: o.limit
      }, c = i.get(a);
      c ? c.push(l) : i.set(a, [
        l
      ]);
    } else if (r.phase === "JunctionGrowthIteration") {
      const o = r.data, a = `${o.seed_x},${o.seed_y}`;
      for (const [l, c] of o.tiles) {
        const h = `${l},${c}`;
        (!s.has(h) || s.get(h).seedX === o.seed_x) && s.set(h, {
          seedX: o.seed_x,
          seedY: o.seed_y,
          outcome: ((_a2 = s.get(a)) == null ? void 0 : _a2.outcome) ?? "Open"
        });
      }
    }
    return {
      lookup(r, o) {
        const a = `${r},${o}`;
        return {
          ghosts: t.get(a) ?? [],
          axis: e.get(a) ?? null,
          junction: s.get(a) ?? null,
          cappedSides: i.get(a) ?? []
        };
      }
    };
  }
  function zw(n) {
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
      a && (t = v_(a, i, o), a.querySelectorAll("input,select,button").forEach((l) => {
        l.closest("[data-snapshot-keep]") || (l.disabled = true);
      }));
    }
    return {
      load: s,
      clear: e
    };
  }
  const Dw = 2200, Hc = 180, co = 200, Hw = 8, Uw = [
    "rows_placed",
    "lanes_planned",
    "bus_routed",
    "poles_placed"
  ];
  function lr(n) {
    return `${n.x ?? 0},${n.y ?? 0},${n.name},${n.recipe ?? ""}`;
  }
  function jw(n) {
    const t = /* @__PURE__ */ new Map(), e = n.trace;
    if (!Array.isArray(e)) return t;
    for (const s of e) {
      const i = s;
      i.phase === "PhaseSnapshot" && i.data && t.set(i.data.phase, i.data);
    }
    return t;
  }
  function Vw(n) {
    const t = jw(n), e = [], s = /* @__PURE__ */ new Set();
    for (const r of Uw) {
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
        const c = lr(l);
        s.has(c) || (s.add(c), a.push(l));
      }
      e.push({
        phase: r,
        entities: a
      });
    }
    const i = [];
    for (const r of n.entities) {
      const o = lr(r);
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
  function Yw(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map(), o = ti(n, t, e, s, (b, C) => {
      r.set(lr(b), C);
    }), a = /* @__PURE__ */ new Set();
    for (const b of r.values()) for (const C of b) a.add(C);
    const l = [];
    for (const b of t.children) {
      const C = b;
      a.has(C) || l.push(C);
    }
    for (const b of l) b.alpha = 0;
    for (const b of r.values()) for (const C of b) C.alpha = 0;
    const c = Vw(n), h = c.reduce((b, C) => b + (C.entities.length > 0 ? 1 : 0), 0);
    if (h === 0) {
      for (const b of l) b.alpha = 1;
      for (const b of r.values()) for (const C of b) C.alpha = 1;
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
    const u = Math.max(0, Dw - Hc * h) / h, p = [], f = /* @__PURE__ */ new Map();
    let m = 0;
    for (const b of c) {
      if (b.entities.length === 0) continue;
      f.set(b.phase, m);
      const C = Math.min(Hw, u / b.entities.length);
      b.entities.forEach((L, A) => {
        const E = r.get(lr(L));
        !E || E.length === 0 || p.push({
          graphics: E,
          revealStartMs: m + A * C
        });
      });
      const T = (b.entities.length - 1) * C;
      m += T + co + Hc;
    }
    if (l.length > 0) {
      const b = f.get("bus_routed") ?? f.get("rows_placed") ?? 0;
      for (const C of l) p.push({
        graphics: [
          C
        ],
        revealStartMs: b
      });
    }
    p.sort((b, C) => b.revealStartMs - C.revealStartMs);
    const g = performance.now();
    let y = 0, w = false, x = p.length === 0;
    const _ = () => {
      if (w || x) return;
      const b = performance.now() - g;
      for (let C = y; C < p.length; C++) {
        const T = p[C];
        if (T.revealStartMs > b) break;
        const L = Math.min(1, (b - T.revealStartMs) / co);
        for (const A of T.graphics) A.alpha = L;
      }
      for (; y < p.length; ) {
        const C = p[y];
        if (b - C.revealStartMs < co) break;
        for (const T of C.graphics) T.alpha = 1;
        y++;
      }
      y >= p.length && (x = true, i.ticker.remove(_), qn());
    };
    return x || (i.ticker.add(_), wr()), {
      controller: o,
      handle: {
        cancel() {
          w || x || (w = true, i.ticker.remove(_), qn(), Ve());
        },
        finish() {
          if (!(w || x)) {
            for (const b of p) for (const C of b.graphics) C.alpha = 1;
            x = true, i.ticker.remove(_), qn(), Ve();
          }
        },
        isDone() {
          return x || w;
        }
      }
    };
  }
  const Xw = 4243680;
  function qw(n, t, e, s = 240) {
    const i = new ft();
    t.addChild(i);
    const r = e.x * S, o = e.y * S, a = e.w * S, l = e.h * S;
    let c = 0;
    const h = () => {
      c += n.ticker.deltaMS;
      const d = Math.max(0, (s - c) / s);
      if (d <= 0) {
        n.ticker.remove(h), i.destroy(), qn();
        return;
      }
      i.clear(), i.rect(r, o, a, l).fill({
        color: Xw,
        alpha: 0.55 * d
      });
    };
    n.ticker.add(h), wr();
  }
  const Di = 150, Uc = 80, Kw = 4, Jw = 300, Tn = 4, En = 900, Zw = 6, Qw = 250, Hs = 800, Hi = 250;
  function qe(n, t, e) {
    return n <= 1 ? t : Math.min(t, e / n);
  }
  function jc(n, t) {
    return `${n},${t}`;
  }
  function tv(n) {
    return n.split(":")[1] ?? "";
  }
  function ev(n, t) {
    return n > 0 ? "East" : n < 0 ? "West" : t > 0 ? "South" : "North";
  }
  function nv(n, t, e, s) {
    const i = iu();
    i.attachTo(n);
    const r = new ft();
    n.addChild(r);
    const o = /* @__PURE__ */ new Map(), a = /* @__PURE__ */ new Set(), l = /* @__PURE__ */ new Map(), c = _r();
    let h = false, d = false, u = false, p = 0;
    const f = () => u ? p : performance.now();
    let m = null, g = 0;
    const y = /* @__PURE__ */ new Map();
    let w = null, x = null;
    const _ = [], v = globalThis.__TRACE_LOGS === true, b = (M, O) => {
      globalThis.__ANIM_LOGS && console.log(`[anim t=${f().toFixed(0)}ms] ${M}`, O);
    };
    function C(M, O) {
      const D = !y.get(M), K = {
        id: M,
        virtualMs: O
      };
      y.set(M, K), D && (w == null ? void 0 : w(K));
    }
    function T(M, O) {
      m === null && (m = M);
      const N = M + (O ? Di : Uc);
      N > g && (g = N);
    }
    function L(M, O) {
      if (M.length === 0) return;
      const N = f();
      for (const K of M) ir(K, c);
      const D = [
        ...M
      ].sort((K, tt) => {
        const ot = (K.y ?? 0) - (tt.y ?? 0);
        return ot !== 0 ? ot : (K.x ?? 0) - (tt.x ?? 0);
      });
      D.forEach((K, tt) => {
        const ot = N + tt * O, at = us(K);
        if (a.has(at)) return;
        a.add(at), Do(i, K, ot, c), l.has(at) || l.set(at, ot), xc(i, K.x ?? 0, K.y ?? 0);
        const mt = o.get(jc(K.x ?? 0, K.y ?? 0));
        mt && mt.fadeOutStartMs === null && (mt.fadeOutStartMs = ot, T(ot, false));
      }), T(N + (D.length - 1) * O, true);
    }
    function A(M, O, N, D, K, tt) {
      const ot = jc(M, O), at = o.get(ot);
      at && at.specKey === K || (F0(i, M, O, N, D, tt, K), at || o.set(ot, {
        specKey: K,
        fadeStartMs: tt,
        fadeOutStartMs: null
      }), T(tt, true));
    }
    function E(M) {
      const O = qe(M.length, Tn, En);
      b("rows_placed", {
        count: M.length,
        stagger_ms: O,
        span_ms: M.length * O
      }), L(M, O);
    }
    function I(M) {
      const O = f(), N = tv(M.spec_key), D = M.tiles;
      if (D.length === 0) return;
      const K = qe(D.length, Kw, Jw);
      b("ghost_routed", {
        spec_key: M.spec_key,
        item: N,
        tile_count: D.length,
        span_ms: D.length * K
      });
      for (let tt = 0; tt < D.length; tt++) {
        const [ot, at] = D[tt];
        let mt = 0, bt = 0;
        tt < D.length - 1 ? (mt = D[tt + 1][0] - ot, bt = D[tt + 1][1] - at) : tt > 0 && (mt = ot - D[tt - 1][0], bt = at - D[tt - 1][1]), A(ot, at, ev(mt, bt), N, M.spec_key, O + tt * K);
      }
    }
    function V(M) {
      const O = M.entities.length, N = qe(O, Tn, En);
      b("committed", {
        source: "spec",
        count: O,
        span_ms: O * N
      }), L(M.entities, N);
    }
    function G(M) {
      const O = f(), N = M.zone_x, D = M.zone_y, K = M.zone_x + M.zone_w - 1, tt = M.zone_y + M.zone_h - 1;
      for (const [mt, bt] of o.entries()) {
        const [Y, et] = mt.split(",").map(Number);
        Y < N || Y > K || et < D || et > tt || bt.fadeOutStartMs === null && (bt.fadeOutStartMs = O, T(O, false), xc(i, Y, et));
      }
      for (const mt of _) mt.clusterId === M.cluster_id && (mt.cleared = true);
      for (let mt = M.zone_y; mt <= tt; mt++) for (let bt = M.zone_x; bt <= K; bt++) {
        const Y = G0(i, bt, mt);
        for (const et of Y) a.delete(et);
      }
      const ot = M.entities.length, at = qe(ot, Zw, Qw);
      b("junction", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        count: ot,
        span_ms: ot * at
      }), L(M.entities, at);
    }
    function F(M) {
      if (d = true, M.phase === "rows_placed") {
        E(M.entities);
        return;
      }
      if (M.phase !== "lanes_planned") {
        if (M.phase === "bus_routed") {
          const O = M.entities.filter((N) => !a.has(us(N)));
          if (O.length > 0) {
            const N = qe(O.length, Tn, En);
            L(O, N);
          }
          return;
        }
        if (M.phase === "poles_placed") {
          const O = M.entities.filter((N) => !a.has(us(N)));
          if (O.length > 0) {
            const N = qe(O.length, Tn, En);
            L(O, N);
          }
          return;
        }
      }
    }
    function $(M) {
      const O = f();
      b("cluster_outline", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        lifetime_ms: Hs,
        fade_ms: Hi
      }), _.push({
        clusterId: M.cluster_id,
        x: M.zone_x,
        y: M.zone_y,
        w: M.zone_w,
        h: M.zone_h,
        startMs: O,
        cleared: false
      }), T(O, true);
    }
    const z = () => {
      if (h) return;
      const M = f();
      ou(i, M);
      for (const [O, N] of o.entries()) if (N.fadeOutStartMs !== null && M >= N.fadeOutStartMs) {
        const [D, K] = O.split(",").map(Number);
        (M - N.fadeOutStartMs) / Uc >= 1 && o.delete(O);
      }
      for (const O of i.ghostContainer.particleChildren) O.alpha < 0.5 && (O.alpha = Math.min(0.5, O.alpha + 16 / Di));
      r.clear();
      for (let O = _.length - 1; O >= 0; O--) {
        const N = _[O], D = M - N.startMs;
        if (!(D < 0)) if (N.cleared || D >= Hs) {
          const K = N.cleared ? Math.max(D, Hs - Hi) : D;
          if (K >= Hs) {
            _.splice(O, 1);
            continue;
          }
          const tt = Math.max(0, 1 - (K - (Hs - Hi)) / Hi);
          r.rect(N.x * S, N.y * S, N.w * S, N.h * S), r.stroke({
            width: 2,
            color: 4508927,
            alpha: 0.9 * tt
          });
        } else r.rect(N.x * S, N.y * S, N.w * S, N.h * S), r.stroke({
          width: 2,
          color: 4508927,
          alpha: 0.9
        });
      }
    };
    t.ticker.add(z), wr();
    let J = true;
    b("streaming_start", {});
    function X(M, O) {
      if (h || u) return;
      w = O ?? null, v && console.log(`[stream t=${f().toFixed(0)}] ${M.phase}`, "data" in M ? M.data : void 0);
      const N = f();
      switch (m === null && (m = N), M.phase) {
        case "PhaseSnapshot": {
          const D = M.data;
          D.phase === "rows_placed" && C("machines", N), D.phase === "poles_placed" && C("poles", N), F(D);
          break;
        }
        case "GhostSpecRouted":
          C("ghost_routes", N), I(M.data);
          break;
        case "GhostSpecCommitted":
          C("committed_routes", N), V(M.data);
          break;
        case "JunctionCommitted":
          C("junctions", N), G(M.data);
          break;
        case "GhostClusterSolved":
          $(M.data);
          break;
        case "TrunkBeltCommitted": {
          const D = M.data, K = D.entities.length, tt = qe(K, Tn, En);
          b("committed", {
            source: "trunk",
            count: K,
            span_ms: K * tt
          }), L(D.entities, tt);
          break;
        }
        case "BalancerCommitted": {
          const D = M.data, K = D.entities.length, tt = qe(K, Tn, En);
          b("committed", {
            source: "balancer",
            count: K,
            span_ms: K * tt
          }), L(D.entities, tt);
          break;
        }
        case "OutputMergerCommitted": {
          const D = M.data, K = D.entities.length, tt = qe(K, Tn, En);
          b("committed", {
            source: "merger",
            count: K,
            span_ms: K * tt
          }), L(D.entities, tt);
          break;
        }
        case "PolesCommitted": {
          const D = M.data, K = D.entities.length, tt = qe(K, Tn, En);
          b("committed", {
            source: "poles",
            count: K,
            span_ms: K * tt
          }), L(D.entities, tt);
          break;
        }
        default: {
          M.phase === "LayoutRetried" && (i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), Ve());
          break;
        }
      }
      w = null;
    }
    return {
      onEvent: X,
      hasCommittedEntities: () => d,
      cancel() {
        h || (h = true, t.ticker.remove(z), J && (qn(), J = false), i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), x = null, Ve());
      },
      finish(M) {
        t.ticker.remove(z), J && (qn(), J = false), W0(i), b("streaming_finish", {
          entity_count: i.count(),
          latest_fade_end_ms: g
        });
        const O = m ?? 0, N = [];
        for (const { particle: tt, iconParticle: ot, revealAt: at } of bc()) N.push({
          kind: "particle",
          particle: tt,
          iconParticle: ot,
          revealAt: at
        });
        const D = M.entities.filter((tt) => !a.has(us(tt)));
        if (D.length > 0) {
          for (const ot of D) ir(ot, c);
          for (const ot of D) Do(i, ot, O, c), a.add(us(ot));
          const tt = new Set(N.map((ot) => ot.particle));
          for (const { particle: ot, iconParticle: at, revealAt: mt } of bc()) tt.has(ot) || N.push({
            kind: "particle",
            particle: ot,
            iconParticle: at,
            revealAt: mt
          });
        }
        const K = z0(i, c);
        if (K.size > 0) for (const tt of N) {
          const ot = K.get(tt.particle);
          ot && (tt.particle = ot);
        }
        return x = N, u = true, p = g, Q(p), U0(M, t);
      },
      seekTo(M) {
        if (h || x === null) return;
        const O = m ?? 0, N = Math.max(g, O);
        p = Math.min(N, Math.max(O, M)), b("scrub", {
          virtualMs: p
        }), Q(p);
      },
      getTimeRange() {
        return {
          firstMs: m ?? 0,
          lastMs: g
        };
      },
      getMilestones() {
        return Array.from(y.values()).sort((M, O) => M.virtualMs - O.virtualMs);
      }
    };
    function Q(M) {
      if (x !== null) {
        for (const O of x) {
          const N = M - O.revealAt, D = N <= 0 ? 0 : N >= Di ? 1 : N / Di;
          O.particle.alpha = D, O.iconParticle && (O.iconParticle.alpha = D);
        }
        Ve();
      }
    }
  }
  const sv = 0.03, iv = 200, ho = [
    "machines",
    "ghost_routes",
    "committed_routes",
    "junctions",
    "poles",
    "optimizing"
  ], rv = {
    machines: "Machines",
    ghost_routes: "Belt routes",
    committed_routes: "Belts placed",
    junctions: "Crossings",
    poles: "Power poles",
    optimizing: "Optimizing"
  };
  function ov(n, t) {
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
    for (const G of ho) {
      const F = document.createElement("div");
      F.className = "ts-chip", F.dataset.milestone = G, F.textContent = rv[G], G === "optimizing" && (F.style.display = "none"), s.appendChild(F), l.set(G, F);
    }
    let c = false, h = null, d = [];
    const u = /* @__PURE__ */ new Set();
    let p = null;
    function f(G) {
      o.style.width = `${G * 100}%`, a.style.left = `${G * 100}%`;
    }
    function m(G) {
      var _a2, _b2;
      p !== G && (p && ((_a2 = l.get(p)) == null ? void 0 : _a2.classList.remove("ts-chip--active")), G && ((_b2 = l.get(G)) == null ? void 0 : _b2.classList.add("ts-chip--active")), p = G);
    }
    function g(G, F) {
      var _a2;
      if (c) return;
      const $ = G.id;
      u.add($), (_a2 = l.get($)) == null ? void 0 : _a2.classList.add("ts-chip--reached"), m($);
      const z = Math.max(1, F.lastMs - F.firstMs), J = (G.virtualMs - F.firstMs) / z;
      f(Math.min(1, Math.max(0, J))), e.classList.add("ts-visible");
    }
    function y(G) {
      return h ? h.firstMs + G * (h.lastMs - h.firstMs) : 0;
    }
    function w(G) {
      for (const F of d) if (Math.abs(G - F.frac) < sv) return {
        frac: F.frac,
        snapped: true
      };
      return {
        frac: G,
        snapped: false
      };
    }
    function x(G) {
      if (!h) return;
      const F = i.getBoundingClientRect(), $ = (G - F.left) / F.width, z = Math.min(1, Math.max(0, $)), { frac: J, snapped: X } = w(z);
      f(J), X ? a.classList.add("ts-thumb--snapped") : a.classList.remove("ts-thumb--snapped"), t(y(J));
    }
    let _ = null, v = null;
    function b(G) {
      if (!c || !h) return;
      G.preventDefault();
      try {
        i.setPointerCapture(G.pointerId);
      } catch {
      }
      const F = (z) => x(z.clientX), $ = (z) => {
        _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), _ = null, v = null, a.classList.remove("ts-thumb--snapped");
      };
      _ = F, v = $, document.addEventListener("pointermove", F), document.addEventListener("pointerup", $, {
        once: true
      }), x(G.clientX);
    }
    i.addEventListener("pointerdown", b);
    function C(G, F) {
      const $ = G.lastMs - G.firstMs;
      if ($ < iv || F.length === 0) {
        E();
        return;
      }
      c = true, h = G, e.classList.add("ts-scrub-mode"), e.classList.add("ts-visible"), d = F.map((z) => ({
        id: z.id,
        frac: (z.virtualMs - G.firstMs) / $
      })), s.style.justifyContent = "flex-start", s.style.position = "relative";
      for (const z of l.values()) z.style.position = "absolute", z.style.transform = "translateX(-50%)";
      for (const z of ho) {
        const J = l.get(z);
        if (!J) continue;
        const X = d.find((Q) => Q.id === z);
        X ? (J.style.left = `${X.frac * 100}%`, J.style.display = "", J.classList.add("ts-chip--reached")) : J.style.display = "none";
      }
      L(), requestAnimationFrame(A), f(1), m(null);
    }
    let T = null;
    function L() {
      if (T && T.remove(), !h) return;
      const G = document.createElement("div");
      G.className = "ts-ticks";
      for (const F of d) {
        const $ = document.createElement("div");
        $.className = "ts-tick", $.style.left = `${F.frac * 100}%`, G.appendChild($);
      }
      i.appendChild(G), T = G;
    }
    function A() {
      if (!c) return;
      const G = 6, F = s.clientWidth;
      if (F <= 0) return;
      const $ = ho.map((J) => {
        var _a2;
        const X = l.get(J);
        if (!X || X.style.display === "none") return null;
        const Q = J === "optimizing" ? 1 : ((_a2 = d.find((M) => M.id === J)) == null ? void 0 : _a2.frac) ?? 0;
        return {
          el: X,
          originalFrac: Q
        };
      }).filter((J) => J !== null);
      let z = -1 / 0;
      for (const { el: J, originalFrac: X } of $) {
        const Q = J.offsetWidth / 2, M = X * F, O = z + Q + G, N = Math.max(M, O);
        J.style.left = `${N / F * 100}%`, z = N + Q;
      }
    }
    function E() {
      c = false, h = null, d = [], u.clear(), p = null, T && (T.remove(), T = null), e.classList.remove("ts-visible", "ts-scrub-mode"), o.style.width = "0", a.style.left = "0", a.classList.remove("ts-thumb--snapped"), s.style.justifyContent = "space-between", s.style.position = "";
      for (const [G, F] of l) F.style.position = "", F.style.transform = "", F.style.left = "", F.style.display = G === "optimizing" ? "none" : "", F.classList.remove("ts-chip--reached", "ts-chip--active", "ts-chip--in-progress");
    }
    function I(G) {
      const F = l.get("optimizing");
      if (F) {
        if (F.classList.remove("ts-chip--in-progress", "ts-chip--reached"), G === "idle") {
          F.style.display = "none";
          return;
        }
        F.style.display = "", e.classList.add("ts-visible"), c && (F.style.position = "absolute", F.style.transform = "translateX(-50%)", F.style.left = "100%", requestAnimationFrame(A)), G === "active" ? F.classList.add("ts-chip--in-progress") : G === "done" && F.classList.add("ts-chip--reached");
      }
    }
    function V() {
      _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), i.removeEventListener("pointerdown", b), e.remove();
    }
    return {
      noteMilestone: g,
      arm: C,
      markOptimizeState: I,
      reset: E,
      destroy: V
    };
  }
  const av = `
.spaghettio-busy {
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
.spaghettio-busy.visible { opacity: 1; }
.spaghettio-busy-spin {
  width: 12px;
  height: 12px;
  border: 2px solid #2a2a2a;
  border-top-color: #569cd6;
  border-radius: 50%;
  animation: spaghettio-busy-spin 0.7s linear infinite;
  display: inline-block;
}
@keyframes spaghettio-busy-spin { to { transform: rotate(360deg); } }
`;
  function lv() {
    if (document.getElementById("spaghettio-busy-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-busy-style", n.textContent = av, document.head.appendChild(n);
  }
  const cv = 120;
  function hv(n) {
    lv();
    const t = document.createElement("div");
    t.className = "spaghettio-busy";
    const e = document.createElement("span");
    e.className = "spaghettio-busy-spin", t.appendChild(e);
    const s = document.createElement("span");
    s.textContent = "computing\u2026", t.appendChild(s), n.appendChild(t);
    let i = null;
    Lb((r) => {
      r > 0 ? i === null && !t.classList.contains("visible") && (i = setTimeout(() => {
        t.classList.add("visible"), i = null;
      }, cv)) : (i !== null && (clearTimeout(i), i = null), t.classList.remove("visible"));
    });
  }
  function Ke(n, t) {
    return n.filter((e) => e.phase === t);
  }
  const Gn = "color:#9cdcfe;font-weight:bold", de = "color:#888", _e = "color:#e0e0e0", ds = "color:#6a6", Ui = "color:#ffaa00", uo = "color:#f66", ji = "color:#c586c0";
  function dv(n) {
    var _a2;
    const t = Array.isArray(n.trace) ? n.trace : [];
    if (t.length === 0) return;
    const e = Ke(t, "PhaseTime"), s = Ke(t, "SatInvocation"), i = Ke(t, "JunctionSolved"), r = Ke(t, "JunctionGrowthCapped"), o = Ke(t, "GhostClusterSolved"), a = Ke(t, "GhostRoutingComplete"), l = Ke(t, "RegionWalkerVeto"), c = Ke(t, "JunctionGrowthIteration"), h = Ke(t, "NegotiateComplete"), d = Ke(t, "ValidationCompleted"), u = t.filter((x) => x.phase === "LayoutRetried"), p = e.reduce((x, _) => x + _.data.duration_ms, 0), f = s.reduce((x, _) => x + _.data.solve_time_us, 0), m = s.filter((x) => x.data.satisfied).length, g = ((_a2 = n.entities) == null ? void 0 : _a2.length) ?? 0, y = u.length > 0 ? r.length > 0 ? ` \xB7 ${r.length} capped \xB7 retried (${u[0].data.caps_before} caps recovered)` : ` \xB7 retried (${u[0].data.caps_before} caps recovered)` : r.length > 0 ? ` \xB7 ${r.length} capped` : "", w = r.length > 0 ? Ui : u.length > 0 ? ji : ds;
    if (console.log(`%c\u25B6 layout %c${n.width}\xD7${n.height}  %c${g} entities  %c${p}ms  %cSAT ${Math.round(f / 1e3)}ms (${s.length}\xD7)%c${y}`, Gn, _e, de, _e, ji, w), console.groupCollapsed("%c  \u21B3 breakdown", de), e.length > 0) {
      const x = [
        ...e
      ].sort((_, v) => v.data.duration_ms - _.data.duration_ms);
      console.log(`%cphases%c ${p}ms total`, Gn, de);
      for (const _ of x) {
        const v = _.data, b = p > 0 ? v.duration_ms / p * 100 : 0, C = Math.max(1, Math.round(b / 100 * 24)), T = "\u2588".repeat(C);
        console.log(`  %c${v.phase.padEnd(18)}%c ${String(v.duration_ms).padStart(5)}ms  %c${T}%c ${b.toFixed(1)}%`, de, _e, ji, de);
      }
    }
    if (s.length > 0) {
      const x = f / 1e3, _ = p > 0 ? x / p * 100 : 0, v = f / s.length, b = [
        ...s
      ].sort((T, L) => L.data.solve_time_us - T.data.solve_time_us)[0], C = [
        ...s
      ].sort((T, L) => L.data.zone_w * L.data.zone_h - T.data.zone_w * T.data.zone_h)[0];
      console.log(`%cSAT%c ${s.length} invocations \xB7 ${x.toFixed(1)}ms (%c${_.toFixed(1)}%%%c of total)`, Gn, _e, ji, _e), console.log(`  %csatisfied%c ${m}  %cunsat%c ${s.length - m}  %cavg%c ${(v / 1e3).toFixed(2)}ms`, de, ds, de, uo, de, _e), b && console.log(`  %cslowest call%c ${(b.data.solve_time_us / 1e3).toFixed(1)}ms \u2014 %c${b.data.zone_w}\xD7${b.data.zone_h} @ (${b.data.zone_x},${b.data.zone_y}), ${b.data.variables} vars, ${b.data.clauses} clauses`, de, _e, de), C && C !== b && console.log(`  %cbiggest zone%c ${C.data.zone_w}\xD7${C.data.zone_h} @ (${C.data.zone_x},${C.data.zone_y}) \u2014 ${C.data.variables} vars`, de, _e);
    }
    if (o.length > 0 || i.length > 0 || r.length > 0) {
      if (console.log("%cjunctions", Gn), console.log(`  %cclusters%c ${o.length}  %csolved%c ${i.length}  %ccapped%c ${r.length}  %cvetoes%c ${l.length}`, de, _e, de, ds, de, r.length > 0 ? Ui : _e, de, _e), c.length > 0) {
        const x = /* @__PURE__ */ new Map();
        for (const v of c) {
          const b = `${v.data.seed_x},${v.data.seed_y}`;
          x.set(b, Math.max(x.get(b) ?? 0, v.data.iter));
        }
        const _ = [
          ...x.entries()
        ].sort((v, b) => b[1] - v[1])[0];
        _ && _[1] > 0 && console.log(`  %chardest%c junction at (${_[0]}) needed ${_[1] + 1} growth iters`, de, _e);
      }
      if (r.length > 0) for (const x of r) console.log(`    %c\u26A0 capped at (${x.data.tile_x},${x.data.tile_y})%c \u2014 ${x.data.reason}, ${x.data.region_tiles} tiles after ${x.data.iters} iters`, Ui, de);
    }
    if (a.length > 0) {
      const x = a[0].data, _ = x.unroutable_count > 0 ? uo : ds;
      console.log(`%cghost router%c ${x.entity_count} routed entities, ${x.cluster_count} clusters, max cluster ${x.max_cluster_tiles} tiles  %c${x.unroutable_count} unroutable`, Gn, _e, _);
    }
    if (h.length > 0) {
      const x = h[0].data;
      console.log(`%cA* negotiate%c ${x.specs} specs, ${x.iterations} iters, ${x.duration_ms}ms`, Gn, _e);
    }
    if (d.length > 0) {
      const x = d[0].data, _ = x.error_count > 0 ? uo : ds, v = x.warning_count > 0 ? Ui : ds;
      console.log(`%cvalidation  %c${x.error_count} errors  %c${x.warning_count} warnings`, Gn, _, v);
    }
    console.groupEnd();
  }
  const uv = [
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
  async function pv() {
    await fu();
    const n = Yb();
    await a0(uv);
    const t = document.getElementById("app"), e = window.location.hash, s = new URLSearchParams(window.location.search);
    if (e.startsWith("#/balancers")) {
      const { renderBalancerShowcase: r } = await Pn(async () => {
        const { renderBalancerShowcase: o } = await import("./balancers-BWB7Sajo.js");
        return {
          renderBalancerShowcase: o
        };
      }, []);
      r(t, n);
      return;
    }
    if (!(e.startsWith("#/layout") || s.has("generator") || wb())) {
      const r = document.createElement("div");
      t.appendChild(r), t_(r, n, {
        onOpenGenerator: () => {
          r.remove(), Vc(n), window.history.replaceState({}, "", "#/layout");
        }
      });
      return;
    }
    Vc(n);
  }
  async function Vc(n) {
    const t = document.getElementById("canvas-container");
    if (!t) throw new Error("Missing #canvas-container element");
    const e = document.getElementById("app");
    e.style.display = "flex";
    const s = document.getElementById("sidebar");
    s && (s.style.display = ""), t.style.display = "";
    const { app: i, viewport: r, requestRender: o, beginAnimating: a, endAnimating: l } = await b0(t), c = _0(r);
    let h = false;
    Ds(r, null), Rw();
    const d = $w(t), { debugCb: u, colorCb: p, heatmapCb: f, powerWiresCb: m, regionsCb: g, soloRegionsCb: y, ghostTilesCb: w, traceOverlayCb: x } = d, _ = Bw(t);
    $i(p.checked);
    const v = () => {
      globalThis.__ANIM_LOGS = u.checked;
    };
    u.addEventListener("change", v), v();
    const b = Nw(t), C = lw();
    let T = null;
    const L = yw(t, r, {
      onChange: (W) => {
        if (C.update(W), W) {
          I.alpha = A.isActive() ? 0.2 : 0.35;
          const rt = W.iter.bbox;
          T = {
            bboxX: rt.x,
            bboxY: rt.y,
            bboxW: rt.w,
            bboxH: rt.h
          };
        } else I.alpha = 1, T = null, A.isActive() && A.exit();
        o();
      },
      onEditRequested: (W) => {
        I.alpha = 0.2, A.enter(W), o();
      }
    }), A = Iw({
      viewport: r,
      canvas: i.canvas,
      engine: n,
      jd: L,
      satZoneOverlayLayer: C.layer
    });
    w_(t, (W) => wt.load(W));
    function E(W) {
      I.removeChildren();
      const rt = iu();
      return rt.attachTo(I), D0(W, rt, i);
    }
    const I = new Ut();
    I.isRenderGroup = true, I.eventMode = "none", r.addChild(I);
    const V = new Ut();
    V.eventMode = "none", r.addChild(V), r.addChild(C.layer);
    const G = new ft();
    G.label = "pin-highlight", r.addChild(G), b.onPinChange((W) => {
      if (G.clear(), W) {
        const rt = W.x * S, ut = W.y * S;
        G.setStrokeStyle({
          width: 2,
          color: 8440063,
          alpha: 0.95
        }), G.rect(rt - 2, ut - 2, S + 4, S + 4).stroke();
      }
      o();
    }), r.moveCenter(en / 2, en / 2);
    const F = (W) => {
    };
    let $ = null;
    function z(W) {
      $ = W, b.onHover(W, W == null ? void 0 : W.x, W == null ? void 0 : W.y);
    }
    let J = /* @__PURE__ */ new Map();
    function X(W) {
      const rt = /* @__PURE__ */ new Map();
      for (const ut of W.entities) {
        const yt = ut.x ?? 0, dt = ut.y ?? 0, $t = Ne[ut.name];
        if ($t) {
          const [Wt, kt] = $t;
          for (let Nt = 0; Nt < kt; Nt++) for (let ye = 0; ye < Wt; ye++) rt.set(`${yt + ye},${dt + Nt}`, ut);
        } else if (ie.has(ut.name)) {
          rt.set(`${yt},${dt}`, ut);
          const [Wt, kt] = ma(ut.direction);
          rt.set(`${yt + Wt},${dt + kt}`, ut);
        } else rt.set(`${yt},${dt}`, ut);
      }
      J = rt;
    }
    function Q(W) {
      return {
        highlightItem: (rt) => {
          W.highlightItem(rt), o();
        },
        highlightBeltNetwork: (rt) => {
          W.highlightBeltNetwork(rt), o();
        },
        clearHighlight: () => {
          W.clearHighlight(), o();
        },
        chainKey: W.chainKey
      };
    }
    let M = false, O = null, N = null, D = false, K = null;
    const tt = {
      update() {
      },
      getPhaseIndex() {
        return -1;
      },
      reset() {
      }
    };
    function ot() {
      var _a2;
      N && (I.removeChild(N), N.destroy(), N = null);
      const W = tt.getPhaseIndex();
      if (u.checked && W >= 0, D && (K == null ? void 0 : K.cancel(), K = null, D = false, j)) {
        const ut = ti(j, I, z, F);
        b.setHighlightController(Q(ut)), o();
      }
      if (!u.checked || !x.checked || !((_a2 = j == null ? void 0 : j.trace) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      const rt = j.trace;
      N = C_(rt, j.width ?? 0, j.height ?? 0, I, (ut) => {
        b.setTooltipOverride(ut ? `<span style="color:#8af">TRACE</span> ${ut}` : null);
      }), o();
    }
    let at = null, mt = null, bt = null, Y = [], et = null, lt = null, ct = null;
    const _t = document.createElement("div");
    _t.className = "validation-badge", _t.style.display = "none", t.appendChild(_t);
    function Et(W) {
      if (!W || W.length === 0) {
        _t.style.display = "none";
        return;
      }
      const rt = W.filter((dt) => dt.severity === "Error").length, ut = W.length - rt;
      let yt;
      rt > 0 && ut > 0 ? yt = `\u26A0 ${rt} error${rt > 1 ? "s" : ""}, ${ut} warning${ut > 1 ? "s" : ""}` : rt > 0 ? yt = `\u26A0 ${rt} error${rt > 1 ? "s" : ""}` : yt = `\u26A0 ${ut} warning${ut > 1 ? "s" : ""}`, _t.textContent = yt, _t.classList.toggle("has-errors", rt > 0), _t.style.display = "block";
    }
    let jt = null, U = null, Z = null, it = null, pt = null;
    const It = 1;
    function zt(W, rt) {
      const ut = W * S + S / 2, yt = rt * S + S / 2;
      r.scale.x < It && r.setZoom(It, false), r.moveCenter(ut, yt);
    }
    function Dt(W) {
      var _a2, _b2;
      const rt = [];
      for (const ut of W.regions ?? []) {
        if (ut.kind !== "unresolved") continue;
        const yt = ut.x + Math.floor(ut.width / 2), dt = ut.y + Math.floor(ut.height / 2), $t = ((_b2 = (_a2 = ut.ports) == null ? void 0 : _a2.find((Wt) => Wt.item)) == null ? void 0 : _b2.item) ?? "unknown";
        rt.push({
          severity: "Warning",
          category: `ghost-router \xB7 ${$t}`,
          message: `unresolved crossing at (${yt}, ${dt})`,
          x: yt,
          y: dt
        });
      }
      for (const ut of W.warnings ?? []) /^ghost router:.*unresolved crossings/i.test(ut) || rt.push({
        severity: "Warning",
        category: "layout",
        message: ut,
        x: void 0,
        y: void 0
      });
      return rt;
    }
    function Vt() {
      if (at && (I.removeChild(at), at.destroy(), at = null), j && !mt && ct !== j) {
        const dt = j;
        ct = dt, n.validateLayout(dt, R).then(($t) => {
          j === dt && (mt = $t, ct = null, Vt(), ns(dt));
        }).catch(() => {
          j === dt && (mt = [], ct = null, Vt(), ns(dt));
        });
      }
      const W = j ? Dt(j) : [], rt = [
        ...mt ?? [],
        ...W
      ], ut = (dt) => {
        if (dt.x == null || dt.y == null || !lt) return null;
        const $t = lt.lookup(dt.x, dt.y).cappedSides;
        return $t.length === 0 ? null : [
          ...new Set($t.map((kt) => kt.limit))
        ].sort().join("+");
      };
      if (Rs == null ? void 0 : Rs.updateValidation(rt, zt, ut), Et(rt), se(rt), !j || rt.length === 0) {
        o();
        return;
      }
      at = A_(rt, I, (dt) => {
        b.setTooltipOverride(dt ? `<span style="color:#f44">VALIDATION</span> ${dt}` : null);
      }).layer, o();
    }
    function se(W) {
      if (W && (Y = W), bt && (I.removeChild(bt), bt.destroy({
        children: true
      }), bt = null), !f.checked || !j || Y.length === 0) {
        o();
        return;
      }
      bt = M_(Y, j.entities, I), o();
    }
    function Ce() {
      var _a2;
      if (et && (I.removeChild(et), et.destroy({
        children: true
      }), et = null), !m.checked || !((_a2 = j == null ? void 0 : j.power_wires) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      et = P_(j.power_wires, j.entities, I), o();
    }
    function At() {
      if (pt && (r.removeChild(pt), pt.destroy({
        children: true
      }), pt = null), !u.checked || !w.checked || !j) {
        o();
        return;
      }
      const W = gw(j.trace);
      if (!W) {
        o();
        return;
      }
      pt = W, r.addChildAt(pt, 0), o();
    }
    function Rt() {
      var _a2;
      if (jt && (I.removeChild(jt), jt.destroy(), jt = null), Z && (I.removeChild(Z), Z.destroy(), Z = null), U = null, it = null, !u.checked || !(g == null ? void 0 : g.checked) || !j) {
        o();
        return;
      }
      if (j.regions && j.regions.length > 0) {
        const W = G_(j);
        jt = W.layer, U = W.hitTest, I.addChild(jt);
      }
      if ((_a2 = j.trace) == null ? void 0 : _a2.length) {
        const W = j_(j.trace);
        if (W.length > 0) {
          const rt = J_(W);
          Z = rt.layer, it = rt.hitTest, I.addChild(Z);
        }
      }
      o();
    }
    const Lt = document.createElement("div");
    Lt.style.cssText = "position:absolute;bottom:34px;left:8px;background:rgba(0,0,0,0.8);color:#e0e0e0;font:11px monospace;padding:6px 8px;border-radius:3px;border:1px solid #00e0a0;z-index:10;display:none;min-width:200px", t.appendChild(Lt);
    const he = document.createElement("div");
    he.style.cssText = "color:#00e0a0;margin-bottom:4px", Lt.appendChild(he);
    const k = document.createElement("textarea");
    k.placeholder = "Add a note\u2026", k.rows = 2, k.style.cssText = "width:100%;box-sizing:border-box;background:#2a2a2a;color:#e0e0e0;border:1px solid #555;border-radius:2px;font:11px monospace;resize:vertical;margin-bottom:4px", Lt.appendChild(k);
    const P = document.createElement("div");
    P.style.cssText = "color:#777", P.textContent = "Ctrl+C to copy JSON", Lt.appendChild(P);
    let q = false, j = null, R = null, B = null, nt = null, H = null;
    const st = ov(t, (W) => H == null ? void 0 : H.seekTo(W));
    hv(t);
    const wt = zw({
      sidebarEl: document.getElementById("sidebar"),
      getSidebarCtrl: () => Rs,
      renderLayoutOnCanvas: Fe,
      setCachedValidationIssues: (W) => {
        mt = W;
      },
      updateValidationOverlay: Vt,
      panToTile: zt,
      onDebugEnable: () => d.setDebugEnabled(true),
      onClear: () => {
        wt.clear(), I.removeChildren(), V.removeChildren(), b.clearPin(), b.setTileContext(null), j = null, R = null, _.update(null), J = /* @__PURE__ */ new Map(), mt = null, Ds(r, null), r.moveCenter(en / 2, en / 2), Et(null), Rs == null ? void 0 : Rs.updateValidation([], zt), L.close();
      }
    });
    function Ot(W) {
      W.length === 0 ? (Lt.style.display = "none", k.value = "") : (he.textContent = `${W.length} entit${W.length === 1 ? "y" : "ies"} selected`, Lt.style.display = "block");
    }
    async function Bt(W) {
      if (q || !(W.regions ?? []).some((Ft) => Ft.kind === "crossing_zone")) return;
      q = true, st.markOptimizeState("active"), B && (B.destroy(), B = null);
      const ut = (Ft) => Ft === "transport-belt" || Ft === "fast-transport-belt" || Ft === "express-transport-belt" || Ft === "underground-belt" || Ft === "fast-underground-belt" || Ft === "express-underground-belt", yt = [], dt = 130, $t = 90, Wt = 520;
      let kt = 0, Nt = -1, ye = 0, be = false, Bn = false, ss = null;
      const Eu = (Ft) => {
        if (!j) return;
        const Xe = Ft.zone_x, We = Ft.zone_y, is = Xe + Ft.zone_w, vr = We + Ft.zone_h, ku = (Ls) => {
          const Ia = Ls.x ?? 0, Ra = Ls.y ?? 0;
          return Ia >= Xe && Ia < is && Ra >= We && Ra < vr;
        };
        j.entities = j.entities.filter((Ls) => !(ku(Ls) && ut(Ls.name))).concat(Ft.entities), qw(i, r, {
          x: Xe,
          y: We,
          w: Ft.zone_w,
          h: Ft.zone_h
        }), E(j), o();
      }, Au = () => be && yt.length === 0, Pa = (Ft) => {
        if (!Bn) {
          for (; yt.length > 0; ) {
            const Xe = yt[0], We = Xe.imp.region_id === Nt, is = We ? Math.min(Wt, ye * $t) : 0, vr = dt + is;
            if (Ft - kt < vr) break;
            yt.shift(), Eu(Xe.imp), We ? ye += 1 : (ye = 1, Nt = Xe.imp.region_id), kt = Ft;
            break;
          }
          if (Au()) {
            ss = null;
            return;
          }
          ss = requestAnimationFrame(Pa);
        }
      };
      ss = requestAnimationFrame(Pa);
      try {
        const Ft = await n.optimizeAllRegions(W, {
          perRegionBudgetMs: 800,
          onImprovement: (We) => {
            We.iter !== 0 && yt.push({
              imp: We
            });
          }
        });
        be = true, await new Promise((We) => {
          const is = () => {
            yt.length === 0 ? We() : requestAnimationFrame(is);
          };
          is();
        }), j = Ft, _.update(Ft), X(Ft), window.__layout = Ft;
        const Xe = E(Ft);
        b.setHighlightController(Q(Xe)), o();
      } catch (Ft) {
        (Ft instanceof Error ? Ft.message : String(Ft)).includes("superseded") || console.error("[auto-optimize] failed", Ft);
      } finally {
        Bn = true, be = true, ss !== null && cancelAnimationFrame(ss), q = false, st.markOptimizeState("done"), j && (B = Cc(i.canvas, r, I, j, Ot));
      }
    }
    i.canvas.addEventListener("pointermove", (W) => {
      const rt = i.canvas.getBoundingClientRect(), ut = W.clientX - rt.left, yt = W.clientY - rt.top, dt = r.toWorld(ut, yt), $t = Math.floor(dt.x / S), Wt = Math.floor(dt.y / S);
      b.setCursorTile($t, Wt);
      const kt = J.get(`${$t},${Wt}`) ?? null;
      kt !== $ && z(kt), $ || b.onHover(null, $t, Wt);
    }), i.canvas.addEventListener("pointerleave", () => {
      b.setCursorTile(null), $ && z(null);
    });
    const qt = 4;
    let ge = null;
    i.canvas.addEventListener("pointerdown", (W) => {
      if (W.button !== 0 || W.shiftKey || W.altKey || W.ctrlKey || W.metaKey) {
        ge = null;
        return;
      }
      ge = {
        x: W.clientX,
        y: W.clientY,
        shifted: false
      };
    }), i.canvas.addEventListener("pointerup", (W) => {
      if (!ge) return;
      const rt = W.clientX - ge.x, ut = W.clientY - ge.y;
      if (ge = null, Math.hypot(rt, ut) > qt || W.button !== 0 || W.shiftKey || W.altKey || W.ctrlKey || W.metaKey) return;
      const yt = i.canvas.getBoundingClientRect(), dt = r.toWorld(W.clientX - yt.left, W.clientY - yt.top), $t = Math.floor(dt.x / S), Wt = Math.floor(dt.y / S);
      if (!g.checked) {
        const be = $ && $.x === $t && $.y === Wt ? $ : null;
        if (!be) return;
        b.pinTile(be, $t, Wt);
        return;
      }
      const kt = (it == null ? void 0 : it(dt.x, dt.y)) ?? null;
      if (kt) {
        L.open(kt, j == null ? void 0 : j.trace);
        return;
      }
      if (T) {
        const be = dt.x / S, Bn = dt.y / S;
        if (!(be >= T.bboxX && Bn >= T.bboxY && be < T.bboxX + T.bboxW && Bn < T.bboxY + T.bboxH)) {
          L.close();
          return;
        }
      }
      const Nt = (U == null ? void 0 : U(dt.x, dt.y)) ?? null;
      if (Nt) {
        const be = (Nt.region.x + Nt.region.width / 2) * S, Bn = (Nt.region.y + Nt.region.height / 2) * S;
        r.moveCenter(be, Bn);
      }
      const ye = b.getPinnedTile();
      if (ye && ye.x === $t && ye.y === Wt) b.clearPin();
      else {
        const be = $ && $.x === $t && $.y === Wt ? $ : null;
        if (!be) return;
        b.pinTile(be, $t, Wt);
      }
    }), document.addEventListener("keydown", (W) => {
      W.key === "Shift" && r.plugins.pause("drag");
    }), document.addEventListener("keyup", (W) => {
      W.key === "Shift" && r.plugins.resume("drag");
    }), window.addEventListener("blur", () => r.plugins.resume("drag"));
    function es(W) {
      nt == null ? void 0 : nt.cancel(), nt = null, H == null ? void 0 : H.cancel(), H = null, st.reset(), I.removeChildren(), V.removeChildren(), R = W, Ds(r, W), W || r.moveCenter(en / 2, en / 2);
    }
    function Ma() {
      H == null ? void 0 : H.cancel(), st.reset(), Ds(r, null), H = nv(I, i);
      let W = false, rt = null;
      return (ut) => {
        if (ut.phase === "PhaseSnapshot") {
          const dt = ut.data;
          dt.width > 0 && dt.height > 0 && (W || (r.fit(true, dt.width * S * 1.15, dt.height * S * 1.25), r.moveCenter(dt.width * S / 2, dt.height * S / 2), W = true), rt ? rt.resize(dt.width + 2, dt.height + 2) : rt = Is(dt.width + 2, dt.height + 2));
        }
        H == null ? void 0 : H.onEvent(ut, (dt) => {
          H && st.noteMilestone(dt, H.getTimeRange());
        });
      };
    }
    function Is(W, rt) {
      let ut = W, yt = rt, dt = false;
      if (h) return Wn(c, ut, yt), o(), dt = true, {
        cancel: () => {
        },
        resize(Nt, ye) {
          Wn(c, Nt, ye), o();
        }
      };
      const $t = 250, Wt = performance.now();
      c.alpha = 1, Wn(c, ut, yt, 0);
      const kt = () => {
        if (dt) return;
        const Nt = Math.min(1, (performance.now() - Wt) / $t);
        Wn(c, ut, yt, Nt), o(), Nt >= 1 && (dt = true, h = true, i.ticker.remove(kt), l());
      };
      return i.ticker.add(kt), a(), {
        cancel() {
          dt || (dt = true, h = true, i.ticker.remove(kt), l(), Wn(c, ut, yt), o());
        },
        resize(Nt, ye) {
          ut = Nt, yt = ye, dt && (Wn(c, ut, yt), o());
        }
      };
    }
    function Fe(W, rt) {
      Vd(l0(W.entities)), j = W, _.update(W), rt && (R = rt), X(W), window.__layout = W, dv(W), D = false, K == null ? void 0 : K.cancel(), K = null, B && (B.destroy(), B = null), nt == null ? void 0 : nt.cancel(), nt = null, Lt.style.display = "none", k.value = "", mt = null, Ds(r, null);
      let ut;
      if (H == null ? void 0 : H.hasCommittedEntities()) ut = H.finish(W), st.arm(H.getTimeRange(), H.getMilestones());
      else if (H == null ? void 0 : H.cancel(), H = null, st.reset(), (Array.isArray(W.trace) ? W.trace : []).some((kt) => kt.phase === "PhaseSnapshot")) {
        const kt = Yw(W, I, z, F, i);
        ut = kt.controller, nt = kt.handle;
      } else ut = E(W);
      b.setHighlightController(Q(ut)), lt = Gw(W.trace), b.setTileContext(lt), b.clearPin(), B = Cc(i.canvas, r, I, W, Ot), ot(), Vt(), Rt(), At(), Ce(), sb(V, W, R);
      const yt = W.width ?? 0, dt = W.height ?? 0;
      if (Wn(c, yt + 2, dt + 2), yt > 0 && dt > 0) {
        const $t = yt * 32, Wt = dt * 32, kt = 192;
        r.fit(true, $t * 1.1, (Wt + kt) * 1.2), r.moveCenter($t / 2, (Wt - kt) / 2);
      }
      M && (I.alpha = 0.12), o(), requestAnimationFrame(() => {
        j === W && ns(W);
      });
    }
    function ns(W) {
      j !== W || ct === W || mt === null || [
        ...mt,
        ...Dt(W)
      ].length > 0 || Bt(W);
    }
    document.addEventListener("keydown", (W) => {
      if (W.ctrlKey) {
        if (W.key === "c") {
          if (!B || B.getSelected().length === 0) return;
          W.preventDefault();
          const rt = (Rs == null ? void 0 : Rs.getParams()) ?? null, ut = B.buildJson(rt, k.value.trim());
          navigator.clipboard.writeText(ut).catch(() => {
          }), P.textContent = "Copied!", setTimeout(() => {
            P.textContent = "Ctrl+C to copy JSON";
          }, 2e3);
        } else if (W.key === "o") {
          W.preventDefault();
          const rt = document.createElement("input");
          rt.type = "file", rt.accept = ".fls", rt.addEventListener("change", async () => {
            var _a2;
            const ut = (_a2 = rt.files) == null ? void 0 : _a2[0];
            if (ut) try {
              const yt = await ut.text(), dt = await vu(yt);
              wt.load(dt);
            } catch (yt) {
              alert(`Failed to load snapshot: ${yt}`);
            }
          }), rt.click();
        }
      }
    });
    const bn = document.getElementById("sidebar");
    let Rs = null;
    if (bn) {
      let W = function(kt) {
        const Nt = document.createElement("button");
        return Nt.textContent = kt, Nt.style.cssText = "flex:1;padding:10px 4px;background:none;border:none;border-bottom:2px solid transparent;color:#777;font:12px 'JetBrains Mono','Consolas',monospace;cursor:pointer;letter-spacing:0.5px;transition:all 0.15s", Nt;
      }, rt = function(kt) {
        const Nt = kt === "generate";
        $t.style.display = Nt ? "flex" : "none", Wt.style.display = Nt ? "none" : "flex", yt.style.borderBottomColor = Nt ? "#569cd6" : "transparent", yt.style.color = Nt ? "#d4d4d4" : "#777", dt.style.borderBottomColor = Nt ? "transparent" : "#569cd6", dt.style.color = Nt ? "#777" : "#d4d4d4";
      };
      const ut = document.createElement("div");
      ut.style.cssText = "display:flex;border-bottom:1px solid #2a2a2a;background:#141414;flex-shrink:0";
      const yt = W("Generate"), dt = W("Corpus");
      ut.appendChild(yt), ut.appendChild(dt);
      const $t = document.createElement("div");
      $t.style.cssText = "flex:1;overflow:hidden;display:flex;flex-direction:column;";
      const Wt = document.createElement("div");
      Wt.style.cssText = "flex:1;overflow:hidden;display:none;flex-direction:column;", bn.style.cssText += ";display:flex;flex-direction:column;padding:0;overflow:hidden;", bn.appendChild(ut), bn.appendChild($t), bn.appendChild(Wt), yt.onclick = () => rt("generate"), dt.onclick = () => rt("corpus"), rt("generate"), Rs = kb($t, n, {
        renderGraph: es,
        renderLayout: Fe,
        startStreaming: Ma
      }), u.addEventListener("change", () => {
        ot(), Vt(), Rt(), At();
      }), w.addEventListener("change", () => {
        At();
      }), x.addEventListener("change", () => {
        ot();
      }), p.addEventListener("change", () => {
        $i(p.checked), j && Fe(j);
      }), f.addEventListener("change", () => {
        se();
      }), m.addEventListener("change", () => {
        Ce();
      }), g.addEventListener("change", () => {
        Rt();
      }), y.addEventListener("change", () => {
        const kt = () => o();
        y.checked ? (M = true, O = {
          colorChecked: p.checked,
          regionsChecked: g.checked,
          entityAlpha: I.alpha
        }, g.checked || (g.checked = true, Rt()), p.checked && (p.checked = false, $i(false), j && Fe(j)), I.alpha = 0.12, Rt(), kt()) : (M = false, O && (I.alpha = O.entityAlpha, g.checked !== O.regionsChecked && (g.checked = O.regionsChecked, Rt()), p.checked !== O.colorChecked && (p.checked = O.colorChecked, $i(p.checked), j && Fe(j)), O = null), kt());
      }), Kb(Wt, Fe);
    }
  }
  pv().catch((n) => {
    console.error("Failed to initialize app:", n);
  });
})();
export {
  yh as $,
  ea as A,
  Ji as B,
  Ut as C,
  om as D,
  rn as E,
  Zp as F,
  fi as G,
  Zn as H,
  Cs as I,
  le as J,
  ee as K,
  Kt as L,
  qs as M,
  ah as N,
  vt as O,
  ht as P,
  ft as Q,
  Ht as R,
  ur as S,
  S as T,
  pe as U,
  fy as V,
  ag as W,
  fr as X,
  Zm as Y,
  yn as Z,
  mr as _,
  __tla,
  ir as a,
  Pt as a0,
  Fh as a1,
  vo as a2,
  Gt as a3,
  Yn as a4,
  ni as a5,
  Tt as a6,
  sp as a7,
  sm as a8,
  Ps as a9,
  ud as aA,
  Wh as aB,
  pi as aC,
  vf as aD,
  yf as aE,
  Kh as aF,
  cm as aG,
  $m as aH,
  Om as aI,
  Hm as aJ,
  zm as aK,
  Um as aL,
  dl as aa,
  Vi as ab,
  Zo as ac,
  Bh as ad,
  Me as ae,
  ta as af,
  Lm as ag,
  Bm as ah,
  Dm as ai,
  Wm as aj,
  Hu as ak,
  jm as al,
  ve as am,
  rh as an,
  ke as ao,
  cr as ap,
  Xa as aq,
  Ye as ar,
  jy as as,
  _p as at,
  Ka as au,
  Rr as av,
  Ja as aw,
  wp as ax,
  lh as ay,
  gr as az,
  Fn as b,
  _r as c,
  gv as d,
  Zi as e,
  Qo as f,
  Mt as g,
  Ct as h,
  Oo as i,
  Kn as j,
  cn as k,
  Bo as l,
  Ii as m,
  Xt as n,
  Be as o,
  gd as p,
  yd as q,
  ti as r,
  Re as s,
  mv as t,
  $y as u,
  te as v,
  wy as w,
  ne as x,
  Jt as y,
  St as z
};
