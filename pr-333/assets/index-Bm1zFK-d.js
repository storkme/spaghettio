const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/browserAll-CAtaZWJn.js","assets/webworkerAll-Cdd-vEc6.js","assets/Filter-BYb4NxAO.js","assets/WebGPURenderer-CrlozIr6.js","assets/BufferResource-C3imr1V9.js","assets/RenderTargetSystem-C_S4rKSS.js","assets/WebGLRenderer-CBX9wrZj.js","assets/CanvasRenderer-6eSCbLWC.js"])))=>i.map(i=>d[i]);
let Mh, da, nr, zt, _m, on, pf, gi, ts, Ss, he, ne, Kt, Ks, bh, vt, lt, ft, Dt, gr, S, me, ky, wg, xr, ug, _n, br, lr, Rt, Kh, Mo, Wt, qn, ii, Mt, yp, ym, Is, Sd, Jh, mi, Of, If, ld, Cm, Xm, Km, sg, eg, ig, vl, Ji, la, Vh, Re, ha, Vm, qm, ng, Qm, sp, rg, Te, yh, Ie, pr, rl, We, rx, $p, al, Nr, ll, Bp, _h, _r, Gn, Sr, Gv, sr, ca, It, Tt, Ho, Zn, un, Do, Oi, Yt, Ge, kd, Md, ni, Ne, Wv, Xy, ee, By, ie, Jt, At;
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
  const Hu = "modulepreload", Uu = function(n) {
    return "/spaghettio/pr-333/" + n;
  }, Ua = {}, Rn = function(t, e, s) {
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
        if (c = Uu(c), c in Ua) return;
        Ua[c] = true;
        const h = c.endsWith(".css"), d = h ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${c}"]${d}`)) return;
        const u = document.createElement("link");
        if (u.rel = h ? "stylesheet" : Hu, h || (u.as = "script"), u.crossOrigin = "", u.href = c, l && u.setAttribute("nonce", l), document.head.appendChild(u), h) return new Promise((p, f) => {
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
  lt = ((n) => (n.Application = "application", n.WebGLPipes = "webgl-pipes", n.WebGLPipesAdaptor = "webgl-pipes-adaptor", n.WebGLSystem = "webgl-system", n.WebGPUPipes = "webgpu-pipes", n.WebGPUPipesAdaptor = "webgpu-pipes-adaptor", n.WebGPUSystem = "webgpu-system", n.CanvasSystem = "canvas-system", n.CanvasPipesAdaptor = "canvas-pipes-adaptor", n.CanvasPipes = "canvas-pipes", n.Asset = "asset", n.LoadParser = "load-parser", n.ResolveParser = "resolve-parser", n.CacheParser = "cache-parser", n.DetectionParser = "detection-parser", n.MaskEffect = "mask-effect", n.BlendMode = "blend-mode", n.TextureSource = "texture-source", n.Environment = "environment", n.ShapeBuilder = "shape-builder", n.Batcher = "batcher", n))(lt || {});
  let _o, vi, ju, Yu;
  _o = (n) => {
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
  vi = (n, t) => _o(n).priority ?? t;
  Wt = {
    _addHandlers: {},
    _removeHandlers: {},
    _queue: {},
    remove(...n) {
      return n.map(_o).forEach((t) => {
        t.type.forEach((e) => {
          var _a2, _b2;
          return (_b2 = (_a2 = this._removeHandlers)[e]) == null ? void 0 : _b2.call(_a2, t);
        });
      }), this;
    },
    add(...n) {
      return n.map(_o).forEach((t) => {
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
        }), t.sort((r, o) => vi(o.value, e) - vi(r.value, e)));
      }, (s) => {
        const i = t.findIndex((r) => r.name === s.name);
        i !== -1 && t.splice(i, 1);
      });
    },
    handleByList(n, t, e = -1) {
      return this.handle(n, (s) => {
        t.includes(s.ref) || (t.push(s.ref), t.sort((i, r) => vi(r, e) - vi(i, e)));
      }, (s) => {
        const i = t.indexOf(s.ref);
        i !== -1 && t.splice(i, 1);
      });
    },
    mixin(n, ...t) {
      for (const e of t) Object.defineProperties(n.prototype, Object.getOwnPropertyDescriptors(e));
    }
  };
  ju = {
    extension: {
      type: lt.Environment,
      name: "browser",
      priority: -1
    },
    test: () => true,
    load: async () => {
      await Rn(() => import("./browserAll-CAtaZWJn.js"), __vite__mapDeps([0,1,2]));
    }
  };
  Yu = {
    extension: {
      type: lt.Environment,
      name: "webworker",
      priority: 0
    },
    test: () => typeof self < "u" && self.WorkerGlobalScope !== void 0,
    load: async () => {
      await Rn(() => import("./webworkerAll-Cdd-vEc6.js"), __vite__mapDeps([1,2]));
    }
  };
  class fe {
    constructor(t, e, s) {
      this._x = e || 0, this._y = s || 0, this._observer = t;
    }
    clone(t) {
      return new fe(t ?? this._observer, this._x, this._y);
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
  function rh(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var kr = {
    exports: {}
  }, ja;
  function Vu() {
    return ja || (ja = 1, (function(n) {
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
    })(kr)), kr.exports;
  }
  var Xu = Vu();
  let qu, Ku, Ju;
  on = rh(Xu);
  qu = Math.PI * 2;
  Ku = 180 / Math.PI;
  Ju = Math.PI / 180;
  Mt = class {
    constructor(t = 0, e = 0) {
      this.x = 0, this.y = 0, this.x = t, this.y = e;
    }
    clone() {
      return new Mt(this.x, this.y);
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
      return Mr.x = 0, Mr.y = 0, Mr;
    }
  };
  const Mr = new Mt();
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
      e = e || new Mt();
      const s = t.x, i = t.y;
      return e.x = this.a * s + this.c * i + this.tx, e.y = this.b * s + this.d * i + this.ty, e;
    }
    applyInverse(t, e) {
      e = e || new Mt();
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
      return c < 1e-5 || Math.abs(qu - c) < 1e-5 ? (t.rotation = l, t.skew.x = t.skew.y = 0) : (t.rotation = 0, t.skew.x = a, t.skew.y = l), t.scale.x = Math.sqrt(e * e + s * s), t.scale.y = Math.sqrt(i * i + r * r), t.position.x = this.tx + (o.x * e + o.y * i), t.position.y = this.ty + (o.x * s + o.y * r), t;
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
      return Qu.identity();
    }
    static get shared() {
      return Zu.identity();
    }
  };
  const Zu = new vt(), Qu = new vt(), Hn = [
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
  ], Un = [
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
  ], jn = [
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
  ], Yn = [
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
  ], wo = [], oh = [], Ci = Math.sign;
  function tp() {
    for (let n = 0; n < 16; n++) {
      const t = [];
      wo.push(t);
      for (let e = 0; e < 16; e++) {
        const s = Ci(Hn[n] * Hn[e] + jn[n] * Un[e]), i = Ci(Un[n] * Hn[e] + Yn[n] * Un[e]), r = Ci(Hn[n] * jn[e] + jn[n] * Yn[e]), o = Ci(Un[n] * jn[e] + Yn[n] * Yn[e]);
        for (let a = 0; a < 16; a++) if (Hn[a] === s && Un[a] === i && jn[a] === r && Yn[a] === o) {
          t.push(a);
          break;
        }
      }
    }
    for (let n = 0; n < 16; n++) {
      const t = new vt();
      t.set(Hn[n], Un[n], jn[n], Yn[n], 0, 0), oh.push(t);
    }
  }
  tp();
  let Si;
  At = {
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
    uX: (n) => Hn[n],
    uY: (n) => Un[n],
    vX: (n) => jn[n],
    vY: (n) => Yn[n],
    inv: (n) => n & 8 ? n & 15 : -n & 7,
    add: (n, t) => wo[n][t],
    sub: (n, t) => wo[n][At.inv(t)],
    rotate180: (n) => n ^ 4,
    isVertical: (n) => (n & 3) === 2,
    byDirection: (n, t) => Math.abs(n) * 2 <= Math.abs(t) ? t >= 0 ? At.S : At.N : Math.abs(t) * 2 <= Math.abs(n) ? n > 0 ? At.E : At.W : t > 0 ? n > 0 ? At.SE : At.SW : n > 0 ? At.NE : At.NW,
    matrixAppendRotationInv: (n, t, e = 0, s = 0, i = 0, r = 0) => {
      const o = oh[At.inv(t)], a = o.a, l = o.b, c = o.c, h = o.d, d = e - Math.min(0, a * i, c * r, a * i + c * r), u = s - Math.min(0, l * i, h * r, l * i + h * r), p = n.a, f = n.b, g = n.c, m = n.d;
      n.a = a * p + l * g, n.b = a * f + l * m, n.c = c * p + h * g, n.d = c * f + h * m, n.tx = d * p + u * g + n.tx, n.ty = d * f + u * m + n.ty;
    },
    transformRectCoords: (n, t, e, s) => {
      const { x: i, y: r, width: o, height: a } = n, { x: l, y: c, width: h, height: d } = t;
      return e === At.E ? (s.set(i + l, r + c, o, a), s) : e === At.S ? s.set(h - r - a + l, i + c, a, o) : e === At.W ? s.set(h - i - o + l, d - r - a + c, o, a) : e === At.N ? s.set(r + l, d - i - o + c, a, o) : s.set(i + l, r + c, o, a);
    }
  };
  Si = [
    new Mt(),
    new Mt(),
    new Mt(),
    new Mt()
  ];
  Dt = class {
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
      return new Dt(0, 0, 0, 0);
    }
    clone() {
      return new Dt(this.x, this.y, this.width, this.height);
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
        const T = this.x < t.x ? t.x : this.x;
        if ((this.right > t.right ? t.right : this.right) <= T) return false;
        const E = this.y < t.y ? t.y : this.y;
        return (this.bottom > t.bottom ? t.bottom : this.bottom) > E;
      }
      const s = this.left, i = this.right, r = this.top, o = this.bottom;
      if (i <= s || o <= r) return false;
      const a = Si[0].set(t.left, t.top), l = Si[1].set(t.left, t.bottom), c = Si[2].set(t.right, t.top), h = Si[3].set(t.right, t.bottom);
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
      return t || (t = new Dt()), t.copyFrom(this), t;
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
  const Pr = {
    default: -1
  };
  ne = function(n = "default") {
    return Pr[n] === void 0 && (Pr[n] = -1), ++Pr[n];
  };
  let Ya, ep, fs;
  Ya = /* @__PURE__ */ new Set();
  ee = "8.0.0";
  ep = "8.3.4";
  fs = {
    quiet: false,
    noColor: false
  };
  It = ((n, t, e = 3) => {
    if (fs.quiet || Ya.has(t)) return;
    let s = new Error().stack;
    const i = `${t}
Deprecated since v${n}`, r = typeof console.groupCollapsed == "function" && !fs.noColor;
    typeof s > "u" ? console.warn("PixiJS Deprecation Warning: ", i) : (s = s.split(`
`).splice(e).join(`
`), r ? (console.groupCollapsed("%cPixiJS Deprecation Warning: %c%s", "color:#614108;background:#fffbe6", "font-weight:normal;color:#614108;background:#fffbe6", i), console.warn(s), console.groupEnd()) : (console.warn("PixiJS Deprecation Warning: ", i), console.warn(s))), Ya.add(t);
  });
  Object.defineProperties(It, {
    quiet: {
      get: () => fs.quiet,
      set: (n) => {
        fs.quiet = n;
      },
      enumerable: true,
      configurable: false
    },
    noColor: {
      get: () => fs.noColor,
      set: (n) => {
        fs.noColor = n;
      },
      enumerable: true,
      configurable: false
    }
  });
  const ah = () => {
  };
  function vs(n) {
    return n += n === 0 ? 1 : 0, --n, n |= n >>> 1, n |= n >>> 2, n |= n >>> 4, n |= n >>> 8, n |= n >>> 16, n + 1;
  }
  function Va(n) {
    return !(n & n - 1) && !!n;
  }
  function lh(n) {
    const t = {};
    for (const e in n) n[e] !== void 0 && (t[e] = n[e]);
    return t;
  }
  const Xa = /* @__PURE__ */ Object.create(null);
  function np(n) {
    const t = Xa[n];
    return t === void 0 && (Xa[n] = ne("resource")), t;
  }
  const ch = class hh extends on {
    constructor(t = {}) {
      super(), this._resourceType = "textureSampler", this._touched = 0, this._maxAnisotropy = 1, this.destroyed = false, t = {
        ...hh.defaultOptions,
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
      return this._sharedResourceId = np(t), this._resourceId;
    }
    destroy() {
      this.destroyed = true, this.emit("destroy", this), this.emit("change", this), this.removeAllListeners();
    }
  };
  ch.defaultOptions = {
    addressMode: "clamp-to-edge",
    scaleMode: "linear"
  };
  Zn = ch;
  const dh = class uh extends on {
    constructor(t = {}) {
      super(), this.options = t, this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.uid = ne("textureSource"), this._resourceType = "textureSource", this._resourceId = ne("resource"), this.uploadMethodId = "unknown", this._resolution = 1, this.pixelWidth = 1, this.pixelHeight = 1, this.width = 1, this.height = 1, this.sampleCount = 1, this.mipLevelCount = 1, this.autoGenerateMipmaps = false, this.format = "rgba8unorm", this.dimension = "2d", this.viewDimension = "2d", this.arrayLayerCount = 1, this.antialias = false, this._touched = 0, this._batchTick = -1, this._textureBindLocation = -1, t = {
        ...uh.defaultOptions,
        ...t
      }, this.label = t.label ?? "", this.resource = t.resource, this.autoGarbageCollect = t.autoGarbageCollect, this._resolution = t.resolution, t.width ? this.pixelWidth = t.width * this._resolution : this.pixelWidth = this.resource ? this.resourceWidth ?? 1 : 1, t.height ? this.pixelHeight = t.height * this._resolution : this.pixelHeight = this.resource ? this.resourceHeight ?? 1 : 1, this.width = this.pixelWidth / this._resolution, this.height = this.pixelHeight / this._resolution, this.format = t.format, this.dimension = t.dimensions, this.viewDimension = t.viewDimension ?? t.dimensions, this.arrayLayerCount = t.arrayLayerCount, this.mipLevelCount = t.mipLevelCount, this.autoGenerateMipmaps = t.autoGenerateMipmaps, this.sampleCount = t.sampleCount, this.antialias = t.antialias, this.alphaMode = t.alphaMode, this.style = new Zn(lh(t)), this.destroyed = false, this._refreshPOT();
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
      this.isPowerOfTwo = Va(this.pixelWidth) && Va(this.pixelHeight);
    }
    static test(t) {
      throw new Error("Unimplemented");
    }
  };
  dh.defaultOptions = {
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
  Re = dh;
  class ra extends Re {
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
  ra.extension = lt.TextureSource;
  const qa = new vt();
  sp = class {
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
      i && (qa.set(s.width / i.width, 0, 0, s.height / i.height, -i.x / i.width, -i.y / i.height), this.mapCoord.append(qa));
      const r = t.source, o = this.uClampFrame, a = this.clampMargin / r._resolution, l = this.clampOffset / r._resolution;
      return o[0] = (t.frame.x + a + l) / r.width, o[1] = (t.frame.y + a + l) / r.height, o[2] = (t.frame.x + t.frame.width - a + l) / r.width, o[3] = (t.frame.y + t.frame.height - a + l) / r.height, this.uClampOffset[0] = this.clampOffset / r.pixelWidth, this.uClampOffset[1] = this.clampOffset / r.pixelHeight, this.isSimple = t.frame.width === r.width && t.frame.height === r.height && t.rotate === 0, true;
    }
  };
  Tt = class extends on {
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
      }, this.frame = new Dt(), this.noFrame = false, this.dynamic = false, this.isTexture = true, this.label = e, this.source = (t == null ? void 0 : t.source) ?? new Re(), this.noFrame = !s, s) this.frame.copyFrom(s);
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
      return this._textureMatrix || (this._textureMatrix = new sp(this)), this._textureMatrix;
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
        c = At.add(c, At.NW), t.x0 = u + h * At.uX(c), t.y0 = p + d * At.uY(c), c = At.add(c, 2), t.x1 = u + h * At.uX(c), t.y1 = p + d * At.uY(c), c = At.add(c, 2), t.x2 = u + h * At.uX(c), t.y2 = p + d * At.uY(c), c = At.add(c, 2), t.x3 = u + h * At.uX(c), t.y3 = p + d * At.uY(c);
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
  Tt.EMPTY = new Tt({
    label: "EMPTY",
    source: new Re({
      label: "EMPTY"
    })
  });
  Tt.EMPTY.destroy = ah;
  Tt.WHITE = new Tt({
    source: new ra({
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
  Tt.WHITE.destroy = ah;
  function ph(n, t, e) {
    const { width: s, height: i } = e.orig, r = e.trim;
    if (r) {
      const o = r.width, a = r.height;
      n.minX = r.x - t._x * s, n.maxX = n.minX + o, n.minY = r.y - t._y * i, n.maxY = n.minY + a;
    } else n.minX = -t._x * s, n.maxX = n.minX + s, n.minY = -t._y * i, n.maxY = n.minY + i;
  }
  const Ka = new vt();
  Ie = class {
    constructor(t = 1 / 0, e = 1 / 0, s = -1 / 0, i = -1 / 0) {
      this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = Ka, this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
    }
    isEmpty() {
      return this.minX > this.maxX || this.minY > this.maxY;
    }
    get rectangle() {
      this._rectangle || (this._rectangle = new Dt());
      const t = this._rectangle;
      return this.minX > this.maxX || this.minY > this.maxY ? (t.x = 0, t.y = 0, t.width = 0, t.height = 0) : t.copyFromBounds(this), t;
    }
    clear() {
      return this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = Ka, this;
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
      return new Ie(this.minX, this.minY, this.maxX, this.maxY);
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
  var ip = {
    grad: 0.9,
    turn: 360,
    rad: 360 / (2 * Math.PI)
  }, an = function(n) {
    return typeof n == "string" ? n.length > 0 : typeof n == "number";
  }, ce = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = Math.pow(10, t)), Math.round(e * n) / e + 0;
  }, Be = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = 1), n > e ? e : n > t ? n : t;
  }, fh = function(n) {
    return (n = isFinite(n) ? n % 360 : 0) > 0 ? n : n + 360;
  }, Ja = function(n) {
    return {
      r: Be(n.r, 0, 255),
      g: Be(n.g, 0, 255),
      b: Be(n.b, 0, 255),
      a: Be(n.a)
    };
  }, Ir = function(n) {
    return {
      r: ce(n.r),
      g: ce(n.g),
      b: ce(n.b),
      a: ce(n.a, 3)
    };
  }, rp = /^#([0-9a-f]{3,8})$/i, Ti = function(n) {
    var t = n.toString(16);
    return t.length < 2 ? "0" + t : t;
  }, mh = function(n) {
    var t = n.r, e = n.g, s = n.b, i = n.a, r = Math.max(t, e, s), o = r - Math.min(t, e, s), a = o ? r === t ? (e - s) / o : r === e ? 2 + (s - t) / o : 4 + (t - e) / o : 0;
    return {
      h: 60 * (a < 0 ? a + 6 : a),
      s: r ? o / r * 100 : 0,
      v: r / 255 * 100,
      a: i
    };
  }, gh = function(n) {
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
  }, Za = function(n) {
    return {
      h: fh(n.h),
      s: Be(n.s, 0, 100),
      l: Be(n.l, 0, 100),
      a: Be(n.a)
    };
  }, Qa = function(n) {
    return {
      h: ce(n.h),
      s: ce(n.s),
      l: ce(n.l),
      a: ce(n.a, 3)
    };
  }, tl = function(n) {
    return gh((e = (t = n).s, {
      h: t.h,
      s: (e *= ((s = t.l) < 50 ? s : 100 - s) / 100) > 0 ? 2 * e / (s + e) * 100 : 0,
      v: s + e,
      a: t.a
    }));
    var t, e, s;
  }, qs = function(n) {
    return {
      h: (t = mh(n)).h,
      s: (i = (200 - (e = t.s)) * (s = t.v) / 100) > 0 && i < 200 ? e * s / 100 / (i <= 100 ? i : 200 - i) * 100 : 0,
      l: i / 2,
      a: t.a
    };
    var t, e, s, i;
  }, op = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s*,\s*([+-]?\d*\.?\d+)%\s*,\s*([+-]?\d*\.?\d+)%\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, ap = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s+([+-]?\d*\.?\d+)%\s+([+-]?\d*\.?\d+)%\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, lp = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, cp = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, vo = {
    string: [
      [
        function(n) {
          var t = rp.exec(n);
          return t ? (n = t[1]).length <= 4 ? {
            r: parseInt(n[0] + n[0], 16),
            g: parseInt(n[1] + n[1], 16),
            b: parseInt(n[2] + n[2], 16),
            a: n.length === 4 ? ce(parseInt(n[3] + n[3], 16) / 255, 2) : 1
          } : n.length === 6 || n.length === 8 ? {
            r: parseInt(n.substr(0, 2), 16),
            g: parseInt(n.substr(2, 2), 16),
            b: parseInt(n.substr(4, 2), 16),
            a: n.length === 8 ? ce(parseInt(n.substr(6, 2), 16) / 255, 2) : 1
          } : null : null;
        },
        "hex"
      ],
      [
        function(n) {
          var t = lp.exec(n) || cp.exec(n);
          return t ? t[2] !== t[4] || t[4] !== t[6] ? null : Ja({
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
          var t = op.exec(n) || ap.exec(n);
          if (!t) return null;
          var e, s, i = Za({
            h: (e = t[1], s = t[2], s === void 0 && (s = "deg"), Number(e) * (ip[s] || 1)),
            s: Number(t[3]),
            l: Number(t[4]),
            a: t[5] === void 0 ? 1 : Number(t[5]) / (t[6] ? 100 : 1)
          });
          return tl(i);
        },
        "hsl"
      ]
    ],
    object: [
      [
        function(n) {
          var t = n.r, e = n.g, s = n.b, i = n.a, r = i === void 0 ? 1 : i;
          return an(t) && an(e) && an(s) ? Ja({
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
          if (!an(t) || !an(e) || !an(s)) return null;
          var o = Za({
            h: Number(t),
            s: Number(e),
            l: Number(s),
            a: Number(r)
          });
          return tl(o);
        },
        "hsl"
      ],
      [
        function(n) {
          var t = n.h, e = n.s, s = n.v, i = n.a, r = i === void 0 ? 1 : i;
          if (!an(t) || !an(e) || !an(s)) return null;
          var o = (function(a) {
            return {
              h: fh(a.h),
              s: Be(a.s, 0, 100),
              v: Be(a.v, 0, 100),
              a: Be(a.a)
            };
          })({
            h: Number(t),
            s: Number(e),
            v: Number(s),
            a: Number(r)
          });
          return gh(o);
        },
        "hsv"
      ]
    ]
  }, el = function(n, t) {
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
  }, hp = function(n) {
    return typeof n == "string" ? el(n.trim(), vo.string) : typeof n == "object" && n !== null ? el(n, vo.object) : [
      null,
      void 0
    ];
  }, Rr = function(n, t) {
    var e = qs(n);
    return {
      h: e.h,
      s: Be(e.s + 100 * t, 0, 100),
      l: e.l,
      a: e.a
    };
  }, Lr = function(n) {
    return (299 * n.r + 587 * n.g + 114 * n.b) / 1e3 / 255;
  }, nl = function(n, t) {
    var e = qs(n);
    return {
      h: e.h,
      s: e.s,
      l: Be(e.l + 100 * t, 0, 100),
      a: e.a
    };
  }, Co = (function() {
    function n(t) {
      this.parsed = hp(t)[0], this.rgba = this.parsed || {
        r: 0,
        g: 0,
        b: 0,
        a: 1
      };
    }
    return n.prototype.isValid = function() {
      return this.parsed !== null;
    }, n.prototype.brightness = function() {
      return ce(Lr(this.rgba), 2);
    }, n.prototype.isDark = function() {
      return Lr(this.rgba) < 0.5;
    }, n.prototype.isLight = function() {
      return Lr(this.rgba) >= 0.5;
    }, n.prototype.toHex = function() {
      return t = Ir(this.rgba), e = t.r, s = t.g, i = t.b, o = (r = t.a) < 1 ? Ti(ce(255 * r)) : "", "#" + Ti(e) + Ti(s) + Ti(i) + o;
      var t, e, s, i, r, o;
    }, n.prototype.toRgb = function() {
      return Ir(this.rgba);
    }, n.prototype.toRgbString = function() {
      return t = Ir(this.rgba), e = t.r, s = t.g, i = t.b, (r = t.a) < 1 ? "rgba(" + e + ", " + s + ", " + i + ", " + r + ")" : "rgb(" + e + ", " + s + ", " + i + ")";
      var t, e, s, i, r;
    }, n.prototype.toHsl = function() {
      return Qa(qs(this.rgba));
    }, n.prototype.toHslString = function() {
      return t = Qa(qs(this.rgba)), e = t.h, s = t.s, i = t.l, (r = t.a) < 1 ? "hsla(" + e + ", " + s + "%, " + i + "%, " + r + ")" : "hsl(" + e + ", " + s + "%, " + i + "%)";
      var t, e, s, i, r;
    }, n.prototype.toHsv = function() {
      return t = mh(this.rgba), {
        h: ce(t.h),
        s: ce(t.s),
        v: ce(t.v),
        a: ce(t.a, 3)
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
      return t === void 0 && (t = 0.1), Ze(Rr(this.rgba, t));
    }, n.prototype.desaturate = function(t) {
      return t === void 0 && (t = 0.1), Ze(Rr(this.rgba, -t));
    }, n.prototype.grayscale = function() {
      return Ze(Rr(this.rgba, -1));
    }, n.prototype.lighten = function(t) {
      return t === void 0 && (t = 0.1), Ze(nl(this.rgba, t));
    }, n.prototype.darken = function(t) {
      return t === void 0 && (t = 0.1), Ze(nl(this.rgba, -t));
    }, n.prototype.rotate = function(t) {
      return t === void 0 && (t = 15), this.hue(this.hue() + t);
    }, n.prototype.alpha = function(t) {
      return typeof t == "number" ? Ze({
        r: (e = this.rgba).r,
        g: e.g,
        b: e.b,
        a: t
      }) : ce(this.rgba.a, 3);
      var e;
    }, n.prototype.hue = function(t) {
      var e = qs(this.rgba);
      return typeof t == "number" ? Ze({
        h: t,
        s: e.s,
        l: e.l,
        a: e.a
      }) : ce(e.h);
    }, n.prototype.isEqual = function(t) {
      return this.toHex() === Ze(t).toHex();
    }, n;
  })(), Ze = function(n) {
    return n instanceof Co ? n : new Co(n);
  }, sl = [], dp = function(n) {
    n.forEach(function(t) {
      sl.indexOf(t) < 0 && (t(Co, vo), sl.push(t));
    });
  };
  function up(n, t) {
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
  dp([
    up
  ]);
  const Cs = class js {
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
      if (t instanceof js) this._value = this._cloneSource(t._value), this._int = t._int, this._components.set(t._components);
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
      const [e, s, i, r] = js._temp.setValue(t)._components;
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
          const a = js.HEX_PATTERN.exec(t);
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
      return typeof t == "number" || typeof t == "string" || t instanceof Number || t instanceof js || Array.isArray(t) || t instanceof Uint8Array || t instanceof Uint8ClampedArray || t instanceof Float32Array || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 && t.a !== void 0;
    }
  };
  Cs.shared = new Cs();
  Cs._temp = new Cs();
  Cs.HEX_PATTERN = /^(#|0x)?(([a-f0-9]{3}){1,2}([a-f0-9]{2})?)$/i;
  Yt = Cs;
  const pp = {
    cullArea: null,
    cullable: false,
    cullableChildren: true
  };
  let $r = 0;
  const il = 500;
  Jt = function(...n) {
    $r !== il && ($r++, $r === il ? console.warn("PixiJS Warning: too many warnings, no more warnings will be reported to the console by PixiJS.") : console.warn("PixiJS Warning: ", ...n));
  };
  mi = {
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
  class fp {
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
  class mp {
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
      return this._poolsByClass.has(t) || this._poolsByClass.set(t, new fp(t)), this._poolsByClass.get(t);
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
  Te = new mp();
  mi.register(Te);
  const gp = {
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
  yp = function(n, t, e) {
    const s = n.length;
    let i;
    if (t >= s || e === 0) return;
    e = t + e > s ? s - t : e;
    const r = s - e;
    for (i = t; i < r; ++i) n[i] = n[i + e];
    n.length = r;
  };
  const xp = {
    allowChildren: true,
    removeChildren(n = 0, t) {
      var _a2;
      const e = t ?? this.children.length, s = e - n, i = [];
      if (s > 0 && s <= e) {
        for (let o = e - 1; o >= n; o--) {
          const a = this.children[o];
          a && (i.push(a), a.parent = null);
        }
        yp(this.children, n, e);
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
  }, bp = {
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
  rl = class {
    constructor() {
      this.pipe = "filter", this.priority = 1;
    }
    destroy() {
      for (let t = 0; t < this.filters.length; t++) this.filters[t].destroy();
      this.filters = null, this.filterArea = null;
    }
  };
  class _p {
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
        if (s.test(t)) return Te.get(s.maskClass, t);
      }
      return t;
    }
    returnMaskEffect(t) {
      Te.return(t);
    }
  }
  const So = new _p();
  Wt.handleByList(lt.MaskEffect, So._effectClasses);
  const wp = {
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
      (t == null ? void 0 : t.mask) !== n && (t && (this.removeEffect(t), So.returnMaskEffect(t), this._maskEffect = null), n != null && (this._maskEffect = So.getMaskEffect(n), this.addEffect(this._maskEffect)));
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
      const t = this._filterEffect || (this._filterEffect = new rl());
      n = n;
      const e = (n == null ? void 0 : n.length) > 0, s = ((_a2 = t.filters) == null ? void 0 : _a2.length) > 0, i = e !== s;
      n = Array.isArray(n) ? n.slice(0) : n, t.filters = Object.freeze(n), i && (e ? this.addEffect(t) : (this.removeEffect(t), t.filters = n ?? null));
    },
    get filters() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filters;
    },
    set filterArea(n) {
      this._filterEffect || (this._filterEffect = new rl()), this._filterEffect.filterArea = n;
    },
    get filterArea() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filterArea;
    }
  }, vp = {
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
  }, _e = Te.getPool(vt), yn = Te.getPool(Ie), Cp = new vt(), Sp = {
    getFastGlobalBounds(n, t) {
      t || (t = new Ie()), t.clear(), this._getGlobalBoundsRecursive(!!n, t, this.parentRenderLayer), t.isValid || t.set(0, 0, 0, 0);
      const e = this.renderGroup || this.parentRenderGroup;
      return t.applyMatrix(e.worldTransform), t;
    },
    _getGlobalBoundsRecursive(n, t, e) {
      let s = t;
      if (n && this.parentRenderLayer && this.parentRenderLayer !== e || this.localDisplayStatus !== 7 || !this.measurable) return;
      const i = !!this.effects.length;
      if ((this.renderGroup || i) && (s = yn.get().clear()), this.boundsArea) t.addRect(this.boundsArea, this.worldTransform);
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
        r && s.applyMatrix(o.worldTransform.copyTo(Cp).invert()), t.addBounds(s), yn.return(s);
      } else this.renderGroup && (t.addBounds(s, this.relativeGroupTransform), yn.return(s));
    }
  };
  yh = function(n, t, e) {
    e.clear();
    let s, i;
    return n.parent ? t ? s = n.parent.worldTransform : (i = _e.get().identity(), s = oa(n, i)) : s = vt.IDENTITY, xh(n, e, s, t), i && _e.return(i), e.isValid || e.set(0, 0, 0, 0), e;
  };
  function xh(n, t, e, s) {
    var _a2, _b2;
    if (!n.visible || !n.measurable) return;
    let i;
    s ? i = n.worldTransform : (n.updateLocalTransform(), i = _e.get(), i.appendFrom(n.localTransform, e));
    const r = t, o = !!n.effects.length;
    if (o && (t = yn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, i);
    else {
      const a = n.bounds;
      a && !a.isEmpty() && (t.matrix = i, t.addBounds(a));
      for (let l = 0; l < n.children.length; l++) xh(n.children[l], t, i, s);
    }
    if (o) {
      for (let a = 0; a < n.effects.length; a++) (_b2 = (_a2 = n.effects[a]).addBounds) == null ? void 0 : _b2.call(_a2, t);
      r.addBounds(t, vt.IDENTITY), yn.return(t);
    }
    s || _e.return(i);
  }
  function oa(n, t) {
    const e = n.parent;
    return e && (oa(e, t), e.updateLocalTransform(), t.append(e.localTransform)), t;
  }
  bh = function(n, t) {
    if (n === 16777215 || !t) return t;
    if (t === 16777215 || !n) return n;
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = t >> 16 & 255, o = t >> 8 & 255, a = t & 255, l = e * r / 255 | 0, c = s * o / 255 | 0, h = i * a / 255 | 0;
    return (l << 16) + (c << 8) + h;
  };
  const ol = 16777215;
  al = function(n, t) {
    return n === ol ? t : t === ol ? n : bh(n, t);
  };
  Ks = function(n) {
    return ((n & 255) << 16) + (n & 65280) + (n >> 16 & 255);
  };
  const Tp = {
    getGlobalAlpha(n) {
      if (n) return this.renderGroup ? this.renderGroup.worldAlpha : this.parentRenderGroup ? this.parentRenderGroup.worldAlpha * this.alpha : this.alpha;
      let t = this.alpha, e = this.parent;
      for (; e; ) t *= e.alpha, e = e.parent;
      return t;
    },
    getGlobalTransform(n = new vt(), t) {
      if (t) return n.copyFrom(this.worldTransform);
      this.updateLocalTransform();
      const e = oa(this, _e.get().identity());
      return n.appendFrom(this.localTransform, e), _e.return(e), n;
    },
    getGlobalTint(n) {
      if (n) return this.renderGroup ? Ks(this.renderGroup.worldColor) : this.parentRenderGroup ? Ks(al(this.localColor, this.parentRenderGroup.worldColor)) : this.tint;
      let t = this.localColor, e = this.parent;
      for (; e; ) t = al(t, e.localColor), e = e.parent;
      return Ks(t);
    }
  };
  _h = function(n, t, e) {
    return t.clear(), e || (e = vt.IDENTITY), wh(n, t, e, n, true), t.isValid || t.set(0, 0, 0, 0), t;
  };
  function wh(n, t, e, s, i) {
    var _a2, _b2;
    let r;
    if (i) r = _e.get(), r = e.copyTo(r);
    else {
      if (!n.visible || !n.measurable) return;
      n.updateLocalTransform();
      const l = n.localTransform;
      r = _e.get(), r.appendFrom(l, e);
    }
    const o = t, a = !!n.effects.length;
    if (a && (t = yn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, r);
    else {
      n.renderPipeId && (t.matrix = r, t.addBounds(n.bounds));
      const l = n.children;
      for (let c = 0; c < l.length; c++) wh(l[c], t, r, s, false);
    }
    if (a) {
      for (let l = 0; l < n.effects.length; l++) (_b2 = (_a2 = n.effects[l]).addLocalBounds) == null ? void 0 : _b2.call(_a2, t, s);
      o.addBounds(t, vt.IDENTITY), yn.return(t);
    }
    _e.return(r);
  }
  function vh(n, t) {
    const e = n.children;
    for (let s = 0; s < e.length; s++) {
      const i = e[s], r = i.uid, o = (i._didViewChangeTick & 65535) << 16 | i._didContainerChangeTick & 65535, a = t.index;
      (t.data[a] !== r || t.data[a + 1] !== o) && (t.data[t.index] = r, t.data[t.index + 1] = o, t.didChange = true), t.index = a + 2, i.children.length && vh(i, t);
    }
    return t.didChange;
  }
  const Ep = new vt(), Ap = {
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
        localBounds: new Ie()
      });
      const n = this._localBoundsCacheData;
      return n.index = 1, n.didChange = false, n.data[0] !== this._didViewChangeTick && (n.didChange = true, n.data[0] = this._didViewChangeTick), vh(this, n), n.didChange && _h(this, n.localBounds, Ep), n.localBounds;
    },
    getBounds(n, t) {
      return yh(this, n, t || new Ie());
    }
  }, kp = {
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
  }, Mp = {
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
      this.sortDirty && (this.sortDirty = false, this.children.sort(Pp));
    }
  };
  function Pp(n, t) {
    return n._zIndex - t._zIndex;
  }
  const Ip = {
    getGlobalPosition(n = new Mt(), t = false) {
      return this.parent ? this.parent.toGlobal(this._position, n, t) : (n.x = this._position.x, n.y = this._position.y), n;
    },
    toGlobal(n, t, e = false) {
      const s = this.getGlobalTransform(_e.get(), e);
      return t = s.apply(n, t), _e.return(s), t;
    },
    toLocal(n, t, e, s) {
      t && (n = t.toGlobal(n, e, s));
      const i = this.getGlobalTransform(_e.get(), s);
      return e = i.applyInverse(n, e), _e.return(i), e;
    }
  };
  class aa {
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
  let Rp = 0;
  class Lp {
    constructor(t) {
      this._poolKeyHash = /* @__PURE__ */ Object.create(null), this._texturePool = {}, this.textureOptions = t || {}, this.enableFullScreen = false, this.textureStyle = new Zn(this.textureOptions);
    }
    createTexture(t, e, s, i) {
      const r = new Re({
        ...this.textureOptions,
        width: t,
        height: e,
        resolution: 1,
        antialias: s,
        autoGarbageCollect: false,
        autoGenerateMipmaps: i
      });
      return new Tt({
        source: r,
        label: `texturePool_${Rp++}`
      });
    }
    getOptimalTexture(t, e, s = 1, i, r = false) {
      let o = Math.ceil(t * s - 1e-6), a = Math.ceil(e * s - 1e-6);
      o = vs(o), a = vs(a);
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
  pr = new Lp();
  mi.register(pr);
  $p = class {
    constructor() {
      this.renderPipeId = "renderGroup", this.root = null, this.canBundle = false, this.renderGroupParent = null, this.renderGroupChildren = [], this.worldTransform = new vt(), this.worldColorAlpha = 4294967295, this.worldColor = 16777215, this.worldAlpha = 1, this.childrenToUpdate = /* @__PURE__ */ Object.create(null), this.updateTick = 0, this.gcTick = 0, this.childrenRenderablesToUpdate = {
        list: [],
        index: 0
      }, this.structureDidChange = true, this.instructionSet = new aa(), this._onRenderContainers = [], this.textureNeedsUpdate = true, this.isCachedAsTexture = false, this._matrixDirty = 7;
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
      this.isCachedAsTexture = false, this.texture && (pr.returnTexture(this.texture, true), this.texture = null);
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
  function To(n, t, e = {}) {
    for (const s in t) !e[s] && t[s] !== void 0 && (n[s] = t[s]);
  }
  let Br, Ei, Or, Ai;
  Br = new fe(null);
  Ei = new fe(null);
  Or = new fe(null, 1, 1);
  Ai = new fe(null);
  ll = 1;
  Bp = 2;
  Nr = 4;
  zt = class extends on {
    constructor(t = {}) {
      var _a2, _b2;
      super(), this.uid = ne("renderable"), this._updateFlags = 15, this.renderGroup = null, this.parentRenderGroup = null, this.parentRenderGroupIndex = 0, this.didChange = false, this.didViewUpdate = false, this.relativeRenderGroupDepth = 0, this.children = [], this.parent = null, this.includeInBuild = true, this.measurable = true, this.isSimple = true, this.parentRenderLayer = null, this.updateTick = -1, this.localTransform = new vt(), this.relativeGroupTransform = new vt(), this.groupTransform = this.relativeGroupTransform, this.destroyed = false, this._position = new fe(this, 0, 0), this._scale = Or, this._pivot = Ei, this._origin = Ai, this._skew = Br, this._cx = 1, this._sx = 0, this._cy = 0, this._sy = 1, this._rotation = 0, this.localColor = 16777215, this.localAlpha = 1, this.groupAlpha = 1, this.groupColor = 16777215, this.groupColorAlpha = 4294967295, this.localBlendMode = "inherit", this.groupBlendMode = "normal", this.localDisplayStatus = 7, this.globalDisplayStatus = 7, this._didContainerChangeTick = 0, this._didViewChangeTick = 0, this._didLocalTransformChangeId = -1, this.effects = [], To(this, t, {
        children: true,
        parent: true,
        effects: true
      }), (_a2 = t.children) == null ? void 0 : _a2.forEach((e) => this.addChild(e)), (_b2 = t.parent) == null ? void 0 : _b2.addChild(this);
    }
    static mixin(t) {
      It("8.8.0", "Container.mixin is deprecated, please use extensions.mixin instead."), Wt.mixin(zt, t);
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
      t == null ? void 0 : t.removeChild(this), this.renderGroup = Te.get($p, this), this.groupTransform = vt.IDENTITY, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
    }
    disableRenderGroup() {
      if (!this.renderGroup) return;
      const t = this.parentRenderGroup;
      t == null ? void 0 : t.removeChild(this), Te.return(this.renderGroup), this.renderGroup = null, this.groupTransform = this.relativeGroupTransform, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
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
      return this.rotation * Ku;
    }
    set angle(t) {
      this.rotation = t * Ju;
    }
    get pivot() {
      return this._pivot === Ei && (this._pivot = new fe(this, 0, 0)), this._pivot;
    }
    set pivot(t) {
      this._pivot === Ei && (this._pivot = new fe(this, 0, 0), this._origin !== Ai && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._pivot.set(t) : this._pivot.copyFrom(t);
    }
    get skew() {
      return this._skew === Br && (this._skew = new fe(this, 0, 0)), this._skew;
    }
    set skew(t) {
      this._skew === Br && (this._skew = new fe(this, 0, 0)), this._skew.copyFrom(t);
    }
    get scale() {
      return this._scale === Or && (this._scale = new fe(this, 1, 1)), this._scale;
    }
    set scale(t) {
      this._scale === Or && (this._scale = new fe(this, 0, 0)), typeof t == "string" && (t = parseFloat(t)), typeof t == "number" ? this._scale.set(t) : this._scale.copyFrom(t);
    }
    get origin() {
      return this._origin === Ai && (this._origin = new fe(this, 0, 0)), this._origin;
    }
    set origin(t) {
      this._origin === Ai && (this._origin = new fe(this, 0, 0), this._pivot !== Ei && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._origin.set(t) : this._origin.copyFrom(t);
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
      t !== this.localAlpha && (this.localAlpha = t, this._updateFlags |= ll, this._onUpdate());
    }
    get alpha() {
      return this.localAlpha;
    }
    set tint(t) {
      const s = Yt.shared.setValue(t ?? 16777215).toBgrNumber();
      s !== this.localColor && (this.localColor = s, this._updateFlags |= ll, this._onUpdate());
    }
    get tint() {
      return Ks(this.localColor);
    }
    set blendMode(t) {
      this.localBlendMode !== t && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Bp, this.localBlendMode = t, this._onUpdate());
    }
    get blendMode() {
      return this.localBlendMode;
    }
    get visible() {
      return !!(this.localDisplayStatus & 2);
    }
    set visible(t) {
      const e = t ? 2 : 0;
      (this.localDisplayStatus & 2) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Nr, this.localDisplayStatus ^= 2, this._onUpdate(), this.emit("visibleChanged", t));
    }
    get culled() {
      return !(this.localDisplayStatus & 4);
    }
    set culled(t) {
      const e = t ? 0 : 4;
      (this.localDisplayStatus & 4) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Nr, this.localDisplayStatus ^= 4, this._onUpdate());
    }
    get renderable() {
      return !!(this.localDisplayStatus & 1);
    }
    set renderable(t) {
      const e = t ? 1 : 0;
      (this.localDisplayStatus & 1) !== e && (this._updateFlags |= Nr, this.localDisplayStatus ^= 1, this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._onUpdate());
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
  Wt.mixin(zt, xp, Sp, Ip, kp, Ap, wp, vp, Mp, pp, gp, Tp, bp);
  class fr extends zt {
    constructor(t) {
      super(t), this.canBundle = true, this.allowChildren = false, this._roundPixels = 0, this._lastUsed = -1, this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this._bounds = new Ie(0, 1, 0, 0), this._boundsDirty = true, this.autoGarbageCollect = t.autoGarbageCollect ?? true;
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
  We = class extends fr {
    constructor(t = Tt.EMPTY) {
      t instanceof Tt && (t = {
        texture: t
      });
      const { texture: e = Tt.EMPTY, anchor: s, roundPixels: i, width: r, height: o, ...a } = t;
      super({
        label: "Sprite",
        ...a
      }), this.renderPipeId = "sprite", this.batched = true, this._visualBounds = {
        minX: 0,
        maxX: 1,
        minY: 0,
        maxY: 0
      }, this._anchor = new fe({
        _onUpdate: () => {
          this.onViewUpdate();
        }
      }), s ? this.anchor = s : e.defaultAnchor && (this.anchor = e.defaultAnchor), this.texture = e, this.allowChildren = false, this.roundPixels = i ?? false, r !== void 0 && (this.width = r), o !== void 0 && (this.height = o);
    }
    static from(t, e = false) {
      return t instanceof Tt ? new We(t) : new We(Tt.from(t, e));
    }
    set texture(t) {
      t || (t = Tt.EMPTY);
      const e = this._texture;
      e !== t && (e && e.dynamic && e.off("update", this.onViewUpdate, this), t.dynamic && t.on("update", this.onViewUpdate, this), this._texture = t, this._width && this._setWidth(this._width, this._texture.orig.width), this._height && this._setHeight(this._height, this._texture.orig.height), this.onViewUpdate());
    }
    get texture() {
      return this._texture;
    }
    get visualBounds() {
      return ph(this._visualBounds, this._anchor, this._texture), this._visualBounds;
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
  const Op = new Ie();
  function Ch(n, t, e) {
    const s = Op;
    n.measurable = true, yh(n, e, s), t.addBoundsMask(s), n.measurable = false;
  }
  function Sh(n, t, e) {
    const s = yn.get();
    n.measurable = true;
    const i = _e.get().identity(), r = Th(n, e, i);
    _h(n, s, r), n.measurable = false, t.addBoundsMask(s), _e.return(i), yn.return(s);
  }
  function Th(n, t, e) {
    return n ? (n !== t && (Th(n.parent, t, e), n.updateLocalTransform(), e.append(n.localTransform)), e) : (Jt("Mask bounds, renderable is not inside the root container"), e);
  }
  class Eh {
    constructor(t) {
      this.priority = 0, this.inverse = false, this.pipe = "alphaMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t, this.renderMaskToTexture = !(t instanceof We), this.mask.renderable = this.renderMaskToTexture, this.mask.includeInBuild = !this.renderMaskToTexture, this.mask.measurable = false;
    }
    reset() {
      this.mask !== null && (this.mask.measurable = true, this.mask = null);
    }
    addBounds(t, e) {
      this.inverse || Ch(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      Sh(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof We;
    }
  }
  Eh.extension = lt.MaskEffect;
  class Ah {
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
  Ah.extension = lt.MaskEffect;
  class kh {
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
      Ch(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      Sh(this.mask, t, e);
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
  kh.extension = lt.MaskEffect;
  const Np = {
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
  let cl = Np;
  Rt = {
    get() {
      return cl;
    },
    set(n) {
      cl = n;
    }
  };
  Mh = class extends Re {
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
  Mh.extension = lt.TextureSource;
  Ss = class extends Re {
    constructor(t) {
      super(t), this.uploadMethodId = "image", this.autoGarbageCollect = true;
    }
    static test(t) {
      return globalThis.HTMLImageElement && t instanceof HTMLImageElement || typeof ImageBitmap < "u" && t instanceof ImageBitmap || globalThis.VideoFrame && t instanceof VideoFrame;
    }
  };
  Ss.extension = lt.TextureSource;
  ii = ((n) => (n[n.INTERACTION = 50] = "INTERACTION", n[n.HIGH = 25] = "HIGH", n[n.NORMAL = 0] = "NORMAL", n[n.LOW = -25] = "LOW", n[n.UTILITY = -50] = "UTILITY", n))(ii || {});
  class Fr {
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
  const Ph = class Ae {
    constructor() {
      this.autoStart = false, this.deltaTime = 1, this.lastTime = -1, this.speed = 1, this.started = false, this._requestId = null, this._maxElapsedMS = 100, this._minElapsedMS = 0, this._protected = false, this._lastFrame = -1, this._head = new Fr(null, null, 1 / 0), this.deltaMS = 1 / Ae.targetFPMS, this.elapsedMS = 1 / Ae.targetFPMS, this._tick = (t) => {
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
    add(t, e, s = ii.NORMAL) {
      return this._addListener(new Fr(t, e, s));
    }
    addOnce(t, e, s = ii.NORMAL) {
      return this._addListener(new Fr(t, e, s, true));
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
        this.deltaMS = e, this.deltaTime = this.deltaMS * Ae.targetFPMS;
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
      const e = Math.min(Math.max(0, t) / 1e3, Ae.targetFPMS);
      this._maxElapsedMS = 1 / e, this._minElapsedMS && t > this.maxFPS && (this.maxFPS = t);
    }
    get maxFPS() {
      return this._minElapsedMS ? Math.round(1e3 / this._minElapsedMS) : 0;
    }
    set maxFPS(t) {
      t === 0 ? this._minElapsedMS = 0 : (t < this.minFPS && (this.minFPS = t), this._minElapsedMS = 1 / (t / 1e3));
    }
    static get shared() {
      if (!Ae._shared) {
        const t = Ae._shared = new Ae();
        t.autoStart = true, t._protected = true;
      }
      return Ae._shared;
    }
    static get system() {
      if (!Ae._system) {
        const t = Ae._system = new Ae();
        t.autoStart = true, t._protected = true;
      }
      return Ae._system;
    }
  };
  Ph.targetFPMS = 0.06;
  let Wr;
  qn = Ph;
  async function Ih() {
    return Wr ?? (Wr = (async () => {
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
    })()), Wr;
  }
  const mr = class Rh extends Re {
    constructor(t) {
      super(t), this.isReady = false, this.uploadMethodId = "video", t = {
        ...Rh.defaultOptions,
        ...t
      }, this._autoUpdate = true, this._isConnectedToTicker = false, this._updateFPS = t.updateFPS || 0, this._msToNextUpdate = 0, this.autoPlay = t.autoPlay !== false, this.alphaMode = t.alphaMode ?? "premultiply-alpha-on-upload", this._videoFrameRequestCallback = this._videoFrameRequestCallback.bind(this), this._videoFrameRequestCallbackHandle = null, this._load = null, this._resolve = null, this._reject = null, this._onCanPlay = this._onCanPlay.bind(this), this._onCanPlayThrough = this._onCanPlayThrough.bind(this), this._onError = this._onError.bind(this), this._onPlayStart = this._onPlayStart.bind(this), this._onPlayStop = this._onPlayStop.bind(this), this._onSeeked = this._onSeeked.bind(this), t.autoLoad !== false && this.load();
    }
    updateFrame() {
      if (!this.destroyed) {
        if (this._updateFPS) {
          const t = qn.shared.elapsedMS * this.resource.playbackRate;
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
      return (t.readyState === t.HAVE_ENOUGH_DATA || t.readyState === t.HAVE_FUTURE_DATA) && t.width && t.height && (t.complete = true), t.addEventListener("play", this._onPlayStart), t.addEventListener("pause", this._onPlayStop), t.addEventListener("seeked", this._onSeeked), this._isSourceReady() ? this._mediaReady() : (e.preload || t.addEventListener("canplay", this._onCanPlay), t.addEventListener("canplaythrough", this._onCanPlayThrough), t.addEventListener("error", this._onError, true)), this.alphaMode = await Ih(), this._load = new Promise((s, i) => {
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
      this._autoUpdate && this._isSourcePlaying() ? !this._updateFPS && this.resource.requestVideoFrameCallback ? (this._isConnectedToTicker && (qn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0), this._videoFrameRequestCallbackHandle === null && (this._videoFrameRequestCallbackHandle = this.resource.requestVideoFrameCallback(this._videoFrameRequestCallback))) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker || (qn.shared.add(this.updateFrame, this), this._isConnectedToTicker = true, this._msToNextUpdate = 0)) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker && (qn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0));
    }
    static test(t) {
      return globalThis.HTMLVideoElement && t instanceof HTMLVideoElement;
    }
  };
  mr.extension = lt.TextureSource;
  mr.defaultOptions = {
    ...Re.defaultOptions,
    autoLoad: true,
    autoPlay: true,
    updateFPS: 0,
    crossorigin: true,
    loop: false,
    muted: true,
    playsinline: true,
    preload: false
  };
  mr.MIME_TYPES = {
    ogv: "video/ogg",
    mov: "video/quicktime",
    m4v: "video/mp4"
  };
  let Js = mr;
  const Ye = (n, t, e = false) => (Array.isArray(n) || (n = [
    n
  ]), t ? n.map((s) => typeof s == "string" || e ? t(s) : s) : n);
  class Fp {
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
      const s = Ye(t);
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
  let Eo;
  ie = new Fp();
  Eo = [];
  Wt.handleByList(lt.TextureSource, Eo);
  function Lh(n = {}) {
    const t = n && n.resource, e = t ? n.resource : n, s = t ? n : {
      resource: n
    };
    for (let i = 0; i < Eo.length; i++) {
      const r = Eo[i];
      if (r.test(e)) return new r(s);
    }
    throw new Error(`Could not find a source type for resource: ${s.resource}`);
  }
  function Wp(n = {}, t = false) {
    const e = n && n.resource, s = e ? n.resource : n, i = e ? n : {
      resource: n
    };
    if (!t && ie.has(s)) return ie.get(s);
    const r = new Tt({
      source: Lh(i)
    });
    return r.on("destroy", () => {
      ie.has(s) && ie.remove(s);
    }), t || ie.set(s, r), r;
  }
  function Gp(n, t = false) {
    return typeof n == "string" ? ie.get(n) : n instanceof Re ? new Tt({
      source: n
    }) : Wp(n, t);
  }
  Tt.from = Gp;
  Re.from = Lh;
  Wt.add(Eh, Ah, kh, Js, Ss, Mh, ra);
  var Bn = ((n) => (n[n.Low = 0] = "Low", n[n.Normal = 1] = "Normal", n[n.High = 2] = "High", n))(Bn || {});
  function He(n) {
    if (typeof n != "string") throw new TypeError(`Path must be a string. Received ${JSON.stringify(n)}`);
  }
  function Bs(n) {
    return n.split("?")[0].split("#")[0];
  }
  function zp(n) {
    return n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  function Dp(n, t, e) {
    return n.replace(new RegExp(zp(t), "g"), e);
  }
  function Hp(n, t) {
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
  const Pe = {
    toPosix(n) {
      return Dp(n, "\\", "/");
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
      He(n), n = this.toPosix(n);
      const t = /^file:\/\/\//.exec(n);
      if (t) return t[0];
      const e = /^[^/:]+:\/{0,2}/.exec(n);
      return e ? e[0] : "";
    },
    toAbsolute(n, t, e) {
      if (He(n), this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      const s = Bs(this.toPosix(t ?? Rt.get().getBaseUrl())), i = Bs(this.toPosix(e ?? this.rootname(s)));
      return n = this.toPosix(n), n.startsWith("/") ? Pe.join(i, n.slice(1)) : this.isAbsolute(n) ? n : this.join(s, n);
    },
    normalize(n) {
      if (He(n), n.length === 0) return ".";
      if (this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      n = this.toPosix(n);
      let t = "";
      const e = n.startsWith("/");
      this.hasProtocol(n) && (t = this.rootname(n), n = n.slice(t.length));
      const s = n.endsWith("/");
      return n = Hp(n), n.length > 0 && s && (n += "/"), e ? `/${n}` : t + n;
    },
    isAbsolute(n) {
      return He(n), n = this.toPosix(n), this.hasProtocol(n) ? true : n.startsWith("/");
    },
    join(...n) {
      if (n.length === 0) return ".";
      let t;
      for (let e = 0; e < n.length; ++e) {
        const s = n[e];
        if (He(s), s.length > 0) if (t === void 0) t = s;
        else {
          const i = n[e - 1] ?? "";
          this.joinExtensions.includes(this.extname(i).toLowerCase()) ? t += `/../${s}` : t += `/${s}`;
        }
      }
      return t === void 0 ? "." : this.normalize(t);
    },
    dirname(n) {
      if (He(n), n.length === 0) return ".";
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
      He(n), n = this.toPosix(n);
      let t = "";
      if (n.startsWith("/") ? t = "/" : t = this.getProtocol(n), this.isUrl(n)) {
        const e = n.indexOf("/", t.length);
        e !== -1 ? t = n.slice(0, e) : t = n, t.endsWith("/") || (t += "/");
      }
      return t;
    },
    basename(n, t) {
      He(n), t && He(t), n = Bs(this.toPosix(n));
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
      He(n), n = Bs(this.toPosix(n));
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
      He(n);
      const t = {
        root: "",
        dir: "",
        base: "",
        ext: "",
        name: ""
      };
      if (n.length === 0) return t;
      n = Bs(this.toPosix(n));
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
  function $h(n, t, e, s, i) {
    const r = t[e];
    for (let o = 0; o < r.length; o++) {
      const a = r[o];
      e < t.length - 1 ? $h(n.replace(s[e], a), t, e + 1, s, i) : i.push(n.replace(s[e], a));
    }
  }
  function Up(n) {
    const t = /\{(.*?)\}/g, e = n.match(t), s = [];
    if (e) {
      const i = [];
      e.forEach((r) => {
        const o = r.substring(1, r.length - 1).split(",");
        i.push(o);
      }), $h(n, i, 0, e, s);
    } else s.push(n);
    return s;
  }
  const er = (n) => !Array.isArray(n);
  class ks {
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
      return Ye(e || s, (r) => typeof r == "string" ? r : Array.isArray(r) ? r.map((o) => (o == null ? void 0 : o.src) ?? o) : (r == null ? void 0 : r.src) ? r.src : r, true);
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
      }, Ye(e).forEach((r) => {
        const { src: o } = r;
        let { data: a, format: l, loadParser: c, parser: h } = r;
        const d = Ye(o).map((g) => typeof g == "string" ? Up(g) : Array.isArray(g) ? g : [
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
      const e = er(t);
      t = Ye(t);
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
      const e = er(t);
      t = Ye(t);
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
      return (this._basePath || this._rootPath) && (t.src = Pe.toAbsolute(t.src, this._basePath, this._rootPath)), t.alias = s ?? t.alias ?? [
        t.src
      ], t.src = this._appendDefaultSearchParams(t.src), t.data = {
        ...i || {},
        ...t.data
      }, t.loadParser = r ?? t.loadParser, t.parser = o ?? t.parser, t.format = a ?? t.format ?? jp(t.src), l !== void 0 && (t.progressSize = l), t;
    }
  }
  ks.RETINA_PREFIX = /@([0-9\.]+)x/;
  function jp(n) {
    return n.split(".").pop().split("?").shift().split("#").shift();
  }
  const Ao = (n, t) => {
    const e = t.split("?")[1];
    return e && (n += `?${e}`), n;
  }, Bh = class Ys {
    constructor(t, e) {
      this.linkedSheets = [];
      let s = t;
      (t == null ? void 0 : t.source) instanceof Re && (s = {
        texture: t,
        data: e
      });
      const { texture: i, data: r, cachePrefix: o = "" } = s;
      this.cachePrefix = o, this._texture = i instanceof Tt ? i : null, this.textureSource = i.source, this.textures = {}, this.animations = {}, this.data = r;
      const a = parseFloat(r.meta.scale);
      a ? (this.resolution = a, i.source.resolution = this.resolution) : this.resolution = i.source._resolution, this._frames = this.data.frames, this._frameKeys = Object.keys(this._frames), this._batchIndex = 0, this._callback = null;
    }
    parse() {
      return new Promise((t) => {
        this._callback = t, this._batchIndex = 0, this._frameKeys.length <= Ys.BATCH_SIZE ? (this._processFrames(0), this._processAnimations(), this._parseComplete()) : this._nextBatch();
      });
    }
    parseSync() {
      return this._processFrames(0, true), this._processAnimations(), this.textures;
    }
    _processFrames(t, e = false) {
      let s = t;
      const i = e ? 1 / 0 : Ys.BATCH_SIZE;
      for (; s - t < i && s < this._frameKeys.length; ) {
        const r = this._frameKeys[s], o = this._frames[r], a = o.frame;
        if (a) {
          let l = null, c = null;
          const h = o.trimmed !== false && o.sourceSize ? o.sourceSize : o.frame, d = new Dt(0, 0, Math.floor(h.w) / this.resolution, Math.floor(h.h) / this.resolution);
          o.rotated ? l = new Dt(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.h) / this.resolution, Math.floor(a.w) / this.resolution) : l = new Dt(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution), o.trimmed !== false && o.spriteSourceSize && (c = new Dt(Math.floor(o.spriteSourceSize.x) / this.resolution, Math.floor(o.spriteSourceSize.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution)), this.textures[r] = new Tt({
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
      this._processFrames(this._batchIndex * Ys.BATCH_SIZE), this._batchIndex++, setTimeout(() => {
        this._batchIndex * Ys.BATCH_SIZE < this._frameKeys.length ? this._nextBatch() : (this._processAnimations(), this._parseComplete());
      }, 0);
    }
    destroy(t = false) {
      var _a2;
      for (const e in this.textures) this.textures[e].destroy();
      this._frames = null, this._frameKeys = null, this.data = null, this.textures = null, t && ((_a2 = this._texture) == null ? void 0 : _a2.destroy(), this.textureSource.destroy()), this._texture = null, this.textureSource = null, this.linkedSheets = [];
    }
  };
  Bh.BATCH_SIZE = 1e3;
  let hl = Bh;
  const Yp = [
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
  function Oh(n, t, e) {
    const s = {};
    if (n.forEach((i) => {
      s[i] = t;
    }), Object.keys(t.textures).forEach((i) => {
      s[`${t.cachePrefix}${i}`] = t.textures[i];
    }), !e) {
      const i = Pe.dirname(n[0]);
      t.linkedSheets.forEach((r, o) => {
        const a = Oh([
          `${i}/${t.data.meta.related_multi_packs[o]}`
        ], r, true);
        Object.assign(s, a);
      });
    }
    return s;
  }
  const Vp = {
    extension: lt.Asset,
    cache: {
      test: (n) => n instanceof hl,
      getCacheableAssets: (n, t) => Oh(n, t, false)
    },
    resolver: {
      extension: {
        type: lt.ResolveParser,
        name: "resolveSpritesheet"
      },
      test: (n) => {
        const e = n.split("?")[0].split("."), s = e.pop(), i = e.pop();
        return s === "json" && Yp.includes(i);
      },
      parse: (n) => {
        var _a2;
        const t = n.split(".");
        return {
          resolution: parseFloat(((_a2 = ks.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
          format: t[t.length - 2],
          src: n
        };
      }
    },
    loader: {
      name: "spritesheetLoader",
      id: "spritesheet",
      extension: {
        type: lt.LoadParser,
        priority: Bn.Normal,
        name: "spritesheetLoader"
      },
      async testParse(n, t) {
        return Pe.extname(t.src).toLowerCase() === ".json" && !!n.frames;
      },
      async parse(n, t, e) {
        var _a2, _b2;
        const { texture: s, imageFilename: i, textureOptions: r, cachePrefix: o } = (t == null ? void 0 : t.data) ?? {};
        let a = Pe.dirname(t.src);
        a && a.lastIndexOf("/") !== a.length - 1 && (a += "/");
        let l;
        if (s instanceof Tt) l = s;
        else {
          const d = Ao(a + (i ?? n.meta.image), t.src);
          l = (await e.load([
            {
              src: d,
              data: r
            }
          ]))[d];
        }
        const c = new hl({
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
            ((_b2 = t.data) == null ? void 0 : _b2.ignoreMultiPack) || (f = Ao(f, t.src), d.push(e.load({
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
  Wt.add(Vp);
  const Gr = /* @__PURE__ */ Object.create(null), dl = /* @__PURE__ */ Object.create(null);
  la = function(n, t) {
    let e = dl[n];
    return e === void 0 && (Gr[t] === void 0 && (Gr[t] = 1), dl[n] = e = Gr[t]++), e;
  };
  let ki;
  function Nh() {
    return (!ki || (ki == null ? void 0 : ki.isContextLost())) && (ki = Rt.get().createCanvas().getContext("webgl", {})), ki;
  }
  let Mi;
  function Xp() {
    if (!Mi) {
      Mi = "mediump";
      const n = Nh();
      n && n.getShaderPrecisionFormat && (Mi = n.getShaderPrecisionFormat(n.FRAGMENT_SHADER, n.HIGH_FLOAT).precision ? "highp" : "mediump");
    }
    return Mi;
  }
  function qp(n, t, e) {
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
  function Kp(n, t, e) {
    const s = e ? t.maxSupportedFragmentPrecision : t.maxSupportedVertexPrecision;
    if (n.substring(0, 9) !== "precision") {
      let i = e ? t.requestedFragmentPrecision : t.requestedVertexPrecision;
      return i === "highp" && s !== "highp" && (i = "mediump"), `precision ${i} float;
${n}`;
    } else if (s !== "highp" && n.substring(0, 15) === "precision highp") return n.replace("precision highp", "precision mediump");
    return n;
  }
  function Jp(n, t) {
    return t ? `#version 300 es
${n}` : n;
  }
  const Zp = {}, Qp = {};
  function tf(n, { name: t = "pixi-program" }, e = true) {
    t = t.replace(/\s+/g, "-"), t += e ? "-fragment" : "-vertex";
    const s = e ? Zp : Qp;
    return s[t] ? (s[t]++, t += `-${s[t]}`) : s[t] = 1, n.indexOf("#define SHADER_NAME") !== -1 ? n : `${`#define SHADER_NAME ${t}`}
${n}`;
  }
  function ef(n, t) {
    return t ? n.replace("#version 300 es", "") : n;
  }
  const zr = {
    stripVersion: ef,
    ensurePrecision: Kp,
    addProgramDefines: qp,
    setProgramName: tf,
    insertVersion: Jp
  }, Os = /* @__PURE__ */ Object.create(null), Fh = class ko {
    constructor(t) {
      t = {
        ...ko.defaultOptions,
        ...t
      };
      const e = t.fragment.indexOf("#version 300 es") !== -1, s = {
        stripVersion: e,
        ensurePrecision: {
          requestedFragmentPrecision: t.preferredFragmentPrecision,
          requestedVertexPrecision: t.preferredVertexPrecision,
          maxSupportedVertexPrecision: "highp",
          maxSupportedFragmentPrecision: Xp()
        },
        setProgramName: {
          name: t.name
        },
        addProgramDefines: e,
        insertVersion: e
      };
      let i = t.fragment, r = t.vertex;
      Object.keys(zr).forEach((o) => {
        const a = s[o];
        i = zr[o](i, a, true), r = zr[o](r, a, false);
      }), this.fragment = i, this.vertex = r, this.transformFeedbackVaryings = t.transformFeedbackVaryings, this._key = la(`${this.vertex}:${this.fragment}`, "gl-program");
    }
    destroy() {
      this.fragment = null, this.vertex = null, this._attributeData = null, this._uniformData = null, this._uniformBlockData = null, this.transformFeedbackVaryings = null, Os[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex}:${t.fragment}`;
      return Os[e] || (Os[e] = new ko(t), Os[e]._cacheKey = e), Os[e];
    }
  };
  Fh.defaultOptions = {
    preferredVertexPrecision: "highp",
    preferredFragmentPrecision: "mediump"
  };
  ca = Fh;
  const ul = {
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
  nr = function(n) {
    return ul[n] ?? ul.float32;
  };
  const nf = {
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
  }, pl = /@location\((\d+)\)\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_<>]+)(?:,|\s|\)|$)/g;
  function fl(n, t) {
    let e;
    for (; (e = pl.exec(n)) !== null; ) {
      const s = nf[e[3]] ?? "float32";
      t[e[2]] = {
        location: parseInt(e[1], 10),
        format: s,
        stride: nr(s).stride,
        offset: 0,
        instance: false,
        start: 0
      };
    }
    pl.lastIndex = 0;
  }
  function sf(n) {
    return n.replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  }
  function rf({ source: n, entryPoint: t }) {
    const e = {}, s = sf(n), i = s.indexOf(`fn ${t}(`);
    if (i === -1) return e;
    const r = s.indexOf("->", i);
    if (r === -1) return e;
    const o = s.substring(i, r);
    if (fl(o, e), Object.keys(e).length === 0) {
      const a = o.match(/\(\s*\w+\s*:\s*(\w+)/);
      if (a) {
        const l = a[1], c = new RegExp(`struct\\s+${l}\\s*\\{([^}]+)\\}`, "s"), h = s.match(c);
        h && fl(h[1], e);
      }
    }
    return e;
  }
  function Dr(n) {
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
  var Vn = ((n) => (n[n.VERTEX = 1] = "VERTEX", n[n.FRAGMENT = 2] = "FRAGMENT", n[n.COMPUTE = 4] = "COMPUTE", n))(Vn || {});
  function of({ groups: n }) {
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
  function af({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = {}), t[s.group][s.name] = s.binding;
    }
    return t;
  }
  function lf(n, t) {
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
  const Ns = /* @__PURE__ */ Object.create(null);
  gi = class {
    constructor(t) {
      var _a2, _b2;
      this._layoutKey = 0, this._attributeLocationsKey = 0;
      const { fragment: e, vertex: s, layout: i, gpuLayout: r, name: o } = t;
      if (this.name = o, this.fragment = e, this.vertex = s, e.source === s.source) {
        const a = Dr(e.source);
        this.structsAndGroups = a;
      } else {
        const a = Dr(s.source), l = Dr(e.source);
        this.structsAndGroups = lf(a, l);
      }
      this.layout = i ?? af(this.structsAndGroups), this.gpuLayout = r ?? of(this.structsAndGroups), this.autoAssignGlobalUniforms = ((_a2 = this.layout[0]) == null ? void 0 : _a2.globalUniforms) !== void 0, this.autoAssignLocalUniforms = ((_b2 = this.layout[1]) == null ? void 0 : _b2.localUniforms) !== void 0, this._generateProgramKey();
    }
    _generateProgramKey() {
      const { vertex: t, fragment: e } = this, s = t.source + e.source + t.entryPoint + e.entryPoint;
      this._layoutKey = la(s, "program");
    }
    get attributeData() {
      return this._attributeData ?? (this._attributeData = rf(this.vertex)), this._attributeData;
    }
    destroy() {
      this.gpuLayout = null, this.layout = null, this.structsAndGroups = null, this.fragment = null, this.vertex = null, Ns[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex.source}:${t.fragment.source}:${t.fragment.entryPoint}:${t.vertex.entryPoint}`;
      return Ns[e] || (Ns[e] = new gi(t), Ns[e]._cacheKey = e), Ns[e];
    }
  };
  const Wh = [
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
  ], cf = Wh.reduce((n, t) => (n[t] = true, n), {});
  function hf(n, t) {
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
  const Gh = class zh {
    constructor(t, e) {
      this._touched = 0, this.uid = ne("uniform"), this._resourceType = "uniformGroup", this._resourceId = ne("resource"), this.isUniformGroup = true, this._dirtyId = 0, this.destroyed = false, e = {
        ...zh.defaultOptions,
        ...e
      }, this.uniformStructures = t;
      const s = {};
      for (const i in t) {
        const r = t[i];
        if (r.name = i, r.size = r.size ?? 1, !cf[r.type]) {
          const o = r.type.match(/^array<(\w+(?:<\w+>)?),\s*(\d+)>$/);
          if (o) {
            const [, a, l] = o;
            throw new Error(`Uniform type ${r.type} is not supported. Use type: '${a}', size: ${l} instead.`);
          }
          throw new Error(`Uniform type ${r.type} is not supported. Supported uniform types are: ${Wh.join(", ")}`);
        }
        r.value ?? (r.value = hf(r.type, r.size)), s[i] = r.value;
      }
      this.uniforms = s, this._dirtyId = 1, this.ubo = e.ubo, this.isStatic = e.isStatic, this._signature = la(Object.keys(s).map((i) => `${i}-${t[i].type}`).join("-"), "uniform-group");
    }
    update() {
      this._dirtyId++;
    }
  };
  Gh.defaultOptions = {
    ubo: false,
    isStatic: false
  };
  ha = Gh;
  Ji = class {
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
  Mo = ((n) => (n[n.WEBGL = 1] = "WEBGL", n[n.WEBGPU = 2] = "WEBGPU", n[n.CANVAS = 4] = "CANVAS", n[n.BOTH = 3] = "BOTH", n))(Mo || {});
  gr = class extends on {
    constructor(t) {
      super(), this.uid = ne("shader"), this._uniformBindMap = /* @__PURE__ */ Object.create(null), this._ownedBindGroups = [], this._destroyed = false;
      let { gpuProgram: e, glProgram: s, groups: i, resources: r, compatibleRenderers: o, groupMap: a } = t;
      this.gpuProgram = e, this.glProgram = s, o === void 0 && (o = 0, e && (o |= Mo.WEBGPU), s && (o |= Mo.WEBGL)), this.compatibleRenderers = o;
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
        for (const h in r) l[h] || (i[99] || (i[99] = new Ji(), this._ownedBindGroups.push(i[99])), l[h] = {
          group: 99,
          binding: c,
          name: h
        }, a[99] = a[99] || {}, a[99][c] = h, c++);
        for (const h in r) {
          const d = h;
          let u = r[h];
          !u.source && !u._resourceType && (u = new ha(u));
          const p = l[d];
          p && (i[p.group] || (i[p.group] = new Ji(), this._ownedBindGroups.push(i[p.group])), i[p.group].setResource(u, p.binding));
        }
      }
      this.groups = i, this._uniformBindMap = a, this.resources = this._buildResourceAccessor(i, l);
    }
    addResource(t, e, s) {
      var i, r;
      (i = this._uniformBindMap)[e] || (i[e] = {}), (r = this._uniformBindMap[e])[s] || (r[s] = t), this.groups[e] || (this.groups[e] = new Ji(), this._ownedBindGroups.push(this.groups[e]));
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
      return e && (r = gi.from(e)), s && (o = ca.from(s)), new gr({
        gpuProgram: r,
        glProgram: o,
        ...i
      });
    }
  };
  const df = {
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
  }, Hr = 0, Ur = 1, jr = 2, Yr = 3, Vr = 4, Xr = 5, Po = class Dh {
    constructor() {
      this.data = 0, this.blendMode = "normal", this.polygonOffset = 0, this.blend = true, this.depthMask = true;
    }
    get blend() {
      return !!(this.data & 1 << Hr);
    }
    set blend(t) {
      !!(this.data & 1 << Hr) !== t && (this.data ^= 1 << Hr);
    }
    get offsets() {
      return !!(this.data & 1 << Ur);
    }
    set offsets(t) {
      !!(this.data & 1 << Ur) !== t && (this.data ^= 1 << Ur);
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
      return !!(this.data & 1 << jr);
    }
    set culling(t) {
      !!(this.data & 1 << jr) !== t && (this.data ^= 1 << jr);
    }
    get depthTest() {
      return !!(this.data & 1 << Yr);
    }
    set depthTest(t) {
      !!(this.data & 1 << Yr) !== t && (this.data ^= 1 << Yr);
    }
    get depthMask() {
      return !!(this.data & 1 << Xr);
    }
    set depthMask(t) {
      !!(this.data & 1 << Xr) !== t && (this.data ^= 1 << Xr);
    }
    get clockwiseFrontFace() {
      return !!(this.data & 1 << Vr);
    }
    set clockwiseFrontFace(t) {
      !!(this.data & 1 << Vr) !== t && (this.data ^= 1 << Vr);
    }
    get blendMode() {
      return this._blendMode;
    }
    set blendMode(t) {
      this.blend = t !== "none", this._blendMode = t, this._blendModeId = df[t] || 0;
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
      const t = new Dh();
      return t.depthTest = false, t.blend = true, t;
    }
  };
  Po.default2d = Po.for2d();
  sr = Po;
  const Io = [];
  Wt.handleByNamedList(lt.Environment, Io);
  async function uf(n) {
    if (!n) for (let t = 0; t < Io.length; t++) {
      const e = Io[t];
      if (e.value.test()) {
        await e.value.load();
        return;
      }
    }
  }
  let Fs;
  pf = function() {
    if (typeof Fs == "boolean") return Fs;
    try {
      Fs = new Function("param1", "param2", "param3", "return param1[param2] === param3;")({
        a: "b"
      }, "a", "b") === true;
    } catch {
      Fs = false;
    }
    return Fs;
  };
  function ml(n, t, e = 2) {
    const s = t && t.length, i = s ? t[0] * e : n.length;
    let r = Hh(n, 0, i, e, true);
    const o = [];
    if (!r || r.next === r.prev) return o;
    let a, l, c;
    if (s && (r = xf(n, t, r, e)), n.length > 80 * e) {
      a = n[0], l = n[1];
      let h = a, d = l;
      for (let u = e; u < i; u += e) {
        const p = n[u], f = n[u + 1];
        p < a && (a = p), f < l && (l = f), p > h && (h = p), f > d && (d = f);
      }
      c = Math.max(h - a, d - l), c = c !== 0 ? 32767 / c : 0;
    }
    return ri(r, o, e, a, l, c, 0), o;
  }
  function Hh(n, t, e, s, i) {
    let r;
    if (i === Mf(n, t, e, s) > 0) for (let o = t; o < e; o += s) r = gl(o / s | 0, n[o], n[o + 1], r);
    else for (let o = e - s; o >= t; o -= s) r = gl(o / s | 0, n[o], n[o + 1], r);
    return r && Ts(r, r.next) && (ai(r), r = r.next), r;
  }
  function Qn(n, t) {
    if (!n) return n;
    t || (t = n);
    let e = n, s;
    do
      if (s = false, !e.steiner && (Ts(e, e.next) || Qt(e.prev, e, e.next) === 0)) {
        if (ai(e), e = t = e.prev, e === e.next) break;
        s = true;
      } else e = e.next;
    while (s || e !== t);
    return t;
  }
  function ri(n, t, e, s, i, r, o) {
    if (!n) return;
    !o && r && Cf(n, s, i, r);
    let a = n;
    for (; n.prev !== n.next; ) {
      const l = n.prev, c = n.next;
      if (r ? mf(n, s, i, r) : ff(n)) {
        t.push(l.i, n.i, c.i), ai(n), n = c.next, a = c.next;
        continue;
      }
      if (n = c, n === a) {
        o ? o === 1 ? (n = gf(Qn(n), t), ri(n, t, e, s, i, r, 2)) : o === 2 && yf(n, t, e, s, i, r) : ri(Qn(n), t, e, s, i, r, 1);
        break;
      }
    }
  }
  function ff(n) {
    const t = n.prev, e = n, s = n.next;
    if (Qt(t, e, s) >= 0) return false;
    const i = t.x, r = e.x, o = s.x, a = t.y, l = e.y, c = s.y, h = Math.min(i, r, o), d = Math.min(a, l, c), u = Math.max(i, r, o), p = Math.max(a, l, c);
    let f = s.next;
    for (; f !== t; ) {
      if (f.x >= h && f.x <= u && f.y >= d && f.y <= p && Vs(i, a, r, l, o, c, f.x, f.y) && Qt(f.prev, f, f.next) >= 0) return false;
      f = f.next;
    }
    return true;
  }
  function mf(n, t, e, s) {
    const i = n.prev, r = n, o = n.next;
    if (Qt(i, r, o) >= 0) return false;
    const a = i.x, l = r.x, c = o.x, h = i.y, d = r.y, u = o.y, p = Math.min(a, l, c), f = Math.min(h, d, u), g = Math.max(a, l, c), m = Math.max(h, d, u), y = Ro(p, f, t, e, s), b = Ro(g, m, t, e, s);
    let x = n.prevZ, _ = n.nextZ;
    for (; x && x.z >= y && _ && _.z <= b; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && Vs(a, h, l, d, c, u, x.x, x.y) && Qt(x.prev, x, x.next) >= 0 || (x = x.prevZ, _.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && Vs(a, h, l, d, c, u, _.x, _.y) && Qt(_.prev, _, _.next) >= 0)) return false;
      _ = _.nextZ;
    }
    for (; x && x.z >= y; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && Vs(a, h, l, d, c, u, x.x, x.y) && Qt(x.prev, x, x.next) >= 0) return false;
      x = x.prevZ;
    }
    for (; _ && _.z <= b; ) {
      if (_.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && Vs(a, h, l, d, c, u, _.x, _.y) && Qt(_.prev, _, _.next) >= 0) return false;
      _ = _.nextZ;
    }
    return true;
  }
  function gf(n, t) {
    let e = n;
    do {
      const s = e.prev, i = e.next.next;
      !Ts(s, i) && jh(s, e, e.next, i) && oi(s, i) && oi(i, s) && (t.push(s.i, e.i, i.i), ai(e), ai(e.next), e = n = i), e = e.next;
    } while (e !== n);
    return Qn(e);
  }
  function yf(n, t, e, s, i, r) {
    let o = n;
    do {
      let a = o.next.next;
      for (; a !== o.prev; ) {
        if (o.i !== a.i && Ef(o, a)) {
          let l = Yh(o, a);
          o = Qn(o, o.next), l = Qn(l, l.next), ri(o, t, e, s, i, r, 0), ri(l, t, e, s, i, r, 0);
          return;
        }
        a = a.next;
      }
      o = o.next;
    } while (o !== n);
  }
  function xf(n, t, e, s) {
    const i = [];
    for (let r = 0, o = t.length; r < o; r++) {
      const a = t[r] * s, l = r < o - 1 ? t[r + 1] * s : n.length, c = Hh(n, a, l, s, false);
      c === c.next && (c.steiner = true), i.push(Tf(c));
    }
    i.sort(bf);
    for (let r = 0; r < i.length; r++) e = _f(i[r], e);
    return e;
  }
  function bf(n, t) {
    let e = n.x - t.x;
    if (e === 0 && (e = n.y - t.y, e === 0)) {
      const s = (n.next.y - n.y) / (n.next.x - n.x), i = (t.next.y - t.y) / (t.next.x - t.x);
      e = s - i;
    }
    return e;
  }
  function _f(n, t) {
    const e = wf(n, t);
    if (!e) return t;
    const s = Yh(e, n);
    return Qn(s, s.next), Qn(e, e.next);
  }
  function wf(n, t) {
    let e = t;
    const s = n.x, i = n.y;
    let r = -1 / 0, o;
    if (Ts(n, e)) return e;
    do {
      if (Ts(n, e.next)) return e.next;
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
      if (s >= e.x && e.x >= l && s !== e.x && Uh(i < c ? s : r, i, l, c, i < c ? r : s, i, e.x, e.y)) {
        const d = Math.abs(i - e.y) / (s - e.x);
        oi(e, n) && (d < h || d === h && (e.x > o.x || e.x === o.x && vf(o, e))) && (o = e, h = d);
      }
      e = e.next;
    } while (e !== a);
    return o;
  }
  function vf(n, t) {
    return Qt(n.prev, n, t.prev) < 0 && Qt(t.next, n, n.next) < 0;
  }
  function Cf(n, t, e, s) {
    let i = n;
    do
      i.z === 0 && (i.z = Ro(i.x, i.y, t, e, s)), i.prevZ = i.prev, i.nextZ = i.next, i = i.next;
    while (i !== n);
    i.prevZ.nextZ = null, i.prevZ = null, Sf(i);
  }
  function Sf(n) {
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
  function Ro(n, t, e, s, i) {
    return n = (n - e) * i | 0, t = (t - s) * i | 0, n = (n | n << 8) & 16711935, n = (n | n << 4) & 252645135, n = (n | n << 2) & 858993459, n = (n | n << 1) & 1431655765, t = (t | t << 8) & 16711935, t = (t | t << 4) & 252645135, t = (t | t << 2) & 858993459, t = (t | t << 1) & 1431655765, n | t << 1;
  }
  function Tf(n) {
    let t = n, e = n;
    do
      (t.x < e.x || t.x === e.x && t.y < e.y) && (e = t), t = t.next;
    while (t !== n);
    return e;
  }
  function Uh(n, t, e, s, i, r, o, a) {
    return (i - o) * (t - a) >= (n - o) * (r - a) && (n - o) * (s - a) >= (e - o) * (t - a) && (e - o) * (r - a) >= (i - o) * (s - a);
  }
  function Vs(n, t, e, s, i, r, o, a) {
    return !(n === o && t === a) && Uh(n, t, e, s, i, r, o, a);
  }
  function Ef(n, t) {
    return n.next.i !== t.i && n.prev.i !== t.i && !Af(n, t) && (oi(n, t) && oi(t, n) && kf(n, t) && (Qt(n.prev, n, t.prev) || Qt(n, t.prev, t)) || Ts(n, t) && Qt(n.prev, n, n.next) > 0 && Qt(t.prev, t, t.next) > 0);
  }
  function Qt(n, t, e) {
    return (t.y - n.y) * (e.x - t.x) - (t.x - n.x) * (e.y - t.y);
  }
  function Ts(n, t) {
    return n.x === t.x && n.y === t.y;
  }
  function jh(n, t, e, s) {
    const i = Ii(Qt(n, t, e)), r = Ii(Qt(n, t, s)), o = Ii(Qt(e, s, n)), a = Ii(Qt(e, s, t));
    return !!(i !== r && o !== a || i === 0 && Pi(n, e, t) || r === 0 && Pi(n, s, t) || o === 0 && Pi(e, n, s) || a === 0 && Pi(e, t, s));
  }
  function Pi(n, t, e) {
    return t.x <= Math.max(n.x, e.x) && t.x >= Math.min(n.x, e.x) && t.y <= Math.max(n.y, e.y) && t.y >= Math.min(n.y, e.y);
  }
  function Ii(n) {
    return n > 0 ? 1 : n < 0 ? -1 : 0;
  }
  function Af(n, t) {
    let e = n;
    do {
      if (e.i !== n.i && e.next.i !== n.i && e.i !== t.i && e.next.i !== t.i && jh(e, e.next, n, t)) return true;
      e = e.next;
    } while (e !== n);
    return false;
  }
  function oi(n, t) {
    return Qt(n.prev, n, n.next) < 0 ? Qt(n, t, n.next) >= 0 && Qt(n, n.prev, t) >= 0 : Qt(n, t, n.prev) < 0 || Qt(n, n.next, t) < 0;
  }
  function kf(n, t) {
    let e = n, s = false;
    const i = (n.x + t.x) / 2, r = (n.y + t.y) / 2;
    do
      e.y > r != e.next.y > r && e.next.y !== e.y && i < (e.next.x - e.x) * (r - e.y) / (e.next.y - e.y) + e.x && (s = !s), e = e.next;
    while (e !== n);
    return s;
  }
  function Yh(n, t) {
    const e = Lo(n.i, n.x, n.y), s = Lo(t.i, t.x, t.y), i = n.next, r = t.prev;
    return n.next = t, t.prev = n, e.next = i, i.prev = e, s.next = e, e.prev = s, r.next = s, s.prev = r, s;
  }
  function gl(n, t, e, s) {
    const i = Lo(n, t, e);
    return s ? (i.next = s.next, i.prev = s, s.next.prev = i, s.next = i) : (i.prev = i, i.next = i), i;
  }
  function ai(n) {
    n.next.prev = n.prev, n.prev.next = n.next, n.prevZ && (n.prevZ.nextZ = n.nextZ), n.nextZ && (n.nextZ.prevZ = n.prevZ);
  }
  function Lo(n, t, e) {
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
  function Mf(n, t, e, s) {
    let i = 0;
    for (let r = t, o = e - s; r < e; r += s) i += (n[o] - n[r]) * (n[r + 1] + n[o + 1]), o = r;
    return i;
  }
  const Pf = ml.default || ml;
  Vh = ((n) => (n[n.NONE = 0] = "NONE", n[n.COLOR = 16384] = "COLOR", n[n.STENCIL = 1024] = "STENCIL", n[n.DEPTH = 256] = "DEPTH", n[n.COLOR_DEPTH = 16640] = "COLOR_DEPTH", n[n.COLOR_STENCIL = 17408] = "COLOR_STENCIL", n[n.DEPTH_STENCIL = 1280] = "DEPTH_STENCIL", n[n.ALL = 17664] = "ALL", n))(Vh || {});
  If = class {
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
  const Rf = [
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
  ], Xh = class qh extends on {
    constructor(t) {
      super(), this.tick = 0, this.uid = ne("renderer"), this.runners = /* @__PURE__ */ Object.create(null), this.renderPipes = /* @__PURE__ */ Object.create(null), this._initOptions = {}, this._systemsHash = /* @__PURE__ */ Object.create(null), this.type = t.type, this.name = t.name, this.config = t;
      const e = [
        ...Rf,
        ...this.config.runners ?? []
      ];
      this._addRunners(...e), this._unsafeEvalCheck();
    }
    async init(t = {}) {
      const e = t.skipExtensionImports === true ? true : t.manageImports === false;
      await uf(e), this._addSystems(this.config.systems), this._addPipes(this.config.renderPipes, this.config.renderPipeAdaptors);
      for (const s in this._systemsHash) t = {
        ...this._systemsHash[s].constructor.defaultOptions,
        ...t
      };
      t = {
        ...qh.defaultOptions,
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
      t.target || (t.target = e.renderTarget.renderTarget), t.clearColor || (t.clearColor = this.background.colorRgba), t.clear ?? (t.clear = Vh.ALL);
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
        this.runners[e] = new If(e);
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
      this.runners.destroy.items.reverse(), this.runners.destroy.emit(t), (t === true || typeof t == "object" && t.releaseGlobalResources) && mi.release(), Object.values(this.runners).forEach((e) => {
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
      if (!pf()) throw new Error("Current environment does not allow unsafe-eval, please use pixi.js/unsafe-eval module to enable support.");
    }
    resetState() {
      this.runners.resetState.emit();
    }
  };
  Xh.defaultOptions = {
    resolution: 1,
    failIfMajorPerformanceCaveat: false,
    roundPixels: false
  };
  let Ri;
  Kh = Xh;
  function Lf(n) {
    return Ri !== void 0 || (Ri = (() => {
      var _a2;
      const t = {
        stencil: true,
        failIfMajorPerformanceCaveat: n ?? Kh.defaultOptions.failIfMajorPerformanceCaveat
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
    })()), Ri;
  }
  let Li;
  async function $f(n = {}) {
    return Li !== void 0 || (Li = await (async () => {
      const t = Rt.get().getNavigator().gpu;
      if (!t) return false;
      try {
        return await (await t.requestAdapter(n)).requestDevice(), true;
      } catch {
        return false;
      }
    })()), Li;
  }
  const yl = [
    "webgl",
    "webgpu",
    "canvas"
  ];
  async function Bf(n) {
    let t = [];
    n.preference ? (t.push(n.preference), yl.forEach((r) => {
      r !== n.preference && t.push(r);
    })) : t = yl.slice();
    let e, s = {};
    for (let r = 0; r < t.length; r++) {
      const o = t[r];
      if (o === "webgpu" && await $f()) {
        const { WebGPURenderer: a } = await Rn(async () => {
          const { WebGPURenderer: l } = await import("./WebGPURenderer-CrlozIr6.js");
          return {
            WebGPURenderer: l
          };
        }, __vite__mapDeps([3,4,5,2]));
        e = a, s = {
          ...n,
          ...n.webgpu
        };
        break;
      } else if (o === "webgl" && Lf(n.failIfMajorPerformanceCaveat ?? Kh.defaultOptions.failIfMajorPerformanceCaveat)) {
        const { WebGLRenderer: a } = await Rn(async () => {
          const { WebGLRenderer: l } = await import("./WebGLRenderer-CBX9wrZj.js");
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
        const { CanvasRenderer: a } = await Rn(async () => {
          const { CanvasRenderer: l } = await import("./CanvasRenderer-6eSCbLWC.js");
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
  Jh = "8.17.1";
  class Zh {
    static init() {
      var _a2;
      (_a2 = globalThis.__PIXI_APP_INIT__) == null ? void 0 : _a2.call(globalThis, this, Jh);
    }
    static destroy() {
    }
  }
  Zh.extension = lt.Application;
  Of = class {
    constructor(t) {
      this._renderer = t;
    }
    init() {
      var _a2;
      (_a2 = globalThis.__PIXI_RENDERER_INIT__) == null ? void 0 : _a2.call(globalThis, this._renderer, Jh);
    }
    destroy() {
      this._renderer = null;
    }
  };
  Of.extension = {
    type: [
      lt.WebGLSystem,
      lt.WebGPUSystem
    ],
    name: "initHook",
    priority: -10
  };
  class Qh {
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
  Qh.extension = lt.Application;
  class td {
    static init(t) {
      t = Object.assign({
        autoStart: true,
        sharedTicker: false
      }, t), Object.defineProperty(this, "ticker", {
        configurable: true,
        set(e) {
          this._ticker && this._ticker.remove(this.render, this), this._ticker = e, e && e.add(this.render, this, ii.LOW);
        },
        get() {
          return this._ticker;
        }
      }), this.stop = () => {
        this._ticker.stop();
      }, this.start = () => {
        this._ticker.start();
      }, this._ticker = null, this.ticker = t.sharedTicker ? qn.shared : new qn(), t.autoStart && this.start();
    }
    static destroy() {
      if (this._ticker) {
        const t = this._ticker;
        this.ticker = null, t.destroy();
      }
    }
  }
  td.extension = lt.Application;
  Wt.add(Qh);
  Wt.add(td);
  const ed = class $o {
    constructor(...t) {
      this.stage = new zt(), t[0] !== void 0 && It(ee, "Application constructor options are deprecated, please use Application.init() instead.");
    }
    async init(t) {
      t = {
        ...t
      }, this.stage || (this.stage = new zt()), this.renderer = await Bf(t), $o._plugins.forEach((e) => {
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
      const s = $o._plugins.slice(0);
      s.reverse(), s.forEach((i) => {
        i.destroy.call(this);
      }), this.stage.destroy(e), this.stage = null, this.renderer.destroy(t), this.renderer = null;
    }
  };
  ed._plugins = [];
  da = ed;
  Wt.handleByList(lt.Application, da._plugins);
  Wt.add(Zh);
  const qr = {
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
  }, xl = {
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
  }, bl = {
    test(n) {
      return typeof n == "string" && n.match(/<font(\s|>)/) ? xl.test(Rt.get().parseXML(n)) : false;
    },
    parse(n) {
      return xl.parse(Rt.get().parseXML(n));
    }
  }, Nf = [
    ".xml",
    ".fnt"
  ], Ff = {
    extension: {
      type: lt.CacheParser,
      name: "cacheBitmapFont"
    },
    test: (n) => !!(n == null ? void 0 : n.pages) && !!(n == null ? void 0 : n.chars) && typeof (n == null ? void 0 : n.fontFamily) == "string" && n.fontFamily !== "",
    getCacheableAssets(n, t) {
      const e = {};
      return n.forEach((s) => {
        e[s] = t, e[`${s}-bitmap`] = t;
      }), e[`${t.fontFamily}-bitmap`] = t, e;
    }
  }, Wf = {
    extension: {
      type: lt.LoadParser,
      priority: Bn.Normal
    },
    name: "loadBitmapFont",
    id: "bitmap-font",
    test(n) {
      return Nf.includes(Pe.extname(n).toLowerCase());
    },
    async testParse(n) {
      return qr.test(n) || bl.test(n);
    },
    async parse(n, t, e) {
      const s = qr.test(n) ? qr.parse(n) : bl.parse(n), { src: i } = t, { pages: r } = s, o = [], a = s.distanceField ? {
        scaleMode: "linear",
        alphaMode: "premultiply-alpha-on-upload",
        autoGenerateMipmaps: false,
        resolution: 1
      } : {};
      for (let u = 0; u < r.length; ++u) {
        const p = r[u].file;
        let f = Pe.join(Pe.dirname(i), p);
        f = Ao(f, i), o.push({
          src: f,
          data: a
        });
      }
      const [l, { BitmapFont: c }] = await Promise.all([
        e.load(o),
        Rn(() => import("./BitmapFont-DJnsbd52.js"), [])
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
  class Gf {
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
  const zf = {
    extension: {
      type: lt.CacheParser,
      name: "cacheTextureArray"
    },
    test: (n) => Array.isArray(n) && n.every((t) => t instanceof Tt),
    getCacheableAssets: (n, t) => {
      const e = {};
      return n.forEach((s) => {
        t.forEach((i, r) => {
          e[s + (r === 0 ? "" : r + 1)] = i;
        });
      }), e;
    }
  };
  async function nd(n) {
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
  const Df = {
    extension: {
      type: lt.DetectionParser,
      priority: 1
    },
    test: async () => nd("data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A="),
    add: async (n) => [
      ...n,
      "avif"
    ],
    remove: async (n) => n.filter((t) => t !== "avif")
  }, _l = [
    "png",
    "jpg",
    "jpeg"
  ], Hf = {
    extension: {
      type: lt.DetectionParser,
      priority: -1
    },
    test: () => Promise.resolve(true),
    add: async (n) => [
      ...n,
      ..._l
    ],
    remove: async (n) => n.filter((t) => !_l.includes(t))
  }, Uf = "WorkerGlobalScope" in globalThis && globalThis instanceof globalThis.WorkerGlobalScope;
  function yr(n) {
    return Uf ? false : document.createElement("video").canPlayType(n) !== "";
  }
  const jf = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => yr("video/mp4"),
    add: async (n) => [
      ...n,
      "mp4",
      "m4v"
    ],
    remove: async (n) => n.filter((t) => t !== "mp4" && t !== "m4v")
  }, Yf = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => yr("video/ogg"),
    add: async (n) => [
      ...n,
      "ogv"
    ],
    remove: async (n) => n.filter((t) => t !== "ogv")
  }, Vf = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => yr("video/webm"),
    add: async (n) => [
      ...n,
      "webm"
    ],
    remove: async (n) => n.filter((t) => t !== "webm")
  }, Xf = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => nd("data:image/webp;base64,UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAAAfQ//73v/+BiOh/AAA="),
    add: async (n) => [
      ...n,
      "webp"
    ],
    remove: async (n) => n.filter((t) => t !== "webp")
  }, sd = class Zi {
    constructor() {
      this.loadOptions = {
        ...Zi.defaultOptions
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
        ...Zi.defaultOptions,
        ...this.loadOptions,
        onProgress: e
      } : {
        ...Zi.defaultOptions,
        ...this.loadOptions,
        ...e || {}
      }, { onProgress: i, onError: r, strategy: o, retryCount: a, retryDelay: l } = s;
      let c = 0;
      const h = {}, d = er(t), u = Ye(t, (g) => ({
        alias: [
          g
        ],
        src: g,
        data: {}
      })), p = u.reduce((g, m) => g + (m.progressSize || 1), 0), f = u.map(async (g) => {
        const m = Pe.toAbsolute(g.src);
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
      const s = Ye(t, (i) => ({
        alias: [
          i
        ],
        src: i
      })).map(async (i) => {
        var _a2, _b2;
        const r = Pe.toAbsolute(i.src), o = this.promiseCache[r];
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
  sd.defaultOptions = {
    onProgress: void 0,
    onError: void 0,
    strategy: "throw",
    retryCount: 3,
    retryDelay: 250
  };
  let qf = sd;
  function Ms(n, t) {
    if (Array.isArray(t)) {
      for (const e of t) if (n.startsWith(`data:${e}`)) return true;
      return false;
    }
    return n.startsWith(`data:${t}`);
  }
  function Ps(n, t) {
    const e = n.split("?")[0], s = Pe.extname(e).toLowerCase();
    return Array.isArray(t) ? t.includes(s) : s === t;
  }
  const Kf = ".json", Jf = "application/json", Zf = {
    extension: {
      type: lt.LoadParser,
      priority: Bn.Low
    },
    name: "loadJson",
    id: "json",
    test(n) {
      return Ms(n, Jf) || Ps(n, Kf);
    },
    async load(n) {
      return await (await Rt.get().fetch(n)).json();
    }
  }, Qf = ".txt", tm = "text/plain", em = {
    name: "loadTxt",
    id: "text",
    extension: {
      type: lt.LoadParser,
      priority: Bn.Low,
      name: "loadTxt"
    },
    test(n) {
      return Ms(n, tm) || Ps(n, Qf);
    },
    async load(n) {
      return await (await Rt.get().fetch(n)).text();
    }
  }, nm = [
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
  ], sm = [
    ".ttf",
    ".otf",
    ".woff",
    ".woff2"
  ], im = [
    "font/ttf",
    "font/otf",
    "font/woff",
    "font/woff2"
  ], rm = /^(--|-?[A-Z_])[0-9A-Z_-]*$/i;
  function om(n) {
    const t = Pe.extname(n), i = Pe.basename(n, t).replace(/(-|_)/g, " ").toLowerCase().split(" ").map((a) => a.charAt(0).toUpperCase() + a.slice(1));
    let r = i.length > 0;
    for (const a of i) if (!a.match(rm)) {
      r = false;
      break;
    }
    let o = i.join(" ");
    return r || (o = `"${o.replace(/[\\"]/g, "\\$&")}"`), o;
  }
  const am = /^[0-9A-Za-z%:/?#\[\]@!\$&'()\*\+,;=\-._~]*$/;
  function lm(n) {
    return am.test(n) ? n : encodeURI(n);
  }
  const cm = {
    extension: {
      type: lt.LoadParser,
      priority: Bn.Low
    },
    name: "loadWebFont",
    id: "web-font",
    test(n) {
      return Ms(n, im) || Ps(n, sm);
    },
    async load(n, t) {
      var _a2, _b2, _c2;
      const e = Rt.get().getFontFaceSet();
      if (e) {
        const s = [], i = ((_a2 = t.data) == null ? void 0 : _a2.family) ?? om(n), r = ((_c2 = (_b2 = t.data) == null ? void 0 : _b2.weights) == null ? void 0 : _c2.filter((a) => nm.includes(a))) ?? [
          "normal"
        ], o = t.data ?? {};
        for (let a = 0; a < r.length; a++) {
          const l = r[a], c = new FontFace(i, `url('${lm(n)}')`, {
            ...o,
            weight: l
          });
          await c.load(), e.add(c), s.push(c);
        }
        return ie.has(`${i}-and-url`) ? ie.get(`${i}-and-url`).entries.push({
          url: n,
          faces: s
        }) : ie.set(`${i}-and-url`, {
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
      ], e = t[0].family, s = ie.get(`${e}-and-url`), i = s.entries.find((r) => r.faces.some((o) => t.indexOf(o) !== -1));
      i.faces = i.faces.filter((r) => t.indexOf(r) === -1), i.faces.length === 0 && (s.entries = s.entries.filter((r) => r !== i)), t.forEach((r) => {
        Rt.get().getFontFaceSet().delete(r);
      }), s.entries.length === 0 && ie.remove(`${e}-and-url`);
    }
  };
  var Kr, wl;
  function hm() {
    if (wl) return Kr;
    wl = 1, Kr = e;
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
    return Kr;
  }
  var dm = hm();
  const um = rh(dm);
  function pm(n, t) {
    const e = um(n), s = [];
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
  class ua {
    constructor(t = 0, e = 0, s = 0) {
      this.type = "circle", this.x = t, this.y = e, this.radius = s;
    }
    clone() {
      return new ua(this.x, this.y, this.radius);
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
      return t || (t = new Dt()), t.x = this.x - this.radius, t.y = this.y - this.radius, t.width = this.radius * 2, t.height = this.radius * 2, t;
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
  class pa {
    constructor(t = 0, e = 0, s = 0, i = 0) {
      this.type = "ellipse", this.x = t, this.y = e, this.halfWidth = s, this.halfHeight = i;
    }
    clone() {
      return new pa(this.x, this.y, this.halfWidth, this.halfHeight);
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
      return t || (t = new Dt()), t.x = this.x - this.halfWidth, t.y = this.y - this.halfHeight, t.width = this.halfWidth * 2, t.height = this.halfHeight * 2, t;
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
  function fm(n, t, e, s, i, r) {
    const o = n - e, a = t - s, l = i - e, c = r - s, h = o * l + a * c, d = l * l + c * c;
    let u = -1;
    d !== 0 && (u = h / d);
    let p, f;
    u < 0 ? (p = e, f = s) : u > 1 ? (p = i, f = r) : (p = e + u * l, f = s + u * c);
    const g = n - p, m = t - f;
    return g * g + m * m;
  }
  let mm, gm;
  class Zs {
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
      const e = this.getBounds(mm), s = t.getBounds(gm);
      if (!e.containsRect(s)) return false;
      const i = t.points;
      for (let r = 0; r < i.length; r += 2) {
        const o = i[r], a = i[r + 1];
        if (!this.contains(o, a)) return false;
      }
      return true;
    }
    clone() {
      const t = this.points.slice(), e = new Zs(t);
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
        const d = l[h], u = l[h + 1], p = l[(h + 2) % l.length], f = l[(h + 3) % l.length], g = fm(t, e, d, u, p, f), m = Math.sign((p - d) * (e - u) - (f - u) * (t - d));
        if (g <= (m < 0 ? a : o)) return true;
      }
      return false;
    }
    getBounds(t) {
      t || (t = new Dt());
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
  const $i = (n, t, e, s, i, r, o) => {
    const a = n - e, l = t - s, c = Math.sqrt(a * a + l * l);
    return c >= i - r && c <= i + o;
  };
  class fa {
    constructor(t = 0, e = 0, s = 0, i = 0, r = 20) {
      this.type = "roundedRectangle", this.x = t, this.y = e, this.width = s, this.height = i, this.radius = r;
    }
    getBounds(t) {
      return t || (t = new Dt()), t.x = this.x, t.y = this.y, t.width = this.width, t.height = this.height, t;
    }
    clone() {
      return new fa(this.x, this.y, this.width, this.height, this.radius);
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
      return (t >= r - h && t <= r + d || t >= m - d && t <= m + h) && e >= p && e <= p + g || (e >= o - h && e <= o + d || e >= y - d && e <= y + h) && t >= u && t <= u + f ? true : t < u && e < p && $i(t, e, u, p, c, d, h) || t > m - c && e < p && $i(t, e, m - c, p, c, d, h) || t > m - c && e > y - c && $i(t, e, m - c, y - c, c, d, h) || t < u && e > y - c && $i(t, e, u, y - c, c, d, h);
    }
    toString() {
      return `[pixi.js/math:RoundedRectangle x=${this.x} y=${this.y}width=${this.width} height=${this.height} radius=${this.radius}]`;
    }
  }
  const id = {};
  ym = function(n, t, e) {
    let s = 2166136261;
    for (let i = 0; i < t; i++) s ^= n[i].uid, s = Math.imul(s, 16777619), s >>>= 0;
    return id[s] || xm(n, t, s, e);
  };
  function xm(n, t, e, s) {
    const i = {};
    let r = 0;
    for (let a = 0; a < s; a++) {
      const l = a < t ? n[a] : Tt.EMPTY.source;
      i[r++] = l.source, i[r++] = l.style;
    }
    const o = new Ji(i);
    return id[e] = o, o;
  }
  class ms {
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
  vl = function(n, t, e, s) {
    if (e ?? (e = 0), s ?? (s = Math.min(n.byteLength - e, t.byteLength)), !(e & 7) && !(s & 7)) {
      const i = s / 8;
      new Float64Array(t, 0, i).set(new Float64Array(n, e, i));
    } else if (!(e & 3) && !(s & 3)) {
      const i = s / 4;
      new Float32Array(t, 0, i).set(new Float32Array(n, e, i));
    } else new Uint8Array(t).set(new Uint8Array(n, e, s));
  };
  const bm = {
    normal: "normal-npm",
    add: "add-npm",
    screen: "screen-npm"
  };
  _m = ((n) => (n[n.DISABLED = 0] = "DISABLED", n[n.RENDERING_MASK_ADD = 1] = "RENDERING_MASK_ADD", n[n.MASK_ACTIVE = 2] = "MASK_ACTIVE", n[n.INVERSE_MASK_ACTIVE = 3] = "INVERSE_MASK_ACTIVE", n[n.RENDERING_MASK_REMOVE = 4] = "RENDERING_MASK_REMOVE", n[n.NONE = 5] = "NONE", n))(_m || {});
  function Bo(n, t) {
    return t.alphaMode === "no-premultiply-alpha" && bm[n] || n;
  }
  const wm = [
    "precision mediump float;",
    "void main(void){",
    "float test = 0.1;",
    "%forloop%",
    "gl_FragColor = vec4(0.0);",
    "}"
  ].join(`
`);
  function vm(n) {
    let t = "";
    for (let e = 0; e < n; ++e) e > 0 && (t += `
else `), e < n - 1 && (t += `if(test == ${e}.0){}`);
    return t;
  }
  Cm = function(n, t) {
    if (n === 0) throw new Error("Invalid value of `0` passed to `checkMaxIfStatementsInShader`");
    const e = t.createShader(t.FRAGMENT_SHADER);
    try {
      for (; ; ) {
        const s = wm.replace(/%forloop%/gi, vm(n));
        if (t.shaderSource(e, s), t.compileShader(e), !t.getShaderParameter(e, t.COMPILE_STATUS)) n = n / 2 | 0;
        else break;
      }
    } finally {
      t.deleteShader(e);
    }
    return n;
  };
  let os = null;
  function Sm() {
    var _a2;
    if (os) return os;
    const n = Nh();
    return os = n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS), os = Cm(os, n), (_a2 = n.getExtension("WEBGL_lose_context")) == null ? void 0 : _a2.loseContext(), os;
  }
  class Tm {
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
  class Em {
    constructor() {
      this.renderPipeId = "batch", this.action = "startBatch", this.start = 0, this.size = 0, this.textures = new Tm(), this.blendMode = "normal", this.topology = "triangle-strip", this.canBundle = true;
    }
    destroy() {
      this.textures = null, this.gpuBindGroup = null, this.bindGroup = null, this.batcher = null, this.elements = null;
    }
  }
  const Qs = [];
  let ir = 0;
  mi.register({
    clear: () => {
      if (Qs.length > 0) for (const n of Qs) n && n.destroy();
      Qs.length = 0, ir = 0;
    }
  });
  function Cl() {
    return ir > 0 ? Qs[--ir] : new Em();
  }
  function Sl(n) {
    n.elements = null, Qs[ir++] = n;
  }
  let Ws = 0;
  const rd = class od {
    constructor(t) {
      this.uid = ne("batcher"), this.dirty = true, this.batchIndex = 0, this.batches = [], this._elements = [], t = {
        ...od.defaultOptions,
        ...t
      }, t.maxTextures || (It("v8.8.0", "maxTextures is a required option for Batcher now, please pass it in the options"), t.maxTextures = Sm());
      const { maxTextures: e, attributesInitialSize: s, indicesInitialSize: i } = t;
      this.attributeBuffer = new ms(s * 4), this.indexBuffer = new Uint16Array(i), this.maxTextures = e;
    }
    begin() {
      this.elementSize = 0, this.elementStart = 0, this.indexSize = 0, this.attributeSize = 0;
      for (let t = 0; t < this.batchIndex; t++) Sl(this.batches[t]);
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
      let s = Cl(), i = s.textures;
      i.clear();
      const r = e[this.elementStart];
      let o = Bo(r.blendMode, r.texture._source), a = r.topology;
      this.attributeSize * 4 > this.attributeBuffer.size && this._resizeAttributeBuffer(this.attributeSize * 4), this.indexSize > this.indexBuffer.length && this._resizeIndexBuffer(this.indexSize);
      const l = this.attributeBuffer.float32View, c = this.attributeBuffer.uint32View, h = this.indexBuffer;
      let d = this._batchIndexSize, u = this._batchIndexStart, p = "startBatch", f = [];
      const g = this.maxTextures;
      for (let m = this.elementStart; m < this.elementSize; ++m) {
        const y = e[m];
        e[m] = null;
        const x = y.texture._source, _ = Bo(y.blendMode, x), v = o !== _ || a !== y.topology;
        if (x._batchTick === Ws && !v) {
          y._textureId = x._textureBindLocation, d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize)), y._batch = s, f.push(y);
          continue;
        }
        x._batchTick = Ws, (i.count >= g || v) && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), p = "renderBatch", u = d, o = _, a = y.topology, s = Cl(), i = s.textures, i.clear(), f = [], ++Ws), y._textureId = x._textureBindLocation = i.count, i.ids[x.uid] = i.count, i.textures[i.count++] = x, y._batch = s, f.push(y), d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize));
      }
      i.count > 0 && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), u = d, ++Ws), this.elementStart = this.elementSize, this._batchIndexStart = u, this._batchIndexSize = d;
    }
    _finishBatch(t, e, s, i, r, o, a, l, c) {
      t.gpuBindGroup = null, t.bindGroup = null, t.action = l, t.batcher = this, t.textures = i, t.blendMode = r, t.topology = o, t.start = e, t.size = s, t.elements = c, ++Ws, this.batches[this.batchIndex++] = t, a.add(t);
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
      const e = Math.max(t, this.attributeBuffer.size * 2), s = new ms(e);
      vl(this.attributeBuffer.rawBinaryData, s.rawBinaryData), this.attributeBuffer = s;
    }
    _resizeIndexBuffer(t) {
      const e = this.indexBuffer;
      let s = Math.max(t, e.length * 1.5);
      s += s % 2;
      const i = s > 65535 ? new Uint32Array(s) : new Uint16Array(s);
      if (i.BYTES_PER_ELEMENT !== e.BYTES_PER_ELEMENT) for (let r = 0; r < e.length; r++) i[r] = e[r];
      else vl(e.buffer, i.buffer);
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
        for (let e = 0; e < this.batchIndex; e++) Sl(this.batches[e]);
        this.batches = null, this.geometry.destroy(true), this.geometry = null, t.shader && ((_a2 = this.shader) == null ? void 0 : _a2.destroy(), this.shader = null);
        for (let e = 0; e < this._elements.length; e++) this._elements[e] && (this._elements[e]._batch = null);
        this._elements = null, this.indexBuffer = null, this.attributeBuffer.destroy(), this.attributeBuffer = null;
      }
    }
  };
  rd.defaultOptions = {
    maxTextures: null,
    attributesInitialSize: 4,
    indicesInitialSize: 6
  };
  let Am = rd;
  he = ((n) => (n[n.MAP_READ = 1] = "MAP_READ", n[n.MAP_WRITE = 2] = "MAP_WRITE", n[n.COPY_SRC = 4] = "COPY_SRC", n[n.COPY_DST = 8] = "COPY_DST", n[n.INDEX = 16] = "INDEX", n[n.VERTEX = 32] = "VERTEX", n[n.UNIFORM = 64] = "UNIFORM", n[n.STORAGE = 128] = "STORAGE", n[n.INDIRECT = 256] = "INDIRECT", n[n.QUERY_RESOLVE = 512] = "QUERY_RESOLVE", n[n.STATIC = 1024] = "STATIC", n))(he || {});
  ts = class extends on {
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
      return !!(this.descriptor.usage & he.STATIC);
    }
    set static(t) {
      t ? this.descriptor.usage |= he.STATIC : this.descriptor.usage &= ~he.STATIC;
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
  function ad(n, t) {
    if (!(n instanceof ts)) {
      let e = t ? he.INDEX : he.VERTEX;
      n instanceof Array && (t ? (n = new Uint32Array(n), e = he.INDEX | he.COPY_DST) : (n = new Float32Array(n), e = he.VERTEX | he.COPY_DST)), n = new ts({
        data: n,
        label: t ? "index-mesh-buffer" : "vertex-mesh-buffer",
        usage: e
      });
    }
    return n;
  }
  function km(n, t, e) {
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
  function Mm(n) {
    return (n instanceof ts || Array.isArray(n) || n.BYTES_PER_ELEMENT) && (n = {
      buffer: n
    }), n.buffer = ad(n.buffer, false), n;
  }
  ld = class extends on {
    constructor(t = {}) {
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ne("geometry"), this._layoutKey = 0, this.instanceCount = 1, this._bounds = new Ie(), this._boundsDirty = true;
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
      const s = Mm(e);
      this.buffers.indexOf(s.buffer) === -1 && (this.buffers.push(s.buffer), s.buffer.on("update", this.onBufferUpdate, this), s.buffer.on("change", this.onBufferUpdate, this)), this.attributes[t] = s;
    }
    addIndex(t) {
      this.indexBuffer = ad(t, true), this.buffers.push(this.indexBuffer);
    }
    get bounds() {
      return this._boundsDirty ? (this._boundsDirty = false, km(this, "aPosition", this._bounds)) : this._bounds;
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
  const Pm = new Float32Array(1), Im = new Uint32Array(1);
  class Rm extends ld {
    constructor() {
      const e = new ts({
        data: Pm,
        label: "attribute-batch-buffer",
        usage: he.VERTEX | he.COPY_DST,
        shrinkToFit: false
      }), s = new ts({
        data: Im,
        label: "index-batch-buffer",
        usage: he.INDEX | he.COPY_DST,
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
  function Tl(n, t, e) {
    if (n) for (const s in n) {
      const i = s.toLocaleLowerCase(), r = t[i];
      if (r) {
        let o = n[s];
        s === "header" && (o = o.replace(/@in\s+[^;]+;\s*/g, "").replace(/@out\s+[^;]+;\s*/g, "")), e && r.push(`//----${e}----//`), r.push(o);
      } else Jt(`${s} placement hook does not exist in shader`);
    }
  }
  const Lm = /\{\{(.*?)\}\}/g;
  function El(n) {
    var _a2;
    const t = {};
    return (((_a2 = n.match(Lm)) == null ? void 0 : _a2.map((s) => s.replace(/[{()}]/g, ""))) ?? []).forEach((s) => {
      t[s] = [];
    }), t;
  }
  function Al(n, t) {
    let e;
    const s = /@in\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function kl(n, t, e = false) {
    const s = [];
    Al(t, s), n.forEach((a) => {
      a.header && Al(a.header, s);
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
  function Ml(n, t) {
    let e;
    const s = /@out\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function $m(n) {
    const e = /\b(\w+)\s*:/g.exec(n);
    return e ? e[1] : "";
  }
  function Bm(n) {
    const t = /@.*?\s+/g;
    return n.replace(t, "");
  }
  function Om(n, t) {
    const e = [];
    Ml(t, e), n.forEach((l) => {
      l.header && Ml(l.header, e);
    });
    let s = 0;
    const i = e.sort().map((l) => l.indexOf("builtin") > -1 ? l : `@location(${s++}) ${l}`).join(`,
`), r = e.sort().map((l) => `       var ${Bm(l)};`).join(`
`), o = `return VSOutput(
            ${e.sort().map((l) => ` ${$m(l)}`).join(`,
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
  function Pl(n, t) {
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
  const Mn = /* @__PURE__ */ Object.create(null), Jr = /* @__PURE__ */ new Map();
  let Nm = 0;
  function Fm({ template: n, bits: t }) {
    const e = cd(n, t);
    if (Mn[e]) return Mn[e];
    const { vertex: s, fragment: i } = Gm(n, t);
    return Mn[e] = hd(s, i, t), Mn[e];
  }
  function Wm({ template: n, bits: t }) {
    const e = cd(n, t);
    return Mn[e] || (Mn[e] = hd(n.vertex, n.fragment, t)), Mn[e];
  }
  function Gm(n, t) {
    const e = t.map((o) => o.vertex).filter((o) => !!o), s = t.map((o) => o.fragment).filter((o) => !!o);
    let i = kl(e, n.vertex, true);
    i = Om(e, i);
    const r = kl(s, n.fragment, true);
    return {
      vertex: i,
      fragment: r
    };
  }
  function cd(n, t) {
    return t.map((e) => (Jr.has(e) || Jr.set(e, Nm++), Jr.get(e))).sort((e, s) => e - s).join("-") + n.vertex + n.fragment;
  }
  function hd(n, t, e) {
    const s = El(n), i = El(t);
    return e.forEach((r) => {
      Tl(r.vertex, s, r.name), Tl(r.fragment, i, r.name);
    }), {
      vertex: Pl(n, s),
      fragment: Pl(t, i)
    };
  }
  const zm = `
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
`, Dm = `
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
`, Hm = `
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
`, Um = `

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
`, jm = {
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
  }, Ym = {
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
  Vm = function({ bits: n, name: t }) {
    const e = Fm({
      template: {
        fragment: Dm,
        vertex: zm
      },
      bits: [
        jm,
        ...n
      ]
    });
    return gi.from({
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
  Xm = function({ bits: n, name: t }) {
    return new ca({
      name: t,
      ...Wm({
        template: {
          vertex: Hm,
          fragment: Um
        },
        bits: [
          Ym,
          ...n
        ]
      })
    });
  };
  let Zr;
  qm = {
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
  Km = {
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
  Zr = {};
  function Jm(n) {
    const t = [];
    if (n === 1) t.push("@group(1) @binding(0) var textureSource1: texture_2d<f32>;"), t.push("@group(1) @binding(1) var textureSampler1: sampler;");
    else {
      let e = 0;
      for (let s = 0; s < n; s++) t.push(`@group(1) @binding(${e++}) var textureSource${s + 1}: texture_2d<f32>;`), t.push(`@group(1) @binding(${e++}) var textureSampler${s + 1}: sampler;`);
    }
    return t.join(`
`);
  }
  function Zm(n) {
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
  Qm = function(n) {
    return Zr[n] || (Zr[n] = {
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

                ${Jm(n)}
            `,
        main: `
                var uvDx = dpdx(vUV);
                var uvDy = dpdy(vUV);

                ${Zm(n)}
            `
      }
    }), Zr[n];
  };
  const Qr = {};
  function tg(n) {
    const t = [];
    for (let e = 0; e < n; e++) e > 0 && t.push("else"), e < n - 1 && t.push(`if(vTextureId < ${e}.5)`), t.push("{"), t.push(`	outColor = texture(uTextures[${e}], vUV);`), t.push("}");
    return t.join(`
`);
  }
  eg = function(n) {
    return Qr[n] || (Qr[n] = {
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

                ${tg(n)}
            `
      }
    }), Qr[n];
  };
  let Il;
  ng = {
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
  sg = {
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
  Il = {};
  ig = function(n) {
    let t = Il[n];
    if (t) return t;
    const e = new Int32Array(n);
    for (let s = 0; s < n; s++) e[s] = s;
    return t = Il[n] = new ha({
      uTextures: {
        value: e,
        type: "i32",
        size: n
      }
    }, {
      isStatic: true
    }), t;
  };
  class Rl extends gr {
    constructor(t) {
      const e = Xm({
        name: "batch",
        bits: [
          Km,
          eg(t),
          sg
        ]
      }), s = Vm({
        name: "batch",
        bits: [
          qm,
          Qm(t),
          ng
        ]
      });
      super({
        glProgram: e,
        gpuProgram: s,
        resources: {
          batchSamplers: ig(t)
        }
      }), this.maxTextures = t;
    }
  }
  let Gs = null;
  const dd = class ud extends Am {
    constructor(t) {
      super(t), this.geometry = new Rm(), this.name = ud.extension.name, this.vertexSize = 6, Gs ?? (Gs = new Rl(t.maxTextures)), this.shader = Gs;
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
      this.shader.maxTextures !== t && (Gs = new Rl(t), this.shader = Gs);
    }
    destroy() {
      this.shader = null, super.destroy();
    }
  };
  dd.extension = {
    type: [
      lt.Batcher
    ],
    name: "default"
  };
  rg = dd;
  Is = class {
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
  function og(n, t, e, s, i, r, o, a = null) {
    let l = 0;
    e *= t, i *= r;
    const c = a.a, h = a.b, d = a.c, u = a.d, p = a.tx, f = a.ty;
    for (; l < o; ) {
      const g = n[e], m = n[e + 1];
      s[i] = c * g + d * m + p, s[i + 1] = h * g + u * m + f, i += r, e += t, l++;
    }
  }
  function ag(n, t, e, s) {
    let i = 0;
    for (t *= e; i < s; ) n[t] = 0, n[t + 1] = 0, t += e, i++;
  }
  function pd(n, t, e, s, i) {
    const r = t.a, o = t.b, a = t.c, l = t.d, c = t.tx, h = t.ty;
    e || (e = 0), s || (s = 2), i || (i = n.length / s - e);
    let d = e * s;
    for (let u = 0; u < i; u++) {
      const p = n[d], f = n[d + 1];
      n[d] = r * p + a * f + c, n[d + 1] = o * p + l * f + h, d += s;
    }
  }
  const lg = new vt();
  class ma {
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
      return s ? bh(e, s.groupColor) + (this.alpha * s.groupAlpha * 255 << 24) : e + (this.alpha * 255 << 24);
    }
    get transform() {
      var _a2;
      return ((_a2 = this.renderable) == null ? void 0 : _a2.groupTransform) || lg;
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
  const li = {
    extension: {
      type: lt.ShapeBuilder,
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
        const v = Math.PI / 2 * (_ / l), w = i + Math.cos(v) * o, C = r + Math.sin(v) * a, T = e + w, P = e - w, E = s + C, A = s - C;
        t[h++] = T, t[h++] = E, t[--d] = E, t[--d] = P, t[u++] = P, t[u++] = A, t[--p] = A, t[--p] = T;
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
  }, cg = {
    ...li,
    extension: {
      ...li.extension,
      name: "ellipse"
    }
  }, hg = {
    ...li,
    extension: {
      ...li.extension,
      name: "roundedRectangle"
    }
  }, fd = 1e-4, Ll = 1e-4;
  function dg(n) {
    const t = n.length;
    if (t < 6) return 1;
    let e = 0;
    for (let s = 0, i = n[t - 2], r = n[t - 1]; s < t; s += 2) {
      const o = n[s], a = n[s + 1];
      e += (o - i) * (a + r), i = o, r = a;
    }
    return e < 0 ? -1 : 1;
  }
  function $l(n, t, e, s, i, r, o, a) {
    const l = n - e * i, c = t - s * i, h = n + e * r, d = t + s * r;
    let u, p;
    o ? (u = s, p = -e) : (u = -s, p = e);
    const f = l + u, g = c + p, m = h + u, y = d + p;
    return a.push(f, g), a.push(m, y), 2;
  }
  function Fn(n, t, e, s, i, r, o, a) {
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
  ug = function(n, t, e, s, i, r) {
    const o = fd;
    if (n.length === 0) return;
    const a = t;
    let l = a.alignment;
    if (t.alignment !== 0.5) {
      let U = dg(n);
      l = (l - 0.5) * U + 0.5;
    }
    const c = new Mt(n[0], n[1]), h = new Mt(n[n.length - 2], n[n.length - 1]), d = s, u = Math.abs(c.x - h.x) < o && Math.abs(c.y - h.y) < o;
    if (d) {
      n = n.slice(), u && (n.pop(), n.pop(), h.set(n[n.length - 2], n[n.length - 1]));
      const U = (c.x + h.x) * 0.5, Z = (h.y + c.y) * 0.5;
      n.unshift(U, Z), n.push(U, Z);
    }
    const p = i, f = n.length / 2;
    let g = n.length;
    const m = p.length / 2, y = a.width / 2, b = y * y, x = a.miterLimit * a.miterLimit;
    let _ = n[0], v = n[1], w = n[2], C = n[3], T = 0, P = 0, E = -(v - C), A = _ - w, D = 0, O = 0, H = Math.sqrt(E * E + A * A);
    E /= H, A /= H, E *= y, A *= y;
    const G = l, L = (1 - G) * 2, $ = G * 2;
    d || (a.cap === "round" ? g += Fn(_ - E * (L - $) * 0.5, v - A * (L - $) * 0.5, _ - E * L, v - A * L, _ + E * $, v + A * $, p, true) + 2 : a.cap === "square" && (g += $l(_, v, E, A, L, $, true, p))), p.push(_ - E * L, v - A * L), p.push(_ + E * $, v + A * $);
    for (let U = 1; U < f - 1; ++U) {
      _ = n[(U - 1) * 2], v = n[(U - 1) * 2 + 1], w = n[U * 2], C = n[U * 2 + 1], T = n[(U + 1) * 2], P = n[(U + 1) * 2 + 1], E = -(v - C), A = _ - w, H = Math.sqrt(E * E + A * A), E /= H, A /= H, E *= y, A *= y, D = -(C - P), O = w - T, H = Math.sqrt(D * D + O * O), D /= H, O /= H, D *= y, O *= y;
      const Z = w - _, M = v - C, B = w - T, N = P - C, j = Z * B + M * N, J = M * B - N * Z, st = J < 0;
      if (Math.abs(J) < 1e-3 * Math.abs(j)) {
        p.push(w - E * L, C - A * L), p.push(w + E * $, C + A * $), j >= 0 && (a.join === "round" ? g += Fn(w, C, w - E * L, C - A * L, w - D * L, C - O * L, p, false) + 4 : g += 2, p.push(w - D * $, C - O * $), p.push(w + D * L, C + O * L));
        continue;
      }
      const ht = (-E + _) * (-A + C) - (-E + w) * (-A + v), ut = (-D + T) * (-O + C) - (-D + w) * (-O + P), mt = (Z * ut - B * ht) / J, bt = (N * ht - M * ut) / J, X = (mt - w) * (mt - w) + (bt - C) * (bt - C), it = w + (mt - w) * L, rt = C + (bt - C) * L, et = w - (mt - w) * $, St = C - (bt - C) * $, Lt = Math.min(Z * Z + M * M, B * B + N * N), Ht = st ? L : $, Vt = Lt + Ht * Ht * b;
      X <= Vt ? a.join === "bevel" || X / b > x ? (st ? (p.push(it, rt), p.push(w + E * $, C + A * $), p.push(it, rt), p.push(w + D * $, C + O * $)) : (p.push(w - E * L, C - A * L), p.push(et, St), p.push(w - D * L, C - O * L), p.push(et, St)), g += 2) : a.join === "round" ? st ? (p.push(it, rt), p.push(w + E * $, C + A * $), g += Fn(w, C, w + E * $, C + A * $, w + D * $, C + O * $, p, true) + 4, p.push(it, rt), p.push(w + D * $, C + O * $)) : (p.push(w - E * L, C - A * L), p.push(et, St), g += Fn(w, C, w - E * L, C - A * L, w - D * L, C - O * L, p, false) + 4, p.push(w - D * L, C - O * L), p.push(et, St)) : (p.push(it, rt), p.push(et, St)) : (p.push(w - E * L, C - A * L), p.push(w + E * $, C + A * $), a.join === "round" ? st ? g += Fn(w, C, w + E * $, C + A * $, w + D * $, C + O * $, p, true) + 2 : g += Fn(w, C, w - E * L, C - A * L, w - D * L, C - O * L, p, false) + 2 : a.join === "miter" && X / b <= x && (st ? (p.push(et, St), p.push(et, St)) : (p.push(it, rt), p.push(it, rt)), g += 2), p.push(w - D * L, C - O * L), p.push(w + D * $, C + O * $), g += 2);
    }
    _ = n[(f - 2) * 2], v = n[(f - 2) * 2 + 1], w = n[(f - 1) * 2], C = n[(f - 1) * 2 + 1], E = -(v - C), A = _ - w, H = Math.sqrt(E * E + A * A), E /= H, A /= H, E *= y, A *= y, p.push(w - E * L, C - A * L), p.push(w + E * $, C + A * $), d || (a.cap === "round" ? g += Fn(w - E * (L - $) * 0.5, C - A * (L - $) * 0.5, w - E * L, C - A * L, w + E * $, C + A * $, p, false) + 2 : a.cap === "square" && (g += $l(w, C, E, A, L, $, false, p)));
    const K = Ll * Ll;
    for (let U = m; U < g + m - 2; ++U) _ = p[U * 2], v = p[U * 2 + 1], w = p[(U + 1) * 2], C = p[(U + 1) * 2 + 1], T = p[(U + 2) * 2], P = p[(U + 2) * 2 + 1], !(Math.abs(_ * (C - P) + w * (P - v) + T * (v - C)) < K) && r.push(U, U + 1, U + 2);
  };
  function pg(n, t, e, s) {
    const i = fd;
    if (n.length === 0) return;
    const r = n[0], o = n[1], a = n[n.length - 2], l = n[n.length - 1], c = t || Math.abs(r - a) < i && Math.abs(o - l) < i, h = e, d = n.length / 2, u = h.length / 2;
    for (let p = 0; p < d; p++) h.push(n[p * 2]), h.push(n[p * 2 + 1]);
    for (let p = 0; p < d - 1; p++) s.push(u + p, u + p + 1);
    c && s.push(u + d - 1, u);
  }
  function md(n, t, e, s, i, r, o) {
    const a = Pf(n, t, 2);
    if (!a) return;
    for (let c = 0; c < a.length; c += 3) r[o++] = a[c] + i, r[o++] = a[c + 1] + i, r[o++] = a[c + 2] + i;
    let l = i * s;
    for (let c = 0; c < n.length; c += 2) e[l] = n[c], e[l + 1] = n[c + 1], l += s;
  }
  const fg = [], mg = {
    extension: {
      type: lt.ShapeBuilder,
      name: "polygon"
    },
    build(n, t) {
      for (let e = 0; e < n.points.length; e++) t[e] = n.points[e];
      return true;
    },
    triangulate(n, t, e, s, i, r) {
      md(n, fg, t, e, s, i, r);
    }
  }, gg = {
    extension: {
      type: lt.ShapeBuilder,
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
  }, yg = {
    extension: {
      type: lt.ShapeBuilder,
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
  }, Bl = [
    {
      offset: 0,
      color: "white"
    },
    {
      offset: 1,
      color: "black"
    }
  ], ga = class Oo {
    constructor(...t) {
      this.uid = ne("fillGradient"), this._tick = 0, this.type = "linear", this.colorStops = [];
      let e = xg(t);
      e = {
        ...e.type === "radial" ? Oo.defaultRadialOptions : Oo.defaultLinearOptions,
        ...lh(e)
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
          const m = t;
          t = s, s = m, r *= -1;
        }
        if (o < 0) {
          const m = e;
          e = i, i = m, o *= -1;
        }
      }
      const l = this.colorStops.length ? this.colorStops : Bl, c = this._textureSize, { canvas: h, context: d } = Nl(c, 1), u = a ? d.createLinearGradient(this._textureSize, 0, 0, 0) : d.createLinearGradient(0, 0, this._textureSize, 0);
      Ol(u, l), d.fillStyle = u, d.fillRect(0, 0, c, 1), this.texture = new Tt({
        source: new Ss({
          resource: h,
          addressMode: this._wrapMode
        })
      });
      const p = Math.sqrt(r * r + o * o), f = Math.atan2(o, r), g = new vt();
      g.scale(p / c, 1), g.rotate(f), g.translate(t, e), this.textureSpace === "local" && g.scale(c, c), this.transform = g;
    }
    buildGradient() {
      this.texture || this._tick++, this.type === "linear" ? this.buildLinearGradient() : this.buildRadialGradient();
    }
    buildRadialGradient() {
      if (this.texture) return;
      const t = this.colorStops.length ? this.colorStops : Bl, e = this._textureSize, { canvas: s, context: i } = Nl(e, e), { x: r, y: o } = this.center, { x: a, y: l } = this.outerCenter, c = this.innerRadius, h = this.outerRadius, d = a - h, u = l - h, p = e / (h * 2), f = (r - d) * p, g = (o - u) * p, m = i.createRadialGradient(f, g, c * p, (a - d) * p, (l - u) * p, h * p);
      Ol(m, t), i.fillStyle = t[t.length - 1].color, i.fillRect(0, 0, e, e), i.fillStyle = m, i.translate(f, g), i.rotate(this.rotation), i.scale(1, this.scale), i.translate(-f, -g), i.fillRect(0, 0, e, e), this.texture = new Tt({
        source: new Ss({
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
  ga.defaultLinearOptions = {
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
  ga.defaultRadialOptions = {
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
  _n = ga;
  function Ol(n, t) {
    for (let e = 0; e < t.length; e++) {
      const s = t[e];
      n.addColorStop(s.offset, s.color);
    }
  }
  function Nl(n, t) {
    const e = Rt.get().createCanvas(n, t), s = e.getContext("2d");
    return {
      canvas: e,
      context: s
    };
  }
  function xg(n) {
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
      textureSize: n[5] ?? _n.defaultLinearOptions.textureSize
    }), t;
  }
  const bg = new vt(), _g = new Dt();
  wg = function(n, t, e, s) {
    const i = t.matrix ? n.copyFrom(t.matrix).invert() : n.identity();
    if (t.textureSpace === "local") {
      const o = e.getBounds(_g);
      t.width && o.pad(t.width);
      const { x: a, y: l } = o, c = 1 / o.width, h = 1 / o.height, d = -a * c, u = -l * h, p = i.a, f = i.b, g = i.c, m = i.d;
      i.a *= c, i.b *= c, i.c *= h, i.d *= h, i.tx = d * p + u * g + i.tx, i.ty = d * f + u * m + i.ty;
    } else i.translate(t.texture.frame.x, t.texture.frame.y), i.scale(1 / t.texture.source.width, 1 / t.texture.source.height);
    const r = t.texture.source.style;
    return !(t.fill instanceof _n) && r.addressMode === "clamp-to-edge" && (r.addressMode = "repeat", r.update()), s && i.append(bg.copyFrom(s).invert()), i;
  };
  xr = {};
  Wt.handleByMap(lt.ShapeBuilder, xr);
  Wt.add(gg, mg, yg, li, cg, hg);
  const vg = new Dt(), Cg = new vt();
  function Sg(n, t) {
    const { geometryData: e, batches: s } = t;
    s.length = 0, e.indices.length = 0, e.vertices.length = 0, e.uvs.length = 0;
    for (let i = 0; i < n.instructions.length; i++) {
      const r = n.instructions[i];
      if (r.action === "texture") Tg(r.data, s, e);
      else if (r.action === "fill" || r.action === "stroke") {
        const o = r.action === "stroke", a = r.data.path.shapePath, l = r.data.style, c = r.data.hole;
        o && c && Fl(c.shapePath, l, true, s, e), c && (a.shapePrimitives[a.shapePrimitives.length - 1].holes = c.shapePath.shapePrimitives), Fl(a, l, o, s, e);
      }
    }
  }
  function Tg(n, t, e) {
    const s = [], i = xr.rectangle, r = vg;
    r.x = n.dx, r.y = n.dy, r.width = n.dw, r.height = n.dh;
    const o = n.transform;
    if (!i.build(r, s)) return;
    const { vertices: a, uvs: l, indices: c } = e, h = c.length, d = a.length / 2;
    o && pd(s, o), i.triangulate(s, a, 2, d, c, h);
    const u = n.image, p = u.uvs;
    l.push(p.x0, p.y0, p.x1, p.y1, p.x3, p.y3, p.x2, p.y2);
    const f = Te.get(ma);
    f.indexOffset = h, f.indexSize = c.length - h, f.attributeOffset = d, f.attributeSize = a.length / 2 - d, f.baseColor = n.style, f.alpha = n.alpha, f.texture = u, f.geometryData = e, t.push(f);
  }
  function Fl(n, t, e, s, i) {
    const { vertices: r, uvs: o, indices: a } = i;
    n.shapePrimitives.forEach(({ shape: l, transform: c, holes: h }) => {
      const d = [], u = xr[l.type];
      if (!u.build(l, d)) return;
      const p = a.length, f = r.length / 2;
      let g = "triangle-list";
      if (c && pd(d, c), e) {
        const x = l.closePath ?? true, _ = t;
        _.pixelLine ? (pg(d, x, r, a), g = "line-list") : ug(d, _, false, x, r, a);
      } else if (h) {
        const x = [], _ = d.slice();
        Eg(h).forEach((w) => {
          x.push(_.length / 2), _.push(...w);
        }), md(_, x, r, 2, f, a, p);
      } else u.triangulate(d, r, 2, f, a, p);
      const m = o.length / 2, y = t.texture;
      if (y !== Tt.WHITE) {
        const x = wg(Cg, t, l, c);
        og(r, 2, f, o, m, 2, r.length / 2 - f, x);
      } else ag(o, m, 2, r.length / 2 - f);
      const b = Te.get(ma);
      b.indexOffset = p, b.indexSize = a.length - p, b.attributeOffset = f, b.attributeSize = r.length / 2 - f, b.baseColor = t.color, b.alpha = t.alpha, b.texture = y, b.geometryData = i, b.topology = g, s.push(b);
    });
  }
  function Eg(n) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e].shape, i = [];
      xr[s.type].build(s, i) && t.push(i);
    }
    return t;
  }
  class Ag {
    constructor() {
      this.batches = [], this.geometryData = {
        vertices: [],
        uvs: [],
        indices: []
      };
    }
    reset() {
      this.batches && this.batches.forEach((t) => {
        Te.return(t);
      }), this.graphicsData && Te.return(this.graphicsData), this.isBatchable = false, this.context = null, this.batches.length = 0, this.geometryData.indices.length = 0, this.geometryData.vertices.length = 0, this.geometryData.uvs.length = 0, this.graphicsData = null;
    }
    destroy() {
      this.reset(), this.batches = null, this.geometryData = null;
    }
  }
  class kg {
    constructor() {
      this.instructions = new aa();
    }
    init(t) {
      const e = t.maxTextures;
      this.batcher ? this.batcher._updateMaxTextures(e) : this.batcher = new rg({
        maxTextures: e
      }), this.instructions.reset();
    }
    get geometry() {
      return It(ep, "GraphicsContextRenderData#geometry is deprecated, please use batcher.geometry instead."), this.batcher.geometry;
    }
    destroy() {
      this.batcher.destroy(), this.instructions.destroy(), this.batcher = null, this.instructions = null;
    }
  }
  const ya = class No {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Is({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      No.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? No.defaultOptions.bezierSmoothness;
    }
    getContextRenderData(t) {
      return t._gpuData[this._renderer.uid].graphicsData || this._initContextRenderData(t);
    }
    updateGpuContext(t) {
      const e = !!t._gpuData[this._renderer.uid], s = t._gpuData[this._renderer.uid] || this._initContext(t);
      if (t.dirty || !e) {
        e && s.reset(), Sg(t, s);
        const i = t.batchMode;
        t.customShader || i === "no-batch" ? s.isBatchable = false : i === "auto" ? s.isBatchable = s.geometryData.vertices.length < 400 : s.isBatchable = true, t.dirty = false;
      }
      return s;
    }
    getGpuContext(t) {
      return t._gpuData[this._renderer.uid] || this._initContext(t);
    }
    _initContextRenderData(t) {
      const e = Te.get(kg, {
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
        u.bindGroup = ym(u.textures.textures, u.textures.count, this._renderer.limits.maxBatchableTextures);
      }
      return e;
    }
    _initContext(t) {
      const e = new Ag();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  ya.extension = {
    type: [
      lt.WebGLSystem,
      lt.WebGPUSystem
    ],
    name: "graphicsContext"
  };
  ya.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let xa = ya;
  const Mg = 8, Bi = 11920929e-14, Pg = 1;
  function gd(n, t, e, s, i, r, o, a, l, c) {
    const d = Math.min(0.99, Math.max(0, c ?? xa.defaultOptions.bezierSmoothness));
    let u = (Pg - d) / 1;
    return u *= u, Ig(t, e, s, i, r, o, a, l, n, u), n;
  }
  function Ig(n, t, e, s, i, r, o, a, l, c) {
    Fo(n, t, e, s, i, r, o, a, l, c, 0), l.push(o, a);
  }
  function Fo(n, t, e, s, i, r, o, a, l, c, h) {
    if (h > Mg) return;
    const d = (n + e) / 2, u = (t + s) / 2, p = (e + i) / 2, f = (s + r) / 2, g = (i + o) / 2, m = (r + a) / 2, y = (d + p) / 2, b = (u + f) / 2, x = (p + g) / 2, _ = (f + m) / 2, v = (y + x) / 2, w = (b + _) / 2;
    if (h > 0) {
      let C = o - n, T = a - t;
      const P = Math.abs((e - o) * T - (s - a) * C), E = Math.abs((i - o) * T - (r - a) * C);
      if (P > Bi && E > Bi) {
        if ((P + E) * (P + E) <= c * (C * C + T * T)) {
          l.push(v, w);
          return;
        }
      } else if (P > Bi) {
        if (P * P <= c * (C * C + T * T)) {
          l.push(v, w);
          return;
        }
      } else if (E > Bi) {
        if (E * E <= c * (C * C + T * T)) {
          l.push(v, w);
          return;
        }
      } else if (C = v - (n + o) / 2, T = w - (t + a) / 2, C * C + T * T <= c) {
        l.push(v, w);
        return;
      }
    }
    Fo(n, t, d, u, y, b, v, w, l, c, h + 1), Fo(v, w, x, _, g, m, o, a, l, c, h + 1);
  }
  const Rg = 8, Lg = 11920929e-14, $g = 1;
  function Bg(n, t, e, s, i, r, o, a) {
    const c = Math.min(0.99, Math.max(0, a ?? xa.defaultOptions.bezierSmoothness));
    let h = ($g - c) / 1;
    return h *= h, Og(t, e, s, i, r, o, n, h), n;
  }
  function Og(n, t, e, s, i, r, o, a) {
    Wo(o, n, t, e, s, i, r, a, 0), o.push(i, r);
  }
  function Wo(n, t, e, s, i, r, o, a, l) {
    if (l > Rg) return;
    const c = (t + s) / 2, h = (e + i) / 2, d = (s + r) / 2, u = (i + o) / 2, p = (c + d) / 2, f = (h + u) / 2;
    let g = r - t, m = o - e;
    const y = Math.abs((s - r) * m - (i - o) * g);
    if (y > Lg) {
      if (y * y <= a * (g * g + m * m)) {
        n.push(p, f);
        return;
      }
    } else if (g = p - (t + r) / 2, m = f - (e + o) / 2, g * g + m * m <= a) {
      n.push(p, f);
      return;
    }
    Wo(n, t, e, c, h, p, f, a, l + 1), Wo(n, p, f, d, u, r, o, a, l + 1);
  }
  function yd(n, t, e, s, i, r, o, a) {
    let l = Math.abs(i - r);
    (!o && i > r || o && r > i) && (l = 2 * Math.PI - l), a || (a = Math.max(6, Math.floor(6 * Math.pow(s, 1 / 3) * (l / Math.PI)))), a = Math.max(a, 3);
    let c = l / a, h = i;
    c *= o ? -1 : 1;
    for (let d = 0; d < a + 1; d++) {
      const u = Math.cos(h), p = Math.sin(h), f = t + u * s, g = e + p * s;
      n.push(f, g), h += c;
    }
  }
  function Ng(n, t, e, s, i, r) {
    const o = n[n.length - 2], l = n[n.length - 1] - e, c = o - t, h = i - e, d = s - t, u = Math.abs(l * d - c * h);
    if (u < 1e-8 || r === 0) {
      (n[n.length - 2] !== t || n[n.length - 1] !== e) && n.push(t, e);
      return;
    }
    const p = l * l + c * c, f = h * h + d * d, g = l * h + c * d, m = r * Math.sqrt(p) / u, y = r * Math.sqrt(f) / u, b = m * g / p, x = y * g / f, _ = m * d + y * c, v = m * h + y * l, w = c * (y + b), C = l * (y + b), T = d * (m + x), P = h * (m + x), E = Math.atan2(C - v, w - _), A = Math.atan2(P - v, T - _);
    yd(n, _ + t, v + e, r, E, A, c * h > d * l);
  }
  const ti = Math.PI * 2, to = {
    centerX: 0,
    centerY: 0,
    ang1: 0,
    ang2: 0
  }, eo = ({ x: n, y: t }, e, s, i, r, o, a, l) => {
    n *= e, t *= s;
    const c = i * n - r * t, h = r * n + i * t;
    return l.x = c + o, l.y = h + a, l;
  };
  function Fg(n, t) {
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
  const Wl = (n, t, e, s) => {
    const i = n * s - t * e < 0 ? -1 : 1;
    let r = n * e + t * s;
    return r > 1 && (r = 1), r < -1 && (r = -1), i * Math.acos(r);
  }, Wg = (n, t, e, s, i, r, o, a, l, c, h, d, u) => {
    const p = Math.pow(i, 2), f = Math.pow(r, 2), g = Math.pow(h, 2), m = Math.pow(d, 2);
    let y = p * f - p * m - f * g;
    y < 0 && (y = 0), y /= p * m + f * g, y = Math.sqrt(y) * (o === a ? -1 : 1);
    const b = y * i / r * d, x = y * -r / i * h, _ = c * b - l * x + (n + e) / 2, v = l * b + c * x + (t + s) / 2, w = (h - b) / i, C = (d - x) / r, T = (-h - b) / i, P = (-d - x) / r, E = Wl(1, 0, w, C);
    let A = Wl(w, C, T, P);
    a === 0 && A > 0 && (A -= ti), a === 1 && A < 0 && (A += ti), u.centerX = _, u.centerY = v, u.ang1 = E, u.ang2 = A;
  };
  function Gg(n, t, e, s, i, r, o, a = 0, l = 0, c = 0) {
    if (r === 0 || o === 0) return;
    const h = Math.sin(a * ti / 360), d = Math.cos(a * ti / 360), u = d * (t - s) / 2 + h * (e - i) / 2, p = -h * (t - s) / 2 + d * (e - i) / 2;
    if (u === 0 && p === 0) return;
    r = Math.abs(r), o = Math.abs(o);
    const f = Math.pow(u, 2) / Math.pow(r, 2) + Math.pow(p, 2) / Math.pow(o, 2);
    f > 1 && (r *= Math.sqrt(f), o *= Math.sqrt(f)), Wg(t, e, s, i, r, o, l, c, h, d, u, p, to);
    let { ang1: g, ang2: m } = to;
    const { centerX: y, centerY: b } = to;
    let x = Math.abs(m) / (ti / 4);
    Math.abs(1 - x) < 1e-7 && (x = 1);
    const _ = Math.max(Math.ceil(x), 1);
    m /= _;
    let v = n[n.length - 2], w = n[n.length - 1];
    const C = {
      x: 0,
      y: 0
    };
    for (let T = 0; T < _; T++) {
      const P = Fg(g, m), { x: E, y: A } = eo(P[0], r, o, d, h, y, b, C), { x: D, y: O } = eo(P[1], r, o, d, h, y, b, C), { x: H, y: G } = eo(P[2], r, o, d, h, y, b, C);
      gd(n, v, w, E, A, D, O, H, G), v = H, w = G, g += m;
    }
  }
  function zg(n, t, e) {
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
  function Dg(n, t, e, s) {
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
  const Hg = new Dt();
  class Ug {
    constructor(t) {
      this.shapePrimitives = [], this._currentPoly = null, this._bounds = new Ie(), this._graphicsPath2D = t, this.signed = t.checkForHoles;
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
      return yd(a, t, e, s, i, r, o), this;
    }
    arcTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly.points;
      return Ng(o, t, e, s, i, r), this;
    }
    arcToSvg(t, e, s, i, r, o, a) {
      const l = this._currentPoly.points;
      return Gg(l, this._currentPoly.lastX, this._currentPoly.lastY, o, a, t, e, s, i, r), this;
    }
    bezierCurveTo(t, e, s, i, r, o, a) {
      this._ensurePoly();
      const l = this._currentPoly;
      return gd(this._currentPoly.points, l.lastX, l.lastY, t, e, s, i, r, o, a), this;
    }
    quadraticCurveTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly;
      return Bg(this._currentPoly.points, o.lastX, o.lastY, t, e, s, i, r), this;
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
      return this.drawShape(new Dt(t, e, s, i), r), this;
    }
    circle(t, e, s, i) {
      return this.drawShape(new ua(t, e, s), i), this;
    }
    poly(t, e, s) {
      const i = new Zs(t);
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
      return t.length < 3 ? this : (s ? Dg(this, t, e, i) : zg(this, t, e), this.closePath());
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
      return this.drawShape(new pa(t, e, s, i), r), this;
    }
    roundRect(t, e, s, i, r, o) {
      return this.drawShape(new fa(t, e, s, i, r), o), this;
    }
    drawShape(t, e) {
      return this.endPoly(), this.shapePrimitives.push({
        shape: t,
        transform: e
      }), this;
    }
    startPoly(t, e) {
      let s = this._currentPoly;
      return s && this.endPoly(), s = new Zs(), s.points.push(t, e), this._currentPoly = s, this;
    }
    endPoly(t = false) {
      const e = this._currentPoly;
      return e && e.points.length > 2 && (e.closePath = t, this.shapePrimitives.push({
        shape: e
      })), this._currentPoly = null, this;
    }
    _ensurePoly(t = true) {
      if (!this._currentPoly && (this._currentPoly = new Zs(), t)) {
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
        const i = e[s], r = i.shape.getBounds(Hg);
        i.transform ? t.addRect(r, i.transform) : t.addRect(r);
      }
      return t;
    }
  }
  class xn {
    constructor(t, e = false) {
      this.instructions = [], this.uid = ne("graphicsPath"), this._dirty = true, this.checkForHoles = e, typeof t == "string" ? pm(t, this) : this.instructions = (t == null ? void 0 : t.slice()) ?? [];
    }
    get shapePath() {
      return this._shapePath || (this._shapePath = new Ug(this)), this._dirty && (this._dirty = false, this._shapePath.buildPath()), this._shapePath;
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
      const o = this.instructions[this.instructions.length - 1], a = this.getLastPoint(Mt.shared);
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
      const i = this.instructions[this.instructions.length - 1], r = this.getLastPoint(Mt.shared);
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
      const e = new xn();
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
            b[4] = zs(b[3], t);
            break;
          case "rect":
            b[4] = zs(b[4], t);
            break;
          case "ellipse":
            b[8] = zs(b[8], t);
            break;
          case "roundRect":
            b[5] = zs(b[5], t);
            break;
          case "addPath":
            b[0].transform(t);
            break;
          case "poly":
            b[2] = zs(b[2], t);
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
  function zs(n, t) {
    return n ? n.prepend(t) : t.clone();
  }
  function te(n, t, e) {
    const s = n.getAttribute(t);
    return s ? Number(s) : e;
  }
  function jg(n, t) {
    const e = n.querySelectorAll("defs");
    for (let s = 0; s < e.length; s++) {
      const i = e[s];
      for (let r = 0; r < i.children.length; r++) {
        const o = i.children[r];
        switch (o.nodeName.toLowerCase()) {
          case "lineargradient":
            t.defs[o.id] = Yg(o);
            break;
          case "radialgradient":
            t.defs[o.id] = Vg();
            break;
        }
      }
    }
  }
  function Yg(n) {
    const t = te(n, "x1", 0), e = te(n, "y1", 0), s = te(n, "x2", 1), i = te(n, "y2", 0), r = n.getAttribute("gradientUnits") || "objectBoundingBox", o = new _n(t, e, s, i, r === "objectBoundingBox" ? "local" : "global");
    for (let a = 0; a < n.children.length; a++) {
      const l = n.children[a], c = te(l, "offset", 0), h = Yt.shared.setValue(l.getAttribute("stop-color")).toNumber();
      o.addColorStop(c, h);
    }
    return o;
  }
  function Vg(n) {
    return Jt("[SVG Parser] Radial gradients are not yet supported"), new _n(0, 0, 1, 0);
  }
  function Gl(n) {
    const t = n.match(/url\s*\(\s*['"]?\s*#([^'"\s)]+)\s*['"]?\s*\)/i);
    return t ? t[1] : "";
  }
  const zl = {
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
  function xd(n, t) {
    const e = n.getAttribute("style"), s = {}, i = {}, r = {
      strokeStyle: s,
      fillStyle: i,
      useFill: false,
      useStroke: false
    };
    for (const o in zl) {
      const a = n.getAttribute(o);
      a && Dl(t, r, o, a.trim());
    }
    if (e) {
      const o = e.split(";");
      for (let a = 0; a < o.length; a++) {
        const l = o[a].trim(), [c, h] = l.split(":");
        zl[c] && Dl(t, r, c, h.trim());
      }
    }
    return {
      strokeStyle: r.useStroke ? s : null,
      fillStyle: r.useFill ? i : null,
      useFill: r.useFill,
      useStroke: r.useStroke
    };
  }
  function Dl(n, t, e, s) {
    switch (e) {
      case "stroke":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Gl(s);
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
            const i = Gl(s);
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
  function Xg(n) {
    if (n.length <= 2) return true;
    const t = n.map((a) => a.area).sort((a, l) => l - a), [e, s] = t, i = t[t.length - 1], r = e / s, o = s / i;
    return !(r > 3 && o < 2);
  }
  function qg(n) {
    return n.split(/(?=[Mm])/).filter((s) => s.trim().length > 0);
  }
  function Kg(n) {
    const t = n.match(/[-+]?[0-9]*\.?[0-9]+/g);
    if (!t || t.length < 4) return 0;
    const e = t.map(Number), s = [], i = [];
    for (let h = 0; h < e.length; h += 2) h + 1 < e.length && (s.push(e[h]), i.push(e[h + 1]));
    if (s.length === 0 || i.length === 0) return 0;
    const r = Math.min(...s), o = Math.max(...s), a = Math.min(...i), l = Math.max(...i);
    return (o - r) * (l - a);
  }
  function Hl(n, t) {
    const e = new xn(n, false);
    for (const s of e.instructions) t.instructions.push(s);
  }
  function Jg(n, t) {
    if (typeof n == "string") {
      const o = document.createElement("div");
      o.innerHTML = n.trim(), n = o.querySelector("svg");
    }
    const e = {
      context: t,
      defs: {},
      path: new xn()
    };
    jg(n, e);
    const s = n.children, { fillStyle: i, strokeStyle: r } = xd(n, e);
    for (let o = 0; o < s.length; o++) {
      const a = s[o];
      a.nodeName.toLowerCase() !== "defs" && bd(a, e, i, r);
    }
    return t;
  }
  function bd(n, t, e, s) {
    const i = n.children, { fillStyle: r, strokeStyle: o } = xd(n, t);
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
    let l, c, h, d, u, p, f, g, m, y, b, x, _, v, w, C, T;
    switch (n.nodeName.toLowerCase()) {
      case "path": {
        v = n.getAttribute("d");
        const P = n.getAttribute("fill-rule"), E = qg(v), A = P === "evenodd", D = E.length > 1;
        if (A && D) {
          const H = E.map((L) => ({
            path: L,
            area: Kg(L)
          }));
          if (H.sort((L, $) => $.area - L.area), E.length > 3 || !Xg(H)) for (let L = 0; L < H.length; L++) {
            const $ = H[L], K = L === 0;
            t.context.beginPath();
            const U = new xn(void 0, true);
            Hl($.path, U), t.context.path(U), K ? (e && t.context.fill(e), s && t.context.stroke(s)) : t.context.cut();
          }
          else for (let L = 0; L < H.length; L++) {
            const $ = H[L], K = L % 2 === 1;
            t.context.beginPath();
            const U = new xn(void 0, true);
            Hl($.path, U), t.context.path(U), K ? t.context.cut() : (e && t.context.fill(e), s && t.context.stroke(s));
          }
        } else {
          const H = P ? P === "evenodd" : true;
          w = new xn(v, H), t.context.path(w), e && t.context.fill(e), s && t.context.stroke(s);
        }
        break;
      }
      case "circle":
        f = te(n, "cx", 0), g = te(n, "cy", 0), m = te(n, "r", 0), t.context.ellipse(f, g, m, m), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "rect":
        l = te(n, "x", 0), c = te(n, "y", 0), C = te(n, "width", 0), T = te(n, "height", 0), y = te(n, "rx", 0), b = te(n, "ry", 0), y || b ? t.context.roundRect(l, c, C, T, y || b) : t.context.rect(l, c, C, T), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "ellipse":
        f = te(n, "cx", 0), g = te(n, "cy", 0), y = te(n, "rx", 0), b = te(n, "ry", 0), t.context.beginPath(), t.context.ellipse(f, g, y, b), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "line":
        h = te(n, "x1", 0), d = te(n, "y1", 0), u = te(n, "x2", 0), p = te(n, "y2", 0), t.context.beginPath(), t.context.moveTo(h, d), t.context.lineTo(u, p), s && t.context.stroke(s);
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
        Jt(`[SVG parser] <${n.nodeName}> elements unsupported`);
        break;
      }
    }
    a && (e = null);
    for (let P = 0; P < i.length; P++) bd(i[P], t, e, s);
  }
  const Ul = {
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
  br = class {
    constructor(t, e) {
      this.uid = ne("fillPattern"), this._tick = 0, this.transform = new vt(), this.texture = t, this.transform.scale(1 / t.frame.width, 1 / t.frame.height), e && (t.source.style.addressModeU = Ul[e].addressModeU, t.source.style.addressModeV = Ul[e].addressModeV);
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
  function Zg(n) {
    return Yt.isColorLike(n);
  }
  function jl(n) {
    return n instanceof br;
  }
  function Yl(n) {
    return n instanceof _n;
  }
  function Qg(n) {
    return n instanceof Tt;
  }
  function ty(n, t, e) {
    const s = Yt.shared.setValue(t ?? 0);
    return n.color = s.toNumber(), n.alpha = s.alpha === 1 ? e.alpha : s.alpha, n.texture = Tt.WHITE, {
      ...e,
      ...n
    };
  }
  function ey(n, t, e) {
    return n.texture = t, {
      ...e,
      ...n
    };
  }
  function Vl(n, t, e) {
    return n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, {
      ...e,
      ...n
    };
  }
  function Xl(n, t, e) {
    return t.buildGradient(), n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, n.textureSpace = t.textureSpace, {
      ...e,
      ...n
    };
  }
  function ny(n, t) {
    const e = {
      ...t,
      ...n
    }, s = Yt.shared.setValue(e.color);
    return e.alpha *= s.alpha, e.color = s.toNumber(), e;
  }
  function Kn(n, t) {
    if (n == null) return null;
    const e = {}, s = n;
    return Zg(n) ? ty(e, n, t) : Qg(n) ? ey(e, n, t) : jl(n) ? Vl(e, n, t) : Yl(n) ? Xl(e, n, t) : s.fill && jl(s.fill) ? Vl(s, s.fill, t) : s.fill && Yl(s.fill) ? Xl(s, s.fill, t) : ny(s, t);
  }
  function rr(n, t) {
    const { width: e, alignment: s, miterLimit: i, cap: r, join: o, pixelLine: a, ...l } = t, c = Kn(n, l);
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
  function sy(n, t) {
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
        let E = (b * _ + x * v) / Math.sqrt(w * C);
        E < -1 ? E = -1 : E > 1 && (E = 1);
        const A = Math.sqrt((1 - E) * 0.5);
        if (A < 1e-6) continue;
        const D = Math.min(1 / A, t);
        D > e && (e = D);
      }
    }
    return e;
  }
  const iy = new Mt(), ql = new vt(), ba = class Qe extends on {
    constructor() {
      super(...arguments), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ne("graphicsContext"), this.dirty = true, this.batchMode = "auto", this.instructions = [], this.destroyed = false, this._activePath = new xn(), this._transform = new vt(), this._fillStyle = {
        ...Qe.defaultFillStyle
      }, this._strokeStyle = {
        ...Qe.defaultStrokeStyle
      }, this._stateStack = [], this._tick = 0, this._bounds = new Ie(), this._boundsDirty = true;
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
      this._fillStyle = Kn(t, Qe.defaultFillStyle);
    }
    get strokeStyle() {
      return this._strokeStyle;
    }
    set strokeStyle(t) {
      this._strokeStyle = rr(t, Qe.defaultStrokeStyle);
    }
    setFillStyle(t) {
      return this._fillStyle = Kn(t, Qe.defaultFillStyle), this;
    }
    setStrokeStyle(t) {
      return this._strokeStyle = Kn(t, Qe.defaultStrokeStyle), this;
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
      return this._activePath = new xn(), this;
    }
    fill(t, e) {
      let s;
      const i = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (i == null ? void 0 : i.action) === "stroke" ? s = i.data.path : s = this._activePath.clone(), s ? (t != null && (e !== void 0 && typeof t == "number" && (It(ee, "GraphicsContext.fill(color, alpha) is deprecated, use GraphicsContext.fill({ color, alpha }) instead"), t = {
        color: t,
        alpha: e
      }), this._fillStyle = Kn(t, Qe.defaultFillStyle)), this.instructions.push({
        action: "fill",
        data: {
          style: this.fillStyle,
          path: s
        }
      }), this.onUpdate(), this._initNextPathLocation(), this._tick = 0, this) : this;
    }
    _initNextPathLocation() {
      const { x: t, y: e } = this._activePath.getLastPoint(Mt.shared);
      this._activePath.clear(), this._activePath.moveTo(t, e);
    }
    stroke(t) {
      let e;
      const s = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (s == null ? void 0 : s.action) === "fill" ? e = s.data.path : e = this._activePath.clone(), e ? (t != null && (this._strokeStyle = rr(t, Qe.defaultStrokeStyle)), this.instructions.push({
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
      return this._tick++, Jg(t, this), this;
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
      return t instanceof vt ? (this._transform.append(t), this) : (ql.set(t, e, s, i, r, o), this._transform.append(ql), this);
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
          r.style.join === "miter" && (a *= sy(r.path, r.style.miterLimit));
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
          const u = c[h].transform, p = u ? u.applyInverse(t, iy) : t;
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
  ba.defaultFillStyle = {
    color: 16777215,
    alpha: 1,
    texture: Tt.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local"
  };
  ba.defaultStrokeStyle = {
    width: 1,
    color: 16777215,
    alpha: 1,
    alignment: 0.5,
    miterLimit: 10,
    cap: "butt",
    join: "miter",
    texture: Tt.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local",
    pixelLine: false
  };
  let $e = ba;
  function _a(n, t = 1) {
    var _a2;
    const e = (_a2 = ks.RETINA_PREFIX) == null ? void 0 : _a2.exec(n);
    return e ? parseFloat(e[1]) : t;
  }
  function wa(n, t, e) {
    n.label = e, n._sourceOrigin = e;
    const s = new Tt({
      source: n,
      label: e
    }), i = () => {
      delete t.promiseCache[e], ie.has(e) && ie.remove(e);
    };
    return s.source.once("destroy", () => {
      t.promiseCache[e] && (Jt("[Assets] A TextureSource managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the TextureSource."), i());
    }), s.once("destroy", () => {
      n.destroyed || (Jt("[Assets] A Texture managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the Texture."), i());
    }), s;
  }
  const ry = ".svg", oy = "image/svg+xml", ay = {
    extension: {
      type: lt.LoadParser,
      priority: Bn.Low,
      name: "loadSVG"
    },
    name: "loadSVG",
    id: "svg",
    config: {
      crossOrigin: "anonymous",
      parseAsGraphicsContext: false
    },
    test(n) {
      return Ms(n, oy) || Ps(n, ry);
    },
    async load(n, t, e) {
      var _a2;
      return ((_a2 = t.data) == null ? void 0 : _a2.parseAsGraphicsContext) ?? this.config.parseAsGraphicsContext ? cy(n) : ly(n, t, e, this.config.crossOrigin);
    },
    unload(n) {
      n.destroy(true);
    }
  };
  async function ly(n, t, e, s) {
    var _a2, _b2, _c2;
    const i = await Rt.get().fetch(n), r = Rt.get().createImage();
    r.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(await i.text())}`, r.crossOrigin = s, await r.decode();
    const o = ((_a2 = t.data) == null ? void 0 : _a2.width) ?? r.width, a = ((_b2 = t.data) == null ? void 0 : _b2.height) ?? r.height, l = ((_c2 = t.data) == null ? void 0 : _c2.resolution) || _a(n), c = Math.ceil(o * l), h = Math.ceil(a * l), d = Rt.get().createCanvas(c, h), u = d.getContext("2d");
    u.imageSmoothingEnabled = true, u.imageSmoothingQuality = "high", u.drawImage(r, 0, 0, o * l, a * l);
    const { parseAsGraphicsContext: p, ...f } = t.data ?? {}, g = new Ss({
      resource: d,
      alphaMode: "premultiply-alpha-on-upload",
      resolution: l,
      ...f
    });
    return wa(g, e, n);
  }
  async function cy(n) {
    const e = await (await Rt.get().fetch(n)).text(), s = new $e();
    return s.svg(e), s;
  }
  const hy = `(function () {
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
  let xs = null, Go = class {
    constructor() {
      xs || (xs = URL.createObjectURL(new Blob([
        hy
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(xs);
    }
  };
  Go.revokeObjectURL = function() {
    xs && (URL.revokeObjectURL(xs), xs = null);
  };
  const dy = `(function () {
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
  let bs = null;
  class _d {
    constructor() {
      bs || (bs = URL.createObjectURL(new Blob([
        dy
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(bs);
    }
  }
  _d.revokeObjectURL = function() {
    bs && (URL.revokeObjectURL(bs), bs = null);
  };
  let Kl = 0, no;
  class uy {
    constructor() {
      this._initialized = false, this._createdWorkers = 0, this._workerPool = [], this._queue = [], this._resolveHash = {};
    }
    isImageBitmapSupported() {
      return this._isImageBitmapSupported !== void 0 ? this._isImageBitmapSupported : (this._isImageBitmapSupported = new Promise((t) => {
        const { worker: e } = new Go();
        e.addEventListener("message", (s) => {
          e.terminate(), Go.revokeObjectURL(), t(s.data);
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
      no === void 0 && (no = navigator.hardwareConcurrency || 4);
      let t = this._workerPool.pop();
      return !t && this._createdWorkers < no && (this._createdWorkers++, t = new _d().worker, t.addEventListener("message", (e) => {
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
      this._resolveHash[Kl] = {
        resolve: e.resolve,
        reject: e.reject
      }, t.postMessage({
        data: e.arguments,
        uuid: Kl++,
        id: s
      });
    }
    reset() {
      this._workerPool.forEach((t) => t.terminate()), this._workerPool.length = 0, Object.values(this._resolveHash).forEach(({ reject: t }) => {
        t == null ? void 0 : t(new Error("WorkerManager has been reset before completion"));
      }), this._resolveHash = {}, this._queue.length = 0, this._initialized = false, this._createdWorkers = 0;
    }
  }
  const Jl = new uy(), py = [
    ".jpeg",
    ".jpg",
    ".png",
    ".webp",
    ".avif"
  ], fy = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/avif"
  ];
  async function my(n, t) {
    var _a2;
    const e = await Rt.get().fetch(n);
    if (!e.ok) throw new Error(`[loadImageBitmap] Failed to fetch ${n}: ${e.status} ${e.statusText}`);
    const s = await e.blob();
    return ((_a2 = t == null ? void 0 : t.data) == null ? void 0 : _a2.alphaMode) === "premultiplied-alpha" ? createImageBitmap(s, {
      premultiplyAlpha: "none"
    }) : createImageBitmap(s);
  }
  const wd = {
    name: "loadTextures",
    id: "texture",
    extension: {
      type: lt.LoadParser,
      priority: Bn.High,
      name: "loadTextures"
    },
    config: {
      preferWorkers: true,
      preferCreateImageBitmap: true,
      crossOrigin: "anonymous"
    },
    test(n) {
      return Ms(n, fy) || Ps(n, py);
    },
    async load(n, t, e) {
      var _a2;
      let s = null;
      globalThis.createImageBitmap && this.config.preferCreateImageBitmap ? this.config.preferWorkers && await Jl.isImageBitmapSupported() ? s = await Jl.loadImageBitmap(n, t) : s = await my(n, t) : s = await new Promise((r, o) => {
        s = Rt.get().createImage(), s.crossOrigin = this.config.crossOrigin, s.src = n, s.complete ? r(s) : (s.onload = () => {
          r(s);
        }, s.onerror = o);
      });
      const i = new Ss({
        resource: s,
        alphaMode: "premultiply-alpha-on-upload",
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || _a(n),
        ...t.data
      });
      return wa(i, e, n);
    },
    unload(n) {
      n.destroy(true);
    }
  }, gy = [
    ".mp4",
    ".m4v",
    ".webm",
    ".ogg",
    ".ogv",
    ".h264",
    ".avi",
    ".mov"
  ];
  let so, io;
  function yy(n, t, e) {
    e === void 0 && !t.startsWith("data:") ? n.crossOrigin = by(t) : e !== false && (n.crossOrigin = typeof e == "string" ? e : "anonymous");
  }
  function xy(n) {
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
  function by(n, t = globalThis.location) {
    if (n.startsWith("data:")) return "";
    t || (t = globalThis.location);
    const e = new URL(n, document.baseURI);
    return e.hostname !== t.hostname || e.port !== t.port || e.protocol !== t.protocol ? "anonymous" : "";
  }
  function _y() {
    const n = [], t = [];
    for (const e of gy) {
      const s = Js.MIME_TYPES[e.substring(1)] || `video/${e.substring(1)}`;
      yr(s) && (n.push(e), t.includes(s) || t.push(s));
    }
    return {
      validVideoExtensions: n,
      validVideoMime: t
    };
  }
  const wy = {
    name: "loadVideo",
    id: "video",
    extension: {
      type: lt.LoadParser,
      name: "loadVideo"
    },
    test(n) {
      if (!so || !io) {
        const { validVideoExtensions: s, validVideoMime: i } = _y();
        so = s, io = i;
      }
      const t = Ms(n, io), e = Ps(n, so);
      return t || e;
    },
    async load(n, t, e) {
      var _a2, _b2;
      const s = {
        ...Js.defaultOptions,
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || _a(n),
        alphaMode: ((_b2 = t.data) == null ? void 0 : _b2.alphaMode) || await Ih(),
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
      }), s.muted === true && (i.muted = true), yy(i, n, s.crossorigin);
      const o = document.createElement("source");
      let a;
      if (s.mime) a = s.mime;
      else if (n.startsWith("data:")) a = n.slice(5, n.indexOf(";"));
      else if (!n.startsWith("blob:")) {
        const l = n.split("?")[0].slice(n.lastIndexOf(".") + 1).toLowerCase();
        a = Js.MIME_TYPES[l] || `video/${l}`;
      }
      return o.src = n, a && (o.type = a), new Promise((l, c) => {
        s.preload && !s.autoPlay && i.load(), i.addEventListener("canplay", h), i.addEventListener("error", d), o.addEventListener("error", d), i.appendChild(o);
        async function h() {
          const p = new Js({
            ...s,
            resource: i
          });
          u(), t.data.preload && await xy(i), l(wa(p, e, n));
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
  }, vd = {
    extension: {
      type: lt.ResolveParser,
      name: "resolveTexture"
    },
    test: wd.test,
    parse: (n) => {
      var _a2;
      return {
        resolution: parseFloat(((_a2 = ks.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
        format: n.split(".").pop(),
        src: n
      };
    }
  }, vy = {
    extension: {
      type: lt.ResolveParser,
      priority: -2,
      name: "resolveJson"
    },
    test: (n) => ks.RETINA_PREFIX.test(n) && n.endsWith(".json"),
    parse: vd.parse
  };
  class Cy {
    constructor() {
      this._detections = [], this._initialized = false, this.resolver = new ks(), this.loader = new qf(), this.cache = ie, this._backgroundLoader = new Gf(this.loader), this._backgroundLoader.active = true, this.reset();
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
      const s = er(t), i = Ye(t).map((a) => {
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
      if (typeof t == "string") return ie.get(t);
      const e = {};
      for (let s = 0; s < t.length; s++) e[s] = ie.get(t[s]);
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
        }), ie.set(l, a);
      }), r;
    }
    async unload(t) {
      this._initialized || await this.init();
      const e = Ye(t).map((i) => typeof i != "string" ? i.src : i), s = this.resolver.resolve(e);
      await this._unloadFromResolved(s);
    }
    async unloadBundle(t) {
      this._initialized || await this.init(), t = Ye(t);
      const e = this.resolver.resolveBundle(t), s = Object.keys(e).map((i) => this._unloadFromResolved(e[i]));
      await Promise.all(s);
    }
    async _unloadFromResolved(t) {
      const e = Object.values(t);
      e.forEach((s) => {
        ie.remove(s.src);
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
  const Oe = new Cy();
  Wt.handleByList(lt.LoadParser, Oe.loader.parsers).handleByList(lt.ResolveParser, Oe.resolver.parsers).handleByList(lt.CacheParser, Oe.cache.parsers).handleByList(lt.DetectionParser, Oe.detections);
  Wt.add(zf, Hf, Df, Xf, jf, Yf, Vf, Zf, em, cm, ay, wd, wy, Wf, Ff, vd, vy);
  const Zl = {
    loader: lt.LoadParser,
    resolver: lt.ResolveParser,
    cache: lt.CacheParser,
    detection: lt.DetectionParser
  };
  Wt.handle(lt.Asset, (n) => {
    const t = n.ref;
    Object.entries(Zl).filter(([e]) => !!t[e]).forEach(([e, s]) => Wt.add(Object.assign(t[e], {
      extension: t[e].extension ?? s
    })));
  }, (n) => {
    const t = n.ref;
    Object.keys(Zl).filter((e) => !!t[e]).forEach((e) => Wt.remove(t[e]));
  });
  class Sy {
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
  class Ty {
    constructor() {
      this.instructions = new aa();
    }
    init() {
      this.instructions.reset();
    }
    destroy() {
      this.instructions.destroy(), this.instructions = null;
    }
  }
  const va = class zo {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Is({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      zo.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? zo.defaultOptions.bezierSmoothness;
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
      const e = new Ty(), s = this.getGpuContext(t);
      return s.graphicsData = e, e.init(), e;
    }
    _initContext(t) {
      const e = new Sy();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  va.extension = {
    type: [
      lt.CanvasSystem
    ],
    name: "graphicsContext"
  };
  va.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let Ey = va;
  class Cd {
    constructor(t, e) {
      this.state = sr.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Is({
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
  Cd.extension = {
    type: [
      lt.CanvasPipes
    ],
    name: "graphics"
  };
  Sd = function(n, t, e) {
    const s = (n >> 24 & 255) / 255;
    t[e++] = (n & 255) / 255 * s, t[e++] = (n >> 8 & 255) / 255 * s, t[e++] = (n >> 16 & 255) / 255 * s, t[e++] = s;
  };
  class Ay {
    constructor() {
      this.batches = [], this.batched = false;
    }
    destroy() {
      this.batches.forEach((t) => {
        Te.return(t);
      }), this.batches.length = 0;
    }
  }
  class Td {
    constructor(t, e) {
      this.state = sr.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Is({
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
      o.uTransformMatrix = t.groupTransform, o.uRound = e._roundPixels | t._roundPixels, Sd(t.groupColorAlpha, o.uColor, 0), this._adaptor.execute(this, t);
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
      const e = new Ay();
      return t._gpuData[this.renderer.uid] = e, this._managedGraphics.add(t), e;
    }
    _updateBatchesForRenderable(t, e) {
      const s = t.context, r = this.renderer.graphicsContext.getGpuContext(s), o = this.renderer._roundPixels | t._roundPixels;
      e.batches = r.batches.map((a) => {
        const l = Te.get(ma);
        return a.copyTo(l), l.renderable = t, l.roundPixels = o, l;
      });
    }
    destroy() {
      this._managedGraphics.destroy(), this.renderer = null, this._adaptor.destroy(), this._adaptor = null, this.state = null;
    }
  }
  Td.extension = {
    type: [
      lt.WebGLPipes,
      lt.WebGPUPipes
    ],
    name: "graphics"
  };
  Wt.add(Cd);
  Wt.add(Td);
  Wt.add(Ey);
  Wt.add(xa);
  ft = class extends fr {
    constructor(t) {
      t instanceof $e && (t = {
        context: t
      });
      const { context: e, roundPixels: s, ...i } = t || {};
      super({
        label: "Graphics",
        ...i
      }), this.renderPipeId = "graphics", e ? this.context = e : (this.context = this._ownedContext = new $e(), this.context.autoGarbageCollect = this.autoGarbageCollect), this.didViewUpdate = true, this.allowChildren = false, this.roundPixels = s ?? false;
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
      return (t.width !== $e.defaultStrokeStyle.width || t.color !== $e.defaultStrokeStyle.color || t.alpha !== $e.defaultStrokeStyle.alpha) && this.context.stroke(), this;
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
  let as;
  function Ql(n) {
    const t = Rt.get().createCanvas(6, 1), e = t.getContext("2d");
    return e.fillStyle = n, e.fillRect(0, 0, 6, 1), t;
  }
  ky = function() {
    if (as !== void 0) return as;
    try {
      const n = Ql("#ff00ff"), t = Ql("#ffff00"), s = Rt.get().createCanvas(6, 1).getContext("2d");
      s.globalCompositeOperation = "multiply", s.drawImage(n, 0, 0), s.drawImage(t, 2, 0);
      const i = s.getImageData(2, 0, 1, 1);
      if (!i) as = false;
      else {
        const r = i.data;
        as = r[0] === 255 && r[1] === 0 && r[2] === 0;
      }
    } catch {
      as = false;
    }
    return as;
  };
  Kt = {
    canvas: null,
    convertTintToImage: false,
    cacheStepsPerColorChannel: 8,
    canUseMultiply: ky(),
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
      const a = At.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
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
      const a = At.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
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
      const a = At.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy";
      const h = Kt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && Kt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
      const d = t >> 16 & 255, u = t >> 8 & 255, p = t & 255, f = s.getImageData(0, 0, l, c), g = f.data;
      for (let m = 0; m < g.length; m += 4) g[m] = g[m] * d / 255, g[m + 1] = g[m + 1] * u / 255, g[m + 2] = g[m + 2] * p / 255;
      s.putImageData(f, 0, 0);
    },
    _applyInverseRotation: (n, t, e, s) => {
      const i = At.inv(t), r = At.uX(i), o = At.uY(i), a = At.vX(i), l = At.vY(i), c = -Math.min(0, r * e, a * s, r * e + a * s), h = -Math.min(0, o * e, l * s, o * e + l * s);
      n.transform(r, o, a, l, c, h);
    }
  };
  Kt.tintMethod = Kt.canUseMultiply ? Kt.tintWithMultiply : Kt.tintWithPerPixel;
  class My extends fr {
    constructor(t, e) {
      const { text: s, resolution: i, style: r, anchor: o, width: a, height: l, roundPixels: c, ...h } = t;
      super({
        ...h
      }), this.batched = true, this._resolution = null, this._autoResolution = true, this._didTextUpdate = true, this._styleClass = e, this.text = s ?? "", this.style = r, this.resolution = i ?? null, this.allowChildren = false, this._anchor = new fe({
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
  function Py(n, t) {
    let e = n[0] ?? {};
    return (typeof e == "string" || n[1]) && (It(ee, `use new ${t}({ text: "hi!", style }) instead`), e = {
      text: e,
      style: n[1]
    }), e;
  }
  class Iy {
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
      t = Math.ceil(t * s - 1e-6), e = Math.ceil(e * s - 1e-6), t = vs(t), e = vs(e);
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
  Do = new Iy();
  mi.register(Do);
  let Wn = null, fn = null;
  function Ry(n, t) {
    Wn || (Wn = Rt.get().createCanvas(256, 128), fn = Wn.getContext("2d", {
      willReadFrequently: true
    }), fn.globalCompositeOperation = "copy", fn.globalAlpha = 1), (Wn.width < n || Wn.height < t) && (Wn.width = vs(n), Wn.height = vs(t));
  }
  function tc(n, t, e) {
    for (let s = 0, i = 4 * e * t; s < t; ++s, i += 4) if (n[i + 3] !== 0) return false;
    return true;
  }
  function ec(n, t, e, s, i) {
    const r = 4 * t;
    for (let o = s, a = s * r + 4 * e; o <= i; ++o, a += r) if (n[a + 3] !== 0) return false;
    return true;
  }
  function Ly(...n) {
    let t = n[0];
    t.canvas || (t = {
      canvas: n[0],
      resolution: n[1]
    });
    const { canvas: e } = t, s = Math.min(t.resolution ?? 1, 1), i = t.width ?? e.width, r = t.height ?? e.height;
    let o = t.output;
    if (Ry(i, r), !fn) throw new TypeError("Failed to get canvas 2D context");
    fn.drawImage(e, 0, 0, i, r, 0, 0, i * s, r * s);
    const l = fn.getImageData(0, 0, i, r).data;
    let c = 0, h = 0, d = i - 1, u = r - 1;
    for (; h < r && tc(l, i, h); ) ++h;
    if (h === r) return Dt.EMPTY;
    for (; tc(l, i, u); ) --u;
    for (; ec(l, i, c, h, u); ) ++c;
    for (; ec(l, i, d, h, u); ) --d;
    return ++d, ++u, fn.globalCompositeOperation = "source-over", fn.strokeRect(c, h, d - c, u - h), fn.globalCompositeOperation = "copy", o ?? (o = new Dt()), o.set(c / s, h / s, (d - c) / s, (u - h) / s), o;
  }
  class $y {
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
  By = function(n = 1e3, t = 0, e = false) {
    if (isNaN(n) || n < 0) throw new TypeError("Invalid max value");
    if (isNaN(t) || t < 0) throw new TypeError("Invalid ttl value");
    if (typeof e != "boolean") throw new TypeError("Invalid resetTtl value");
    return new $y(n, t, e);
  };
  function Ed(n) {
    return !!n.tagStyles && Object.keys(n.tagStyles).length > 0;
  }
  function Ad(n) {
    return n.includes("<");
  }
  function Oy(n, t) {
    return n.clone().assign(t);
  }
  function Ny(n, t) {
    const e = [], s = t.tagStyles;
    if (!Ed(t) || !Ad(n)) return e.push({
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
            const u = i[i.length - 1], p = Oy(u, s[d]);
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
  const Fy = [
    10,
    13
  ], Wy = new Set(Fy), Gy = [
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
  ], zy = new Set(Gy), Dy = [
    9,
    32
  ], Hy = new Set(Dy), Uy = [
    45,
    8208,
    8211,
    8212,
    173
  ], jy = new Set(Uy), Yy = /(\r\n|\r|\n)/, Vy = /(?:\r\n|\r|\n)/;
  function or(n) {
    return typeof n != "string" ? false : Wy.has(n.charCodeAt(0));
  }
  Ne = function(n, t) {
    return typeof n != "string" ? false : zy.has(n.charCodeAt(0));
  };
  Wv = function(n) {
    return typeof n != "string" ? false : Hy.has(n.charCodeAt(0));
  };
  Xy = function(n) {
    return typeof n != "string" ? false : jy.has(n.charCodeAt(0));
  };
  kd = function(n) {
    return n === "normal" || n === "pre-line";
  };
  Md = function(n) {
    return n === "normal";
  };
  function dn(n) {
    if (typeof n != "string") return "";
    let t = n.length - 1;
    for (; t >= 0 && Ne(n[t]); ) t--;
    return t < n.length - 1 ? n.slice(0, t + 1) : n;
  }
  function Pd(n) {
    const t = [], e = [];
    if (typeof n != "string") return t;
    for (let s = 0; s < n.length; s++) {
      const i = n[s], r = n[s + 1];
      if (Ne(i) || or(i)) {
        e.length > 0 && (t.push(e.join("")), e.length = 0), i === "\r" && r === `
` ? (t.push(`\r
`), s++) : t.push(i);
        continue;
      }
      e.push(i), Xy(i) && r && !Ne(r) && !or(r) && (t.push(e.join("")), e.length = 0);
    }
    return e.length > 0 && t.push(e.join("")), t;
  }
  function Id(n, t, e, s) {
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
  const qy = /\r\n|\r|\n/g;
  function Ky(n, t, e, s, i, r, o, a) {
    var _a2, _b2;
    const l = Ny(n, t);
    if (Md(t.whiteSpace)) for (let G = 0; G < l.length; G++) {
      const L = l[G];
      l[G] = {
        text: L.text.replace(qy, " "),
        style: L.style
      };
    }
    const h = [];
    let d = [];
    for (const G of l) {
      const L = G.text.split(Yy);
      for (let $ = 0; $ < L.length; $++) {
        const K = L[$];
        K === `\r
` || K === "\r" || K === `
` ? (h.push(d), d = []) : K.length > 0 && d.push({
          text: K,
          style: G.style
        });
      }
    }
    (d.length > 0 || h.length === 0) && h.push(d);
    const u = e ? Jy(h, t, s, i, o, a) : h, p = [], f = [], g = [], m = [], y = [];
    let b = 0;
    const x = t._fontString, _ = r(x);
    _.fontSize === 0 && (_.fontSize = t.fontSize, _.ascent = t.fontSize);
    let v = "", w = !!t.dropShadow, C = ((_a2 = t._stroke) == null ? void 0 : _a2.width) || 0;
    for (const G of u) {
      let L = 0, $ = _.ascent, K = _.descent, U = "";
      for (const M of G) {
        const B = M.style._fontString, N = r(B);
        B !== v && (s.font = B, v = B);
        const j = i(M.text, M.style.letterSpacing, s);
        L += j, $ = Math.max($, N.ascent), K = Math.max(K, N.descent), U += M.text;
        const J = ((_b2 = M.style._stroke) == null ? void 0 : _b2.width) || 0;
        J > C && (C = J), !w && M.style.dropShadow && (w = true);
      }
      G.length === 0 && ($ = _.ascent, K = _.descent), p.push(L), f.push($), g.push(K), y.push(U);
      const Z = t.lineHeight || $ + K;
      m.push(Z + t.leading), b = Math.max(b, L);
    }
    const T = C, A = (e && t.align !== "left" ? Math.max(b, t.wordWrapWidth) : b) + T + (t.dropShadow ? t.dropShadow.distance : 0);
    let D = 0;
    for (let G = 0; G < m.length; G++) D += m[G];
    D = Math.max(D, m[0] + T);
    const O = D + (t.dropShadow ? t.dropShadow.distance : 0), H = t.lineHeight || _.fontSize;
    return {
      width: A,
      height: O,
      lines: y,
      lineWidths: p,
      lineHeight: H + t.leading,
      maxLineWidth: b,
      fontProperties: _,
      runsByLine: u,
      lineAscents: f,
      lineDescents: g,
      lineHeights: m,
      hasDropShadow: w
    };
  }
  function Jy(n, t, e, s, i, r) {
    var _a2;
    const { letterSpacing: o, whiteSpace: a, wordWrapWidth: l, breakWords: c } = t, h = kd(a), d = l + o, u = {};
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
      const y = Zy(m), b = g.length, x = (A) => {
        let D = 0, O = A;
        do {
          const { token: H, style: G } = y[O];
          D += f(H, G), O++;
        } while (O < y.length && y[O].continuesFromPrevious);
        return D;
      }, _ = (A) => {
        const D = [];
        let O = A;
        do
          D.push({
            token: y[O].token,
            style: y[O].style
          }), O++;
        while (O < y.length && y[O].continuesFromPrevious);
        return D;
      };
      let v = [], w = 0, C = !h, T = null;
      const P = () => {
        T && T.text.length > 0 && v.push(T), T = null;
      }, E = () => {
        if (P(), v.length > 0) {
          const A = v[v.length - 1];
          A.text = dn(A.text), A.text.length === 0 && v.pop();
        }
        g.push(v), v = [], w = 0, C = false;
      };
      for (let A = 0; A < y.length; A++) {
        const { token: D, style: O, continuesFromPrevious: H } = y[A], G = f(D, O);
        if (h) {
          const K = Ne(D), U = (T == null ? void 0 : T.text[T.text.length - 1]) ?? ((_a2 = v[v.length - 1]) == null ? void 0 : _a2.text.slice(-1)) ?? "", Z = U ? Ne(U) : false;
          if (K && Z) continue;
        }
        const L = !H, $ = L ? x(A) : G;
        if ($ > d && L) if (w > 0 && E(), c) {
          const K = _(A);
          for (let U = 0; U < K.length; U++) {
            const Z = K[U].token, M = K[U].style, B = Id(Z, c, r, i);
            for (const N of B) {
              const j = f(N, M);
              j + w > d && E(), !T || T.style !== M ? (P(), T = {
                text: N,
                style: M
              }) : T.text += N, w += j;
            }
          }
          A += K.length - 1;
        } else {
          const K = _(A);
          P(), g.push(K.map((U) => ({
            text: U.token,
            style: U.style
          }))), C = false, A += K.length - 1;
        }
        else if ($ + w > d && L) {
          if (Ne(D)) {
            C = false;
            continue;
          }
          E(), T = {
            text: D,
            style: O
          }, w = G;
        } else if (H && !c) !T || T.style !== O ? (P(), T = {
          text: D,
          style: O
        }) : T.text += D, w += G;
        else {
          const K = Ne(D);
          if (w === 0 && K && !C) continue;
          !T || T.style !== O ? (P(), T = {
            text: D,
            style: O
          }) : T.text += D, w += G;
        }
      }
      if (P(), v.length > 0) {
        const A = v[v.length - 1];
        A.text = dn(A.text), A.text.length === 0 && v.pop();
      }
      (v.length > 0 || g.length === b) && g.push(v);
    }
    return g;
  }
  function Zy(n) {
    const t = [];
    let e = false;
    for (const s of n) {
      const i = Pd(s.text);
      let r = true;
      for (const o of i) {
        const a = Ne(o) || or(o), l = r && e && !a;
        t.push({
          token: o,
          style: s.style,
          continuesFromPrevious: l
        }), e = !a, r = false;
      }
    }
    return t;
  }
  const Qy = {
    willReadFrequently: true
  };
  function nc(n, t, e, s, i) {
    let r = e[n];
    return typeof r != "number" && (r = i(n, t, s) + t, e[n] = r), r;
  }
  function tx(n, t, e, s, i, r, o) {
    const a = e.getContext("2d", Qy);
    a.font = t._fontString;
    let l = 0, c = "";
    const h = [], d = /* @__PURE__ */ Object.create(null), { letterSpacing: u, whiteSpace: p } = t, f = kd(p), g = Md(p);
    let m = !f;
    const y = t.wordWrapWidth + u, b = Pd(n);
    for (let _ = 0; _ < b.length; _++) {
      let v = b[_];
      if (or(v)) {
        if (!g) {
          h.push(dn(c)), m = !f, c = "", l = 0;
          continue;
        }
        v = " ";
      }
      if (f) {
        const C = Ne(v), T = Ne(c[c.length - 1]);
        if (C && T) continue;
      }
      const w = nc(v, u, d, a, s);
      if (w > y) if (c !== "" && (h.push(dn(c)), c = "", l = 0), i(v, t.breakWords)) {
        const C = Id(v, t.breakWords, o, r);
        for (const T of C) {
          const P = nc(T, u, d, a, s);
          P + l > y && (h.push(dn(c)), m = false, c = "", l = 0), c += T, l += P;
        }
      } else c.length > 0 && (h.push(dn(c)), c = "", l = 0), h.push(dn(v)), m = false, c = "", l = 0;
      else w + l > y && (m = false, h.push(dn(c)), c = "", l = 0), (c.length > 0 || !Ne(v) || m) && (c += v, l += w);
    }
    const x = dn(c);
    return x.length > 0 && h.push(x), h.join(`
`);
  }
  const sc = {
    willReadFrequently: true
  }, wn = class wt {
    static get experimentalLetterSpacingSupported() {
      let t = wt._experimentalLetterSpacingSupported;
      if (t === void 0) {
        const e = Rt.get().getCanvasRenderingContext2D().prototype;
        t = wt._experimentalLetterSpacingSupported = "letterSpacing" in e || "textLetterSpacing" in e;
      }
      return t;
    }
    constructor(t, e, s, i, r, o, a, l, c, h) {
      this.text = t, this.style = e, this.width = s, this.height = i, this.lines = r, this.lineWidths = o, this.lineHeight = a, this.maxLineWidth = l, this.fontProperties = c, h && (this.runsByLine = h.runsByLine, this.lineAscents = h.lineAscents, this.lineDescents = h.lineDescents, this.lineHeights = h.lineHeights, this.hasDropShadow = h.hasDropShadow);
    }
    static measureText(t = " ", e, s = wt._canvas, i = e.wordWrap) {
      var _a2;
      const r = `${t}-${e.styleKey}-wordWrap-${i}`;
      if (wt._measurementCache.has(r)) return wt._measurementCache.get(r);
      if (Ed(e) && Ad(t)) {
        const v = Ky(t, e, i, wt._context, wt._measureText, wt.measureFont, wt.canBreakChars, wt.wordWrapSplit), w = new wt(t, e, v.width, v.height, v.lines, v.lineWidths, v.lineHeight, v.maxLineWidth, v.fontProperties, {
          runsByLine: v.runsByLine,
          lineAscents: v.lineAscents,
          lineDescents: v.lineDescents,
          lineHeights: v.lineHeights,
          hasDropShadow: v.hasDropShadow
        });
        return wt._measurementCache.set(r, w), w;
      }
      const a = e._fontString, l = wt.measureFont(a);
      l.fontSize === 0 && (l.fontSize = e.fontSize, l.ascent = e.fontSize, l.descent = 0);
      const c = wt._context;
      c.font = a;
      const d = (i ? wt._wordWrap(t, e, s) : t).split(Vy), u = new Array(d.length);
      let p = 0;
      for (let v = 0; v < d.length; v++) {
        const w = wt._measureText(d[v], e.letterSpacing, c);
        u[v] = w, p = Math.max(p, w);
      }
      const f = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0, g = e.lineHeight || l.fontSize, m = wt._getAlignWidth(p, e, i), y = wt._adjustWidthForStyle(m, e), b = Math.max(g, l.fontSize + f) + (d.length - 1) * (g + e.leading), x = wt._adjustHeightForStyle(b, e), _ = new wt(t, e, y, x, d, u, g + e.leading, p, l);
      return wt._measurementCache.set(r, _), _;
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
      wt.experimentalLetterSpacingSupported && (wt.experimentalLetterSpacing ? (s.letterSpacing = `${e}px`, s.textLetterSpacing = `${e}px`, i = true) : (s.letterSpacing = "0px", s.textLetterSpacing = "0px"));
      const r = s.measureText(t);
      let o = r.width;
      const a = -(r.actualBoundingBoxLeft ?? 0);
      let c = (r.actualBoundingBoxRight ?? 0) - a;
      if (o > 0) if (i) o -= e, c -= e;
      else {
        const h = (wt.graphemeSegmenter(t).length - 1) * e;
        o += h, c += h;
      }
      return Math.max(o, c);
    }
    static _wordWrap(t, e, s = wt._canvas) {
      return tx(t, e, s, wt._measureText, wt.canBreakWords, wt.canBreakChars, wt.wordWrapSplit);
    }
    static isBreakingSpace(t, e) {
      return Ne(t);
    }
    static canBreakWords(t, e) {
      return e;
    }
    static canBreakChars(t, e, s, i, r) {
      return true;
    }
    static wordWrapSplit(t) {
      return wt.graphemeSegmenter(t);
    }
    static measureFont(t) {
      if (wt._fonts[t]) return wt._fonts[t];
      const e = wt._context;
      e.font = t;
      const s = e.measureText(wt.METRICS_STRING + wt.BASELINE_SYMBOL), i = s.actualBoundingBoxAscent ?? 0, r = s.actualBoundingBoxDescent ?? 0, o = {
        ascent: i,
        descent: r,
        fontSize: i + r
      };
      return wt._fonts[t] = o, o;
    }
    static clearMetrics(t = "") {
      t ? delete wt._fonts[t] : wt._fonts = {};
    }
    static get _canvas() {
      var _a2;
      if (!wt.__canvas) {
        let t;
        try {
          const e = new OffscreenCanvas(0, 0);
          if ((_a2 = e.getContext("2d", sc)) == null ? void 0 : _a2.measureText) return wt.__canvas = e, e;
          t = Rt.get().createCanvas();
        } catch {
          t = Rt.get().createCanvas();
        }
        t.width = t.height = 10, wt.__canvas = t;
      }
      return wt.__canvas;
    }
    static get _context() {
      return wt.__context || (wt.__context = wt._canvas.getContext("2d", sc)), wt.__context;
    }
  };
  wn.METRICS_STRING = "|\xC9q\xC5";
  wn.BASELINE_SYMBOL = "M";
  wn.BASELINE_MULTIPLIER = 1.4;
  wn.HEIGHT_MULTIPLIER = 2;
  wn.graphemeSegmenter = (() => {
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
  wn.experimentalLetterSpacing = false;
  wn._fonts = {};
  wn._measurementCache = By(1e3);
  un = wn;
  const ex = [
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui"
  ];
  Ho = function(n) {
    const t = typeof n.fontSize == "number" ? `${n.fontSize}px` : n.fontSize;
    let e = n.fontFamily;
    Array.isArray(n.fontFamily) || (e = n.fontFamily.split(","));
    for (let s = e.length - 1; s >= 0; s--) {
      let i = e[s].trim();
      !/([\"\'])[^\'\"]+\1/.test(i) && !ex.includes(i) && (i = `"${i}"`), e[s] = i;
    }
    return `${n.fontStyle} ${n.fontVariant} ${n.fontWeight} ${t} ${e.join(",")}`;
  };
  const ic = 1e5;
  Oi = function(n, t, e, s = 0, i = 0, r = 0) {
    if (n.texture === Tt.WHITE && !n.fill) return Yt.shared.setValue(n.color).setAlpha(n.alpha ?? 1).toHexa();
    if (n.fill) {
      if (n.fill instanceof br) {
        const o = n.fill, a = t.createPattern(o.texture.source.resource, "repeat"), l = o.transform.copyTo(vt.shared);
        return l.scale(o.texture.source.pixelWidth, o.texture.source.pixelHeight), a.setTransform(l), a;
      } else if (n.fill instanceof _n) {
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
              y = Math.max(0, Math.min(1, y)), d.addColorStop(Math.floor(y * ic) / ic, Yt.shared.setValue(m.color).toHex());
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
  const rc = new Dt();
  function ls(n) {
    let t = 0;
    for (let e = 0; e < n.length; e++) n.charCodeAt(e) === 32 && t++;
    return t;
  }
  class nx {
    getCanvasAndContext(t) {
      const { text: e, style: s, resolution: i = 1 } = t, r = s._getFinalPadding(), o = un.measureText(e || " ", s), a = Math.ceil(Math.ceil(Math.max(1, o.width) + r * 2) * i), l = Math.ceil(Math.ceil(Math.max(1, o.height) + r * 2) * i), c = Do.getOptimalCanvasAndContext(a, l);
      this._renderTextToCanvas(s, r, i, c, o);
      const h = s.trim ? Ly({
        canvas: c.canvas,
        width: a,
        height: l,
        resolution: 1,
        output: rc
      }) : rc.set(0, 0, a, l);
      return {
        canvasAndContext: c,
        frame: h
      };
    }
    returnCanvasAndContext(t) {
      Do.returnCanvasAndContext(t);
    }
    _renderTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      if (r.runsByLine && r.runsByLine.length > 0) {
        this._renderTaggedTextToCanvas(r, t, e, s, i);
        return;
      }
      const { canvas: o, context: a } = i, l = Ho(t), c = r.lines, h = r.lineHeight, d = r.lineWidths, u = r.maxLineWidth, p = r.fontProperties, f = o.height;
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
        const C = t.dropShadow && w === 0, T = C ? Math.ceil(Math.max(1, f) + e * 2) : 0, P = T * s;
        if (C) this._setupDropShadow(a, t, s, P);
        else {
          const E = t._gradientBounds, A = t._gradientOffset;
          if (E) {
            const D = {
              width: E.width,
              height: E.height,
              lineHeight: E.height,
              lines: r.lines
            };
            this._setFillAndStrokeStyles(a, t, D, e, _, (A == null ? void 0 : A.x) ?? 0, (A == null ? void 0 : A.y) ?? 0);
          } else A ? this._setFillAndStrokeStyles(a, t, r, e, _, A.x, A.y) : this._setFillAndStrokeStyles(a, t, r, e, _);
          a.shadowColor = "rgba(0,0,0,0)";
        }
        for (let E = 0; E < c.length; E++) {
          g = _, m = _ + E * h + p.ascent + v, g += this._getAlignmentOffset(d[E], b, t.align);
          let A = 0;
          if (t.align === "justify" && t.wordWrap && E < c.length - 1) {
            const D = ls(c[E]);
            D > 0 && (A = (b - d[E]) / D);
          }
          ((_c2 = t._stroke) == null ? void 0 : _c2.width) && this._drawLetterSpacing(c[E], t, i, g + e, m + e - T, true, A), t._fill !== void 0 && this._drawLetterSpacing(c[E], t, i, g + e, m + e - T, false, A);
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
          const T = Ho(C.style);
          a.font = T, w.push({
            width: un._measureText(C.text, C.style.letterSpacing, a),
            font: T
          });
        }
        x.push(w);
      }
      for (let _ = 0; _ < g; ++_) {
        const v = p && _ === 0, w = v ? Math.ceil(Math.max(1, f) + s * 2) : 0, C = w * i;
        v || (a.shadowColor = "rgba(0,0,0,0)");
        let T = b;
        for (let P = 0; P < l.length; P++) {
          const E = l[P], A = c[P], D = d[P], O = u[P], H = x[P];
          let G = b;
          G += this._getAlignmentOffset(A, m, e.align);
          let L = 0;
          if (e.align === "justify" && e.wordWrap && P < l.length - 1) {
            let U = 0;
            for (const Z of E) U += ls(Z.text);
            U > 0 && (L = (m - A) / U);
          }
          const $ = T + D;
          let K = G + s;
          for (let U = 0; U < E.length; U++) {
            const Z = E[U], { width: M, font: B } = H[U];
            if (a.font = B, a.textBaseline = Z.style.textBaseline, (_c2 = Z.style._stroke) == null ? void 0 : _c2.width) {
              const j = Z.style._stroke;
              if (a.lineWidth = j.width, a.miterLimit = j.miterLimit, a.lineJoin = j.join, a.lineCap = j.cap, v) if (Z.style.dropShadow) this._setupDropShadow(a, Z.style, i, C);
              else {
                const J = ls(Z.text);
                K += M + J * L;
                continue;
              }
              else {
                const J = un.measureFont(B), st = Z.style.lineHeight || J.fontSize, ht = {
                  width: M,
                  height: st,
                  lineHeight: st,
                  lines: [
                    Z.text
                  ]
                };
                a.strokeStyle = Oi(j, a, ht, s * 2, K - s, T);
              }
              this._drawLetterSpacing(Z.text, Z.style, r, K, $ + s - w, true, L);
            }
            const N = ls(Z.text);
            K += M + N * L;
          }
          K = G + s;
          for (let U = 0; U < E.length; U++) {
            const Z = E[U], { width: M, font: B } = H[U];
            if (a.font = B, a.textBaseline = Z.style.textBaseline, Z.style._fill !== void 0) {
              if (v) if (Z.style.dropShadow) this._setupDropShadow(a, Z.style, i, C);
              else {
                const j = ls(Z.text);
                K += M + j * L;
                continue;
              }
              else {
                const j = un.measureFont(B), J = Z.style.lineHeight || j.fontSize, st = {
                  width: M,
                  height: J,
                  lineHeight: J,
                  lines: [
                    Z.text
                  ]
                };
                a.fillStyle = Oi(Z.style._fill, a, st, s * 2, K - s, T);
              }
              this._drawLetterSpacing(Z.text, Z.style, r, K, $ + s - w, false, L);
            }
            const N = ls(Z.text);
            K += M + N * L;
          }
          T += O;
        }
      }
    }
    _setFillAndStrokeStyles(t, e, s, i, r, o = 0, a = 0) {
      var _a2;
      if (t.fillStyle = e._fill ? Oi(e._fill, t, s, i * 2, o, a) : null, (_a2 = e._stroke) == null ? void 0 : _a2.width) {
        const l = r + i * 2;
        t.strokeStyle = Oi(e._stroke, t, s, l, o, a);
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
      if (un.experimentalLetterSpacingSupported && (un.experimentalLetterSpacing ? (l.letterSpacing = `${c}px`, l.textLetterSpacing = `${c}px`, h = true) : (l.letterSpacing = "0px", l.textLetterSpacing = "0px")), (c === 0 || h) && a === 0) {
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
      const u = un.graphemeSegmenter(t);
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
  const gs = new nx(), Ca = class Xn extends on {
    constructor(t = {}) {
      super(), this.uid = ne("textStyle"), this._tick = 0, this._cachedFontString = null, sx(t), t instanceof Xn && (t = t._toObject());
      const i = {
        ...Xn.defaultTextStyle,
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
        ...Xn.defaultDropShadow,
        ...t
      }) : this._dropShadow = t ? this._createProxy({
        ...Xn.defaultDropShadow
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
        ...$e.defaultFillStyle,
        ...t
      }, () => {
        this._fill = Kn({
          ...this._originalFill
        }, $e.defaultFillStyle);
      })), this._fill = Kn(t === 0 ? "black" : t, $e.defaultFillStyle), this.update());
    }
    get stroke() {
      return this._originalStroke;
    }
    set stroke(t) {
      t !== this._originalStroke && (this._originalStroke = t, this._isFillStyle(t) && (this._originalStroke = this._createProxy({
        ...$e.defaultStrokeStyle,
        ...t
      }, () => {
        this._stroke = rr({
          ...this._originalStroke
        }, $e.defaultStrokeStyle);
      })), this._stroke = rr(t, $e.defaultStrokeStyle), this.update());
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
      const t = Xn.defaultTextStyle;
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
      return this._cachedFontString === null && (this._cachedFontString = Ho(this)), this._cachedFontString;
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
      return new Xn(this._toObject());
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
      return (t ?? null) !== null && !(Yt.isColorLike(t) || t instanceof _n || t instanceof br);
    }
  };
  Ca.defaultDropShadow = {
    alpha: 1,
    angle: Math.PI / 6,
    blur: 0,
    color: "black",
    distance: 5
  };
  Ca.defaultTextStyle = {
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
  Ge = Ca;
  function sx(n) {
    const t = n;
    if (typeof t.dropShadow == "boolean" && t.dropShadow) {
      const e = Ge.defaultDropShadow;
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
      else if (e instanceof _n || e instanceof br) s.fill = e;
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
      const e = new _n({
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
  function ix(n, t) {
    const { texture: e, bounds: s } = n, i = t._style._getFinalPadding();
    ph(s, t._anchor, e);
    const r = t._anchor._x * i * 2, o = t._anchor._y * i * 2;
    s.minX -= i - r, s.minY -= i - o, s.maxX -= i - r, s.maxY -= i - o;
  }
  rx = class {
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
  class ox extends rx {
  }
  class Rd {
    constructor(t) {
      this._renderer = t, t.runners.resolutionChange.add(this), this._managedTexts = new Is({
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
        (s.currentKey !== t.styleKey || t._resolution !== i) && this._updateGpuText(t), t._didTextUpdate = false, ix(s, t);
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
      const e = new ox();
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
  Rd.extension = {
    type: [
      lt.WebGLPipes,
      lt.WebGPUPipes,
      lt.CanvasPipes
    ],
    name: "text"
  };
  const ax = new Ie();
  function lx(n, t, e, s, i = false) {
    const r = ax;
    r.minX = 0, r.minY = 0, r.maxX = n.width / s | 0, r.maxY = n.height / s | 0;
    const o = pr.getOptimalTexture(r.width, r.height, s, false, i);
    return o.source.uploadMethodId = "image", o.source.resource = n, o.source.alphaMode = "premultiply-alpha-on-upload", o.frame.width = t / s, o.frame.height = e / s, o.source.emit("update", o.source), o.updateUvs(), o;
  }
  class Ld {
    constructor(t, e) {
      this._activeTextures = {}, this._renderer = t, this._retainCanvasContext = e;
    }
    getTexture(t, e, s, i) {
      typeof t == "string" && (It("8.0.0", "CanvasTextSystem.getTexture: Use object TextOptions instead of separate arguments"), t = {
        text: t,
        style: s,
        resolution: e
      }), t.style instanceof Ge || (t.style = new Ge(t.style)), t.textureStyle instanceof Zn || (t.textureStyle = new Zn(t.textureStyle)), typeof t.text != "string" && (t.text = t.text.toString());
      const { text: r, style: o, textureStyle: a, autoGenerateMipmaps: l } = t, c = t.resolution ?? this._renderer.resolution, { frame: h, canvasAndContext: d } = gs.getCanvasAndContext({
        text: r,
        style: o,
        resolution: c
      }), u = lx(d.canvas, h.width, h.height, c, l);
      if (a && (u.source.style = a), o.trim && (h.pad(o.padding), u.frame.copyFrom(h), u.frame.scale(1 / c), u.updateUvs()), o.filters) {
        const p = this._applyFilters(u, o.filters);
        return this.returnTexture(u), gs.returnCanvasAndContext(d), p;
      }
      return this._renderer.texture.initSource(u._source), this._retainCanvasContext || gs.returnCanvasAndContext(d), u;
    }
    returnTexture(t) {
      const e = t.source, s = e.resource;
      if (this._retainCanvasContext && (s == null ? void 0 : s.getContext)) {
        const i = s.getContext("2d");
        i && gs.returnCanvasAndContext({
          canvas: s,
          context: i
        });
      }
      e.resource = null, e.uploadMethodId = "unknown", e.alphaMode = "no-premultiply-alpha", pr.returnTexture(t, true);
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
  class $d extends Ld {
    constructor(t) {
      super(t, true);
    }
  }
  $d.extension = {
    type: [
      lt.CanvasSystem
    ],
    name: "canvasText"
  };
  class Bd extends Ld {
    constructor(t) {
      super(t, false);
    }
  }
  Bd.extension = {
    type: [
      lt.WebGLSystem,
      lt.WebGPUSystem
    ],
    name: "canvasText"
  };
  Wt.add($d);
  Wt.add(Bd);
  Wt.add(Rd);
  class Ve extends My {
    constructor(...t) {
      const e = Py(t, "Text");
      super(e, Ge), this.renderPipeId = "text", e.textureStyle && (this.textureStyle = e.textureStyle instanceof Zn ? e.textureStyle : new Zn(e.textureStyle)), this.autoGenerateMipmaps = e.autoGenerateMipmaps ?? Re.defaultOptions.autoGenerateMipmaps;
    }
    updateBounds() {
      const t = this._bounds, e = this._anchor;
      let s = 0, i = 0;
      if (this._style.trim) {
        const { frame: r, canvasAndContext: o } = gs.getCanvasAndContext({
          text: this.text,
          style: this._style,
          resolution: 1
        });
        gs.returnCanvasAndContext(o), s = r.width, i = r.height;
      } else {
        const r = un.measureText(this._text, this._style);
        s = r.width, i = r.height;
      }
      t.minX = -e._x * s, t.maxX = t.minX + s, t.minY = -e._y * i, t.maxY = t.minY + i;
    }
  }
  _r = class extends Tt {
    static create(t) {
      const { dynamic: e, ...s } = t;
      return new _r({
        source: new Re(s),
        dynamic: e ?? false
      });
    }
    resize(t, e, s) {
      return this.source.resize(t, e, s), this;
    }
  };
  class cx {
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
        m !== 16777215 && (y = Kt.getTintedCanvas({
          texture: u
        }, m));
        const b = u.frame, x = u.source.resolution, _ = b.x * x, v = b.y * x, w = b.width * x, C = b.height * x;
        i.globalAlpha = f;
        const T = -d.anchorX * b.width, P = -d.anchorY * b.height;
        d.rotation !== 0 || d.scaleX !== 1 || d.scaleY !== 1 ? (i.save(), i.translate(d.x, d.y), i.rotate(d.rotation), i.scale(d.scaleX, d.scaleY), i.drawImage(y, _, v, w, C, T, P, b.width, b.height), i.restore()) : i.drawImage(y, _, v, w, C, d.x + T, d.y + P, b.width, b.height);
      }
      i.restore();
    }
  }
  function oc(n, t = null) {
    const e = n * 6;
    if (e > 65535 ? t || (t = new Uint32Array(e)) : t || (t = new Uint16Array(e)), t.length !== e) throw new Error(`Out buffer length is incorrect, got ${t.length} and expected ${e}`);
    for (let s = 0, i = 0; s < e; s += 6, i += 4) t[s + 0] = i + 0, t[s + 1] = i + 1, t[s + 2] = i + 2, t[s + 3] = i + 0, t[s + 4] = i + 2, t[s + 5] = i + 3;
    return t;
  }
  function hx(n) {
    return {
      dynamicUpdate: ac(n, true),
      staticUpdate: ac(n, false)
    };
  }
  function ac(n, t) {
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
      const a = nr(o.format);
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
  class dx {
    constructor(t) {
      this._size = 0, this._generateParticleUpdateCache = {};
      const e = this._size = t.size ?? 1e3, s = t.properties;
      let i = 0, r = 0;
      for (const h in s) {
        const d = s[h], u = nr(d.format);
        d.dynamic ? r += u.stride : i += u.stride;
      }
      this._dynamicStride = r / 4, this._staticStride = i / 4, this.staticAttributeBuffer = new ms(e * 4 * i), this.dynamicAttributeBuffer = new ms(e * 4 * r), this.indexBuffer = oc(e);
      const o = new ld();
      let a = 0, l = 0;
      this._staticBuffer = new ts({
        data: new Float32Array(1),
        label: "static-particle-buffer",
        shrinkToFit: false,
        usage: he.VERTEX | he.COPY_DST
      }), this._dynamicBuffer = new ts({
        data: new Float32Array(1),
        label: "dynamic-particle-buffer",
        shrinkToFit: false,
        usage: he.VERTEX | he.COPY_DST
      });
      for (const h in s) {
        const d = s[h], u = nr(d.format);
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
      const e = ux(t);
      return this._generateParticleUpdateCache[e] ? this._generateParticleUpdateCache[e] : (this._generateParticleUpdateCache[e] = this.generateParticleUpdate(t), this._generateParticleUpdateCache[e]);
    }
    generateParticleUpdate(t) {
      return hx(t);
    }
    update(t, e) {
      t.length > this._size && (e = true, this._size = Math.max(t.length, this._size * 1.5 | 0), this.staticAttributeBuffer = new ms(this._size * this._staticStride * 4 * 4), this.dynamicAttributeBuffer = new ms(this._size * this._dynamicStride * 4 * 4), this.indexBuffer = oc(this._size), this.geometry.indexBuffer.setDataWithSize(this.indexBuffer, this.indexBuffer.byteLength, true));
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
  function ux(n) {
    const t = [];
    for (const e in n) {
      const s = n[e];
      t.push(e, s.code, s.dynamic ? "d" : "s");
    }
    return t.join("_");
  }
  var px = `varying vec2 vUV;
varying vec4 vColor;

uniform sampler2D uTexture;

void main(void){
    vec4 color = texture2D(uTexture, vUV) * vColor;
    gl_FragColor = color;
}`, fx = `attribute vec2 aVertex;
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
`, lc = `
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
  class mx extends gr {
    constructor() {
      const t = ca.from({
        vertex: fx,
        fragment: px
      }), e = gi.from({
        fragment: {
          source: lc,
          entryPoint: "mainFragment"
        },
        vertex: {
          source: lc,
          entryPoint: "mainVertex"
        }
      });
      super({
        glProgram: t,
        gpuProgram: e,
        resources: {
          uTexture: Tt.WHITE.source,
          uSampler: new Zn({}),
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
  class wr {
    constructor(t, e) {
      this.state = sr.for2d(), this.localUniforms = new ha({
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
      }), this.renderer = t, this.adaptor = e, this.defaultShader = new mx(), this.state = sr.for2d(), this._managedContainers = new Is({
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
      return t._gpuData[this.renderer.uid] = new dx({
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
      i.update(e, t._childrenDirty), t._childrenDirty = false, r.blendMode = Bo(t.blendMode, t.texture._source);
      const o = this.localUniforms.uniforms, a = o.uTranslationMatrix;
      t.worldTransform.copyTo(a);
      const l = s.globalUniforms.globalUniformData;
      a.tx -= l.offset.x, a.ty -= l.offset.y, a.prepend(l.projectionMatrix), o.uResolution = l.resolution, o.uRound = s._roundPixels | t._roundPixels, Sd(t.groupColorAlpha, o.uColor, 0), this.adaptor.execute(this, t);
    }
    destroy() {
      this._managedContainers.destroy(), this.renderer = null, this.defaultShader && (this.defaultShader.destroy(), this.defaultShader = null);
    }
  }
  wr.extension = {
    type: [
      lt.CanvasPipes
    ],
    name: "particle"
  };
  class Od extends wr {
    constructor(t) {
      super(t, new cx());
    }
  }
  Od.extension = {
    type: [
      lt.CanvasPipes
    ],
    name: "particle"
  };
  class gx {
    execute(t, e) {
      const s = t.state, i = t.renderer, r = e.shader || t.defaultShader;
      r.resources.uTexture = e.texture._source, r.resources.uniforms = t.localUniforms;
      const o = i.gl, a = t.getBuffers(e);
      i.shader.bind(r), i.state.set(s), i.geometry.bind(a.geometry, r.glProgram);
      const c = a.geometry.indexBuffer.data.BYTES_PER_ELEMENT === 2 ? o.UNSIGNED_SHORT : o.UNSIGNED_INT;
      o.drawElements(o.TRIANGLES, e.particleChildren.length * 6, c, 0);
    }
  }
  class Nd extends wr {
    constructor(t) {
      super(t, new gx());
    }
  }
  Nd.extension = {
    type: [
      lt.WebGLPipes
    ],
    name: "particle"
  };
  class yx {
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
  class Fd extends wr {
    constructor(t) {
      super(t, new yx());
    }
  }
  Fd.extension = {
    type: [
      lt.WebGPUPipes
    ],
    name: "particle"
  };
  const Wd = class Uo {
    constructor(t) {
      if (t instanceof Tt) this.texture = t, To(this, Uo.defaultOptions, {});
      else {
        const e = {
          ...Uo.defaultOptions,
          ...t
        };
        To(this, e, {});
      }
    }
    get alpha() {
      return this._alpha;
    }
    set alpha(t) {
      this._alpha = Math.min(Math.max(t, 0), 1), this._updateColor();
    }
    get tint() {
      return Ks(this._tint);
    }
    set tint(t) {
      this._tint = Yt.shared.setValue(t ?? 16777215).toBgrNumber(), this._updateColor();
    }
    _updateColor() {
      this.color = this._tint + ((this._alpha * 255 | 0) << 24);
    }
  };
  Wd.defaultOptions = {
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
  let ei = Wd;
  const cc = {
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
  Wt.add(Nd);
  Wt.add(Fd);
  Wt.add(Od);
  const xx = new Ie(0, 0, 0, 0), Gd = class jo extends fr {
    constructor(t = {}) {
      t = {
        ...jo.defaultOptions,
        ...t,
        dynamicProperties: {
          ...jo.defaultOptions.dynamicProperties,
          ...t == null ? void 0 : t.dynamicProperties
        }
      };
      const { dynamicProperties: e, shader: s, roundPixels: i, texture: r, particles: o, ...a } = t;
      super({
        label: "ParticleContainer",
        ...a
      }), this.renderPipeId = "particle", this.batched = false, this._childrenDirty = false, this.texture = r || null, this.shader = s, this._properties = {};
      for (const l in cc) {
        const c = cc[l], h = e[l];
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
      return xx;
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
  Gd.defaultOptions = {
    dynamicProperties: {
      vertex: false,
      position: true,
      rotation: false,
      uvs: false,
      color: false
    },
    roundPixels: false
  };
  let bx = Gd;
  Wt.add(ju, Yu);
  var _x = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
  function wx(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var zd = {
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
    }).call(_x);
  })(zd);
  var vx = zd.exports;
  const hc = wx(vx);
  function vr(n, t) {
    if (n) {
      if (typeof n == "function") return n;
      if (typeof n == "string") return hc[n];
    } else return hc[t];
  }
  class Cx {
    constructor(t) {
      this.viewport = t, this.touches = [], this.addListeners();
    }
    addListeners() {
      this.viewport.eventMode = "static", this.viewport.forceHitArea || (this.viewport.hitArea = new Dt(0, 0, this.viewport.worldWidth, this.viewport.worldHeight)), this.viewport.on("pointerdown", this.down, this), this.viewport.options.allowPreserveDragOutside ? this.viewport.on("globalpointermove", this.move, this) : this.viewport.on("pointermove", this.move, this), this.viewport.on("pointerup", this.up, this), this.viewport.on("pointerupoutside", this.up, this), this.viewport.on("pointercancel", this.up, this), this.viewport.options.allowPreserveDragOutside || this.viewport.on("pointerleave", this.up, this), this.wheelFunction = (t) => this.handleWheel(t), this.viewport.options.events.domElement.addEventListener("wheel", this.wheelFunction, {
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
      const e = new Mt();
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
  const Ds = [
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
  class Sx {
    constructor(t) {
      this.viewport = t, this.list = [], this.plugins = {};
    }
    add(t, e, s = Ds.length) {
      const i = this.plugins[t];
      i && i.destroy(), this.plugins[t] = e;
      const r = Ds.indexOf(t);
      r !== -1 && Ds.splice(r, 1), Ds.splice(s, 0, t), this.sort();
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
      for (const t of Ds) this.plugins[t] && this.list.push(this.plugins[t]);
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
  class ze {
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
  const Tx = {
    removeOnInterrupt: false,
    ease: "linear",
    time: 1e3
  };
  class Ex extends ze {
    constructor(t, e = {}) {
      super(t), this.startWidth = null, this.startHeight = null, this.deltaWidth = null, this.deltaHeight = null, this.width = null, this.height = null, this.time = 0, this.options = Object.assign({}, Tx, e), this.options.ease = vr(this.options.ease), this.setupPosition(), this.setupZoom(), this.time = 0;
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
      const e = new Mt(this.parent.scale.x, this.parent.scale.y);
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
          const i = this.startX, r = this.startY, o = this.deltaX, a = this.deltaY, l = new Mt(this.parent.x, this.parent.y);
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
  const Ax = {
    sides: "all",
    friction: 0.5,
    time: 150,
    ease: "easeInOutSine",
    underflow: "center",
    bounceBox: null
  };
  class kx extends ze {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Ax, e), this.ease = vr(this.options.ease, "easeInOutSine"), this.options.sides ? this.options.sides === "all" ? this.top = this.bottom = this.left = this.right = true : this.options.sides === "horizontal" ? (this.right = this.left = true, this.top = this.bottom = false) : this.options.sides === "vertical" ? (this.left = this.right = false, this.top = this.bottom = true) : (this.top = this.options.sides.indexOf("top") !== -1, this.bottom = this.options.sides.indexOf("bottom") !== -1, this.left = this.options.sides.indexOf("left") !== -1, this.right = this.options.sides.indexOf("right") !== -1) : this.left = this.top = this.right = this.bottom = false;
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
          topLeft: new Mt(e * this.parent.scale.x, s * this.parent.scale.y),
          bottomRight: new Mt(i * this.parent.scale.x - this.parent.screenWidth, r * this.parent.scale.y - this.parent.screenHeight)
        };
      }
      return {
        left: this.parent.left < 0,
        right: this.parent.right > this.parent.worldWidth,
        top: this.parent.top < 0,
        bottom: this.parent.bottom > this.parent.worldHeight,
        topLeft: new Mt(0, 0),
        bottomRight: new Mt(this.parent.worldWidth * this.parent.scale.x - this.parent.screenWidth, this.parent.worldHeight * this.parent.scale.y - this.parent.screenHeight)
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
  const Mx = {
    left: false,
    right: false,
    top: false,
    bottom: false,
    direction: null,
    underflow: "center"
  };
  class Px extends ze {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Mx, e), this.options.direction && (this.options.left = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.right = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.top = this.options.direction === "y" || this.options.direction === "all" ? true : null, this.options.bottom = this.options.direction === "y" || this.options.direction === "all" ? true : null), this.parseUnderflow(), this.last = {
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
      const t = new Mt(this.parent.x, this.parent.y), e = this.parent.plugins.decelerate || {};
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
  const Ix = {
    minWidth: null,
    minHeight: null,
    maxWidth: null,
    maxHeight: null,
    minScale: null,
    maxScale: null
  };
  class Rx extends ze {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Ix, e), this.clamp();
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
  const Lx = {
    friction: 0.98,
    bounce: 0.8,
    minSpeed: 0.01
  }, Cn = 16;
  class $x extends ze {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Lx, e), this.saved = [], this.timeSinceRelease = 0, this.reset(), this.parent.on("moved", (s) => this.handleMoved(s));
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
        this.parent.x += this.x * Cn / o * (Math.pow(r, i / Cn) - Math.pow(r, s / Cn)), this.x *= Math.pow(this.percentChangeX, t / Cn);
      }
      if (this.y) {
        const r = this.percentChangeY, o = Math.log(r);
        this.parent.y += this.y * Cn / o * (Math.pow(r, i / Cn) - Math.pow(r, s / Cn)), this.y *= Math.pow(this.percentChangeY, t / Cn);
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
  const Bx = {
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
  class Ox extends ze {
    constructor(t, e = {}) {
      super(t), this.windowEventHandlers = [], this.options = Object.assign({}, Bx, e), this.moved = false, this.reverse = this.options.reverse ? 1 : -1, this.xDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "x", this.yDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "y", this.keyIsPressed = false, this.parseUnderflow(), this.mouseButtons(this.options.mouseButtons), this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
            screen: new Mt(this.last.x, this.last.y),
            world: this.parent.toWorld(new Mt(this.last.x, this.last.y)),
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
        const s = new Mt(this.last.x, this.last.y);
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
  const Nx = {
    speed: 0,
    acceleration: null,
    radius: null
  };
  class Fx extends ze {
    constructor(t, e, s = {}) {
      super(t), this.target = e, this.options = Object.assign({}, Nx, s), this.velocity = {
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
  const Wx = {
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
  class Gx extends ze {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Wx, e), this.reverse = this.options.reverse ? 1 : -1, this.radiusSquared = typeof this.options.radius == "number" ? Math.pow(this.options.radius, 2) : null, this.resize();
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
  const zx = {
    noDrag: false,
    percent: 1,
    center: null,
    factor: 1,
    axis: "all"
  }, Dx = new Mt();
  class Hx extends ze {
    constructor(t, e = {}) {
      super(t), this.active = false, this.pinching = false, this.moved = false, this.options = Object.assign({}, zx, e);
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
      const { x: e, y: s } = (this.parent.parent || this.parent).toLocal(t.global, void 0, Dx), i = this.parent.input.touches;
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
          const c = new Mt(r.last.x + (o.last.x - r.last.x) / 2, r.last.y + (o.last.y - r.last.y) / 2);
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
  const Ux = {
    topLeft: false,
    friction: 0.8,
    time: 1e3,
    ease: "easeInOutSine",
    interrupt: true,
    removeOnComplete: false,
    removeOnInterrupt: false,
    forceStart: false
  };
  class jx extends ze {
    constructor(t, e, s, i = {}) {
      super(t), this.options = Object.assign({}, Ux, i), this.ease = vr(i.ease, "easeInOutSine"), this.x = e, this.y = s, this.options.forceStart && this.snapStart();
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
  const Yx = {
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
  class Vx extends ze {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Yx, e), this.ease = vr(this.options.ease), this.xIndependent = false, this.yIndependent = false, this.xScale = 0, this.yScale = 0, this.options.width > 0 && (this.xScale = t.screenWidth / this.options.width, this.xIndependent = true), this.options.height > 0 && (this.yScale = t.screenHeight / this.options.height, this.yIndependent = true), this.xScale = this.xIndependent ? this.xScale : this.yScale, this.yScale = this.yIndependent ? this.yScale : this.xScale, this.options.time === 0 ? (t.container.scale.x = this.xScale, t.container.scale.y = this.yScale, this.options.removeOnComplete && this.parent.plugins.remove("snap-zoom")) : e.forceStart && this.createSnapping();
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
  const Xx = {
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
  class qx extends ze {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Xx, e), this.keyIsPressed = false, this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
  const Kx = {
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
    ticker: qn.shared,
    allowPreserveDragOutside: false
  };
  class Dd extends zt {
    constructor(t) {
      super(), this._disableOnContextMenu = (e) => e.preventDefault(), this.options = {
        ...Kx,
        ...t
      }, this.screenWidth = this.options.screenWidth, this.screenHeight = this.options.screenHeight, this._worldWidth = this.options.worldWidth, this._worldHeight = this.options.worldHeight, this.forceHitArea = this.options.forceHitArea, this.threshold = this.options.threshold, this.options.disableOnContextMenu && this.options.events.domElement.addEventListener("contextmenu", this._disableOnContextMenu), this.options.noTicker || (this.tickerFunction = () => this.update(this.options.ticker.elapsedMS), this.options.ticker.add(this.tickerFunction)), this.input = new Cx(this), this.plugins = new Sx(this);
    }
    destroy(t) {
      var e;
      !this.options.noTicker && this.tickerFunction && this.options.ticker.remove(this.tickerFunction), this.options.disableOnContextMenu && ((e = this.options.events.domElement) == null || e.removeEventListener("contextmenu", this._disableOnContextMenu)), this.input.destroy(), super.destroy(t);
    }
    update(t) {
      this.pause || (this.plugins.update(t), this.lastViewport && (this.lastViewport.x !== this.x || this.lastViewport.y !== this.y ? this.moving = true : this.moving && (this.emit("moved-end", this), this.moving = false), this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y ? this.zooming = true : this.zooming && (this.emit("zoomed-end", this), this.zooming = false)), this.forceHitArea || (this._hitAreaDefault = new Dt(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight), this.hitArea = this._hitAreaDefault), this._dirty = this._dirty || !this.lastViewport || this.lastViewport.x !== this.x || this.lastViewport.y !== this.y || this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y, this.lastViewport = {
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
      return new Dt(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight);
    }
    toWorld(t, e) {
      return arguments.length === 2 ? this.toLocal(new Mt(t, e)) : this.toLocal(t);
    }
    toScreen(t, e) {
      return arguments.length === 2 ? this.toGlobal(new Mt(t, e)) : this.toGlobal(t);
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
      return new Mt(this.worldScreenWidth / 2 - this.x / this.scale.x, this.worldScreenHeight / 2 - this.y / this.scale.y);
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
      return new Mt(-this.x / this.scale.x, -this.y / this.scale.y);
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
      return this.plugins.add("snap-zoom", new Vx(this, t)), this;
    }
    OOB() {
      return {
        left: this.left < 0,
        right: this.right > this.worldWidth,
        top: this.top < 0,
        bottom: this.bottom > this.worldHeight,
        cornerPoint: new Mt(this.worldWidth * this.scale.x - this.screenWidth, this.worldHeight * this.scale.y - this.screenHeight)
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
      t ? (this._forceHitArea = t, this.hitArea = t) : (this._forceHitArea = null, this.hitArea = new Dt(0, 0, this.worldWidth, this.worldHeight));
    }
    drag(t) {
      return this.plugins.add("drag", new Ox(this, t)), this;
    }
    clamp(t) {
      return this.plugins.add("clamp", new Px(this, t)), this;
    }
    decelerate(t) {
      return this.plugins.add("decelerate", new $x(this, t)), this;
    }
    bounce(t) {
      return this.plugins.add("bounce", new kx(this, t)), this;
    }
    pinch(t) {
      return this.plugins.add("pinch", new Hx(this, t)), this;
    }
    snap(t, e, s) {
      return this.plugins.add("snap", new jx(this, t, e, s)), this;
    }
    follow(t, e) {
      return this.plugins.add("follow", new Fx(this, t, e)), this;
    }
    wheel(t) {
      return this.plugins.add("wheel", new qx(this, t)), this;
    }
    animate(t) {
      return this.plugins.add("animate", new Ex(this, t)), this;
    }
    clampZoom(t) {
      return this.plugins.add("clamp-zoom", new Rx(this, t)), this;
    }
    mouseEdges(t) {
      return this.plugins.add("mouse-edges", new Gx(this, t)), this;
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
  const Jx = 32, Yo = /* @__PURE__ */ new Set([
    "transport-belt",
    "fast-transport-belt",
    "express-transport-belt"
  ]), Ni = /* @__PURE__ */ new Set([
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt"
  ]), ro = /* @__PURE__ */ new Set([
    "splitter",
    "fast-splitter",
    "express-splitter"
  ]), Zx = /* @__PURE__ */ new Set([
    "inserter",
    "fast-inserter",
    "long-handed-inserter"
  ]), Qx = /* @__PURE__ */ new Set([
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
  ]), dc = {
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
  function ar(n) {
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
  function Fi(n, t, e, s, i, r) {
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
  const t0 = 9;
  function e0(n) {
    const t = {
      nodes: /* @__PURE__ */ new Map(),
      outEdges: /* @__PURE__ */ new Map(),
      inEdges: /* @__PURE__ */ new Map(),
      tileToAnchor: /* @__PURE__ */ new Map(),
      entityMap: /* @__PURE__ */ new Map()
    };
    for (const e of n.entities) t.entityMap.set(tn(e.x ?? 0, e.y ?? 0), e);
    for (const e of n.entities) {
      if (!Yo.has(e.name) && !Ni.has(e.name) && !ro.has(e.name)) continue;
      const s = e.x ?? 0, i = e.y ?? 0, r = tn(s, i);
      if (t.nodes.set(r, e), t.tileToAnchor.set(r, r), ro.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South", a = s + (o ? 1 : 0), l = i + (o ? 0 : 1);
        t.tileToAnchor.set(tn(a, l), r);
      }
    }
    for (const [e, s] of t.nodes) {
      const i = s.x ?? 0, r = s.y ?? 0, o = s.direction ?? "North", [a, l] = ar(o);
      if (Yo.has(s.name)) {
        const c = t.tileToAnchor.get(tn(i + a, r + l));
        if (c !== void 0 && c !== e) {
          const h = t.nodes.get(c), [d, u] = ar(h.direction), p = a * u - l * d;
          Fi(t, e, c, "both", p > 0, false);
        }
      } else if (Ni.has(s.name)) if (s.io_type === "input") for (let c = 1; c <= t0; c++) {
        const h = t.entityMap.get(tn(i + a * c, r + l * c));
        if (h) {
          if (Ni.has(h.name) && h.name === s.name && h.io_type === "input" && h.direction === o) break;
          if (Ni.has(h.name) && h.name === s.name && h.io_type === "output" && h.direction === o) {
            const d = t.tileToAnchor.get(tn(h.x ?? 0, h.y ?? 0));
            d !== void 0 && Fi(t, e, d, "both", false, false);
            break;
          }
        }
      }
      else {
        const c = t.tileToAnchor.get(tn(i + a, r + l));
        c !== void 0 && c !== e && Fi(t, e, c, "both", false, false);
      }
      else if (ro.has(s.name)) {
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
          f !== void 0 && f !== e && Fi(t, e, f, "both", false, true);
        }
      }
    }
    return t;
  }
  function n0(n, t) {
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
  function s0(n, t) {
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
        h && Zx.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  function i0(n, t) {
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
        h && Qx.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  const Vo = {
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
  function uc(n, t) {
    const e = t.nodes.get(n);
    if (!e || !Yo.has(e.name)) return null;
    const s = e.direction ?? "North";
    for (const i of t.inEdges.get(n) ?? []) {
      const r = t.nodes.get(i.from);
      if (!r) continue;
      const o = r.direction ?? "North";
      if (`${o}_${s}` in Vo) return {
        inDir: o,
        outDir: s
      };
    }
    return null;
  }
  function r0(n, t, e, s, i, r, o) {
    const a = s - t, l = i - e, c = Math.sqrt(a * a + l * l);
    if (c === 0) return;
    const h = a / c, d = l / c;
    let u = 0, p = true;
    for (; u < c; ) {
      const f = Math.min(p ? r : o, c - u);
      p && n.moveTo(t + h * u, e + d * u).lineTo(t + h * (u + f), e + d * (u + f)).stroke(), u += f, p = !p;
    }
  }
  function o0(n, t, e, s, i, r, o, a, l) {
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
  function a0(n, t, e, s, i) {
    const r = Jx, o = r / 2;
    for (const l of e) {
      if (t.has(l)) continue;
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = dc[c.name] ?? 14733424, [p, f] = ar(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.05
      }), n.setStrokeStyle({
        width: 1.5,
        color: u,
        alpha: 0.28,
        cap: "round"
      });
      const y = uc(l, i);
      if (y) {
        const b = Vo[`${y.inDir}_${y.outDir}`], x = b.cx(h, d, r), _ = b.cy(h, d, r);
        o0(n, x, _, o, b.startAngle, b.endAngle, b.anticlockwise, 5 / o, 3 / o);
      } else r0(n, g - p * r * 0.45, m - f * r * 0.45, g + p * r * 0.45, m + f * r * 0.45, 5, 3);
    }
    for (const l of t) {
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = dc[c.name] ?? 14733424, [p, f] = ar(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.2
      }), n.setStrokeStyle({
        width: 2,
        color: u,
        alpha: 0.85,
        cap: "round"
      });
      const y = uc(l, i);
      if (y) {
        const b = Vo[`${y.inDir}_${y.outDir}`], x = b.cx(h, d, r), _ = b.cy(h, d, r), v = x + o * Math.cos(b.startAngle), w = _ + o * Math.sin(b.startAngle);
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
  let l0, c0, On, Hd, Ud, h0, pc, fc, d0, u0, p0;
  S = 32;
  l0 = {
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
  c0 = 4872810;
  On = {
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
  Hd = {
    inserter: 6983230,
    "fast-inserter": 4886736,
    "long-handed-inserter": 13647936,
    "stack-inserter": 4114794,
    "bulk-inserter": 3123354
  };
  Ud = 9079434;
  h0 = 6974058;
  pc = 2039583;
  fc = 12623920;
  d0 = 2762e3;
  u0 = 0.35;
  p0 = {
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
  function f0(n, t) {
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = 0.21 * e + 0.72 * s + 0.07 * i, o = Math.round(r + (e - r) * t), a = Math.round(r + (s - r) * t), l = Math.round(r + (i - r) * t);
    return o << 16 | a << 8 | l;
  }
  const mc = Object.fromEntries(Object.entries(p0).map(([n, t]) => [
    n,
    f0(t, u0)
  ]));
  function m0(n, t, e) {
    const s = t * Math.min(e, 1 - e), i = (r) => {
      const o = (r + n * 12) % 12;
      return Math.round((e - s * Math.max(-1, Math.min(o - 3, 9 - o, 1))) * 255);
    };
    return i(0) << 16 | i(8) << 8 | i(4);
  }
  let jd = true;
  function Wi(n) {
    jd = n;
  }
  let Xo = /* @__PURE__ */ new Map();
  function g0(n) {
    return Xo.get(n);
  }
  function Yd(n) {
    Xo = /* @__PURE__ */ new Map();
    for (const t of n) Xo.set(t.recipe, {
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
  function $n(n) {
    if (!jd) return 7829367;
    if (!n) return 6710886;
    if (n in mc) return mc[n];
    let t = 0;
    for (let s = 0; s < n.length; s++) t = (t << 5) - t + n.charCodeAt(s) | 0;
    const e = Math.abs(t) % 30 * 12;
    return m0(e / 360, 0.2, 0.48);
  }
  const Ee = {
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
    ],
    "big-mining-drill": [
      5,
      5
    ],
    pumpjack: [
      3,
      3
    ]
  };
  function ge(n) {
    return n.split("-").map((t) => t.charAt(0).toUpperCase() + t.slice(1)).join(" ");
  }
  let Me, Cr, rn, oe, ci, Sa;
  Me = new Set(Object.keys(Ee));
  Cr = new Set(Object.keys(Hd));
  rn = new Set(Object.keys(On).filter((n) => !n.includes("underground") && !n.includes("splitter")));
  me = new Set(Object.keys(On).filter((n) => n.includes("underground")));
  oe = new Set(Object.keys(On).filter((n) => n.includes("splitter")));
  ci = /* @__PURE__ */ new Set([
    "pipe",
    "pipe-to-ground"
  ]);
  Sa = /* @__PURE__ */ new Set([
    "medium-electric-pole",
    "small-electric-pole"
  ]);
  function yi(n) {
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
  function Ln(n) {
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
  function Ta(n) {
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
  function Vd(n, t) {
    const e = n.direction ?? "North", [s, i] = Ln(e);
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
      if (!d || !(rn.has(d.name) || me.has(d.name) && d.io_type === "output" || oe.has(d.name))) continue;
      const [p, f] = Ln(d.direction), g = oe.has(d.name) ? c : d.x ?? 0, m = oe.has(d.name) ? h : d.y ?? 0;
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
  function gc(n, t) {
    const e = Math.round((n >> 16 & 255) * t), s = Math.round((n >> 8 & 255) * t), i = Math.round((n & 255) * t);
    return e << 16 | s << 8 | i;
  }
  const hi = 3, Xd = 3815994, Ea = 5592405, Aa = 0.9, Gi = S * (1 - Aa) / 2, qd = S * Aa;
  function ka(n, t) {
    const e = new ft(), s = S, i = qd, [, r] = On[n.name] ?? [
      11046960,
      14733424
    ], o = $n(n.carries);
    if (t) y0(e, s, r, n.direction, t, o);
    else {
      e.rect(Gi, Gi, i, i).fill(Xd), e.setStrokeStyle({
        width: 1,
        color: Ea,
        alignment: 0
      }), e.rect(Gi, Gi, i, i).stroke();
      const a = new ft();
      a.x = s / 2, a.y = s / 2, a.rotation = yi(n.direction), a.rect(-i / 2, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(1, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(-1, -i / 2, 2, i).fill(657930), x0(a, i, r), e.addChild(a);
    }
    return e;
  }
  function y0(n, t, e, s, i, r) {
    const o = new ft();
    o.x = t / 2, o.y = t / 2, o.rotation = yi(s), o.scale.set(Aa);
    const a = t / 2, c = (i.turn === "cw" ? 1 : -1) * a, h = -a, d = i.turn === "ccw" ? 0 : Math.PI / 2, u = i.turn === "ccw" ? Math.PI / 2 : Math.PI;
    o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().fill(Xd), o.setStrokeStyle({
      width: 1,
      color: Ea,
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
    const x = t * 0.22, _ = Math.max(1, t * 0.07), v = t * 0.5, w = x / v, C = v + x, T = v - x;
    o.setStrokeStyle({
      width: _,
      color: e,
      cap: "round",
      join: "round"
    });
    const P = Math.PI / 2, E = i.turn === "cw" ? Math.PI : 0;
    for (const A of [
      0.6
    ]) {
      const D = P + A * (E - P), O = i.turn === "cw" ? D - w : D + w, H = c + v * Math.cos(D), G = h + v * Math.sin(D), L = c + C * Math.cos(O), $ = h + C * Math.sin(O), K = c + T * Math.cos(O), U = h + T * Math.sin(O);
      o.moveTo(L, $).lineTo(H, G).lineTo(K, U).stroke();
    }
    n.addChild(o);
  }
  function x0(n, t, e) {
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
  function Kd(n) {
    const t = new ft(), e = S, [, s] = On[n.name] ?? [
      11046960,
      14733424
    ], i = n.io_type === "input", r = e / 2, o = i ? 1 : -1, a = new ft();
    a.x = r, a.y = r, a.rotation = yi(n.direction);
    const l = $n(n.carries), c = qd / 2, h = e * 0.25, d = o * r, u = 0;
    a.moveTo(-c, d).lineTo(c, d).lineTo(h, u).lineTo(-h, u).closePath().fill({
      color: l,
      alpha: 0.7
    }), a.setStrokeStyle({
      width: 1,
      color: Ea,
      alpha: 0.8
    }), a.moveTo(-c, d).lineTo(-h, u).lineTo(h, u).lineTo(c, d).stroke();
    const p = e * 0.38, f = e * 0.3, g = o * e * 0.22, m = g - f / 2, y = g + f / 2;
    return a.moveTo(0, m).lineTo(p / 2, y).lineTo(-p / 2, y).closePath().fill(s), t.addChild(a), t;
  }
  function Jd(n) {
    const t = new ft(), [e, s] = On[n.name] ?? [
      11046960,
      14733424
    ], i = n.direction === "North" || n.direction === "South", r = i ? S * 2 - 1 : S - 1, o = i ? S - 1 : S * 2 - 1, a = i ? r / 2 : o / 2, l = Math.max(2, Math.min(r, o) * 0.18);
    t.roundRect(0, 0, r, o, hi).fill(e), t.roundRect(0, 0, r, o, hi).fill({
      color: $n(n.carries),
      alpha: 0.3
    }), i ? t.rect(a - l / 2, 0, l, o).fill(gc(e, 0.5)) : t.rect(0, a - l / 2, r, l).fill(gc(e, 0.5));
    const c = yi(n.direction), h = a * 0.25, d = Math.max(1, a * 0.12);
    for (let u = 0; u < 2; u++) {
      const p = i ? a * u + a / 2 : r / 2, f = i ? o / 2 : a * u + a / 2, g = new ft();
      g.x = p, g.y = f, g.rotation = c, g.setStrokeStyle({
        width: d,
        color: s,
        cap: "round"
      }), g.moveTo(-h, h * 0.5).lineTo(0, -h * 0.5).lineTo(h, h * 0.5).stroke(), t.addChild(g);
    }
    return t;
  }
  function Zd(n) {
    const t = new ft(), e = S - 1, s = n.carries ? $n(n.carries) : Hd[n.name] ?? 6983230;
    t.roundRect(0, 0, e, e, hi).fill(2767402);
    const i = new ft();
    i.x = e / 2, i.y = e / 2, i.rotation = yi(n.direction), i.circle(0, e * 0.2, e * 0.15).fill(4473924);
    const r = Math.max(1.5, e * 0.12);
    i.setStrokeStyle({
      width: r,
      color: s,
      cap: "round"
    }), i.moveTo(0, e * 0.2).lineTo(0, -e * 0.35).stroke();
    const o = -e * 0.35, a = e * 0.18;
    return i.moveTo(-a, o - a * 0.6).lineTo(0, o).lineTo(a, o - a * 0.6).stroke(), t.addChild(i), t;
  }
  const Ma = 1, Pa = 2, Ia = 4, Ra = 8;
  function b0(n) {
    const [t, e] = Ln(n.direction);
    return [
      -t,
      -e
    ];
  }
  function Qd(n, t, e) {
    if (n.name === "pipe") return true;
    if (n.name === "pipe-to-ground") {
      const [s, i] = b0(n);
      return -t === s && -e === i;
    }
    return false;
  }
  function tu(n, t) {
    const e = new ft(), s = S - 1, i = n.name === "pipe-to-ground", r = i ? h0 : Ud;
    e.roundRect(0, 0, s, s, hi).fill(pc);
    const o = s / 2, a = s / 2, l = Math.max(2, s * 0.4);
    if (i) {
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      });
      const [c, h] = Ln(n.direction);
      e.moveTo(o, a).lineTo(o - c * s / 2, a - h * s / 2).stroke(), e.circle(o, a, l * 0.4).fill(r), e.circle(o, a, l * 0.25).fill(pc);
    } else if (t === 0) e.circle(o, a, l * 0.4).fill(r);
    else {
      const c = !!(t & Ma), h = !!(t & Pa), d = !!(t & Ia), u = !!(t & Ra), p = (c ? 1 : 0) + (h ? 1 : 0) + (d ? 1 : 0) + (u ? 1 : 0);
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      }), p === 1 ? (c ? e.moveTo(o, a).lineTo(o, 0).stroke() : h ? e.moveTo(o, a).lineTo(s, a).stroke() : d ? e.moveTo(o, a).lineTo(o, s).stroke() : e.moveTo(o, a).lineTo(0, a).stroke(), e.circle(o, a, l * 0.4).fill(r)) : c && d && !h && !u ? e.moveTo(o, 0).lineTo(o, s).stroke() : h && u && !c && !d ? e.moveTo(0, a).lineTo(s, a).stroke() : p === 2 ? c && h ? e.moveTo(o, 0).quadraticCurveTo(o, a, s, a).stroke() : h && d ? e.moveTo(s, a).quadraticCurveTo(o, a, o, s).stroke() : d && u ? e.moveTo(o, s).quadraticCurveTo(o, a, 0, a).stroke() : e.moveTo(0, a).quadraticCurveTo(o, a, o, 0).stroke() : p === 3 ? u ? d ? h ? (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, s).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(0, a).stroke()) : (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, 0).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(s, a).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(0, a).lineTo(s, a).stroke());
    }
    return e;
  }
  function eu() {
    const n = new ft(), t = S - 1;
    n.roundRect(0, 0, t, t, hi).fill(d0);
    const e = t / 2, s = t / 2, i = t * 0.38, r = Math.max(1.5, t * 0.2);
    return n.rect(e - r / 2, s - i, r, i * 2).fill(fc), n.rect(e - i, s - r / 2, i * 2, r).fill(fc), n.circle(e, s, r * 0.6).fill(14729280), n;
  }
  function nu(n) {
    const t = new ft(), [e, s] = Ee[n.name] ?? [
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
      const f = Math.sqrt((u - h) ** 2 + (p - d) ** 2), g = (u - h) / f, m = (p - d) / f;
      let y = 0;
      for (; y < f; ) {
        const b = Math.min(y + o, f);
        t.moveTo(h + g * y, d + m * y).lineTo(h + g * b, d + m * b).stroke(), y = b + a;
      }
    }
    const l = 1.8, c = qo(`/spaghettio/pr-333/entity-frames/${n.name}.png`);
    if (c) {
      const h = new We(c), d = S / _0;
      h.scale.set(d * l), h.x = -i * (l - 1) / 2, h.y = -r * (l - 1) / 2, t.addChild(h);
    } else {
      const h = qo(`/spaghettio/pr-333/icons/${n.name}.png`);
      if (h) {
        const d = new We(h), u = Math.min(i, r) * 0.8 * l;
        d.width = u, d.height = u, d.x = (i - u) / 2, d.y = (r - u) / 2, t.addChild(d);
      } else {
        const d = l0[n.name] ?? c0;
        t.roundRect(2, 2, i - 4, r - 4, 3).fill({
          color: d,
          alpha: 0.5
        });
      }
    }
    return t;
  }
  function su() {
    const n = new ft(), t = S - 1;
    return n.rect(0, 0, t, t).fill(4872810), n.setStrokeStyle({
      width: 1,
      color: 0,
      alpha: 0.4
    }), n.rect(0, 0, t, t).stroke(), n;
  }
  const _0 = 64;
  async function w0(n) {
    const t = "/spaghettio/pr-333/", e = [
      ...n.map((s) => `${t}icons/${s}.png`),
      ...n.map((s) => `${t}entity-frames/${s}.png`)
    ];
    await Promise.allSettled(e.map((s) => Oe.load(s)));
  }
  async function iu(n) {
    const t = "/spaghettio/pr-333/";
    await Promise.allSettled(n.map((e) => Oe.load(`${t}icons/${e}.png`)));
  }
  function v0(n) {
    const t = /* @__PURE__ */ new Set();
    for (const e of n) e.carries && t.add(e.carries);
    return Array.from(t);
  }
  function qo(n) {
    return ie.has(n) ? Oe.get(n) ?? null : null;
  }
  const C0 = {
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
  function ru(n) {
    const t = C0[n.name];
    if (!t) return [];
    const e = n.mirror ?? false;
    return t.filter(([, , , s]) => s === "always" || s === "default" && !e || s === "mirror" && e);
  }
  function ou(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of ru(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  Sr = function() {
    return {
      tileMap: /* @__PURE__ */ new Map(),
      machineByTile: /* @__PURE__ */ new Map()
    };
  };
  lr = function(n, t) {
    const e = n.x ?? 0, s = n.y ?? 0;
    if (t.tileMap.set(`${e},${s}`, n), oe.has(n.name)) {
      const [i, r] = Ta(n.direction);
      t.tileMap.set(`${e + i},${s + r}`, n);
    }
    if (Me.has(n.name)) {
      const [i, r] = Ee[n.name] ?? [
        1,
        1
      ];
      for (let o = 0; o < r; o++) for (let a = 0; a < i; a++) t.machineByTile.set(`${e + a},${s + o}`, n);
    }
  };
  Gn = function(n, t) {
    let e;
    if (rn.has(n.name)) e = ka(n, Vd(n, t.tileMap));
    else if (me.has(n.name)) e = Kd(n);
    else if (oe.has(n.name)) e = Jd(n);
    else if (Cr.has(n.name)) e = Zd(n);
    else if (ci.has(n.name)) {
      let s = 0;
      if (n.name === "pipe") {
        const i = n.x ?? 0, r = n.y ?? 0;
        for (const [o, a, l] of [
          [
            0,
            -1,
            Ma
          ],
          [
            1,
            0,
            Pa
          ],
          [
            0,
            1,
            Ia
          ],
          [
            -1,
            0,
            Ra
          ]
        ]) {
          const c = `${i + o},${r + a}`, h = t.tileMap.get(c);
          if (h && Qd(h, o, a)) {
            s |= l;
            continue;
          }
          const d = t.machineByTile.get(c);
          d && ou(i, r, d) && (s |= l);
        }
      }
      e = tu(n, s);
    } else Sa.has(n.name) ? e = eu() : Me.has(n.name) ? e = nu(n) : e = su();
    return n.quality && n.quality !== "normal" && E0(e, n.quality), e.x = (n.x ?? 0) * S, e.y = (n.y ?? 0) * S, e;
  };
  const au = {
    uncommon: 5229135,
    rare: 5213130,
    epic: 10899402,
    legendary: 15246141
  }, S0 = [
    "quality-uncommon",
    "quality-rare",
    "quality-epic",
    "quality-legendary"
  ];
  async function T0() {
    const n = "/spaghettio/pr-333/";
    await Promise.allSettled(S0.map((t) => Oe.load(`${n}icons/${t}.png`)));
  }
  function E0(n, t) {
    const e = n.getLocalBounds(), s = 1.5, i = qo(`/spaghettio/pr-333/icons/quality-${t}.png`);
    if (i) {
      const c = S * 0.42, h = new We(i);
      h.width = c, h.height = c, h.x = e.minX + s, h.y = e.maxY - c - s, n.addChild(h);
      return;
    }
    const r = au[t];
    if (r === void 0) return;
    const o = S * 0.14, a = e.minX + o + s, l = e.maxY - o - s;
    n.poly([
      a,
      l - o,
      a + o,
      l,
      a,
      l + o,
      a - o,
      l
    ]).fill({
      color: r,
      alpha: 0.95
    }).stroke({
      color: 1710618,
      width: 1,
      alpha: 0.8
    });
  }
  Gv = function(n, t, e = 8) {
    if (!me.has(n.name) || n.io_type !== "input") return null;
    const [s, i] = Ln(n.direction), r = n.x ?? 0, o = n.y ?? 0;
    for (let a = 1; a <= e; a++) {
      const l = t.get(`${r + s * a},${o + i * a}`);
      if (l) {
        if (me.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "input") return null;
        if (me.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "output") {
          const [c] = On[n.name] ?? [
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
  ni = function(n, t, e, s, i, r) {
    t.removeChildren();
    const o = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (o.set(`${f.x ?? 0},${f.y ?? 0}`, f), oe.has(f.name)) {
      const [g, m] = Ta(f.direction);
      o.set(`${(f.x ?? 0) + g},${(f.y ?? 0) + m}`, f);
    }
    if (r) for (const f of r) {
      const g = `${f.x ?? 0},${f.y ?? 0}`;
      o.has(g) || o.set(g, f);
    }
    const a = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (Me.has(f.name)) {
      const [g, m] = Ee[f.name] ?? [
        1,
        1
      ], y = f.x ?? 0, b = f.y ?? 0;
      for (let x = 0; x < m; x++) for (let _ = 0; _ < g; _++) a.set(`${y + _},${b + x}`, f);
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const m of n.entities) me.has(m.name) && f.set(`${m.x ?? 0},${m.y ?? 0}`, m);
      const g = 8;
      for (const m of n.entities) {
        if (!me.has(m.name) || m.io_type !== "input") continue;
        const [y, b] = Ln(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 1; v <= g; v++) {
          const w = f.get(`${x + y * v},${_ + b * v}`);
          if (w) {
            if (me.has(w.name) && w.name === m.name && w.direction === m.direction && w.io_type === "input") break;
            if (me.has(w.name) && w.name === m.name && w.direction === m.direction && w.io_type === "output") {
              const [C] = On[m.name] ?? [
                11046960,
                14733424
              ], T = new ft(), P = Math.abs(y) > 0;
              for (let E = 1; E < v; E++) {
                const A = (x + y * E) * S, D = (_ + b * E) * S;
                P ? T.rect(A, D + S * 0.25, S, S * 0.5).fill({
                  color: C,
                  alpha: 0.25
                }) : T.rect(A + S * 0.25, D, S * 0.5, S).fill({
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
      for (const m of n.entities) m.name === "pipe-to-ground" && f.set(`${m.x ?? 0},${m.y ?? 0}`, m);
      const g = 10;
      for (const m of n.entities) {
        if (m.name !== "pipe-to-ground" || m.io_type !== "input") continue;
        const [y, b] = Ln(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 2; v <= g; v++) {
          const w = f.get(`${x + y * v},${_ + b * v}`);
          if (!w) continue;
          const [C, T] = Ln(w.direction);
          if (w.io_type !== "output" || C !== -y || T !== -b) break;
          const P = new ft();
          P.setStrokeStyle({
            width: 2,
            color: Ud,
            alpha: 0.55,
            cap: "round"
          });
          const E = (x + 0.5 + y * 0.5) * S, A = (_ + 0.5 + b * 0.5) * S, D = (v - 1) * S, O = 5, H = 3;
          let G = 0;
          for (; G < D; ) {
            const L = Math.min(G + O, D);
            P.moveTo(E + y * G, A + b * G).lineTo(E + y * L, A + b * L).stroke(), G = L + H;
          }
          t.addChild(P);
          break;
        }
      }
    }
    const l = /* @__PURE__ */ new Map(), c = [], h = /* @__PURE__ */ new Map();
    for (const f of n.entities) {
      let g;
      if (rn.has(f.name)) g = ka(f, Vd(f, o));
      else if (me.has(f.name)) g = Kd(f);
      else if (oe.has(f.name)) g = Jd(f);
      else if (Cr.has(f.name)) g = Zd(f);
      else if (ci.has(f.name)) {
        let y = 0;
        if (f.name === "pipe") {
          const b = f.x ?? 0, x = f.y ?? 0;
          for (const [_, v, w] of [
            [
              0,
              -1,
              Ma
            ],
            [
              1,
              0,
              Pa
            ],
            [
              0,
              1,
              Ia
            ],
            [
              -1,
              0,
              Ra
            ]
          ]) {
            if (v === -1 && x + v < 0) {
              y |= w;
              continue;
            }
            const C = `${b + _},${x + v}`, T = o.get(C);
            if (T && Qd(T, _, v)) {
              y |= w;
              continue;
            }
            const P = a.get(C);
            P && ou(b, x, P) && (y |= w);
          }
        }
        g = tu(f, y);
      } else Sa.has(f.name) ? g = eu() : Me.has(f.name) ? g = nu(f) : g = su();
      g.x = (f.x ?? 0) * S, g.y = (f.y ?? 0) * S, s && (g.eventMode = "static", g.cursor = "pointer", g.on("click", () => s(f)));
      const m = yc(f);
      m && (l.has(m) || l.set(m, []), l.get(m).push(g)), h.set(g, `${f.x ?? 0},${f.y ?? 0}`), c.push(g), t.addChild(g), i == null ? void 0 : i(f, [
        g
      ]);
    }
    const d = e0(n);
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
        const { downstream: y, upstream: b } = n0(m, d), x = /* @__PURE__ */ new Set([
          ...y,
          ...b
        ]), _ = s0(x, d.entityMap), v = i0(_, d.entityMap);
        for (const w of c) {
          const C = h.get(w);
          if (!C) {
            w.alpha = 0.15;
            continue;
          }
          x.has(C) ? w.alpha = 0.5 : _.has(C) ? w.alpha = 0.9 : v.has(C) ? w.alpha = 0.75 : w.alpha = 0.15;
        }
        u = new ft(), a0(u, y, b, m, d), t.addChild(u);
      },
      clearHighlight() {
        p();
      },
      chainKey: yc
    };
  };
  function yc(n) {
    return n.carries ? n.carries : n.recipe ? n.recipe : null;
  }
  const di = 4096, xt = 128, en = di / xt;
  let Pn = null;
  const ui = /* @__PURE__ */ new Map();
  let sn = 0, pi = null;
  function A0(n) {
    pi = n;
  }
  function pn(n, t, e, s) {
    const i = ui.get(n);
    if (i) return i;
    if (!pi) return console.warn("[atlas] getEntityTexture called before initAtlas; returning blank texture"), Tt.EMPTY;
    Pn || (Pn = _r.create({
      width: di,
      height: di
    })), sn >= en * en && (console.warn("[atlas] atlas is full \u2014 variant will reuse slot 0:", n), sn = 0);
    const r = sn % en, o = Math.floor(sn / en), a = r * xt, l = o * xt;
    sn++;
    const c = new ft();
    s(c);
    const h = new vt(1, 0, 0, 1, a, l);
    pi.render({
      container: c,
      target: Pn,
      transform: h,
      clear: false
    }), c.destroy({
      children: true
    });
    const d = new Dt(a, l, xt, xt), u = new Tt({
      source: Pn.source,
      frame: d
    });
    return ui.set(n, u), u;
  }
  function xc(n, t, e, s) {
    const i = ui.get(n);
    if (i) return i;
    if (!pi) return console.warn("[atlas] getMultiCellTexture called before initAtlas; returning blank texture"), Tt.EMPTY;
    Pn || (Pn = _r.create({
      width: di,
      height: di
    }));
    const r = sn % en;
    r + t > en && (sn += en - r);
    const a = sn % en, l = Math.floor(sn / en), c = a * xt, h = l * xt;
    sn = (l + e) * en;
    const d = t * xt, u = e * xt, p = new ft();
    s(p, d, u);
    const f = new vt(1, 0, 0, 1, c, h);
    pi.render({
      container: p,
      target: Pn,
      transform: f,
      clear: false
    }), p.destroy({
      children: true
    });
    const g = new Dt(c, h, d, u), m = new Tt({
      source: Pn.source,
      frame: g
    });
    return ui.set(n, m), m;
  }
  function Ko(n) {
    const t = `icon:${n}`, e = ui.get(t);
    if (e) return e;
    const s = `/spaghettio/pr-333/icons/${n}.png`;
    if (ie.has(s)) {
      const r = Oe.get(s);
      if (r) return pn(t, xt, xt, (a) => {
        const c = xt - 16;
        a.rect(8, 8, c, c).fill({
          texture: r
        });
      });
    }
    const i = $n(n);
    return pn(t, xt, xt, (r) => {
      const o = xt / 2, a = xt / 2;
      r.circle(o, a, 7).fill({
        color: i,
        alpha: 0.85
      });
    });
  }
  function k0(n, t, e = "straight") {
    return `belt:${n}:${t}:${e}`;
  }
  function M0(n) {
    return `pipe:${n}`;
  }
  function P0(n, t, e) {
    return `ugbelt:${n}:${t}:${e}`;
  }
  function I0(n, t) {
    return `splitter:${n}:${t}`;
  }
  function R0(n, t) {
    return `inserter:${n}:${t}`;
  }
  function L0(n) {
    return `machine:${n}`;
  }
  function $0(n) {
    return `pole:${n}`;
  }
  function B0(n) {
    return `ptg:${n}`;
  }
  const nn = 3200;
  let lu = null, cu = null, hu = null;
  function Xe() {
    lu == null ? void 0 : lu();
  }
  function Tr() {
    cu == null ? void 0 : cu();
  }
  function Jn() {
    hu == null ? void 0 : hu();
  }
  async function O0(n) {
    const t = new da();
    await t.init({
      resizeTo: n,
      background: 1973790,
      antialias: true,
      autoStart: false,
      sharedTicker: false
    }), A0(t.renderer), t.ticker.add(() => t.render(), null, ii.LOW), n.appendChild(t.canvas), t.canvas.addEventListener("contextmenu", (h) => h.preventDefault());
    const e = new Dd({
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
    lu = r, cu = o, hu = a, e.on("moved", r), e.on("zoomed", r);
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
  const zi = 32, bc = 2763306, _c = 3815994;
  function N0(n) {
    const t = new ft();
    return n.addChildAt(t, 0), t;
  }
  function zn(n, t, e, s = 1) {
    if (n.clear(), t <= 0 || e <= 0) return;
    const i = Math.max(0, Math.min(1, s));
    if (i === 0) return;
    const r = t * zi, a = e * zi * i;
    for (let l = 0; l <= t; l++) {
      const c = l * zi, h = l % 10 === 0;
      n.moveTo(c, 0).lineTo(c, a).stroke({
        width: h ? 1.5 : 1,
        color: h ? _c : bc
      });
    }
    for (let l = 0; l <= e; l++) {
      const c = l * zi;
      if (c > a) break;
      const h = l % 10 === 0;
      n.moveTo(0, c).lineTo(r, c).stroke({
        width: h ? 1.5 : 1,
        color: h ? _c : bc
      });
    }
  }
  const Qi = {
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
  }, F0 = 8947848;
  function du(n) {
    const t = n >> 16 & 255, e = n >> 8 & 255, s = n & 255;
    return `rgb(${t},${e},${s})`;
  }
  const W0 = [
    {
      name: "transport-belt",
      color: Qi["transport-belt"],
      throughput: 15
    },
    {
      name: "fast-transport-belt",
      color: Qi["fast-transport-belt"],
      throughput: 30
    },
    {
      name: "express-transport-belt",
      color: Qi["express-transport-belt"],
      throughput: 45
    }
  ];
  function uu(n) {
    for (const t of W0) if (n <= t.throughput) return t;
    return null;
  }
  const G0 = 240, wc = 80, La = 180, z0 = 60, vc = 100, Cc = 100, Sc = "production-graph";
  function D0(n) {
    return n <= 15 ? 13153632 : n <= 30 ? 14700624 : n <= 45 ? 5284064 : 16711816;
  }
  const Tc = new Ge({
    fontSize: 13,
    fontWeight: "bold",
    fill: 14737632,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: La - 12
  }), Ec = new Ge({
    fontSize: 11,
    fill: 10280190,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: La - 12
  }), Ac = new Ge({
    fontSize: 10,
    fill: 16777215,
    fontFamily: "sans-serif"
  });
  function Hs(n, t) {
    const e = n.getChildByName(Sc);
    e && (e.destroy({
      children: true
    }), n.removeChild(e));
    const s = new zt();
    if (s.label = Sc, n.addChild(s), !t || t.machines.length === 0) return s;
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
        x: vc + (x + a) * G0,
        y: Cc + w * wc,
        w: La,
        h: z0,
        machine: v
      });
    });
    const p = /* @__PURE__ */ new Map();
    c.forEach((x, _) => {
      p.set(x, {
        x: vc,
        y: Cc + _ * wc,
        w: 140,
        h: 40
      });
    });
    const f = new ft();
    s.addChild(f);
    for (const x of t.machines) {
      const _ = u.get(x.recipe);
      if (_) for (const v of x.inputs) {
        const w = h.get(v.item), C = w ? u.get(w.recipe) : p.get(v.item);
        if (!C) continue;
        const T = D0(v.rate), P = C.x + C.w, E = C.y + C.h * 2 / 3, A = _.x, D = _.y + _.h / 3, O = (P + A) / 2;
        f.moveTo(P, E).lineTo(O, E).lineTo(O, D).lineTo(A, D).stroke({
          color: T,
          width: 2,
          alpha: 0.85
        });
        const H = `${v.rate.toFixed(1)}/s ${v.item}`, G = (P + O) / 2, L = E - 14, $ = un.measureText(H, Ac), K = new ft();
        K.rect(G - 2, L - 1, $.width + 4, $.height + 2).fill({
          color: 1973790,
          alpha: 0.7
        }), s.addChild(K);
        const U = new Ve({
          text: H,
          style: Ac
        });
        U.position.set(G, L), s.addChild(U);
      }
    }
    for (const x of u.values()) {
      const _ = x.machine, v = Qi[_.entity] ?? F0, w = new ft();
      w.rect(x.x, x.y, x.w, x.h).fill({
        color: v,
        alpha: 0.6
      }).stroke({
        color: v,
        width: 2
      }), s.addChild(w);
      const C = new Ve({
        text: `${_.count.toFixed(1)} \xD7 ${_.entity}`,
        style: Tc
      });
      C.position.set(x.x + 6, x.y + 6), s.addChild(C);
      const T = new Ve({
        text: _.recipe,
        style: Ec
      });
      T.position.set(x.x + 6, x.y + 24), s.addChild(T);
    }
    for (const [x, _] of p) {
      const v = d.get(x), w = v !== void 0 ? `${v.toFixed(1)}/s` : "", C = new ft();
      C.rect(_.x, _.y, _.w, _.h).fill({
        color: 2763306,
        alpha: 0.8
      }).stroke({
        color: 8947848,
        width: 1.5
      }), s.addChild(C);
      const T = new Ve({
        text: w,
        style: Tc
      });
      T.position.set(_.x + 6, _.y + 4), s.addChild(T);
      const P = new Ve({
        text: x,
        style: Ec
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
  function ae(n, t) {
    return `${n},${t}`;
  }
  function Ce(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function Sn(n) {
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
  function xe(n, t, e) {
    if (t === e) return;
    let s = n.get(t);
    s || (s = [], n.set(t, s)), s.includes(e) || s.push(e);
    let i = n.get(e);
    i || (i = [], n.set(e, i)), i.includes(t) || i.push(t);
  }
  function H0(n) {
    const t = /* @__PURE__ */ new Map();
    for (const e of n.entities) {
      const s = e.x ?? 0, i = e.y ?? 0, r = Ce(e);
      if (t.set(ae(s, i), r), oe.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South";
        t.set(ae(s + (o ? 1 : 0), i + (o ? 0 : 1)), r);
      }
      if (Me.has(e.name)) {
        const [o, a] = Ee[e.name] ?? [
          1,
          1
        ];
        for (let l = 0; l < a; l++) for (let c = 0; c < o; c++) c === 0 && l === 0 || t.set(ae(s + c, i + l), r);
      }
    }
    return t;
  }
  const U0 = 9;
  function pu(n) {
    const t = /* @__PURE__ */ new Map(), e = H0(n);
    for (const o of n.entities) {
      const a = Ce(o);
      t.has(a) || t.set(a, []);
    }
    const s = /* @__PURE__ */ new Map();
    for (const o of n.entities) s.set(ae(o.x ?? 0, o.y ?? 0), o);
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
      const a = o.x ?? 0, l = o.y ?? 0, c = Ce(o), [h, d] = Sn(o.direction);
      if (rn.has(o.name)) {
        const u = e.get(ae(a + h, l + d));
        u && xe(t, c, u);
        const p = e.get(ae(a - h, l - d));
        p && xe(t, c, p);
        for (const [f, g] of i) {
          if (f === h && g === d || f === -h && g === -d) continue;
          const m = s.get(ae(a + f, l + g));
          if (!m || !rn.has(m.name) && !me.has(m.name) && !oe.has(m.name)) continue;
          const [y, b] = Sn(m.direction), x = oe.has(m.name) ? a + f : m.x ?? 0, _ = oe.has(m.name) ? l + g : m.y ?? 0;
          x + y === a && _ + b === l && xe(t, c, Ce(m));
        }
      } else if (me.has(o.name)) {
        if (o.io_type === "input") for (let u = 1; u <= U0; u++) {
          const p = s.get(ae(a + h * u, l + d * u));
          if (p) {
            if (me.has(p.name) && p.name === o.name && p.io_type === "input" && p.direction === o.direction) break;
            if (me.has(p.name) && p.name === o.name && p.io_type === "output" && p.direction === o.direction) {
              xe(t, c, Ce(p));
              break;
            }
          }
        }
        else {
          const u = e.get(ae(a + h, l + d));
          u && xe(t, c, u);
        }
        for (const [u, p] of i) {
          const f = s.get(ae(a + u, l + p));
          if (!f || !rn.has(f.name) && !oe.has(f.name)) continue;
          const [g, m] = Sn(f.direction);
          (f.x ?? 0) + g === a && (f.y ?? 0) + m === l && xe(t, c, Ce(f));
        }
      } else if (oe.has(o.name)) {
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
          const y = e.get(ae(a + g + h, l + m + d));
          y && y !== c && xe(t, c, y);
          const b = e.get(ae(a + g - h, l + m - d));
          b && b !== c && xe(t, c, b);
        }
      }
    }
    for (const o of n.entities) {
      if (!Cr.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = Ce(o), [h, d] = Sn(o.direction), p = o.name === "long-handed-inserter" ? 2 : 1, f = e.get(ae(a - h * p, l - d * p)), g = e.get(ae(a + h * p, l + d * p));
      f && xe(t, c, f), g && xe(t, c, g);
    }
    const r = 10;
    for (const o of n.entities) {
      if (!ci.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = Ce(o);
      if (o.name === "pipe") for (const [h, d] of i) {
        const u = s.get(ae(a + h, l + d));
        if (u) {
          if (u.name === "pipe") xe(t, c, Ce(u));
          else if (u.name === "pipe-to-ground") {
            const [p, f] = Sn(u.direction);
            h === p && d === f && xe(t, c, Ce(u));
          } else if (Me.has(u.name)) {
            const p = e.get(ae(u.x ?? 0, u.y ?? 0));
            p && xe(t, c, p);
          }
        }
      }
      else if (o.name === "pipe-to-ground") {
        if (o.io_type === "input") {
          const [p, f] = Sn(o.direction);
          for (let g = 2; g <= r; g++) {
            const m = s.get(ae(a + p * g, l + f * g));
            if (!m || m.name !== "pipe-to-ground") continue;
            const [y, b] = Sn(m.direction);
            if (m.io_type === "output" && y === -p && b === -f) {
              xe(t, c, Ce(m));
              break;
            }
            break;
          }
        }
        const [h, d] = Sn(o.direction), u = s.get(ae(a - h, l - d));
        u && u.name === "pipe" && xe(t, c, Ce(u));
      }
    }
    for (const o of n.entities) {
      if (!Me.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = Ce(o), [h, d] = Ee[o.name] ?? [
        1,
        1
      ];
      for (let u = 0; u < d; u++) for (let p = 0; p < h; p++) for (const [f, g] of i) {
        const m = a + p + f, y = l + u + g;
        if (m >= a && m < a + h && y >= l && y < l + d) continue;
        const b = s.get(ae(m, y));
        b && ci.has(b.name) && xe(t, c, Ce(b));
      }
    }
    return t;
  }
  function fu(n, t) {
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
  const mu = 150, j0 = 64, cr = 0.2, Y0 = 5, V0 = 100, X0 = 200, q0 = (n) => 1 - Math.pow(1 - n, 3), K0 = (n) => n;
  function Di() {
    return new bx({
      dynamicProperties: {
        color: true,
        position: false,
        rotation: false,
        vertex: false,
        uvs: false
      }
    });
  }
  function gu() {
    const n = Di(), t = new zt(), e = Di(), s = Di(), i = Di();
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
        n.removeParticles(), t.removeChildren(), e.removeParticles(), s.removeParticles(), i.removeParticles(), le.clear(), Es.clear();
      },
      count() {
        return n.particleChildren.length + e.particleChildren.length + s.particleChildren.length + i.particleChildren.length;
      }
    };
  }
  const le = /* @__PURE__ */ new Map();
  function ps(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function J0(n, t) {
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
      if (!u || !(rn.has(u.name) || me.has(u.name) && u.io_type === "output" || oe.has(u.name))) continue;
      const [f, g] = s[u.direction ?? "North"] ?? [
        0,
        -1
      ], m = oe.has(u.name) ? h : u.x ?? 0, y = oe.has(u.name) ? d : u.y ?? 0;
      if (!(m + f !== (n.x ?? 0) || y + g !== (n.y ?? 0))) if (u.direction === e) o = true;
      else {
        const b = f * r - g * i;
        b !== 0 && (a = b > 0 ? "cw" : "ccw");
      }
    }
    return a && !o ? a === "cw" ? "corner-cw" : "corner-ccw" : "straight";
  }
  function Z0(n, t) {
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
      d && yu(e, s, d) && (r |= l);
    }
    return r;
  }
  function yu(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of ru(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  const Q0 = 9079434;
  function tb(n, t, e) {
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
        if (!d || !yu(o, a, d)) continue;
        const u = o * S + S / 2, p = a * S + S / 2, f = S * 1.5, g = u + l * f, m = p + c * f, y = new ft();
        y.moveTo(u, p).lineTo(g, m).stroke({
          width: i,
          color: Q0,
          cap: "round"
        }), s.addChild(y);
      }
    }
  }
  function $a(n, t) {
    if (n.quality && (n = {
      ...n,
      quality: void 0
    }), rn.has(n.name)) {
      const s = J0(n, t.tileMap), i = k0(n.name, n.direction ?? "North", s);
      return pn(i, xt, xt, (r) => {
        const o = xt / S;
        let a = null;
        s === "corner-cw" ? a = {
          turn: "cw"
        } : s === "corner-ccw" && (a = {
          turn: "ccw"
        });
        const l = ka(n, a);
        l.scale.set(o), r.addChild(l);
      });
    }
    if (me.has(n.name)) {
      const s = n.io_type ?? "input", i = P0(n.name, n.direction ?? "North", s);
      return pn(i, xt, xt, (r) => {
        const o = xt / S, a = Gn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (oe.has(n.name)) {
      const s = I0(n.name, n.direction ?? "North"), i = n.direction === "North" || n.direction === "South";
      return xc(s, i ? 2 : 1, i ? 1 : 2, (a, l, c) => {
        const h = xt / S, d = Gn(n, t);
        d.scale.set(h), d.x = 0, d.y = 0, a.addChild(d);
      });
    }
    if (n.name === "pipe") {
      const s = Z0(n, t), i = M0(s);
      return pn(i, xt, xt, (r) => {
        const o = xt / S, a = Gn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (n.name === "pipe-to-ground") {
      const s = B0(n.direction ?? "North");
      return pn(s, xt, xt, (i) => {
        const r = xt / S, o = Gn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Cr.has(n.name)) {
      const s = R0(n.name, n.direction ?? "North");
      return pn(s, xt, xt, (i) => {
        const r = xt / S, o = Gn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Sa.has(n.name)) {
      const s = $0(n.name);
      return pn(s, xt, xt, (i) => {
        const r = xt / S, o = Gn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Me.has(n.name)) {
      const [s, i] = Ee[n.name] ?? [
        1,
        1
      ], r = L0(n.name);
      return xc(r, s, i, (o, a, l) => {
        const c = `/spaghettio/pr-333/entity-frames/${n.name}.png`, h = ie.has(c) ? Oe.get(c) ?? null : null, d = 1.8, u = xt / j0;
        if (h) {
          const p = new We(h);
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
    return pn(e, xt, xt, (s) => {
      const i = xt / S, r = Gn(n, t);
      r.scale.set(i), r.x = 0, r.y = 0, s.addChild(r);
    });
  }
  function Jo(n, t, e, s = Sr()) {
    const i = ps(t);
    if (le.has(i)) return;
    const r = t.x ?? 0, o = t.y ?? 0, a = $a(t, s);
    let l = S / xt, c = S / xt;
    if (oe.has(t.name)) {
      const g = t.direction === "North" || t.direction === "South", m = g ? 2 : 1, y = g ? 1 : 2;
      l = m * S / (m * xt), c = y * S / (y * xt);
    } else if (Me.has(t.name)) {
      const [g, m] = Ee[t.name] ?? [
        1,
        1
      ];
      l = g * S / (g * xt), c = m * S / (m * xt);
    }
    const h = r * S, d = o * S, u = new ei({
      texture: a,
      x: h,
      y: d,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: l,
      scaleY: c
    });
    Me.has(t.name) ? n.machineContainer.addParticle(u) : n.beltContainer.addParticle(u);
    let p;
    if (t.carries && !Me.has(t.name)) {
      const g = Ko(t.carries), m = S * 0.35, y = (S - m) / 2;
      p = new ei({
        texture: g,
        x: h + y,
        y: d + y,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: m / xt,
        scaleY: m / xt
      }), n.iconContainer.addParticle(p);
    }
    let f;
    if (t.quality && t.quality !== "normal") {
      const g = Ko(`quality-${t.quality}`), m = S * 0.4;
      let y = 1;
      Me.has(t.name) ? y = (Ee[t.name] ?? [
        1,
        1
      ])[1] : t.name === "substation" && (y = 2);
      const b = 1.5;
      f = new ei({
        texture: g,
        x: h + b,
        y: d + y * S - m - b,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: m / xt,
        scaleY: m / xt
      }), n.iconContainer.addParticle(f);
    }
    le.set(i, {
      entity: u,
      icon: p,
      badge: f,
      revealAt: e,
      placedEntity: t
    });
  }
  const Es = /* @__PURE__ */ new Map();
  function eb(n, t, e, s, i, r, o) {
    const a = `${t},${e}`, l = Es.get(a);
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
    }, h = Sr();
    lr(c, h);
    const d = $a(c, h), u = $n(i), p = new ei({
      texture: d,
      x: t * S,
      y: e * S,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: S / xt,
      scaleY: S / xt,
      tint: u
    });
    return n.ghostContainer.addParticle(p), l || Es.set(a, {
      particle: p,
      specKey: o
    }), p;
  }
  function kc(n, t, e) {
    const s = `${t},${e}`, i = Es.get(s);
    i && (n.ghostContainer.removeParticle(i.particle), Es.delete(s));
  }
  function nb(n) {
    n.ghostContainer.removeParticles(), Es.clear();
  }
  function sb(n, t, e) {
    const s = [];
    for (const [i, r] of le.entries()) r.placedEntity.x !== t || r.placedEntity.y !== e || (Me.has(r.placedEntity.name) ? n.machineContainer.removeParticle(r.entity) : n.beltContainer.removeParticle(r.entity), r.icon && n.iconContainer.removeParticle(r.icon), le.delete(i), s.push(i));
    return s;
  }
  function ib(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const [s, i] of le.entries()) {
      const r = i.placedEntity.name;
      if (r !== "pipe" && !rn.has(r)) continue;
      const o = $a(i.placedEntity, t);
      if (i.entity.texture === o) continue;
      const a = i.entity, l = new ei({
        texture: o,
        x: a.x,
        y: a.y,
        alpha: a.alpha,
        anchorX: a.anchorX,
        anchorY: a.anchorY,
        scaleX: a.scaleX,
        scaleY: a.scaleY
      });
      n.beltContainer.removeParticle(a), n.beltContainer.addParticle(l), le.set(s, {
        ...i,
        entity: l
      }), e.set(a, l);
    }
    return e;
  }
  function xu(n, t) {
    for (const e of le.values()) {
      const s = Math.min(1, Math.max(0, (t - e.revealAt) / mu));
      e.entity.alpha = s, e.icon && (e.icon.alpha = s), e.badge && (e.badge.alpha = s);
    }
  }
  function* Mc(n) {
    for (const t of le.values()) yield {
      particle: t.entity,
      iconParticle: t.icon,
      badgeParticle: t.badge,
      revealAt: t.revealAt
    };
  }
  function bu(n) {
    const t = /* @__PURE__ */ new Map();
    let e = false;
    function s() {
      e || (e = true, n.add(r), Tr());
    }
    function i() {
      e && (e = false, n.remove(r), Jn());
    }
    function r() {
      const c = performance.now();
      let h = false;
      for (const [d, u] of t) {
        const p = c - u.startTime, f = Math.min(1, p / u.duration), g = u.ease(f), m = u.startAlpha + (u.targetAlpha - u.startAlpha) * g;
        u.entityParticle.alpha = m, u.iconParticle && (u.iconParticle.alpha = m), u.badgeParticle && (u.badgeParticle.alpha = m), f >= 1 ? t.delete(d) : h = true;
      }
      h || i(), Xe();
    }
    function o(c, h, d, u, p) {
      const f = t.get(c), g = performance.now();
      let m;
      if (f) {
        const b = g - f.startTime, x = Math.min(1, b / f.duration);
        m = f.startAlpha + (f.targetAlpha - f.startAlpha) * f.ease(x);
      } else m = h.alpha;
      if (Math.abs(m - p) < 1e-3) {
        t.delete(c), h.alpha = p, d && (d.alpha = p), u && (u.alpha = p);
        return;
      }
      const y = p > m;
      t.set(c, {
        entityParticle: h,
        iconParticle: d,
        badgeParticle: u,
        startAlpha: m,
        targetAlpha: p,
        startTime: g,
        duration: y ? V0 : X0,
        ease: y ? q0 : K0
      }), s();
    }
    function a(c) {
      t.clear();
      for (const h of le.values()) h.entity.alpha = c, h.icon && (h.icon.alpha = c), h.badge && (h.badge.alpha = c);
      i(), Xe();
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
  function _u(n) {
    let t = 0;
    for (const i of n.values()) i > t && (t = i);
    const e = Math.max(t, Y0), s = /* @__PURE__ */ new Map();
    for (const [i, r] of n) {
      const o = cr + (1 - cr) * (1 - r / e);
      s.set(i, o);
    }
    return s;
  }
  function In(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function rb(n, t, e) {
    t.clear(), t.layout = n;
    const s = Sr();
    for (const l of n.entities) lr(l, s);
    const i = 0;
    for (const l of n.entities) Jo(t, l, i, s);
    tb(t, n, s), xu(t, mu + 1);
    const r = pu(n);
    if (!e) return ob();
    const o = bu(e.ticker);
    function a(l) {
      const c = _u(l);
      for (const h of le.values()) {
        const d = In(h.placedEntity), u = c.get(d) ?? cr;
        o.animateTo(d, h.entity, h.icon, h.badge, u);
      }
    }
    return {
      highlightItem(l) {
        if (o.cancelAll(1), !!l) for (const c of le.values()) {
          const h = c.placedEntity, u = (h.carries ?? h.recipe ?? null) === l ? 1 : 0.15, p = In(h);
          o.animateTo(p, c.entity, c.icon, c.badge, u);
        }
      },
      highlightBeltNetwork(l) {
        if (!l) {
          o.cancelAll(1);
          return;
        }
        const c = In(l), h = fu(r, c);
        a(h);
      },
      clearHighlight() {
        for (const l of le.values()) {
          const c = In(l.placedEntity);
          o.animateTo(c, l.entity, l.icon, l.badge, 1);
        }
      },
      chainKey(l) {
        return l.carries ?? l.recipe ?? null;
      }
    };
  }
  function ob() {
    function n() {
      for (const t of le.values()) t.entity.alpha = 1, t.icon && (t.icon.alpha = 1), t.badge && (t.badge.alpha = 1);
    }
    return {
      highlightItem(t) {
        if (n(), !!t) for (const e of le.values()) {
          const s = e.placedEntity, r = (s.carries ?? s.recipe ?? null) === t ? 1 : 0.15;
          e.entity.alpha = r, e.icon && (e.icon.alpha = r), e.badge && (e.badge.alpha = r);
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
  function ab(n, t) {
    const e = pu(n), s = bu(t.ticker);
    function i(r) {
      const o = In(r), a = fu(e, o), l = _u(a);
      for (const c of le.values()) {
        const h = In(c.placedEntity), d = l.get(h) ?? cr;
        s.animateTo(h, c.entity, c.icon, c.badge, d);
      }
    }
    return {
      highlightItem(r) {
        if (s.cancelAll(1), !!r) for (const o of le.values()) {
          const a = o.placedEntity, c = (a.carries ?? a.recipe ?? null) === r ? 1 : 0.15;
          o.entity.alpha = c, o.icon && (o.icon.alpha = c), o.badge && (o.badge.alpha = c);
        }
      },
      highlightBeltNetwork(r) {
        if (!r) {
          for (const o of le.values()) {
            const a = In(o.placedEntity);
            s.animateTo(a, o.entity, o.icon, o.badge, 1);
          }
          return;
        }
        i(r);
      },
      clearHighlight() {
        for (const r of le.values()) {
          const o = In(r.placedEntity);
          s.animateTo(o, r.entity, r.icon, r.badge, 1);
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
  const Pc = 6, lb = 18, cb = 2, hb = 26, db = -Math.PI / 3, Ic = 2, ub = 16777215, pb = 0.55, Rc = 4, fb = 0, mb = 0.6;
  function gb(n) {
    return n.carries ? rn.has(n.name) || me.has(n.name) || ci.has(n.name) : false;
  }
  function yb(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const l of t.external_inputs) e.set(l.item, {
      rate: l.rate,
      isFluid: !!l.is_fluid
    });
    if (e.size === 0) return [];
    const s = /* @__PURE__ */ new Map();
    for (const l of n.entities) {
      if (!gb(l)) continue;
      const c = l.carries;
      if (!e.has(c)) continue;
      const h = l.x ?? 0, d = l.y ?? 0, u = s.get(h);
      (!u || d < u.y) && s.set(h, {
        y: d,
        carries: c
      });
    }
    if (s.size === 0) return [];
    const i = Array.from(s.entries()).filter(([, l]) => l.y === fb).map(([l, c]) => ({
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
  function xb(n) {
    return `${n.toFixed(1)}/s`;
  }
  function bb(n) {
    const t = n.xMax - n.xMin + 1, e = Math.min(hb, lb + (t - 1) * cb), s = new zt();
    s.eventMode = "none";
    const i = Ko(n.item), r = new We(i);
    r.width = e, r.height = e, r.x = 0, r.y = -e / 2, s.addChild(r);
    const o = new Ge({
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
    }), a = new Ve({
      text: xb(n.rate),
      style: o
    });
    a.x = e + Pc, a.y = -a.height / 2, s.addChild(a);
    const l = new Ge({
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
    }), c = new Ve({
      text: ge(n.item),
      style: l
    });
    return c.alpha = mb, c.x = a.x + a.width + Pc, c.y = -c.height / 2, s.addChild(c), s;
  }
  function _b(n, t, e) {
    if (n.removeChildren(), !e) return;
    const s = yb(t, e);
    if (s.length !== 0) for (const i of s) {
      const r = i.topY * S - Ic, o = i.xMin * S + Rc, a = (i.xMax - i.xMin + 1) * S - 2 * Rc;
      if (a > 0) {
        const d = new ft();
        d.rect(o, r, a, Ic).fill({
          color: ub,
          alpha: pb
        }), n.addChild(d);
      }
      const l = bb(i);
      l.rotation = db;
      const c = (i.xMin + i.xMax + 1) / 2 * S, h = i.topY * S - S * 0.5;
      l.x = c, l.y = h, n.addChild(l);
    }
  }
  function wb(n) {
    return oe.has(n.name) ? n.direction === "East" || n.direction === "West" ? [
      1,
      2
    ] : [
      2,
      1
    ] : Ee[n.name] ?? [
      1,
      1
    ];
  }
  const oo = 57504;
  function Lc(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map();
    for (const v of s.entities) r.set(`${v.x ?? 0},${v.y ?? 0}`, v);
    let o = null, a = false, l = [];
    const c = new ft();
    e.addChild(c);
    const h = new ft();
    e.addChild(h);
    function d(v, w) {
      const C = n.getBoundingClientRect();
      return t.toWorld(v - C.left, w - C.top);
    }
    function u(v, w) {
      if (!o) return;
      const C = d(o.sx, o.sy), T = d(v, w), P = Math.min(C.x, T.x), E = Math.min(C.y, T.y), A = Math.abs(T.x - C.x), D = Math.abs(T.y - C.y);
      c.clear(), c.rect(P, E, A, D).fill({
        color: oo,
        alpha: 0.18
      }), c.setStrokeStyle({
        width: 1,
        color: oo,
        alpha: 0.8
      }), c.rect(P, E, A, D).stroke(), Xe();
    }
    function p(v) {
      if (h.clear(), v.length !== 0) {
        h.setStrokeStyle({
          width: 1.5,
          color: oo,
          alpha: 0.9
        });
        for (const w of v) {
          const [C, T] = wb(w), P = (w.x ?? 0) * S + 1, E = (w.y ?? 0) * S + 1;
          h.rect(P, E, C * S - 2, T * S - 2).stroke();
        }
      }
    }
    function f(v, w) {
      if (!o) return [];
      const C = d(o.sx, o.sy), T = d(v, w), P = Math.min(Math.floor(C.x / S), Math.floor(T.x / S)), E = Math.max(Math.floor(C.x / S), Math.floor(T.x / S)), A = Math.min(Math.floor(C.y / S), Math.floor(T.y / S)), D = Math.max(Math.floor(C.y / S), Math.floor(T.y / S)), O = [];
      for (let H = P; H <= E; H++) for (let G = A; G <= D; G++) {
        const L = r.get(`${H},${G}`);
        L && O.push(L);
      }
      return O;
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
        if (a) v.stopImmediatePropagation(), c.clear(), l = f(v.clientX, v.clientY), p(l), i(l), Xe();
        else if (o !== null) {
          const w = d(v.clientX, v.clientY), C = Math.floor(w.x / S), T = Math.floor(w.y / S);
          r.has(`${C},${T}`) && (l = [], h.clear(), i([]), Xe());
        }
        o = null, a = false;
      }
    };
    function b() {
      l = [], c.clear(), h.clear(), i([]), Xe();
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
  const vb = {
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
  }, Cb = {
    codes: vb
  }, wu = Cb, Sb = new Map(Object.entries(wu.codes)), Tb = new Map(Object.entries(wu.codes).map(([n, t]) => [
    t,
    n
  ]));
  function Eb(n) {
    return Sb.get(n) ?? null;
  }
  function Ab(n) {
    return Tb.get(n) ?? null;
  }
  const kb = [
    "partitioned-per-consumer",
    "partitioned-decomposed"
  ], Mb = [
    "horizontal-stack"
  ], Pb = [
    "regular",
    "fast"
  ], Ib = [
    "uncommon",
    "rare",
    "epic",
    "legendary"
  ], Rb = [
    "tree"
  ], Ba = [
    "2",
    "3",
    "4"
  ], Oa = /^[sp][1-3][urel]?$/, ao = [
    "iron-plate",
    "copper-plate",
    "steel-plate",
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], As = [
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], Na = "iron-gear-wheel", Fa = 10, _s = {
    crafting: "assembling-machine-3",
    smelting: "electric-furnace"
  }, fi = {
    crafting: "craft",
    smelting: "smelt"
  }, Lb = "machine", hr = "#/l/", Xs = "_", dr = ",", $b = {
    pd: "partitioned-decomposed"
  }, $c = {
    "partitioned-decomposed": "pd"
  }, Bb = {
    hs: "horizontal-stack"
  }, Bc = {
    "horizontal-stack": "hs"
  }, Ob = {
    r: "regular",
    f: "fast"
  }, Oc = {
    regular: "r",
    fast: "f"
  }, Nb = {
    u: "uncommon",
    r: "rare",
    e: "epic",
    l: "legendary"
  }, Nc = {
    uncommon: "u",
    rare: "r",
    epic: "e",
    legendary: "l"
  }, Fb = {
    t: "tree"
  }, Fc = {
    tree: "t"
  }, vu = "st";
  function cs(n) {
    return Eb(n) ?? n;
  }
  function hs(n) {
    return Ab(n);
  }
  function Wb() {
    const n = window.location.hash;
    if (!n.startsWith(hr)) return null;
    const t = n.slice(hr.length), e = t.indexOf("?"), s = e >= 0 ? t.slice(0, e) : t, i = e >= 0 ? t.slice(e + 1) : "", r = s.split("/"), o = (U) => {
      const Z = r[U];
      return Z === void 0 || Z === "" || Z === Xs ? null : Z;
    }, a = o(0);
    let l;
    if (a) {
      const U = hs(a);
      if (U === null) return null;
      l = U;
    } else l = Na;
    const c = o(1), h = c !== null ? parseFloat(c) : NaN, d = !isNaN(h) && h > 0 ? h : Fa, u = o(2), p = {};
    if (u) {
      const U = hs(u);
      if (U === null) return null;
      p.crafting = U;
    }
    const f = o(3);
    let g;
    if (f) {
      const U = f.split(dr).filter((M) => M.length > 0), Z = [];
      for (const M of U) {
        const B = hs(M);
        if (B === null) return null;
        Z.push(B);
      }
      g = Z;
    } else g = As;
    const m = o(4);
    let y = null;
    if (m && (y = hs(m), y === null)) return null;
    const b = new URLSearchParams(i), x = b.get("s"), _ = x ? $b[x] ?? null : null, v = b.get("rl"), w = v ? Bb[v] ?? null : null, C = b.get("it"), T = C ? Ob[C] ?? null : null, P = b.get("q"), E = P ? Nb[P] ?? null : null, A = b.get("w"), D = A ? Fb[A] ?? null : null, O = b.get(vu), H = O && Ba.includes(O) ? O : null, G = b.get("m"), L = G && Oa.test(G) ? G : null, $ = b.get("ci");
    let K = [];
    if ($) {
      const U = $.split(dr).filter((Z) => Z.length > 0);
      for (const Z of U) {
        const M = hs(Z);
        if (M === null) return null;
        K.push(M);
      }
    }
    for (const [U, Z] of Object.entries(fi)) {
      if (U === "crafting") continue;
      const M = b.get(Z);
      if (!M) continue;
      const B = hs(M);
      if (B === null) return null;
      p[U] = B;
    }
    return {
      item: l,
      rate: d,
      machines: p,
      inputs: g,
      belt: y,
      strategy: _,
      rowLayout: w,
      inserterTier: T,
      quality: E,
      wireMode: D,
      stacking: H,
      modules: L,
      customInputs: K
    };
  }
  function Gb() {
    const n = new URLSearchParams(window.location.search), t = n.get("item") ?? Na, e = parseFloat(n.get("rate") ?? ""), s = isNaN(e) || e <= 0 ? Fa : e, i = {};
    for (const [P, E] of Object.entries(fi)) {
      const A = n.get(E);
      A && (i[P] = A);
    }
    const r = n.get(Lb);
    r && !i.crafting && (i.crafting = r);
    const o = n.get("in"), a = o ? o.split(",").filter((P) => P.length > 0) : As, l = n.get("belt"), c = n.get("strategy");
    let h = c && kb.includes(c) ? c : null;
    h === "partitioned-per-consumer" && (h = "partitioned-decomposed");
    const d = n.get("row_layout"), u = d && Mb.includes(d) ? d : null, p = n.get("inserter_tier"), f = p && Pb.includes(p) ? p : null, g = n.get("wire_mode"), m = g && Rb.includes(g) ? g : null, y = n.get("quality"), b = y && Ib.includes(y) ? y : null, x = n.get("stacking"), _ = x && Ba.includes(x) ? x : null, v = n.get("modules"), w = v && Oa.test(v) ? v : null, C = n.get("ci"), T = C ? C.split(",").filter((P) => P.length > 0) : [];
    return {
      item: t,
      rate: s,
      machines: i,
      inputs: a,
      belt: l,
      strategy: h,
      rowLayout: u,
      inserterTier: f,
      quality: b,
      wireMode: m,
      stacking: _,
      modules: w,
      customInputs: T
    };
  }
  function zb() {
    return Wb() ?? Gb();
  }
  function Db() {
    if (window.location.hash.startsWith(hr)) return true;
    const n = new URLSearchParams(window.location.search);
    return n.has("item") || n.has("rate") || n.has("machine") || n.has("in") || n.has("belt");
  }
  function Hb(n) {
    for (const [t, e] of Object.entries(fi)) {
      const s = n[t];
      if (s && s !== _s[t]) return false;
    }
    for (const t of Object.keys(n)) if (!(t in fi)) return false;
    return true;
  }
  function Ub(n) {
    const t = cs(n.item), e = String(n.rate), s = n.machines.crafting, i = !s || s === _s.crafting ? Xs : cs(s), r = n.inputs.length === As.length && n.inputs.every((u, p) => u === As[p]), o = n.inputs.length === 0 || r ? Xs : n.inputs.map(cs).join(dr), a = n.belt ? cs(n.belt) : Xs, l = new URLSearchParams();
    n.strategy && $c[n.strategy] && l.set("s", $c[n.strategy]), n.rowLayout && Bc[n.rowLayout] && l.set("rl", Bc[n.rowLayout]), n.inserterTier && Oc[n.inserterTier] && l.set("it", Oc[n.inserterTier]), n.quality && Nc[n.quality] && l.set("q", Nc[n.quality]), n.wireMode && Fc[n.wireMode] && l.set("w", Fc[n.wireMode]), n.stacking && Ba.includes(n.stacking) && l.set(vu, n.stacking), n.modules && Oa.test(n.modules) && l.set("m", n.modules), n.customInputs.length > 0 && l.set("ci", n.customInputs.map(cs).join(dr));
    for (const [u, p] of Object.entries(fi)) {
      if (u === "crafting") continue;
      const f = n.machines[u];
      f && f !== _s[u] && l.set(p, cs(f));
    }
    const c = [
      t,
      e,
      i,
      o,
      a
    ], h = l.toString();
    if (h.length === 0) for (; c.length > 2 && c[c.length - 1] === Xs; ) c.pop();
    let d = `${hr}${c.join("/")}`;
    return h.length > 0 && (d += `?${h}`), d;
  }
  function jb(n) {
    const t = n.item === Na && n.rate === Fa && Hb(n.machines) && n.inputs.length === As.length && n.inputs.every((s, i) => s === As[i]) && !n.belt && !n.strategy && !n.rowLayout && !n.inserterTier && n.customInputs.length === 0, e = window.location.pathname;
    if (t) {
      history.replaceState(null, "", e);
      return;
    }
    history.replaceState(null, "", e + Ub(n));
  }
  const lo = "[INCOMPATIBLE_MACHINE]";
  function gn(n, t = 14) {
    const e = document.createElement("img");
    return e.src = `/spaghettio/pr-333/icons/${n}.png`, e.width = t, e.height = t, e.style.cssText = "image-rendering:pixelated", e.onerror = () => {
      e.style.display = "none";
    }, e;
  }
  function Yb(n, t) {
    const e = document.createElement("option");
    return e.value = n, e.textContent = ge(n), n === t && (e.selected = true), e;
  }
  function Hi(n, t, e) {
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
  function Wc(n, t, e, s) {
    for (const i of t) {
      const r = document.createElement("div");
      r.className = `sb-machine-flow ${e}`, s && r.appendChild(document.createTextNode(s)), r.appendChild(gn(i.item, 13)), r.appendChild(document.createTextNode(ge(i.item)));
      const o = document.createElement("span");
      o.className = "flow-rate";
      const a = uu(i.rate), l = a ? du(a.color) : "#f88";
      o.style.color = l, o.textContent = `${i.rate.toFixed(1)}/s`, r.appendChild(o), n.appendChild(r);
    }
  }
  const Gc = /* @__PURE__ */ new Set([
    "water",
    "crude-oil",
    "petroleum-gas",
    "light-oil",
    "heavy-oil",
    "sulfuric-acid",
    "lubricant",
    "steam"
  ]), Cu = "spaghettio-recent-items", zc = 5;
  function Su() {
    try {
      const n = localStorage.getItem(Cu);
      return n ? JSON.parse(n) : [];
    } catch {
      return [];
    }
  }
  function Vb(n) {
    const t = Su().filter((e) => e !== n);
    t.unshift(n), t.length > zc && (t.length = zc);
    try {
      localStorage.setItem(Cu, JSON.stringify(t));
    } catch {
    }
  }
  function Xb(n, t, e) {
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
        a.appendChild(gn(s, 14));
        const x = document.createElement("span");
        x.textContent = ge(s), a.appendChild(x);
      } else {
        const x = document.createElement("span");
        x.className = "sb-picker-placeholder", x.textContent = "Select item\u2026", a.appendChild(x);
      }
    }
    function p(x) {
      const _ = document.createElement("div");
      _.className = "sb-picker-item" + (x === s ? " selected" : ""), _.dataset.slug = x, _.appendChild(gn(x, 14));
      const v = document.createElement("span");
      return v.textContent = ge(x), _.appendChild(v), _.addEventListener("mousedown", (w) => {
        w.preventDefault(), g(x);
      }), _;
    }
    function f(x) {
      d.innerHTML = "", r = null;
      const _ = x.trim().toLowerCase(), v = _ ? n.filter((w) => w.includes(_) || ge(w).toLowerCase().includes(_)) : n;
      if (!_) {
        const w = Su().filter((C) => n.includes(C));
        if (w.length > 0) {
          const C = document.createElement("div");
          C.className = "sb-picker-section-label", C.textContent = "Recent", d.appendChild(C);
          for (const P of w) d.appendChild(p(P));
          const T = document.createElement("div");
          T.className = "sb-picker-divider", d.appendChild(T);
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
      s = x, Vb(x), o.classList.remove("item-invalid"), u(), y(), e(x);
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
  function qb(n, t, e) {
    var _a2;
    n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "sidebar-inner";
    const { section: i, body: r } = Hi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="2"/></svg>', "Target"), o = t.allProducibleItems(), a = new Set(o);
    function l(V, nt) {
      const tt = document.createElement("div");
      tt.className = "sb-field";
      const Ct = document.createElement("span");
      return Ct.className = "sb-field-label", Ct.textContent = V, tt.appendChild(Ct), nt.style.flex = "1", nt.style.minWidth = "0", tt.appendChild(nt), tt;
    }
    const c = Xb(o, "", () => Bt());
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
    for (const V of h) {
      const nt = document.createElement("select");
      nt.className = "sb-select", nt.dataset.cat = V.category;
      const tt = _s[V.category] ?? "";
      for (const Ct of V.options) {
        const Ut = Yb(Ct.value, tt);
        Ct.disabled && (Ut.disabled = true), Ct.title && (Ut.title = Ct.title), nt.appendChild(Ut);
      }
      r.appendChild(l(V.label, nt)), u.set(V.category, nt);
    }
    const p = u.get("crafting");
    for (const V of d) {
      const nt = document.createElement("span");
      nt.className = "sb-machine-readonly", nt.textContent = ge(V.machine), r.appendChild(l(V.label, nt));
    }
    function f() {
      const V = {};
      for (const [nt, tt] of u) tt.value && (V[nt] = tt.value);
      return V;
    }
    const g = document.createElement("select");
    g.className = "sb-select", [
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
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, g.appendChild(tt);
    }), r.appendChild(l("Belt", g));
    const m = document.createElement("select");
    m.className = "sb-select", [
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
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, m.appendChild(tt);
    }), r.appendChild(l("Inserter tier", m));
    const y = document.createElement("select");
    y.className = "sb-select", [
      [
        "Normal (default)",
        ""
      ],
      [
        "Uncommon",
        "uncommon"
      ],
      [
        "Rare",
        "rare"
      ],
      [
        "Epic",
        "epic"
      ],
      [
        "Legendary",
        "legendary"
      ]
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, y.appendChild(tt);
    }), r.appendChild(l("Build quality", y));
    const b = document.createElement("select");
    b.className = "sb-select", [
      [
        "None (default)",
        ""
      ],
      [
        "Speed 1",
        "s1"
      ],
      [
        "Speed 2",
        "s2"
      ],
      [
        "Speed 3",
        "s3"
      ],
      [
        "Productivity 1",
        "p1"
      ],
      [
        "Productivity 2",
        "p2"
      ],
      [
        "Productivity 3",
        "p3"
      ]
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, b.appendChild(tt);
    }), r.appendChild(l("Modules", b));
    const x = document.createElement("select");
    x.className = "sb-select", [
      [
        "Normal (default)",
        ""
      ],
      [
        "Uncommon",
        "u"
      ],
      [
        "Rare",
        "r"
      ],
      [
        "Epic",
        "e"
      ],
      [
        "Legendary",
        "l"
      ]
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, x.appendChild(tt);
    }), r.appendChild(l("Module quality", x));
    const _ = () => b.value ? `${b.value}${x.value}` : null, v = document.createElement("select");
    v.className = "sb-select", [
      [
        "Dense (default)",
        ""
      ],
      [
        "Minimal tree",
        "tree"
      ]
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, v.appendChild(tt);
    }), r.appendChild(l("Pole wiring", v));
    const w = document.createElement("select");
    w.className = "sb-select", w.title = "Belt stack size (Space Age research): multiplies belt capacity", [
      [
        "Off (default)",
        ""
      ],
      [
        "\xD72",
        "2"
      ],
      [
        "\xD73",
        "3"
      ],
      [
        "\xD74",
        "4"
      ]
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, w.appendChild(tt);
    }), r.appendChild(l("Belt stacking", w));
    const C = document.createElement("select");
    C.className = "sb-select", [
      [
        "Pooled (default)",
        ""
      ],
      [
        "Partitioned + decomposed",
        "partitioned-decomposed"
      ]
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, C.appendChild(tt);
    }), r.appendChild(l("Strategy", C));
    const T = document.createElement("select");
    T.className = "sb-select", [
      [
        "Vertical split (today)",
        ""
      ],
      [
        "Horizontal stack (RFC)",
        "horizontal-stack"
      ]
    ].forEach(([V, nt]) => {
      const tt = document.createElement("option");
      tt.value = nt, tt.textContent = V, T.appendChild(tt);
    }), r.appendChild(l("Row layout", T));
    const P = document.createElement("div");
    P.className = "sb-field";
    const E = document.createElement("span");
    E.className = "sb-field-label", E.textContent = "Rate", P.appendChild(E);
    const A = document.createElement("input");
    A.type = "number", A.className = "sb-input", A.step = "0.5", A.min = "0.1", A.style.cssText = "flex:1;min-width:0", A.placeholder = "10", P.appendChild(A);
    const D = document.createElement("span");
    D.className = "sb-rate-suffix", D.textContent = "/s", P.appendChild(D), r.appendChild(P), s.appendChild(i);
    const { section: O, body: H } = Hi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="6" rx="1"/><line x1="5" y1="8" x2="11" y2="8"/></svg>', "Inputs"), G = document.createElement("div");
    G.className = "sb-tags";
    const L = /* @__PURE__ */ new Map();
    ao.forEach((V) => {
      const nt = document.createElement("label");
      nt.className = `sb-tag${Gc.has(V) ? " fluid" : ""}`;
      const tt = document.createElement("span");
      tt.className = "sb-tag-check", tt.textContent = "\u2713";
      const Ct = document.createElement("input");
      Ct.type = "checkbox", Ct.value = V, Ct.style.display = "none", L.set(V, Ct), nt.appendChild(tt), nt.appendChild(gn(V, 14)), nt.appendChild(document.createTextNode(ge(V))), nt.appendChild(Ct), Ct.addEventListener("change", () => {
        nt.classList.toggle("active", Ct.checked);
      }), G.appendChild(nt);
    }), H.appendChild(G);
    let $ = [];
    const K = document.createElement("div");
    K.className = "sb-tags sb-custom-tags", H.appendChild(K);
    const U = document.createElement("datalist");
    U.id = "spaghettio-custom-inputs-datalist";
    const Z = new Set(ao);
    o.filter((V) => !Z.has(V)).forEach((V) => {
      const nt = document.createElement("option");
      nt.value = V, U.appendChild(nt);
    }), H.appendChild(U);
    const M = document.createElement("input");
    M.type = "text", M.className = "sb-input sb-custom-input-field", M.setAttribute("list", "spaghettio-custom-inputs-datalist"), M.autocomplete = "off", M.placeholder = "+ add input\u2026", H.appendChild(M);
    function B(V) {
      const nt = document.createElement("div");
      nt.className = `sb-tag sb-custom-tag active${Gc.has(V) ? " fluid" : ""}`, nt.dataset.item = V, nt.appendChild(gn(V, 14)), nt.appendChild(document.createTextNode(ge(V)));
      const tt = document.createElement("span");
      tt.className = "sb-tag-remove", tt.textContent = "\xD7", tt.addEventListener("click", (Ct) => {
        Ct.stopPropagation(), $ = $.filter((Ut) => Ut !== V), nt.remove(), Bt();
      }), nt.appendChild(tt), K.appendChild(nt);
    }
    function N(V) {
      const nt = V.trim();
      !nt || !a.has(nt) || Z.has(nt) || $.includes(nt) || ($.push(nt), B(nt), M.value = "", Bt());
    }
    M.addEventListener("keydown", (V) => {
      V.key === "Enter" && N(M.value);
    }), M.addEventListener("change", () => {
      N(M.value);
    }), s.appendChild(O);
    const { section: j, body: J, countEl: st } = Hi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>', "Solver", ""), ht = document.createElement("div");
    J.appendChild(ht), s.appendChild(j);
    const ut = document.createElement("div");
    ut.className = "sb-actions", ut.style.display = "none";
    const mt = document.createElement("button");
    mt.className = "sb-btn sb-btn-secondary", mt.textContent = "Copy Blueprint", mt.style.flex = "1", ut.appendChild(mt);
    const bt = document.createElement("div");
    bt.className = "sb-copy-status", ut.appendChild(bt), J.appendChild(ut);
    const { section: X, body: it, countEl: rt } = Hi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.8" fill="currentColor" stroke="none"/></svg>', "Validation", "");
    X.style.display = "none", s.appendChild(X), n.appendChild(s);
    const et = zb();
    c.setValue(et.item), A.value = String(et.rate);
    const St = /* @__PURE__ */ new Set([
      "assembling-machine-1",
      "assembling-machine-2",
      "assembling-machine-3"
    ]);
    for (const [V, nt] of u) {
      const tt = et.machines[V], Ct = new Set(Array.from(nt.options).filter((Ut) => !Ut.disabled).map((Ut) => Ut.value));
      nt.value = tt && Ct.has(tt) ? tt : _s[V] ?? ((_a2 = nt.options[0]) == null ? void 0 : _a2.value) ?? "";
    }
    L.forEach((V, nt) => {
      V.checked = et.inputs.includes(nt);
      const tt = V.closest(".sb-tag");
      tt && tt.classList.toggle("active", V.checked);
    }), et.belt && (g.value = et.belt), et.strategy && (C.value = et.strategy), et.rowLayout && (T.value = et.rowLayout), et.inserterTier && (m.value = et.inserterTier), et.quality && (y.value = et.quality), et.modules && (b.value = et.modules.slice(0, 2), x.value = et.modules.slice(2)), et.wireMode && (v.value = et.wireMode), et.stacking && (w.value = et.stacking);
    for (const V of et.customInputs) a.has(V) && !Z.has(V) && !$.includes(V) && ($.push(V), B(V));
    const Lt = document.createElement("div");
    Lt.className = "sb-config-error", Lt.style.display = "none", ht.before(Lt);
    function Ht(V) {
      V ? (Lt.textContent = V, Lt.style.display = "") : (Lt.textContent = "", Lt.style.display = "none");
    }
    let Vt = null, Xt = et.item, Zt = null, Et = 0;
    function Bt() {
      Vt !== null && clearTimeout(Vt), Vt = setTimeout(() => {
        qt().catch((V) => console.error("runSolve failed:", V));
      }, 150);
    }
    async function qt() {
      var _a3;
      const V = c.getValue(), nt = parseFloat(A.value), tt = ao.filter((z) => {
        var _a4;
        return (_a4 = L.get(z)) == null ? void 0 : _a4.checked;
      }), Ct = [
        ...tt,
        ...$
      ];
      if (!a.has(V)) {
        c.setInvalid(true);
        return;
      }
      if (c.setInvalid(false), isNaN(nt) || nt <= 0) return;
      if (V !== Xt) {
        const z = t.defaultMachineForItem(V, p.value);
        St.has(z) && (p.value = z), Xt = V;
      }
      const Ut = f();
      jb({
        item: V,
        rate: nt,
        machines: Ut,
        inputs: tt,
        belt: g.value || null,
        strategy: C.value || null,
        rowLayout: T.value || null,
        inserterTier: m.value || null,
        quality: y.value || null,
        wireMode: v.value || null,
        stacking: w.value || null,
        modules: _(),
        customInputs: $
      });
      const ye = ++Et;
      ht.innerHTML = "", Ht(null), Zt = null, ut.style.display = "none";
      let Gt;
      try {
        Gt = await t.solve(V, nt, Ct, Ut, Ut.crafting ?? _s.crafting, y.value || void 0, _() ?? void 0);
      } catch (z) {
        if (ye !== Et) return;
        e.renderGraph(null), st && (st.textContent = "error");
        const Q = String(z instanceof Error ? z.message : z);
        if (Q.includes(lo)) {
          const R = Q.indexOf(lo), F = Q.slice(R + lo.length).trim();
          Ht(F);
        } else {
          const R = document.createElement("div");
          R.className = "sb-result-error", R.textContent = Q, ht.appendChild(R);
        }
        return;
      }
      if (ye !== Et) return;
      Kb(ht, Gt), e.renderGraph(Gt);
      const re = Gt.machines.reduce((z, Q) => z + Math.ceil(Q.count), 0);
      st && (st.textContent = `${re} machines`);
      const k = /* @__PURE__ */ new Set();
      for (const z of Gt.machines) {
        for (const Q of z.inputs) k.add(Q.item);
        for (const Q of z.outputs) k.add(Q.item);
      }
      for (const z of Gt.external_inputs) k.add(z.item);
      for (const z of Gt.external_outputs) k.add(z.item);
      if (await iu(Array.from(k)), ye !== Et) return;
      let I;
      try {
        const z = g.value || void 0, Q = C.value || void 0, R = T.value || void 0, F = m.value || void 0, Y = y.value || void 0, q = v.value || void 0, ot = w.value || void 0, gt = e.startStreaming();
        I = await t.buildLayoutStreaming(Gt, z, Q, R, F, Y, q, ot, gt);
      } catch (z) {
        if (ye !== Et) return;
        const Q = document.createElement("div");
        Q.className = "sb-result-error", Q.textContent = `Layout error: ${z}`, ht.appendChild(Q);
        return;
      }
      ye === Et && (Zt = I, Yd(Gt.machines), e.renderLayout(I, Gt), ut.style.display = ((_a3 = I.warnings) == null ? void 0 : _a3.length) ? "none" : "flex");
    }
    mt.addEventListener("click", async () => {
      if (!Zt) return;
      const V = await t.exportBlueprint(Zt, c.getValue());
      await navigator.clipboard.writeText(V), bt.textContent = "Copied!", setTimeout(() => {
        bt.textContent = "";
      }, 2e3);
    }), A.addEventListener("input", Bt);
    for (const V of u.values()) V.addEventListener("change", Bt);
    return g.addEventListener("change", Bt), C.addEventListener("change", Bt), T.addEventListener("change", Bt), m.addEventListener("change", Bt), y.addEventListener("change", Bt), b.addEventListener("change", Bt), x.addEventListener("change", Bt), v.addEventListener("change", Bt), w.addEventListener("change", Bt), L.forEach((V) => V.addEventListener("change", Bt)), qt().catch((V) => console.error("runSolve failed:", V)), {
      getParams() {
        const V = c.getValue(), nt = parseFloat(A.value);
        return !V || isNaN(nt) || nt <= 0 ? null : {
          item: V,
          rate: nt
        };
      },
      setParams(V, nt) {
        c.setValue(V.item), A.value = String(V.rate), V.machine && St.has(V.machine) ? p.value = V.machine : p.value = "assembling-machine-3", V.inputs && L.forEach((tt, Ct) => {
          tt.checked = V.inputs.includes(Ct);
          const Ut = tt.closest(".sb-tag");
          Ut && Ut.classList.toggle("active", tt.checked);
        }), V.belt ? g.value = V.belt : g.value = "", K.innerHTML = "", $ = [];
        for (const tt of V.customInputs ?? []) a.has(tt) && !Z.has(tt) && !$.includes(tt) && ($.push(tt), B(tt));
        Xt = V.item, (nt == null ? void 0 : nt.skipAutoSolve) || Bt();
      },
      updateValidation(V, nt, tt) {
        if (it.innerHTML = "", V.length === 0) {
          X.style.display = "none", rt && (rt.textContent = "");
          return;
        }
        X.style.display = "";
        const Ct = V.filter((Gt) => Gt.severity === "Error").length, Ut = V.length - Ct;
        rt && (Ct > 0 ? (rt.textContent = `${Ct} error${Ct !== 1 ? "s" : ""}`, rt.style.color = "#f66") : (rt.textContent = `${Ut} warning${Ut !== 1 ? "s" : ""}`, rt.style.color = "#fa0"));
        const ye = /* @__PURE__ */ new Map();
        for (const Gt of V) {
          let re = ye.get(Gt.category);
          re || (re = [], ye.set(Gt.category, re)), re.push(Gt);
        }
        for (const [Gt, re] of ye) {
          const I = re.some((pt) => pt.severity === "Error") ? "#f44" : "#fa0", z = re.find((pt) => pt.x != null && pt.y != null), Q = document.createElement("div");
          Q.className = "sb-val-group";
          const R = document.createElement("div");
          R.className = "sb-val-group-header";
          const F = document.createElement("span");
          F.className = "sb-val-group-chevron", F.textContent = "\u25BE", F.addEventListener("click", (pt) => {
            pt.stopPropagation();
            const yt = gt.style.display === "none";
            gt.style.display = yt ? "" : "none", F.textContent = yt ? "\u25BE" : "\u25B8";
          }), R.appendChild(F);
          const Y = document.createElement("span");
          Y.className = "sb-val-group-dot", Y.style.background = I, R.appendChild(Y);
          const q = document.createElement("span");
          q.className = "sb-val-group-name", q.textContent = Gt, R.appendChild(q);
          const ot = document.createElement("span");
          ot.className = "sb-val-group-count", ot.textContent = String(re.length), R.appendChild(ot);
          const gt = document.createElement("div");
          if (gt.className = "sb-val-group-body", tt) {
            const pt = re.filter((yt) => yt.detail != null);
            if (pt.length > 0) {
              const yt = /* @__PURE__ */ new Map();
              for (const se of pt) {
                const ue = tt(se) ?? "unattributed";
                yt.set(ue, (yt.get(ue) ?? 0) + 1);
              }
              const kt = document.createElement("div");
              kt.className = "sb-val-cause-rollup", kt.style.cssText = "font-size:11px;color:#b90;padding:1px 0 2px 22px", kt.textContent = [
                ...yt.entries()
              ].sort((se, ue) => ue[1] - se[1]).map(([se, ue]) => `${ue} \xD7 ${se}`).join(" \xB7 "), gt.appendChild(kt);
            }
          }
          z && (R.classList.add("clickable"), R.addEventListener("click", () => {
            nt(z.x, z.y);
          }));
          for (const pt of re) {
            const yt = document.createElement("div"), kt = pt.x != null && pt.y != null;
            yt.className = "sb-val-issue" + (kt ? " clickable" : "");
            const se = document.createElement("span");
            if (se.className = "sb-val-issue-msg", se.textContent = pt.message, yt.appendChild(se), kt) {
              const ue = document.createElement("span");
              ue.className = "sb-val-issue-coord", ue.textContent = `${pt.x}, ${pt.y}`, yt.appendChild(ue), yt.addEventListener("click", (bi) => {
                bi.stopPropagation();
                const Le = yt.classList.contains("pinned");
                it.querySelectorAll(".sb-val-issue.pinned").forEach((vn) => vn.classList.remove("pinned")), Le || yt.classList.add("pinned"), nt(pt.x, pt.y);
              });
            } else yt.style.opacity = "0.6";
            gt.appendChild(yt);
          }
          Q.appendChild(R), Q.appendChild(gt), it.appendChild(Q);
        }
      }
    };
  }
  function Kb(n, t) {
    if (t.external_inputs.length > 0) {
      const l = document.createElement("div");
      l.className = "sb-ext-section-title", l.textContent = "External inputs", n.appendChild(l);
      for (const h of t.external_inputs) {
        const d = document.createElement("div");
        d.className = "sb-ext-flow", d.appendChild(gn(h.item, 14)), d.appendChild(document.createTextNode(ge(h.item)));
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
      u.className = "sb-machine-group-header", u.appendChild(gn(l, 16));
      const p = document.createElement("span");
      p.className = "sb-machine-group-name", p.textContent = ge(l), u.appendChild(p);
      const f = document.createElement("span");
      f.className = "sb-machine-group-count", f.textContent = `\xD7${h}`, u.appendChild(f), d.appendChild(u);
      const g = document.createElement("div");
      g.className = "sb-machine-group-body";
      for (const m of c) {
        const y = document.createElement("div");
        y.className = "sb-machine-flow", y.style.cssText = "color:#6b7280;margin-bottom:2px", y.appendChild(document.createTextNode("\u2192 ")), y.appendChild(gn(m.recipe, 13)), y.appendChild(document.createTextNode(ge(m.recipe))), g.appendChild(y), Wc(g, m.inputs, "flow-in", "\u25B6 "), Wc(g, m.outputs, "flow-out", "\u25C0 ");
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
      c.style.cssText = "display:flex;align-items:center;gap:4px;padding:2px 0;font-size:11px;color:#b5cea8", c.appendChild(gn(l.item, 13)), c.appendChild(document.createTextNode(ge(l.item)));
      const h = uu(l.rate);
      if (h) {
        const d = du(h.color);
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
  let Fe = null, Wa = 0;
  const es = /* @__PURE__ */ new Map();
  let de = null;
  const Zo = "spaghettio:sat-cache:v1";
  let mn = new Uint8Array(0);
  function Jb() {
    try {
      const n = localStorage.getItem(Zo);
      if (!n) return new Uint8Array(0);
      const t = atob(n), e = new Uint8Array(t.length);
      for (let s = 0; s < t.length; s++) e[s] = t.charCodeAt(s);
      return e;
    } catch (n) {
      return console.warn("[engine] could not read SAT cache from localStorage", n), new Uint8Array(0);
    }
  }
  function Zb(n) {
    let t = "";
    for (let i = 0; i < n.length; i += 8192) t += String.fromCharCode.apply(null, Array.from(n.subarray(i, i + 8192)));
    const s = btoa(t);
    try {
      localStorage.setItem(Zo, s);
    } catch (i) {
      if (i instanceof DOMException && (i.name === "QuotaExceededError" || i.code === 22)) {
        console.warn("[engine] SAT cache quota exceeded \u2014 clearing");
        try {
          localStorage.removeItem(Zo);
        } catch {
        }
        mn = new Uint8Array(0);
      } else console.warn("[engine] failed to persist SAT cache", i);
    }
  }
  function Qb(n) {
    const t = new Uint8Array(mn.length + n.length);
    t.set(mn, 0), t.set(n, mn.length), mn = t, Zb(mn);
  }
  let Qo = [], ta = [], Tu = /* @__PURE__ */ new Map(), Eu = /* @__PURE__ */ new Map(), ea = /* @__PURE__ */ new Set(), na = 0;
  function bn(n) {
    na += n;
    for (const t of ea) t(na);
  }
  function t_(n) {
    return ea.add(n), n(na), () => ea.delete(n);
  }
  function Se(n, t) {
    if (!Fe) throw new Error("Engine not initialized \u2014 call initEngine() first");
    const e = ++Wa;
    return bn(1), new Promise((s, i) => {
      es.set(e, {
        resolve: (r) => {
          bn(-1), de === e && (de = null), s(r);
        },
        reject: (r) => {
          bn(-1), de === e && (de = null), i(r);
        },
        onEvent: t
      }), Fe.postMessage({
        id: e,
        ...n
      });
    });
  }
  async function Au() {
    if (Fe) return;
    if (Fe = new Worker(new URL("/spaghettio/pr-333/assets/engine.worker-D6UavBWb.js", import.meta.url), {
      type: "module",
      name: "spaghettio-engine"
    }), Fe.onmessage = (e) => {
      const { id: s } = e.data, i = es.get(s);
      if (i) {
        if ("streamEvents" in e.data) {
          if (globalThis.__TRACE_LOGS === true) {
            const r = {};
            for (const o of e.data.streamEvents) {
              const a = o.phase ?? "?";
              r[a] = (r[a] ?? 0) + 1;
            }
            console.log(`[main  t=${performance.now().toFixed(0)}ms] arrived ${e.data.streamEvents.length}:`, r);
          }
          if (i.onEvent) for (const r of e.data.streamEvents) i.onEvent(r);
          return;
        }
        es.delete(s), e.data.ok ? i.resolve(e.data.result) : i.reject(new Error(e.data.error));
      }
    }, Fe.onerror = (e) => {
      console.error("[engine.worker] error", e);
    }, await Se({
      method: "init"
    }), mn = Jb(), mn.length > 0) try {
      const e = await Se({
        method: "seedZoneCache",
        bytes: mn
      });
      globalThis.__TRACE_LOGS === true && console.log(`[engine] seeded ${e} SAT zone records from localStorage`);
    } catch (e) {
      console.warn("[engine] seedZoneCache failed; persistence disabled this session", e);
    }
    Qo = await Se({
      method: "allProducibleItems"
    }), ta = await Se({
      method: "allProducerMachines"
    });
    const n = await Se({
      method: "defaultMachinesForItems",
      items: Qo,
      fallback: "assembling-machine-3"
    });
    Tu = new Map(n);
    const t = await Se({
      method: "moduleSlotsForEntities",
      entities: [
        ...ta,
        "beacon",
        "lab",
        "biolab",
        "electric-mining-drill",
        "big-mining-drill",
        "pumpjack"
      ]
    });
    Eu = new Map(t);
  }
  async function e_(n, t, e, s, i, r, o) {
    return de !== null && await Ga(), Se({
      method: "solve",
      targetItem: n,
      targetRate: t,
      availableInputs: e,
      palette: s,
      defaultMachine: i,
      quality: r ?? null,
      modules: o ?? null
    });
  }
  function n_() {
    return Qo;
  }
  function s_() {
    return ta;
  }
  function i_(n, t, e, s, i, r, o, a) {
    return Se({
      method: "layout",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null,
      quality: r ?? null,
      wireMode: o ?? null,
      stacking: a ?? null
    });
  }
  function r_(n, t, e, s, i, r, o, a) {
    return Se({
      method: "layoutTraced",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null,
      quality: r ?? null,
      wireMode: o ?? null,
      stacking: a ?? null
    });
  }
  async function Ga() {
    if (!Fe) return;
    Fe.terminate(), Fe = null;
    const n = new Error("Engine superseded by a newer request");
    for (const [, t] of es) t.reject(n);
    es.clear(), de = null, await Au();
  }
  async function o_(n, t, e, s, i, r, o, a, l) {
    de !== null && await Ga();
    const c = ++Wa;
    return de = c, bn(1), new Promise((h, d) => {
      es.set(c, {
        resolve: (p) => {
          bn(-1), de === c && (de = null), h(p);
        },
        reject: (p) => {
          bn(-1), de === c && (de = null), d(p);
        },
        onEvent: l
      });
      const u = globalThis.__TRACE_LOGS === true;
      Fe.postMessage({
        id: c,
        method: "layoutStreaming",
        result: n,
        maxBeltTier: t ?? null,
        strategy: e ?? null,
        rowLayout: s ?? null,
        maxInserterTier: i ?? null,
        quality: r ?? null,
        wireMode: o ?? null,
        stacking: a ?? null,
        traceLogs: u
      });
    });
  }
  function a_(n, t) {
    return Se({
      method: "exportBlueprint",
      layout: n,
      label: t
    });
  }
  function l_(n, t) {
    return Tu.get(n) ?? t;
  }
  function c_(n) {
    return Eu.get(n) ?? 0;
  }
  function h_(n, t) {
    return Se({
      method: "validateLayout",
      layout: n,
      solverResult: t
    });
  }
  function d_(n, t) {
    return Se({
      method: "solveFixture",
      fixtureJson: n,
      pinsJson: JSON.stringify(t)
    });
  }
  function u_(n) {
    return Se({
      method: "parseBlueprint",
      bp: n
    });
  }
  async function ku(n, t, e, s, i = 0) {
    if (de !== null && await Ga(), !Fe) throw new Error("Engine not initialized");
    const r = ++Wa;
    return de = r, bn(1), new Promise((o, a) => {
      es.set(r, {
        resolve: (l) => {
          bn(-1), de === r && (de = null), o(l);
        },
        reject: (l) => {
          bn(-1), de === r && (de = null), a(l);
        },
        onEvent: (l) => {
          const c = l;
          if (c.phase === "SatImprovement" && c.data) s(c.data);
          else if (c.phase === "SatOptimumProven" && c.data) {
            const h = c.data, d = h.record_bytes instanceof Uint8Array ? h.record_bytes : new Uint8Array(h.record_bytes);
            Qb(d);
          }
        }
      }), Fe.postMessage({
        id: r,
        method: "improveRegionStreaming",
        layout: n,
        regionId: t,
        budgetMs: e,
        maxIters: i
      });
    });
  }
  async function p_(n, t) {
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
        e = await ku(e, o, t.perRegionBudgetMs, (l) => {
          var _a2;
          l.iter > 0 && (a = true), (_a2 = t.onImprovement) == null ? void 0 : _a2.call(t, l);
        }, 1), a ? i += 1 : s.delete(o);
      }
      if (i === 0) break;
    }
    return e;
  }
  function f_(n, t) {
    return Se({
      method: "balancerShowcase",
      maxInputs: n,
      maxOutputs: t
    });
  }
  function Mu() {
    return {
      solve: e_,
      allProducibleItems: n_,
      allProducerMachines: s_,
      buildLayout: i_,
      buildLayoutTraced: r_,
      buildLayoutStreaming: o_,
      exportBlueprint: a_,
      defaultMachineForItem: l_,
      moduleSlots: c_,
      validateLayout: h_,
      solveFixture: d_,
      improveRegion: ku,
      optimizeAllRegions: p_,
      balancerShowcase: f_
    };
  }
  const m_ = `
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
  function g_() {
    if (document.getElementById("spaghettio-corpus-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-corpus-style", n.textContent = m_, document.head.appendChild(n);
  }
  function y_(n, t) {
    g_(), n.innerHTML = "";
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
      const M = m.value.trim(), B = ++b;
      if (!M) {
        m.classList.remove("error"), y.style.display = "none";
        return;
      }
      u_(M).then((N) => {
        B === b && (m.classList.remove("error"), y.style.display = "none", t(N));
      }).catch((N) => {
        B === b && (m.classList.add("error"), y.textContent = String(N), y.style.display = "block");
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
    const T = document.createElement("label");
    T.textContent = "Product";
    const P = document.createElement("select");
    C.appendChild(T), C.appendChild(P), x.appendChild(C);
    const E = document.createElement("div");
    E.className = "corpus-filter-row";
    const A = document.createElement("input");
    A.type = "checkbox";
    const D = document.createElement("label");
    D.style.display = "flex", D.style.alignItems = "center", D.style.gap = "5px", D.style.cursor = "pointer", D.appendChild(A), D.appendChild(document.createTextNode("Bus layouts only")), E.appendChild(D), x.appendChild(E), e.appendChild(x);
    const O = document.createElement("div");
    O.className = "corpus-count", O.style.display = "none", e.appendChild(O);
    const H = document.createElement("div");
    H.className = "corpus-list", e.appendChild(H);
    const G = document.createElement("div");
    G.className = "corpus-stats", e.appendChild(G);
    function L() {
      i = s.filter((M) => !(o && !M.stats.is_bus_layout || a && a !== "__all__" && M.stats.final_product !== a || l && !M.name.toLowerCase().includes(l.toLowerCase()))), r = -1, $();
    }
    function $() {
      if (H.innerHTML = "", O.textContent = `${i.length} of ${s.length} blueprint(s)`, i.length === 0) {
        const M = document.createElement("div");
        M.className = "corpus-empty", M.textContent = s.length === 0 ? "No corpus loaded yet." : "No blueprints match the current filters.", H.appendChild(M), G.classList.remove("visible");
        return;
      }
      for (let M = 0; M < i.length; M++) {
        const B = i[M], N = document.createElement("div");
        N.className = "corpus-item" + (M === r ? " selected" : "");
        const j = document.createElement("div");
        j.className = "corpus-item-name", j.textContent = B.name, j.title = B.name, N.appendChild(j);
        const J = document.createElement("div");
        if (J.className = "corpus-item-meta", B.stats.is_bus_layout) {
          const ut = document.createElement("span");
          ut.className = "corpus-badge corpus-badge-bus", ut.textContent = "BUS", J.appendChild(ut);
        }
        if (B.stats.final_product) {
          const ut = document.createElement("span");
          ut.className = "corpus-badge corpus-badge-product", ut.textContent = B.stats.final_product, J.appendChild(ut);
        }
        const st = document.createElement("span");
        st.className = "corpus-badge corpus-badge-machines", st.textContent = `${B.stats.machine_count}m`, J.appendChild(st), N.appendChild(J);
        const ht = M;
        N.addEventListener("click", () => K(ht)), H.appendChild(N);
      }
    }
    function K(M) {
      r = M;
      const B = i[M];
      $(), t(B.layout), U(B);
    }
    function U(M) {
      G.innerHTML = "", G.classList.add("visible");
      const B = document.createElement("div");
      B.className = "corpus-stats-title", B.textContent = M.name, B.title = M.name, G.appendChild(B);
      const N = document.createElement("div");
      N.className = "corpus-stats-grid";
      const j = M.stats, J = [
        [
          "machines",
          String(j.machine_count)
        ],
        [
          "recipes",
          String(j.recipe_count)
        ],
        [
          "is_bus",
          j.is_bus_layout ? "yes" : "no"
        ],
        [
          "density",
          j.density.toFixed(2)
        ]
      ];
      j.is_bus_layout && J.push([
        "bus_lanes",
        String(j.bus_lane_count)
      ], [
        "bus_pitch",
        j.bus_pitch.toFixed(1)
      ], [
        "row_pitch",
        j.row_pitch.toFixed(1)
      ], [
        "rows",
        String(j.row_count)
      ]), J.push([
        "bbox",
        `${j.bbox_width}\xD7${j.bbox_height}`
      ], [
        "belt_tiles",
        String(j.belt_tiles)
      ]), j.pipe_tiles > 0 && J.push([
        "pipe_tiles",
        String(j.pipe_tiles)
      ]);
      for (const [st, ht] of J) {
        const ut = document.createElement("div");
        ut.className = "corpus-stats-row";
        const mt = document.createElement("span");
        mt.className = "corpus-stats-key", mt.textContent = st;
        const bt = document.createElement("span");
        bt.className = "corpus-stats-val", bt.textContent = ht, ut.appendChild(mt), ut.appendChild(bt), N.appendChild(ut);
      }
      G.appendChild(N);
    }
    function Z() {
      const M = new Set(s.map((N) => N.stats.final_product).filter(Boolean));
      P.innerHTML = "";
      const B = document.createElement("option");
      B.value = "__all__", B.textContent = "All products", P.appendChild(B);
      for (const N of Array.from(M).sort()) {
        const j = document.createElement("option");
        j.value = N, j.textContent = N, P.appendChild(j);
      }
    }
    u.addEventListener("change", () => {
      var _a2;
      const M = (_a2 = u.files) == null ? void 0 : _a2[0];
      if (!M) return;
      const B = new FileReader();
      B.onload = (N) => {
        var _a3;
        try {
          s = JSON.parse((_a3 = N.target) == null ? void 0 : _a3.result).blueprints ?? [], i = s, r = -1, p.textContent = `Reload corpus.json (${s.length} blueprints)`, x.style.display = "", O.style.display = "", G.classList.remove("visible"), Z(), L();
        } catch (j) {
          alert(`Failed to parse corpus.json: ${j}`);
        }
      }, B.readAsText(M);
    }), w.addEventListener("input", () => {
      l = w.value, L();
    }), P.addEventListener("change", () => {
      a = P.value, L();
    }), A.addEventListener("change", () => {
      o = A.checked, L();
    }), $();
  }
  const x_ = [
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
  ], b_ = `
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
  function __() {
    if (document.getElementById("spaghettio-landing-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-landing-style", n.textContent = b_, document.head.appendChild(n);
  }
  function w_(n, t, e) {
    __(), n.innerHTML = "";
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
    for (const u of x_) {
      const p = document.createElement("div");
      p.className = `spaghettio-landing-card ${u.status}`;
      const f = document.createElement("div");
      f.className = "spaghettio-landing-tier", f.innerHTML = `<span>${u.tier}</span>`, p.appendChild(f);
      const g = document.createElement("div");
      g.className = "spaghettio-landing-card-body";
      const m = document.createElement("div");
      m.className = "spaghettio-landing-card-title";
      const y = document.createElement("img");
      y.src = `/spaghettio/pr-333/icons/${u.item}.png`, y.className = "spaghettio-landing-card-icon", y.onerror = () => {
        y.style.display = "none";
      }, m.appendChild(y), m.appendChild(document.createTextNode(u.label));
      const b = document.createElement("span");
      b.className = "spaghettio-landing-card-rate", b.textContent = `${u.rate}/s`, m.appendChild(b), g.appendChild(m);
      const x = document.createElement("div");
      x.className = "spaghettio-landing-card-desc", x.textContent = u.desc, g.appendChild(x), p.appendChild(g);
      const _ = document.createElement("div");
      _.className = `spaghettio-landing-status ${u.status}`, _.textContent = u.status === "solved" ? "Solved" : u.status === "partial" ? "Partial" : "WIP", p.appendChild(_);
      const v = document.createElement("div");
      v.className = "spaghettio-landing-entities", v.textContent = "\u2014", p.appendChild(v), u.status !== "wip" && p.addEventListener("click", () => {
        v_(t, u, p, v);
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
  function v_(n, t, e, s) {
    e.classList.contains("loading") || (e.classList.add("loading"), s.innerHTML = '<span class="spaghettio-spinner"></span>', (async () => {
      let i, r;
      try {
        const o = n.defaultMachineForItem(t.item, t.machine);
        i = await n.solve(t.item, t.rate, t.inputs, {}, o), r = await n.buildLayout(i, t.beltTier);
      } catch (o) {
        e.classList.remove("loading"), s.textContent = "error", console.error("Landing solve/layout failed:", o);
        return;
      }
      s.textContent = String(r.entities.length), e.classList.remove("loading"), Yd(i.machines.map((o) => ({
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
      }))), C_(t, r, i).catch((o) => {
        console.error("Modal init failed:", o);
      });
    })());
  }
  async function C_(n, t, e) {
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
    d.src = `/spaghettio/pr-333/icons/${n.item}.png`, d.onerror = () => {
      d.style.display = "none";
    }, h.appendChild(d), h.appendChild(document.createTextNode(` ${n.label} \u2014 ${n.rate}/s`)), c.appendChild(h);
    const u = document.createElement("div");
    u.className = "spaghettio-preview-stats";
    const p = `${t.width ?? 0}\xD7${t.height ?? 0}`, f = e.machines.reduce((T, P) => T + Math.ceil(P.count), 0);
    u.innerHTML = `<span>${f} machines</span><span>${p} tiles</span>`, c.appendChild(u);
    const g = document.createElement("button");
    g.className = "spaghettio-preview-close", g.textContent = "\xD7", g.addEventListener("click", a), c.appendChild(g), l.appendChild(c);
    const m = document.createElement("div");
    m.className = "spaghettio-preview-canvas", l.appendChild(m);
    const y = document.createElement("div");
    y.className = "spaghettio-preview-badge", y.textContent = `0 / ${t.entities.length}`, m.appendChild(y), o = new da(), await o.init({
      resizeTo: m,
      background: 1118481,
      antialias: true
    }), m.insertBefore(o.canvas, y), o.canvas.addEventListener("contextmenu", (T) => T.preventDefault());
    const b = (t.width ?? 20) * S, x = (t.height ?? 20) * S, _ = Math.max(b, x, 600) + 200, v = new Dd({
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
    const { renderLayoutAnimated: C } = await Rn(async () => {
      const { renderLayoutAnimated: T } = await import("./animated-DpErhEvI.js");
      return {
        renderLayoutAnimated: T
      };
    }, []);
    C(t, w, y, () => {
    });
  }
  var ke = Uint8Array, ys = Uint16Array, S_ = Int32Array, Pu = new ke([
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
  ]), Iu = new ke([
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
  ]), T_ = new ke([
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
  ]), Ru = function(n, t) {
    for (var e = new ys(31), s = 0; s < 31; ++s) e[s] = t += 1 << n[s - 1];
    for (var i = new S_(e[30]), s = 1; s < 30; ++s) for (var r = e[s]; r < e[s + 1]; ++r) i[r] = r - e[s] << 5 | s;
    return {
      b: e,
      r: i
    };
  }, Lu = Ru(Pu, 2), $u = Lu.b, E_ = Lu.r;
  $u[28] = 258, E_[258] = 28;
  var A_ = Ru(Iu, 0), k_ = A_.b, sa = new ys(32768);
  for (var jt = 0; jt < 32768; ++jt) {
    var Tn = (jt & 43690) >> 1 | (jt & 21845) << 1;
    Tn = (Tn & 52428) >> 2 | (Tn & 13107) << 2, Tn = (Tn & 61680) >> 4 | (Tn & 3855) << 4, sa[jt] = ((Tn & 65280) >> 8 | (Tn & 255) << 8) >> 1;
  }
  var si = (function(n, t, e) {
    for (var s = n.length, i = 0, r = new ys(t); i < s; ++i) n[i] && ++r[n[i] - 1];
    var o = new ys(t);
    for (i = 1; i < t; ++i) o[i] = o[i - 1] + r[i - 1] << 1;
    var a;
    if (e) {
      a = new ys(1 << t);
      var l = 15 - t;
      for (i = 0; i < s; ++i) if (n[i]) for (var c = i << 4 | n[i], h = t - n[i], d = o[n[i] - 1]++ << h, u = d | (1 << h) - 1; d <= u; ++d) a[sa[d] >> l] = c;
    } else for (a = new ys(s), i = 0; i < s; ++i) n[i] && (a[i] = sa[o[n[i] - 1]++] >> 15 - n[i]);
    return a;
  }), xi = new ke(288);
  for (var jt = 0; jt < 144; ++jt) xi[jt] = 8;
  for (var jt = 144; jt < 256; ++jt) xi[jt] = 9;
  for (var jt = 256; jt < 280; ++jt) xi[jt] = 7;
  for (var jt = 280; jt < 288; ++jt) xi[jt] = 8;
  var Bu = new ke(32);
  for (var jt = 0; jt < 32; ++jt) Bu[jt] = 5;
  var M_ = si(xi, 9, 1), P_ = si(Bu, 5, 1), co = function(n) {
    for (var t = n[0], e = 1; e < n.length; ++e) n[e] > t && (t = n[e]);
    return t;
  }, Ue = function(n, t, e) {
    var s = t / 8 | 0;
    return (n[s] | n[s + 1] << 8) >> (t & 7) & e;
  }, ho = function(n, t) {
    var e = t / 8 | 0;
    return (n[e] | n[e + 1] << 8 | n[e + 2] << 16) >> (t & 7);
  }, I_ = function(n) {
    return (n + 7) / 8 | 0;
  }, R_ = function(n, t, e) {
    return (e == null || e > n.length) && (e = n.length), new ke(n.subarray(t, e));
  }, L_ = [
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
  ], je = function(n, t, e) {
    var s = new Error(t || L_[n]);
    if (s.code = n, Error.captureStackTrace && Error.captureStackTrace(s, je), !e) throw s;
    return s;
  }, $_ = function(n, t, e, s) {
    var i = n.length, r = 0;
    if (!i || t.f && !t.l) return e || new ke(0);
    var o = !e, a = o || t.i != 2, l = t.i;
    o && (e = new ke(i * 3));
    var c = function(X) {
      var it = e.length;
      if (X > it) {
        var rt = new ke(Math.max(it * 2, X));
        rt.set(e), e = rt;
      }
    }, h = t.f || 0, d = t.p || 0, u = t.b || 0, p = t.l, f = t.d, g = t.m, m = t.n, y = i * 8;
    do {
      if (!p) {
        h = Ue(n, d, 1);
        var b = Ue(n, d + 1, 3);
        if (d += 3, b) if (b == 1) p = M_, f = P_, g = 9, m = 5;
        else if (b == 2) {
          var w = Ue(n, d, 31) + 257, C = Ue(n, d + 10, 15) + 4, T = w + Ue(n, d + 5, 31) + 1;
          d += 14;
          for (var P = new ke(T), E = new ke(19), A = 0; A < C; ++A) E[T_[A]] = Ue(n, d + A * 3, 7);
          d += C * 3;
          for (var D = co(E), O = (1 << D) - 1, H = si(E, D, 1), A = 0; A < T; ) {
            var G = H[Ue(n, d, O)];
            d += G & 15;
            var x = G >> 4;
            if (x < 16) P[A++] = x;
            else {
              var L = 0, $ = 0;
              for (x == 16 ? ($ = 3 + Ue(n, d, 3), d += 2, L = P[A - 1]) : x == 17 ? ($ = 3 + Ue(n, d, 7), d += 3) : x == 18 && ($ = 11 + Ue(n, d, 127), d += 7); $--; ) P[A++] = L;
            }
          }
          var K = P.subarray(0, w), U = P.subarray(w);
          g = co(K), m = co(U), p = si(K, g, 1), f = si(U, m, 1);
        } else je(1);
        else {
          var x = I_(d) + 4, _ = n[x - 4] | n[x - 3] << 8, v = x + _;
          if (v > i) {
            l && je(0);
            break;
          }
          a && c(u + _), e.set(n.subarray(x, v), u), t.b = u += _, t.p = d = v * 8, t.f = h;
          continue;
        }
        if (d > y) {
          l && je(0);
          break;
        }
      }
      a && c(u + 131072);
      for (var Z = (1 << g) - 1, M = (1 << m) - 1, B = d; ; B = d) {
        var L = p[ho(n, d) & Z], N = L >> 4;
        if (d += L & 15, d > y) {
          l && je(0);
          break;
        }
        if (L || je(2), N < 256) e[u++] = N;
        else if (N == 256) {
          B = d, p = null;
          break;
        } else {
          var j = N - 254;
          if (N > 264) {
            var A = N - 257, J = Pu[A];
            j = Ue(n, d, (1 << J) - 1) + $u[A], d += J;
          }
          var st = f[ho(n, d) & M], ht = st >> 4;
          st || je(3), d += st & 15;
          var U = k_[ht];
          if (ht > 3) {
            var J = Iu[ht];
            U += ho(n, d) & (1 << J) - 1, d += J;
          }
          if (d > y) {
            l && je(0);
            break;
          }
          a && c(u + 131072);
          var ut = u + j;
          if (u < U) {
            var mt = r - U, bt = Math.min(U, ut);
            for (mt + u < 0 && je(3); u < bt; ++u) e[u] = s[mt + u];
          }
          for (; u < ut; ++u) e[u] = e[u - U];
        }
      }
      t.l = p, t.p = B, t.b = u, t.f = h, p && (h = 1, t.m = g, t.d = f, t.n = m);
    } while (!h);
    return u != e.length && o ? R_(e, 0, u) : e.subarray(0, u);
  }, B_ = new ke(0), O_ = function(n) {
    (n[0] != 31 || n[1] != 139 || n[2] != 8) && je(6, "invalid gzip data");
    var t = n[3], e = 10;
    t & 4 && (e += (n[10] | n[11] << 8) + 2);
    for (var s = (t >> 3 & 1) + (t >> 4 & 1); s > 0; s -= !n[e++]) ;
    return e + (t & 2);
  }, N_ = function(n) {
    var t = n.length;
    return (n[t - 4] | n[t - 3] << 8 | n[t - 2] << 16 | n[t - 1] << 24) >>> 0;
  };
  function F_(n, t) {
    var e = O_(n);
    return e + 8 > n.length && je(6, "invalid gzip data"), $_(n.subarray(e, -8), {
      i: 2
    }, new ke(N_(n)), t);
  }
  var W_ = typeof TextDecoder < "u" && new TextDecoder(), G_ = 0;
  try {
    W_.decode(B_, {
      stream: true
    }), G_ = 1;
  } catch {
  }
  const uo = "fls1";
  async function Ou(n) {
    const t = typeof n == "string" ? n : new TextDecoder().decode(n);
    if (!t.startsWith(uo)) throw new Error(`Not a layout snapshot: expected "${uo}" prefix, got "${t.slice(0, 4)}"`);
    const e = t.slice(uo.length), s = Uint8Array.from(atob(e), (o) => o.charCodeAt(0)), i = F_(s), r = new TextDecoder().decode(i);
    return JSON.parse(r);
  }
  function z_(n) {
    return new Promise((t, e) => {
      const s = new FileReader();
      s.onload = () => t(s.result), s.onerror = () => e(new Error("Failed to read file")), s.readAsText(n);
    });
  }
  function D_(n, t) {
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
          const i = await z_(s), r = await Ou(i);
          t(r);
        } catch (i) {
          alert(`Failed to load snapshot: ${i}`);
        }
      }
    });
  }
  function H_(n, t, e) {
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
  function Dc(n, t, e, s, i, r, o, a) {
    const l = s - t, c = i - e, h = Math.sqrt(l * l + c * c);
    if (h === 0) return;
    const d = l / h, u = c / h;
    let p = 0;
    for (; p < h; ) {
      const f = Math.min(p + r, h);
      n.moveTo(t + d * p, e + u * p).lineTo(t + d * f, e + u * f).stroke(a), p = f + o;
    }
  }
  function U_(n, t, e, s, i) {
    const r = new zt(), o = n.find((h) => h.phase === "LanesPlanned");
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
      const d = h.data, u = d.from_x * S + S / 2, p = d.from_y * S + S / 2, f = 3, g = new ft();
      g.label = "RouteFailure", g.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), g.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), Dc(g, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), g.eventMode = "static", g.on("pointerenter", () => i(`Route failed: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) [${d.spec_key}]`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    const l = j_;
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
      const d = h.data, u = d.from_x * S + S / 2, p = d.from_y * S + S / 2, f = 4, g = new ft();
      g.label = "RouteFailure", g.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), g.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), Dc(g, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), g.eventMode = "static", g.on("pointerenter", () => i(`Ghost failed: ${d.spec_key} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y})`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    for (const h of n) {
      if (h.phase !== "GhostClusterSolved" && h.phase !== "GhostClusterFailed") continue;
      const d = h.phase === "GhostClusterFailed", u = d ? null : h.data, p = d ? h.data : null, f = u ?? p, g = d ? 16729156 : 4500223, m = new ft();
      m.rect(f.zone_x * S, f.zone_y * S, f.zone_w * S, f.zone_h * S).fill({
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
  const j_ = [
    5676246,
    6996096,
    13672512,
    11567312
  ], Y_ = {
    Error: 16729156,
    Warning: 16755200
  }, V_ = 0.85;
  function X_(n, t, e) {
    const s = new zt(), i = /* @__PURE__ */ new Map();
    for (const r of n) {
      if (r.x == null || r.y == null) continue;
      const o = Y_[r.severity] ?? 4500223, a = new ft();
      a.rect(r.x * S, r.y * S, S, S).stroke({
        width: 2,
        color: o,
        alpha: V_
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
  function q_(n) {
    const t = Math.max(0, Math.min(1, n)), e = Math.round(64 + 96 * t);
    return {
      color: 255 << 16 | e << 8 | 32,
      alpha: 0.45 * (1 - t) + 0.12
    };
  }
  function K_(n, t, e) {
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
      const a = Ee[o.name];
      a && i.set(`${o.x},${o.y}`, a);
    }
    const r = new zt();
    r.eventMode = "none";
    for (const [o, a] of s) {
      const [l, c] = o.split(",").map(Number), h = i.get(o), [d, u] = h ?? [
        1,
        1
      ], { color: p, alpha: f } = q_(a), g = new ft();
      g.rect(l * S, c * S, d * S, u * S).fill({
        color: p,
        alpha: f
      }), r.addChild(g);
    }
    return e.addChild(r), r;
  }
  function Hc(n) {
    const [t, e] = Ee[n.name] ?? [
      1,
      1
    ], s = n.x ?? 0, i = n.y ?? 0;
    return {
      cx: (s + t / 2) * S,
      cy: (i + e / 2) * S
    };
  }
  function J_(n, t, e) {
    if (!n || n.length === 0) return null;
    const s = new zt();
    s.eventMode = "none";
    const i = new ft();
    for (const [r, o] of n) {
      const a = t[r], l = t[o];
      if (!a || !l) continue;
      const { cx: c, cy: h } = Hc(a), { cx: d, cy: u } = Hc(l);
      i.moveTo(c, h), i.lineTo(d, u);
    }
    return i.stroke({
      color: 16751164,
      width: 1.5,
      alpha: 0.85
    }), s.addChild(i), e.addChild(s), s;
  }
  function Z_(n) {
    return n.startsWith("speed-module") ? "speed" : n.startsWith("productivity-module") ? "productivity" : n.startsWith("efficiency-module") ? "efficiency" : n.startsWith("quality-module") ? "quality" : "unknown";
  }
  const Q_ = {
    speed: 4886736,
    productivity: 14238510,
    efficiency: 5025616,
    quality: 3127472,
    unknown: 9079434
  }, En = S * 0.16, po = S * 0.05, fo = 1.5;
  function tw(n, t, e) {
    const s = new zt();
    s.eventMode = "none";
    let i = false;
    for (const r of n) {
      if (!r.items || r.items.length === 0) continue;
      const o = r.items.reduce((x, _) => x + _.count, 0), a = Math.max(t(r.name), o);
      if (a === 0) continue;
      const [l, c] = Ee[r.name] ?? [
        1,
        1
      ], h = l * S, d = c * S, u = (r.x ?? 0) * S, p = (r.y ?? 0) * S, f = a * En + (a - 1) * po, g = u + Math.max(h - f - fo, fo), m = p + d - En - fo, y = new ft();
      let b = 0;
      for (const x of r.items) {
        const _ = Q_[Z_(x.item)], v = x.quality && x.quality !== "normal" ? au[x.quality] : void 0, w = v ?? 0, C = v !== void 0 ? 0.95 : 0.35, T = v !== void 0 ? 1 : 0.75;
        for (let P = 0; P < x.count && b < a; P++, b++) {
          const E = g + b * (En + po);
          y.rect(E, m, En, En).fill({
            color: _,
            alpha: 0.9
          }).stroke({
            color: w,
            width: T,
            alpha: C
          });
        }
      }
      for (; b < a; b++) {
        const x = g + b * (En + po);
        y.rect(x, m, En, En).fill({
          color: 1710618,
          alpha: 0.35
        }).stroke({
          color: 9079434,
          width: 0.75,
          alpha: 0.55
        });
      }
      s.addChild(y), i = true;
    }
    return i ? (e.addChild(s), s) : null;
  }
  function ew(n) {
    const t = n.point.direction;
    return t === "East" || t === "West" ? "horizontal" : "vertical";
  }
  function nw(n) {
    const t = new Set(n.map(ew));
    return t.size === 1 ? [
      ...t
    ][0] : "mixed";
  }
  function sw(n) {
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
    for (const a of e.values()) a.axis = nw([
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
  function iw(n) {
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
  function rw(n) {
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
  const ow = 0.35;
  function aw(n, t, e, s, i) {
    const r = S * 0.45;
    n.setStrokeStyle({
      width: 3,
      color: i,
      alpha: ow
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
  function mo(n) {
    return [
      n.point.x,
      n.point.y
    ];
  }
  function lw(n, t, e, s, i, r, o = 2, a = 6, l = 4, c = 0.9) {
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
  function cw(n) {
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
  function hw(n) {
    const t = new zt(), e = n.regions ?? [], s = [];
    if (e.length === 0) return {
      layer: t,
      items: s,
      hitTest: () => null
    };
    for (const r of e) {
      const o = sw(r), a = iw(r.kind), l = rw(o.cls), c = r.x * S, h = r.y * S, d = r.width * S, u = r.height * S;
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
      const g = r.ports ?? [], m = cw(g);
      for (const { item: y, inPort: b, outPort: x } of m) {
        const [_, v] = mo(b), [w, C] = mo(x), T = _ * S + S / 2, P = v * S + S / 2, E = w * S + S / 2, A = C * S + S / 2, D = $n(y), O = new ft();
        lw(O, T, P, E, A, D), t.addChild(O);
      }
      for (const y of g) {
        const [b, x] = mo(y), _ = b * S + S / 2, v = x * S + S / 2, w = new ft(), C = y.item ? $n(y.item) : 8947848;
        aw(w, _, v, y.point.direction, C), t.addChild(w);
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
  const dw = /* @__PURE__ */ new Set([
    "JunctionGrowthStarted",
    "JunctionGrowthIteration",
    "JunctionStrategyAttempt",
    "SatInvocation",
    "JunctionSolved",
    "JunctionGrowthCapped",
    "RegionWalkerVeto"
  ]);
  function uw(n) {
    return dw.has(n.phase);
  }
  function pw(n) {
    const t = n.data;
    return typeof t.seed_x == "number" && typeof t.seed_y == "number" ? [
      t.seed_x,
      t.seed_y
    ] : [
      t.tile_x,
      t.tile_y
    ];
  }
  function fw(n, t) {
    return `${n},${t}`;
  }
  function mw(n) {
    const t = /* @__PURE__ */ new Map(), e = (a, l) => {
      const c = fw(a, l);
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
      if (!uw(a)) continue;
      const [l, c] = pw(a), h = e(l, c);
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
  function Uc(n) {
    return n.iterations.length === 0 ? null : n.iterations[n.defaultIterIndex] ?? n.iterations[n.iterations.length - 1];
  }
  function gw(n) {
    return n ? `${n.entity_name}@(${n.entity_x},${n.entity_y}) ${n.direction}` : "";
  }
  function yw(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function xw(n, t, e) {
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
        item: yw(c.spec_key),
        specKey: c.spec_key,
        tiles: h
      });
    }
    return a;
  }
  const jc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, bw = new Ge({
    fontFamily: "monospace",
    fontSize: 10,
    fill: 16777215,
    dropShadow: {
      color: 0,
      distance: 1,
      blur: 2,
      alpha: 0.8
    }
  }), _w = new Ge({
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
  function ww(n) {
    const t = new zt(), e = [], s = [];
    for (const o of n) {
      if (o.outcome.kind !== "Solved") continue;
      const a = Uc(o);
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
      const a = Uc(o);
      if (!a) continue;
      const l = a.bbox;
      if (l.w <= 0 || l.h <= 0 || o.outcome.kind === "Capped" && i(o.seed.x, o.seed.y)) continue;
      const c = l.x * S, h = l.y * S, d = l.w * S, u = l.h * S, p = jc[o.outcome.kind] ?? jc.Open, f = new ft();
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
      const g = new Ve({
        text: `Junction (${o.seed.x},${o.seed.y})`,
        style: bw
      });
      g.x = c + 3, g.y = h + 2, t.addChild(g);
      const m = new Ve({
        text: vw(o),
        style: _w
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
  function vw(n) {
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
  const Yc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, Cw = 0.04, Sw = 9060416, Tw = 0.55, Ew = 4243680, Aw = 5592405, kw = 0.55, Mw = 16777215, Vc = 0.85, Pw = 16777215, Xc = new Ge({
    fontFamily: "monospace",
    fontSize: 7,
    fontWeight: "700",
    fill: Mw
  }), qc = /* @__PURE__ */ new Map();
  function Iw(n) {
    let t = qc.get(n);
    if (!t) {
      const s = `/spaghettio/pr-333/icons/${n}.png`;
      t = Oe.load(s).catch(() => null), qc.set(n, t);
    }
    return t;
  }
  function Rw() {
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
      const { cluster: a, iter: l } = r, c = l.bbox, h = Yc[a.outcome.kind] ?? Yc.Open, d = new ft();
      d.rect(c.x * S, c.y * S, c.w * S, c.h * S).fill({
        color: h,
        alpha: Cw
      }), n.addChild(d);
      const u = Lw(c.x * S, c.y * S, c.w * S, c.h * S, {
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
        color: Sw,
        alpha: Tw
      });
      for (const [m, y] of l.forbidden) $w(p, m * S, y * S, S);
      n.addChild(p);
      const f = new ft();
      f.circle((a.seed.x + 0.5) * S, (a.seed.y + 0.5) * S, S * 0.42).stroke({
        width: 3,
        color: Ew,
        alpha: 0.95
      }), n.addChild(f);
      const g = ((_a2 = l.sat) == null ? void 0 : _a2.boundaries) ?? l.boundaries;
      for (const m of g) Ow(n, m, o, () => t);
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
  function Lw(n, t, e, s, i) {
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
      const d = c - a, u = h - l, p = Math.hypot(d, u), f = d / p, g = u / p, m = i.dashLen + i.gapLen;
      for (let y = 0; y < p; y += m) {
        const b = Math.min(y + i.dashLen, p);
        r.moveTo(a + f * y, l + g * y).lineTo(a + f * b, l + g * b).stroke();
      }
    }
    return r;
  }
  function $w(n, t, e, s) {
    const i = s;
    n.moveTo(t, e + i).lineTo(t + i, e).stroke(), n.moveTo(t + i / 3, e + i).lineTo(t + i, e + 2 * i / 3).stroke(), n.moveTo(t, e + 2 * i / 3).lineTo(t + 2 * i / 3, e).stroke();
  }
  function Bw(n, t) {
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
  function Ow(n, t, e, s) {
    const i = t.x * S, r = t.y * S, o = S / 3, a = Bw(t.is_input, t.direction), l = a === "top" || a === "bottom";
    let c = i, h = r, d = S, u = S;
    a === "top" ? u = o : a === "bottom" ? (h = r + S - o, u = o) : (a === "left" || (c = i + S - o), d = o);
    const p = new ft();
    p.rect(c, h, d, u).fill({
      color: Aw,
      alpha: kw
    }), t.interior && (p.setStrokeStyle({
      width: 1,
      color: Pw,
      alpha: 0.5
    }), p.rect(c, h, d, u).stroke()), n.addChild(p);
    const f = Nw(t.direction), [g, m, y, b] = l ? [
      c + d / 6,
      h + u / 2,
      c + d * 5 / 6,
      h + u / 2
    ] : [
      c + d / 2,
      h + u / 6,
      c + d / 2,
      h + u * 5 / 6
    ], x = new Ve({
      text: f,
      style: Xc
    });
    x.anchor.set(0.5), x.x = g, x.y = m, x.alpha = Vc, n.addChild(x);
    const _ = new Ve({
      text: f,
      style: Xc
    });
    _.anchor.set(0.5), _.x = y, _.y = b, _.alpha = Vc, n.addChild(_);
    const v = c + d / 2, w = h + u / 2;
    Iw(t.item).then((C) => {
      if (e !== s() || !C) return;
      const T = new We(C);
      T.anchor.set(0.5), T.x = v, T.y = w;
      const P = o * 0.95;
      T.width = P, T.height = P, n.addChild(T);
    });
  }
  function Nw(n) {
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
  const Fw = 4247776, Ww = 0.18;
  function Gw(n) {
    if (!n || n.length === 0) return null;
    const t = /* @__PURE__ */ new Set();
    for (const i of n) if (i.phase === "GhostSpecRouted") for (const [r, o] of i.data.tiles) t.add(`${r},${o}`);
    if (t.size === 0) return null;
    const e = new zt();
    e.label = "ghost-tiles-overlay";
    const s = new ft();
    for (const i of t) {
      const [r, o] = i.split(","), a = Number(r), l = Number(o);
      s.rect(a * S, l * S, S, S).fill({
        color: Fw,
        alpha: Ww
      });
    }
    return e.addChild(s), e;
  }
  function zw(n, t, e) {
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
    const T = document.createElement("div");
    T.className = "jd-detail";
    const P = document.createElement("div");
    P.className = "jd-footer", P.textContent = "Esc close \xB7 \u2190/\u2192 step all \xB7 w/s iter \xB7 a/d variant \xB7 Home/End first/last", x.append(_, T, P), n.append(b, x);
    let E = null, A = 0, D = null, O = false;
    function H(X, it) {
      E = X, D = it ?? null, A = X.defaultIterIndex, s.classList.add("jd-open"), st(), J(), M();
    }
    function G() {
      E && ($(), E = null, D = null, s.classList.remove("jd-open"), e.onChange(null));
    }
    function L() {
      !E || O || (O = true, b.classList.add("jd-open"), x.classList.add("jd-open"), ht());
    }
    function $() {
      O && (O = false, b.classList.remove("jd-open"), x.classList.remove("jd-open"));
    }
    function K() {
      O ? $() : L();
    }
    function U() {
      return E !== null;
    }
    function Z(X) {
      if (!E) return;
      const it = Math.max(0, Math.min(E.iterations.length - 1, X));
      it !== A && (A = it, st(), J(), M());
    }
    function M() {
      if (!E) return;
      const X = E.iterations[A];
      if (!X) return;
      const it = t.toScreen(X.bbox.x * S, X.bbox.y * S), rt = t.toScreen((X.bbox.x + X.bbox.w) * S, (X.bbox.y + X.bbox.h) * S), et = n.getBoundingClientRect();
      if (!(rt.x < 0 || it.x > et.width || rt.y < 0 || it.y > et.height)) return;
      const Lt = (X.bbox.x + X.bbox.w / 2) * S, Ht = (X.bbox.y + X.bbox.h / 2) * S;
      t.moveCenter(Lt, Ht);
    }
    function B() {
      const X = /* @__PURE__ */ new Map(), it = E;
      if (!it) return X;
      for (let rt = 0; rt < it.iterations.length; rt++) {
        const et = it.iterations[rt], St = X.get(et.iter) ?? [];
        St.push(rt), X.set(et.iter, St);
      }
      return X;
    }
    function N(X) {
      if (!E) return;
      const it = B(), rt = Array.from(it.keys()).sort((Xt, Zt) => Xt - Zt), et = E.iterations[A].iter, St = rt.indexOf(et), Lt = rt[Math.max(0, Math.min(rt.length - 1, St + X))], Ht = it.get(Lt) ?? [], Vt = Ht.find((Xt) => E.iterations[Xt].variant === "");
      Z(Vt ?? Ht[0] ?? A);
    }
    function j(X) {
      if (!E) return;
      const it = B(), rt = E.iterations[A].iter, et = it.get(rt) ?? [];
      if (et.length <= 1) return;
      const Lt = (et.indexOf(A) + X + et.length) % et.length;
      Z(et[Lt]);
    }
    function J() {
      if (!E) return;
      const X = E.iterations[A];
      if (!X) return;
      const it = n.getBoundingClientRect(), rt = s.offsetWidth || 200, et = s.offsetHeight || 70, St = (X.bbox.x + X.bbox.w) * S, Lt = (X.bbox.y + X.bbox.h) * S, Ht = X.bbox.y * S, Vt = t.toScreen(St, Lt), Xt = t.toScreen(St, Ht);
      let Zt = Vt.x - rt, Et = Vt.y;
      Et + et > it.height - 4 && (Et = Xt.y - et), Zt = Math.max(4, Math.min(Zt, it.width - rt - 4)), Et = Math.max(4, Math.min(Et, it.height - et - 4)), s.style.left = `${Zt}px`, s.style.top = `${Et}px`;
    }
    t.on("moved", J), t.on("zoomed", J), window.addEventListener("resize", J);
    function st() {
      if (!E) return;
      const X = E, it = X.iterations[A];
      r.textContent = `Junction (${X.seed.x},${X.seed.y})`, o.className = `jd-status-pill jd-${X.outcome.kind.toLowerCase()}`, o.textContent = Kc(X);
      const rt = X.iterations.length, et = it && it.variant ? ` \xB7 ${it.variant}` : "";
      f.textContent = `iter ${it ? it.iter : "-"}${et} \xB7 ${A + 1}/${rt}`, p.disabled = A <= 0, g.disabled = A >= rt - 1, m.disabled = A === X.defaultIterIndex, y.innerHTML = "";
      for (const St of Hw(X, it)) {
        const Lt = document.createElement("div");
        Lt.className = `jd-inline-summary-row jd-inline-summary-row--${St.tone}`, Lt.textContent = St.text, y.appendChild(Lt);
      }
      O && ht(), it && e.onChange({
        cluster: X,
        iter: it,
        trace: D
      });
    }
    function ht() {
      if (!E) return;
      const X = E, it = X.iterations[A];
      w.className = `jd-status-pill jd-${X.outcome.kind.toLowerCase()}`, w.textContent = Kc(X), v.textContent = `Junction (${X.seed.x},${X.seed.y})`, jw(T, X, it);
    }
    d.addEventListener("click", G), a.addEventListener("click", K), l.addEventListener("click", bt), c.addEventListener("click", () => {
      if (!E) return;
      const X = ia(E, E.iterations[A], D);
      Qw(c, X);
    }), h.addEventListener("click", () => {
      if (!E || !e.onEditRequested) return;
      const X = E.iterations[A];
      X && e.onEditRequested({
        cluster: E,
        iter: X,
        trace: D
      });
    }), C.addEventListener("click", $), b.addEventListener("click", $);
    function ut() {
      if (!E) return null;
      const X = E.iterations[A];
      return X ? {
        cluster: E,
        iter: X,
        trace: D
      } : null;
    }
    function mt(X) {
      s.classList.toggle("jd-edit-mode", X), c.disabled = X, l.disabled = X, h.disabled = X;
    }
    function bt() {
      var _a2;
      if (!E) return;
      const X = Dw(E, A), it = JSON.stringify(X, (et, St) => typeof St == "bigint" ? String(St) : St, 2), rt = (et) => {
        const St = l.textContent;
        l.textContent = et ? "\u2713" : "!", l.classList.add("jd-inline-btn--flash"), window.setTimeout(() => {
          l.textContent = St, l.classList.remove("jd-inline-btn--flash");
        }, 900);
      };
      if ((_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText) navigator.clipboard.writeText(it).then(() => rt(true), () => rt(false));
      else {
        const et = document.createElement("textarea");
        et.value = it, et.style.position = "fixed", et.style.opacity = "0", document.body.appendChild(et), et.select();
        try {
          document.execCommand("copy"), rt(true);
        } catch {
          rt(false);
        }
        document.body.removeChild(et);
      }
    }
    return p.addEventListener("click", () => Z(A - 1)), g.addEventListener("click", () => Z(A + 1)), m.addEventListener("click", () => {
      E && Z(E.defaultIterIndex);
    }), document.addEventListener("keydown", (X) => {
      var _a2, _b2;
      if (!U()) return;
      const it = (_b2 = (_a2 = X.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (it === "INPUT" || it === "TEXTAREA" || it === "SELECT") return;
      const rt = X.key, et = () => {
        X.stopImmediatePropagation(), X.preventDefault();
      };
      rt === "Escape" ? (O ? $() : G(), et()) : rt === "ArrowLeft" ? (Z(A - 1), et()) : rt === "ArrowRight" ? (Z(A + 1), et()) : rt === "Home" ? (Z(0), et()) : rt === "End" && E ? (Z(E.iterations.length - 1), et()) : rt === "w" || rt === "W" ? (N(-1), et()) : rt === "s" || rt === "S" ? (N(1), et()) : rt === "a" || rt === "A" ? (j(-1), et()) : rt === "d" || rt === "D" ? (j(1), et()) : (rt === "i" || rt === "I") && (K(), et());
    }, {
      capture: true
    }), {
      open: H,
      close: G,
      isOpen: U,
      inlineEl: s,
      getSelection: ut,
      setEditMode: mt
    };
  }
  function Dw(n, t) {
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
  function Kc(n) {
    switch (n.outcome.kind) {
      case "Solved":
        return `Solved \xB7 ${n.outcome.regionTiles}t`;
      case "Capped":
        return `Capped \xB7 ${n.outcome.iters} iter`;
      case "Open":
        return "Open";
    }
  }
  function Hw(n, t) {
    if (t) {
      if (t.veto) return [
        {
          text: `veto \xB7 ${Ui(t.veto.broken_segment, 22)} @ (${t.veto.break_tile_x},${t.veto.break_tile_y})`,
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
        const i = s.detail ? ` \xB7 ${Ui(s.detail, 28)}` : "";
        return [
          {
            text: `${s.strategy} \u2192 ${Ui(s.outcome, 12)}${i}`,
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
            text: `cap: ${Ui(n.outcome.reason, 32)}`,
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
  function Ui(n, t) {
    return n.length <= t ? n : `${n.slice(0, t - 1)}\u2026`;
  }
  function Nu(n) {
    var _a2;
    return ((_a2 = n.sat) == null ? void 0 : _a2.boundaries) ?? n.boundaries;
  }
  function Uw(n) {
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
  function jw(n, t, e) {
    n.innerHTML = "", n.appendChild(Yw(t)), n.appendChild(Vw(t, e)), e && (n.appendChild(Xw(e)), n.appendChild(qw(e)), n.appendChild(Kw(e)), e.veto && n.appendChild(Jw(e))), t.nearbyStamped.length > 0 && n.appendChild(Zw(t));
  }
  function ns(n, t = true) {
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
  function Yw(n) {
    const { details: t, bodyEl: e } = ns("Summary"), s = document.createElement("div");
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
  function Vw(n, t) {
    const { details: e, bodyEl: s } = ns("Participating specs");
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
  function Xw(n) {
    const t = Nu(n), e = n.sat ? " (as fed to SAT)" : " (spec perimeter)", { details: s, bodyEl: i } = ns(`Boundaries${e}`);
    if (t.length === 0) {
      const r = document.createElement("div");
      return r.className = "jd-row jd-row--dim", r.textContent = "(none)", i.appendChild(r), s;
    }
    for (const r of t) {
      const o = document.createElement("div");
      o.className = "jd-row";
      const a = r.is_input ? "IN " : "OUT", l = r.interior ? " (interior)" : "", c = r.external_feeder ? ` \u2190 ${gw(r.external_feeder)}` : "";
      o.style.color = r.is_input ? "#9f9" : "#f99";
      const h = r.spec_key ? ` \xB7 ${r.spec_key}` : "";
      o.textContent = `${a} (${r.x},${r.y}) ${Uw(r.direction)} ${r.direction} \xB7 ${r.item}${l}${h}${c}`, i.appendChild(o);
    }
    return s;
  }
  function qw(n) {
    const { details: t, bodyEl: e } = ns("Strategy attempts");
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
  function Kw(n) {
    const { details: t, bodyEl: e } = ns("SAT", !!n.sat);
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
  function Jw(n) {
    const { details: t, bodyEl: e } = ns("Walker veto");
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
  function Zw(n) {
    const { details: t, bodyEl: e } = ns("Nearby stamped", false);
    for (const s of n.nearbyStamped) {
      const i = document.createElement("div");
      i.className = "jd-row";
      const r = s.carries ? ` carries=${s.carries}` : "", o = s.segment_id ? ` \xB7 seg=${s.segment_id}` : "";
      i.textContent = `(${s.x},${s.y}) ${s.name} ${s.direction}${r}${o}${s.feeds_seed_area ? "  \u26A0 feeds seed" : ""}`, e.appendChild(i);
    }
    return t;
  }
  const Fu = {
    "transport-belt": 4,
    "fast-transport-belt": 6,
    "express-transport-belt": 8
  };
  function ia(n, t, e, s) {
    var _a2, _b2;
    const i = n.seed, r = (t == null ? void 0 : t.bbox) ?? {
      x: i.x,
      y: i.y,
      w: 1,
      h: 1
    }, o = (t == null ? void 0 : t.iter) ?? 0, a = ((_a2 = t == null ? void 0 : t.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", l = ((_b2 = t == null ? void 0 : t.sat) == null ? void 0 : _b2.max_reach) ?? Fu[a] ?? 4, c = Nu(t ?? {
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
    })), h = t && e ? xw(e, r, 2).map((p) => ({
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
  function Qw(n, t) {
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
  function tv(n) {
    return n.x === void 0 || n.y === void 0 || n.direction === void 0 ? null : n;
  }
  const ji = {
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
  }, Jc = {
    North: "East",
    East: "South",
    South: "West",
    West: "North"
  }, Yi = {
    "transport-belt": "transport-belt",
    "fast-transport-belt": "fast-transport-belt",
    "express-transport-belt": "express-transport-belt"
  }, Zc = {
    "transport-belt": "underground-belt",
    "fast-transport-belt": "fast-underground-belt",
    "express-transport-belt": "express-underground-belt"
  };
  function ev(n) {
    const { viewport: t, canvas: e, engine: s, jd: i, satZoneOverlayLayer: r } = n;
    let o = null, a = null, l = null, c = null, h = null, d = null, u = false, p = null, f = [], g = [], m = [], y = [], b = "belt", x = "East", _ = 0, v = null, w = "idle", C = 0, T = null, P = null;
    function E(k, I) {
      return `${k},${I}`;
    }
    function A(k, I) {
      if (!p) return false;
      const z = p.bbox;
      return k >= z.x && k < z.x + z.w && I >= z.y && I < z.y + z.h;
    }
    function D(k, I) {
      return f.find((z) => z.x === k && z.y === I);
    }
    function O() {
      if (!p || p.items.length === 0) return null;
      const k = Math.max(0, Math.min(_, p.items.length - 1));
      return p.items[k] ?? null;
    }
    function H(k, I, z) {
      if (!p) return null;
      const [Q, R] = ji[z], F = k - Q, Y = I - R, q = D(F, Y);
      if (q && q.direction === z && (Yi[q.name] === q.name || q.io_type === "output")) return q.carries ?? null;
      const ot = p.boundaries.find((gt) => gt.x === k && gt.y === I && gt.isInput && gt.dir === z);
      return ot ? ot.item : null;
    }
    function G(k, I) {
      return H(k.x, k.y, I) ?? O();
    }
    function L(k, I) {
      if (!p) return null;
      const [z, Q] = ji[I];
      for (let R = 1; R <= p.maxReach + 1; R++) {
        const F = k.x - z * R, Y = k.y - Q * R, q = D(F, Y);
        if (q && q.io_type === "input" && q.direction === I) return q.carries ?? null;
      }
      return null;
    }
    function $() {
      g.push(f.map((k) => ({
        ...k
      }))), g.length > 64 && g.shift(), m.length = 0;
    }
    function K(k, I) {
      $(), f = k, Z(), ht();
    }
    function U() {
      if (!p) return [];
      const k = Yi[p.beltTier] ?? "transport-belt", I = [];
      for (const z of p.boundaries) {
        if (!z.isInput) continue;
        const [Q, R] = ji[z.dir];
        I.push({
          name: k,
          x: z.x - Q,
          y: z.y - R,
          direction: z.dir,
          carries: z.item
        });
      }
      return I;
    }
    function Z() {
      const k = U();
      o && ni({
        entities: f
      }, o, void 0, void 0, void 0, k), a && ni({
        entities: y
      }, a, void 0, void 0, void 0, k), M();
    }
    function M() {
      if (c) {
        c.removeChildren();
        for (const k of f) {
          const I = k.carries;
          if (!I) continue;
          const z = `/spaghettio/pr-333/icons/${I}.png`, Q = Oe.get(z);
          if (!Q) continue;
          const R = new We(Q), F = S * 0.55;
          R.width = F, R.height = F, R.x = k.x * S + (S - F) / 2, R.y = k.y * S + (S - F) / 2, R.alpha = 0.85, c.addChild(R);
        }
      }
    }
    function B(k, I) {
      l && (ni({
        entities: k
      }, l, void 0, void 0, void 0, U()), l.alpha = I ? 0.5 : 0.45, l.tint = I ? 16733525 : 16777215);
    }
    function N() {
      l && (l.removeChildren(), l.tint = 16777215);
    }
    function j(k, I = "") {
      if (w = k, !d) return;
      d.classList.remove("ok", "solving", "invalid", "idle"), d.classList.add(k === "valid" ? "ok" : k);
      const z = k === "valid" ? "\u25CF" : k === "solving" ? "\u25D4" : k === "invalid" ? "\u25CF" : "\u25CB";
      d.textContent = z;
      let Q = "";
      k === "valid" ? Q = P !== null ? `valid \xB7 cost ${P} / yours ${J(f)}` : "valid" : k === "solving" ? Q = "solving\u2026" : k === "invalid" ? Q = "invalid" : Q = "no edits yet", d.title = I ? `${Q}
${I}` : Q, Ct();
    }
    function J(k) {
      let I = 0;
      for (const z of k) Yi[z.name] === z.name ? I += 1 : Zc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] === z.name && (I += 5);
      return I;
    }
    function st() {
      if (!p) return "no zone";
      const k = /* @__PURE__ */ new Set();
      for (const I of f) {
        const z = E(I.x, I.y);
        if (k.has(z)) return `duplicate entity at (${I.x},${I.y})`;
        if (k.add(z), !A(I.x, I.y)) return `entity at (${I.x},${I.y}) outside bbox`;
        if (p.forbidden.has(z)) return `entity at (${I.x},${I.y}) on forbidden tile`;
      }
      for (const I of f) {
        if (I.io_type !== "input") continue;
        const [z, Q] = ji[I.direction];
        let R = false;
        for (let F = 1; F <= p.maxReach + 1; F++) {
          const Y = I.x + z * F, q = I.y + Q * F, ot = D(Y, q);
          if (ot) {
            if (ot.io_type === "output" && ot.direction === I.direction && ot.carries === I.carries) {
              R = true;
              break;
            }
            if (ot.io_type === "input" && ot.carries === I.carries) return `UG-in at (${I.x},${I.y}) blocked by another UG-in at (${Y},${q})`;
          }
        }
        if (!R) return `UG-in at (${I.x},${I.y}) has no matching UG-out within reach ${p.maxReach}`;
      }
      return null;
    }
    function ht() {
      if (!u || !p) return;
      const k = st();
      if (k) {
        y = [], P = null, Z(), j("invalid", k);
        return;
      }
      j("solving"), T !== null && window.clearTimeout(T);
      const I = ++C;
      T = window.setTimeout(() => {
        ut(I);
      }, 300);
    }
    async function ut(k) {
      if (p) try {
        const I = await s.solveFixture(p.fixtureJson, f);
        if (k !== C || !u) return;
        if (!I) {
          y = [], P = null, Z(), j("invalid", "SAT cannot complete this layout");
          return;
        }
        const z = new Set(f.map((R) => E(R.x, R.y))), Q = [];
        for (const R of I.entities) {
          const F = tv(R);
          F && !z.has(E(F.x, F.y)) && Q.push(F);
        }
        y = Q, P = I.cost, Z(), j("valid");
      } catch (I) {
        if (k !== C) return;
        y = [], Z(), j("invalid", `solver error: ${I instanceof Error ? I.message : String(I)}`);
      }
    }
    function mt(k) {
      const I = e.getBoundingClientRect(), z = t.toWorld(k.clientX - I.left, k.clientY - I.top);
      return {
        x: Math.floor(z.x / S),
        y: Math.floor(z.y / S)
      };
    }
    function bt(k, I, z) {
      const Q = [], R = I.x === k.x ? 0 : I.x > k.x ? 1 : -1, F = I.y === k.y ? 0 : I.y > k.y ? 1 : -1;
      if (z) {
        for (let q = k.y; q !== I.y + F && F !== 0; q += F) Q.push({
          x: k.x,
          y: q
        });
        F === 0 && Q.push({
          x: k.x,
          y: k.y
        });
        for (let q = k.x + R; R !== 0 && q !== I.x + R; q += R) Q.push({
          x: q,
          y: I.y
        });
      } else {
        for (let q = k.x; q !== I.x + R && R !== 0; q += R) Q.push({
          x: q,
          y: k.y
        });
        R === 0 && Q.push({
          x: k.x,
          y: k.y
        });
        for (let q = k.y + F; F !== 0 && q !== I.y + F; q += F) Q.push({
          x: I.x,
          y: q
        });
      }
      const Y = [];
      for (const q of Q) {
        const ot = Y[Y.length - 1];
        (!ot || ot.x !== q.x || ot.y !== q.y) && Y.push(q);
      }
      return Y;
    }
    function X(k, I) {
      return k.x === I.x && k.y === I.y - 1 ? "South" : k.x === I.x && k.y === I.y + 1 ? "North" : k.y === I.y && k.x === I.x - 1 ? "East" : k.y === I.y && k.x === I.x + 1 ? "West" : null;
    }
    function it(k) {
      if (!p || k.length === 0) return null;
      for (const ot of k) if (!A(ot.x, ot.y)) return null;
      const I = (ot) => !p.forbidden.has(E(ot.x, ot.y)), z = k[0], Q = k[k.length - 1];
      if (!z || !Q || !I(z) || !I(Q)) return null;
      const R = [], F = k.length > 1 ? X(k[0], k[1]) ?? x : x, Y = G(k[0], F);
      let q = 0;
      for (; q < k.length; ) {
        const ot = k[q], gt = k[q + 1] ?? null, pt = gt ? X(ot, gt) : q > 0 ? X(k[q - 1], ot) : x;
        if (!pt) return null;
        let yt = q + 1;
        for (; yt < k.length && I(k[yt]); ) yt++;
        if (yt === k.length) {
          R.push(rt(ot, pt, Y)), q++;
          continue;
        }
        let kt = yt;
        for (; kt < k.length && !I(k[kt]); ) kt++;
        if (kt === k.length) return null;
        const se = k[yt - 1], ue = k[kt];
        if (Math.abs(ue.x - se.x) + Math.abs(ue.y - se.y) > p.maxReach + 1) return null;
        for (let vn = q; vn < yt - 1; vn++) {
          const _i = k[vn], wi = X(_i, k[vn + 1]);
          if (!wi) return null;
          R.push(rt(_i, wi, Y));
        }
        const Le = X(se, ue);
        if (!Le) return null;
        R.push(et(se, Le, "input", Y)), R.push(et(ue, Le, "output", Y)), q = kt + 1;
      }
      return R;
    }
    function rt(k, I, z) {
      return {
        name: Yi[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "transport-belt",
        x: k.x,
        y: k.y,
        direction: I,
        carries: z ?? void 0
      };
    }
    function et(k, I, z, Q) {
      return {
        name: Zc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "underground-belt",
        x: k.x,
        y: k.y,
        direction: I,
        io_type: z,
        carries: Q ?? void 0
      };
    }
    function St(k) {
      const I = new Set(k.map((Q) => E(Q.x, Q.y))), z = f.filter((Q) => !I.has(E(Q.x, Q.y))).concat(k);
      K(z);
    }
    function Lt(k) {
      if (!p || !A(k.x, k.y) || !f.some((z) => z.x === k.x && z.y === k.y)) return;
      const I = f.filter((z) => !(z.x === k.x && z.y === k.y));
      K(I);
    }
    function Ht(k) {
      if (!p) return;
      const I = p.boundaries.find((Q) => Q.x === k.x && Q.y === k.y && Q.isInput);
      if (!I) return;
      const z = p.items.indexOf(I.item);
      z >= 0 && z !== _ && (_ = z, tt());
    }
    function Vt(k) {
      if (!u || !p || k.button !== 0) return;
      const I = mt(k);
      if (!I || !A(I.x, I.y)) return;
      if (Ht(I), b === "erase") {
        Lt(I), k.stopPropagation(), k.preventDefault();
        return;
      }
      if (b === "ug-in" || b === "ug-out") {
        const R = b === "ug-in" ? H(I.x, I.y, x) ?? O() : L(I, x) ?? H(I.x, I.y, x) ?? O(), F = b === "ug-in" ? et(I, x, "input", R) : et(I, x, "output", R), Y = f.filter((q) => !(q.x === I.x && q.y === I.y)).concat(F);
        K(Y), k.stopPropagation(), k.preventDefault();
        return;
      }
      v = {
        startX: I.x,
        startY: I.y,
        bendVerticalFirst: false
      };
      const z = bt({
        x: I.x,
        y: I.y
      }, I, false), Q = it(z);
      B(Q ?? [], Q === null), k.stopPropagation(), k.preventDefault();
    }
    function Xt(k) {
      if (!u || !v || !p) return;
      const I = mt(k);
      if (!I) return;
      const z = bt({
        x: v.startX,
        y: v.startY
      }, I, v.bendVerticalFirst), Q = it(z);
      B(Q ?? [], Q === null);
    }
    function Zt(k) {
      if (!u || !v || !p) {
        v = null, N();
        return;
      }
      const I = mt(k);
      if (!I) {
        v = null, N();
        return;
      }
      const z = bt({
        x: v.startX,
        y: v.startY
      }, I, v.bendVerticalFirst), Q = it(z);
      if (v = null, N(), !Q) {
        j("invalid", "drag rejected: out of bounds, on obstacle, or UG too long");
        return;
      }
      St(Q), k.stopPropagation(), k.preventDefault();
    }
    function Et(k) {
      var _a2, _b2;
      if (!u) return;
      const I = (_b2 = (_a2 = k.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (I === "INPUT" || I === "TEXTAREA" || I === "SELECT") return;
      const z = () => {
        k.stopImmediatePropagation(), k.preventDefault();
      };
      if (k.key === "Escape") {
        Gt(), z();
        return;
      }
      if (k.key === "1") {
        Bt("belt"), z();
        return;
      }
      if (k.key === "2") {
        Bt("ug-in"), z();
        return;
      }
      if (k.key === "3") {
        Bt("ug-out"), z();
        return;
      }
      if (k.key === "0") {
        Bt("erase"), z();
        return;
      }
      if (k.key === "r" || k.key === "R") {
        v ? v.bendVerticalFirst = !v.bendVerticalFirst : (x = Jc[x], tt()), z();
        return;
      }
      if (k.key === "[" && p) {
        _ = (_ - 1 + p.items.length) % p.items.length, tt(), z();
        return;
      }
      if (k.key === "]" && p) {
        _ = (_ + 1) % p.items.length, tt(), z();
        return;
      }
      if ((k.key === "Enter" || k.key === "a" || k.key === "A") && w === "valid" && y.length > 0) {
        nt(), z();
        return;
      }
      if ((k.ctrlKey || k.metaKey) && (k.key === "z" || k.key === "Z")) {
        k.shiftKey ? V() : qt(), z();
        return;
      }
    }
    function Bt(k) {
      b = k, tt();
    }
    function qt() {
      g.length !== 0 && (m.push(f), f = g.pop(), Z(), ht());
    }
    function V() {
      m.length !== 0 && (g.push(f), f = m.pop(), Z(), ht());
    }
    function nt() {
      if (y.length === 0) return;
      const k = f.concat(y.map((I) => ({
        ...I
      })));
      y = [], K(k);
    }
    function tt() {
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
      for (const [ot, gt, pt] of k) {
        const yt = document.createElement("button");
        yt.className = "se-tool" + (b === ot ? " se-tool-active" : ""), yt.textContent = gt, yt.title = pt, yt.addEventListener("click", () => Bt(ot)), h.appendChild(yt);
      }
      const I = document.createElement("button");
      I.className = "se-dir";
      const z = {
        North: "\u2191",
        East: "\u2192",
        South: "\u2193",
        West: "\u2190"
      };
      if (I.textContent = z[x], I.title = "Brush direction (R rotates)", I.addEventListener("click", () => {
        x = Jc[x], tt();
      }), h.appendChild(I), p && p.items.length > 1) {
        const ot = document.createElement("select");
        ot.className = "se-item";
        for (const [gt, pt] of p.items.entries()) {
          const yt = document.createElement("option");
          yt.value = String(gt), yt.textContent = pt, gt === _ && (yt.selected = true), ot.appendChild(yt);
        }
        ot.addEventListener("change", () => {
          _ = Number(ot.value) | 0;
        }), h.appendChild(ot);
      } else if (p && p.items.length === 1) {
        const ot = document.createElement("span");
        ot.className = "se-item-label", ot.textContent = p.items[0], h.appendChild(ot);
      }
      const Q = document.createElement("span");
      Q.style.flex = "1", h.appendChild(Q);
      const R = document.createElement("button");
      R.className = "se-accept", R.textContent = "Accept", R.title = "Promote ghost into painted layer (Enter)", R.addEventListener("click", nt), R.disabled = !(w === "valid" && y.length > 0), h.appendChild(R);
      const F = document.createElement("button");
      F.className = "se-revert", F.textContent = "Revert", F.title = "Discard all painted edits", F.addEventListener("click", () => {
        K([]);
      }), h.appendChild(F);
      const Y = document.createElement("button");
      Y.className = "se-export", Y.textContent = "Export", Y.title = "Save fixture JSON (clipboard + download)", Y.addEventListener("click", Ut), Y.disabled = w !== "valid", h.appendChild(Y);
      const q = document.createElement("button");
      q.className = "se-done", q.textContent = "Done", q.title = "Exit edit mode (Esc)", q.addEventListener("click", Gt), h.appendChild(q);
    }
    function Ct() {
      if (!h) return;
      const k = h.querySelector(".se-accept");
      k && (k.disabled = !(w === "valid" && y.length > 0));
      const I = h.querySelector(".se-export");
      I && (I.disabled = w !== "valid");
    }
    function Ut() {
      var _a2;
      if (!p || w !== "valid") return;
      const k = P ?? J(f), I = ia(p.selection.cluster, p.selection.iter, p.selection.trace, {
        maxCost: k,
        paintedEntities: f
      }), z = (p.selection.cluster.seed ? `fixture_${p.selection.cluster.seed.x}_${p.selection.cluster.seed.y}_painted` : "fixture_painted") + ".json";
      (_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText(I).catch(() => {
      });
      const Q = new Blob([
        I
      ], {
        type: "application/json"
      }), R = URL.createObjectURL(Q), F = document.createElement("a");
      F.href = R, F.download = z, document.body.appendChild(F), F.click(), document.body.removeChild(F), URL.revokeObjectURL(R);
    }
    function ye(k) {
      var _a2, _b2, _c2, _d2;
      u && Gt();
      const I = k.iter, z = ((_a2 = I.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", Q = ((_b2 = I.sat) == null ? void 0 : _b2.max_reach) ?? Fu[z] ?? 4, R = ((_c2 = I.sat) == null ? void 0 : _c2.boundaries) ?? I.boundaries, F = Array.from(new Set(R.map((q) => q.item))), Y = R.map((q) => ({
        x: q.x,
        y: q.y,
        item: q.item,
        isInput: q.is_input,
        dir: q.direction
      }));
      p = {
        bbox: {
          x: I.bbox.x,
          y: I.bbox.y,
          w: I.bbox.w,
          h: I.bbox.h
        },
        forbidden: new Set((I.forbidden ?? []).map((q) => `${q[0]},${q[1]}`)),
        beltTier: z,
        maxReach: Q,
        items: F,
        boundaries: Y,
        fixtureJson: ia(k.cluster, k.iter, k.trace),
        selection: k
      }, f = [], g = [], m = [], y = [], b = "belt", x = "East", _ = 0, v = null, P = null, o = new zt(), a = new zt(), a.alpha = 0.55, l = new zt(), c = new zt(), t.addChild(o), t.addChild(a), t.addChild(c), t.addChild(l), t.setChildIndex(r, t.children.length - 1), h = document.createElement("div"), h.className = "se-toolbar", i.inlineEl.appendChild(h), d = document.createElement("span"), d.className = "se-status", (_d2 = i.inlineEl.querySelector(".jd-inline-head")) == null ? void 0 : _d2.appendChild(d), i.setEditMode(true), t.plugins.pause("drag"), e.addEventListener("pointerdown", Vt, {
        capture: true
      }), e.addEventListener("pointerup", Zt, {
        capture: true
      }), e.addEventListener("pointermove", Xt, {
        capture: true
      }), document.addEventListener("keydown", Et, {
        capture: true
      }), u = true, tt(), j("idle");
    }
    function Gt() {
      u && (u = false, T !== null && (window.clearTimeout(T), T = null), C++, o && (t.removeChild(o), o.destroy({
        children: true
      }), o = null), a && (t.removeChild(a), a.destroy({
        children: true
      }), a = null), l && (t.removeChild(l), l.destroy({
        children: true
      }), l = null), c && (t.removeChild(c), c.destroy({
        children: true
      }), c = null), h && (h.remove(), h = null), d && (d.remove(), d = null), e.removeEventListener("pointerdown", Vt, {
        capture: true
      }), e.removeEventListener("pointerup", Zt, {
        capture: true
      }), e.removeEventListener("pointermove", Xt, {
        capture: true
      }), document.removeEventListener("keydown", Et, {
        capture: true
      }), t.plugins.resume("drag"), i.setEditMode(false), p = null, f = [], g = [], m = [], y = []);
    }
    function re() {
      return u;
    }
    return {
      enter: ye,
      exit: Gt,
      isActive: re
    };
  }
  let ws = {
    master: false,
    stepThrough: true,
    satZones: false,
    soloRegions: false,
    ghostTiles: false,
    itemColors: true,
    traceOverlay: false,
    heatmap: false,
    powerWires: false,
    moduleSlots: true
  };
  const tr = [];
  function nv() {
    const n = new URLSearchParams(window.location.search).get("debug") === "1", t = localStorage.getItem("fk-debug") === "1", e = localStorage.getItem("fk-sat-zones") === "1", s = localStorage.getItem("fk-ghost-tiles") === "1", i = localStorage.getItem("fk-item-colors"), r = localStorage.getItem("fk-trace-overlay") === "1", o = localStorage.getItem("fk-heatmap") === "1", a = localStorage.getItem("fk-power-wires") === "1", l = localStorage.getItem("fk-module-slots");
    ws = {
      ...ws,
      master: n || t,
      satZones: e,
      ghostTiles: s,
      itemColors: i === null ? true : i === "1",
      traceOverlay: r,
      heatmap: o,
      powerWires: a,
      moduleSlots: l === null ? true : l === "1"
    };
  }
  function Wu() {
    return ws;
  }
  function ln(n) {
    ws = {
      ...ws,
      ...n
    }, "master" in n && localStorage.setItem("fk-debug", n.master ? "1" : "0"), "satZones" in n && localStorage.setItem("fk-sat-zones", n.satZones ? "1" : "0"), "ghostTiles" in n && localStorage.setItem("fk-ghost-tiles", n.ghostTiles ? "1" : "0"), "itemColors" in n && localStorage.setItem("fk-item-colors", n.itemColors ? "1" : "0"), "traceOverlay" in n && localStorage.setItem("fk-trace-overlay", n.traceOverlay ? "1" : "0"), "heatmap" in n && localStorage.setItem("fk-heatmap", n.heatmap ? "1" : "0"), "powerWires" in n && localStorage.setItem("fk-power-wires", n.powerWires ? "1" : "0"), "moduleSlots" in n && localStorage.setItem("fk-module-slots", n.moduleSlots ? "1" : "0");
    for (const t of tr) t(ws);
  }
  function sv(n) {
    return tr.push(n), () => {
      const t = tr.indexOf(n);
      t >= 0 && tr.splice(t, 1);
    };
  }
  function cn(n, t, e = false) {
    const s = document.createElement("input");
    s.type = "checkbox", s.checked = e;
    const i = document.createElement("div");
    i.className = "overlay-toggle";
    const r = document.createElement("label");
    return r.appendChild(s), r.appendChild(document.createTextNode(t)), i.appendChild(r), n.appendChild(i), s;
  }
  function iv(n) {
    n.style.position = "relative";
    const t = document.createElement("div");
    t.className = "overlay-panel";
    const e = Wu(), s = cn(t, "Debug", e.master), i = cn(t, "Item colours", e.itemColors), r = cn(t, "Starvation heatmap", e.heatmap), o = cn(t, "Power wires", e.powerWires), a = cn(t, "Module slots", e.moduleSlots), l = document.createElement("div");
    l.className = "overlay-sub-panel", l.style.display = e.master ? "flex" : "none";
    const c = cn(l, "SAT Zones", e.satZones), h = cn(l, "Ghost tiles", e.ghostTiles), d = cn(l, "Trace overlay", e.traceOverlay), u = cn(l, "Solo regions", e.soloRegions);
    return t.appendChild(l), n.appendChild(t), s.addEventListener("change", () => {
      l.style.display = s.checked ? "flex" : "none", ln({
        master: s.checked
      });
    }), c.addEventListener("change", () => {
      ln({
        satZones: c.checked
      });
    }), h.addEventListener("change", () => {
      ln({
        ghostTiles: h.checked
      });
    }), d.addEventListener("change", () => {
      ln({
        traceOverlay: d.checked
      });
    }), i.addEventListener("change", () => {
      ln({
        itemColors: i.checked
      });
    }), r.addEventListener("change", () => {
      ln({
        heatmap: r.checked
      });
    }), o.addEventListener("change", () => {
      ln({
        powerWires: o.checked
      });
    }), a.addEventListener("change", () => {
      ln({
        moduleSlots: a.checked
      });
    }), {
      setDebugEnabled(p) {
        s.checked = p, l.style.display = p ? "flex" : "none", ln({
          master: p
        });
      },
      debugCb: s,
      colorCb: i,
      heatmapCb: r,
      powerWiresCb: o,
      moduleSlotsCb: a,
      regionsCb: c,
      soloRegionsCb: u,
      ghostTilesCb: h,
      traceOverlayCb: d
    };
  }
  function rv(n) {
    const t = document.createElement("div");
    t.className = "retry-panel", n.appendChild(t);
    let e = null;
    function s(r) {
      if (!(r == null ? void 0 : r.trace)) return null;
      for (const o of r.trace) if (o.phase === "LayoutRetried") return o.data;
      return null;
    }
    function i() {
      const r = Wu().master, o = s(e);
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
        const g = document.createElement("span");
        g.className = "recipe", g.textContent = p;
        const m = document.createElement("span");
        m.className = "gap", m.textContent = `+${u} tile${u === 1 ? "" : "s"}`, f.appendChild(document.createTextNode(`row ${d} (`)), f.appendChild(g), f.appendChild(document.createTextNode("): ")), f.appendChild(m), t.appendChild(f);
      }
    }
    return sv(() => i()), {
      update(r) {
        e = r, i();
      }
    };
  }
  const ov = {
    North: "\u2191",
    East: "\u2192",
    South: "\u2193",
    West: "\u2190"
  }, Qc = {
    N: "\u2191",
    E: "\u2192",
    S: "\u2193",
    W: "\u2190"
  };
  function hn(n = 16) {
    const t = document.createElement("img");
    return t.width = n, t.height = n, t.style.cssText = "vertical-align:middle;margin-right:3px;image-rendering:pixelated", t.addEventListener("error", () => {
      t.style.display = "none";
    }), t;
  }
  function ds(n, t) {
    n.style.display = "", n.src = `/spaghettio/pr-333/icons/${t}.png`;
  }
  function go(n, t) {
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
  function av(n) {
    const t = document.createElement("div");
    t.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:240px;line-height:1.5", document.body.appendChild(t);
    const e = document.createElement("div");
    t.appendChild(e);
    const s = document.createElement("span");
    s.style.color = "#888", s.style.display = "none", t.appendChild(s);
    const i = document.createElement("div");
    t.appendChild(i);
    const r = document.createElement("div"), o = hn(16), a = document.createElement("b");
    r.append(o, a), r.style.display = "none", i.appendChild(r);
    const l = document.createElement("div");
    l.style.display = "none", i.appendChild(l);
    const c = document.createElement("div"), h = hn(16), d = document.createElement("span");
    c.append(h, d), c.style.display = "none", i.appendChild(c);
    const u = document.createElement("div");
    u.style.color = "#b5cea8", u.style.display = "none", i.appendChild(u);
    const p = document.createElement("div");
    p.style.display = "none", i.appendChild(p);
    const f = document.createElement("div"), g = hn(16), m = document.createElement("span");
    f.append(g, m), f.style.display = "none", i.appendChild(f);
    function y() {
      const R = document.createElement("div");
      R.style.color = "#aaa";
      const F = document.createElement("span"), Y = hn(14), q = document.createElement("span");
      return R.append(F, Y, q), R;
    }
    const b = go(i, y), x = document.createElement("div");
    x.style.color = "#9cdcfe", x.style.display = "none", i.appendChild(x);
    const _ = document.createElement("div");
    _.style.display = "none", t.appendChild(_);
    const v = document.createElement("div");
    v.style.display = "none", t.appendChild(v);
    const w = document.createElement("div");
    w.style.display = "none", t.appendChild(w);
    const C = document.createElement("div");
    C.style.display = "none", t.appendChild(C);
    const T = document.createElement("div");
    T.style.cssText = "position:absolute;top:8px;right:8px;background:#141414;color:#e0e0e0;border:1px solid #888;padding:8px 10px;font:12px monospace;border-radius:4px;display:none;z-index:20;min-width:220px;max-width:340px;line-height:1.55;box-shadow:0 4px 14px rgba(0,0,0,0.5)", n.appendChild(T);
    const P = document.createElement("div");
    P.style.cssText = "display:flex;justify-content:space-between;align-items:center;margin-bottom:4px";
    const E = document.createElement("span");
    E.style.cssText = "color:#8af;font-weight:bold", E.textContent = "pinned";
    const A = document.createElement("span");
    A.style.color = "#888", P.append(E, A), T.appendChild(P);
    const D = document.createElement("div");
    T.appendChild(D);
    const O = document.createElement("div"), H = hn(16), G = document.createElement("b");
    O.append(H, G), O.style.display = "none", D.appendChild(O);
    const L = document.createElement("span");
    L.style.color = "#888", L.textContent = "no entity at tile", L.style.display = "none", D.appendChild(L);
    const $ = document.createElement("div");
    $.style.display = "none", D.appendChild($);
    const K = document.createElement("div"), U = hn(16), Z = document.createElement("span");
    K.append(U, Z), K.style.display = "none", D.appendChild(K);
    const M = document.createElement("div");
    M.style.color = "#b5cea8", M.style.display = "none", D.appendChild(M);
    const B = document.createElement("div");
    B.style.display = "none", D.appendChild(B);
    const N = document.createElement("div"), j = hn(16), J = document.createElement("span");
    N.append(j, J), N.style.display = "none", D.appendChild(N);
    const st = go(D, y), ht = document.createElement("div");
    ht.style.color = "#9cdcfe", ht.style.display = "none", D.appendChild(ht);
    const ut = document.createElement("div");
    ut.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", ut.style.display = "none";
    const mt = document.createElement("span"), bt = document.createElement("span");
    bt.style.color = "#888", ut.append(mt, bt), T.appendChild(ut);
    const X = document.createElement("div");
    X.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", X.style.display = "none", T.appendChild(X);
    const it = document.createElement("div");
    it.style.marginTop = "4px", it.style.display = "none", T.appendChild(it);
    const rt = document.createElement("div");
    rt.style.display = "none", T.appendChild(rt);
    const et = document.createElement("div");
    et.style.marginTop = "4px", rt.appendChild(et);
    function St() {
      const R = document.createElement("div");
      return R.style.marginLeft = "4px", R;
    }
    const Lt = go(rt, St), Ht = document.createElement("div");
    Ht.style.cssText = "color:#555;margin-top:6px;font-size:10px", Ht.textContent = "click elsewhere or press Esc to unpin", T.appendChild(Ht), document.addEventListener("mousemove", (R) => {
      t.style.left = R.clientX + 14 + "px", t.style.top = R.clientY - 10 + "px";
    });
    let Vt = null, Xt = null, Zt = null, Et = null, Bt = null, qt = null;
    const V = /* @__PURE__ */ new Set();
    function nt() {
      const R = qt ? {
        x: qt.x,
        y: qt.y
      } : null;
      for (const F of V) F(R);
    }
    function tt(R, F) {
      ds(F.headerIcon, R.name), F.headerName.textContent = ge(R.name), F.header.style.display = "", R.direction && R.name !== "pipe" ? (F.dirRow.textContent = `${ov[R.direction] ?? ""} ${R.direction}`, F.dirRow.style.display = "") : F.dirRow.style.display = "none", R.carries ? (ds(F.carriesIcon, R.carries), F.carriesName.textContent = " " + ge(R.carries), F.carriesRow.style.display = "") : F.carriesRow.style.display = "none", R.rate != null ? (F.rateRow.textContent = `${R.rate.toFixed(1)}/s`, F.rateRow.style.display = "") : F.rateRow.style.display = "none", R.io_type ? (F.ioRow.textContent = `io: ${R.io_type}`, F.ioRow.style.display = "") : F.ioRow.style.display = "none";
      let Y = 0;
      if (R.recipe) {
        ds(F.recipeIcon, R.recipe), F.recipeName.textContent = " " + ge(R.recipe), F.recipeRow.style.display = "";
        const q = g0(R.recipe);
        if (q) {
          const ot = [
            ...q.inputs.map((gt) => ({
              arrow: "\u25B6",
              item: gt.item,
              rate: gt.rate
            })),
            ...q.outputs.map((gt) => ({
              arrow: "\u25C0",
              item: gt.item,
              rate: gt.rate
            }))
          ];
          for (const gt of ot) {
            const pt = F.flowPool.get(Y++), [yt, kt, se] = pt.children;
            yt.textContent = `${gt.arrow} `, ds(kt, gt.item), se.textContent = `${ge(gt.item)} ${gt.rate.toFixed(1)}/s`;
          }
        }
      } else F.recipeRow.style.display = "none";
      return F.flowPool.trim(Y), R.segment_id ? (F.segmentRow.textContent = R.segment_id, F.segmentRow.style.display = "") : F.segmentRow.style.display = "none", Y;
    }
    function Ct(R) {
      if (R.ghosts.length === 0) return _.style.display = "none", false;
      if (_.style.display = "", R.ghosts.length === 1) {
        const F = R.ghosts[0], Y = F.direction ? Qc[F.direction] : "";
        _.textContent = "";
        const q = document.createElement("span");
        q.style.color = "#8af", q.textContent = "ghost ";
        const ot = hn(12);
        ds(ot, F.item);
        const gt = document.createTextNode(`${F.item} ${Y}`);
        _.append(q, ot, gt);
      } else {
        _.textContent = "";
        const F = document.createElement("span");
        F.style.color = "#8af", F.textContent = `${R.ghosts.length} ghosts crossing`, _.appendChild(F);
      }
      return true;
    }
    function Ut(R) {
      if (!R.axis) return v.style.display = "none", false;
      const { vert: F, horiz: Y } = R.axis;
      if (F === 0 && Y === 0) return v.style.display = "none", false;
      const q = F >= 2 || Y >= 2, ot = F >= 1 && Y >= 1, gt = q ? "#ff6060" : ot ? "#60b0ff" : "#888";
      return v.style.display = "", v.style.color = gt, v.textContent = `axis V${F} H${Y}`, true;
    }
    function ye(R) {
      if (!R.junction) return w.style.display = "none", false;
      const F = R.junction, Y = F.outcome === "Solved" ? "#80d080" : F.outcome === "Capped" ? "#e0b060" : "#c06060";
      return w.style.display = "", w.style.color = Y, w.textContent = `junction seed (${F.seedX},${F.seedY}) \xB7 ${F.outcome}`, true;
    }
    function Gt(R) {
      if (R.cappedSides.length === 0) return C.style.display = "none", false;
      const F = R.cappedSides.map((Y) => {
        const q = Y.required - Y.shortfall;
        return `\u26A0 ${Y.sideIsOutput ? "out" : "in"}: ${Y.placedCount}\xD7${Y.placedEntity} moves ${q.toFixed(2)}/s of ${Y.required.toFixed(2)}/s \xB7 ${Y.limit}`;
      });
      return C.style.display = "", C.style.color = "#ffa060", C.textContent = F.join(" | "), true;
    }
    function re(R) {
      if (R.ghosts.length === 0) {
        rt.style.display = "none";
        return;
      }
      rt.style.display = "", R.ghosts.length >= 2 ? (et.style.color = "#ffa060", et.textContent = `\u26A0 ${R.ghosts.length} ghost specs at this tile`) : (et.style.color = "#8af", et.textContent = "ghost");
      let F = 0;
      for (const Y of R.ghosts) {
        const q = Y.direction ? Qc[Y.direction] : "\xB7", ot = Lt.get(F++);
        ot.textContent = "";
        const gt = document.createTextNode(`${q} `), pt = hn(14);
        ds(pt, Y.item);
        const yt = document.createTextNode(Y.item);
        if (ot.append(gt, pt, yt), Y.isStart) {
          const kt = document.createElement("span");
          kt.style.color = "#80d080", kt.textContent = " start", ot.appendChild(kt);
        } else if (Y.isEnd) {
          const kt = document.createElement("span");
          kt.style.color = "#d08080", kt.textContent = " end", ot.appendChild(kt);
        }
      }
      Lt.trim(F);
    }
    function k() {
      if (Xt !== null) {
        e.innerHTML = Xt, e.style.display = "", i.style.display = "none", _.style.display = "none", v.style.display = "none", w.style.display = "none", Et ? (s.textContent = `(${Et.x}, ${Et.y})`, s.style.display = "", s.style.display = "block") : s.style.display = "none", t.style.display = "block";
        return;
      }
      e.style.display = "none", e.innerHTML = "", i.style.display = "";
      let R = false;
      if (Zt ? (R = true, tt(Zt, {
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
      })) : (r.style.display = "none", l.style.display = "none", c.style.display = "none", u.style.display = "none", p.style.display = "none", f.style.display = "none", b.trim(0), x.style.display = "none"), Et) {
        const F = Bt == null ? void 0 : Bt.lookup(Et.x, Et.y);
        F ? (Ct(F) && (R = true), Ut(F) && (R = true), ye(F) && (R = true), Gt(F) && (R = true)) : (_.style.display = "none", v.style.display = "none", w.style.display = "none", C.style.display = "none"), s.textContent = `(${Et.x}, ${Et.y})`, s.style.display = "block", R = true;
      } else s.style.display = "none", _.style.display = "none", v.style.display = "none", w.style.display = "none";
      if (!R) {
        t.style.display = "none", Vt && Vt.clearHighlight();
        return;
      }
      t.style.display = "block", Zt && Vt ? Vt.highlightBeltNetwork(Zt) : Vt && Vt.clearHighlight();
    }
    function I() {
      if (!qt) {
        T.style.display = "none";
        return;
      }
      const { entity: R, x: F, y: Y } = qt, q = Bt == null ? void 0 : Bt.lookup(F, Y);
      if (A.textContent = `(${F}, ${Y})`, R ? (L.style.display = "none", tt(R, {
        header: O,
        headerIcon: H,
        headerName: G,
        dirRow: $,
        carriesRow: K,
        carriesIcon: U,
        carriesName: Z,
        rateRow: M,
        ioRow: B,
        recipeRow: N,
        recipeIcon: j,
        recipeName: J,
        flowPool: st,
        segmentRow: ht
      })) : (O.style.display = "none", $.style.display = "none", K.style.display = "none", M.style.display = "none", B.style.display = "none", N.style.display = "none", st.trim(0), ht.style.display = "none", L.style.display = ""), q) {
        if (q.junction) {
          const ot = q.junction.outcome === "Solved" ? "#80d080" : q.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
          mt.style.color = ot, mt.textContent = `junction seed (${q.junction.seedX},${q.junction.seedY})`, bt.textContent = ` \xB7 ${q.junction.outcome}`, ut.style.display = "";
        } else ut.style.display = "none";
        if (q.axis) {
          const { vert: ot, horiz: gt } = q.axis;
          if (ot > 0 || gt > 0) {
            const pt = ot >= 2 || gt >= 2, yt = ot >= 1 && gt >= 1, kt = pt ? " same-axis conflict" : yt ? " perpendicular crossing" : "", se = pt ? "#ff6060" : yt ? "#60b0ff" : "#bbb";
            it.style.color = se, it.textContent = `axis: V=${ot} H=${gt}${kt}`, it.style.display = "";
          } else it.style.display = "none";
        } else it.style.display = "none";
        if (re(q), q.cappedSides.length > 0) {
          const ot = {
            "tier-cap": "a faster inserter tier at the same slot count would cover this \u2014 max inserter tier is the binding constraint",
            "column-contest": "this side lost the shared inserter column to the other belt; that one column would have covered it",
            geometry: "the row shape offers no further usable slot (belt span / fixed tiles) \u2014 a template geometry limit"
          };
          X.replaceChildren();
          const gt = document.createElement("div");
          gt.style.color = "#ffa060", gt.textContent = `\u26A0 ${q.cappedSides.length} under-provisioned inserter side${q.cappedSides.length > 1 ? "s" : ""}`, X.appendChild(gt);
          for (const pt of q.cappedSides) {
            const yt = (pt.required - pt.shortfall).toFixed(2), kt = document.createElement("div");
            kt.style.cssText = "margin-top:2px;color:#ccc", kt.textContent = `${pt.sideIsOutput ? "output" : "input"}: ${pt.placedCount}\xD7${pt.placedEntity} moves ${yt}/s of ${pt.required.toFixed(2)}/s needed (short ${pt.shortfall.toFixed(2)}/s) \u2014 ${pt.limit}: ${ot[pt.limit] ?? pt.limit}`, X.appendChild(kt);
          }
          X.style.display = "";
        } else X.style.display = "none";
      } else ut.style.display = "none", it.style.display = "none", rt.style.display = "none", X.style.display = "none";
      T.style.display = "block";
    }
    function z() {
      k(), I();
    }
    function Q(R, F, Y) {
      Zt = R, F !== void 0 && Y !== void 0 ? Et = {
        x: F,
        y: Y
      } : R && (Et = {
        x: R.x ?? 0,
        y: R.y ?? 0
      }), z();
    }
    return document.addEventListener("keydown", (R) => {
      R.key === "Escape" && qt && (qt = null, nt(), z());
    }), {
      onHover: Q,
      setHighlightController(R) {
        Vt = R;
      },
      setTooltipOverride(R) {
        Xt = R, z();
      },
      setCursorTile(R, F) {
        R === null || F === void 0 ? Et = null : Et = {
          x: R,
          y: F
        }, z();
      },
      setTileContext(R) {
        Bt = R, z();
      },
      pinTile(R, F, Y) {
        qt = {
          entity: R,
          x: F,
          y: Y
        }, nt(), z();
      },
      clearPin() {
        qt = null, nt(), z();
      },
      getPinnedTile() {
        return qt ? {
          x: qt.x,
          y: qt.y
        } : null;
      },
      onPinChange(R) {
        return V.add(R), () => V.delete(R);
      }
    };
  }
  function lv(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function th(n, t, e, s) {
    return e > n ? "E" : e < n ? "W" : s > t ? "S" : s < t ? "N" : null;
  }
  const cv = {
    ghosts: [],
    axis: null,
    junction: null,
    cappedSides: []
  };
  function hv(n) {
    var _a2;
    if (!n || n.length === 0) return {
      lookup: () => cv
    };
    const t = /* @__PURE__ */ new Map(), e = /* @__PURE__ */ new Map(), s = /* @__PURE__ */ new Map(), i = /* @__PURE__ */ new Map();
    for (const r of n) if (r.phase === "GhostSpecRouted") {
      const { spec_key: o, tiles: a } = r.data, l = lv(o);
      if (!a || a.length === 0) continue;
      for (let c = 0; c < a.length; c++) {
        const [h, d] = a[c];
        let u = null;
        c < a.length - 1 ? u = th(h, d, a[c + 1][0], a[c + 1][1]) : c > 0 && (u = th(a[c - 1][0], a[c - 1][1], h, d));
        const p = `${h},${d}`, f = t.get(p), g = {
          item: l,
          specKey: o,
          direction: u,
          isStart: c === 0,
          isEnd: c === a.length - 1
        };
        f ? f.push(g) : t.set(p, [
          g
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
  function dv(n) {
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
      a && (t = H_(a, i, o), a.querySelectorAll("input,select,button").forEach((l) => {
        l.closest("[data-snapshot-keep]") || (l.disabled = true);
      }));
    }
    return {
      load: s,
      clear: e
    };
  }
  const uv = 2200, eh = 180, yo = 200, pv = 8, fv = [
    "rows_placed",
    "lanes_planned",
    "bus_routed",
    "poles_placed"
  ];
  function ur(n) {
    return `${n.x ?? 0},${n.y ?? 0},${n.name},${n.recipe ?? ""}`;
  }
  function mv(n) {
    const t = /* @__PURE__ */ new Map(), e = n.trace;
    if (!Array.isArray(e)) return t;
    for (const s of e) {
      const i = s;
      i.phase === "PhaseSnapshot" && i.data && t.set(i.data.phase, i.data);
    }
    return t;
  }
  function gv(n) {
    const t = mv(n), e = [], s = /* @__PURE__ */ new Set();
    for (const r of fv) {
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
        const c = ur(l);
        s.has(c) || (s.add(c), a.push(l));
      }
      e.push({
        phase: r,
        entities: a
      });
    }
    const i = [];
    for (const r of n.entities) {
      const o = ur(r);
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
  function yv(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map(), o = ni(n, t, e, s, (w, C) => {
      r.set(ur(w), C);
    }), a = /* @__PURE__ */ new Set();
    for (const w of r.values()) for (const C of w) a.add(C);
    const l = [];
    for (const w of t.children) {
      const C = w;
      a.has(C) || l.push(C);
    }
    for (const w of l) w.alpha = 0;
    for (const w of r.values()) for (const C of w) C.alpha = 0;
    const c = gv(n), h = c.reduce((w, C) => w + (C.entities.length > 0 ? 1 : 0), 0);
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
    const u = Math.max(0, uv - eh * h) / h, p = [], f = /* @__PURE__ */ new Map();
    let g = 0;
    for (const w of c) {
      if (w.entities.length === 0) continue;
      f.set(w.phase, g);
      const C = Math.min(pv, u / w.entities.length);
      w.entities.forEach((P, E) => {
        const A = r.get(ur(P));
        !A || A.length === 0 || p.push({
          graphics: A,
          revealStartMs: g + E * C
        });
      });
      const T = (w.entities.length - 1) * C;
      g += T + yo + eh;
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
        const T = p[C];
        if (T.revealStartMs > w) break;
        const P = Math.min(1, (w - T.revealStartMs) / yo);
        for (const E of T.graphics) E.alpha = P;
      }
      for (; y < p.length; ) {
        const C = p[y];
        if (w - C.revealStartMs < yo) break;
        for (const T of C.graphics) T.alpha = 1;
        y++;
      }
      y >= p.length && (x = true, i.ticker.remove(_), Jn());
    };
    return x || (i.ticker.add(_), Tr()), {
      controller: o,
      handle: {
        cancel() {
          b || x || (b = true, i.ticker.remove(_), Jn(), Xe());
        },
        finish() {
          if (!(b || x)) {
            for (const w of p) for (const C of w.graphics) C.alpha = 1;
            x = true, i.ticker.remove(_), Jn(), Xe();
          }
        },
        isDone() {
          return x || b;
        }
      }
    };
  }
  const xv = 4243680;
  function bv(n, t, e, s = 240) {
    const i = new ft();
    t.addChild(i);
    const r = e.x * S, o = e.y * S, a = e.w * S, l = e.h * S;
    let c = 0;
    const h = () => {
      c += n.ticker.deltaMS;
      const d = Math.max(0, (s - c) / s);
      if (d <= 0) {
        n.ticker.remove(h), i.destroy(), Jn();
        return;
      }
      i.clear(), i.rect(r, o, a, l).fill({
        color: xv,
        alpha: 0.55 * d
      });
    };
    n.ticker.add(h), Tr();
  }
  const Vi = 150, nh = 80, _v = 4, wv = 300, An = 4, kn = 900, vv = 6, Cv = 250, Us = 800, Xi = 250;
  function Ke(n, t, e) {
    return n <= 1 ? t : Math.min(t, e / n);
  }
  function sh(n, t) {
    return `${n},${t}`;
  }
  function Sv(n) {
    return n.split(":")[1] ?? "";
  }
  function Tv(n, t) {
    return n > 0 ? "East" : n < 0 ? "West" : t > 0 ? "South" : "North";
  }
  function Ev(n, t, e, s) {
    const i = gu();
    i.attachTo(n);
    const r = new ft();
    n.addChild(r);
    const o = /* @__PURE__ */ new Map(), a = /* @__PURE__ */ new Set(), l = /* @__PURE__ */ new Map(), c = Sr();
    let h = false, d = false, u = false, p = 0;
    const f = () => u ? p : performance.now();
    let g = null, m = 0;
    const y = /* @__PURE__ */ new Map();
    let b = null, x = null;
    const _ = [], v = globalThis.__TRACE_LOGS === true, w = (M, B) => {
      globalThis.__ANIM_LOGS && console.log(`[anim t=${f().toFixed(0)}ms] ${M}`, B);
    };
    function C(M, B) {
      const j = !y.get(M), J = {
        id: M,
        virtualMs: B
      };
      y.set(M, J), j && (b == null ? void 0 : b(J));
    }
    function T(M, B) {
      g === null && (g = M);
      const N = M + (B ? Vi : nh);
      N > m && (m = N);
    }
    function P(M, B) {
      if (M.length === 0) return;
      const N = f();
      for (const J of M) lr(J, c);
      const j = [
        ...M
      ].sort((J, st) => {
        const ht = (J.y ?? 0) - (st.y ?? 0);
        return ht !== 0 ? ht : (J.x ?? 0) - (st.x ?? 0);
      });
      j.forEach((J, st) => {
        const ht = N + st * B, ut = ps(J);
        if (a.has(ut)) return;
        a.add(ut), Jo(i, J, ht, c), l.has(ut) || l.set(ut, ht), kc(i, J.x ?? 0, J.y ?? 0);
        const mt = o.get(sh(J.x ?? 0, J.y ?? 0));
        mt && mt.fadeOutStartMs === null && (mt.fadeOutStartMs = ht, T(ht, false));
      }), T(N + (j.length - 1) * B, true);
    }
    function E(M, B, N, j, J, st) {
      const ht = sh(M, B), ut = o.get(ht);
      ut && ut.specKey === J || (eb(i, M, B, N, j, st, J), ut || o.set(ht, {
        specKey: J,
        fadeStartMs: st,
        fadeOutStartMs: null
      }), T(st, true));
    }
    function A(M) {
      const B = Ke(M.length, An, kn);
      w("rows_placed", {
        count: M.length,
        stagger_ms: B,
        span_ms: M.length * B
      }), P(M, B);
    }
    function D(M) {
      const B = f(), N = Sv(M.spec_key), j = M.tiles;
      if (j.length === 0) return;
      const J = Ke(j.length, _v, wv);
      w("ghost_routed", {
        spec_key: M.spec_key,
        item: N,
        tile_count: j.length,
        span_ms: j.length * J
      });
      for (let st = 0; st < j.length; st++) {
        const [ht, ut] = j[st];
        let mt = 0, bt = 0;
        st < j.length - 1 ? (mt = j[st + 1][0] - ht, bt = j[st + 1][1] - ut) : st > 0 && (mt = ht - j[st - 1][0], bt = ut - j[st - 1][1]), E(ht, ut, Tv(mt, bt), N, M.spec_key, B + st * J);
      }
    }
    function O(M) {
      const B = M.entities.length, N = Ke(B, An, kn);
      w("committed", {
        source: "spec",
        count: B,
        span_ms: B * N
      }), P(M.entities, N);
    }
    function H(M) {
      const B = f(), N = M.zone_x, j = M.zone_y, J = M.zone_x + M.zone_w - 1, st = M.zone_y + M.zone_h - 1;
      for (const [mt, bt] of o.entries()) {
        const [X, it] = mt.split(",").map(Number);
        X < N || X > J || it < j || it > st || bt.fadeOutStartMs === null && (bt.fadeOutStartMs = B, T(B, false), kc(i, X, it));
      }
      for (const mt of _) mt.clusterId === M.cluster_id && (mt.cleared = true);
      for (let mt = M.zone_y; mt <= st; mt++) for (let bt = M.zone_x; bt <= J; bt++) {
        const X = sb(i, bt, mt);
        for (const it of X) a.delete(it);
      }
      const ht = M.entities.length, ut = Ke(ht, vv, Cv);
      w("junction", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        count: ht,
        span_ms: ht * ut
      }), P(M.entities, ut);
    }
    function G(M) {
      if (d = true, M.phase === "rows_placed") {
        A(M.entities);
        return;
      }
      if (M.phase !== "lanes_planned") {
        if (M.phase === "bus_routed") {
          const B = M.entities.filter((N) => !a.has(ps(N)));
          if (B.length > 0) {
            const N = Ke(B.length, An, kn);
            P(B, N);
          }
          return;
        }
        if (M.phase === "poles_placed") {
          const B = M.entities.filter((N) => !a.has(ps(N)));
          if (B.length > 0) {
            const N = Ke(B.length, An, kn);
            P(B, N);
          }
          return;
        }
      }
    }
    function L(M) {
      const B = f();
      w("cluster_outline", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        lifetime_ms: Us,
        fade_ms: Xi
      }), _.push({
        clusterId: M.cluster_id,
        x: M.zone_x,
        y: M.zone_y,
        w: M.zone_w,
        h: M.zone_h,
        startMs: B,
        cleared: false
      }), T(B, true);
    }
    const $ = () => {
      if (h) return;
      const M = f();
      xu(i, M);
      for (const [B, N] of o.entries()) if (N.fadeOutStartMs !== null && M >= N.fadeOutStartMs) {
        const [j, J] = B.split(",").map(Number);
        (M - N.fadeOutStartMs) / nh >= 1 && o.delete(B);
      }
      for (const B of i.ghostContainer.particleChildren) B.alpha < 0.5 && (B.alpha = Math.min(0.5, B.alpha + 16 / Vi));
      r.clear();
      for (let B = _.length - 1; B >= 0; B--) {
        const N = _[B], j = M - N.startMs;
        if (!(j < 0)) if (N.cleared || j >= Us) {
          const J = N.cleared ? Math.max(j, Us - Xi) : j;
          if (J >= Us) {
            _.splice(B, 1);
            continue;
          }
          const st = Math.max(0, 1 - (J - (Us - Xi)) / Xi);
          r.rect(N.x * S, N.y * S, N.w * S, N.h * S), r.stroke({
            width: 2,
            color: 4508927,
            alpha: 0.9 * st
          });
        } else r.rect(N.x * S, N.y * S, N.w * S, N.h * S), r.stroke({
          width: 2,
          color: 4508927,
          alpha: 0.9
        });
      }
    };
    t.ticker.add($), Tr();
    let K = true;
    w("streaming_start", {});
    function U(M, B) {
      if (h || u) return;
      b = B ?? null, v && console.log(`[stream t=${f().toFixed(0)}] ${M.phase}`, "data" in M ? M.data : void 0);
      const N = f();
      switch (g === null && (g = N), M.phase) {
        case "PhaseSnapshot": {
          const j = M.data;
          j.phase === "rows_placed" && C("machines", N), j.phase === "poles_placed" && C("poles", N), G(j);
          break;
        }
        case "GhostSpecRouted":
          C("ghost_routes", N), D(M.data);
          break;
        case "GhostSpecCommitted":
          C("committed_routes", N), O(M.data);
          break;
        case "JunctionCommitted":
          C("junctions", N), H(M.data);
          break;
        case "GhostClusterSolved":
          L(M.data);
          break;
        case "TrunkBeltCommitted": {
          const j = M.data, J = j.entities.length, st = Ke(J, An, kn);
          w("committed", {
            source: "trunk",
            count: J,
            span_ms: J * st
          }), P(j.entities, st);
          break;
        }
        case "BalancerCommitted": {
          const j = M.data, J = j.entities.length, st = Ke(J, An, kn);
          w("committed", {
            source: "balancer",
            count: J,
            span_ms: J * st
          }), P(j.entities, st);
          break;
        }
        case "OutputMergerCommitted": {
          const j = M.data, J = j.entities.length, st = Ke(J, An, kn);
          w("committed", {
            source: "merger",
            count: J,
            span_ms: J * st
          }), P(j.entities, st);
          break;
        }
        case "PolesCommitted": {
          const j = M.data, J = j.entities.length, st = Ke(J, An, kn);
          w("committed", {
            source: "poles",
            count: J,
            span_ms: J * st
          }), P(j.entities, st);
          break;
        }
        default: {
          M.phase === "LayoutRetried" && (i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), Xe());
          break;
        }
      }
      b = null;
    }
    return {
      onEvent: U,
      hasCommittedEntities: () => d,
      cancel() {
        h || (h = true, t.ticker.remove($), K && (Jn(), K = false), i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), x = null, Xe());
      },
      finish(M) {
        t.ticker.remove($), K && (Jn(), K = false), nb(i), w("streaming_finish", {
          entity_count: i.count(),
          latest_fade_end_ms: m
        });
        const B = g ?? 0, N = [];
        for (const { particle: st, iconParticle: ht, badgeParticle: ut, revealAt: mt } of Mc()) N.push({
          kind: "particle",
          particle: st,
          iconParticle: ht,
          badgeParticle: ut,
          revealAt: mt
        });
        const j = M.entities.filter((st) => !a.has(ps(st)));
        if (j.length > 0) {
          for (const ht of j) lr(ht, c);
          for (const ht of j) Jo(i, ht, B, c), a.add(ps(ht));
          const st = new Set(N.map((ht) => ht.particle));
          for (const { particle: ht, iconParticle: ut, badgeParticle: mt, revealAt: bt } of Mc()) st.has(ht) || N.push({
            kind: "particle",
            particle: ht,
            iconParticle: ut,
            badgeParticle: mt,
            revealAt: bt
          });
        }
        const J = ib(i, c);
        if (J.size > 0) for (const st of N) {
          const ht = J.get(st.particle);
          ht && (st.particle = ht);
        }
        return x = N, u = true, p = m, Z(p), ab(M, t);
      },
      seekTo(M) {
        if (h || x === null) return;
        const B = g ?? 0, N = Math.max(m, B);
        p = Math.min(N, Math.max(B, M)), w("scrub", {
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
        return Array.from(y.values()).sort((M, B) => M.virtualMs - B.virtualMs);
      }
    };
    function Z(M) {
      if (x !== null) {
        for (const B of x) {
          const N = M - B.revealAt, j = N <= 0 ? 0 : N >= Vi ? 1 : N / Vi;
          B.particle.alpha = j, B.iconParticle && (B.iconParticle.alpha = j), B.badgeParticle && (B.badgeParticle.alpha = j);
        }
        Xe();
      }
    }
  }
  const Av = 0.03, kv = 200, xo = [
    "machines",
    "ghost_routes",
    "committed_routes",
    "junctions",
    "poles",
    "optimizing"
  ], Mv = {
    machines: "Machines",
    ghost_routes: "Belt routes",
    committed_routes: "Belts placed",
    junctions: "Crossings",
    poles: "Power poles",
    optimizing: "Optimizing"
  };
  function Pv(n, t) {
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
    for (const H of xo) {
      const G = document.createElement("div");
      G.className = "ts-chip", G.dataset.milestone = H, G.textContent = Mv[H], H === "optimizing" && (G.style.display = "none"), s.appendChild(G), l.set(H, G);
    }
    let c = false, h = null, d = [];
    const u = /* @__PURE__ */ new Set();
    let p = null;
    function f(H) {
      o.style.width = `${H * 100}%`, a.style.left = `${H * 100}%`;
    }
    function g(H) {
      var _a2, _b2;
      p !== H && (p && ((_a2 = l.get(p)) == null ? void 0 : _a2.classList.remove("ts-chip--active")), H && ((_b2 = l.get(H)) == null ? void 0 : _b2.classList.add("ts-chip--active")), p = H);
    }
    function m(H, G) {
      var _a2;
      if (c) return;
      const L = H.id;
      u.add(L), (_a2 = l.get(L)) == null ? void 0 : _a2.classList.add("ts-chip--reached"), g(L);
      const $ = Math.max(1, G.lastMs - G.firstMs), K = (H.virtualMs - G.firstMs) / $;
      f(Math.min(1, Math.max(0, K))), e.classList.add("ts-visible");
    }
    function y(H) {
      return h ? h.firstMs + H * (h.lastMs - h.firstMs) : 0;
    }
    function b(H) {
      for (const G of d) if (Math.abs(H - G.frac) < Av) return {
        frac: G.frac,
        snapped: true
      };
      return {
        frac: H,
        snapped: false
      };
    }
    function x(H) {
      if (!h) return;
      const G = i.getBoundingClientRect(), L = (H - G.left) / G.width, $ = Math.min(1, Math.max(0, L)), { frac: K, snapped: U } = b($);
      f(K), U ? a.classList.add("ts-thumb--snapped") : a.classList.remove("ts-thumb--snapped"), t(y(K));
    }
    let _ = null, v = null;
    function w(H) {
      if (!c || !h) return;
      H.preventDefault();
      try {
        i.setPointerCapture(H.pointerId);
      } catch {
      }
      const G = ($) => x($.clientX), L = ($) => {
        _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), _ = null, v = null, a.classList.remove("ts-thumb--snapped");
      };
      _ = G, v = L, document.addEventListener("pointermove", G), document.addEventListener("pointerup", L, {
        once: true
      }), x(H.clientX);
    }
    i.addEventListener("pointerdown", w);
    function C(H, G) {
      const L = H.lastMs - H.firstMs;
      if (L < kv || G.length === 0) {
        A();
        return;
      }
      c = true, h = H, e.classList.add("ts-scrub-mode"), e.classList.add("ts-visible"), d = G.map(($) => ({
        id: $.id,
        frac: ($.virtualMs - H.firstMs) / L
      })), s.style.justifyContent = "flex-start", s.style.position = "relative";
      for (const $ of l.values()) $.style.position = "absolute", $.style.transform = "translateX(-50%)";
      for (const $ of xo) {
        const K = l.get($);
        if (!K) continue;
        const U = d.find((Z) => Z.id === $);
        U ? (K.style.left = `${U.frac * 100}%`, K.style.display = "", K.classList.add("ts-chip--reached")) : K.style.display = "none";
      }
      P(), requestAnimationFrame(E), f(1), g(null);
    }
    let T = null;
    function P() {
      if (T && T.remove(), !h) return;
      const H = document.createElement("div");
      H.className = "ts-ticks";
      for (const G of d) {
        const L = document.createElement("div");
        L.className = "ts-tick", L.style.left = `${G.frac * 100}%`, H.appendChild(L);
      }
      i.appendChild(H), T = H;
    }
    function E() {
      if (!c) return;
      const H = 6, G = s.clientWidth;
      if (G <= 0) return;
      const L = xo.map((K) => {
        var _a2;
        const U = l.get(K);
        if (!U || U.style.display === "none") return null;
        const Z = K === "optimizing" ? 1 : ((_a2 = d.find((M) => M.id === K)) == null ? void 0 : _a2.frac) ?? 0;
        return {
          el: U,
          originalFrac: Z
        };
      }).filter((K) => K !== null);
      let $ = -1 / 0;
      for (const { el: K, originalFrac: U } of L) {
        const Z = K.offsetWidth / 2, M = U * G, B = $ + Z + H, N = Math.max(M, B);
        K.style.left = `${N / G * 100}%`, $ = N + Z;
      }
    }
    function A() {
      c = false, h = null, d = [], u.clear(), p = null, T && (T.remove(), T = null), e.classList.remove("ts-visible", "ts-scrub-mode"), o.style.width = "0", a.style.left = "0", a.classList.remove("ts-thumb--snapped"), s.style.justifyContent = "space-between", s.style.position = "";
      for (const [H, G] of l) G.style.position = "", G.style.transform = "", G.style.left = "", G.style.display = H === "optimizing" ? "none" : "", G.classList.remove("ts-chip--reached", "ts-chip--active", "ts-chip--in-progress");
    }
    function D(H) {
      const G = l.get("optimizing");
      if (G) {
        if (G.classList.remove("ts-chip--in-progress", "ts-chip--reached"), H === "idle") {
          G.style.display = "none";
          return;
        }
        G.style.display = "", e.classList.add("ts-visible"), c && (G.style.position = "absolute", G.style.transform = "translateX(-50%)", G.style.left = "100%", requestAnimationFrame(E)), H === "active" ? G.classList.add("ts-chip--in-progress") : H === "done" && G.classList.add("ts-chip--reached");
      }
    }
    function O() {
      _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), i.removeEventListener("pointerdown", w), e.remove();
    }
    return {
      noteMilestone: m,
      arm: C,
      markOptimizeState: D,
      reset: A,
      destroy: O
    };
  }
  const Iv = `
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
  function Rv() {
    if (document.getElementById("spaghettio-busy-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-busy-style", n.textContent = Iv, document.head.appendChild(n);
  }
  const Lv = 120;
  function $v(n) {
    Rv();
    const t = document.createElement("div");
    t.className = "spaghettio-busy";
    const e = document.createElement("span");
    e.className = "spaghettio-busy-spin", t.appendChild(e);
    const s = document.createElement("span");
    s.textContent = "computing\u2026", t.appendChild(s), n.appendChild(t);
    let i = null;
    t_((r) => {
      r > 0 ? i === null && !t.classList.contains("visible") && (i = setTimeout(() => {
        t.classList.add("visible"), i = null;
      }, Lv)) : (i !== null && (clearTimeout(i), i = null), t.classList.remove("visible"));
    });
  }
  function Je(n, t) {
    return n.filter((e) => e.phase === t);
  }
  const Dn = "color:#9cdcfe;font-weight:bold", pe = "color:#888", ve = "color:#e0e0e0", us = "color:#6a6", qi = "color:#ffaa00", bo = "color:#f66", Ki = "color:#c586c0";
  function Bv(n) {
    var _a2;
    const t = Array.isArray(n.trace) ? n.trace : [];
    if (t.length === 0) return;
    const e = Je(t, "PhaseTime"), s = Je(t, "SatInvocation"), i = Je(t, "JunctionSolved"), r = Je(t, "JunctionGrowthCapped"), o = Je(t, "GhostClusterSolved"), a = Je(t, "GhostRoutingComplete"), l = Je(t, "RegionWalkerVeto"), c = Je(t, "JunctionGrowthIteration"), h = Je(t, "NegotiateComplete"), d = Je(t, "ValidationCompleted"), u = t.filter((x) => x.phase === "LayoutRetried"), p = e.reduce((x, _) => x + _.data.duration_ms, 0), f = s.reduce((x, _) => x + _.data.solve_time_us, 0), g = s.filter((x) => x.data.satisfied).length, m = ((_a2 = n.entities) == null ? void 0 : _a2.length) ?? 0, y = u.length > 0 ? r.length > 0 ? ` \xB7 ${r.length} capped \xB7 retried (${u[0].data.caps_before} caps recovered)` : ` \xB7 retried (${u[0].data.caps_before} caps recovered)` : r.length > 0 ? ` \xB7 ${r.length} capped` : "", b = r.length > 0 ? qi : u.length > 0 ? Ki : us;
    if (console.log(`%c\u25B6 layout %c${n.width}\xD7${n.height}  %c${m} entities  %c${p}ms  %cSAT ${Math.round(f / 1e3)}ms (${s.length}\xD7)%c${y}`, Dn, ve, pe, ve, Ki, b), console.groupCollapsed("%c  \u21B3 breakdown", pe), e.length > 0) {
      const x = [
        ...e
      ].sort((_, v) => v.data.duration_ms - _.data.duration_ms);
      console.log(`%cphases%c ${p}ms total`, Dn, pe);
      for (const _ of x) {
        const v = _.data, w = p > 0 ? v.duration_ms / p * 100 : 0, C = Math.max(1, Math.round(w / 100 * 24)), T = "\u2588".repeat(C);
        console.log(`  %c${v.phase.padEnd(18)}%c ${String(v.duration_ms).padStart(5)}ms  %c${T}%c ${w.toFixed(1)}%`, pe, ve, Ki, pe);
      }
    }
    if (s.length > 0) {
      const x = f / 1e3, _ = p > 0 ? x / p * 100 : 0, v = f / s.length, w = [
        ...s
      ].sort((T, P) => P.data.solve_time_us - T.data.solve_time_us)[0], C = [
        ...s
      ].sort((T, P) => P.data.zone_w * P.data.zone_h - T.data.zone_w * T.data.zone_h)[0];
      console.log(`%cSAT%c ${s.length} invocations \xB7 ${x.toFixed(1)}ms (%c${_.toFixed(1)}%%%c of total)`, Dn, ve, Ki, ve), console.log(`  %csatisfied%c ${g}  %cunsat%c ${s.length - g}  %cavg%c ${(v / 1e3).toFixed(2)}ms`, pe, us, pe, bo, pe, ve), w && console.log(`  %cslowest call%c ${(w.data.solve_time_us / 1e3).toFixed(1)}ms \u2014 %c${w.data.zone_w}\xD7${w.data.zone_h} @ (${w.data.zone_x},${w.data.zone_y}), ${w.data.variables} vars, ${w.data.clauses} clauses`, pe, ve, pe), C && C !== w && console.log(`  %cbiggest zone%c ${C.data.zone_w}\xD7${C.data.zone_h} @ (${C.data.zone_x},${C.data.zone_y}) \u2014 ${C.data.variables} vars`, pe, ve);
    }
    if (o.length > 0 || i.length > 0 || r.length > 0) {
      if (console.log("%cjunctions", Dn), console.log(`  %cclusters%c ${o.length}  %csolved%c ${i.length}  %ccapped%c ${r.length}  %cvetoes%c ${l.length}`, pe, ve, pe, us, pe, r.length > 0 ? qi : ve, pe, ve), c.length > 0) {
        const x = /* @__PURE__ */ new Map();
        for (const v of c) {
          const w = `${v.data.seed_x},${v.data.seed_y}`;
          x.set(w, Math.max(x.get(w) ?? 0, v.data.iter));
        }
        const _ = [
          ...x.entries()
        ].sort((v, w) => w[1] - v[1])[0];
        _ && _[1] > 0 && console.log(`  %chardest%c junction at (${_[0]}) needed ${_[1] + 1} growth iters`, pe, ve);
      }
      if (r.length > 0) for (const x of r) console.log(`    %c\u26A0 capped at (${x.data.tile_x},${x.data.tile_y})%c \u2014 ${x.data.reason}, ${x.data.region_tiles} tiles after ${x.data.iters} iters`, qi, pe);
    }
    if (a.length > 0) {
      const x = a[0].data, _ = x.unroutable_count > 0 ? bo : us;
      console.log(`%cghost router%c ${x.entity_count} routed entities, ${x.cluster_count} clusters, max cluster ${x.max_cluster_tiles} tiles  %c${x.unroutable_count} unroutable`, Dn, ve, _);
    }
    if (h.length > 0) {
      const x = h[0].data;
      console.log(`%cA* negotiate%c ${x.specs} specs, ${x.iterations} iters, ${x.duration_ms}ms`, Dn, ve);
    }
    if (d.length > 0) {
      const x = d[0].data, _ = x.error_count > 0 ? bo : us, v = x.warning_count > 0 ? qi : us;
      console.log(`%cvalidation  %c${x.error_count} errors  %c${x.warning_count} warnings`, Dn, _, v);
    }
    console.groupEnd();
  }
  const Ov = [
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
  async function Nv() {
    await Au();
    const n = Mu();
    await w0(Ov), await T0();
    const t = document.getElementById("app"), e = window.location.hash, s = new URLSearchParams(window.location.search);
    if (e.startsWith("#/balancers")) {
      const { renderBalancerShowcase: r } = await Rn(async () => {
        const { renderBalancerShowcase: o } = await import("./balancers-Bp27teVb.js");
        return {
          renderBalancerShowcase: o
        };
      }, []);
      r(t, n);
      return;
    }
    if (!(e.startsWith("#/layout") || s.has("generator") || Db())) {
      const r = document.createElement("div");
      t.appendChild(r), w_(r, n, {
        onOpenGenerator: () => {
          r.remove(), ih(n), window.history.replaceState({}, "", "#/layout");
        }
      });
      return;
    }
    ih(n);
  }
  async function ih(n) {
    const t = document.getElementById("canvas-container");
    if (!t) throw new Error("Missing #canvas-container element");
    const e = document.getElementById("app");
    e.style.display = "flex";
    const s = document.getElementById("sidebar");
    s && (s.style.display = ""), t.style.display = "";
    const { app: i, viewport: r, requestRender: o, beginAnimating: a, endAnimating: l } = await O0(t), c = N0(r);
    let h = false;
    Hs(r, null), nv();
    const d = iv(t), { debugCb: u, colorCb: p, heatmapCb: f, powerWiresCb: g, moduleSlotsCb: m, regionsCb: y, soloRegionsCb: b, ghostTilesCb: x, traceOverlayCb: _ } = d, v = rv(t);
    Wi(p.checked);
    const w = () => {
      globalThis.__ANIM_LOGS = u.checked;
    };
    u.addEventListener("change", w), w();
    const C = av(t), T = Rw();
    let P = null;
    const E = zw(t, r, {
      onChange: (W) => {
        if (T.update(W), W) {
          O.alpha = A.isActive() ? 0.2 : 0.35;
          const at = W.iter.bbox;
          P = {
            bboxX: at.x,
            bboxY: at.y,
            bboxW: at.w,
            bboxH: at.h
          };
        } else O.alpha = 1, P = null, A.isActive() && A.exit();
        o();
      },
      onEditRequested: (W) => {
        O.alpha = 0.2, A.enter(W), o();
      }
    }), A = ev({
      viewport: r,
      canvas: i.canvas,
      engine: n,
      jd: E,
      satZoneOverlayLayer: T.layer
    });
    D_(t, (W) => kt.load(W));
    function D(W) {
      O.removeChildren();
      const at = gu();
      return at.attachTo(O), rb(W, at, i);
    }
    const O = new zt();
    O.isRenderGroup = true, O.eventMode = "none", r.addChild(O);
    const H = new zt();
    H.eventMode = "none", r.addChild(H), r.addChild(T.layer);
    const G = new ft();
    G.label = "pin-highlight", r.addChild(G), C.onPinChange((W) => {
      if (G.clear(), W) {
        const at = W.x * S, dt = W.y * S;
        G.setStrokeStyle({
          width: 2,
          color: 8440063,
          alpha: 0.95
        }), G.rect(at - 2, dt - 2, S + 4, S + 4).stroke();
      }
      o();
    }), r.moveCenter(nn / 2, nn / 2);
    const L = (W) => {
    };
    let $ = null;
    function K(W) {
      $ = W, C.onHover(W, W == null ? void 0 : W.x, W == null ? void 0 : W.y);
    }
    let U = /* @__PURE__ */ new Map();
    function Z(W) {
      const at = /* @__PURE__ */ new Map();
      for (const dt of W.entities) {
        const _t = dt.x ?? 0, ct = dt.y ?? 0, $t = Ee[dt.name];
        if ($t) {
          const [Ft, Pt] = $t;
          for (let Ot = 0; Ot < Pt; Ot++) for (let be = 0; be < Ft; be++) at.set(`${_t + be},${ct + Ot}`, dt);
        } else if (oe.has(dt.name)) {
          at.set(`${_t},${ct}`, dt);
          const [Ft, Pt] = Ta(dt.direction);
          at.set(`${_t + Ft},${ct + Pt}`, dt);
        } else at.set(`${_t},${ct}`, dt);
      }
      U = at;
    }
    function M(W) {
      return {
        highlightItem: (at) => {
          W.highlightItem(at), o();
        },
        highlightBeltNetwork: (at) => {
          W.highlightBeltNetwork(at), o();
        },
        clearHighlight: () => {
          W.clearHighlight(), o();
        },
        chainKey: W.chainKey
      };
    }
    let B = false, N = null, j = null, J = false, st = null;
    const ht = {
      update() {
      },
      getPhaseIndex() {
        return -1;
      },
      reset() {
      }
    };
    function ut() {
      var _a2;
      j && (O.removeChild(j), j.destroy(), j = null);
      const W = ht.getPhaseIndex();
      if (u.checked && W >= 0, J && (st == null ? void 0 : st.cancel(), st = null, J = false, Y)) {
        const dt = ni(Y, O, K, L);
        C.setHighlightController(M(dt)), o();
      }
      if (!u.checked || !_.checked || !((_a2 = Y == null ? void 0 : Y.trace) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      const at = Y.trace;
      j = U_(at, Y.width ?? 0, Y.height ?? 0, O, (dt) => {
        C.setTooltipOverride(dt ? `<span style="color:#8af">TRACE</span> ${dt}` : null);
      }), o();
    }
    let mt = null, bt = null, X = null, it = [], rt = null, et = null, St = null, Lt = null;
    const Ht = document.createElement("div");
    Ht.className = "validation-badge", Ht.style.display = "none", t.appendChild(Ht);
    function Vt(W) {
      if (!W || W.length === 0) {
        Ht.style.display = "none";
        return;
      }
      const at = W.filter((ct) => ct.severity === "Error").length, dt = W.length - at;
      let _t;
      at > 0 && dt > 0 ? _t = `\u26A0 ${at} error${at > 1 ? "s" : ""}, ${dt} warning${dt > 1 ? "s" : ""}` : at > 0 ? _t = `\u26A0 ${at} error${at > 1 ? "s" : ""}` : _t = `\u26A0 ${dt} warning${dt > 1 ? "s" : ""}`, Ht.textContent = _t, Ht.classList.toggle("has-errors", at > 0), Ht.style.display = "block";
    }
    let Xt = null, Zt = null, Et = null, Bt = null, qt = null;
    const V = 1;
    function nt(W, at) {
      const dt = W * S + S / 2, _t = at * S + S / 2;
      r.scale.x < V && r.setZoom(V, false), r.moveCenter(dt, _t);
    }
    function tt(W) {
      var _a2, _b2;
      const at = [];
      for (const dt of W.regions ?? []) {
        if (dt.kind !== "unresolved") continue;
        const _t = dt.x + Math.floor(dt.width / 2), ct = dt.y + Math.floor(dt.height / 2), $t = ((_b2 = (_a2 = dt.ports) == null ? void 0 : _a2.find((Ft) => Ft.item)) == null ? void 0 : _b2.item) ?? "unknown";
        at.push({
          severity: "Warning",
          category: `ghost-router \xB7 ${$t}`,
          message: `unresolved crossing at (${_t}, ${ct})`,
          x: _t,
          y: ct
        });
      }
      for (const dt of W.warnings ?? []) /^ghost router:.*unresolved crossings/i.test(dt) || at.push({
        severity: "Warning",
        category: "layout",
        message: dt,
        x: void 0,
        y: void 0
      });
      return at;
    }
    function Ct() {
      if (mt && (O.removeChild(mt), mt.destroy(), mt = null), Y && !bt && Lt !== Y) {
        const ct = Y;
        Lt = ct, n.validateLayout(ct, q).then(($t) => {
          Y === ct && (bt = $t, Lt = null, Ct(), Er(ct));
        }).catch(() => {
          Y === ct && (bt = [], Lt = null, Ct(), Er(ct));
        });
      }
      const W = Y ? tt(Y) : [], at = [
        ...bt ?? [],
        ...W
      ], dt = (ct) => {
        if (ct.x == null || ct.y == null || !St) return null;
        const $t = St.lookup(ct.x, ct.y).cappedSides;
        return $t.length === 0 ? null : [
          ...new Set($t.map((Pt) => Pt.limit))
        ].sort().join("+");
      };
      if (Ls == null ? void 0 : Ls.updateValidation(at, nt, dt), Vt(at), Ut(at), !Y || at.length === 0) {
        o();
        return;
      }
      mt = X_(at, O, (ct) => {
        C.setTooltipOverride(ct ? `<span style="color:#f44">VALIDATION</span> ${ct}` : null);
      }).layer, o();
    }
    function Ut(W) {
      if (W && (it = W), X && (O.removeChild(X), X.destroy({
        children: true
      }), X = null), !f.checked || !Y || it.length === 0) {
        o();
        return;
      }
      X = K_(it, Y.entities, O), o();
    }
    function ye() {
      var _a2;
      if (rt && (O.removeChild(rt), rt.destroy({
        children: true
      }), rt = null), !g.checked || !((_a2 = Y == null ? void 0 : Y.power_wires) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      rt = J_(Y.power_wires, Y.entities, O), o();
    }
    function Gt() {
      var _a2;
      if (et && (O.removeChild(et), et.destroy({
        children: true
      }), et = null), !m.checked || !((_a2 = Y == null ? void 0 : Y.entities) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      et = tw(Y.entities, Mu().moduleSlots, O), o();
    }
    function re() {
      if (qt && (r.removeChild(qt), qt.destroy({
        children: true
      }), qt = null), !u.checked || !x.checked || !Y) {
        o();
        return;
      }
      const W = Gw(Y.trace);
      if (!W) {
        o();
        return;
      }
      qt = W, r.addChildAt(qt, 0), o();
    }
    function k() {
      var _a2;
      if (Xt && (O.removeChild(Xt), Xt.destroy(), Xt = null), Et && (O.removeChild(Et), Et.destroy(), Et = null), Zt = null, Bt = null, !u.checked || !(y == null ? void 0 : y.checked) || !Y) {
        o();
        return;
      }
      if (Y.regions && Y.regions.length > 0) {
        const W = hw(Y);
        Xt = W.layer, Zt = W.hitTest, O.addChild(Xt);
      }
      if ((_a2 = Y.trace) == null ? void 0 : _a2.length) {
        const W = mw(Y.trace);
        if (W.length > 0) {
          const at = ww(W);
          Et = at.layer, Bt = at.hitTest, O.addChild(Et);
        }
      }
      o();
    }
    const I = document.createElement("div");
    I.style.cssText = "position:absolute;bottom:34px;left:8px;background:rgba(0,0,0,0.8);color:#e0e0e0;font:11px monospace;padding:6px 8px;border-radius:3px;border:1px solid #00e0a0;z-index:10;display:none;min-width:200px", t.appendChild(I);
    const z = document.createElement("div");
    z.style.cssText = "color:#00e0a0;margin-bottom:4px", I.appendChild(z);
    const Q = document.createElement("textarea");
    Q.placeholder = "Add a note\u2026", Q.rows = 2, Q.style.cssText = "width:100%;box-sizing:border-box;background:#2a2a2a;color:#e0e0e0;border:1px solid #555;border-radius:2px;font:11px monospace;resize:vertical;margin-bottom:4px", I.appendChild(Q);
    const R = document.createElement("div");
    R.style.cssText = "color:#777", R.textContent = "Ctrl+C to copy JSON", I.appendChild(R);
    let F = false, Y = null, q = null, ot = null, gt = null, pt = null;
    const yt = Pv(t, (W) => pt == null ? void 0 : pt.seekTo(W));
    $v(t);
    const kt = dv({
      sidebarEl: document.getElementById("sidebar"),
      getSidebarCtrl: () => Ls,
      renderLayoutOnCanvas: ss,
      setCachedValidationIssues: (W) => {
        bt = W;
      },
      updateValidationOverlay: Ct,
      panToTile: nt,
      onDebugEnable: () => d.setDebugEnabled(true),
      onClear: () => {
        kt.clear(), O.removeChildren(), H.removeChildren(), C.clearPin(), C.setTileContext(null), Y = null, q = null, v.update(null), U = /* @__PURE__ */ new Map(), bt = null, Hs(r, null), r.moveCenter(nn / 2, nn / 2), Vt(null), Ls == null ? void 0 : Ls.updateValidation([], nt), E.close();
      }
    });
    function se(W) {
      W.length === 0 ? (I.style.display = "none", Q.value = "") : (z.textContent = `${W.length} entit${W.length === 1 ? "y" : "ies"} selected`, I.style.display = "block");
    }
    async function ue(W) {
      if (F || !(W.regions ?? []).some((Nt) => Nt.kind === "crossing_zone")) return;
      F = true, yt.markOptimizeState("active"), ot && (ot.destroy(), ot = null);
      const dt = (Nt) => Nt === "transport-belt" || Nt === "fast-transport-belt" || Nt === "express-transport-belt" || Nt === "underground-belt" || Nt === "fast-underground-belt" || Nt === "express-underground-belt", _t = [], ct = 130, $t = 90, Ft = 520;
      let Pt = 0, Ot = -1, be = 0, we = false, Nn = false, is = null;
      const Gu = (Nt) => {
        if (!Y) return;
        const qe = Nt.zone_x, De = Nt.zone_y, rs = qe + Nt.zone_w, Ar = De + Nt.zone_h, Du = ($s) => {
          const Da = $s.x ?? 0, Ha = $s.y ?? 0;
          return Da >= qe && Da < rs && Ha >= De && Ha < Ar;
        };
        Y.entities = Y.entities.filter(($s) => !(Du($s) && dt($s.name))).concat(Nt.entities), bv(i, r, {
          x: qe,
          y: De,
          w: Nt.zone_w,
          h: Nt.zone_h
        }), D(Y), o();
      }, zu = () => we && _t.length === 0, za = (Nt) => {
        if (!Nn) {
          for (; _t.length > 0; ) {
            const qe = _t[0], De = qe.imp.region_id === Ot, rs = De ? Math.min(Ft, be * $t) : 0, Ar = ct + rs;
            if (Nt - Pt < Ar) break;
            _t.shift(), Gu(qe.imp), De ? be += 1 : (be = 1, Ot = qe.imp.region_id), Pt = Nt;
            break;
          }
          if (zu()) {
            is = null;
            return;
          }
          is = requestAnimationFrame(za);
        }
      };
      is = requestAnimationFrame(za);
      try {
        const Nt = await n.optimizeAllRegions(W, {
          perRegionBudgetMs: 800,
          onImprovement: (De) => {
            De.iter !== 0 && _t.push({
              imp: De
            });
          }
        });
        we = true, await new Promise((De) => {
          const rs = () => {
            _t.length === 0 ? De() : requestAnimationFrame(rs);
          };
          rs();
        }), Y = Nt, v.update(Nt), Z(Nt), window.__layout = Nt;
        const qe = D(Nt);
        C.setHighlightController(M(qe)), o();
      } catch (Nt) {
        (Nt instanceof Error ? Nt.message : String(Nt)).includes("superseded") || console.error("[auto-optimize] failed", Nt);
      } finally {
        Nn = true, we = true, is !== null && cancelAnimationFrame(is), F = false, yt.markOptimizeState("done"), Y && (ot = Lc(i.canvas, r, O, Y, se));
      }
    }
    i.canvas.addEventListener("pointermove", (W) => {
      const at = i.canvas.getBoundingClientRect(), dt = W.clientX - at.left, _t = W.clientY - at.top, ct = r.toWorld(dt, _t), $t = Math.floor(ct.x / S), Ft = Math.floor(ct.y / S);
      C.setCursorTile($t, Ft);
      const Pt = U.get(`${$t},${Ft}`) ?? null;
      Pt !== $ && K(Pt), $ || C.onHover(null, $t, Ft);
    }), i.canvas.addEventListener("pointerleave", () => {
      C.setCursorTile(null), $ && K(null);
    });
    const bi = 4;
    let Le = null;
    i.canvas.addEventListener("pointerdown", (W) => {
      if (W.button !== 0 || W.shiftKey || W.altKey || W.ctrlKey || W.metaKey) {
        Le = null;
        return;
      }
      Le = {
        x: W.clientX,
        y: W.clientY,
        shifted: false
      };
    }), i.canvas.addEventListener("pointerup", (W) => {
      if (!Le) return;
      const at = W.clientX - Le.x, dt = W.clientY - Le.y;
      if (Le = null, Math.hypot(at, dt) > bi || W.button !== 0 || W.shiftKey || W.altKey || W.ctrlKey || W.metaKey) return;
      const _t = i.canvas.getBoundingClientRect(), ct = r.toWorld(W.clientX - _t.left, W.clientY - _t.top), $t = Math.floor(ct.x / S), Ft = Math.floor(ct.y / S);
      if (!y.checked) {
        const we = $ && $.x === $t && $.y === Ft ? $ : null;
        if (!we) return;
        C.pinTile(we, $t, Ft);
        return;
      }
      const Pt = (Bt == null ? void 0 : Bt(ct.x, ct.y)) ?? null;
      if (Pt) {
        E.open(Pt, Y == null ? void 0 : Y.trace);
        return;
      }
      if (P) {
        const we = ct.x / S, Nn = ct.y / S;
        if (!(we >= P.bboxX && Nn >= P.bboxY && we < P.bboxX + P.bboxW && Nn < P.bboxY + P.bboxH)) {
          E.close();
          return;
        }
      }
      const Ot = (Zt == null ? void 0 : Zt(ct.x, ct.y)) ?? null;
      if (Ot) {
        const we = (Ot.region.x + Ot.region.width / 2) * S, Nn = (Ot.region.y + Ot.region.height / 2) * S;
        r.moveCenter(we, Nn);
      }
      const be = C.getPinnedTile();
      if (be && be.x === $t && be.y === Ft) C.clearPin();
      else {
        const we = $ && $.x === $t && $.y === Ft ? $ : null;
        if (!we) return;
        C.pinTile(we, $t, Ft);
      }
    }), document.addEventListener("keydown", (W) => {
      W.key === "Shift" && r.plugins.pause("drag");
    }), document.addEventListener("keyup", (W) => {
      W.key === "Shift" && r.plugins.resume("drag");
    }), window.addEventListener("blur", () => r.plugins.resume("drag"));
    function vn(W) {
      gt == null ? void 0 : gt.cancel(), gt = null, pt == null ? void 0 : pt.cancel(), pt = null, yt.reset(), O.removeChildren(), H.removeChildren(), q = W, Hs(r, W), W || r.moveCenter(nn / 2, nn / 2);
    }
    function _i() {
      pt == null ? void 0 : pt.cancel(), yt.reset(), Hs(r, null), pt = Ev(O, i);
      let W = false, at = null;
      return (dt) => {
        if (dt.phase === "PhaseSnapshot") {
          const ct = dt.data;
          ct.width > 0 && ct.height > 0 && (W || (r.fit(true, ct.width * S * 1.15, ct.height * S * 1.25), r.moveCenter(ct.width * S / 2, ct.height * S / 2), W = true), at ? at.resize(ct.width + 2, ct.height + 2) : at = wi(ct.width + 2, ct.height + 2));
        }
        pt == null ? void 0 : pt.onEvent(dt, (ct) => {
          pt && yt.noteMilestone(ct, pt.getTimeRange());
        });
      };
    }
    function wi(W, at) {
      let dt = W, _t = at, ct = false;
      if (h) return zn(c, dt, _t), o(), ct = true, {
        cancel: () => {
        },
        resize(Ot, be) {
          zn(c, Ot, be), o();
        }
      };
      const $t = 250, Ft = performance.now();
      c.alpha = 1, zn(c, dt, _t, 0);
      const Pt = () => {
        if (ct) return;
        const Ot = Math.min(1, (performance.now() - Ft) / $t);
        zn(c, dt, _t, Ot), o(), Ot >= 1 && (ct = true, h = true, i.ticker.remove(Pt), l());
      };
      return i.ticker.add(Pt), a(), {
        cancel() {
          ct || (ct = true, h = true, i.ticker.remove(Pt), l(), zn(c, dt, _t), o());
        },
        resize(Ot, be) {
          dt = Ot, _t = be, ct && (zn(c, dt, _t), o());
        }
      };
    }
    function ss(W, at) {
      iu(v0(W.entities)), Y = W, v.update(W), at && (q = at), Z(W), window.__layout = W, Bv(W), J = false, st == null ? void 0 : st.cancel(), st = null, ot && (ot.destroy(), ot = null), gt == null ? void 0 : gt.cancel(), gt = null, I.style.display = "none", Q.value = "", bt = null, Hs(r, null);
      let dt;
      if (pt == null ? void 0 : pt.hasCommittedEntities()) dt = pt.finish(W), yt.arm(pt.getTimeRange(), pt.getMilestones());
      else if (pt == null ? void 0 : pt.cancel(), pt = null, yt.reset(), (Array.isArray(W.trace) ? W.trace : []).some((Pt) => Pt.phase === "PhaseSnapshot")) {
        const Pt = yv(W, O, K, L, i);
        dt = Pt.controller, gt = Pt.handle;
      } else dt = D(W);
      C.setHighlightController(M(dt)), St = hv(W.trace), C.setTileContext(St), C.clearPin(), ot = Lc(i.canvas, r, O, W, se), ut(), Ct(), k(), re(), ye(), Gt(), _b(H, W, q);
      const _t = W.width ?? 0, ct = W.height ?? 0;
      if (zn(c, _t + 2, ct + 2), _t > 0 && ct > 0) {
        const $t = _t * 32, Ft = ct * 32, Pt = 192;
        r.fit(true, $t * 1.1, (Ft + Pt) * 1.2), r.moveCenter($t / 2, (Ft - Pt) / 2);
      }
      B && (O.alpha = 0.12), o(), requestAnimationFrame(() => {
        Y === W && Er(W);
      });
    }
    function Er(W) {
      Y !== W || Lt === W || bt === null || [
        ...bt,
        ...tt(W)
      ].length > 0 || ue(W);
    }
    document.addEventListener("keydown", (W) => {
      if (W.ctrlKey) {
        if (W.key === "c") {
          if (!ot || ot.getSelected().length === 0) return;
          W.preventDefault();
          const at = (Ls == null ? void 0 : Ls.getParams()) ?? null, dt = ot.buildJson(at, Q.value.trim());
          navigator.clipboard.writeText(dt).catch(() => {
          }), R.textContent = "Copied!", setTimeout(() => {
            R.textContent = "Ctrl+C to copy JSON";
          }, 2e3);
        } else if (W.key === "o") {
          W.preventDefault();
          const at = document.createElement("input");
          at.type = "file", at.accept = ".fls", at.addEventListener("change", async () => {
            var _a2;
            const dt = (_a2 = at.files) == null ? void 0 : _a2[0];
            if (dt) try {
              const _t = await dt.text(), ct = await Ou(_t);
              kt.load(ct);
            } catch (_t) {
              alert(`Failed to load snapshot: ${_t}`);
            }
          }), at.click();
        }
      }
    });
    const Rs = document.getElementById("sidebar");
    let Ls = null;
    if (Rs) {
      let W = function(Pt) {
        const Ot = document.createElement("button");
        return Ot.textContent = Pt, Ot.style.cssText = "flex:1;padding:10px 4px;background:none;border:none;border-bottom:2px solid transparent;color:#777;font:12px 'JetBrains Mono','Consolas',monospace;cursor:pointer;letter-spacing:0.5px;transition:all 0.15s", Ot;
      }, at = function(Pt) {
        const Ot = Pt === "generate";
        $t.style.display = Ot ? "flex" : "none", Ft.style.display = Ot ? "none" : "flex", _t.style.borderBottomColor = Ot ? "#569cd6" : "transparent", _t.style.color = Ot ? "#d4d4d4" : "#777", ct.style.borderBottomColor = Ot ? "transparent" : "#569cd6", ct.style.color = Ot ? "#777" : "#d4d4d4";
      };
      const dt = document.createElement("div");
      dt.style.cssText = "display:flex;border-bottom:1px solid #2a2a2a;background:#141414;flex-shrink:0";
      const _t = W("Generate"), ct = W("Corpus");
      dt.appendChild(_t), dt.appendChild(ct);
      const $t = document.createElement("div");
      $t.style.cssText = "flex:1;overflow:hidden;display:flex;flex-direction:column;";
      const Ft = document.createElement("div");
      Ft.style.cssText = "flex:1;overflow:hidden;display:none;flex-direction:column;", Rs.style.cssText += ";display:flex;flex-direction:column;padding:0;overflow:hidden;", Rs.appendChild(dt), Rs.appendChild($t), Rs.appendChild(Ft), _t.onclick = () => at("generate"), ct.onclick = () => at("corpus"), at("generate"), Ls = qb($t, n, {
        renderGraph: vn,
        renderLayout: ss,
        startStreaming: _i
      }), u.addEventListener("change", () => {
        ut(), Ct(), k(), re();
      }), x.addEventListener("change", () => {
        re();
      }), _.addEventListener("change", () => {
        ut();
      }), p.addEventListener("change", () => {
        Wi(p.checked), Y && ss(Y);
      }), f.addEventListener("change", () => {
        Ut();
      }), g.addEventListener("change", () => {
        ye();
      }), m.addEventListener("change", () => {
        Gt();
      }), y.addEventListener("change", () => {
        k();
      }), b.addEventListener("change", () => {
        const Pt = () => o();
        b.checked ? (B = true, N = {
          colorChecked: p.checked,
          regionsChecked: y.checked,
          entityAlpha: O.alpha
        }, y.checked || (y.checked = true, k()), p.checked && (p.checked = false, Wi(false), Y && ss(Y)), O.alpha = 0.12, k(), Pt()) : (B = false, N && (O.alpha = N.entityAlpha, y.checked !== N.regionsChecked && (y.checked = N.regionsChecked, k()), p.checked !== N.colorChecked && (p.checked = N.colorChecked, Wi(p.checked), Y && ss(Y)), N = null), Pt());
      }), y_(Ft, ss);
    }
  }
  Nv().catch((n) => {
    console.error("Failed to initialize app:", n);
  });
})();
export {
  Mh as $,
  da as A,
  nr as B,
  zt as C,
  _m as D,
  on as E,
  pf as F,
  gi as G,
  ts as H,
  Ss as I,
  he as J,
  ne as K,
  Kt as L,
  Ks as M,
  bh as N,
  vt as O,
  lt as P,
  ft as Q,
  Dt as R,
  gr as S,
  S as T,
  me as U,
  ky as V,
  wg as W,
  xr as X,
  ug as Y,
  _n as Z,
  br as _,
  __tla,
  lr as a,
  Rt as a0,
  Kh as a1,
  Mo as a2,
  Wt as a3,
  qn as a4,
  ii as a5,
  Mt as a6,
  yp as a7,
  ym as a8,
  Is as a9,
  Sd as aA,
  Jh as aB,
  mi as aC,
  Of as aD,
  If as aE,
  ld as aF,
  Cm as aG,
  Xm as aH,
  Km as aI,
  sg as aJ,
  eg as aK,
  ig as aL,
  vl as aa,
  Ji as ab,
  la as ac,
  Vh as ad,
  Re as ae,
  ha as af,
  Vm as ag,
  qm as ah,
  ng as ai,
  Qm as aj,
  sp as ak,
  rg as al,
  Te as am,
  yh as an,
  Ie as ao,
  pr as ap,
  rl as aq,
  We as ar,
  rx as as,
  $p as at,
  al as au,
  Nr as av,
  ll as aw,
  Bp as ax,
  _h as ay,
  _r as az,
  Gn as b,
  Sr as c,
  Gv as d,
  sr as e,
  ca as f,
  It as g,
  Tt as h,
  Ho as i,
  Zn as j,
  un as k,
  Do as l,
  Oi as m,
  Yt as n,
  Ge as o,
  kd as p,
  Md as q,
  ni as r,
  Ne as s,
  Wv as t,
  Xy as u,
  ee as v,
  By as w,
  ie as x,
  Jt as y,
  At as z
};
