const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/browserAll-BNyAUVsS.js","assets/webworkerAll-aOtVoIbn.js","assets/Filter-BAI0dhlP.js","assets/WebGPURenderer-DIXZPbil.js","assets/BufferResource-DIYGG85Q.js","assets/RenderTargetSystem-0nmWFec1.js","assets/WebGLRenderer-BNABB76k.js","assets/CanvasRenderer-DYKzJi1K.js"])))=>i.map(i=>d[i]);
let ph, Qo, qi, Ut, nm, on, Xp, di, Qn, vs, ce, ne, Kt, Vs, sh, vt, ct, ft, Ht, hr, S, fe, hy, sg, ur, Xm, bn, pr, nr, Rt, $h, _o, zt, Xn, Qs, Et, Qu, Qf, Ms, ld, Bh, hi, xf, pf, Vh, rm, Pm, Rm, Wm, Nm, Gm, ll, Ui, Ko, Ih, ke, Zo, Mm, Im, Fm, Bm, Wu, zm, ve, eh, Ee, ar, ja, je, zy, gp, Ya, Pr, Xa, yp, ih, fr, Nn, xr, dv, Ki, Jo, It, Ct, $o, Jn, dn, Lo, Mi, Yt, $e, ud, pd, Js, Ie, hv, Py, ee, yy, se, Jt, St;
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
  const Tu = "modulepreload", Au = function(n) {
    return "/spaghettio/" + n;
  }, Pa = {}, Mn = function(t, e, s) {
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
        if (c = Au(c), c in Pa) return;
        Pa[c] = true;
        const h = c.endsWith(".css"), d = h ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${c}"]${d}`)) return;
        const u = document.createElement("link");
        if (u.rel = h ? "stylesheet" : Tu, h || (u.as = "script"), u.crossOrigin = "", u.href = c, l && u.setAttribute("nonce", l), document.head.appendChild(u), h) return new Promise((p, f) => {
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
  ct = ((n) => (n.Application = "application", n.WebGLPipes = "webgl-pipes", n.WebGLPipesAdaptor = "webgl-pipes-adaptor", n.WebGLSystem = "webgl-system", n.WebGPUPipes = "webgpu-pipes", n.WebGPUPipesAdaptor = "webgpu-pipes-adaptor", n.WebGPUSystem = "webgpu-system", n.CanvasSystem = "canvas-system", n.CanvasPipesAdaptor = "canvas-pipes-adaptor", n.CanvasPipes = "canvas-pipes", n.Asset = "asset", n.LoadParser = "load-parser", n.ResolveParser = "resolve-parser", n.CacheParser = "cache-parser", n.DetectionParser = "detection-parser", n.MaskEffect = "mask-effect", n.BlendMode = "blend-mode", n.TextureSource = "texture-source", n.Environment = "environment", n.ShapeBuilder = "shape-builder", n.Batcher = "batcher", n))(ct || {});
  let ho, mi, Eu, ku;
  ho = (n) => {
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
  mi = (n, t) => ho(n).priority ?? t;
  zt = {
    _addHandlers: {},
    _removeHandlers: {},
    _queue: {},
    remove(...n) {
      return n.map(ho).forEach((t) => {
        t.type.forEach((e) => {
          var _a2, _b2;
          return (_b2 = (_a2 = this._removeHandlers)[e]) == null ? void 0 : _b2.call(_a2, t);
        });
      }), this;
    },
    add(...n) {
      return n.map(ho).forEach((t) => {
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
        }), t.sort((r, o) => mi(o.value, e) - mi(r.value, e)));
      }, (s) => {
        const i = t.findIndex((r) => r.name === s.name);
        i !== -1 && t.splice(i, 1);
      });
    },
    handleByList(n, t, e = -1) {
      return this.handle(n, (s) => {
        t.includes(s.ref) || (t.push(s.ref), t.sort((i, r) => mi(r, e) - mi(i, e)));
      }, (s) => {
        const i = t.indexOf(s.ref);
        i !== -1 && t.splice(i, 1);
      });
    },
    mixin(n, ...t) {
      for (const e of t) Object.defineProperties(n.prototype, Object.getOwnPropertyDescriptors(e));
    }
  };
  Eu = {
    extension: {
      type: ct.Environment,
      name: "browser",
      priority: -1
    },
    test: () => true,
    load: async () => {
      await Mn(() => import("./browserAll-BNyAUVsS.js"), __vite__mapDeps([0,1,2]));
    }
  };
  ku = {
    extension: {
      type: ct.Environment,
      name: "webworker",
      priority: 0
    },
    test: () => typeof self < "u" && self.WorkerGlobalScope !== void 0,
    load: async () => {
      await Mn(() => import("./webworkerAll-aOtVoIbn.js"), __vite__mapDeps([1,2]));
    }
  };
  class pe {
    constructor(t, e, s) {
      this._x = e || 0, this._y = s || 0, this._observer = t;
    }
    clone(t) {
      return new pe(t ?? this._observer, this._x, this._y);
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
  function Hc(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var wr = {
    exports: {}
  }, Ia;
  function Mu() {
    return Ia || (Ia = 1, (function(n) {
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
          var b = g.length, _;
          for (x = 0; x < b; x++) switch (g[x].once && this.removeListener(c, g[x].fn, void 0, true), y) {
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
              if (!w) for (_ = 1, w = new Array(y - 1); _ < y; _++) w[_ - 1] = arguments[_];
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
    })(wr)), wr.exports;
  }
  var Pu = Mu();
  let Iu, Ru, Lu;
  on = Hc(Pu);
  Iu = Math.PI * 2;
  Ru = 180 / Math.PI;
  Lu = Math.PI / 180;
  Et = class {
    constructor(t = 0, e = 0) {
      this.x = 0, this.y = 0, this.x = t, this.y = e;
    }
    clone() {
      return new Et(this.x, this.y);
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
      return vr.x = 0, vr.y = 0, vr;
    }
  };
  const vr = new Et();
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
      e = e || new Et();
      const s = t.x, i = t.y;
      return e.x = this.a * s + this.c * i + this.tx, e.y = this.b * s + this.d * i + this.ty, e;
    }
    applyInverse(t, e) {
      e = e || new Et();
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
      return c < 1e-5 || Math.abs(Iu - c) < 1e-5 ? (t.rotation = l, t.skew.x = t.skew.y = 0) : (t.rotation = 0, t.skew.x = a, t.skew.y = l), t.scale.x = Math.sqrt(e * e + s * s), t.scale.y = Math.sqrt(i * i + r * r), t.position.x = this.tx + (o.x * e + o.y * i), t.position.y = this.ty + (o.x * s + o.y * r), t;
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
      return Bu.identity();
    }
    static get shared() {
      return $u.identity();
    }
  };
  const $u = new vt(), Bu = new vt(), Dn = [
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
  ], Hn = [
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
  ], Un = [
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
  ], jn = [
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
  ], uo = [], Uc = [], gi = Math.sign;
  function Ou() {
    for (let n = 0; n < 16; n++) {
      const t = [];
      uo.push(t);
      for (let e = 0; e < 16; e++) {
        const s = gi(Dn[n] * Dn[e] + Un[n] * Hn[e]), i = gi(Hn[n] * Dn[e] + jn[n] * Hn[e]), r = gi(Dn[n] * Un[e] + Un[n] * jn[e]), o = gi(Hn[n] * Un[e] + jn[n] * jn[e]);
        for (let a = 0; a < 16; a++) if (Dn[a] === s && Hn[a] === i && Un[a] === r && jn[a] === o) {
          t.push(a);
          break;
        }
      }
    }
    for (let n = 0; n < 16; n++) {
      const t = new vt();
      t.set(Dn[n], Hn[n], Un[n], jn[n], 0, 0), Uc.push(t);
    }
  }
  Ou();
  let yi;
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
    uX: (n) => Dn[n],
    uY: (n) => Hn[n],
    vX: (n) => Un[n],
    vY: (n) => jn[n],
    inv: (n) => n & 8 ? n & 15 : -n & 7,
    add: (n, t) => uo[n][t],
    sub: (n, t) => uo[n][St.inv(t)],
    rotate180: (n) => n ^ 4,
    isVertical: (n) => (n & 3) === 2,
    byDirection: (n, t) => Math.abs(n) * 2 <= Math.abs(t) ? t >= 0 ? St.S : St.N : Math.abs(t) * 2 <= Math.abs(n) ? n > 0 ? St.E : St.W : t > 0 ? n > 0 ? St.SE : St.SW : n > 0 ? St.NE : St.NW,
    matrixAppendRotationInv: (n, t, e = 0, s = 0, i = 0, r = 0) => {
      const o = Uc[St.inv(t)], a = o.a, l = o.b, c = o.c, h = o.d, d = e - Math.min(0, a * i, c * r, a * i + c * r), u = s - Math.min(0, l * i, h * r, l * i + h * r), p = n.a, f = n.b, m = n.c, g = n.d;
      n.a = a * p + l * m, n.b = a * f + l * g, n.c = c * p + h * m, n.d = c * f + h * g, n.tx = d * p + u * m + n.tx, n.ty = d * f + u * g + n.ty;
    },
    transformRectCoords: (n, t, e, s) => {
      const { x: i, y: r, width: o, height: a } = n, { x: l, y: c, width: h, height: d } = t;
      return e === St.E ? (s.set(i + l, r + c, o, a), s) : e === St.S ? s.set(h - r - a + l, i + c, a, o) : e === St.W ? s.set(h - i - o + l, d - r - a + c, o, a) : e === St.N ? s.set(r + l, d - i - o + c, a, o) : s.set(i + l, r + c, o, a);
    }
  };
  yi = [
    new Et(),
    new Et(),
    new Et(),
    new Et()
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
        const E = this.x < t.x ? t.x : this.x;
        if ((this.right > t.right ? t.right : this.right) <= E) return false;
        const k = this.y < t.y ? t.y : this.y;
        return (this.bottom > t.bottom ? t.bottom : this.bottom) > k;
      }
      const s = this.left, i = this.right, r = this.top, o = this.bottom;
      if (i <= s || o <= r) return false;
      const a = yi[0].set(t.left, t.top), l = yi[1].set(t.left, t.bottom), c = yi[2].set(t.right, t.top), h = yi[3].set(t.right, t.bottom);
      if (c.x <= a.x || l.y <= a.y) return false;
      const d = Math.sign(e.a * e.d - e.b * e.c);
      if (d === 0 || (e.apply(a, a), e.apply(l, l), e.apply(c, c), e.apply(h, h), Math.max(a.x, l.x, c.x, h.x) <= s || Math.min(a.x, l.x, c.x, h.x) >= i || Math.max(a.y, l.y, c.y, h.y) <= r || Math.min(a.y, l.y, c.y, h.y) >= o)) return false;
      const u = d * (l.y - a.y), p = d * (a.x - l.x), f = u * s + p * r, m = u * i + p * r, g = u * s + p * o, y = u * i + p * o;
      if (Math.max(f, m, g, y) <= u * a.x + p * a.y || Math.min(f, m, g, y) >= u * h.x + p * h.y) return false;
      const w = d * (a.y - c.y), x = d * (c.x - a.x), b = w * s + x * r, _ = w * i + x * r, v = w * s + x * o, C = w * i + x * o;
      return !(Math.max(b, _, v, C) <= w * a.x + x * a.y || Math.min(b, _, v, C) >= w * h.x + x * h.y);
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
  const Cr = {
    default: -1
  };
  ne = function(n = "default") {
    return Cr[n] === void 0 && (Cr[n] = -1), ++Cr[n];
  };
  let Ra, Nu, us;
  Ra = /* @__PURE__ */ new Set();
  ee = "8.0.0";
  Nu = "8.3.4";
  us = {
    quiet: false,
    noColor: false
  };
  It = ((n, t, e = 3) => {
    if (us.quiet || Ra.has(t)) return;
    let s = new Error().stack;
    const i = `${t}
Deprecated since v${n}`, r = typeof console.groupCollapsed == "function" && !us.noColor;
    typeof s > "u" ? console.warn("PixiJS Deprecation Warning: ", i) : (s = s.split(`
`).splice(e).join(`
`), r ? (console.groupCollapsed("%cPixiJS Deprecation Warning: %c%s", "color:#614108;background:#fffbe6", "font-weight:normal;color:#614108;background:#fffbe6", i), console.warn(s), console.groupEnd()) : (console.warn("PixiJS Deprecation Warning: ", i), console.warn(s))), Ra.add(t);
  });
  Object.defineProperties(It, {
    quiet: {
      get: () => us.quiet,
      set: (n) => {
        us.quiet = n;
      },
      enumerable: true,
      configurable: false
    },
    noColor: {
      get: () => us.noColor,
      set: (n) => {
        us.noColor = n;
      },
      enumerable: true,
      configurable: false
    }
  });
  const jc = () => {
  };
  function _s(n) {
    return n += n === 0 ? 1 : 0, --n, n |= n >>> 1, n |= n >>> 2, n |= n >>> 4, n |= n >>> 8, n |= n >>> 16, n + 1;
  }
  function La(n) {
    return !(n & n - 1) && !!n;
  }
  function Vc(n) {
    const t = {};
    for (const e in n) n[e] !== void 0 && (t[e] = n[e]);
    return t;
  }
  const $a = /* @__PURE__ */ Object.create(null);
  function Fu(n) {
    const t = $a[n];
    return t === void 0 && ($a[n] = ne("resource")), t;
  }
  const Yc = class Xc extends on {
    constructor(t = {}) {
      super(), this._resourceType = "textureSampler", this._touched = 0, this._maxAnisotropy = 1, this.destroyed = false, t = {
        ...Xc.defaultOptions,
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
      It(ee, "TextureStyle.wrapMode is now TextureStyle.addressMode"), this.addressMode = t;
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
      return this._sharedResourceId = Fu(t), this._resourceId;
    }
    destroy() {
      this.destroyed = true, this.emit("destroy", this), this.emit("change", this), this.removeAllListeners();
    }
  };
  Yc.defaultOptions = {
    addressMode: "clamp-to-edge",
    scaleMode: "linear"
  };
  Jn = Yc;
  const qc = class Kc extends on {
    constructor(t = {}) {
      super(), this.options = t, this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.uid = ne("textureSource"), this._resourceType = "textureSource", this._resourceId = ne("resource"), this.uploadMethodId = "unknown", this._resolution = 1, this.pixelWidth = 1, this.pixelHeight = 1, this.width = 1, this.height = 1, this.sampleCount = 1, this.mipLevelCount = 1, this.autoGenerateMipmaps = false, this.format = "rgba8unorm", this.dimension = "2d", this.viewDimension = "2d", this.arrayLayerCount = 1, this.antialias = false, this._touched = 0, this._batchTick = -1, this._textureBindLocation = -1, t = {
        ...Kc.defaultOptions,
        ...t
      }, this.label = t.label ?? "", this.resource = t.resource, this.autoGarbageCollect = t.autoGarbageCollect, this._resolution = t.resolution, t.width ? this.pixelWidth = t.width * this._resolution : this.pixelWidth = this.resource ? this.resourceWidth ?? 1 : 1, t.height ? this.pixelHeight = t.height * this._resolution : this.pixelHeight = this.resource ? this.resourceHeight ?? 1 : 1, this.width = this.pixelWidth / this._resolution, this.height = this.pixelHeight / this._resolution, this.format = t.format, this.dimension = t.dimensions, this.viewDimension = t.viewDimension ?? t.dimensions, this.arrayLayerCount = t.arrayLayerCount, this.mipLevelCount = t.mipLevelCount, this.autoGenerateMipmaps = t.autoGenerateMipmaps, this.sampleCount = t.sampleCount, this.antialias = t.antialias, this.alphaMode = t.alphaMode, this.style = new Jn(Vc(t)), this.destroyed = false, this._refreshPOT();
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
      this._resourceId = ne("resource"), this.emit("change", this), this.emit("unload", this);
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
      return this.width = i / s, this.height = r / s, this._resolution = s, this.pixelWidth === i && this.pixelHeight === r ? false : (this._refreshPOT(), this.pixelWidth = i, this.pixelHeight = r, this.emit("resize", this), this._resourceId = ne("resource"), this.emit("change", this), true);
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
      this.isPowerOfTwo = La(this.pixelWidth) && La(this.pixelHeight);
    }
    static test(t) {
      throw new Error("Unimplemented");
    }
  };
  qc.defaultOptions = {
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
  ke = qc;
  class Yo extends ke {
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
  Yo.extension = ct.TextureSource;
  const Ba = new vt();
  Wu = class {
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
      i && (Ba.set(s.width / i.width, 0, 0, s.height / i.height, -i.x / i.width, -i.y / i.height), this.mapCoord.append(Ba));
      const r = t.source, o = this.uClampFrame, a = this.clampMargin / r._resolution, l = this.clampOffset / r._resolution;
      return o[0] = (t.frame.x + a + l) / r.width, o[1] = (t.frame.y + a + l) / r.height, o[2] = (t.frame.x + t.frame.width - a + l) / r.width, o[3] = (t.frame.y + t.frame.height - a + l) / r.height, this.uClampOffset[0] = this.clampOffset / r.pixelWidth, this.uClampOffset[1] = this.clampOffset / r.pixelHeight, this.isSimple = t.frame.width === r.width && t.frame.height === r.height && t.rotate === 0, true;
    }
  };
  Ct = class extends on {
    constructor({ source: t, label: e, frame: s, orig: i, trim: r, defaultAnchor: o, defaultBorders: a, rotate: l, dynamic: c } = {}) {
      if (super(), this.uid = ne("texture"), this.uvs = {
        x0: 0,
        y0: 0,
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
        x3: 0,
        y3: 0
      }, this.frame = new Ht(), this.noFrame = false, this.dynamic = false, this.isTexture = true, this.label = e, this.source = (t == null ? void 0 : t.source) ?? new ke(), this.noFrame = !s, s) this.frame.copyFrom(s);
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
      return this._textureMatrix || (this._textureMatrix = new Wu(this)), this._textureMatrix;
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
      return It(ee, "Texture.baseTexture is now Texture.source"), this._source;
    }
  };
  Ct.EMPTY = new Ct({
    label: "EMPTY",
    source: new ke({
      label: "EMPTY"
    })
  });
  Ct.EMPTY.destroy = jc;
  Ct.WHITE = new Ct({
    source: new Yo({
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
  Ct.WHITE.destroy = jc;
  function Jc(n, t, e) {
    const { width: s, height: i } = e.orig, r = e.trim;
    if (r) {
      const o = r.width, a = r.height;
      n.minX = r.x - t._x * s, n.maxX = n.minX + o, n.minY = r.y - t._y * i, n.maxY = n.minY + a;
    } else n.minX = -t._x * s, n.maxX = n.minX + s, n.minY = -t._y * i, n.maxY = n.minY + i;
  }
  const Oa = new vt();
  Ee = class {
    constructor(t = 1 / 0, e = 1 / 0, s = -1 / 0, i = -1 / 0) {
      this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = Oa, this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
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
      return this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = Oa, this;
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
      return new Ee(this.minX, this.minY, this.maxX, this.maxY);
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
  var Gu = {
    grad: 0.9,
    turn: 360,
    rad: 360 / (2 * Math.PI)
  }, ln = function(n) {
    return typeof n == "string" ? n.length > 0 : typeof n == "number";
  }, le = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = Math.pow(10, t)), Math.round(e * n) / e + 0;
  }, Pe = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = 1), n > e ? e : n > t ? n : t;
  }, Zc = function(n) {
    return (n = isFinite(n) ? n % 360 : 0) > 0 ? n : n + 360;
  }, Na = function(n) {
    return {
      r: Pe(n.r, 0, 255),
      g: Pe(n.g, 0, 255),
      b: Pe(n.b, 0, 255),
      a: Pe(n.a)
    };
  }, Sr = function(n) {
    return {
      r: le(n.r),
      g: le(n.g),
      b: le(n.b),
      a: le(n.a, 3)
    };
  }, zu = /^#([0-9a-f]{3,8})$/i, xi = function(n) {
    var t = n.toString(16);
    return t.length < 2 ? "0" + t : t;
  }, Qc = function(n) {
    var t = n.r, e = n.g, s = n.b, i = n.a, r = Math.max(t, e, s), o = r - Math.min(t, e, s), a = o ? r === t ? (e - s) / o : r === e ? 2 + (s - t) / o : 4 + (t - e) / o : 0;
    return {
      h: 60 * (a < 0 ? a + 6 : a),
      s: r ? o / r * 100 : 0,
      v: r / 255 * 100,
      a: i
    };
  }, th = function(n) {
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
  }, Fa = function(n) {
    return {
      h: Zc(n.h),
      s: Pe(n.s, 0, 100),
      l: Pe(n.l, 0, 100),
      a: Pe(n.a)
    };
  }, Wa = function(n) {
    return {
      h: le(n.h),
      s: le(n.s),
      l: le(n.l),
      a: le(n.a, 3)
    };
  }, Ga = function(n) {
    return th((e = (t = n).s, {
      h: t.h,
      s: (e *= ((s = t.l) < 50 ? s : 100 - s) / 100) > 0 ? 2 * e / (s + e) * 100 : 0,
      v: s + e,
      a: t.a
    }));
    var t, e, s;
  }, js = function(n) {
    return {
      h: (t = Qc(n)).h,
      s: (i = (200 - (e = t.s)) * (s = t.v) / 100) > 0 && i < 200 ? e * s / 100 / (i <= 100 ? i : 200 - i) * 100 : 0,
      l: i / 2,
      a: t.a
    };
    var t, e, s, i;
  }, Du = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s*,\s*([+-]?\d*\.?\d+)%\s*,\s*([+-]?\d*\.?\d+)%\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Hu = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s+([+-]?\d*\.?\d+)%\s+([+-]?\d*\.?\d+)%\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Uu = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, ju = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, po = {
    string: [
      [
        function(n) {
          var t = zu.exec(n);
          return t ? (n = t[1]).length <= 4 ? {
            r: parseInt(n[0] + n[0], 16),
            g: parseInt(n[1] + n[1], 16),
            b: parseInt(n[2] + n[2], 16),
            a: n.length === 4 ? le(parseInt(n[3] + n[3], 16) / 255, 2) : 1
          } : n.length === 6 || n.length === 8 ? {
            r: parseInt(n.substr(0, 2), 16),
            g: parseInt(n.substr(2, 2), 16),
            b: parseInt(n.substr(4, 2), 16),
            a: n.length === 8 ? le(parseInt(n.substr(6, 2), 16) / 255, 2) : 1
          } : null : null;
        },
        "hex"
      ],
      [
        function(n) {
          var t = Uu.exec(n) || ju.exec(n);
          return t ? t[2] !== t[4] || t[4] !== t[6] ? null : Na({
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
          var t = Du.exec(n) || Hu.exec(n);
          if (!t) return null;
          var e, s, i = Fa({
            h: (e = t[1], s = t[2], s === void 0 && (s = "deg"), Number(e) * (Gu[s] || 1)),
            s: Number(t[3]),
            l: Number(t[4]),
            a: t[5] === void 0 ? 1 : Number(t[5]) / (t[6] ? 100 : 1)
          });
          return Ga(i);
        },
        "hsl"
      ]
    ],
    object: [
      [
        function(n) {
          var t = n.r, e = n.g, s = n.b, i = n.a, r = i === void 0 ? 1 : i;
          return ln(t) && ln(e) && ln(s) ? Na({
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
          if (!ln(t) || !ln(e) || !ln(s)) return null;
          var o = Fa({
            h: Number(t),
            s: Number(e),
            l: Number(s),
            a: Number(r)
          });
          return Ga(o);
        },
        "hsl"
      ],
      [
        function(n) {
          var t = n.h, e = n.s, s = n.v, i = n.a, r = i === void 0 ? 1 : i;
          if (!ln(t) || !ln(e) || !ln(s)) return null;
          var o = (function(a) {
            return {
              h: Zc(a.h),
              s: Pe(a.s, 0, 100),
              v: Pe(a.v, 0, 100),
              a: Pe(a.a)
            };
          })({
            h: Number(t),
            s: Number(e),
            v: Number(s),
            a: Number(r)
          });
          return th(o);
        },
        "hsv"
      ]
    ]
  }, za = function(n, t) {
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
  }, Vu = function(n) {
    return typeof n == "string" ? za(n.trim(), po.string) : typeof n == "object" && n !== null ? za(n, po.object) : [
      null,
      void 0
    ];
  }, Tr = function(n, t) {
    var e = js(n);
    return {
      h: e.h,
      s: Pe(e.s + 100 * t, 0, 100),
      l: e.l,
      a: e.a
    };
  }, Ar = function(n) {
    return (299 * n.r + 587 * n.g + 114 * n.b) / 1e3 / 255;
  }, Da = function(n, t) {
    var e = js(n);
    return {
      h: e.h,
      s: e.s,
      l: Pe(e.l + 100 * t, 0, 100),
      a: e.a
    };
  }, fo = (function() {
    function n(t) {
      this.parsed = Vu(t)[0], this.rgba = this.parsed || {
        r: 0,
        g: 0,
        b: 0,
        a: 1
      };
    }
    return n.prototype.isValid = function() {
      return this.parsed !== null;
    }, n.prototype.brightness = function() {
      return le(Ar(this.rgba), 2);
    }, n.prototype.isDark = function() {
      return Ar(this.rgba) < 0.5;
    }, n.prototype.isLight = function() {
      return Ar(this.rgba) >= 0.5;
    }, n.prototype.toHex = function() {
      return t = Sr(this.rgba), e = t.r, s = t.g, i = t.b, o = (r = t.a) < 1 ? xi(le(255 * r)) : "", "#" + xi(e) + xi(s) + xi(i) + o;
      var t, e, s, i, r, o;
    }, n.prototype.toRgb = function() {
      return Sr(this.rgba);
    }, n.prototype.toRgbString = function() {
      return t = Sr(this.rgba), e = t.r, s = t.g, i = t.b, (r = t.a) < 1 ? "rgba(" + e + ", " + s + ", " + i + ", " + r + ")" : "rgb(" + e + ", " + s + ", " + i + ")";
      var t, e, s, i, r;
    }, n.prototype.toHsl = function() {
      return Wa(js(this.rgba));
    }, n.prototype.toHslString = function() {
      return t = Wa(js(this.rgba)), e = t.h, s = t.s, i = t.l, (r = t.a) < 1 ? "hsla(" + e + ", " + s + "%, " + i + "%, " + r + ")" : "hsl(" + e + ", " + s + "%, " + i + "%)";
      var t, e, s, i, r;
    }, n.prototype.toHsv = function() {
      return t = Qc(this.rgba), {
        h: le(t.h),
        s: le(t.s),
        v: le(t.v),
        a: le(t.a, 3)
      };
      var t;
    }, n.prototype.invert = function() {
      return Ze({
        r: 255 - (t = this.rgba).r,
        g: 255 - t.g,
        b: 255 - t.b,
        a: t.a
      });
      var t;
    }, n.prototype.saturate = function(t) {
      return t === void 0 && (t = 0.1), Ze(Tr(this.rgba, t));
    }, n.prototype.desaturate = function(t) {
      return t === void 0 && (t = 0.1), Ze(Tr(this.rgba, -t));
    }, n.prototype.grayscale = function() {
      return Ze(Tr(this.rgba, -1));
    }, n.prototype.lighten = function(t) {
      return t === void 0 && (t = 0.1), Ze(Da(this.rgba, t));
    }, n.prototype.darken = function(t) {
      return t === void 0 && (t = 0.1), Ze(Da(this.rgba, -t));
    }, n.prototype.rotate = function(t) {
      return t === void 0 && (t = 15), this.hue(this.hue() + t);
    }, n.prototype.alpha = function(t) {
      return typeof t == "number" ? Ze({
        r: (e = this.rgba).r,
        g: e.g,
        b: e.b,
        a: t
      }) : le(this.rgba.a, 3);
      var e;
    }, n.prototype.hue = function(t) {
      var e = js(this.rgba);
      return typeof t == "number" ? Ze({
        h: t,
        s: e.s,
        l: e.l,
        a: e.a
      }) : le(e.h);
    }, n.prototype.isEqual = function(t) {
      return this.toHex() === Ze(t).toHex();
    }, n;
  })(), Ze = function(n) {
    return n instanceof fo ? n : new fo(n);
  }, Ha = [], Yu = function(n) {
    n.forEach(function(t) {
      Ha.indexOf(t) < 0 && (t(fo, po), Ha.push(t));
    });
  };
  function Xu(n, t) {
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
  Yu([
    Xu
  ]);
  const ws = class zs {
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
      if (t instanceof zs) this._value = this._cloneSource(t._value), this._int = t._int, this._components.set(t._components);
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
      const [e, s, i, r] = zs._temp.setValue(t)._components;
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
          const a = zs.HEX_PATTERN.exec(t);
          a && (t = `#${a[2]}`);
        }
        const o = Ze(t);
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
      return typeof t == "number" || typeof t == "string" || t instanceof Number || t instanceof zs || Array.isArray(t) || t instanceof Uint8Array || t instanceof Uint8ClampedArray || t instanceof Float32Array || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 && t.a !== void 0;
    }
  };
  ws.shared = new ws();
  ws._temp = new ws();
  ws.HEX_PATTERN = /^(#|0x)?(([a-f0-9]{3}){1,2}([a-f0-9]{2})?)$/i;
  Yt = ws;
  const qu = {
    cullArea: null,
    cullable: false,
    cullableChildren: true
  };
  let Er = 0;
  const Ua = 500;
  Jt = function(...n) {
    Er !== Ua && (Er++, Er === Ua ? console.warn("PixiJS Warning: too many warnings, no more warnings will be reported to the console by PixiJS.") : console.warn("PixiJS Warning: ", ...n));
  };
  hi = {
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
  class Ku {
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
  class Ju {
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
      return this._poolsByClass.has(t) || this._poolsByClass.set(t, new Ku(t)), this._poolsByClass.get(t);
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
  ve = new Ju();
  hi.register(ve);
  const Zu = {
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
      It("v8.6.0", "cacheAsBitmap is deprecated, use cacheAsTexture instead."), this.cacheAsTexture(n);
    }
  };
  Qu = function(n, t, e) {
    const s = n.length;
    let i;
    if (t >= s || e === 0) return;
    e = t + e > s ? s - t : e;
    const r = s - e;
    for (i = t; i < r; ++i) n[i] = n[i + e];
    n.length = r;
  };
  const tp = {
    allowChildren: true,
    removeChildren(n = 0, t) {
      var _a2;
      const e = t ?? this.children.length, s = e - n, i = [];
      if (s > 0 && s <= e) {
        for (let o = e - 1; o >= n; o--) {
          const a = this.children[o];
          a && (i.push(a), a.parent = null);
        }
        Qu(this.children, n, e);
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
      this.allowChildren || It(ee, "addChildAt: Only Containers will be allowed to add children in v8.0.0");
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
  }, ep = {
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
  ja = class {
    constructor() {
      this.pipe = "filter", this.priority = 1;
    }
    destroy() {
      for (let t = 0; t < this.filters.length; t++) this.filters[t].destroy();
      this.filters = null, this.filterArea = null;
    }
  };
  class np {
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
  const mo = new np();
  zt.handleByList(ct.MaskEffect, mo._effectClasses);
  const sp = {
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
      (t == null ? void 0 : t.mask) !== n && (t && (this.removeEffect(t), mo.returnMaskEffect(t), this._maskEffect = null), n != null && (this._maskEffect = mo.getMaskEffect(n), this.addEffect(this._maskEffect)));
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
      const t = this._filterEffect || (this._filterEffect = new ja());
      n = n;
      const e = (n == null ? void 0 : n.length) > 0, s = ((_a2 = t.filters) == null ? void 0 : _a2.length) > 0, i = e !== s;
      n = Array.isArray(n) ? n.slice(0) : n, t.filters = Object.freeze(n), i && (e ? this.addEffect(t) : (this.removeEffect(t), t.filters = n ?? null));
    },
    get filters() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filters;
    },
    set filterArea(n) {
      this._filterEffect || (this._filterEffect = new ja()), this._filterEffect.filterArea = n;
    },
    get filterArea() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filterArea;
    }
  }, ip = {
    label: null,
    get name() {
      return It(ee, "Container.name property has been removed, use Container.label instead"), this.label;
    },
    set name(n) {
      It(ee, "Container.name property has been removed, use Container.label instead"), this.label = n;
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
  }, xe = ve.getPool(vt), gn = ve.getPool(Ee), rp = new vt(), op = {
    getFastGlobalBounds(n, t) {
      t || (t = new Ee()), t.clear(), this._getGlobalBoundsRecursive(!!n, t, this.parentRenderLayer), t.isValid || t.set(0, 0, 0, 0);
      const e = this.renderGroup || this.parentRenderGroup;
      return t.applyMatrix(e.worldTransform), t;
    },
    _getGlobalBoundsRecursive(n, t, e) {
      let s = t;
      if (n && this.parentRenderLayer && this.parentRenderLayer !== e || this.localDisplayStatus !== 7 || !this.measurable) return;
      const i = !!this.effects.length;
      if ((this.renderGroup || i) && (s = gn.get().clear()), this.boundsArea) t.addRect(this.boundsArea, this.worldTransform);
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
        r && s.applyMatrix(o.worldTransform.copyTo(rp).invert()), t.addBounds(s), gn.return(s);
      } else this.renderGroup && (t.addBounds(s, this.relativeGroupTransform), gn.return(s));
    }
  };
  eh = function(n, t, e) {
    e.clear();
    let s, i;
    return n.parent ? t ? s = n.parent.worldTransform : (i = xe.get().identity(), s = Xo(n, i)) : s = vt.IDENTITY, nh(n, e, s, t), i && xe.return(i), e.isValid || e.set(0, 0, 0, 0), e;
  };
  function nh(n, t, e, s) {
    var _a2, _b2;
    if (!n.visible || !n.measurable) return;
    let i;
    s ? i = n.worldTransform : (n.updateLocalTransform(), i = xe.get(), i.appendFrom(n.localTransform, e));
    const r = t, o = !!n.effects.length;
    if (o && (t = gn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, i);
    else {
      const a = n.bounds;
      a && !a.isEmpty() && (t.matrix = i, t.addBounds(a));
      for (let l = 0; l < n.children.length; l++) nh(n.children[l], t, i, s);
    }
    if (o) {
      for (let a = 0; a < n.effects.length; a++) (_b2 = (_a2 = n.effects[a]).addBounds) == null ? void 0 : _b2.call(_a2, t);
      r.addBounds(t, vt.IDENTITY), gn.return(t);
    }
    s || xe.return(i);
  }
  function Xo(n, t) {
    const e = n.parent;
    return e && (Xo(e, t), e.updateLocalTransform(), t.append(e.localTransform)), t;
  }
  sh = function(n, t) {
    if (n === 16777215 || !t) return t;
    if (t === 16777215 || !n) return n;
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = t >> 16 & 255, o = t >> 8 & 255, a = t & 255, l = e * r / 255 | 0, c = s * o / 255 | 0, h = i * a / 255 | 0;
    return (l << 16) + (c << 8) + h;
  };
  const Va = 16777215;
  Ya = function(n, t) {
    return n === Va ? t : t === Va ? n : sh(n, t);
  };
  Vs = function(n) {
    return ((n & 255) << 16) + (n & 65280) + (n >> 16 & 255);
  };
  const ap = {
    getGlobalAlpha(n) {
      if (n) return this.renderGroup ? this.renderGroup.worldAlpha : this.parentRenderGroup ? this.parentRenderGroup.worldAlpha * this.alpha : this.alpha;
      let t = this.alpha, e = this.parent;
      for (; e; ) t *= e.alpha, e = e.parent;
      return t;
    },
    getGlobalTransform(n = new vt(), t) {
      if (t) return n.copyFrom(this.worldTransform);
      this.updateLocalTransform();
      const e = Xo(this, xe.get().identity());
      return n.appendFrom(this.localTransform, e), xe.return(e), n;
    },
    getGlobalTint(n) {
      if (n) return this.renderGroup ? Vs(this.renderGroup.worldColor) : this.parentRenderGroup ? Vs(Ya(this.localColor, this.parentRenderGroup.worldColor)) : this.tint;
      let t = this.localColor, e = this.parent;
      for (; e; ) t = Ya(t, e.localColor), e = e.parent;
      return Vs(t);
    }
  };
  ih = function(n, t, e) {
    return t.clear(), e || (e = vt.IDENTITY), rh(n, t, e, n, true), t.isValid || t.set(0, 0, 0, 0), t;
  };
  function rh(n, t, e, s, i) {
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
    if (a && (t = gn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, r);
    else {
      n.renderPipeId && (t.matrix = r, t.addBounds(n.bounds));
      const l = n.children;
      for (let c = 0; c < l.length; c++) rh(l[c], t, r, s, false);
    }
    if (a) {
      for (let l = 0; l < n.effects.length; l++) (_b2 = (_a2 = n.effects[l]).addLocalBounds) == null ? void 0 : _b2.call(_a2, t, s);
      o.addBounds(t, vt.IDENTITY), gn.return(t);
    }
    xe.return(r);
  }
  function oh(n, t) {
    const e = n.children;
    for (let s = 0; s < e.length; s++) {
      const i = e[s], r = i.uid, o = (i._didViewChangeTick & 65535) << 16 | i._didContainerChangeTick & 65535, a = t.index;
      (t.data[a] !== r || t.data[a + 1] !== o) && (t.data[t.index] = r, t.data[t.index + 1] = o, t.didChange = true), t.index = a + 2, i.children.length && oh(i, t);
    }
    return t.didChange;
  }
  const lp = new vt(), cp = {
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
        localBounds: new Ee()
      });
      const n = this._localBoundsCacheData;
      return n.index = 1, n.didChange = false, n.data[0] !== this._didViewChangeTick && (n.didChange = true, n.data[0] = this._didViewChangeTick), oh(this, n), n.didChange && ih(this, n.localBounds, lp), n.localBounds;
    },
    getBounds(n, t) {
      return eh(this, n, t || new Ee());
    }
  }, hp = {
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
  }, dp = {
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
      this.sortDirty && (this.sortDirty = false, this.children.sort(up));
    }
  };
  function up(n, t) {
    return n._zIndex - t._zIndex;
  }
  const pp = {
    getGlobalPosition(n = new Et(), t = false) {
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
  class qo {
    constructor() {
      this.uid = ne("instructionSet"), this.instructions = [], this.instructionSize = 0, this.renderables = [], this.gcTick = 0;
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
  let fp = 0;
  class mp {
    constructor(t) {
      this._poolKeyHash = /* @__PURE__ */ Object.create(null), this._texturePool = {}, this.textureOptions = t || {}, this.enableFullScreen = false, this.textureStyle = new Jn(this.textureOptions);
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
        label: `texturePool_${fp++}`
      });
    }
    getOptimalTexture(t, e, s = 1, i, r = false) {
      let o = Math.ceil(t * s - 1e-6), a = Math.ceil(e * s - 1e-6);
      o = _s(o), a = _s(a);
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
  ar = new mp();
  hi.register(ar);
  gp = class {
    constructor() {
      this.renderPipeId = "renderGroup", this.root = null, this.canBundle = false, this.renderGroupParent = null, this.renderGroupChildren = [], this.worldTransform = new vt(), this.worldColorAlpha = 4294967295, this.worldColor = 16777215, this.worldAlpha = 1, this.childrenToUpdate = /* @__PURE__ */ Object.create(null), this.updateTick = 0, this.gcTick = 0, this.childrenRenderablesToUpdate = {
        list: [],
        index: 0
      }, this.structureDidChange = true, this.instructionSet = new qo(), this._onRenderContainers = [], this.textureNeedsUpdate = true, this.isCachedAsTexture = false, this._matrixDirty = 7;
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
      this.isCachedAsTexture = false, this.texture && (ar.returnTexture(this.texture, true), this.texture = null);
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
  function go(n, t, e = {}) {
    for (const s in t) !e[s] && t[s] !== void 0 && (n[s] = t[s]);
  }
  let kr, bi, Mr, _i;
  kr = new pe(null);
  bi = new pe(null);
  Mr = new pe(null, 1, 1);
  _i = new pe(null);
  Xa = 1;
  yp = 2;
  Pr = 4;
  Ut = class extends on {
    constructor(t = {}) {
      var _a2, _b2;
      super(), this.uid = ne("renderable"), this._updateFlags = 15, this.renderGroup = null, this.parentRenderGroup = null, this.parentRenderGroupIndex = 0, this.didChange = false, this.didViewUpdate = false, this.relativeRenderGroupDepth = 0, this.children = [], this.parent = null, this.includeInBuild = true, this.measurable = true, this.isSimple = true, this.parentRenderLayer = null, this.updateTick = -1, this.localTransform = new vt(), this.relativeGroupTransform = new vt(), this.groupTransform = this.relativeGroupTransform, this.destroyed = false, this._position = new pe(this, 0, 0), this._scale = Mr, this._pivot = bi, this._origin = _i, this._skew = kr, this._cx = 1, this._sx = 0, this._cy = 0, this._sy = 1, this._rotation = 0, this.localColor = 16777215, this.localAlpha = 1, this.groupAlpha = 1, this.groupColor = 16777215, this.groupColorAlpha = 4294967295, this.localBlendMode = "inherit", this.groupBlendMode = "normal", this.localDisplayStatus = 7, this.globalDisplayStatus = 7, this._didContainerChangeTick = 0, this._didViewChangeTick = 0, this._didLocalTransformChangeId = -1, this.effects = [], go(this, t, {
        children: true,
        parent: true,
        effects: true
      }), (_a2 = t.children) == null ? void 0 : _a2.forEach((e) => this.addChild(e)), (_b2 = t.parent) == null ? void 0 : _b2.addChild(this);
    }
    static mixin(t) {
      It("8.8.0", "Container.mixin is deprecated, please use extensions.mixin instead."), zt.mixin(Ut, t);
    }
    set _didChangeId(t) {
      this._didViewChangeTick = t >> 12 & 4095, this._didContainerChangeTick = t & 4095;
    }
    get _didChangeId() {
      return this._didContainerChangeTick & 4095 | (this._didViewChangeTick & 4095) << 12;
    }
    addChild(...t) {
      if (this.allowChildren || It(ee, "addChild: Only Containers will be allowed to add children in v8.0.0"), t.length > 1) {
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
      t == null ? void 0 : t.removeChild(this), this.renderGroup = ve.get(gp, this), this.groupTransform = vt.IDENTITY, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
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
      return this.rotation * Ru;
    }
    set angle(t) {
      this.rotation = t * Lu;
    }
    get pivot() {
      return this._pivot === bi && (this._pivot = new pe(this, 0, 0)), this._pivot;
    }
    set pivot(t) {
      this._pivot === bi && (this._pivot = new pe(this, 0, 0), this._origin !== _i && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._pivot.set(t) : this._pivot.copyFrom(t);
    }
    get skew() {
      return this._skew === kr && (this._skew = new pe(this, 0, 0)), this._skew;
    }
    set skew(t) {
      this._skew === kr && (this._skew = new pe(this, 0, 0)), this._skew.copyFrom(t);
    }
    get scale() {
      return this._scale === Mr && (this._scale = new pe(this, 1, 1)), this._scale;
    }
    set scale(t) {
      this._scale === Mr && (this._scale = new pe(this, 0, 0)), typeof t == "string" && (t = parseFloat(t)), typeof t == "number" ? this._scale.set(t) : this._scale.copyFrom(t);
    }
    get origin() {
      return this._origin === _i && (this._origin = new pe(this, 0, 0)), this._origin;
    }
    set origin(t) {
      this._origin === _i && (this._origin = new pe(this, 0, 0), this._pivot !== bi && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._origin.set(t) : this._origin.copyFrom(t);
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
      t !== this.localAlpha && (this.localAlpha = t, this._updateFlags |= Xa, this._onUpdate());
    }
    get alpha() {
      return this.localAlpha;
    }
    set tint(t) {
      const s = Yt.shared.setValue(t ?? 16777215).toBgrNumber();
      s !== this.localColor && (this.localColor = s, this._updateFlags |= Xa, this._onUpdate());
    }
    get tint() {
      return Vs(this.localColor);
    }
    set blendMode(t) {
      this.localBlendMode !== t && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= yp, this.localBlendMode = t, this._onUpdate());
    }
    get blendMode() {
      return this.localBlendMode;
    }
    get visible() {
      return !!(this.localDisplayStatus & 2);
    }
    set visible(t) {
      const e = t ? 2 : 0;
      (this.localDisplayStatus & 2) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Pr, this.localDisplayStatus ^= 2, this._onUpdate(), this.emit("visibleChanged", t));
    }
    get culled() {
      return !(this.localDisplayStatus & 4);
    }
    set culled(t) {
      const e = t ? 0 : 4;
      (this.localDisplayStatus & 4) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Pr, this.localDisplayStatus ^= 4, this._onUpdate());
    }
    get renderable() {
      return !!(this.localDisplayStatus & 1);
    }
    set renderable(t) {
      const e = t ? 1 : 0;
      (this.localDisplayStatus & 1) !== e && (this._updateFlags |= Pr, this.localDisplayStatus ^= 1, this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._onUpdate());
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
  zt.mixin(Ut, tp, op, pp, hp, cp, sp, ip, dp, qu, Zu, ap, ep);
  class lr extends Ut {
    constructor(t) {
      super(t), this.canBundle = true, this.allowChildren = false, this._roundPixels = 0, this._lastUsed = -1, this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this._bounds = new Ee(0, 1, 0, 0), this._boundsDirty = true, this.autoGarbageCollect = t.autoGarbageCollect ?? true;
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
  je = class extends lr {
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
      }, this._anchor = new pe({
        _onUpdate: () => {
          this.onViewUpdate();
        }
      }), s ? this.anchor = s : e.defaultAnchor && (this.anchor = e.defaultAnchor), this.texture = e, this.allowChildren = false, this.roundPixels = i ?? false, r !== void 0 && (this.width = r), o !== void 0 && (this.height = o);
    }
    static from(t, e = false) {
      return t instanceof Ct ? new je(t) : new je(Ct.from(t, e));
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
      return Jc(this._visualBounds, this._anchor, this._texture), this._visualBounds;
    }
    get sourceBounds() {
      return It("8.6.1", "Sprite.sourceBounds is deprecated, use visualBounds instead."), this.visualBounds;
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
  const xp = new Ee();
  function ah(n, t, e) {
    const s = xp;
    n.measurable = true, eh(n, e, s), t.addBoundsMask(s), n.measurable = false;
  }
  function lh(n, t, e) {
    const s = gn.get();
    n.measurable = true;
    const i = xe.get().identity(), r = ch(n, e, i);
    ih(n, s, r), n.measurable = false, t.addBoundsMask(s), xe.return(i), gn.return(s);
  }
  function ch(n, t, e) {
    return n ? (n !== t && (ch(n.parent, t, e), n.updateLocalTransform(), e.append(n.localTransform)), e) : (Jt("Mask bounds, renderable is not inside the root container"), e);
  }
  class hh {
    constructor(t) {
      this.priority = 0, this.inverse = false, this.pipe = "alphaMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t, this.renderMaskToTexture = !(t instanceof je), this.mask.renderable = this.renderMaskToTexture, this.mask.includeInBuild = !this.renderMaskToTexture, this.mask.measurable = false;
    }
    reset() {
      this.mask !== null && (this.mask.measurable = true, this.mask = null);
    }
    addBounds(t, e) {
      this.inverse || ah(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      lh(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof je;
    }
  }
  hh.extension = ct.MaskEffect;
  class dh {
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
  dh.extension = ct.MaskEffect;
  class uh {
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
      ah(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      lh(this.mask, t, e);
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
  uh.extension = ct.MaskEffect;
  const bp = {
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
  let qa = bp;
  Rt = {
    get() {
      return qa;
    },
    set(n) {
      qa = n;
    }
  };
  ph = class extends ke {
    constructor(t) {
      t.resource || (t.resource = Rt.get().createCanvas()), t.width || (t.width = t.resource.width, t.autoDensity || (t.width /= t.resolution)), t.height || (t.height = t.resource.height, t.autoDensity || (t.height /= t.resolution)), super(t), this.uploadMethodId = "image", this.autoDensity = t.autoDensity, this.resizeCanvas(), this.transparent = !!t.transparent;
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
  ph.extension = ct.TextureSource;
  vs = class extends ke {
    constructor(t) {
      super(t), this.uploadMethodId = "image", this.autoGarbageCollect = true;
    }
    static test(t) {
      return globalThis.HTMLImageElement && t instanceof HTMLImageElement || typeof ImageBitmap < "u" && t instanceof ImageBitmap || globalThis.VideoFrame && t instanceof VideoFrame;
    }
  };
  vs.extension = ct.TextureSource;
  Qs = ((n) => (n[n.INTERACTION = 50] = "INTERACTION", n[n.HIGH = 25] = "HIGH", n[n.NORMAL = 0] = "NORMAL", n[n.LOW = -25] = "LOW", n[n.UTILITY = -50] = "UTILITY", n))(Qs || {});
  class Ir {
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
  const fh = class Ce {
    constructor() {
      this.autoStart = false, this.deltaTime = 1, this.lastTime = -1, this.speed = 1, this.started = false, this._requestId = null, this._maxElapsedMS = 100, this._minElapsedMS = 0, this._protected = false, this._lastFrame = -1, this._head = new Ir(null, null, 1 / 0), this.deltaMS = 1 / Ce.targetFPMS, this.elapsedMS = 1 / Ce.targetFPMS, this._tick = (t) => {
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
    add(t, e, s = Qs.NORMAL) {
      return this._addListener(new Ir(t, e, s));
    }
    addOnce(t, e, s = Qs.NORMAL) {
      return this._addListener(new Ir(t, e, s, true));
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
        this.deltaMS = e, this.deltaTime = this.deltaMS * Ce.targetFPMS;
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
      const e = Math.min(Math.max(0, t) / 1e3, Ce.targetFPMS);
      this._maxElapsedMS = 1 / e, this._minElapsedMS && t > this.maxFPS && (this.maxFPS = t);
    }
    get maxFPS() {
      return this._minElapsedMS ? Math.round(1e3 / this._minElapsedMS) : 0;
    }
    set maxFPS(t) {
      t === 0 ? this._minElapsedMS = 0 : (t < this.minFPS && (this.minFPS = t), this._minElapsedMS = 1 / (t / 1e3));
    }
    static get shared() {
      if (!Ce._shared) {
        const t = Ce._shared = new Ce();
        t.autoStart = true, t._protected = true;
      }
      return Ce._shared;
    }
    static get system() {
      if (!Ce._system) {
        const t = Ce._system = new Ce();
        t.autoStart = true, t._protected = true;
      }
      return Ce._system;
    }
  };
  fh.targetFPMS = 0.06;
  let Rr;
  Xn = fh;
  async function mh() {
    return Rr ?? (Rr = (async () => {
      var _a2;
      const t = Rt.get().createCanvas(1, 1).getContext("webgl");
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
    })()), Rr;
  }
  const cr = class gh extends ke {
    constructor(t) {
      super(t), this.isReady = false, this.uploadMethodId = "video", t = {
        ...gh.defaultOptions,
        ...t
      }, this._autoUpdate = true, this._isConnectedToTicker = false, this._updateFPS = t.updateFPS || 0, this._msToNextUpdate = 0, this.autoPlay = t.autoPlay !== false, this.alphaMode = t.alphaMode ?? "premultiply-alpha-on-upload", this._videoFrameRequestCallback = this._videoFrameRequestCallback.bind(this), this._videoFrameRequestCallbackHandle = null, this._load = null, this._resolve = null, this._reject = null, this._onCanPlay = this._onCanPlay.bind(this), this._onCanPlayThrough = this._onCanPlayThrough.bind(this), this._onError = this._onError.bind(this), this._onPlayStart = this._onPlayStart.bind(this), this._onPlayStop = this._onPlayStop.bind(this), this._onSeeked = this._onSeeked.bind(this), t.autoLoad !== false && this.load();
    }
    updateFrame() {
      if (!this.destroyed) {
        if (this._updateFPS) {
          const t = Xn.shared.elapsedMS * this.resource.playbackRate;
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
      return (t.readyState === t.HAVE_ENOUGH_DATA || t.readyState === t.HAVE_FUTURE_DATA) && t.width && t.height && (t.complete = true), t.addEventListener("play", this._onPlayStart), t.addEventListener("pause", this._onPlayStop), t.addEventListener("seeked", this._onSeeked), this._isSourceReady() ? this._mediaReady() : (e.preload || t.addEventListener("canplay", this._onCanPlay), t.addEventListener("canplaythrough", this._onCanPlayThrough), t.addEventListener("error", this._onError, true)), this.alphaMode = await mh(), this._load = new Promise((s, i) => {
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
      this._autoUpdate && this._isSourcePlaying() ? !this._updateFPS && this.resource.requestVideoFrameCallback ? (this._isConnectedToTicker && (Xn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0), this._videoFrameRequestCallbackHandle === null && (this._videoFrameRequestCallbackHandle = this.resource.requestVideoFrameCallback(this._videoFrameRequestCallback))) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker || (Xn.shared.add(this.updateFrame, this), this._isConnectedToTicker = true, this._msToNextUpdate = 0)) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker && (Xn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0));
    }
    static test(t) {
      return globalThis.HTMLVideoElement && t instanceof HTMLVideoElement;
    }
  };
  cr.extension = ct.TextureSource;
  cr.defaultOptions = {
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
  cr.MIME_TYPES = {
    ogv: "video/ogg",
    mov: "video/quicktime",
    m4v: "video/mp4"
  };
  let Ys = cr;
  const ze = (n, t, e = false) => (Array.isArray(n) || (n = [
    n
  ]), t ? n.map((s) => typeof s == "string" || e ? t(s) : s) : n);
  class _p {
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
      const s = ze(t);
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
  let yo;
  se = new _p();
  yo = [];
  zt.handleByList(ct.TextureSource, yo);
  function yh(n = {}) {
    const t = n && n.resource, e = t ? n.resource : n, s = t ? n : {
      resource: n
    };
    for (let i = 0; i < yo.length; i++) {
      const r = yo[i];
      if (r.test(e)) return new r(s);
    }
    throw new Error(`Could not find a source type for resource: ${s.resource}`);
  }
  function wp(n = {}, t = false) {
    const e = n && n.resource, s = e ? n.resource : n, i = e ? n : {
      resource: n
    };
    if (!t && se.has(s)) return se.get(s);
    const r = new Ct({
      source: yh(i)
    });
    return r.on("destroy", () => {
      se.has(s) && se.remove(s);
    }), t || se.set(s, r), r;
  }
  function vp(n, t = false) {
    return typeof n == "string" ? se.get(n) : n instanceof ke ? new Ct({
      source: n
    }) : wp(n, t);
  }
  Ct.from = vp;
  ke.from = yh;
  zt.add(hh, dh, uh, Ys, vs, ph, Yo);
  var Rn = ((n) => (n[n.Low = 0] = "Low", n[n.Normal = 1] = "Normal", n[n.High = 2] = "High", n))(Rn || {});
  function Fe(n) {
    if (typeof n != "string") throw new TypeError(`Path must be a string. Received ${JSON.stringify(n)}`);
  }
  function Is(n) {
    return n.split("?")[0].split("#")[0];
  }
  function Cp(n) {
    return n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  function Sp(n, t, e) {
    return n.replace(new RegExp(Cp(t), "g"), e);
  }
  function Tp(n, t) {
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
      return Sp(n, "\\", "/");
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
      Fe(n), n = this.toPosix(n);
      const t = /^file:\/\/\//.exec(n);
      if (t) return t[0];
      const e = /^[^/:]+:\/{0,2}/.exec(n);
      return e ? e[0] : "";
    },
    toAbsolute(n, t, e) {
      if (Fe(n), this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      const s = Is(this.toPosix(t ?? Rt.get().getBaseUrl())), i = Is(this.toPosix(e ?? this.rootname(s)));
      return n = this.toPosix(n), n.startsWith("/") ? Ae.join(i, n.slice(1)) : this.isAbsolute(n) ? n : this.join(s, n);
    },
    normalize(n) {
      if (Fe(n), n.length === 0) return ".";
      if (this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      n = this.toPosix(n);
      let t = "";
      const e = n.startsWith("/");
      this.hasProtocol(n) && (t = this.rootname(n), n = n.slice(t.length));
      const s = n.endsWith("/");
      return n = Tp(n), n.length > 0 && s && (n += "/"), e ? `/${n}` : t + n;
    },
    isAbsolute(n) {
      return Fe(n), n = this.toPosix(n), this.hasProtocol(n) ? true : n.startsWith("/");
    },
    join(...n) {
      if (n.length === 0) return ".";
      let t;
      for (let e = 0; e < n.length; ++e) {
        const s = n[e];
        if (Fe(s), s.length > 0) if (t === void 0) t = s;
        else {
          const i = n[e - 1] ?? "";
          this.joinExtensions.includes(this.extname(i).toLowerCase()) ? t += `/../${s}` : t += `/${s}`;
        }
      }
      return t === void 0 ? "." : this.normalize(t);
    },
    dirname(n) {
      if (Fe(n), n.length === 0) return ".";
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
      Fe(n), n = this.toPosix(n);
      let t = "";
      if (n.startsWith("/") ? t = "/" : t = this.getProtocol(n), this.isUrl(n)) {
        const e = n.indexOf("/", t.length);
        e !== -1 ? t = n.slice(0, e) : t = n, t.endsWith("/") || (t += "/");
      }
      return t;
    },
    basename(n, t) {
      Fe(n), t && Fe(t), n = Is(this.toPosix(n));
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
      Fe(n), n = Is(this.toPosix(n));
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
      Fe(n);
      const t = {
        root: "",
        dir: "",
        base: "",
        ext: "",
        name: ""
      };
      if (n.length === 0) return t;
      n = Is(this.toPosix(n));
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
  function xh(n, t, e, s, i) {
    const r = t[e];
    for (let o = 0; o < r.length; o++) {
      const a = r[o];
      e < t.length - 1 ? xh(n.replace(s[e], a), t, e + 1, s, i) : i.push(n.replace(s[e], a));
    }
  }
  function Ap(n) {
    const t = /\{(.*?)\}/g, e = n.match(t), s = [];
    if (e) {
      const i = [];
      e.forEach((r) => {
        const o = r.substring(1, r.length - 1).split(",");
        i.push(o);
      }), xh(n, i, 0, e, s);
    } else s.push(n);
    return s;
  }
  const Xi = (n) => !Array.isArray(n);
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
      return ze(e || s, (r) => typeof r == "string" ? r : Array.isArray(r) ? r.map((o) => (o == null ? void 0 : o.src) ?? o) : (r == null ? void 0 : r.src) ? r.src : r, true);
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
      }, ze(e).forEach((r) => {
        const { src: o } = r;
        let { data: a, format: l, loadParser: c, parser: h } = r;
        const d = ze(o).map((m) => typeof m == "string" ? Ap(m) : Array.isArray(m) ? m : [
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
      const e = Xi(t);
      t = ze(t);
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
      const e = Xi(t);
      t = ze(t);
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
      }, t.loadParser = r ?? t.loadParser, t.parser = o ?? t.parser, t.format = a ?? t.format ?? Ep(t.src), l !== void 0 && (t.progressSize = l), t;
    }
  }
  As.RETINA_PREFIX = /@([0-9\.]+)x/;
  function Ep(n) {
    return n.split(".").pop().split("?").shift().split("#").shift();
  }
  const xo = (n, t) => {
    const e = t.split("?")[1];
    return e && (n += `?${e}`), n;
  }, bh = class Ds {
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
        this._callback = t, this._batchIndex = 0, this._frameKeys.length <= Ds.BATCH_SIZE ? (this._processFrames(0), this._processAnimations(), this._parseComplete()) : this._nextBatch();
      });
    }
    parseSync() {
      return this._processFrames(0, true), this._processAnimations(), this.textures;
    }
    _processFrames(t, e = false) {
      let s = t;
      const i = e ? 1 / 0 : Ds.BATCH_SIZE;
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
      this._processFrames(this._batchIndex * Ds.BATCH_SIZE), this._batchIndex++, setTimeout(() => {
        this._batchIndex * Ds.BATCH_SIZE < this._frameKeys.length ? this._nextBatch() : (this._processAnimations(), this._parseComplete());
      }, 0);
    }
    destroy(t = false) {
      var _a2;
      for (const e in this.textures) this.textures[e].destroy();
      this._frames = null, this._frameKeys = null, this.data = null, this.textures = null, t && ((_a2 = this._texture) == null ? void 0 : _a2.destroy(), this.textureSource.destroy()), this._texture = null, this.textureSource = null, this.linkedSheets = [];
    }
  };
  bh.BATCH_SIZE = 1e3;
  let Ka = bh;
  const kp = [
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
  function _h(n, t, e) {
    const s = {};
    if (n.forEach((i) => {
      s[i] = t;
    }), Object.keys(t.textures).forEach((i) => {
      s[`${t.cachePrefix}${i}`] = t.textures[i];
    }), !e) {
      const i = Ae.dirname(n[0]);
      t.linkedSheets.forEach((r, o) => {
        const a = _h([
          `${i}/${t.data.meta.related_multi_packs[o]}`
        ], r, true);
        Object.assign(s, a);
      });
    }
    return s;
  }
  const Mp = {
    extension: ct.Asset,
    cache: {
      test: (n) => n instanceof Ka,
      getCacheableAssets: (n, t) => _h(n, t, false)
    },
    resolver: {
      extension: {
        type: ct.ResolveParser,
        name: "resolveSpritesheet"
      },
      test: (n) => {
        const e = n.split("?")[0].split("."), s = e.pop(), i = e.pop();
        return s === "json" && kp.includes(i);
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
        type: ct.LoadParser,
        priority: Rn.Normal,
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
          const d = xo(a + (i ?? n.meta.image), t.src);
          l = (await e.load([
            {
              src: d,
              data: r
            }
          ]))[d];
        }
        const c = new Ka({
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
            ((_b2 = t.data) == null ? void 0 : _b2.ignoreMultiPack) || (f = xo(f, t.src), d.push(e.load({
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
  zt.add(Mp);
  const Lr = /* @__PURE__ */ Object.create(null), Ja = /* @__PURE__ */ Object.create(null);
  Ko = function(n, t) {
    let e = Ja[n];
    return e === void 0 && (Lr[t] === void 0 && (Lr[t] = 1), Ja[n] = e = Lr[t]++), e;
  };
  let wi;
  function wh() {
    return (!wi || (wi == null ? void 0 : wi.isContextLost())) && (wi = Rt.get().createCanvas().getContext("webgl", {})), wi;
  }
  let vi;
  function Pp() {
    if (!vi) {
      vi = "mediump";
      const n = wh();
      n && n.getShaderPrecisionFormat && (vi = n.getShaderPrecisionFormat(n.FRAGMENT_SHADER, n.HIGH_FLOAT).precision ? "highp" : "mediump");
    }
    return vi;
  }
  function Ip(n, t, e) {
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
  function Rp(n, t, e) {
    const s = e ? t.maxSupportedFragmentPrecision : t.maxSupportedVertexPrecision;
    if (n.substring(0, 9) !== "precision") {
      let i = e ? t.requestedFragmentPrecision : t.requestedVertexPrecision;
      return i === "highp" && s !== "highp" && (i = "mediump"), `precision ${i} float;
${n}`;
    } else if (s !== "highp" && n.substring(0, 15) === "precision highp") return n.replace("precision highp", "precision mediump");
    return n;
  }
  function Lp(n, t) {
    return t ? `#version 300 es
${n}` : n;
  }
  const $p = {}, Bp = {};
  function Op(n, { name: t = "pixi-program" }, e = true) {
    t = t.replace(/\s+/g, "-"), t += e ? "-fragment" : "-vertex";
    const s = e ? $p : Bp;
    return s[t] ? (s[t]++, t += `-${s[t]}`) : s[t] = 1, n.indexOf("#define SHADER_NAME") !== -1 ? n : `${`#define SHADER_NAME ${t}`}
${n}`;
  }
  function Np(n, t) {
    return t ? n.replace("#version 300 es", "") : n;
  }
  const $r = {
    stripVersion: Np,
    ensurePrecision: Rp,
    addProgramDefines: Ip,
    setProgramName: Op,
    insertVersion: Lp
  }, Rs = /* @__PURE__ */ Object.create(null), vh = class bo {
    constructor(t) {
      t = {
        ...bo.defaultOptions,
        ...t
      };
      const e = t.fragment.indexOf("#version 300 es") !== -1, s = {
        stripVersion: e,
        ensurePrecision: {
          requestedFragmentPrecision: t.preferredFragmentPrecision,
          requestedVertexPrecision: t.preferredVertexPrecision,
          maxSupportedVertexPrecision: "highp",
          maxSupportedFragmentPrecision: Pp()
        },
        setProgramName: {
          name: t.name
        },
        addProgramDefines: e,
        insertVersion: e
      };
      let i = t.fragment, r = t.vertex;
      Object.keys($r).forEach((o) => {
        const a = s[o];
        i = $r[o](i, a, true), r = $r[o](r, a, false);
      }), this.fragment = i, this.vertex = r, this.transformFeedbackVaryings = t.transformFeedbackVaryings, this._key = Ko(`${this.vertex}:${this.fragment}`, "gl-program");
    }
    destroy() {
      this.fragment = null, this.vertex = null, this._attributeData = null, this._uniformData = null, this._uniformBlockData = null, this.transformFeedbackVaryings = null, Rs[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex}:${t.fragment}`;
      return Rs[e] || (Rs[e] = new bo(t), Rs[e]._cacheKey = e), Rs[e];
    }
  };
  vh.defaultOptions = {
    preferredVertexPrecision: "highp",
    preferredFragmentPrecision: "mediump"
  };
  Jo = vh;
  const Za = {
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
  qi = function(n) {
    return Za[n] ?? Za.float32;
  };
  const Fp = {
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
  }, Qa = /@location\((\d+)\)\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_<>]+)(?:,|\s|\)|$)/g;
  function tl(n, t) {
    let e;
    for (; (e = Qa.exec(n)) !== null; ) {
      const s = Fp[e[3]] ?? "float32";
      t[e[2]] = {
        location: parseInt(e[1], 10),
        format: s,
        stride: qi(s).stride,
        offset: 0,
        instance: false,
        start: 0
      };
    }
    Qa.lastIndex = 0;
  }
  function Wp(n) {
    return n.replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  }
  function Gp({ source: n, entryPoint: t }) {
    const e = {}, s = Wp(n), i = s.indexOf(`fn ${t}(`);
    if (i === -1) return e;
    const r = s.indexOf("->", i);
    if (r === -1) return e;
    const o = s.substring(i, r);
    if (tl(o, e), Object.keys(e).length === 0) {
      const a = o.match(/\(\s*\w+\s*:\s*(\w+)/);
      if (a) {
        const l = a[1], c = new RegExp(`struct\\s+${l}\\s*\\{([^}]+)\\}`, "s"), h = s.match(c);
        h && tl(h[1], e);
      }
    }
    return e;
  }
  function Br(n) {
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
  var Vn = ((n) => (n[n.VERTEX = 1] = "VERTEX", n[n.FRAGMENT = 2] = "FRAGMENT", n[n.COMPUTE = 4] = "COMPUTE", n))(Vn || {});
  function zp({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = []), s.isUniform ? t[s.group].push({
        binding: s.binding,
        visibility: Vn.VERTEX | Vn.FRAGMENT,
        buffer: {
          type: "uniform"
        }
      }) : s.type === "sampler" ? t[s.group].push({
        binding: s.binding,
        visibility: Vn.FRAGMENT,
        sampler: {
          type: "filtering"
        }
      }) : s.type === "texture_2d" || s.type.startsWith("texture_2d<") ? t[s.group].push({
        binding: s.binding,
        visibility: Vn.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d",
          multisampled: false
        }
      }) : s.type === "texture_2d_array" || s.type.startsWith("texture_2d_array<") ? t[s.group].push({
        binding: s.binding,
        visibility: Vn.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d-array",
          multisampled: false
        }
      }) : (s.type === "texture_cube" || s.type.startsWith("texture_cube<")) && t[s.group].push({
        binding: s.binding,
        visibility: Vn.FRAGMENT,
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
  function Dp({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = {}), t[s.group][s.name] = s.binding;
    }
    return t;
  }
  function Hp(n, t) {
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
  const Ls = /* @__PURE__ */ Object.create(null);
  di = class {
    constructor(t) {
      var _a2, _b2;
      this._layoutKey = 0, this._attributeLocationsKey = 0;
      const { fragment: e, vertex: s, layout: i, gpuLayout: r, name: o } = t;
      if (this.name = o, this.fragment = e, this.vertex = s, e.source === s.source) {
        const a = Br(e.source);
        this.structsAndGroups = a;
      } else {
        const a = Br(s.source), l = Br(e.source);
        this.structsAndGroups = Hp(a, l);
      }
      this.layout = i ?? Dp(this.structsAndGroups), this.gpuLayout = r ?? zp(this.structsAndGroups), this.autoAssignGlobalUniforms = ((_a2 = this.layout[0]) == null ? void 0 : _a2.globalUniforms) !== void 0, this.autoAssignLocalUniforms = ((_b2 = this.layout[1]) == null ? void 0 : _b2.localUniforms) !== void 0, this._generateProgramKey();
    }
    _generateProgramKey() {
      const { vertex: t, fragment: e } = this, s = t.source + e.source + t.entryPoint + e.entryPoint;
      this._layoutKey = Ko(s, "program");
    }
    get attributeData() {
      return this._attributeData ?? (this._attributeData = Gp(this.vertex)), this._attributeData;
    }
    destroy() {
      this.gpuLayout = null, this.layout = null, this.structsAndGroups = null, this.fragment = null, this.vertex = null, Ls[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex.source}:${t.fragment.source}:${t.fragment.entryPoint}:${t.vertex.entryPoint}`;
      return Ls[e] || (Ls[e] = new di(t), Ls[e]._cacheKey = e), Ls[e];
    }
  };
  const Ch = [
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
  ], Up = Ch.reduce((n, t) => (n[t] = true, n), {});
  function jp(n, t) {
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
  const Sh = class Th {
    constructor(t, e) {
      this._touched = 0, this.uid = ne("uniform"), this._resourceType = "uniformGroup", this._resourceId = ne("resource"), this.isUniformGroup = true, this._dirtyId = 0, this.destroyed = false, e = {
        ...Th.defaultOptions,
        ...e
      }, this.uniformStructures = t;
      const s = {};
      for (const i in t) {
        const r = t[i];
        if (r.name = i, r.size = r.size ?? 1, !Up[r.type]) {
          const o = r.type.match(/^array<(\w+(?:<\w+>)?),\s*(\d+)>$/);
          if (o) {
            const [, a, l] = o;
            throw new Error(`Uniform type ${r.type} is not supported. Use type: '${a}', size: ${l} instead.`);
          }
          throw new Error(`Uniform type ${r.type} is not supported. Supported uniform types are: ${Ch.join(", ")}`);
        }
        r.value ?? (r.value = jp(r.type, r.size)), s[i] = r.value;
      }
      this.uniforms = s, this._dirtyId = 1, this.ubo = e.ubo, this.isStatic = e.isStatic, this._signature = Ko(Object.keys(s).map((i) => `${i}-${t[i].type}`).join("-"), "uniform-group");
    }
    update() {
      this._dirtyId++;
    }
  };
  Sh.defaultOptions = {
    ubo: false,
    isStatic: false
  };
  Zo = Sh;
  Ui = class {
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
  _o = ((n) => (n[n.WEBGL = 1] = "WEBGL", n[n.WEBGPU = 2] = "WEBGPU", n[n.CANVAS = 4] = "CANVAS", n[n.BOTH = 3] = "BOTH", n))(_o || {});
  hr = class extends on {
    constructor(t) {
      super(), this.uid = ne("shader"), this._uniformBindMap = /* @__PURE__ */ Object.create(null), this._ownedBindGroups = [], this._destroyed = false;
      let { gpuProgram: e, glProgram: s, groups: i, resources: r, compatibleRenderers: o, groupMap: a } = t;
      this.gpuProgram = e, this.glProgram = s, o === void 0 && (o = 0, e && (o |= _o.WEBGPU), s && (o |= _o.WEBGL)), this.compatibleRenderers = o;
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
        for (const h in r) l[h] || (i[99] || (i[99] = new Ui(), this._ownedBindGroups.push(i[99])), l[h] = {
          group: 99,
          binding: c,
          name: h
        }, a[99] = a[99] || {}, a[99][c] = h, c++);
        for (const h in r) {
          const d = h;
          let u = r[h];
          !u.source && !u._resourceType && (u = new Zo(u));
          const p = l[d];
          p && (i[p.group] || (i[p.group] = new Ui(), this._ownedBindGroups.push(i[p.group])), i[p.group].setResource(u, p.binding));
        }
      }
      this.groups = i, this._uniformBindMap = a, this.resources = this._buildResourceAccessor(i, l);
    }
    addResource(t, e, s) {
      var i, r;
      (i = this._uniformBindMap)[e] || (i[e] = {}), (r = this._uniformBindMap[e])[s] || (r[s] = t), this.groups[e] || (this.groups[e] = new Ui(), this._ownedBindGroups.push(this.groups[e]));
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
      return e && (r = di.from(e)), s && (o = Jo.from(s)), new hr({
        gpuProgram: r,
        glProgram: o,
        ...i
      });
    }
  };
  const Vp = {
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
  }, Or = 0, Nr = 1, Fr = 2, Wr = 3, Gr = 4, zr = 5, wo = class Ah {
    constructor() {
      this.data = 0, this.blendMode = "normal", this.polygonOffset = 0, this.blend = true, this.depthMask = true;
    }
    get blend() {
      return !!(this.data & 1 << Or);
    }
    set blend(t) {
      !!(this.data & 1 << Or) !== t && (this.data ^= 1 << Or);
    }
    get offsets() {
      return !!(this.data & 1 << Nr);
    }
    set offsets(t) {
      !!(this.data & 1 << Nr) !== t && (this.data ^= 1 << Nr);
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
      return !!(this.data & 1 << Fr);
    }
    set culling(t) {
      !!(this.data & 1 << Fr) !== t && (this.data ^= 1 << Fr);
    }
    get depthTest() {
      return !!(this.data & 1 << Wr);
    }
    set depthTest(t) {
      !!(this.data & 1 << Wr) !== t && (this.data ^= 1 << Wr);
    }
    get depthMask() {
      return !!(this.data & 1 << zr);
    }
    set depthMask(t) {
      !!(this.data & 1 << zr) !== t && (this.data ^= 1 << zr);
    }
    get clockwiseFrontFace() {
      return !!(this.data & 1 << Gr);
    }
    set clockwiseFrontFace(t) {
      !!(this.data & 1 << Gr) !== t && (this.data ^= 1 << Gr);
    }
    get blendMode() {
      return this._blendMode;
    }
    set blendMode(t) {
      this.blend = t !== "none", this._blendMode = t, this._blendModeId = Vp[t] || 0;
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
      const t = new Ah();
      return t.depthTest = false, t.blend = true, t;
    }
  };
  wo.default2d = wo.for2d();
  Ki = wo;
  const vo = [];
  zt.handleByNamedList(ct.Environment, vo);
  async function Yp(n) {
    if (!n) for (let t = 0; t < vo.length; t++) {
      const e = vo[t];
      if (e.value.test()) {
        await e.value.load();
        return;
      }
    }
  }
  let $s;
  Xp = function() {
    if (typeof $s == "boolean") return $s;
    try {
      $s = new Function("param1", "param2", "param3", "return param1[param2] === param3;")({
        a: "b"
      }, "a", "b") === true;
    } catch {
      $s = false;
    }
    return $s;
  };
  function el(n, t, e = 2) {
    const s = t && t.length, i = s ? t[0] * e : n.length;
    let r = Eh(n, 0, i, e, true);
    const o = [];
    if (!r || r.next === r.prev) return o;
    let a, l, c;
    if (s && (r = Qp(n, t, r, e)), n.length > 80 * e) {
      a = n[0], l = n[1];
      let h = a, d = l;
      for (let u = e; u < i; u += e) {
        const p = n[u], f = n[u + 1];
        p < a && (a = p), f < l && (l = f), p > h && (h = p), f > d && (d = f);
      }
      c = Math.max(h - a, d - l), c = c !== 0 ? 32767 / c : 0;
    }
    return ti(r, o, e, a, l, c, 0), o;
  }
  function Eh(n, t, e, s, i) {
    let r;
    if (i === df(n, t, e, s) > 0) for (let o = t; o < e; o += s) r = nl(o / s | 0, n[o], n[o + 1], r);
    else for (let o = e - s; o >= t; o -= s) r = nl(o / s | 0, n[o], n[o + 1], r);
    return r && Cs(r, r.next) && (ni(r), r = r.next), r;
  }
  function Zn(n, t) {
    if (!n) return n;
    t || (t = n);
    let e = n, s;
    do
      if (s = false, !e.steiner && (Cs(e, e.next) || Qt(e.prev, e, e.next) === 0)) {
        if (ni(e), e = t = e.prev, e === e.next) break;
        s = true;
      } else e = e.next;
    while (s || e !== t);
    return t;
  }
  function ti(n, t, e, s, i, r, o) {
    if (!n) return;
    !o && r && rf(n, s, i, r);
    let a = n;
    for (; n.prev !== n.next; ) {
      const l = n.prev, c = n.next;
      if (r ? Kp(n, s, i, r) : qp(n)) {
        t.push(l.i, n.i, c.i), ni(n), n = c.next, a = c.next;
        continue;
      }
      if (n = c, n === a) {
        o ? o === 1 ? (n = Jp(Zn(n), t), ti(n, t, e, s, i, r, 2)) : o === 2 && Zp(n, t, e, s, i, r) : ti(Zn(n), t, e, s, i, r, 1);
        break;
      }
    }
  }
  function qp(n) {
    const t = n.prev, e = n, s = n.next;
    if (Qt(t, e, s) >= 0) return false;
    const i = t.x, r = e.x, o = s.x, a = t.y, l = e.y, c = s.y, h = Math.min(i, r, o), d = Math.min(a, l, c), u = Math.max(i, r, o), p = Math.max(a, l, c);
    let f = s.next;
    for (; f !== t; ) {
      if (f.x >= h && f.x <= u && f.y >= d && f.y <= p && Hs(i, a, r, l, o, c, f.x, f.y) && Qt(f.prev, f, f.next) >= 0) return false;
      f = f.next;
    }
    return true;
  }
  function Kp(n, t, e, s) {
    const i = n.prev, r = n, o = n.next;
    if (Qt(i, r, o) >= 0) return false;
    const a = i.x, l = r.x, c = o.x, h = i.y, d = r.y, u = o.y, p = Math.min(a, l, c), f = Math.min(h, d, u), m = Math.max(a, l, c), g = Math.max(h, d, u), y = Co(p, f, t, e, s), w = Co(m, g, t, e, s);
    let x = n.prevZ, b = n.nextZ;
    for (; x && x.z >= y && b && b.z <= w; ) {
      if (x.x >= p && x.x <= m && x.y >= f && x.y <= g && x !== i && x !== o && Hs(a, h, l, d, c, u, x.x, x.y) && Qt(x.prev, x, x.next) >= 0 || (x = x.prevZ, b.x >= p && b.x <= m && b.y >= f && b.y <= g && b !== i && b !== o && Hs(a, h, l, d, c, u, b.x, b.y) && Qt(b.prev, b, b.next) >= 0)) return false;
      b = b.nextZ;
    }
    for (; x && x.z >= y; ) {
      if (x.x >= p && x.x <= m && x.y >= f && x.y <= g && x !== i && x !== o && Hs(a, h, l, d, c, u, x.x, x.y) && Qt(x.prev, x, x.next) >= 0) return false;
      x = x.prevZ;
    }
    for (; b && b.z <= w; ) {
      if (b.x >= p && b.x <= m && b.y >= f && b.y <= g && b !== i && b !== o && Hs(a, h, l, d, c, u, b.x, b.y) && Qt(b.prev, b, b.next) >= 0) return false;
      b = b.nextZ;
    }
    return true;
  }
  function Jp(n, t) {
    let e = n;
    do {
      const s = e.prev, i = e.next.next;
      !Cs(s, i) && Mh(s, e, e.next, i) && ei(s, i) && ei(i, s) && (t.push(s.i, e.i, i.i), ni(e), ni(e.next), e = n = i), e = e.next;
    } while (e !== n);
    return Zn(e);
  }
  function Zp(n, t, e, s, i, r) {
    let o = n;
    do {
      let a = o.next.next;
      for (; a !== o.prev; ) {
        if (o.i !== a.i && lf(o, a)) {
          let l = Ph(o, a);
          o = Zn(o, o.next), l = Zn(l, l.next), ti(o, t, e, s, i, r, 0), ti(l, t, e, s, i, r, 0);
          return;
        }
        a = a.next;
      }
      o = o.next;
    } while (o !== n);
  }
  function Qp(n, t, e, s) {
    const i = [];
    for (let r = 0, o = t.length; r < o; r++) {
      const a = t[r] * s, l = r < o - 1 ? t[r + 1] * s : n.length, c = Eh(n, a, l, s, false);
      c === c.next && (c.steiner = true), i.push(af(c));
    }
    i.sort(tf);
    for (let r = 0; r < i.length; r++) e = ef(i[r], e);
    return e;
  }
  function tf(n, t) {
    let e = n.x - t.x;
    if (e === 0 && (e = n.y - t.y, e === 0)) {
      const s = (n.next.y - n.y) / (n.next.x - n.x), i = (t.next.y - t.y) / (t.next.x - t.x);
      e = s - i;
    }
    return e;
  }
  function ef(n, t) {
    const e = nf(n, t);
    if (!e) return t;
    const s = Ph(e, n);
    return Zn(s, s.next), Zn(e, e.next);
  }
  function nf(n, t) {
    let e = t;
    const s = n.x, i = n.y;
    let r = -1 / 0, o;
    if (Cs(n, e)) return e;
    do {
      if (Cs(n, e.next)) return e.next;
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
      if (s >= e.x && e.x >= l && s !== e.x && kh(i < c ? s : r, i, l, c, i < c ? r : s, i, e.x, e.y)) {
        const d = Math.abs(i - e.y) / (s - e.x);
        ei(e, n) && (d < h || d === h && (e.x > o.x || e.x === o.x && sf(o, e))) && (o = e, h = d);
      }
      e = e.next;
    } while (e !== a);
    return o;
  }
  function sf(n, t) {
    return Qt(n.prev, n, t.prev) < 0 && Qt(t.next, n, n.next) < 0;
  }
  function rf(n, t, e, s) {
    let i = n;
    do
      i.z === 0 && (i.z = Co(i.x, i.y, t, e, s)), i.prevZ = i.prev, i.nextZ = i.next, i = i.next;
    while (i !== n);
    i.prevZ.nextZ = null, i.prevZ = null, of(i);
  }
  function of(n) {
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
  function Co(n, t, e, s, i) {
    return n = (n - e) * i | 0, t = (t - s) * i | 0, n = (n | n << 8) & 16711935, n = (n | n << 4) & 252645135, n = (n | n << 2) & 858993459, n = (n | n << 1) & 1431655765, t = (t | t << 8) & 16711935, t = (t | t << 4) & 252645135, t = (t | t << 2) & 858993459, t = (t | t << 1) & 1431655765, n | t << 1;
  }
  function af(n) {
    let t = n, e = n;
    do
      (t.x < e.x || t.x === e.x && t.y < e.y) && (e = t), t = t.next;
    while (t !== n);
    return e;
  }
  function kh(n, t, e, s, i, r, o, a) {
    return (i - o) * (t - a) >= (n - o) * (r - a) && (n - o) * (s - a) >= (e - o) * (t - a) && (e - o) * (r - a) >= (i - o) * (s - a);
  }
  function Hs(n, t, e, s, i, r, o, a) {
    return !(n === o && t === a) && kh(n, t, e, s, i, r, o, a);
  }
  function lf(n, t) {
    return n.next.i !== t.i && n.prev.i !== t.i && !cf(n, t) && (ei(n, t) && ei(t, n) && hf(n, t) && (Qt(n.prev, n, t.prev) || Qt(n, t.prev, t)) || Cs(n, t) && Qt(n.prev, n, n.next) > 0 && Qt(t.prev, t, t.next) > 0);
  }
  function Qt(n, t, e) {
    return (t.y - n.y) * (e.x - t.x) - (t.x - n.x) * (e.y - t.y);
  }
  function Cs(n, t) {
    return n.x === t.x && n.y === t.y;
  }
  function Mh(n, t, e, s) {
    const i = Si(Qt(n, t, e)), r = Si(Qt(n, t, s)), o = Si(Qt(e, s, n)), a = Si(Qt(e, s, t));
    return !!(i !== r && o !== a || i === 0 && Ci(n, e, t) || r === 0 && Ci(n, s, t) || o === 0 && Ci(e, n, s) || a === 0 && Ci(e, t, s));
  }
  function Ci(n, t, e) {
    return t.x <= Math.max(n.x, e.x) && t.x >= Math.min(n.x, e.x) && t.y <= Math.max(n.y, e.y) && t.y >= Math.min(n.y, e.y);
  }
  function Si(n) {
    return n > 0 ? 1 : n < 0 ? -1 : 0;
  }
  function cf(n, t) {
    let e = n;
    do {
      if (e.i !== n.i && e.next.i !== n.i && e.i !== t.i && e.next.i !== t.i && Mh(e, e.next, n, t)) return true;
      e = e.next;
    } while (e !== n);
    return false;
  }
  function ei(n, t) {
    return Qt(n.prev, n, n.next) < 0 ? Qt(n, t, n.next) >= 0 && Qt(n, n.prev, t) >= 0 : Qt(n, t, n.prev) < 0 || Qt(n, n.next, t) < 0;
  }
  function hf(n, t) {
    let e = n, s = false;
    const i = (n.x + t.x) / 2, r = (n.y + t.y) / 2;
    do
      e.y > r != e.next.y > r && e.next.y !== e.y && i < (e.next.x - e.x) * (r - e.y) / (e.next.y - e.y) + e.x && (s = !s), e = e.next;
    while (e !== n);
    return s;
  }
  function Ph(n, t) {
    const e = So(n.i, n.x, n.y), s = So(t.i, t.x, t.y), i = n.next, r = t.prev;
    return n.next = t, t.prev = n, e.next = i, i.prev = e, s.next = e, e.prev = s, r.next = s, s.prev = r, s;
  }
  function nl(n, t, e, s) {
    const i = So(n, t, e);
    return s ? (i.next = s.next, i.prev = s, s.next.prev = i, s.next = i) : (i.prev = i, i.next = i), i;
  }
  function ni(n) {
    n.next.prev = n.prev, n.prev.next = n.next, n.prevZ && (n.prevZ.nextZ = n.nextZ), n.nextZ && (n.nextZ.prevZ = n.prevZ);
  }
  function So(n, t, e) {
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
  function df(n, t, e, s) {
    let i = 0;
    for (let r = t, o = e - s; r < e; r += s) i += (n[o] - n[r]) * (n[r + 1] + n[o + 1]), o = r;
    return i;
  }
  const uf = el.default || el;
  Ih = ((n) => (n[n.NONE = 0] = "NONE", n[n.COLOR = 16384] = "COLOR", n[n.STENCIL = 1024] = "STENCIL", n[n.DEPTH = 256] = "DEPTH", n[n.COLOR_DEPTH = 16640] = "COLOR_DEPTH", n[n.COLOR_STENCIL = 17408] = "COLOR_STENCIL", n[n.DEPTH_STENCIL = 1280] = "DEPTH_STENCIL", n[n.ALL = 17664] = "ALL", n))(Ih || {});
  pf = class {
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
  const ff = [
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
  ], Rh = class Lh extends on {
    constructor(t) {
      super(), this.tick = 0, this.uid = ne("renderer"), this.runners = /* @__PURE__ */ Object.create(null), this.renderPipes = /* @__PURE__ */ Object.create(null), this._initOptions = {}, this._systemsHash = /* @__PURE__ */ Object.create(null), this.type = t.type, this.name = t.name, this.config = t;
      const e = [
        ...ff,
        ...this.config.runners ?? []
      ];
      this._addRunners(...e), this._unsafeEvalCheck();
    }
    async init(t = {}) {
      const e = t.skipExtensionImports === true ? true : t.manageImports === false;
      await Yp(e), this._addSystems(this.config.systems), this._addPipes(this.config.renderPipes, this.config.renderPipeAdaptors);
      for (const s in this._systemsHash) t = {
        ...this._systemsHash[s].constructor.defaultOptions,
        ...t
      };
      t = {
        ...Lh.defaultOptions,
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
      }, e && (It(ee, "passing a second argument is deprecated, please use render options instead"), s.target = e.renderTexture)), s.target || (s.target = this.view.renderTarget), s.target === this.view.renderTarget && (this._lastObjectRendered = s.container, s.clearColor ?? (s.clearColor = this.background.colorRgba), s.clear ?? (s.clear = this.background.clearBeforeRender)), s.clearColor) {
        const i = Array.isArray(s.clearColor) && s.clearColor.length === 4;
        s.clearColor = i ? s.clearColor : Yt.shared.setValue(s.clearColor).toArray();
      }
      s.transform || (s.container.updateLocalTransform(), s.transform = s.container.localTransform), s.container.visible && (s.container.enableRenderGroup(), this.runners.prerender.emit(s), this.runners.renderStart.emit(s), this.runners.render.emit(s), this.runners.renderEnd.emit(s), this.runners.postrender.emit(s));
    }
    resize(t, e, s) {
      const i = this.view.resolution;
      this.view.resize(t, e, s), this.emit("resize", this.view.screen.width, this.view.screen.height, this.view.resolution), s !== void 0 && s !== i && this.runners.resolutionChange.emit(s);
    }
    clear(t = {}) {
      const e = this;
      t.target || (t.target = e.renderTarget.renderTarget), t.clearColor || (t.clearColor = this.background.colorRgba), t.clear ?? (t.clear = Ih.ALL);
      const { clear: s, clearColor: i, target: r, mipLevel: o, layer: a } = t;
      Yt.shared.setValue(i ?? this.background.colorRgba), e.renderTarget.clear(r, s, Yt.shared.toArray(), o ?? 0, a ?? 0);
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
        this.runners[e] = new pf(e);
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
      this.runners.destroy.items.reverse(), this.runners.destroy.emit(t), (t === true || typeof t == "object" && t.releaseGlobalResources) && hi.release(), Object.values(this.runners).forEach((e) => {
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
      if (!Xp()) throw new Error("Current environment does not allow unsafe-eval, please use pixi.js/unsafe-eval module to enable support.");
    }
    resetState() {
      this.runners.resetState.emit();
    }
  };
  Rh.defaultOptions = {
    resolution: 1,
    failIfMajorPerformanceCaveat: false,
    roundPixels: false
  };
  let Ti;
  $h = Rh;
  function mf(n) {
    return Ti !== void 0 || (Ti = (() => {
      var _a2;
      const t = {
        stencil: true,
        failIfMajorPerformanceCaveat: n ?? $h.defaultOptions.failIfMajorPerformanceCaveat
      };
      try {
        if (!Rt.get().getWebGLRenderingContext()) return false;
        let s = Rt.get().createCanvas().getContext("webgl", t);
        const i = !!((_a2 = s == null ? void 0 : s.getContextAttributes()) == null ? void 0 : _a2.stencil);
        if (s) {
          const r = s.getExtension("WEBGL_lose_context");
          r && r.loseContext();
        }
        return s = null, i;
      } catch {
        return false;
      }
    })()), Ti;
  }
  let Ai;
  async function gf(n = {}) {
    return Ai !== void 0 || (Ai = await (async () => {
      const t = Rt.get().getNavigator().gpu;
      if (!t) return false;
      try {
        return await (await t.requestAdapter(n)).requestDevice(), true;
      } catch {
        return false;
      }
    })()), Ai;
  }
  const sl = [
    "webgl",
    "webgpu",
    "canvas"
  ];
  async function yf(n) {
    let t = [];
    n.preference ? (t.push(n.preference), sl.forEach((r) => {
      r !== n.preference && t.push(r);
    })) : t = sl.slice();
    let e, s = {};
    for (let r = 0; r < t.length; r++) {
      const o = t[r];
      if (o === "webgpu" && await gf()) {
        const { WebGPURenderer: a } = await Mn(async () => {
          const { WebGPURenderer: l } = await import("./WebGPURenderer-DIXZPbil.js");
          return {
            WebGPURenderer: l
          };
        }, __vite__mapDeps([3,4,5,2]));
        e = a, s = {
          ...n,
          ...n.webgpu
        };
        break;
      } else if (o === "webgl" && mf(n.failIfMajorPerformanceCaveat ?? $h.defaultOptions.failIfMajorPerformanceCaveat)) {
        const { WebGLRenderer: a } = await Mn(async () => {
          const { WebGLRenderer: l } = await import("./WebGLRenderer-BNABB76k.js");
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
        const { CanvasRenderer: a } = await Mn(async () => {
          const { CanvasRenderer: l } = await import("./CanvasRenderer-DYKzJi1K.js");
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
  Bh = "8.17.1";
  class Oh {
    static init() {
      var _a2;
      (_a2 = globalThis.__PIXI_APP_INIT__) == null ? void 0 : _a2.call(globalThis, this, Bh);
    }
    static destroy() {
    }
  }
  Oh.extension = ct.Application;
  xf = class {
    constructor(t) {
      this._renderer = t;
    }
    init() {
      var _a2;
      (_a2 = globalThis.__PIXI_RENDERER_INIT__) == null ? void 0 : _a2.call(globalThis, this._renderer, Bh);
    }
    destroy() {
      this._renderer = null;
    }
  };
  xf.extension = {
    type: [
      ct.WebGLSystem,
      ct.WebGPUSystem
    ],
    name: "initHook",
    priority: -10
  };
  class Nh {
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
  Nh.extension = ct.Application;
  class Fh {
    static init(t) {
      t = Object.assign({
        autoStart: true,
        sharedTicker: false
      }, t), Object.defineProperty(this, "ticker", {
        configurable: true,
        set(e) {
          this._ticker && this._ticker.remove(this.render, this), this._ticker = e, e && e.add(this.render, this, Qs.LOW);
        },
        get() {
          return this._ticker;
        }
      }), this.stop = () => {
        this._ticker.stop();
      }, this.start = () => {
        this._ticker.start();
      }, this._ticker = null, this.ticker = t.sharedTicker ? Xn.shared : new Xn(), t.autoStart && this.start();
    }
    static destroy() {
      if (this._ticker) {
        const t = this._ticker;
        this.ticker = null, t.destroy();
      }
    }
  }
  Fh.extension = ct.Application;
  zt.add(Nh);
  zt.add(Fh);
  const Wh = class To {
    constructor(...t) {
      this.stage = new Ut(), t[0] !== void 0 && It(ee, "Application constructor options are deprecated, please use Application.init() instead.");
    }
    async init(t) {
      t = {
        ...t
      }, this.stage || (this.stage = new Ut()), this.renderer = await yf(t), To._plugins.forEach((e) => {
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
      return It(ee, "Application.view is deprecated, please use Application.canvas instead."), this.renderer.canvas;
    }
    get screen() {
      return this.renderer.screen;
    }
    destroy(t = false, e = false) {
      const s = To._plugins.slice(0);
      s.reverse(), s.forEach((i) => {
        i.destroy.call(this);
      }), this.stage.destroy(e), this.stage = null, this.renderer.destroy(t), this.renderer = null;
    }
  };
  Wh._plugins = [];
  Qo = Wh;
  zt.handleByList(ct.Application, Qo._plugins);
  zt.add(Oh);
  const Dr = {
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
          const g = p[m].split("="), y = g[0], w = g[1].replace(/"/gm, ""), x = parseFloat(w), b = isNaN(x) ? w : x;
          f[y] = b;
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
  }, il = {
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
  }, rl = {
    test(n) {
      return typeof n == "string" && n.match(/<font(\s|>)/) ? il.test(Rt.get().parseXML(n)) : false;
    },
    parse(n) {
      return il.parse(Rt.get().parseXML(n));
    }
  }, bf = [
    ".xml",
    ".fnt"
  ], _f = {
    extension: {
      type: ct.CacheParser,
      name: "cacheBitmapFont"
    },
    test: (n) => !!(n == null ? void 0 : n.pages) && !!(n == null ? void 0 : n.chars) && typeof (n == null ? void 0 : n.fontFamily) == "string" && n.fontFamily !== "",
    getCacheableAssets(n, t) {
      const e = {};
      return n.forEach((s) => {
        e[s] = t, e[`${s}-bitmap`] = t;
      }), e[`${t.fontFamily}-bitmap`] = t, e;
    }
  }, wf = {
    extension: {
      type: ct.LoadParser,
      priority: Rn.Normal
    },
    name: "loadBitmapFont",
    id: "bitmap-font",
    test(n) {
      return bf.includes(Ae.extname(n).toLowerCase());
    },
    async testParse(n) {
      return Dr.test(n) || rl.test(n);
    },
    async parse(n, t, e) {
      const s = Dr.test(n) ? Dr.parse(n) : rl.parse(n), { src: i } = t, { pages: r } = s, o = [], a = s.distanceField ? {
        scaleMode: "linear",
        alphaMode: "premultiply-alpha-on-upload",
        autoGenerateMipmaps: false,
        resolution: 1
      } : {};
      for (let u = 0; u < r.length; ++u) {
        const p = r[u].file;
        let f = Ae.join(Ae.dirname(i), p);
        f = xo(f, i), o.push({
          src: f,
          data: a
        });
      }
      const [l, { BitmapFont: c }] = await Promise.all([
        e.load(o),
        Mn(() => import("./BitmapFont-lbImtPtE.js"), [])
      ]), h = o.map((u) => l[u.src]);
      return new c({
        data: s,
        textures: h
      }, i);
    },
    async load(n, t) {
      return await (await Rt.get().fetch(n)).text();
    },
    async unload(n, t, e) {
      await Promise.all(n.pages.map((s) => e.unload(s.texture.source._sourceOrigin))), n.destroy();
    }
  };
  class vf {
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
  const Cf = {
    extension: {
      type: ct.CacheParser,
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
  async function Gh(n) {
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
  const Sf = {
    extension: {
      type: ct.DetectionParser,
      priority: 1
    },
    test: async () => Gh("data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A="),
    add: async (n) => [
      ...n,
      "avif"
    ],
    remove: async (n) => n.filter((t) => t !== "avif")
  }, ol = [
    "png",
    "jpg",
    "jpeg"
  ], Tf = {
    extension: {
      type: ct.DetectionParser,
      priority: -1
    },
    test: () => Promise.resolve(true),
    add: async (n) => [
      ...n,
      ...ol
    ],
    remove: async (n) => n.filter((t) => !ol.includes(t))
  }, Af = "WorkerGlobalScope" in globalThis && globalThis instanceof globalThis.WorkerGlobalScope;
  function dr(n) {
    return Af ? false : document.createElement("video").canPlayType(n) !== "";
  }
  const Ef = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => dr("video/mp4"),
    add: async (n) => [
      ...n,
      "mp4",
      "m4v"
    ],
    remove: async (n) => n.filter((t) => t !== "mp4" && t !== "m4v")
  }, kf = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => dr("video/ogg"),
    add: async (n) => [
      ...n,
      "ogv"
    ],
    remove: async (n) => n.filter((t) => t !== "ogv")
  }, Mf = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => dr("video/webm"),
    add: async (n) => [
      ...n,
      "webm"
    ],
    remove: async (n) => n.filter((t) => t !== "webm")
  }, Pf = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => Gh("data:image/webp;base64,UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAAAfQ//73v/+BiOh/AAA="),
    add: async (n) => [
      ...n,
      "webp"
    ],
    remove: async (n) => n.filter((t) => t !== "webp")
  }, zh = class ji {
    constructor() {
      this.loadOptions = {
        ...ji.defaultOptions
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
        ...ji.defaultOptions,
        ...this.loadOptions,
        onProgress: e
      } : {
        ...ji.defaultOptions,
        ...this.loadOptions,
        ...e || {}
      }, { onProgress: i, onError: r, strategy: o, retryCount: a, retryDelay: l } = s;
      let c = 0;
      const h = {}, d = Xi(t), u = ze(t, (m) => ({
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
      const s = ze(t, (i) => ({
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
  zh.defaultOptions = {
    onProgress: void 0,
    onError: void 0,
    strategy: "throw",
    retryCount: 3,
    retryDelay: 250
  };
  let If = zh;
  function Es(n, t) {
    if (Array.isArray(t)) {
      for (const e of t) if (n.startsWith(`data:${e}`)) return true;
      return false;
    }
    return n.startsWith(`data:${t}`);
  }
  function ks(n, t) {
    const e = n.split("?")[0], s = Ae.extname(e).toLowerCase();
    return Array.isArray(t) ? t.includes(s) : s === t;
  }
  const Rf = ".json", Lf = "application/json", $f = {
    extension: {
      type: ct.LoadParser,
      priority: Rn.Low
    },
    name: "loadJson",
    id: "json",
    test(n) {
      return Es(n, Lf) || ks(n, Rf);
    },
    async load(n) {
      return await (await Rt.get().fetch(n)).json();
    }
  }, Bf = ".txt", Of = "text/plain", Nf = {
    name: "loadTxt",
    id: "text",
    extension: {
      type: ct.LoadParser,
      priority: Rn.Low,
      name: "loadTxt"
    },
    test(n) {
      return Es(n, Of) || ks(n, Bf);
    },
    async load(n) {
      return await (await Rt.get().fetch(n)).text();
    }
  }, Ff = [
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
  ], Wf = [
    ".ttf",
    ".otf",
    ".woff",
    ".woff2"
  ], Gf = [
    "font/ttf",
    "font/otf",
    "font/woff",
    "font/woff2"
  ], zf = /^(--|-?[A-Z_])[0-9A-Z_-]*$/i;
  function Df(n) {
    const t = Ae.extname(n), i = Ae.basename(n, t).replace(/(-|_)/g, " ").toLowerCase().split(" ").map((a) => a.charAt(0).toUpperCase() + a.slice(1));
    let r = i.length > 0;
    for (const a of i) if (!a.match(zf)) {
      r = false;
      break;
    }
    let o = i.join(" ");
    return r || (o = `"${o.replace(/[\\"]/g, "\\$&")}"`), o;
  }
  const Hf = /^[0-9A-Za-z%:/?#\[\]@!\$&'()\*\+,;=\-._~]*$/;
  function Uf(n) {
    return Hf.test(n) ? n : encodeURI(n);
  }
  const jf = {
    extension: {
      type: ct.LoadParser,
      priority: Rn.Low
    },
    name: "loadWebFont",
    id: "web-font",
    test(n) {
      return Es(n, Gf) || ks(n, Wf);
    },
    async load(n, t) {
      var _a2, _b2, _c2;
      const e = Rt.get().getFontFaceSet();
      if (e) {
        const s = [], i = ((_a2 = t.data) == null ? void 0 : _a2.family) ?? Df(n), r = ((_c2 = (_b2 = t.data) == null ? void 0 : _b2.weights) == null ? void 0 : _c2.filter((a) => Ff.includes(a))) ?? [
          "normal"
        ], o = t.data ?? {};
        for (let a = 0; a < r.length; a++) {
          const l = r[a], c = new FontFace(i, `url('${Uf(n)}')`, {
            ...o,
            weight: l
          });
          await c.load(), e.add(c), s.push(c);
        }
        return se.has(`${i}-and-url`) ? se.get(`${i}-and-url`).entries.push({
          url: n,
          faces: s
        }) : se.set(`${i}-and-url`, {
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
      ], e = t[0].family, s = se.get(`${e}-and-url`), i = s.entries.find((r) => r.faces.some((o) => t.indexOf(o) !== -1));
      i.faces = i.faces.filter((r) => t.indexOf(r) === -1), i.faces.length === 0 && (s.entries = s.entries.filter((r) => r !== i)), t.forEach((r) => {
        Rt.get().getFontFaceSet().delete(r);
      }), s.entries.length === 0 && se.remove(`${e}-and-url`);
    }
  };
  var Hr, al;
  function Vf() {
    if (al) return Hr;
    al = 1, Hr = e;
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
    return Hr;
  }
  var Yf = Vf();
  const Xf = Hc(Yf);
  function qf(n, t) {
    const e = Xf(n), s = [];
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
  class ta {
    constructor(t = 0, e = 0, s = 0) {
      this.type = "circle", this.x = t, this.y = e, this.radius = s;
    }
    clone() {
      return new ta(this.x, this.y, this.radius);
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
  class ea {
    constructor(t = 0, e = 0, s = 0, i = 0) {
      this.type = "ellipse", this.x = t, this.y = e, this.halfWidth = s, this.halfHeight = i;
    }
    clone() {
      return new ea(this.x, this.y, this.halfWidth, this.halfHeight);
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
  function Kf(n, t, e, s, i, r) {
    const o = n - e, a = t - s, l = i - e, c = r - s, h = o * l + a * c, d = l * l + c * c;
    let u = -1;
    d !== 0 && (u = h / d);
    let p, f;
    u < 0 ? (p = e, f = s) : u > 1 ? (p = i, f = r) : (p = e + u * l, f = s + u * c);
    const m = n - p, g = t - f;
    return m * m + g * g;
  }
  let Jf, Zf;
  class Xs {
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
      const e = this.getBounds(Jf), s = t.getBounds(Zf);
      if (!e.containsRect(s)) return false;
      const i = t.points;
      for (let r = 0; r < i.length; r += 2) {
        const o = i[r], a = i[r + 1];
        if (!this.contains(o, a)) return false;
      }
      return true;
    }
    clone() {
      const t = this.points.slice(), e = new Xs(t);
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
        const d = l[h], u = l[h + 1], p = l[(h + 2) % l.length], f = l[(h + 3) % l.length], m = Kf(t, e, d, u, p, f), g = Math.sign((p - d) * (e - u) - (f - u) * (t - d));
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
      return It("8.11.0", "Polygon.lastX is deprecated, please use Polygon.lastX instead."), this.points[this.points.length - 2];
    }
    get y() {
      return It("8.11.0", "Polygon.y is deprecated, please use Polygon.lastY instead."), this.points[this.points.length - 1];
    }
    get startX() {
      return this.points[0];
    }
    get startY() {
      return this.points[1];
    }
  }
  const Ei = (n, t, e, s, i, r, o) => {
    const a = n - e, l = t - s, c = Math.sqrt(a * a + l * l);
    return c >= i - r && c <= i + o;
  };
  class na {
    constructor(t = 0, e = 0, s = 0, i = 0, r = 20) {
      this.type = "roundedRectangle", this.x = t, this.y = e, this.width = s, this.height = i, this.radius = r;
    }
    getBounds(t) {
      return t || (t = new Ht()), t.x = this.x, t.y = this.y, t.width = this.width, t.height = this.height, t;
    }
    clone() {
      return new na(this.x, this.y, this.width, this.height, this.radius);
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
      return (t >= r - h && t <= r + d || t >= g - d && t <= g + h) && e >= p && e <= p + m || (e >= o - h && e <= o + d || e >= y - d && e <= y + h) && t >= u && t <= u + f ? true : t < u && e < p && Ei(t, e, u, p, c, d, h) || t > g - c && e < p && Ei(t, e, g - c, p, c, d, h) || t > g - c && e > y - c && Ei(t, e, g - c, y - c, c, d, h) || t < u && e > y - c && Ei(t, e, u, y - c, c, d, h);
    }
    toString() {
      return `[pixi.js/math:RoundedRectangle x=${this.x} y=${this.y}width=${this.width} height=${this.height} radius=${this.radius}]`;
    }
  }
  const Dh = {};
  Qf = function(n, t, e) {
    let s = 2166136261;
    for (let i = 0; i < t; i++) s ^= n[i].uid, s = Math.imul(s, 16777619), s >>>= 0;
    return Dh[s] || tm(n, t, s, e);
  };
  function tm(n, t, e, s) {
    const i = {};
    let r = 0;
    for (let a = 0; a < s; a++) {
      const l = a < t ? n[a] : Ct.EMPTY.source;
      i[r++] = l.source, i[r++] = l.style;
    }
    const o = new Ui(i);
    return Dh[e] = o, o;
  }
  class ps {
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
  ll = function(n, t, e, s) {
    if (e ?? (e = 0), s ?? (s = Math.min(n.byteLength - e, t.byteLength)), !(e & 7) && !(s & 7)) {
      const i = s / 8;
      new Float64Array(t, 0, i).set(new Float64Array(n, e, i));
    } else if (!(e & 3) && !(s & 3)) {
      const i = s / 4;
      new Float32Array(t, 0, i).set(new Float32Array(n, e, i));
    } else new Uint8Array(t).set(new Uint8Array(n, e, s));
  };
  const em = {
    normal: "normal-npm",
    add: "add-npm",
    screen: "screen-npm"
  };
  nm = ((n) => (n[n.DISABLED = 0] = "DISABLED", n[n.RENDERING_MASK_ADD = 1] = "RENDERING_MASK_ADD", n[n.MASK_ACTIVE = 2] = "MASK_ACTIVE", n[n.INVERSE_MASK_ACTIVE = 3] = "INVERSE_MASK_ACTIVE", n[n.RENDERING_MASK_REMOVE = 4] = "RENDERING_MASK_REMOVE", n[n.NONE = 5] = "NONE", n))(nm || {});
  function Ao(n, t) {
    return t.alphaMode === "no-premultiply-alpha" && em[n] || n;
  }
  const sm = [
    "precision mediump float;",
    "void main(void){",
    "float test = 0.1;",
    "%forloop%",
    "gl_FragColor = vec4(0.0);",
    "}"
  ].join(`
`);
  function im(n) {
    let t = "";
    for (let e = 0; e < n; ++e) e > 0 && (t += `
else `), e < n - 1 && (t += `if(test == ${e}.0){}`);
    return t;
  }
  rm = function(n, t) {
    if (n === 0) throw new Error("Invalid value of `0` passed to `checkMaxIfStatementsInShader`");
    const e = t.createShader(t.FRAGMENT_SHADER);
    try {
      for (; ; ) {
        const s = sm.replace(/%forloop%/gi, im(n));
        if (t.shaderSource(e, s), t.compileShader(e), !t.getShaderParameter(e, t.COMPILE_STATUS)) n = n / 2 | 0;
        else break;
      }
    } finally {
      t.deleteShader(e);
    }
    return n;
  };
  let is = null;
  function om() {
    var _a2;
    if (is) return is;
    const n = wh();
    return is = n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS), is = rm(is, n), (_a2 = n.getExtension("WEBGL_lose_context")) == null ? void 0 : _a2.loseContext(), is;
  }
  class am {
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
  class lm {
    constructor() {
      this.renderPipeId = "batch", this.action = "startBatch", this.start = 0, this.size = 0, this.textures = new am(), this.blendMode = "normal", this.topology = "triangle-strip", this.canBundle = true;
    }
    destroy() {
      this.textures = null, this.gpuBindGroup = null, this.bindGroup = null, this.batcher = null, this.elements = null;
    }
  }
  const qs = [];
  let Ji = 0;
  hi.register({
    clear: () => {
      if (qs.length > 0) for (const n of qs) n && n.destroy();
      qs.length = 0, Ji = 0;
    }
  });
  function cl() {
    return Ji > 0 ? qs[--Ji] : new lm();
  }
  function hl(n) {
    n.elements = null, qs[Ji++] = n;
  }
  let Bs = 0;
  const Hh = class Uh {
    constructor(t) {
      this.uid = ne("batcher"), this.dirty = true, this.batchIndex = 0, this.batches = [], this._elements = [], t = {
        ...Uh.defaultOptions,
        ...t
      }, t.maxTextures || (It("v8.8.0", "maxTextures is a required option for Batcher now, please pass it in the options"), t.maxTextures = om());
      const { maxTextures: e, attributesInitialSize: s, indicesInitialSize: i } = t;
      this.attributeBuffer = new ps(s * 4), this.indexBuffer = new Uint16Array(i), this.maxTextures = e;
    }
    begin() {
      this.elementSize = 0, this.elementStart = 0, this.indexSize = 0, this.attributeSize = 0;
      for (let t = 0; t < this.batchIndex; t++) hl(this.batches[t]);
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
      let s = cl(), i = s.textures;
      i.clear();
      const r = e[this.elementStart];
      let o = Ao(r.blendMode, r.texture._source), a = r.topology;
      this.attributeSize * 4 > this.attributeBuffer.size && this._resizeAttributeBuffer(this.attributeSize * 4), this.indexSize > this.indexBuffer.length && this._resizeIndexBuffer(this.indexSize);
      const l = this.attributeBuffer.float32View, c = this.attributeBuffer.uint32View, h = this.indexBuffer;
      let d = this._batchIndexSize, u = this._batchIndexStart, p = "startBatch", f = [];
      const m = this.maxTextures;
      for (let g = this.elementStart; g < this.elementSize; ++g) {
        const y = e[g];
        e[g] = null;
        const x = y.texture._source, b = Ao(y.blendMode, x), _ = o !== b || a !== y.topology;
        if (x._batchTick === Bs && !_) {
          y._textureId = x._textureBindLocation, d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize)), y._batch = s, f.push(y);
          continue;
        }
        x._batchTick = Bs, (i.count >= m || _) && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), p = "renderBatch", u = d, o = b, a = y.topology, s = cl(), i = s.textures, i.clear(), f = [], ++Bs), y._textureId = x._textureBindLocation = i.count, i.ids[x.uid] = i.count, i.textures[i.count++] = x, y._batch = s, f.push(y), d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize));
      }
      i.count > 0 && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), u = d, ++Bs), this.elementStart = this.elementSize, this._batchIndexStart = u, this._batchIndexSize = d;
    }
    _finishBatch(t, e, s, i, r, o, a, l, c) {
      t.gpuBindGroup = null, t.bindGroup = null, t.action = l, t.batcher = this, t.textures = i, t.blendMode = r, t.topology = o, t.start = e, t.size = s, t.elements = c, ++Bs, this.batches[this.batchIndex++] = t, a.add(t);
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
      const e = Math.max(t, this.attributeBuffer.size * 2), s = new ps(e);
      ll(this.attributeBuffer.rawBinaryData, s.rawBinaryData), this.attributeBuffer = s;
    }
    _resizeIndexBuffer(t) {
      const e = this.indexBuffer;
      let s = Math.max(t, e.length * 1.5);
      s += s % 2;
      const i = s > 65535 ? new Uint32Array(s) : new Uint16Array(s);
      if (i.BYTES_PER_ELEMENT !== e.BYTES_PER_ELEMENT) for (let r = 0; r < e.length; r++) i[r] = e[r];
      else ll(e.buffer, i.buffer);
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
        for (let e = 0; e < this.batchIndex; e++) hl(this.batches[e]);
        this.batches = null, this.geometry.destroy(true), this.geometry = null, t.shader && ((_a2 = this.shader) == null ? void 0 : _a2.destroy(), this.shader = null);
        for (let e = 0; e < this._elements.length; e++) this._elements[e] && (this._elements[e]._batch = null);
        this._elements = null, this.indexBuffer = null, this.attributeBuffer.destroy(), this.attributeBuffer = null;
      }
    }
  };
  Hh.defaultOptions = {
    maxTextures: null,
    attributesInitialSize: 4,
    indicesInitialSize: 6
  };
  let cm = Hh;
  ce = ((n) => (n[n.MAP_READ = 1] = "MAP_READ", n[n.MAP_WRITE = 2] = "MAP_WRITE", n[n.COPY_SRC = 4] = "COPY_SRC", n[n.COPY_DST = 8] = "COPY_DST", n[n.INDEX = 16] = "INDEX", n[n.VERTEX = 32] = "VERTEX", n[n.UNIFORM = 64] = "UNIFORM", n[n.STORAGE = 128] = "STORAGE", n[n.INDIRECT = 256] = "INDIRECT", n[n.QUERY_RESOLVE = 512] = "QUERY_RESOLVE", n[n.STATIC = 1024] = "STATIC", n))(ce || {});
  Qn = class extends on {
    constructor(t) {
      let { data: e, size: s } = t;
      const { usage: i, label: r, shrinkToFit: o } = t;
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.autoGarbageCollect = true, this.uid = ne("buffer"), this._resourceType = "buffer", this._resourceId = ne("resource"), this._touched = 0, this._updateID = 1, this._dataInt32 = null, this.shrinkToFit = true, this.destroyed = false, e instanceof Array && (e = new Float32Array(e)), this._data = e, s ?? (s = e == null ? void 0 : e.byteLength);
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
      return !!(this.descriptor.usage & ce.STATIC);
    }
    set static(t) {
      t ? this.descriptor.usage |= ce.STATIC : this.descriptor.usage &= ~ce.STATIC;
    }
    setDataWithSize(t, e, s) {
      if (this._updateID++, this._updateSize = e * t.BYTES_PER_ELEMENT, this._data === t) {
        s && this.emit("update", this);
        return;
      }
      const i = this._data;
      if (this._data = t, this._dataInt32 = null, !i || i.length !== t.length) {
        !this.shrinkToFit && i && t.byteLength < i.byteLength ? s && this.emit("update", this) : (this.descriptor.size = t.byteLength, this._resourceId = ne("resource"), this.emit("change", this));
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
  function jh(n, t) {
    if (!(n instanceof Qn)) {
      let e = t ? ce.INDEX : ce.VERTEX;
      n instanceof Array && (t ? (n = new Uint32Array(n), e = ce.INDEX | ce.COPY_DST) : (n = new Float32Array(n), e = ce.VERTEX | ce.COPY_DST)), n = new Qn({
        data: n,
        label: t ? "index-mesh-buffer" : "vertex-mesh-buffer",
        usage: e
      });
    }
    return n;
  }
  function hm(n, t, e) {
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
  function dm(n) {
    return (n instanceof Qn || Array.isArray(n) || n.BYTES_PER_ELEMENT) && (n = {
      buffer: n
    }), n.buffer = jh(n.buffer, false), n;
  }
  Vh = class extends on {
    constructor(t = {}) {
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ne("geometry"), this._layoutKey = 0, this.instanceCount = 1, this._bounds = new Ee(), this._boundsDirty = true;
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
      const s = dm(e);
      this.buffers.indexOf(s.buffer) === -1 && (this.buffers.push(s.buffer), s.buffer.on("update", this.onBufferUpdate, this), s.buffer.on("change", this.onBufferUpdate, this)), this.attributes[t] = s;
    }
    addIndex(t) {
      this.indexBuffer = jh(t, true), this.buffers.push(this.indexBuffer);
    }
    get bounds() {
      return this._boundsDirty ? (this._boundsDirty = false, hm(this, "aPosition", this._bounds)) : this._bounds;
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
  const um = new Float32Array(1), pm = new Uint32Array(1);
  class fm extends Vh {
    constructor() {
      const e = new Qn({
        data: um,
        label: "attribute-batch-buffer",
        usage: ce.VERTEX | ce.COPY_DST,
        shrinkToFit: false
      }), s = new Qn({
        data: pm,
        label: "index-batch-buffer",
        usage: ce.INDEX | ce.COPY_DST,
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
  function dl(n, t, e) {
    if (n) for (const s in n) {
      const i = s.toLocaleLowerCase(), r = t[i];
      if (r) {
        let o = n[s];
        s === "header" && (o = o.replace(/@in\s+[^;]+;\s*/g, "").replace(/@out\s+[^;]+;\s*/g, "")), e && r.push(`//----${e}----//`), r.push(o);
      } else Jt(`${s} placement hook does not exist in shader`);
    }
  }
  const mm = /\{\{(.*?)\}\}/g;
  function ul(n) {
    var _a2;
    const t = {};
    return (((_a2 = n.match(mm)) == null ? void 0 : _a2.map((s) => s.replace(/[{()}]/g, ""))) ?? []).forEach((s) => {
      t[s] = [];
    }), t;
  }
  function pl(n, t) {
    let e;
    const s = /@in\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function fl(n, t, e = false) {
    const s = [];
    pl(t, s), n.forEach((a) => {
      a.header && pl(a.header, s);
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
  function ml(n, t) {
    let e;
    const s = /@out\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function gm(n) {
    const e = /\b(\w+)\s*:/g.exec(n);
    return e ? e[1] : "";
  }
  function ym(n) {
    const t = /@.*?\s+/g;
    return n.replace(t, "");
  }
  function xm(n, t) {
    const e = [];
    ml(t, e), n.forEach((l) => {
      l.header && ml(l.header, e);
    });
    let s = 0;
    const i = e.sort().map((l) => l.indexOf("builtin") > -1 ? l : `@location(${s++}) ${l}`).join(`,
`), r = e.sort().map((l) => `       var ${ym(l)};`).join(`
`), o = `return VSOutput(
            ${e.sort().map((l) => ` ${gm(l)}`).join(`,
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
  function gl(n, t) {
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
  const An = /* @__PURE__ */ Object.create(null), Ur = /* @__PURE__ */ new Map();
  let bm = 0;
  function _m({ template: n, bits: t }) {
    const e = Yh(n, t);
    if (An[e]) return An[e];
    const { vertex: s, fragment: i } = vm(n, t);
    return An[e] = Xh(s, i, t), An[e];
  }
  function wm({ template: n, bits: t }) {
    const e = Yh(n, t);
    return An[e] || (An[e] = Xh(n.vertex, n.fragment, t)), An[e];
  }
  function vm(n, t) {
    const e = t.map((o) => o.vertex).filter((o) => !!o), s = t.map((o) => o.fragment).filter((o) => !!o);
    let i = fl(e, n.vertex, true);
    i = xm(e, i);
    const r = fl(s, n.fragment, true);
    return {
      vertex: i,
      fragment: r
    };
  }
  function Yh(n, t) {
    return t.map((e) => (Ur.has(e) || Ur.set(e, bm++), Ur.get(e))).sort((e, s) => e - s).join("-") + n.vertex + n.fragment;
  }
  function Xh(n, t, e) {
    const s = ul(n), i = ul(t);
    return e.forEach((r) => {
      dl(r.vertex, s, r.name), dl(r.fragment, i, r.name);
    }), {
      vertex: gl(n, s),
      fragment: gl(t, i)
    };
  }
  const Cm = `
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
`, Sm = `
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
`, Tm = `
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
`, Am = `

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
`, Em = {
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
  }, km = {
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
  Mm = function({ bits: n, name: t }) {
    const e = _m({
      template: {
        fragment: Sm,
        vertex: Cm
      },
      bits: [
        Em,
        ...n
      ]
    });
    return di.from({
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
  Pm = function({ bits: n, name: t }) {
    return new Jo({
      name: t,
      ...wm({
        template: {
          vertex: Tm,
          fragment: Am
        },
        bits: [
          km,
          ...n
        ]
      })
    });
  };
  let jr;
  Im = {
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
  Rm = {
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
  jr = {};
  function Lm(n) {
    const t = [];
    if (n === 1) t.push("@group(1) @binding(0) var textureSource1: texture_2d<f32>;"), t.push("@group(1) @binding(1) var textureSampler1: sampler;");
    else {
      let e = 0;
      for (let s = 0; s < n; s++) t.push(`@group(1) @binding(${e++}) var textureSource${s + 1}: texture_2d<f32>;`), t.push(`@group(1) @binding(${e++}) var textureSampler${s + 1}: sampler;`);
    }
    return t.join(`
`);
  }
  function $m(n) {
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
  Bm = function(n) {
    return jr[n] || (jr[n] = {
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

                ${Lm(n)}
            `,
        main: `
                var uvDx = dpdx(vUV);
                var uvDy = dpdy(vUV);

                ${$m(n)}
            `
      }
    }), jr[n];
  };
  const Vr = {};
  function Om(n) {
    const t = [];
    for (let e = 0; e < n; e++) e > 0 && t.push("else"), e < n - 1 && t.push(`if(vTextureId < ${e}.5)`), t.push("{"), t.push(`	outColor = texture(uTextures[${e}], vUV);`), t.push("}");
    return t.join(`
`);
  }
  Nm = function(n) {
    return Vr[n] || (Vr[n] = {
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

                ${Om(n)}
            `
      }
    }), Vr[n];
  };
  let yl;
  Fm = {
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
  Wm = {
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
  yl = {};
  Gm = function(n) {
    let t = yl[n];
    if (t) return t;
    const e = new Int32Array(n);
    for (let s = 0; s < n; s++) e[s] = s;
    return t = yl[n] = new Zo({
      uTextures: {
        value: e,
        type: "i32",
        size: n
      }
    }, {
      isStatic: true
    }), t;
  };
  class xl extends hr {
    constructor(t) {
      const e = Pm({
        name: "batch",
        bits: [
          Rm,
          Nm(t),
          Wm
        ]
      }), s = Mm({
        name: "batch",
        bits: [
          Im,
          Bm(t),
          Fm
        ]
      });
      super({
        glProgram: e,
        gpuProgram: s,
        resources: {
          batchSamplers: Gm(t)
        }
      }), this.maxTextures = t;
    }
  }
  let Os = null;
  const qh = class Kh extends cm {
    constructor(t) {
      super(t), this.geometry = new fm(), this.name = Kh.extension.name, this.vertexSize = 6, Os ?? (Os = new xl(t.maxTextures)), this.shader = Os;
    }
    packAttributes(t, e, s, i, r) {
      const o = r << 16 | t.roundPixels & 65535, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, { positions: f, uvs: m } = t, g = t.color, y = t.attributeOffset, w = y + t.attributeSize;
      for (let x = y; x < w; x++) {
        const b = x * 2, _ = f[b], v = f[b + 1];
        e[i++] = l * _ + h * v + u, e[i++] = d * v + c * _ + p, e[i++] = m[b], e[i++] = m[b + 1], s[i++] = g, s[i++] = o;
      }
    }
    packQuadAttributes(t, e, s, i, r) {
      const o = t.texture, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, f = t.bounds, m = f.maxX, g = f.minX, y = f.maxY, w = f.minY, x = o.uvs, b = t.color, _ = r << 16 | t.roundPixels & 65535;
      e[i + 0] = l * g + h * w + u, e[i + 1] = d * w + c * g + p, e[i + 2] = x.x0, e[i + 3] = x.y0, s[i + 4] = b, s[i + 5] = _, e[i + 6] = l * m + h * w + u, e[i + 7] = d * w + c * m + p, e[i + 8] = x.x1, e[i + 9] = x.y1, s[i + 10] = b, s[i + 11] = _, e[i + 12] = l * m + h * y + u, e[i + 13] = d * y + c * m + p, e[i + 14] = x.x2, e[i + 15] = x.y2, s[i + 16] = b, s[i + 17] = _, e[i + 18] = l * g + h * y + u, e[i + 19] = d * y + c * g + p, e[i + 20] = x.x3, e[i + 21] = x.y3, s[i + 22] = b, s[i + 23] = _;
    }
    _updateMaxTextures(t) {
      this.shader.maxTextures !== t && (Os = new xl(t), this.shader = Os);
    }
    destroy() {
      this.shader = null, super.destroy();
    }
  };
  qh.extension = {
    type: [
      ct.Batcher
    ],
    name: "default"
  };
  zm = qh;
  Ms = class {
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
  function Dm(n, t, e, s, i, r, o, a = null) {
    let l = 0;
    e *= t, i *= r;
    const c = a.a, h = a.b, d = a.c, u = a.d, p = a.tx, f = a.ty;
    for (; l < o; ) {
      const m = n[e], g = n[e + 1];
      s[i] = c * m + d * g + p, s[i + 1] = h * m + u * g + f, i += r, e += t, l++;
    }
  }
  function Hm(n, t, e, s) {
    let i = 0;
    for (t *= e; i < s; ) n[t] = 0, n[t + 1] = 0, t += e, i++;
  }
  function Jh(n, t, e, s, i) {
    const r = t.a, o = t.b, a = t.c, l = t.d, c = t.tx, h = t.ty;
    e || (e = 0), s || (s = 2), i || (i = n.length / s - e);
    let d = e * s;
    for (let u = 0; u < i; u++) {
      const p = n[d], f = n[d + 1];
      n[d] = r * p + a * f + c, n[d + 1] = o * p + l * f + h, d += s;
    }
  }
  const Um = new vt();
  class sa {
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
      return s ? sh(e, s.groupColor) + (this.alpha * s.groupAlpha * 255 << 24) : e + (this.alpha * 255 << 24);
    }
    get transform() {
      var _a2;
      return ((_a2 = this.renderable) == null ? void 0 : _a2.groupTransform) || Um;
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
  const si = {
    extension: {
      type: ct.ShapeBuilder,
      name: "circle"
    },
    build(n, t) {
      let e, s, i, r, o, a;
      if (n.type === "circle") {
        const b = n;
        if (o = a = b.radius, o <= 0) return false;
        e = b.x, s = b.y, i = r = 0;
      } else if (n.type === "ellipse") {
        const b = n;
        if (o = b.halfWidth, a = b.halfHeight, o <= 0 || a <= 0) return false;
        e = b.x, s = b.y, i = r = 0;
      } else {
        const b = n, _ = b.width / 2, v = b.height / 2;
        e = b.x + _, s = b.y + v, o = a = Math.max(0, Math.min(b.radius, Math.min(_, v))), i = _ - o, r = v - a;
      }
      if (i < 0 || r < 0) return false;
      const l = Math.ceil(2.3 * Math.sqrt(o + a)), c = l * 8 + (i ? 4 : 0) + (r ? 4 : 0);
      if (c === 0) return false;
      if (l === 0) return t[0] = t[6] = e + i, t[1] = t[3] = s + r, t[2] = t[4] = e - i, t[5] = t[7] = s - r, true;
      let h = 0, d = l * 4 + (i ? 2 : 0) + 2, u = d, p = c, f = i + o, m = r, g = e + f, y = e - f, w = s + m;
      if (t[h++] = g, t[h++] = w, t[--d] = w, t[--d] = y, r) {
        const b = s - m;
        t[u++] = y, t[u++] = b, t[--p] = b, t[--p] = g;
      }
      for (let b = 1; b < l; b++) {
        const _ = Math.PI / 2 * (b / l), v = i + Math.cos(_) * o, C = r + Math.sin(_) * a, E = e + v, R = e - v, k = s + C, T = s - C;
        t[h++] = E, t[h++] = k, t[--d] = k, t[--d] = R, t[u++] = R, t[u++] = T, t[--p] = T, t[--p] = E;
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
  }, jm = {
    ...si,
    extension: {
      ...si.extension,
      name: "ellipse"
    }
  }, Vm = {
    ...si,
    extension: {
      ...si.extension,
      name: "roundedRectangle"
    }
  }, Zh = 1e-4, bl = 1e-4;
  function Ym(n) {
    const t = n.length;
    if (t < 6) return 1;
    let e = 0;
    for (let s = 0, i = n[t - 2], r = n[t - 1]; s < t; s += 2) {
      const o = n[s], a = n[s + 1];
      e += (o - i) * (a + r), i = o, r = a;
    }
    return e < 0 ? -1 : 1;
  }
  function _l(n, t, e, s, i, r, o, a) {
    const l = n - e * i, c = t - s * i, h = n + e * r, d = t + s * r;
    let u, p;
    o ? (u = s, p = -e) : (u = -s, p = e);
    const f = l + u, m = c + p, g = h + u, y = d + p;
    return a.push(f, m), a.push(g, y), 2;
  }
  function Bn(n, t, e, s, i, r, o, a) {
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
  Xm = function(n, t, e, s, i, r) {
    const o = Zh;
    if (n.length === 0) return;
    const a = t;
    let l = a.alignment;
    if (t.alignment !== 0.5) {
      let X = Ym(n);
      l = (l - 0.5) * X + 0.5;
    }
    const c = new Et(n[0], n[1]), h = new Et(n[n.length - 2], n[n.length - 1]), d = s, u = Math.abs(c.x - h.x) < o && Math.abs(c.y - h.y) < o;
    if (d) {
      n = n.slice(), u && (n.pop(), n.pop(), h.set(n[n.length - 2], n[n.length - 1]));
      const X = (c.x + h.x) * 0.5, Z = (h.y + c.y) * 0.5;
      n.unshift(X, Z), n.push(X, Z);
    }
    const p = i, f = n.length / 2;
    let m = n.length;
    const g = p.length / 2, y = a.width / 2, w = y * y, x = a.miterLimit * a.miterLimit;
    let b = n[0], _ = n[1], v = n[2], C = n[3], E = 0, R = 0, k = -(_ - C), T = b - v, O = 0, U = 0, G = Math.sqrt(k * k + T * T);
    k /= G, T /= G, k *= y, T *= y;
    const B = l, L = (1 - B) * 2, D = B * 2;
    d || (a.cap === "round" ? m += Bn(b - k * (L - D) * 0.5, _ - T * (L - D) * 0.5, b - k * L, _ - T * L, b + k * D, _ + T * D, p, true) + 2 : a.cap === "square" && (m += _l(b, _, k, T, L, D, true, p))), p.push(b - k * L, _ - T * L), p.push(b + k * D, _ + T * D);
    for (let X = 1; X < f - 1; ++X) {
      b = n[(X - 1) * 2], _ = n[(X - 1) * 2 + 1], v = n[X * 2], C = n[X * 2 + 1], E = n[(X + 1) * 2], R = n[(X + 1) * 2 + 1], k = -(_ - C), T = b - v, G = Math.sqrt(k * k + T * T), k /= G, T /= G, k *= y, T *= y, O = -(C - R), U = v - E, G = Math.sqrt(O * O + U * U), O /= G, U /= G, O *= y, U *= y;
      const Z = v - b, M = _ - C, N = v - E, W = R - C, z = Z * N + M * W, q = M * N - W * Z, Q = q < 0;
      if (Math.abs(q) < 1e-3 * Math.abs(z)) {
        p.push(v - k * L, C - T * L), p.push(v + k * D, C + T * D), z >= 0 && (a.join === "round" ? m += Bn(v, C, v - k * L, C - T * L, v - O * L, C - U * L, p, false) + 4 : m += 2, p.push(v - O * D, C - U * D), p.push(v + O * L, C + U * L));
        continue;
      }
      const at = (-k + b) * (-T + C) - (-k + v) * (-T + _), st = (-O + E) * (-U + C) - (-O + v) * (-U + R), mt = (Z * st - N * at) / q, _t = (W * at - M * st) / q, Y = (mt - v) * (mt - v) + (_t - C) * (_t - C), tt = v + (mt - v) * L, ot = C + (_t - C) * L, dt = v - (mt - v) * D, bt = C - (_t - C) * D, Tt = Math.min(Z * Z + M * M, N * N + W * W), jt = Q ? L : D, H = Tt + jt * jt * w;
      Y <= H ? a.join === "bevel" || Y / w > x ? (Q ? (p.push(tt, ot), p.push(v + k * D, C + T * D), p.push(tt, ot), p.push(v + O * D, C + U * D)) : (p.push(v - k * L, C - T * L), p.push(dt, bt), p.push(v - O * L, C - U * L), p.push(dt, bt)), m += 2) : a.join === "round" ? Q ? (p.push(tt, ot), p.push(v + k * D, C + T * D), m += Bn(v, C, v + k * D, C + T * D, v + O * D, C + U * D, p, true) + 4, p.push(tt, ot), p.push(v + O * D, C + U * D)) : (p.push(v - k * L, C - T * L), p.push(dt, bt), m += Bn(v, C, v - k * L, C - T * L, v - O * L, C - U * L, p, false) + 4, p.push(v - O * L, C - U * L), p.push(dt, bt)) : (p.push(tt, ot), p.push(dt, bt)) : (p.push(v - k * L, C - T * L), p.push(v + k * D, C + T * D), a.join === "round" ? Q ? m += Bn(v, C, v + k * D, C + T * D, v + O * D, C + U * D, p, true) + 2 : m += Bn(v, C, v - k * L, C - T * L, v - O * L, C - U * L, p, false) + 2 : a.join === "miter" && Y / w <= x && (Q ? (p.push(dt, bt), p.push(dt, bt)) : (p.push(tt, ot), p.push(tt, ot)), m += 2), p.push(v - O * L, C - U * L), p.push(v + O * D, C + U * D), m += 2);
    }
    b = n[(f - 2) * 2], _ = n[(f - 2) * 2 + 1], v = n[(f - 1) * 2], C = n[(f - 1) * 2 + 1], k = -(_ - C), T = b - v, G = Math.sqrt(k * k + T * T), k /= G, T /= G, k *= y, T *= y, p.push(v - k * L, C - T * L), p.push(v + k * D, C + T * D), d || (a.cap === "round" ? m += Bn(v - k * (L - D) * 0.5, C - T * (L - D) * 0.5, v - k * L, C - T * L, v + k * D, C + T * D, p, false) + 2 : a.cap === "square" && (m += _l(v, C, k, T, L, D, false, p)));
    const K = bl * bl;
    for (let X = g; X < m + g - 2; ++X) b = p[X * 2], _ = p[X * 2 + 1], v = p[(X + 1) * 2], C = p[(X + 1) * 2 + 1], E = p[(X + 2) * 2], R = p[(X + 2) * 2 + 1], !(Math.abs(b * (C - R) + v * (R - _) + E * (_ - C)) < K) && r.push(X, X + 1, X + 2);
  };
  function qm(n, t, e, s) {
    const i = Zh;
    if (n.length === 0) return;
    const r = n[0], o = n[1], a = n[n.length - 2], l = n[n.length - 1], c = t || Math.abs(r - a) < i && Math.abs(o - l) < i, h = e, d = n.length / 2, u = h.length / 2;
    for (let p = 0; p < d; p++) h.push(n[p * 2]), h.push(n[p * 2 + 1]);
    for (let p = 0; p < d - 1; p++) s.push(u + p, u + p + 1);
    c && s.push(u + d - 1, u);
  }
  function Qh(n, t, e, s, i, r, o) {
    const a = uf(n, t, 2);
    if (!a) return;
    for (let c = 0; c < a.length; c += 3) r[o++] = a[c] + i, r[o++] = a[c + 1] + i, r[o++] = a[c + 2] + i;
    let l = i * s;
    for (let c = 0; c < n.length; c += 2) e[l] = n[c], e[l + 1] = n[c + 1], l += s;
  }
  const Km = [], Jm = {
    extension: {
      type: ct.ShapeBuilder,
      name: "polygon"
    },
    build(n, t) {
      for (let e = 0; e < n.points.length; e++) t[e] = n.points[e];
      return true;
    },
    triangulate(n, t, e, s, i, r) {
      Qh(n, Km, t, e, s, i, r);
    }
  }, Zm = {
    extension: {
      type: ct.ShapeBuilder,
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
  }, Qm = {
    extension: {
      type: ct.ShapeBuilder,
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
  }, wl = [
    {
      offset: 0,
      color: "white"
    },
    {
      offset: 1,
      color: "black"
    }
  ], ia = class Eo {
    constructor(...t) {
      this.uid = ne("fillGradient"), this._tick = 0, this.type = "linear", this.colorStops = [];
      let e = tg(t);
      e = {
        ...e.type === "radial" ? Eo.defaultRadialOptions : Eo.defaultLinearOptions,
        ...Vc(e)
      }, this._textureSize = e.textureSize, this._wrapMode = e.wrapMode, e.type === "radial" ? (this.center = e.center, this.outerCenter = e.outerCenter ?? this.center, this.innerRadius = e.innerRadius, this.outerRadius = e.outerRadius, this.scale = e.scale, this.rotation = e.rotation) : (this.start = e.start, this.end = e.end), this.textureSpace = e.textureSpace, this.type = e.type, e.colorStops.forEach((i) => {
        this.addColorStop(i.offset, i.color);
      });
    }
    addColorStop(t, e) {
      return this.colorStops.push({
        offset: t,
        color: Yt.shared.setValue(e).toHexa()
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
      const l = this.colorStops.length ? this.colorStops : wl, c = this._textureSize, { canvas: h, context: d } = Cl(c, 1), u = a ? d.createLinearGradient(this._textureSize, 0, 0, 0) : d.createLinearGradient(0, 0, this._textureSize, 0);
      vl(u, l), d.fillStyle = u, d.fillRect(0, 0, c, 1), this.texture = new Ct({
        source: new vs({
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
      const t = this.colorStops.length ? this.colorStops : wl, e = this._textureSize, { canvas: s, context: i } = Cl(e, e), { x: r, y: o } = this.center, { x: a, y: l } = this.outerCenter, c = this.innerRadius, h = this.outerRadius, d = a - h, u = l - h, p = e / (h * 2), f = (r - d) * p, m = (o - u) * p, g = i.createRadialGradient(f, m, c * p, (a - d) * p, (l - u) * p, h * p);
      vl(g, t), i.fillStyle = t[t.length - 1].color, i.fillRect(0, 0, e, e), i.fillStyle = g, i.translate(f, m), i.rotate(this.rotation), i.scale(1, this.scale), i.translate(-f, -m), i.fillRect(0, 0, e, e), this.texture = new Ct({
        source: new vs({
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
  ia.defaultLinearOptions = {
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
  ia.defaultRadialOptions = {
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
  bn = ia;
  function vl(n, t) {
    for (let e = 0; e < t.length; e++) {
      const s = t[e];
      n.addColorStop(s.offset, s.color);
    }
  }
  function Cl(n, t) {
    const e = Rt.get().createCanvas(n, t), s = e.getContext("2d");
    return {
      canvas: e,
      context: s
    };
  }
  function tg(n) {
    let t = n[0] ?? {};
    return (typeof t == "number" || n[1]) && (It("8.5.2", "use options object instead"), t = {
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
      textureSize: n[5] ?? bn.defaultLinearOptions.textureSize
    }), t;
  }
  const eg = new vt(), ng = new Ht();
  sg = function(n, t, e, s) {
    const i = t.matrix ? n.copyFrom(t.matrix).invert() : n.identity();
    if (t.textureSpace === "local") {
      const o = e.getBounds(ng);
      t.width && o.pad(t.width);
      const { x: a, y: l } = o, c = 1 / o.width, h = 1 / o.height, d = -a * c, u = -l * h, p = i.a, f = i.b, m = i.c, g = i.d;
      i.a *= c, i.b *= c, i.c *= h, i.d *= h, i.tx = d * p + u * m + i.tx, i.ty = d * f + u * g + i.ty;
    } else i.translate(t.texture.frame.x, t.texture.frame.y), i.scale(1 / t.texture.source.width, 1 / t.texture.source.height);
    const r = t.texture.source.style;
    return !(t.fill instanceof bn) && r.addressMode === "clamp-to-edge" && (r.addressMode = "repeat", r.update()), s && i.append(eg.copyFrom(s).invert()), i;
  };
  ur = {};
  zt.handleByMap(ct.ShapeBuilder, ur);
  zt.add(Zm, Jm, Qm, si, jm, Vm);
  const ig = new Ht(), rg = new vt();
  function og(n, t) {
    const { geometryData: e, batches: s } = t;
    s.length = 0, e.indices.length = 0, e.vertices.length = 0, e.uvs.length = 0;
    for (let i = 0; i < n.instructions.length; i++) {
      const r = n.instructions[i];
      if (r.action === "texture") ag(r.data, s, e);
      else if (r.action === "fill" || r.action === "stroke") {
        const o = r.action === "stroke", a = r.data.path.shapePath, l = r.data.style, c = r.data.hole;
        o && c && Sl(c.shapePath, l, true, s, e), c && (a.shapePrimitives[a.shapePrimitives.length - 1].holes = c.shapePath.shapePrimitives), Sl(a, l, o, s, e);
      }
    }
  }
  function ag(n, t, e) {
    const s = [], i = ur.rectangle, r = ig;
    r.x = n.dx, r.y = n.dy, r.width = n.dw, r.height = n.dh;
    const o = n.transform;
    if (!i.build(r, s)) return;
    const { vertices: a, uvs: l, indices: c } = e, h = c.length, d = a.length / 2;
    o && Jh(s, o), i.triangulate(s, a, 2, d, c, h);
    const u = n.image, p = u.uvs;
    l.push(p.x0, p.y0, p.x1, p.y1, p.x3, p.y3, p.x2, p.y2);
    const f = ve.get(sa);
    f.indexOffset = h, f.indexSize = c.length - h, f.attributeOffset = d, f.attributeSize = a.length / 2 - d, f.baseColor = n.style, f.alpha = n.alpha, f.texture = u, f.geometryData = e, t.push(f);
  }
  function Sl(n, t, e, s, i) {
    const { vertices: r, uvs: o, indices: a } = i;
    n.shapePrimitives.forEach(({ shape: l, transform: c, holes: h }) => {
      const d = [], u = ur[l.type];
      if (!u.build(l, d)) return;
      const p = a.length, f = r.length / 2;
      let m = "triangle-list";
      if (c && Jh(d, c), e) {
        const x = l.closePath ?? true, b = t;
        b.pixelLine ? (qm(d, x, r, a), m = "line-list") : Xm(d, b, false, x, r, a);
      } else if (h) {
        const x = [], b = d.slice();
        lg(h).forEach((v) => {
          x.push(b.length / 2), b.push(...v);
        }), Qh(b, x, r, 2, f, a, p);
      } else u.triangulate(d, r, 2, f, a, p);
      const g = o.length / 2, y = t.texture;
      if (y !== Ct.WHITE) {
        const x = sg(rg, t, l, c);
        Dm(r, 2, f, o, g, 2, r.length / 2 - f, x);
      } else Hm(o, g, 2, r.length / 2 - f);
      const w = ve.get(sa);
      w.indexOffset = p, w.indexSize = a.length - p, w.attributeOffset = f, w.attributeSize = r.length / 2 - f, w.baseColor = t.color, w.alpha = t.alpha, w.texture = y, w.geometryData = i, w.topology = m, s.push(w);
    });
  }
  function lg(n) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e].shape, i = [];
      ur[s.type].build(s, i) && t.push(i);
    }
    return t;
  }
  class cg {
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
  class hg {
    constructor() {
      this.instructions = new qo();
    }
    init(t) {
      const e = t.maxTextures;
      this.batcher ? this.batcher._updateMaxTextures(e) : this.batcher = new zm({
        maxTextures: e
      }), this.instructions.reset();
    }
    get geometry() {
      return It(Nu, "GraphicsContextRenderData#geometry is deprecated, please use batcher.geometry instead."), this.batcher.geometry;
    }
    destroy() {
      this.batcher.destroy(), this.instructions.destroy(), this.batcher = null, this.instructions = null;
    }
  }
  const ra = class ko {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Ms({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      ko.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? ko.defaultOptions.bezierSmoothness;
    }
    getContextRenderData(t) {
      return t._gpuData[this._renderer.uid].graphicsData || this._initContextRenderData(t);
    }
    updateGpuContext(t) {
      const e = !!t._gpuData[this._renderer.uid], s = t._gpuData[this._renderer.uid] || this._initContext(t);
      if (t.dirty || !e) {
        e && s.reset(), og(t, s);
        const i = t.batchMode;
        t.customShader || i === "no-batch" ? s.isBatchable = false : i === "auto" ? s.isBatchable = s.geometryData.vertices.length < 400 : s.isBatchable = true, t.dirty = false;
      }
      return s;
    }
    getGpuContext(t) {
      return t._gpuData[this._renderer.uid] || this._initContext(t);
    }
    _initContextRenderData(t) {
      const e = ve.get(hg, {
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
        u.bindGroup = Qf(u.textures.textures, u.textures.count, this._renderer.limits.maxBatchableTextures);
      }
      return e;
    }
    _initContext(t) {
      const e = new cg();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  ra.extension = {
    type: [
      ct.WebGLSystem,
      ct.WebGPUSystem
    ],
    name: "graphicsContext"
  };
  ra.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let oa = ra;
  const dg = 8, ki = 11920929e-14, ug = 1;
  function td(n, t, e, s, i, r, o, a, l, c) {
    const d = Math.min(0.99, Math.max(0, c ?? oa.defaultOptions.bezierSmoothness));
    let u = (ug - d) / 1;
    return u *= u, pg(t, e, s, i, r, o, a, l, n, u), n;
  }
  function pg(n, t, e, s, i, r, o, a, l, c) {
    Mo(n, t, e, s, i, r, o, a, l, c, 0), l.push(o, a);
  }
  function Mo(n, t, e, s, i, r, o, a, l, c, h) {
    if (h > dg) return;
    const d = (n + e) / 2, u = (t + s) / 2, p = (e + i) / 2, f = (s + r) / 2, m = (i + o) / 2, g = (r + a) / 2, y = (d + p) / 2, w = (u + f) / 2, x = (p + m) / 2, b = (f + g) / 2, _ = (y + x) / 2, v = (w + b) / 2;
    if (h > 0) {
      let C = o - n, E = a - t;
      const R = Math.abs((e - o) * E - (s - a) * C), k = Math.abs((i - o) * E - (r - a) * C);
      if (R > ki && k > ki) {
        if ((R + k) * (R + k) <= c * (C * C + E * E)) {
          l.push(_, v);
          return;
        }
      } else if (R > ki) {
        if (R * R <= c * (C * C + E * E)) {
          l.push(_, v);
          return;
        }
      } else if (k > ki) {
        if (k * k <= c * (C * C + E * E)) {
          l.push(_, v);
          return;
        }
      } else if (C = _ - (n + o) / 2, E = v - (t + a) / 2, C * C + E * E <= c) {
        l.push(_, v);
        return;
      }
    }
    Mo(n, t, d, u, y, w, _, v, l, c, h + 1), Mo(_, v, x, b, m, g, o, a, l, c, h + 1);
  }
  const fg = 8, mg = 11920929e-14, gg = 1;
  function yg(n, t, e, s, i, r, o, a) {
    const c = Math.min(0.99, Math.max(0, a ?? oa.defaultOptions.bezierSmoothness));
    let h = (gg - c) / 1;
    return h *= h, xg(t, e, s, i, r, o, n, h), n;
  }
  function xg(n, t, e, s, i, r, o, a) {
    Po(o, n, t, e, s, i, r, a, 0), o.push(i, r);
  }
  function Po(n, t, e, s, i, r, o, a, l) {
    if (l > fg) return;
    const c = (t + s) / 2, h = (e + i) / 2, d = (s + r) / 2, u = (i + o) / 2, p = (c + d) / 2, f = (h + u) / 2;
    let m = r - t, g = o - e;
    const y = Math.abs((s - r) * g - (i - o) * m);
    if (y > mg) {
      if (y * y <= a * (m * m + g * g)) {
        n.push(p, f);
        return;
      }
    } else if (m = p - (t + r) / 2, g = f - (e + o) / 2, m * m + g * g <= a) {
      n.push(p, f);
      return;
    }
    Po(n, t, e, c, h, p, f, a, l + 1), Po(n, p, f, d, u, r, o, a, l + 1);
  }
  function ed(n, t, e, s, i, r, o, a) {
    let l = Math.abs(i - r);
    (!o && i > r || o && r > i) && (l = 2 * Math.PI - l), a || (a = Math.max(6, Math.floor(6 * Math.pow(s, 1 / 3) * (l / Math.PI)))), a = Math.max(a, 3);
    let c = l / a, h = i;
    c *= o ? -1 : 1;
    for (let d = 0; d < a + 1; d++) {
      const u = Math.cos(h), p = Math.sin(h), f = t + u * s, m = e + p * s;
      n.push(f, m), h += c;
    }
  }
  function bg(n, t, e, s, i, r) {
    const o = n[n.length - 2], l = n[n.length - 1] - e, c = o - t, h = i - e, d = s - t, u = Math.abs(l * d - c * h);
    if (u < 1e-8 || r === 0) {
      (n[n.length - 2] !== t || n[n.length - 1] !== e) && n.push(t, e);
      return;
    }
    const p = l * l + c * c, f = h * h + d * d, m = l * h + c * d, g = r * Math.sqrt(p) / u, y = r * Math.sqrt(f) / u, w = g * m / p, x = y * m / f, b = g * d + y * c, _ = g * h + y * l, v = c * (y + w), C = l * (y + w), E = d * (g + x), R = h * (g + x), k = Math.atan2(C - _, v - b), T = Math.atan2(R - _, E - b);
    ed(n, b + t, _ + e, r, k, T, c * h > d * l);
  }
  const Ks = Math.PI * 2, Yr = {
    centerX: 0,
    centerY: 0,
    ang1: 0,
    ang2: 0
  }, Xr = ({ x: n, y: t }, e, s, i, r, o, a, l) => {
    n *= e, t *= s;
    const c = i * n - r * t, h = r * n + i * t;
    return l.x = c + o, l.y = h + a, l;
  };
  function _g(n, t) {
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
  const Tl = (n, t, e, s) => {
    const i = n * s - t * e < 0 ? -1 : 1;
    let r = n * e + t * s;
    return r > 1 && (r = 1), r < -1 && (r = -1), i * Math.acos(r);
  }, wg = (n, t, e, s, i, r, o, a, l, c, h, d, u) => {
    const p = Math.pow(i, 2), f = Math.pow(r, 2), m = Math.pow(h, 2), g = Math.pow(d, 2);
    let y = p * f - p * g - f * m;
    y < 0 && (y = 0), y /= p * g + f * m, y = Math.sqrt(y) * (o === a ? -1 : 1);
    const w = y * i / r * d, x = y * -r / i * h, b = c * w - l * x + (n + e) / 2, _ = l * w + c * x + (t + s) / 2, v = (h - w) / i, C = (d - x) / r, E = (-h - w) / i, R = (-d - x) / r, k = Tl(1, 0, v, C);
    let T = Tl(v, C, E, R);
    a === 0 && T > 0 && (T -= Ks), a === 1 && T < 0 && (T += Ks), u.centerX = b, u.centerY = _, u.ang1 = k, u.ang2 = T;
  };
  function vg(n, t, e, s, i, r, o, a = 0, l = 0, c = 0) {
    if (r === 0 || o === 0) return;
    const h = Math.sin(a * Ks / 360), d = Math.cos(a * Ks / 360), u = d * (t - s) / 2 + h * (e - i) / 2, p = -h * (t - s) / 2 + d * (e - i) / 2;
    if (u === 0 && p === 0) return;
    r = Math.abs(r), o = Math.abs(o);
    const f = Math.pow(u, 2) / Math.pow(r, 2) + Math.pow(p, 2) / Math.pow(o, 2);
    f > 1 && (r *= Math.sqrt(f), o *= Math.sqrt(f)), wg(t, e, s, i, r, o, l, c, h, d, u, p, Yr);
    let { ang1: m, ang2: g } = Yr;
    const { centerX: y, centerY: w } = Yr;
    let x = Math.abs(g) / (Ks / 4);
    Math.abs(1 - x) < 1e-7 && (x = 1);
    const b = Math.max(Math.ceil(x), 1);
    g /= b;
    let _ = n[n.length - 2], v = n[n.length - 1];
    const C = {
      x: 0,
      y: 0
    };
    for (let E = 0; E < b; E++) {
      const R = _g(m, g), { x: k, y: T } = Xr(R[0], r, o, d, h, y, w, C), { x: O, y: U } = Xr(R[1], r, o, d, h, y, w, C), { x: G, y: B } = Xr(R[2], r, o, d, h, y, w, C);
      td(n, _, v, k, T, O, U, G, B), _ = G, v = B, m += g;
    }
  }
  function Cg(n, t, e) {
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
      const w = a.x + d.nx * y + -d.ny * g * p, x = a.y + d.ny * y + d.nx * g * p, b = Math.atan2(h.ny, h.nx) + Math.PI / 2 * p, _ = Math.atan2(d.ny, d.nx) - Math.PI / 2 * p;
      o === 0 && n.moveTo(w + Math.cos(b) * g, x + Math.sin(b) * g), n.arc(w, x, g, b, _, f), r = a;
    }
  }
  function Sg(n, t, e, s) {
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
  const Tg = new Ht();
  class Ag {
    constructor(t) {
      this.shapePrimitives = [], this._currentPoly = null, this._bounds = new Ee(), this._graphicsPath2D = t, this.signed = t.checkForHoles;
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
      return ed(a, t, e, s, i, r, o), this;
    }
    arcTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly.points;
      return bg(o, t, e, s, i, r), this;
    }
    arcToSvg(t, e, s, i, r, o, a) {
      const l = this._currentPoly.points;
      return vg(l, this._currentPoly.lastX, this._currentPoly.lastY, o, a, t, e, s, i, r), this;
    }
    bezierCurveTo(t, e, s, i, r, o, a) {
      this._ensurePoly();
      const l = this._currentPoly;
      return td(this._currentPoly.points, l.lastX, l.lastY, t, e, s, i, r, o, a), this;
    }
    quadraticCurveTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly;
      return yg(this._currentPoly.points, o.lastX, o.lastY, t, e, s, i, r), this;
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
      return this.drawShape(new ta(t, e, s), i), this;
    }
    poly(t, e, s) {
      const i = new Xs(t);
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
        const p = u * h + c, f = t + s * Math.cos(p), m = e + s * Math.sin(p), g = p + Math.PI + d, y = p - Math.PI - d, w = f + r * Math.cos(g), x = m + r * Math.sin(g), b = f + r * Math.cos(y), _ = m + r * Math.sin(y);
        u === 0 ? this.moveTo(w, x) : this.lineTo(w, x), this.quadraticCurveTo(f, m, b, _, a);
      }
      return this.closePath();
    }
    roundShape(t, e, s = false, i) {
      return t.length < 3 ? this : (s ? Sg(this, t, e, i) : Cg(this, t, e), this.closePath());
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
      return this.drawShape(new ea(t, e, s, i), r), this;
    }
    roundRect(t, e, s, i, r, o) {
      return this.drawShape(new na(t, e, s, i, r), o), this;
    }
    drawShape(t, e) {
      return this.endPoly(), this.shapePrimitives.push({
        shape: t,
        transform: e
      }), this;
    }
    startPoly(t, e) {
      let s = this._currentPoly;
      return s && this.endPoly(), s = new Xs(), s.points.push(t, e), this._currentPoly = s, this;
    }
    endPoly(t = false) {
      const e = this._currentPoly;
      return e && e.points.length > 2 && (e.closePath = t, this.shapePrimitives.push({
        shape: e
      })), this._currentPoly = null, this;
    }
    _ensurePoly(t = true) {
      if (!this._currentPoly && (this._currentPoly = new Xs(), t)) {
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
        const i = e[s], r = i.shape.getBounds(Tg);
        i.transform ? t.addRect(r, i.transform) : t.addRect(r);
      }
      return t;
    }
  }
  class yn {
    constructor(t, e = false) {
      this.instructions = [], this.uid = ne("graphicsPath"), this._dirty = true, this.checkForHoles = e, typeof t == "string" ? qf(t, this) : this.instructions = (t == null ? void 0 : t.slice()) ?? [];
    }
    get shapePath() {
      return this._shapePath || (this._shapePath = new Ag(this)), this._dirty && (this._dirty = false, this._shapePath.buildPath()), this._shapePath;
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
      const o = this.instructions[this.instructions.length - 1], a = this.getLastPoint(Et.shared);
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
      const i = this.instructions[this.instructions.length - 1], r = this.getLastPoint(Et.shared);
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
      const e = new yn();
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
            w[4] = Ns(w[3], t);
            break;
          case "rect":
            w[4] = Ns(w[4], t);
            break;
          case "ellipse":
            w[8] = Ns(w[8], t);
            break;
          case "roundRect":
            w[5] = Ns(w[5], t);
            break;
          case "addPath":
            w[0].transform(t);
            break;
          case "poly":
            w[2] = Ns(w[2], t);
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
  function Ns(n, t) {
    return n ? n.prepend(t) : t.clone();
  }
  function te(n, t, e) {
    const s = n.getAttribute(t);
    return s ? Number(s) : e;
  }
  function Eg(n, t) {
    const e = n.querySelectorAll("defs");
    for (let s = 0; s < e.length; s++) {
      const i = e[s];
      for (let r = 0; r < i.children.length; r++) {
        const o = i.children[r];
        switch (o.nodeName.toLowerCase()) {
          case "lineargradient":
            t.defs[o.id] = kg(o);
            break;
          case "radialgradient":
            t.defs[o.id] = Mg();
            break;
        }
      }
    }
  }
  function kg(n) {
    const t = te(n, "x1", 0), e = te(n, "y1", 0), s = te(n, "x2", 1), i = te(n, "y2", 0), r = n.getAttribute("gradientUnits") || "objectBoundingBox", o = new bn(t, e, s, i, r === "objectBoundingBox" ? "local" : "global");
    for (let a = 0; a < n.children.length; a++) {
      const l = n.children[a], c = te(l, "offset", 0), h = Yt.shared.setValue(l.getAttribute("stop-color")).toNumber();
      o.addColorStop(c, h);
    }
    return o;
  }
  function Mg(n) {
    return Jt("[SVG Parser] Radial gradients are not yet supported"), new bn(0, 0, 1, 0);
  }
  function Al(n) {
    const t = n.match(/url\s*\(\s*['"]?\s*#([^'"\s)]+)\s*['"]?\s*\)/i);
    return t ? t[1] : "";
  }
  const El = {
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
  function nd(n, t) {
    const e = n.getAttribute("style"), s = {}, i = {}, r = {
      strokeStyle: s,
      fillStyle: i,
      useFill: false,
      useStroke: false
    };
    for (const o in El) {
      const a = n.getAttribute(o);
      a && kl(t, r, o, a.trim());
    }
    if (e) {
      const o = e.split(";");
      for (let a = 0; a < o.length; a++) {
        const l = o[a].trim(), [c, h] = l.split(":");
        El[c] && kl(t, r, c, h.trim());
      }
    }
    return {
      strokeStyle: r.useStroke ? s : null,
      fillStyle: r.useFill ? i : null,
      useFill: r.useFill,
      useStroke: r.useStroke
    };
  }
  function kl(n, t, e, s) {
    switch (e) {
      case "stroke":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Al(s);
            t.strokeStyle.fill = n.defs[i];
          } else t.strokeStyle.color = Yt.shared.setValue(s).toNumber();
          t.useStroke = true;
        }
        break;
      case "stroke-width":
        t.strokeStyle.width = Number(s);
        break;
      case "fill":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Al(s);
            t.fillStyle.fill = n.defs[i];
          } else t.fillStyle.color = Yt.shared.setValue(s).toNumber();
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
  function Pg(n) {
    if (n.length <= 2) return true;
    const t = n.map((a) => a.area).sort((a, l) => l - a), [e, s] = t, i = t[t.length - 1], r = e / s, o = s / i;
    return !(r > 3 && o < 2);
  }
  function Ig(n) {
    return n.split(/(?=[Mm])/).filter((s) => s.trim().length > 0);
  }
  function Rg(n) {
    const t = n.match(/[-+]?[0-9]*\.?[0-9]+/g);
    if (!t || t.length < 4) return 0;
    const e = t.map(Number), s = [], i = [];
    for (let h = 0; h < e.length; h += 2) h + 1 < e.length && (s.push(e[h]), i.push(e[h + 1]));
    if (s.length === 0 || i.length === 0) return 0;
    const r = Math.min(...s), o = Math.max(...s), a = Math.min(...i), l = Math.max(...i);
    return (o - r) * (l - a);
  }
  function Ml(n, t) {
    const e = new yn(n, false);
    for (const s of e.instructions) t.instructions.push(s);
  }
  function Lg(n, t) {
    if (typeof n == "string") {
      const o = document.createElement("div");
      o.innerHTML = n.trim(), n = o.querySelector("svg");
    }
    const e = {
      context: t,
      defs: {},
      path: new yn()
    };
    Eg(n, e);
    const s = n.children, { fillStyle: i, strokeStyle: r } = nd(n, e);
    for (let o = 0; o < s.length; o++) {
      const a = s[o];
      a.nodeName.toLowerCase() !== "defs" && sd(a, e, i, r);
    }
    return t;
  }
  function sd(n, t, e, s) {
    const i = n.children, { fillStyle: r, strokeStyle: o } = nd(n, t);
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
    let l, c, h, d, u, p, f, m, g, y, w, x, b, _, v, C, E;
    switch (n.nodeName.toLowerCase()) {
      case "path": {
        _ = n.getAttribute("d");
        const R = n.getAttribute("fill-rule"), k = Ig(_), T = R === "evenodd", O = k.length > 1;
        if (T && O) {
          const G = k.map((L) => ({
            path: L,
            area: Rg(L)
          }));
          if (G.sort((L, D) => D.area - L.area), k.length > 3 || !Pg(G)) for (let L = 0; L < G.length; L++) {
            const D = G[L], K = L === 0;
            t.context.beginPath();
            const X = new yn(void 0, true);
            Ml(D.path, X), t.context.path(X), K ? (e && t.context.fill(e), s && t.context.stroke(s)) : t.context.cut();
          }
          else for (let L = 0; L < G.length; L++) {
            const D = G[L], K = L % 2 === 1;
            t.context.beginPath();
            const X = new yn(void 0, true);
            Ml(D.path, X), t.context.path(X), K ? t.context.cut() : (e && t.context.fill(e), s && t.context.stroke(s));
          }
        } else {
          const G = R ? R === "evenodd" : true;
          v = new yn(_, G), t.context.path(v), e && t.context.fill(e), s && t.context.stroke(s);
        }
        break;
      }
      case "circle":
        f = te(n, "cx", 0), m = te(n, "cy", 0), g = te(n, "r", 0), t.context.ellipse(f, m, g, g), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "rect":
        l = te(n, "x", 0), c = te(n, "y", 0), C = te(n, "width", 0), E = te(n, "height", 0), y = te(n, "rx", 0), w = te(n, "ry", 0), y || w ? t.context.roundRect(l, c, C, E, y || w) : t.context.rect(l, c, C, E), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "ellipse":
        f = te(n, "cx", 0), m = te(n, "cy", 0), y = te(n, "rx", 0), w = te(n, "ry", 0), t.context.beginPath(), t.context.ellipse(f, m, y, w), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "line":
        h = te(n, "x1", 0), d = te(n, "y1", 0), u = te(n, "x2", 0), p = te(n, "y2", 0), t.context.beginPath(), t.context.moveTo(h, d), t.context.lineTo(u, p), s && t.context.stroke(s);
        break;
      case "polygon":
        b = n.getAttribute("points"), x = b.match(/-?\d+/g).map((R) => parseInt(R, 10)), t.context.poly(x, true), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "polyline":
        b = n.getAttribute("points"), x = b.match(/-?\d+/g).map((R) => parseInt(R, 10)), t.context.poly(x, false), s && t.context.stroke(s);
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
    for (let R = 0; R < i.length; R++) sd(i[R], t, e, s);
  }
  const Pl = {
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
  pr = class {
    constructor(t, e) {
      this.uid = ne("fillPattern"), this._tick = 0, this.transform = new vt(), this.texture = t, this.transform.scale(1 / t.frame.width, 1 / t.frame.height), e && (t.source.style.addressModeU = Pl[e].addressModeU, t.source.style.addressModeV = Pl[e].addressModeV);
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
  function $g(n) {
    return Yt.isColorLike(n);
  }
  function Il(n) {
    return n instanceof pr;
  }
  function Rl(n) {
    return n instanceof bn;
  }
  function Bg(n) {
    return n instanceof Ct;
  }
  function Og(n, t, e) {
    const s = Yt.shared.setValue(t ?? 0);
    return n.color = s.toNumber(), n.alpha = s.alpha === 1 ? e.alpha : s.alpha, n.texture = Ct.WHITE, {
      ...e,
      ...n
    };
  }
  function Ng(n, t, e) {
    return n.texture = t, {
      ...e,
      ...n
    };
  }
  function Ll(n, t, e) {
    return n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, {
      ...e,
      ...n
    };
  }
  function $l(n, t, e) {
    return t.buildGradient(), n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, n.textureSpace = t.textureSpace, {
      ...e,
      ...n
    };
  }
  function Fg(n, t) {
    const e = {
      ...t,
      ...n
    }, s = Yt.shared.setValue(e.color);
    return e.alpha *= s.alpha, e.color = s.toNumber(), e;
  }
  function qn(n, t) {
    if (n == null) return null;
    const e = {}, s = n;
    return $g(n) ? Og(e, n, t) : Bg(n) ? Ng(e, n, t) : Il(n) ? Ll(e, n, t) : Rl(n) ? $l(e, n, t) : s.fill && Il(s.fill) ? Ll(s, s.fill, t) : s.fill && Rl(s.fill) ? $l(s, s.fill, t) : Fg(s, t);
  }
  function Zi(n, t) {
    const { width: e, alignment: s, miterLimit: i, cap: r, join: o, pixelLine: a, ...l } = t, c = qn(n, l);
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
  function Wg(n, t) {
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
        const h = (c - 2 + a) % a, d = (c + 2) % a, u = o[h], p = o[h + 1], f = o[c], m = o[c + 1], g = o[d], y = o[d + 1], w = u - f, x = p - m, b = g - f, _ = y - m, v = w * w + x * x, C = b * b + _ * _;
        if (v < 1e-12 || C < 1e-12) continue;
        let k = (w * b + x * _) / Math.sqrt(v * C);
        k < -1 ? k = -1 : k > 1 && (k = 1);
        const T = Math.sqrt((1 - k) * 0.5);
        if (T < 1e-6) continue;
        const O = Math.min(1 / T, t);
        O > e && (e = O);
      }
    }
    return e;
  }
  const Gg = new Et(), Bl = new vt(), aa = class Qe extends on {
    constructor() {
      super(...arguments), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ne("graphicsContext"), this.dirty = true, this.batchMode = "auto", this.instructions = [], this.destroyed = false, this._activePath = new yn(), this._transform = new vt(), this._fillStyle = {
        ...Qe.defaultFillStyle
      }, this._strokeStyle = {
        ...Qe.defaultStrokeStyle
      }, this._stateStack = [], this._tick = 0, this._bounds = new Ee(), this._boundsDirty = true;
    }
    clone() {
      const t = new Qe();
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
      this._fillStyle = qn(t, Qe.defaultFillStyle);
    }
    get strokeStyle() {
      return this._strokeStyle;
    }
    set strokeStyle(t) {
      this._strokeStyle = Zi(t, Qe.defaultStrokeStyle);
    }
    setFillStyle(t) {
      return this._fillStyle = qn(t, Qe.defaultFillStyle), this;
    }
    setStrokeStyle(t) {
      return this._strokeStyle = qn(t, Qe.defaultStrokeStyle), this;
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
          style: e || e === 0 ? Yt.shared.setValue(e).toNumber() : 16777215
        }
      }), this.onUpdate(), this;
    }
    beginPath() {
      return this._activePath = new yn(), this;
    }
    fill(t, e) {
      let s;
      const i = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (i == null ? void 0 : i.action) === "stroke" ? s = i.data.path : s = this._activePath.clone(), s ? (t != null && (e !== void 0 && typeof t == "number" && (It(ee, "GraphicsContext.fill(color, alpha) is deprecated, use GraphicsContext.fill({ color, alpha }) instead"), t = {
        color: t,
        alpha: e
      }), this._fillStyle = qn(t, Qe.defaultFillStyle)), this.instructions.push({
        action: "fill",
        data: {
          style: this.fillStyle,
          path: s
        }
      }), this.onUpdate(), this._initNextPathLocation(), this._tick = 0, this) : this;
    }
    _initNextPathLocation() {
      const { x: t, y: e } = this._activePath.getLastPoint(Et.shared);
      this._activePath.clear(), this._activePath.moveTo(t, e);
    }
    stroke(t) {
      let e;
      const s = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (s == null ? void 0 : s.action) === "fill" ? e = s.data.path : e = this._activePath.clone(), e ? (t != null && (this._strokeStyle = Zi(t, Qe.defaultStrokeStyle)), this.instructions.push({
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
      return this._tick++, Lg(t, this), this;
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
      return t instanceof vt ? (this._transform.append(t), this) : (Bl.set(t, e, s, i, r, o), this._transform.append(Bl), this);
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
          r.style.join === "miter" && (a *= Wg(r.path, r.style.miterLimit));
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
          const u = c[h].transform, p = u ? u.applyInverse(t, Gg) : t;
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
  aa.defaultFillStyle = {
    color: 16777215,
    alpha: 1,
    texture: Ct.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local"
  };
  aa.defaultStrokeStyle = {
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
  let Me = aa;
  function la(n, t = 1) {
    var _a2;
    const e = (_a2 = As.RETINA_PREFIX) == null ? void 0 : _a2.exec(n);
    return e ? parseFloat(e[1]) : t;
  }
  function ca(n, t, e) {
    n.label = e, n._sourceOrigin = e;
    const s = new Ct({
      source: n,
      label: e
    }), i = () => {
      delete t.promiseCache[e], se.has(e) && se.remove(e);
    };
    return s.source.once("destroy", () => {
      t.promiseCache[e] && (Jt("[Assets] A TextureSource managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the TextureSource."), i());
    }), s.once("destroy", () => {
      n.destroyed || (Jt("[Assets] A Texture managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the Texture."), i());
    }), s;
  }
  const zg = ".svg", Dg = "image/svg+xml", Hg = {
    extension: {
      type: ct.LoadParser,
      priority: Rn.Low,
      name: "loadSVG"
    },
    name: "loadSVG",
    id: "svg",
    config: {
      crossOrigin: "anonymous",
      parseAsGraphicsContext: false
    },
    test(n) {
      return Es(n, Dg) || ks(n, zg);
    },
    async load(n, t, e) {
      var _a2;
      return ((_a2 = t.data) == null ? void 0 : _a2.parseAsGraphicsContext) ?? this.config.parseAsGraphicsContext ? jg(n) : Ug(n, t, e, this.config.crossOrigin);
    },
    unload(n) {
      n.destroy(true);
    }
  };
  async function Ug(n, t, e, s) {
    var _a2, _b2, _c2;
    const i = await Rt.get().fetch(n), r = Rt.get().createImage();
    r.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(await i.text())}`, r.crossOrigin = s, await r.decode();
    const o = ((_a2 = t.data) == null ? void 0 : _a2.width) ?? r.width, a = ((_b2 = t.data) == null ? void 0 : _b2.height) ?? r.height, l = ((_c2 = t.data) == null ? void 0 : _c2.resolution) || la(n), c = Math.ceil(o * l), h = Math.ceil(a * l), d = Rt.get().createCanvas(c, h), u = d.getContext("2d");
    u.imageSmoothingEnabled = true, u.imageSmoothingQuality = "high", u.drawImage(r, 0, 0, o * l, a * l);
    const { parseAsGraphicsContext: p, ...f } = t.data ?? {}, m = new vs({
      resource: d,
      alphaMode: "premultiply-alpha-on-upload",
      resolution: l,
      ...f
    });
    return ca(m, e, n);
  }
  async function jg(n) {
    const e = await (await Rt.get().fetch(n)).text(), s = new Me();
    return s.svg(e), s;
  }
  const Vg = `(function () {
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
  let gs = null, Io = class {
    constructor() {
      gs || (gs = URL.createObjectURL(new Blob([
        Vg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(gs);
    }
  };
  Io.revokeObjectURL = function() {
    gs && (URL.revokeObjectURL(gs), gs = null);
  };
  const Yg = `(function () {
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
  let ys = null;
  class id {
    constructor() {
      ys || (ys = URL.createObjectURL(new Blob([
        Yg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(ys);
    }
  }
  id.revokeObjectURL = function() {
    ys && (URL.revokeObjectURL(ys), ys = null);
  };
  let Ol = 0, qr;
  class Xg {
    constructor() {
      this._initialized = false, this._createdWorkers = 0, this._workerPool = [], this._queue = [], this._resolveHash = {};
    }
    isImageBitmapSupported() {
      return this._isImageBitmapSupported !== void 0 ? this._isImageBitmapSupported : (this._isImageBitmapSupported = new Promise((t) => {
        const { worker: e } = new Io();
        e.addEventListener("message", (s) => {
          e.terminate(), Io.revokeObjectURL(), t(s.data);
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
      qr === void 0 && (qr = navigator.hardwareConcurrency || 4);
      let t = this._workerPool.pop();
      return !t && this._createdWorkers < qr && (this._createdWorkers++, t = new id().worker, t.addEventListener("message", (e) => {
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
      this._resolveHash[Ol] = {
        resolve: e.resolve,
        reject: e.reject
      }, t.postMessage({
        data: e.arguments,
        uuid: Ol++,
        id: s
      });
    }
    reset() {
      this._workerPool.forEach((t) => t.terminate()), this._workerPool.length = 0, Object.values(this._resolveHash).forEach(({ reject: t }) => {
        t == null ? void 0 : t(new Error("WorkerManager has been reset before completion"));
      }), this._resolveHash = {}, this._queue.length = 0, this._initialized = false, this._createdWorkers = 0;
    }
  }
  const Nl = new Xg(), qg = [
    ".jpeg",
    ".jpg",
    ".png",
    ".webp",
    ".avif"
  ], Kg = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/avif"
  ];
  async function Jg(n, t) {
    var _a2;
    const e = await Rt.get().fetch(n);
    if (!e.ok) throw new Error(`[loadImageBitmap] Failed to fetch ${n}: ${e.status} ${e.statusText}`);
    const s = await e.blob();
    return ((_a2 = t == null ? void 0 : t.data) == null ? void 0 : _a2.alphaMode) === "premultiplied-alpha" ? createImageBitmap(s, {
      premultiplyAlpha: "none"
    }) : createImageBitmap(s);
  }
  const rd = {
    name: "loadTextures",
    id: "texture",
    extension: {
      type: ct.LoadParser,
      priority: Rn.High,
      name: "loadTextures"
    },
    config: {
      preferWorkers: true,
      preferCreateImageBitmap: true,
      crossOrigin: "anonymous"
    },
    test(n) {
      return Es(n, Kg) || ks(n, qg);
    },
    async load(n, t, e) {
      var _a2;
      let s = null;
      globalThis.createImageBitmap && this.config.preferCreateImageBitmap ? this.config.preferWorkers && await Nl.isImageBitmapSupported() ? s = await Nl.loadImageBitmap(n, t) : s = await Jg(n, t) : s = await new Promise((r, o) => {
        s = Rt.get().createImage(), s.crossOrigin = this.config.crossOrigin, s.src = n, s.complete ? r(s) : (s.onload = () => {
          r(s);
        }, s.onerror = o);
      });
      const i = new vs({
        resource: s,
        alphaMode: "premultiply-alpha-on-upload",
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || la(n),
        ...t.data
      });
      return ca(i, e, n);
    },
    unload(n) {
      n.destroy(true);
    }
  }, Zg = [
    ".mp4",
    ".m4v",
    ".webm",
    ".ogg",
    ".ogv",
    ".h264",
    ".avi",
    ".mov"
  ];
  let Kr, Jr;
  function Qg(n, t, e) {
    e === void 0 && !t.startsWith("data:") ? n.crossOrigin = ey(t) : e !== false && (n.crossOrigin = typeof e == "string" ? e : "anonymous");
  }
  function ty(n) {
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
  function ey(n, t = globalThis.location) {
    if (n.startsWith("data:")) return "";
    t || (t = globalThis.location);
    const e = new URL(n, document.baseURI);
    return e.hostname !== t.hostname || e.port !== t.port || e.protocol !== t.protocol ? "anonymous" : "";
  }
  function ny() {
    const n = [], t = [];
    for (const e of Zg) {
      const s = Ys.MIME_TYPES[e.substring(1)] || `video/${e.substring(1)}`;
      dr(s) && (n.push(e), t.includes(s) || t.push(s));
    }
    return {
      validVideoExtensions: n,
      validVideoMime: t
    };
  }
  const sy = {
    name: "loadVideo",
    id: "video",
    extension: {
      type: ct.LoadParser,
      name: "loadVideo"
    },
    test(n) {
      if (!Kr || !Jr) {
        const { validVideoExtensions: s, validVideoMime: i } = ny();
        Kr = s, Jr = i;
      }
      const t = Es(n, Jr), e = ks(n, Kr);
      return t || e;
    },
    async load(n, t, e) {
      var _a2, _b2;
      const s = {
        ...Ys.defaultOptions,
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || la(n),
        alphaMode: ((_b2 = t.data) == null ? void 0 : _b2.alphaMode) || await mh(),
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
      }), s.muted === true && (i.muted = true), Qg(i, n, s.crossorigin);
      const o = document.createElement("source");
      let a;
      if (s.mime) a = s.mime;
      else if (n.startsWith("data:")) a = n.slice(5, n.indexOf(";"));
      else if (!n.startsWith("blob:")) {
        const l = n.split("?")[0].slice(n.lastIndexOf(".") + 1).toLowerCase();
        a = Ys.MIME_TYPES[l] || `video/${l}`;
      }
      return o.src = n, a && (o.type = a), new Promise((l, c) => {
        s.preload && !s.autoPlay && i.load(), i.addEventListener("canplay", h), i.addEventListener("error", d), o.addEventListener("error", d), i.appendChild(o);
        async function h() {
          const p = new Ys({
            ...s,
            resource: i
          });
          u(), t.data.preload && await ty(i), l(ca(p, e, n));
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
  }, od = {
    extension: {
      type: ct.ResolveParser,
      name: "resolveTexture"
    },
    test: rd.test,
    parse: (n) => {
      var _a2;
      return {
        resolution: parseFloat(((_a2 = As.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
        format: n.split(".").pop(),
        src: n
      };
    }
  }, iy = {
    extension: {
      type: ct.ResolveParser,
      priority: -2,
      name: "resolveJson"
    },
    test: (n) => As.RETINA_PREFIX.test(n) && n.endsWith(".json"),
    parse: od.parse
  };
  class ry {
    constructor() {
      this._detections = [], this._initialized = false, this.resolver = new As(), this.loader = new If(), this.cache = se, this._backgroundLoader = new vf(this.loader), this._backgroundLoader.active = true, this.reset();
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
      const s = Xi(t), i = ze(t).map((a) => {
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
      if (typeof t == "string") return se.get(t);
      const e = {};
      for (let s = 0; s < t.length; s++) e[s] = se.get(t[s]);
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
        }), se.set(l, a);
      }), r;
    }
    async unload(t) {
      this._initialized || await this.init();
      const e = ze(t).map((i) => typeof i != "string" ? i.src : i), s = this.resolver.resolve(e);
      await this._unloadFromResolved(s);
    }
    async unloadBundle(t) {
      this._initialized || await this.init(), t = ze(t);
      const e = this.resolver.resolveBundle(t), s = Object.keys(e).map((i) => this._unloadFromResolved(e[i]));
      await Promise.all(s);
    }
    async _unloadFromResolved(t) {
      const e = Object.values(t);
      e.forEach((s) => {
        se.remove(s.src);
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
  const He = new ry();
  zt.handleByList(ct.LoadParser, He.loader.parsers).handleByList(ct.ResolveParser, He.resolver.parsers).handleByList(ct.CacheParser, He.cache.parsers).handleByList(ct.DetectionParser, He.detections);
  zt.add(Cf, Tf, Sf, Pf, Ef, kf, Mf, $f, Nf, jf, Hg, rd, sy, wf, _f, od, iy);
  const Fl = {
    loader: ct.LoadParser,
    resolver: ct.ResolveParser,
    cache: ct.CacheParser,
    detection: ct.DetectionParser
  };
  zt.handle(ct.Asset, (n) => {
    const t = n.ref;
    Object.entries(Fl).filter(([e]) => !!t[e]).forEach(([e, s]) => zt.add(Object.assign(t[e], {
      extension: t[e].extension ?? s
    })));
  }, (n) => {
    const t = n.ref;
    Object.keys(Fl).filter((e) => !!t[e]).forEach((e) => zt.remove(t[e]));
  });
  class oy {
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
  class ay {
    constructor() {
      this.instructions = new qo();
    }
    init() {
      this.instructions.reset();
    }
    destroy() {
      this.instructions.destroy(), this.instructions = null;
    }
  }
  const ha = class Ro {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Ms({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      Ro.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? Ro.defaultOptions.bezierSmoothness;
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
      const e = new ay(), s = this.getGpuContext(t);
      return s.graphicsData = e, e.init(), e;
    }
    _initContext(t) {
      const e = new oy();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  ha.extension = {
    type: [
      ct.CanvasSystem
    ],
    name: "graphicsContext"
  };
  ha.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let ly = ha;
  class ad {
    constructor(t, e) {
      this.state = Ki.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Ms({
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
  ad.extension = {
    type: [
      ct.CanvasPipes
    ],
    name: "graphics"
  };
  ld = function(n, t, e) {
    const s = (n >> 24 & 255) / 255;
    t[e++] = (n & 255) / 255 * s, t[e++] = (n >> 8 & 255) / 255 * s, t[e++] = (n >> 16 & 255) / 255 * s, t[e++] = s;
  };
  class cy {
    constructor() {
      this.batches = [], this.batched = false;
    }
    destroy() {
      this.batches.forEach((t) => {
        ve.return(t);
      }), this.batches.length = 0;
    }
  }
  class cd {
    constructor(t, e) {
      this.state = Ki.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Ms({
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
      o.uTransformMatrix = t.groupTransform, o.uRound = e._roundPixels | t._roundPixels, ld(t.groupColorAlpha, o.uColor, 0), this._adaptor.execute(this, t);
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
      const e = new cy();
      return t._gpuData[this.renderer.uid] = e, this._managedGraphics.add(t), e;
    }
    _updateBatchesForRenderable(t, e) {
      const s = t.context, r = this.renderer.graphicsContext.getGpuContext(s), o = this.renderer._roundPixels | t._roundPixels;
      e.batches = r.batches.map((a) => {
        const l = ve.get(sa);
        return a.copyTo(l), l.renderable = t, l.roundPixels = o, l;
      });
    }
    destroy() {
      this._managedGraphics.destroy(), this.renderer = null, this._adaptor.destroy(), this._adaptor = null, this.state = null;
    }
  }
  cd.extension = {
    type: [
      ct.WebGLPipes,
      ct.WebGPUPipes
    ],
    name: "graphics"
  };
  zt.add(ad);
  zt.add(cd);
  zt.add(ly);
  zt.add(oa);
  ft = class extends lr {
    constructor(t) {
      t instanceof Me && (t = {
        context: t
      });
      const { context: e, roundPixels: s, ...i } = t || {};
      super({
        label: "Graphics",
        ...i
      }), this.renderPipeId = "graphics", e ? this.context = e : (this.context = this._ownedContext = new Me(), this.context.autoGarbageCollect = this.autoGarbageCollect), this.didViewUpdate = true, this.allowChildren = false, this.roundPixels = s ?? false;
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
      It(ee, "Graphics#lineStyle is no longer needed. Use Graphics#setStrokeStyle to set the stroke style.");
      const i = {};
      return t && (i.width = t), e && (i.color = e), s && (i.alpha = s), this.context.strokeStyle = i, this;
    }
    beginFill(t, e) {
      It(ee, "Graphics#beginFill is no longer needed. Use Graphics#fill to fill the shape with the desired style.");
      const s = {};
      return t !== void 0 && (s.color = t), e !== void 0 && (s.alpha = e), this.context.fillStyle = s, this;
    }
    endFill() {
      It(ee, "Graphics#endFill is no longer needed. Use Graphics#fill to fill the shape with the desired style."), this.context.fill();
      const t = this.context.strokeStyle;
      return (t.width !== Me.defaultStrokeStyle.width || t.color !== Me.defaultStrokeStyle.color || t.alpha !== Me.defaultStrokeStyle.alpha) && this.context.stroke(), this;
    }
    drawCircle(...t) {
      return It(ee, "Graphics#drawCircle has been renamed to Graphics#circle"), this._callContextMethod("circle", t);
    }
    drawEllipse(...t) {
      return It(ee, "Graphics#drawEllipse has been renamed to Graphics#ellipse"), this._callContextMethod("ellipse", t);
    }
    drawPolygon(...t) {
      return It(ee, "Graphics#drawPolygon has been renamed to Graphics#poly"), this._callContextMethod("poly", t);
    }
    drawRect(...t) {
      return It(ee, "Graphics#drawRect has been renamed to Graphics#rect"), this._callContextMethod("rect", t);
    }
    drawRoundedRect(...t) {
      return It(ee, "Graphics#drawRoundedRect has been renamed to Graphics#roundRect"), this._callContextMethod("roundRect", t);
    }
    drawStar(...t) {
      return It(ee, "Graphics#drawStar has been renamed to Graphics#star"), this._callContextMethod("star", t);
    }
  };
  let rs;
  function Wl(n) {
    const t = Rt.get().createCanvas(6, 1), e = t.getContext("2d");
    return e.fillStyle = n, e.fillRect(0, 0, 6, 1), t;
  }
  hy = function() {
    if (rs !== void 0) return rs;
    try {
      const n = Wl("#ff00ff"), t = Wl("#ffff00"), s = Rt.get().createCanvas(6, 1).getContext("2d");
      s.globalCompositeOperation = "multiply", s.drawImage(n, 0, 0), s.drawImage(t, 2, 0);
      const i = s.getImageData(2, 0, 1, 1);
      if (!i) rs = false;
      else {
        const r = i.data;
        rs = r[0] === 255 && r[1] === 0 && r[2] === 0;
      }
    } catch {
      rs = false;
    }
    return rs;
  };
  Kt = {
    canvas: null,
    convertTintToImage: false,
    cacheStepsPerColorChannel: 8,
    canUseMultiply: hy(),
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
        const l = Rt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d"), h = c.createImageData(t.pixelWidth, t.pixelHeight), d = h.data, u = e instanceof ArrayBuffer ? new Uint8Array(e) : new Uint8Array(e.buffer, e.byteOffset, e.byteLength);
        if (t.format === "bgra8unorm") for (let p = 0; p < d.length && p + 3 < u.length; p += 4) d[p] = u[p + 2], d[p + 1] = u[p + 1], d[p + 2] = u[p], d[p + 3] = u[p + 3];
        else d.set(u.subarray(0, d.length));
        return c.putImageData(h, 0, 0), Kt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      if (s) {
        const a = Rt.get().createCanvas(t.pixelWidth, t.pixelHeight), l = a.getContext("2d", {
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
        const l = Rt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d");
        return l.width = t.pixelWidth, l.height = t.pixelHeight, c.drawImage(e, 0, 0), Kt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      return e;
    },
    getTintedCanvas: (n, t) => {
      const e = n.texture, s = Yt.shared.setValue(t).toHex(), i = e.tintCache || (e.tintCache = {}), r = i[s], o = e.source._resourceId;
      if ((r == null ? void 0 : r.tintId) === o) return r;
      const a = r && "getContext" in r ? r : Rt.get().createCanvas();
      return Kt.tintMethod(e, t, a), a.tintId = o, i[s] = a, i[s];
    },
    getTintedPattern: (n, t) => {
      const e = Yt.shared.setValue(t).toHex(), s = n.patternCache || (n.patternCache = {}), i = n.source._resourceId;
      let r = s[e];
      return (r == null ? void 0 : r.tintId) === i || (Kt.canvas || (Kt.canvas = Rt.get().createCanvas()), Kt.tintMethod(n, t, Kt.canvas), r = Kt.canvas.getContext("2d").createPattern(Kt.canvas, "repeat"), r.tintId = i, s[e] = r), r;
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
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.fillStyle = Yt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "multiply";
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
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy", s.fillStyle = Yt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "destination-atop";
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
  class dy extends lr {
    constructor(t, e) {
      const { text: s, resolution: i, style: r, anchor: o, width: a, height: l, roundPixels: c, ...h } = t;
      super({
        ...h
      }), this.batched = true, this._resolution = null, this._autoResolution = true, this._didTextUpdate = true, this._styleClass = e, this.text = s ?? "", this.style = r, this.resolution = i ?? null, this.allowChildren = false, this._anchor = new pe({
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
  function uy(n, t) {
    let e = n[0] ?? {};
    return (typeof e == "string" || n[1]) && (It(ee, `use new ${t}({ text: "hi!", style }) instead`), e = {
      text: e,
      style: n[1]
    }), e;
  }
  class py {
    constructor(t) {
      this._canvasPool = /* @__PURE__ */ Object.create(null), this.canvasOptions = t || {}, this.enableFullScreen = false;
    }
    _createCanvasAndContext(t, e) {
      const s = Rt.get().createCanvas();
      s.width = t, s.height = e;
      const i = s.getContext("2d");
      return {
        canvas: s,
        context: i
      };
    }
    getOptimalCanvasAndContext(t, e, s = 1) {
      t = Math.ceil(t * s - 1e-6), e = Math.ceil(e * s - 1e-6), t = _s(t), e = _s(e);
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
  Lo = new py();
  hi.register(Lo);
  let On = null, pn = null;
  function fy(n, t) {
    On || (On = Rt.get().createCanvas(256, 128), pn = On.getContext("2d", {
      willReadFrequently: true
    }), pn.globalCompositeOperation = "copy", pn.globalAlpha = 1), (On.width < n || On.height < t) && (On.width = _s(n), On.height = _s(t));
  }
  function Gl(n, t, e) {
    for (let s = 0, i = 4 * e * t; s < t; ++s, i += 4) if (n[i + 3] !== 0) return false;
    return true;
  }
  function zl(n, t, e, s, i) {
    const r = 4 * t;
    for (let o = s, a = s * r + 4 * e; o <= i; ++o, a += r) if (n[a + 3] !== 0) return false;
    return true;
  }
  function my(...n) {
    let t = n[0];
    t.canvas || (t = {
      canvas: n[0],
      resolution: n[1]
    });
    const { canvas: e } = t, s = Math.min(t.resolution ?? 1, 1), i = t.width ?? e.width, r = t.height ?? e.height;
    let o = t.output;
    if (fy(i, r), !pn) throw new TypeError("Failed to get canvas 2D context");
    pn.drawImage(e, 0, 0, i, r, 0, 0, i * s, r * s);
    const l = pn.getImageData(0, 0, i, r).data;
    let c = 0, h = 0, d = i - 1, u = r - 1;
    for (; h < r && Gl(l, i, h); ) ++h;
    if (h === r) return Ht.EMPTY;
    for (; Gl(l, i, u); ) --u;
    for (; zl(l, i, c, h, u); ) ++c;
    for (; zl(l, i, d, h, u); ) --d;
    return ++d, ++u, pn.globalCompositeOperation = "source-over", pn.strokeRect(c, h, d - c, u - h), pn.globalCompositeOperation = "copy", o ?? (o = new Ht()), o.set(c / s, h / s, (d - c) / s, (u - h) / s), o;
  }
  class gy {
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
  yy = function(n = 1e3, t = 0, e = false) {
    if (isNaN(n) || n < 0) throw new TypeError("Invalid max value");
    if (isNaN(t) || t < 0) throw new TypeError("Invalid ttl value");
    if (typeof e != "boolean") throw new TypeError("Invalid resetTtl value");
    return new gy(n, t, e);
  };
  function hd(n) {
    return !!n.tagStyles && Object.keys(n.tagStyles).length > 0;
  }
  function dd(n) {
    return n.includes("<");
  }
  function xy(n, t) {
    return n.clone().assign(t);
  }
  function by(n, t) {
    const e = [], s = t.tagStyles;
    if (!hd(t) || !dd(n)) return e.push({
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
            const u = i[i.length - 1], p = xy(u, s[d]);
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
  const _y = [
    10,
    13
  ], wy = new Set(_y), vy = [
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
  ], Cy = new Set(vy), Sy = [
    9,
    32
  ], Ty = new Set(Sy), Ay = [
    45,
    8208,
    8211,
    8212,
    173
  ], Ey = new Set(Ay), ky = /(\r\n|\r|\n)/, My = /(?:\r\n|\r|\n)/;
  function Qi(n) {
    return typeof n != "string" ? false : wy.has(n.charCodeAt(0));
  }
  Ie = function(n, t) {
    return typeof n != "string" ? false : Cy.has(n.charCodeAt(0));
  };
  hv = function(n) {
    return typeof n != "string" ? false : Ty.has(n.charCodeAt(0));
  };
  Py = function(n) {
    return typeof n != "string" ? false : Ey.has(n.charCodeAt(0));
  };
  ud = function(n) {
    return n === "normal" || n === "pre-line";
  };
  pd = function(n) {
    return n === "normal";
  };
  function hn(n) {
    if (typeof n != "string") return "";
    let t = n.length - 1;
    for (; t >= 0 && Ie(n[t]); ) t--;
    return t < n.length - 1 ? n.slice(0, t + 1) : n;
  }
  function fd(n) {
    const t = [], e = [];
    if (typeof n != "string") return t;
    for (let s = 0; s < n.length; s++) {
      const i = n[s], r = n[s + 1];
      if (Ie(i) || Qi(i)) {
        e.length > 0 && (t.push(e.join("")), e.length = 0), i === "\r" && r === `
` ? (t.push(`\r
`), s++) : t.push(i);
        continue;
      }
      e.push(i), Py(i) && r && !Ie(r) && !Qi(r) && (t.push(e.join("")), e.length = 0);
    }
    return e.length > 0 && t.push(e.join("")), t;
  }
  function md(n, t, e, s) {
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
  const Iy = /\r\n|\r|\n/g;
  function Ry(n, t, e, s, i, r, o, a) {
    var _a2, _b2;
    const l = by(n, t);
    if (pd(t.whiteSpace)) for (let B = 0; B < l.length; B++) {
      const L = l[B];
      l[B] = {
        text: L.text.replace(Iy, " "),
        style: L.style
      };
    }
    const h = [];
    let d = [];
    for (const B of l) {
      const L = B.text.split(ky);
      for (let D = 0; D < L.length; D++) {
        const K = L[D];
        K === `\r
` || K === "\r" || K === `
` ? (h.push(d), d = []) : K.length > 0 && d.push({
          text: K,
          style: B.style
        });
      }
    }
    (d.length > 0 || h.length === 0) && h.push(d);
    const u = e ? Ly(h, t, s, i, o, a) : h, p = [], f = [], m = [], g = [], y = [];
    let w = 0;
    const x = t._fontString, b = r(x);
    b.fontSize === 0 && (b.fontSize = t.fontSize, b.ascent = t.fontSize);
    let _ = "", v = !!t.dropShadow, C = ((_a2 = t._stroke) == null ? void 0 : _a2.width) || 0;
    for (const B of u) {
      let L = 0, D = b.ascent, K = b.descent, X = "";
      for (const M of B) {
        const N = M.style._fontString, W = r(N);
        N !== _ && (s.font = N, _ = N);
        const z = i(M.text, M.style.letterSpacing, s);
        L += z, D = Math.max(D, W.ascent), K = Math.max(K, W.descent), X += M.text;
        const q = ((_b2 = M.style._stroke) == null ? void 0 : _b2.width) || 0;
        q > C && (C = q), !v && M.style.dropShadow && (v = true);
      }
      B.length === 0 && (D = b.ascent, K = b.descent), p.push(L), f.push(D), m.push(K), y.push(X);
      const Z = t.lineHeight || D + K;
      g.push(Z + t.leading), w = Math.max(w, L);
    }
    const E = C, T = (e && t.align !== "left" ? Math.max(w, t.wordWrapWidth) : w) + E + (t.dropShadow ? t.dropShadow.distance : 0);
    let O = 0;
    for (let B = 0; B < g.length; B++) O += g[B];
    O = Math.max(O, g[0] + E);
    const U = O + (t.dropShadow ? t.dropShadow.distance : 0), G = t.lineHeight || b.fontSize;
    return {
      width: T,
      height: U,
      lines: y,
      lineWidths: p,
      lineHeight: G + t.leading,
      maxLineWidth: w,
      fontProperties: b,
      runsByLine: u,
      lineAscents: f,
      lineDescents: m,
      lineHeights: g,
      hasDropShadow: v
    };
  }
  function Ly(n, t, e, s, i, r) {
    var _a2;
    const { letterSpacing: o, whiteSpace: a, wordWrapWidth: l, breakWords: c } = t, h = ud(a), d = l + o, u = {};
    let p = "";
    const f = (g, y) => {
      const w = `${g}|${y.styleKey}`;
      let x = u[w];
      if (x === void 0) {
        const b = y._fontString;
        b !== p && (e.font = b, p = b), x = s(g, y.letterSpacing, e) + y.letterSpacing, u[w] = x;
      }
      return x;
    }, m = [];
    for (const g of n) {
      const y = $y(g), w = m.length, x = (T) => {
        let O = 0, U = T;
        do {
          const { token: G, style: B } = y[U];
          O += f(G, B), U++;
        } while (U < y.length && y[U].continuesFromPrevious);
        return O;
      }, b = (T) => {
        const O = [];
        let U = T;
        do
          O.push({
            token: y[U].token,
            style: y[U].style
          }), U++;
        while (U < y.length && y[U].continuesFromPrevious);
        return O;
      };
      let _ = [], v = 0, C = !h, E = null;
      const R = () => {
        E && E.text.length > 0 && _.push(E), E = null;
      }, k = () => {
        if (R(), _.length > 0) {
          const T = _[_.length - 1];
          T.text = hn(T.text), T.text.length === 0 && _.pop();
        }
        m.push(_), _ = [], v = 0, C = false;
      };
      for (let T = 0; T < y.length; T++) {
        const { token: O, style: U, continuesFromPrevious: G } = y[T], B = f(O, U);
        if (h) {
          const K = Ie(O), X = (E == null ? void 0 : E.text[E.text.length - 1]) ?? ((_a2 = _[_.length - 1]) == null ? void 0 : _a2.text.slice(-1)) ?? "", Z = X ? Ie(X) : false;
          if (K && Z) continue;
        }
        const L = !G, D = L ? x(T) : B;
        if (D > d && L) if (v > 0 && k(), c) {
          const K = b(T);
          for (let X = 0; X < K.length; X++) {
            const Z = K[X].token, M = K[X].style, N = md(Z, c, r, i);
            for (const W of N) {
              const z = f(W, M);
              z + v > d && k(), !E || E.style !== M ? (R(), E = {
                text: W,
                style: M
              }) : E.text += W, v += z;
            }
          }
          T += K.length - 1;
        } else {
          const K = b(T);
          R(), m.push(K.map((X) => ({
            text: X.token,
            style: X.style
          }))), C = false, T += K.length - 1;
        }
        else if (D + v > d && L) {
          if (Ie(O)) {
            C = false;
            continue;
          }
          k(), E = {
            text: O,
            style: U
          }, v = B;
        } else if (G && !c) !E || E.style !== U ? (R(), E = {
          text: O,
          style: U
        }) : E.text += O, v += B;
        else {
          const K = Ie(O);
          if (v === 0 && K && !C) continue;
          !E || E.style !== U ? (R(), E = {
            text: O,
            style: U
          }) : E.text += O, v += B;
        }
      }
      if (R(), _.length > 0) {
        const T = _[_.length - 1];
        T.text = hn(T.text), T.text.length === 0 && _.pop();
      }
      (_.length > 0 || m.length === w) && m.push(_);
    }
    return m;
  }
  function $y(n) {
    const t = [];
    let e = false;
    for (const s of n) {
      const i = fd(s.text);
      let r = true;
      for (const o of i) {
        const a = Ie(o) || Qi(o), l = r && e && !a;
        t.push({
          token: o,
          style: s.style,
          continuesFromPrevious: l
        }), e = !a, r = false;
      }
    }
    return t;
  }
  const By = {
    willReadFrequently: true
  };
  function Dl(n, t, e, s, i) {
    let r = e[n];
    return typeof r != "number" && (r = i(n, t, s) + t, e[n] = r), r;
  }
  function Oy(n, t, e, s, i, r, o) {
    const a = e.getContext("2d", By);
    a.font = t._fontString;
    let l = 0, c = "";
    const h = [], d = /* @__PURE__ */ Object.create(null), { letterSpacing: u, whiteSpace: p } = t, f = ud(p), m = pd(p);
    let g = !f;
    const y = t.wordWrapWidth + u, w = fd(n);
    for (let b = 0; b < w.length; b++) {
      let _ = w[b];
      if (Qi(_)) {
        if (!m) {
          h.push(hn(c)), g = !f, c = "", l = 0;
          continue;
        }
        _ = " ";
      }
      if (f) {
        const C = Ie(_), E = Ie(c[c.length - 1]);
        if (C && E) continue;
      }
      const v = Dl(_, u, d, a, s);
      if (v > y) if (c !== "" && (h.push(hn(c)), c = "", l = 0), i(_, t.breakWords)) {
        const C = md(_, t.breakWords, o, r);
        for (const E of C) {
          const R = Dl(E, u, d, a, s);
          R + l > y && (h.push(hn(c)), g = false, c = "", l = 0), c += E, l += R;
        }
      } else c.length > 0 && (h.push(hn(c)), c = "", l = 0), h.push(hn(_)), g = false, c = "", l = 0;
      else v + l > y && (g = false, h.push(hn(c)), c = "", l = 0), (c.length > 0 || !Ie(_) || g) && (c += _, l += v);
    }
    const x = hn(c);
    return x.length > 0 && h.push(x), h.join(`
`);
  }
  const Hl = {
    willReadFrequently: true
  }, _n = class xt {
    static get experimentalLetterSpacingSupported() {
      let t = xt._experimentalLetterSpacingSupported;
      if (t === void 0) {
        const e = Rt.get().getCanvasRenderingContext2D().prototype;
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
      if (hd(e) && dd(t)) {
        const _ = Ry(t, e, i, xt._context, xt._measureText, xt.measureFont, xt.canBreakChars, xt.wordWrapSplit), v = new xt(t, e, _.width, _.height, _.lines, _.lineWidths, _.lineHeight, _.maxLineWidth, _.fontProperties, {
          runsByLine: _.runsByLine,
          lineAscents: _.lineAscents,
          lineDescents: _.lineDescents,
          lineHeights: _.lineHeights,
          hasDropShadow: _.hasDropShadow
        });
        return xt._measurementCache.set(r, v), v;
      }
      const a = e._fontString, l = xt.measureFont(a);
      l.fontSize === 0 && (l.fontSize = e.fontSize, l.ascent = e.fontSize, l.descent = 0);
      const c = xt._context;
      c.font = a;
      const d = (i ? xt._wordWrap(t, e, s) : t).split(My), u = new Array(d.length);
      let p = 0;
      for (let _ = 0; _ < d.length; _++) {
        const v = xt._measureText(d[_], e.letterSpacing, c);
        u[_] = v, p = Math.max(p, v);
      }
      const f = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0, m = e.lineHeight || l.fontSize, g = xt._getAlignWidth(p, e, i), y = xt._adjustWidthForStyle(g, e), w = Math.max(m, l.fontSize + f) + (d.length - 1) * (m + e.leading), x = xt._adjustHeightForStyle(w, e), b = new xt(t, e, y, x, d, u, m + e.leading, p, l);
      return xt._measurementCache.set(r, b), b;
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
      return Oy(t, e, s, xt._measureText, xt.canBreakWords, xt.canBreakChars, xt.wordWrapSplit);
    }
    static isBreakingSpace(t, e) {
      return Ie(t);
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
          if ((_a2 = e.getContext("2d", Hl)) == null ? void 0 : _a2.measureText) return xt.__canvas = e, e;
          t = Rt.get().createCanvas();
        } catch {
          t = Rt.get().createCanvas();
        }
        t.width = t.height = 10, xt.__canvas = t;
      }
      return xt.__canvas;
    }
    static get _context() {
      return xt.__context || (xt.__context = xt._canvas.getContext("2d", Hl)), xt.__context;
    }
  };
  _n.METRICS_STRING = "|\xC9q\xC5";
  _n.BASELINE_SYMBOL = "M";
  _n.BASELINE_MULTIPLIER = 1.4;
  _n.HEIGHT_MULTIPLIER = 2;
  _n.graphemeSegmenter = (() => {
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
  _n.experimentalLetterSpacing = false;
  _n._fonts = {};
  _n._measurementCache = yy(1e3);
  dn = _n;
  const Ny = [
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui"
  ];
  $o = function(n) {
    const t = typeof n.fontSize == "number" ? `${n.fontSize}px` : n.fontSize;
    let e = n.fontFamily;
    Array.isArray(n.fontFamily) || (e = n.fontFamily.split(","));
    for (let s = e.length - 1; s >= 0; s--) {
      let i = e[s].trim();
      !/([\"\'])[^\'\"]+\1/.test(i) && !Ny.includes(i) && (i = `"${i}"`), e[s] = i;
    }
    return `${n.fontStyle} ${n.fontVariant} ${n.fontWeight} ${t} ${e.join(",")}`;
  };
  const Ul = 1e5;
  Mi = function(n, t, e, s = 0, i = 0, r = 0) {
    if (n.texture === Ct.WHITE && !n.fill) return Yt.shared.setValue(n.color).setAlpha(n.alpha ?? 1).toHexa();
    if (n.fill) {
      if (n.fill instanceof pr) {
        const o = n.fill, a = t.createPattern(o.texture.source.resource, "repeat"), l = o.transform.copyTo(vt.shared);
        return l.scale(o.texture.source.pixelWidth, o.texture.source.pixelHeight), a.setTransform(l), a;
      } else if (n.fill instanceof bn) {
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
              y = Math.max(0, Math.min(1, y)), d.addColorStop(Math.floor(y * Ul) / Ul, Yt.shared.setValue(g.color).toHex());
            });
          }
        } else o.colorStops.forEach((p) => {
          d.addColorStop(p.offset, Yt.shared.setValue(p.color).toHex());
        });
        return d;
      }
    } else {
      const o = t.createPattern(n.texture.source.resource, "repeat"), a = n.matrix.copyTo(vt.shared);
      return a.scale(n.texture.source.pixelWidth, n.texture.source.pixelHeight), o.setTransform(a), o;
    }
    return Jt("FillStyle not recognised", n), "red";
  };
  const jl = new Ht();
  function os(n) {
    let t = 0;
    for (let e = 0; e < n.length; e++) n.charCodeAt(e) === 32 && t++;
    return t;
  }
  class Fy {
    getCanvasAndContext(t) {
      const { text: e, style: s, resolution: i = 1 } = t, r = s._getFinalPadding(), o = dn.measureText(e || " ", s), a = Math.ceil(Math.ceil(Math.max(1, o.width) + r * 2) * i), l = Math.ceil(Math.ceil(Math.max(1, o.height) + r * 2) * i), c = Lo.getOptimalCanvasAndContext(a, l);
      this._renderTextToCanvas(s, r, i, c, o);
      const h = s.trim ? my({
        canvas: c.canvas,
        width: a,
        height: l,
        resolution: 1,
        output: jl
      }) : jl.set(0, 0, a, l);
      return {
        canvasAndContext: c,
        frame: h
      };
    }
    returnCanvasAndContext(t) {
      Lo.returnCanvasAndContext(t);
    }
    _renderTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      if (r.runsByLine && r.runsByLine.length > 0) {
        this._renderTaggedTextToCanvas(r, t, e, s, i);
        return;
      }
      const { canvas: o, context: a } = i, l = $o(t), c = r.lines, h = r.lineHeight, d = r.lineWidths, u = r.maxLineWidth, p = r.fontProperties, f = o.height;
      if (a.resetTransform(), a.scale(s, s), a.textBaseline = t.textBaseline, (_a2 = t._stroke) == null ? void 0 : _a2.width) {
        const v = t._stroke;
        a.lineWidth = v.width, a.miterLimit = v.miterLimit, a.lineJoin = v.join, a.lineCap = v.cap;
      }
      a.font = l;
      let m, g;
      const y = t.dropShadow ? 2 : 1, w = t.wordWrap ? Math.max(t.wordWrapWidth, u) : u, b = (((_b2 = t._stroke) == null ? void 0 : _b2.width) ?? 0) / 2;
      let _ = (h - p.fontSize) / 2;
      h - p.fontSize < 0 && (_ = 0);
      for (let v = 0; v < y; ++v) {
        const C = t.dropShadow && v === 0, E = C ? Math.ceil(Math.max(1, f) + e * 2) : 0, R = E * s;
        if (C) this._setupDropShadow(a, t, s, R);
        else {
          const k = t._gradientBounds, T = t._gradientOffset;
          if (k) {
            const O = {
              width: k.width,
              height: k.height,
              lineHeight: k.height,
              lines: r.lines
            };
            this._setFillAndStrokeStyles(a, t, O, e, b, (T == null ? void 0 : T.x) ?? 0, (T == null ? void 0 : T.y) ?? 0);
          } else T ? this._setFillAndStrokeStyles(a, t, r, e, b, T.x, T.y) : this._setFillAndStrokeStyles(a, t, r, e, b);
          a.shadowColor = "rgba(0,0,0,0)";
        }
        for (let k = 0; k < c.length; k++) {
          m = b, g = b + k * h + p.ascent + _, m += this._getAlignmentOffset(d[k], w, t.align);
          let T = 0;
          if (t.align === "justify" && t.wordWrap && k < c.length - 1) {
            const O = os(c[k]);
            O > 0 && (T = (w - d[k]) / O);
          }
          ((_c2 = t._stroke) == null ? void 0 : _c2.width) && this._drawLetterSpacing(c[k], t, i, m + e, g + e - E, true, T), t._fill !== void 0 && this._drawLetterSpacing(c[k], t, i, m + e, g + e - E, false, T);
        }
      }
    }
    _renderTaggedTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      const { canvas: o, context: a } = r, { runsByLine: l, lineWidths: c, maxLineWidth: h, lineAscents: d, lineHeights: u, hasDropShadow: p } = t, f = o.height;
      a.resetTransform(), a.scale(i, i), a.textBaseline = e.textBaseline;
      const m = p ? 2 : 1, g = e.wordWrap ? Math.max(e.wordWrapWidth, h) : h;
      let y = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0;
      for (const b of l) for (const _ of b) {
        const v = ((_b2 = _.style._stroke) == null ? void 0 : _b2.width) ?? 0;
        v > y && (y = v);
      }
      const w = y / 2, x = [];
      for (let b = 0; b < l.length; b++) {
        const _ = l[b], v = [];
        for (const C of _) {
          const E = $o(C.style);
          a.font = E, v.push({
            width: dn._measureText(C.text, C.style.letterSpacing, a),
            font: E
          });
        }
        x.push(v);
      }
      for (let b = 0; b < m; ++b) {
        const _ = p && b === 0, v = _ ? Math.ceil(Math.max(1, f) + s * 2) : 0, C = v * i;
        _ || (a.shadowColor = "rgba(0,0,0,0)");
        let E = w;
        for (let R = 0; R < l.length; R++) {
          const k = l[R], T = c[R], O = d[R], U = u[R], G = x[R];
          let B = w;
          B += this._getAlignmentOffset(T, g, e.align);
          let L = 0;
          if (e.align === "justify" && e.wordWrap && R < l.length - 1) {
            let X = 0;
            for (const Z of k) X += os(Z.text);
            X > 0 && (L = (g - T) / X);
          }
          const D = E + O;
          let K = B + s;
          for (let X = 0; X < k.length; X++) {
            const Z = k[X], { width: M, font: N } = G[X];
            if (a.font = N, a.textBaseline = Z.style.textBaseline, (_c2 = Z.style._stroke) == null ? void 0 : _c2.width) {
              const z = Z.style._stroke;
              if (a.lineWidth = z.width, a.miterLimit = z.miterLimit, a.lineJoin = z.join, a.lineCap = z.cap, _) if (Z.style.dropShadow) this._setupDropShadow(a, Z.style, i, C);
              else {
                const q = os(Z.text);
                K += M + q * L;
                continue;
              }
              else {
                const q = dn.measureFont(N), Q = Z.style.lineHeight || q.fontSize, at = {
                  width: M,
                  height: Q,
                  lineHeight: Q,
                  lines: [
                    Z.text
                  ]
                };
                a.strokeStyle = Mi(z, a, at, s * 2, K - s, E);
              }
              this._drawLetterSpacing(Z.text, Z.style, r, K, D + s - v, true, L);
            }
            const W = os(Z.text);
            K += M + W * L;
          }
          K = B + s;
          for (let X = 0; X < k.length; X++) {
            const Z = k[X], { width: M, font: N } = G[X];
            if (a.font = N, a.textBaseline = Z.style.textBaseline, Z.style._fill !== void 0) {
              if (_) if (Z.style.dropShadow) this._setupDropShadow(a, Z.style, i, C);
              else {
                const z = os(Z.text);
                K += M + z * L;
                continue;
              }
              else {
                const z = dn.measureFont(N), q = Z.style.lineHeight || z.fontSize, Q = {
                  width: M,
                  height: q,
                  lineHeight: q,
                  lines: [
                    Z.text
                  ]
                };
                a.fillStyle = Mi(Z.style._fill, a, Q, s * 2, K - s, E);
              }
              this._drawLetterSpacing(Z.text, Z.style, r, K, D + s - v, false, L);
            }
            const W = os(Z.text);
            K += M + W * L;
          }
          E += U;
        }
      }
    }
    _setFillAndStrokeStyles(t, e, s, i, r, o = 0, a = 0) {
      var _a2;
      if (t.fillStyle = e._fill ? Mi(e._fill, t, s, i * 2, o, a) : null, (_a2 = e._stroke) == null ? void 0 : _a2.width) {
        const l = r + i * 2;
        t.strokeStyle = Mi(e._stroke, t, s, l, o, a);
      }
    }
    _setupDropShadow(t, e, s, i) {
      t.fillStyle = "black", t.strokeStyle = "black";
      const r = e.dropShadow, o = r.color, a = r.alpha;
      t.shadowColor = Yt.shared.setValue(o).setAlpha(a).toRgbaString();
      const l = r.blur * s, c = r.distance * s;
      t.shadowBlur = l, t.shadowOffsetX = Math.cos(r.angle) * c, t.shadowOffsetY = Math.sin(r.angle) * c + i;
    }
    _getAlignmentOffset(t, e, s) {
      return s === "right" ? e - t : s === "center" ? (e - t) / 2 : 0;
    }
    _drawLetterSpacing(t, e, s, i, r, o = false, a = 0) {
      const { context: l } = s, c = e.letterSpacing;
      let h = false;
      if (dn.experimentalLetterSpacingSupported && (dn.experimentalLetterSpacing ? (l.letterSpacing = `${c}px`, l.textLetterSpacing = `${c}px`, h = true) : (l.letterSpacing = "0px", l.textLetterSpacing = "0px")), (c === 0 || h) && a === 0) {
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
      const u = dn.graphemeSegmenter(t);
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
  const fs = new Fy(), da = class Yn extends on {
    constructor(t = {}) {
      super(), this.uid = ne("textStyle"), this._tick = 0, this._cachedFontString = null, Wy(t), t instanceof Yn && (t = t._toObject());
      const i = {
        ...Yn.defaultTextStyle,
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
        ...Yn.defaultDropShadow,
        ...t
      }) : this._dropShadow = t ? this._createProxy({
        ...Yn.defaultDropShadow
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
        ...Me.defaultFillStyle,
        ...t
      }, () => {
        this._fill = qn({
          ...this._originalFill
        }, Me.defaultFillStyle);
      })), this._fill = qn(t === 0 ? "black" : t, Me.defaultFillStyle), this.update());
    }
    get stroke() {
      return this._originalStroke;
    }
    set stroke(t) {
      t !== this._originalStroke && (this._originalStroke = t, this._isFillStyle(t) && (this._originalStroke = this._createProxy({
        ...Me.defaultStrokeStyle,
        ...t
      }, () => {
        this._stroke = Zi({
          ...this._originalStroke
        }, Me.defaultStrokeStyle);
      })), this._stroke = Zi(t, Me.defaultStrokeStyle), this.update());
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
      const t = Yn.defaultTextStyle;
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
      return this._cachedFontString === null && (this._cachedFontString = $o(this)), this._cachedFontString;
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
      return new Yn(this._toObject());
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
      return (t ?? null) !== null && !(Yt.isColorLike(t) || t instanceof bn || t instanceof pr);
    }
  };
  da.defaultDropShadow = {
    alpha: 1,
    angle: Math.PI / 6,
    blur: 0,
    color: "black",
    distance: 5
  };
  da.defaultTextStyle = {
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
  $e = da;
  function Wy(n) {
    const t = n;
    if (typeof t.dropShadow == "boolean" && t.dropShadow) {
      const e = $e.defaultDropShadow;
      n.dropShadow = {
        alpha: t.dropShadowAlpha ?? e.alpha,
        angle: t.dropShadowAngle ?? e.angle,
        blur: t.dropShadowBlur ?? e.blur,
        color: t.dropShadowColor ?? e.color,
        distance: t.dropShadowDistance ?? e.distance
      };
    }
    if (t.strokeThickness !== void 0) {
      It(ee, "strokeThickness is now a part of stroke");
      const e = t.stroke;
      let s = {};
      if (Yt.isColorLike(e)) s.color = e;
      else if (e instanceof bn || e instanceof pr) s.fill = e;
      else if (Object.hasOwnProperty.call(e, "color") || Object.hasOwnProperty.call(e, "fill")) s = e;
      else throw new Error("Invalid stroke value.");
      n.stroke = {
        ...s,
        width: t.strokeThickness
      };
    }
    if (Array.isArray(t.fillGradientStops)) {
      if (It(ee, "gradient fill is now a fill pattern: `new FillGradient(...)`"), !Array.isArray(t.fill) || t.fill.length === 0) throw new Error("Invalid fill value. Expected an array of colors for gradient fill.");
      t.fill.length !== t.fillGradientStops.length && Jt("The number of fill colors must match the number of fill gradient stops.");
      const e = new bn({
        start: {
          x: 0,
          y: 0
        },
        end: {
          x: 0,
          y: 1
        },
        textureSpace: "local"
      }), s = t.fillGradientStops.slice(), i = t.fill.map((r) => Yt.shared.setValue(r).toNumber());
      s.forEach((r, o) => {
        e.addColorStop(r, i[o]);
      }), n.fill = {
        fill: e
      };
    }
  }
  function Gy(n, t) {
    const { texture: e, bounds: s } = n, i = t._style._getFinalPadding();
    Jc(s, t._anchor, e);
    const r = t._anchor._x * i * 2, o = t._anchor._y * i * 2;
    s.minX -= i - r, s.minY -= i - o, s.maxX -= i - r, s.maxY -= i - o;
  }
  zy = class {
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
  class Dy extends zy {
  }
  class gd {
    constructor(t) {
      this._renderer = t, t.runners.resolutionChange.add(this), this._managedTexts = new Ms({
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
        (s.currentKey !== t.styleKey || t._resolution !== i) && this._updateGpuText(t), t._didTextUpdate = false, Gy(s, t);
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
      const e = new Dy();
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
  gd.extension = {
    type: [
      ct.WebGLPipes,
      ct.WebGPUPipes,
      ct.CanvasPipes
    ],
    name: "text"
  };
  const Hy = new Ee();
  function Uy(n, t, e, s, i = false) {
    const r = Hy;
    r.minX = 0, r.minY = 0, r.maxX = n.width / s | 0, r.maxY = n.height / s | 0;
    const o = ar.getOptimalTexture(r.width, r.height, s, false, i);
    return o.source.uploadMethodId = "image", o.source.resource = n, o.source.alphaMode = "premultiply-alpha-on-upload", o.frame.width = t / s, o.frame.height = e / s, o.source.emit("update", o.source), o.updateUvs(), o;
  }
  class yd {
    constructor(t, e) {
      this._activeTextures = {}, this._renderer = t, this._retainCanvasContext = e;
    }
    getTexture(t, e, s, i) {
      typeof t == "string" && (It("8.0.0", "CanvasTextSystem.getTexture: Use object TextOptions instead of separate arguments"), t = {
        text: t,
        style: s,
        resolution: e
      }), t.style instanceof $e || (t.style = new $e(t.style)), t.textureStyle instanceof Jn || (t.textureStyle = new Jn(t.textureStyle)), typeof t.text != "string" && (t.text = t.text.toString());
      const { text: r, style: o, textureStyle: a, autoGenerateMipmaps: l } = t, c = t.resolution ?? this._renderer.resolution, { frame: h, canvasAndContext: d } = fs.getCanvasAndContext({
        text: r,
        style: o,
        resolution: c
      }), u = Uy(d.canvas, h.width, h.height, c, l);
      if (a && (u.source.style = a), o.trim && (h.pad(o.padding), u.frame.copyFrom(h), u.frame.scale(1 / c), u.updateUvs()), o.filters) {
        const p = this._applyFilters(u, o.filters);
        return this.returnTexture(u), fs.returnCanvasAndContext(d), p;
      }
      return this._renderer.texture.initSource(u._source), this._retainCanvasContext || fs.returnCanvasAndContext(d), u;
    }
    returnTexture(t) {
      const e = t.source, s = e.resource;
      if (this._retainCanvasContext && (s == null ? void 0 : s.getContext)) {
        const i = s.getContext("2d");
        i && fs.returnCanvasAndContext({
          canvas: s,
          context: i
        });
      }
      e.resource = null, e.uploadMethodId = "unknown", e.alphaMode = "no-premultiply-alpha", ar.returnTexture(t, true);
    }
    renderTextToCanvas() {
      It("8.10.0", "CanvasTextSystem.renderTextToCanvas: no longer supported, use CanvasTextSystem.getTexture instead");
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
  class xd extends yd {
    constructor(t) {
      super(t, true);
    }
  }
  xd.extension = {
    type: [
      ct.CanvasSystem
    ],
    name: "canvasText"
  };
  class bd extends yd {
    constructor(t) {
      super(t, false);
    }
  }
  bd.extension = {
    type: [
      ct.WebGLSystem,
      ct.WebGPUSystem
    ],
    name: "canvasText"
  };
  zt.add(xd);
  zt.add(bd);
  zt.add(gd);
  class De extends dy {
    constructor(...t) {
      const e = uy(t, "Text");
      super(e, $e), this.renderPipeId = "text", e.textureStyle && (this.textureStyle = e.textureStyle instanceof Jn ? e.textureStyle : new Jn(e.textureStyle)), this.autoGenerateMipmaps = e.autoGenerateMipmaps ?? ke.defaultOptions.autoGenerateMipmaps;
    }
    updateBounds() {
      const t = this._bounds, e = this._anchor;
      let s = 0, i = 0;
      if (this._style.trim) {
        const { frame: r, canvasAndContext: o } = fs.getCanvasAndContext({
          text: this.text,
          style: this._style,
          resolution: 1
        });
        fs.returnCanvasAndContext(o), s = r.width, i = r.height;
      } else {
        const r = dn.measureText(this._text, this._style);
        s = r.width, i = r.height;
      }
      t.minX = -e._x * s, t.maxX = t.minX + s, t.minY = -e._y * i, t.maxY = t.minY + i;
    }
  }
  fr = class extends Ct {
    static create(t) {
      const { dynamic: e, ...s } = t;
      return new fr({
        source: new ke(s),
        dynamic: e ?? false
      });
    }
    resize(t, e, s) {
      return this.source.resize(t, e, s), this;
    }
  };
  class jy {
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
        const w = u.frame, x = u.source.resolution, b = w.x * x, _ = w.y * x, v = w.width * x, C = w.height * x;
        i.globalAlpha = f;
        const E = -d.anchorX * w.width, R = -d.anchorY * w.height;
        d.rotation !== 0 || d.scaleX !== 1 || d.scaleY !== 1 ? (i.save(), i.translate(d.x, d.y), i.rotate(d.rotation), i.scale(d.scaleX, d.scaleY), i.drawImage(y, b, _, v, C, E, R, w.width, w.height), i.restore()) : i.drawImage(y, b, _, v, C, d.x + E, d.y + R, w.width, w.height);
      }
      i.restore();
    }
  }
  function Vl(n, t = null) {
    const e = n * 6;
    if (e > 65535 ? t || (t = new Uint32Array(e)) : t || (t = new Uint16Array(e)), t.length !== e) throw new Error(`Out buffer length is incorrect, got ${t.length} and expected ${e}`);
    for (let s = 0, i = 0; s < e; s += 6, i += 4) t[s + 0] = i + 0, t[s + 1] = i + 1, t[s + 2] = i + 2, t[s + 3] = i + 0, t[s + 4] = i + 2, t[s + 5] = i + 3;
    return t;
  }
  function Vy(n) {
    return {
      dynamicUpdate: Yl(n, true),
      staticUpdate: Yl(n, false)
    };
  }
  function Yl(n, t) {
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
      const a = qi(o.format);
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
  class Yy {
    constructor(t) {
      this._size = 0, this._generateParticleUpdateCache = {};
      const e = this._size = t.size ?? 1e3, s = t.properties;
      let i = 0, r = 0;
      for (const h in s) {
        const d = s[h], u = qi(d.format);
        d.dynamic ? r += u.stride : i += u.stride;
      }
      this._dynamicStride = r / 4, this._staticStride = i / 4, this.staticAttributeBuffer = new ps(e * 4 * i), this.dynamicAttributeBuffer = new ps(e * 4 * r), this.indexBuffer = Vl(e);
      const o = new Vh();
      let a = 0, l = 0;
      this._staticBuffer = new Qn({
        data: new Float32Array(1),
        label: "static-particle-buffer",
        shrinkToFit: false,
        usage: ce.VERTEX | ce.COPY_DST
      }), this._dynamicBuffer = new Qn({
        data: new Float32Array(1),
        label: "dynamic-particle-buffer",
        shrinkToFit: false,
        usage: ce.VERTEX | ce.COPY_DST
      });
      for (const h in s) {
        const d = s[h], u = qi(d.format);
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
      const e = Xy(t);
      return this._generateParticleUpdateCache[e] ? this._generateParticleUpdateCache[e] : (this._generateParticleUpdateCache[e] = this.generateParticleUpdate(t), this._generateParticleUpdateCache[e]);
    }
    generateParticleUpdate(t) {
      return Vy(t);
    }
    update(t, e) {
      t.length > this._size && (e = true, this._size = Math.max(t.length, this._size * 1.5 | 0), this.staticAttributeBuffer = new ps(this._size * this._staticStride * 4 * 4), this.dynamicAttributeBuffer = new ps(this._size * this._dynamicStride * 4 * 4), this.indexBuffer = Vl(this._size), this.geometry.indexBuffer.setDataWithSize(this.indexBuffer, this.indexBuffer.byteLength, true));
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
  function Xy(n) {
    const t = [];
    for (const e in n) {
      const s = n[e];
      t.push(e, s.code, s.dynamic ? "d" : "s");
    }
    return t.join("_");
  }
  var qy = `varying vec2 vUV;
varying vec4 vColor;

uniform sampler2D uTexture;

void main(void){
    vec4 color = texture2D(uTexture, vUV) * vColor;
    gl_FragColor = color;
}`, Ky = `attribute vec2 aVertex;
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
`, Xl = `
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
  class Jy extends hr {
    constructor() {
      const t = Jo.from({
        vertex: Ky,
        fragment: qy
      }), e = di.from({
        fragment: {
          source: Xl,
          entryPoint: "mainFragment"
        },
        vertex: {
          source: Xl,
          entryPoint: "mainVertex"
        }
      });
      super({
        glProgram: t,
        gpuProgram: e,
        resources: {
          uTexture: Ct.WHITE.source,
          uSampler: new Jn({}),
          uniforms: {
            uTranslationMatrix: {
              value: new vt(),
              type: "mat3x3<f32>"
            },
            uColor: {
              value: new Yt(16777215),
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
  class mr {
    constructor(t, e) {
      this.state = Ki.for2d(), this.localUniforms = new Zo({
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
      }), this.renderer = t, this.adaptor = e, this.defaultShader = new Jy(), this.state = Ki.for2d(), this._managedContainers = new Ms({
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
      return t._gpuData[this.renderer.uid] = new Yy({
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
      i.update(e, t._childrenDirty), t._childrenDirty = false, r.blendMode = Ao(t.blendMode, t.texture._source);
      const o = this.localUniforms.uniforms, a = o.uTranslationMatrix;
      t.worldTransform.copyTo(a);
      const l = s.globalUniforms.globalUniformData;
      a.tx -= l.offset.x, a.ty -= l.offset.y, a.prepend(l.projectionMatrix), o.uResolution = l.resolution, o.uRound = s._roundPixels | t._roundPixels, ld(t.groupColorAlpha, o.uColor, 0), this.adaptor.execute(this, t);
    }
    destroy() {
      this._managedContainers.destroy(), this.renderer = null, this.defaultShader && (this.defaultShader.destroy(), this.defaultShader = null);
    }
  }
  mr.extension = {
    type: [
      ct.CanvasPipes
    ],
    name: "particle"
  };
  class _d extends mr {
    constructor(t) {
      super(t, new jy());
    }
  }
  _d.extension = {
    type: [
      ct.CanvasPipes
    ],
    name: "particle"
  };
  class Zy {
    execute(t, e) {
      const s = t.state, i = t.renderer, r = e.shader || t.defaultShader;
      r.resources.uTexture = e.texture._source, r.resources.uniforms = t.localUniforms;
      const o = i.gl, a = t.getBuffers(e);
      i.shader.bind(r), i.state.set(s), i.geometry.bind(a.geometry, r.glProgram);
      const c = a.geometry.indexBuffer.data.BYTES_PER_ELEMENT === 2 ? o.UNSIGNED_SHORT : o.UNSIGNED_INT;
      o.drawElements(o.TRIANGLES, e.particleChildren.length * 6, c, 0);
    }
  }
  class wd extends mr {
    constructor(t) {
      super(t, new Zy());
    }
  }
  wd.extension = {
    type: [
      ct.WebGLPipes
    ],
    name: "particle"
  };
  class Qy {
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
  class vd extends mr {
    constructor(t) {
      super(t, new Qy());
    }
  }
  vd.extension = {
    type: [
      ct.WebGPUPipes
    ],
    name: "particle"
  };
  const Cd = class Bo {
    constructor(t) {
      if (t instanceof Ct) this.texture = t, go(this, Bo.defaultOptions, {});
      else {
        const e = {
          ...Bo.defaultOptions,
          ...t
        };
        go(this, e, {});
      }
    }
    get alpha() {
      return this._alpha;
    }
    set alpha(t) {
      this._alpha = Math.min(Math.max(t, 0), 1), this._updateColor();
    }
    get tint() {
      return Vs(this._tint);
    }
    set tint(t) {
      this._tint = Yt.shared.setValue(t ?? 16777215).toBgrNumber(), this._updateColor();
    }
    _updateColor() {
      this.color = this._tint + ((this._alpha * 255 | 0) << 24);
    }
  };
  Cd.defaultOptions = {
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
  let tr = Cd;
  const ql = {
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
  zt.add(wd);
  zt.add(vd);
  zt.add(_d);
  const tx = new Ee(0, 0, 0, 0), Sd = class Oo extends lr {
    constructor(t = {}) {
      t = {
        ...Oo.defaultOptions,
        ...t,
        dynamicProperties: {
          ...Oo.defaultOptions.dynamicProperties,
          ...t == null ? void 0 : t.dynamicProperties
        }
      };
      const { dynamicProperties: e, shader: s, roundPixels: i, texture: r, particles: o, ...a } = t;
      super({
        label: "ParticleContainer",
        ...a
      }), this.renderPipeId = "particle", this.batched = false, this._childrenDirty = false, this.texture = r || null, this.shader = s, this._properties = {};
      for (const l in ql) {
        const c = ql[l], h = e[l];
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
      return tx;
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
  Sd.defaultOptions = {
    dynamicProperties: {
      vertex: false,
      position: true,
      rotation: false,
      uvs: false,
      color: false
    },
    roundPixels: false
  };
  let ex = Sd;
  zt.add(Eu, ku);
  var nx = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
  function sx(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var Td = {
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
    }).call(nx);
  })(Td);
  var ix = Td.exports;
  const Kl = sx(ix);
  function gr(n, t) {
    if (n) {
      if (typeof n == "function") return n;
      if (typeof n == "string") return Kl[n];
    } else return Kl[t];
  }
  class rx {
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
      const e = new Et();
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
  const Fs = [
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
  class ox {
    constructor(t) {
      this.viewport = t, this.list = [], this.plugins = {};
    }
    add(t, e, s = Fs.length) {
      const i = this.plugins[t];
      i && i.destroy(), this.plugins[t] = e;
      const r = Fs.indexOf(t);
      r !== -1 && Fs.splice(r, 1), Fs.splice(s, 0, t), this.sort();
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
      for (const t of Fs) this.plugins[t] && this.list.push(this.plugins[t]);
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
  class Be {
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
  const ax = {
    removeOnInterrupt: false,
    ease: "linear",
    time: 1e3
  };
  class lx extends Be {
    constructor(t, e = {}) {
      super(t), this.startWidth = null, this.startHeight = null, this.deltaWidth = null, this.deltaHeight = null, this.width = null, this.height = null, this.time = 0, this.options = Object.assign({}, ax, e), this.options.ease = gr(this.options.ease), this.setupPosition(), this.setupZoom(), this.time = 0;
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
      const e = new Et(this.parent.scale.x, this.parent.scale.y);
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
          const i = this.startX, r = this.startY, o = this.deltaX, a = this.deltaY, l = new Et(this.parent.x, this.parent.y);
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
  const cx = {
    sides: "all",
    friction: 0.5,
    time: 150,
    ease: "easeInOutSine",
    underflow: "center",
    bounceBox: null
  };
  class hx extends Be {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, cx, e), this.ease = gr(this.options.ease, "easeInOutSine"), this.options.sides ? this.options.sides === "all" ? this.top = this.bottom = this.left = this.right = true : this.options.sides === "horizontal" ? (this.right = this.left = true, this.top = this.bottom = false) : this.options.sides === "vertical" ? (this.left = this.right = false, this.top = this.bottom = true) : (this.top = this.options.sides.indexOf("top") !== -1, this.bottom = this.options.sides.indexOf("bottom") !== -1, this.left = this.options.sides.indexOf("left") !== -1, this.right = this.options.sides.indexOf("right") !== -1) : this.left = this.top = this.right = this.bottom = false;
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
          topLeft: new Et(e * this.parent.scale.x, s * this.parent.scale.y),
          bottomRight: new Et(i * this.parent.scale.x - this.parent.screenWidth, r * this.parent.scale.y - this.parent.screenHeight)
        };
      }
      return {
        left: this.parent.left < 0,
        right: this.parent.right > this.parent.worldWidth,
        top: this.parent.top < 0,
        bottom: this.parent.bottom > this.parent.worldHeight,
        topLeft: new Et(0, 0),
        bottomRight: new Et(this.parent.worldWidth * this.parent.scale.x - this.parent.screenWidth, this.parent.worldHeight * this.parent.scale.y - this.parent.screenHeight)
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
  const dx = {
    left: false,
    right: false,
    top: false,
    bottom: false,
    direction: null,
    underflow: "center"
  };
  class ux extends Be {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, dx, e), this.options.direction && (this.options.left = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.right = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.top = this.options.direction === "y" || this.options.direction === "all" ? true : null, this.options.bottom = this.options.direction === "y" || this.options.direction === "all" ? true : null), this.parseUnderflow(), this.last = {
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
      const t = new Et(this.parent.x, this.parent.y), e = this.parent.plugins.decelerate || {};
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
  const px = {
    minWidth: null,
    minHeight: null,
    maxWidth: null,
    maxHeight: null,
    minScale: null,
    maxScale: null
  };
  class fx extends Be {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, px, e), this.clamp();
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
  const mx = {
    friction: 0.98,
    bounce: 0.8,
    minSpeed: 0.01
  }, wn = 16;
  class gx extends Be {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, mx, e), this.saved = [], this.timeSinceRelease = 0, this.reset(), this.parent.on("moved", (s) => this.handleMoved(s));
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
        this.parent.x += this.x * wn / o * (Math.pow(r, i / wn) - Math.pow(r, s / wn)), this.x *= Math.pow(this.percentChangeX, t / wn);
      }
      if (this.y) {
        const r = this.percentChangeY, o = Math.log(r);
        this.parent.y += this.y * wn / o * (Math.pow(r, i / wn) - Math.pow(r, s / wn)), this.y *= Math.pow(this.percentChangeY, t / wn);
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
  const yx = {
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
  class xx extends Be {
    constructor(t, e = {}) {
      super(t), this.windowEventHandlers = [], this.options = Object.assign({}, yx, e), this.moved = false, this.reverse = this.options.reverse ? 1 : -1, this.xDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "x", this.yDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "y", this.keyIsPressed = false, this.parseUnderflow(), this.mouseButtons(this.options.mouseButtons), this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
            screen: new Et(this.last.x, this.last.y),
            world: this.parent.toWorld(new Et(this.last.x, this.last.y)),
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
        const s = new Et(this.last.x, this.last.y);
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
  const bx = {
    speed: 0,
    acceleration: null,
    radius: null
  };
  class _x extends Be {
    constructor(t, e, s = {}) {
      super(t), this.target = e, this.options = Object.assign({}, bx, s), this.velocity = {
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
  const wx = {
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
  class vx extends Be {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, wx, e), this.reverse = this.options.reverse ? 1 : -1, this.radiusSquared = typeof this.options.radius == "number" ? Math.pow(this.options.radius, 2) : null, this.resize();
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
  const Cx = {
    noDrag: false,
    percent: 1,
    center: null,
    factor: 1,
    axis: "all"
  }, Sx = new Et();
  class Tx extends Be {
    constructor(t, e = {}) {
      super(t), this.active = false, this.pinching = false, this.moved = false, this.options = Object.assign({}, Cx, e);
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
      const { x: e, y: s } = (this.parent.parent || this.parent).toLocal(t.global, void 0, Sx), i = this.parent.input.touches;
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
          const c = new Et(r.last.x + (o.last.x - r.last.x) / 2, r.last.y + (o.last.y - r.last.y) / 2);
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
  const Ax = {
    topLeft: false,
    friction: 0.8,
    time: 1e3,
    ease: "easeInOutSine",
    interrupt: true,
    removeOnComplete: false,
    removeOnInterrupt: false,
    forceStart: false
  };
  class Ex extends Be {
    constructor(t, e, s, i = {}) {
      super(t), this.options = Object.assign({}, Ax, i), this.ease = gr(i.ease, "easeInOutSine"), this.x = e, this.y = s, this.options.forceStart && this.snapStart();
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
  const kx = {
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
  class Mx extends Be {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, kx, e), this.ease = gr(this.options.ease), this.xIndependent = false, this.yIndependent = false, this.xScale = 0, this.yScale = 0, this.options.width > 0 && (this.xScale = t.screenWidth / this.options.width, this.xIndependent = true), this.options.height > 0 && (this.yScale = t.screenHeight / this.options.height, this.yIndependent = true), this.xScale = this.xIndependent ? this.xScale : this.yScale, this.yScale = this.yIndependent ? this.yScale : this.xScale, this.options.time === 0 ? (t.container.scale.x = this.xScale, t.container.scale.y = this.yScale, this.options.removeOnComplete && this.parent.plugins.remove("snap-zoom")) : e.forceStart && this.createSnapping();
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
  const Px = {
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
  class Ix extends Be {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Px, e), this.keyIsPressed = false, this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
  const Rx = {
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
    ticker: Xn.shared,
    allowPreserveDragOutside: false
  };
  class Ad extends Ut {
    constructor(t) {
      super(), this._disableOnContextMenu = (e) => e.preventDefault(), this.options = {
        ...Rx,
        ...t
      }, this.screenWidth = this.options.screenWidth, this.screenHeight = this.options.screenHeight, this._worldWidth = this.options.worldWidth, this._worldHeight = this.options.worldHeight, this.forceHitArea = this.options.forceHitArea, this.threshold = this.options.threshold, this.options.disableOnContextMenu && this.options.events.domElement.addEventListener("contextmenu", this._disableOnContextMenu), this.options.noTicker || (this.tickerFunction = () => this.update(this.options.ticker.elapsedMS), this.options.ticker.add(this.tickerFunction)), this.input = new rx(this), this.plugins = new ox(this);
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
      return arguments.length === 2 ? this.toLocal(new Et(t, e)) : this.toLocal(t);
    }
    toScreen(t, e) {
      return arguments.length === 2 ? this.toGlobal(new Et(t, e)) : this.toGlobal(t);
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
      return new Et(this.worldScreenWidth / 2 - this.x / this.scale.x, this.worldScreenHeight / 2 - this.y / this.scale.y);
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
      return new Et(-this.x / this.scale.x, -this.y / this.scale.y);
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
      return this.plugins.add("snap-zoom", new Mx(this, t)), this;
    }
    OOB() {
      return {
        left: this.left < 0,
        right: this.right > this.worldWidth,
        top: this.top < 0,
        bottom: this.bottom > this.worldHeight,
        cornerPoint: new Et(this.worldWidth * this.scale.x - this.screenWidth, this.worldHeight * this.scale.y - this.screenHeight)
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
      return this.plugins.add("drag", new xx(this, t)), this;
    }
    clamp(t) {
      return this.plugins.add("clamp", new ux(this, t)), this;
    }
    decelerate(t) {
      return this.plugins.add("decelerate", new gx(this, t)), this;
    }
    bounce(t) {
      return this.plugins.add("bounce", new hx(this, t)), this;
    }
    pinch(t) {
      return this.plugins.add("pinch", new Tx(this, t)), this;
    }
    snap(t, e, s) {
      return this.plugins.add("snap", new Ex(this, t, e, s)), this;
    }
    follow(t, e) {
      return this.plugins.add("follow", new _x(this, t, e)), this;
    }
    wheel(t) {
      return this.plugins.add("wheel", new Ix(this, t)), this;
    }
    animate(t) {
      return this.plugins.add("animate", new lx(this, t)), this;
    }
    clampZoom(t) {
      return this.plugins.add("clamp-zoom", new fx(this, t)), this;
    }
    mouseEdges(t) {
      return this.plugins.add("mouse-edges", new vx(this, t)), this;
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
  const Lx = 32, No = /* @__PURE__ */ new Set([
    "transport-belt",
    "fast-transport-belt",
    "express-transport-belt"
  ]), Pi = /* @__PURE__ */ new Set([
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt"
  ]), Zr = /* @__PURE__ */ new Set([
    "splitter",
    "fast-splitter",
    "express-splitter"
  ]), $x = /* @__PURE__ */ new Set([
    "inserter",
    "fast-inserter",
    "long-handed-inserter"
  ]), Bx = /* @__PURE__ */ new Set([
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
  ]), Jl = {
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
  function tn(n, t) {
    return `${n},${t}`;
  }
  function er(n) {
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
  function Ii(n, t, e, s, i, r) {
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
  const Ox = 9;
  function Nx(n) {
    const t = {
      nodes: /* @__PURE__ */ new Map(),
      outEdges: /* @__PURE__ */ new Map(),
      inEdges: /* @__PURE__ */ new Map(),
      tileToAnchor: /* @__PURE__ */ new Map(),
      entityMap: /* @__PURE__ */ new Map()
    };
    for (const e of n.entities) t.entityMap.set(tn(e.x ?? 0, e.y ?? 0), e);
    for (const e of n.entities) {
      if (!No.has(e.name) && !Pi.has(e.name) && !Zr.has(e.name)) continue;
      const s = e.x ?? 0, i = e.y ?? 0, r = tn(s, i);
      if (t.nodes.set(r, e), t.tileToAnchor.set(r, r), Zr.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South", a = s + (o ? 1 : 0), l = i + (o ? 0 : 1);
        t.tileToAnchor.set(tn(a, l), r);
      }
    }
    for (const [e, s] of t.nodes) {
      const i = s.x ?? 0, r = s.y ?? 0, o = s.direction ?? "North", [a, l] = er(o);
      if (No.has(s.name)) {
        const c = t.tileToAnchor.get(tn(i + a, r + l));
        if (c !== void 0 && c !== e) {
          const h = t.nodes.get(c), [d, u] = er(h.direction), p = a * u - l * d;
          Ii(t, e, c, "both", p > 0, false);
        }
      } else if (Pi.has(s.name)) if (s.io_type === "input") for (let c = 1; c <= Ox; c++) {
        const h = t.entityMap.get(tn(i + a * c, r + l * c));
        if (h) {
          if (Pi.has(h.name) && h.name === s.name && h.io_type === "input" && h.direction === o) break;
          if (Pi.has(h.name) && h.name === s.name && h.io_type === "output" && h.direction === o) {
            const d = t.tileToAnchor.get(tn(h.x ?? 0, h.y ?? 0));
            d !== void 0 && Ii(t, e, d, "both", false, false);
            break;
          }
        }
      }
      else {
        const c = t.tileToAnchor.get(tn(i + a, r + l));
        c !== void 0 && c !== e && Ii(t, e, c, "both", false, false);
      }
      else if (Zr.has(s.name)) {
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
          const f = t.tileToAnchor.get(tn(u, p));
          f !== void 0 && f !== e && Ii(t, e, f, "both", false, true);
        }
      }
    }
    return t;
  }
  function Fx(n, t) {
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
  function Wx(n, t) {
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
        const c = tn(r + a, o + l), h = t.get(c);
        h && $x.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  function Gx(n, t) {
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
        const c = tn(r + a, o + l), h = t.get(c);
        h && Bx.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  const Fo = {
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
  function Zl(n, t) {
    const e = t.nodes.get(n);
    if (!e || !No.has(e.name)) return null;
    const s = e.direction ?? "North";
    for (const i of t.inEdges.get(n) ?? []) {
      const r = t.nodes.get(i.from);
      if (!r) continue;
      const o = r.direction ?? "North";
      if (`${o}_${s}` in Fo) return {
        inDir: o,
        outDir: s
      };
    }
    return null;
  }
  function zx(n, t, e, s, i, r, o) {
    const a = s - t, l = i - e, c = Math.sqrt(a * a + l * l);
    if (c === 0) return;
    const h = a / c, d = l / c;
    let u = 0, p = true;
    for (; u < c; ) {
      const f = Math.min(p ? r : o, c - u);
      p && n.moveTo(t + h * u, e + d * u).lineTo(t + h * (u + f), e + d * (u + f)).stroke(), u += f, p = !p;
    }
  }
  function Dx(n, t, e, s, i, r, o, a, l) {
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
  function Hx(n, t, e, s, i) {
    const r = Lx, o = r / 2;
    for (const l of e) {
      if (t.has(l)) continue;
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = Jl[c.name] ?? 14733424, [p, f] = er(c.direction), m = h + r / 2, g = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.05
      }), n.setStrokeStyle({
        width: 1.5,
        color: u,
        alpha: 0.28,
        cap: "round"
      });
      const y = Zl(l, i);
      if (y) {
        const w = Fo[`${y.inDir}_${y.outDir}`], x = w.cx(h, d, r), b = w.cy(h, d, r);
        Dx(n, x, b, o, w.startAngle, w.endAngle, w.anticlockwise, 5 / o, 3 / o);
      } else zx(n, m - p * r * 0.45, g - f * r * 0.45, m + p * r * 0.45, g + f * r * 0.45, 5, 3);
    }
    for (const l of t) {
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = Jl[c.name] ?? 14733424, [p, f] = er(c.direction), m = h + r / 2, g = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.2
      }), n.setStrokeStyle({
        width: 2,
        color: u,
        alpha: 0.85,
        cap: "round"
      });
      const y = Zl(l, i);
      if (y) {
        const w = Fo[`${y.inDir}_${y.outDir}`], x = w.cx(h, d, r), b = w.cy(h, d, r), _ = x + o * Math.cos(w.startAngle), v = b + o * Math.sin(w.startAngle);
        n.moveTo(_, v).arc(x, b, o, w.startAngle, w.endAngle, w.anticlockwise).stroke();
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
  let Ux, jx, Ln, Ed, kd, Vx, Ql, tc, Yx, Xx, qx;
  S = 32;
  Ux = {
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
  jx = 4872810;
  Ln = {
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
  Ed = {
    inserter: 6983230,
    "fast-inserter": 4886736,
    "long-handed-inserter": 13647936
  };
  kd = 9079434;
  Vx = 6974058;
  Ql = 2039583;
  tc = 12623920;
  Yx = 2762e3;
  Xx = 0.35;
  qx = {
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
  function Kx(n, t) {
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = 0.21 * e + 0.72 * s + 0.07 * i, o = Math.round(r + (e - r) * t), a = Math.round(r + (s - r) * t), l = Math.round(r + (i - r) * t);
    return o << 16 | a << 8 | l;
  }
  const ec = Object.fromEntries(Object.entries(qx).map(([n, t]) => [
    n,
    Kx(t, Xx)
  ]));
  function Jx(n, t, e) {
    const s = t * Math.min(e, 1 - e), i = (r) => {
      const o = (r + n * 12) % 12;
      return Math.round((e - s * Math.max(-1, Math.min(o - 3, 9 - o, 1))) * 255);
    };
    return i(0) << 16 | i(8) << 8 | i(4);
  }
  let Md = true;
  function Ri(n) {
    Md = n;
  }
  let Wo = /* @__PURE__ */ new Map();
  function Zx(n) {
    return Wo.get(n);
  }
  function Pd(n) {
    Wo = /* @__PURE__ */ new Map();
    for (const t of n) Wo.set(t.recipe, {
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
  function In(n) {
    if (!Md) return 7829367;
    if (!n) return 6710886;
    if (n in ec) return ec[n];
    let t = 0;
    for (let s = 0; s < n.length; s++) t = (t << 5) - t + n.charCodeAt(s) | 0;
    const e = Math.abs(t) % 30 * 12;
    return Jx(e / 360, 0.2, 0.48);
  }
  const Ve = {
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
  function me(n) {
    return n.split("-").map((t) => t.charAt(0).toUpperCase() + t.slice(1)).join(" ");
  }
  let Le, yr, rn, ie, ii, ua;
  Le = new Set(Object.keys(Ve));
  yr = new Set(Object.keys(Ed));
  rn = new Set(Object.keys(Ln).filter((n) => !n.includes("underground") && !n.includes("splitter")));
  fe = new Set(Object.keys(Ln).filter((n) => n.includes("underground")));
  ie = new Set(Object.keys(Ln).filter((n) => n.includes("splitter")));
  ii = /* @__PURE__ */ new Set([
    "pipe",
    "pipe-to-ground"
  ]);
  ua = /* @__PURE__ */ new Set([
    "medium-electric-pole",
    "small-electric-pole"
  ]);
  function ui(n) {
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
  function Pn(n) {
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
  function pa(n) {
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
  function Id(n, t) {
    const e = n.direction ?? "North", [s, i] = Pn(e);
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
      if (!d || !(rn.has(d.name) || fe.has(d.name) && d.io_type === "output" || ie.has(d.name))) continue;
      const [p, f] = Pn(d.direction), m = ie.has(d.name) ? c : d.x ?? 0, g = ie.has(d.name) ? h : d.y ?? 0;
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
  function nc(n, t) {
    const e = Math.round((n >> 16 & 255) * t), s = Math.round((n >> 8 & 255) * t), i = Math.round((n & 255) * t);
    return e << 16 | s << 8 | i;
  }
  const ri = 3, Rd = 3815994, fa = 5592405, ma = 0.9, Li = S * (1 - ma) / 2, Ld = S * ma;
  function ga(n, t) {
    const e = new ft(), s = S, i = Ld, [, r] = Ln[n.name] ?? [
      11046960,
      14733424
    ], o = In(n.carries);
    if (t) Qx(e, s, r, n.direction, t, o);
    else {
      e.rect(Li, Li, i, i).fill(Rd), e.setStrokeStyle({
        width: 1,
        color: fa,
        alignment: 0
      }), e.rect(Li, Li, i, i).stroke();
      const a = new ft();
      a.x = s / 2, a.y = s / 2, a.rotation = ui(n.direction), a.rect(-i / 2, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(1, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(-1, -i / 2, 2, i).fill(657930), t0(a, i, r), e.addChild(a);
    }
    return e;
  }
  function Qx(n, t, e, s, i, r) {
    const o = new ft();
    o.x = t / 2, o.y = t / 2, o.rotation = ui(s), o.scale.set(ma);
    const a = t / 2, c = (i.turn === "cw" ? 1 : -1) * a, h = -a, d = i.turn === "ccw" ? 0 : Math.PI / 2, u = i.turn === "ccw" ? Math.PI / 2 : Math.PI;
    o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().fill(Rd), o.setStrokeStyle({
      width: 1,
      color: fa,
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
    const x = t * 0.22, b = Math.max(1, t * 0.07), _ = t * 0.5, v = x / _, C = _ + x, E = _ - x;
    o.setStrokeStyle({
      width: b,
      color: e,
      cap: "round",
      join: "round"
    });
    const R = Math.PI / 2, k = i.turn === "cw" ? Math.PI : 0;
    for (const T of [
      0.6
    ]) {
      const O = R + T * (k - R), U = i.turn === "cw" ? O - v : O + v, G = c + _ * Math.cos(O), B = h + _ * Math.sin(O), L = c + C * Math.cos(U), D = h + C * Math.sin(U), K = c + E * Math.cos(U), X = h + E * Math.sin(U);
      o.moveTo(L, D).lineTo(G, B).lineTo(K, X).stroke();
    }
    n.addChild(o);
  }
  function t0(n, t, e) {
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
  function $d(n) {
    const t = new ft(), e = S, [, s] = Ln[n.name] ?? [
      11046960,
      14733424
    ], i = n.io_type === "input", r = e / 2, o = i ? 1 : -1, a = new ft();
    a.x = r, a.y = r, a.rotation = ui(n.direction);
    const l = In(n.carries), c = Ld / 2, h = e * 0.25, d = o * r, u = 0;
    a.moveTo(-c, d).lineTo(c, d).lineTo(h, u).lineTo(-h, u).closePath().fill({
      color: l,
      alpha: 0.7
    }), a.setStrokeStyle({
      width: 1,
      color: fa,
      alpha: 0.8
    }), a.moveTo(-c, d).lineTo(-h, u).lineTo(h, u).lineTo(c, d).stroke();
    const p = e * 0.38, f = e * 0.3, m = o * e * 0.22, g = m - f / 2, y = m + f / 2;
    return a.moveTo(0, g).lineTo(p / 2, y).lineTo(-p / 2, y).closePath().fill(s), t.addChild(a), t;
  }
  function Bd(n) {
    const t = new ft(), [e, s] = Ln[n.name] ?? [
      11046960,
      14733424
    ], i = n.direction === "North" || n.direction === "South", r = i ? S * 2 - 1 : S - 1, o = i ? S - 1 : S * 2 - 1, a = i ? r / 2 : o / 2, l = Math.max(2, Math.min(r, o) * 0.18);
    t.roundRect(0, 0, r, o, ri).fill(e), t.roundRect(0, 0, r, o, ri).fill({
      color: In(n.carries),
      alpha: 0.3
    }), i ? t.rect(a - l / 2, 0, l, o).fill(nc(e, 0.5)) : t.rect(0, a - l / 2, r, l).fill(nc(e, 0.5));
    const c = ui(n.direction), h = a * 0.25, d = Math.max(1, a * 0.12);
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
  function Od(n) {
    const t = new ft(), e = S - 1, s = n.carries ? In(n.carries) : Ed[n.name] ?? 6983230;
    t.roundRect(0, 0, e, e, ri).fill(2767402);
    const i = new ft();
    i.x = e / 2, i.y = e / 2, i.rotation = ui(n.direction), i.circle(0, e * 0.2, e * 0.15).fill(4473924);
    const r = Math.max(1.5, e * 0.12);
    i.setStrokeStyle({
      width: r,
      color: s,
      cap: "round"
    }), i.moveTo(0, e * 0.2).lineTo(0, -e * 0.35).stroke();
    const o = -e * 0.35, a = e * 0.18;
    return i.moveTo(-a, o - a * 0.6).lineTo(0, o).lineTo(a, o - a * 0.6).stroke(), t.addChild(i), t;
  }
  const ya = 1, xa = 2, ba = 4, _a = 8;
  function e0(n) {
    const [t, e] = Pn(n.direction);
    return [
      -t,
      -e
    ];
  }
  function Nd(n, t, e) {
    if (n.name === "pipe") return true;
    if (n.name === "pipe-to-ground") {
      const [s, i] = e0(n);
      return -t === s && -e === i;
    }
    return false;
  }
  function Fd(n, t) {
    const e = new ft(), s = S - 1, i = n.name === "pipe-to-ground", r = i ? Vx : kd;
    e.roundRect(0, 0, s, s, ri).fill(Ql);
    const o = s / 2, a = s / 2, l = Math.max(2, s * 0.4);
    if (i) {
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      });
      const [c, h] = Pn(n.direction);
      e.moveTo(o, a).lineTo(o - c * s / 2, a - h * s / 2).stroke(), e.circle(o, a, l * 0.4).fill(r), e.circle(o, a, l * 0.25).fill(Ql);
    } else if (t === 0) e.circle(o, a, l * 0.4).fill(r);
    else {
      const c = !!(t & ya), h = !!(t & xa), d = !!(t & ba), u = !!(t & _a), p = (c ? 1 : 0) + (h ? 1 : 0) + (d ? 1 : 0) + (u ? 1 : 0);
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      }), p === 1 ? (c ? e.moveTo(o, a).lineTo(o, 0).stroke() : h ? e.moveTo(o, a).lineTo(s, a).stroke() : d ? e.moveTo(o, a).lineTo(o, s).stroke() : e.moveTo(o, a).lineTo(0, a).stroke(), e.circle(o, a, l * 0.4).fill(r)) : c && d && !h && !u ? e.moveTo(o, 0).lineTo(o, s).stroke() : h && u && !c && !d ? e.moveTo(0, a).lineTo(s, a).stroke() : p === 2 ? c && h ? e.moveTo(o, 0).quadraticCurveTo(o, a, s, a).stroke() : h && d ? e.moveTo(s, a).quadraticCurveTo(o, a, o, s).stroke() : d && u ? e.moveTo(o, s).quadraticCurveTo(o, a, 0, a).stroke() : e.moveTo(0, a).quadraticCurveTo(o, a, o, 0).stroke() : p === 3 ? u ? d ? h ? (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, s).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(0, a).stroke()) : (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, 0).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(s, a).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(0, a).lineTo(s, a).stroke());
    }
    return e;
  }
  function Wd() {
    const n = new ft(), t = S - 1;
    n.roundRect(0, 0, t, t, ri).fill(Yx);
    const e = t / 2, s = t / 2, i = t * 0.38, r = Math.max(1.5, t * 0.2);
    return n.rect(e - r / 2, s - i, r, i * 2).fill(tc), n.rect(e - i, s - r / 2, i * 2, r).fill(tc), n.circle(e, s, r * 0.6).fill(14729280), n;
  }
  function Gd(n) {
    const t = new ft(), [e, s] = Ve[n.name] ?? [
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
    const l = 1.8, c = sc(`/spaghettio/entity-frames/${n.name}.png`);
    if (c) {
      const h = new je(c), d = S / n0;
      h.scale.set(d * l), h.x = -i * (l - 1) / 2, h.y = -r * (l - 1) / 2, t.addChild(h);
    } else {
      const h = sc(`/spaghettio/icons/${n.name}.png`);
      if (h) {
        const d = new je(h), u = Math.min(i, r) * 0.8 * l;
        d.width = u, d.height = u, d.x = (i - u) / 2, d.y = (r - u) / 2, t.addChild(d);
      } else {
        const d = Ux[n.name] ?? jx;
        t.roundRect(2, 2, i - 4, r - 4, 3).fill({
          color: d,
          alpha: 0.5
        });
      }
    }
    return t;
  }
  function zd() {
    const n = new ft(), t = S - 1;
    return n.rect(0, 0, t, t).fill(4872810), n.setStrokeStyle({
      width: 1,
      color: 0,
      alpha: 0.4
    }), n.rect(0, 0, t, t).stroke(), n;
  }
  const n0 = 64;
  async function s0(n) {
    const t = "/spaghettio/", e = [
      ...n.map((s) => `${t}icons/${s}.png`),
      ...n.map((s) => `${t}entity-frames/${s}.png`)
    ];
    await Promise.allSettled(e.map((s) => He.load(s)));
  }
  async function Dd(n) {
    const t = "/spaghettio/";
    await Promise.allSettled(n.map((e) => He.load(`${t}icons/${e}.png`)));
  }
  function i0(n) {
    const t = /* @__PURE__ */ new Set();
    for (const e of n) e.carries && t.add(e.carries);
    return Array.from(t);
  }
  function sc(n) {
    return se.has(n) ? He.get(n) ?? null : null;
  }
  const r0 = {
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
  function Hd(n) {
    const t = r0[n.name];
    if (!t) return [];
    const e = n.mirror ?? false;
    return t.filter(([, , , s]) => s === "always" || s === "default" && !e || s === "mirror" && e);
  }
  function Ud(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of Hd(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  xr = function() {
    return {
      tileMap: /* @__PURE__ */ new Map(),
      machineByTile: /* @__PURE__ */ new Map()
    };
  };
  nr = function(n, t) {
    const e = n.x ?? 0, s = n.y ?? 0;
    if (t.tileMap.set(`${e},${s}`, n), ie.has(n.name)) {
      const [i, r] = pa(n.direction);
      t.tileMap.set(`${e + i},${s + r}`, n);
    }
    if (Le.has(n.name)) {
      const [i, r] = Ve[n.name] ?? [
        1,
        1
      ];
      for (let o = 0; o < r; o++) for (let a = 0; a < i; a++) t.machineByTile.set(`${e + a},${s + o}`, n);
    }
  };
  Nn = function(n, t) {
    let e;
    if (rn.has(n.name)) e = ga(n, Id(n, t.tileMap));
    else if (fe.has(n.name)) e = $d(n);
    else if (ie.has(n.name)) e = Bd(n);
    else if (yr.has(n.name)) e = Od(n);
    else if (ii.has(n.name)) {
      let s = 0;
      if (n.name === "pipe") {
        const i = n.x ?? 0, r = n.y ?? 0;
        for (const [o, a, l] of [
          [
            0,
            -1,
            ya
          ],
          [
            1,
            0,
            xa
          ],
          [
            0,
            1,
            ba
          ],
          [
            -1,
            0,
            _a
          ]
        ]) {
          const c = `${i + o},${r + a}`, h = t.tileMap.get(c);
          if (h && Nd(h, o, a)) {
            s |= l;
            continue;
          }
          const d = t.machineByTile.get(c);
          d && Ud(i, r, d) && (s |= l);
        }
      }
      e = Fd(n, s);
    } else ua.has(n.name) ? e = Wd() : Le.has(n.name) ? e = Gd(n) : e = zd();
    return e.x = (n.x ?? 0) * S, e.y = (n.y ?? 0) * S, e;
  };
  dv = function(n, t, e = 8) {
    if (!fe.has(n.name) || n.io_type !== "input") return null;
    const [s, i] = Pn(n.direction), r = n.x ?? 0, o = n.y ?? 0;
    for (let a = 1; a <= e; a++) {
      const l = t.get(`${r + s * a},${o + i * a}`);
      if (l) {
        if (fe.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "input") return null;
        if (fe.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "output") {
          const [c] = Ln[n.name] ?? [
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
  Js = function(n, t, e, s, i, r) {
    t.removeChildren();
    const o = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (o.set(`${f.x ?? 0},${f.y ?? 0}`, f), ie.has(f.name)) {
      const [m, g] = pa(f.direction);
      o.set(`${(f.x ?? 0) + m},${(f.y ?? 0) + g}`, f);
    }
    if (r) for (const f of r) {
      const m = `${f.x ?? 0},${f.y ?? 0}`;
      o.has(m) || o.set(m, f);
    }
    const a = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (Le.has(f.name)) {
      const [m, g] = Ve[f.name] ?? [
        1,
        1
      ], y = f.x ?? 0, w = f.y ?? 0;
      for (let x = 0; x < g; x++) for (let b = 0; b < m; b++) a.set(`${y + b},${w + x}`, f);
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const g of n.entities) fe.has(g.name) && f.set(`${g.x ?? 0},${g.y ?? 0}`, g);
      const m = 8;
      for (const g of n.entities) {
        if (!fe.has(g.name) || g.io_type !== "input") continue;
        const [y, w] = Pn(g.direction), x = g.x ?? 0, b = g.y ?? 0;
        for (let _ = 1; _ <= m; _++) {
          const v = f.get(`${x + y * _},${b + w * _}`);
          if (v) {
            if (fe.has(v.name) && v.name === g.name && v.direction === g.direction && v.io_type === "input") break;
            if (fe.has(v.name) && v.name === g.name && v.direction === g.direction && v.io_type === "output") {
              const [C] = Ln[g.name] ?? [
                11046960,
                14733424
              ], E = new ft(), R = Math.abs(y) > 0;
              for (let k = 1; k < _; k++) {
                const T = (x + y * k) * S, O = (b + w * k) * S;
                R ? E.rect(T, O + S * 0.25, S, S * 0.5).fill({
                  color: C,
                  alpha: 0.25
                }) : E.rect(T + S * 0.25, O, S * 0.5, S).fill({
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
      for (const g of n.entities) g.name === "pipe-to-ground" && f.set(`${g.x ?? 0},${g.y ?? 0}`, g);
      const m = 10;
      for (const g of n.entities) {
        if (g.name !== "pipe-to-ground" || g.io_type !== "input") continue;
        const [y, w] = Pn(g.direction), x = g.x ?? 0, b = g.y ?? 0;
        for (let _ = 2; _ <= m; _++) {
          const v = f.get(`${x + y * _},${b + w * _}`);
          if (!v) continue;
          const [C, E] = Pn(v.direction);
          if (v.io_type !== "output" || C !== -y || E !== -w) break;
          const R = new ft();
          R.setStrokeStyle({
            width: 2,
            color: kd,
            alpha: 0.55,
            cap: "round"
          });
          const k = (x + 0.5 + y * 0.5) * S, T = (b + 0.5 + w * 0.5) * S, O = (_ - 1) * S, U = 5, G = 3;
          let B = 0;
          for (; B < O; ) {
            const L = Math.min(B + U, O);
            R.moveTo(k + y * B, T + w * B).lineTo(k + y * L, T + w * L).stroke(), B = L + G;
          }
          t.addChild(R);
          break;
        }
      }
    }
    const l = /* @__PURE__ */ new Map(), c = [], h = /* @__PURE__ */ new Map();
    for (const f of n.entities) {
      let m;
      if (rn.has(f.name)) m = ga(f, Id(f, o));
      else if (fe.has(f.name)) m = $d(f);
      else if (ie.has(f.name)) m = Bd(f);
      else if (yr.has(f.name)) m = Od(f);
      else if (ii.has(f.name)) {
        let y = 0;
        if (f.name === "pipe") {
          const w = f.x ?? 0, x = f.y ?? 0;
          for (const [b, _, v] of [
            [
              0,
              -1,
              ya
            ],
            [
              1,
              0,
              xa
            ],
            [
              0,
              1,
              ba
            ],
            [
              -1,
              0,
              _a
            ]
          ]) {
            if (_ === -1 && x + _ < 0) {
              y |= v;
              continue;
            }
            const C = `${w + b},${x + _}`, E = o.get(C);
            if (E && Nd(E, b, _)) {
              y |= v;
              continue;
            }
            const R = a.get(C);
            R && Ud(w, x, R) && (y |= v);
          }
        }
        m = Fd(f, y);
      } else ua.has(f.name) ? m = Wd() : Le.has(f.name) ? m = Gd(f) : m = zd();
      m.x = (f.x ?? 0) * S, m.y = (f.y ?? 0) * S, s && (m.eventMode = "static", m.cursor = "pointer", m.on("click", () => s(f)));
      const g = ic(f);
      g && (l.has(g) || l.set(g, []), l.get(g).push(m)), h.set(m, `${f.x ?? 0},${f.y ?? 0}`), c.push(m), t.addChild(m), i == null ? void 0 : i(f, [
        m
      ]);
    }
    const d = Nx(n);
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
        const { downstream: y, upstream: w } = Fx(g, d), x = /* @__PURE__ */ new Set([
          ...y,
          ...w
        ]), b = Wx(x, d.entityMap), _ = Gx(b, d.entityMap);
        for (const v of c) {
          const C = h.get(v);
          if (!C) {
            v.alpha = 0.15;
            continue;
          }
          x.has(C) ? v.alpha = 0.5 : b.has(C) ? v.alpha = 0.9 : _.has(C) ? v.alpha = 0.75 : v.alpha = 0.15;
        }
        u = new ft(), Hx(u, y, w, g, d), t.addChild(u);
      },
      clearHighlight() {
        p();
      },
      chainKey: ic
    };
  };
  function ic(n) {
    return n.carries ? n.carries : n.recipe ? n.recipe : null;
  }
  const oi = 4096, gt = 128, en = oi / gt;
  let En = null;
  const ai = /* @__PURE__ */ new Map();
  let sn = 0, li = null;
  function o0(n) {
    li = n;
  }
  function un(n, t, e, s) {
    const i = ai.get(n);
    if (i) return i;
    if (!li) return console.warn("[atlas] getEntityTexture called before initAtlas; returning blank texture"), Ct.EMPTY;
    En || (En = fr.create({
      width: oi,
      height: oi
    })), sn >= en * en && (console.warn("[atlas] atlas is full \u2014 variant will reuse slot 0:", n), sn = 0);
    const r = sn % en, o = Math.floor(sn / en), a = r * gt, l = o * gt;
    sn++;
    const c = new ft();
    s(c);
    const h = new vt(1, 0, 0, 1, a, l);
    li.render({
      container: c,
      target: En,
      transform: h,
      clear: false
    }), c.destroy({
      children: true
    });
    const d = new Ht(a, l, gt, gt), u = new Ct({
      source: En.source,
      frame: d
    });
    return ai.set(n, u), u;
  }
  function rc(n, t, e, s) {
    const i = ai.get(n);
    if (i) return i;
    if (!li) return console.warn("[atlas] getMultiCellTexture called before initAtlas; returning blank texture"), Ct.EMPTY;
    En || (En = fr.create({
      width: oi,
      height: oi
    }));
    const r = sn % en;
    r + t > en && (sn += en - r);
    const a = sn % en, l = Math.floor(sn / en), c = a * gt, h = l * gt;
    sn = (l + e) * en;
    const d = t * gt, u = e * gt, p = new ft();
    s(p, d, u);
    const f = new vt(1, 0, 0, 1, c, h);
    li.render({
      container: p,
      target: En,
      transform: f,
      clear: false
    }), p.destroy({
      children: true
    });
    const m = new Ht(c, h, d, u), g = new Ct({
      source: En.source,
      frame: m
    });
    return ai.set(n, g), g;
  }
  function jd(n) {
    const t = `icon:${n}`, e = ai.get(t);
    if (e) return e;
    const s = `/spaghettio/icons/${n}.png`;
    if (se.has(s)) {
      const r = He.get(s);
      if (r) return un(t, gt, gt, (a) => {
        const c = gt - 16;
        a.rect(8, 8, c, c).fill({
          texture: r
        });
      });
    }
    const i = In(n);
    return un(t, gt, gt, (r) => {
      const o = gt / 2, a = gt / 2;
      r.circle(o, a, 7).fill({
        color: i,
        alpha: 0.85
      });
    });
  }
  function a0(n, t, e = "straight") {
    return `belt:${n}:${t}:${e}`;
  }
  function l0(n) {
    return `pipe:${n}`;
  }
  function c0(n, t, e) {
    return `ugbelt:${n}:${t}:${e}`;
  }
  function h0(n, t) {
    return `splitter:${n}:${t}`;
  }
  function d0(n, t) {
    return `inserter:${n}:${t}`;
  }
  function u0(n) {
    return `machine:${n}`;
  }
  function p0(n) {
    return `pole:${n}`;
  }
  function f0(n) {
    return `ptg:${n}`;
  }
  const nn = 3200;
  let Vd = null, Yd = null, Xd = null;
  function Ue() {
    Vd == null ? void 0 : Vd();
  }
  function br() {
    Yd == null ? void 0 : Yd();
  }
  function Kn() {
    Xd == null ? void 0 : Xd();
  }
  async function m0(n) {
    const t = new Qo();
    await t.init({
      resizeTo: n,
      background: 1973790,
      antialias: true,
      autoStart: false,
      sharedTicker: false
    }), o0(t.renderer), t.ticker.add(() => t.render(), null, Qs.LOW), n.appendChild(t.canvas), t.canvas.addEventListener("contextmenu", (h) => h.preventDefault());
    const e = new Ad({
      screenWidth: n.clientWidth,
      screenHeight: n.clientHeight,
      worldWidth: nn,
      worldHeight: nn,
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
    Vd = r, Yd = o, Xd = a, e.on("moved", r), e.on("zoomed", r);
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
      e.resize(n.clientWidth, n.clientHeight, nn, nn), r();
    }), r(), {
      app: t,
      viewport: e,
      requestRender: r,
      beginAnimating: o,
      endAnimating: a
    };
  }
  const $i = 32, oc = 2763306, ac = 3815994;
  function g0(n) {
    const t = new ft();
    return n.addChildAt(t, 0), t;
  }
  function Fn(n, t, e, s = 1) {
    if (n.clear(), t <= 0 || e <= 0) return;
    const i = Math.max(0, Math.min(1, s));
    if (i === 0) return;
    const r = t * $i, a = e * $i * i;
    for (let l = 0; l <= t; l++) {
      const c = l * $i, h = l % 10 === 0;
      n.moveTo(c, 0).lineTo(c, a).stroke({
        width: h ? 1.5 : 1,
        color: h ? ac : oc
      });
    }
    for (let l = 0; l <= e; l++) {
      const c = l * $i;
      if (c > a) break;
      const h = l % 10 === 0;
      n.moveTo(0, c).lineTo(r, c).stroke({
        width: h ? 1.5 : 1,
        color: h ? ac : oc
      });
    }
  }
  const Vi = {
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
  }, y0 = 8947848;
  function qd(n) {
    const t = n >> 16 & 255, e = n >> 8 & 255, s = n & 255;
    return `rgb(${t},${e},${s})`;
  }
  const x0 = [
    {
      name: "transport-belt",
      color: Vi["transport-belt"],
      throughput: 15
    },
    {
      name: "fast-transport-belt",
      color: Vi["fast-transport-belt"],
      throughput: 30
    },
    {
      name: "express-transport-belt",
      color: Vi["express-transport-belt"],
      throughput: 45
    }
  ];
  function Kd(n) {
    for (const t of x0) if (n <= t.throughput) return t;
    return null;
  }
  const b0 = 240, lc = 80, wa = 180, _0 = 60, cc = 100, hc = 100, dc = "production-graph";
  function w0(n) {
    return n <= 15 ? 13153632 : n <= 30 ? 14700624 : n <= 45 ? 5284064 : 16711816;
  }
  const uc = new $e({
    fontSize: 13,
    fontWeight: "bold",
    fill: 14737632,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: wa - 12
  }), pc = new $e({
    fontSize: 11,
    fill: 10280190,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: wa - 12
  }), fc = new $e({
    fontSize: 10,
    fill: 16777215,
    fontFamily: "sans-serif"
  });
  function Ws(n, t) {
    const e = n.getChildByName(dc);
    e && (e.destroy({
      children: true
    }), n.removeChild(e));
    const s = new Ut();
    if (s.label = dc, n.addChild(s), !t || t.machines.length === 0) return s;
    const { dependency_order: i } = t, r = i.length, o = /* @__PURE__ */ new Map();
    i.forEach((x, b) => {
      o.set(x, r - 1 - b);
    });
    const a = 1, l = /* @__PURE__ */ new Map();
    for (const x of t.machines) {
      const b = o.get(x.recipe) ?? 0;
      l.has(b) || l.set(b, []), l.get(b).push(x);
    }
    for (const x of l.values()) x.sort((b, _) => b.recipe.localeCompare(_.recipe));
    const c = [
      ...new Set(t.external_inputs.map((x) => x.item))
    ].sort(), h = /* @__PURE__ */ new Map();
    for (const x of t.machines) for (const b of x.outputs) h.set(b.item, x);
    const d = /* @__PURE__ */ new Map();
    for (const x of t.external_inputs) d.set(x.item, x.rate);
    const u = /* @__PURE__ */ new Map();
    for (const [x, b] of l) b.forEach((_, v) => {
      u.set(_.recipe, {
        x: cc + (x + a) * b0,
        y: hc + v * lc,
        w: wa,
        h: _0,
        machine: _
      });
    });
    const p = /* @__PURE__ */ new Map();
    c.forEach((x, b) => {
      p.set(x, {
        x: cc,
        y: hc + b * lc,
        w: 140,
        h: 40
      });
    });
    const f = new ft();
    s.addChild(f);
    for (const x of t.machines) {
      const b = u.get(x.recipe);
      if (b) for (const _ of x.inputs) {
        const v = h.get(_.item), C = v ? u.get(v.recipe) : p.get(_.item);
        if (!C) continue;
        const E = w0(_.rate), R = C.x + C.w, k = C.y + C.h * 2 / 3, T = b.x, O = b.y + b.h / 3, U = (R + T) / 2;
        f.moveTo(R, k).lineTo(U, k).lineTo(U, O).lineTo(T, O).stroke({
          color: E,
          width: 2,
          alpha: 0.85
        });
        const G = `${_.rate.toFixed(1)}/s ${_.item}`, B = (R + U) / 2, L = k - 14, D = dn.measureText(G, fc), K = new ft();
        K.rect(B - 2, L - 1, D.width + 4, D.height + 2).fill({
          color: 1973790,
          alpha: 0.7
        }), s.addChild(K);
        const X = new De({
          text: G,
          style: fc
        });
        X.position.set(B, L), s.addChild(X);
      }
    }
    for (const x of u.values()) {
      const b = x.machine, _ = Vi[b.entity] ?? y0, v = new ft();
      v.rect(x.x, x.y, x.w, x.h).fill({
        color: _,
        alpha: 0.6
      }).stroke({
        color: _,
        width: 2
      }), s.addChild(v);
      const C = new De({
        text: `${b.count.toFixed(1)} \xD7 ${b.entity}`,
        style: uc
      });
      C.position.set(x.x + 6, x.y + 6), s.addChild(C);
      const E = new De({
        text: b.recipe,
        style: pc
      });
      E.position.set(x.x + 6, x.y + 24), s.addChild(E);
    }
    for (const [x, b] of p) {
      const _ = d.get(x), v = _ !== void 0 ? `${_.toFixed(1)}/s` : "", C = new ft();
      C.rect(b.x, b.y, b.w, b.h).fill({
        color: 2763306,
        alpha: 0.8
      }).stroke({
        color: 8947848,
        width: 1.5
      }), s.addChild(C);
      const E = new De({
        text: v,
        style: uc
      });
      E.position.set(b.x + 6, b.y + 4), s.addChild(E);
      const R = new De({
        text: x,
        style: pc
      });
      R.position.set(b.x + 6, b.y + 20), s.addChild(R);
    }
    let m = 1 / 0, g = -1 / 0, y = 1 / 0, w = -1 / 0;
    for (const x of [
      ...u.values(),
      ...p.values()
    ]) x.x < m && (m = x.x), x.x + x.w > g && (g = x.x + x.w), x.y < y && (y = x.y), x.y + x.h > w && (w = x.y + x.h);
    return n.moveCenter((m + g) / 2, (y + w) / 2), s;
  }
  function oe(n, t) {
    return `${n},${t}`;
  }
  function we(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function vn(n) {
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
  function ge(n, t, e) {
    if (t === e) return;
    let s = n.get(t);
    s || (s = [], n.set(t, s)), s.includes(e) || s.push(e);
    let i = n.get(e);
    i || (i = [], n.set(e, i)), i.includes(t) || i.push(t);
  }
  function v0(n) {
    const t = /* @__PURE__ */ new Map();
    for (const e of n.entities) {
      const s = e.x ?? 0, i = e.y ?? 0, r = we(e);
      if (t.set(oe(s, i), r), ie.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South";
        t.set(oe(s + (o ? 1 : 0), i + (o ? 0 : 1)), r);
      }
      if (Le.has(e.name)) {
        const [o, a] = Ve[e.name] ?? [
          1,
          1
        ];
        for (let l = 0; l < a; l++) for (let c = 0; c < o; c++) c === 0 && l === 0 || t.set(oe(s + c, i + l), r);
      }
    }
    return t;
  }
  const C0 = 9;
  function Jd(n) {
    const t = /* @__PURE__ */ new Map(), e = v0(n);
    for (const o of n.entities) {
      const a = we(o);
      t.has(a) || t.set(a, []);
    }
    const s = /* @__PURE__ */ new Map();
    for (const o of n.entities) s.set(oe(o.x ?? 0, o.y ?? 0), o);
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
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = vn(o.direction);
      if (rn.has(o.name)) {
        const u = e.get(oe(a + h, l + d));
        u && ge(t, c, u);
        const p = e.get(oe(a - h, l - d));
        p && ge(t, c, p);
        for (const [f, m] of i) {
          if (f === h && m === d || f === -h && m === -d) continue;
          const g = s.get(oe(a + f, l + m));
          if (!g || !rn.has(g.name) && !fe.has(g.name) && !ie.has(g.name)) continue;
          const [y, w] = vn(g.direction), x = ie.has(g.name) ? a + f : g.x ?? 0, b = ie.has(g.name) ? l + m : g.y ?? 0;
          x + y === a && b + w === l && ge(t, c, we(g));
        }
      } else if (fe.has(o.name)) {
        if (o.io_type === "input") for (let u = 1; u <= C0; u++) {
          const p = s.get(oe(a + h * u, l + d * u));
          if (p) {
            if (fe.has(p.name) && p.name === o.name && p.io_type === "input" && p.direction === o.direction) break;
            if (fe.has(p.name) && p.name === o.name && p.io_type === "output" && p.direction === o.direction) {
              ge(t, c, we(p));
              break;
            }
          }
        }
        else {
          const u = e.get(oe(a + h, l + d));
          u && ge(t, c, u);
        }
        for (const [u, p] of i) {
          const f = s.get(oe(a + u, l + p));
          if (!f || !rn.has(f.name) && !ie.has(f.name)) continue;
          const [m, g] = vn(f.direction);
          (f.x ?? 0) + m === a && (f.y ?? 0) + g === l && ge(t, c, we(f));
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
          const y = e.get(oe(a + m + h, l + g + d));
          y && y !== c && ge(t, c, y);
          const w = e.get(oe(a + m - h, l + g - d));
          w && w !== c && ge(t, c, w);
        }
      }
    }
    for (const o of n.entities) {
      if (!yr.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = vn(o.direction), p = o.name === "long-handed-inserter" ? 2 : 1, f = e.get(oe(a - h * p, l - d * p)), m = e.get(oe(a + h * p, l + d * p));
      f && ge(t, c, f), m && ge(t, c, m);
    }
    const r = 10;
    for (const o of n.entities) {
      if (!ii.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o);
      if (o.name === "pipe") for (const [h, d] of i) {
        const u = s.get(oe(a + h, l + d));
        if (u) {
          if (u.name === "pipe") ge(t, c, we(u));
          else if (u.name === "pipe-to-ground") {
            const [p, f] = vn(u.direction);
            h === p && d === f && ge(t, c, we(u));
          } else if (Le.has(u.name)) {
            const p = e.get(oe(u.x ?? 0, u.y ?? 0));
            p && ge(t, c, p);
          }
        }
      }
      else if (o.name === "pipe-to-ground") {
        if (o.io_type === "input") {
          const [p, f] = vn(o.direction);
          for (let m = 2; m <= r; m++) {
            const g = s.get(oe(a + p * m, l + f * m));
            if (!g || g.name !== "pipe-to-ground") continue;
            const [y, w] = vn(g.direction);
            if (g.io_type === "output" && y === -p && w === -f) {
              ge(t, c, we(g));
              break;
            }
            break;
          }
        }
        const [h, d] = vn(o.direction), u = s.get(oe(a - h, l - d));
        u && u.name === "pipe" && ge(t, c, we(u));
      }
    }
    for (const o of n.entities) {
      if (!Le.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = Ve[o.name] ?? [
        1,
        1
      ];
      for (let u = 0; u < d; u++) for (let p = 0; p < h; p++) for (const [f, m] of i) {
        const g = a + p + f, y = l + u + m;
        if (g >= a && g < a + h && y >= l && y < l + d) continue;
        const w = s.get(oe(g, y));
        w && ii.has(w.name) && ge(t, c, we(w));
      }
    }
    return t;
  }
  function Zd(n, t) {
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
  const Qd = 150, S0 = 64, sr = 0.2, T0 = 5, A0 = 100, E0 = 200, k0 = (n) => 1 - Math.pow(1 - n, 3), M0 = (n) => n;
  function Bi() {
    return new ex({
      dynamicProperties: {
        color: true,
        position: false,
        rotation: false,
        vertex: false,
        uvs: false
      }
    });
  }
  function tu() {
    const n = Bi(), t = new Ut(), e = Bi(), s = Bi(), i = Bi();
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
        n.removeParticles(), t.removeChildren(), e.removeParticles(), s.removeParticles(), i.removeParticles(), ae.clear(), Ss.clear();
      },
      count() {
        return n.particleChildren.length + e.particleChildren.length + s.particleChildren.length + i.particleChildren.length;
      }
    };
  }
  const ae = /* @__PURE__ */ new Map();
  function ds(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function P0(n, t) {
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
      if (!u || !(rn.has(u.name) || fe.has(u.name) && u.io_type === "output" || ie.has(u.name))) continue;
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
  function I0(n, t) {
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
      d && eu(e, s, d) && (r |= l);
    }
    return r;
  }
  function eu(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of Hd(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  const R0 = 9079434;
  function L0(n, t, e) {
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
        if (!d || !eu(o, a, d)) continue;
        const u = o * S + S / 2, p = a * S + S / 2, f = S * 1.5, m = u + l * f, g = p + c * f, y = new ft();
        y.moveTo(u, p).lineTo(m, g).stroke({
          width: i,
          color: R0,
          cap: "round"
        }), s.addChild(y);
      }
    }
  }
  function va(n, t) {
    if (rn.has(n.name)) {
      const s = P0(n, t.tileMap), i = a0(n.name, n.direction ?? "North", s);
      return un(i, gt, gt, (r) => {
        const o = gt / S;
        let a = null;
        s === "corner-cw" ? a = {
          turn: "cw"
        } : s === "corner-ccw" && (a = {
          turn: "ccw"
        });
        const l = ga(n, a);
        l.scale.set(o), r.addChild(l);
      });
    }
    if (fe.has(n.name)) {
      const s = n.io_type ?? "input", i = c0(n.name, n.direction ?? "North", s);
      return un(i, gt, gt, (r) => {
        const o = gt / S, a = Nn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (ie.has(n.name)) {
      const s = h0(n.name, n.direction ?? "North"), i = n.direction === "North" || n.direction === "South";
      return rc(s, i ? 2 : 1, i ? 1 : 2, (a, l, c) => {
        const h = gt / S, d = Nn(n, t);
        d.scale.set(h), d.x = 0, d.y = 0, a.addChild(d);
      });
    }
    if (n.name === "pipe") {
      const s = I0(n, t), i = l0(s);
      return un(i, gt, gt, (r) => {
        const o = gt / S, a = Nn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (n.name === "pipe-to-ground") {
      const s = f0(n.direction ?? "North");
      return un(s, gt, gt, (i) => {
        const r = gt / S, o = Nn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (yr.has(n.name)) {
      const s = d0(n.name, n.direction ?? "North");
      return un(s, gt, gt, (i) => {
        const r = gt / S, o = Nn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (ua.has(n.name)) {
      const s = p0(n.name);
      return un(s, gt, gt, (i) => {
        const r = gt / S, o = Nn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Le.has(n.name)) {
      const [s, i] = Ve[n.name] ?? [
        1,
        1
      ], r = u0(n.name);
      return rc(r, s, i, (o, a, l) => {
        const c = `/spaghettio/entity-frames/${n.name}.png`, h = se.has(c) ? He.get(c) ?? null : null, d = 1.8, u = gt / S0;
        if (h) {
          const p = new je(h);
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
    return un(e, gt, gt, (s) => {
      const i = gt / S, r = Nn(n, t);
      r.scale.set(i), r.x = 0, r.y = 0, s.addChild(r);
    });
  }
  function Go(n, t, e, s = xr()) {
    const i = ds(t);
    if (ae.has(i)) return;
    const r = t.x ?? 0, o = t.y ?? 0, a = va(t, s);
    let l = S / gt, c = S / gt;
    if (ie.has(t.name)) {
      const f = t.direction === "North" || t.direction === "South", m = f ? 2 : 1, g = f ? 1 : 2;
      l = m * S / (m * gt), c = g * S / (g * gt);
    } else if (Le.has(t.name)) {
      const [f, m] = Ve[t.name] ?? [
        1,
        1
      ];
      l = f * S / (f * gt), c = m * S / (m * gt);
    }
    const h = r * S, d = o * S, u = new tr({
      texture: a,
      x: h,
      y: d,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: l,
      scaleY: c
    });
    Le.has(t.name) ? n.machineContainer.addParticle(u) : n.beltContainer.addParticle(u);
    let p;
    if (t.carries && !Le.has(t.name)) {
      const f = jd(t.carries), m = S * 0.35, g = (S - m) / 2;
      p = new tr({
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
    ae.set(i, {
      entity: u,
      icon: p,
      revealAt: e,
      placedEntity: t
    });
  }
  const Ss = /* @__PURE__ */ new Map();
  function $0(n, t, e, s, i, r, o) {
    const a = `${t},${e}`, l = Ss.get(a);
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
    }, h = xr();
    nr(c, h);
    const d = va(c, h), u = In(i), p = new tr({
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
    return n.ghostContainer.addParticle(p), l || Ss.set(a, {
      particle: p,
      specKey: o
    }), p;
  }
  function mc(n, t, e) {
    const s = `${t},${e}`, i = Ss.get(s);
    i && (n.ghostContainer.removeParticle(i.particle), Ss.delete(s));
  }
  function B0(n) {
    n.ghostContainer.removeParticles(), Ss.clear();
  }
  function O0(n, t, e) {
    const s = [];
    for (const [i, r] of ae.entries()) r.placedEntity.x !== t || r.placedEntity.y !== e || (Le.has(r.placedEntity.name) ? n.machineContainer.removeParticle(r.entity) : n.beltContainer.removeParticle(r.entity), r.icon && n.iconContainer.removeParticle(r.icon), ae.delete(i), s.push(i));
    return s;
  }
  function N0(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const [s, i] of ae.entries()) {
      const r = i.placedEntity.name;
      if (r !== "pipe" && !rn.has(r)) continue;
      const o = va(i.placedEntity, t);
      if (i.entity.texture === o) continue;
      const a = i.entity, l = new tr({
        texture: o,
        x: a.x,
        y: a.y,
        alpha: a.alpha,
        anchorX: a.anchorX,
        anchorY: a.anchorY,
        scaleX: a.scaleX,
        scaleY: a.scaleY
      });
      n.beltContainer.removeParticle(a), n.beltContainer.addParticle(l), ae.set(s, {
        ...i,
        entity: l
      }), e.set(a, l);
    }
    return e;
  }
  function nu(n, t) {
    for (const e of ae.values()) {
      const s = Math.min(1, Math.max(0, (t - e.revealAt) / Qd));
      e.entity.alpha = s, e.icon && (e.icon.alpha = s);
    }
  }
  function* gc(n) {
    for (const t of ae.values()) yield {
      particle: t.entity,
      iconParticle: t.icon,
      revealAt: t.revealAt
    };
  }
  function su(n) {
    const t = /* @__PURE__ */ new Map();
    let e = false;
    function s() {
      e || (e = true, n.add(r), br());
    }
    function i() {
      e && (e = false, n.remove(r), Kn());
    }
    function r() {
      const c = performance.now();
      let h = false;
      for (const [d, u] of t) {
        const p = c - u.startTime, f = Math.min(1, p / u.duration), m = u.ease(f), g = u.startAlpha + (u.targetAlpha - u.startAlpha) * m;
        u.entityParticle.alpha = g, u.iconParticle && (u.iconParticle.alpha = g), f >= 1 ? t.delete(d) : h = true;
      }
      h || i(), Ue();
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
        duration: g ? A0 : E0,
        ease: g ? k0 : M0
      }), s();
    }
    function a(c) {
      t.clear();
      for (const h of ae.values()) h.entity.alpha = c, h.icon && (h.icon.alpha = c);
      i(), Ue();
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
  function iu(n) {
    let t = 0;
    for (const i of n.values()) i > t && (t = i);
    const e = Math.max(t, T0), s = /* @__PURE__ */ new Map();
    for (const [i, r] of n) {
      const o = sr + (1 - sr) * (1 - r / e);
      s.set(i, o);
    }
    return s;
  }
  function kn(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function F0(n, t, e) {
    t.clear(), t.layout = n;
    const s = xr();
    for (const l of n.entities) nr(l, s);
    const i = 0;
    for (const l of n.entities) Go(t, l, i, s);
    L0(t, n, s), nu(t, Qd + 1);
    const r = Jd(n);
    if (!e) return W0();
    const o = su(e.ticker);
    function a(l) {
      const c = iu(l);
      for (const h of ae.values()) {
        const d = kn(h.placedEntity), u = c.get(d) ?? sr;
        o.animateTo(d, h.entity, h.icon, u);
      }
    }
    return {
      highlightItem(l) {
        if (o.cancelAll(1), !!l) for (const c of ae.values()) {
          const h = c.placedEntity, u = (h.carries ?? h.recipe ?? null) === l ? 1 : 0.15, p = kn(h);
          o.animateTo(p, c.entity, c.icon, u);
        }
      },
      highlightBeltNetwork(l) {
        if (!l) {
          o.cancelAll(1);
          return;
        }
        const c = kn(l), h = Zd(r, c);
        a(h);
      },
      clearHighlight() {
        for (const l of ae.values()) {
          const c = kn(l.placedEntity);
          o.animateTo(c, l.entity, l.icon, 1);
        }
      },
      chainKey(l) {
        return l.carries ?? l.recipe ?? null;
      }
    };
  }
  function W0() {
    function n() {
      for (const t of ae.values()) t.entity.alpha = 1, t.icon && (t.icon.alpha = 1);
    }
    return {
      highlightItem(t) {
        if (n(), !!t) for (const e of ae.values()) {
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
  function G0(n, t) {
    const e = Jd(n), s = su(t.ticker);
    function i(r) {
      const o = kn(r), a = Zd(e, o), l = iu(a);
      for (const c of ae.values()) {
        const h = kn(c.placedEntity), d = l.get(h) ?? sr;
        s.animateTo(h, c.entity, c.icon, d);
      }
    }
    return {
      highlightItem(r) {
        if (s.cancelAll(1), !!r) for (const o of ae.values()) {
          const a = o.placedEntity, c = (a.carries ?? a.recipe ?? null) === r ? 1 : 0.15;
          o.entity.alpha = c, o.icon && (o.icon.alpha = c);
        }
      },
      highlightBeltNetwork(r) {
        if (!r) {
          for (const o of ae.values()) {
            const a = kn(o.placedEntity);
            s.animateTo(a, o.entity, o.icon, 1);
          }
          return;
        }
        i(r);
      },
      clearHighlight() {
        for (const r of ae.values()) {
          const o = kn(r.placedEntity);
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
  const yc = 6, z0 = 18, D0 = 2, H0 = 26, U0 = -Math.PI / 3, xc = 2, j0 = 16777215, V0 = 0.55, bc = 4, Y0 = 0, X0 = 0.6;
  function q0(n) {
    return n.carries ? rn.has(n.name) || fe.has(n.name) || ii.has(n.name) : false;
  }
  function K0(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const l of t.external_inputs) e.set(l.item, {
      rate: l.rate,
      isFluid: !!l.is_fluid
    });
    if (e.size === 0) return [];
    const s = /* @__PURE__ */ new Map();
    for (const l of n.entities) {
      if (!q0(l)) continue;
      const c = l.carries;
      if (!e.has(c)) continue;
      const h = l.x ?? 0, d = l.y ?? 0, u = s.get(h);
      (!u || d < u.y) && s.set(h, {
        y: d,
        carries: c
      });
    }
    if (s.size === 0) return [];
    const i = Array.from(s.entries()).filter(([, l]) => l.y === Y0).map(([l, c]) => ({
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
  function J0(n) {
    return `${n.toFixed(1)}/s`;
  }
  function Z0(n) {
    const t = n.xMax - n.xMin + 1, e = Math.min(H0, z0 + (t - 1) * D0), s = new Ut();
    s.eventMode = "none";
    const i = jd(n.item), r = new je(i);
    r.width = e, r.height = e, r.x = 0, r.y = -e / 2, s.addChild(r);
    const o = new $e({
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
    }), a = new De({
      text: J0(n.rate),
      style: o
    });
    a.x = e + yc, a.y = -a.height / 2, s.addChild(a);
    const l = new $e({
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
    }), c = new De({
      text: me(n.item),
      style: l
    });
    return c.alpha = X0, c.x = a.x + a.width + yc, c.y = -c.height / 2, s.addChild(c), s;
  }
  function Q0(n, t, e) {
    if (n.removeChildren(), !e) return;
    const s = K0(t, e);
    if (s.length !== 0) for (const i of s) {
      const r = i.topY * S - xc, o = i.xMin * S + bc, a = (i.xMax - i.xMin + 1) * S - 2 * bc;
      if (a > 0) {
        const d = new ft();
        d.rect(o, r, a, xc).fill({
          color: j0,
          alpha: V0
        }), n.addChild(d);
      }
      const l = Z0(i);
      l.rotation = U0;
      const c = (i.xMin + i.xMax + 1) / 2 * S, h = i.topY * S - S * 0.5;
      l.x = c, l.y = h, n.addChild(l);
    }
  }
  function tb(n) {
    return ie.has(n.name) ? n.direction === "East" || n.direction === "West" ? [
      1,
      2
    ] : [
      2,
      1
    ] : Ve[n.name] ?? [
      1,
      1
    ];
  }
  const Qr = 57504;
  function _c(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map();
    for (const _ of s.entities) r.set(`${_.x ?? 0},${_.y ?? 0}`, _);
    let o = null, a = false, l = [];
    const c = new ft();
    e.addChild(c);
    const h = new ft();
    e.addChild(h);
    function d(_, v) {
      const C = n.getBoundingClientRect();
      return t.toWorld(_ - C.left, v - C.top);
    }
    function u(_, v) {
      if (!o) return;
      const C = d(o.sx, o.sy), E = d(_, v), R = Math.min(C.x, E.x), k = Math.min(C.y, E.y), T = Math.abs(E.x - C.x), O = Math.abs(E.y - C.y);
      c.clear(), c.rect(R, k, T, O).fill({
        color: Qr,
        alpha: 0.18
      }), c.setStrokeStyle({
        width: 1,
        color: Qr,
        alpha: 0.8
      }), c.rect(R, k, T, O).stroke(), Ue();
    }
    function p(_) {
      if (h.clear(), _.length !== 0) {
        h.setStrokeStyle({
          width: 1.5,
          color: Qr,
          alpha: 0.9
        });
        for (const v of _) {
          const [C, E] = tb(v), R = (v.x ?? 0) * S + 1, k = (v.y ?? 0) * S + 1;
          h.rect(R, k, C * S - 2, E * S - 2).stroke();
        }
      }
    }
    function f(_, v) {
      if (!o) return [];
      const C = d(o.sx, o.sy), E = d(_, v), R = Math.min(Math.floor(C.x / S), Math.floor(E.x / S)), k = Math.max(Math.floor(C.x / S), Math.floor(E.x / S)), T = Math.min(Math.floor(C.y / S), Math.floor(E.y / S)), O = Math.max(Math.floor(C.y / S), Math.floor(E.y / S)), U = [];
      for (let G = R; G <= k; G++) for (let B = T; B <= O; B++) {
        const L = r.get(`${G},${B}`);
        L && U.push(L);
      }
      return U;
    }
    const m = (_) => {
      _.button !== 0 || !_.shiftKey || (o = {
        sx: _.clientX,
        sy: _.clientY
      }, a = false);
    }, g = (_) => {
      if (!o) return;
      const v = _.clientX - o.sx, C = _.clientY - o.sy;
      !a && v * v + C * C > 36 && (a = true), a && u(_.clientX, _.clientY);
    }, y = (_) => {
      if (_.button === 0) {
        if (a) _.stopImmediatePropagation(), c.clear(), l = f(_.clientX, _.clientY), p(l), i(l), Ue();
        else if (o !== null) {
          const v = d(_.clientX, _.clientY), C = Math.floor(v.x / S), E = Math.floor(v.y / S);
          r.has(`${C},${E}`) && (l = [], h.clear(), i([]), Ue());
        }
        o = null, a = false;
      }
    };
    function w() {
      l = [], c.clear(), h.clear(), i([]), Ue();
    }
    const x = (_) => {
      _.preventDefault(), l.length > 0 && w();
    }, b = (_) => {
      _.key === "Escape" && l.length > 0 && w();
    };
    return n.addEventListener("pointerdown", m, {
      capture: true
    }), n.addEventListener("pointermove", g, {
      capture: true
    }), n.addEventListener("pointerup", y, {
      capture: true
    }), n.addEventListener("contextmenu", x), window.addEventListener("keydown", b), {
      destroy() {
        n.removeEventListener("pointerdown", m, {
          capture: true
        }), n.removeEventListener("pointermove", g, {
          capture: true
        }), n.removeEventListener("pointerup", y, {
          capture: true
        }), n.removeEventListener("contextmenu", x), window.removeEventListener("keydown", b), c.destroy(), h.destroy();
      },
      clear: w,
      getSelected() {
        return [
          ...l
        ];
      },
      buildJson(_, v) {
        return JSON.stringify({
          params: _,
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
          note: v
        }, null, 2);
      }
    };
  }
  const eb = {
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
  }, nb = {
    codes: eb
  }, ru = nb, sb = new Map(Object.entries(ru.codes)), ib = new Map(Object.entries(ru.codes).map(([n, t]) => [
    t,
    n
  ]));
  function rb(n) {
    return sb.get(n) ?? null;
  }
  function ob(n) {
    return ib.get(n) ?? null;
  }
  const ab = [
    "partitioned-per-consumer",
    "partitioned-decomposed"
  ], lb = [
    "horizontal-stack"
  ], cb = [
    "regular",
    "fast"
  ], to = [
    "iron-plate",
    "copper-plate",
    "steel-plate",
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], Ts = [
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], Ca = "iron-gear-wheel", Sa = 10, xs = {
    crafting: "assembling-machine-3",
    smelting: "electric-furnace"
  }, ci = {
    crafting: "craft",
    smelting: "smelt"
  }, hb = "machine", ir = "#/l/", Us = "_", rr = ",", db = {
    pd: "partitioned-decomposed"
  }, wc = {
    "partitioned-decomposed": "pd"
  }, ub = {
    hs: "horizontal-stack"
  }, vc = {
    "horizontal-stack": "hs"
  }, pb = {
    r: "regular",
    f: "fast"
  }, Cc = {
    regular: "r",
    fast: "f"
  };
  function as(n) {
    return rb(n) ?? n;
  }
  function ls(n) {
    return ob(n);
  }
  function fb() {
    const n = window.location.hash;
    if (!n.startsWith(ir)) return null;
    const t = n.slice(ir.length), e = t.indexOf("?"), s = e >= 0 ? t.slice(0, e) : t, i = e >= 0 ? t.slice(e + 1) : "", r = s.split("/"), o = (T) => {
      const O = r[T];
      return O === void 0 || O === "" || O === Us ? null : O;
    }, a = o(0);
    let l;
    if (a) {
      const T = ls(a);
      if (T === null) return null;
      l = T;
    } else l = Ca;
    const c = o(1), h = c !== null ? parseFloat(c) : NaN, d = !isNaN(h) && h > 0 ? h : Sa, u = o(2), p = {};
    if (u) {
      const T = ls(u);
      if (T === null) return null;
      p.crafting = T;
    }
    const f = o(3);
    let m;
    if (f) {
      const T = f.split(rr).filter((U) => U.length > 0), O = [];
      for (const U of T) {
        const G = ls(U);
        if (G === null) return null;
        O.push(G);
      }
      m = O;
    } else m = Ts;
    const g = o(4);
    let y = null;
    if (g && (y = ls(g), y === null)) return null;
    const w = new URLSearchParams(i), x = w.get("s"), b = x ? db[x] ?? null : null, _ = w.get("rl"), v = _ ? ub[_] ?? null : null, C = w.get("it"), E = C ? pb[C] ?? null : null, R = w.get("ci");
    let k = [];
    if (R) {
      const T = R.split(rr).filter((O) => O.length > 0);
      for (const O of T) {
        const U = ls(O);
        if (U === null) return null;
        k.push(U);
      }
    }
    for (const [T, O] of Object.entries(ci)) {
      if (T === "crafting") continue;
      const U = w.get(O);
      if (!U) continue;
      const G = ls(U);
      if (G === null) return null;
      p[T] = G;
    }
    return {
      item: l,
      rate: d,
      machines: p,
      inputs: m,
      belt: y,
      strategy: b,
      rowLayout: v,
      inserterTier: E,
      customInputs: k
    };
  }
  function mb() {
    const n = new URLSearchParams(window.location.search), t = n.get("item") ?? Ca, e = parseFloat(n.get("rate") ?? ""), s = isNaN(e) || e <= 0 ? Sa : e, i = {};
    for (const [y, w] of Object.entries(ci)) {
      const x = n.get(w);
      x && (i[y] = x);
    }
    const r = n.get(hb);
    r && !i.crafting && (i.crafting = r);
    const o = n.get("in"), a = o ? o.split(",").filter((y) => y.length > 0) : Ts, l = n.get("belt"), c = n.get("strategy");
    let h = c && ab.includes(c) ? c : null;
    h === "partitioned-per-consumer" && (h = "partitioned-decomposed");
    const d = n.get("row_layout"), u = d && lb.includes(d) ? d : null, p = n.get("inserter_tier"), f = p && cb.includes(p) ? p : null, m = n.get("ci"), g = m ? m.split(",").filter((y) => y.length > 0) : [];
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
  function gb() {
    return fb() ?? mb();
  }
  function yb() {
    if (window.location.hash.startsWith(ir)) return true;
    const n = new URLSearchParams(window.location.search);
    return n.has("item") || n.has("rate") || n.has("machine") || n.has("in") || n.has("belt");
  }
  function xb(n) {
    for (const [t, e] of Object.entries(ci)) {
      const s = n[t];
      if (s && s !== xs[t]) return false;
    }
    for (const t of Object.keys(n)) if (!(t in ci)) return false;
    return true;
  }
  function bb(n) {
    const t = as(n.item), e = String(n.rate), s = n.machines.crafting, i = !s || s === xs.crafting ? Us : as(s), r = n.inputs.length === Ts.length && n.inputs.every((u, p) => u === Ts[p]), o = n.inputs.length === 0 || r ? Us : n.inputs.map(as).join(rr), a = n.belt ? as(n.belt) : Us, l = new URLSearchParams();
    n.strategy && wc[n.strategy] && l.set("s", wc[n.strategy]), n.rowLayout && vc[n.rowLayout] && l.set("rl", vc[n.rowLayout]), n.inserterTier && Cc[n.inserterTier] && l.set("it", Cc[n.inserterTier]), n.customInputs.length > 0 && l.set("ci", n.customInputs.map(as).join(rr));
    for (const [u, p] of Object.entries(ci)) {
      if (u === "crafting") continue;
      const f = n.machines[u];
      f && f !== xs[u] && l.set(p, as(f));
    }
    const c = [
      t,
      e,
      i,
      o,
      a
    ], h = l.toString();
    if (h.length === 0) for (; c.length > 2 && c[c.length - 1] === Us; ) c.pop();
    let d = `${ir}${c.join("/")}`;
    return h.length > 0 && (d += `?${h}`), d;
  }
  function _b(n) {
    const t = n.item === Ca && n.rate === Sa && xb(n.machines) && n.inputs.length === Ts.length && n.inputs.every((s, i) => s === Ts[i]) && !n.belt && !n.strategy && !n.rowLayout && !n.inserterTier && n.customInputs.length === 0, e = window.location.pathname;
    if (t) {
      history.replaceState(null, "", e);
      return;
    }
    history.replaceState(null, "", e + bb(n));
  }
  const eo = "[INCOMPATIBLE_MACHINE]";
  function mn(n, t = 14) {
    const e = document.createElement("img");
    return e.src = `/spaghettio/icons/${n}.png`, e.width = t, e.height = t, e.style.cssText = "image-rendering:pixelated", e.onerror = () => {
      e.style.display = "none";
    }, e;
  }
  function wb(n, t) {
    const e = document.createElement("option");
    return e.value = n, e.textContent = me(n), n === t && (e.selected = true), e;
  }
  function Oi(n, t, e) {
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
  function Sc(n, t, e, s) {
    for (const i of t) {
      const r = document.createElement("div");
      r.className = `sb-machine-flow ${e}`, s && r.appendChild(document.createTextNode(s)), r.appendChild(mn(i.item, 13)), r.appendChild(document.createTextNode(me(i.item)));
      const o = document.createElement("span");
      o.className = "flow-rate";
      const a = Kd(i.rate), l = a ? qd(a.color) : "#f88";
      o.style.color = l, o.textContent = `${i.rate.toFixed(1)}/s`, r.appendChild(o), n.appendChild(r);
    }
  }
  const Tc = /* @__PURE__ */ new Set([
    "water",
    "crude-oil",
    "petroleum-gas",
    "light-oil",
    "heavy-oil",
    "sulfuric-acid",
    "lubricant",
    "steam"
  ]), ou = "spaghettio-recent-items", Ac = 5;
  function au() {
    try {
      const n = localStorage.getItem(ou);
      return n ? JSON.parse(n) : [];
    } catch {
      return [];
    }
  }
  function vb(n) {
    const t = au().filter((e) => e !== n);
    t.unshift(n), t.length > Ac && (t.length = Ac);
    try {
      localStorage.setItem(ou, JSON.stringify(t));
    } catch {
    }
  }
  function Cb(n, t, e) {
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
        a.appendChild(mn(s, 14));
        const x = document.createElement("span");
        x.textContent = me(s), a.appendChild(x);
      } else {
        const x = document.createElement("span");
        x.className = "sb-picker-placeholder", x.textContent = "Select item\u2026", a.appendChild(x);
      }
    }
    function p(x) {
      const b = document.createElement("div");
      b.className = "sb-picker-item" + (x === s ? " selected" : ""), b.dataset.slug = x, b.appendChild(mn(x, 14));
      const _ = document.createElement("span");
      return _.textContent = me(x), b.appendChild(_), b.addEventListener("mousedown", (v) => {
        v.preventDefault(), m(x);
      }), b;
    }
    function f(x) {
      d.innerHTML = "", r = null;
      const b = x.trim().toLowerCase(), _ = b ? n.filter((v) => v.includes(b) || me(v).toLowerCase().includes(b)) : n;
      if (!b) {
        const v = au().filter((C) => n.includes(C));
        if (v.length > 0) {
          const C = document.createElement("div");
          C.className = "sb-picker-section-label", C.textContent = "Recent", d.appendChild(C);
          for (const R of v) d.appendChild(p(R));
          const E = document.createElement("div");
          E.className = "sb-picker-divider", d.appendChild(E);
        }
      }
      for (const v of _) d.appendChild(p(v));
      if (!b && s) {
        const v = d.querySelector(`[data-slug="${s}"]`);
        v && v.scrollIntoView({
          block: "nearest"
        });
      }
    }
    function m(x) {
      s = x, vb(x), o.classList.remove("item-invalid"), u(), y(), e(x);
    }
    function g() {
      i = true, o.classList.add("open"), c.style.display = "", l.textContent = "\u25B4", h.value = "", f(""), requestAnimationFrame(() => h.focus());
    }
    function y() {
      i = false, o.classList.remove("open"), c.style.display = "none", l.textContent = "\u25BE", r = null;
    }
    function w(x) {
      const b = d.querySelectorAll(".sb-picker-item");
      if (!b.length) return;
      const _ = Array.from(b);
      let v = r ? _.indexOf(r) : -1;
      v = Math.max(0, Math.min(_.length - 1, v + x)), r == null ? void 0 : r.classList.remove("highlighted"), r = _[v], r.classList.add("highlighted"), r.scrollIntoView({
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
  function Sb(n, t, e) {
    var _a2;
    n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "sidebar-inner";
    const { section: i, body: r } = Oi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="2"/></svg>', "Target"), o = t.allProducibleItems(), a = new Set(o);
    function l(H, J) {
      const it = document.createElement("div");
      it.className = "sb-field";
      const pt = document.createElement("span");
      return pt.className = "sb-field-label", pt.textContent = H, it.appendChild(pt), J.style.flex = "1", J.style.minWidth = "0", it.appendChild(J), it;
    }
    const c = Cb(o, "", () => Tt());
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
    for (const H of h) {
      const J = document.createElement("select");
      J.className = "sb-select", J.dataset.cat = H.category;
      const it = xs[H.category] ?? "";
      for (const pt of H.options) {
        const Lt = wb(pt.value, it);
        pt.disabled && (Lt.disabled = true), pt.title && (Lt.title = pt.title), J.appendChild(Lt);
      }
      r.appendChild(l(H.label, J)), u.set(H.category, J);
    }
    const p = u.get("crafting");
    for (const H of d) {
      const J = document.createElement("span");
      J.className = "sb-machine-readonly", J.textContent = me(H.machine), r.appendChild(l(H.label, J));
    }
    function f() {
      const H = {};
      for (const [J, it] of u) it.value && (H[J] = it.value);
      return H;
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
    ].forEach(([H, J]) => {
      const it = document.createElement("option");
      it.value = J, it.textContent = H, m.appendChild(it);
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
    ].forEach(([H, J]) => {
      const it = document.createElement("option");
      it.value = J, it.textContent = H, g.appendChild(it);
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
    ].forEach(([H, J]) => {
      const it = document.createElement("option");
      it.value = J, it.textContent = H, y.appendChild(it);
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
    ].forEach(([H, J]) => {
      const it = document.createElement("option");
      it.value = J, it.textContent = H, w.appendChild(it);
    }), r.appendChild(l("Row layout", w));
    const x = document.createElement("div");
    x.className = "sb-field";
    const b = document.createElement("span");
    b.className = "sb-field-label", b.textContent = "Rate", x.appendChild(b);
    const _ = document.createElement("input");
    _.type = "number", _.className = "sb-input", _.step = "0.5", _.min = "0.1", _.style.cssText = "flex:1;min-width:0", _.placeholder = "10", x.appendChild(_);
    const v = document.createElement("span");
    v.className = "sb-rate-suffix", v.textContent = "/s", x.appendChild(v), r.appendChild(x), s.appendChild(i);
    const { section: C, body: E } = Oi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="6" rx="1"/><line x1="5" y1="8" x2="11" y2="8"/></svg>', "Inputs"), R = document.createElement("div");
    R.className = "sb-tags";
    const k = /* @__PURE__ */ new Map();
    to.forEach((H) => {
      const J = document.createElement("label");
      J.className = `sb-tag${Tc.has(H) ? " fluid" : ""}`;
      const it = document.createElement("span");
      it.className = "sb-tag-check", it.textContent = "\u2713";
      const pt = document.createElement("input");
      pt.type = "checkbox", pt.value = H, pt.style.display = "none", k.set(H, pt), J.appendChild(it), J.appendChild(mn(H, 14)), J.appendChild(document.createTextNode(me(H))), J.appendChild(pt), pt.addEventListener("change", () => {
        J.classList.toggle("active", pt.checked);
      }), R.appendChild(J);
    }), E.appendChild(R);
    let T = [];
    const O = document.createElement("div");
    O.className = "sb-tags sb-custom-tags", E.appendChild(O);
    const U = document.createElement("datalist");
    U.id = "spaghettio-custom-inputs-datalist";
    const G = new Set(to);
    o.filter((H) => !G.has(H)).forEach((H) => {
      const J = document.createElement("option");
      J.value = H, U.appendChild(J);
    }), E.appendChild(U);
    const B = document.createElement("input");
    B.type = "text", B.className = "sb-input sb-custom-input-field", B.setAttribute("list", "spaghettio-custom-inputs-datalist"), B.autocomplete = "off", B.placeholder = "+ add input\u2026", E.appendChild(B);
    function L(H) {
      const J = document.createElement("div");
      J.className = `sb-tag sb-custom-tag active${Tc.has(H) ? " fluid" : ""}`, J.dataset.item = H, J.appendChild(mn(H, 14)), J.appendChild(document.createTextNode(me(H)));
      const it = document.createElement("span");
      it.className = "sb-tag-remove", it.textContent = "\xD7", it.addEventListener("click", (pt) => {
        pt.stopPropagation(), T = T.filter((Lt) => Lt !== H), J.remove(), Tt();
      }), J.appendChild(it), O.appendChild(J);
    }
    function D(H) {
      const J = H.trim();
      !J || !a.has(J) || G.has(J) || T.includes(J) || (T.push(J), L(J), B.value = "", Tt());
    }
    B.addEventListener("keydown", (H) => {
      H.key === "Enter" && D(B.value);
    }), B.addEventListener("change", () => {
      D(B.value);
    }), s.appendChild(C);
    const { section: K, body: X, countEl: Z } = Oi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>', "Solver", ""), M = document.createElement("div");
    X.appendChild(M), s.appendChild(K);
    const N = document.createElement("div");
    N.className = "sb-actions", N.style.display = "none";
    const W = document.createElement("button");
    W.className = "sb-btn sb-btn-secondary", W.textContent = "Copy Blueprint", W.style.flex = "1", N.appendChild(W);
    const z = document.createElement("div");
    z.className = "sb-copy-status", N.appendChild(z), X.appendChild(N);
    const { section: q, body: Q, countEl: at } = Oi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.8" fill="currentColor" stroke="none"/></svg>', "Validation", "");
    q.style.display = "none", s.appendChild(q), n.appendChild(s);
    const st = gb();
    c.setValue(st.item), _.value = String(st.rate);
    const mt = /* @__PURE__ */ new Set([
      "assembling-machine-1",
      "assembling-machine-2",
      "assembling-machine-3"
    ]);
    for (const [H, J] of u) {
      const it = st.machines[H], pt = new Set(Array.from(J.options).filter((Lt) => !Lt.disabled).map((Lt) => Lt.value));
      J.value = it && pt.has(it) ? it : xs[H] ?? ((_a2 = J.options[0]) == null ? void 0 : _a2.value) ?? "";
    }
    k.forEach((H, J) => {
      H.checked = st.inputs.includes(J);
      const it = H.closest(".sb-tag");
      it && it.classList.toggle("active", H.checked);
    }), st.belt && (m.value = st.belt), st.strategy && (y.value = st.strategy), st.rowLayout && (w.value = st.rowLayout), st.inserterTier && (g.value = st.inserterTier);
    for (const H of st.customInputs) a.has(H) && !G.has(H) && !T.includes(H) && (T.push(H), L(H));
    const _t = document.createElement("div");
    _t.className = "sb-config-error", _t.style.display = "none", M.before(_t);
    function Y(H) {
      H ? (_t.textContent = H, _t.style.display = "") : (_t.textContent = "", _t.style.display = "none");
    }
    let tt = null, ot = st.item, dt = null, bt = 0;
    function Tt() {
      tt !== null && clearTimeout(tt), tt = setTimeout(() => {
        jt().catch((H) => console.error("runSolve failed:", H));
      }, 150);
    }
    async function jt() {
      var _a3;
      const H = c.getValue(), J = parseFloat(_.value), it = to.filter((kt) => {
        var _a4;
        return (_a4 = k.get(kt)) == null ? void 0 : _a4.checked;
      }), pt = [
        ...it,
        ...T
      ];
      if (!a.has(H)) {
        c.setInvalid(true);
        return;
      }
      if (c.setInvalid(false), isNaN(J) || J <= 0) return;
      if (H !== ot) {
        const kt = t.defaultMachineForItem(H, p.value);
        mt.has(kt) && (p.value = kt), ot = H;
      }
      const Lt = f();
      _b({
        item: H,
        rate: J,
        machines: Lt,
        inputs: it,
        belt: m.value || null,
        strategy: y.value || null,
        rowLayout: w.value || null,
        inserterTier: g.value || null,
        customInputs: T
      });
      const Ot = ++bt;
      M.innerHTML = "", Y(null), dt = null, N.style.display = "none";
      let Dt;
      try {
        Dt = await t.solve(H, J, pt, Lt, Lt.crafting ?? xs.crafting);
      } catch (kt) {
        if (Ot !== bt) return;
        e.renderGraph(null), Z && (Z.textContent = "error");
        const Mt = String(kt instanceof Error ? kt.message : kt);
        if (Mt.includes(eo)) {
          const Wt = Mt.indexOf(eo), de = Mt.slice(Wt + eo.length).trim();
          Y(de);
        } else {
          const Wt = document.createElement("div");
          Wt.className = "sb-result-error", Wt.textContent = Mt, M.appendChild(Wt);
        }
        return;
      }
      if (Ot !== bt) return;
      Tb(M, Dt), e.renderGraph(Dt);
      const Zt = Dt.machines.reduce((kt, Mt) => kt + Math.ceil(Mt.count), 0);
      Z && (Z.textContent = `${Zt} machines`);
      const Xt = /* @__PURE__ */ new Set();
      for (const kt of Dt.machines) {
        for (const Mt of kt.inputs) Xt.add(Mt.item);
        for (const Mt of kt.outputs) Xt.add(Mt.item);
      }
      for (const kt of Dt.external_inputs) Xt.add(kt.item);
      for (const kt of Dt.external_outputs) Xt.add(kt.item);
      if (await Dd(Array.from(Xt)), Ot !== bt) return;
      let re;
      try {
        const kt = m.value || void 0, Mt = y.value || void 0, Wt = w.value || void 0, de = g.value || void 0, A = e.startStreaming();
        re = await t.buildLayoutStreaming(Dt, kt, Mt, Wt, de, A);
      } catch (kt) {
        if (Ot !== bt) return;
        const Mt = document.createElement("div");
        Mt.className = "sb-result-error", Mt.textContent = `Layout error: ${kt}`, M.appendChild(Mt);
        return;
      }
      Ot === bt && (dt = re, Pd(Dt.machines), e.renderLayout(re, Dt), N.style.display = ((_a3 = re.warnings) == null ? void 0 : _a3.length) ? "none" : "flex");
    }
    W.addEventListener("click", async () => {
      if (!dt) return;
      const H = await t.exportBlueprint(dt, c.getValue());
      await navigator.clipboard.writeText(H), z.textContent = "Copied!", setTimeout(() => {
        z.textContent = "";
      }, 2e3);
    }), _.addEventListener("input", Tt);
    for (const H of u.values()) H.addEventListener("change", Tt);
    return m.addEventListener("change", Tt), y.addEventListener("change", Tt), w.addEventListener("change", Tt), g.addEventListener("change", Tt), k.forEach((H) => H.addEventListener("change", Tt)), jt().catch((H) => console.error("runSolve failed:", H)), {
      getParams() {
        const H = c.getValue(), J = parseFloat(_.value);
        return !H || isNaN(J) || J <= 0 ? null : {
          item: H,
          rate: J
        };
      },
      setParams(H, J) {
        c.setValue(H.item), _.value = String(H.rate), H.machine && mt.has(H.machine) ? p.value = H.machine : p.value = "assembling-machine-3", H.inputs && k.forEach((it, pt) => {
          it.checked = H.inputs.includes(pt);
          const Lt = it.closest(".sb-tag");
          Lt && Lt.classList.toggle("active", it.checked);
        }), H.belt ? m.value = H.belt : m.value = "", O.innerHTML = "", T = [];
        for (const it of H.customInputs ?? []) a.has(it) && !G.has(it) && !T.includes(it) && (T.push(it), L(it));
        ot = H.item, (J == null ? void 0 : J.skipAutoSolve) || Tt();
      },
      updateValidation(H, J, it) {
        if (Q.innerHTML = "", H.length === 0) {
          q.style.display = "none", at && (at.textContent = "");
          return;
        }
        q.style.display = "";
        const pt = H.filter((Dt) => Dt.severity === "Error").length, Lt = H.length - pt;
        at && (pt > 0 ? (at.textContent = `${pt} error${pt !== 1 ? "s" : ""}`, at.style.color = "#f66") : (at.textContent = `${Lt} warning${Lt !== 1 ? "s" : ""}`, at.style.color = "#fa0"));
        const Ot = /* @__PURE__ */ new Map();
        for (const Dt of H) {
          let Zt = Ot.get(Dt.category);
          Zt || (Zt = [], Ot.set(Dt.category, Zt)), Zt.push(Dt);
        }
        for (const [Dt, Zt] of Ot) {
          const re = Zt.some((P) => P.severity === "Error") ? "#f44" : "#fa0", kt = Zt.find((P) => P.x != null && P.y != null), Mt = document.createElement("div");
          Mt.className = "sb-val-group";
          const Wt = document.createElement("div");
          Wt.className = "sb-val-group-header";
          const de = document.createElement("span");
          de.className = "sb-val-group-chevron", de.textContent = "\u25BE", de.addEventListener("click", (P) => {
            P.stopPropagation();
            const $ = rt.style.display === "none";
            rt.style.display = $ ? "" : "none", de.textContent = $ ? "\u25BE" : "\u25B8";
          }), Wt.appendChild(de);
          const A = document.createElement("span");
          A.className = "sb-val-group-dot", A.style.background = re, Wt.appendChild(A);
          const I = document.createElement("span");
          I.className = "sb-val-group-name", I.textContent = Dt, Wt.appendChild(I);
          const j = document.createElement("span");
          j.className = "sb-val-group-count", j.textContent = String(Zt.length), Wt.appendChild(j);
          const rt = document.createElement("div");
          if (rt.className = "sb-val-group-body", it) {
            const P = Zt.filter(($) => $.detail != null);
            if (P.length > 0) {
              const $ = /* @__PURE__ */ new Map();
              for (const V of P) {
                const lt = it(V) ?? "unattributed";
                $.set(lt, ($.get(lt) ?? 0) + 1);
              }
              const et = document.createElement("div");
              et.className = "sb-val-cause-rollup", et.style.cssText = "font-size:11px;color:#b90;padding:1px 0 2px 22px", et.textContent = [
                ...$.entries()
              ].sort((V, lt) => lt[1] - V[1]).map(([V, lt]) => `${lt} \xD7 ${V}`).join(" \xB7 "), rt.appendChild(et);
            }
          }
          kt && (Wt.classList.add("clickable"), Wt.addEventListener("click", () => {
            J(kt.x, kt.y);
          }));
          for (const P of Zt) {
            const $ = document.createElement("div"), et = P.x != null && P.y != null;
            $.className = "sb-val-issue" + (et ? " clickable" : "");
            const V = document.createElement("span");
            if (V.className = "sb-val-issue-msg", V.textContent = P.message, $.appendChild(V), et) {
              const lt = document.createElement("span");
              lt.className = "sb-val-issue-coord", lt.textContent = `${P.x}, ${P.y}`, $.appendChild(lt), $.addEventListener("click", (wt) => {
                wt.stopPropagation();
                const At = $.classList.contains("pinned");
                Q.querySelectorAll(".sb-val-issue.pinned").forEach((Bt) => Bt.classList.remove("pinned")), At || $.classList.add("pinned"), J(P.x, P.y);
              });
            } else $.style.opacity = "0.6";
            rt.appendChild($);
          }
          Mt.appendChild(Wt), Mt.appendChild(rt), Q.appendChild(Mt);
        }
      }
    };
  }
  function Tb(n, t) {
    if (t.external_inputs.length > 0) {
      const l = document.createElement("div");
      l.className = "sb-ext-section-title", l.textContent = "External inputs", n.appendChild(l);
      for (const h of t.external_inputs) {
        const d = document.createElement("div");
        d.className = "sb-ext-flow", d.appendChild(mn(h.item, 14)), d.appendChild(document.createTextNode(me(h.item)));
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
      u.className = "sb-machine-group-header", u.appendChild(mn(l, 16));
      const p = document.createElement("span");
      p.className = "sb-machine-group-name", p.textContent = me(l), u.appendChild(p);
      const f = document.createElement("span");
      f.className = "sb-machine-group-count", f.textContent = `\xD7${h}`, u.appendChild(f), d.appendChild(u);
      const m = document.createElement("div");
      m.className = "sb-machine-group-body";
      for (const g of c) {
        const y = document.createElement("div");
        y.className = "sb-machine-flow", y.style.cssText = "color:#6b7280;margin-bottom:2px", y.appendChild(document.createTextNode("\u2192 ")), y.appendChild(mn(g.recipe, 13)), y.appendChild(document.createTextNode(me(g.recipe))), m.appendChild(y), Sc(m, g.inputs, "flow-in", "\u25B6 "), Sc(m, g.outputs, "flow-out", "\u25C0 ");
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
      c.style.cssText = "display:flex;align-items:center;gap:4px;padding:2px 0;font-size:11px;color:#b5cea8", c.appendChild(mn(l.item, 13)), c.appendChild(document.createTextNode(me(l.item)));
      const h = Kd(l.rate);
      if (h) {
        const d = qd(h.color);
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
  let Re = null, Ta = 0;
  const ts = /* @__PURE__ */ new Map();
  let he = null;
  const zo = "spaghettio:sat-cache:v1";
  let fn = new Uint8Array(0);
  function Ab() {
    try {
      const n = localStorage.getItem(zo);
      if (!n) return new Uint8Array(0);
      const t = atob(n), e = new Uint8Array(t.length);
      for (let s = 0; s < t.length; s++) e[s] = t.charCodeAt(s);
      return e;
    } catch (n) {
      return console.warn("[engine] could not read SAT cache from localStorage", n), new Uint8Array(0);
    }
  }
  function Eb(n) {
    let t = "";
    for (let i = 0; i < n.length; i += 8192) t += String.fromCharCode.apply(null, Array.from(n.subarray(i, i + 8192)));
    const s = btoa(t);
    try {
      localStorage.setItem(zo, s);
    } catch (i) {
      if (i instanceof DOMException && (i.name === "QuotaExceededError" || i.code === 22)) {
        console.warn("[engine] SAT cache quota exceeded \u2014 clearing");
        try {
          localStorage.removeItem(zo);
        } catch {
        }
        fn = new Uint8Array(0);
      } else console.warn("[engine] failed to persist SAT cache", i);
    }
  }
  function kb(n) {
    const t = new Uint8Array(fn.length + n.length);
    t.set(fn, 0), t.set(n, fn.length), fn = t, Eb(fn);
  }
  let Do = [], lu = [], cu = /* @__PURE__ */ new Map(), Ho = /* @__PURE__ */ new Set(), Uo = 0;
  function xn(n) {
    Uo += n;
    for (const t of Ho) t(Uo);
  }
  function Mb(n) {
    return Ho.add(n), n(Uo), () => Ho.delete(n);
  }
  function Se(n, t) {
    if (!Re) throw new Error("Engine not initialized \u2014 call initEngine() first");
    const e = ++Ta;
    return xn(1), new Promise((s, i) => {
      ts.set(e, {
        resolve: (r) => {
          xn(-1), he === e && (he = null), s(r);
        },
        reject: (r) => {
          xn(-1), he === e && (he = null), i(r);
        },
        onEvent: t
      }), Re.postMessage({
        id: e,
        ...n
      });
    });
  }
  async function hu() {
    if (Re) return;
    if (Re = new Worker(new URL("/spaghettio/assets/engine.worker-BTE_NCxz.js", import.meta.url), {
      type: "module",
      name: "spaghettio-engine"
    }), Re.onmessage = (t) => {
      const { id: e } = t.data, s = ts.get(e);
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
        ts.delete(e), t.data.ok ? s.resolve(t.data.result) : s.reject(new Error(t.data.error));
      }
    }, Re.onerror = (t) => {
      console.error("[engine.worker] error", t);
    }, await Se({
      method: "init"
    }), fn = Ab(), fn.length > 0) try {
      const t = await Se({
        method: "seedZoneCache",
        bytes: fn
      });
      globalThis.__TRACE_LOGS === true && console.log(`[engine] seeded ${t} SAT zone records from localStorage`);
    } catch (t) {
      console.warn("[engine] seedZoneCache failed; persistence disabled this session", t);
    }
    Do = await Se({
      method: "allProducibleItems"
    }), lu = await Se({
      method: "allProducerMachines"
    });
    const n = await Se({
      method: "defaultMachinesForItems",
      items: Do,
      fallback: "assembling-machine-3"
    });
    cu = new Map(n);
  }
  async function Pb(n, t, e, s, i) {
    return he !== null && await Aa(), Se({
      method: "solve",
      targetItem: n,
      targetRate: t,
      availableInputs: e,
      palette: s,
      defaultMachine: i
    });
  }
  function Ib() {
    return Do;
  }
  function Rb() {
    return lu;
  }
  function Lb(n, t, e, s, i) {
    return Se({
      method: "layout",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null
    });
  }
  function $b(n, t, e, s, i) {
    return Se({
      method: "layoutTraced",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null
    });
  }
  async function Aa() {
    if (!Re) return;
    Re.terminate(), Re = null;
    const n = new Error("Engine superseded by a newer request");
    for (const [, t] of ts) t.reject(n);
    ts.clear(), he = null, await hu();
  }
  async function Bb(n, t, e, s, i, r) {
    he !== null && await Aa();
    const o = ++Ta;
    return he = o, xn(1), new Promise((a, l) => {
      ts.set(o, {
        resolve: (h) => {
          xn(-1), he === o && (he = null), a(h);
        },
        reject: (h) => {
          xn(-1), he === o && (he = null), l(h);
        },
        onEvent: r
      });
      const c = globalThis.__TRACE_LOGS === true;
      Re.postMessage({
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
  function Ob(n, t) {
    return Se({
      method: "exportBlueprint",
      layout: n,
      label: t
    });
  }
  function Nb(n, t) {
    return cu.get(n) ?? t;
  }
  function Fb(n, t) {
    return Se({
      method: "validateLayout",
      layout: n,
      solverResult: t
    });
  }
  function Wb(n, t) {
    return Se({
      method: "solveFixture",
      fixtureJson: n,
      pinsJson: JSON.stringify(t)
    });
  }
  function Gb(n) {
    return Se({
      method: "parseBlueprint",
      bp: n
    });
  }
  async function du(n, t, e, s, i = 0) {
    if (he !== null && await Aa(), !Re) throw new Error("Engine not initialized");
    const r = ++Ta;
    return he = r, xn(1), new Promise((o, a) => {
      ts.set(r, {
        resolve: (l) => {
          xn(-1), he === r && (he = null), o(l);
        },
        reject: (l) => {
          xn(-1), he === r && (he = null), a(l);
        },
        onEvent: (l) => {
          const c = l;
          if (c.phase === "SatImprovement" && c.data) s(c.data);
          else if (c.phase === "SatOptimumProven" && c.data) {
            const h = c.data, d = h.record_bytes instanceof Uint8Array ? h.record_bytes : new Uint8Array(h.record_bytes);
            kb(d);
          }
        }
      }), Re.postMessage({
        id: r,
        method: "improveRegionStreaming",
        layout: n,
        regionId: t,
        budgetMs: e,
        maxIters: i
      });
    });
  }
  async function zb(n, t) {
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
        e = await du(e, o, t.perRegionBudgetMs, (l) => {
          var _a2;
          l.iter > 0 && (a = true), (_a2 = t.onImprovement) == null ? void 0 : _a2.call(t, l);
        }, 1), a ? i += 1 : s.delete(o);
      }
      if (i === 0) break;
    }
    return e;
  }
  function Db(n, t) {
    return Se({
      method: "balancerShowcase",
      maxInputs: n,
      maxOutputs: t
    });
  }
  function Hb() {
    return {
      solve: Pb,
      allProducibleItems: Ib,
      allProducerMachines: Rb,
      buildLayout: Lb,
      buildLayoutTraced: $b,
      buildLayoutStreaming: Bb,
      exportBlueprint: Ob,
      defaultMachineForItem: Nb,
      validateLayout: Fb,
      solveFixture: Wb,
      improveRegion: du,
      optimizeAllRegions: zb,
      balancerShowcase: Db
    };
  }
  const Ub = `
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
  function jb() {
    if (document.getElementById("spaghettio-corpus-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-corpus-style", n.textContent = Ub, document.head.appendChild(n);
  }
  function Vb(n, t) {
    jb(), n.innerHTML = "";
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
      const M = g.value.trim(), N = ++w;
      if (!M) {
        g.classList.remove("error"), y.style.display = "none";
        return;
      }
      Gb(M).then((W) => {
        N === w && (g.classList.remove("error"), y.style.display = "none", t(W));
      }).catch((W) => {
        N === w && (g.classList.add("error"), y.textContent = String(W), y.style.display = "block");
      });
    });
    const x = document.createElement("div");
    x.className = "corpus-filters", x.style.display = "none";
    const b = document.createElement("div");
    b.className = "corpus-filter-row";
    const _ = document.createElement("label");
    _.textContent = "Search";
    const v = document.createElement("input");
    v.type = "text", v.placeholder = "filter by name\u2026", b.appendChild(_), b.appendChild(v), x.appendChild(b);
    const C = document.createElement("div");
    C.className = "corpus-filter-row";
    const E = document.createElement("label");
    E.textContent = "Product";
    const R = document.createElement("select");
    C.appendChild(E), C.appendChild(R), x.appendChild(C);
    const k = document.createElement("div");
    k.className = "corpus-filter-row";
    const T = document.createElement("input");
    T.type = "checkbox";
    const O = document.createElement("label");
    O.style.display = "flex", O.style.alignItems = "center", O.style.gap = "5px", O.style.cursor = "pointer", O.appendChild(T), O.appendChild(document.createTextNode("Bus layouts only")), k.appendChild(O), x.appendChild(k), e.appendChild(x);
    const U = document.createElement("div");
    U.className = "corpus-count", U.style.display = "none", e.appendChild(U);
    const G = document.createElement("div");
    G.className = "corpus-list", e.appendChild(G);
    const B = document.createElement("div");
    B.className = "corpus-stats", e.appendChild(B);
    function L() {
      i = s.filter((M) => !(o && !M.stats.is_bus_layout || a && a !== "__all__" && M.stats.final_product !== a || l && !M.name.toLowerCase().includes(l.toLowerCase()))), r = -1, D();
    }
    function D() {
      if (G.innerHTML = "", U.textContent = `${i.length} of ${s.length} blueprint(s)`, i.length === 0) {
        const M = document.createElement("div");
        M.className = "corpus-empty", M.textContent = s.length === 0 ? "No corpus loaded yet." : "No blueprints match the current filters.", G.appendChild(M), B.classList.remove("visible");
        return;
      }
      for (let M = 0; M < i.length; M++) {
        const N = i[M], W = document.createElement("div");
        W.className = "corpus-item" + (M === r ? " selected" : "");
        const z = document.createElement("div");
        z.className = "corpus-item-name", z.textContent = N.name, z.title = N.name, W.appendChild(z);
        const q = document.createElement("div");
        if (q.className = "corpus-item-meta", N.stats.is_bus_layout) {
          const st = document.createElement("span");
          st.className = "corpus-badge corpus-badge-bus", st.textContent = "BUS", q.appendChild(st);
        }
        if (N.stats.final_product) {
          const st = document.createElement("span");
          st.className = "corpus-badge corpus-badge-product", st.textContent = N.stats.final_product, q.appendChild(st);
        }
        const Q = document.createElement("span");
        Q.className = "corpus-badge corpus-badge-machines", Q.textContent = `${N.stats.machine_count}m`, q.appendChild(Q), W.appendChild(q);
        const at = M;
        W.addEventListener("click", () => K(at)), G.appendChild(W);
      }
    }
    function K(M) {
      r = M;
      const N = i[M];
      D(), t(N.layout), X(N);
    }
    function X(M) {
      B.innerHTML = "", B.classList.add("visible");
      const N = document.createElement("div");
      N.className = "corpus-stats-title", N.textContent = M.name, N.title = M.name, B.appendChild(N);
      const W = document.createElement("div");
      W.className = "corpus-stats-grid";
      const z = M.stats, q = [
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
      z.is_bus_layout && q.push([
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
      ]), q.push([
        "bbox",
        `${z.bbox_width}\xD7${z.bbox_height}`
      ], [
        "belt_tiles",
        String(z.belt_tiles)
      ]), z.pipe_tiles > 0 && q.push([
        "pipe_tiles",
        String(z.pipe_tiles)
      ]);
      for (const [Q, at] of q) {
        const st = document.createElement("div");
        st.className = "corpus-stats-row";
        const mt = document.createElement("span");
        mt.className = "corpus-stats-key", mt.textContent = Q;
        const _t = document.createElement("span");
        _t.className = "corpus-stats-val", _t.textContent = at, st.appendChild(mt), st.appendChild(_t), W.appendChild(st);
      }
      B.appendChild(W);
    }
    function Z() {
      const M = new Set(s.map((W) => W.stats.final_product).filter(Boolean));
      R.innerHTML = "";
      const N = document.createElement("option");
      N.value = "__all__", N.textContent = "All products", R.appendChild(N);
      for (const W of Array.from(M).sort()) {
        const z = document.createElement("option");
        z.value = W, z.textContent = W, R.appendChild(z);
      }
    }
    u.addEventListener("change", () => {
      var _a2;
      const M = (_a2 = u.files) == null ? void 0 : _a2[0];
      if (!M) return;
      const N = new FileReader();
      N.onload = (W) => {
        var _a3;
        try {
          s = JSON.parse((_a3 = W.target) == null ? void 0 : _a3.result).blueprints ?? [], i = s, r = -1, p.textContent = `Reload corpus.json (${s.length} blueprints)`, x.style.display = "", U.style.display = "", B.classList.remove("visible"), Z(), L();
        } catch (z) {
          alert(`Failed to parse corpus.json: ${z}`);
        }
      }, N.readAsText(M);
    }), v.addEventListener("input", () => {
      l = v.value, L();
    }), R.addEventListener("change", () => {
      a = R.value, L();
    }), T.addEventListener("change", () => {
      o = T.checked, L();
    }), D();
  }
  const Yb = [
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
  ], Xb = `
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
  function qb() {
    if (document.getElementById("spaghettio-landing-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-landing-style", n.textContent = Xb, document.head.appendChild(n);
  }
  function Kb(n, t, e) {
    qb(), n.innerHTML = "";
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
    for (const u of Yb) {
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
      const b = document.createElement("div");
      b.className = `spaghettio-landing-status ${u.status}`, b.textContent = u.status === "solved" ? "Solved" : u.status === "partial" ? "Partial" : "WIP", p.appendChild(b);
      const _ = document.createElement("div");
      _.className = "spaghettio-landing-entities", _.textContent = "\u2014", p.appendChild(_), u.status !== "wip" && p.addEventListener("click", () => {
        Jb(t, u, p, _);
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
  function Jb(n, t, e, s) {
    e.classList.contains("loading") || (e.classList.add("loading"), s.innerHTML = '<span class="spaghettio-spinner"></span>', (async () => {
      let i, r;
      try {
        const o = n.defaultMachineForItem(t.item, t.machine);
        i = await n.solve(t.item, t.rate, t.inputs, {}, o), r = await n.buildLayout(i, t.beltTier);
      } catch (o) {
        e.classList.remove("loading"), s.textContent = "error", console.error("Landing solve/layout failed:", o);
        return;
      }
      s.textContent = String(r.entities.length), e.classList.remove("loading"), Pd(i.machines.map((o) => ({
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
      }))), Zb(t, r, i).catch((o) => {
        console.error("Modal init failed:", o);
      });
    })());
  }
  async function Zb(n, t, e) {
    const s = document.createElement("div");
    s.className = "spaghettio-preview-backdrop", document.body.appendChild(s);
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
    const p = `${t.width ?? 0}\xD7${t.height ?? 0}`, f = e.machines.reduce((E, R) => E + Math.ceil(R.count), 0);
    u.innerHTML = `<span>${f} machines</span><span>${p} tiles</span>`, c.appendChild(u);
    const m = document.createElement("button");
    m.className = "spaghettio-preview-close", m.textContent = "\xD7", m.addEventListener("click", a), c.appendChild(m), l.appendChild(c);
    const g = document.createElement("div");
    g.className = "spaghettio-preview-canvas", l.appendChild(g);
    const y = document.createElement("div");
    y.className = "spaghettio-preview-badge", y.textContent = `0 / ${t.entities.length}`, g.appendChild(y), o = new Qo(), await o.init({
      resizeTo: g,
      background: 1118481,
      antialias: true
    }), g.insertBefore(o.canvas, y), o.canvas.addEventListener("contextmenu", (E) => E.preventDefault());
    const w = (t.width ?? 20) * S, x = (t.height ?? 20) * S, b = Math.max(w, x, 600) + 200, _ = new Ad({
      screenWidth: g.clientWidth,
      screenHeight: g.clientHeight,
      worldWidth: b,
      worldHeight: b,
      events: o.renderer.events
    });
    _.drag({
      mouseButtons: "left"
    }).pinch().wheel().decelerate(), o.stage.addChild(_);
    const v = new Ut();
    _.addChild(v), _.fit(true, w * 1.15, x * 1.2), _.moveCenter(w / 2, x / 2);
    const { renderLayoutAnimated: C } = await Mn(async () => {
      const { renderLayoutAnimated: E } = await import("./animated-I16Ktz9U.js");
      return {
        renderLayoutAnimated: E
      };
    }, []);
    C(t, v, y, () => {
    });
  }
  var Te = Uint8Array, ms = Uint16Array, Qb = Int32Array, uu = new Te([
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
  ]), pu = new Te([
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
  ]), t_ = new Te([
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
  ]), fu = function(n, t) {
    for (var e = new ms(31), s = 0; s < 31; ++s) e[s] = t += 1 << n[s - 1];
    for (var i = new Qb(e[30]), s = 1; s < 30; ++s) for (var r = e[s]; r < e[s + 1]; ++r) i[r] = r - e[s] << 5 | s;
    return {
      b: e,
      r: i
    };
  }, mu = fu(uu, 2), gu = mu.b, e_ = mu.r;
  gu[28] = 258, e_[258] = 28;
  var n_ = fu(pu, 0), s_ = n_.b, jo = new ms(32768);
  for (var Vt = 0; Vt < 32768; ++Vt) {
    var Cn = (Vt & 43690) >> 1 | (Vt & 21845) << 1;
    Cn = (Cn & 52428) >> 2 | (Cn & 13107) << 2, Cn = (Cn & 61680) >> 4 | (Cn & 3855) << 4, jo[Vt] = ((Cn & 65280) >> 8 | (Cn & 255) << 8) >> 1;
  }
  var Zs = (function(n, t, e) {
    for (var s = n.length, i = 0, r = new ms(t); i < s; ++i) n[i] && ++r[n[i] - 1];
    var o = new ms(t);
    for (i = 1; i < t; ++i) o[i] = o[i - 1] + r[i - 1] << 1;
    var a;
    if (e) {
      a = new ms(1 << t);
      var l = 15 - t;
      for (i = 0; i < s; ++i) if (n[i]) for (var c = i << 4 | n[i], h = t - n[i], d = o[n[i] - 1]++ << h, u = d | (1 << h) - 1; d <= u; ++d) a[jo[d] >> l] = c;
    } else for (a = new ms(s), i = 0; i < s; ++i) n[i] && (a[i] = jo[o[n[i] - 1]++] >> 15 - n[i]);
    return a;
  }), pi = new Te(288);
  for (var Vt = 0; Vt < 144; ++Vt) pi[Vt] = 8;
  for (var Vt = 144; Vt < 256; ++Vt) pi[Vt] = 9;
  for (var Vt = 256; Vt < 280; ++Vt) pi[Vt] = 7;
  for (var Vt = 280; Vt < 288; ++Vt) pi[Vt] = 8;
  var yu = new Te(32);
  for (var Vt = 0; Vt < 32; ++Vt) yu[Vt] = 5;
  var i_ = Zs(pi, 9, 1), r_ = Zs(yu, 5, 1), no = function(n) {
    for (var t = n[0], e = 1; e < n.length; ++e) n[e] > t && (t = n[e]);
    return t;
  }, We = function(n, t, e) {
    var s = t / 8 | 0;
    return (n[s] | n[s + 1] << 8) >> (t & 7) & e;
  }, so = function(n, t) {
    var e = t / 8 | 0;
    return (n[e] | n[e + 1] << 8 | n[e + 2] << 16) >> (t & 7);
  }, o_ = function(n) {
    return (n + 7) / 8 | 0;
  }, a_ = function(n, t, e) {
    return (e == null || e > n.length) && (e = n.length), new Te(n.subarray(t, e));
  }, l_ = [
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
  ], Ge = function(n, t, e) {
    var s = new Error(t || l_[n]);
    if (s.code = n, Error.captureStackTrace && Error.captureStackTrace(s, Ge), !e) throw s;
    return s;
  }, c_ = function(n, t, e, s) {
    var i = n.length, r = 0;
    if (!i || t.f && !t.l) return e || new Te(0);
    var o = !e, a = o || t.i != 2, l = t.i;
    o && (e = new Te(i * 3));
    var c = function(Y) {
      var tt = e.length;
      if (Y > tt) {
        var ot = new Te(Math.max(tt * 2, Y));
        ot.set(e), e = ot;
      }
    }, h = t.f || 0, d = t.p || 0, u = t.b || 0, p = t.l, f = t.d, m = t.m, g = t.n, y = i * 8;
    do {
      if (!p) {
        h = We(n, d, 1);
        var w = We(n, d + 1, 3);
        if (d += 3, w) if (w == 1) p = i_, f = r_, m = 9, g = 5;
        else if (w == 2) {
          var v = We(n, d, 31) + 257, C = We(n, d + 10, 15) + 4, E = v + We(n, d + 5, 31) + 1;
          d += 14;
          for (var R = new Te(E), k = new Te(19), T = 0; T < C; ++T) k[t_[T]] = We(n, d + T * 3, 7);
          d += C * 3;
          for (var O = no(k), U = (1 << O) - 1, G = Zs(k, O, 1), T = 0; T < E; ) {
            var B = G[We(n, d, U)];
            d += B & 15;
            var x = B >> 4;
            if (x < 16) R[T++] = x;
            else {
              var L = 0, D = 0;
              for (x == 16 ? (D = 3 + We(n, d, 3), d += 2, L = R[T - 1]) : x == 17 ? (D = 3 + We(n, d, 7), d += 3) : x == 18 && (D = 11 + We(n, d, 127), d += 7); D--; ) R[T++] = L;
            }
          }
          var K = R.subarray(0, v), X = R.subarray(v);
          m = no(K), g = no(X), p = Zs(K, m, 1), f = Zs(X, g, 1);
        } else Ge(1);
        else {
          var x = o_(d) + 4, b = n[x - 4] | n[x - 3] << 8, _ = x + b;
          if (_ > i) {
            l && Ge(0);
            break;
          }
          a && c(u + b), e.set(n.subarray(x, _), u), t.b = u += b, t.p = d = _ * 8, t.f = h;
          continue;
        }
        if (d > y) {
          l && Ge(0);
          break;
        }
      }
      a && c(u + 131072);
      for (var Z = (1 << m) - 1, M = (1 << g) - 1, N = d; ; N = d) {
        var L = p[so(n, d) & Z], W = L >> 4;
        if (d += L & 15, d > y) {
          l && Ge(0);
          break;
        }
        if (L || Ge(2), W < 256) e[u++] = W;
        else if (W == 256) {
          N = d, p = null;
          break;
        } else {
          var z = W - 254;
          if (W > 264) {
            var T = W - 257, q = uu[T];
            z = We(n, d, (1 << q) - 1) + gu[T], d += q;
          }
          var Q = f[so(n, d) & M], at = Q >> 4;
          Q || Ge(3), d += Q & 15;
          var X = s_[at];
          if (at > 3) {
            var q = pu[at];
            X += so(n, d) & (1 << q) - 1, d += q;
          }
          if (d > y) {
            l && Ge(0);
            break;
          }
          a && c(u + 131072);
          var st = u + z;
          if (u < X) {
            var mt = r - X, _t = Math.min(X, st);
            for (mt + u < 0 && Ge(3); u < _t; ++u) e[u] = s[mt + u];
          }
          for (; u < st; ++u) e[u] = e[u - X];
        }
      }
      t.l = p, t.p = N, t.b = u, t.f = h, p && (h = 1, t.m = m, t.d = f, t.n = g);
    } while (!h);
    return u != e.length && o ? a_(e, 0, u) : e.subarray(0, u);
  }, h_ = new Te(0), d_ = function(n) {
    (n[0] != 31 || n[1] != 139 || n[2] != 8) && Ge(6, "invalid gzip data");
    var t = n[3], e = 10;
    t & 4 && (e += (n[10] | n[11] << 8) + 2);
    for (var s = (t >> 3 & 1) + (t >> 4 & 1); s > 0; s -= !n[e++]) ;
    return e + (t & 2);
  }, u_ = function(n) {
    var t = n.length;
    return (n[t - 4] | n[t - 3] << 8 | n[t - 2] << 16 | n[t - 1] << 24) >>> 0;
  };
  function p_(n, t) {
    var e = d_(n);
    return e + 8 > n.length && Ge(6, "invalid gzip data"), c_(n.subarray(e, -8), {
      i: 2
    }, new Te(u_(n)), t);
  }
  var f_ = typeof TextDecoder < "u" && new TextDecoder(), m_ = 0;
  try {
    f_.decode(h_, {
      stream: true
    }), m_ = 1;
  } catch {
  }
  const io = "fls1";
  async function xu(n) {
    const t = typeof n == "string" ? n : new TextDecoder().decode(n);
    if (!t.startsWith(io)) throw new Error(`Not a layout snapshot: expected "${io}" prefix, got "${t.slice(0, 4)}"`);
    const e = t.slice(io.length), s = Uint8Array.from(atob(e), (o) => o.charCodeAt(0)), i = p_(s), r = new TextDecoder().decode(i);
    return JSON.parse(r);
  }
  function g_(n) {
    return new Promise((t, e) => {
      const s = new FileReader();
      s.onload = () => t(s.result), s.onerror = () => e(new Error("Failed to read file")), s.readAsText(n);
    });
  }
  function y_(n, t) {
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
          const i = await g_(s), r = await xu(i);
          t(r);
        } catch (i) {
          alert(`Failed to load snapshot: ${i}`);
        }
      }
    });
  }
  function x_(n, t, e) {
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
  function Ec(n, t, e, s, i, r, o, a) {
    const l = s - t, c = i - e, h = Math.sqrt(l * l + c * c);
    if (h === 0) return;
    const d = l / h, u = c / h;
    let p = 0;
    for (; p < h; ) {
      const f = Math.min(p + r, h);
      n.moveTo(t + d * p, e + u * p).lineTo(t + d * f, e + u * f).stroke(a), p = f + o;
    }
  }
  function b_(n, t, e, s, i) {
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
      }), Ec(m, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), m.eventMode = "static", m.on("pointerenter", () => i(`Route failed: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) [${d.spec_key}]`)), m.on("pointerleave", () => i(null)), r.addChild(m);
    }
    const l = __;
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
      }), Ec(m, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
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
  const __ = [
    5676246,
    6996096,
    13672512,
    11567312
  ], w_ = {
    Error: 16729156,
    Warning: 16755200
  }, v_ = 0.85;
  function C_(n, t, e) {
    const s = new Ut(), i = /* @__PURE__ */ new Map();
    for (const r of n) {
      if (r.x == null || r.y == null) continue;
      const o = w_[r.severity] ?? 4500223, a = new ft();
      a.rect(r.x * S, r.y * S, S, S).stroke({
        width: 2,
        color: o,
        alpha: v_
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
  function S_(n) {
    const t = Math.max(0, Math.min(1, n)), e = Math.round(64 + 96 * t);
    return {
      color: 255 << 16 | e << 8 | 32,
      alpha: 0.45 * (1 - t) + 0.12
    };
  }
  function T_(n, t, e) {
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
      const a = Ve[o.name];
      a && i.set(`${o.x},${o.y}`, a);
    }
    const r = new Ut();
    r.eventMode = "none";
    for (const [o, a] of s) {
      const [l, c] = o.split(",").map(Number), h = i.get(o), [d, u] = h ?? [
        1,
        1
      ], { color: p, alpha: f } = S_(a), m = new ft();
      m.rect(l * S, c * S, d * S, u * S).fill({
        color: p,
        alpha: f
      }), r.addChild(m);
    }
    return e.addChild(r), r;
  }
  function A_(n) {
    const t = n.point.direction;
    return t === "East" || t === "West" ? "horizontal" : "vertical";
  }
  function E_(n) {
    const t = new Set(n.map(A_));
    return t.size === 1 ? [
      ...t
    ][0] : "mixed";
  }
  function k_(n) {
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
    for (const a of e.values()) a.axis = E_([
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
  function M_(n) {
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
  function P_(n) {
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
  const I_ = 0.35;
  function R_(n, t, e, s, i) {
    const r = S * 0.45;
    n.setStrokeStyle({
      width: 3,
      color: i,
      alpha: I_
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
  function ro(n) {
    return [
      n.point.x,
      n.point.y
    ];
  }
  function L_(n, t, e, s, i, r, o = 2, a = 6, l = 4, c = 0.9) {
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
  function $_(n) {
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
  function B_(n) {
    const t = new Ut(), e = n.regions ?? [], s = [];
    if (e.length === 0) return {
      layer: t,
      items: s,
      hitTest: () => null
    };
    for (const r of e) {
      const o = k_(r), a = M_(r.kind), l = P_(o.cls), c = r.x * S, h = r.y * S, d = r.width * S, u = r.height * S;
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
      const m = r.ports ?? [], g = $_(m);
      for (const { item: y, inPort: w, outPort: x } of g) {
        const [b, _] = ro(w), [v, C] = ro(x), E = b * S + S / 2, R = _ * S + S / 2, k = v * S + S / 2, T = C * S + S / 2, O = In(y), U = new ft();
        L_(U, E, R, k, T, O), t.addChild(U);
      }
      for (const y of m) {
        const [w, x] = ro(y), b = w * S + S / 2, _ = x * S + S / 2, v = new ft(), C = y.item ? In(y.item) : 8947848;
        R_(v, b, _, y.point.direction, C), t.addChild(v);
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
  const O_ = /* @__PURE__ */ new Set([
    "JunctionGrowthStarted",
    "JunctionGrowthIteration",
    "JunctionStrategyAttempt",
    "SatInvocation",
    "JunctionSolved",
    "JunctionGrowthCapped",
    "RegionWalkerVeto"
  ]);
  function N_(n) {
    return O_.has(n.phase);
  }
  function F_(n) {
    const t = n.data;
    return typeof t.seed_x == "number" && typeof t.seed_y == "number" ? [
      t.seed_x,
      t.seed_y
    ] : [
      t.tile_x,
      t.tile_y
    ];
  }
  function W_(n, t) {
    return `${n},${t}`;
  }
  function G_(n) {
    const t = /* @__PURE__ */ new Map(), e = (a, l) => {
      const c = W_(a, l);
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
      if (!N_(a)) continue;
      const [l, c] = F_(a), h = e(l, c);
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
  function kc(n) {
    return n.iterations.length === 0 ? null : n.iterations[n.defaultIterIndex] ?? n.iterations[n.iterations.length - 1];
  }
  function z_(n) {
    return n ? `${n.entity_name}@(${n.entity_x},${n.entity_y}) ${n.direction}` : "";
  }
  function D_(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function H_(n, t, e) {
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
        item: D_(c.spec_key),
        specKey: c.spec_key,
        tiles: h
      });
    }
    return a;
  }
  const Mc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, U_ = new $e({
    fontFamily: "monospace",
    fontSize: 10,
    fill: 16777215,
    dropShadow: {
      color: 0,
      distance: 1,
      blur: 2,
      alpha: 0.8
    }
  }), j_ = new $e({
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
  function V_(n) {
    const t = new Ut(), e = [], s = [];
    for (const o of n) {
      if (o.outcome.kind !== "Solved") continue;
      const a = kc(o);
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
      const a = kc(o);
      if (!a) continue;
      const l = a.bbox;
      if (l.w <= 0 || l.h <= 0 || o.outcome.kind === "Capped" && i(o.seed.x, o.seed.y)) continue;
      const c = l.x * S, h = l.y * S, d = l.w * S, u = l.h * S, p = Mc[o.outcome.kind] ?? Mc.Open, f = new ft();
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
      const m = new De({
        text: `Junction (${o.seed.x},${o.seed.y})`,
        style: U_
      });
      m.x = c + 3, m.y = h + 2, t.addChild(m);
      const g = new De({
        text: Y_(o),
        style: j_
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
  function Y_(n) {
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
  const Pc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, X_ = 0.04, q_ = 9060416, K_ = 0.55, J_ = 4243680, Z_ = 5592405, Q_ = 0.55, tw = 16777215, Ic = 0.85, ew = 16777215, Rc = new $e({
    fontFamily: "monospace",
    fontSize: 7,
    fontWeight: "700",
    fill: tw
  }), Lc = /* @__PURE__ */ new Map();
  function nw(n) {
    let t = Lc.get(n);
    if (!t) {
      const s = `/spaghettio/icons/${n}.png`;
      t = He.load(s).catch(() => null), Lc.set(n, t);
    }
    return t;
  }
  function sw() {
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
      const { cluster: a, iter: l } = r, c = l.bbox, h = Pc[a.outcome.kind] ?? Pc.Open, d = new ft();
      d.rect(c.x * S, c.y * S, c.w * S, c.h * S).fill({
        color: h,
        alpha: X_
      }), n.addChild(d);
      const u = iw(c.x * S, c.y * S, c.w * S, c.h * S, {
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
        color: q_,
        alpha: K_
      });
      for (const [g, y] of l.forbidden) rw(p, g * S, y * S, S);
      n.addChild(p);
      const f = new ft();
      f.circle((a.seed.x + 0.5) * S, (a.seed.y + 0.5) * S, S * 0.42).stroke({
        width: 3,
        color: J_,
        alpha: 0.95
      }), n.addChild(f);
      const m = ((_a2 = l.sat) == null ? void 0 : _a2.boundaries) ?? l.boundaries;
      for (const g of m) aw(n, g, o, () => t);
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
  function iw(n, t, e, s, i) {
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
  function rw(n, t, e, s) {
    const i = s;
    n.moveTo(t, e + i).lineTo(t + i, e).stroke(), n.moveTo(t + i / 3, e + i).lineTo(t + i, e + 2 * i / 3).stroke(), n.moveTo(t, e + 2 * i / 3).lineTo(t + 2 * i / 3, e).stroke();
  }
  function ow(n, t) {
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
  function aw(n, t, e, s) {
    const i = t.x * S, r = t.y * S, o = S / 3, a = ow(t.is_input, t.direction), l = a === "top" || a === "bottom";
    let c = i, h = r, d = S, u = S;
    a === "top" ? u = o : a === "bottom" ? (h = r + S - o, u = o) : (a === "left" || (c = i + S - o), d = o);
    const p = new ft();
    p.rect(c, h, d, u).fill({
      color: Z_,
      alpha: Q_
    }), t.interior && (p.setStrokeStyle({
      width: 1,
      color: ew,
      alpha: 0.5
    }), p.rect(c, h, d, u).stroke()), n.addChild(p);
    const f = lw(t.direction), [m, g, y, w] = l ? [
      c + d / 6,
      h + u / 2,
      c + d * 5 / 6,
      h + u / 2
    ] : [
      c + d / 2,
      h + u / 6,
      c + d / 2,
      h + u * 5 / 6
    ], x = new De({
      text: f,
      style: Rc
    });
    x.anchor.set(0.5), x.x = m, x.y = g, x.alpha = Ic, n.addChild(x);
    const b = new De({
      text: f,
      style: Rc
    });
    b.anchor.set(0.5), b.x = y, b.y = w, b.alpha = Ic, n.addChild(b);
    const _ = c + d / 2, v = h + u / 2;
    nw(t.item).then((C) => {
      if (e !== s() || !C) return;
      const E = new je(C);
      E.anchor.set(0.5), E.x = _, E.y = v;
      const R = o * 0.95;
      E.width = R, E.height = R, n.addChild(E);
    });
  }
  function lw(n) {
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
  const cw = 4247776, hw = 0.18;
  function dw(n) {
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
        color: cw,
        alpha: hw
      });
    }
    return e.addChild(s), e;
  }
  function uw(n, t, e) {
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
    const b = document.createElement("div");
    b.className = "jd-titlebar";
    const _ = document.createElement("span");
    _.className = "jd-title", _.textContent = "Junction details";
    const v = document.createElement("span");
    v.className = "jd-status-pill";
    const C = document.createElement("span");
    C.className = "jd-close", C.textContent = "\xD7", C.title = "Close details (Esc)", b.append(_, v, C);
    const E = document.createElement("div");
    E.className = "jd-detail";
    const R = document.createElement("div");
    R.className = "jd-footer", R.textContent = "Esc close \xB7 \u2190/\u2192 step all \xB7 w/s iter \xB7 a/d variant \xB7 Home/End first/last", x.append(b, E, R), n.append(w, x);
    let k = null, T = 0, O = null, U = false;
    function G(Y, tt) {
      k = Y, O = tt ?? null, T = Y.defaultIterIndex, s.classList.add("jd-open"), Q(), q(), M();
    }
    function B() {
      k && (D(), k = null, O = null, s.classList.remove("jd-open"), e.onChange(null));
    }
    function L() {
      !k || U || (U = true, w.classList.add("jd-open"), x.classList.add("jd-open"), at());
    }
    function D() {
      U && (U = false, w.classList.remove("jd-open"), x.classList.remove("jd-open"));
    }
    function K() {
      U ? D() : L();
    }
    function X() {
      return k !== null;
    }
    function Z(Y) {
      if (!k) return;
      const tt = Math.max(0, Math.min(k.iterations.length - 1, Y));
      tt !== T && (T = tt, Q(), q(), M());
    }
    function M() {
      if (!k) return;
      const Y = k.iterations[T];
      if (!Y) return;
      const tt = t.toScreen(Y.bbox.x * S, Y.bbox.y * S), ot = t.toScreen((Y.bbox.x + Y.bbox.w) * S, (Y.bbox.y + Y.bbox.h) * S), dt = n.getBoundingClientRect();
      if (!(ot.x < 0 || tt.x > dt.width || ot.y < 0 || tt.y > dt.height)) return;
      const Tt = (Y.bbox.x + Y.bbox.w / 2) * S, jt = (Y.bbox.y + Y.bbox.h / 2) * S;
      t.moveCenter(Tt, jt);
    }
    function N() {
      const Y = /* @__PURE__ */ new Map(), tt = k;
      if (!tt) return Y;
      for (let ot = 0; ot < tt.iterations.length; ot++) {
        const dt = tt.iterations[ot], bt = Y.get(dt.iter) ?? [];
        bt.push(ot), Y.set(dt.iter, bt);
      }
      return Y;
    }
    function W(Y) {
      if (!k) return;
      const tt = N(), ot = Array.from(tt.keys()).sort((J, it) => J - it), dt = k.iterations[T].iter, bt = ot.indexOf(dt), Tt = ot[Math.max(0, Math.min(ot.length - 1, bt + Y))], jt = tt.get(Tt) ?? [], H = jt.find((J) => k.iterations[J].variant === "");
      Z(H ?? jt[0] ?? T);
    }
    function z(Y) {
      if (!k) return;
      const tt = N(), ot = k.iterations[T].iter, dt = tt.get(ot) ?? [];
      if (dt.length <= 1) return;
      const Tt = (dt.indexOf(T) + Y + dt.length) % dt.length;
      Z(dt[Tt]);
    }
    function q() {
      if (!k) return;
      const Y = k.iterations[T];
      if (!Y) return;
      const tt = n.getBoundingClientRect(), ot = s.offsetWidth || 200, dt = s.offsetHeight || 70, bt = (Y.bbox.x + Y.bbox.w) * S, Tt = (Y.bbox.y + Y.bbox.h) * S, jt = Y.bbox.y * S, H = t.toScreen(bt, Tt), J = t.toScreen(bt, jt);
      let it = H.x - ot, pt = H.y;
      pt + dt > tt.height - 4 && (pt = J.y - dt), it = Math.max(4, Math.min(it, tt.width - ot - 4)), pt = Math.max(4, Math.min(pt, tt.height - dt - 4)), s.style.left = `${it}px`, s.style.top = `${pt}px`;
    }
    t.on("moved", q), t.on("zoomed", q), window.addEventListener("resize", q);
    function Q() {
      if (!k) return;
      const Y = k, tt = Y.iterations[T];
      r.textContent = `Junction (${Y.seed.x},${Y.seed.y})`, o.className = `jd-status-pill jd-${Y.outcome.kind.toLowerCase()}`, o.textContent = $c(Y);
      const ot = Y.iterations.length, dt = tt && tt.variant ? ` \xB7 ${tt.variant}` : "";
      f.textContent = `iter ${tt ? tt.iter : "-"}${dt} \xB7 ${T + 1}/${ot}`, p.disabled = T <= 0, m.disabled = T >= ot - 1, g.disabled = T === Y.defaultIterIndex, y.innerHTML = "";
      for (const bt of fw(Y, tt)) {
        const Tt = document.createElement("div");
        Tt.className = `jd-inline-summary-row jd-inline-summary-row--${bt.tone}`, Tt.textContent = bt.text, y.appendChild(Tt);
      }
      U && at(), tt && e.onChange({
        cluster: Y,
        iter: tt,
        trace: O
      });
    }
    function at() {
      if (!k) return;
      const Y = k, tt = Y.iterations[T];
      v.className = `jd-status-pill jd-${Y.outcome.kind.toLowerCase()}`, v.textContent = $c(Y), _.textContent = `Junction (${Y.seed.x},${Y.seed.y})`, gw(E, Y, tt);
    }
    d.addEventListener("click", B), a.addEventListener("click", K), l.addEventListener("click", _t), c.addEventListener("click", () => {
      if (!k) return;
      const Y = Vo(k, k.iterations[T], O);
      Sw(c, Y);
    }), h.addEventListener("click", () => {
      if (!k || !e.onEditRequested) return;
      const Y = k.iterations[T];
      Y && e.onEditRequested({
        cluster: k,
        iter: Y,
        trace: O
      });
    }), C.addEventListener("click", D), w.addEventListener("click", D);
    function st() {
      if (!k) return null;
      const Y = k.iterations[T];
      return Y ? {
        cluster: k,
        iter: Y,
        trace: O
      } : null;
    }
    function mt(Y) {
      s.classList.toggle("jd-edit-mode", Y), c.disabled = Y, l.disabled = Y, h.disabled = Y;
    }
    function _t() {
      var _a2;
      if (!k) return;
      const Y = pw(k, T), tt = JSON.stringify(Y, (dt, bt) => typeof bt == "bigint" ? String(bt) : bt, 2), ot = (dt) => {
        const bt = l.textContent;
        l.textContent = dt ? "\u2713" : "!", l.classList.add("jd-inline-btn--flash"), window.setTimeout(() => {
          l.textContent = bt, l.classList.remove("jd-inline-btn--flash");
        }, 900);
      };
      if ((_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText) navigator.clipboard.writeText(tt).then(() => ot(true), () => ot(false));
      else {
        const dt = document.createElement("textarea");
        dt.value = tt, dt.style.position = "fixed", dt.style.opacity = "0", document.body.appendChild(dt), dt.select();
        try {
          document.execCommand("copy"), ot(true);
        } catch {
          ot(false);
        }
        document.body.removeChild(dt);
      }
    }
    return p.addEventListener("click", () => Z(T - 1)), m.addEventListener("click", () => Z(T + 1)), g.addEventListener("click", () => {
      k && Z(k.defaultIterIndex);
    }), document.addEventListener("keydown", (Y) => {
      var _a2, _b2;
      if (!X()) return;
      const tt = (_b2 = (_a2 = Y.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (tt === "INPUT" || tt === "TEXTAREA" || tt === "SELECT") return;
      const ot = Y.key, dt = () => {
        Y.stopImmediatePropagation(), Y.preventDefault();
      };
      ot === "Escape" ? (U ? D() : B(), dt()) : ot === "ArrowLeft" ? (Z(T - 1), dt()) : ot === "ArrowRight" ? (Z(T + 1), dt()) : ot === "Home" ? (Z(0), dt()) : ot === "End" && k ? (Z(k.iterations.length - 1), dt()) : ot === "w" || ot === "W" ? (W(-1), dt()) : ot === "s" || ot === "S" ? (W(1), dt()) : ot === "a" || ot === "A" ? (z(-1), dt()) : ot === "d" || ot === "D" ? (z(1), dt()) : (ot === "i" || ot === "I") && (K(), dt());
    }, {
      capture: true
    }), {
      open: G,
      close: B,
      isOpen: X,
      inlineEl: s,
      getSelection: st,
      setEditMode: mt
    };
  }
  function pw(n, t) {
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
  function $c(n) {
    switch (n.outcome.kind) {
      case "Solved":
        return `Solved \xB7 ${n.outcome.regionTiles}t`;
      case "Capped":
        return `Capped \xB7 ${n.outcome.iters} iter`;
      case "Open":
        return "Open";
    }
  }
  function fw(n, t) {
    if (t) {
      if (t.veto) return [
        {
          text: `veto \xB7 ${Ni(t.veto.broken_segment, 22)} @ (${t.veto.break_tile_x},${t.veto.break_tile_y})`,
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
        const i = s.detail ? ` \xB7 ${Ni(s.detail, 28)}` : "";
        return [
          {
            text: `${s.strategy} \u2192 ${Ni(s.outcome, 12)}${i}`,
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
            text: `cap: ${Ni(n.outcome.reason, 32)}`,
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
  function Ni(n, t) {
    return n.length <= t ? n : `${n.slice(0, t - 1)}\u2026`;
  }
  function bu(n) {
    var _a2;
    return ((_a2 = n.sat) == null ? void 0 : _a2.boundaries) ?? n.boundaries;
  }
  function mw(n) {
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
  function gw(n, t, e) {
    n.innerHTML = "", n.appendChild(yw(t)), n.appendChild(xw(t, e)), e && (n.appendChild(bw(e)), n.appendChild(_w(e)), n.appendChild(ww(e)), e.veto && n.appendChild(vw(e))), t.nearbyStamped.length > 0 && n.appendChild(Cw(t));
  }
  function es(n, t = true) {
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
  function yw(n) {
    const { details: t, bodyEl: e } = es("Summary"), s = document.createElement("div");
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
  function xw(n, t) {
    const { details: e, bodyEl: s } = es("Participating specs");
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
  function bw(n) {
    const t = bu(n), e = n.sat ? " (as fed to SAT)" : " (spec perimeter)", { details: s, bodyEl: i } = es(`Boundaries${e}`);
    if (t.length === 0) {
      const r = document.createElement("div");
      return r.className = "jd-row jd-row--dim", r.textContent = "(none)", i.appendChild(r), s;
    }
    for (const r of t) {
      const o = document.createElement("div");
      o.className = "jd-row";
      const a = r.is_input ? "IN " : "OUT", l = r.interior ? " (interior)" : "", c = r.external_feeder ? ` \u2190 ${z_(r.external_feeder)}` : "";
      o.style.color = r.is_input ? "#9f9" : "#f99";
      const h = r.spec_key ? ` \xB7 ${r.spec_key}` : "";
      o.textContent = `${a} (${r.x},${r.y}) ${mw(r.direction)} ${r.direction} \xB7 ${r.item}${l}${h}${c}`, i.appendChild(o);
    }
    return s;
  }
  function _w(n) {
    const { details: t, bodyEl: e } = es("Strategy attempts");
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
  function ww(n) {
    const { details: t, bodyEl: e } = es("SAT", !!n.sat);
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
  function vw(n) {
    const { details: t, bodyEl: e } = es("Walker veto");
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
  function Cw(n) {
    const { details: t, bodyEl: e } = es("Nearby stamped", false);
    for (const s of n.nearbyStamped) {
      const i = document.createElement("div");
      i.className = "jd-row";
      const r = s.carries ? ` carries=${s.carries}` : "", o = s.segment_id ? ` \xB7 seg=${s.segment_id}` : "";
      i.textContent = `(${s.x},${s.y}) ${s.name} ${s.direction}${r}${o}${s.feeds_seed_area ? "  \u26A0 feeds seed" : ""}`, e.appendChild(i);
    }
    return t;
  }
  const _u = {
    "transport-belt": 4,
    "fast-transport-belt": 6,
    "express-transport-belt": 8
  };
  function Vo(n, t, e, s) {
    var _a2, _b2;
    const i = n.seed, r = (t == null ? void 0 : t.bbox) ?? {
      x: i.x,
      y: i.y,
      w: 1,
      h: 1
    }, o = (t == null ? void 0 : t.iter) ?? 0, a = ((_a2 = t == null ? void 0 : t.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", l = ((_b2 = t == null ? void 0 : t.sat) == null ? void 0 : _b2.max_reach) ?? _u[a] ?? 4, c = bu(t ?? {
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
    })), h = t && e ? H_(e, r, 2).map((p) => ({
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
  function Sw(n, t) {
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
  function Tw(n) {
    return n.x === void 0 || n.y === void 0 || n.direction === void 0 ? null : n;
  }
  const Fi = {
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
  }, Bc = {
    North: "East",
    East: "South",
    South: "West",
    West: "North"
  }, Wi = {
    "transport-belt": "transport-belt",
    "fast-transport-belt": "fast-transport-belt",
    "express-transport-belt": "express-transport-belt"
  }, Oc = {
    "transport-belt": "underground-belt",
    "fast-transport-belt": "fast-underground-belt",
    "express-transport-belt": "express-underground-belt"
  };
  function Aw(n) {
    const { viewport: t, canvas: e, engine: s, jd: i, satZoneOverlayLayer: r } = n;
    let o = null, a = null, l = null, c = null, h = null, d = null, u = false, p = null, f = [], m = [], g = [], y = [], w = "belt", x = "East", b = 0, _ = null, v = "idle", C = 0, E = null, R = null;
    function k(A, I) {
      return `${A},${I}`;
    }
    function T(A, I) {
      if (!p) return false;
      const j = p.bbox;
      return A >= j.x && A < j.x + j.w && I >= j.y && I < j.y + j.h;
    }
    function O(A, I) {
      return f.find((j) => j.x === A && j.y === I);
    }
    function U() {
      if (!p || p.items.length === 0) return null;
      const A = Math.max(0, Math.min(b, p.items.length - 1));
      return p.items[A] ?? null;
    }
    function G(A, I, j) {
      if (!p) return null;
      const [rt, P] = Fi[j], $ = A - rt, et = I - P, V = O($, et);
      if (V && V.direction === j && (Wi[V.name] === V.name || V.io_type === "output")) return V.carries ?? null;
      const lt = p.boundaries.find((wt) => wt.x === A && wt.y === I && wt.isInput && wt.dir === j);
      return lt ? lt.item : null;
    }
    function B(A, I) {
      return G(A.x, A.y, I) ?? U();
    }
    function L(A, I) {
      if (!p) return null;
      const [j, rt] = Fi[I];
      for (let P = 1; P <= p.maxReach + 1; P++) {
        const $ = A.x - j * P, et = A.y - rt * P, V = O($, et);
        if (V && V.io_type === "input" && V.direction === I) return V.carries ?? null;
      }
      return null;
    }
    function D() {
      m.push(f.map((A) => ({
        ...A
      }))), m.length > 64 && m.shift(), g.length = 0;
    }
    function K(A, I) {
      D(), f = A, Z(), at();
    }
    function X() {
      if (!p) return [];
      const A = Wi[p.beltTier] ?? "transport-belt", I = [];
      for (const j of p.boundaries) {
        if (!j.isInput) continue;
        const [rt, P] = Fi[j.dir];
        I.push({
          name: A,
          x: j.x - rt,
          y: j.y - P,
          direction: j.dir,
          carries: j.item
        });
      }
      return I;
    }
    function Z() {
      const A = X();
      o && Js({
        entities: f
      }, o, void 0, void 0, void 0, A), a && Js({
        entities: y
      }, a, void 0, void 0, void 0, A), M();
    }
    function M() {
      if (c) {
        c.removeChildren();
        for (const A of f) {
          const I = A.carries;
          if (!I) continue;
          const j = `/spaghettio/icons/${I}.png`, rt = He.get(j);
          if (!rt) continue;
          const P = new je(rt), $ = S * 0.55;
          P.width = $, P.height = $, P.x = A.x * S + (S - $) / 2, P.y = A.y * S + (S - $) / 2, P.alpha = 0.85, c.addChild(P);
        }
      }
    }
    function N(A, I) {
      l && (Js({
        entities: A
      }, l, void 0, void 0, void 0, X()), l.alpha = I ? 0.5 : 0.45, l.tint = I ? 16733525 : 16777215);
    }
    function W() {
      l && (l.removeChildren(), l.tint = 16777215);
    }
    function z(A, I = "") {
      if (v = A, !d) return;
      d.classList.remove("ok", "solving", "invalid", "idle"), d.classList.add(A === "valid" ? "ok" : A);
      const j = A === "valid" ? "\u25CF" : A === "solving" ? "\u25D4" : A === "invalid" ? "\u25CF" : "\u25CB";
      d.textContent = j;
      let rt = "";
      A === "valid" ? rt = R !== null ? `valid \xB7 cost ${R} / yours ${q(f)}` : "valid" : A === "solving" ? rt = "solving\u2026" : A === "invalid" ? rt = "invalid" : rt = "no edits yet", d.title = I ? `${rt}
${I}` : rt, re();
    }
    function q(A) {
      let I = 0;
      for (const j of A) Wi[j.name] === j.name ? I += 1 : Oc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] === j.name && (I += 5);
      return I;
    }
    function Q() {
      if (!p) return "no zone";
      const A = /* @__PURE__ */ new Set();
      for (const I of f) {
        const j = k(I.x, I.y);
        if (A.has(j)) return `duplicate entity at (${I.x},${I.y})`;
        if (A.add(j), !T(I.x, I.y)) return `entity at (${I.x},${I.y}) outside bbox`;
        if (p.forbidden.has(j)) return `entity at (${I.x},${I.y}) on forbidden tile`;
      }
      for (const I of f) {
        if (I.io_type !== "input") continue;
        const [j, rt] = Fi[I.direction];
        let P = false;
        for (let $ = 1; $ <= p.maxReach + 1; $++) {
          const et = I.x + j * $, V = I.y + rt * $, lt = O(et, V);
          if (lt) {
            if (lt.io_type === "output" && lt.direction === I.direction && lt.carries === I.carries) {
              P = true;
              break;
            }
            if (lt.io_type === "input" && lt.carries === I.carries) return `UG-in at (${I.x},${I.y}) blocked by another UG-in at (${et},${V})`;
          }
        }
        if (!P) return `UG-in at (${I.x},${I.y}) has no matching UG-out within reach ${p.maxReach}`;
      }
      return null;
    }
    function at() {
      if (!u || !p) return;
      const A = Q();
      if (A) {
        y = [], R = null, Z(), z("invalid", A);
        return;
      }
      z("solving"), E !== null && window.clearTimeout(E);
      const I = ++C;
      E = window.setTimeout(() => {
        st(I);
      }, 300);
    }
    async function st(A) {
      if (p) try {
        const I = await s.solveFixture(p.fixtureJson, f);
        if (A !== C || !u) return;
        if (!I) {
          y = [], R = null, Z(), z("invalid", "SAT cannot complete this layout");
          return;
        }
        const j = new Set(f.map((P) => k(P.x, P.y))), rt = [];
        for (const P of I.entities) {
          const $ = Tw(P);
          $ && !j.has(k($.x, $.y)) && rt.push($);
        }
        y = rt, R = I.cost, Z(), z("valid");
      } catch (I) {
        if (A !== C) return;
        y = [], Z(), z("invalid", `solver error: ${I instanceof Error ? I.message : String(I)}`);
      }
    }
    function mt(A) {
      const I = e.getBoundingClientRect(), j = t.toWorld(A.clientX - I.left, A.clientY - I.top);
      return {
        x: Math.floor(j.x / S),
        y: Math.floor(j.y / S)
      };
    }
    function _t(A, I, j) {
      const rt = [], P = I.x === A.x ? 0 : I.x > A.x ? 1 : -1, $ = I.y === A.y ? 0 : I.y > A.y ? 1 : -1;
      if (j) {
        for (let V = A.y; V !== I.y + $ && $ !== 0; V += $) rt.push({
          x: A.x,
          y: V
        });
        $ === 0 && rt.push({
          x: A.x,
          y: A.y
        });
        for (let V = A.x + P; P !== 0 && V !== I.x + P; V += P) rt.push({
          x: V,
          y: I.y
        });
      } else {
        for (let V = A.x; V !== I.x + P && P !== 0; V += P) rt.push({
          x: V,
          y: A.y
        });
        P === 0 && rt.push({
          x: A.x,
          y: A.y
        });
        for (let V = A.y + $; $ !== 0 && V !== I.y + $; V += $) rt.push({
          x: I.x,
          y: V
        });
      }
      const et = [];
      for (const V of rt) {
        const lt = et[et.length - 1];
        (!lt || lt.x !== V.x || lt.y !== V.y) && et.push(V);
      }
      return et;
    }
    function Y(A, I) {
      return A.x === I.x && A.y === I.y - 1 ? "South" : A.x === I.x && A.y === I.y + 1 ? "North" : A.y === I.y && A.x === I.x - 1 ? "East" : A.y === I.y && A.x === I.x + 1 ? "West" : null;
    }
    function tt(A) {
      if (!p || A.length === 0) return null;
      for (const lt of A) if (!T(lt.x, lt.y)) return null;
      const I = (lt) => !p.forbidden.has(k(lt.x, lt.y)), j = A[0], rt = A[A.length - 1];
      if (!j || !rt || !I(j) || !I(rt)) return null;
      const P = [], $ = A.length > 1 ? Y(A[0], A[1]) ?? x : x, et = B(A[0], $);
      let V = 0;
      for (; V < A.length; ) {
        const lt = A[V], wt = A[V + 1] ?? null, At = wt ? Y(lt, wt) : V > 0 ? Y(A[V - 1], lt) : x;
        if (!At) return null;
        let Bt = V + 1;
        for (; Bt < A.length && I(A[Bt]); ) Bt++;
        if (Bt === A.length) {
          P.push(ot(lt, At, et)), V++;
          continue;
        }
        let qt = Bt;
        for (; qt < A.length && !I(A[qt]); ) qt++;
        if (qt === A.length) return null;
        const Ye = A[Bt - 1], Oe = A[qt];
        if (Math.abs(Oe.x - Ye.x) + Math.abs(Oe.y - Ye.y) > p.maxReach + 1) return null;
        for (let Xe = V; Xe < Bt - 1; Xe++) {
          const F = A[Xe], nt = Y(F, A[Xe + 1]);
          if (!nt) return null;
          P.push(ot(F, nt, et));
        }
        const an = Y(Ye, Oe);
        if (!an) return null;
        P.push(dt(Ye, an, "input", et)), P.push(dt(Oe, an, "output", et)), V = qt + 1;
      }
      return P;
    }
    function ot(A, I, j) {
      return {
        name: Wi[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "transport-belt",
        x: A.x,
        y: A.y,
        direction: I,
        carries: j ?? void 0
      };
    }
    function dt(A, I, j, rt) {
      return {
        name: Oc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "underground-belt",
        x: A.x,
        y: A.y,
        direction: I,
        io_type: j,
        carries: rt ?? void 0
      };
    }
    function bt(A) {
      const I = new Set(A.map((rt) => k(rt.x, rt.y))), j = f.filter((rt) => !I.has(k(rt.x, rt.y))).concat(A);
      K(j);
    }
    function Tt(A) {
      if (!p || !T(A.x, A.y) || !f.some((j) => j.x === A.x && j.y === A.y)) return;
      const I = f.filter((j) => !(j.x === A.x && j.y === A.y));
      K(I);
    }
    function jt(A) {
      if (!p) return;
      const I = p.boundaries.find((rt) => rt.x === A.x && rt.y === A.y && rt.isInput);
      if (!I) return;
      const j = p.items.indexOf(I.item);
      j >= 0 && j !== b && (b = j, Xt());
    }
    function H(A) {
      if (!u || !p || A.button !== 0) return;
      const I = mt(A);
      if (!I || !T(I.x, I.y)) return;
      if (jt(I), w === "erase") {
        Tt(I), A.stopPropagation(), A.preventDefault();
        return;
      }
      if (w === "ug-in" || w === "ug-out") {
        const P = w === "ug-in" ? G(I.x, I.y, x) ?? U() : L(I, x) ?? G(I.x, I.y, x) ?? U(), $ = w === "ug-in" ? dt(I, x, "input", P) : dt(I, x, "output", P), et = f.filter((V) => !(V.x === I.x && V.y === I.y)).concat($);
        K(et), A.stopPropagation(), A.preventDefault();
        return;
      }
      _ = {
        startX: I.x,
        startY: I.y,
        bendVerticalFirst: false
      };
      const j = _t({
        x: I.x,
        y: I.y
      }, I, false), rt = tt(j);
      N(rt ?? [], rt === null), A.stopPropagation(), A.preventDefault();
    }
    function J(A) {
      if (!u || !_ || !p) return;
      const I = mt(A);
      if (!I) return;
      const j = _t({
        x: _.startX,
        y: _.startY
      }, I, _.bendVerticalFirst), rt = tt(j);
      N(rt ?? [], rt === null);
    }
    function it(A) {
      if (!u || !_ || !p) {
        _ = null, W();
        return;
      }
      const I = mt(A);
      if (!I) {
        _ = null, W();
        return;
      }
      const j = _t({
        x: _.startX,
        y: _.startY
      }, I, _.bendVerticalFirst), rt = tt(j);
      if (_ = null, W(), !rt) {
        z("invalid", "drag rejected: out of bounds, on obstacle, or UG too long");
        return;
      }
      bt(rt), A.stopPropagation(), A.preventDefault();
    }
    function pt(A) {
      var _a2, _b2;
      if (!u) return;
      const I = (_b2 = (_a2 = A.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (I === "INPUT" || I === "TEXTAREA" || I === "SELECT") return;
      const j = () => {
        A.stopImmediatePropagation(), A.preventDefault();
      };
      if (A.key === "Escape") {
        Wt(), j();
        return;
      }
      if (A.key === "1") {
        Lt("belt"), j();
        return;
      }
      if (A.key === "2") {
        Lt("ug-in"), j();
        return;
      }
      if (A.key === "3") {
        Lt("ug-out"), j();
        return;
      }
      if (A.key === "0") {
        Lt("erase"), j();
        return;
      }
      if (A.key === "r" || A.key === "R") {
        _ ? _.bendVerticalFirst = !_.bendVerticalFirst : (x = Bc[x], Xt()), j();
        return;
      }
      if (A.key === "[" && p) {
        b = (b - 1 + p.items.length) % p.items.length, Xt(), j();
        return;
      }
      if (A.key === "]" && p) {
        b = (b + 1) % p.items.length, Xt(), j();
        return;
      }
      if ((A.key === "Enter" || A.key === "a" || A.key === "A") && v === "valid" && y.length > 0) {
        Zt(), j();
        return;
      }
      if ((A.ctrlKey || A.metaKey) && (A.key === "z" || A.key === "Z")) {
        A.shiftKey ? Dt() : Ot(), j();
        return;
      }
    }
    function Lt(A) {
      w = A, Xt();
    }
    function Ot() {
      m.length !== 0 && (g.push(f), f = m.pop(), Z(), at());
    }
    function Dt() {
      g.length !== 0 && (m.push(f), f = g.pop(), Z(), at());
    }
    function Zt() {
      if (y.length === 0) return;
      const A = f.concat(y.map((I) => ({
        ...I
      })));
      y = [], K(A);
    }
    function Xt() {
      if (!h) return;
      h.innerHTML = "";
      const A = [
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
      for (const [lt, wt, At] of A) {
        const Bt = document.createElement("button");
        Bt.className = "se-tool" + (w === lt ? " se-tool-active" : ""), Bt.textContent = wt, Bt.title = At, Bt.addEventListener("click", () => Lt(lt)), h.appendChild(Bt);
      }
      const I = document.createElement("button");
      I.className = "se-dir";
      const j = {
        North: "\u2191",
        East: "\u2192",
        South: "\u2193",
        West: "\u2190"
      };
      if (I.textContent = j[x], I.title = "Brush direction (R rotates)", I.addEventListener("click", () => {
        x = Bc[x], Xt();
      }), h.appendChild(I), p && p.items.length > 1) {
        const lt = document.createElement("select");
        lt.className = "se-item";
        for (const [wt, At] of p.items.entries()) {
          const Bt = document.createElement("option");
          Bt.value = String(wt), Bt.textContent = At, wt === b && (Bt.selected = true), lt.appendChild(Bt);
        }
        lt.addEventListener("change", () => {
          b = Number(lt.value) | 0;
        }), h.appendChild(lt);
      } else if (p && p.items.length === 1) {
        const lt = document.createElement("span");
        lt.className = "se-item-label", lt.textContent = p.items[0], h.appendChild(lt);
      }
      const rt = document.createElement("span");
      rt.style.flex = "1", h.appendChild(rt);
      const P = document.createElement("button");
      P.className = "se-accept", P.textContent = "Accept", P.title = "Promote ghost into painted layer (Enter)", P.addEventListener("click", Zt), P.disabled = !(v === "valid" && y.length > 0), h.appendChild(P);
      const $ = document.createElement("button");
      $.className = "se-revert", $.textContent = "Revert", $.title = "Discard all painted edits", $.addEventListener("click", () => {
        K([]);
      }), h.appendChild($);
      const et = document.createElement("button");
      et.className = "se-export", et.textContent = "Export", et.title = "Save fixture JSON (clipboard + download)", et.addEventListener("click", kt), et.disabled = v !== "valid", h.appendChild(et);
      const V = document.createElement("button");
      V.className = "se-done", V.textContent = "Done", V.title = "Exit edit mode (Esc)", V.addEventListener("click", Wt), h.appendChild(V);
    }
    function re() {
      if (!h) return;
      const A = h.querySelector(".se-accept");
      A && (A.disabled = !(v === "valid" && y.length > 0));
      const I = h.querySelector(".se-export");
      I && (I.disabled = v !== "valid");
    }
    function kt() {
      var _a2;
      if (!p || v !== "valid") return;
      const A = R ?? q(f), I = Vo(p.selection.cluster, p.selection.iter, p.selection.trace, {
        maxCost: A,
        paintedEntities: f
      }), j = (p.selection.cluster.seed ? `fixture_${p.selection.cluster.seed.x}_${p.selection.cluster.seed.y}_painted` : "fixture_painted") + ".json";
      (_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText(I).catch(() => {
      });
      const rt = new Blob([
        I
      ], {
        type: "application/json"
      }), P = URL.createObjectURL(rt), $ = document.createElement("a");
      $.href = P, $.download = j, document.body.appendChild($), $.click(), document.body.removeChild($), URL.revokeObjectURL(P);
    }
    function Mt(A) {
      var _a2, _b2, _c2, _d2;
      u && Wt();
      const I = A.iter, j = ((_a2 = I.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", rt = ((_b2 = I.sat) == null ? void 0 : _b2.max_reach) ?? _u[j] ?? 4, P = ((_c2 = I.sat) == null ? void 0 : _c2.boundaries) ?? I.boundaries, $ = Array.from(new Set(P.map((V) => V.item))), et = P.map((V) => ({
        x: V.x,
        y: V.y,
        item: V.item,
        isInput: V.is_input,
        dir: V.direction
      }));
      p = {
        bbox: {
          x: I.bbox.x,
          y: I.bbox.y,
          w: I.bbox.w,
          h: I.bbox.h
        },
        forbidden: new Set((I.forbidden ?? []).map((V) => `${V[0]},${V[1]}`)),
        beltTier: j,
        maxReach: rt,
        items: $,
        boundaries: et,
        fixtureJson: Vo(A.cluster, A.iter, A.trace),
        selection: A
      }, f = [], m = [], g = [], y = [], w = "belt", x = "East", b = 0, _ = null, R = null, o = new Ut(), a = new Ut(), a.alpha = 0.55, l = new Ut(), c = new Ut(), t.addChild(o), t.addChild(a), t.addChild(c), t.addChild(l), t.setChildIndex(r, t.children.length - 1), h = document.createElement("div"), h.className = "se-toolbar", i.inlineEl.appendChild(h), d = document.createElement("span"), d.className = "se-status", (_d2 = i.inlineEl.querySelector(".jd-inline-head")) == null ? void 0 : _d2.appendChild(d), i.setEditMode(true), t.plugins.pause("drag"), e.addEventListener("pointerdown", H, {
        capture: true
      }), e.addEventListener("pointerup", it, {
        capture: true
      }), e.addEventListener("pointermove", J, {
        capture: true
      }), document.addEventListener("keydown", pt, {
        capture: true
      }), u = true, Xt(), z("idle");
    }
    function Wt() {
      u && (u = false, E !== null && (window.clearTimeout(E), E = null), C++, o && (t.removeChild(o), o.destroy({
        children: true
      }), o = null), a && (t.removeChild(a), a.destroy({
        children: true
      }), a = null), l && (t.removeChild(l), l.destroy({
        children: true
      }), l = null), c && (t.removeChild(c), c.destroy({
        children: true
      }), c = null), h && (h.remove(), h = null), d && (d.remove(), d = null), e.removeEventListener("pointerdown", H, {
        capture: true
      }), e.removeEventListener("pointerup", it, {
        capture: true
      }), e.removeEventListener("pointermove", J, {
        capture: true
      }), document.removeEventListener("keydown", pt, {
        capture: true
      }), t.plugins.resume("drag"), i.setEditMode(false), p = null, f = [], m = [], g = [], y = []);
    }
    function de() {
      return u;
    }
    return {
      enter: Mt,
      exit: Wt,
      isActive: de
    };
  }
  let bs = {
    master: false,
    stepThrough: true,
    satZones: false,
    soloRegions: false,
    ghostTiles: false,
    itemColors: true,
    traceOverlay: false,
    heatmap: false
  };
  const Yi = [];
  function Ew() {
    const n = new URLSearchParams(window.location.search).get("debug") === "1", t = localStorage.getItem("fk-debug") === "1", e = localStorage.getItem("fk-sat-zones") === "1", s = localStorage.getItem("fk-ghost-tiles") === "1", i = localStorage.getItem("fk-item-colors"), r = localStorage.getItem("fk-trace-overlay") === "1", o = localStorage.getItem("fk-heatmap") === "1";
    bs = {
      ...bs,
      master: n || t,
      satZones: e,
      ghostTiles: s,
      itemColors: i === null ? true : i === "1",
      traceOverlay: r,
      heatmap: o
    };
  }
  function wu() {
    return bs;
  }
  function Wn(n) {
    bs = {
      ...bs,
      ...n
    }, "master" in n && localStorage.setItem("fk-debug", n.master ? "1" : "0"), "satZones" in n && localStorage.setItem("fk-sat-zones", n.satZones ? "1" : "0"), "ghostTiles" in n && localStorage.setItem("fk-ghost-tiles", n.ghostTiles ? "1" : "0"), "itemColors" in n && localStorage.setItem("fk-item-colors", n.itemColors ? "1" : "0"), "traceOverlay" in n && localStorage.setItem("fk-trace-overlay", n.traceOverlay ? "1" : "0"), "heatmap" in n && localStorage.setItem("fk-heatmap", n.heatmap ? "1" : "0");
    for (const t of Yi) t(bs);
  }
  function kw(n) {
    return Yi.push(n), () => {
      const t = Yi.indexOf(n);
      t >= 0 && Yi.splice(t, 1);
    };
  }
  function Gn(n, t, e = false) {
    const s = document.createElement("input");
    s.type = "checkbox", s.checked = e;
    const i = document.createElement("div");
    i.className = "overlay-toggle";
    const r = document.createElement("label");
    return r.appendChild(s), r.appendChild(document.createTextNode(t)), i.appendChild(r), n.appendChild(i), s;
  }
  function Mw(n) {
    n.style.position = "relative";
    const t = document.createElement("div");
    t.className = "overlay-panel";
    const e = wu(), s = Gn(t, "Debug", e.master), i = Gn(t, "Item colours", e.itemColors), r = Gn(t, "Starvation heatmap", e.heatmap), o = document.createElement("div");
    o.className = "overlay-sub-panel", o.style.display = e.master ? "flex" : "none";
    const a = Gn(o, "SAT Zones", e.satZones), l = Gn(o, "Ghost tiles", e.ghostTiles), c = Gn(o, "Trace overlay", e.traceOverlay), h = Gn(o, "Solo regions", e.soloRegions);
    return t.appendChild(o), n.appendChild(t), s.addEventListener("change", () => {
      o.style.display = s.checked ? "flex" : "none", Wn({
        master: s.checked
      });
    }), a.addEventListener("change", () => {
      Wn({
        satZones: a.checked
      });
    }), l.addEventListener("change", () => {
      Wn({
        ghostTiles: l.checked
      });
    }), c.addEventListener("change", () => {
      Wn({
        traceOverlay: c.checked
      });
    }), i.addEventListener("change", () => {
      Wn({
        itemColors: i.checked
      });
    }), r.addEventListener("change", () => {
      Wn({
        heatmap: r.checked
      });
    }), {
      setDebugEnabled(d) {
        s.checked = d, o.style.display = d ? "flex" : "none", Wn({
          master: d
        });
      },
      debugCb: s,
      colorCb: i,
      heatmapCb: r,
      regionsCb: a,
      soloRegionsCb: h,
      ghostTilesCb: l,
      traceOverlayCb: c
    };
  }
  function Pw(n) {
    const t = document.createElement("div");
    t.className = "retry-panel", n.appendChild(t);
    let e = null;
    function s(r) {
      if (!(r == null ? void 0 : r.trace)) return null;
      for (const o of r.trace) if (o.phase === "LayoutRetried") return o.data;
      return null;
    }
    function i() {
      const r = wu().master, o = s(e);
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
    return kw(() => i()), {
      update(r) {
        e = r, i();
      }
    };
  }
  const Iw = {
    North: "\u2191",
    East: "\u2192",
    South: "\u2193",
    West: "\u2190"
  }, Nc = {
    N: "\u2191",
    E: "\u2192",
    S: "\u2193",
    W: "\u2190"
  };
  function cn(n = 16) {
    const t = document.createElement("img");
    return t.width = n, t.height = n, t.style.cssText = "vertical-align:middle;margin-right:3px;image-rendering:pixelated", t.addEventListener("error", () => {
      t.style.display = "none";
    }), t;
  }
  function cs(n, t) {
    n.style.display = "", n.src = `/spaghettio/icons/${t}.png`;
  }
  function oo(n, t) {
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
  function Rw(n) {
    const t = document.createElement("div");
    t.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:240px;line-height:1.5", document.body.appendChild(t);
    const e = document.createElement("div");
    t.appendChild(e);
    const s = document.createElement("span");
    s.style.color = "#888", s.style.display = "none", t.appendChild(s);
    const i = document.createElement("div");
    t.appendChild(i);
    const r = document.createElement("div"), o = cn(16), a = document.createElement("b");
    r.append(o, a), r.style.display = "none", i.appendChild(r);
    const l = document.createElement("div");
    l.style.display = "none", i.appendChild(l);
    const c = document.createElement("div"), h = cn(16), d = document.createElement("span");
    c.append(h, d), c.style.display = "none", i.appendChild(c);
    const u = document.createElement("div");
    u.style.color = "#b5cea8", u.style.display = "none", i.appendChild(u);
    const p = document.createElement("div");
    p.style.display = "none", i.appendChild(p);
    const f = document.createElement("div"), m = cn(16), g = document.createElement("span");
    f.append(m, g), f.style.display = "none", i.appendChild(f);
    function y() {
      const P = document.createElement("div");
      P.style.color = "#aaa";
      const $ = document.createElement("span"), et = cn(14), V = document.createElement("span");
      return P.append($, et, V), P;
    }
    const w = oo(i, y), x = document.createElement("div");
    x.style.color = "#9cdcfe", x.style.display = "none", i.appendChild(x);
    const b = document.createElement("div");
    b.style.display = "none", t.appendChild(b);
    const _ = document.createElement("div");
    _.style.display = "none", t.appendChild(_);
    const v = document.createElement("div");
    v.style.display = "none", t.appendChild(v);
    const C = document.createElement("div");
    C.style.display = "none", t.appendChild(C);
    const E = document.createElement("div");
    E.style.cssText = "position:absolute;top:8px;right:8px;background:#141414;color:#e0e0e0;border:1px solid #888;padding:8px 10px;font:12px monospace;border-radius:4px;display:none;z-index:20;min-width:220px;max-width:340px;line-height:1.55;box-shadow:0 4px 14px rgba(0,0,0,0.5)", n.appendChild(E);
    const R = document.createElement("div");
    R.style.cssText = "display:flex;justify-content:space-between;align-items:center;margin-bottom:4px";
    const k = document.createElement("span");
    k.style.cssText = "color:#8af;font-weight:bold", k.textContent = "pinned";
    const T = document.createElement("span");
    T.style.color = "#888", R.append(k, T), E.appendChild(R);
    const O = document.createElement("div");
    E.appendChild(O);
    const U = document.createElement("div"), G = cn(16), B = document.createElement("b");
    U.append(G, B), U.style.display = "none", O.appendChild(U);
    const L = document.createElement("span");
    L.style.color = "#888", L.textContent = "no entity at tile", L.style.display = "none", O.appendChild(L);
    const D = document.createElement("div");
    D.style.display = "none", O.appendChild(D);
    const K = document.createElement("div"), X = cn(16), Z = document.createElement("span");
    K.append(X, Z), K.style.display = "none", O.appendChild(K);
    const M = document.createElement("div");
    M.style.color = "#b5cea8", M.style.display = "none", O.appendChild(M);
    const N = document.createElement("div");
    N.style.display = "none", O.appendChild(N);
    const W = document.createElement("div"), z = cn(16), q = document.createElement("span");
    W.append(z, q), W.style.display = "none", O.appendChild(W);
    const Q = oo(O, y), at = document.createElement("div");
    at.style.color = "#9cdcfe", at.style.display = "none", O.appendChild(at);
    const st = document.createElement("div");
    st.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", st.style.display = "none";
    const mt = document.createElement("span"), _t = document.createElement("span");
    _t.style.color = "#888", st.append(mt, _t), E.appendChild(st);
    const Y = document.createElement("div");
    Y.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", Y.style.display = "none", E.appendChild(Y);
    const tt = document.createElement("div");
    tt.style.marginTop = "4px", tt.style.display = "none", E.appendChild(tt);
    const ot = document.createElement("div");
    ot.style.display = "none", E.appendChild(ot);
    const dt = document.createElement("div");
    dt.style.marginTop = "4px", ot.appendChild(dt);
    function bt() {
      const P = document.createElement("div");
      return P.style.marginLeft = "4px", P;
    }
    const Tt = oo(ot, bt), jt = document.createElement("div");
    jt.style.cssText = "color:#555;margin-top:6px;font-size:10px", jt.textContent = "click elsewhere or press Esc to unpin", E.appendChild(jt), document.addEventListener("mousemove", (P) => {
      t.style.left = P.clientX + 14 + "px", t.style.top = P.clientY - 10 + "px";
    });
    let H = null, J = null, it = null, pt = null, Lt = null, Ot = null;
    const Dt = /* @__PURE__ */ new Set();
    function Zt() {
      const P = Ot ? {
        x: Ot.x,
        y: Ot.y
      } : null;
      for (const $ of Dt) $(P);
    }
    function Xt(P, $) {
      cs($.headerIcon, P.name), $.headerName.textContent = me(P.name), $.header.style.display = "", P.direction && P.name !== "pipe" ? ($.dirRow.textContent = `${Iw[P.direction] ?? ""} ${P.direction}`, $.dirRow.style.display = "") : $.dirRow.style.display = "none", P.carries ? (cs($.carriesIcon, P.carries), $.carriesName.textContent = " " + me(P.carries), $.carriesRow.style.display = "") : $.carriesRow.style.display = "none", P.rate != null ? ($.rateRow.textContent = `${P.rate.toFixed(1)}/s`, $.rateRow.style.display = "") : $.rateRow.style.display = "none", P.io_type ? ($.ioRow.textContent = `io: ${P.io_type}`, $.ioRow.style.display = "") : $.ioRow.style.display = "none";
      let et = 0;
      if (P.recipe) {
        cs($.recipeIcon, P.recipe), $.recipeName.textContent = " " + me(P.recipe), $.recipeRow.style.display = "";
        const V = Zx(P.recipe);
        if (V) {
          const lt = [
            ...V.inputs.map((wt) => ({
              arrow: "\u25B6",
              item: wt.item,
              rate: wt.rate
            })),
            ...V.outputs.map((wt) => ({
              arrow: "\u25C0",
              item: wt.item,
              rate: wt.rate
            }))
          ];
          for (const wt of lt) {
            const At = $.flowPool.get(et++), [Bt, qt, Ye] = At.children;
            Bt.textContent = `${wt.arrow} `, cs(qt, wt.item), Ye.textContent = `${me(wt.item)} ${wt.rate.toFixed(1)}/s`;
          }
        }
      } else $.recipeRow.style.display = "none";
      return $.flowPool.trim(et), P.segment_id ? ($.segmentRow.textContent = P.segment_id, $.segmentRow.style.display = "") : $.segmentRow.style.display = "none", et;
    }
    function re(P) {
      if (P.ghosts.length === 0) return b.style.display = "none", false;
      if (b.style.display = "", P.ghosts.length === 1) {
        const $ = P.ghosts[0], et = $.direction ? Nc[$.direction] : "";
        b.textContent = "";
        const V = document.createElement("span");
        V.style.color = "#8af", V.textContent = "ghost ";
        const lt = cn(12);
        cs(lt, $.item);
        const wt = document.createTextNode(`${$.item} ${et}`);
        b.append(V, lt, wt);
      } else {
        b.textContent = "";
        const $ = document.createElement("span");
        $.style.color = "#8af", $.textContent = `${P.ghosts.length} ghosts crossing`, b.appendChild($);
      }
      return true;
    }
    function kt(P) {
      if (!P.axis) return _.style.display = "none", false;
      const { vert: $, horiz: et } = P.axis;
      if ($ === 0 && et === 0) return _.style.display = "none", false;
      const V = $ >= 2 || et >= 2, lt = $ >= 1 && et >= 1, wt = V ? "#ff6060" : lt ? "#60b0ff" : "#888";
      return _.style.display = "", _.style.color = wt, _.textContent = `axis V${$} H${et}`, true;
    }
    function Mt(P) {
      if (!P.junction) return v.style.display = "none", false;
      const $ = P.junction, et = $.outcome === "Solved" ? "#80d080" : $.outcome === "Capped" ? "#e0b060" : "#c06060";
      return v.style.display = "", v.style.color = et, v.textContent = `junction seed (${$.seedX},${$.seedY}) \xB7 ${$.outcome}`, true;
    }
    function Wt(P) {
      if (P.cappedSides.length === 0) return C.style.display = "none", false;
      const $ = P.cappedSides.map((et) => {
        const V = et.required - et.shortfall;
        return `\u26A0 ${et.sideIsOutput ? "out" : "in"}: ${et.placedCount}\xD7${et.placedEntity} moves ${V.toFixed(2)}/s of ${et.required.toFixed(2)}/s \xB7 ${et.limit}`;
      });
      return C.style.display = "", C.style.color = "#ffa060", C.textContent = $.join(" | "), true;
    }
    function de(P) {
      if (P.ghosts.length === 0) {
        ot.style.display = "none";
        return;
      }
      ot.style.display = "", P.ghosts.length >= 2 ? (dt.style.color = "#ffa060", dt.textContent = `\u26A0 ${P.ghosts.length} ghost specs at this tile`) : (dt.style.color = "#8af", dt.textContent = "ghost");
      let $ = 0;
      for (const et of P.ghosts) {
        const V = et.direction ? Nc[et.direction] : "\xB7", lt = Tt.get($++);
        lt.textContent = "";
        const wt = document.createTextNode(`${V} `), At = cn(14);
        cs(At, et.item);
        const Bt = document.createTextNode(et.item);
        if (lt.append(wt, At, Bt), et.isStart) {
          const qt = document.createElement("span");
          qt.style.color = "#80d080", qt.textContent = " start", lt.appendChild(qt);
        } else if (et.isEnd) {
          const qt = document.createElement("span");
          qt.style.color = "#d08080", qt.textContent = " end", lt.appendChild(qt);
        }
      }
      Tt.trim($);
    }
    function A() {
      if (J !== null) {
        e.innerHTML = J, e.style.display = "", i.style.display = "none", b.style.display = "none", _.style.display = "none", v.style.display = "none", pt ? (s.textContent = `(${pt.x}, ${pt.y})`, s.style.display = "", s.style.display = "block") : s.style.display = "none", t.style.display = "block";
        return;
      }
      e.style.display = "none", e.innerHTML = "", i.style.display = "";
      let P = false;
      if (it ? (P = true, Xt(it, {
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
        const $ = Lt == null ? void 0 : Lt.lookup(pt.x, pt.y);
        $ ? (re($) && (P = true), kt($) && (P = true), Mt($) && (P = true), Wt($) && (P = true)) : (b.style.display = "none", _.style.display = "none", v.style.display = "none", C.style.display = "none"), s.textContent = `(${pt.x}, ${pt.y})`, s.style.display = "block", P = true;
      } else s.style.display = "none", b.style.display = "none", _.style.display = "none", v.style.display = "none";
      if (!P) {
        t.style.display = "none", H && H.clearHighlight();
        return;
      }
      t.style.display = "block", it && H ? H.highlightBeltNetwork(it) : H && H.clearHighlight();
    }
    function I() {
      if (!Ot) {
        E.style.display = "none";
        return;
      }
      const { entity: P, x: $, y: et } = Ot, V = Lt == null ? void 0 : Lt.lookup($, et);
      if (T.textContent = `(${$}, ${et})`, P ? (L.style.display = "none", Xt(P, {
        header: U,
        headerIcon: G,
        headerName: B,
        dirRow: D,
        carriesRow: K,
        carriesIcon: X,
        carriesName: Z,
        rateRow: M,
        ioRow: N,
        recipeRow: W,
        recipeIcon: z,
        recipeName: q,
        flowPool: Q,
        segmentRow: at
      })) : (U.style.display = "none", D.style.display = "none", K.style.display = "none", M.style.display = "none", N.style.display = "none", W.style.display = "none", Q.trim(0), at.style.display = "none", L.style.display = ""), V) {
        if (V.junction) {
          const lt = V.junction.outcome === "Solved" ? "#80d080" : V.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
          mt.style.color = lt, mt.textContent = `junction seed (${V.junction.seedX},${V.junction.seedY})`, _t.textContent = ` \xB7 ${V.junction.outcome}`, st.style.display = "";
        } else st.style.display = "none";
        if (V.axis) {
          const { vert: lt, horiz: wt } = V.axis;
          if (lt > 0 || wt > 0) {
            const At = lt >= 2 || wt >= 2, Bt = lt >= 1 && wt >= 1, qt = At ? " same-axis conflict" : Bt ? " perpendicular crossing" : "", Ye = At ? "#ff6060" : Bt ? "#60b0ff" : "#bbb";
            tt.style.color = Ye, tt.textContent = `axis: V=${lt} H=${wt}${qt}`, tt.style.display = "";
          } else tt.style.display = "none";
        } else tt.style.display = "none";
        if (de(V), V.cappedSides.length > 0) {
          const lt = {
            "tier-cap": "a faster inserter tier at the same slot count would cover this \u2014 max inserter tier is the binding constraint",
            "column-contest": "this side lost the shared inserter column to the other belt; that one column would have covered it",
            geometry: "the row shape offers no further usable slot (belt span / fixed tiles) \u2014 a template geometry limit"
          };
          Y.replaceChildren();
          const wt = document.createElement("div");
          wt.style.color = "#ffa060", wt.textContent = `\u26A0 ${V.cappedSides.length} under-provisioned inserter side${V.cappedSides.length > 1 ? "s" : ""}`, Y.appendChild(wt);
          for (const At of V.cappedSides) {
            const Bt = (At.required - At.shortfall).toFixed(2), qt = document.createElement("div");
            qt.style.cssText = "margin-top:2px;color:#ccc", qt.textContent = `${At.sideIsOutput ? "output" : "input"}: ${At.placedCount}\xD7${At.placedEntity} moves ${Bt}/s of ${At.required.toFixed(2)}/s needed (short ${At.shortfall.toFixed(2)}/s) \u2014 ${At.limit}: ${lt[At.limit] ?? At.limit}`, Y.appendChild(qt);
          }
          Y.style.display = "";
        } else Y.style.display = "none";
      } else st.style.display = "none", tt.style.display = "none", ot.style.display = "none", Y.style.display = "none";
      E.style.display = "block";
    }
    function j() {
      A(), I();
    }
    function rt(P, $, et) {
      it = P, $ !== void 0 && et !== void 0 ? pt = {
        x: $,
        y: et
      } : P && (pt = {
        x: P.x ?? 0,
        y: P.y ?? 0
      }), j();
    }
    return document.addEventListener("keydown", (P) => {
      P.key === "Escape" && Ot && (Ot = null, Zt(), j());
    }), {
      onHover: rt,
      setHighlightController(P) {
        H = P;
      },
      setTooltipOverride(P) {
        J = P, j();
      },
      setCursorTile(P, $) {
        P === null || $ === void 0 ? pt = null : pt = {
          x: P,
          y: $
        }, j();
      },
      setTileContext(P) {
        Lt = P, j();
      },
      pinTile(P, $, et) {
        Ot = {
          entity: P,
          x: $,
          y: et
        }, Zt(), j();
      },
      clearPin() {
        Ot = null, Zt(), j();
      },
      getPinnedTile() {
        return Ot ? {
          x: Ot.x,
          y: Ot.y
        } : null;
      },
      onPinChange(P) {
        return Dt.add(P), () => Dt.delete(P);
      }
    };
  }
  function Lw(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function Fc(n, t, e, s) {
    return e > n ? "E" : e < n ? "W" : s > t ? "S" : s < t ? "N" : null;
  }
  const $w = {
    ghosts: [],
    axis: null,
    junction: null,
    cappedSides: []
  };
  function Bw(n) {
    var _a2;
    if (!n || n.length === 0) return {
      lookup: () => $w
    };
    const t = /* @__PURE__ */ new Map(), e = /* @__PURE__ */ new Map(), s = /* @__PURE__ */ new Map(), i = /* @__PURE__ */ new Map();
    for (const r of n) if (r.phase === "GhostSpecRouted") {
      const { spec_key: o, tiles: a } = r.data, l = Lw(o);
      if (!a || a.length === 0) continue;
      for (let c = 0; c < a.length; c++) {
        const [h, d] = a[c];
        let u = null;
        c < a.length - 1 ? u = Fc(h, d, a[c + 1][0], a[c + 1][1]) : c > 0 && (u = Fc(a[c - 1][0], a[c - 1][1], h, d));
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
  function Ow(n) {
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
      a && (t = x_(a, i, o), a.querySelectorAll("input,select,button").forEach((l) => {
        l.closest("[data-snapshot-keep]") || (l.disabled = true);
      }));
    }
    return {
      load: s,
      clear: e
    };
  }
  const Nw = 2200, Wc = 180, ao = 200, Fw = 8, Ww = [
    "rows_placed",
    "lanes_planned",
    "bus_routed",
    "poles_placed"
  ];
  function or(n) {
    return `${n.x ?? 0},${n.y ?? 0},${n.name},${n.recipe ?? ""}`;
  }
  function Gw(n) {
    const t = /* @__PURE__ */ new Map(), e = n.trace;
    if (!Array.isArray(e)) return t;
    for (const s of e) {
      const i = s;
      i.phase === "PhaseSnapshot" && i.data && t.set(i.data.phase, i.data);
    }
    return t;
  }
  function zw(n) {
    const t = Gw(n), e = [], s = /* @__PURE__ */ new Set();
    for (const r of Ww) {
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
        const c = or(l);
        s.has(c) || (s.add(c), a.push(l));
      }
      e.push({
        phase: r,
        entities: a
      });
    }
    const i = [];
    for (const r of n.entities) {
      const o = or(r);
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
  function Dw(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map(), o = Js(n, t, e, s, (v, C) => {
      r.set(or(v), C);
    }), a = /* @__PURE__ */ new Set();
    for (const v of r.values()) for (const C of v) a.add(C);
    const l = [];
    for (const v of t.children) {
      const C = v;
      a.has(C) || l.push(C);
    }
    for (const v of l) v.alpha = 0;
    for (const v of r.values()) for (const C of v) C.alpha = 0;
    const c = zw(n), h = c.reduce((v, C) => v + (C.entities.length > 0 ? 1 : 0), 0);
    if (h === 0) {
      for (const v of l) v.alpha = 1;
      for (const v of r.values()) for (const C of v) C.alpha = 1;
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
    const u = Math.max(0, Nw - Wc * h) / h, p = [], f = /* @__PURE__ */ new Map();
    let m = 0;
    for (const v of c) {
      if (v.entities.length === 0) continue;
      f.set(v.phase, m);
      const C = Math.min(Fw, u / v.entities.length);
      v.entities.forEach((R, k) => {
        const T = r.get(or(R));
        !T || T.length === 0 || p.push({
          graphics: T,
          revealStartMs: m + k * C
        });
      });
      const E = (v.entities.length - 1) * C;
      m += E + ao + Wc;
    }
    if (l.length > 0) {
      const v = f.get("bus_routed") ?? f.get("rows_placed") ?? 0;
      for (const C of l) p.push({
        graphics: [
          C
        ],
        revealStartMs: v
      });
    }
    p.sort((v, C) => v.revealStartMs - C.revealStartMs);
    const g = performance.now();
    let y = 0, w = false, x = p.length === 0;
    const b = () => {
      if (w || x) return;
      const v = performance.now() - g;
      for (let C = y; C < p.length; C++) {
        const E = p[C];
        if (E.revealStartMs > v) break;
        const R = Math.min(1, (v - E.revealStartMs) / ao);
        for (const k of E.graphics) k.alpha = R;
      }
      for (; y < p.length; ) {
        const C = p[y];
        if (v - C.revealStartMs < ao) break;
        for (const E of C.graphics) E.alpha = 1;
        y++;
      }
      y >= p.length && (x = true, i.ticker.remove(b), Kn());
    };
    return x || (i.ticker.add(b), br()), {
      controller: o,
      handle: {
        cancel() {
          w || x || (w = true, i.ticker.remove(b), Kn(), Ue());
        },
        finish() {
          if (!(w || x)) {
            for (const v of p) for (const C of v.graphics) C.alpha = 1;
            x = true, i.ticker.remove(b), Kn(), Ue();
          }
        },
        isDone() {
          return x || w;
        }
      }
    };
  }
  const Hw = 4243680;
  function Uw(n, t, e, s = 240) {
    const i = new ft();
    t.addChild(i);
    const r = e.x * S, o = e.y * S, a = e.w * S, l = e.h * S;
    let c = 0;
    const h = () => {
      c += n.ticker.deltaMS;
      const d = Math.max(0, (s - c) / s);
      if (d <= 0) {
        n.ticker.remove(h), i.destroy(), Kn();
        return;
      }
      i.clear(), i.rect(r, o, a, l).fill({
        color: Hw,
        alpha: 0.55 * d
      });
    };
    n.ticker.add(h), br();
  }
  const Gi = 150, Gc = 80, jw = 4, Vw = 300, Sn = 4, Tn = 900, Yw = 6, Xw = 250, Gs = 800, zi = 250;
  function Ke(n, t, e) {
    return n <= 1 ? t : Math.min(t, e / n);
  }
  function zc(n, t) {
    return `${n},${t}`;
  }
  function qw(n) {
    return n.split(":")[1] ?? "";
  }
  function Kw(n, t) {
    return n > 0 ? "East" : n < 0 ? "West" : t > 0 ? "South" : "North";
  }
  function Jw(n, t, e, s) {
    const i = tu();
    i.attachTo(n);
    const r = new ft();
    n.addChild(r);
    const o = /* @__PURE__ */ new Map(), a = /* @__PURE__ */ new Set(), l = /* @__PURE__ */ new Map(), c = xr();
    let h = false, d = false, u = false, p = 0;
    const f = () => u ? p : performance.now();
    let m = null, g = 0;
    const y = /* @__PURE__ */ new Map();
    let w = null, x = null;
    const b = [], _ = globalThis.__TRACE_LOGS === true, v = (M, N) => {
      globalThis.__ANIM_LOGS && console.log(`[anim t=${f().toFixed(0)}ms] ${M}`, N);
    };
    function C(M, N) {
      const z = !y.get(M), q = {
        id: M,
        virtualMs: N
      };
      y.set(M, q), z && (w == null ? void 0 : w(q));
    }
    function E(M, N) {
      m === null && (m = M);
      const W = M + (N ? Gi : Gc);
      W > g && (g = W);
    }
    function R(M, N) {
      if (M.length === 0) return;
      const W = f();
      for (const q of M) nr(q, c);
      const z = [
        ...M
      ].sort((q, Q) => {
        const at = (q.y ?? 0) - (Q.y ?? 0);
        return at !== 0 ? at : (q.x ?? 0) - (Q.x ?? 0);
      });
      z.forEach((q, Q) => {
        const at = W + Q * N, st = ds(q);
        if (a.has(st)) return;
        a.add(st), Go(i, q, at, c), l.has(st) || l.set(st, at), mc(i, q.x ?? 0, q.y ?? 0);
        const mt = o.get(zc(q.x ?? 0, q.y ?? 0));
        mt && mt.fadeOutStartMs === null && (mt.fadeOutStartMs = at, E(at, false));
      }), E(W + (z.length - 1) * N, true);
    }
    function k(M, N, W, z, q, Q) {
      const at = zc(M, N), st = o.get(at);
      st && st.specKey === q || ($0(i, M, N, W, z, Q, q), st || o.set(at, {
        specKey: q,
        fadeStartMs: Q,
        fadeOutStartMs: null
      }), E(Q, true));
    }
    function T(M) {
      const N = Ke(M.length, Sn, Tn);
      v("rows_placed", {
        count: M.length,
        stagger_ms: N,
        span_ms: M.length * N
      }), R(M, N);
    }
    function O(M) {
      const N = f(), W = qw(M.spec_key), z = M.tiles;
      if (z.length === 0) return;
      const q = Ke(z.length, jw, Vw);
      v("ghost_routed", {
        spec_key: M.spec_key,
        item: W,
        tile_count: z.length,
        span_ms: z.length * q
      });
      for (let Q = 0; Q < z.length; Q++) {
        const [at, st] = z[Q];
        let mt = 0, _t = 0;
        Q < z.length - 1 ? (mt = z[Q + 1][0] - at, _t = z[Q + 1][1] - st) : Q > 0 && (mt = at - z[Q - 1][0], _t = st - z[Q - 1][1]), k(at, st, Kw(mt, _t), W, M.spec_key, N + Q * q);
      }
    }
    function U(M) {
      const N = M.entities.length, W = Ke(N, Sn, Tn);
      v("committed", {
        source: "spec",
        count: N,
        span_ms: N * W
      }), R(M.entities, W);
    }
    function G(M) {
      const N = f(), W = M.zone_x, z = M.zone_y, q = M.zone_x + M.zone_w - 1, Q = M.zone_y + M.zone_h - 1;
      for (const [mt, _t] of o.entries()) {
        const [Y, tt] = mt.split(",").map(Number);
        Y < W || Y > q || tt < z || tt > Q || _t.fadeOutStartMs === null && (_t.fadeOutStartMs = N, E(N, false), mc(i, Y, tt));
      }
      for (const mt of b) mt.clusterId === M.cluster_id && (mt.cleared = true);
      for (let mt = M.zone_y; mt <= Q; mt++) for (let _t = M.zone_x; _t <= q; _t++) {
        const Y = O0(i, _t, mt);
        for (const tt of Y) a.delete(tt);
      }
      const at = M.entities.length, st = Ke(at, Yw, Xw);
      v("junction", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        count: at,
        span_ms: at * st
      }), R(M.entities, st);
    }
    function B(M) {
      if (d = true, M.phase === "rows_placed") {
        T(M.entities);
        return;
      }
      if (M.phase !== "lanes_planned") {
        if (M.phase === "bus_routed") {
          const N = M.entities.filter((W) => !a.has(ds(W)));
          if (N.length > 0) {
            const W = Ke(N.length, Sn, Tn);
            R(N, W);
          }
          return;
        }
        if (M.phase === "poles_placed") {
          const N = M.entities.filter((W) => !a.has(ds(W)));
          if (N.length > 0) {
            const W = Ke(N.length, Sn, Tn);
            R(N, W);
          }
          return;
        }
      }
    }
    function L(M) {
      const N = f();
      v("cluster_outline", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        lifetime_ms: Gs,
        fade_ms: zi
      }), b.push({
        clusterId: M.cluster_id,
        x: M.zone_x,
        y: M.zone_y,
        w: M.zone_w,
        h: M.zone_h,
        startMs: N,
        cleared: false
      }), E(N, true);
    }
    const D = () => {
      if (h) return;
      const M = f();
      nu(i, M);
      for (const [N, W] of o.entries()) if (W.fadeOutStartMs !== null && M >= W.fadeOutStartMs) {
        const [z, q] = N.split(",").map(Number);
        (M - W.fadeOutStartMs) / Gc >= 1 && o.delete(N);
      }
      for (const N of i.ghostContainer.particleChildren) N.alpha < 0.5 && (N.alpha = Math.min(0.5, N.alpha + 16 / Gi));
      r.clear();
      for (let N = b.length - 1; N >= 0; N--) {
        const W = b[N], z = M - W.startMs;
        if (!(z < 0)) if (W.cleared || z >= Gs) {
          const q = W.cleared ? Math.max(z, Gs - zi) : z;
          if (q >= Gs) {
            b.splice(N, 1);
            continue;
          }
          const Q = Math.max(0, 1 - (q - (Gs - zi)) / zi);
          r.rect(W.x * S, W.y * S, W.w * S, W.h * S), r.stroke({
            width: 2,
            color: 4508927,
            alpha: 0.9 * Q
          });
        } else r.rect(W.x * S, W.y * S, W.w * S, W.h * S), r.stroke({
          width: 2,
          color: 4508927,
          alpha: 0.9
        });
      }
    };
    t.ticker.add(D), br();
    let K = true;
    v("streaming_start", {});
    function X(M, N) {
      if (h || u) return;
      w = N ?? null, _ && console.log(`[stream t=${f().toFixed(0)}] ${M.phase}`, "data" in M ? M.data : void 0);
      const W = f();
      switch (m === null && (m = W), M.phase) {
        case "PhaseSnapshot": {
          const z = M.data;
          z.phase === "rows_placed" && C("machines", W), z.phase === "poles_placed" && C("poles", W), B(z);
          break;
        }
        case "GhostSpecRouted":
          C("ghost_routes", W), O(M.data);
          break;
        case "GhostSpecCommitted":
          C("committed_routes", W), U(M.data);
          break;
        case "JunctionCommitted":
          C("junctions", W), G(M.data);
          break;
        case "GhostClusterSolved":
          L(M.data);
          break;
        case "TrunkBeltCommitted": {
          const z = M.data, q = z.entities.length, Q = Ke(q, Sn, Tn);
          v("committed", {
            source: "trunk",
            count: q,
            span_ms: q * Q
          }), R(z.entities, Q);
          break;
        }
        case "BalancerCommitted": {
          const z = M.data, q = z.entities.length, Q = Ke(q, Sn, Tn);
          v("committed", {
            source: "balancer",
            count: q,
            span_ms: q * Q
          }), R(z.entities, Q);
          break;
        }
        case "OutputMergerCommitted": {
          const z = M.data, q = z.entities.length, Q = Ke(q, Sn, Tn);
          v("committed", {
            source: "merger",
            count: q,
            span_ms: q * Q
          }), R(z.entities, Q);
          break;
        }
        case "PolesCommitted": {
          const z = M.data, q = z.entities.length, Q = Ke(q, Sn, Tn);
          v("committed", {
            source: "poles",
            count: q,
            span_ms: q * Q
          }), R(z.entities, Q);
          break;
        }
        default: {
          M.phase === "LayoutRetried" && (i.clear(), o.clear(), a.clear(), b.length = 0, r.clear(), Ue());
          break;
        }
      }
      w = null;
    }
    return {
      onEvent: X,
      hasCommittedEntities: () => d,
      cancel() {
        h || (h = true, t.ticker.remove(D), K && (Kn(), K = false), i.clear(), o.clear(), a.clear(), b.length = 0, r.clear(), x = null, Ue());
      },
      finish(M) {
        t.ticker.remove(D), K && (Kn(), K = false), B0(i), v("streaming_finish", {
          entity_count: i.count(),
          latest_fade_end_ms: g
        });
        const N = m ?? 0, W = [];
        for (const { particle: Q, iconParticle: at, revealAt: st } of gc()) W.push({
          kind: "particle",
          particle: Q,
          iconParticle: at,
          revealAt: st
        });
        const z = M.entities.filter((Q) => !a.has(ds(Q)));
        if (z.length > 0) {
          for (const at of z) nr(at, c);
          for (const at of z) Go(i, at, N, c), a.add(ds(at));
          const Q = new Set(W.map((at) => at.particle));
          for (const { particle: at, iconParticle: st, revealAt: mt } of gc()) Q.has(at) || W.push({
            kind: "particle",
            particle: at,
            iconParticle: st,
            revealAt: mt
          });
        }
        const q = N0(i, c);
        if (q.size > 0) for (const Q of W) {
          const at = q.get(Q.particle);
          at && (Q.particle = at);
        }
        return x = W, u = true, p = g, Z(p), G0(M, t);
      },
      seekTo(M) {
        if (h || x === null) return;
        const N = m ?? 0, W = Math.max(g, N);
        p = Math.min(W, Math.max(N, M)), v("scrub", {
          virtualMs: p
        }), Z(p);
      },
      getTimeRange() {
        return {
          firstMs: m ?? 0,
          lastMs: g
        };
      },
      getMilestones() {
        return Array.from(y.values()).sort((M, N) => M.virtualMs - N.virtualMs);
      }
    };
    function Z(M) {
      if (x !== null) {
        for (const N of x) {
          const W = M - N.revealAt, z = W <= 0 ? 0 : W >= Gi ? 1 : W / Gi;
          N.particle.alpha = z, N.iconParticle && (N.iconParticle.alpha = z);
        }
        Ue();
      }
    }
  }
  const Zw = 0.03, Qw = 200, lo = [
    "machines",
    "ghost_routes",
    "committed_routes",
    "junctions",
    "poles",
    "optimizing"
  ], tv = {
    machines: "Machines",
    ghost_routes: "Belt routes",
    committed_routes: "Belts placed",
    junctions: "Crossings",
    poles: "Power poles",
    optimizing: "Optimizing"
  };
  function ev(n, t) {
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
    for (const G of lo) {
      const B = document.createElement("div");
      B.className = "ts-chip", B.dataset.milestone = G, B.textContent = tv[G], G === "optimizing" && (B.style.display = "none"), s.appendChild(B), l.set(G, B);
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
    function g(G, B) {
      var _a2;
      if (c) return;
      const L = G.id;
      u.add(L), (_a2 = l.get(L)) == null ? void 0 : _a2.classList.add("ts-chip--reached"), m(L);
      const D = Math.max(1, B.lastMs - B.firstMs), K = (G.virtualMs - B.firstMs) / D;
      f(Math.min(1, Math.max(0, K))), e.classList.add("ts-visible");
    }
    function y(G) {
      return h ? h.firstMs + G * (h.lastMs - h.firstMs) : 0;
    }
    function w(G) {
      for (const B of d) if (Math.abs(G - B.frac) < Zw) return {
        frac: B.frac,
        snapped: true
      };
      return {
        frac: G,
        snapped: false
      };
    }
    function x(G) {
      if (!h) return;
      const B = i.getBoundingClientRect(), L = (G - B.left) / B.width, D = Math.min(1, Math.max(0, L)), { frac: K, snapped: X } = w(D);
      f(K), X ? a.classList.add("ts-thumb--snapped") : a.classList.remove("ts-thumb--snapped"), t(y(K));
    }
    let b = null, _ = null;
    function v(G) {
      if (!c || !h) return;
      G.preventDefault();
      try {
        i.setPointerCapture(G.pointerId);
      } catch {
      }
      const B = (D) => x(D.clientX), L = (D) => {
        b && document.removeEventListener("pointermove", b), _ && document.removeEventListener("pointerup", _), b = null, _ = null, a.classList.remove("ts-thumb--snapped");
      };
      b = B, _ = L, document.addEventListener("pointermove", B), document.addEventListener("pointerup", L, {
        once: true
      }), x(G.clientX);
    }
    i.addEventListener("pointerdown", v);
    function C(G, B) {
      const L = G.lastMs - G.firstMs;
      if (L < Qw || B.length === 0) {
        T();
        return;
      }
      c = true, h = G, e.classList.add("ts-scrub-mode"), e.classList.add("ts-visible"), d = B.map((D) => ({
        id: D.id,
        frac: (D.virtualMs - G.firstMs) / L
      })), s.style.justifyContent = "flex-start", s.style.position = "relative";
      for (const D of l.values()) D.style.position = "absolute", D.style.transform = "translateX(-50%)";
      for (const D of lo) {
        const K = l.get(D);
        if (!K) continue;
        const X = d.find((Z) => Z.id === D);
        X ? (K.style.left = `${X.frac * 100}%`, K.style.display = "", K.classList.add("ts-chip--reached")) : K.style.display = "none";
      }
      R(), requestAnimationFrame(k), f(1), m(null);
    }
    let E = null;
    function R() {
      if (E && E.remove(), !h) return;
      const G = document.createElement("div");
      G.className = "ts-ticks";
      for (const B of d) {
        const L = document.createElement("div");
        L.className = "ts-tick", L.style.left = `${B.frac * 100}%`, G.appendChild(L);
      }
      i.appendChild(G), E = G;
    }
    function k() {
      if (!c) return;
      const G = 6, B = s.clientWidth;
      if (B <= 0) return;
      const L = lo.map((K) => {
        var _a2;
        const X = l.get(K);
        if (!X || X.style.display === "none") return null;
        const Z = K === "optimizing" ? 1 : ((_a2 = d.find((M) => M.id === K)) == null ? void 0 : _a2.frac) ?? 0;
        return {
          el: X,
          originalFrac: Z
        };
      }).filter((K) => K !== null);
      let D = -1 / 0;
      for (const { el: K, originalFrac: X } of L) {
        const Z = K.offsetWidth / 2, M = X * B, N = D + Z + G, W = Math.max(M, N);
        K.style.left = `${W / B * 100}%`, D = W + Z;
      }
    }
    function T() {
      c = false, h = null, d = [], u.clear(), p = null, E && (E.remove(), E = null), e.classList.remove("ts-visible", "ts-scrub-mode"), o.style.width = "0", a.style.left = "0", a.classList.remove("ts-thumb--snapped"), s.style.justifyContent = "space-between", s.style.position = "";
      for (const [G, B] of l) B.style.position = "", B.style.transform = "", B.style.left = "", B.style.display = G === "optimizing" ? "none" : "", B.classList.remove("ts-chip--reached", "ts-chip--active", "ts-chip--in-progress");
    }
    function O(G) {
      const B = l.get("optimizing");
      if (B) {
        if (B.classList.remove("ts-chip--in-progress", "ts-chip--reached"), G === "idle") {
          B.style.display = "none";
          return;
        }
        B.style.display = "", e.classList.add("ts-visible"), c && (B.style.position = "absolute", B.style.transform = "translateX(-50%)", B.style.left = "100%", requestAnimationFrame(k)), G === "active" ? B.classList.add("ts-chip--in-progress") : G === "done" && B.classList.add("ts-chip--reached");
      }
    }
    function U() {
      b && document.removeEventListener("pointermove", b), _ && document.removeEventListener("pointerup", _), i.removeEventListener("pointerdown", v), e.remove();
    }
    return {
      noteMilestone: g,
      arm: C,
      markOptimizeState: O,
      reset: T,
      destroy: U
    };
  }
  const nv = `
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
  function sv() {
    if (document.getElementById("spaghettio-busy-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-busy-style", n.textContent = nv, document.head.appendChild(n);
  }
  const iv = 120;
  function rv(n) {
    sv();
    const t = document.createElement("div");
    t.className = "spaghettio-busy";
    const e = document.createElement("span");
    e.className = "spaghettio-busy-spin", t.appendChild(e);
    const s = document.createElement("span");
    s.textContent = "computing\u2026", t.appendChild(s), n.appendChild(t);
    let i = null;
    Mb((r) => {
      r > 0 ? i === null && !t.classList.contains("visible") && (i = setTimeout(() => {
        t.classList.add("visible"), i = null;
      }, iv)) : (i !== null && (clearTimeout(i), i = null), t.classList.remove("visible"));
    });
  }
  function Je(n, t) {
    return n.filter((e) => e.phase === t);
  }
  const zn = "color:#9cdcfe;font-weight:bold", ue = "color:#888", _e = "color:#e0e0e0", hs = "color:#6a6", Di = "color:#ffaa00", co = "color:#f66", Hi = "color:#c586c0";
  function ov(n) {
    var _a2;
    const t = Array.isArray(n.trace) ? n.trace : [];
    if (t.length === 0) return;
    const e = Je(t, "PhaseTime"), s = Je(t, "SatInvocation"), i = Je(t, "JunctionSolved"), r = Je(t, "JunctionGrowthCapped"), o = Je(t, "GhostClusterSolved"), a = Je(t, "GhostRoutingComplete"), l = Je(t, "RegionWalkerVeto"), c = Je(t, "JunctionGrowthIteration"), h = Je(t, "NegotiateComplete"), d = Je(t, "ValidationCompleted"), u = t.filter((x) => x.phase === "LayoutRetried"), p = e.reduce((x, b) => x + b.data.duration_ms, 0), f = s.reduce((x, b) => x + b.data.solve_time_us, 0), m = s.filter((x) => x.data.satisfied).length, g = ((_a2 = n.entities) == null ? void 0 : _a2.length) ?? 0, y = u.length > 0 ? r.length > 0 ? ` \xB7 ${r.length} capped \xB7 retried (${u[0].data.caps_before} caps recovered)` : ` \xB7 retried (${u[0].data.caps_before} caps recovered)` : r.length > 0 ? ` \xB7 ${r.length} capped` : "", w = r.length > 0 ? Di : u.length > 0 ? Hi : hs;
    if (console.log(`%c\u25B6 layout %c${n.width}\xD7${n.height}  %c${g} entities  %c${p}ms  %cSAT ${Math.round(f / 1e3)}ms (${s.length}\xD7)%c${y}`, zn, _e, ue, _e, Hi, w), console.groupCollapsed("%c  \u21B3 breakdown", ue), e.length > 0) {
      const x = [
        ...e
      ].sort((b, _) => _.data.duration_ms - b.data.duration_ms);
      console.log(`%cphases%c ${p}ms total`, zn, ue);
      for (const b of x) {
        const _ = b.data, v = p > 0 ? _.duration_ms / p * 100 : 0, C = Math.max(1, Math.round(v / 100 * 24)), E = "\u2588".repeat(C);
        console.log(`  %c${_.phase.padEnd(18)}%c ${String(_.duration_ms).padStart(5)}ms  %c${E}%c ${v.toFixed(1)}%`, ue, _e, Hi, ue);
      }
    }
    if (s.length > 0) {
      const x = f / 1e3, b = p > 0 ? x / p * 100 : 0, _ = f / s.length, v = [
        ...s
      ].sort((E, R) => R.data.solve_time_us - E.data.solve_time_us)[0], C = [
        ...s
      ].sort((E, R) => R.data.zone_w * R.data.zone_h - E.data.zone_w * E.data.zone_h)[0];
      console.log(`%cSAT%c ${s.length} invocations \xB7 ${x.toFixed(1)}ms (%c${b.toFixed(1)}%%%c of total)`, zn, _e, Hi, _e), console.log(`  %csatisfied%c ${m}  %cunsat%c ${s.length - m}  %cavg%c ${(_ / 1e3).toFixed(2)}ms`, ue, hs, ue, co, ue, _e), v && console.log(`  %cslowest call%c ${(v.data.solve_time_us / 1e3).toFixed(1)}ms \u2014 %c${v.data.zone_w}\xD7${v.data.zone_h} @ (${v.data.zone_x},${v.data.zone_y}), ${v.data.variables} vars, ${v.data.clauses} clauses`, ue, _e, ue), C && C !== v && console.log(`  %cbiggest zone%c ${C.data.zone_w}\xD7${C.data.zone_h} @ (${C.data.zone_x},${C.data.zone_y}) \u2014 ${C.data.variables} vars`, ue, _e);
    }
    if (o.length > 0 || i.length > 0 || r.length > 0) {
      if (console.log("%cjunctions", zn), console.log(`  %cclusters%c ${o.length}  %csolved%c ${i.length}  %ccapped%c ${r.length}  %cvetoes%c ${l.length}`, ue, _e, ue, hs, ue, r.length > 0 ? Di : _e, ue, _e), c.length > 0) {
        const x = /* @__PURE__ */ new Map();
        for (const _ of c) {
          const v = `${_.data.seed_x},${_.data.seed_y}`;
          x.set(v, Math.max(x.get(v) ?? 0, _.data.iter));
        }
        const b = [
          ...x.entries()
        ].sort((_, v) => v[1] - _[1])[0];
        b && b[1] > 0 && console.log(`  %chardest%c junction at (${b[0]}) needed ${b[1] + 1} growth iters`, ue, _e);
      }
      if (r.length > 0) for (const x of r) console.log(`    %c\u26A0 capped at (${x.data.tile_x},${x.data.tile_y})%c \u2014 ${x.data.reason}, ${x.data.region_tiles} tiles after ${x.data.iters} iters`, Di, ue);
    }
    if (a.length > 0) {
      const x = a[0].data, b = x.unroutable_count > 0 ? co : hs;
      console.log(`%cghost router%c ${x.entity_count} routed entities, ${x.cluster_count} clusters, max cluster ${x.max_cluster_tiles} tiles  %c${x.unroutable_count} unroutable`, zn, _e, b);
    }
    if (h.length > 0) {
      const x = h[0].data;
      console.log(`%cA* negotiate%c ${x.specs} specs, ${x.iterations} iters, ${x.duration_ms}ms`, zn, _e);
    }
    if (d.length > 0) {
      const x = d[0].data, b = x.error_count > 0 ? co : hs, _ = x.warning_count > 0 ? Di : hs;
      console.log(`%cvalidation  %c${x.error_count} errors  %c${x.warning_count} warnings`, zn, b, _);
    }
    console.groupEnd();
  }
  const av = [
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
  async function lv() {
    await hu();
    const n = Hb();
    await s0(av);
    const t = document.getElementById("app"), e = window.location.hash, s = new URLSearchParams(window.location.search);
    if (e.startsWith("#/balancers")) {
      const { renderBalancerShowcase: r } = await Mn(async () => {
        const { renderBalancerShowcase: o } = await import("./balancers-a8gWnH6Q.js");
        return {
          renderBalancerShowcase: o
        };
      }, []);
      r(t, n);
      return;
    }
    if (!(e.startsWith("#/layout") || s.has("generator") || yb())) {
      const r = document.createElement("div");
      t.appendChild(r), Kb(r, n, {
        onOpenGenerator: () => {
          r.remove(), Dc(n), window.history.replaceState({}, "", "#/layout");
        }
      });
      return;
    }
    Dc(n);
  }
  async function Dc(n) {
    const t = document.getElementById("canvas-container");
    if (!t) throw new Error("Missing #canvas-container element");
    const e = document.getElementById("app");
    e.style.display = "flex";
    const s = document.getElementById("sidebar");
    s && (s.style.display = ""), t.style.display = "";
    const { app: i, viewport: r, requestRender: o, beginAnimating: a, endAnimating: l } = await m0(t), c = g0(r);
    let h = false;
    Ws(r, null), Ew();
    const d = Mw(t), { debugCb: u, colorCb: p, heatmapCb: f, regionsCb: m, soloRegionsCb: g, ghostTilesCb: y, traceOverlayCb: w } = d, x = Pw(t);
    Ri(p.checked);
    const b = () => {
      globalThis.__ANIM_LOGS = u.checked;
    };
    u.addEventListener("change", b), b();
    const _ = Rw(t), v = sw();
    let C = null;
    const E = uw(t, r, {
      onChange: (F) => {
        if (v.update(F), F) {
          T.alpha = R.isActive() ? 0.2 : 0.35;
          const nt = F.iter.bbox;
          C = {
            bboxX: nt.x,
            bboxY: nt.y,
            bboxW: nt.w,
            bboxH: nt.h
          };
        } else T.alpha = 1, C = null, R.isActive() && R.exit();
        o();
      },
      onEditRequested: (F) => {
        T.alpha = 0.2, R.enter(F), o();
      }
    }), R = Aw({
      viewport: r,
      canvas: i.canvas,
      engine: n,
      jd: E,
      satZoneOverlayLayer: v.layer
    });
    y_(t, (F) => et.load(F));
    function k(F) {
      T.removeChildren();
      const nt = tu();
      return nt.attachTo(T), F0(F, nt, i);
    }
    const T = new Ut();
    T.isRenderGroup = true, T.eventMode = "none", r.addChild(T);
    const O = new Ut();
    O.eventMode = "none", r.addChild(O), r.addChild(v.layer);
    const U = new ft();
    U.label = "pin-highlight", r.addChild(U), _.onPinChange((F) => {
      if (U.clear(), F) {
        const nt = F.x * S, ut = F.y * S;
        U.setStrokeStyle({
          width: 2,
          color: 8440063,
          alpha: 0.95
        }), U.rect(nt - 2, ut - 2, S + 4, S + 4).stroke();
      }
      o();
    }), r.moveCenter(nn / 2, nn / 2);
    const G = (F) => {
    };
    let B = null;
    function L(F) {
      B = F, _.onHover(F, F == null ? void 0 : F.x, F == null ? void 0 : F.y);
    }
    let D = /* @__PURE__ */ new Map();
    function K(F) {
      const nt = /* @__PURE__ */ new Map();
      for (const ut of F.entities) {
        const yt = ut.x ?? 0, ht = ut.y ?? 0, $t = Ve[ut.name];
        if ($t) {
          const [Gt, Pt] = $t;
          for (let Nt = 0; Nt < Pt; Nt++) for (let ye = 0; ye < Gt; ye++) nt.set(`${yt + ye},${ht + Nt}`, ut);
        } else if (ie.has(ut.name)) {
          nt.set(`${yt},${ht}`, ut);
          const [Gt, Pt] = pa(ut.direction);
          nt.set(`${yt + Gt},${ht + Pt}`, ut);
        } else nt.set(`${yt},${ht}`, ut);
      }
      D = nt;
    }
    function X(F) {
      return {
        highlightItem: (nt) => {
          F.highlightItem(nt), o();
        },
        highlightBeltNetwork: (nt) => {
          F.highlightBeltNetwork(nt), o();
        },
        clearHighlight: () => {
          F.clearHighlight(), o();
        },
        chainKey: F.chainKey
      };
    }
    let Z = false, M = null, N = null, W = false, z = null;
    const q = {
      update() {
      },
      getPhaseIndex() {
        return -1;
      },
      reset() {
      }
    };
    function Q() {
      var _a2;
      N && (T.removeChild(N), N.destroy(), N = null);
      const F = q.getPhaseIndex();
      if (u.checked && F >= 0, W && (z == null ? void 0 : z.cancel(), z = null, W = false, A)) {
        const ut = Js(A, T, L, G);
        _.setHighlightController(X(ut)), o();
      }
      if (!u.checked || !w.checked || !((_a2 = A == null ? void 0 : A.trace) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      const nt = A.trace;
      N = b_(nt, A.width ?? 0, A.height ?? 0, T, (ut) => {
        _.setTooltipOverride(ut ? `<span style="color:#8af">TRACE</span> ${ut}` : null);
      }), o();
    }
    let at = null, st = null, mt = null, _t = [], Y = null, tt = null;
    const ot = document.createElement("div");
    ot.className = "validation-badge", ot.style.display = "none", t.appendChild(ot);
    function dt(F) {
      if (!F || F.length === 0) {
        ot.style.display = "none";
        return;
      }
      const nt = F.filter((ht) => ht.severity === "Error").length, ut = F.length - nt;
      let yt;
      nt > 0 && ut > 0 ? yt = `\u26A0 ${nt} error${nt > 1 ? "s" : ""}, ${ut} warning${ut > 1 ? "s" : ""}` : nt > 0 ? yt = `\u26A0 ${nt} error${nt > 1 ? "s" : ""}` : yt = `\u26A0 ${ut} warning${ut > 1 ? "s" : ""}`, ot.textContent = yt, ot.classList.toggle("has-errors", nt > 0), ot.style.display = "block";
    }
    let bt = null, Tt = null, jt = null, H = null, J = null;
    const it = 1;
    function pt(F, nt) {
      const ut = F * S + S / 2, yt = nt * S + S / 2;
      r.scale.x < it && r.setZoom(it, false), r.moveCenter(ut, yt);
    }
    function Lt(F) {
      var _a2, _b2;
      const nt = [];
      for (const ut of F.regions ?? []) {
        if (ut.kind !== "unresolved") continue;
        const yt = ut.x + Math.floor(ut.width / 2), ht = ut.y + Math.floor(ut.height / 2), $t = ((_b2 = (_a2 = ut.ports) == null ? void 0 : _a2.find((Gt) => Gt.item)) == null ? void 0 : _b2.item) ?? "unknown";
        nt.push({
          severity: "Warning",
          category: `ghost-router \xB7 ${$t}`,
          message: `unresolved crossing at (${yt}, ${ht})`,
          x: yt,
          y: ht
        });
      }
      for (const ut of F.warnings ?? []) /^ghost router:.*unresolved crossings/i.test(ut) || nt.push({
        severity: "Warning",
        category: "layout",
        message: ut,
        x: void 0,
        y: void 0
      });
      return nt;
    }
    function Ot() {
      if (at && (T.removeChild(at), at.destroy(), at = null), A && !st && tt !== A) {
        const ht = A;
        tt = ht, n.validateLayout(ht, I).then(($t) => {
          A === ht && (st = $t, tt = null, Ot(), fi(ht));
        }).catch(() => {
          A === ht && (st = [], tt = null, Ot(), fi(ht));
        });
      }
      const F = A ? Lt(A) : [], nt = [
        ...st ?? [],
        ...F
      ], ut = (ht) => {
        if (ht.x == null || ht.y == null || !Y) return null;
        const $t = Y.lookup(ht.x, ht.y).cappedSides;
        return $t.length === 0 ? null : [
          ...new Set($t.map((Pt) => Pt.limit))
        ].sort().join("+");
      };
      if (Xe == null ? void 0 : Xe.updateValidation(nt, pt, ut), dt(nt), Dt(nt), !A || nt.length === 0) {
        o();
        return;
      }
      at = C_(nt, T, (ht) => {
        _.setTooltipOverride(ht ? `<span style="color:#f44">VALIDATION</span> ${ht}` : null);
      }).layer, o();
    }
    function Dt(F) {
      if (F && (_t = F), mt && (T.removeChild(mt), mt.destroy({
        children: true
      }), mt = null), !f.checked || !A || _t.length === 0) {
        o();
        return;
      }
      mt = T_(_t, A.entities, T), o();
    }
    function Zt() {
      if (J && (r.removeChild(J), J.destroy({
        children: true
      }), J = null), !u.checked || !y.checked || !A) {
        o();
        return;
      }
      const F = dw(A.trace);
      if (!F) {
        o();
        return;
      }
      J = F, r.addChildAt(J, 0), o();
    }
    function Xt() {
      var _a2;
      if (bt && (T.removeChild(bt), bt.destroy(), bt = null), jt && (T.removeChild(jt), jt.destroy(), jt = null), Tt = null, H = null, !u.checked || !(m == null ? void 0 : m.checked) || !A) {
        o();
        return;
      }
      if (A.regions && A.regions.length > 0) {
        const F = B_(A);
        bt = F.layer, Tt = F.hitTest, T.addChild(bt);
      }
      if ((_a2 = A.trace) == null ? void 0 : _a2.length) {
        const F = G_(A.trace);
        if (F.length > 0) {
          const nt = V_(F);
          jt = nt.layer, H = nt.hitTest, T.addChild(jt);
        }
      }
      o();
    }
    const re = document.createElement("div");
    re.style.cssText = "position:absolute;bottom:34px;left:8px;background:rgba(0,0,0,0.8);color:#e0e0e0;font:11px monospace;padding:6px 8px;border-radius:3px;border:1px solid #00e0a0;z-index:10;display:none;min-width:200px", t.appendChild(re);
    const kt = document.createElement("div");
    kt.style.cssText = "color:#00e0a0;margin-bottom:4px", re.appendChild(kt);
    const Mt = document.createElement("textarea");
    Mt.placeholder = "Add a note\u2026", Mt.rows = 2, Mt.style.cssText = "width:100%;box-sizing:border-box;background:#2a2a2a;color:#e0e0e0;border:1px solid #555;border-radius:2px;font:11px monospace;resize:vertical;margin-bottom:4px", re.appendChild(Mt);
    const Wt = document.createElement("div");
    Wt.style.cssText = "color:#777", Wt.textContent = "Ctrl+C to copy JSON", re.appendChild(Wt);
    let de = false, A = null, I = null, j = null, rt = null, P = null;
    const $ = ev(t, (F) => P == null ? void 0 : P.seekTo(F));
    rv(t);
    const et = Ow({
      sidebarEl: document.getElementById("sidebar"),
      getSidebarCtrl: () => Xe,
      renderLayoutOnCanvas: Oe,
      setCachedValidationIssues: (F) => {
        st = F;
      },
      updateValidationOverlay: Ot,
      panToTile: pt,
      onDebugEnable: () => d.setDebugEnabled(true),
      onClear: () => {
        et.clear(), T.removeChildren(), O.removeChildren(), _.clearPin(), _.setTileContext(null), A = null, I = null, x.update(null), D = /* @__PURE__ */ new Map(), st = null, Ws(r, null), r.moveCenter(nn / 2, nn / 2), dt(null), Xe == null ? void 0 : Xe.updateValidation([], pt), E.close();
      }
    });
    function V(F) {
      F.length === 0 ? (re.style.display = "none", Mt.value = "") : (kt.textContent = `${F.length} entit${F.length === 1 ? "y" : "ies"} selected`, re.style.display = "block");
    }
    async function lt(F) {
      if (de || !(F.regions ?? []).some((Ft) => Ft.kind === "crossing_zone")) return;
      de = true, $.markOptimizeState("active"), j && (j.destroy(), j = null);
      const ut = (Ft) => Ft === "transport-belt" || Ft === "fast-transport-belt" || Ft === "express-transport-belt" || Ft === "underground-belt" || Ft === "fast-underground-belt" || Ft === "express-underground-belt", yt = [], ht = 130, $t = 90, Gt = 520;
      let Pt = 0, Nt = -1, ye = 0, be = false, $n = false, ns = null;
      const vu = (Ft) => {
        if (!A) return;
        const qe = Ft.zone_x, Ne = Ft.zone_y, ss = qe + Ft.zone_w, _r = Ne + Ft.zone_h, Su = (Ps) => {
          const ka = Ps.x ?? 0, Ma = Ps.y ?? 0;
          return ka >= qe && ka < ss && Ma >= Ne && Ma < _r;
        };
        A.entities = A.entities.filter((Ps) => !(Su(Ps) && ut(Ps.name))).concat(Ft.entities), Uw(i, r, {
          x: qe,
          y: Ne,
          w: Ft.zone_w,
          h: Ft.zone_h
        }), k(A), o();
      }, Cu = () => be && yt.length === 0, Ea = (Ft) => {
        if (!$n) {
          for (; yt.length > 0; ) {
            const qe = yt[0], Ne = qe.imp.region_id === Nt, ss = Ne ? Math.min(Gt, ye * $t) : 0, _r = ht + ss;
            if (Ft - Pt < _r) break;
            yt.shift(), vu(qe.imp), Ne ? ye += 1 : (ye = 1, Nt = qe.imp.region_id), Pt = Ft;
            break;
          }
          if (Cu()) {
            ns = null;
            return;
          }
          ns = requestAnimationFrame(Ea);
        }
      };
      ns = requestAnimationFrame(Ea);
      try {
        const Ft = await n.optimizeAllRegions(F, {
          perRegionBudgetMs: 800,
          onImprovement: (Ne) => {
            Ne.iter !== 0 && yt.push({
              imp: Ne
            });
          }
        });
        be = true, await new Promise((Ne) => {
          const ss = () => {
            yt.length === 0 ? Ne() : requestAnimationFrame(ss);
          };
          ss();
        }), A = Ft, x.update(Ft), K(Ft), window.__layout = Ft;
        const qe = k(Ft);
        _.setHighlightController(X(qe)), o();
      } catch (Ft) {
        (Ft instanceof Error ? Ft.message : String(Ft)).includes("superseded") || console.error("[auto-optimize] failed", Ft);
      } finally {
        $n = true, be = true, ns !== null && cancelAnimationFrame(ns), de = false, $.markOptimizeState("done"), A && (j = _c(i.canvas, r, T, A, V));
      }
    }
    i.canvas.addEventListener("pointermove", (F) => {
      const nt = i.canvas.getBoundingClientRect(), ut = F.clientX - nt.left, yt = F.clientY - nt.top, ht = r.toWorld(ut, yt), $t = Math.floor(ht.x / S), Gt = Math.floor(ht.y / S);
      _.setCursorTile($t, Gt);
      const Pt = D.get(`${$t},${Gt}`) ?? null;
      Pt !== B && L(Pt), B || _.onHover(null, $t, Gt);
    }), i.canvas.addEventListener("pointerleave", () => {
      _.setCursorTile(null), B && L(null);
    });
    const wt = 4;
    let At = null;
    i.canvas.addEventListener("pointerdown", (F) => {
      if (F.button !== 0 || F.shiftKey || F.altKey || F.ctrlKey || F.metaKey) {
        At = null;
        return;
      }
      At = {
        x: F.clientX,
        y: F.clientY,
        shifted: false
      };
    }), i.canvas.addEventListener("pointerup", (F) => {
      if (!At) return;
      const nt = F.clientX - At.x, ut = F.clientY - At.y;
      if (At = null, Math.hypot(nt, ut) > wt || F.button !== 0 || F.shiftKey || F.altKey || F.ctrlKey || F.metaKey) return;
      const yt = i.canvas.getBoundingClientRect(), ht = r.toWorld(F.clientX - yt.left, F.clientY - yt.top), $t = Math.floor(ht.x / S), Gt = Math.floor(ht.y / S);
      if (!m.checked) {
        const be = B && B.x === $t && B.y === Gt ? B : null;
        if (!be) return;
        _.pinTile(be, $t, Gt);
        return;
      }
      const Pt = (H == null ? void 0 : H(ht.x, ht.y)) ?? null;
      if (Pt) {
        E.open(Pt, A == null ? void 0 : A.trace);
        return;
      }
      if (C) {
        const be = ht.x / S, $n = ht.y / S;
        if (!(be >= C.bboxX && $n >= C.bboxY && be < C.bboxX + C.bboxW && $n < C.bboxY + C.bboxH)) {
          E.close();
          return;
        }
      }
      const Nt = (Tt == null ? void 0 : Tt(ht.x, ht.y)) ?? null;
      if (Nt) {
        const be = (Nt.region.x + Nt.region.width / 2) * S, $n = (Nt.region.y + Nt.region.height / 2) * S;
        r.moveCenter(be, $n);
      }
      const ye = _.getPinnedTile();
      if (ye && ye.x === $t && ye.y === Gt) _.clearPin();
      else {
        const be = B && B.x === $t && B.y === Gt ? B : null;
        if (!be) return;
        _.pinTile(be, $t, Gt);
      }
    }), document.addEventListener("keydown", (F) => {
      F.key === "Shift" && r.plugins.pause("drag");
    }), document.addEventListener("keyup", (F) => {
      F.key === "Shift" && r.plugins.resume("drag");
    }), window.addEventListener("blur", () => r.plugins.resume("drag"));
    function Bt(F) {
      rt == null ? void 0 : rt.cancel(), rt = null, P == null ? void 0 : P.cancel(), P = null, $.reset(), T.removeChildren(), O.removeChildren(), I = F, Ws(r, F), F || r.moveCenter(nn / 2, nn / 2);
    }
    function qt() {
      P == null ? void 0 : P.cancel(), $.reset(), Ws(r, null), P = Jw(T, i);
      let F = false, nt = null;
      return (ut) => {
        if (ut.phase === "PhaseSnapshot") {
          const ht = ut.data;
          ht.width > 0 && ht.height > 0 && (F || (r.fit(true, ht.width * S * 1.15, ht.height * S * 1.25), r.moveCenter(ht.width * S / 2, ht.height * S / 2), F = true), nt ? nt.resize(ht.width + 2, ht.height + 2) : nt = Ye(ht.width + 2, ht.height + 2));
        }
        P == null ? void 0 : P.onEvent(ut, (ht) => {
          P && $.noteMilestone(ht, P.getTimeRange());
        });
      };
    }
    function Ye(F, nt) {
      let ut = F, yt = nt, ht = false;
      if (h) return Fn(c, ut, yt), o(), ht = true, {
        cancel: () => {
        },
        resize(Nt, ye) {
          Fn(c, Nt, ye), o();
        }
      };
      const $t = 250, Gt = performance.now();
      c.alpha = 1, Fn(c, ut, yt, 0);
      const Pt = () => {
        if (ht) return;
        const Nt = Math.min(1, (performance.now() - Gt) / $t);
        Fn(c, ut, yt, Nt), o(), Nt >= 1 && (ht = true, h = true, i.ticker.remove(Pt), l());
      };
      return i.ticker.add(Pt), a(), {
        cancel() {
          ht || (ht = true, h = true, i.ticker.remove(Pt), l(), Fn(c, ut, yt), o());
        },
        resize(Nt, ye) {
          ut = Nt, yt = ye, ht && (Fn(c, ut, yt), o());
        }
      };
    }
    function Oe(F, nt) {
      Dd(i0(F.entities)), A = F, x.update(F), nt && (I = nt), K(F), window.__layout = F, ov(F), W = false, z == null ? void 0 : z.cancel(), z = null, j && (j.destroy(), j = null), rt == null ? void 0 : rt.cancel(), rt = null, re.style.display = "none", Mt.value = "", st = null, Ws(r, null);
      let ut;
      if (P == null ? void 0 : P.hasCommittedEntities()) ut = P.finish(F), $.arm(P.getTimeRange(), P.getMilestones());
      else if (P == null ? void 0 : P.cancel(), P = null, $.reset(), (Array.isArray(F.trace) ? F.trace : []).some((Pt) => Pt.phase === "PhaseSnapshot")) {
        const Pt = Dw(F, T, L, G, i);
        ut = Pt.controller, rt = Pt.handle;
      } else ut = k(F);
      _.setHighlightController(X(ut)), Y = Bw(F.trace), _.setTileContext(Y), _.clearPin(), j = _c(i.canvas, r, T, F, V), Q(), Ot(), Xt(), Zt(), Q0(O, F, I);
      const yt = F.width ?? 0, ht = F.height ?? 0;
      if (Fn(c, yt + 2, ht + 2), yt > 0 && ht > 0) {
        const $t = yt * 32, Gt = ht * 32, Pt = 192;
        r.fit(true, $t * 1.1, (Gt + Pt) * 1.2), r.moveCenter($t / 2, (Gt - Pt) / 2);
      }
      Z && (T.alpha = 0.12), o(), requestAnimationFrame(() => {
        A === F && fi(F);
      });
    }
    function fi(F) {
      A !== F || tt === F || st === null || [
        ...st,
        ...Lt(F)
      ].length > 0 || lt(F);
    }
    document.addEventListener("keydown", (F) => {
      if (F.ctrlKey) {
        if (F.key === "c") {
          if (!j || j.getSelected().length === 0) return;
          F.preventDefault();
          const nt = (Xe == null ? void 0 : Xe.getParams()) ?? null, ut = j.buildJson(nt, Mt.value.trim());
          navigator.clipboard.writeText(ut).catch(() => {
          }), Wt.textContent = "Copied!", setTimeout(() => {
            Wt.textContent = "Ctrl+C to copy JSON";
          }, 2e3);
        } else if (F.key === "o") {
          F.preventDefault();
          const nt = document.createElement("input");
          nt.type = "file", nt.accept = ".fls", nt.addEventListener("change", async () => {
            var _a2;
            const ut = (_a2 = nt.files) == null ? void 0 : _a2[0];
            if (ut) try {
              const yt = await ut.text(), ht = await xu(yt);
              et.load(ht);
            } catch (yt) {
              alert(`Failed to load snapshot: ${yt}`);
            }
          }), nt.click();
        }
      }
    });
    const an = document.getElementById("sidebar");
    let Xe = null;
    if (an) {
      let F = function(Pt) {
        const Nt = document.createElement("button");
        return Nt.textContent = Pt, Nt.style.cssText = "flex:1;padding:10px 4px;background:none;border:none;border-bottom:2px solid transparent;color:#777;font:12px 'JetBrains Mono','Consolas',monospace;cursor:pointer;letter-spacing:0.5px;transition:all 0.15s", Nt;
      }, nt = function(Pt) {
        const Nt = Pt === "generate";
        $t.style.display = Nt ? "flex" : "none", Gt.style.display = Nt ? "none" : "flex", yt.style.borderBottomColor = Nt ? "#569cd6" : "transparent", yt.style.color = Nt ? "#d4d4d4" : "#777", ht.style.borderBottomColor = Nt ? "transparent" : "#569cd6", ht.style.color = Nt ? "#777" : "#d4d4d4";
      };
      const ut = document.createElement("div");
      ut.style.cssText = "display:flex;border-bottom:1px solid #2a2a2a;background:#141414;flex-shrink:0";
      const yt = F("Generate"), ht = F("Corpus");
      ut.appendChild(yt), ut.appendChild(ht);
      const $t = document.createElement("div");
      $t.style.cssText = "flex:1;overflow:hidden;display:flex;flex-direction:column;";
      const Gt = document.createElement("div");
      Gt.style.cssText = "flex:1;overflow:hidden;display:none;flex-direction:column;", an.style.cssText += ";display:flex;flex-direction:column;padding:0;overflow:hidden;", an.appendChild(ut), an.appendChild($t), an.appendChild(Gt), yt.onclick = () => nt("generate"), ht.onclick = () => nt("corpus"), nt("generate"), Xe = Sb($t, n, {
        renderGraph: Bt,
        renderLayout: Oe,
        startStreaming: qt
      }), u.addEventListener("change", () => {
        Q(), Ot(), Xt(), Zt();
      }), y.addEventListener("change", () => {
        Zt();
      }), w.addEventListener("change", () => {
        Q();
      }), p.addEventListener("change", () => {
        Ri(p.checked), A && Oe(A);
      }), f.addEventListener("change", () => {
        Dt();
      }), m.addEventListener("change", () => {
        Xt();
      }), g.addEventListener("change", () => {
        const Pt = () => o();
        g.checked ? (Z = true, M = {
          colorChecked: p.checked,
          regionsChecked: m.checked,
          entityAlpha: T.alpha
        }, m.checked || (m.checked = true, Xt()), p.checked && (p.checked = false, Ri(false), A && Oe(A)), T.alpha = 0.12, Xt(), Pt()) : (Z = false, M && (T.alpha = M.entityAlpha, m.checked !== M.regionsChecked && (m.checked = M.regionsChecked, Xt()), p.checked !== M.colorChecked && (p.checked = M.colorChecked, Ri(p.checked), A && Oe(A)), M = null), Pt());
      }), Vb(Gt, Oe);
    }
  }
  lv().catch((n) => {
    console.error("Failed to initialize app:", n);
  });
})();
export {
  ph as $,
  Qo as A,
  qi as B,
  Ut as C,
  nm as D,
  on as E,
  Xp as F,
  di as G,
  Qn as H,
  vs as I,
  ce as J,
  ne as K,
  Kt as L,
  Vs as M,
  sh as N,
  vt as O,
  ct as P,
  ft as Q,
  Ht as R,
  hr as S,
  S as T,
  fe as U,
  hy as V,
  sg as W,
  ur as X,
  Xm as Y,
  bn as Z,
  pr as _,
  __tla,
  nr as a,
  Rt as a0,
  $h as a1,
  _o as a2,
  zt as a3,
  Xn as a4,
  Qs as a5,
  Et as a6,
  Qu as a7,
  Qf as a8,
  Ms as a9,
  ld as aA,
  Bh as aB,
  hi as aC,
  xf as aD,
  pf as aE,
  Vh as aF,
  rm as aG,
  Pm as aH,
  Rm as aI,
  Wm as aJ,
  Nm as aK,
  Gm as aL,
  ll as aa,
  Ui as ab,
  Ko as ac,
  Ih as ad,
  ke as ae,
  Zo as af,
  Mm as ag,
  Im as ah,
  Fm as ai,
  Bm as aj,
  Wu as ak,
  zm as al,
  ve as am,
  eh as an,
  Ee as ao,
  ar as ap,
  ja as aq,
  je as ar,
  zy as as,
  gp as at,
  Ya as au,
  Pr as av,
  Xa as aw,
  yp as ax,
  ih as ay,
  fr as az,
  Nn as b,
  xr as c,
  dv as d,
  Ki as e,
  Jo as f,
  It as g,
  Ct as h,
  $o as i,
  Jn as j,
  dn as k,
  Lo as l,
  Mi as m,
  Yt as n,
  $e as o,
  ud as p,
  pd as q,
  Js as r,
  Ie as s,
  hv as t,
  Py as u,
  ee as v,
  yy as w,
  se as x,
  Jt as y,
  St as z
};
