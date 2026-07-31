const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/browserAll-aAgS91v-.js","assets/webworkerAll-BfE4hGQY.js","assets/Filter-cqUkGLu6.js","assets/WebGPURenderer-Gckc7DmK.js","assets/BufferResource-CvayQKDI.js","assets/RenderTargetSystem-WwbLk5DZ.js","assets/WebGLRenderer-DOVN7vGF.js","assets/CanvasRenderer-D8WLCQmN.js"])))=>i.map(i=>d[i]);
let Fh, xa, rr, Ht, Om, hn, kf, vi, ns, Ms, ue, ie, Zt, ni, kh, Ct, ct, pt, Yt, br, S, ye, Hy, Bg, wr, Ag, vn, vr, dr, Bt, rd, Bo, Ut, Jn, hi, Rt, Rp, Rm, Bs, Ld, od, wi, Jf, Yf, yd, Fm, cg, dg, xg, gg, bg, Pl, tr, ma, nd, Oe, ya, lg, hg, yg, fg, xp, _g, Me, Th, $e, gr, pl, ze, _x, qp, ml, zr, gl, Kp, Mh, Cr, Dn, Ar, f1, or, ga, Ot, At, Ko, ts, fn, qo, Wi, Jt, De, Nd, Fd, li, We, p1, cx, se, Ky, oe, Qt, Pt;
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
  const ip = "modulepreload", rp = function(n) {
    return "/spaghettio/pr-536/" + n;
  }, Za = {}, Ln = function(t, e, s) {
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
        if (c = rp(c), c in Za) return;
        Za[c] = true;
        const h = c.endsWith(".css"), d = h ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${c}"]${d}`)) return;
        const u = document.createElement("link");
        if (u.rel = h ? "stylesheet" : ip, h || (u.as = "script"), u.crossOrigin = "", u.href = c, l && u.setAttribute("nonce", l), document.head.appendChild(u), h) return new Promise((p, f) => {
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
  let Ao, Ei, op, ap;
  Ao = (n) => {
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
  Ei = (n, t) => Ao(n).priority ?? t;
  Ut = {
    _addHandlers: {},
    _removeHandlers: {},
    _queue: {},
    remove(...n) {
      return n.map(Ao).forEach((t) => {
        t.type.forEach((e) => {
          var _a2, _b2;
          return (_b2 = (_a2 = this._removeHandlers)[e]) == null ? void 0 : _b2.call(_a2, t);
        });
      }), this;
    },
    add(...n) {
      return n.map(Ao).forEach((t) => {
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
        }), t.sort((r, o) => Ei(o.value, e) - Ei(r.value, e)));
      }, (s) => {
        const i = t.findIndex((r) => r.name === s.name);
        i !== -1 && t.splice(i, 1);
      });
    },
    handleByList(n, t, e = -1) {
      return this.handle(n, (s) => {
        t.includes(s.ref) || (t.push(s.ref), t.sort((i, r) => Ei(r, e) - Ei(i, e)));
      }, (s) => {
        const i = t.indexOf(s.ref);
        i !== -1 && t.splice(i, 1);
      });
    },
    mixin(n, ...t) {
      for (const e of t) Object.defineProperties(n.prototype, Object.getOwnPropertyDescriptors(e));
    }
  };
  op = {
    extension: {
      type: ct.Environment,
      name: "browser",
      priority: -1
    },
    test: () => true,
    load: async () => {
      await Ln(() => import("./browserAll-aAgS91v-.js"), __vite__mapDeps([0,1,2]));
    }
  };
  ap = {
    extension: {
      type: ct.Environment,
      name: "webworker",
      priority: 0
    },
    test: () => typeof self < "u" && self.WorkerGlobalScope !== void 0,
    load: async () => {
      await Ln(() => import("./webworkerAll-BfE4hGQY.js"), __vite__mapDeps([1,2]));
    }
  };
  class ge {
    constructor(t, e, s) {
      this._x = e || 0, this._y = s || 0, this._observer = t;
    }
    clone(t) {
      return new ge(t ?? this._observer, this._x, this._y);
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
  function fh(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var Rr = {
    exports: {}
  }, Qa;
  function lp() {
    return Qa || (Qa = 1, (function(n) {
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
    })(Rr)), Rr.exports;
  }
  var cp = lp();
  let hp, dp, up;
  hn = fh(cp);
  hp = Math.PI * 2;
  dp = 180 / Math.PI;
  up = Math.PI / 180;
  Rt = class {
    constructor(t = 0, e = 0) {
      this.x = 0, this.y = 0, this.x = t, this.y = e;
    }
    clone() {
      return new Rt(this.x, this.y);
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
      return Lr.x = 0, Lr.y = 0, Lr;
    }
  };
  const Lr = new Rt();
  Ct = class {
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
      e = e || new Rt();
      const s = t.x, i = t.y;
      return e.x = this.a * s + this.c * i + this.tx, e.y = this.b * s + this.d * i + this.ty, e;
    }
    applyInverse(t, e) {
      e = e || new Rt();
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
      return c < 1e-5 || Math.abs(hp - c) < 1e-5 ? (t.rotation = l, t.skew.x = t.skew.y = 0) : (t.rotation = 0, t.skew.x = a, t.skew.y = l), t.scale.x = Math.sqrt(e * e + s * s), t.scale.y = Math.sqrt(i * i + r * r), t.position.x = this.tx + (o.x * e + o.y * i), t.position.y = this.ty + (o.x * s + o.y * r), t;
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
      const t = new Ct();
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
      return fp.identity();
    }
    static get shared() {
      return pp.identity();
    }
  };
  const pp = new Ct(), fp = new Ct(), jn = [
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
  ], Yn = [
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
  ], Vn = [
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
  ], Xn = [
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
  ], ko = [], mh = [], Ti = Math.sign;
  function mp() {
    for (let n = 0; n < 16; n++) {
      const t = [];
      ko.push(t);
      for (let e = 0; e < 16; e++) {
        const s = Ti(jn[n] * jn[e] + Vn[n] * Yn[e]), i = Ti(Yn[n] * jn[e] + Xn[n] * Yn[e]), r = Ti(jn[n] * Vn[e] + Vn[n] * Xn[e]), o = Ti(Yn[n] * Vn[e] + Xn[n] * Xn[e]);
        for (let a = 0; a < 16; a++) if (jn[a] === s && Yn[a] === i && Vn[a] === r && Xn[a] === o) {
          t.push(a);
          break;
        }
      }
    }
    for (let n = 0; n < 16; n++) {
      const t = new Ct();
      t.set(jn[n], Yn[n], Vn[n], Xn[n], 0, 0), mh.push(t);
    }
  }
  mp();
  let Ai;
  Pt = {
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
    uX: (n) => jn[n],
    uY: (n) => Yn[n],
    vX: (n) => Vn[n],
    vY: (n) => Xn[n],
    inv: (n) => n & 8 ? n & 15 : -n & 7,
    add: (n, t) => ko[n][t],
    sub: (n, t) => ko[n][Pt.inv(t)],
    rotate180: (n) => n ^ 4,
    isVertical: (n) => (n & 3) === 2,
    byDirection: (n, t) => Math.abs(n) * 2 <= Math.abs(t) ? t >= 0 ? Pt.S : Pt.N : Math.abs(t) * 2 <= Math.abs(n) ? n > 0 ? Pt.E : Pt.W : t > 0 ? n > 0 ? Pt.SE : Pt.SW : n > 0 ? Pt.NE : Pt.NW,
    matrixAppendRotationInv: (n, t, e = 0, s = 0, i = 0, r = 0) => {
      const o = mh[Pt.inv(t)], a = o.a, l = o.b, c = o.c, h = o.d, d = e - Math.min(0, a * i, c * r, a * i + c * r), u = s - Math.min(0, l * i, h * r, l * i + h * r), p = n.a, f = n.b, g = n.c, m = n.d;
      n.a = a * p + l * g, n.b = a * f + l * m, n.c = c * p + h * g, n.d = c * f + h * m, n.tx = d * p + u * g + n.tx, n.ty = d * f + u * m + n.ty;
    },
    transformRectCoords: (n, t, e, s) => {
      const { x: i, y: r, width: o, height: a } = n, { x: l, y: c, width: h, height: d } = t;
      return e === Pt.E ? (s.set(i + l, r + c, o, a), s) : e === Pt.S ? s.set(h - r - a + l, i + c, a, o) : e === Pt.W ? s.set(h - i - o + l, d - r - a + c, o, a) : e === Pt.N ? s.set(r + l, d - i - o + c, a, o) : s.set(i + l, r + c, o, a);
    }
  };
  Ai = [
    new Rt(),
    new Rt(),
    new Rt(),
    new Rt()
  ];
  Yt = class {
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
      return new Yt(0, 0, 0, 0);
    }
    clone() {
      return new Yt(this.x, this.y, this.width, this.height);
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
      const a = Ai[0].set(t.left, t.top), l = Ai[1].set(t.left, t.bottom), c = Ai[2].set(t.right, t.top), h = Ai[3].set(t.right, t.bottom);
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
      return t || (t = new Yt()), t.copyFrom(this), t;
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
  const $r = {
    default: -1
  };
  ie = function(n = "default") {
    return $r[n] === void 0 && ($r[n] = -1), ++$r[n];
  };
  let tl, gp, bs;
  tl = /* @__PURE__ */ new Set();
  se = "8.0.0";
  gp = "8.3.4";
  bs = {
    quiet: false,
    noColor: false
  };
  Ot = ((n, t, e = 3) => {
    if (bs.quiet || tl.has(t)) return;
    let s = new Error().stack;
    const i = `${t}
Deprecated since v${n}`, r = typeof console.groupCollapsed == "function" && !bs.noColor;
    typeof s > "u" ? console.warn("PixiJS Deprecation Warning: ", i) : (s = s.split(`
`).splice(e).join(`
`), r ? (console.groupCollapsed("%cPixiJS Deprecation Warning: %c%s", "color:#614108;background:#fffbe6", "font-weight:normal;color:#614108;background:#fffbe6", i), console.warn(s), console.groupEnd()) : (console.warn("PixiJS Deprecation Warning: ", i), console.warn(s))), tl.add(t);
  });
  Object.defineProperties(Ot, {
    quiet: {
      get: () => bs.quiet,
      set: (n) => {
        bs.quiet = n;
      },
      enumerable: true,
      configurable: false
    },
    noColor: {
      get: () => bs.noColor,
      set: (n) => {
        bs.noColor = n;
      },
      enumerable: true,
      configurable: false
    }
  });
  const gh = () => {
  };
  function As(n) {
    return n += n === 0 ? 1 : 0, --n, n |= n >>> 1, n |= n >>> 2, n |= n >>> 4, n |= n >>> 8, n |= n >>> 16, n + 1;
  }
  function el(n) {
    return !(n & n - 1) && !!n;
  }
  function yh(n) {
    const t = {};
    for (const e in n) n[e] !== void 0 && (t[e] = n[e]);
    return t;
  }
  const nl = /* @__PURE__ */ Object.create(null);
  function yp(n) {
    const t = nl[n];
    return t === void 0 && (nl[n] = ie("resource")), t;
  }
  const xh = class bh extends hn {
    constructor(t = {}) {
      super(), this._resourceType = "textureSampler", this._touched = 0, this._maxAnisotropy = 1, this.destroyed = false, t = {
        ...bh.defaultOptions,
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
      Ot(se, "TextureStyle.wrapMode is now TextureStyle.addressMode"), this.addressMode = t;
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
      return this._sharedResourceId = yp(t), this._resourceId;
    }
    destroy() {
      this.destroyed = true, this.emit("destroy", this), this.emit("change", this), this.removeAllListeners();
    }
  };
  xh.defaultOptions = {
    addressMode: "clamp-to-edge",
    scaleMode: "linear"
  };
  ts = xh;
  const _h = class wh extends hn {
    constructor(t = {}) {
      super(), this.options = t, this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.uid = ie("textureSource"), this._resourceType = "textureSource", this._resourceId = ie("resource"), this.uploadMethodId = "unknown", this._resolution = 1, this.pixelWidth = 1, this.pixelHeight = 1, this.width = 1, this.height = 1, this.sampleCount = 1, this.mipLevelCount = 1, this.autoGenerateMipmaps = false, this.format = "rgba8unorm", this.dimension = "2d", this.viewDimension = "2d", this.arrayLayerCount = 1, this.antialias = false, this._touched = 0, this._batchTick = -1, this._textureBindLocation = -1, t = {
        ...wh.defaultOptions,
        ...t
      }, this.label = t.label ?? "", this.resource = t.resource, this.autoGarbageCollect = t.autoGarbageCollect, this._resolution = t.resolution, t.width ? this.pixelWidth = t.width * this._resolution : this.pixelWidth = this.resource ? this.resourceWidth ?? 1 : 1, t.height ? this.pixelHeight = t.height * this._resolution : this.pixelHeight = this.resource ? this.resourceHeight ?? 1 : 1, this.width = this.pixelWidth / this._resolution, this.height = this.pixelHeight / this._resolution, this.format = t.format, this.dimension = t.dimensions, this.viewDimension = t.viewDimension ?? t.dimensions, this.arrayLayerCount = t.arrayLayerCount, this.mipLevelCount = t.mipLevelCount, this.autoGenerateMipmaps = t.autoGenerateMipmaps, this.sampleCount = t.sampleCount, this.antialias = t.antialias, this.alphaMode = t.alphaMode, this.style = new ts(yh(t)), this.destroyed = false, this._refreshPOT();
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
      this._resourceId = ie("resource"), this.emit("change", this), this.emit("unload", this);
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
      return this.width = i / s, this.height = r / s, this._resolution = s, this.pixelWidth === i && this.pixelHeight === r ? false : (this._refreshPOT(), this.pixelWidth = i, this.pixelHeight = r, this.emit("resize", this), this._resourceId = ie("resource"), this.emit("change", this), true);
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
      this.isPowerOfTwo = el(this.pixelWidth) && el(this.pixelHeight);
    }
    static test(t) {
      throw new Error("Unimplemented");
    }
  };
  _h.defaultOptions = {
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
  Oe = _h;
  class ua extends Oe {
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
  ua.extension = ct.TextureSource;
  const sl = new Ct();
  xp = class {
    constructor(t, e) {
      this.mapCoord = new Ct(), this.uClampFrame = new Float32Array(4), this.uClampOffset = new Float32Array(2), this._textureID = -1, this._updateID = 0, this.clampOffset = 0, typeof e > "u" ? this.clampMargin = t.width < 10 ? 0 : 0.5 : this.clampMargin = e, this.isSimple = false, this.texture = t;
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
      i && (sl.set(s.width / i.width, 0, 0, s.height / i.height, -i.x / i.width, -i.y / i.height), this.mapCoord.append(sl));
      const r = t.source, o = this.uClampFrame, a = this.clampMargin / r._resolution, l = this.clampOffset / r._resolution;
      return o[0] = (t.frame.x + a + l) / r.width, o[1] = (t.frame.y + a + l) / r.height, o[2] = (t.frame.x + t.frame.width - a + l) / r.width, o[3] = (t.frame.y + t.frame.height - a + l) / r.height, this.uClampOffset[0] = this.clampOffset / r.pixelWidth, this.uClampOffset[1] = this.clampOffset / r.pixelHeight, this.isSimple = t.frame.width === r.width && t.frame.height === r.height && t.rotate === 0, true;
    }
  };
  At = class extends hn {
    constructor({ source: t, label: e, frame: s, orig: i, trim: r, defaultAnchor: o, defaultBorders: a, rotate: l, dynamic: c } = {}) {
      if (super(), this.uid = ie("texture"), this.uvs = {
        x0: 0,
        y0: 0,
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
        x3: 0,
        y3: 0
      }, this.frame = new Yt(), this.noFrame = false, this.dynamic = false, this.isTexture = true, this.label = e, this.source = (t == null ? void 0 : t.source) ?? new Oe(), this.noFrame = !s, s) this.frame.copyFrom(s);
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
      return this._textureMatrix || (this._textureMatrix = new xp(this)), this._textureMatrix;
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
        c = Pt.add(c, Pt.NW), t.x0 = u + h * Pt.uX(c), t.y0 = p + d * Pt.uY(c), c = Pt.add(c, 2), t.x1 = u + h * Pt.uX(c), t.y1 = p + d * Pt.uY(c), c = Pt.add(c, 2), t.x2 = u + h * Pt.uX(c), t.y2 = p + d * Pt.uY(c), c = Pt.add(c, 2), t.x3 = u + h * Pt.uX(c), t.y3 = p + d * Pt.uY(c);
      } else t.x0 = r, t.y0 = o, t.x1 = r + a, t.y1 = o, t.x2 = r + a, t.y2 = o + l, t.x3 = r, t.y3 = o + l;
    }
    destroy(t = false) {
      this._source && (this._source.off("resize", this.update, this), t && (this._source.destroy(), this._source = null)), this._textureMatrix = null, this.destroyed = true, this.emit("destroy", this), this.removeAllListeners();
    }
    update() {
      this.noFrame && (this.frame.width = this._source.width, this.frame.height = this._source.height), this.updateUvs(), this.emit("update", this);
    }
    get baseTexture() {
      return Ot(se, "Texture.baseTexture is now Texture.source"), this._source;
    }
  };
  At.EMPTY = new At({
    label: "EMPTY",
    source: new Oe({
      label: "EMPTY"
    })
  });
  At.EMPTY.destroy = gh;
  At.WHITE = new At({
    source: new ua({
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
  At.WHITE.destroy = gh;
  function vh(n, t, e) {
    const { width: s, height: i } = e.orig, r = e.trim;
    if (r) {
      const o = r.width, a = r.height;
      n.minX = r.x - t._x * s, n.maxX = n.minX + o, n.minY = r.y - t._y * i, n.maxY = n.minY + a;
    } else n.minX = -t._x * s, n.maxX = n.minX + s, n.minY = -t._y * i, n.maxY = n.minY + i;
  }
  const il = new Ct();
  $e = class {
    constructor(t = 1 / 0, e = 1 / 0, s = -1 / 0, i = -1 / 0) {
      this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = il, this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
    }
    isEmpty() {
      return this.minX > this.maxX || this.minY > this.maxY;
    }
    get rectangle() {
      this._rectangle || (this._rectangle = new Yt());
      const t = this._rectangle;
      return this.minX > this.maxX || this.minY > this.maxY ? (t.x = 0, t.y = 0, t.width = 0, t.height = 0) : t.copyFromBounds(this), t;
    }
    clear() {
      return this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = il, this;
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
      return new $e(this.minX, this.minY, this.maxX, this.maxY);
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
  var bp = {
    grad: 0.9,
    turn: 360,
    rad: 360 / (2 * Math.PI)
  }, dn = function(n) {
    return typeof n == "string" ? n.length > 0 : typeof n == "number";
  }, de = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = Math.pow(10, t)), Math.round(e * n) / e + 0;
  }, Ne = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = 1), n > e ? e : n > t ? n : t;
  }, Ch = function(n) {
    return (n = isFinite(n) ? n % 360 : 0) > 0 ? n : n + 360;
  }, rl = function(n) {
    return {
      r: Ne(n.r, 0, 255),
      g: Ne(n.g, 0, 255),
      b: Ne(n.b, 0, 255),
      a: Ne(n.a)
    };
  }, Or = function(n) {
    return {
      r: de(n.r),
      g: de(n.g),
      b: de(n.b),
      a: de(n.a, 3)
    };
  }, _p = /^#([0-9a-f]{3,8})$/i, ki = function(n) {
    var t = n.toString(16);
    return t.length < 2 ? "0" + t : t;
  }, Sh = function(n) {
    var t = n.r, e = n.g, s = n.b, i = n.a, r = Math.max(t, e, s), o = r - Math.min(t, e, s), a = o ? r === t ? (e - s) / o : r === e ? 2 + (s - t) / o : 4 + (t - e) / o : 0;
    return {
      h: 60 * (a < 0 ? a + 6 : a),
      s: r ? o / r * 100 : 0,
      v: r / 255 * 100,
      a: i
    };
  }, Eh = function(n) {
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
  }, ol = function(n) {
    return {
      h: Ch(n.h),
      s: Ne(n.s, 0, 100),
      l: Ne(n.l, 0, 100),
      a: Ne(n.a)
    };
  }, al = function(n) {
    return {
      h: de(n.h),
      s: de(n.s),
      l: de(n.l),
      a: de(n.a, 3)
    };
  }, ll = function(n) {
    return Eh((e = (t = n).s, {
      h: t.h,
      s: (e *= ((s = t.l) < 50 ? s : 100 - s) / 100) > 0 ? 2 * e / (s + e) * 100 : 0,
      v: s + e,
      a: t.a
    }));
    var t, e, s;
  }, ei = function(n) {
    return {
      h: (t = Sh(n)).h,
      s: (i = (200 - (e = t.s)) * (s = t.v) / 100) > 0 && i < 200 ? e * s / 100 / (i <= 100 ? i : 200 - i) * 100 : 0,
      l: i / 2,
      a: t.a
    };
    var t, e, s, i;
  }, wp = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s*,\s*([+-]?\d*\.?\d+)%\s*,\s*([+-]?\d*\.?\d+)%\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, vp = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s+([+-]?\d*\.?\d+)%\s+([+-]?\d*\.?\d+)%\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Cp = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Sp = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Mo = {
    string: [
      [
        function(n) {
          var t = _p.exec(n);
          return t ? (n = t[1]).length <= 4 ? {
            r: parseInt(n[0] + n[0], 16),
            g: parseInt(n[1] + n[1], 16),
            b: parseInt(n[2] + n[2], 16),
            a: n.length === 4 ? de(parseInt(n[3] + n[3], 16) / 255, 2) : 1
          } : n.length === 6 || n.length === 8 ? {
            r: parseInt(n.substr(0, 2), 16),
            g: parseInt(n.substr(2, 2), 16),
            b: parseInt(n.substr(4, 2), 16),
            a: n.length === 8 ? de(parseInt(n.substr(6, 2), 16) / 255, 2) : 1
          } : null : null;
        },
        "hex"
      ],
      [
        function(n) {
          var t = Cp.exec(n) || Sp.exec(n);
          return t ? t[2] !== t[4] || t[4] !== t[6] ? null : rl({
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
          var t = wp.exec(n) || vp.exec(n);
          if (!t) return null;
          var e, s, i = ol({
            h: (e = t[1], s = t[2], s === void 0 && (s = "deg"), Number(e) * (bp[s] || 1)),
            s: Number(t[3]),
            l: Number(t[4]),
            a: t[5] === void 0 ? 1 : Number(t[5]) / (t[6] ? 100 : 1)
          });
          return ll(i);
        },
        "hsl"
      ]
    ],
    object: [
      [
        function(n) {
          var t = n.r, e = n.g, s = n.b, i = n.a, r = i === void 0 ? 1 : i;
          return dn(t) && dn(e) && dn(s) ? rl({
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
          if (!dn(t) || !dn(e) || !dn(s)) return null;
          var o = ol({
            h: Number(t),
            s: Number(e),
            l: Number(s),
            a: Number(r)
          });
          return ll(o);
        },
        "hsl"
      ],
      [
        function(n) {
          var t = n.h, e = n.s, s = n.v, i = n.a, r = i === void 0 ? 1 : i;
          if (!dn(t) || !dn(e) || !dn(s)) return null;
          var o = (function(a) {
            return {
              h: Ch(a.h),
              s: Ne(a.s, 0, 100),
              v: Ne(a.v, 0, 100),
              a: Ne(a.a)
            };
          })({
            h: Number(t),
            s: Number(e),
            v: Number(s),
            a: Number(r)
          });
          return Eh(o);
        },
        "hsv"
      ]
    ]
  }, cl = function(n, t) {
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
  }, Ep = function(n) {
    return typeof n == "string" ? cl(n.trim(), Mo.string) : typeof n == "object" && n !== null ? cl(n, Mo.object) : [
      null,
      void 0
    ];
  }, Br = function(n, t) {
    var e = ei(n);
    return {
      h: e.h,
      s: Ne(e.s + 100 * t, 0, 100),
      l: e.l,
      a: e.a
    };
  }, Nr = function(n) {
    return (299 * n.r + 587 * n.g + 114 * n.b) / 1e3 / 255;
  }, hl = function(n, t) {
    var e = ei(n);
    return {
      h: e.h,
      s: e.s,
      l: Ne(e.l + 100 * t, 0, 100),
      a: e.a
    };
  }, Po = (function() {
    function n(t) {
      this.parsed = Ep(t)[0], this.rgba = this.parsed || {
        r: 0,
        g: 0,
        b: 0,
        a: 1
      };
    }
    return n.prototype.isValid = function() {
      return this.parsed !== null;
    }, n.prototype.brightness = function() {
      return de(Nr(this.rgba), 2);
    }, n.prototype.isDark = function() {
      return Nr(this.rgba) < 0.5;
    }, n.prototype.isLight = function() {
      return Nr(this.rgba) >= 0.5;
    }, n.prototype.toHex = function() {
      return t = Or(this.rgba), e = t.r, s = t.g, i = t.b, o = (r = t.a) < 1 ? ki(de(255 * r)) : "", "#" + ki(e) + ki(s) + ki(i) + o;
      var t, e, s, i, r, o;
    }, n.prototype.toRgb = function() {
      return Or(this.rgba);
    }, n.prototype.toRgbString = function() {
      return t = Or(this.rgba), e = t.r, s = t.g, i = t.b, (r = t.a) < 1 ? "rgba(" + e + ", " + s + ", " + i + ", " + r + ")" : "rgb(" + e + ", " + s + ", " + i + ")";
      var t, e, s, i, r;
    }, n.prototype.toHsl = function() {
      return al(ei(this.rgba));
    }, n.prototype.toHslString = function() {
      return t = al(ei(this.rgba)), e = t.h, s = t.s, i = t.l, (r = t.a) < 1 ? "hsla(" + e + ", " + s + "%, " + i + "%, " + r + ")" : "hsl(" + e + ", " + s + "%, " + i + "%)";
      var t, e, s, i, r;
    }, n.prototype.toHsv = function() {
      return t = Sh(this.rgba), {
        h: de(t.h),
        s: de(t.s),
        v: de(t.v),
        a: de(t.a, 3)
      };
      var t;
    }, n.prototype.invert = function() {
      return nn({
        r: 255 - (t = this.rgba).r,
        g: 255 - t.g,
        b: 255 - t.b,
        a: t.a
      });
      var t;
    }, n.prototype.saturate = function(t) {
      return t === void 0 && (t = 0.1), nn(Br(this.rgba, t));
    }, n.prototype.desaturate = function(t) {
      return t === void 0 && (t = 0.1), nn(Br(this.rgba, -t));
    }, n.prototype.grayscale = function() {
      return nn(Br(this.rgba, -1));
    }, n.prototype.lighten = function(t) {
      return t === void 0 && (t = 0.1), nn(hl(this.rgba, t));
    }, n.prototype.darken = function(t) {
      return t === void 0 && (t = 0.1), nn(hl(this.rgba, -t));
    }, n.prototype.rotate = function(t) {
      return t === void 0 && (t = 15), this.hue(this.hue() + t);
    }, n.prototype.alpha = function(t) {
      return typeof t == "number" ? nn({
        r: (e = this.rgba).r,
        g: e.g,
        b: e.b,
        a: t
      }) : de(this.rgba.a, 3);
      var e;
    }, n.prototype.hue = function(t) {
      var e = ei(this.rgba);
      return typeof t == "number" ? nn({
        h: t,
        s: e.s,
        l: e.l,
        a: e.a
      }) : de(e.h);
    }, n.prototype.isEqual = function(t) {
      return this.toHex() === nn(t).toHex();
    }, n;
  })(), nn = function(n) {
    return n instanceof Po ? n : new Po(n);
  }, dl = [], Tp = function(n) {
    n.forEach(function(t) {
      dl.indexOf(t) < 0 && (t(Po, Mo), dl.push(t));
    });
  };
  function Ap(n, t) {
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
  Tp([
    Ap
  ]);
  const ks = class Js {
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
      if (t instanceof Js) this._value = this._cloneSource(t._value), this._int = t._int, this._components.set(t._components);
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
      const [e, s, i, r] = Js._temp.setValue(t)._components;
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
          const a = Js.HEX_PATTERN.exec(t);
          a && (t = `#${a[2]}`);
        }
        const o = nn(t);
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
      return typeof t == "number" || typeof t == "string" || t instanceof Number || t instanceof Js || Array.isArray(t) || t instanceof Uint8Array || t instanceof Uint8ClampedArray || t instanceof Float32Array || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 || t.r !== void 0 && t.g !== void 0 && t.b !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 || t.h !== void 0 && t.s !== void 0 && t.l !== void 0 && t.a !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 || t.h !== void 0 && t.s !== void 0 && t.v !== void 0 && t.a !== void 0;
    }
  };
  ks.shared = new ks();
  ks._temp = new ks();
  ks.HEX_PATTERN = /^(#|0x)?(([a-f0-9]{3}){1,2}([a-f0-9]{2})?)$/i;
  Jt = ks;
  const kp = {
    cullArea: null,
    cullable: false,
    cullableChildren: true
  };
  let Fr = 0;
  const ul = 500;
  Qt = function(...n) {
    Fr !== ul && (Fr++, Fr === ul ? console.warn("PixiJS Warning: too many warnings, no more warnings will be reported to the console by PixiJS.") : console.warn("PixiJS Warning: ", ...n));
  };
  wi = {
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
  class Mp {
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
  class Pp {
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
      return this._poolsByClass.has(t) || this._poolsByClass.set(t, new Mp(t)), this._poolsByClass.get(t);
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
  Me = new Pp();
  wi.register(Me);
  const Ip = {
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
      Ot("v8.6.0", "cacheAsBitmap is deprecated, use cacheAsTexture instead."), this.cacheAsTexture(n);
    }
  };
  Rp = function(n, t, e) {
    const s = n.length;
    let i;
    if (t >= s || e === 0) return;
    e = t + e > s ? s - t : e;
    const r = s - e;
    for (i = t; i < r; ++i) n[i] = n[i + e];
    n.length = r;
  };
  const Lp = {
    allowChildren: true,
    removeChildren(n = 0, t) {
      var _a2;
      const e = t ?? this.children.length, s = e - n, i = [];
      if (s > 0 && s <= e) {
        for (let o = e - 1; o >= n; o--) {
          const a = this.children[o];
          a && (i.push(a), a.parent = null);
        }
        Rp(this.children, n, e);
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
      this.allowChildren || Ot(se, "addChildAt: Only Containers will be allowed to add children in v8.0.0");
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
  }, $p = {
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
  pl = class {
    constructor() {
      this.pipe = "filter", this.priority = 1;
    }
    destroy() {
      for (let t = 0; t < this.filters.length; t++) this.filters[t].destroy();
      this.filters = null, this.filterArea = null;
    }
  };
  class Op {
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
        if (s.test(t)) return Me.get(s.maskClass, t);
      }
      return t;
    }
    returnMaskEffect(t) {
      Me.return(t);
    }
  }
  const Io = new Op();
  Ut.handleByList(ct.MaskEffect, Io._effectClasses);
  const Bp = {
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
      (t == null ? void 0 : t.mask) !== n && (t && (this.removeEffect(t), Io.returnMaskEffect(t), this._maskEffect = null), n != null && (this._maskEffect = Io.getMaskEffect(n), this.addEffect(this._maskEffect)));
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
      const t = this._filterEffect || (this._filterEffect = new pl());
      n = n;
      const e = (n == null ? void 0 : n.length) > 0, s = ((_a2 = t.filters) == null ? void 0 : _a2.length) > 0, i = e !== s;
      n = Array.isArray(n) ? n.slice(0) : n, t.filters = Object.freeze(n), i && (e ? this.addEffect(t) : (this.removeEffect(t), t.filters = n ?? null));
    },
    get filters() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filters;
    },
    set filterArea(n) {
      this._filterEffect || (this._filterEffect = new pl()), this._filterEffect.filterArea = n;
    },
    get filterArea() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filterArea;
    }
  }, Np = {
    label: null,
    get name() {
      return Ot(se, "Container.name property has been removed, use Container.label instead"), this.label;
    },
    set name(n) {
      Ot(se, "Container.name property has been removed, use Container.label instead"), this.label = n;
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
  }, Ce = Me.getPool(Ct), bn = Me.getPool($e), Fp = new Ct(), Wp = {
    getFastGlobalBounds(n, t) {
      t || (t = new $e()), t.clear(), this._getGlobalBoundsRecursive(!!n, t, this.parentRenderLayer), t.isValid || t.set(0, 0, 0, 0);
      const e = this.renderGroup || this.parentRenderGroup;
      return t.applyMatrix(e.worldTransform), t;
    },
    _getGlobalBoundsRecursive(n, t, e) {
      let s = t;
      if (n && this.parentRenderLayer && this.parentRenderLayer !== e || this.localDisplayStatus !== 7 || !this.measurable) return;
      const i = !!this.effects.length;
      if ((this.renderGroup || i) && (s = bn.get().clear()), this.boundsArea) t.addRect(this.boundsArea, this.worldTransform);
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
        r && s.applyMatrix(o.worldTransform.copyTo(Fp).invert()), t.addBounds(s), bn.return(s);
      } else this.renderGroup && (t.addBounds(s, this.relativeGroupTransform), bn.return(s));
    }
  };
  Th = function(n, t, e) {
    e.clear();
    let s, i;
    return n.parent ? t ? s = n.parent.worldTransform : (i = Ce.get().identity(), s = pa(n, i)) : s = Ct.IDENTITY, Ah(n, e, s, t), i && Ce.return(i), e.isValid || e.set(0, 0, 0, 0), e;
  };
  function Ah(n, t, e, s) {
    var _a2, _b2;
    if (!n.visible || !n.measurable) return;
    let i;
    s ? i = n.worldTransform : (n.updateLocalTransform(), i = Ce.get(), i.appendFrom(n.localTransform, e));
    const r = t, o = !!n.effects.length;
    if (o && (t = bn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, i);
    else {
      const a = n.bounds;
      a && !a.isEmpty() && (t.matrix = i, t.addBounds(a));
      for (let l = 0; l < n.children.length; l++) Ah(n.children[l], t, i, s);
    }
    if (o) {
      for (let a = 0; a < n.effects.length; a++) (_b2 = (_a2 = n.effects[a]).addBounds) == null ? void 0 : _b2.call(_a2, t);
      r.addBounds(t, Ct.IDENTITY), bn.return(t);
    }
    s || Ce.return(i);
  }
  function pa(n, t) {
    const e = n.parent;
    return e && (pa(e, t), e.updateLocalTransform(), t.append(e.localTransform)), t;
  }
  kh = function(n, t) {
    if (n === 16777215 || !t) return t;
    if (t === 16777215 || !n) return n;
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = t >> 16 & 255, o = t >> 8 & 255, a = t & 255, l = e * r / 255 | 0, c = s * o / 255 | 0, h = i * a / 255 | 0;
    return (l << 16) + (c << 8) + h;
  };
  const fl = 16777215;
  ml = function(n, t) {
    return n === fl ? t : t === fl ? n : kh(n, t);
  };
  ni = function(n) {
    return ((n & 255) << 16) + (n & 65280) + (n >> 16 & 255);
  };
  const Gp = {
    getGlobalAlpha(n) {
      if (n) return this.renderGroup ? this.renderGroup.worldAlpha : this.parentRenderGroup ? this.parentRenderGroup.worldAlpha * this.alpha : this.alpha;
      let t = this.alpha, e = this.parent;
      for (; e; ) t *= e.alpha, e = e.parent;
      return t;
    },
    getGlobalTransform(n = new Ct(), t) {
      if (t) return n.copyFrom(this.worldTransform);
      this.updateLocalTransform();
      const e = pa(this, Ce.get().identity());
      return n.appendFrom(this.localTransform, e), Ce.return(e), n;
    },
    getGlobalTint(n) {
      if (n) return this.renderGroup ? ni(this.renderGroup.worldColor) : this.parentRenderGroup ? ni(ml(this.localColor, this.parentRenderGroup.worldColor)) : this.tint;
      let t = this.localColor, e = this.parent;
      for (; e; ) t = ml(t, e.localColor), e = e.parent;
      return ni(t);
    }
  };
  Mh = function(n, t, e) {
    return t.clear(), e || (e = Ct.IDENTITY), Ph(n, t, e, n, true), t.isValid || t.set(0, 0, 0, 0), t;
  };
  function Ph(n, t, e, s, i) {
    var _a2, _b2;
    let r;
    if (i) r = Ce.get(), r = e.copyTo(r);
    else {
      if (!n.visible || !n.measurable) return;
      n.updateLocalTransform();
      const l = n.localTransform;
      r = Ce.get(), r.appendFrom(l, e);
    }
    const o = t, a = !!n.effects.length;
    if (a && (t = bn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, r);
    else {
      n.renderPipeId && (t.matrix = r, t.addBounds(n.bounds));
      const l = n.children;
      for (let c = 0; c < l.length; c++) Ph(l[c], t, r, s, false);
    }
    if (a) {
      for (let l = 0; l < n.effects.length; l++) (_b2 = (_a2 = n.effects[l]).addLocalBounds) == null ? void 0 : _b2.call(_a2, t, s);
      o.addBounds(t, Ct.IDENTITY), bn.return(t);
    }
    Ce.return(r);
  }
  function Ih(n, t) {
    const e = n.children;
    for (let s = 0; s < e.length; s++) {
      const i = e[s], r = i.uid, o = (i._didViewChangeTick & 65535) << 16 | i._didContainerChangeTick & 65535, a = t.index;
      (t.data[a] !== r || t.data[a + 1] !== o) && (t.data[t.index] = r, t.data[t.index + 1] = o, t.didChange = true), t.index = a + 2, i.children.length && Ih(i, t);
    }
    return t.didChange;
  }
  const zp = new Ct(), Dp = {
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
        localBounds: new $e()
      });
      const n = this._localBoundsCacheData;
      return n.index = 1, n.didChange = false, n.data[0] !== this._didViewChangeTick && (n.didChange = true, n.data[0] = this._didViewChangeTick), Ih(this, n), n.didChange && Mh(this, n.localBounds, zp), n.localBounds;
    },
    getBounds(n, t) {
      return Th(this, n, t || new $e());
    }
  }, Hp = {
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
  }, Up = {
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
      this.sortDirty && (this.sortDirty = false, this.children.sort(jp));
    }
  };
  function jp(n, t) {
    return n._zIndex - t._zIndex;
  }
  const Yp = {
    getGlobalPosition(n = new Rt(), t = false) {
      return this.parent ? this.parent.toGlobal(this._position, n, t) : (n.x = this._position.x, n.y = this._position.y), n;
    },
    toGlobal(n, t, e = false) {
      const s = this.getGlobalTransform(Ce.get(), e);
      return t = s.apply(n, t), Ce.return(s), t;
    },
    toLocal(n, t, e, s) {
      t && (n = t.toGlobal(n, e, s));
      const i = this.getGlobalTransform(Ce.get(), s);
      return e = i.applyInverse(n, e), Ce.return(i), e;
    }
  };
  class fa {
    constructor() {
      this.uid = ie("instructionSet"), this.instructions = [], this.instructionSize = 0, this.renderables = [], this.gcTick = 0;
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
  let Vp = 0;
  class Xp {
    constructor(t) {
      this._poolKeyHash = /* @__PURE__ */ Object.create(null), this._texturePool = {}, this.textureOptions = t || {}, this.enableFullScreen = false, this.textureStyle = new ts(this.textureOptions);
    }
    createTexture(t, e, s, i) {
      const r = new Oe({
        ...this.textureOptions,
        width: t,
        height: e,
        resolution: 1,
        antialias: s,
        autoGarbageCollect: false,
        autoGenerateMipmaps: i
      });
      return new At({
        source: r,
        label: `texturePool_${Vp++}`
      });
    }
    getOptimalTexture(t, e, s = 1, i, r = false) {
      let o = Math.ceil(t * s - 1e-6), a = Math.ceil(e * s - 1e-6);
      o = As(o), a = As(a);
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
  gr = new Xp();
  wi.register(gr);
  qp = class {
    constructor() {
      this.renderPipeId = "renderGroup", this.root = null, this.canBundle = false, this.renderGroupParent = null, this.renderGroupChildren = [], this.worldTransform = new Ct(), this.worldColorAlpha = 4294967295, this.worldColor = 16777215, this.worldAlpha = 1, this.childrenToUpdate = /* @__PURE__ */ Object.create(null), this.updateTick = 0, this.gcTick = 0, this.childrenRenderablesToUpdate = {
        list: [],
        index: 0
      }, this.structureDidChange = true, this.instructionSet = new fa(), this._onRenderContainers = [], this.textureNeedsUpdate = true, this.isCachedAsTexture = false, this._matrixDirty = 7;
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
      this.isCachedAsTexture = false, this.texture && (gr.returnTexture(this.texture, true), this.texture = null);
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
      return (this._matrixDirty & 1) === 0 ? this._inverseWorldTransform : (this._matrixDirty &= -2, this._inverseWorldTransform || (this._inverseWorldTransform = new Ct()), this._inverseWorldTransform.copyFrom(this.worldTransform).invert());
    }
    get textureOffsetInverseTransform() {
      return (this._matrixDirty & 2) === 0 ? this._textureOffsetInverseTransform : (this._matrixDirty &= -3, this._textureOffsetInverseTransform || (this._textureOffsetInverseTransform = new Ct()), this._textureOffsetInverseTransform.copyFrom(this.inverseWorldTransform).translate(-this._textureBounds.x, -this._textureBounds.y));
    }
    get inverseParentTextureTransform() {
      if ((this._matrixDirty & 4) === 0) return this._inverseParentTextureTransform;
      this._matrixDirty &= -5;
      const t = this._parentCacheAsTextureRenderGroup;
      return t ? (this._inverseParentTextureTransform || (this._inverseParentTextureTransform = new Ct()), this._inverseParentTextureTransform.copyFrom(this.worldTransform).prepend(t.inverseWorldTransform).translate(-t._textureBounds.x, -t._textureBounds.y)) : this.worldTransform;
    }
    get cacheToLocalTransform() {
      return this.isCachedAsTexture ? this.textureOffsetInverseTransform : this._parentCacheAsTextureRenderGroup ? this._parentCacheAsTextureRenderGroup.textureOffsetInverseTransform : null;
    }
  };
  function Ro(n, t, e = {}) {
    for (const s in t) !e[s] && t[s] !== void 0 && (n[s] = t[s]);
  }
  let Wr, Mi, Gr, Pi;
  Wr = new ge(null);
  Mi = new ge(null);
  Gr = new ge(null, 1, 1);
  Pi = new ge(null);
  gl = 1;
  Kp = 2;
  zr = 4;
  Ht = class extends hn {
    constructor(t = {}) {
      var _a2, _b2;
      super(), this.uid = ie("renderable"), this._updateFlags = 15, this.renderGroup = null, this.parentRenderGroup = null, this.parentRenderGroupIndex = 0, this.didChange = false, this.didViewUpdate = false, this.relativeRenderGroupDepth = 0, this.children = [], this.parent = null, this.includeInBuild = true, this.measurable = true, this.isSimple = true, this.parentRenderLayer = null, this.updateTick = -1, this.localTransform = new Ct(), this.relativeGroupTransform = new Ct(), this.groupTransform = this.relativeGroupTransform, this.destroyed = false, this._position = new ge(this, 0, 0), this._scale = Gr, this._pivot = Mi, this._origin = Pi, this._skew = Wr, this._cx = 1, this._sx = 0, this._cy = 0, this._sy = 1, this._rotation = 0, this.localColor = 16777215, this.localAlpha = 1, this.groupAlpha = 1, this.groupColor = 16777215, this.groupColorAlpha = 4294967295, this.localBlendMode = "inherit", this.groupBlendMode = "normal", this.localDisplayStatus = 7, this.globalDisplayStatus = 7, this._didContainerChangeTick = 0, this._didViewChangeTick = 0, this._didLocalTransformChangeId = -1, this.effects = [], Ro(this, t, {
        children: true,
        parent: true,
        effects: true
      }), (_a2 = t.children) == null ? void 0 : _a2.forEach((e) => this.addChild(e)), (_b2 = t.parent) == null ? void 0 : _b2.addChild(this);
    }
    static mixin(t) {
      Ot("8.8.0", "Container.mixin is deprecated, please use extensions.mixin instead."), Ut.mixin(Ht, t);
    }
    set _didChangeId(t) {
      this._didViewChangeTick = t >> 12 & 4095, this._didContainerChangeTick = t & 4095;
    }
    get _didChangeId() {
      return this._didContainerChangeTick & 4095 | (this._didViewChangeTick & 4095) << 12;
    }
    addChild(...t) {
      if (this.allowChildren || Ot(se, "addChild: Only Containers will be allowed to add children in v8.0.0"), t.length > 1) {
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
      t == null ? void 0 : t.removeChild(this), this.renderGroup = Me.get(qp, this), this.groupTransform = Ct.IDENTITY, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
    }
    disableRenderGroup() {
      if (!this.renderGroup) return;
      const t = this.parentRenderGroup;
      t == null ? void 0 : t.removeChild(this), Me.return(this.renderGroup), this.renderGroup = null, this.groupTransform = this.relativeGroupTransform, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
    }
    _updateIsSimple() {
      this.isSimple = !this.renderGroup && this.effects.length === 0;
    }
    get worldTransform() {
      return this._worldTransform || (this._worldTransform = new Ct()), this.renderGroup ? this._worldTransform.copyFrom(this.renderGroup.worldTransform) : this.parentRenderGroup && this._worldTransform.appendFrom(this.relativeGroupTransform, this.parentRenderGroup.worldTransform), this._worldTransform;
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
      return this.rotation * dp;
    }
    set angle(t) {
      this.rotation = t * up;
    }
    get pivot() {
      return this._pivot === Mi && (this._pivot = new ge(this, 0, 0)), this._pivot;
    }
    set pivot(t) {
      this._pivot === Mi && (this._pivot = new ge(this, 0, 0), this._origin !== Pi && Qt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._pivot.set(t) : this._pivot.copyFrom(t);
    }
    get skew() {
      return this._skew === Wr && (this._skew = new ge(this, 0, 0)), this._skew;
    }
    set skew(t) {
      this._skew === Wr && (this._skew = new ge(this, 0, 0)), this._skew.copyFrom(t);
    }
    get scale() {
      return this._scale === Gr && (this._scale = new ge(this, 1, 1)), this._scale;
    }
    set scale(t) {
      this._scale === Gr && (this._scale = new ge(this, 0, 0)), typeof t == "string" && (t = parseFloat(t)), typeof t == "number" ? this._scale.set(t) : this._scale.copyFrom(t);
    }
    get origin() {
      return this._origin === Pi && (this._origin = new ge(this, 0, 0)), this._origin;
    }
    set origin(t) {
      this._origin === Pi && (this._origin = new ge(this, 0, 0), this._pivot !== Mi && Qt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._origin.set(t) : this._origin.copyFrom(t);
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
      t !== this.localAlpha && (this.localAlpha = t, this._updateFlags |= gl, this._onUpdate());
    }
    get alpha() {
      return this.localAlpha;
    }
    set tint(t) {
      const s = Jt.shared.setValue(t ?? 16777215).toBgrNumber();
      s !== this.localColor && (this.localColor = s, this._updateFlags |= gl, this._onUpdate());
    }
    get tint() {
      return ni(this.localColor);
    }
    set blendMode(t) {
      this.localBlendMode !== t && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= Kp, this.localBlendMode = t, this._onUpdate());
    }
    get blendMode() {
      return this.localBlendMode;
    }
    get visible() {
      return !!(this.localDisplayStatus & 2);
    }
    set visible(t) {
      const e = t ? 2 : 0;
      (this.localDisplayStatus & 2) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= zr, this.localDisplayStatus ^= 2, this._onUpdate(), this.emit("visibleChanged", t));
    }
    get culled() {
      return !(this.localDisplayStatus & 4);
    }
    set culled(t) {
      const e = t ? 0 : 4;
      (this.localDisplayStatus & 4) !== e && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= zr, this.localDisplayStatus ^= 4, this._onUpdate());
    }
    get renderable() {
      return !!(this.localDisplayStatus & 1);
    }
    set renderable(t) {
      const e = t ? 1 : 0;
      (this.localDisplayStatus & 1) !== e && (this._updateFlags |= zr, this.localDisplayStatus ^= 1, this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._onUpdate());
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
  Ut.mixin(Ht, Lp, Wp, Yp, Hp, Dp, Bp, Np, Up, kp, Ip, Gp, $p);
  class yr extends Ht {
    constructor(t) {
      super(t), this.canBundle = true, this.allowChildren = false, this._roundPixels = 0, this._lastUsed = -1, this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this._bounds = new $e(0, 1, 0, 0), this._boundsDirty = true, this.autoGarbageCollect = t.autoGarbageCollect ?? true;
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
  ze = class extends yr {
    constructor(t = At.EMPTY) {
      t instanceof At && (t = {
        texture: t
      });
      const { texture: e = At.EMPTY, anchor: s, roundPixels: i, width: r, height: o, ...a } = t;
      super({
        label: "Sprite",
        ...a
      }), this.renderPipeId = "sprite", this.batched = true, this._visualBounds = {
        minX: 0,
        maxX: 1,
        minY: 0,
        maxY: 0
      }, this._anchor = new ge({
        _onUpdate: () => {
          this.onViewUpdate();
        }
      }), s ? this.anchor = s : e.defaultAnchor && (this.anchor = e.defaultAnchor), this.texture = e, this.allowChildren = false, this.roundPixels = i ?? false, r !== void 0 && (this.width = r), o !== void 0 && (this.height = o);
    }
    static from(t, e = false) {
      return t instanceof At ? new ze(t) : new ze(At.from(t, e));
    }
    set texture(t) {
      t || (t = At.EMPTY);
      const e = this._texture;
      e !== t && (e && e.dynamic && e.off("update", this.onViewUpdate, this), t.dynamic && t.on("update", this.onViewUpdate, this), this._texture = t, this._width && this._setWidth(this._width, this._texture.orig.width), this._height && this._setHeight(this._height, this._texture.orig.height), this.onViewUpdate());
    }
    get texture() {
      return this._texture;
    }
    get visualBounds() {
      return vh(this._visualBounds, this._anchor, this._texture), this._visualBounds;
    }
    get sourceBounds() {
      return Ot("8.6.1", "Sprite.sourceBounds is deprecated, use visualBounds instead."), this.visualBounds;
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
  const Jp = new $e();
  function Rh(n, t, e) {
    const s = Jp;
    n.measurable = true, Th(n, e, s), t.addBoundsMask(s), n.measurable = false;
  }
  function Lh(n, t, e) {
    const s = bn.get();
    n.measurable = true;
    const i = Ce.get().identity(), r = $h(n, e, i);
    Mh(n, s, r), n.measurable = false, t.addBoundsMask(s), Ce.return(i), bn.return(s);
  }
  function $h(n, t, e) {
    return n ? (n !== t && ($h(n.parent, t, e), n.updateLocalTransform(), e.append(n.localTransform)), e) : (Qt("Mask bounds, renderable is not inside the root container"), e);
  }
  class Oh {
    constructor(t) {
      this.priority = 0, this.inverse = false, this.pipe = "alphaMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t, this.renderMaskToTexture = !(t instanceof ze), this.mask.renderable = this.renderMaskToTexture, this.mask.includeInBuild = !this.renderMaskToTexture, this.mask.measurable = false;
    }
    reset() {
      this.mask !== null && (this.mask.measurable = true, this.mask = null);
    }
    addBounds(t, e) {
      this.inverse || Rh(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      Lh(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof ze;
    }
  }
  Oh.extension = ct.MaskEffect;
  class Bh {
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
  Bh.extension = ct.MaskEffect;
  class Nh {
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
      Rh(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      Lh(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof Ht;
    }
  }
  Nh.extension = ct.MaskEffect;
  const Zp = {
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
  let yl = Zp;
  Bt = {
    get() {
      return yl;
    },
    set(n) {
      yl = n;
    }
  };
  Fh = class extends Oe {
    constructor(t) {
      t.resource || (t.resource = Bt.get().createCanvas()), t.width || (t.width = t.resource.width, t.autoDensity || (t.width /= t.resolution)), t.height || (t.height = t.resource.height, t.autoDensity || (t.height /= t.resolution)), super(t), this.uploadMethodId = "image", this.autoDensity = t.autoDensity, this.resizeCanvas(), this.transparent = !!t.transparent;
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
  Fh.extension = ct.TextureSource;
  Ms = class extends Oe {
    constructor(t) {
      super(t), this.uploadMethodId = "image", this.autoGarbageCollect = true;
    }
    static test(t) {
      return globalThis.HTMLImageElement && t instanceof HTMLImageElement || typeof ImageBitmap < "u" && t instanceof ImageBitmap || globalThis.VideoFrame && t instanceof VideoFrame;
    }
  };
  Ms.extension = ct.TextureSource;
  hi = ((n) => (n[n.INTERACTION = 50] = "INTERACTION", n[n.HIGH = 25] = "HIGH", n[n.NORMAL = 0] = "NORMAL", n[n.LOW = -25] = "LOW", n[n.UTILITY = -50] = "UTILITY", n))(hi || {});
  class Dr {
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
  const Wh = class Pe {
    constructor() {
      this.autoStart = false, this.deltaTime = 1, this.lastTime = -1, this.speed = 1, this.started = false, this._requestId = null, this._maxElapsedMS = 100, this._minElapsedMS = 0, this._protected = false, this._lastFrame = -1, this._head = new Dr(null, null, 1 / 0), this.deltaMS = 1 / Pe.targetFPMS, this.elapsedMS = 1 / Pe.targetFPMS, this._tick = (t) => {
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
    add(t, e, s = hi.NORMAL) {
      return this._addListener(new Dr(t, e, s));
    }
    addOnce(t, e, s = hi.NORMAL) {
      return this._addListener(new Dr(t, e, s, true));
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
        this.deltaMS = e, this.deltaTime = this.deltaMS * Pe.targetFPMS;
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
      const e = Math.min(Math.max(0, t) / 1e3, Pe.targetFPMS);
      this._maxElapsedMS = 1 / e, this._minElapsedMS && t > this.maxFPS && (this.maxFPS = t);
    }
    get maxFPS() {
      return this._minElapsedMS ? Math.round(1e3 / this._minElapsedMS) : 0;
    }
    set maxFPS(t) {
      t === 0 ? this._minElapsedMS = 0 : (t < this.minFPS && (this.minFPS = t), this._minElapsedMS = 1 / (t / 1e3));
    }
    static get shared() {
      if (!Pe._shared) {
        const t = Pe._shared = new Pe();
        t.autoStart = true, t._protected = true;
      }
      return Pe._shared;
    }
    static get system() {
      if (!Pe._system) {
        const t = Pe._system = new Pe();
        t.autoStart = true, t._protected = true;
      }
      return Pe._system;
    }
  };
  Wh.targetFPMS = 0.06;
  let Hr;
  Jn = Wh;
  async function Gh() {
    return Hr ?? (Hr = (async () => {
      var _a2;
      const t = Bt.get().createCanvas(1, 1).getContext("webgl");
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
    })()), Hr;
  }
  const xr = class zh extends Oe {
    constructor(t) {
      super(t), this.isReady = false, this.uploadMethodId = "video", t = {
        ...zh.defaultOptions,
        ...t
      }, this._autoUpdate = true, this._isConnectedToTicker = false, this._updateFPS = t.updateFPS || 0, this._msToNextUpdate = 0, this.autoPlay = t.autoPlay !== false, this.alphaMode = t.alphaMode ?? "premultiply-alpha-on-upload", this._videoFrameRequestCallback = this._videoFrameRequestCallback.bind(this), this._videoFrameRequestCallbackHandle = null, this._load = null, this._resolve = null, this._reject = null, this._onCanPlay = this._onCanPlay.bind(this), this._onCanPlayThrough = this._onCanPlayThrough.bind(this), this._onError = this._onError.bind(this), this._onPlayStart = this._onPlayStart.bind(this), this._onPlayStop = this._onPlayStop.bind(this), this._onSeeked = this._onSeeked.bind(this), t.autoLoad !== false && this.load();
    }
    updateFrame() {
      if (!this.destroyed) {
        if (this._updateFPS) {
          const t = Jn.shared.elapsedMS * this.resource.playbackRate;
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
      return (t.readyState === t.HAVE_ENOUGH_DATA || t.readyState === t.HAVE_FUTURE_DATA) && t.width && t.height && (t.complete = true), t.addEventListener("play", this._onPlayStart), t.addEventListener("pause", this._onPlayStop), t.addEventListener("seeked", this._onSeeked), this._isSourceReady() ? this._mediaReady() : (e.preload || t.addEventListener("canplay", this._onCanPlay), t.addEventListener("canplaythrough", this._onCanPlayThrough), t.addEventListener("error", this._onError, true)), this.alphaMode = await Gh(), this._load = new Promise((s, i) => {
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
      this._autoUpdate && this._isSourcePlaying() ? !this._updateFPS && this.resource.requestVideoFrameCallback ? (this._isConnectedToTicker && (Jn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0), this._videoFrameRequestCallbackHandle === null && (this._videoFrameRequestCallbackHandle = this.resource.requestVideoFrameCallback(this._videoFrameRequestCallback))) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker || (Jn.shared.add(this.updateFrame, this), this._isConnectedToTicker = true, this._msToNextUpdate = 0)) : (this._videoFrameRequestCallbackHandle !== null && (this.resource.cancelVideoFrameCallback(this._videoFrameRequestCallbackHandle), this._videoFrameRequestCallbackHandle = null), this._isConnectedToTicker && (Jn.shared.remove(this.updateFrame, this), this._isConnectedToTicker = false, this._msToNextUpdate = 0));
    }
    static test(t) {
      return globalThis.HTMLVideoElement && t instanceof HTMLVideoElement;
    }
  };
  xr.extension = ct.TextureSource;
  xr.defaultOptions = {
    ...Oe.defaultOptions,
    autoLoad: true,
    autoPlay: true,
    updateFPS: 0,
    crossorigin: true,
    loop: false,
    muted: true,
    playsinline: true,
    preload: false
  };
  xr.MIME_TYPES = {
    ogv: "video/ogg",
    mov: "video/quicktime",
    m4v: "video/mp4"
  };
  let si = xr;
  const Xe = (n, t, e = false) => (Array.isArray(n) || (n = [
    n
  ]), t ? n.map((s) => typeof s == "string" || e ? t(s) : s) : n);
  class Qp {
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
      return e || Qt(`[Assets] Asset id ${t} was not found in the Cache`), e;
    }
    set(t, e) {
      const s = Xe(t);
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
        this._cache.has(l) && this._cache.get(l) !== c && Qt("[Cache] already has key:", l), this._cache.set(l, r.get(l));
      });
    }
    remove(t) {
      if (!this._cacheMap.has(t)) {
        Qt(`[Assets] Asset id ${t} was not found in the Cache`);
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
  let Lo;
  oe = new Qp();
  Lo = [];
  Ut.handleByList(ct.TextureSource, Lo);
  function Dh(n = {}) {
    const t = n && n.resource, e = t ? n.resource : n, s = t ? n : {
      resource: n
    };
    for (let i = 0; i < Lo.length; i++) {
      const r = Lo[i];
      if (r.test(e)) return new r(s);
    }
    throw new Error(`Could not find a source type for resource: ${s.resource}`);
  }
  function tf(n = {}, t = false) {
    const e = n && n.resource, s = e ? n.resource : n, i = e ? n : {
      resource: n
    };
    if (!t && oe.has(s)) return oe.get(s);
    const r = new At({
      source: Dh(i)
    });
    return r.on("destroy", () => {
      oe.has(s) && oe.remove(s);
    }), t || oe.set(s, r), r;
  }
  function ef(n, t = false) {
    return typeof n == "string" ? oe.get(n) : n instanceof Oe ? new At({
      source: n
    }) : tf(n, t);
  }
  At.from = ef;
  Oe.from = Dh;
  Ut.add(Oh, Bh, Nh, si, Ms, Fh, ua);
  var Bn = ((n) => (n[n.Low = 0] = "Low", n[n.Normal = 1] = "Normal", n[n.High = 2] = "High", n))(Bn || {});
  function je(n) {
    if (typeof n != "string") throw new TypeError(`Path must be a string. Received ${JSON.stringify(n)}`);
  }
  function Gs(n) {
    return n.split("?")[0].split("#")[0];
  }
  function nf(n) {
    return n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  function sf(n, t, e) {
    return n.replace(new RegExp(nf(t), "g"), e);
  }
  function rf(n, t) {
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
  const Le = {
    toPosix(n) {
      return sf(n, "\\", "/");
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
      je(n), n = this.toPosix(n);
      const t = /^file:\/\/\//.exec(n);
      if (t) return t[0];
      const e = /^[^/:]+:\/{0,2}/.exec(n);
      return e ? e[0] : "";
    },
    toAbsolute(n, t, e) {
      if (je(n), this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      const s = Gs(this.toPosix(t ?? Bt.get().getBaseUrl())), i = Gs(this.toPosix(e ?? this.rootname(s)));
      return n = this.toPosix(n), n.startsWith("/") ? Le.join(i, n.slice(1)) : this.isAbsolute(n) ? n : this.join(s, n);
    },
    normalize(n) {
      if (je(n), n.length === 0) return ".";
      if (this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      n = this.toPosix(n);
      let t = "";
      const e = n.startsWith("/");
      this.hasProtocol(n) && (t = this.rootname(n), n = n.slice(t.length));
      const s = n.endsWith("/");
      return n = rf(n), n.length > 0 && s && (n += "/"), e ? `/${n}` : t + n;
    },
    isAbsolute(n) {
      return je(n), n = this.toPosix(n), this.hasProtocol(n) ? true : n.startsWith("/");
    },
    join(...n) {
      if (n.length === 0) return ".";
      let t;
      for (let e = 0; e < n.length; ++e) {
        const s = n[e];
        if (je(s), s.length > 0) if (t === void 0) t = s;
        else {
          const i = n[e - 1] ?? "";
          this.joinExtensions.includes(this.extname(i).toLowerCase()) ? t += `/../${s}` : t += `/${s}`;
        }
      }
      return t === void 0 ? "." : this.normalize(t);
    },
    dirname(n) {
      if (je(n), n.length === 0) return ".";
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
      je(n), n = this.toPosix(n);
      let t = "";
      if (n.startsWith("/") ? t = "/" : t = this.getProtocol(n), this.isUrl(n)) {
        const e = n.indexOf("/", t.length);
        e !== -1 ? t = n.slice(0, e) : t = n, t.endsWith("/") || (t += "/");
      }
      return t;
    },
    basename(n, t) {
      je(n), t && je(t), n = Gs(this.toPosix(n));
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
      je(n), n = Gs(this.toPosix(n));
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
      je(n);
      const t = {
        root: "",
        dir: "",
        base: "",
        ext: "",
        name: ""
      };
      if (n.length === 0) return t;
      n = Gs(this.toPosix(n));
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
  function Hh(n, t, e, s, i) {
    const r = t[e];
    for (let o = 0; o < r.length; o++) {
      const a = r[o];
      e < t.length - 1 ? Hh(n.replace(s[e], a), t, e + 1, s, i) : i.push(n.replace(s[e], a));
    }
  }
  function of(n) {
    const t = /\{(.*?)\}/g, e = n.match(t), s = [];
    if (e) {
      const i = [];
      e.forEach((r) => {
        const o = r.substring(1, r.length - 1).split(",");
        i.push(o);
      }), Hh(n, i, 0, e, s);
    } else s.push(n);
    return s;
  }
  const ir = (n) => !Array.isArray(n);
  class Ls {
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
      return Xe(e || s, (r) => typeof r == "string" ? r : Array.isArray(r) ? r.map((o) => (o == null ? void 0 : o.src) ?? o) : (r == null ? void 0 : r.src) ? r.src : r, true);
    }
    removeAlias(t, e) {
      this._assetMap[t] && (e && e !== this._resolverHash[t] || (delete this._resolverHash[t], delete this._assetMap[t]));
    }
    addManifest(t) {
      this._manifest && Qt("[Resolver] Manifest already exists, this will be overwritten"), this._manifest = t, t.bundles.forEach((e) => {
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
        this.hasKey(r) && Qt(`[Resolver] already has key: ${r} overwriting`);
      }, Xe(e).forEach((r) => {
        const { src: o } = r;
        let { data: a, format: l, loadParser: c, parser: h } = r;
        const d = Xe(o).map((g) => typeof g == "string" ? of(g) : Array.isArray(g) ? g : [
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
      const e = ir(t);
      t = Xe(t);
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
      const e = ir(t);
      t = Xe(t);
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
      return (this._basePath || this._rootPath) && (t.src = Le.toAbsolute(t.src, this._basePath, this._rootPath)), t.alias = s ?? t.alias ?? [
        t.src
      ], t.src = this._appendDefaultSearchParams(t.src), t.data = {
        ...i || {},
        ...t.data
      }, t.loadParser = r ?? t.loadParser, t.parser = o ?? t.parser, t.format = a ?? t.format ?? af(t.src), l !== void 0 && (t.progressSize = l), t;
    }
  }
  Ls.RETINA_PREFIX = /@([0-9\.]+)x/;
  function af(n) {
    return n.split(".").pop().split("?").shift().split("#").shift();
  }
  const $o = (n, t) => {
    const e = t.split("?")[1];
    return e && (n += `?${e}`), n;
  }, Uh = class Zs {
    constructor(t, e) {
      this.linkedSheets = [];
      let s = t;
      (t == null ? void 0 : t.source) instanceof Oe && (s = {
        texture: t,
        data: e
      });
      const { texture: i, data: r, cachePrefix: o = "" } = s;
      this.cachePrefix = o, this._texture = i instanceof At ? i : null, this.textureSource = i.source, this.textures = {}, this.animations = {}, this.data = r;
      const a = parseFloat(r.meta.scale);
      a ? (this.resolution = a, i.source.resolution = this.resolution) : this.resolution = i.source._resolution, this._frames = this.data.frames, this._frameKeys = Object.keys(this._frames), this._batchIndex = 0, this._callback = null;
    }
    parse() {
      return new Promise((t) => {
        this._callback = t, this._batchIndex = 0, this._frameKeys.length <= Zs.BATCH_SIZE ? (this._processFrames(0), this._processAnimations(), this._parseComplete()) : this._nextBatch();
      });
    }
    parseSync() {
      return this._processFrames(0, true), this._processAnimations(), this.textures;
    }
    _processFrames(t, e = false) {
      let s = t;
      const i = e ? 1 / 0 : Zs.BATCH_SIZE;
      for (; s - t < i && s < this._frameKeys.length; ) {
        const r = this._frameKeys[s], o = this._frames[r], a = o.frame;
        if (a) {
          let l = null, c = null;
          const h = o.trimmed !== false && o.sourceSize ? o.sourceSize : o.frame, d = new Yt(0, 0, Math.floor(h.w) / this.resolution, Math.floor(h.h) / this.resolution);
          o.rotated ? l = new Yt(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.h) / this.resolution, Math.floor(a.w) / this.resolution) : l = new Yt(Math.floor(a.x) / this.resolution, Math.floor(a.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution), o.trimmed !== false && o.spriteSourceSize && (c = new Yt(Math.floor(o.spriteSourceSize.x) / this.resolution, Math.floor(o.spriteSourceSize.y) / this.resolution, Math.floor(a.w) / this.resolution, Math.floor(a.h) / this.resolution)), this.textures[r] = new At({
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
      this._processFrames(this._batchIndex * Zs.BATCH_SIZE), this._batchIndex++, setTimeout(() => {
        this._batchIndex * Zs.BATCH_SIZE < this._frameKeys.length ? this._nextBatch() : (this._processAnimations(), this._parseComplete());
      }, 0);
    }
    destroy(t = false) {
      var _a2;
      for (const e in this.textures) this.textures[e].destroy();
      this._frames = null, this._frameKeys = null, this.data = null, this.textures = null, t && ((_a2 = this._texture) == null ? void 0 : _a2.destroy(), this.textureSource.destroy()), this._texture = null, this.textureSource = null, this.linkedSheets = [];
    }
  };
  Uh.BATCH_SIZE = 1e3;
  let xl = Uh;
  const lf = [
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
  function jh(n, t, e) {
    const s = {};
    if (n.forEach((i) => {
      s[i] = t;
    }), Object.keys(t.textures).forEach((i) => {
      s[`${t.cachePrefix}${i}`] = t.textures[i];
    }), !e) {
      const i = Le.dirname(n[0]);
      t.linkedSheets.forEach((r, o) => {
        const a = jh([
          `${i}/${t.data.meta.related_multi_packs[o]}`
        ], r, true);
        Object.assign(s, a);
      });
    }
    return s;
  }
  const cf = {
    extension: ct.Asset,
    cache: {
      test: (n) => n instanceof xl,
      getCacheableAssets: (n, t) => jh(n, t, false)
    },
    resolver: {
      extension: {
        type: ct.ResolveParser,
        name: "resolveSpritesheet"
      },
      test: (n) => {
        const e = n.split("?")[0].split("."), s = e.pop(), i = e.pop();
        return s === "json" && lf.includes(i);
      },
      parse: (n) => {
        var _a2;
        const t = n.split(".");
        return {
          resolution: parseFloat(((_a2 = Ls.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
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
        priority: Bn.Normal,
        name: "spritesheetLoader"
      },
      async testParse(n, t) {
        return Le.extname(t.src).toLowerCase() === ".json" && !!n.frames;
      },
      async parse(n, t, e) {
        var _a2, _b2;
        const { texture: s, imageFilename: i, textureOptions: r, cachePrefix: o } = (t == null ? void 0 : t.data) ?? {};
        let a = Le.dirname(t.src);
        a && a.lastIndexOf("/") !== a.length - 1 && (a += "/");
        let l;
        if (s instanceof At) l = s;
        else {
          const d = $o(a + (i ?? n.meta.image), t.src);
          l = (await e.load([
            {
              src: d,
              data: r
            }
          ]))[d];
        }
        const c = new xl({
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
            ((_b2 = t.data) == null ? void 0 : _b2.ignoreMultiPack) || (f = $o(f, t.src), d.push(e.load({
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
  Ut.add(cf);
  const Ur = /* @__PURE__ */ Object.create(null), bl = /* @__PURE__ */ Object.create(null);
  ma = function(n, t) {
    let e = bl[n];
    return e === void 0 && (Ur[t] === void 0 && (Ur[t] = 1), bl[n] = e = Ur[t]++), e;
  };
  let Ii;
  function Yh() {
    return (!Ii || (Ii == null ? void 0 : Ii.isContextLost())) && (Ii = Bt.get().createCanvas().getContext("webgl", {})), Ii;
  }
  let Ri;
  function hf() {
    if (!Ri) {
      Ri = "mediump";
      const n = Yh();
      n && n.getShaderPrecisionFormat && (Ri = n.getShaderPrecisionFormat(n.FRAGMENT_SHADER, n.HIGH_FLOAT).precision ? "highp" : "mediump");
    }
    return Ri;
  }
  function df(n, t, e) {
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
  function uf(n, t, e) {
    const s = e ? t.maxSupportedFragmentPrecision : t.maxSupportedVertexPrecision;
    if (n.substring(0, 9) !== "precision") {
      let i = e ? t.requestedFragmentPrecision : t.requestedVertexPrecision;
      return i === "highp" && s !== "highp" && (i = "mediump"), `precision ${i} float;
${n}`;
    } else if (s !== "highp" && n.substring(0, 15) === "precision highp") return n.replace("precision highp", "precision mediump");
    return n;
  }
  function pf(n, t) {
    return t ? `#version 300 es
${n}` : n;
  }
  const ff = {}, mf = {};
  function gf(n, { name: t = "pixi-program" }, e = true) {
    t = t.replace(/\s+/g, "-"), t += e ? "-fragment" : "-vertex";
    const s = e ? ff : mf;
    return s[t] ? (s[t]++, t += `-${s[t]}`) : s[t] = 1, n.indexOf("#define SHADER_NAME") !== -1 ? n : `${`#define SHADER_NAME ${t}`}
${n}`;
  }
  function yf(n, t) {
    return t ? n.replace("#version 300 es", "") : n;
  }
  const jr = {
    stripVersion: yf,
    ensurePrecision: uf,
    addProgramDefines: df,
    setProgramName: gf,
    insertVersion: pf
  }, zs = /* @__PURE__ */ Object.create(null), Vh = class Oo {
    constructor(t) {
      t = {
        ...Oo.defaultOptions,
        ...t
      };
      const e = t.fragment.indexOf("#version 300 es") !== -1, s = {
        stripVersion: e,
        ensurePrecision: {
          requestedFragmentPrecision: t.preferredFragmentPrecision,
          requestedVertexPrecision: t.preferredVertexPrecision,
          maxSupportedVertexPrecision: "highp",
          maxSupportedFragmentPrecision: hf()
        },
        setProgramName: {
          name: t.name
        },
        addProgramDefines: e,
        insertVersion: e
      };
      let i = t.fragment, r = t.vertex;
      Object.keys(jr).forEach((o) => {
        const a = s[o];
        i = jr[o](i, a, true), r = jr[o](r, a, false);
      }), this.fragment = i, this.vertex = r, this.transformFeedbackVaryings = t.transformFeedbackVaryings, this._key = ma(`${this.vertex}:${this.fragment}`, "gl-program");
    }
    destroy() {
      this.fragment = null, this.vertex = null, this._attributeData = null, this._uniformData = null, this._uniformBlockData = null, this.transformFeedbackVaryings = null, zs[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex}:${t.fragment}`;
      return zs[e] || (zs[e] = new Oo(t), zs[e]._cacheKey = e), zs[e];
    }
  };
  Vh.defaultOptions = {
    preferredVertexPrecision: "highp",
    preferredFragmentPrecision: "mediump"
  };
  ga = Vh;
  const _l = {
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
  rr = function(n) {
    return _l[n] ?? _l.float32;
  };
  const xf = {
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
  }, wl = /@location\((\d+)\)\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_<>]+)(?:,|\s|\)|$)/g;
  function vl(n, t) {
    let e;
    for (; (e = wl.exec(n)) !== null; ) {
      const s = xf[e[3]] ?? "float32";
      t[e[2]] = {
        location: parseInt(e[1], 10),
        format: s,
        stride: rr(s).stride,
        offset: 0,
        instance: false,
        start: 0
      };
    }
    wl.lastIndex = 0;
  }
  function bf(n) {
    return n.replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  }
  function _f({ source: n, entryPoint: t }) {
    const e = {}, s = bf(n), i = s.indexOf(`fn ${t}(`);
    if (i === -1) return e;
    const r = s.indexOf("->", i);
    if (r === -1) return e;
    const o = s.substring(i, r);
    if (vl(o, e), Object.keys(e).length === 0) {
      const a = o.match(/\(\s*\w+\s*:\s*(\w+)/);
      if (a) {
        const l = a[1], c = new RegExp(`struct\\s+${l}\\s*\\{([^}]+)\\}`, "s"), h = s.match(c);
        h && vl(h[1], e);
      }
    }
    return e;
  }
  function Yr(n) {
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
  var qn = ((n) => (n[n.VERTEX = 1] = "VERTEX", n[n.FRAGMENT = 2] = "FRAGMENT", n[n.COMPUTE = 4] = "COMPUTE", n))(qn || {});
  function wf({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = []), s.isUniform ? t[s.group].push({
        binding: s.binding,
        visibility: qn.VERTEX | qn.FRAGMENT,
        buffer: {
          type: "uniform"
        }
      }) : s.type === "sampler" ? t[s.group].push({
        binding: s.binding,
        visibility: qn.FRAGMENT,
        sampler: {
          type: "filtering"
        }
      }) : s.type === "texture_2d" || s.type.startsWith("texture_2d<") ? t[s.group].push({
        binding: s.binding,
        visibility: qn.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d",
          multisampled: false
        }
      }) : s.type === "texture_2d_array" || s.type.startsWith("texture_2d_array<") ? t[s.group].push({
        binding: s.binding,
        visibility: qn.FRAGMENT,
        texture: {
          sampleType: "float",
          viewDimension: "2d-array",
          multisampled: false
        }
      }) : (s.type === "texture_cube" || s.type.startsWith("texture_cube<")) && t[s.group].push({
        binding: s.binding,
        visibility: qn.FRAGMENT,
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
  function vf({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = {}), t[s.group][s.name] = s.binding;
    }
    return t;
  }
  function Cf(n, t) {
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
  const Ds = /* @__PURE__ */ Object.create(null);
  vi = class {
    constructor(t) {
      var _a2, _b2;
      this._layoutKey = 0, this._attributeLocationsKey = 0;
      const { fragment: e, vertex: s, layout: i, gpuLayout: r, name: o } = t;
      if (this.name = o, this.fragment = e, this.vertex = s, e.source === s.source) {
        const a = Yr(e.source);
        this.structsAndGroups = a;
      } else {
        const a = Yr(s.source), l = Yr(e.source);
        this.structsAndGroups = Cf(a, l);
      }
      this.layout = i ?? vf(this.structsAndGroups), this.gpuLayout = r ?? wf(this.structsAndGroups), this.autoAssignGlobalUniforms = ((_a2 = this.layout[0]) == null ? void 0 : _a2.globalUniforms) !== void 0, this.autoAssignLocalUniforms = ((_b2 = this.layout[1]) == null ? void 0 : _b2.localUniforms) !== void 0, this._generateProgramKey();
    }
    _generateProgramKey() {
      const { vertex: t, fragment: e } = this, s = t.source + e.source + t.entryPoint + e.entryPoint;
      this._layoutKey = ma(s, "program");
    }
    get attributeData() {
      return this._attributeData ?? (this._attributeData = _f(this.vertex)), this._attributeData;
    }
    destroy() {
      this.gpuLayout = null, this.layout = null, this.structsAndGroups = null, this.fragment = null, this.vertex = null, Ds[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex.source}:${t.fragment.source}:${t.fragment.entryPoint}:${t.vertex.entryPoint}`;
      return Ds[e] || (Ds[e] = new vi(t), Ds[e]._cacheKey = e), Ds[e];
    }
  };
  const Xh = [
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
  ], Sf = Xh.reduce((n, t) => (n[t] = true, n), {});
  function Ef(n, t) {
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
  const qh = class Kh {
    constructor(t, e) {
      this._touched = 0, this.uid = ie("uniform"), this._resourceType = "uniformGroup", this._resourceId = ie("resource"), this.isUniformGroup = true, this._dirtyId = 0, this.destroyed = false, e = {
        ...Kh.defaultOptions,
        ...e
      }, this.uniformStructures = t;
      const s = {};
      for (const i in t) {
        const r = t[i];
        if (r.name = i, r.size = r.size ?? 1, !Sf[r.type]) {
          const o = r.type.match(/^array<(\w+(?:<\w+>)?),\s*(\d+)>$/);
          if (o) {
            const [, a, l] = o;
            throw new Error(`Uniform type ${r.type} is not supported. Use type: '${a}', size: ${l} instead.`);
          }
          throw new Error(`Uniform type ${r.type} is not supported. Supported uniform types are: ${Xh.join(", ")}`);
        }
        r.value ?? (r.value = Ef(r.type, r.size)), s[i] = r.value;
      }
      this.uniforms = s, this._dirtyId = 1, this.ubo = e.ubo, this.isStatic = e.isStatic, this._signature = ma(Object.keys(s).map((i) => `${i}-${t[i].type}`).join("-"), "uniform-group");
    }
    update() {
      this._dirtyId++;
    }
  };
  qh.defaultOptions = {
    ubo: false,
    isStatic: false
  };
  ya = qh;
  tr = class {
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
  Bo = ((n) => (n[n.WEBGL = 1] = "WEBGL", n[n.WEBGPU = 2] = "WEBGPU", n[n.CANVAS = 4] = "CANVAS", n[n.BOTH = 3] = "BOTH", n))(Bo || {});
  br = class extends hn {
    constructor(t) {
      super(), this.uid = ie("shader"), this._uniformBindMap = /* @__PURE__ */ Object.create(null), this._ownedBindGroups = [], this._destroyed = false;
      let { gpuProgram: e, glProgram: s, groups: i, resources: r, compatibleRenderers: o, groupMap: a } = t;
      this.gpuProgram = e, this.glProgram = s, o === void 0 && (o = 0, e && (o |= Bo.WEBGPU), s && (o |= Bo.WEBGL)), this.compatibleRenderers = o;
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
        for (const h in r) l[h] || (i[99] || (i[99] = new tr(), this._ownedBindGroups.push(i[99])), l[h] = {
          group: 99,
          binding: c,
          name: h
        }, a[99] = a[99] || {}, a[99][c] = h, c++);
        for (const h in r) {
          const d = h;
          let u = r[h];
          !u.source && !u._resourceType && (u = new ya(u));
          const p = l[d];
          p && (i[p.group] || (i[p.group] = new tr(), this._ownedBindGroups.push(i[p.group])), i[p.group].setResource(u, p.binding));
        }
      }
      this.groups = i, this._uniformBindMap = a, this.resources = this._buildResourceAccessor(i, l);
    }
    addResource(t, e, s) {
      var i, r;
      (i = this._uniformBindMap)[e] || (i[e] = {}), (r = this._uniformBindMap[e])[s] || (r[s] = t), this.groups[e] || (this.groups[e] = new tr(), this._ownedBindGroups.push(this.groups[e]));
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
      return e && (r = vi.from(e)), s && (o = ga.from(s)), new br({
        gpuProgram: r,
        glProgram: o,
        ...i
      });
    }
  };
  const Tf = {
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
  }, Vr = 0, Xr = 1, qr = 2, Kr = 3, Jr = 4, Zr = 5, No = class Jh {
    constructor() {
      this.data = 0, this.blendMode = "normal", this.polygonOffset = 0, this.blend = true, this.depthMask = true;
    }
    get blend() {
      return !!(this.data & 1 << Vr);
    }
    set blend(t) {
      !!(this.data & 1 << Vr) !== t && (this.data ^= 1 << Vr);
    }
    get offsets() {
      return !!(this.data & 1 << Xr);
    }
    set offsets(t) {
      !!(this.data & 1 << Xr) !== t && (this.data ^= 1 << Xr);
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
      return !!(this.data & 1 << qr);
    }
    set culling(t) {
      !!(this.data & 1 << qr) !== t && (this.data ^= 1 << qr);
    }
    get depthTest() {
      return !!(this.data & 1 << Kr);
    }
    set depthTest(t) {
      !!(this.data & 1 << Kr) !== t && (this.data ^= 1 << Kr);
    }
    get depthMask() {
      return !!(this.data & 1 << Zr);
    }
    set depthMask(t) {
      !!(this.data & 1 << Zr) !== t && (this.data ^= 1 << Zr);
    }
    get clockwiseFrontFace() {
      return !!(this.data & 1 << Jr);
    }
    set clockwiseFrontFace(t) {
      !!(this.data & 1 << Jr) !== t && (this.data ^= 1 << Jr);
    }
    get blendMode() {
      return this._blendMode;
    }
    set blendMode(t) {
      this.blend = t !== "none", this._blendMode = t, this._blendModeId = Tf[t] || 0;
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
      const t = new Jh();
      return t.depthTest = false, t.blend = true, t;
    }
  };
  No.default2d = No.for2d();
  or = No;
  const Fo = [];
  Ut.handleByNamedList(ct.Environment, Fo);
  async function Af(n) {
    if (!n) for (let t = 0; t < Fo.length; t++) {
      const e = Fo[t];
      if (e.value.test()) {
        await e.value.load();
        return;
      }
    }
  }
  let Hs;
  kf = function() {
    if (typeof Hs == "boolean") return Hs;
    try {
      Hs = new Function("param1", "param2", "param3", "return param1[param2] === param3;")({
        a: "b"
      }, "a", "b") === true;
    } catch {
      Hs = false;
    }
    return Hs;
  };
  function Cl(n, t, e = 2) {
    const s = t && t.length, i = s ? t[0] * e : n.length;
    let r = Zh(n, 0, i, e, true);
    const o = [];
    if (!r || r.next === r.prev) return o;
    let a, l, c;
    if (s && (r = Lf(n, t, r, e)), n.length > 80 * e) {
      a = n[0], l = n[1];
      let h = a, d = l;
      for (let u = e; u < i; u += e) {
        const p = n[u], f = n[u + 1];
        p < a && (a = p), f < l && (l = f), p > h && (h = p), f > d && (d = f);
      }
      c = Math.max(h - a, d - l), c = c !== 0 ? 32767 / c : 0;
    }
    return di(r, o, e, a, l, c, 0), o;
  }
  function Zh(n, t, e, s, i) {
    let r;
    if (i === Uf(n, t, e, s) > 0) for (let o = t; o < e; o += s) r = Sl(o / s | 0, n[o], n[o + 1], r);
    else for (let o = e - s; o >= t; o -= s) r = Sl(o / s | 0, n[o], n[o + 1], r);
    return r && Ps(r, r.next) && (pi(r), r = r.next), r;
  }
  function es(n, t) {
    if (!n) return n;
    t || (t = n);
    let e = n, s;
    do
      if (s = false, !e.steiner && (Ps(e, e.next) || ee(e.prev, e, e.next) === 0)) {
        if (pi(e), e = t = e.prev, e === e.next) break;
        s = true;
      } else e = e.next;
    while (s || e !== t);
    return t;
  }
  function di(n, t, e, s, i, r, o) {
    if (!n) return;
    !o && r && Ff(n, s, i, r);
    let a = n;
    for (; n.prev !== n.next; ) {
      const l = n.prev, c = n.next;
      if (r ? Pf(n, s, i, r) : Mf(n)) {
        t.push(l.i, n.i, c.i), pi(n), n = c.next, a = c.next;
        continue;
      }
      if (n = c, n === a) {
        o ? o === 1 ? (n = If(es(n), t), di(n, t, e, s, i, r, 2)) : o === 2 && Rf(n, t, e, s, i, r) : di(es(n), t, e, s, i, r, 1);
        break;
      }
    }
  }
  function Mf(n) {
    const t = n.prev, e = n, s = n.next;
    if (ee(t, e, s) >= 0) return false;
    const i = t.x, r = e.x, o = s.x, a = t.y, l = e.y, c = s.y, h = Math.min(i, r, o), d = Math.min(a, l, c), u = Math.max(i, r, o), p = Math.max(a, l, c);
    let f = s.next;
    for (; f !== t; ) {
      if (f.x >= h && f.x <= u && f.y >= d && f.y <= p && Qs(i, a, r, l, o, c, f.x, f.y) && ee(f.prev, f, f.next) >= 0) return false;
      f = f.next;
    }
    return true;
  }
  function Pf(n, t, e, s) {
    const i = n.prev, r = n, o = n.next;
    if (ee(i, r, o) >= 0) return false;
    const a = i.x, l = r.x, c = o.x, h = i.y, d = r.y, u = o.y, p = Math.min(a, l, c), f = Math.min(h, d, u), g = Math.max(a, l, c), m = Math.max(h, d, u), y = Wo(p, f, t, e, s), b = Wo(g, m, t, e, s);
    let x = n.prevZ, _ = n.nextZ;
    for (; x && x.z >= y && _ && _.z <= b; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && Qs(a, h, l, d, c, u, x.x, x.y) && ee(x.prev, x, x.next) >= 0 || (x = x.prevZ, _.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && Qs(a, h, l, d, c, u, _.x, _.y) && ee(_.prev, _, _.next) >= 0)) return false;
      _ = _.nextZ;
    }
    for (; x && x.z >= y; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && Qs(a, h, l, d, c, u, x.x, x.y) && ee(x.prev, x, x.next) >= 0) return false;
      x = x.prevZ;
    }
    for (; _ && _.z <= b; ) {
      if (_.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && Qs(a, h, l, d, c, u, _.x, _.y) && ee(_.prev, _, _.next) >= 0) return false;
      _ = _.nextZ;
    }
    return true;
  }
  function If(n, t) {
    let e = n;
    do {
      const s = e.prev, i = e.next.next;
      !Ps(s, i) && td(s, e, e.next, i) && ui(s, i) && ui(i, s) && (t.push(s.i, e.i, i.i), pi(e), pi(e.next), e = n = i), e = e.next;
    } while (e !== n);
    return es(e);
  }
  function Rf(n, t, e, s, i, r) {
    let o = n;
    do {
      let a = o.next.next;
      for (; a !== o.prev; ) {
        if (o.i !== a.i && zf(o, a)) {
          let l = ed(o, a);
          o = es(o, o.next), l = es(l, l.next), di(o, t, e, s, i, r, 0), di(l, t, e, s, i, r, 0);
          return;
        }
        a = a.next;
      }
      o = o.next;
    } while (o !== n);
  }
  function Lf(n, t, e, s) {
    const i = [];
    for (let r = 0, o = t.length; r < o; r++) {
      const a = t[r] * s, l = r < o - 1 ? t[r + 1] * s : n.length, c = Zh(n, a, l, s, false);
      c === c.next && (c.steiner = true), i.push(Gf(c));
    }
    i.sort($f);
    for (let r = 0; r < i.length; r++) e = Of(i[r], e);
    return e;
  }
  function $f(n, t) {
    let e = n.x - t.x;
    if (e === 0 && (e = n.y - t.y, e === 0)) {
      const s = (n.next.y - n.y) / (n.next.x - n.x), i = (t.next.y - t.y) / (t.next.x - t.x);
      e = s - i;
    }
    return e;
  }
  function Of(n, t) {
    const e = Bf(n, t);
    if (!e) return t;
    const s = ed(e, n);
    return es(s, s.next), es(e, e.next);
  }
  function Bf(n, t) {
    let e = t;
    const s = n.x, i = n.y;
    let r = -1 / 0, o;
    if (Ps(n, e)) return e;
    do {
      if (Ps(n, e.next)) return e.next;
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
      if (s >= e.x && e.x >= l && s !== e.x && Qh(i < c ? s : r, i, l, c, i < c ? r : s, i, e.x, e.y)) {
        const d = Math.abs(i - e.y) / (s - e.x);
        ui(e, n) && (d < h || d === h && (e.x > o.x || e.x === o.x && Nf(o, e))) && (o = e, h = d);
      }
      e = e.next;
    } while (e !== a);
    return o;
  }
  function Nf(n, t) {
    return ee(n.prev, n, t.prev) < 0 && ee(t.next, n, n.next) < 0;
  }
  function Ff(n, t, e, s) {
    let i = n;
    do
      i.z === 0 && (i.z = Wo(i.x, i.y, t, e, s)), i.prevZ = i.prev, i.nextZ = i.next, i = i.next;
    while (i !== n);
    i.prevZ.nextZ = null, i.prevZ = null, Wf(i);
  }
  function Wf(n) {
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
  function Wo(n, t, e, s, i) {
    return n = (n - e) * i | 0, t = (t - s) * i | 0, n = (n | n << 8) & 16711935, n = (n | n << 4) & 252645135, n = (n | n << 2) & 858993459, n = (n | n << 1) & 1431655765, t = (t | t << 8) & 16711935, t = (t | t << 4) & 252645135, t = (t | t << 2) & 858993459, t = (t | t << 1) & 1431655765, n | t << 1;
  }
  function Gf(n) {
    let t = n, e = n;
    do
      (t.x < e.x || t.x === e.x && t.y < e.y) && (e = t), t = t.next;
    while (t !== n);
    return e;
  }
  function Qh(n, t, e, s, i, r, o, a) {
    return (i - o) * (t - a) >= (n - o) * (r - a) && (n - o) * (s - a) >= (e - o) * (t - a) && (e - o) * (r - a) >= (i - o) * (s - a);
  }
  function Qs(n, t, e, s, i, r, o, a) {
    return !(n === o && t === a) && Qh(n, t, e, s, i, r, o, a);
  }
  function zf(n, t) {
    return n.next.i !== t.i && n.prev.i !== t.i && !Df(n, t) && (ui(n, t) && ui(t, n) && Hf(n, t) && (ee(n.prev, n, t.prev) || ee(n, t.prev, t)) || Ps(n, t) && ee(n.prev, n, n.next) > 0 && ee(t.prev, t, t.next) > 0);
  }
  function ee(n, t, e) {
    return (t.y - n.y) * (e.x - t.x) - (t.x - n.x) * (e.y - t.y);
  }
  function Ps(n, t) {
    return n.x === t.x && n.y === t.y;
  }
  function td(n, t, e, s) {
    const i = $i(ee(n, t, e)), r = $i(ee(n, t, s)), o = $i(ee(e, s, n)), a = $i(ee(e, s, t));
    return !!(i !== r && o !== a || i === 0 && Li(n, e, t) || r === 0 && Li(n, s, t) || o === 0 && Li(e, n, s) || a === 0 && Li(e, t, s));
  }
  function Li(n, t, e) {
    return t.x <= Math.max(n.x, e.x) && t.x >= Math.min(n.x, e.x) && t.y <= Math.max(n.y, e.y) && t.y >= Math.min(n.y, e.y);
  }
  function $i(n) {
    return n > 0 ? 1 : n < 0 ? -1 : 0;
  }
  function Df(n, t) {
    let e = n;
    do {
      if (e.i !== n.i && e.next.i !== n.i && e.i !== t.i && e.next.i !== t.i && td(e, e.next, n, t)) return true;
      e = e.next;
    } while (e !== n);
    return false;
  }
  function ui(n, t) {
    return ee(n.prev, n, n.next) < 0 ? ee(n, t, n.next) >= 0 && ee(n, n.prev, t) >= 0 : ee(n, t, n.prev) < 0 || ee(n, n.next, t) < 0;
  }
  function Hf(n, t) {
    let e = n, s = false;
    const i = (n.x + t.x) / 2, r = (n.y + t.y) / 2;
    do
      e.y > r != e.next.y > r && e.next.y !== e.y && i < (e.next.x - e.x) * (r - e.y) / (e.next.y - e.y) + e.x && (s = !s), e = e.next;
    while (e !== n);
    return s;
  }
  function ed(n, t) {
    const e = Go(n.i, n.x, n.y), s = Go(t.i, t.x, t.y), i = n.next, r = t.prev;
    return n.next = t, t.prev = n, e.next = i, i.prev = e, s.next = e, e.prev = s, r.next = s, s.prev = r, s;
  }
  function Sl(n, t, e, s) {
    const i = Go(n, t, e);
    return s ? (i.next = s.next, i.prev = s, s.next.prev = i, s.next = i) : (i.prev = i, i.next = i), i;
  }
  function pi(n) {
    n.next.prev = n.prev, n.prev.next = n.next, n.prevZ && (n.prevZ.nextZ = n.nextZ), n.nextZ && (n.nextZ.prevZ = n.prevZ);
  }
  function Go(n, t, e) {
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
  function Uf(n, t, e, s) {
    let i = 0;
    for (let r = t, o = e - s; r < e; r += s) i += (n[o] - n[r]) * (n[r + 1] + n[o + 1]), o = r;
    return i;
  }
  const jf = Cl.default || Cl;
  nd = ((n) => (n[n.NONE = 0] = "NONE", n[n.COLOR = 16384] = "COLOR", n[n.STENCIL = 1024] = "STENCIL", n[n.DEPTH = 256] = "DEPTH", n[n.COLOR_DEPTH = 16640] = "COLOR_DEPTH", n[n.COLOR_STENCIL = 17408] = "COLOR_STENCIL", n[n.DEPTH_STENCIL = 1280] = "DEPTH_STENCIL", n[n.ALL = 17664] = "ALL", n))(nd || {});
  Yf = class {
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
  const Vf = [
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
  ], sd = class id extends hn {
    constructor(t) {
      super(), this.tick = 0, this.uid = ie("renderer"), this.runners = /* @__PURE__ */ Object.create(null), this.renderPipes = /* @__PURE__ */ Object.create(null), this._initOptions = {}, this._systemsHash = /* @__PURE__ */ Object.create(null), this.type = t.type, this.name = t.name, this.config = t;
      const e = [
        ...Vf,
        ...this.config.runners ?? []
      ];
      this._addRunners(...e), this._unsafeEvalCheck();
    }
    async init(t = {}) {
      const e = t.skipExtensionImports === true ? true : t.manageImports === false;
      await Af(e), this._addSystems(this.config.systems), this._addPipes(this.config.renderPipes, this.config.renderPipeAdaptors);
      for (const s in this._systemsHash) t = {
        ...this._systemsHash[s].constructor.defaultOptions,
        ...t
      };
      t = {
        ...id.defaultOptions,
        ...t
      }, this._roundPixels = t.roundPixels ? 1 : 0;
      for (let s = 0; s < this.runners.init.items.length; s++) await this.runners.init.items[s].init(t);
      this._initOptions = t;
    }
    render(t, e) {
      this.tick++;
      let s = t;
      if (s instanceof Ht && (s = {
        container: s
      }, e && (Ot(se, "passing a second argument is deprecated, please use render options instead"), s.target = e.renderTexture)), s.target || (s.target = this.view.renderTarget), s.target === this.view.renderTarget && (this._lastObjectRendered = s.container, s.clearColor ?? (s.clearColor = this.background.colorRgba), s.clear ?? (s.clear = this.background.clearBeforeRender)), s.clearColor) {
        const i = Array.isArray(s.clearColor) && s.clearColor.length === 4;
        s.clearColor = i ? s.clearColor : Jt.shared.setValue(s.clearColor).toArray();
      }
      s.transform || (s.container.updateLocalTransform(), s.transform = s.container.localTransform), s.container.visible && (s.container.enableRenderGroup(), this.runners.prerender.emit(s), this.runners.renderStart.emit(s), this.runners.render.emit(s), this.runners.renderEnd.emit(s), this.runners.postrender.emit(s));
    }
    resize(t, e, s) {
      const i = this.view.resolution;
      this.view.resize(t, e, s), this.emit("resize", this.view.screen.width, this.view.screen.height, this.view.resolution), s !== void 0 && s !== i && this.runners.resolutionChange.emit(s);
    }
    clear(t = {}) {
      const e = this;
      t.target || (t.target = e.renderTarget.renderTarget), t.clearColor || (t.clearColor = this.background.colorRgba), t.clear ?? (t.clear = nd.ALL);
      const { clear: s, clearColor: i, target: r, mipLevel: o, layer: a } = t;
      Jt.shared.setValue(i ?? this.background.colorRgba), e.renderTarget.clear(r, s, Jt.shared.toArray(), o ?? 0, a ?? 0);
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
        this.runners[e] = new Yf(e);
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
      this.runners.destroy.items.reverse(), this.runners.destroy.emit(t), (t === true || typeof t == "object" && t.releaseGlobalResources) && wi.release(), Object.values(this.runners).forEach((e) => {
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
      if (!kf()) throw new Error("Current environment does not allow unsafe-eval, please use pixi.js/unsafe-eval module to enable support.");
    }
    resetState() {
      this.runners.resetState.emit();
    }
  };
  sd.defaultOptions = {
    resolution: 1,
    failIfMajorPerformanceCaveat: false,
    roundPixels: false
  };
  let Oi;
  rd = sd;
  function Xf(n) {
    return Oi !== void 0 || (Oi = (() => {
      var _a2;
      const t = {
        stencil: true,
        failIfMajorPerformanceCaveat: n ?? rd.defaultOptions.failIfMajorPerformanceCaveat
      };
      try {
        if (!Bt.get().getWebGLRenderingContext()) return false;
        let s = Bt.get().createCanvas().getContext("webgl", t);
        const i = !!((_a2 = s == null ? void 0 : s.getContextAttributes()) == null ? void 0 : _a2.stencil);
        if (s) {
          const r = s.getExtension("WEBGL_lose_context");
          r && r.loseContext();
        }
        return s = null, i;
      } catch {
        return false;
      }
    })()), Oi;
  }
  let Bi;
  async function qf(n = {}) {
    return Bi !== void 0 || (Bi = await (async () => {
      const t = Bt.get().getNavigator().gpu;
      if (!t) return false;
      try {
        return await (await t.requestAdapter(n)).requestDevice(), true;
      } catch {
        return false;
      }
    })()), Bi;
  }
  const El = [
    "webgl",
    "webgpu",
    "canvas"
  ];
  async function Kf(n) {
    let t = [];
    n.preference ? (t.push(n.preference), El.forEach((r) => {
      r !== n.preference && t.push(r);
    })) : t = El.slice();
    let e, s = {};
    for (let r = 0; r < t.length; r++) {
      const o = t[r];
      if (o === "webgpu" && await qf()) {
        const { WebGPURenderer: a } = await Ln(async () => {
          const { WebGPURenderer: l } = await import("./WebGPURenderer-Gckc7DmK.js");
          return {
            WebGPURenderer: l
          };
        }, __vite__mapDeps([3,4,5,2]));
        e = a, s = {
          ...n,
          ...n.webgpu
        };
        break;
      } else if (o === "webgl" && Xf(n.failIfMajorPerformanceCaveat ?? rd.defaultOptions.failIfMajorPerformanceCaveat)) {
        const { WebGLRenderer: a } = await Ln(async () => {
          const { WebGLRenderer: l } = await import("./WebGLRenderer-DOVN7vGF.js");
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
        const { CanvasRenderer: a } = await Ln(async () => {
          const { CanvasRenderer: l } = await import("./CanvasRenderer-D8WLCQmN.js");
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
  od = "8.17.1";
  class ad {
    static init() {
      var _a2;
      (_a2 = globalThis.__PIXI_APP_INIT__) == null ? void 0 : _a2.call(globalThis, this, od);
    }
    static destroy() {
    }
  }
  ad.extension = ct.Application;
  Jf = class {
    constructor(t) {
      this._renderer = t;
    }
    init() {
      var _a2;
      (_a2 = globalThis.__PIXI_RENDERER_INIT__) == null ? void 0 : _a2.call(globalThis, this._renderer, od);
    }
    destroy() {
      this._renderer = null;
    }
  };
  Jf.extension = {
    type: [
      ct.WebGLSystem,
      ct.WebGPUSystem
    ],
    name: "initHook",
    priority: -10
  };
  class ld {
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
  ld.extension = ct.Application;
  class cd {
    static init(t) {
      t = Object.assign({
        autoStart: true,
        sharedTicker: false
      }, t), Object.defineProperty(this, "ticker", {
        configurable: true,
        set(e) {
          this._ticker && this._ticker.remove(this.render, this), this._ticker = e, e && e.add(this.render, this, hi.LOW);
        },
        get() {
          return this._ticker;
        }
      }), this.stop = () => {
        this._ticker.stop();
      }, this.start = () => {
        this._ticker.start();
      }, this._ticker = null, this.ticker = t.sharedTicker ? Jn.shared : new Jn(), t.autoStart && this.start();
    }
    static destroy() {
      if (this._ticker) {
        const t = this._ticker;
        this.ticker = null, t.destroy();
      }
    }
  }
  cd.extension = ct.Application;
  Ut.add(ld);
  Ut.add(cd);
  const hd = class zo {
    constructor(...t) {
      this.stage = new Ht(), t[0] !== void 0 && Ot(se, "Application constructor options are deprecated, please use Application.init() instead.");
    }
    async init(t) {
      t = {
        ...t
      }, this.stage || (this.stage = new Ht()), this.renderer = await Kf(t), zo._plugins.forEach((e) => {
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
      return Ot(se, "Application.view is deprecated, please use Application.canvas instead."), this.renderer.canvas;
    }
    get screen() {
      return this.renderer.screen;
    }
    destroy(t = false, e = false) {
      const s = zo._plugins.slice(0);
      s.reverse(), s.forEach((i) => {
        i.destroy.call(this);
      }), this.stage.destroy(e), this.stage = null, this.renderer.destroy(t), this.renderer = null;
    }
  };
  hd._plugins = [];
  xa = hd;
  Ut.handleByList(ct.Application, xa._plugins);
  Ut.add(ad);
  const Qr = {
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
  }, Tl = {
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
  }, Al = {
    test(n) {
      return typeof n == "string" && n.match(/<font(\s|>)/) ? Tl.test(Bt.get().parseXML(n)) : false;
    },
    parse(n) {
      return Tl.parse(Bt.get().parseXML(n));
    }
  }, Zf = [
    ".xml",
    ".fnt"
  ], Qf = {
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
  }, tm = {
    extension: {
      type: ct.LoadParser,
      priority: Bn.Normal
    },
    name: "loadBitmapFont",
    id: "bitmap-font",
    test(n) {
      return Zf.includes(Le.extname(n).toLowerCase());
    },
    async testParse(n) {
      return Qr.test(n) || Al.test(n);
    },
    async parse(n, t, e) {
      const s = Qr.test(n) ? Qr.parse(n) : Al.parse(n), { src: i } = t, { pages: r } = s, o = [], a = s.distanceField ? {
        scaleMode: "linear",
        alphaMode: "premultiply-alpha-on-upload",
        autoGenerateMipmaps: false,
        resolution: 1
      } : {};
      for (let u = 0; u < r.length; ++u) {
        const p = r[u].file;
        let f = Le.join(Le.dirname(i), p);
        f = $o(f, i), o.push({
          src: f,
          data: a
        });
      }
      const [l, { BitmapFont: c }] = await Promise.all([
        e.load(o),
        Ln(() => import("./BitmapFont-BQZ50G-8.js"), [])
      ]), h = o.map((u) => l[u.src]);
      return new c({
        data: s,
        textures: h
      }, i);
    },
    async load(n, t) {
      return await (await Bt.get().fetch(n)).text();
    },
    async unload(n, t, e) {
      await Promise.all(n.pages.map((s) => e.unload(s.texture.source._sourceOrigin))), n.destroy();
    }
  };
  class em {
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
  const nm = {
    extension: {
      type: ct.CacheParser,
      name: "cacheTextureArray"
    },
    test: (n) => Array.isArray(n) && n.every((t) => t instanceof At),
    getCacheableAssets: (n, t) => {
      const e = {};
      return n.forEach((s) => {
        t.forEach((i, r) => {
          e[s + (r === 0 ? "" : r + 1)] = i;
        });
      }), e;
    }
  };
  async function dd(n) {
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
  const sm = {
    extension: {
      type: ct.DetectionParser,
      priority: 1
    },
    test: async () => dd("data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A="),
    add: async (n) => [
      ...n,
      "avif"
    ],
    remove: async (n) => n.filter((t) => t !== "avif")
  }, kl = [
    "png",
    "jpg",
    "jpeg"
  ], im = {
    extension: {
      type: ct.DetectionParser,
      priority: -1
    },
    test: () => Promise.resolve(true),
    add: async (n) => [
      ...n,
      ...kl
    ],
    remove: async (n) => n.filter((t) => !kl.includes(t))
  }, rm = "WorkerGlobalScope" in globalThis && globalThis instanceof globalThis.WorkerGlobalScope;
  function _r(n) {
    return rm ? false : document.createElement("video").canPlayType(n) !== "";
  }
  const om = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => _r("video/mp4"),
    add: async (n) => [
      ...n,
      "mp4",
      "m4v"
    ],
    remove: async (n) => n.filter((t) => t !== "mp4" && t !== "m4v")
  }, am = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => _r("video/ogg"),
    add: async (n) => [
      ...n,
      "ogv"
    ],
    remove: async (n) => n.filter((t) => t !== "ogv")
  }, lm = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => _r("video/webm"),
    add: async (n) => [
      ...n,
      "webm"
    ],
    remove: async (n) => n.filter((t) => t !== "webm")
  }, cm = {
    extension: {
      type: ct.DetectionParser,
      priority: 0
    },
    test: async () => dd("data:image/webp;base64,UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAAAfQ//73v/+BiOh/AAA="),
    add: async (n) => [
      ...n,
      "webp"
    ],
    remove: async (n) => n.filter((t) => t !== "webp")
  }, ud = class er {
    constructor() {
      this.loadOptions = {
        ...er.defaultOptions
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
        if ((e.parser || e.loadParser) && (r = this._parserHash[e.parser || e.loadParser], e.loadParser && Qt(`[Assets] "loadParser" is deprecated, use "parser" instead for ${t}`), r || Qt(`[Assets] specified load parser "${e.parser || e.loadParser}" not found while loading ${t}`)), !r) {
          for (let o = 0; o < this.parsers.length; o++) {
            const a = this.parsers[o];
            if (a.load && ((_a2 = a.test) == null ? void 0 : _a2.call(a, t, e, this))) {
              r = a;
              break;
            }
          }
          if (!r) return Qt(`[Assets] ${t} could not be loaded as we don't know how to parse it, ensure the correct parser has been added`), null;
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
        ...er.defaultOptions,
        ...this.loadOptions,
        onProgress: e
      } : {
        ...er.defaultOptions,
        ...this.loadOptions,
        ...e || {}
      }, { onProgress: i, onError: r, strategy: o, retryCount: a, retryDelay: l } = s;
      let c = 0;
      const h = {}, d = ir(t), u = Xe(t, (g) => ({
        alias: [
          g
        ],
        src: g,
        data: {}
      })), p = u.reduce((g, m) => g + (m.progressSize || 1), 0), f = u.map(async (g) => {
        const m = Le.toAbsolute(g.src);
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
      const s = Xe(t, (i) => ({
        alias: [
          i
        ],
        src: i
      })).map(async (i) => {
        var _a2, _b2;
        const r = Le.toAbsolute(i.src), o = this.promiseCache[r];
        if (o) {
          const a = await o.promise;
          delete this.promiseCache[r], await ((_b2 = (_a2 = o.parser) == null ? void 0 : _a2.unload) == null ? void 0 : _b2.call(_a2, a, i, this));
        }
      });
      await Promise.all(s);
    }
    _validateParsers() {
      this._parsersValidated = true, this._parserHash = this._parsers.filter((t) => t.name || t.id).reduce((t, e) => (!e.name && !e.id ? Qt("[Assets] parser should have an id") : (t[e.name] || t[e.id]) && Qt(`[Assets] parser id conflict "${e.id}"`), t[e.name] = e, e.id && (t[e.id] = e), t), {});
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
  ud.defaultOptions = {
    onProgress: void 0,
    onError: void 0,
    strategy: "throw",
    retryCount: 3,
    retryDelay: 250
  };
  let hm = ud;
  function $s(n, t) {
    if (Array.isArray(t)) {
      for (const e of t) if (n.startsWith(`data:${e}`)) return true;
      return false;
    }
    return n.startsWith(`data:${t}`);
  }
  function Os(n, t) {
    const e = n.split("?")[0], s = Le.extname(e).toLowerCase();
    return Array.isArray(t) ? t.includes(s) : s === t;
  }
  const dm = ".json", um = "application/json", pm = {
    extension: {
      type: ct.LoadParser,
      priority: Bn.Low
    },
    name: "loadJson",
    id: "json",
    test(n) {
      return $s(n, um) || Os(n, dm);
    },
    async load(n) {
      return await (await Bt.get().fetch(n)).json();
    }
  }, fm = ".txt", mm = "text/plain", gm = {
    name: "loadTxt",
    id: "text",
    extension: {
      type: ct.LoadParser,
      priority: Bn.Low,
      name: "loadTxt"
    },
    test(n) {
      return $s(n, mm) || Os(n, fm);
    },
    async load(n) {
      return await (await Bt.get().fetch(n)).text();
    }
  }, ym = [
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
  ], xm = [
    ".ttf",
    ".otf",
    ".woff",
    ".woff2"
  ], bm = [
    "font/ttf",
    "font/otf",
    "font/woff",
    "font/woff2"
  ], _m = /^(--|-?[A-Z_])[0-9A-Z_-]*$/i;
  function wm(n) {
    const t = Le.extname(n), i = Le.basename(n, t).replace(/(-|_)/g, " ").toLowerCase().split(" ").map((a) => a.charAt(0).toUpperCase() + a.slice(1));
    let r = i.length > 0;
    for (const a of i) if (!a.match(_m)) {
      r = false;
      break;
    }
    let o = i.join(" ");
    return r || (o = `"${o.replace(/[\\"]/g, "\\$&")}"`), o;
  }
  const vm = /^[0-9A-Za-z%:/?#\[\]@!\$&'()\*\+,;=\-._~]*$/;
  function Cm(n) {
    return vm.test(n) ? n : encodeURI(n);
  }
  const Sm = {
    extension: {
      type: ct.LoadParser,
      priority: Bn.Low
    },
    name: "loadWebFont",
    id: "web-font",
    test(n) {
      return $s(n, bm) || Os(n, xm);
    },
    async load(n, t) {
      var _a2, _b2, _c2;
      const e = Bt.get().getFontFaceSet();
      if (e) {
        const s = [], i = ((_a2 = t.data) == null ? void 0 : _a2.family) ?? wm(n), r = ((_c2 = (_b2 = t.data) == null ? void 0 : _b2.weights) == null ? void 0 : _c2.filter((a) => ym.includes(a))) ?? [
          "normal"
        ], o = t.data ?? {};
        for (let a = 0; a < r.length; a++) {
          const l = r[a], c = new FontFace(i, `url('${Cm(n)}')`, {
            ...o,
            weight: l
          });
          await c.load(), e.add(c), s.push(c);
        }
        return oe.has(`${i}-and-url`) ? oe.get(`${i}-and-url`).entries.push({
          url: n,
          faces: s
        }) : oe.set(`${i}-and-url`, {
          entries: [
            {
              url: n,
              faces: s
            }
          ]
        }), s.length === 1 ? s[0] : s;
      }
      return Qt("[loadWebFont] FontFace API is not supported. Skipping loading font"), null;
    },
    unload(n) {
      const t = Array.isArray(n) ? n : [
        n
      ], e = t[0].family, s = oe.get(`${e}-and-url`), i = s.entries.find((r) => r.faces.some((o) => t.indexOf(o) !== -1));
      i.faces = i.faces.filter((r) => t.indexOf(r) === -1), i.faces.length === 0 && (s.entries = s.entries.filter((r) => r !== i)), t.forEach((r) => {
        Bt.get().getFontFaceSet().delete(r);
      }), s.entries.length === 0 && oe.remove(`${e}-and-url`);
    }
  };
  var to, Ml;
  function Em() {
    if (Ml) return to;
    Ml = 1, to = e;
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
    return to;
  }
  var Tm = Em();
  const Am = fh(Tm);
  function km(n, t) {
    const e = Am(n), s = [];
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
          Qt(`Unknown SVG path command: ${c}`);
      }
      c !== "Z" && c !== "z" && i === null && (i = {
        startX: r,
        startY: o
      }, s.push(i));
    }
    return t;
  }
  class ba {
    constructor(t = 0, e = 0, s = 0) {
      this.type = "circle", this.x = t, this.y = e, this.radius = s;
    }
    clone() {
      return new ba(this.x, this.y, this.radius);
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
      return t || (t = new Yt()), t.x = this.x - this.radius, t.y = this.y - this.radius, t.width = this.radius * 2, t.height = this.radius * 2, t;
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
  class _a {
    constructor(t = 0, e = 0, s = 0, i = 0) {
      this.type = "ellipse", this.x = t, this.y = e, this.halfWidth = s, this.halfHeight = i;
    }
    clone() {
      return new _a(this.x, this.y, this.halfWidth, this.halfHeight);
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
      return t || (t = new Yt()), t.x = this.x - this.halfWidth, t.y = this.y - this.halfHeight, t.width = this.halfWidth * 2, t.height = this.halfHeight * 2, t;
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
  function Mm(n, t, e, s, i, r) {
    const o = n - e, a = t - s, l = i - e, c = r - s, h = o * l + a * c, d = l * l + c * c;
    let u = -1;
    d !== 0 && (u = h / d);
    let p, f;
    u < 0 ? (p = e, f = s) : u > 1 ? (p = i, f = r) : (p = e + u * l, f = s + u * c);
    const g = n - p, m = t - f;
    return g * g + m * m;
  }
  let Pm, Im;
  class ii {
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
      const e = this.getBounds(Pm), s = t.getBounds(Im);
      if (!e.containsRect(s)) return false;
      const i = t.points;
      for (let r = 0; r < i.length; r += 2) {
        const o = i[r], a = i[r + 1];
        if (!this.contains(o, a)) return false;
      }
      return true;
    }
    clone() {
      const t = this.points.slice(), e = new ii(t);
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
        const d = l[h], u = l[h + 1], p = l[(h + 2) % l.length], f = l[(h + 3) % l.length], g = Mm(t, e, d, u, p, f), m = Math.sign((p - d) * (e - u) - (f - u) * (t - d));
        if (g <= (m < 0 ? a : o)) return true;
      }
      return false;
    }
    getBounds(t) {
      t || (t = new Yt());
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
      return Ot("8.11.0", "Polygon.lastX is deprecated, please use Polygon.lastX instead."), this.points[this.points.length - 2];
    }
    get y() {
      return Ot("8.11.0", "Polygon.y is deprecated, please use Polygon.lastY instead."), this.points[this.points.length - 1];
    }
    get startX() {
      return this.points[0];
    }
    get startY() {
      return this.points[1];
    }
  }
  const Ni = (n, t, e, s, i, r, o) => {
    const a = n - e, l = t - s, c = Math.sqrt(a * a + l * l);
    return c >= i - r && c <= i + o;
  };
  class wa {
    constructor(t = 0, e = 0, s = 0, i = 0, r = 20) {
      this.type = "roundedRectangle", this.x = t, this.y = e, this.width = s, this.height = i, this.radius = r;
    }
    getBounds(t) {
      return t || (t = new Yt()), t.x = this.x, t.y = this.y, t.width = this.width, t.height = this.height, t;
    }
    clone() {
      return new wa(this.x, this.y, this.width, this.height, this.radius);
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
      return (t >= r - h && t <= r + d || t >= m - d && t <= m + h) && e >= p && e <= p + g || (e >= o - h && e <= o + d || e >= y - d && e <= y + h) && t >= u && t <= u + f ? true : t < u && e < p && Ni(t, e, u, p, c, d, h) || t > m - c && e < p && Ni(t, e, m - c, p, c, d, h) || t > m - c && e > y - c && Ni(t, e, m - c, y - c, c, d, h) || t < u && e > y - c && Ni(t, e, u, y - c, c, d, h);
    }
    toString() {
      return `[pixi.js/math:RoundedRectangle x=${this.x} y=${this.y}width=${this.width} height=${this.height} radius=${this.radius}]`;
    }
  }
  const pd = {};
  Rm = function(n, t, e) {
    let s = 2166136261;
    for (let i = 0; i < t; i++) s ^= n[i].uid, s = Math.imul(s, 16777619), s >>>= 0;
    return pd[s] || Lm(n, t, s, e);
  };
  function Lm(n, t, e, s) {
    const i = {};
    let r = 0;
    for (let a = 0; a < s; a++) {
      const l = a < t ? n[a] : At.EMPTY.source;
      i[r++] = l.source, i[r++] = l.style;
    }
    const o = new tr(i);
    return pd[e] = o, o;
  }
  class _s {
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
  Pl = function(n, t, e, s) {
    if (e ?? (e = 0), s ?? (s = Math.min(n.byteLength - e, t.byteLength)), !(e & 7) && !(s & 7)) {
      const i = s / 8;
      new Float64Array(t, 0, i).set(new Float64Array(n, e, i));
    } else if (!(e & 3) && !(s & 3)) {
      const i = s / 4;
      new Float32Array(t, 0, i).set(new Float32Array(n, e, i));
    } else new Uint8Array(t).set(new Uint8Array(n, e, s));
  };
  const $m = {
    normal: "normal-npm",
    add: "add-npm",
    screen: "screen-npm"
  };
  Om = ((n) => (n[n.DISABLED = 0] = "DISABLED", n[n.RENDERING_MASK_ADD = 1] = "RENDERING_MASK_ADD", n[n.MASK_ACTIVE = 2] = "MASK_ACTIVE", n[n.INVERSE_MASK_ACTIVE = 3] = "INVERSE_MASK_ACTIVE", n[n.RENDERING_MASK_REMOVE = 4] = "RENDERING_MASK_REMOVE", n[n.NONE = 5] = "NONE", n))(Om || {});
  function Do(n, t) {
    return t.alphaMode === "no-premultiply-alpha" && $m[n] || n;
  }
  const Bm = [
    "precision mediump float;",
    "void main(void){",
    "float test = 0.1;",
    "%forloop%",
    "gl_FragColor = vec4(0.0);",
    "}"
  ].join(`
`);
  function Nm(n) {
    let t = "";
    for (let e = 0; e < n; ++e) e > 0 && (t += `
else `), e < n - 1 && (t += `if(test == ${e}.0){}`);
    return t;
  }
  Fm = function(n, t) {
    if (n === 0) throw new Error("Invalid value of `0` passed to `checkMaxIfStatementsInShader`");
    const e = t.createShader(t.FRAGMENT_SHADER);
    try {
      for (; ; ) {
        const s = Bm.replace(/%forloop%/gi, Nm(n));
        if (t.shaderSource(e, s), t.compileShader(e), !t.getShaderParameter(e, t.COMPILE_STATUS)) n = n / 2 | 0;
        else break;
      }
    } finally {
      t.deleteShader(e);
    }
    return n;
  };
  let hs = null;
  function Wm() {
    var _a2;
    if (hs) return hs;
    const n = Yh();
    return hs = n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS), hs = Fm(hs, n), (_a2 = n.getExtension("WEBGL_lose_context")) == null ? void 0 : _a2.loseContext(), hs;
  }
  class Gm {
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
  class zm {
    constructor() {
      this.renderPipeId = "batch", this.action = "startBatch", this.start = 0, this.size = 0, this.textures = new Gm(), this.blendMode = "normal", this.topology = "triangle-strip", this.canBundle = true;
    }
    destroy() {
      this.textures = null, this.gpuBindGroup = null, this.bindGroup = null, this.batcher = null, this.elements = null;
    }
  }
  const ri = [];
  let ar = 0;
  wi.register({
    clear: () => {
      if (ri.length > 0) for (const n of ri) n && n.destroy();
      ri.length = 0, ar = 0;
    }
  });
  function Il() {
    return ar > 0 ? ri[--ar] : new zm();
  }
  function Rl(n) {
    n.elements = null, ri[ar++] = n;
  }
  let Us = 0;
  const fd = class md {
    constructor(t) {
      this.uid = ie("batcher"), this.dirty = true, this.batchIndex = 0, this.batches = [], this._elements = [], t = {
        ...md.defaultOptions,
        ...t
      }, t.maxTextures || (Ot("v8.8.0", "maxTextures is a required option for Batcher now, please pass it in the options"), t.maxTextures = Wm());
      const { maxTextures: e, attributesInitialSize: s, indicesInitialSize: i } = t;
      this.attributeBuffer = new _s(s * 4), this.indexBuffer = new Uint16Array(i), this.maxTextures = e;
    }
    begin() {
      this.elementSize = 0, this.elementStart = 0, this.indexSize = 0, this.attributeSize = 0;
      for (let t = 0; t < this.batchIndex; t++) Rl(this.batches[t]);
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
      let s = Il(), i = s.textures;
      i.clear();
      const r = e[this.elementStart];
      let o = Do(r.blendMode, r.texture._source), a = r.topology;
      this.attributeSize * 4 > this.attributeBuffer.size && this._resizeAttributeBuffer(this.attributeSize * 4), this.indexSize > this.indexBuffer.length && this._resizeIndexBuffer(this.indexSize);
      const l = this.attributeBuffer.float32View, c = this.attributeBuffer.uint32View, h = this.indexBuffer;
      let d = this._batchIndexSize, u = this._batchIndexStart, p = "startBatch", f = [];
      const g = this.maxTextures;
      for (let m = this.elementStart; m < this.elementSize; ++m) {
        const y = e[m];
        e[m] = null;
        const x = y.texture._source, _ = Do(y.blendMode, x), v = o !== _ || a !== y.topology;
        if (x._batchTick === Us && !v) {
          y._textureId = x._textureBindLocation, d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize)), y._batch = s, f.push(y);
          continue;
        }
        x._batchTick = Us, (i.count >= g || v) && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), p = "renderBatch", u = d, o = _, a = y.topology, s = Il(), i = s.textures, i.clear(), f = [], ++Us), y._textureId = x._textureBindLocation = i.count, i.ids[x.uid] = i.count, i.textures[i.count++] = x, y._batch = s, f.push(y), d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize));
      }
      i.count > 0 && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), u = d, ++Us), this.elementStart = this.elementSize, this._batchIndexStart = u, this._batchIndexSize = d;
    }
    _finishBatch(t, e, s, i, r, o, a, l, c) {
      t.gpuBindGroup = null, t.bindGroup = null, t.action = l, t.batcher = this, t.textures = i, t.blendMode = r, t.topology = o, t.start = e, t.size = s, t.elements = c, ++Us, this.batches[this.batchIndex++] = t, a.add(t);
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
      const e = Math.max(t, this.attributeBuffer.size * 2), s = new _s(e);
      Pl(this.attributeBuffer.rawBinaryData, s.rawBinaryData), this.attributeBuffer = s;
    }
    _resizeIndexBuffer(t) {
      const e = this.indexBuffer;
      let s = Math.max(t, e.length * 1.5);
      s += s % 2;
      const i = s > 65535 ? new Uint32Array(s) : new Uint16Array(s);
      if (i.BYTES_PER_ELEMENT !== e.BYTES_PER_ELEMENT) for (let r = 0; r < e.length; r++) i[r] = e[r];
      else Pl(e.buffer, i.buffer);
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
        for (let e = 0; e < this.batchIndex; e++) Rl(this.batches[e]);
        this.batches = null, this.geometry.destroy(true), this.geometry = null, t.shader && ((_a2 = this.shader) == null ? void 0 : _a2.destroy(), this.shader = null);
        for (let e = 0; e < this._elements.length; e++) this._elements[e] && (this._elements[e]._batch = null);
        this._elements = null, this.indexBuffer = null, this.attributeBuffer.destroy(), this.attributeBuffer = null;
      }
    }
  };
  fd.defaultOptions = {
    maxTextures: null,
    attributesInitialSize: 4,
    indicesInitialSize: 6
  };
  let Dm = fd;
  ue = ((n) => (n[n.MAP_READ = 1] = "MAP_READ", n[n.MAP_WRITE = 2] = "MAP_WRITE", n[n.COPY_SRC = 4] = "COPY_SRC", n[n.COPY_DST = 8] = "COPY_DST", n[n.INDEX = 16] = "INDEX", n[n.VERTEX = 32] = "VERTEX", n[n.UNIFORM = 64] = "UNIFORM", n[n.STORAGE = 128] = "STORAGE", n[n.INDIRECT = 256] = "INDIRECT", n[n.QUERY_RESOLVE = 512] = "QUERY_RESOLVE", n[n.STATIC = 1024] = "STATIC", n))(ue || {});
  ns = class extends hn {
    constructor(t) {
      let { data: e, size: s } = t;
      const { usage: i, label: r, shrinkToFit: o } = t;
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.autoGarbageCollect = true, this.uid = ie("buffer"), this._resourceType = "buffer", this._resourceId = ie("resource"), this._touched = 0, this._updateID = 1, this._dataInt32 = null, this.shrinkToFit = true, this.destroyed = false, e instanceof Array && (e = new Float32Array(e)), this._data = e, s ?? (s = e == null ? void 0 : e.byteLength);
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
      return !!(this.descriptor.usage & ue.STATIC);
    }
    set static(t) {
      t ? this.descriptor.usage |= ue.STATIC : this.descriptor.usage &= ~ue.STATIC;
    }
    setDataWithSize(t, e, s) {
      if (this._updateID++, this._updateSize = e * t.BYTES_PER_ELEMENT, this._data === t) {
        s && this.emit("update", this);
        return;
      }
      const i = this._data;
      if (this._data = t, this._dataInt32 = null, !i || i.length !== t.length) {
        !this.shrinkToFit && i && t.byteLength < i.byteLength ? s && this.emit("update", this) : (this.descriptor.size = t.byteLength, this._resourceId = ie("resource"), this.emit("change", this));
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
  function gd(n, t) {
    if (!(n instanceof ns)) {
      let e = t ? ue.INDEX : ue.VERTEX;
      n instanceof Array && (t ? (n = new Uint32Array(n), e = ue.INDEX | ue.COPY_DST) : (n = new Float32Array(n), e = ue.VERTEX | ue.COPY_DST)), n = new ns({
        data: n,
        label: t ? "index-mesh-buffer" : "vertex-mesh-buffer",
        usage: e
      });
    }
    return n;
  }
  function Hm(n, t, e) {
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
  function Um(n) {
    return (n instanceof ns || Array.isArray(n) || n.BYTES_PER_ELEMENT) && (n = {
      buffer: n
    }), n.buffer = gd(n.buffer, false), n;
  }
  yd = class extends hn {
    constructor(t = {}) {
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ie("geometry"), this._layoutKey = 0, this.instanceCount = 1, this._bounds = new $e(), this._boundsDirty = true;
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
      const s = Um(e);
      this.buffers.indexOf(s.buffer) === -1 && (this.buffers.push(s.buffer), s.buffer.on("update", this.onBufferUpdate, this), s.buffer.on("change", this.onBufferUpdate, this)), this.attributes[t] = s;
    }
    addIndex(t) {
      this.indexBuffer = gd(t, true), this.buffers.push(this.indexBuffer);
    }
    get bounds() {
      return this._boundsDirty ? (this._boundsDirty = false, Hm(this, "aPosition", this._bounds)) : this._bounds;
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
  const jm = new Float32Array(1), Ym = new Uint32Array(1);
  class Vm extends yd {
    constructor() {
      const e = new ns({
        data: jm,
        label: "attribute-batch-buffer",
        usage: ue.VERTEX | ue.COPY_DST,
        shrinkToFit: false
      }), s = new ns({
        data: Ym,
        label: "index-batch-buffer",
        usage: ue.INDEX | ue.COPY_DST,
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
  function Ll(n, t, e) {
    if (n) for (const s in n) {
      const i = s.toLocaleLowerCase(), r = t[i];
      if (r) {
        let o = n[s];
        s === "header" && (o = o.replace(/@in\s+[^;]+;\s*/g, "").replace(/@out\s+[^;]+;\s*/g, "")), e && r.push(`//----${e}----//`), r.push(o);
      } else Qt(`${s} placement hook does not exist in shader`);
    }
  }
  const Xm = /\{\{(.*?)\}\}/g;
  function $l(n) {
    var _a2;
    const t = {};
    return (((_a2 = n.match(Xm)) == null ? void 0 : _a2.map((s) => s.replace(/[{()}]/g, ""))) ?? []).forEach((s) => {
      t[s] = [];
    }), t;
  }
  function Ol(n, t) {
    let e;
    const s = /@in\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function Bl(n, t, e = false) {
    const s = [];
    Ol(t, s), n.forEach((a) => {
      a.header && Ol(a.header, s);
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
  function Nl(n, t) {
    let e;
    const s = /@out\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function qm(n) {
    const e = /\b(\w+)\s*:/g.exec(n);
    return e ? e[1] : "";
  }
  function Km(n) {
    const t = /@.*?\s+/g;
    return n.replace(t, "");
  }
  function Jm(n, t) {
    const e = [];
    Nl(t, e), n.forEach((l) => {
      l.header && Nl(l.header, e);
    });
    let s = 0;
    const i = e.sort().map((l) => l.indexOf("builtin") > -1 ? l : `@location(${s++}) ${l}`).join(`,
`), r = e.sort().map((l) => `       var ${Km(l)};`).join(`
`), o = `return VSOutput(
            ${e.sort().map((l) => ` ${qm(l)}`).join(`,
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
  function Fl(n, t) {
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
  const Pn = /* @__PURE__ */ Object.create(null), eo = /* @__PURE__ */ new Map();
  let Zm = 0;
  function Qm({ template: n, bits: t }) {
    const e = xd(n, t);
    if (Pn[e]) return Pn[e];
    const { vertex: s, fragment: i } = eg(n, t);
    return Pn[e] = bd(s, i, t), Pn[e];
  }
  function tg({ template: n, bits: t }) {
    const e = xd(n, t);
    return Pn[e] || (Pn[e] = bd(n.vertex, n.fragment, t)), Pn[e];
  }
  function eg(n, t) {
    const e = t.map((o) => o.vertex).filter((o) => !!o), s = t.map((o) => o.fragment).filter((o) => !!o);
    let i = Bl(e, n.vertex, true);
    i = Jm(e, i);
    const r = Bl(s, n.fragment, true);
    return {
      vertex: i,
      fragment: r
    };
  }
  function xd(n, t) {
    return t.map((e) => (eo.has(e) || eo.set(e, Zm++), eo.get(e))).sort((e, s) => e - s).join("-") + n.vertex + n.fragment;
  }
  function bd(n, t, e) {
    const s = $l(n), i = $l(t);
    return e.forEach((r) => {
      Ll(r.vertex, s, r.name), Ll(r.fragment, i, r.name);
    }), {
      vertex: Fl(n, s),
      fragment: Fl(t, i)
    };
  }
  const ng = `
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
`, sg = `
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
`, ig = `
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
`, rg = `

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
`, og = {
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
  }, ag = {
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
  lg = function({ bits: n, name: t }) {
    const e = Qm({
      template: {
        fragment: sg,
        vertex: ng
      },
      bits: [
        og,
        ...n
      ]
    });
    return vi.from({
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
  cg = function({ bits: n, name: t }) {
    return new ga({
      name: t,
      ...tg({
        template: {
          vertex: ig,
          fragment: rg
        },
        bits: [
          ag,
          ...n
        ]
      })
    });
  };
  let no;
  hg = {
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
  dg = {
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
  no = {};
  function ug(n) {
    const t = [];
    if (n === 1) t.push("@group(1) @binding(0) var textureSource1: texture_2d<f32>;"), t.push("@group(1) @binding(1) var textureSampler1: sampler;");
    else {
      let e = 0;
      for (let s = 0; s < n; s++) t.push(`@group(1) @binding(${e++}) var textureSource${s + 1}: texture_2d<f32>;`), t.push(`@group(1) @binding(${e++}) var textureSampler${s + 1}: sampler;`);
    }
    return t.join(`
`);
  }
  function pg(n) {
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
  fg = function(n) {
    return no[n] || (no[n] = {
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

                ${ug(n)}
            `,
        main: `
                var uvDx = dpdx(vUV);
                var uvDy = dpdy(vUV);

                ${pg(n)}
            `
      }
    }), no[n];
  };
  const so = {};
  function mg(n) {
    const t = [];
    for (let e = 0; e < n; e++) e > 0 && t.push("else"), e < n - 1 && t.push(`if(vTextureId < ${e}.5)`), t.push("{"), t.push(`	outColor = texture(uTextures[${e}], vUV);`), t.push("}");
    return t.join(`
`);
  }
  gg = function(n) {
    return so[n] || (so[n] = {
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

                ${mg(n)}
            `
      }
    }), so[n];
  };
  let Wl;
  yg = {
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
  xg = {
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
  Wl = {};
  bg = function(n) {
    let t = Wl[n];
    if (t) return t;
    const e = new Int32Array(n);
    for (let s = 0; s < n; s++) e[s] = s;
    return t = Wl[n] = new ya({
      uTextures: {
        value: e,
        type: "i32",
        size: n
      }
    }, {
      isStatic: true
    }), t;
  };
  class Gl extends br {
    constructor(t) {
      const e = cg({
        name: "batch",
        bits: [
          dg,
          gg(t),
          xg
        ]
      }), s = lg({
        name: "batch",
        bits: [
          hg,
          fg(t),
          yg
        ]
      });
      super({
        glProgram: e,
        gpuProgram: s,
        resources: {
          batchSamplers: bg(t)
        }
      }), this.maxTextures = t;
    }
  }
  let js = null;
  const _d = class wd extends Dm {
    constructor(t) {
      super(t), this.geometry = new Vm(), this.name = wd.extension.name, this.vertexSize = 6, js ?? (js = new Gl(t.maxTextures)), this.shader = js;
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
      this.shader.maxTextures !== t && (js = new Gl(t), this.shader = js);
    }
    destroy() {
      this.shader = null, super.destroy();
    }
  };
  _d.extension = {
    type: [
      ct.Batcher
    ],
    name: "default"
  };
  _g = _d;
  Bs = class {
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
  function wg(n, t, e, s, i, r, o, a = null) {
    let l = 0;
    e *= t, i *= r;
    const c = a.a, h = a.b, d = a.c, u = a.d, p = a.tx, f = a.ty;
    for (; l < o; ) {
      const g = n[e], m = n[e + 1];
      s[i] = c * g + d * m + p, s[i + 1] = h * g + u * m + f, i += r, e += t, l++;
    }
  }
  function vg(n, t, e, s) {
    let i = 0;
    for (t *= e; i < s; ) n[t] = 0, n[t + 1] = 0, t += e, i++;
  }
  function vd(n, t, e, s, i) {
    const r = t.a, o = t.b, a = t.c, l = t.d, c = t.tx, h = t.ty;
    e || (e = 0), s || (s = 2), i || (i = n.length / s - e);
    let d = e * s;
    for (let u = 0; u < i; u++) {
      const p = n[d], f = n[d + 1];
      n[d] = r * p + a * f + c, n[d + 1] = o * p + l * f + h, d += s;
    }
  }
  const Cg = new Ct();
  class va {
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
      return s ? kh(e, s.groupColor) + (this.alpha * s.groupAlpha * 255 << 24) : e + (this.alpha * 255 << 24);
    }
    get transform() {
      var _a2;
      return ((_a2 = this.renderable) == null ? void 0 : _a2.groupTransform) || Cg;
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
  const fi = {
    extension: {
      type: ct.ShapeBuilder,
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
        const v = Math.PI / 2 * (_ / l), w = i + Math.cos(v) * o, C = r + Math.sin(v) * a, T = e + w, R = e - w, E = s + C, A = s - C;
        t[h++] = T, t[h++] = E, t[--d] = E, t[--d] = R, t[u++] = R, t[u++] = A, t[--p] = A, t[--p] = T;
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
  }, Sg = {
    ...fi,
    extension: {
      ...fi.extension,
      name: "ellipse"
    }
  }, Eg = {
    ...fi,
    extension: {
      ...fi.extension,
      name: "roundedRectangle"
    }
  }, Cd = 1e-4, zl = 1e-4;
  function Tg(n) {
    const t = n.length;
    if (t < 6) return 1;
    let e = 0;
    for (let s = 0, i = n[t - 2], r = n[t - 1]; s < t; s += 2) {
      const o = n[s], a = n[s + 1];
      e += (o - i) * (a + r), i = o, r = a;
    }
    return e < 0 ? -1 : 1;
  }
  function Dl(n, t, e, s, i, r, o, a) {
    const l = n - e * i, c = t - s * i, h = n + e * r, d = t + s * r;
    let u, p;
    o ? (u = s, p = -e) : (u = -s, p = e);
    const f = l + u, g = c + p, m = h + u, y = d + p;
    return a.push(f, g), a.push(m, y), 2;
  }
  function Gn(n, t, e, s, i, r, o, a) {
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
  Ag = function(n, t, e, s, i, r) {
    const o = Cd;
    if (n.length === 0) return;
    const a = t;
    let l = a.alignment;
    if (t.alignment !== 0.5) {
      let Y = Tg(n);
      l = (l - 0.5) * Y + 0.5;
    }
    const c = new Rt(n[0], n[1]), h = new Rt(n[n.length - 2], n[n.length - 1]), d = s, u = Math.abs(c.x - h.x) < o && Math.abs(c.y - h.y) < o;
    if (d) {
      n = n.slice(), u && (n.pop(), n.pop(), h.set(n[n.length - 2], n[n.length - 1]));
      const Y = (c.x + h.x) * 0.5, K = (h.y + c.y) * 0.5;
      n.unshift(Y, K), n.push(Y, K);
    }
    const p = i, f = n.length / 2;
    let g = n.length;
    const m = p.length / 2, y = a.width / 2, b = y * y, x = a.miterLimit * a.miterLimit;
    let _ = n[0], v = n[1], w = n[2], C = n[3], T = 0, R = 0, E = -(v - C), A = _ - w, W = 0, j = 0, G = Math.sqrt(E * E + A * A);
    E /= G, A /= G, E *= y, A *= y;
    const z = l, P = (1 - z) * 2, D = z * 2;
    d || (a.cap === "round" ? g += Gn(_ - E * (P - D) * 0.5, v - A * (P - D) * 0.5, _ - E * P, v - A * P, _ + E * D, v + A * D, p, true) + 2 : a.cap === "square" && (g += Dl(_, v, E, A, P, D, true, p))), p.push(_ - E * P, v - A * P), p.push(_ + E * D, v + A * D);
    for (let Y = 1; Y < f - 1; ++Y) {
      _ = n[(Y - 1) * 2], v = n[(Y - 1) * 2 + 1], w = n[Y * 2], C = n[Y * 2 + 1], T = n[(Y + 1) * 2], R = n[(Y + 1) * 2 + 1], E = -(v - C), A = _ - w, G = Math.sqrt(E * E + A * A), E /= G, A /= G, E *= y, A *= y, W = -(C - R), j = w - T, G = Math.sqrt(W * W + j * j), W /= G, j /= G, W *= y, j *= y;
      const K = w - _, I = v - C, N = w - T, O = R - C, B = K * N + I * O, q = I * N - O * K, Q = q < 0;
      if (Math.abs(q) < 1e-3 * Math.abs(B)) {
        p.push(w - E * P, C - A * P), p.push(w + E * D, C + A * D), B >= 0 && (a.join === "round" ? g += Gn(w, C, w - E * P, C - A * P, w - W * P, C - j * P, p, false) + 4 : g += 2, p.push(w - W * D, C - j * D), p.push(w + W * P, C + j * P));
        continue;
      }
      const ut = (-E + _) * (-A + C) - (-E + w) * (-A + v), ft = (-W + T) * (-j + C) - (-W + w) * (-j + R), mt = (K * ft - N * ut) / q, wt = (O * ut - I * ft) / q, V = (mt - w) * (mt - w) + (wt - C) * (wt - C), st = w + (mt - w) * P, it = C + (wt - C) * P, lt = w - (mt - w) * D, Et = C - (wt - C) * D, Nt = Math.min(K * K + I * I, N * N + O * O), xt = Q ? P : D, jt = Nt + xt * xt * b;
      V <= jt ? a.join === "bevel" || V / b > x ? (Q ? (p.push(st, it), p.push(w + E * D, C + A * D), p.push(st, it), p.push(w + W * D, C + j * D)) : (p.push(w - E * P, C - A * P), p.push(lt, Et), p.push(w - W * P, C - j * P), p.push(lt, Et)), g += 2) : a.join === "round" ? Q ? (p.push(st, it), p.push(w + E * D, C + A * D), g += Gn(w, C, w + E * D, C + A * D, w + W * D, C + j * D, p, true) + 4, p.push(st, it), p.push(w + W * D, C + j * D)) : (p.push(w - E * P, C - A * P), p.push(lt, Et), g += Gn(w, C, w - E * P, C - A * P, w - W * P, C - j * P, p, false) + 4, p.push(w - W * P, C - j * P), p.push(lt, Et)) : (p.push(st, it), p.push(lt, Et)) : (p.push(w - E * P, C - A * P), p.push(w + E * D, C + A * D), a.join === "round" ? Q ? g += Gn(w, C, w + E * D, C + A * D, w + W * D, C + j * D, p, true) + 2 : g += Gn(w, C, w - E * P, C - A * P, w - W * P, C - j * P, p, false) + 2 : a.join === "miter" && V / b <= x && (Q ? (p.push(lt, Et), p.push(lt, Et)) : (p.push(st, it), p.push(st, it)), g += 2), p.push(w - W * P, C - j * P), p.push(w + W * D, C + j * D), g += 2);
    }
    _ = n[(f - 2) * 2], v = n[(f - 2) * 2 + 1], w = n[(f - 1) * 2], C = n[(f - 1) * 2 + 1], E = -(v - C), A = _ - w, G = Math.sqrt(E * E + A * A), E /= G, A /= G, E *= y, A *= y, p.push(w - E * P, C - A * P), p.push(w + E * D, C + A * D), d || (a.cap === "round" ? g += Gn(w - E * (P - D) * 0.5, C - A * (P - D) * 0.5, w - E * P, C - A * P, w + E * D, C + A * D, p, false) + 2 : a.cap === "square" && (g += Dl(w, C, E, A, P, D, false, p)));
    const J = zl * zl;
    for (let Y = m; Y < g + m - 2; ++Y) _ = p[Y * 2], v = p[Y * 2 + 1], w = p[(Y + 1) * 2], C = p[(Y + 1) * 2 + 1], T = p[(Y + 2) * 2], R = p[(Y + 2) * 2 + 1], !(Math.abs(_ * (C - R) + w * (R - v) + T * (v - C)) < J) && r.push(Y, Y + 1, Y + 2);
  };
  function kg(n, t, e, s) {
    const i = Cd;
    if (n.length === 0) return;
    const r = n[0], o = n[1], a = n[n.length - 2], l = n[n.length - 1], c = t || Math.abs(r - a) < i && Math.abs(o - l) < i, h = e, d = n.length / 2, u = h.length / 2;
    for (let p = 0; p < d; p++) h.push(n[p * 2]), h.push(n[p * 2 + 1]);
    for (let p = 0; p < d - 1; p++) s.push(u + p, u + p + 1);
    c && s.push(u + d - 1, u);
  }
  function Sd(n, t, e, s, i, r, o) {
    const a = jf(n, t, 2);
    if (!a) return;
    for (let c = 0; c < a.length; c += 3) r[o++] = a[c] + i, r[o++] = a[c + 1] + i, r[o++] = a[c + 2] + i;
    let l = i * s;
    for (let c = 0; c < n.length; c += 2) e[l] = n[c], e[l + 1] = n[c + 1], l += s;
  }
  const Mg = [], Pg = {
    extension: {
      type: ct.ShapeBuilder,
      name: "polygon"
    },
    build(n, t) {
      for (let e = 0; e < n.points.length; e++) t[e] = n.points[e];
      return true;
    },
    triangulate(n, t, e, s, i, r) {
      Sd(n, Mg, t, e, s, i, r);
    }
  }, Ig = {
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
  }, Rg = {
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
  }, Hl = [
    {
      offset: 0,
      color: "white"
    },
    {
      offset: 1,
      color: "black"
    }
  ], Ca = class Ho {
    constructor(...t) {
      this.uid = ie("fillGradient"), this._tick = 0, this.type = "linear", this.colorStops = [];
      let e = Lg(t);
      e = {
        ...e.type === "radial" ? Ho.defaultRadialOptions : Ho.defaultLinearOptions,
        ...yh(e)
      }, this._textureSize = e.textureSize, this._wrapMode = e.wrapMode, e.type === "radial" ? (this.center = e.center, this.outerCenter = e.outerCenter ?? this.center, this.innerRadius = e.innerRadius, this.outerRadius = e.outerRadius, this.scale = e.scale, this.rotation = e.rotation) : (this.start = e.start, this.end = e.end), this.textureSpace = e.textureSpace, this.type = e.type, e.colorStops.forEach((i) => {
        this.addColorStop(i.offset, i.color);
      });
    }
    addColorStop(t, e) {
      return this.colorStops.push({
        offset: t,
        color: Jt.shared.setValue(e).toHexa()
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
      const l = this.colorStops.length ? this.colorStops : Hl, c = this._textureSize, { canvas: h, context: d } = jl(c, 1), u = a ? d.createLinearGradient(this._textureSize, 0, 0, 0) : d.createLinearGradient(0, 0, this._textureSize, 0);
      Ul(u, l), d.fillStyle = u, d.fillRect(0, 0, c, 1), this.texture = new At({
        source: new Ms({
          resource: h,
          addressMode: this._wrapMode
        })
      });
      const p = Math.sqrt(r * r + o * o), f = Math.atan2(o, r), g = new Ct();
      g.scale(p / c, 1), g.rotate(f), g.translate(t, e), this.textureSpace === "local" && g.scale(c, c), this.transform = g;
    }
    buildGradient() {
      this.texture || this._tick++, this.type === "linear" ? this.buildLinearGradient() : this.buildRadialGradient();
    }
    buildRadialGradient() {
      if (this.texture) return;
      const t = this.colorStops.length ? this.colorStops : Hl, e = this._textureSize, { canvas: s, context: i } = jl(e, e), { x: r, y: o } = this.center, { x: a, y: l } = this.outerCenter, c = this.innerRadius, h = this.outerRadius, d = a - h, u = l - h, p = e / (h * 2), f = (r - d) * p, g = (o - u) * p, m = i.createRadialGradient(f, g, c * p, (a - d) * p, (l - u) * p, h * p);
      Ul(m, t), i.fillStyle = t[t.length - 1].color, i.fillRect(0, 0, e, e), i.fillStyle = m, i.translate(f, g), i.rotate(this.rotation), i.scale(1, this.scale), i.translate(-f, -g), i.fillRect(0, 0, e, e), this.texture = new At({
        source: new Ms({
          resource: s,
          addressMode: this._wrapMode
        })
      });
      const y = new Ct();
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
  Ca.defaultLinearOptions = {
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
  Ca.defaultRadialOptions = {
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
  vn = Ca;
  function Ul(n, t) {
    for (let e = 0; e < t.length; e++) {
      const s = t[e];
      n.addColorStop(s.offset, s.color);
    }
  }
  function jl(n, t) {
    const e = Bt.get().createCanvas(n, t), s = e.getContext("2d");
    return {
      canvas: e,
      context: s
    };
  }
  function Lg(n) {
    let t = n[0] ?? {};
    return (typeof t == "number" || n[1]) && (Ot("8.5.2", "use options object instead"), t = {
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
      textureSize: n[5] ?? vn.defaultLinearOptions.textureSize
    }), t;
  }
  const $g = new Ct(), Og = new Yt();
  Bg = function(n, t, e, s) {
    const i = t.matrix ? n.copyFrom(t.matrix).invert() : n.identity();
    if (t.textureSpace === "local") {
      const o = e.getBounds(Og);
      t.width && o.pad(t.width);
      const { x: a, y: l } = o, c = 1 / o.width, h = 1 / o.height, d = -a * c, u = -l * h, p = i.a, f = i.b, g = i.c, m = i.d;
      i.a *= c, i.b *= c, i.c *= h, i.d *= h, i.tx = d * p + u * g + i.tx, i.ty = d * f + u * m + i.ty;
    } else i.translate(t.texture.frame.x, t.texture.frame.y), i.scale(1 / t.texture.source.width, 1 / t.texture.source.height);
    const r = t.texture.source.style;
    return !(t.fill instanceof vn) && r.addressMode === "clamp-to-edge" && (r.addressMode = "repeat", r.update()), s && i.append($g.copyFrom(s).invert()), i;
  };
  wr = {};
  Ut.handleByMap(ct.ShapeBuilder, wr);
  Ut.add(Ig, Pg, Rg, fi, Sg, Eg);
  const Ng = new Yt(), Fg = new Ct();
  function Wg(n, t) {
    const { geometryData: e, batches: s } = t;
    s.length = 0, e.indices.length = 0, e.vertices.length = 0, e.uvs.length = 0;
    for (let i = 0; i < n.instructions.length; i++) {
      const r = n.instructions[i];
      if (r.action === "texture") Gg(r.data, s, e);
      else if (r.action === "fill" || r.action === "stroke") {
        const o = r.action === "stroke", a = r.data.path.shapePath, l = r.data.style, c = r.data.hole;
        o && c && Yl(c.shapePath, l, true, s, e), c && (a.shapePrimitives[a.shapePrimitives.length - 1].holes = c.shapePath.shapePrimitives), Yl(a, l, o, s, e);
      }
    }
  }
  function Gg(n, t, e) {
    const s = [], i = wr.rectangle, r = Ng;
    r.x = n.dx, r.y = n.dy, r.width = n.dw, r.height = n.dh;
    const o = n.transform;
    if (!i.build(r, s)) return;
    const { vertices: a, uvs: l, indices: c } = e, h = c.length, d = a.length / 2;
    o && vd(s, o), i.triangulate(s, a, 2, d, c, h);
    const u = n.image, p = u.uvs;
    l.push(p.x0, p.y0, p.x1, p.y1, p.x3, p.y3, p.x2, p.y2);
    const f = Me.get(va);
    f.indexOffset = h, f.indexSize = c.length - h, f.attributeOffset = d, f.attributeSize = a.length / 2 - d, f.baseColor = n.style, f.alpha = n.alpha, f.texture = u, f.geometryData = e, t.push(f);
  }
  function Yl(n, t, e, s, i) {
    const { vertices: r, uvs: o, indices: a } = i;
    n.shapePrimitives.forEach(({ shape: l, transform: c, holes: h }) => {
      const d = [], u = wr[l.type];
      if (!u.build(l, d)) return;
      const p = a.length, f = r.length / 2;
      let g = "triangle-list";
      if (c && vd(d, c), e) {
        const x = l.closePath ?? true, _ = t;
        _.pixelLine ? (kg(d, x, r, a), g = "line-list") : Ag(d, _, false, x, r, a);
      } else if (h) {
        const x = [], _ = d.slice();
        zg(h).forEach((w) => {
          x.push(_.length / 2), _.push(...w);
        }), Sd(_, x, r, 2, f, a, p);
      } else u.triangulate(d, r, 2, f, a, p);
      const m = o.length / 2, y = t.texture;
      if (y !== At.WHITE) {
        const x = Bg(Fg, t, l, c);
        wg(r, 2, f, o, m, 2, r.length / 2 - f, x);
      } else vg(o, m, 2, r.length / 2 - f);
      const b = Me.get(va);
      b.indexOffset = p, b.indexSize = a.length - p, b.attributeOffset = f, b.attributeSize = r.length / 2 - f, b.baseColor = t.color, b.alpha = t.alpha, b.texture = y, b.geometryData = i, b.topology = g, s.push(b);
    });
  }
  function zg(n) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e].shape, i = [];
      wr[s.type].build(s, i) && t.push(i);
    }
    return t;
  }
  class Dg {
    constructor() {
      this.batches = [], this.geometryData = {
        vertices: [],
        uvs: [],
        indices: []
      };
    }
    reset() {
      this.batches && this.batches.forEach((t) => {
        Me.return(t);
      }), this.graphicsData && Me.return(this.graphicsData), this.isBatchable = false, this.context = null, this.batches.length = 0, this.geometryData.indices.length = 0, this.geometryData.vertices.length = 0, this.geometryData.uvs.length = 0, this.graphicsData = null;
    }
    destroy() {
      this.reset(), this.batches = null, this.geometryData = null;
    }
  }
  class Hg {
    constructor() {
      this.instructions = new fa();
    }
    init(t) {
      const e = t.maxTextures;
      this.batcher ? this.batcher._updateMaxTextures(e) : this.batcher = new _g({
        maxTextures: e
      }), this.instructions.reset();
    }
    get geometry() {
      return Ot(gp, "GraphicsContextRenderData#geometry is deprecated, please use batcher.geometry instead."), this.batcher.geometry;
    }
    destroy() {
      this.batcher.destroy(), this.instructions.destroy(), this.batcher = null, this.instructions = null;
    }
  }
  const Sa = class Uo {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Bs({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      Uo.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? Uo.defaultOptions.bezierSmoothness;
    }
    getContextRenderData(t) {
      return t._gpuData[this._renderer.uid].graphicsData || this._initContextRenderData(t);
    }
    updateGpuContext(t) {
      const e = !!t._gpuData[this._renderer.uid], s = t._gpuData[this._renderer.uid] || this._initContext(t);
      if (t.dirty || !e) {
        e && s.reset(), Wg(t, s);
        const i = t.batchMode;
        t.customShader || i === "no-batch" ? s.isBatchable = false : i === "auto" ? s.isBatchable = s.geometryData.vertices.length < 400 : s.isBatchable = true, t.dirty = false;
      }
      return s;
    }
    getGpuContext(t) {
      return t._gpuData[this._renderer.uid] || this._initContext(t);
    }
    _initContextRenderData(t) {
      const e = Me.get(Hg, {
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
        u.bindGroup = Rm(u.textures.textures, u.textures.count, this._renderer.limits.maxBatchableTextures);
      }
      return e;
    }
    _initContext(t) {
      const e = new Dg();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  Sa.extension = {
    type: [
      ct.WebGLSystem,
      ct.WebGPUSystem
    ],
    name: "graphicsContext"
  };
  Sa.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let Ea = Sa;
  const Ug = 8, Fi = 11920929e-14, jg = 1;
  function Ed(n, t, e, s, i, r, o, a, l, c) {
    const d = Math.min(0.99, Math.max(0, c ?? Ea.defaultOptions.bezierSmoothness));
    let u = (jg - d) / 1;
    return u *= u, Yg(t, e, s, i, r, o, a, l, n, u), n;
  }
  function Yg(n, t, e, s, i, r, o, a, l, c) {
    jo(n, t, e, s, i, r, o, a, l, c, 0), l.push(o, a);
  }
  function jo(n, t, e, s, i, r, o, a, l, c, h) {
    if (h > Ug) return;
    const d = (n + e) / 2, u = (t + s) / 2, p = (e + i) / 2, f = (s + r) / 2, g = (i + o) / 2, m = (r + a) / 2, y = (d + p) / 2, b = (u + f) / 2, x = (p + g) / 2, _ = (f + m) / 2, v = (y + x) / 2, w = (b + _) / 2;
    if (h > 0) {
      let C = o - n, T = a - t;
      const R = Math.abs((e - o) * T - (s - a) * C), E = Math.abs((i - o) * T - (r - a) * C);
      if (R > Fi && E > Fi) {
        if ((R + E) * (R + E) <= c * (C * C + T * T)) {
          l.push(v, w);
          return;
        }
      } else if (R > Fi) {
        if (R * R <= c * (C * C + T * T)) {
          l.push(v, w);
          return;
        }
      } else if (E > Fi) {
        if (E * E <= c * (C * C + T * T)) {
          l.push(v, w);
          return;
        }
      } else if (C = v - (n + o) / 2, T = w - (t + a) / 2, C * C + T * T <= c) {
        l.push(v, w);
        return;
      }
    }
    jo(n, t, d, u, y, b, v, w, l, c, h + 1), jo(v, w, x, _, g, m, o, a, l, c, h + 1);
  }
  const Vg = 8, Xg = 11920929e-14, qg = 1;
  function Kg(n, t, e, s, i, r, o, a) {
    const c = Math.min(0.99, Math.max(0, a ?? Ea.defaultOptions.bezierSmoothness));
    let h = (qg - c) / 1;
    return h *= h, Jg(t, e, s, i, r, o, n, h), n;
  }
  function Jg(n, t, e, s, i, r, o, a) {
    Yo(o, n, t, e, s, i, r, a, 0), o.push(i, r);
  }
  function Yo(n, t, e, s, i, r, o, a, l) {
    if (l > Vg) return;
    const c = (t + s) / 2, h = (e + i) / 2, d = (s + r) / 2, u = (i + o) / 2, p = (c + d) / 2, f = (h + u) / 2;
    let g = r - t, m = o - e;
    const y = Math.abs((s - r) * m - (i - o) * g);
    if (y > Xg) {
      if (y * y <= a * (g * g + m * m)) {
        n.push(p, f);
        return;
      }
    } else if (g = p - (t + r) / 2, m = f - (e + o) / 2, g * g + m * m <= a) {
      n.push(p, f);
      return;
    }
    Yo(n, t, e, c, h, p, f, a, l + 1), Yo(n, p, f, d, u, r, o, a, l + 1);
  }
  function Td(n, t, e, s, i, r, o, a) {
    let l = Math.abs(i - r);
    (!o && i > r || o && r > i) && (l = 2 * Math.PI - l), a || (a = Math.max(6, Math.floor(6 * Math.pow(s, 1 / 3) * (l / Math.PI)))), a = Math.max(a, 3);
    let c = l / a, h = i;
    c *= o ? -1 : 1;
    for (let d = 0; d < a + 1; d++) {
      const u = Math.cos(h), p = Math.sin(h), f = t + u * s, g = e + p * s;
      n.push(f, g), h += c;
    }
  }
  function Zg(n, t, e, s, i, r) {
    const o = n[n.length - 2], l = n[n.length - 1] - e, c = o - t, h = i - e, d = s - t, u = Math.abs(l * d - c * h);
    if (u < 1e-8 || r === 0) {
      (n[n.length - 2] !== t || n[n.length - 1] !== e) && n.push(t, e);
      return;
    }
    const p = l * l + c * c, f = h * h + d * d, g = l * h + c * d, m = r * Math.sqrt(p) / u, y = r * Math.sqrt(f) / u, b = m * g / p, x = y * g / f, _ = m * d + y * c, v = m * h + y * l, w = c * (y + b), C = l * (y + b), T = d * (m + x), R = h * (m + x), E = Math.atan2(C - v, w - _), A = Math.atan2(R - v, T - _);
    Td(n, _ + t, v + e, r, E, A, c * h > d * l);
  }
  const oi = Math.PI * 2, io = {
    centerX: 0,
    centerY: 0,
    ang1: 0,
    ang2: 0
  }, ro = ({ x: n, y: t }, e, s, i, r, o, a, l) => {
    n *= e, t *= s;
    const c = i * n - r * t, h = r * n + i * t;
    return l.x = c + o, l.y = h + a, l;
  };
  function Qg(n, t) {
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
  const Vl = (n, t, e, s) => {
    const i = n * s - t * e < 0 ? -1 : 1;
    let r = n * e + t * s;
    return r > 1 && (r = 1), r < -1 && (r = -1), i * Math.acos(r);
  }, ty = (n, t, e, s, i, r, o, a, l, c, h, d, u) => {
    const p = Math.pow(i, 2), f = Math.pow(r, 2), g = Math.pow(h, 2), m = Math.pow(d, 2);
    let y = p * f - p * m - f * g;
    y < 0 && (y = 0), y /= p * m + f * g, y = Math.sqrt(y) * (o === a ? -1 : 1);
    const b = y * i / r * d, x = y * -r / i * h, _ = c * b - l * x + (n + e) / 2, v = l * b + c * x + (t + s) / 2, w = (h - b) / i, C = (d - x) / r, T = (-h - b) / i, R = (-d - x) / r, E = Vl(1, 0, w, C);
    let A = Vl(w, C, T, R);
    a === 0 && A > 0 && (A -= oi), a === 1 && A < 0 && (A += oi), u.centerX = _, u.centerY = v, u.ang1 = E, u.ang2 = A;
  };
  function ey(n, t, e, s, i, r, o, a = 0, l = 0, c = 0) {
    if (r === 0 || o === 0) return;
    const h = Math.sin(a * oi / 360), d = Math.cos(a * oi / 360), u = d * (t - s) / 2 + h * (e - i) / 2, p = -h * (t - s) / 2 + d * (e - i) / 2;
    if (u === 0 && p === 0) return;
    r = Math.abs(r), o = Math.abs(o);
    const f = Math.pow(u, 2) / Math.pow(r, 2) + Math.pow(p, 2) / Math.pow(o, 2);
    f > 1 && (r *= Math.sqrt(f), o *= Math.sqrt(f)), ty(t, e, s, i, r, o, l, c, h, d, u, p, io);
    let { ang1: g, ang2: m } = io;
    const { centerX: y, centerY: b } = io;
    let x = Math.abs(m) / (oi / 4);
    Math.abs(1 - x) < 1e-7 && (x = 1);
    const _ = Math.max(Math.ceil(x), 1);
    m /= _;
    let v = n[n.length - 2], w = n[n.length - 1];
    const C = {
      x: 0,
      y: 0
    };
    for (let T = 0; T < _; T++) {
      const R = Qg(g, m), { x: E, y: A } = ro(R[0], r, o, d, h, y, b, C), { x: W, y: j } = ro(R[1], r, o, d, h, y, b, C), { x: G, y: z } = ro(R[2], r, o, d, h, y, b, C);
      Ed(n, v, w, E, A, W, j, G, z), v = G, w = z, g += m;
    }
  }
  function ny(n, t, e) {
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
  function sy(n, t, e, s) {
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
  const iy = new Yt();
  class ry {
    constructor(t) {
      this.shapePrimitives = [], this._currentPoly = null, this._bounds = new $e(), this._graphicsPath2D = t, this.signed = t.checkForHoles;
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
      return Td(a, t, e, s, i, r, o), this;
    }
    arcTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly.points;
      return Zg(o, t, e, s, i, r), this;
    }
    arcToSvg(t, e, s, i, r, o, a) {
      const l = this._currentPoly.points;
      return ey(l, this._currentPoly.lastX, this._currentPoly.lastY, o, a, t, e, s, i, r), this;
    }
    bezierCurveTo(t, e, s, i, r, o, a) {
      this._ensurePoly();
      const l = this._currentPoly;
      return Ed(this._currentPoly.points, l.lastX, l.lastY, t, e, s, i, r, o, a), this;
    }
    quadraticCurveTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly;
      return Kg(this._currentPoly.points, o.lastX, o.lastY, t, e, s, i, r), this;
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
      return this.drawShape(new Yt(t, e, s, i), r), this;
    }
    circle(t, e, s, i) {
      return this.drawShape(new ba(t, e, s), i), this;
    }
    poly(t, e, s) {
      const i = new ii(t);
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
      return t.length < 3 ? this : (s ? sy(this, t, e, i) : ny(this, t, e), this.closePath());
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
      return this.drawShape(new _a(t, e, s, i), r), this;
    }
    roundRect(t, e, s, i, r, o) {
      return this.drawShape(new wa(t, e, s, i, r), o), this;
    }
    drawShape(t, e) {
      return this.endPoly(), this.shapePrimitives.push({
        shape: t,
        transform: e
      }), this;
    }
    startPoly(t, e) {
      let s = this._currentPoly;
      return s && this.endPoly(), s = new ii(), s.points.push(t, e), this._currentPoly = s, this;
    }
    endPoly(t = false) {
      const e = this._currentPoly;
      return e && e.points.length > 2 && (e.closePath = t, this.shapePrimitives.push({
        shape: e
      })), this._currentPoly = null, this;
    }
    _ensurePoly(t = true) {
      if (!this._currentPoly && (this._currentPoly = new ii(), t)) {
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
        const i = e[s], r = i.shape.getBounds(iy);
        i.transform ? t.addRect(r, i.transform) : t.addRect(r);
      }
      return t;
    }
  }
  class _n {
    constructor(t, e = false) {
      this.instructions = [], this.uid = ie("graphicsPath"), this._dirty = true, this.checkForHoles = e, typeof t == "string" ? km(t, this) : this.instructions = (t == null ? void 0 : t.slice()) ?? [];
    }
    get shapePath() {
      return this._shapePath || (this._shapePath = new ry(this)), this._dirty && (this._dirty = false, this._shapePath.buildPath()), this._shapePath;
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
      const o = this.instructions[this.instructions.length - 1], a = this.getLastPoint(Rt.shared);
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
      const i = this.instructions[this.instructions.length - 1], r = this.getLastPoint(Rt.shared);
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
      const e = new _n();
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
            b[4] = Ys(b[3], t);
            break;
          case "rect":
            b[4] = Ys(b[4], t);
            break;
          case "ellipse":
            b[8] = Ys(b[8], t);
            break;
          case "roundRect":
            b[5] = Ys(b[5], t);
            break;
          case "addPath":
            b[0].transform(t);
            break;
          case "poly":
            b[2] = Ys(b[2], t);
            break;
          default:
            Qt("unknown transform action", y.action);
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
  function Ys(n, t) {
    return n ? n.prepend(t) : t.clone();
  }
  function ne(n, t, e) {
    const s = n.getAttribute(t);
    return s ? Number(s) : e;
  }
  function oy(n, t) {
    const e = n.querySelectorAll("defs");
    for (let s = 0; s < e.length; s++) {
      const i = e[s];
      for (let r = 0; r < i.children.length; r++) {
        const o = i.children[r];
        switch (o.nodeName.toLowerCase()) {
          case "lineargradient":
            t.defs[o.id] = ay(o);
            break;
          case "radialgradient":
            t.defs[o.id] = ly();
            break;
        }
      }
    }
  }
  function ay(n) {
    const t = ne(n, "x1", 0), e = ne(n, "y1", 0), s = ne(n, "x2", 1), i = ne(n, "y2", 0), r = n.getAttribute("gradientUnits") || "objectBoundingBox", o = new vn(t, e, s, i, r === "objectBoundingBox" ? "local" : "global");
    for (let a = 0; a < n.children.length; a++) {
      const l = n.children[a], c = ne(l, "offset", 0), h = Jt.shared.setValue(l.getAttribute("stop-color")).toNumber();
      o.addColorStop(c, h);
    }
    return o;
  }
  function ly(n) {
    return Qt("[SVG Parser] Radial gradients are not yet supported"), new vn(0, 0, 1, 0);
  }
  function Xl(n) {
    const t = n.match(/url\s*\(\s*['"]?\s*#([^'"\s)]+)\s*['"]?\s*\)/i);
    return t ? t[1] : "";
  }
  const ql = {
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
  function Ad(n, t) {
    const e = n.getAttribute("style"), s = {}, i = {}, r = {
      strokeStyle: s,
      fillStyle: i,
      useFill: false,
      useStroke: false
    };
    for (const o in ql) {
      const a = n.getAttribute(o);
      a && Kl(t, r, o, a.trim());
    }
    if (e) {
      const o = e.split(";");
      for (let a = 0; a < o.length; a++) {
        const l = o[a].trim(), [c, h] = l.split(":");
        ql[c] && Kl(t, r, c, h.trim());
      }
    }
    return {
      strokeStyle: r.useStroke ? s : null,
      fillStyle: r.useFill ? i : null,
      useFill: r.useFill,
      useStroke: r.useStroke
    };
  }
  function Kl(n, t, e, s) {
    switch (e) {
      case "stroke":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Xl(s);
            t.strokeStyle.fill = n.defs[i];
          } else t.strokeStyle.color = Jt.shared.setValue(s).toNumber();
          t.useStroke = true;
        }
        break;
      case "stroke-width":
        t.strokeStyle.width = Number(s);
        break;
      case "fill":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Xl(s);
            t.fillStyle.fill = n.defs[i];
          } else t.fillStyle.color = Jt.shared.setValue(s).toNumber();
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
  function cy(n) {
    if (n.length <= 2) return true;
    const t = n.map((a) => a.area).sort((a, l) => l - a), [e, s] = t, i = t[t.length - 1], r = e / s, o = s / i;
    return !(r > 3 && o < 2);
  }
  function hy(n) {
    return n.split(/(?=[Mm])/).filter((s) => s.trim().length > 0);
  }
  function dy(n) {
    const t = n.match(/[-+]?[0-9]*\.?[0-9]+/g);
    if (!t || t.length < 4) return 0;
    const e = t.map(Number), s = [], i = [];
    for (let h = 0; h < e.length; h += 2) h + 1 < e.length && (s.push(e[h]), i.push(e[h + 1]));
    if (s.length === 0 || i.length === 0) return 0;
    const r = Math.min(...s), o = Math.max(...s), a = Math.min(...i), l = Math.max(...i);
    return (o - r) * (l - a);
  }
  function Jl(n, t) {
    const e = new _n(n, false);
    for (const s of e.instructions) t.instructions.push(s);
  }
  function uy(n, t) {
    if (typeof n == "string") {
      const o = document.createElement("div");
      o.innerHTML = n.trim(), n = o.querySelector("svg");
    }
    const e = {
      context: t,
      defs: {},
      path: new _n()
    };
    oy(n, e);
    const s = n.children, { fillStyle: i, strokeStyle: r } = Ad(n, e);
    for (let o = 0; o < s.length; o++) {
      const a = s[o];
      a.nodeName.toLowerCase() !== "defs" && kd(a, e, i, r);
    }
    return t;
  }
  function kd(n, t, e, s) {
    const i = n.children, { fillStyle: r, strokeStyle: o } = Ad(n, t);
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
        const R = n.getAttribute("fill-rule"), E = hy(v), A = R === "evenodd", W = E.length > 1;
        if (A && W) {
          const G = E.map((P) => ({
            path: P,
            area: dy(P)
          }));
          if (G.sort((P, D) => D.area - P.area), E.length > 3 || !cy(G)) for (let P = 0; P < G.length; P++) {
            const D = G[P], J = P === 0;
            t.context.beginPath();
            const Y = new _n(void 0, true);
            Jl(D.path, Y), t.context.path(Y), J ? (e && t.context.fill(e), s && t.context.stroke(s)) : t.context.cut();
          }
          else for (let P = 0; P < G.length; P++) {
            const D = G[P], J = P % 2 === 1;
            t.context.beginPath();
            const Y = new _n(void 0, true);
            Jl(D.path, Y), t.context.path(Y), J ? t.context.cut() : (e && t.context.fill(e), s && t.context.stroke(s));
          }
        } else {
          const G = R ? R === "evenodd" : true;
          w = new _n(v, G), t.context.path(w), e && t.context.fill(e), s && t.context.stroke(s);
        }
        break;
      }
      case "circle":
        f = ne(n, "cx", 0), g = ne(n, "cy", 0), m = ne(n, "r", 0), t.context.ellipse(f, g, m, m), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "rect":
        l = ne(n, "x", 0), c = ne(n, "y", 0), C = ne(n, "width", 0), T = ne(n, "height", 0), y = ne(n, "rx", 0), b = ne(n, "ry", 0), y || b ? t.context.roundRect(l, c, C, T, y || b) : t.context.rect(l, c, C, T), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "ellipse":
        f = ne(n, "cx", 0), g = ne(n, "cy", 0), y = ne(n, "rx", 0), b = ne(n, "ry", 0), t.context.beginPath(), t.context.ellipse(f, g, y, b), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "line":
        h = ne(n, "x1", 0), d = ne(n, "y1", 0), u = ne(n, "x2", 0), p = ne(n, "y2", 0), t.context.beginPath(), t.context.moveTo(h, d), t.context.lineTo(u, p), s && t.context.stroke(s);
        break;
      case "polygon":
        _ = n.getAttribute("points"), x = _.match(/-?\d+/g).map((R) => parseInt(R, 10)), t.context.poly(x, true), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "polyline":
        _ = n.getAttribute("points"), x = _.match(/-?\d+/g).map((R) => parseInt(R, 10)), t.context.poly(x, false), s && t.context.stroke(s);
        break;
      case "g":
      case "svg":
        break;
      default: {
        Qt(`[SVG parser] <${n.nodeName}> elements unsupported`);
        break;
      }
    }
    a && (e = null);
    for (let R = 0; R < i.length; R++) kd(i[R], t, e, s);
  }
  const Zl = {
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
  vr = class {
    constructor(t, e) {
      this.uid = ie("fillPattern"), this._tick = 0, this.transform = new Ct(), this.texture = t, this.transform.scale(1 / t.frame.width, 1 / t.frame.height), e && (t.source.style.addressModeU = Zl[e].addressModeU, t.source.style.addressModeV = Zl[e].addressModeV);
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
  function py(n) {
    return Jt.isColorLike(n);
  }
  function Ql(n) {
    return n instanceof vr;
  }
  function tc(n) {
    return n instanceof vn;
  }
  function fy(n) {
    return n instanceof At;
  }
  function my(n, t, e) {
    const s = Jt.shared.setValue(t ?? 0);
    return n.color = s.toNumber(), n.alpha = s.alpha === 1 ? e.alpha : s.alpha, n.texture = At.WHITE, {
      ...e,
      ...n
    };
  }
  function gy(n, t, e) {
    return n.texture = t, {
      ...e,
      ...n
    };
  }
  function ec(n, t, e) {
    return n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, {
      ...e,
      ...n
    };
  }
  function nc(n, t, e) {
    return t.buildGradient(), n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, n.textureSpace = t.textureSpace, {
      ...e,
      ...n
    };
  }
  function yy(n, t) {
    const e = {
      ...t,
      ...n
    }, s = Jt.shared.setValue(e.color);
    return e.alpha *= s.alpha, e.color = s.toNumber(), e;
  }
  function Zn(n, t) {
    if (n == null) return null;
    const e = {}, s = n;
    return py(n) ? my(e, n, t) : fy(n) ? gy(e, n, t) : Ql(n) ? ec(e, n, t) : tc(n) ? nc(e, n, t) : s.fill && Ql(s.fill) ? ec(s, s.fill, t) : s.fill && tc(s.fill) ? nc(s, s.fill, t) : yy(s, t);
  }
  function lr(n, t) {
    const { width: e, alignment: s, miterLimit: i, cap: r, join: o, pixelLine: a, ...l } = t, c = Zn(n, l);
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
  function xy(n, t) {
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
        const W = Math.min(1 / A, t);
        W > e && (e = W);
      }
    }
    return e;
  }
  const by = new Rt(), sc = new Ct(), Ta = class sn extends hn {
    constructor() {
      super(...arguments), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ie("graphicsContext"), this.dirty = true, this.batchMode = "auto", this.instructions = [], this.destroyed = false, this._activePath = new _n(), this._transform = new Ct(), this._fillStyle = {
        ...sn.defaultFillStyle
      }, this._strokeStyle = {
        ...sn.defaultStrokeStyle
      }, this._stateStack = [], this._tick = 0, this._bounds = new $e(), this._boundsDirty = true;
    }
    clone() {
      const t = new sn();
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
      this._fillStyle = Zn(t, sn.defaultFillStyle);
    }
    get strokeStyle() {
      return this._strokeStyle;
    }
    set strokeStyle(t) {
      this._strokeStyle = lr(t, sn.defaultStrokeStyle);
    }
    setFillStyle(t) {
      return this._fillStyle = Zn(t, sn.defaultFillStyle), this;
    }
    setStrokeStyle(t) {
      return this._strokeStyle = Zn(t, sn.defaultStrokeStyle), this;
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
          style: e || e === 0 ? Jt.shared.setValue(e).toNumber() : 16777215
        }
      }), this.onUpdate(), this;
    }
    beginPath() {
      return this._activePath = new _n(), this;
    }
    fill(t, e) {
      let s;
      const i = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (i == null ? void 0 : i.action) === "stroke" ? s = i.data.path : s = this._activePath.clone(), s ? (t != null && (e !== void 0 && typeof t == "number" && (Ot(se, "GraphicsContext.fill(color, alpha) is deprecated, use GraphicsContext.fill({ color, alpha }) instead"), t = {
        color: t,
        alpha: e
      }), this._fillStyle = Zn(t, sn.defaultFillStyle)), this.instructions.push({
        action: "fill",
        data: {
          style: this.fillStyle,
          path: s
        }
      }), this.onUpdate(), this._initNextPathLocation(), this._tick = 0, this) : this;
    }
    _initNextPathLocation() {
      const { x: t, y: e } = this._activePath.getLastPoint(Rt.shared);
      this._activePath.clear(), this._activePath.moveTo(t, e);
    }
    stroke(t) {
      let e;
      const s = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (s == null ? void 0 : s.action) === "fill" ? e = s.data.path : e = this._activePath.clone(), e ? (t != null && (this._strokeStyle = lr(t, sn.defaultStrokeStyle)), this.instructions.push({
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
      return this._tick++, uy(t, this), this;
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
      return t instanceof Ct ? (this._transform.set(t.a, t.b, t.c, t.d, t.tx, t.ty), this) : (this._transform.set(t, e, s, i, r, o), this);
    }
    transform(t, e, s, i, r, o) {
      return t instanceof Ct ? (this._transform.append(t), this) : (sc.set(t, e, s, i, r, o), this._transform.append(sc), this);
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
          r.style.join === "miter" && (a *= xy(r.path, r.style.miterLimit));
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
          const u = c[h].transform, p = u ? u.applyInverse(t, by) : t;
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
  Ta.defaultFillStyle = {
    color: 16777215,
    alpha: 1,
    texture: At.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local"
  };
  Ta.defaultStrokeStyle = {
    width: 1,
    color: 16777215,
    alpha: 1,
    alignment: 0.5,
    miterLimit: 10,
    cap: "butt",
    join: "miter",
    texture: At.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local",
    pixelLine: false
  };
  let Be = Ta;
  function Aa(n, t = 1) {
    var _a2;
    const e = (_a2 = Ls.RETINA_PREFIX) == null ? void 0 : _a2.exec(n);
    return e ? parseFloat(e[1]) : t;
  }
  function ka(n, t, e) {
    n.label = e, n._sourceOrigin = e;
    const s = new At({
      source: n,
      label: e
    }), i = () => {
      delete t.promiseCache[e], oe.has(e) && oe.remove(e);
    };
    return s.source.once("destroy", () => {
      t.promiseCache[e] && (Qt("[Assets] A TextureSource managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the TextureSource."), i());
    }), s.once("destroy", () => {
      n.destroyed || (Qt("[Assets] A Texture managed by Assets was destroyed instead of unloaded! Use Assets.unload() instead of destroying the Texture."), i());
    }), s;
  }
  const _y = ".svg", wy = "image/svg+xml", vy = {
    extension: {
      type: ct.LoadParser,
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
      return $s(n, wy) || Os(n, _y);
    },
    async load(n, t, e) {
      var _a2;
      return ((_a2 = t.data) == null ? void 0 : _a2.parseAsGraphicsContext) ?? this.config.parseAsGraphicsContext ? Sy(n) : Cy(n, t, e, this.config.crossOrigin);
    },
    unload(n) {
      n.destroy(true);
    }
  };
  async function Cy(n, t, e, s) {
    var _a2, _b2, _c2;
    const i = await Bt.get().fetch(n), r = Bt.get().createImage();
    r.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(await i.text())}`, r.crossOrigin = s, await r.decode();
    const o = ((_a2 = t.data) == null ? void 0 : _a2.width) ?? r.width, a = ((_b2 = t.data) == null ? void 0 : _b2.height) ?? r.height, l = ((_c2 = t.data) == null ? void 0 : _c2.resolution) || Aa(n), c = Math.ceil(o * l), h = Math.ceil(a * l), d = Bt.get().createCanvas(c, h), u = d.getContext("2d");
    u.imageSmoothingEnabled = true, u.imageSmoothingQuality = "high", u.drawImage(r, 0, 0, o * l, a * l);
    const { parseAsGraphicsContext: p, ...f } = t.data ?? {}, g = new Ms({
      resource: d,
      alphaMode: "premultiply-alpha-on-upload",
      resolution: l,
      ...f
    });
    return ka(g, e, n);
  }
  async function Sy(n) {
    const e = await (await Bt.get().fetch(n)).text(), s = new Be();
    return s.svg(e), s;
  }
  const Ey = `(function () {
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
  let Cs = null, Vo = class {
    constructor() {
      Cs || (Cs = URL.createObjectURL(new Blob([
        Ey
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(Cs);
    }
  };
  Vo.revokeObjectURL = function() {
    Cs && (URL.revokeObjectURL(Cs), Cs = null);
  };
  const Ty = `(function () {
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
  let Ss = null;
  class Md {
    constructor() {
      Ss || (Ss = URL.createObjectURL(new Blob([
        Ty
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(Ss);
    }
  }
  Md.revokeObjectURL = function() {
    Ss && (URL.revokeObjectURL(Ss), Ss = null);
  };
  let ic = 0, oo;
  class Ay {
    constructor() {
      this._initialized = false, this._createdWorkers = 0, this._workerPool = [], this._queue = [], this._resolveHash = {};
    }
    isImageBitmapSupported() {
      return this._isImageBitmapSupported !== void 0 ? this._isImageBitmapSupported : (this._isImageBitmapSupported = new Promise((t) => {
        const { worker: e } = new Vo();
        e.addEventListener("message", (s) => {
          e.terminate(), Vo.revokeObjectURL(), t(s.data);
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
      oo === void 0 && (oo = navigator.hardwareConcurrency || 4);
      let t = this._workerPool.pop();
      return !t && this._createdWorkers < oo && (this._createdWorkers++, t = new Md().worker, t.addEventListener("message", (e) => {
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
      this._resolveHash[ic] = {
        resolve: e.resolve,
        reject: e.reject
      }, t.postMessage({
        data: e.arguments,
        uuid: ic++,
        id: s
      });
    }
    reset() {
      this._workerPool.forEach((t) => t.terminate()), this._workerPool.length = 0, Object.values(this._resolveHash).forEach(({ reject: t }) => {
        t == null ? void 0 : t(new Error("WorkerManager has been reset before completion"));
      }), this._resolveHash = {}, this._queue.length = 0, this._initialized = false, this._createdWorkers = 0;
    }
  }
  const rc = new Ay(), ky = [
    ".jpeg",
    ".jpg",
    ".png",
    ".webp",
    ".avif"
  ], My = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/avif"
  ];
  async function Py(n, t) {
    var _a2;
    const e = await Bt.get().fetch(n);
    if (!e.ok) throw new Error(`[loadImageBitmap] Failed to fetch ${n}: ${e.status} ${e.statusText}`);
    const s = await e.blob();
    return ((_a2 = t == null ? void 0 : t.data) == null ? void 0 : _a2.alphaMode) === "premultiplied-alpha" ? createImageBitmap(s, {
      premultiplyAlpha: "none"
    }) : createImageBitmap(s);
  }
  const Pd = {
    name: "loadTextures",
    id: "texture",
    extension: {
      type: ct.LoadParser,
      priority: Bn.High,
      name: "loadTextures"
    },
    config: {
      preferWorkers: true,
      preferCreateImageBitmap: true,
      crossOrigin: "anonymous"
    },
    test(n) {
      return $s(n, My) || Os(n, ky);
    },
    async load(n, t, e) {
      var _a2;
      let s = null;
      globalThis.createImageBitmap && this.config.preferCreateImageBitmap ? this.config.preferWorkers && await rc.isImageBitmapSupported() ? s = await rc.loadImageBitmap(n, t) : s = await Py(n, t) : s = await new Promise((r, o) => {
        s = Bt.get().createImage(), s.crossOrigin = this.config.crossOrigin, s.src = n, s.complete ? r(s) : (s.onload = () => {
          r(s);
        }, s.onerror = o);
      });
      const i = new Ms({
        resource: s,
        alphaMode: "premultiply-alpha-on-upload",
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || Aa(n),
        ...t.data
      });
      return ka(i, e, n);
    },
    unload(n) {
      n.destroy(true);
    }
  }, Iy = [
    ".mp4",
    ".m4v",
    ".webm",
    ".ogg",
    ".ogv",
    ".h264",
    ".avi",
    ".mov"
  ];
  let ao, lo;
  function Ry(n, t, e) {
    e === void 0 && !t.startsWith("data:") ? n.crossOrigin = $y(t) : e !== false && (n.crossOrigin = typeof e == "string" ? e : "anonymous");
  }
  function Ly(n) {
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
  function $y(n, t = globalThis.location) {
    if (n.startsWith("data:")) return "";
    t || (t = globalThis.location);
    const e = new URL(n, document.baseURI);
    return e.hostname !== t.hostname || e.port !== t.port || e.protocol !== t.protocol ? "anonymous" : "";
  }
  function Oy() {
    const n = [], t = [];
    for (const e of Iy) {
      const s = si.MIME_TYPES[e.substring(1)] || `video/${e.substring(1)}`;
      _r(s) && (n.push(e), t.includes(s) || t.push(s));
    }
    return {
      validVideoExtensions: n,
      validVideoMime: t
    };
  }
  const By = {
    name: "loadVideo",
    id: "video",
    extension: {
      type: ct.LoadParser,
      name: "loadVideo"
    },
    test(n) {
      if (!ao || !lo) {
        const { validVideoExtensions: s, validVideoMime: i } = Oy();
        ao = s, lo = i;
      }
      const t = $s(n, lo), e = Os(n, ao);
      return t || e;
    },
    async load(n, t, e) {
      var _a2, _b2;
      const s = {
        ...si.defaultOptions,
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || Aa(n),
        alphaMode: ((_b2 = t.data) == null ? void 0 : _b2.alphaMode) || await Gh(),
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
      }), s.muted === true && (i.muted = true), Ry(i, n, s.crossorigin);
      const o = document.createElement("source");
      let a;
      if (s.mime) a = s.mime;
      else if (n.startsWith("data:")) a = n.slice(5, n.indexOf(";"));
      else if (!n.startsWith("blob:")) {
        const l = n.split("?")[0].slice(n.lastIndexOf(".") + 1).toLowerCase();
        a = si.MIME_TYPES[l] || `video/${l}`;
      }
      return o.src = n, a && (o.type = a), new Promise((l, c) => {
        s.preload && !s.autoPlay && i.load(), i.addEventListener("canplay", h), i.addEventListener("error", d), o.addEventListener("error", d), i.appendChild(o);
        async function h() {
          const p = new si({
            ...s,
            resource: i
          });
          u(), t.data.preload && await Ly(i), l(ka(p, e, n));
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
  }, Id = {
    extension: {
      type: ct.ResolveParser,
      name: "resolveTexture"
    },
    test: Pd.test,
    parse: (n) => {
      var _a2;
      return {
        resolution: parseFloat(((_a2 = Ls.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
        format: n.split(".").pop(),
        src: n
      };
    }
  }, Ny = {
    extension: {
      type: ct.ResolveParser,
      priority: -2,
      name: "resolveJson"
    },
    test: (n) => Ls.RETINA_PREFIX.test(n) && n.endsWith(".json"),
    parse: Id.parse
  };
  class Fy {
    constructor() {
      this._detections = [], this._initialized = false, this.resolver = new Ls(), this.loader = new hm(), this.cache = oe, this._backgroundLoader = new em(this.loader), this._backgroundLoader.active = true, this.reset();
    }
    async init(t = {}) {
      var _a2, _b2;
      if (this._initialized) {
        Qt("[Assets]AssetManager already initialized, did you load before calling this Assets.init()?");
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
      const s = ir(t), i = Xe(t).map((a) => {
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
      if (typeof t == "string") return oe.get(t);
      const e = {};
      for (let s = 0; s < t.length; s++) e[s] = oe.get(t[s]);
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
        }), oe.set(l, a);
      }), r;
    }
    async unload(t) {
      this._initialized || await this.init();
      const e = Xe(t).map((i) => typeof i != "string" ? i.src : i), s = this.resolver.resolve(e);
      await this._unloadFromResolved(s);
    }
    async unloadBundle(t) {
      this._initialized || await this.init(), t = Xe(t);
      const e = this.resolver.resolveBundle(t), s = Object.keys(e).map((i) => this._unloadFromResolved(e[i]));
      await Promise.all(s);
    }
    async _unloadFromResolved(t) {
      const e = Object.values(t);
      e.forEach((s) => {
        oe.remove(s.src);
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
  const Fe = new Fy();
  Ut.handleByList(ct.LoadParser, Fe.loader.parsers).handleByList(ct.ResolveParser, Fe.resolver.parsers).handleByList(ct.CacheParser, Fe.cache.parsers).handleByList(ct.DetectionParser, Fe.detections);
  Ut.add(nm, im, sm, cm, om, am, lm, pm, gm, Sm, vy, Pd, By, tm, Qf, Id, Ny);
  const oc = {
    loader: ct.LoadParser,
    resolver: ct.ResolveParser,
    cache: ct.CacheParser,
    detection: ct.DetectionParser
  };
  Ut.handle(ct.Asset, (n) => {
    const t = n.ref;
    Object.entries(oc).filter(([e]) => !!t[e]).forEach(([e, s]) => Ut.add(Object.assign(t[e], {
      extension: t[e].extension ?? s
    })));
  }, (n) => {
    const t = n.ref;
    Object.keys(oc).filter((e) => !!t[e]).forEach((e) => Ut.remove(t[e]));
  });
  class Wy {
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
  class Gy {
    constructor() {
      this.instructions = new fa();
    }
    init() {
      this.instructions.reset();
    }
    destroy() {
      this.instructions.destroy(), this.instructions = null;
    }
  }
  const Ma = class Xo {
    constructor(t) {
      this._renderer = t, this._managedContexts = new Bs({
        renderer: t,
        type: "resource",
        name: "graphicsContext"
      });
    }
    init(t) {
      Xo.defaultOptions.bezierSmoothness = (t == null ? void 0 : t.bezierSmoothness) ?? Xo.defaultOptions.bezierSmoothness;
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
      const e = new Gy(), s = this.getGpuContext(t);
      return s.graphicsData = e, e.init(), e;
    }
    _initContext(t) {
      const e = new Wy();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  Ma.extension = {
    type: [
      ct.CanvasSystem
    ],
    name: "graphicsContext"
  };
  Ma.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let zy = Ma;
  class Rd {
    constructor(t, e) {
      this.state = or.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Bs({
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
  Rd.extension = {
    type: [
      ct.CanvasPipes
    ],
    name: "graphics"
  };
  Ld = function(n, t, e) {
    const s = (n >> 24 & 255) / 255;
    t[e++] = (n & 255) / 255 * s, t[e++] = (n >> 8 & 255) / 255 * s, t[e++] = (n >> 16 & 255) / 255 * s, t[e++] = s;
  };
  class Dy {
    constructor() {
      this.batches = [], this.batched = false;
    }
    destroy() {
      this.batches.forEach((t) => {
        Me.return(t);
      }), this.batches.length = 0;
    }
  }
  class $d {
    constructor(t, e) {
      this.state = or.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Bs({
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
      o.uTransformMatrix = t.groupTransform, o.uRound = e._roundPixels | t._roundPixels, Ld(t.groupColorAlpha, o.uColor, 0), this._adaptor.execute(this, t);
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
      const e = new Dy();
      return t._gpuData[this.renderer.uid] = e, this._managedGraphics.add(t), e;
    }
    _updateBatchesForRenderable(t, e) {
      const s = t.context, r = this.renderer.graphicsContext.getGpuContext(s), o = this.renderer._roundPixels | t._roundPixels;
      e.batches = r.batches.map((a) => {
        const l = Me.get(va);
        return a.copyTo(l), l.renderable = t, l.roundPixels = o, l;
      });
    }
    destroy() {
      this._managedGraphics.destroy(), this.renderer = null, this._adaptor.destroy(), this._adaptor = null, this.state = null;
    }
  }
  $d.extension = {
    type: [
      ct.WebGLPipes,
      ct.WebGPUPipes
    ],
    name: "graphics"
  };
  Ut.add(Rd);
  Ut.add($d);
  Ut.add(zy);
  Ut.add(Ea);
  pt = class extends yr {
    constructor(t) {
      t instanceof Be && (t = {
        context: t
      });
      const { context: e, roundPixels: s, ...i } = t || {};
      super({
        label: "Graphics",
        ...i
      }), this.renderPipeId = "graphics", e ? this.context = e : (this.context = this._ownedContext = new Be(), this.context.autoGarbageCollect = this.autoGarbageCollect), this.didViewUpdate = true, this.allowChildren = false, this.roundPixels = s ?? false;
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
      return t ? new pt(this._context.clone()) : (this._ownedContext = null, new pt(this._context));
    }
    lineStyle(t, e, s) {
      Ot(se, "Graphics#lineStyle is no longer needed. Use Graphics#setStrokeStyle to set the stroke style.");
      const i = {};
      return t && (i.width = t), e && (i.color = e), s && (i.alpha = s), this.context.strokeStyle = i, this;
    }
    beginFill(t, e) {
      Ot(se, "Graphics#beginFill is no longer needed. Use Graphics#fill to fill the shape with the desired style.");
      const s = {};
      return t !== void 0 && (s.color = t), e !== void 0 && (s.alpha = e), this.context.fillStyle = s, this;
    }
    endFill() {
      Ot(se, "Graphics#endFill is no longer needed. Use Graphics#fill to fill the shape with the desired style."), this.context.fill();
      const t = this.context.strokeStyle;
      return (t.width !== Be.defaultStrokeStyle.width || t.color !== Be.defaultStrokeStyle.color || t.alpha !== Be.defaultStrokeStyle.alpha) && this.context.stroke(), this;
    }
    drawCircle(...t) {
      return Ot(se, "Graphics#drawCircle has been renamed to Graphics#circle"), this._callContextMethod("circle", t);
    }
    drawEllipse(...t) {
      return Ot(se, "Graphics#drawEllipse has been renamed to Graphics#ellipse"), this._callContextMethod("ellipse", t);
    }
    drawPolygon(...t) {
      return Ot(se, "Graphics#drawPolygon has been renamed to Graphics#poly"), this._callContextMethod("poly", t);
    }
    drawRect(...t) {
      return Ot(se, "Graphics#drawRect has been renamed to Graphics#rect"), this._callContextMethod("rect", t);
    }
    drawRoundedRect(...t) {
      return Ot(se, "Graphics#drawRoundedRect has been renamed to Graphics#roundRect"), this._callContextMethod("roundRect", t);
    }
    drawStar(...t) {
      return Ot(se, "Graphics#drawStar has been renamed to Graphics#star"), this._callContextMethod("star", t);
    }
  };
  let ds;
  function ac(n) {
    const t = Bt.get().createCanvas(6, 1), e = t.getContext("2d");
    return e.fillStyle = n, e.fillRect(0, 0, 6, 1), t;
  }
  Hy = function() {
    if (ds !== void 0) return ds;
    try {
      const n = ac("#ff00ff"), t = ac("#ffff00"), s = Bt.get().createCanvas(6, 1).getContext("2d");
      s.globalCompositeOperation = "multiply", s.drawImage(n, 0, 0), s.drawImage(t, 2, 0);
      const i = s.getImageData(2, 0, 1, 1);
      if (!i) ds = false;
      else {
        const r = i.data;
        ds = r[0] === 255 && r[1] === 0 && r[2] === 0;
      }
    } catch {
      ds = false;
    }
    return ds;
  };
  Zt = {
    canvas: null,
    convertTintToImage: false,
    cacheStepsPerColorChannel: 8,
    canUseMultiply: Hy(),
    tintMethod: null,
    _canvasSourceCache: /* @__PURE__ */ new WeakMap(),
    _unpremultipliedCache: /* @__PURE__ */ new WeakMap(),
    getCanvasSource: (n) => {
      const t = n.source, e = t == null ? void 0 : t.resource;
      if (!e) return null;
      const s = t.alphaMode === "premultiplied-alpha", i = t.resourceWidth ?? t.pixelWidth, r = t.resourceHeight ?? t.pixelHeight, o = i !== t.pixelWidth || r !== t.pixelHeight;
      if (s) {
        if ((e instanceof HTMLCanvasElement || typeof OffscreenCanvas < "u" && e instanceof OffscreenCanvas) && !o) return e;
        const a = Zt._unpremultipliedCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
      }
      if (e instanceof Uint8Array || e instanceof Uint8ClampedArray || e instanceof Int8Array || e instanceof Uint16Array || e instanceof Int16Array || e instanceof Uint32Array || e instanceof Int32Array || e instanceof Float32Array || e instanceof ArrayBuffer) {
        const a = Zt._canvasSourceCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
        const l = Bt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d"), h = c.createImageData(t.pixelWidth, t.pixelHeight), d = h.data, u = e instanceof ArrayBuffer ? new Uint8Array(e) : new Uint8Array(e.buffer, e.byteOffset, e.byteLength);
        if (t.format === "bgra8unorm") for (let p = 0; p < d.length && p + 3 < u.length; p += 4) d[p] = u[p + 2], d[p + 1] = u[p + 1], d[p + 2] = u[p], d[p + 3] = u[p + 3];
        else d.set(u.subarray(0, d.length));
        return c.putImageData(h, 0, 0), Zt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      if (s) {
        const a = Bt.get().createCanvas(t.pixelWidth, t.pixelHeight), l = a.getContext("2d", {
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
        return l.putImageData(c, 0, 0), Zt._unpremultipliedCache.set(t, {
          canvas: a,
          resourceId: t._resourceId
        }), a;
      }
      if (o) {
        const a = Zt._canvasSourceCache.get(t);
        if ((a == null ? void 0 : a.resourceId) === t._resourceId) return a.canvas;
        const l = Bt.get().createCanvas(t.pixelWidth, t.pixelHeight), c = l.getContext("2d");
        return l.width = t.pixelWidth, l.height = t.pixelHeight, c.drawImage(e, 0, 0), Zt._canvasSourceCache.set(t, {
          canvas: l,
          resourceId: t._resourceId
        }), l;
      }
      return e;
    },
    getTintedCanvas: (n, t) => {
      const e = n.texture, s = Jt.shared.setValue(t).toHex(), i = e.tintCache || (e.tintCache = {}), r = i[s], o = e.source._resourceId;
      if ((r == null ? void 0 : r.tintId) === o) return r;
      const a = r && "getContext" in r ? r : Bt.get().createCanvas();
      return Zt.tintMethod(e, t, a), a.tintId = o, i[s] = a, i[s];
    },
    getTintedPattern: (n, t) => {
      const e = Jt.shared.setValue(t).toHex(), s = n.patternCache || (n.patternCache = {}), i = n.source._resourceId;
      let r = s[e];
      return (r == null ? void 0 : r.tintId) === i || (Zt.canvas || (Zt.canvas = Bt.get().createCanvas()), Zt.tintMethod(n, t, Zt.canvas), r = Zt.canvas.getContext("2d").createPattern(Zt.canvas, "repeat"), r.tintId = i, s[e] = r), r;
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
      const a = Pt.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.fillStyle = Jt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "multiply";
      const h = Zt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && Zt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.globalCompositeOperation = "destination-atop", s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
    },
    tintWithOverlay: (n, t, e) => {
      const s = e.getContext("2d"), i = n.frame.clone(), r = n.source._resolution ?? n.source.resolution ?? 1, o = n.rotate;
      i.x *= r, i.y *= r, i.width *= r, i.height *= r;
      const a = Pt.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy", s.fillStyle = Jt.shared.setValue(t).toHex(), s.fillRect(0, 0, l, c), s.globalCompositeOperation = "destination-atop";
      const h = Zt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && Zt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
    },
    tintWithPerPixel: (n, t, e) => {
      const s = e.getContext("2d"), i = n.frame.clone(), r = n.source._resolution ?? n.source.resolution ?? 1, o = n.rotate;
      i.x *= r, i.y *= r, i.width *= r, i.height *= r;
      const a = Pt.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
      e.width = Math.ceil(l), e.height = Math.ceil(c), s.save(), s.globalCompositeOperation = "copy";
      const h = Zt.getCanvasSource(n);
      if (!h) {
        s.restore();
        return;
      }
      o && Zt._applyInverseRotation(s, o, i.width, i.height), s.drawImage(h, i.x, i.y, i.width, i.height, 0, 0, i.width, i.height), s.restore();
      const d = t >> 16 & 255, u = t >> 8 & 255, p = t & 255, f = s.getImageData(0, 0, l, c), g = f.data;
      for (let m = 0; m < g.length; m += 4) g[m] = g[m] * d / 255, g[m + 1] = g[m + 1] * u / 255, g[m + 2] = g[m + 2] * p / 255;
      s.putImageData(f, 0, 0);
    },
    _applyInverseRotation: (n, t, e, s) => {
      const i = Pt.inv(t), r = Pt.uX(i), o = Pt.uY(i), a = Pt.vX(i), l = Pt.vY(i), c = -Math.min(0, r * e, a * s, r * e + a * s), h = -Math.min(0, o * e, l * s, o * e + l * s);
      n.transform(r, o, a, l, c, h);
    }
  };
  Zt.tintMethod = Zt.canUseMultiply ? Zt.tintWithMultiply : Zt.tintWithPerPixel;
  class Uy extends yr {
    constructor(t, e) {
      const { text: s, resolution: i, style: r, anchor: o, width: a, height: l, roundPixels: c, ...h } = t;
      super({
        ...h
      }), this.batched = true, this._resolution = null, this._autoResolution = true, this._didTextUpdate = true, this._styleClass = e, this.text = s ?? "", this.style = r, this.resolution = i ?? null, this.allowChildren = false, this._anchor = new ge({
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
  function jy(n, t) {
    let e = n[0] ?? {};
    return (typeof e == "string" || n[1]) && (Ot(se, `use new ${t}({ text: "hi!", style }) instead`), e = {
      text: e,
      style: n[1]
    }), e;
  }
  class Yy {
    constructor(t) {
      this._canvasPool = /* @__PURE__ */ Object.create(null), this.canvasOptions = t || {}, this.enableFullScreen = false;
    }
    _createCanvasAndContext(t, e) {
      const s = Bt.get().createCanvas();
      s.width = t, s.height = e;
      const i = s.getContext("2d");
      return {
        canvas: s,
        context: i
      };
    }
    getOptimalCanvasAndContext(t, e, s = 1) {
      t = Math.ceil(t * s - 1e-6), e = Math.ceil(e * s - 1e-6), t = As(t), e = As(e);
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
  qo = new Yy();
  wi.register(qo);
  let zn = null, gn = null;
  function Vy(n, t) {
    zn || (zn = Bt.get().createCanvas(256, 128), gn = zn.getContext("2d", {
      willReadFrequently: true
    }), gn.globalCompositeOperation = "copy", gn.globalAlpha = 1), (zn.width < n || zn.height < t) && (zn.width = As(n), zn.height = As(t));
  }
  function lc(n, t, e) {
    for (let s = 0, i = 4 * e * t; s < t; ++s, i += 4) if (n[i + 3] !== 0) return false;
    return true;
  }
  function cc(n, t, e, s, i) {
    const r = 4 * t;
    for (let o = s, a = s * r + 4 * e; o <= i; ++o, a += r) if (n[a + 3] !== 0) return false;
    return true;
  }
  function Xy(...n) {
    let t = n[0];
    t.canvas || (t = {
      canvas: n[0],
      resolution: n[1]
    });
    const { canvas: e } = t, s = Math.min(t.resolution ?? 1, 1), i = t.width ?? e.width, r = t.height ?? e.height;
    let o = t.output;
    if (Vy(i, r), !gn) throw new TypeError("Failed to get canvas 2D context");
    gn.drawImage(e, 0, 0, i, r, 0, 0, i * s, r * s);
    const l = gn.getImageData(0, 0, i, r).data;
    let c = 0, h = 0, d = i - 1, u = r - 1;
    for (; h < r && lc(l, i, h); ) ++h;
    if (h === r) return Yt.EMPTY;
    for (; lc(l, i, u); ) --u;
    for (; cc(l, i, c, h, u); ) ++c;
    for (; cc(l, i, d, h, u); ) --d;
    return ++d, ++u, gn.globalCompositeOperation = "source-over", gn.strokeRect(c, h, d - c, u - h), gn.globalCompositeOperation = "copy", o ?? (o = new Yt()), o.set(c / s, h / s, (d - c) / s, (u - h) / s), o;
  }
  class qy {
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
  Ky = function(n = 1e3, t = 0, e = false) {
    if (isNaN(n) || n < 0) throw new TypeError("Invalid max value");
    if (isNaN(t) || t < 0) throw new TypeError("Invalid ttl value");
    if (typeof e != "boolean") throw new TypeError("Invalid resetTtl value");
    return new qy(n, t, e);
  };
  function Od(n) {
    return !!n.tagStyles && Object.keys(n.tagStyles).length > 0;
  }
  function Bd(n) {
    return n.includes("<");
  }
  function Jy(n, t) {
    return n.clone().assign(t);
  }
  function Zy(n, t) {
    const e = [], s = t.tagStyles;
    if (!Od(t) || !Bd(n)) return e.push({
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
            const u = i[i.length - 1], p = Jy(u, s[d]);
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
  const Qy = [
    10,
    13
  ], tx = new Set(Qy), ex = [
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
  ], nx = new Set(ex), sx = [
    9,
    32
  ], ix = new Set(sx), rx = [
    45,
    8208,
    8211,
    8212,
    173
  ], ox = new Set(rx), ax = /(\r\n|\r|\n)/, lx = /(?:\r\n|\r|\n)/;
  function cr(n) {
    return typeof n != "string" ? false : tx.has(n.charCodeAt(0));
  }
  We = function(n, t) {
    return typeof n != "string" ? false : nx.has(n.charCodeAt(0));
  };
  p1 = function(n) {
    return typeof n != "string" ? false : ix.has(n.charCodeAt(0));
  };
  cx = function(n) {
    return typeof n != "string" ? false : ox.has(n.charCodeAt(0));
  };
  Nd = function(n) {
    return n === "normal" || n === "pre-line";
  };
  Fd = function(n) {
    return n === "normal";
  };
  function pn(n) {
    if (typeof n != "string") return "";
    let t = n.length - 1;
    for (; t >= 0 && We(n[t]); ) t--;
    return t < n.length - 1 ? n.slice(0, t + 1) : n;
  }
  function Wd(n) {
    const t = [], e = [];
    if (typeof n != "string") return t;
    for (let s = 0; s < n.length; s++) {
      const i = n[s], r = n[s + 1];
      if (We(i) || cr(i)) {
        e.length > 0 && (t.push(e.join("")), e.length = 0), i === "\r" && r === `
` ? (t.push(`\r
`), s++) : t.push(i);
        continue;
      }
      e.push(i), cx(i) && r && !We(r) && !cr(r) && (t.push(e.join("")), e.length = 0);
    }
    return e.length > 0 && t.push(e.join("")), t;
  }
  function Gd(n, t, e, s) {
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
  const hx = /\r\n|\r|\n/g;
  function dx(n, t, e, s, i, r, o, a) {
    var _a2, _b2;
    const l = Zy(n, t);
    if (Fd(t.whiteSpace)) for (let z = 0; z < l.length; z++) {
      const P = l[z];
      l[z] = {
        text: P.text.replace(hx, " "),
        style: P.style
      };
    }
    const h = [];
    let d = [];
    for (const z of l) {
      const P = z.text.split(ax);
      for (let D = 0; D < P.length; D++) {
        const J = P[D];
        J === `\r
` || J === "\r" || J === `
` ? (h.push(d), d = []) : J.length > 0 && d.push({
          text: J,
          style: z.style
        });
      }
    }
    (d.length > 0 || h.length === 0) && h.push(d);
    const u = e ? ux(h, t, s, i, o, a) : h, p = [], f = [], g = [], m = [], y = [];
    let b = 0;
    const x = t._fontString, _ = r(x);
    _.fontSize === 0 && (_.fontSize = t.fontSize, _.ascent = t.fontSize);
    let v = "", w = !!t.dropShadow, C = ((_a2 = t._stroke) == null ? void 0 : _a2.width) || 0;
    for (const z of u) {
      let P = 0, D = _.ascent, J = _.descent, Y = "";
      for (const I of z) {
        const N = I.style._fontString, O = r(N);
        N !== v && (s.font = N, v = N);
        const B = i(I.text, I.style.letterSpacing, s);
        P += B, D = Math.max(D, O.ascent), J = Math.max(J, O.descent), Y += I.text;
        const q = ((_b2 = I.style._stroke) == null ? void 0 : _b2.width) || 0;
        q > C && (C = q), !w && I.style.dropShadow && (w = true);
      }
      z.length === 0 && (D = _.ascent, J = _.descent), p.push(P), f.push(D), g.push(J), y.push(Y);
      const K = t.lineHeight || D + J;
      m.push(K + t.leading), b = Math.max(b, P);
    }
    const T = C, A = (e && t.align !== "left" ? Math.max(b, t.wordWrapWidth) : b) + T + (t.dropShadow ? t.dropShadow.distance : 0);
    let W = 0;
    for (let z = 0; z < m.length; z++) W += m[z];
    W = Math.max(W, m[0] + T);
    const j = W + (t.dropShadow ? t.dropShadow.distance : 0), G = t.lineHeight || _.fontSize;
    return {
      width: A,
      height: j,
      lines: y,
      lineWidths: p,
      lineHeight: G + t.leading,
      maxLineWidth: b,
      fontProperties: _,
      runsByLine: u,
      lineAscents: f,
      lineDescents: g,
      lineHeights: m,
      hasDropShadow: w
    };
  }
  function ux(n, t, e, s, i, r) {
    var _a2;
    const { letterSpacing: o, whiteSpace: a, wordWrapWidth: l, breakWords: c } = t, h = Nd(a), d = l + o, u = {};
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
      const y = px(m), b = g.length, x = (A) => {
        let W = 0, j = A;
        do {
          const { token: G, style: z } = y[j];
          W += f(G, z), j++;
        } while (j < y.length && y[j].continuesFromPrevious);
        return W;
      }, _ = (A) => {
        const W = [];
        let j = A;
        do
          W.push({
            token: y[j].token,
            style: y[j].style
          }), j++;
        while (j < y.length && y[j].continuesFromPrevious);
        return W;
      };
      let v = [], w = 0, C = !h, T = null;
      const R = () => {
        T && T.text.length > 0 && v.push(T), T = null;
      }, E = () => {
        if (R(), v.length > 0) {
          const A = v[v.length - 1];
          A.text = pn(A.text), A.text.length === 0 && v.pop();
        }
        g.push(v), v = [], w = 0, C = false;
      };
      for (let A = 0; A < y.length; A++) {
        const { token: W, style: j, continuesFromPrevious: G } = y[A], z = f(W, j);
        if (h) {
          const J = We(W), Y = (T == null ? void 0 : T.text[T.text.length - 1]) ?? ((_a2 = v[v.length - 1]) == null ? void 0 : _a2.text.slice(-1)) ?? "", K = Y ? We(Y) : false;
          if (J && K) continue;
        }
        const P = !G, D = P ? x(A) : z;
        if (D > d && P) if (w > 0 && E(), c) {
          const J = _(A);
          for (let Y = 0; Y < J.length; Y++) {
            const K = J[Y].token, I = J[Y].style, N = Gd(K, c, r, i);
            for (const O of N) {
              const B = f(O, I);
              B + w > d && E(), !T || T.style !== I ? (R(), T = {
                text: O,
                style: I
              }) : T.text += O, w += B;
            }
          }
          A += J.length - 1;
        } else {
          const J = _(A);
          R(), g.push(J.map((Y) => ({
            text: Y.token,
            style: Y.style
          }))), C = false, A += J.length - 1;
        }
        else if (D + w > d && P) {
          if (We(W)) {
            C = false;
            continue;
          }
          E(), T = {
            text: W,
            style: j
          }, w = z;
        } else if (G && !c) !T || T.style !== j ? (R(), T = {
          text: W,
          style: j
        }) : T.text += W, w += z;
        else {
          const J = We(W);
          if (w === 0 && J && !C) continue;
          !T || T.style !== j ? (R(), T = {
            text: W,
            style: j
          }) : T.text += W, w += z;
        }
      }
      if (R(), v.length > 0) {
        const A = v[v.length - 1];
        A.text = pn(A.text), A.text.length === 0 && v.pop();
      }
      (v.length > 0 || g.length === b) && g.push(v);
    }
    return g;
  }
  function px(n) {
    const t = [];
    let e = false;
    for (const s of n) {
      const i = Wd(s.text);
      let r = true;
      for (const o of i) {
        const a = We(o) || cr(o), l = r && e && !a;
        t.push({
          token: o,
          style: s.style,
          continuesFromPrevious: l
        }), e = !a, r = false;
      }
    }
    return t;
  }
  const fx = {
    willReadFrequently: true
  };
  function hc(n, t, e, s, i) {
    let r = e[n];
    return typeof r != "number" && (r = i(n, t, s) + t, e[n] = r), r;
  }
  function mx(n, t, e, s, i, r, o) {
    const a = e.getContext("2d", fx);
    a.font = t._fontString;
    let l = 0, c = "";
    const h = [], d = /* @__PURE__ */ Object.create(null), { letterSpacing: u, whiteSpace: p } = t, f = Nd(p), g = Fd(p);
    let m = !f;
    const y = t.wordWrapWidth + u, b = Wd(n);
    for (let _ = 0; _ < b.length; _++) {
      let v = b[_];
      if (cr(v)) {
        if (!g) {
          h.push(pn(c)), m = !f, c = "", l = 0;
          continue;
        }
        v = " ";
      }
      if (f) {
        const C = We(v), T = We(c[c.length - 1]);
        if (C && T) continue;
      }
      const w = hc(v, u, d, a, s);
      if (w > y) if (c !== "" && (h.push(pn(c)), c = "", l = 0), i(v, t.breakWords)) {
        const C = Gd(v, t.breakWords, o, r);
        for (const T of C) {
          const R = hc(T, u, d, a, s);
          R + l > y && (h.push(pn(c)), m = false, c = "", l = 0), c += T, l += R;
        }
      } else c.length > 0 && (h.push(pn(c)), c = "", l = 0), h.push(pn(v)), m = false, c = "", l = 0;
      else w + l > y && (m = false, h.push(pn(c)), c = "", l = 0), (c.length > 0 || !We(v) || m) && (c += v, l += w);
    }
    const x = pn(c);
    return x.length > 0 && h.push(x), h.join(`
`);
  }
  const dc = {
    willReadFrequently: true
  }, Cn = class _t {
    static get experimentalLetterSpacingSupported() {
      let t = _t._experimentalLetterSpacingSupported;
      if (t === void 0) {
        const e = Bt.get().getCanvasRenderingContext2D().prototype;
        t = _t._experimentalLetterSpacingSupported = "letterSpacing" in e || "textLetterSpacing" in e;
      }
      return t;
    }
    constructor(t, e, s, i, r, o, a, l, c, h) {
      this.text = t, this.style = e, this.width = s, this.height = i, this.lines = r, this.lineWidths = o, this.lineHeight = a, this.maxLineWidth = l, this.fontProperties = c, h && (this.runsByLine = h.runsByLine, this.lineAscents = h.lineAscents, this.lineDescents = h.lineDescents, this.lineHeights = h.lineHeights, this.hasDropShadow = h.hasDropShadow);
    }
    static measureText(t = " ", e, s = _t._canvas, i = e.wordWrap) {
      var _a2;
      const r = `${t}-${e.styleKey}-wordWrap-${i}`;
      if (_t._measurementCache.has(r)) return _t._measurementCache.get(r);
      if (Od(e) && Bd(t)) {
        const v = dx(t, e, i, _t._context, _t._measureText, _t.measureFont, _t.canBreakChars, _t.wordWrapSplit), w = new _t(t, e, v.width, v.height, v.lines, v.lineWidths, v.lineHeight, v.maxLineWidth, v.fontProperties, {
          runsByLine: v.runsByLine,
          lineAscents: v.lineAscents,
          lineDescents: v.lineDescents,
          lineHeights: v.lineHeights,
          hasDropShadow: v.hasDropShadow
        });
        return _t._measurementCache.set(r, w), w;
      }
      const a = e._fontString, l = _t.measureFont(a);
      l.fontSize === 0 && (l.fontSize = e.fontSize, l.ascent = e.fontSize, l.descent = 0);
      const c = _t._context;
      c.font = a;
      const d = (i ? _t._wordWrap(t, e, s) : t).split(lx), u = new Array(d.length);
      let p = 0;
      for (let v = 0; v < d.length; v++) {
        const w = _t._measureText(d[v], e.letterSpacing, c);
        u[v] = w, p = Math.max(p, w);
      }
      const f = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0, g = e.lineHeight || l.fontSize, m = _t._getAlignWidth(p, e, i), y = _t._adjustWidthForStyle(m, e), b = Math.max(g, l.fontSize + f) + (d.length - 1) * (g + e.leading), x = _t._adjustHeightForStyle(b, e), _ = new _t(t, e, y, x, d, u, g + e.leading, p, l);
      return _t._measurementCache.set(r, _), _;
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
      _t.experimentalLetterSpacingSupported && (_t.experimentalLetterSpacing ? (s.letterSpacing = `${e}px`, s.textLetterSpacing = `${e}px`, i = true) : (s.letterSpacing = "0px", s.textLetterSpacing = "0px"));
      const r = s.measureText(t);
      let o = r.width;
      const a = -(r.actualBoundingBoxLeft ?? 0);
      let c = (r.actualBoundingBoxRight ?? 0) - a;
      if (o > 0) if (i) o -= e, c -= e;
      else {
        const h = (_t.graphemeSegmenter(t).length - 1) * e;
        o += h, c += h;
      }
      return Math.max(o, c);
    }
    static _wordWrap(t, e, s = _t._canvas) {
      return mx(t, e, s, _t._measureText, _t.canBreakWords, _t.canBreakChars, _t.wordWrapSplit);
    }
    static isBreakingSpace(t, e) {
      return We(t);
    }
    static canBreakWords(t, e) {
      return e;
    }
    static canBreakChars(t, e, s, i, r) {
      return true;
    }
    static wordWrapSplit(t) {
      return _t.graphemeSegmenter(t);
    }
    static measureFont(t) {
      if (_t._fonts[t]) return _t._fonts[t];
      const e = _t._context;
      e.font = t;
      const s = e.measureText(_t.METRICS_STRING + _t.BASELINE_SYMBOL), i = s.actualBoundingBoxAscent ?? 0, r = s.actualBoundingBoxDescent ?? 0, o = {
        ascent: i,
        descent: r,
        fontSize: i + r
      };
      return _t._fonts[t] = o, o;
    }
    static clearMetrics(t = "") {
      t ? delete _t._fonts[t] : _t._fonts = {};
    }
    static get _canvas() {
      var _a2;
      if (!_t.__canvas) {
        let t;
        try {
          const e = new OffscreenCanvas(0, 0);
          if ((_a2 = e.getContext("2d", dc)) == null ? void 0 : _a2.measureText) return _t.__canvas = e, e;
          t = Bt.get().createCanvas();
        } catch {
          t = Bt.get().createCanvas();
        }
        t.width = t.height = 10, _t.__canvas = t;
      }
      return _t.__canvas;
    }
    static get _context() {
      return _t.__context || (_t.__context = _t._canvas.getContext("2d", dc)), _t.__context;
    }
  };
  Cn.METRICS_STRING = "|\xC9q\xC5";
  Cn.BASELINE_SYMBOL = "M";
  Cn.BASELINE_MULTIPLIER = 1.4;
  Cn.HEIGHT_MULTIPLIER = 2;
  Cn.graphemeSegmenter = (() => {
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
  Cn.experimentalLetterSpacing = false;
  Cn._fonts = {};
  Cn._measurementCache = Ky(1e3);
  fn = Cn;
  const gx = [
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui"
  ];
  Ko = function(n) {
    const t = typeof n.fontSize == "number" ? `${n.fontSize}px` : n.fontSize;
    let e = n.fontFamily;
    Array.isArray(n.fontFamily) || (e = n.fontFamily.split(","));
    for (let s = e.length - 1; s >= 0; s--) {
      let i = e[s].trim();
      !/([\"\'])[^\'\"]+\1/.test(i) && !gx.includes(i) && (i = `"${i}"`), e[s] = i;
    }
    return `${n.fontStyle} ${n.fontVariant} ${n.fontWeight} ${t} ${e.join(",")}`;
  };
  const uc = 1e5;
  Wi = function(n, t, e, s = 0, i = 0, r = 0) {
    if (n.texture === At.WHITE && !n.fill) return Jt.shared.setValue(n.color).setAlpha(n.alpha ?? 1).toHexa();
    if (n.fill) {
      if (n.fill instanceof vr) {
        const o = n.fill, a = t.createPattern(o.texture.source.resource, "repeat"), l = o.transform.copyTo(Ct.shared);
        return l.scale(o.texture.source.pixelWidth, o.texture.source.pixelHeight), a.setTransform(l), a;
      } else if (n.fill instanceof vn) {
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
              y = Math.max(0, Math.min(1, y)), d.addColorStop(Math.floor(y * uc) / uc, Jt.shared.setValue(m.color).toHex());
            });
          }
        } else o.colorStops.forEach((p) => {
          d.addColorStop(p.offset, Jt.shared.setValue(p.color).toHex());
        });
        return d;
      }
    } else {
      const o = t.createPattern(n.texture.source.resource, "repeat"), a = n.matrix.copyTo(Ct.shared);
      return a.scale(n.texture.source.pixelWidth, n.texture.source.pixelHeight), o.setTransform(a), o;
    }
    return Qt("FillStyle not recognised", n), "red";
  };
  const pc = new Yt();
  function us(n) {
    let t = 0;
    for (let e = 0; e < n.length; e++) n.charCodeAt(e) === 32 && t++;
    return t;
  }
  class yx {
    getCanvasAndContext(t) {
      const { text: e, style: s, resolution: i = 1 } = t, r = s._getFinalPadding(), o = fn.measureText(e || " ", s), a = Math.ceil(Math.ceil(Math.max(1, o.width) + r * 2) * i), l = Math.ceil(Math.ceil(Math.max(1, o.height) + r * 2) * i), c = qo.getOptimalCanvasAndContext(a, l);
      this._renderTextToCanvas(s, r, i, c, o);
      const h = s.trim ? Xy({
        canvas: c.canvas,
        width: a,
        height: l,
        resolution: 1,
        output: pc
      }) : pc.set(0, 0, a, l);
      return {
        canvasAndContext: c,
        frame: h
      };
    }
    returnCanvasAndContext(t) {
      qo.returnCanvasAndContext(t);
    }
    _renderTextToCanvas(t, e, s, i, r) {
      var _a2, _b2, _c2;
      if (r.runsByLine && r.runsByLine.length > 0) {
        this._renderTaggedTextToCanvas(r, t, e, s, i);
        return;
      }
      const { canvas: o, context: a } = i, l = Ko(t), c = r.lines, h = r.lineHeight, d = r.lineWidths, u = r.maxLineWidth, p = r.fontProperties, f = o.height;
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
        const C = t.dropShadow && w === 0, T = C ? Math.ceil(Math.max(1, f) + e * 2) : 0, R = T * s;
        if (C) this._setupDropShadow(a, t, s, R);
        else {
          const E = t._gradientBounds, A = t._gradientOffset;
          if (E) {
            const W = {
              width: E.width,
              height: E.height,
              lineHeight: E.height,
              lines: r.lines
            };
            this._setFillAndStrokeStyles(a, t, W, e, _, (A == null ? void 0 : A.x) ?? 0, (A == null ? void 0 : A.y) ?? 0);
          } else A ? this._setFillAndStrokeStyles(a, t, r, e, _, A.x, A.y) : this._setFillAndStrokeStyles(a, t, r, e, _);
          a.shadowColor = "rgba(0,0,0,0)";
        }
        for (let E = 0; E < c.length; E++) {
          g = _, m = _ + E * h + p.ascent + v, g += this._getAlignmentOffset(d[E], b, t.align);
          let A = 0;
          if (t.align === "justify" && t.wordWrap && E < c.length - 1) {
            const W = us(c[E]);
            W > 0 && (A = (b - d[E]) / W);
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
          const T = Ko(C.style);
          a.font = T, w.push({
            width: fn._measureText(C.text, C.style.letterSpacing, a),
            font: T
          });
        }
        x.push(w);
      }
      for (let _ = 0; _ < g; ++_) {
        const v = p && _ === 0, w = v ? Math.ceil(Math.max(1, f) + s * 2) : 0, C = w * i;
        v || (a.shadowColor = "rgba(0,0,0,0)");
        let T = b;
        for (let R = 0; R < l.length; R++) {
          const E = l[R], A = c[R], W = d[R], j = u[R], G = x[R];
          let z = b;
          z += this._getAlignmentOffset(A, m, e.align);
          let P = 0;
          if (e.align === "justify" && e.wordWrap && R < l.length - 1) {
            let Y = 0;
            for (const K of E) Y += us(K.text);
            Y > 0 && (P = (m - A) / Y);
          }
          const D = T + W;
          let J = z + s;
          for (let Y = 0; Y < E.length; Y++) {
            const K = E[Y], { width: I, font: N } = G[Y];
            if (a.font = N, a.textBaseline = K.style.textBaseline, (_c2 = K.style._stroke) == null ? void 0 : _c2.width) {
              const B = K.style._stroke;
              if (a.lineWidth = B.width, a.miterLimit = B.miterLimit, a.lineJoin = B.join, a.lineCap = B.cap, v) if (K.style.dropShadow) this._setupDropShadow(a, K.style, i, C);
              else {
                const q = us(K.text);
                J += I + q * P;
                continue;
              }
              else {
                const q = fn.measureFont(N), Q = K.style.lineHeight || q.fontSize, ut = {
                  width: I,
                  height: Q,
                  lineHeight: Q,
                  lines: [
                    K.text
                  ]
                };
                a.strokeStyle = Wi(B, a, ut, s * 2, J - s, T);
              }
              this._drawLetterSpacing(K.text, K.style, r, J, D + s - w, true, P);
            }
            const O = us(K.text);
            J += I + O * P;
          }
          J = z + s;
          for (let Y = 0; Y < E.length; Y++) {
            const K = E[Y], { width: I, font: N } = G[Y];
            if (a.font = N, a.textBaseline = K.style.textBaseline, K.style._fill !== void 0) {
              if (v) if (K.style.dropShadow) this._setupDropShadow(a, K.style, i, C);
              else {
                const B = us(K.text);
                J += I + B * P;
                continue;
              }
              else {
                const B = fn.measureFont(N), q = K.style.lineHeight || B.fontSize, Q = {
                  width: I,
                  height: q,
                  lineHeight: q,
                  lines: [
                    K.text
                  ]
                };
                a.fillStyle = Wi(K.style._fill, a, Q, s * 2, J - s, T);
              }
              this._drawLetterSpacing(K.text, K.style, r, J, D + s - w, false, P);
            }
            const O = us(K.text);
            J += I + O * P;
          }
          T += j;
        }
      }
    }
    _setFillAndStrokeStyles(t, e, s, i, r, o = 0, a = 0) {
      var _a2;
      if (t.fillStyle = e._fill ? Wi(e._fill, t, s, i * 2, o, a) : null, (_a2 = e._stroke) == null ? void 0 : _a2.width) {
        const l = r + i * 2;
        t.strokeStyle = Wi(e._stroke, t, s, l, o, a);
      }
    }
    _setupDropShadow(t, e, s, i) {
      t.fillStyle = "black", t.strokeStyle = "black";
      const r = e.dropShadow, o = r.color, a = r.alpha;
      t.shadowColor = Jt.shared.setValue(o).setAlpha(a).toRgbaString();
      const l = r.blur * s, c = r.distance * s;
      t.shadowBlur = l, t.shadowOffsetX = Math.cos(r.angle) * c, t.shadowOffsetY = Math.sin(r.angle) * c + i;
    }
    _getAlignmentOffset(t, e, s) {
      return s === "right" ? e - t : s === "center" ? (e - t) / 2 : 0;
    }
    _drawLetterSpacing(t, e, s, i, r, o = false, a = 0) {
      const { context: l } = s, c = e.letterSpacing;
      let h = false;
      if (fn.experimentalLetterSpacingSupported && (fn.experimentalLetterSpacing ? (l.letterSpacing = `${c}px`, l.textLetterSpacing = `${c}px`, h = true) : (l.letterSpacing = "0px", l.textLetterSpacing = "0px")), (c === 0 || h) && a === 0) {
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
      const u = fn.graphemeSegmenter(t);
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
  const ws = new yx(), Pa = class Kn extends hn {
    constructor(t = {}) {
      super(), this.uid = ie("textStyle"), this._tick = 0, this._cachedFontString = null, xx(t), t instanceof Kn && (t = t._toObject());
      const i = {
        ...Kn.defaultTextStyle,
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
        ...Kn.defaultDropShadow,
        ...t
      }) : this._dropShadow = t ? this._createProxy({
        ...Kn.defaultDropShadow
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
        ...Be.defaultFillStyle,
        ...t
      }, () => {
        this._fill = Zn({
          ...this._originalFill
        }, Be.defaultFillStyle);
      })), this._fill = Zn(t === 0 ? "black" : t, Be.defaultFillStyle), this.update());
    }
    get stroke() {
      return this._originalStroke;
    }
    set stroke(t) {
      t !== this._originalStroke && (this._originalStroke = t, this._isFillStyle(t) && (this._originalStroke = this._createProxy({
        ...Be.defaultStrokeStyle,
        ...t
      }, () => {
        this._stroke = lr({
          ...this._originalStroke
        }, Be.defaultStrokeStyle);
      })), this._stroke = lr(t, Be.defaultStrokeStyle), this.update());
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
      const t = Kn.defaultTextStyle;
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
      return this._cachedFontString === null && (this._cachedFontString = Ko(this)), this._cachedFontString;
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
      return new Kn(this._toObject());
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
      return (t ?? null) !== null && !(Jt.isColorLike(t) || t instanceof vn || t instanceof vr);
    }
  };
  Pa.defaultDropShadow = {
    alpha: 1,
    angle: Math.PI / 6,
    blur: 0,
    color: "black",
    distance: 5
  };
  Pa.defaultTextStyle = {
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
  De = Pa;
  function xx(n) {
    const t = n;
    if (typeof t.dropShadow == "boolean" && t.dropShadow) {
      const e = De.defaultDropShadow;
      n.dropShadow = {
        alpha: t.dropShadowAlpha ?? e.alpha,
        angle: t.dropShadowAngle ?? e.angle,
        blur: t.dropShadowBlur ?? e.blur,
        color: t.dropShadowColor ?? e.color,
        distance: t.dropShadowDistance ?? e.distance
      };
    }
    if (t.strokeThickness !== void 0) {
      Ot(se, "strokeThickness is now a part of stroke");
      const e = t.stroke;
      let s = {};
      if (Jt.isColorLike(e)) s.color = e;
      else if (e instanceof vn || e instanceof vr) s.fill = e;
      else if (Object.hasOwnProperty.call(e, "color") || Object.hasOwnProperty.call(e, "fill")) s = e;
      else throw new Error("Invalid stroke value.");
      n.stroke = {
        ...s,
        width: t.strokeThickness
      };
    }
    if (Array.isArray(t.fillGradientStops)) {
      if (Ot(se, "gradient fill is now a fill pattern: `new FillGradient(...)`"), !Array.isArray(t.fill) || t.fill.length === 0) throw new Error("Invalid fill value. Expected an array of colors for gradient fill.");
      t.fill.length !== t.fillGradientStops.length && Qt("The number of fill colors must match the number of fill gradient stops.");
      const e = new vn({
        start: {
          x: 0,
          y: 0
        },
        end: {
          x: 0,
          y: 1
        },
        textureSpace: "local"
      }), s = t.fillGradientStops.slice(), i = t.fill.map((r) => Jt.shared.setValue(r).toNumber());
      s.forEach((r, o) => {
        e.addColorStop(r, i[o]);
      }), n.fill = {
        fill: e
      };
    }
  }
  function bx(n, t) {
    const { texture: e, bounds: s } = n, i = t._style._getFinalPadding();
    vh(s, t._anchor, e);
    const r = t._anchor._x * i * 2, o = t._anchor._y * i * 2;
    s.minX -= i - r, s.minY -= i - o, s.maxX -= i - r, s.maxY -= i - o;
  }
  _x = class {
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
  class wx extends _x {
  }
  class zd {
    constructor(t) {
      this._renderer = t, t.runners.resolutionChange.add(this), this._managedTexts = new Bs({
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
        (s.currentKey !== t.styleKey || t._resolution !== i) && this._updateGpuText(t), t._didTextUpdate = false, bx(s, t);
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
      const e = new wx();
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
  zd.extension = {
    type: [
      ct.WebGLPipes,
      ct.WebGPUPipes,
      ct.CanvasPipes
    ],
    name: "text"
  };
  const vx = new $e();
  function Cx(n, t, e, s, i = false) {
    const r = vx;
    r.minX = 0, r.minY = 0, r.maxX = n.width / s | 0, r.maxY = n.height / s | 0;
    const o = gr.getOptimalTexture(r.width, r.height, s, false, i);
    return o.source.uploadMethodId = "image", o.source.resource = n, o.source.alphaMode = "premultiply-alpha-on-upload", o.frame.width = t / s, o.frame.height = e / s, o.source.emit("update", o.source), o.updateUvs(), o;
  }
  class Dd {
    constructor(t, e) {
      this._activeTextures = {}, this._renderer = t, this._retainCanvasContext = e;
    }
    getTexture(t, e, s, i) {
      typeof t == "string" && (Ot("8.0.0", "CanvasTextSystem.getTexture: Use object TextOptions instead of separate arguments"), t = {
        text: t,
        style: s,
        resolution: e
      }), t.style instanceof De || (t.style = new De(t.style)), t.textureStyle instanceof ts || (t.textureStyle = new ts(t.textureStyle)), typeof t.text != "string" && (t.text = t.text.toString());
      const { text: r, style: o, textureStyle: a, autoGenerateMipmaps: l } = t, c = t.resolution ?? this._renderer.resolution, { frame: h, canvasAndContext: d } = ws.getCanvasAndContext({
        text: r,
        style: o,
        resolution: c
      }), u = Cx(d.canvas, h.width, h.height, c, l);
      if (a && (u.source.style = a), o.trim && (h.pad(o.padding), u.frame.copyFrom(h), u.frame.scale(1 / c), u.updateUvs()), o.filters) {
        const p = this._applyFilters(u, o.filters);
        return this.returnTexture(u), ws.returnCanvasAndContext(d), p;
      }
      return this._renderer.texture.initSource(u._source), this._retainCanvasContext || ws.returnCanvasAndContext(d), u;
    }
    returnTexture(t) {
      const e = t.source, s = e.resource;
      if (this._retainCanvasContext && (s == null ? void 0 : s.getContext)) {
        const i = s.getContext("2d");
        i && ws.returnCanvasAndContext({
          canvas: s,
          context: i
        });
      }
      e.resource = null, e.uploadMethodId = "unknown", e.alphaMode = "no-premultiply-alpha", gr.returnTexture(t, true);
    }
    renderTextToCanvas() {
      Ot("8.10.0", "CanvasTextSystem.renderTextToCanvas: no longer supported, use CanvasTextSystem.getTexture instead");
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
  class Hd extends Dd {
    constructor(t) {
      super(t, true);
    }
  }
  Hd.extension = {
    type: [
      ct.CanvasSystem
    ],
    name: "canvasText"
  };
  class Ud extends Dd {
    constructor(t) {
      super(t, false);
    }
  }
  Ud.extension = {
    type: [
      ct.WebGLSystem,
      ct.WebGPUSystem
    ],
    name: "canvasText"
  };
  Ut.add(Hd);
  Ut.add(Ud);
  Ut.add(zd);
  class qe extends Uy {
    constructor(...t) {
      const e = jy(t, "Text");
      super(e, De), this.renderPipeId = "text", e.textureStyle && (this.textureStyle = e.textureStyle instanceof ts ? e.textureStyle : new ts(e.textureStyle)), this.autoGenerateMipmaps = e.autoGenerateMipmaps ?? Oe.defaultOptions.autoGenerateMipmaps;
    }
    updateBounds() {
      const t = this._bounds, e = this._anchor;
      let s = 0, i = 0;
      if (this._style.trim) {
        const { frame: r, canvasAndContext: o } = ws.getCanvasAndContext({
          text: this.text,
          style: this._style,
          resolution: 1
        });
        ws.returnCanvasAndContext(o), s = r.width, i = r.height;
      } else {
        const r = fn.measureText(this._text, this._style);
        s = r.width, i = r.height;
      }
      t.minX = -e._x * s, t.maxX = t.minX + s, t.minY = -e._y * i, t.maxY = t.minY + i;
    }
  }
  Cr = class extends At {
    static create(t) {
      const { dynamic: e, ...s } = t;
      return new Cr({
        source: new Oe(s),
        dynamic: e ?? false
      });
    }
    resize(t, e, s) {
      return this.source.resize(t, e, s), this;
    }
  };
  class Sx {
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
        m !== 16777215 && (y = Zt.getTintedCanvas({
          texture: u
        }, m));
        const b = u.frame, x = u.source.resolution, _ = b.x * x, v = b.y * x, w = b.width * x, C = b.height * x;
        i.globalAlpha = f;
        const T = -d.anchorX * b.width, R = -d.anchorY * b.height;
        d.rotation !== 0 || d.scaleX !== 1 || d.scaleY !== 1 ? (i.save(), i.translate(d.x, d.y), i.rotate(d.rotation), i.scale(d.scaleX, d.scaleY), i.drawImage(y, _, v, w, C, T, R, b.width, b.height), i.restore()) : i.drawImage(y, _, v, w, C, d.x + T, d.y + R, b.width, b.height);
      }
      i.restore();
    }
  }
  function fc(n, t = null) {
    const e = n * 6;
    if (e > 65535 ? t || (t = new Uint32Array(e)) : t || (t = new Uint16Array(e)), t.length !== e) throw new Error(`Out buffer length is incorrect, got ${t.length} and expected ${e}`);
    for (let s = 0, i = 0; s < e; s += 6, i += 4) t[s + 0] = i + 0, t[s + 1] = i + 1, t[s + 2] = i + 2, t[s + 3] = i + 0, t[s + 4] = i + 2, t[s + 5] = i + 3;
    return t;
  }
  function Ex(n) {
    return {
      dynamicUpdate: mc(n, true),
      staticUpdate: mc(n, false)
    };
  }
  function mc(n, t) {
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
      const a = rr(o.format);
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
  class Tx {
    constructor(t) {
      this._size = 0, this._generateParticleUpdateCache = {};
      const e = this._size = t.size ?? 1e3, s = t.properties;
      let i = 0, r = 0;
      for (const h in s) {
        const d = s[h], u = rr(d.format);
        d.dynamic ? r += u.stride : i += u.stride;
      }
      this._dynamicStride = r / 4, this._staticStride = i / 4, this.staticAttributeBuffer = new _s(e * 4 * i), this.dynamicAttributeBuffer = new _s(e * 4 * r), this.indexBuffer = fc(e);
      const o = new yd();
      let a = 0, l = 0;
      this._staticBuffer = new ns({
        data: new Float32Array(1),
        label: "static-particle-buffer",
        shrinkToFit: false,
        usage: ue.VERTEX | ue.COPY_DST
      }), this._dynamicBuffer = new ns({
        data: new Float32Array(1),
        label: "dynamic-particle-buffer",
        shrinkToFit: false,
        usage: ue.VERTEX | ue.COPY_DST
      });
      for (const h in s) {
        const d = s[h], u = rr(d.format);
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
      const e = Ax(t);
      return this._generateParticleUpdateCache[e] ? this._generateParticleUpdateCache[e] : (this._generateParticleUpdateCache[e] = this.generateParticleUpdate(t), this._generateParticleUpdateCache[e]);
    }
    generateParticleUpdate(t) {
      return Ex(t);
    }
    update(t, e) {
      t.length > this._size && (e = true, this._size = Math.max(t.length, this._size * 1.5 | 0), this.staticAttributeBuffer = new _s(this._size * this._staticStride * 4 * 4), this.dynamicAttributeBuffer = new _s(this._size * this._dynamicStride * 4 * 4), this.indexBuffer = fc(this._size), this.geometry.indexBuffer.setDataWithSize(this.indexBuffer, this.indexBuffer.byteLength, true));
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
  function Ax(n) {
    const t = [];
    for (const e in n) {
      const s = n[e];
      t.push(e, s.code, s.dynamic ? "d" : "s");
    }
    return t.join("_");
  }
  var kx = `varying vec2 vUV;
varying vec4 vColor;

uniform sampler2D uTexture;

void main(void){
    vec4 color = texture2D(uTexture, vUV) * vColor;
    gl_FragColor = color;
}`, Mx = `attribute vec2 aVertex;
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
`, gc = `
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
  class Px extends br {
    constructor() {
      const t = ga.from({
        vertex: Mx,
        fragment: kx
      }), e = vi.from({
        fragment: {
          source: gc,
          entryPoint: "mainFragment"
        },
        vertex: {
          source: gc,
          entryPoint: "mainVertex"
        }
      });
      super({
        glProgram: t,
        gpuProgram: e,
        resources: {
          uTexture: At.WHITE.source,
          uSampler: new ts({}),
          uniforms: {
            uTranslationMatrix: {
              value: new Ct(),
              type: "mat3x3<f32>"
            },
            uColor: {
              value: new Jt(16777215),
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
  class Sr {
    constructor(t, e) {
      this.state = or.for2d(), this.localUniforms = new ya({
        uTranslationMatrix: {
          value: new Ct(),
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
      }), this.renderer = t, this.adaptor = e, this.defaultShader = new Px(), this.state = or.for2d(), this._managedContainers = new Bs({
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
      return t._gpuData[this.renderer.uid] = new Tx({
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
      i.update(e, t._childrenDirty), t._childrenDirty = false, r.blendMode = Do(t.blendMode, t.texture._source);
      const o = this.localUniforms.uniforms, a = o.uTranslationMatrix;
      t.worldTransform.copyTo(a);
      const l = s.globalUniforms.globalUniformData;
      a.tx -= l.offset.x, a.ty -= l.offset.y, a.prepend(l.projectionMatrix), o.uResolution = l.resolution, o.uRound = s._roundPixels | t._roundPixels, Ld(t.groupColorAlpha, o.uColor, 0), this.adaptor.execute(this, t);
    }
    destroy() {
      this._managedContainers.destroy(), this.renderer = null, this.defaultShader && (this.defaultShader.destroy(), this.defaultShader = null);
    }
  }
  Sr.extension = {
    type: [
      ct.CanvasPipes
    ],
    name: "particle"
  };
  class jd extends Sr {
    constructor(t) {
      super(t, new Sx());
    }
  }
  jd.extension = {
    type: [
      ct.CanvasPipes
    ],
    name: "particle"
  };
  class Ix {
    execute(t, e) {
      const s = t.state, i = t.renderer, r = e.shader || t.defaultShader;
      r.resources.uTexture = e.texture._source, r.resources.uniforms = t.localUniforms;
      const o = i.gl, a = t.getBuffers(e);
      i.shader.bind(r), i.state.set(s), i.geometry.bind(a.geometry, r.glProgram);
      const c = a.geometry.indexBuffer.data.BYTES_PER_ELEMENT === 2 ? o.UNSIGNED_SHORT : o.UNSIGNED_INT;
      o.drawElements(o.TRIANGLES, e.particleChildren.length * 6, c, 0);
    }
  }
  class Yd extends Sr {
    constructor(t) {
      super(t, new Ix());
    }
  }
  Yd.extension = {
    type: [
      ct.WebGLPipes
    ],
    name: "particle"
  };
  class Rx {
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
  class Vd extends Sr {
    constructor(t) {
      super(t, new Rx());
    }
  }
  Vd.extension = {
    type: [
      ct.WebGPUPipes
    ],
    name: "particle"
  };
  const Xd = class Jo {
    constructor(t) {
      if (t instanceof At) this.texture = t, Ro(this, Jo.defaultOptions, {});
      else {
        const e = {
          ...Jo.defaultOptions,
          ...t
        };
        Ro(this, e, {});
      }
    }
    get alpha() {
      return this._alpha;
    }
    set alpha(t) {
      this._alpha = Math.min(Math.max(t, 0), 1), this._updateColor();
    }
    get tint() {
      return ni(this._tint);
    }
    set tint(t) {
      this._tint = Jt.shared.setValue(t ?? 16777215).toBgrNumber(), this._updateColor();
    }
    _updateColor() {
      this.color = this._tint + ((this._alpha * 255 | 0) << 24);
    }
  };
  Xd.defaultOptions = {
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
  let ai = Xd;
  const yc = {
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
  Ut.add(Yd);
  Ut.add(Vd);
  Ut.add(jd);
  const Lx = new $e(0, 0, 0, 0), qd = class Zo extends yr {
    constructor(t = {}) {
      t = {
        ...Zo.defaultOptions,
        ...t,
        dynamicProperties: {
          ...Zo.defaultOptions.dynamicProperties,
          ...t == null ? void 0 : t.dynamicProperties
        }
      };
      const { dynamicProperties: e, shader: s, roundPixels: i, texture: r, particles: o, ...a } = t;
      super({
        label: "ParticleContainer",
        ...a
      }), this.renderPipeId = "particle", this.batched = false, this._childrenDirty = false, this.texture = r || null, this.shader = s, this._properties = {};
      for (const l in yc) {
        const c = yc[l], h = e[l];
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
      return Lx;
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
  qd.defaultOptions = {
    dynamicProperties: {
      vertex: false,
      position: true,
      rotation: false,
      uvs: false,
      color: false
    },
    roundPixels: false
  };
  let $x = qd;
  Ut.add(op, ap);
  var Ox = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
  function Bx(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var Kd = {
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
    }).call(Ox);
  })(Kd);
  var Nx = Kd.exports;
  const xc = Bx(Nx);
  function Er(n, t) {
    if (n) {
      if (typeof n == "function") return n;
      if (typeof n == "string") return xc[n];
    } else return xc[t];
  }
  class Fx {
    constructor(t) {
      this.viewport = t, this.touches = [], this.addListeners();
    }
    addListeners() {
      this.viewport.eventMode = "static", this.viewport.forceHitArea || (this.viewport.hitArea = new Yt(0, 0, this.viewport.worldWidth, this.viewport.worldHeight)), this.viewport.on("pointerdown", this.down, this), this.viewport.options.allowPreserveDragOutside ? this.viewport.on("globalpointermove", this.move, this) : this.viewport.on("pointermove", this.move, this), this.viewport.on("pointerup", this.up, this), this.viewport.on("pointerupoutside", this.up, this), this.viewport.on("pointercancel", this.up, this), this.viewport.options.allowPreserveDragOutside || this.viewport.on("pointerleave", this.up, this), this.wheelFunction = (t) => this.handleWheel(t), this.viewport.options.events.domElement.addEventListener("wheel", this.wheelFunction, {
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
      const e = new Rt();
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
  const Vs = [
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
  class Wx {
    constructor(t) {
      this.viewport = t, this.list = [], this.plugins = {};
    }
    add(t, e, s = Vs.length) {
      const i = this.plugins[t];
      i && i.destroy(), this.plugins[t] = e;
      const r = Vs.indexOf(t);
      r !== -1 && Vs.splice(r, 1), Vs.splice(s, 0, t), this.sort();
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
      for (const t of Vs) this.plugins[t] && this.list.push(this.plugins[t]);
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
  class He {
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
  const Gx = {
    removeOnInterrupt: false,
    ease: "linear",
    time: 1e3
  };
  class zx extends He {
    constructor(t, e = {}) {
      super(t), this.startWidth = null, this.startHeight = null, this.deltaWidth = null, this.deltaHeight = null, this.width = null, this.height = null, this.time = 0, this.options = Object.assign({}, Gx, e), this.options.ease = Er(this.options.ease), this.setupPosition(), this.setupZoom(), this.time = 0;
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
      const e = new Rt(this.parent.scale.x, this.parent.scale.y);
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
          const i = this.startX, r = this.startY, o = this.deltaX, a = this.deltaY, l = new Rt(this.parent.x, this.parent.y);
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
  const Dx = {
    sides: "all",
    friction: 0.5,
    time: 150,
    ease: "easeInOutSine",
    underflow: "center",
    bounceBox: null
  };
  class Hx extends He {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Dx, e), this.ease = Er(this.options.ease, "easeInOutSine"), this.options.sides ? this.options.sides === "all" ? this.top = this.bottom = this.left = this.right = true : this.options.sides === "horizontal" ? (this.right = this.left = true, this.top = this.bottom = false) : this.options.sides === "vertical" ? (this.left = this.right = false, this.top = this.bottom = true) : (this.top = this.options.sides.indexOf("top") !== -1, this.bottom = this.options.sides.indexOf("bottom") !== -1, this.left = this.options.sides.indexOf("left") !== -1, this.right = this.options.sides.indexOf("right") !== -1) : this.left = this.top = this.right = this.bottom = false;
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
          topLeft: new Rt(e * this.parent.scale.x, s * this.parent.scale.y),
          bottomRight: new Rt(i * this.parent.scale.x - this.parent.screenWidth, r * this.parent.scale.y - this.parent.screenHeight)
        };
      }
      return {
        left: this.parent.left < 0,
        right: this.parent.right > this.parent.worldWidth,
        top: this.parent.top < 0,
        bottom: this.parent.bottom > this.parent.worldHeight,
        topLeft: new Rt(0, 0),
        bottomRight: new Rt(this.parent.worldWidth * this.parent.scale.x - this.parent.screenWidth, this.parent.worldHeight * this.parent.scale.y - this.parent.screenHeight)
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
  const Ux = {
    left: false,
    right: false,
    top: false,
    bottom: false,
    direction: null,
    underflow: "center"
  };
  class jx extends He {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Ux, e), this.options.direction && (this.options.left = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.right = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.top = this.options.direction === "y" || this.options.direction === "all" ? true : null, this.options.bottom = this.options.direction === "y" || this.options.direction === "all" ? true : null), this.parseUnderflow(), this.last = {
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
      const t = new Rt(this.parent.x, this.parent.y), e = this.parent.plugins.decelerate || {};
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
  const Yx = {
    minWidth: null,
    minHeight: null,
    maxWidth: null,
    maxHeight: null,
    minScale: null,
    maxScale: null
  };
  class Vx extends He {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Yx, e), this.clamp();
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
  const Xx = {
    friction: 0.98,
    bounce: 0.8,
    minSpeed: 0.01
  }, Sn = 16;
  class qx extends He {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Xx, e), this.saved = [], this.timeSinceRelease = 0, this.reset(), this.parent.on("moved", (s) => this.handleMoved(s));
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
        this.parent.x += this.x * Sn / o * (Math.pow(r, i / Sn) - Math.pow(r, s / Sn)), this.x *= Math.pow(this.percentChangeX, t / Sn);
      }
      if (this.y) {
        const r = this.percentChangeY, o = Math.log(r);
        this.parent.y += this.y * Sn / o * (Math.pow(r, i / Sn) - Math.pow(r, s / Sn)), this.y *= Math.pow(this.percentChangeY, t / Sn);
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
  const Kx = {
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
  class Jx extends He {
    constructor(t, e = {}) {
      super(t), this.windowEventHandlers = [], this.options = Object.assign({}, Kx, e), this.moved = false, this.reverse = this.options.reverse ? 1 : -1, this.xDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "x", this.yDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "y", this.keyIsPressed = false, this.parseUnderflow(), this.mouseButtons(this.options.mouseButtons), this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
            screen: new Rt(this.last.x, this.last.y),
            world: this.parent.toWorld(new Rt(this.last.x, this.last.y)),
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
        const s = new Rt(this.last.x, this.last.y);
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
  const Zx = {
    speed: 0,
    acceleration: null,
    radius: null
  };
  class Qx extends He {
    constructor(t, e, s = {}) {
      super(t), this.target = e, this.options = Object.assign({}, Zx, s), this.velocity = {
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
  const t0 = {
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
  class e0 extends He {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, t0, e), this.reverse = this.options.reverse ? 1 : -1, this.radiusSquared = typeof this.options.radius == "number" ? Math.pow(this.options.radius, 2) : null, this.resize();
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
  const n0 = {
    noDrag: false,
    percent: 1,
    center: null,
    factor: 1,
    axis: "all"
  }, s0 = new Rt();
  class i0 extends He {
    constructor(t, e = {}) {
      super(t), this.active = false, this.pinching = false, this.moved = false, this.options = Object.assign({}, n0, e);
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
      const { x: e, y: s } = (this.parent.parent || this.parent).toLocal(t.global, void 0, s0), i = this.parent.input.touches;
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
          const c = new Rt(r.last.x + (o.last.x - r.last.x) / 2, r.last.y + (o.last.y - r.last.y) / 2);
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
  const r0 = {
    topLeft: false,
    friction: 0.8,
    time: 1e3,
    ease: "easeInOutSine",
    interrupt: true,
    removeOnComplete: false,
    removeOnInterrupt: false,
    forceStart: false
  };
  class o0 extends He {
    constructor(t, e, s, i = {}) {
      super(t), this.options = Object.assign({}, r0, i), this.ease = Er(i.ease, "easeInOutSine"), this.x = e, this.y = s, this.options.forceStart && this.snapStart();
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
  const a0 = {
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
  class l0 extends He {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, a0, e), this.ease = Er(this.options.ease), this.xIndependent = false, this.yIndependent = false, this.xScale = 0, this.yScale = 0, this.options.width > 0 && (this.xScale = t.screenWidth / this.options.width, this.xIndependent = true), this.options.height > 0 && (this.yScale = t.screenHeight / this.options.height, this.yIndependent = true), this.xScale = this.xIndependent ? this.xScale : this.yScale, this.yScale = this.yIndependent ? this.yScale : this.xScale, this.options.time === 0 ? (t.container.scale.x = this.xScale, t.container.scale.y = this.yScale, this.options.removeOnComplete && this.parent.plugins.remove("snap-zoom")) : e.forceStart && this.createSnapping();
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
  const c0 = {
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
  class h0 extends He {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, c0, e), this.keyIsPressed = false, this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
  const d0 = {
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
    ticker: Jn.shared,
    allowPreserveDragOutside: false
  };
  class Jd extends Ht {
    constructor(t) {
      super(), this._disableOnContextMenu = (e) => e.preventDefault(), this.options = {
        ...d0,
        ...t
      }, this.screenWidth = this.options.screenWidth, this.screenHeight = this.options.screenHeight, this._worldWidth = this.options.worldWidth, this._worldHeight = this.options.worldHeight, this.forceHitArea = this.options.forceHitArea, this.threshold = this.options.threshold, this.options.disableOnContextMenu && this.options.events.domElement.addEventListener("contextmenu", this._disableOnContextMenu), this.options.noTicker || (this.tickerFunction = () => this.update(this.options.ticker.elapsedMS), this.options.ticker.add(this.tickerFunction)), this.input = new Fx(this), this.plugins = new Wx(this);
    }
    destroy(t) {
      var e;
      !this.options.noTicker && this.tickerFunction && this.options.ticker.remove(this.tickerFunction), this.options.disableOnContextMenu && ((e = this.options.events.domElement) == null || e.removeEventListener("contextmenu", this._disableOnContextMenu)), this.input.destroy(), super.destroy(t);
    }
    update(t) {
      this.pause || (this.plugins.update(t), this.lastViewport && (this.lastViewport.x !== this.x || this.lastViewport.y !== this.y ? this.moving = true : this.moving && (this.emit("moved-end", this), this.moving = false), this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y ? this.zooming = true : this.zooming && (this.emit("zoomed-end", this), this.zooming = false)), this.forceHitArea || (this._hitAreaDefault = new Yt(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight), this.hitArea = this._hitAreaDefault), this._dirty = this._dirty || !this.lastViewport || this.lastViewport.x !== this.x || this.lastViewport.y !== this.y || this.lastViewport.scaleX !== this.scale.x || this.lastViewport.scaleY !== this.scale.y, this.lastViewport = {
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
      return new Yt(this.left, this.top, this.worldScreenWidth, this.worldScreenHeight);
    }
    toWorld(t, e) {
      return arguments.length === 2 ? this.toLocal(new Rt(t, e)) : this.toLocal(t);
    }
    toScreen(t, e) {
      return arguments.length === 2 ? this.toGlobal(new Rt(t, e)) : this.toGlobal(t);
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
      return new Rt(this.worldScreenWidth / 2 - this.x / this.scale.x, this.worldScreenHeight / 2 - this.y / this.scale.y);
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
      return new Rt(-this.x / this.scale.x, -this.y / this.scale.y);
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
      return this.plugins.add("snap-zoom", new l0(this, t)), this;
    }
    OOB() {
      return {
        left: this.left < 0,
        right: this.right > this.worldWidth,
        top: this.top < 0,
        bottom: this.bottom > this.worldHeight,
        cornerPoint: new Rt(this.worldWidth * this.scale.x - this.screenWidth, this.worldHeight * this.scale.y - this.screenHeight)
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
      t ? (this._forceHitArea = t, this.hitArea = t) : (this._forceHitArea = null, this.hitArea = new Yt(0, 0, this.worldWidth, this.worldHeight));
    }
    drag(t) {
      return this.plugins.add("drag", new Jx(this, t)), this;
    }
    clamp(t) {
      return this.plugins.add("clamp", new jx(this, t)), this;
    }
    decelerate(t) {
      return this.plugins.add("decelerate", new qx(this, t)), this;
    }
    bounce(t) {
      return this.plugins.add("bounce", new Hx(this, t)), this;
    }
    pinch(t) {
      return this.plugins.add("pinch", new i0(this, t)), this;
    }
    snap(t, e, s) {
      return this.plugins.add("snap", new o0(this, t, e, s)), this;
    }
    follow(t, e) {
      return this.plugins.add("follow", new Qx(this, t, e)), this;
    }
    wheel(t) {
      return this.plugins.add("wheel", new h0(this, t)), this;
    }
    animate(t) {
      return this.plugins.add("animate", new zx(this, t)), this;
    }
    clampZoom(t) {
      return this.plugins.add("clamp-zoom", new Vx(this, t)), this;
    }
    mouseEdges(t) {
      return this.plugins.add("mouse-edges", new e0(this, t)), this;
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
  const u0 = 32, Qo = /* @__PURE__ */ new Set([
    "transport-belt",
    "fast-transport-belt",
    "express-transport-belt"
  ]), Gi = /* @__PURE__ */ new Set([
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt"
  ]), co = /* @__PURE__ */ new Set([
    "splitter",
    "fast-splitter",
    "express-splitter"
  ]), p0 = /* @__PURE__ */ new Set([
    "inserter",
    "fast-inserter",
    "long-handed-inserter"
  ]), f0 = /* @__PURE__ */ new Set([
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
  ]), bc = {
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
  function rn(n, t) {
    return `${n},${t}`;
  }
  function hr(n) {
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
  function zi(n, t, e, s, i, r) {
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
  const m0 = 9;
  function g0(n) {
    const t = {
      nodes: /* @__PURE__ */ new Map(),
      outEdges: /* @__PURE__ */ new Map(),
      inEdges: /* @__PURE__ */ new Map(),
      tileToAnchor: /* @__PURE__ */ new Map(),
      entityMap: /* @__PURE__ */ new Map()
    };
    for (const e of n.entities) t.entityMap.set(rn(e.x ?? 0, e.y ?? 0), e);
    for (const e of n.entities) {
      if (!Qo.has(e.name) && !Gi.has(e.name) && !co.has(e.name)) continue;
      const s = e.x ?? 0, i = e.y ?? 0, r = rn(s, i);
      if (t.nodes.set(r, e), t.tileToAnchor.set(r, r), co.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South", a = s + (o ? 1 : 0), l = i + (o ? 0 : 1);
        t.tileToAnchor.set(rn(a, l), r);
      }
    }
    for (const [e, s] of t.nodes) {
      const i = s.x ?? 0, r = s.y ?? 0, o = s.direction ?? "North", [a, l] = hr(o);
      if (Qo.has(s.name)) {
        const c = t.tileToAnchor.get(rn(i + a, r + l));
        if (c !== void 0 && c !== e) {
          const h = t.nodes.get(c), [d, u] = hr(h.direction), p = a * u - l * d;
          zi(t, e, c, "both", p > 0, false);
        }
      } else if (Gi.has(s.name)) if (s.io_type === "input") for (let c = 1; c <= m0; c++) {
        const h = t.entityMap.get(rn(i + a * c, r + l * c));
        if (h) {
          if (Gi.has(h.name) && h.name === s.name && h.io_type === "input" && h.direction === o) break;
          if (Gi.has(h.name) && h.name === s.name && h.io_type === "output" && h.direction === o) {
            const d = t.tileToAnchor.get(rn(h.x ?? 0, h.y ?? 0));
            d !== void 0 && zi(t, e, d, "both", false, false);
            break;
          }
        }
      }
      else {
        const c = t.tileToAnchor.get(rn(i + a, r + l));
        c !== void 0 && c !== e && zi(t, e, c, "both", false, false);
      }
      else if (co.has(s.name)) {
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
          const f = t.tileToAnchor.get(rn(u, p));
          f !== void 0 && f !== e && zi(t, e, f, "both", false, true);
        }
      }
    }
    return t;
  }
  function y0(n, t) {
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
  function x0(n, t) {
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
        const c = rn(r + a, o + l), h = t.get(c);
        h && p0.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  function b0(n, t) {
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
        const c = rn(r + a, o + l), h = t.get(c);
        h && f0.has(h.name) && e.add(c);
      }
    }
    return e;
  }
  const ta = {
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
  function _c(n, t) {
    const e = t.nodes.get(n);
    if (!e || !Qo.has(e.name)) return null;
    const s = e.direction ?? "North";
    for (const i of t.inEdges.get(n) ?? []) {
      const r = t.nodes.get(i.from);
      if (!r) continue;
      const o = r.direction ?? "North";
      if (`${o}_${s}` in ta) return {
        inDir: o,
        outDir: s
      };
    }
    return null;
  }
  function _0(n, t, e, s, i, r, o) {
    const a = s - t, l = i - e, c = Math.sqrt(a * a + l * l);
    if (c === 0) return;
    const h = a / c, d = l / c;
    let u = 0, p = true;
    for (; u < c; ) {
      const f = Math.min(p ? r : o, c - u);
      p && n.moveTo(t + h * u, e + d * u).lineTo(t + h * (u + f), e + d * (u + f)).stroke(), u += f, p = !p;
    }
  }
  function w0(n, t, e, s, i, r, o, a, l) {
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
  function v0(n, t, e, s, i) {
    const r = u0, o = r / 2;
    for (const l of e) {
      if (t.has(l)) continue;
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = bc[c.name] ?? 14733424, [p, f] = hr(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.05
      }), n.setStrokeStyle({
        width: 1.5,
        color: u,
        alpha: 0.28,
        cap: "round"
      });
      const y = _c(l, i);
      if (y) {
        const b = ta[`${y.inDir}_${y.outDir}`], x = b.cx(h, d, r), _ = b.cy(h, d, r);
        w0(n, x, _, o, b.startAngle, b.endAngle, b.anticlockwise, 5 / o, 3 / o);
      } else _0(n, g - p * r * 0.45, m - f * r * 0.45, g + p * r * 0.45, m + f * r * 0.45, 5, 3);
    }
    for (const l of t) {
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = bc[c.name] ?? 14733424, [p, f] = hr(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.2
      }), n.setStrokeStyle({
        width: 2,
        color: u,
        alpha: 0.85,
        cap: "round"
      });
      const y = _c(l, i);
      if (y) {
        const b = ta[`${y.inDir}_${y.outDir}`], x = b.cx(h, d, r), _ = b.cy(h, d, r), v = x + o * Math.cos(b.startAngle), w = _ + o * Math.sin(b.startAngle);
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
  let C0, S0, Nn, Zd, Qd, E0, wc, vc, T0, A0, k0;
  S = 32;
  C0 = {
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
  S0 = 4872810;
  Nn = {
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
  Zd = {
    inserter: 6983230,
    "fast-inserter": 4886736,
    "long-handed-inserter": 13647936,
    "stack-inserter": 4114794,
    "bulk-inserter": 3123354
  };
  Qd = 9079434;
  E0 = 6974058;
  wc = 2039583;
  vc = 12623920;
  T0 = 2762e3;
  A0 = 0.35;
  k0 = {
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
  function M0(n, t) {
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = 0.21 * e + 0.72 * s + 0.07 * i, o = Math.round(r + (e - r) * t), a = Math.round(r + (s - r) * t), l = Math.round(r + (i - r) * t);
    return o << 16 | a << 8 | l;
  }
  const Cc = Object.fromEntries(Object.entries(k0).map(([n, t]) => [
    n,
    M0(t, A0)
  ]));
  function P0(n, t, e) {
    const s = t * Math.min(e, 1 - e), i = (r) => {
      const o = (r + n * 12) % 12;
      return Math.round((e - s * Math.max(-1, Math.min(o - 3, 9 - o, 1))) * 255);
    };
    return i(0) << 16 | i(8) << 8 | i(4);
  }
  let tu = true;
  function Di(n) {
    tu = n;
  }
  let ea = /* @__PURE__ */ new Map();
  function I0(n) {
    return ea.get(n);
  }
  function eu(n) {
    ea = /* @__PURE__ */ new Map();
    for (const t of n) ea.set(t.recipe, {
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
  function On(n) {
    if (!tu) return 7829367;
    if (!n) return 6710886;
    if (n in Cc) return Cc[n];
    let t = 0;
    for (let s = 0; s < n.length; s++) t = (t << 5) - t + n.charCodeAt(s) | 0;
    const e = Math.abs(t) % 30 * 12;
    return P0(e / 360, 0.2, 0.48);
  }
  const Se = {
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
  function xe(n) {
    return n.split("-").map((t) => t.charAt(0).toUpperCase() + t.slice(1)).join(" ");
  }
  let Re, Tr, cn, ae, mi, Ia;
  Re = new Set(Object.keys(Se));
  Tr = new Set(Object.keys(Zd));
  cn = new Set(Object.keys(Nn).filter((n) => !n.includes("underground") && !n.includes("splitter")));
  ye = new Set(Object.keys(Nn).filter((n) => n.includes("underground")));
  ae = new Set(Object.keys(Nn).filter((n) => n.includes("splitter")));
  mi = /* @__PURE__ */ new Set([
    "pipe",
    "pipe-to-ground"
  ]);
  Ia = /* @__PURE__ */ new Set([
    "medium-electric-pole",
    "small-electric-pole"
  ]);
  function Ci(n) {
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
  function $n(n) {
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
  function Ra(n) {
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
  function nu(n, t) {
    const e = n.direction ?? "North", [s, i] = $n(e);
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
      if (!d || !(cn.has(d.name) || ye.has(d.name) && d.io_type === "output" || ae.has(d.name))) continue;
      const [p, f] = $n(d.direction), g = ae.has(d.name) ? c : d.x ?? 0, m = ae.has(d.name) ? h : d.y ?? 0;
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
  function Sc(n, t) {
    const e = Math.round((n >> 16 & 255) * t), s = Math.round((n >> 8 & 255) * t), i = Math.round((n & 255) * t);
    return e << 16 | s << 8 | i;
  }
  const gi = 3, su = 3815994, La = 5592405, $a = 0.9, Hi = S * (1 - $a) / 2, iu = S * $a;
  function Oa(n, t) {
    const e = new pt(), s = S, i = iu, [, r] = Nn[n.name] ?? [
      11046960,
      14733424
    ], o = On(n.carries);
    if (t) R0(e, s, r, n.direction, t, o);
    else {
      e.rect(Hi, Hi, i, i).fill(su), e.setStrokeStyle({
        width: 1,
        color: La,
        alignment: 0
      }), e.rect(Hi, Hi, i, i).stroke();
      const a = new pt();
      a.x = s / 2, a.y = s / 2, a.rotation = Ci(n.direction), a.rect(-i / 2, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(1, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(-1, -i / 2, 2, i).fill(657930), L0(a, i, r), e.addChild(a);
    }
    return e;
  }
  function R0(n, t, e, s, i, r) {
    const o = new pt();
    o.x = t / 2, o.y = t / 2, o.rotation = Ci(s), o.scale.set($a);
    const a = t / 2, c = (i.turn === "cw" ? 1 : -1) * a, h = -a, d = i.turn === "ccw" ? 0 : Math.PI / 2, u = i.turn === "ccw" ? Math.PI / 2 : Math.PI;
    o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().fill(su), o.setStrokeStyle({
      width: 1,
      color: La,
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
    const R = Math.PI / 2, E = i.turn === "cw" ? Math.PI : 0;
    for (const A of [
      0.6
    ]) {
      const W = R + A * (E - R), j = i.turn === "cw" ? W - w : W + w, G = c + v * Math.cos(W), z = h + v * Math.sin(W), P = c + C * Math.cos(j), D = h + C * Math.sin(j), J = c + T * Math.cos(j), Y = h + T * Math.sin(j);
      o.moveTo(P, D).lineTo(G, z).lineTo(J, Y).stroke();
    }
    n.addChild(o);
  }
  function L0(n, t, e) {
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
  function ru(n) {
    const t = new pt(), e = S, [, s] = Nn[n.name] ?? [
      11046960,
      14733424
    ], i = n.io_type === "input", r = e / 2, o = i ? 1 : -1, a = new pt();
    a.x = r, a.y = r, a.rotation = Ci(n.direction);
    const l = On(n.carries), c = iu / 2, h = e * 0.25, d = o * r, u = 0;
    a.moveTo(-c, d).lineTo(c, d).lineTo(h, u).lineTo(-h, u).closePath().fill({
      color: l,
      alpha: 0.7
    }), a.setStrokeStyle({
      width: 1,
      color: La,
      alpha: 0.8
    }), a.moveTo(-c, d).lineTo(-h, u).lineTo(h, u).lineTo(c, d).stroke();
    const p = e * 0.38, f = e * 0.3, g = o * e * 0.22, m = g - f / 2, y = g + f / 2;
    return a.moveTo(0, m).lineTo(p / 2, y).lineTo(-p / 2, y).closePath().fill(s), t.addChild(a), t;
  }
  function ou(n) {
    const t = new pt(), [e, s] = Nn[n.name] ?? [
      11046960,
      14733424
    ], i = n.direction === "North" || n.direction === "South", r = i ? S * 2 - 1 : S - 1, o = i ? S - 1 : S * 2 - 1, a = i ? r / 2 : o / 2, l = Math.max(2, Math.min(r, o) * 0.18);
    t.roundRect(0, 0, r, o, gi).fill(e), t.roundRect(0, 0, r, o, gi).fill({
      color: On(n.carries),
      alpha: 0.3
    }), i ? t.rect(a - l / 2, 0, l, o).fill(Sc(e, 0.5)) : t.rect(0, a - l / 2, r, l).fill(Sc(e, 0.5));
    const c = Ci(n.direction), h = a * 0.25, d = Math.max(1, a * 0.12);
    for (let u = 0; u < 2; u++) {
      const p = i ? a * u + a / 2 : r / 2, f = i ? o / 2 : a * u + a / 2, g = new pt();
      g.x = p, g.y = f, g.rotation = c, g.setStrokeStyle({
        width: d,
        color: s,
        cap: "round"
      }), g.moveTo(-h, h * 0.5).lineTo(0, -h * 0.5).lineTo(h, h * 0.5).stroke(), t.addChild(g);
    }
    return t;
  }
  function au(n) {
    const t = new pt(), e = S - 1, s = n.carries ? On(n.carries) : Zd[n.name] ?? 6983230;
    t.roundRect(0, 0, e, e, gi).fill(2767402);
    const i = new pt();
    i.x = e / 2, i.y = e / 2, i.rotation = Ci(n.direction), i.circle(0, e * 0.2, e * 0.15).fill(4473924);
    const r = Math.max(1.5, e * 0.12);
    i.setStrokeStyle({
      width: r,
      color: s,
      cap: "round"
    }), i.moveTo(0, e * 0.2).lineTo(0, -e * 0.35).stroke();
    const o = -e * 0.35, a = e * 0.18;
    return i.moveTo(-a, o - a * 0.6).lineTo(0, o).lineTo(a, o - a * 0.6).stroke(), t.addChild(i), t;
  }
  const Ba = 1, Na = 2, Fa = 4, Wa = 8;
  function $0(n) {
    const [t, e] = $n(n.direction);
    return [
      -t,
      -e
    ];
  }
  function lu(n, t, e) {
    if (n.name === "pipe") return true;
    if (n.name === "pipe-to-ground") {
      const [s, i] = $0(n);
      return -t === s && -e === i;
    }
    return false;
  }
  function cu(n, t) {
    const e = new pt(), s = S - 1, i = n.name === "pipe-to-ground", r = i ? E0 : Qd;
    e.roundRect(0, 0, s, s, gi).fill(wc);
    const o = s / 2, a = s / 2, l = Math.max(2, s * 0.4);
    if (i) {
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      });
      const [c, h] = $n(n.direction);
      e.moveTo(o, a).lineTo(o - c * s / 2, a - h * s / 2).stroke(), e.circle(o, a, l * 0.4).fill(r), e.circle(o, a, l * 0.25).fill(wc);
    } else if (t === 0) e.circle(o, a, l * 0.4).fill(r);
    else {
      const c = !!(t & Ba), h = !!(t & Na), d = !!(t & Fa), u = !!(t & Wa), p = (c ? 1 : 0) + (h ? 1 : 0) + (d ? 1 : 0) + (u ? 1 : 0);
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      }), p === 1 ? (c ? e.moveTo(o, a).lineTo(o, 0).stroke() : h ? e.moveTo(o, a).lineTo(s, a).stroke() : d ? e.moveTo(o, a).lineTo(o, s).stroke() : e.moveTo(o, a).lineTo(0, a).stroke(), e.circle(o, a, l * 0.4).fill(r)) : c && d && !h && !u ? e.moveTo(o, 0).lineTo(o, s).stroke() : h && u && !c && !d ? e.moveTo(0, a).lineTo(s, a).stroke() : p === 2 ? c && h ? e.moveTo(o, 0).quadraticCurveTo(o, a, s, a).stroke() : h && d ? e.moveTo(s, a).quadraticCurveTo(o, a, o, s).stroke() : d && u ? e.moveTo(o, s).quadraticCurveTo(o, a, 0, a).stroke() : e.moveTo(0, a).quadraticCurveTo(o, a, o, 0).stroke() : p === 3 ? u ? d ? h ? (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, s).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(0, a).stroke()) : (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, 0).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(s, a).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(0, a).lineTo(s, a).stroke());
    }
    return e;
  }
  function hu() {
    const n = new pt(), t = S - 1;
    n.roundRect(0, 0, t, t, gi).fill(T0);
    const e = t / 2, s = t / 2, i = t * 0.38, r = Math.max(1.5, t * 0.2);
    return n.rect(e - r / 2, s - i, r, i * 2).fill(vc), n.rect(e - i, s - r / 2, i * 2, r).fill(vc), n.circle(e, s, r * 0.6).fill(14729280), n;
  }
  function du(n) {
    const t = new pt(), [e, s] = Se[n.name] ?? [
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
    const l = 1.8, c = na(`/spaghettio/pr-536/entity-frames/${n.name}.png`);
    if (c) {
      const h = new ze(c), d = S / O0;
      h.scale.set(d * l), h.x = -i * (l - 1) / 2, h.y = -r * (l - 1) / 2, t.addChild(h);
    } else {
      const h = na(`/spaghettio/pr-536/icons/${n.name}.png`);
      if (h) {
        const d = new ze(h), u = Math.min(i, r) * 0.8 * l;
        d.width = u, d.height = u, d.x = (i - u) / 2, d.y = (r - u) / 2, t.addChild(d);
      } else {
        const d = C0[n.name] ?? S0;
        t.roundRect(2, 2, i - 4, r - 4, 3).fill({
          color: d,
          alpha: 0.5
        });
      }
    }
    return t;
  }
  function uu() {
    const n = new pt(), t = S - 1;
    return n.rect(0, 0, t, t).fill(4872810), n.setStrokeStyle({
      width: 1,
      color: 0,
      alpha: 0.4
    }), n.rect(0, 0, t, t).stroke(), n;
  }
  const O0 = 64;
  async function B0(n) {
    const t = "/spaghettio/pr-536/", e = [
      ...n.map((s) => `${t}icons/${s}.png`),
      ...n.map((s) => `${t}entity-frames/${s}.png`)
    ];
    await Promise.allSettled(e.map((s) => Fe.load(s)));
  }
  async function pu(n) {
    const t = "/spaghettio/pr-536/";
    await Promise.allSettled(n.map((e) => Fe.load(`${t}icons/${e}.png`)));
  }
  function N0(n) {
    const t = /* @__PURE__ */ new Set();
    for (const e of n) e.carries && t.add(e.carries);
    return Array.from(t);
  }
  function na(n) {
    return oe.has(n) ? Fe.get(n) ?? null : null;
  }
  const F0 = {
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
  function fu(n) {
    const t = F0[n.name];
    if (!t) return [];
    const e = n.mirror ?? false;
    return t.filter(([, , , s]) => s === "always" || s === "default" && !e || s === "mirror" && e);
  }
  function mu(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of fu(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  Ar = function() {
    return {
      tileMap: /* @__PURE__ */ new Map(),
      machineByTile: /* @__PURE__ */ new Map()
    };
  };
  dr = function(n, t) {
    const e = n.x ?? 0, s = n.y ?? 0;
    if (t.tileMap.set(`${e},${s}`, n), ae.has(n.name)) {
      const [i, r] = Ra(n.direction);
      t.tileMap.set(`${e + i},${s + r}`, n);
    }
    if (Re.has(n.name)) {
      const [i, r] = Se[n.name] ?? [
        1,
        1
      ];
      for (let o = 0; o < r; o++) for (let a = 0; a < i; a++) t.machineByTile.set(`${e + a},${s + o}`, n);
    }
  };
  Dn = function(n, t) {
    let e;
    if (cn.has(n.name)) e = Oa(n, nu(n, t.tileMap));
    else if (ye.has(n.name)) e = ru(n);
    else if (ae.has(n.name)) e = ou(n);
    else if (Tr.has(n.name)) e = au(n);
    else if (mi.has(n.name)) {
      let s = 0;
      if (n.name === "pipe") {
        const i = n.x ?? 0, r = n.y ?? 0;
        for (const [o, a, l] of [
          [
            0,
            -1,
            Ba
          ],
          [
            1,
            0,
            Na
          ],
          [
            0,
            1,
            Fa
          ],
          [
            -1,
            0,
            Wa
          ]
        ]) {
          const c = `${i + o},${r + a}`, h = t.tileMap.get(c);
          if (h && lu(h, o, a)) {
            s |= l;
            continue;
          }
          const d = t.machineByTile.get(c);
          d && mu(i, r, d) && (s |= l);
        }
      }
      e = cu(n, s);
    } else Ia.has(n.name) ? e = hu() : Re.has(n.name) ? e = du(n) : e = uu();
    return n.quality && n.quality !== "normal" && z0(e, n.quality), e.x = (n.x ?? 0) * S, e.y = (n.y ?? 0) * S, e;
  };
  const gu = {
    uncommon: 5229135,
    rare: 5213130,
    epic: 10899402,
    legendary: 15246141
  }, W0 = [
    "quality-uncommon",
    "quality-rare",
    "quality-epic",
    "quality-legendary"
  ];
  async function G0() {
    const n = "/spaghettio/pr-536/";
    await Promise.allSettled(W0.map((t) => Fe.load(`${n}icons/${t}.png`)));
  }
  function z0(n, t) {
    const e = n.getLocalBounds(), s = 1.5, i = na(`/spaghettio/pr-536/icons/quality-${t}.png`);
    if (i) {
      const c = S * 0.42, h = new ze(i);
      h.width = c, h.height = c, h.x = e.minX + s, h.y = e.maxY - c - s, n.addChild(h);
      return;
    }
    const r = gu[t];
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
  f1 = function(n, t, e = 8) {
    if (!ye.has(n.name) || n.io_type !== "input") return null;
    const [s, i] = $n(n.direction), r = n.x ?? 0, o = n.y ?? 0;
    for (let a = 1; a <= e; a++) {
      const l = t.get(`${r + s * a},${o + i * a}`);
      if (l) {
        if (ye.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "input") return null;
        if (ye.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "output") {
          const [c] = Nn[n.name] ?? [
            11046960,
            14733424
          ], h = new pt(), d = Math.abs(s) > 0;
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
  li = function(n, t, e, s, i, r) {
    t.removeChildren();
    const o = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (o.set(`${f.x ?? 0},${f.y ?? 0}`, f), ae.has(f.name)) {
      const [g, m] = Ra(f.direction);
      o.set(`${(f.x ?? 0) + g},${(f.y ?? 0) + m}`, f);
    }
    if (r) for (const f of r) {
      const g = `${f.x ?? 0},${f.y ?? 0}`;
      o.has(g) || o.set(g, f);
    }
    const a = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (Re.has(f.name)) {
      const [g, m] = Se[f.name] ?? [
        1,
        1
      ], y = f.x ?? 0, b = f.y ?? 0;
      for (let x = 0; x < m; x++) for (let _ = 0; _ < g; _++) a.set(`${y + _},${b + x}`, f);
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const m of n.entities) ye.has(m.name) && f.set(`${m.x ?? 0},${m.y ?? 0}`, m);
      const g = 8;
      for (const m of n.entities) {
        if (!ye.has(m.name) || m.io_type !== "input") continue;
        const [y, b] = $n(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 1; v <= g; v++) {
          const w = f.get(`${x + y * v},${_ + b * v}`);
          if (w) {
            if (ye.has(w.name) && w.name === m.name && w.direction === m.direction && w.io_type === "input") break;
            if (ye.has(w.name) && w.name === m.name && w.direction === m.direction && w.io_type === "output") {
              const [C] = Nn[m.name] ?? [
                11046960,
                14733424
              ], T = new pt(), R = Math.abs(y) > 0;
              for (let E = 1; E < v; E++) {
                const A = (x + y * E) * S, W = (_ + b * E) * S;
                R ? T.rect(A, W + S * 0.25, S, S * 0.5).fill({
                  color: C,
                  alpha: 0.25
                }) : T.rect(A + S * 0.25, W, S * 0.5, S).fill({
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
        const [y, b] = $n(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 2; v <= g; v++) {
          const w = f.get(`${x + y * v},${_ + b * v}`);
          if (!w) continue;
          const [C, T] = $n(w.direction);
          if (w.io_type !== "output" || C !== -y || T !== -b) break;
          const R = new pt();
          R.setStrokeStyle({
            width: 2,
            color: Qd,
            alpha: 0.55,
            cap: "round"
          });
          const E = (x + 0.5 + y * 0.5) * S, A = (_ + 0.5 + b * 0.5) * S, W = (v - 1) * S, j = 5, G = 3;
          let z = 0;
          for (; z < W; ) {
            const P = Math.min(z + j, W);
            R.moveTo(E + y * z, A + b * z).lineTo(E + y * P, A + b * P).stroke(), z = P + G;
          }
          t.addChild(R);
          break;
        }
      }
    }
    const l = /* @__PURE__ */ new Map(), c = [], h = /* @__PURE__ */ new Map();
    for (const f of n.entities) {
      let g;
      if (cn.has(f.name)) g = Oa(f, nu(f, o));
      else if (ye.has(f.name)) g = ru(f);
      else if (ae.has(f.name)) g = ou(f);
      else if (Tr.has(f.name)) g = au(f);
      else if (mi.has(f.name)) {
        let y = 0;
        if (f.name === "pipe") {
          const b = f.x ?? 0, x = f.y ?? 0;
          for (const [_, v, w] of [
            [
              0,
              -1,
              Ba
            ],
            [
              1,
              0,
              Na
            ],
            [
              0,
              1,
              Fa
            ],
            [
              -1,
              0,
              Wa
            ]
          ]) {
            if (v === -1 && x + v < 0) {
              y |= w;
              continue;
            }
            const C = `${b + _},${x + v}`, T = o.get(C);
            if (T && lu(T, _, v)) {
              y |= w;
              continue;
            }
            const R = a.get(C);
            R && mu(b, x, R) && (y |= w);
          }
        }
        g = cu(f, y);
      } else Ia.has(f.name) ? g = hu() : Re.has(f.name) ? g = du(f) : g = uu();
      g.x = (f.x ?? 0) * S, g.y = (f.y ?? 0) * S, s && (g.eventMode = "static", g.cursor = "pointer", g.on("click", () => s(f)));
      const m = Ec(f);
      m && (l.has(m) || l.set(m, []), l.get(m).push(g)), h.set(g, `${f.x ?? 0},${f.y ?? 0}`), c.push(g), t.addChild(g), i == null ? void 0 : i(f, [
        g
      ]);
    }
    const d = g0(n);
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
        const { downstream: y, upstream: b } = y0(m, d), x = /* @__PURE__ */ new Set([
          ...y,
          ...b
        ]), _ = x0(x, d.entityMap), v = b0(_, d.entityMap);
        for (const w of c) {
          const C = h.get(w);
          if (!C) {
            w.alpha = 0.15;
            continue;
          }
          x.has(C) ? w.alpha = 0.5 : _.has(C) ? w.alpha = 0.9 : v.has(C) ? w.alpha = 0.75 : w.alpha = 0.15;
        }
        u = new pt(), v0(u, y, b, m, d), t.addChild(u);
      },
      clearHighlight() {
        p();
      },
      chainKey: Ec
    };
  };
  function Ec(n) {
    return n.carries ? n.carries : n.recipe ? n.recipe : null;
  }
  const yi = 4096, yt = 128, on = yi / yt;
  let In = null;
  const xi = /* @__PURE__ */ new Map();
  let ln = 0, bi = null;
  function D0(n) {
    bi = n;
  }
  function mn(n, t, e, s) {
    const i = xi.get(n);
    if (i) return i;
    if (!bi) return console.warn("[atlas] getEntityTexture called before initAtlas; returning blank texture"), At.EMPTY;
    In || (In = Cr.create({
      width: yi,
      height: yi
    })), ln >= on * on && (console.warn("[atlas] atlas is full \u2014 variant will reuse slot 0:", n), ln = 0);
    const r = ln % on, o = Math.floor(ln / on), a = r * yt, l = o * yt;
    ln++;
    const c = new pt();
    s(c);
    const h = new Ct(1, 0, 0, 1, a, l);
    bi.render({
      container: c,
      target: In,
      transform: h,
      clear: false
    }), c.destroy({
      children: true
    });
    const d = new Yt(a, l, yt, yt), u = new At({
      source: In.source,
      frame: d
    });
    return xi.set(n, u), u;
  }
  function Tc(n, t, e, s) {
    const i = xi.get(n);
    if (i) return i;
    if (!bi) return console.warn("[atlas] getMultiCellTexture called before initAtlas; returning blank texture"), At.EMPTY;
    In || (In = Cr.create({
      width: yi,
      height: yi
    }));
    const r = ln % on;
    r + t > on && (ln += on - r);
    const a = ln % on, l = Math.floor(ln / on), c = a * yt, h = l * yt;
    ln = (l + e) * on;
    const d = t * yt, u = e * yt, p = new pt();
    s(p, d, u);
    const f = new Ct(1, 0, 0, 1, c, h);
    bi.render({
      container: p,
      target: In,
      transform: f,
      clear: false
    }), p.destroy({
      children: true
    });
    const g = new Yt(c, h, d, u), m = new At({
      source: In.source,
      frame: g
    });
    return xi.set(n, m), m;
  }
  function sa(n) {
    const t = `icon:${n}`, e = xi.get(t);
    if (e) return e;
    const s = `/spaghettio/pr-536/icons/${n}.png`;
    if (oe.has(s)) {
      const r = Fe.get(s);
      if (r) return mn(t, yt, yt, (a) => {
        const c = yt - 16;
        a.rect(8, 8, c, c).fill({
          texture: r
        });
      });
    }
    const i = On(n);
    return mn(t, yt, yt, (r) => {
      const o = yt / 2, a = yt / 2;
      r.circle(o, a, 7).fill({
        color: i,
        alpha: 0.85
      });
    });
  }
  function H0(n, t, e = "straight") {
    return `belt:${n}:${t}:${e}`;
  }
  function U0(n) {
    return `pipe:${n}`;
  }
  function j0(n, t, e) {
    return `ugbelt:${n}:${t}:${e}`;
  }
  function Y0(n, t) {
    return `splitter:${n}:${t}`;
  }
  function V0(n, t) {
    return `inserter:${n}:${t}`;
  }
  function X0(n) {
    return `machine:${n}`;
  }
  function q0(n) {
    return `pole:${n}`;
  }
  function K0(n) {
    return `ptg:${n}`;
  }
  const an = 3200;
  let yu = null, xu = null, bu = null;
  function Ke() {
    yu == null ? void 0 : yu();
  }
  function kr() {
    xu == null ? void 0 : xu();
  }
  function Qn() {
    bu == null ? void 0 : bu();
  }
  async function J0(n) {
    const t = new xa();
    await t.init({
      resizeTo: n,
      background: 1973790,
      antialias: true,
      autoStart: false,
      sharedTicker: false
    }), D0(t.renderer), t.ticker.add(() => t.render(), null, hi.LOW), n.appendChild(t.canvas), t.canvas.addEventListener("contextmenu", (h) => h.preventDefault());
    const e = new Jd({
      screenWidth: n.clientWidth,
      screenHeight: n.clientHeight,
      worldWidth: an,
      worldHeight: an,
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
    yu = r, xu = o, bu = a, e.on("moved", r), e.on("zoomed", r);
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
      e.resize(n.clientWidth, n.clientHeight, an, an), r();
    }), r(), {
      app: t,
      viewport: e,
      requestRender: r,
      beginAnimating: o,
      endAnimating: a
    };
  }
  const Ui = 32, Ac = 2763306, kc = 3815994;
  function Z0(n) {
    const t = new pt();
    return n.addChildAt(t, 0), t;
  }
  function Hn(n, t, e, s = 1) {
    if (n.clear(), t <= 0 || e <= 0) return;
    const i = Math.max(0, Math.min(1, s));
    if (i === 0) return;
    const r = t * Ui, a = e * Ui * i;
    for (let l = 0; l <= t; l++) {
      const c = l * Ui, h = l % 10 === 0;
      n.moveTo(c, 0).lineTo(c, a).stroke({
        width: h ? 1.5 : 1,
        color: h ? kc : Ac
      });
    }
    for (let l = 0; l <= e; l++) {
      const c = l * Ui;
      if (c > a) break;
      const h = l % 10 === 0;
      n.moveTo(0, c).lineTo(r, c).stroke({
        width: h ? 1.5 : 1,
        color: h ? kc : Ac
      });
    }
  }
  const nr = {
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
  }, Q0 = 8947848;
  function _u(n) {
    const t = n >> 16 & 255, e = n >> 8 & 255, s = n & 255;
    return `rgb(${t},${e},${s})`;
  }
  const tb = [
    {
      name: "transport-belt",
      color: nr["transport-belt"],
      throughput: 15
    },
    {
      name: "fast-transport-belt",
      color: nr["fast-transport-belt"],
      throughput: 30
    },
    {
      name: "express-transport-belt",
      color: nr["express-transport-belt"],
      throughput: 45
    }
  ];
  function wu(n) {
    for (const t of tb) if (n <= t.throughput) return t;
    return null;
  }
  const eb = 240, Mc = 80, Ga = 180, nb = 60, Pc = 100, Ic = 100, Rc = "production-graph";
  function sb(n) {
    return n <= 15 ? 13153632 : n <= 30 ? 14700624 : n <= 45 ? 5284064 : 16711816;
  }
  const Lc = new De({
    fontSize: 13,
    fontWeight: "bold",
    fill: 14737632,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: Ga - 12
  }), $c = new De({
    fontSize: 11,
    fill: 10280190,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: Ga - 12
  }), Oc = new De({
    fontSize: 10,
    fill: 16777215,
    fontFamily: "sans-serif"
  });
  function Xs(n, t) {
    const e = n.getChildByName(Rc);
    e && (e.destroy({
      children: true
    }), n.removeChild(e));
    const s = new Ht();
    if (s.label = Rc, n.addChild(s), !t || t.machines.length === 0) return s;
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
        x: Pc + (x + a) * eb,
        y: Ic + w * Mc,
        w: Ga,
        h: nb,
        machine: v
      });
    });
    const p = /* @__PURE__ */ new Map();
    c.forEach((x, _) => {
      p.set(x, {
        x: Pc,
        y: Ic + _ * Mc,
        w: 140,
        h: 40
      });
    });
    const f = new pt();
    s.addChild(f);
    for (const x of t.machines) {
      const _ = u.get(x.recipe);
      if (_) for (const v of x.inputs) {
        const w = h.get(v.item), C = w ? u.get(w.recipe) : p.get(v.item);
        if (!C) continue;
        const T = sb(v.rate), R = C.x + C.w, E = C.y + C.h * 2 / 3, A = _.x, W = _.y + _.h / 3, j = (R + A) / 2;
        f.moveTo(R, E).lineTo(j, E).lineTo(j, W).lineTo(A, W).stroke({
          color: T,
          width: 2,
          alpha: 0.85
        });
        const G = `${v.rate.toFixed(1)}/s ${v.item}`, z = (R + j) / 2, P = E - 14, D = fn.measureText(G, Oc), J = new pt();
        J.rect(z - 2, P - 1, D.width + 4, D.height + 2).fill({
          color: 1973790,
          alpha: 0.7
        }), s.addChild(J);
        const Y = new qe({
          text: G,
          style: Oc
        });
        Y.position.set(z, P), s.addChild(Y);
      }
    }
    for (const x of u.values()) {
      const _ = x.machine, v = nr[_.entity] ?? Q0, w = new pt();
      w.rect(x.x, x.y, x.w, x.h).fill({
        color: v,
        alpha: 0.6
      }).stroke({
        color: v,
        width: 2
      }), s.addChild(w);
      const C = new qe({
        text: `${_.count.toFixed(1)} \xD7 ${_.entity}`,
        style: Lc
      });
      C.position.set(x.x + 6, x.y + 6), s.addChild(C);
      const T = new qe({
        text: _.recipe,
        style: $c
      });
      T.position.set(x.x + 6, x.y + 24), s.addChild(T);
    }
    for (const [x, _] of p) {
      const v = d.get(x), w = v !== void 0 ? `${v.toFixed(1)}/s` : "", C = new pt();
      C.rect(_.x, _.y, _.w, _.h).fill({
        color: 2763306,
        alpha: 0.8
      }).stroke({
        color: 8947848,
        width: 1.5
      }), s.addChild(C);
      const T = new qe({
        text: w,
        style: Lc
      });
      T.position.set(_.x + 6, _.y + 4), s.addChild(T);
      const R = new qe({
        text: x,
        style: $c
      });
      R.position.set(_.x + 6, _.y + 20), s.addChild(R);
    }
    let g = 1 / 0, m = -1 / 0, y = 1 / 0, b = -1 / 0;
    for (const x of [
      ...u.values(),
      ...p.values()
    ]) x.x < g && (g = x.x), x.x + x.w > m && (m = x.x + x.w), x.y < y && (y = x.y), x.y + x.h > b && (b = x.y + x.h);
    return n.moveCenter((g + m) / 2, (y + b) / 2), s;
  }
  function ce(n, t) {
    return `${n},${t}`;
  }
  function Ae(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function En(n) {
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
  function we(n, t, e) {
    if (t === e) return;
    let s = n.get(t);
    s || (s = [], n.set(t, s)), s.includes(e) || s.push(e);
    let i = n.get(e);
    i || (i = [], n.set(e, i)), i.includes(t) || i.push(t);
  }
  function ib(n) {
    const t = /* @__PURE__ */ new Map();
    for (const e of n.entities) {
      const s = e.x ?? 0, i = e.y ?? 0, r = Ae(e);
      if (t.set(ce(s, i), r), ae.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South";
        t.set(ce(s + (o ? 1 : 0), i + (o ? 0 : 1)), r);
      }
      if (Re.has(e.name)) {
        const [o, a] = Se[e.name] ?? [
          1,
          1
        ];
        for (let l = 0; l < a; l++) for (let c = 0; c < o; c++) c === 0 && l === 0 || t.set(ce(s + c, i + l), r);
      }
    }
    return t;
  }
  const rb = 9;
  function vu(n) {
    const t = /* @__PURE__ */ new Map(), e = ib(n);
    for (const o of n.entities) {
      const a = Ae(o);
      t.has(a) || t.set(a, []);
    }
    const s = /* @__PURE__ */ new Map();
    for (const o of n.entities) s.set(ce(o.x ?? 0, o.y ?? 0), o);
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
      const a = o.x ?? 0, l = o.y ?? 0, c = Ae(o), [h, d] = En(o.direction);
      if (cn.has(o.name)) {
        const u = e.get(ce(a + h, l + d));
        u && we(t, c, u);
        const p = e.get(ce(a - h, l - d));
        p && we(t, c, p);
        for (const [f, g] of i) {
          if (f === h && g === d || f === -h && g === -d) continue;
          const m = s.get(ce(a + f, l + g));
          if (!m || !cn.has(m.name) && !ye.has(m.name) && !ae.has(m.name)) continue;
          const [y, b] = En(m.direction), x = ae.has(m.name) ? a + f : m.x ?? 0, _ = ae.has(m.name) ? l + g : m.y ?? 0;
          x + y === a && _ + b === l && we(t, c, Ae(m));
        }
      } else if (ye.has(o.name)) {
        if (o.io_type === "input") for (let u = 1; u <= rb; u++) {
          const p = s.get(ce(a + h * u, l + d * u));
          if (p) {
            if (ye.has(p.name) && p.name === o.name && p.io_type === "input" && p.direction === o.direction) break;
            if (ye.has(p.name) && p.name === o.name && p.io_type === "output" && p.direction === o.direction) {
              we(t, c, Ae(p));
              break;
            }
          }
        }
        else {
          const u = e.get(ce(a + h, l + d));
          u && we(t, c, u);
        }
        for (const [u, p] of i) {
          const f = s.get(ce(a + u, l + p));
          if (!f || !cn.has(f.name) && !ae.has(f.name)) continue;
          const [g, m] = En(f.direction);
          (f.x ?? 0) + g === a && (f.y ?? 0) + m === l && we(t, c, Ae(f));
        }
      } else if (ae.has(o.name)) {
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
          const y = e.get(ce(a + g + h, l + m + d));
          y && y !== c && we(t, c, y);
          const b = e.get(ce(a + g - h, l + m - d));
          b && b !== c && we(t, c, b);
        }
      }
    }
    for (const o of n.entities) {
      if (!Tr.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = Ae(o), [h, d] = En(o.direction), p = o.name === "long-handed-inserter" ? 2 : 1, f = e.get(ce(a - h * p, l - d * p)), g = e.get(ce(a + h * p, l + d * p));
      f && we(t, c, f), g && we(t, c, g);
    }
    const r = 10;
    for (const o of n.entities) {
      if (!mi.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = Ae(o);
      if (o.name === "pipe") for (const [h, d] of i) {
        const u = s.get(ce(a + h, l + d));
        if (u) {
          if (u.name === "pipe") we(t, c, Ae(u));
          else if (u.name === "pipe-to-ground") {
            const [p, f] = En(u.direction);
            h === p && d === f && we(t, c, Ae(u));
          } else if (Re.has(u.name)) {
            const p = e.get(ce(u.x ?? 0, u.y ?? 0));
            p && we(t, c, p);
          }
        }
      }
      else if (o.name === "pipe-to-ground") {
        if (o.io_type === "input") {
          const [p, f] = En(o.direction);
          for (let g = 2; g <= r; g++) {
            const m = s.get(ce(a + p * g, l + f * g));
            if (!m || m.name !== "pipe-to-ground") continue;
            const [y, b] = En(m.direction);
            if (m.io_type === "output" && y === -p && b === -f) {
              we(t, c, Ae(m));
              break;
            }
            break;
          }
        }
        const [h, d] = En(o.direction), u = s.get(ce(a - h, l - d));
        u && u.name === "pipe" && we(t, c, Ae(u));
      }
    }
    for (const o of n.entities) {
      if (!Re.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = Ae(o), [h, d] = Se[o.name] ?? [
        1,
        1
      ];
      for (let u = 0; u < d; u++) for (let p = 0; p < h; p++) for (const [f, g] of i) {
        const m = a + p + f, y = l + u + g;
        if (m >= a && m < a + h && y >= l && y < l + d) continue;
        const b = s.get(ce(m, y));
        b && mi.has(b.name) && we(t, c, Ae(b));
      }
    }
    return t;
  }
  function Cu(n, t) {
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
  const Su = 150, ob = 64, ur = 0.2, ab = 5, lb = 100, cb = 200, hb = (n) => 1 - Math.pow(1 - n, 3), db = (n) => n;
  function ji() {
    return new $x({
      dynamicProperties: {
        color: true,
        position: false,
        rotation: false,
        vertex: false,
        uvs: false
      }
    });
  }
  function Eu() {
    const n = ji(), t = new Ht(), e = ji(), s = ji(), i = ji();
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
        n.removeParticles(), t.removeChildren(), e.removeParticles(), s.removeParticles(), i.removeParticles(), he.clear(), Is.clear();
      },
      count() {
        return n.particleChildren.length + e.particleChildren.length + s.particleChildren.length + i.particleChildren.length;
      }
    };
  }
  const he = /* @__PURE__ */ new Map();
  function xs(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function ub(n, t) {
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
      if (!u || !(cn.has(u.name) || ye.has(u.name) && u.io_type === "output" || ae.has(u.name))) continue;
      const [f, g] = s[u.direction ?? "North"] ?? [
        0,
        -1
      ], m = ae.has(u.name) ? h : u.x ?? 0, y = ae.has(u.name) ? d : u.y ?? 0;
      if (!(m + f !== (n.x ?? 0) || y + g !== (n.y ?? 0))) if (u.direction === e) o = true;
      else {
        const b = f * r - g * i;
        b !== 0 && (a = b > 0 ? "cw" : "ccw");
      }
    }
    return a && !o ? a === "cw" ? "corner-cw" : "corner-ccw" : "straight";
  }
  function pb(n, t) {
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
      d && Tu(e, s, d) && (r |= l);
    }
    return r;
  }
  function Tu(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of fu(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  const fb = 9079434;
  function mb(n, t, e) {
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
        if (!d || !Tu(o, a, d)) continue;
        const u = o * S + S / 2, p = a * S + S / 2, f = S * 1.5, g = u + l * f, m = p + c * f, y = new pt();
        y.moveTo(u, p).lineTo(g, m).stroke({
          width: i,
          color: fb,
          cap: "round"
        }), s.addChild(y);
      }
    }
  }
  function za(n, t) {
    if (n.quality && (n = {
      ...n,
      quality: void 0
    }), cn.has(n.name)) {
      const s = ub(n, t.tileMap), i = H0(n.name, n.direction ?? "North", s);
      return mn(i, yt, yt, (r) => {
        const o = yt / S;
        let a = null;
        s === "corner-cw" ? a = {
          turn: "cw"
        } : s === "corner-ccw" && (a = {
          turn: "ccw"
        });
        const l = Oa(n, a);
        l.scale.set(o), r.addChild(l);
      });
    }
    if (ye.has(n.name)) {
      const s = n.io_type ?? "input", i = j0(n.name, n.direction ?? "North", s);
      return mn(i, yt, yt, (r) => {
        const o = yt / S, a = Dn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (ae.has(n.name)) {
      const s = Y0(n.name, n.direction ?? "North"), i = n.direction === "North" || n.direction === "South";
      return Tc(s, i ? 2 : 1, i ? 1 : 2, (a, l, c) => {
        const h = yt / S, d = Dn(n, t);
        d.scale.set(h), d.x = 0, d.y = 0, a.addChild(d);
      });
    }
    if (n.name === "pipe") {
      const s = pb(n, t), i = U0(s);
      return mn(i, yt, yt, (r) => {
        const o = yt / S, a = Dn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (n.name === "pipe-to-ground") {
      const s = K0(n.direction ?? "North");
      return mn(s, yt, yt, (i) => {
        const r = yt / S, o = Dn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Tr.has(n.name)) {
      const s = V0(n.name, n.direction ?? "North");
      return mn(s, yt, yt, (i) => {
        const r = yt / S, o = Dn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Ia.has(n.name)) {
      const s = q0(n.name);
      return mn(s, yt, yt, (i) => {
        const r = yt / S, o = Dn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Re.has(n.name)) {
      const [s, i] = Se[n.name] ?? [
        1,
        1
      ], r = X0(n.name);
      return Tc(r, s, i, (o, a, l) => {
        const c = `/spaghettio/pr-536/entity-frames/${n.name}.png`, h = oe.has(c) ? Fe.get(c) ?? null : null, d = 1.8, u = yt / ob;
        if (h) {
          const p = new ze(h);
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
    return mn(e, yt, yt, (s) => {
      const i = yt / S, r = Dn(n, t);
      r.scale.set(i), r.x = 0, r.y = 0, s.addChild(r);
    });
  }
  function ia(n, t, e, s = Ar()) {
    const i = xs(t);
    if (he.has(i)) return;
    const r = t.x ?? 0, o = t.y ?? 0, a = za(t, s);
    let l = S / yt, c = S / yt;
    if (ae.has(t.name)) {
      const g = t.direction === "North" || t.direction === "South", m = g ? 2 : 1, y = g ? 1 : 2;
      l = m * S / (m * yt), c = y * S / (y * yt);
    } else if (Re.has(t.name)) {
      const [g, m] = Se[t.name] ?? [
        1,
        1
      ];
      l = g * S / (g * yt), c = m * S / (m * yt);
    }
    const h = r * S, d = o * S, u = new ai({
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
      const g = sa(t.carries), m = S * 0.35, y = (S - m) / 2;
      p = new ai({
        texture: g,
        x: h + y,
        y: d + y,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: m / yt,
        scaleY: m / yt
      }), n.iconContainer.addParticle(p);
    }
    let f;
    if (t.quality && t.quality !== "normal") {
      const g = sa(`quality-${t.quality}`), m = S * 0.4;
      let y = 1;
      Re.has(t.name) ? y = (Se[t.name] ?? [
        1,
        1
      ])[1] : t.name === "substation" && (y = 2);
      const b = 1.5;
      f = new ai({
        texture: g,
        x: h + b,
        y: d + y * S - m - b,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: m / yt,
        scaleY: m / yt
      }), n.iconContainer.addParticle(f);
    }
    he.set(i, {
      entity: u,
      icon: p,
      badge: f,
      revealAt: e,
      placedEntity: t
    });
  }
  const Is = /* @__PURE__ */ new Map();
  function gb(n, t, e, s, i, r, o) {
    const a = `${t},${e}`, l = Is.get(a);
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
    }, h = Ar();
    dr(c, h);
    const d = za(c, h), u = On(i), p = new ai({
      texture: d,
      x: t * S,
      y: e * S,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: S / yt,
      scaleY: S / yt,
      tint: u
    });
    return n.ghostContainer.addParticle(p), l || Is.set(a, {
      particle: p,
      specKey: o
    }), p;
  }
  function Bc(n, t, e) {
    const s = `${t},${e}`, i = Is.get(s);
    i && (n.ghostContainer.removeParticle(i.particle), Is.delete(s));
  }
  function yb(n) {
    n.ghostContainer.removeParticles(), Is.clear();
  }
  function xb(n, t, e) {
    const s = [];
    for (const [i, r] of he.entries()) r.placedEntity.x !== t || r.placedEntity.y !== e || (Re.has(r.placedEntity.name) ? n.machineContainer.removeParticle(r.entity) : n.beltContainer.removeParticle(r.entity), r.icon && n.iconContainer.removeParticle(r.icon), he.delete(i), s.push(i));
    return s;
  }
  function bb(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const [s, i] of he.entries()) {
      const r = i.placedEntity.name;
      if (r !== "pipe" && !cn.has(r)) continue;
      const o = za(i.placedEntity, t);
      if (i.entity.texture === o) continue;
      const a = i.entity, l = new ai({
        texture: o,
        x: a.x,
        y: a.y,
        alpha: a.alpha,
        anchorX: a.anchorX,
        anchorY: a.anchorY,
        scaleX: a.scaleX,
        scaleY: a.scaleY
      });
      n.beltContainer.removeParticle(a), n.beltContainer.addParticle(l), he.set(s, {
        ...i,
        entity: l
      }), e.set(a, l);
    }
    return e;
  }
  function Au(n, t) {
    for (const e of he.values()) {
      const s = Math.min(1, Math.max(0, (t - e.revealAt) / Su));
      e.entity.alpha = s, e.icon && (e.icon.alpha = s), e.badge && (e.badge.alpha = s);
    }
  }
  function* Nc(n) {
    for (const t of he.values()) yield {
      particle: t.entity,
      iconParticle: t.icon,
      badgeParticle: t.badge,
      revealAt: t.revealAt
    };
  }
  function ku(n) {
    const t = /* @__PURE__ */ new Map();
    let e = false;
    function s() {
      e || (e = true, n.add(r), kr());
    }
    function i() {
      e && (e = false, n.remove(r), Qn());
    }
    function r() {
      const c = performance.now();
      let h = false;
      for (const [d, u] of t) {
        const p = c - u.startTime, f = Math.min(1, p / u.duration), g = u.ease(f), m = u.startAlpha + (u.targetAlpha - u.startAlpha) * g;
        u.entityParticle.alpha = m, u.iconParticle && (u.iconParticle.alpha = m), u.badgeParticle && (u.badgeParticle.alpha = m), f >= 1 ? t.delete(d) : h = true;
      }
      h || i(), Ke();
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
        duration: y ? lb : cb,
        ease: y ? hb : db
      }), s();
    }
    function a(c) {
      t.clear();
      for (const h of he.values()) h.entity.alpha = c, h.icon && (h.icon.alpha = c), h.badge && (h.badge.alpha = c);
      i(), Ke();
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
  function Mu(n) {
    let t = 0;
    for (const i of n.values()) i > t && (t = i);
    const e = Math.max(t, ab), s = /* @__PURE__ */ new Map();
    for (const [i, r] of n) {
      const o = ur + (1 - ur) * (1 - r / e);
      s.set(i, o);
    }
    return s;
  }
  function Rn(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function _b(n, t, e) {
    t.clear(), t.layout = n;
    const s = Ar();
    for (const l of n.entities) dr(l, s);
    const i = 0;
    for (const l of n.entities) ia(t, l, i, s);
    mb(t, n, s), Au(t, Su + 1);
    const r = vu(n);
    if (!e) return wb();
    const o = ku(e.ticker);
    function a(l) {
      const c = Mu(l);
      for (const h of he.values()) {
        const d = Rn(h.placedEntity), u = c.get(d) ?? ur;
        o.animateTo(d, h.entity, h.icon, h.badge, u);
      }
    }
    return {
      highlightItem(l) {
        if (o.cancelAll(1), !!l) for (const c of he.values()) {
          const h = c.placedEntity, u = (h.carries ?? h.recipe ?? null) === l ? 1 : 0.15, p = Rn(h);
          o.animateTo(p, c.entity, c.icon, c.badge, u);
        }
      },
      highlightBeltNetwork(l) {
        if (!l) {
          o.cancelAll(1);
          return;
        }
        const c = Rn(l), h = Cu(r, c);
        a(h);
      },
      clearHighlight() {
        for (const l of he.values()) {
          const c = Rn(l.placedEntity);
          o.animateTo(c, l.entity, l.icon, l.badge, 1);
        }
      },
      chainKey(l) {
        return l.carries ?? l.recipe ?? null;
      }
    };
  }
  function wb() {
    function n() {
      for (const t of he.values()) t.entity.alpha = 1, t.icon && (t.icon.alpha = 1), t.badge && (t.badge.alpha = 1);
    }
    return {
      highlightItem(t) {
        if (n(), !!t) for (const e of he.values()) {
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
  function vb(n, t) {
    const e = vu(n), s = ku(t.ticker);
    function i(r) {
      const o = Rn(r), a = Cu(e, o), l = Mu(a);
      for (const c of he.values()) {
        const h = Rn(c.placedEntity), d = l.get(h) ?? ur;
        s.animateTo(h, c.entity, c.icon, c.badge, d);
      }
    }
    return {
      highlightItem(r) {
        if (s.cancelAll(1), !!r) for (const o of he.values()) {
          const a = o.placedEntity, c = (a.carries ?? a.recipe ?? null) === r ? 1 : 0.15;
          o.entity.alpha = c, o.icon && (o.icon.alpha = c), o.badge && (o.badge.alpha = c);
        }
      },
      highlightBeltNetwork(r) {
        if (!r) {
          for (const o of he.values()) {
            const a = Rn(o.placedEntity);
            s.animateTo(a, o.entity, o.icon, o.badge, 1);
          }
          return;
        }
        i(r);
      },
      clearHighlight() {
        for (const r of he.values()) {
          const o = Rn(r.placedEntity);
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
  const Fc = 6, Cb = 18, Sb = 2, Eb = 26, Tb = -Math.PI / 3, Wc = 2, Ab = 16777215, kb = 0.55, Gc = 4, Mb = 0, Pb = 0.6;
  function Ib(n) {
    return n.carries ? cn.has(n.name) || ye.has(n.name) || mi.has(n.name) : false;
  }
  function Rb(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const l of t.external_inputs) e.set(l.item, {
      rate: l.rate,
      isFluid: !!l.is_fluid
    });
    if (e.size === 0) return [];
    const s = /* @__PURE__ */ new Map();
    for (const l of n.entities) {
      if (!Ib(l)) continue;
      const c = l.carries;
      if (!e.has(c)) continue;
      const h = l.x ?? 0, d = l.y ?? 0, u = s.get(h);
      (!u || d < u.y) && s.set(h, {
        y: d,
        carries: c
      });
    }
    if (s.size === 0) return [];
    const i = Array.from(s.entries()).filter(([, l]) => l.y === Mb).map(([l, c]) => ({
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
  function Lb(n) {
    return `${n.toFixed(1)}/s`;
  }
  function $b(n) {
    const t = n.xMax - n.xMin + 1, e = Math.min(Eb, Cb + (t - 1) * Sb), s = new Ht();
    s.eventMode = "none";
    const i = sa(n.item), r = new ze(i);
    r.width = e, r.height = e, r.x = 0, r.y = -e / 2, s.addChild(r);
    const o = new De({
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
    }), a = new qe({
      text: Lb(n.rate),
      style: o
    });
    a.x = e + Fc, a.y = -a.height / 2, s.addChild(a);
    const l = new De({
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
    }), c = new qe({
      text: xe(n.item),
      style: l
    });
    return c.alpha = Pb, c.x = a.x + a.width + Fc, c.y = -c.height / 2, s.addChild(c), s;
  }
  function Ob(n, t, e) {
    if (n.removeChildren(), !e) return;
    const s = Rb(t, e);
    if (s.length !== 0) for (const i of s) {
      const r = i.topY * S - Wc, o = i.xMin * S + Gc, a = (i.xMax - i.xMin + 1) * S - 2 * Gc;
      if (a > 0) {
        const d = new pt();
        d.rect(o, r, a, Wc).fill({
          color: Ab,
          alpha: kb
        }), n.addChild(d);
      }
      const l = $b(i);
      l.rotation = Tb;
      const c = (i.xMin + i.xMax + 1) / 2 * S, h = i.topY * S - S * 0.5;
      l.x = c, l.y = h, n.addChild(l);
    }
  }
  function Bb(n) {
    return ae.has(n.name) ? n.direction === "East" || n.direction === "West" ? [
      1,
      2
    ] : [
      2,
      1
    ] : Se[n.name] ?? [
      1,
      1
    ];
  }
  const ho = 57504;
  function zc(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map();
    for (const v of s.entities) r.set(`${v.x ?? 0},${v.y ?? 0}`, v);
    let o = null, a = false, l = [];
    const c = new pt();
    e.addChild(c);
    const h = new pt();
    e.addChild(h);
    function d(v, w) {
      const C = n.getBoundingClientRect();
      return t.toWorld(v - C.left, w - C.top);
    }
    function u(v, w) {
      if (!o) return;
      const C = d(o.sx, o.sy), T = d(v, w), R = Math.min(C.x, T.x), E = Math.min(C.y, T.y), A = Math.abs(T.x - C.x), W = Math.abs(T.y - C.y);
      c.clear(), c.rect(R, E, A, W).fill({
        color: ho,
        alpha: 0.18
      }), c.setStrokeStyle({
        width: 1,
        color: ho,
        alpha: 0.8
      }), c.rect(R, E, A, W).stroke(), Ke();
    }
    function p(v) {
      if (h.clear(), v.length !== 0) {
        h.setStrokeStyle({
          width: 1.5,
          color: ho,
          alpha: 0.9
        });
        for (const w of v) {
          const [C, T] = Bb(w), R = (w.x ?? 0) * S + 1, E = (w.y ?? 0) * S + 1;
          h.rect(R, E, C * S - 2, T * S - 2).stroke();
        }
      }
    }
    function f(v, w) {
      if (!o) return [];
      const C = d(o.sx, o.sy), T = d(v, w), R = Math.min(Math.floor(C.x / S), Math.floor(T.x / S)), E = Math.max(Math.floor(C.x / S), Math.floor(T.x / S)), A = Math.min(Math.floor(C.y / S), Math.floor(T.y / S)), W = Math.max(Math.floor(C.y / S), Math.floor(T.y / S)), j = [];
      for (let G = R; G <= E; G++) for (let z = A; z <= W; z++) {
        const P = r.get(`${G},${z}`);
        P && j.push(P);
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
        if (a) v.stopImmediatePropagation(), c.clear(), l = f(v.clientX, v.clientY), p(l), i(l), Ke();
        else if (o !== null) {
          const w = d(v.clientX, v.clientY), C = Math.floor(w.x / S), T = Math.floor(w.y / S);
          r.has(`${C},${T}`) && (l = [], h.clear(), i([]), Ke());
        }
        o = null, a = false;
      }
    };
    function b() {
      l = [], c.clear(), h.clear(), i([]), Ke();
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
  const Nb = {
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
  }, Fb = {
    codes: Nb
  }, Pu = Fb, Wb = new Map(Object.entries(Pu.codes)), Gb = new Map(Object.entries(Pu.codes).map(([n, t]) => [
    t,
    n
  ]));
  function zb(n) {
    return Wb.get(n) ?? null;
  }
  function Db(n) {
    return Gb.get(n) ?? null;
  }
  const Hb = [
    "partitioned-per-consumer",
    "partitioned-decomposed"
  ], Ub = [
    "horizontal-stack"
  ], jb = [
    "regular",
    "fast"
  ], Yb = [
    "uncommon",
    "rare",
    "epic",
    "legendary"
  ], Vb = [
    "tree"
  ], Da = [
    "2",
    "3",
    "4"
  ], Ha = [
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7"
  ], Ua = /^[sp][1-3][urel]?$/, uo = [
    "iron-plate",
    "copper-plate",
    "steel-plate",
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], Rs = [
    "stone",
    "coal",
    "water",
    "crude-oil",
    "iron-ore",
    "copper-ore"
  ], ja = "iron-gear-wheel", Ya = 10, Es = {
    crafting: "assembling-machine-3",
    smelting: "electric-furnace"
  }, _i = {
    crafting: "craft",
    smelting: "smelt"
  }, Xb = "machine", Iu = "compact", pr = "#/l/", ti = "_", fr = ",", qb = {
    pd: "partitioned-decomposed"
  }, Dc = {
    "partitioned-decomposed": "pd"
  }, Kb = {
    hs: "horizontal-stack"
  }, Hc = {
    "horizontal-stack": "hs"
  }, Jb = {
    r: "regular",
    f: "fast"
  }, Uc = {
    regular: "r",
    fast: "f"
  }, Zb = {
    u: "uncommon",
    r: "rare",
    e: "epic",
    l: "legendary"
  }, jc = {
    uncommon: "u",
    rare: "r",
    epic: "e",
    legendary: "l"
  }, Qb = {
    t: "tree"
  }, Yc = {
    tree: "t"
  }, Ru = "st", Lu = "ir", $u = "di";
  function ps(n) {
    return zb(n) ?? n;
  }
  function fs(n) {
    return Db(n);
  }
  function t_() {
    const n = window.location.hash;
    if (!n.startsWith(pr)) return null;
    const t = n.slice(pr.length), e = t.indexOf("?"), s = e >= 0 ? t.slice(0, e) : t, i = e >= 0 ? t.slice(e + 1) : "", r = s.split("/"), o = (O) => {
      const B = r[O];
      return B === void 0 || B === "" || B === ti ? null : B;
    }, a = o(0);
    let l;
    if (a) {
      const O = fs(a);
      if (O === null) return null;
      l = O;
    } else l = ja;
    const c = o(1), h = c !== null ? parseFloat(c) : NaN, d = !isNaN(h) && h > 0 ? h : Ya, u = o(2), p = {};
    if (u) {
      const O = fs(u);
      if (O === null) return null;
      p.crafting = O;
    }
    const f = o(3);
    let g;
    if (f) {
      const O = f.split(fr).filter((q) => q.length > 0), B = [];
      for (const q of O) {
        const Q = fs(q);
        if (Q === null) return null;
        B.push(Q);
      }
      g = B;
    } else g = Rs;
    const m = o(4);
    let y = null;
    if (m && (y = fs(m), y === null)) return null;
    const b = new URLSearchParams(i), x = b.get("s"), _ = x ? qb[x] ?? null : null, v = b.get("rl"), w = v ? Kb[v] ?? null : null, C = b.get("it"), T = C ? Jb[C] ?? null : null, R = b.get("q"), E = R ? Zb[R] ?? null : null, A = b.get("w"), W = A ? Qb[A] ?? null : null, j = b.get(Ru), G = j && Da.includes(j) ? j : null, z = b.get(Lu), P = z && Ha.includes(z) ? z : null, D = b.get($u) === "1", J = b.get(Iu) === "1", Y = b.get("m"), K = Y && Ua.test(Y) ? Y : null, I = b.get("ci");
    let N = [];
    if (I) {
      const O = I.split(fr).filter((B) => B.length > 0);
      for (const B of O) {
        const q = fs(B);
        if (q === null) return null;
        N.push(q);
      }
    }
    for (const [O, B] of Object.entries(_i)) {
      if (O === "crafting") continue;
      const q = b.get(B);
      if (!q) continue;
      const Q = fs(q);
      if (Q === null) return null;
      p[O] = Q;
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
      wireMode: W,
      stacking: G,
      inserterCapacity: P,
      directInsertion: D,
      compactLayout: J,
      modules: K,
      customInputs: N
    };
  }
  function e_() {
    const n = new URLSearchParams(window.location.search), t = n.get("item") ?? ja, e = parseFloat(n.get("rate") ?? ""), s = isNaN(e) || e <= 0 ? Ya : e, i = {};
    for (const [j, G] of Object.entries(_i)) {
      const z = n.get(G);
      z && (i[j] = z);
    }
    const r = n.get(Xb);
    r && !i.crafting && (i.crafting = r);
    const o = n.get("in"), a = o ? o.split(",").filter((j) => j.length > 0) : Rs, l = n.get("belt"), c = n.get("strategy");
    let h = c && Hb.includes(c) ? c : null;
    h === "partitioned-per-consumer" && (h = "partitioned-decomposed");
    const d = n.get("row_layout"), u = d && Ub.includes(d) ? d : null, p = n.get("inserter_tier"), f = p && jb.includes(p) ? p : null, g = n.get("wire_mode"), m = g && Vb.includes(g) ? g : null, y = n.get("quality"), b = y && Yb.includes(y) ? y : null, x = n.get("stacking"), _ = x && Da.includes(x) ? x : null, v = n.get("inserter_capacity"), w = v && Ha.includes(v) ? v : null, C = n.get("direct_insertion") === "1", T = n.get("compact") === "1", R = n.get("modules"), E = R && Ua.test(R) ? R : null, A = n.get("ci"), W = A ? A.split(",").filter((j) => j.length > 0) : [];
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
      inserterCapacity: w,
      directInsertion: C,
      compactLayout: T,
      modules: E,
      customInputs: W
    };
  }
  function n_() {
    return t_() ?? e_();
  }
  function s_() {
    if (window.location.hash.startsWith(pr)) return true;
    const n = new URLSearchParams(window.location.search);
    return n.has("item") || n.has("rate") || n.has("machine") || n.has("in") || n.has("belt");
  }
  function i_(n) {
    for (const [t, e] of Object.entries(_i)) {
      const s = n[t];
      if (s && s !== Es[t]) return false;
    }
    for (const t of Object.keys(n)) if (!(t in _i)) return false;
    return true;
  }
  function r_(n) {
    const t = ps(n.item), e = String(n.rate), s = n.machines.crafting, i = !s || s === Es.crafting ? ti : ps(s), r = n.inputs.length === Rs.length && n.inputs.every((u, p) => u === Rs[p]), o = n.inputs.length === 0 || r ? ti : n.inputs.map(ps).join(fr), a = n.belt ? ps(n.belt) : ti, l = new URLSearchParams();
    n.strategy && Dc[n.strategy] && l.set("s", Dc[n.strategy]), n.rowLayout && Hc[n.rowLayout] && l.set("rl", Hc[n.rowLayout]), n.inserterTier && Uc[n.inserterTier] && l.set("it", Uc[n.inserterTier]), n.quality && jc[n.quality] && l.set("q", jc[n.quality]), n.wireMode && Yc[n.wireMode] && l.set("w", Yc[n.wireMode]), n.stacking && Da.includes(n.stacking) && l.set(Ru, n.stacking), n.inserterCapacity && Ha.includes(n.inserterCapacity) && l.set(Lu, n.inserterCapacity), n.compactLayout && l.set(Iu, "1"), n.directInsertion && l.set($u, "1"), n.modules && Ua.test(n.modules) && l.set("m", n.modules), n.customInputs.length > 0 && l.set("ci", n.customInputs.map(ps).join(fr));
    for (const [u, p] of Object.entries(_i)) {
      if (u === "crafting") continue;
      const f = n.machines[u];
      f && f !== Es[u] && l.set(p, ps(f));
    }
    const c = [
      t,
      e,
      i,
      o,
      a
    ], h = l.toString();
    if (h.length === 0) for (; c.length > 2 && c[c.length - 1] === ti; ) c.pop();
    let d = `${pr}${c.join("/")}`;
    return h.length > 0 && (d += `?${h}`), d;
  }
  function o_(n) {
    const t = n.item === ja && n.rate === Ya && i_(n.machines) && n.inputs.length === Rs.length && n.inputs.every((s, i) => s === Rs[i]) && !n.belt && !n.strategy && !n.rowLayout && !n.inserterTier && n.customInputs.length === 0, e = window.location.pathname;
    if (t) {
      history.replaceState(null, "", e);
      return;
    }
    history.replaceState(null, "", e + r_(n));
  }
  const po = "[INCOMPATIBLE_MACHINE]";
  function xn(n, t = 14) {
    const e = document.createElement("img");
    return e.src = `/spaghettio/pr-536/icons/${n}.png`, e.width = t, e.height = t, e.style.cssText = "image-rendering:pixelated", e.onerror = () => {
      e.style.display = "none";
    }, e;
  }
  function a_(n, t) {
    const e = document.createElement("option");
    return e.value = n, e.textContent = xe(n), n === t && (e.selected = true), e;
  }
  function Yi(n, t, e) {
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
  function Vc(n, t, e, s) {
    for (const i of t) {
      const r = document.createElement("div");
      r.className = `sb-machine-flow ${e}`, s && r.appendChild(document.createTextNode(s)), r.appendChild(xn(i.item, 13)), r.appendChild(document.createTextNode(xe(i.item)));
      const o = document.createElement("span");
      o.className = "flow-rate";
      const a = wu(i.rate), l = a ? _u(a.color) : "#f88";
      o.style.color = l, o.textContent = `${i.rate.toFixed(1)}/s`, r.appendChild(o), n.appendChild(r);
    }
  }
  const Xc = /* @__PURE__ */ new Set([
    "water",
    "crude-oil",
    "petroleum-gas",
    "light-oil",
    "heavy-oil",
    "sulfuric-acid",
    "lubricant",
    "steam"
  ]), Ou = "spaghettio-recent-items", qc = 5;
  function Bu() {
    try {
      const n = localStorage.getItem(Ou);
      return n ? JSON.parse(n) : [];
    } catch {
      return [];
    }
  }
  function l_(n) {
    const t = Bu().filter((e) => e !== n);
    t.unshift(n), t.length > qc && (t.length = qc);
    try {
      localStorage.setItem(Ou, JSON.stringify(t));
    } catch {
    }
  }
  function c_(n, t, e) {
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
        a.appendChild(xn(s, 14));
        const x = document.createElement("span");
        x.textContent = xe(s), a.appendChild(x);
      } else {
        const x = document.createElement("span");
        x.className = "sb-picker-placeholder", x.textContent = "Select item\u2026", a.appendChild(x);
      }
    }
    function p(x) {
      const _ = document.createElement("div");
      _.className = "sb-picker-item" + (x === s ? " selected" : ""), _.dataset.slug = x, _.appendChild(xn(x, 14));
      const v = document.createElement("span");
      return v.textContent = xe(x), _.appendChild(v), _.addEventListener("mousedown", (w) => {
        w.preventDefault(), g(x);
      }), _;
    }
    function f(x) {
      d.innerHTML = "", r = null;
      const _ = x.trim().toLowerCase(), v = _ ? n.filter((w) => w.includes(_) || xe(w).toLowerCase().includes(_)) : n;
      if (!_) {
        const w = Bu().filter((C) => n.includes(C));
        if (w.length > 0) {
          const C = document.createElement("div");
          C.className = "sb-picker-section-label", C.textContent = "Recent", d.appendChild(C);
          for (const R of w) d.appendChild(p(R));
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
      s = x, l_(x), o.classList.remove("item-invalid"), u(), y(), e(x);
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
  function h_(n, t, e) {
    var _a2;
    n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "sidebar-inner";
    const { section: i, body: r } = Yi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="2"/></svg>', "Target"), o = t.allProducibleItems(), a = new Set(o);
    function l(H, et) {
      const tt = document.createElement("div");
      tt.className = "sb-field";
      const St = document.createElement("span");
      return St.className = "sb-field-label", St.textContent = H, tt.appendChild(St), et.style.flex = "1", et.style.minWidth = "0", tt.appendChild(et), tt;
    }
    const c = c_(o, "", () => $t());
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
      const et = document.createElement("select");
      et.className = "sb-select", et.dataset.cat = H.category;
      const tt = Es[H.category] ?? "";
      for (const St of H.options) {
        const zt = a_(St.value, tt);
        St.disabled && (zt.disabled = true), St.title && (zt.title = St.title), et.appendChild(zt);
      }
      r.appendChild(l(H.label, et)), u.set(H.category, et);
    }
    const p = u.get("crafting");
    for (const H of d) {
      const et = document.createElement("span");
      et.className = "sb-machine-readonly", et.textContent = xe(H.machine), r.appendChild(l(H.label, et));
    }
    function f() {
      const H = {};
      for (const [et, tt] of u) tt.value && (H[et] = tt.value);
      return H;
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
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, g.appendChild(tt);
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
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, m.appendChild(tt);
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
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, y.appendChild(tt);
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
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, b.appendChild(tt);
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
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, x.appendChild(tt);
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
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, v.appendChild(tt);
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
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, w.appendChild(tt);
    }), r.appendChild(l("Belt stacking", w));
    const C = document.createElement("select");
    C.className = "sb-select", C.title = "Inserter capacity bonus research level: raises inserter hand sizes", [
      [
        "Default (L2, red+green)",
        ""
      ],
      [
        "Unresearched (L0)",
        "0"
      ],
      [
        "Level 1",
        "1"
      ],
      [
        "Level 2",
        "2"
      ],
      [
        "Level 3",
        "3"
      ],
      [
        "Level 4",
        "4"
      ],
      [
        "Level 5",
        "5"
      ],
      [
        "Level 6",
        "6"
      ],
      [
        "Level 7 (max)",
        "7"
      ]
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, C.appendChild(tt);
    }), r.appendChild(l("Inserter research", C));
    const T = document.createElement("input");
    T.type = "checkbox", T.className = "sb-checkbox", T.title = "FORCE direct insertion (RFC-053). DI couples a producer straight into its consumer with inserters, so the shared item never touches a belt \u2014 denser, and it lifts the belt-interface throughput ceiling. Leave this UNCHECKED for normal use: the engine already tries DI on every layout and keeps it wherever it is strictly better. Checking this skips that safety comparison and can produce a worse layout; it exists for A/B debugging.", r.appendChild(l("Direct insertion", T));
    const R = document.createElement("input");
    R.type = "checkbox", R.className = "sb-checkbox", R.title = "Compact the finished layout (RFC-057): replace long surface belt runs with underground pairs, then delete redundant rows and columns. Each step is kept only if the layout still validates no worse, so the worst case is no change. On the largest corpus layout this is -26% bounding box and -45% entities.", r.appendChild(l("Compact layout", R));
    const E = document.createElement("select");
    E.className = "sb-select", [
      [
        "Pooled (default)",
        ""
      ],
      [
        "Partitioned + decomposed",
        "partitioned-decomposed"
      ]
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, E.appendChild(tt);
    }), r.appendChild(l("Strategy", E));
    const A = document.createElement("select");
    A.className = "sb-select", [
      [
        "Vertical split (today)",
        ""
      ],
      [
        "Horizontal stack (RFC)",
        "horizontal-stack"
      ]
    ].forEach(([H, et]) => {
      const tt = document.createElement("option");
      tt.value = et, tt.textContent = H, A.appendChild(tt);
    }), r.appendChild(l("Row layout", A));
    const W = document.createElement("div");
    W.className = "sb-field";
    const j = document.createElement("span");
    j.className = "sb-field-label", j.textContent = "Rate", W.appendChild(j);
    const G = document.createElement("input");
    G.type = "number", G.className = "sb-input", G.step = "0.5", G.min = "0.1", G.style.cssText = "flex:1;min-width:0", G.placeholder = "10", W.appendChild(G);
    const z = document.createElement("span");
    z.className = "sb-rate-suffix", z.textContent = "/s", W.appendChild(z), r.appendChild(W), s.appendChild(i);
    const { section: P, body: D } = Yi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="6" rx="1"/><line x1="5" y1="8" x2="11" y2="8"/></svg>', "Inputs"), J = document.createElement("div");
    J.className = "sb-tags";
    const Y = /* @__PURE__ */ new Map();
    uo.forEach((H) => {
      const et = document.createElement("label");
      et.className = `sb-tag${Xc.has(H) ? " fluid" : ""}`;
      const tt = document.createElement("span");
      tt.className = "sb-tag-check", tt.textContent = "\u2713";
      const St = document.createElement("input");
      St.type = "checkbox", St.value = H, St.style.display = "none", Y.set(H, St), et.appendChild(tt), et.appendChild(xn(H, 14)), et.appendChild(document.createTextNode(xe(H))), et.appendChild(St), St.addEventListener("change", () => {
        et.classList.toggle("active", St.checked);
      }), J.appendChild(et);
    }), D.appendChild(J);
    let K = [];
    const I = document.createElement("div");
    I.className = "sb-tags sb-custom-tags", D.appendChild(I);
    const N = document.createElement("datalist");
    N.id = "spaghettio-custom-inputs-datalist";
    const O = new Set(uo);
    o.filter((H) => !O.has(H)).forEach((H) => {
      const et = document.createElement("option");
      et.value = H, N.appendChild(et);
    }), D.appendChild(N);
    const B = document.createElement("input");
    B.type = "text", B.className = "sb-input sb-custom-input-field", B.setAttribute("list", "spaghettio-custom-inputs-datalist"), B.autocomplete = "off", B.placeholder = "+ add input\u2026", D.appendChild(B);
    function q(H) {
      const et = document.createElement("div");
      et.className = `sb-tag sb-custom-tag active${Xc.has(H) ? " fluid" : ""}`, et.dataset.item = H, et.appendChild(xn(H, 14)), et.appendChild(document.createTextNode(xe(H)));
      const tt = document.createElement("span");
      tt.className = "sb-tag-remove", tt.textContent = "\xD7", tt.addEventListener("click", (St) => {
        St.stopPropagation(), K = K.filter((zt) => zt !== H), et.remove(), $t();
      }), et.appendChild(tt), I.appendChild(et);
    }
    function Q(H) {
      const et = H.trim();
      !et || !a.has(et) || O.has(et) || K.includes(et) || (K.push(et), q(et), B.value = "", $t());
    }
    B.addEventListener("keydown", (H) => {
      H.key === "Enter" && Q(B.value);
    }), B.addEventListener("change", () => {
      Q(B.value);
    }), s.appendChild(P);
    const { section: ut, body: ft, countEl: mt } = Yi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>', "Solver", ""), wt = document.createElement("div");
    ft.appendChild(wt), s.appendChild(ut);
    const V = document.createElement("div");
    V.className = "sb-actions", V.style.display = "none";
    const st = document.createElement("button");
    st.className = "sb-btn sb-btn-secondary", st.textContent = "Copy Blueprint", st.style.flex = "1", V.appendChild(st);
    const it = document.createElement("div");
    it.className = "sb-copy-status", V.appendChild(it), ft.appendChild(V);
    const { section: lt, body: Et, countEl: Nt } = Yi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.8" fill="currentColor" stroke="none"/></svg>', "Validation", "");
    lt.style.display = "none", s.appendChild(lt), n.appendChild(s);
    const xt = n_();
    c.setValue(xt.item), G.value = String(xt.rate);
    const jt = /* @__PURE__ */ new Set([
      "assembling-machine-1",
      "assembling-machine-2",
      "assembling-machine-3"
    ]);
    for (const [H, et] of u) {
      const tt = xt.machines[H], St = new Set(Array.from(et.options).filter((zt) => !zt.disabled).map((zt) => zt.value));
      et.value = tt && St.has(tt) ? tt : Es[H] ?? ((_a2 = et.options[0]) == null ? void 0 : _a2.value) ?? "";
    }
    Y.forEach((H, et) => {
      H.checked = xt.inputs.includes(et);
      const tt = H.closest(".sb-tag");
      tt && tt.classList.toggle("active", H.checked);
    }), xt.belt && (g.value = xt.belt), xt.strategy && (E.value = xt.strategy), xt.rowLayout && (A.value = xt.rowLayout), xt.inserterTier && (m.value = xt.inserterTier), xt.quality && (y.value = xt.quality), xt.modules && (b.value = xt.modules.slice(0, 2), x.value = xt.modules.slice(2)), xt.wireMode && (v.value = xt.wireMode), xt.stacking && (w.value = xt.stacking), xt.inserterCapacity && (C.value = xt.inserterCapacity), T.checked = xt.directInsertion === true, R.checked = xt.compactLayout === true;
    for (const H of xt.customInputs) a.has(H) && !O.has(H) && !K.includes(H) && (K.push(H), q(H));
    const Vt = document.createElement("div");
    Vt.className = "sb-config-error", Vt.style.display = "none", wt.before(Vt);
    function te(H) {
      H ? (Vt.textContent = H, Vt.style.display = "") : (Vt.textContent = "", Vt.style.display = "none");
    }
    let Lt = null, be = xt.item, Xt = null, _e = 0;
    function $t() {
      Lt !== null && clearTimeout(Lt), Lt = setTimeout(() => {
        fe().catch((H) => console.error("runSolve failed:", H));
      }, 150);
    }
    async function fe() {
      var _a3;
      const H = c.getValue(), et = parseFloat(G.value), tt = uo.filter((L) => {
        var _a4;
        return (_a4 = Y.get(L)) == null ? void 0 : _a4.checked;
      }), St = [
        ...tt,
        ...K
      ];
      if (!a.has(H)) {
        c.setInvalid(true);
        return;
      }
      if (c.setInvalid(false), isNaN(et) || et <= 0) return;
      if (H !== be) {
        const L = t.defaultMachineForItem(H, p.value);
        jt.has(L) && (p.value = L), be = H;
      }
      const zt = f();
      o_({
        item: H,
        rate: et,
        compactLayout: R.checked,
        machines: zt,
        inputs: tt,
        belt: g.value || null,
        strategy: E.value || null,
        rowLayout: A.value || null,
        inserterTier: m.value || null,
        quality: y.value || null,
        wireMode: v.value || null,
        stacking: w.value || null,
        inserterCapacity: C.value || null,
        directInsertion: T.checked,
        modules: _(),
        customInputs: K
      });
      const k = ++_e;
      wt.innerHTML = "", te(null), Xt = null, V.style.display = "none";
      let M;
      try {
        M = await t.solve(H, et, St, zt, zt.crafting ?? Es.crafting, y.value || void 0, _() ?? void 0);
      } catch (L) {
        if (k !== _e) return;
        e.renderGraph(null), mt && (mt.textContent = "error");
        const Z = String(L instanceof Error ? L.message : L);
        if (Z.includes(po)) {
          const U = Z.indexOf(po), rt = Z.slice(U + po.length).trim();
          te(rt);
        } else {
          const U = document.createElement("div");
          U.className = "sb-result-error", U.textContent = Z, wt.appendChild(U);
        }
        return;
      }
      if (k !== _e) return;
      d_(wt, M), e.renderGraph(M);
      const X = M.machines.reduce((L, Z) => L + Math.ceil(Z.count), 0);
      mt && (mt.textContent = `${X} machines`);
      const ot = /* @__PURE__ */ new Set();
      for (const L of M.machines) {
        for (const Z of L.inputs) ot.add(Z.item);
        for (const Z of L.outputs) ot.add(Z.item);
      }
      for (const L of M.external_inputs) ot.add(L.item);
      for (const L of M.external_outputs) ot.add(L.item);
      if (await pu(Array.from(ot)), k !== _e) return;
      let $;
      try {
        const L = g.value || void 0, Z = E.value || void 0, U = A.value || void 0, rt = m.value || void 0, gt = y.value || void 0, Mt = v.value || void 0, nt = w.value || void 0, Tt = C.value || void 0, vt = T.checked, Wt = R.checked, qt = e.startStreaming();
        $ = await t.buildLayoutStreaming(M, L, Z, U, rt, gt, Mt, nt, Tt, vt, Wt, qt);
      } catch (L) {
        if (k !== _e) return;
        const Z = document.createElement("div");
        Z.className = "sb-result-error", Z.textContent = `Layout error: ${L}`, wt.appendChild(Z);
        return;
      }
      k === _e && (Xt = $, eu(M.machines), e.renderLayout($, M), V.style.display = ((_a3 = $.warnings) == null ? void 0 : _a3.length) ? "none" : "flex");
    }
    st.addEventListener("click", async () => {
      if (!Xt) return;
      const H = await t.exportBlueprint(Xt, c.getValue());
      await navigator.clipboard.writeText(H), it.textContent = "Copied!", setTimeout(() => {
        it.textContent = "";
      }, 2e3);
    }), G.addEventListener("input", $t);
    for (const H of u.values()) H.addEventListener("change", $t);
    return g.addEventListener("change", $t), E.addEventListener("change", $t), A.addEventListener("change", $t), m.addEventListener("change", $t), y.addEventListener("change", $t), T.addEventListener("change", $t), R.addEventListener("change", $t), b.addEventListener("change", $t), x.addEventListener("change", $t), v.addEventListener("change", $t), w.addEventListener("change", $t), C.addEventListener("change", $t), Y.forEach((H) => H.addEventListener("change", $t)), fe().catch((H) => console.error("runSolve failed:", H)), {
      getParams() {
        const H = c.getValue(), et = parseFloat(G.value);
        return !H || isNaN(et) || et <= 0 ? null : {
          item: H,
          rate: et
        };
      },
      setParams(H, et) {
        c.setValue(H.item), G.value = String(H.rate), H.machine && jt.has(H.machine) ? p.value = H.machine : p.value = "assembling-machine-3", H.inputs && Y.forEach((tt, St) => {
          tt.checked = H.inputs.includes(St);
          const zt = tt.closest(".sb-tag");
          zt && zt.classList.toggle("active", tt.checked);
        }), H.belt ? g.value = H.belt : g.value = "", I.innerHTML = "", K = [];
        for (const tt of H.customInputs ?? []) a.has(tt) && !O.has(tt) && !K.includes(tt) && (K.push(tt), q(tt));
        be = H.item, (et == null ? void 0 : et.skipAutoSolve) || $t();
      },
      updateValidation(H, et, tt) {
        if (Et.innerHTML = "", H.length === 0) {
          lt.style.display = "none", Nt && (Nt.textContent = "");
          return;
        }
        lt.style.display = "";
        const St = H.filter((M) => M.severity === "Error").length, zt = H.length - St;
        Nt && (St > 0 ? (Nt.textContent = `${St} error${St !== 1 ? "s" : ""}`, Nt.style.color = "#f66") : (Nt.textContent = `${zt} warning${zt !== 1 ? "s" : ""}`, Nt.style.color = "#fa0"));
        const k = /* @__PURE__ */ new Map();
        for (const M of H) {
          let X = k.get(M.category);
          X || (X = [], k.set(M.category, X)), X.push(M);
        }
        for (const [M, X] of k) {
          const $ = X.some((vt) => vt.severity === "Error") ? "#f44" : "#fa0", L = X.find((vt) => vt.x != null && vt.y != null), Z = document.createElement("div");
          Z.className = "sb-val-group";
          const U = document.createElement("div");
          U.className = "sb-val-group-header";
          const rt = document.createElement("span");
          rt.className = "sb-val-group-chevron", rt.textContent = "\u25BE", rt.addEventListener("click", (vt) => {
            vt.stopPropagation();
            const Wt = Tt.style.display === "none";
            Tt.style.display = Wt ? "" : "none", rt.textContent = Wt ? "\u25BE" : "\u25B8";
          }), U.appendChild(rt);
          const gt = document.createElement("span");
          gt.className = "sb-val-group-dot", gt.style.background = $, U.appendChild(gt);
          const Mt = document.createElement("span");
          Mt.className = "sb-val-group-name", Mt.textContent = M, U.appendChild(Mt);
          const nt = document.createElement("span");
          nt.className = "sb-val-group-count", nt.textContent = String(X.length), U.appendChild(nt);
          const Tt = document.createElement("div");
          if (Tt.className = "sb-val-group-body", tt) {
            const vt = X.filter((Wt) => Wt.detail != null);
            if (vt.length > 0) {
              const Wt = /* @__PURE__ */ new Map();
              for (const re of vt) {
                const le = tt(re) ?? "unattributed";
                Wt.set(le, (Wt.get(le) ?? 0) + 1);
              }
              const qt = document.createElement("div");
              qt.className = "sb-val-cause-rollup", qt.style.cssText = "font-size:11px;color:#b90;padding:1px 0 2px 22px", qt.textContent = [
                ...Wt.entries()
              ].sort((re, le) => le[1] - re[1]).map(([re, le]) => `${le} \xD7 ${re}`).join(" \xB7 "), Tt.appendChild(qt);
            }
          }
          L && (U.classList.add("clickable"), U.addEventListener("click", () => {
            et(L.x, L.y);
          }));
          for (const vt of X) {
            const Wt = document.createElement("div"), qt = vt.x != null && vt.y != null;
            Wt.className = "sb-val-issue" + (qt ? " clickable" : "");
            const re = document.createElement("span");
            if (re.className = "sb-val-issue-msg", re.textContent = vt.message, Wt.appendChild(re), qt) {
              const le = document.createElement("span");
              le.className = "sb-val-issue-coord", le.textContent = `${vt.x}, ${vt.y}`, Wt.appendChild(le), Wt.addEventListener("click", (Fn) => {
                Fn.stopPropagation();
                const rs = Wt.classList.contains("pinned");
                Et.querySelectorAll(".sb-val-issue.pinned").forEach((Mr) => Mr.classList.remove("pinned")), rs || Wt.classList.add("pinned"), et(vt.x, vt.y);
              });
            } else Wt.style.opacity = "0.6";
            Tt.appendChild(Wt);
          }
          Z.appendChild(U), Z.appendChild(Tt), Et.appendChild(Z);
        }
      }
    };
  }
  function d_(n, t) {
    if (t.external_inputs.length > 0) {
      const l = document.createElement("div");
      l.className = "sb-ext-section-title", l.textContent = "External inputs", n.appendChild(l);
      for (const h of t.external_inputs) {
        const d = document.createElement("div");
        d.className = "sb-ext-flow", d.appendChild(xn(h.item, 14)), d.appendChild(document.createTextNode(xe(h.item)));
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
      u.className = "sb-machine-group-header", u.appendChild(xn(l, 16));
      const p = document.createElement("span");
      p.className = "sb-machine-group-name", p.textContent = xe(l), u.appendChild(p);
      const f = document.createElement("span");
      f.className = "sb-machine-group-count", f.textContent = `\xD7${h}`, u.appendChild(f), d.appendChild(u);
      const g = document.createElement("div");
      g.className = "sb-machine-group-body";
      for (const m of c) {
        const y = document.createElement("div");
        y.className = "sb-machine-flow", y.style.cssText = "color:#6b7280;margin-bottom:2px", y.appendChild(document.createTextNode("\u2192 ")), y.appendChild(xn(m.recipe, 13)), y.appendChild(document.createTextNode(xe(m.recipe))), g.appendChild(y), Vc(g, m.inputs, "flow-in", "\u25B6 "), Vc(g, m.outputs, "flow-out", "\u25C0 ");
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
      c.style.cssText = "display:flex;align-items:center;gap:4px;padding:2px 0;font-size:11px;color:#b5cea8", c.appendChild(xn(l.item, 13)), c.appendChild(document.createTextNode(xe(l.item)));
      const h = wu(l.rate);
      if (h) {
        const d = _u(h.color);
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
  let Ge = null, Va = 0;
  const ss = /* @__PURE__ */ new Map();
  let pe = null;
  const ra = "spaghettio:sat-cache:v1";
  let yn = new Uint8Array(0);
  function u_() {
    try {
      const n = localStorage.getItem(ra);
      if (!n) return new Uint8Array(0);
      const t = atob(n), e = new Uint8Array(t.length);
      for (let s = 0; s < t.length; s++) e[s] = t.charCodeAt(s);
      return e;
    } catch (n) {
      return console.warn("[engine] could not read SAT cache from localStorage", n), new Uint8Array(0);
    }
  }
  function p_(n) {
    let t = "";
    for (let i = 0; i < n.length; i += 8192) t += String.fromCharCode.apply(null, Array.from(n.subarray(i, i + 8192)));
    const s = btoa(t);
    try {
      localStorage.setItem(ra, s);
    } catch (i) {
      if (i instanceof DOMException && (i.name === "QuotaExceededError" || i.code === 22)) {
        console.warn("[engine] SAT cache quota exceeded \u2014 clearing");
        try {
          localStorage.removeItem(ra);
        } catch {
        }
        yn = new Uint8Array(0);
      } else console.warn("[engine] failed to persist SAT cache", i);
    }
  }
  function f_(n) {
    const t = new Uint8Array(yn.length + n.length);
    t.set(yn, 0), t.set(n, yn.length), yn = t, p_(yn);
  }
  let oa = [], aa = [], Nu = /* @__PURE__ */ new Map(), Fu = /* @__PURE__ */ new Map(), la = /* @__PURE__ */ new Set(), ca = 0;
  function wn(n) {
    ca += n;
    for (const t of la) t(ca);
  }
  function m_(n) {
    return la.add(n), n(ca), () => la.delete(n);
  }
  function ke(n, t) {
    if (!Ge) throw new Error("Engine not initialized \u2014 call initEngine() first");
    const e = ++Va;
    return wn(1), new Promise((s, i) => {
      ss.set(e, {
        resolve: (r) => {
          wn(-1), pe === e && (pe = null), s(r);
        },
        reject: (r) => {
          wn(-1), pe === e && (pe = null), i(r);
        },
        onEvent: t
      }), Ge.postMessage({
        id: e,
        ...n
      });
    });
  }
  async function Wu() {
    if (Ge) return;
    if (Ge = new Worker(new URL("/spaghettio/pr-536/assets/engine.worker-C18zgkyL.js", import.meta.url), {
      type: "module",
      name: "spaghettio-engine"
    }), Ge.onmessage = (e) => {
      const { id: s } = e.data, i = ss.get(s);
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
        ss.delete(s), e.data.ok ? i.resolve(e.data.result) : i.reject(new Error(e.data.error));
      }
    }, Ge.onerror = (e) => {
      console.error("[engine.worker] error", e);
    }, await ke({
      method: "init"
    }), yn = u_(), yn.length > 0) try {
      const e = await ke({
        method: "seedZoneCache",
        bytes: yn
      });
      globalThis.__TRACE_LOGS === true && console.log(`[engine] seeded ${e} SAT zone records from localStorage`);
    } catch (e) {
      console.warn("[engine] seedZoneCache failed; persistence disabled this session", e);
    }
    oa = await ke({
      method: "allProducibleItems"
    }), aa = await ke({
      method: "allProducerMachines"
    });
    const n = await ke({
      method: "defaultMachinesForItems",
      items: oa,
      fallback: "assembling-machine-3"
    });
    Nu = new Map(n);
    const t = await ke({
      method: "moduleSlotsForEntities",
      entities: [
        ...aa,
        "beacon",
        "lab",
        "biolab",
        "electric-mining-drill",
        "big-mining-drill",
        "pumpjack"
      ]
    });
    Fu = new Map(t);
  }
  async function g_(n, t, e, s, i, r, o) {
    return pe !== null && await Xa(), ke({
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
  function y_() {
    return oa;
  }
  function x_() {
    return aa;
  }
  function b_(n, t, e, s, i, r, o, a, l, c, h) {
    return ke({
      method: "layout",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null,
      quality: r ?? null,
      wireMode: o ?? null,
      stacking: a ?? null,
      inserterCapacity: l ?? null,
      directInsertion: c ? "on" : void 0,
      compactLayout: h ?? false
    });
  }
  function __(n, t, e, s, i, r, o, a, l, c, h) {
    return ke({
      method: "layoutTraced",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null,
      quality: r ?? null,
      wireMode: o ?? null,
      stacking: a ?? null,
      inserterCapacity: l ?? null,
      directInsertion: c ? "on" : void 0,
      compactLayout: h ?? false
    });
  }
  async function Xa() {
    if (!Ge) return;
    Ge.terminate(), Ge = null;
    const n = new Error("Engine superseded by a newer request");
    for (const [, t] of ss) t.reject(n);
    ss.clear(), pe = null, await Wu();
  }
  async function w_(n, t, e, s, i, r, o, a, l, c, h, d) {
    pe !== null && await Xa();
    const u = ++Va;
    return pe = u, wn(1), new Promise((p, f) => {
      ss.set(u, {
        resolve: (m) => {
          wn(-1), pe === u && (pe = null), p(m);
        },
        reject: (m) => {
          wn(-1), pe === u && (pe = null), f(m);
        },
        onEvent: d
      });
      const g = globalThis.__TRACE_LOGS === true;
      Ge.postMessage({
        id: u,
        method: "layoutStreaming",
        result: n,
        maxBeltTier: t ?? null,
        strategy: e ?? null,
        rowLayout: s ?? null,
        maxInserterTier: i ?? null,
        quality: r ?? null,
        wireMode: o ?? null,
        stacking: a ?? null,
        inserterCapacity: l ?? null,
        directInsertion: c ? "on" : void 0,
        compactLayout: h ?? false,
        traceLogs: g
      });
    });
  }
  function v_(n, t) {
    return ke({
      method: "exportBlueprint",
      layout: n,
      label: t
    });
  }
  function C_(n, t) {
    return Nu.get(n) ?? t;
  }
  function S_(n) {
    return Fu.get(n) ?? 0;
  }
  function E_(n, t) {
    return ke({
      method: "validateLayout",
      layout: n,
      solverResult: t
    });
  }
  function T_(n, t) {
    return ke({
      method: "solveFixture",
      fixtureJson: n,
      pinsJson: JSON.stringify(t)
    });
  }
  function A_(n) {
    return ke({
      method: "parseBlueprint",
      bp: n
    });
  }
  async function Gu(n, t, e, s, i = 0) {
    if (pe !== null && await Xa(), !Ge) throw new Error("Engine not initialized");
    const r = ++Va;
    return pe = r, wn(1), new Promise((o, a) => {
      ss.set(r, {
        resolve: (l) => {
          wn(-1), pe === r && (pe = null), o(l);
        },
        reject: (l) => {
          wn(-1), pe === r && (pe = null), a(l);
        },
        onEvent: (l) => {
          const c = l;
          if (c.phase === "SatImprovement" && c.data) s(c.data);
          else if (c.phase === "SatOptimumProven" && c.data) {
            const h = c.data, d = h.record_bytes instanceof Uint8Array ? h.record_bytes : new Uint8Array(h.record_bytes);
            f_(d);
          }
        }
      }), Ge.postMessage({
        id: r,
        method: "improveRegionStreaming",
        layout: n,
        regionId: t,
        budgetMs: e,
        maxIters: i
      });
    });
  }
  async function k_(n, t) {
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
        e = await Gu(e, o, t.perRegionBudgetMs, (l) => {
          var _a2;
          l.iter > 0 && (a = true), (_a2 = t.onImprovement) == null ? void 0 : _a2.call(t, l);
        }, 1), a ? i += 1 : s.delete(o);
      }
      if (i === 0) break;
    }
    return e;
  }
  function M_(n, t) {
    return ke({
      method: "balancerShowcase",
      maxInputs: n,
      maxOutputs: t
    });
  }
  function zu() {
    return {
      solve: g_,
      allProducibleItems: y_,
      allProducerMachines: x_,
      buildLayout: b_,
      buildLayoutTraced: __,
      buildLayoutStreaming: w_,
      exportBlueprint: v_,
      defaultMachineForItem: C_,
      moduleSlots: S_,
      validateLayout: E_,
      solveFixture: T_,
      improveRegion: Gu,
      optimizeAllRegions: k_,
      balancerShowcase: M_
    };
  }
  const P_ = `
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
  function I_() {
    if (document.getElementById("spaghettio-corpus-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-corpus-style", n.textContent = P_, document.head.appendChild(n);
  }
  function R_(n, t) {
    I_(), n.innerHTML = "";
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
      const I = m.value.trim(), N = ++b;
      if (!I) {
        m.classList.remove("error"), y.style.display = "none";
        return;
      }
      A_(I).then((O) => {
        N === b && (m.classList.remove("error"), y.style.display = "none", t(O));
      }).catch((O) => {
        N === b && (m.classList.add("error"), y.textContent = String(O), y.style.display = "block");
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
    const R = document.createElement("select");
    C.appendChild(T), C.appendChild(R), x.appendChild(C);
    const E = document.createElement("div");
    E.className = "corpus-filter-row";
    const A = document.createElement("input");
    A.type = "checkbox";
    const W = document.createElement("label");
    W.style.display = "flex", W.style.alignItems = "center", W.style.gap = "5px", W.style.cursor = "pointer", W.appendChild(A), W.appendChild(document.createTextNode("Bus layouts only")), E.appendChild(W), x.appendChild(E), e.appendChild(x);
    const j = document.createElement("div");
    j.className = "corpus-count", j.style.display = "none", e.appendChild(j);
    const G = document.createElement("div");
    G.className = "corpus-list", e.appendChild(G);
    const z = document.createElement("div");
    z.className = "corpus-stats", e.appendChild(z);
    function P() {
      i = s.filter((I) => !(o && !I.stats.is_bus_layout || a && a !== "__all__" && I.stats.final_product !== a || l && !I.name.toLowerCase().includes(l.toLowerCase()))), r = -1, D();
    }
    function D() {
      if (G.innerHTML = "", j.textContent = `${i.length} of ${s.length} blueprint(s)`, i.length === 0) {
        const I = document.createElement("div");
        I.className = "corpus-empty", I.textContent = s.length === 0 ? "No corpus loaded yet." : "No blueprints match the current filters.", G.appendChild(I), z.classList.remove("visible");
        return;
      }
      for (let I = 0; I < i.length; I++) {
        const N = i[I], O = document.createElement("div");
        O.className = "corpus-item" + (I === r ? " selected" : "");
        const B = document.createElement("div");
        B.className = "corpus-item-name", B.textContent = N.name, B.title = N.name, O.appendChild(B);
        const q = document.createElement("div");
        if (q.className = "corpus-item-meta", N.stats.is_bus_layout) {
          const ft = document.createElement("span");
          ft.className = "corpus-badge corpus-badge-bus", ft.textContent = "BUS", q.appendChild(ft);
        }
        if (N.stats.final_product) {
          const ft = document.createElement("span");
          ft.className = "corpus-badge corpus-badge-product", ft.textContent = N.stats.final_product, q.appendChild(ft);
        }
        const Q = document.createElement("span");
        Q.className = "corpus-badge corpus-badge-machines", Q.textContent = `${N.stats.machine_count}m`, q.appendChild(Q), O.appendChild(q);
        const ut = I;
        O.addEventListener("click", () => J(ut)), G.appendChild(O);
      }
    }
    function J(I) {
      r = I;
      const N = i[I];
      D(), t(N.layout), Y(N);
    }
    function Y(I) {
      z.innerHTML = "", z.classList.add("visible");
      const N = document.createElement("div");
      N.className = "corpus-stats-title", N.textContent = I.name, N.title = I.name, z.appendChild(N);
      const O = document.createElement("div");
      O.className = "corpus-stats-grid";
      const B = I.stats, q = [
        [
          "machines",
          String(B.machine_count)
        ],
        [
          "recipes",
          String(B.recipe_count)
        ],
        [
          "is_bus",
          B.is_bus_layout ? "yes" : "no"
        ],
        [
          "density",
          B.density.toFixed(2)
        ]
      ];
      B.is_bus_layout && q.push([
        "bus_lanes",
        String(B.bus_lane_count)
      ], [
        "bus_pitch",
        B.bus_pitch.toFixed(1)
      ], [
        "row_pitch",
        B.row_pitch.toFixed(1)
      ], [
        "rows",
        String(B.row_count)
      ]), q.push([
        "bbox",
        `${B.bbox_width}\xD7${B.bbox_height}`
      ], [
        "belt_tiles",
        String(B.belt_tiles)
      ]), B.pipe_tiles > 0 && q.push([
        "pipe_tiles",
        String(B.pipe_tiles)
      ]);
      for (const [Q, ut] of q) {
        const ft = document.createElement("div");
        ft.className = "corpus-stats-row";
        const mt = document.createElement("span");
        mt.className = "corpus-stats-key", mt.textContent = Q;
        const wt = document.createElement("span");
        wt.className = "corpus-stats-val", wt.textContent = ut, ft.appendChild(mt), ft.appendChild(wt), O.appendChild(ft);
      }
      z.appendChild(O);
    }
    function K() {
      const I = new Set(s.map((O) => O.stats.final_product).filter(Boolean));
      R.innerHTML = "";
      const N = document.createElement("option");
      N.value = "__all__", N.textContent = "All products", R.appendChild(N);
      for (const O of Array.from(I).sort()) {
        const B = document.createElement("option");
        B.value = O, B.textContent = O, R.appendChild(B);
      }
    }
    u.addEventListener("change", () => {
      var _a2;
      const I = (_a2 = u.files) == null ? void 0 : _a2[0];
      if (!I) return;
      const N = new FileReader();
      N.onload = (O) => {
        var _a3;
        try {
          s = JSON.parse((_a3 = O.target) == null ? void 0 : _a3.result).blueprints ?? [], i = s, r = -1, p.textContent = `Reload corpus.json (${s.length} blueprints)`, x.style.display = "", j.style.display = "", z.classList.remove("visible"), K(), P();
        } catch (B) {
          alert(`Failed to parse corpus.json: ${B}`);
        }
      }, N.readAsText(I);
    }), w.addEventListener("input", () => {
      l = w.value, P();
    }), R.addEventListener("change", () => {
      a = R.value, P();
    }), A.addEventListener("change", () => {
      o = A.checked, P();
    }), D();
  }
  const L_ = [
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
  ], $_ = `
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
  function O_() {
    if (document.getElementById("spaghettio-landing-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-landing-style", n.textContent = $_, document.head.appendChild(n);
  }
  function B_(n, t, e) {
    O_(), n.innerHTML = "";
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
    for (const u of L_) {
      const p = document.createElement("div");
      p.className = `spaghettio-landing-card ${u.status}`;
      const f = document.createElement("div");
      f.className = "spaghettio-landing-tier", f.innerHTML = `<span>${u.tier}</span>`, p.appendChild(f);
      const g = document.createElement("div");
      g.className = "spaghettio-landing-card-body";
      const m = document.createElement("div");
      m.className = "spaghettio-landing-card-title";
      const y = document.createElement("img");
      y.src = `/spaghettio/pr-536/icons/${u.item}.png`, y.className = "spaghettio-landing-card-icon", y.onerror = () => {
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
        N_(t, u, p, v);
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
  function N_(n, t, e, s) {
    e.classList.contains("loading") || (e.classList.add("loading"), s.innerHTML = '<span class="spaghettio-spinner"></span>', (async () => {
      let i, r;
      try {
        const o = n.defaultMachineForItem(t.item, t.machine);
        i = await n.solve(t.item, t.rate, t.inputs, {}, o), r = await n.buildLayout(i, t.beltTier);
      } catch (o) {
        e.classList.remove("loading"), s.textContent = "error", console.error("Landing solve/layout failed:", o);
        return;
      }
      s.textContent = String(r.entities.length), e.classList.remove("loading"), eu(i.machines.map((o) => ({
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
      }))), F_(t, r, i).catch((o) => {
        console.error("Modal init failed:", o);
      });
    })());
  }
  async function F_(n, t, e) {
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
    d.src = `/spaghettio/pr-536/icons/${n.item}.png`, d.onerror = () => {
      d.style.display = "none";
    }, h.appendChild(d), h.appendChild(document.createTextNode(` ${n.label} \u2014 ${n.rate}/s`)), c.appendChild(h);
    const u = document.createElement("div");
    u.className = "spaghettio-preview-stats";
    const p = `${t.width ?? 0}\xD7${t.height ?? 0}`, f = e.machines.reduce((T, R) => T + Math.ceil(R.count), 0);
    u.innerHTML = `<span>${f} machines</span><span>${p} tiles</span>`, c.appendChild(u);
    const g = document.createElement("button");
    g.className = "spaghettio-preview-close", g.textContent = "\xD7", g.addEventListener("click", a), c.appendChild(g), l.appendChild(c);
    const m = document.createElement("div");
    m.className = "spaghettio-preview-canvas", l.appendChild(m);
    const y = document.createElement("div");
    y.className = "spaghettio-preview-badge", y.textContent = `0 / ${t.entities.length}`, m.appendChild(y), o = new xa(), await o.init({
      resizeTo: m,
      background: 1118481,
      antialias: true
    }), m.insertBefore(o.canvas, y), o.canvas.addEventListener("contextmenu", (T) => T.preventDefault());
    const b = (t.width ?? 20) * S, x = (t.height ?? 20) * S, _ = Math.max(b, x, 600) + 200, v = new Jd({
      screenWidth: m.clientWidth,
      screenHeight: m.clientHeight,
      worldWidth: _,
      worldHeight: _,
      events: o.renderer.events
    });
    v.drag({
      mouseButtons: "left"
    }).pinch().wheel().decelerate(), o.stage.addChild(v);
    const w = new Ht();
    v.addChild(w), v.fit(true, b * 1.15, x * 1.2), v.moveCenter(b / 2, x / 2);
    const { renderLayoutAnimated: C } = await Ln(async () => {
      const { renderLayoutAnimated: T } = await import("./animated-BHHwU9mc.js");
      return {
        renderLayoutAnimated: T
      };
    }, []);
    C(t, w, y, () => {
    });
  }
  var Ie = Uint8Array, vs = Uint16Array, W_ = Int32Array, Du = new Ie([
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
  ]), Hu = new Ie([
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
  ]), G_ = new Ie([
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
  ]), Uu = function(n, t) {
    for (var e = new vs(31), s = 0; s < 31; ++s) e[s] = t += 1 << n[s - 1];
    for (var i = new W_(e[30]), s = 1; s < 30; ++s) for (var r = e[s]; r < e[s + 1]; ++r) i[r] = r - e[s] << 5 | s;
    return {
      b: e,
      r: i
    };
  }, ju = Uu(Du, 2), Yu = ju.b, z_ = ju.r;
  Yu[28] = 258, z_[258] = 28;
  var D_ = Uu(Hu, 0), H_ = D_.b, ha = new vs(32768);
  for (var Kt = 0; Kt < 32768; ++Kt) {
    var Tn = (Kt & 43690) >> 1 | (Kt & 21845) << 1;
    Tn = (Tn & 52428) >> 2 | (Tn & 13107) << 2, Tn = (Tn & 61680) >> 4 | (Tn & 3855) << 4, ha[Kt] = ((Tn & 65280) >> 8 | (Tn & 255) << 8) >> 1;
  }
  var ci = (function(n, t, e) {
    for (var s = n.length, i = 0, r = new vs(t); i < s; ++i) n[i] && ++r[n[i] - 1];
    var o = new vs(t);
    for (i = 1; i < t; ++i) o[i] = o[i - 1] + r[i - 1] << 1;
    var a;
    if (e) {
      a = new vs(1 << t);
      var l = 15 - t;
      for (i = 0; i < s; ++i) if (n[i]) for (var c = i << 4 | n[i], h = t - n[i], d = o[n[i] - 1]++ << h, u = d | (1 << h) - 1; d <= u; ++d) a[ha[d] >> l] = c;
    } else for (a = new vs(s), i = 0; i < s; ++i) n[i] && (a[i] = ha[o[n[i] - 1]++] >> 15 - n[i]);
    return a;
  }), Si = new Ie(288);
  for (var Kt = 0; Kt < 144; ++Kt) Si[Kt] = 8;
  for (var Kt = 144; Kt < 256; ++Kt) Si[Kt] = 9;
  for (var Kt = 256; Kt < 280; ++Kt) Si[Kt] = 7;
  for (var Kt = 280; Kt < 288; ++Kt) Si[Kt] = 8;
  var Vu = new Ie(32);
  for (var Kt = 0; Kt < 32; ++Kt) Vu[Kt] = 5;
  var U_ = ci(Si, 9, 1), j_ = ci(Vu, 5, 1), fo = function(n) {
    for (var t = n[0], e = 1; e < n.length; ++e) n[e] > t && (t = n[e]);
    return t;
  }, Ye = function(n, t, e) {
    var s = t / 8 | 0;
    return (n[s] | n[s + 1] << 8) >> (t & 7) & e;
  }, mo = function(n, t) {
    var e = t / 8 | 0;
    return (n[e] | n[e + 1] << 8 | n[e + 2] << 16) >> (t & 7);
  }, Y_ = function(n) {
    return (n + 7) / 8 | 0;
  }, V_ = function(n, t, e) {
    return (e == null || e > n.length) && (e = n.length), new Ie(n.subarray(t, e));
  }, X_ = [
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
  ], Ve = function(n, t, e) {
    var s = new Error(t || X_[n]);
    if (s.code = n, Error.captureStackTrace && Error.captureStackTrace(s, Ve), !e) throw s;
    return s;
  }, q_ = function(n, t, e, s) {
    var i = n.length, r = 0;
    if (!i || t.f && !t.l) return e || new Ie(0);
    var o = !e, a = o || t.i != 2, l = t.i;
    o && (e = new Ie(i * 3));
    var c = function(V) {
      var st = e.length;
      if (V > st) {
        var it = new Ie(Math.max(st * 2, V));
        it.set(e), e = it;
      }
    }, h = t.f || 0, d = t.p || 0, u = t.b || 0, p = t.l, f = t.d, g = t.m, m = t.n, y = i * 8;
    do {
      if (!p) {
        h = Ye(n, d, 1);
        var b = Ye(n, d + 1, 3);
        if (d += 3, b) if (b == 1) p = U_, f = j_, g = 9, m = 5;
        else if (b == 2) {
          var w = Ye(n, d, 31) + 257, C = Ye(n, d + 10, 15) + 4, T = w + Ye(n, d + 5, 31) + 1;
          d += 14;
          for (var R = new Ie(T), E = new Ie(19), A = 0; A < C; ++A) E[G_[A]] = Ye(n, d + A * 3, 7);
          d += C * 3;
          for (var W = fo(E), j = (1 << W) - 1, G = ci(E, W, 1), A = 0; A < T; ) {
            var z = G[Ye(n, d, j)];
            d += z & 15;
            var x = z >> 4;
            if (x < 16) R[A++] = x;
            else {
              var P = 0, D = 0;
              for (x == 16 ? (D = 3 + Ye(n, d, 3), d += 2, P = R[A - 1]) : x == 17 ? (D = 3 + Ye(n, d, 7), d += 3) : x == 18 && (D = 11 + Ye(n, d, 127), d += 7); D--; ) R[A++] = P;
            }
          }
          var J = R.subarray(0, w), Y = R.subarray(w);
          g = fo(J), m = fo(Y), p = ci(J, g, 1), f = ci(Y, m, 1);
        } else Ve(1);
        else {
          var x = Y_(d) + 4, _ = n[x - 4] | n[x - 3] << 8, v = x + _;
          if (v > i) {
            l && Ve(0);
            break;
          }
          a && c(u + _), e.set(n.subarray(x, v), u), t.b = u += _, t.p = d = v * 8, t.f = h;
          continue;
        }
        if (d > y) {
          l && Ve(0);
          break;
        }
      }
      a && c(u + 131072);
      for (var K = (1 << g) - 1, I = (1 << m) - 1, N = d; ; N = d) {
        var P = p[mo(n, d) & K], O = P >> 4;
        if (d += P & 15, d > y) {
          l && Ve(0);
          break;
        }
        if (P || Ve(2), O < 256) e[u++] = O;
        else if (O == 256) {
          N = d, p = null;
          break;
        } else {
          var B = O - 254;
          if (O > 264) {
            var A = O - 257, q = Du[A];
            B = Ye(n, d, (1 << q) - 1) + Yu[A], d += q;
          }
          var Q = f[mo(n, d) & I], ut = Q >> 4;
          Q || Ve(3), d += Q & 15;
          var Y = H_[ut];
          if (ut > 3) {
            var q = Hu[ut];
            Y += mo(n, d) & (1 << q) - 1, d += q;
          }
          if (d > y) {
            l && Ve(0);
            break;
          }
          a && c(u + 131072);
          var ft = u + B;
          if (u < Y) {
            var mt = r - Y, wt = Math.min(Y, ft);
            for (mt + u < 0 && Ve(3); u < wt; ++u) e[u] = s[mt + u];
          }
          for (; u < ft; ++u) e[u] = e[u - Y];
        }
      }
      t.l = p, t.p = N, t.b = u, t.f = h, p && (h = 1, t.m = g, t.d = f, t.n = m);
    } while (!h);
    return u != e.length && o ? V_(e, 0, u) : e.subarray(0, u);
  }, K_ = new Ie(0), J_ = function(n) {
    (n[0] != 31 || n[1] != 139 || n[2] != 8) && Ve(6, "invalid gzip data");
    var t = n[3], e = 10;
    t & 4 && (e += (n[10] | n[11] << 8) + 2);
    for (var s = (t >> 3 & 1) + (t >> 4 & 1); s > 0; s -= !n[e++]) ;
    return e + (t & 2);
  }, Z_ = function(n) {
    var t = n.length;
    return (n[t - 4] | n[t - 3] << 8 | n[t - 2] << 16 | n[t - 1] << 24) >>> 0;
  };
  function Q_(n, t) {
    var e = J_(n);
    return e + 8 > n.length && Ve(6, "invalid gzip data"), q_(n.subarray(e, -8), {
      i: 2
    }, new Ie(Z_(n)), t);
  }
  var tw = typeof TextDecoder < "u" && new TextDecoder(), ew = 0;
  try {
    tw.decode(K_, {
      stream: true
    }), ew = 1;
  } catch {
  }
  const go = "fls1";
  async function Xu(n) {
    const t = typeof n == "string" ? n : new TextDecoder().decode(n);
    if (!t.startsWith(go)) throw new Error(`Not a layout snapshot: expected "${go}" prefix, got "${t.slice(0, 4)}"`);
    const e = t.slice(go.length), s = Uint8Array.from(atob(e), (o) => o.charCodeAt(0)), i = Q_(s), r = new TextDecoder().decode(i);
    return JSON.parse(r);
  }
  function nw(n) {
    return new Promise((t, e) => {
      const s = new FileReader();
      s.onload = () => t(s.result), s.onerror = () => e(new Error("Failed to read file")), s.readAsText(n);
    });
  }
  function sw(n, t) {
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
          const i = await nw(s), r = await Xu(i);
          t(r);
        } catch (i) {
          alert(`Failed to load snapshot: ${i}`);
        }
      }
    });
  }
  function iw(n, t, e) {
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
  function Kc(n, t, e, s, i, r, o, a) {
    const l = s - t, c = i - e, h = Math.sqrt(l * l + c * c);
    if (h === 0) return;
    const d = l / h, u = c / h;
    let p = 0;
    for (; p < h; ) {
      const f = Math.min(p + r, h);
      n.moveTo(t + d * p, e + u * p).lineTo(t + d * f, e + u * f).stroke(a), p = f + o;
    }
  }
  function rw(n, t, e, s, i) {
    const r = new Ht(), o = n.find((h) => h.phase === "LanesPlanned");
    if (o) for (const h of o.data.lanes) {
      const d = new pt(), u = h.x * S;
      d.rect(u, 0, S, e * S).fill({
        color: h.is_fluid ? 4500223 : 4521864,
        alpha: 0.04
      }), d.eventMode = "static", d.on("pointerenter", () => i(`Lane: ${h.item} @ x=${h.x} (${h.rate.toFixed(1)}/s${h.is_fluid ? " fluid" : ""})`)), d.on("pointerleave", () => i(null)), r.addChild(d);
    }
    const a = n.find((h) => h.phase === "RowsPlaced");
    if (a) for (const h of a.data.rows) {
      const d = new pt(), u = h.y_end * S;
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
      const p = new pt();
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
      const d = h.data, u = new pt();
      u.moveTo(d.from_x * S + S / 2, d.from_y * S + S / 2).lineTo(d.to_x * S + S / 2, d.to_y * S + S / 2).stroke({
        width: 2,
        color: 8978244,
        alpha: 0.5
      }), u.eventMode = "static", u.on("pointerenter", () => i(`Tap-off: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) len=${d.path_len}`)), u.on("pointerleave", () => i(null)), r.addChild(u);
    }
    for (const h of n) {
      if (h.phase !== "MergerBlockPlaced") continue;
      const d = h.data, u = new pt();
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
      const d = h.data, u = d.from_x * S + S / 2, p = d.from_y * S + S / 2, f = 3, g = new pt();
      g.label = "RouteFailure", g.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), g.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), Kc(g, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), g.eventMode = "static", g.on("pointerenter", () => i(`Route failed: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) [${d.spec_key}]`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    const l = ow;
    let c = 0;
    for (const h of n) {
      if (h.phase !== "GhostSpecRouted") continue;
      const d = h.data, u = l[c % l.length];
      c++;
      const p = new pt();
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
      const d = h.data, u = d.from_x * S + S / 2, p = d.from_y * S + S / 2, f = 4, g = new pt();
      g.label = "RouteFailure", g.moveTo(u - f, p - f).lineTo(u + f, p + f).stroke({
        width: 2,
        color: 16724787
      }), g.moveTo(u + f, p - f).lineTo(u - f, p + f).stroke({
        width: 2,
        color: 16724787
      }), Kc(g, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), g.eventMode = "static", g.on("pointerenter", () => i(`Ghost failed: ${d.spec_key} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y})`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    for (const h of n) {
      if (h.phase !== "GhostClusterSolved" && h.phase !== "GhostClusterFailed") continue;
      const d = h.phase === "GhostClusterFailed", u = d ? null : h.data, p = d ? h.data : null, f = u ?? p, g = d ? 16729156 : 4500223, m = new pt();
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
  const ow = [
    5676246,
    6996096,
    13672512,
    11567312
  ], aw = {
    Error: 16729156,
    Warning: 16755200
  }, lw = 0.85;
  function cw(n, t, e) {
    const s = new Ht(), i = /* @__PURE__ */ new Map();
    for (const r of n) {
      if (r.x == null || r.y == null) continue;
      const o = aw[r.severity] ?? 4500223, a = new pt();
      a.rect(r.x * S, r.y * S, S, S).stroke({
        width: 2,
        color: o,
        alpha: lw
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
  function hw(n) {
    const t = Math.max(0, Math.min(1, n)), e = Math.round(64 + 96 * t);
    return {
      color: 255 << 16 | e << 8 | 32,
      alpha: 0.45 * (1 - t) + 0.12
    };
  }
  function dw(n, t, e) {
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
      const a = Se[o.name];
      a && i.set(`${o.x},${o.y}`, a);
    }
    const r = new Ht();
    r.eventMode = "none";
    for (const [o, a] of s) {
      const [l, c] = o.split(",").map(Number), h = i.get(o), [d, u] = h ?? [
        1,
        1
      ], { color: p, alpha: f } = hw(a), g = new pt();
      g.rect(l * S, c * S, d * S, u * S).fill({
        color: p,
        alpha: f
      }), r.addChild(g);
    }
    return e.addChild(r), r;
  }
  function Jc(n) {
    const [t, e] = Se[n.name] ?? [
      1,
      1
    ], s = n.x ?? 0, i = n.y ?? 0;
    return {
      cx: (s + t / 2) * S,
      cy: (i + e / 2) * S
    };
  }
  function uw(n, t, e) {
    if (!n || n.length === 0) return null;
    const s = new Ht();
    s.eventMode = "none";
    const i = new pt();
    for (const [r, o] of n) {
      const a = t[r], l = t[o];
      if (!a || !l) continue;
      const { cx: c, cy: h } = Jc(a), { cx: d, cy: u } = Jc(l);
      i.moveTo(c, h), i.lineTo(d, u);
    }
    return i.stroke({
      color: 16751164,
      width: 1.5,
      alpha: 0.85
    }), s.addChild(i), e.addChild(s), s;
  }
  function pw(n) {
    return n.startsWith("speed-module") ? "speed" : n.startsWith("productivity-module") ? "productivity" : n.startsWith("efficiency-module") ? "efficiency" : n.startsWith("quality-module") ? "quality" : "unknown";
  }
  const fw = {
    speed: 4886736,
    productivity: 14238510,
    efficiency: 5025616,
    quality: 3127472,
    unknown: 9079434
  }, An = S * 0.16, yo = S * 0.05, xo = 1.5;
  function mw(n, t, e) {
    const s = new Ht();
    s.eventMode = "none";
    let i = false;
    for (const r of n) {
      if (!r.items || r.items.length === 0) continue;
      const o = r.items.reduce((x, _) => x + _.count, 0), a = Math.max(t(r.name), o);
      if (a === 0) continue;
      const [l, c] = Se[r.name] ?? [
        1,
        1
      ], h = l * S, d = c * S, u = (r.x ?? 0) * S, p = (r.y ?? 0) * S, f = a * An + (a - 1) * yo, g = u + Math.max(h - f - xo, xo), m = p + d - An - xo, y = new pt();
      let b = 0;
      for (const x of r.items) {
        const _ = fw[pw(x.item)], v = x.quality && x.quality !== "normal" ? gu[x.quality] : void 0, w = v ?? 0, C = v !== void 0 ? 0.95 : 0.35, T = v !== void 0 ? 1 : 0.75;
        for (let R = 0; R < x.count && b < a; R++, b++) {
          const E = g + b * (An + yo);
          y.rect(E, m, An, An).fill({
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
        const x = g + b * (An + yo);
        y.rect(x, m, An, An).fill({
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
  const gw = {
    working: 5025616,
    item_ingredient_shortage: 14238510,
    no_ingredients: 14238510,
    full_output: 16751164,
    no_power: 10181046
  }, yw = 9079434, xw = 0.4;
  function bw(n) {
    return gw[n] ?? yw;
  }
  const _w = 8, ww = 0.5, ms = {
    r: 245,
    g: 197,
    b: 24
  }, bo = {
    r: 217,
    g: 67,
    b: 46
  };
  function vw(n) {
    const t = Math.max(0, Math.min(1, n / _w)), e = Math.round(ms.r + (bo.r - ms.r) * t), s = Math.round(ms.g + (bo.g - ms.g) * t), i = Math.round(ms.b + (bo.b - ms.b) * t);
    return e << 16 | s << 8 | i;
  }
  const Cw = S * 0.14, Sw = 0.9;
  function Ew(n) {
    return n === "working" ? null : n.startsWith("waiting_for_source") ? 14238510 : n.startsWith("waiting_for_space") ? 16751164 : 9079434;
  }
  function Tw(n, t) {
    const e = new Ht();
    e.eventMode = "none";
    let s = false;
    const i = new pt();
    for (const [a, l, c] of n.belts) c <= 0 || (i.rect(a * S, l * S, S, S).fill({
      color: vw(c),
      alpha: ww
    }), s = true);
    e.addChild(i);
    const r = new pt();
    for (const [a, l, c, h] of n.machines) {
      const [d, u] = Se[c] ?? [
        1,
        1
      ];
      r.rect(a * S, l * S, d * S, u * S).fill({
        color: bw(h),
        alpha: xw
      }), s = true;
    }
    e.addChild(r);
    const o = new pt();
    for (const [a, l, c] of n.inserters) {
      const h = Ew(c);
      if (h === null) continue;
      const d = a * S + S / 2, u = l * S + S / 2;
      o.circle(d, u, Cw).fill({
        color: h,
        alpha: Sw
      }), s = true;
    }
    return e.addChild(o), s ? (t.addChild(e), e) : null;
  }
  function qs(n) {
    return typeof n == "object" && n !== null && !Array.isArray(n);
  }
  function _o(n, t) {
    return Array.isArray(n) && n.every((e) => Array.isArray(e) && e.length >= t);
  }
  function Aw(n) {
    let t;
    try {
      t = JSON.parse(n);
    } catch {
      throw new Error("Not valid JSON.");
    }
    if (!qs(t)) throw new Error("Expected a JSON object at the top level.");
    const e = [];
    typeof t.game_version != "string" && e.push("missing `game_version` (string)");
    const s = t.report;
    if (!qs(s)) e.push("missing `report` object");
    else if (typeof s.overall_verdict != "string" && e.push("`report.overall_verdict` missing"), typeof s.entities != "number" && e.push("`report.entities` missing"), !Array.isArray(s.items)) e.push("`report.items` is not an array");
    else for (const [o, a] of s.items.entries()) if (!qs(a) || typeof a.item != "string" || typeof a.planned_rate != "number") {
      e.push(`\`report.items[${o}]\` is malformed`);
      break;
    }
    const i = t.run_params;
    qs(i) || e.push("missing `run_params` object");
    const r = t.sim_state;
    if (qs(r) ? (_o(r.belts, 3) || e.push("`sim_state.belts` is not an array of [x,y,count]"), _o(r.machines, 4) || e.push("`sim_state.machines` is not an array of [x,y,name,status]"), _o(r.inserters, 3) || e.push("`sim_state.inserters` is not an array of [x,y,status]"), (typeof r.offx != "number" || typeof r.offy != "number") && e.push("`sim_state.offx`/`offy` missing")) : e.push("missing `sim_state` object"), e.length > 0) throw new Error(`Not a valid sim report: ${e.join("; ")}.`);
    return t;
  }
  function Zc(n) {
    return n === "PASS" ? "#4caf50" : n === "WARN" ? "#ff9a3c" : n === "FAIL" ? "#f66" : "#aaa";
  }
  function wo(n) {
    return n == null ? "?" : `${n}/s`;
  }
  function kw(n, t) {
    const e = document.createElement("div");
    e.className = "sim-report-panel", n.appendChild(e);
    let s = null, i = null;
    function r() {
      const l = document.createElement("div");
      l.className = "sim-report-legend";
      const c = document.createElement("div");
      c.className = "sim-legend-title", c.textContent = "Legend", l.appendChild(c);
      const h = [
        [
          "#4caf50",
          "machine working"
        ],
        [
          "#d9432e",
          "machine shortage"
        ],
        [
          "#ff9a3c",
          "machine full output"
        ],
        [
          "#9b59b6",
          "machine no power"
        ],
        [
          "#8a8a8a",
          "machine / inserter other"
        ]
      ];
      for (const [f, g] of h) {
        const m = document.createElement("div"), y = document.createElement("span");
        y.className = "sim-swatch", y.style.background = f, m.appendChild(y), m.appendChild(document.createTextNode(g)), l.appendChild(m);
      }
      const d = document.createElement("div"), u = document.createElement("span");
      u.className = "sim-swatch sim-swatch-grad", d.appendChild(u), d.appendChild(document.createTextNode("belt fill (low \u2192 high)")), l.appendChild(d);
      const p = [
        [
          "#d9432e",
          "inserter waiting for source"
        ],
        [
          "#ff9a3c",
          "inserter waiting for space"
        ]
      ];
      for (const [f, g] of p) {
        const m = document.createElement("div"), y = document.createElement("span");
        y.className = "sim-dot", y.style.background = f, m.appendChild(y), m.appendChild(document.createTextNode(g)), l.appendChild(m);
      }
      return l;
    }
    function o() {
      if (e.replaceChildren(), !s) {
        e.classList.remove("visible");
        return;
      }
      e.classList.add("visible");
      const l = s.report;
      if (i && i.entities !== l.entities) {
        const h = document.createElement("div");
        h.className = "sim-report-mismatch", h.textContent = `\u26A0 report is for a different layout (${l.entities} entities vs current ${i.entities})`, e.appendChild(h);
      }
      const c = document.createElement("div");
      c.className = "sim-report-title", c.style.color = Zc(l.overall_verdict), c.textContent = `SIM ${l.overall_verdict ?? "?"} \u2014 ${l.label ?? "report"}`, e.appendChild(c);
      for (const h of l.items) {
        const d = document.createElement("div");
        d.className = "sim-report-row", h.is_target && d.classList.add("target");
        let u = `${h.item}: ${wo(h.planned_rate)} planned \u2192 ${wo(h.measured_produced_rate)} produced`;
        if (h.measured_delivered_rate != null && (u += ` \u2192 ${wo(h.measured_delivered_rate)} delivered`), d.appendChild(document.createTextNode(u)), h.verdict) {
          const p = document.createElement("span");
          p.className = "sim-report-verdict", p.style.color = Zc(h.verdict), p.textContent = ` [${h.verdict}]`, d.appendChild(p);
        }
        e.appendChild(d);
      }
      e.appendChild(r());
    }
    function a(l) {
      e.replaceChildren(), e.classList.add("visible");
      const c = document.createElement("div");
      c.className = "sim-report-error", c.textContent = `Failed to load sim report: ${l}`, e.appendChild(c);
    }
    return {
      loadFromText(l) {
        try {
          s = Aw(l);
        } catch (c) {
          s = null, a(c instanceof Error ? c.message : String(c)), t(null);
          return;
        }
        o(), t(s);
      },
      clear() {
        s = null, e.classList.remove("visible"), e.replaceChildren(), t(null);
      },
      getReport() {
        return s;
      },
      setLayoutInfo(l) {
        i = l, s && o();
      }
    };
  }
  function Mw(n) {
    const t = n.point.direction;
    return t === "East" || t === "West" ? "horizontal" : "vertical";
  }
  function Pw(n) {
    const t = new Set(n.map(Mw));
    return t.size === 1 ? [
      ...t
    ][0] : "mixed";
  }
  function Iw(n) {
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
    for (const a of e.values()) a.axis = Pw([
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
  function Rw(n) {
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
  function Lw(n) {
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
  const $w = 0.35;
  function Ow(n, t, e, s, i) {
    const r = S * 0.45;
    n.setStrokeStyle({
      width: 3,
      color: i,
      alpha: $w
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
  function vo(n) {
    return [
      n.point.x,
      n.point.y
    ];
  }
  function Bw(n, t, e, s, i, r, o = 2, a = 6, l = 4, c = 0.9) {
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
  function Nw(n) {
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
  function Fw(n) {
    const t = new Ht(), e = n.regions ?? [], s = [];
    if (e.length === 0) return {
      layer: t,
      items: s,
      hitTest: () => null
    };
    for (const r of e) {
      const o = Iw(r), a = Rw(r.kind), l = Lw(o.cls), c = r.x * S, h = r.y * S, d = r.width * S, u = r.height * S;
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
      const p = new pt(), f = r.kind === "crossing_zone" ? 0.06 : 0.14;
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
      const g = r.ports ?? [], m = Nw(g);
      for (const { item: y, inPort: b, outPort: x } of m) {
        const [_, v] = vo(b), [w, C] = vo(x), T = _ * S + S / 2, R = v * S + S / 2, E = w * S + S / 2, A = C * S + S / 2, W = On(y), j = new pt();
        Bw(j, T, R, E, A, W), t.addChild(j);
      }
      for (const y of g) {
        const [b, x] = vo(y), _ = b * S + S / 2, v = x * S + S / 2, w = new pt(), C = y.item ? On(y.item) : 8947848;
        Ow(w, _, v, y.point.direction, C), t.addChild(w);
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
  const Ww = /* @__PURE__ */ new Set([
    "JunctionGrowthStarted",
    "JunctionGrowthIteration",
    "JunctionStrategyAttempt",
    "SatInvocation",
    "JunctionSolved",
    "JunctionGrowthCapped",
    "RegionWalkerVeto"
  ]);
  function Gw(n) {
    return Ww.has(n.phase);
  }
  function zw(n) {
    const t = n.data;
    return typeof t.seed_x == "number" && typeof t.seed_y == "number" ? [
      t.seed_x,
      t.seed_y
    ] : [
      t.tile_x,
      t.tile_y
    ];
  }
  function Dw(n, t) {
    return `${n},${t}`;
  }
  function Hw(n) {
    const t = /* @__PURE__ */ new Map(), e = (a, l) => {
      const c = Dw(a, l);
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
      if (!Gw(a)) continue;
      const [l, c] = zw(a), h = e(l, c);
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
  function Qc(n) {
    return n.iterations.length === 0 ? null : n.iterations[n.defaultIterIndex] ?? n.iterations[n.iterations.length - 1];
  }
  function Uw(n) {
    return n ? `${n.entity_name}@(${n.entity_x},${n.entity_y}) ${n.direction}` : "";
  }
  function jw(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function Yw(n, t, e) {
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
        item: jw(c.spec_key),
        specKey: c.spec_key,
        tiles: h
      });
    }
    return a;
  }
  const th = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, Vw = new De({
    fontFamily: "monospace",
    fontSize: 10,
    fill: 16777215,
    dropShadow: {
      color: 0,
      distance: 1,
      blur: 2,
      alpha: 0.8
    }
  }), Xw = new De({
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
  function qw(n) {
    const t = new Ht(), e = [], s = [];
    for (const o of n) {
      if (o.outcome.kind !== "Solved") continue;
      const a = Qc(o);
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
      const a = Qc(o);
      if (!a) continue;
      const l = a.bbox;
      if (l.w <= 0 || l.h <= 0 || o.outcome.kind === "Capped" && i(o.seed.x, o.seed.y)) continue;
      const c = l.x * S, h = l.y * S, d = l.w * S, u = l.h * S, p = th[o.outcome.kind] ?? th.Open, f = new pt();
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
      const g = new qe({
        text: `Junction (${o.seed.x},${o.seed.y})`,
        style: Vw
      });
      g.x = c + 3, g.y = h + 2, t.addChild(g);
      const m = new qe({
        text: Kw(o),
        style: Xw
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
  function Kw(n) {
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
  const eh = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, Jw = 0.04, Zw = 9060416, Qw = 0.55, tv = 4243680, ev = 5592405, nv = 0.55, sv = 16777215, nh = 0.85, iv = 16777215, sh = new De({
    fontFamily: "monospace",
    fontSize: 7,
    fontWeight: "700",
    fill: sv
  }), ih = /* @__PURE__ */ new Map();
  function rv(n) {
    let t = ih.get(n);
    if (!t) {
      const s = `/spaghettio/pr-536/icons/${n}.png`;
      t = Fe.load(s).catch(() => null), ih.set(n, t);
    }
    return t;
  }
  function ov() {
    const n = new Ht();
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
      const { cluster: a, iter: l } = r, c = l.bbox, h = eh[a.outcome.kind] ?? eh.Open, d = new pt();
      d.rect(c.x * S, c.y * S, c.w * S, c.h * S).fill({
        color: h,
        alpha: Jw
      }), n.addChild(d);
      const u = av(c.x * S, c.y * S, c.w * S, c.h * S, {
        dashLen: S * 0.45,
        gapLen: S * 0.25,
        width: 3,
        color: h,
        alpha: 0.95
      });
      n.addChild(u);
      const p = new pt();
      p.setStrokeStyle({
        width: 1,
        color: Zw,
        alpha: Qw
      });
      for (const [m, y] of l.forbidden) lv(p, m * S, y * S, S);
      n.addChild(p);
      const f = new pt();
      f.circle((a.seed.x + 0.5) * S, (a.seed.y + 0.5) * S, S * 0.42).stroke({
        width: 3,
        color: tv,
        alpha: 0.95
      }), n.addChild(f);
      const g = ((_a2 = l.sat) == null ? void 0 : _a2.boundaries) ?? l.boundaries;
      for (const m of g) hv(n, m, o, () => t);
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
  function av(n, t, e, s, i) {
    const r = new pt();
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
  function lv(n, t, e, s) {
    const i = s;
    n.moveTo(t, e + i).lineTo(t + i, e).stroke(), n.moveTo(t + i / 3, e + i).lineTo(t + i, e + 2 * i / 3).stroke(), n.moveTo(t, e + 2 * i / 3).lineTo(t + 2 * i / 3, e).stroke();
  }
  function cv(n, t) {
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
  function hv(n, t, e, s) {
    const i = t.x * S, r = t.y * S, o = S / 3, a = cv(t.is_input, t.direction), l = a === "top" || a === "bottom";
    let c = i, h = r, d = S, u = S;
    a === "top" ? u = o : a === "bottom" ? (h = r + S - o, u = o) : (a === "left" || (c = i + S - o), d = o);
    const p = new pt();
    p.rect(c, h, d, u).fill({
      color: ev,
      alpha: nv
    }), t.interior && (p.setStrokeStyle({
      width: 1,
      color: iv,
      alpha: 0.5
    }), p.rect(c, h, d, u).stroke()), n.addChild(p);
    const f = dv(t.direction), [g, m, y, b] = l ? [
      c + d / 6,
      h + u / 2,
      c + d * 5 / 6,
      h + u / 2
    ] : [
      c + d / 2,
      h + u / 6,
      c + d / 2,
      h + u * 5 / 6
    ], x = new qe({
      text: f,
      style: sh
    });
    x.anchor.set(0.5), x.x = g, x.y = m, x.alpha = nh, n.addChild(x);
    const _ = new qe({
      text: f,
      style: sh
    });
    _.anchor.set(0.5), _.x = y, _.y = b, _.alpha = nh, n.addChild(_);
    const v = c + d / 2, w = h + u / 2;
    rv(t.item).then((C) => {
      if (e !== s() || !C) return;
      const T = new ze(C);
      T.anchor.set(0.5), T.x = v, T.y = w;
      const R = o * 0.95;
      T.width = R, T.height = R, n.addChild(T);
    });
  }
  function dv(n) {
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
  const uv = 4247776, pv = 0.18;
  function fv(n) {
    if (!n || n.length === 0) return null;
    const t = /* @__PURE__ */ new Set();
    for (const i of n) if (i.phase === "GhostSpecRouted") for (const [r, o] of i.data.tiles) t.add(`${r},${o}`);
    if (t.size === 0) return null;
    const e = new Ht();
    e.label = "ghost-tiles-overlay";
    const s = new pt();
    for (const i of t) {
      const [r, o] = i.split(","), a = Number(r), l = Number(o);
      s.rect(a * S, l * S, S, S).fill({
        color: uv,
        alpha: pv
      });
    }
    return e.addChild(s), e;
  }
  function mv(n, t, e) {
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
    const R = document.createElement("div");
    R.className = "jd-footer", R.textContent = "Esc close \xB7 \u2190/\u2192 step all \xB7 w/s iter \xB7 a/d variant \xB7 Home/End first/last", x.append(_, T, R), n.append(b, x);
    let E = null, A = 0, W = null, j = false;
    function G(V, st) {
      E = V, W = st ?? null, A = V.defaultIterIndex, s.classList.add("jd-open"), Q(), q(), I();
    }
    function z() {
      E && (D(), E = null, W = null, s.classList.remove("jd-open"), e.onChange(null));
    }
    function P() {
      !E || j || (j = true, b.classList.add("jd-open"), x.classList.add("jd-open"), ut());
    }
    function D() {
      j && (j = false, b.classList.remove("jd-open"), x.classList.remove("jd-open"));
    }
    function J() {
      j ? D() : P();
    }
    function Y() {
      return E !== null;
    }
    function K(V) {
      if (!E) return;
      const st = Math.max(0, Math.min(E.iterations.length - 1, V));
      st !== A && (A = st, Q(), q(), I());
    }
    function I() {
      if (!E) return;
      const V = E.iterations[A];
      if (!V) return;
      const st = t.toScreen(V.bbox.x * S, V.bbox.y * S), it = t.toScreen((V.bbox.x + V.bbox.w) * S, (V.bbox.y + V.bbox.h) * S), lt = n.getBoundingClientRect();
      if (!(it.x < 0 || st.x > lt.width || it.y < 0 || st.y > lt.height)) return;
      const Nt = (V.bbox.x + V.bbox.w / 2) * S, xt = (V.bbox.y + V.bbox.h / 2) * S;
      t.moveCenter(Nt, xt);
    }
    function N() {
      const V = /* @__PURE__ */ new Map(), st = E;
      if (!st) return V;
      for (let it = 0; it < st.iterations.length; it++) {
        const lt = st.iterations[it], Et = V.get(lt.iter) ?? [];
        Et.push(it), V.set(lt.iter, Et);
      }
      return V;
    }
    function O(V) {
      if (!E) return;
      const st = N(), it = Array.from(st.keys()).sort((Vt, te) => Vt - te), lt = E.iterations[A].iter, Et = it.indexOf(lt), Nt = it[Math.max(0, Math.min(it.length - 1, Et + V))], xt = st.get(Nt) ?? [], jt = xt.find((Vt) => E.iterations[Vt].variant === "");
      K(jt ?? xt[0] ?? A);
    }
    function B(V) {
      if (!E) return;
      const st = N(), it = E.iterations[A].iter, lt = st.get(it) ?? [];
      if (lt.length <= 1) return;
      const Nt = (lt.indexOf(A) + V + lt.length) % lt.length;
      K(lt[Nt]);
    }
    function q() {
      if (!E) return;
      const V = E.iterations[A];
      if (!V) return;
      const st = n.getBoundingClientRect(), it = s.offsetWidth || 200, lt = s.offsetHeight || 70, Et = (V.bbox.x + V.bbox.w) * S, Nt = (V.bbox.y + V.bbox.h) * S, xt = V.bbox.y * S, jt = t.toScreen(Et, Nt), Vt = t.toScreen(Et, xt);
      let te = jt.x - it, Lt = jt.y;
      Lt + lt > st.height - 4 && (Lt = Vt.y - lt), te = Math.max(4, Math.min(te, st.width - it - 4)), Lt = Math.max(4, Math.min(Lt, st.height - lt - 4)), s.style.left = `${te}px`, s.style.top = `${Lt}px`;
    }
    t.on("moved", q), t.on("zoomed", q), window.addEventListener("resize", q);
    function Q() {
      if (!E) return;
      const V = E, st = V.iterations[A];
      r.textContent = `Junction (${V.seed.x},${V.seed.y})`, o.className = `jd-status-pill jd-${V.outcome.kind.toLowerCase()}`, o.textContent = rh(V);
      const it = V.iterations.length, lt = st && st.variant ? ` \xB7 ${st.variant}` : "";
      f.textContent = `iter ${st ? st.iter : "-"}${lt} \xB7 ${A + 1}/${it}`, p.disabled = A <= 0, g.disabled = A >= it - 1, m.disabled = A === V.defaultIterIndex, y.innerHTML = "";
      for (const Et of yv(V, st)) {
        const Nt = document.createElement("div");
        Nt.className = `jd-inline-summary-row jd-inline-summary-row--${Et.tone}`, Nt.textContent = Et.text, y.appendChild(Nt);
      }
      j && ut(), st && e.onChange({
        cluster: V,
        iter: st,
        trace: W
      });
    }
    function ut() {
      if (!E) return;
      const V = E, st = V.iterations[A];
      w.className = `jd-status-pill jd-${V.outcome.kind.toLowerCase()}`, w.textContent = rh(V), v.textContent = `Junction (${V.seed.x},${V.seed.y})`, bv(T, V, st);
    }
    d.addEventListener("click", z), a.addEventListener("click", J), l.addEventListener("click", wt), c.addEventListener("click", () => {
      if (!E) return;
      const V = da(E, E.iterations[A], W);
      Av(c, V);
    }), h.addEventListener("click", () => {
      if (!E || !e.onEditRequested) return;
      const V = E.iterations[A];
      V && e.onEditRequested({
        cluster: E,
        iter: V,
        trace: W
      });
    }), C.addEventListener("click", D), b.addEventListener("click", D);
    function ft() {
      if (!E) return null;
      const V = E.iterations[A];
      return V ? {
        cluster: E,
        iter: V,
        trace: W
      } : null;
    }
    function mt(V) {
      s.classList.toggle("jd-edit-mode", V), c.disabled = V, l.disabled = V, h.disabled = V;
    }
    function wt() {
      var _a2;
      if (!E) return;
      const V = gv(E, A), st = JSON.stringify(V, (lt, Et) => typeof Et == "bigint" ? String(Et) : Et, 2), it = (lt) => {
        const Et = l.textContent;
        l.textContent = lt ? "\u2713" : "!", l.classList.add("jd-inline-btn--flash"), window.setTimeout(() => {
          l.textContent = Et, l.classList.remove("jd-inline-btn--flash");
        }, 900);
      };
      if ((_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText) navigator.clipboard.writeText(st).then(() => it(true), () => it(false));
      else {
        const lt = document.createElement("textarea");
        lt.value = st, lt.style.position = "fixed", lt.style.opacity = "0", document.body.appendChild(lt), lt.select();
        try {
          document.execCommand("copy"), it(true);
        } catch {
          it(false);
        }
        document.body.removeChild(lt);
      }
    }
    return p.addEventListener("click", () => K(A - 1)), g.addEventListener("click", () => K(A + 1)), m.addEventListener("click", () => {
      E && K(E.defaultIterIndex);
    }), document.addEventListener("keydown", (V) => {
      var _a2, _b2;
      if (!Y()) return;
      const st = (_b2 = (_a2 = V.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (st === "INPUT" || st === "TEXTAREA" || st === "SELECT") return;
      const it = V.key, lt = () => {
        V.stopImmediatePropagation(), V.preventDefault();
      };
      it === "Escape" ? (j ? D() : z(), lt()) : it === "ArrowLeft" ? (K(A - 1), lt()) : it === "ArrowRight" ? (K(A + 1), lt()) : it === "Home" ? (K(0), lt()) : it === "End" && E ? (K(E.iterations.length - 1), lt()) : it === "w" || it === "W" ? (O(-1), lt()) : it === "s" || it === "S" ? (O(1), lt()) : it === "a" || it === "A" ? (B(-1), lt()) : it === "d" || it === "D" ? (B(1), lt()) : (it === "i" || it === "I") && (J(), lt());
    }, {
      capture: true
    }), {
      open: G,
      close: z,
      isOpen: Y,
      inlineEl: s,
      getSelection: ft,
      setEditMode: mt
    };
  }
  function gv(n, t) {
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
  function rh(n) {
    switch (n.outcome.kind) {
      case "Solved":
        return `Solved \xB7 ${n.outcome.regionTiles}t`;
      case "Capped":
        return `Capped \xB7 ${n.outcome.iters} iter`;
      case "Open":
        return "Open";
    }
  }
  function yv(n, t) {
    if (t) {
      if (t.veto) return [
        {
          text: `veto \xB7 ${Vi(t.veto.broken_segment, 22)} @ (${t.veto.break_tile_x},${t.veto.break_tile_y})`,
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
        const i = s.detail ? ` \xB7 ${Vi(s.detail, 28)}` : "";
        return [
          {
            text: `${s.strategy} \u2192 ${Vi(s.outcome, 12)}${i}`,
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
            text: `cap: ${Vi(n.outcome.reason, 32)}`,
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
  function Vi(n, t) {
    return n.length <= t ? n : `${n.slice(0, t - 1)}\u2026`;
  }
  function qu(n) {
    var _a2;
    return ((_a2 = n.sat) == null ? void 0 : _a2.boundaries) ?? n.boundaries;
  }
  function xv(n) {
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
  function bv(n, t, e) {
    n.innerHTML = "", n.appendChild(_v(t)), n.appendChild(wv(t, e)), e && (n.appendChild(vv(e)), n.appendChild(Cv(e)), n.appendChild(Sv(e)), e.veto && n.appendChild(Ev(e))), t.nearbyStamped.length > 0 && n.appendChild(Tv(t));
  }
  function is(n, t = true) {
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
  function _v(n) {
    const { details: t, bodyEl: e } = is("Summary"), s = document.createElement("div");
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
  function wv(n, t) {
    const { details: e, bodyEl: s } = is("Participating specs");
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
  function vv(n) {
    const t = qu(n), e = n.sat ? " (as fed to SAT)" : " (spec perimeter)", { details: s, bodyEl: i } = is(`Boundaries${e}`);
    if (t.length === 0) {
      const r = document.createElement("div");
      return r.className = "jd-row jd-row--dim", r.textContent = "(none)", i.appendChild(r), s;
    }
    for (const r of t) {
      const o = document.createElement("div");
      o.className = "jd-row";
      const a = r.is_input ? "IN " : "OUT", l = r.interior ? " (interior)" : "", c = r.external_feeder ? ` \u2190 ${Uw(r.external_feeder)}` : "";
      o.style.color = r.is_input ? "#9f9" : "#f99";
      const h = r.spec_key ? ` \xB7 ${r.spec_key}` : "";
      o.textContent = `${a} (${r.x},${r.y}) ${xv(r.direction)} ${r.direction} \xB7 ${r.item}${l}${h}${c}`, i.appendChild(o);
    }
    return s;
  }
  function Cv(n) {
    const { details: t, bodyEl: e } = is("Strategy attempts");
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
  function Sv(n) {
    const { details: t, bodyEl: e } = is("SAT", !!n.sat);
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
  function Ev(n) {
    const { details: t, bodyEl: e } = is("Walker veto");
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
  function Tv(n) {
    const { details: t, bodyEl: e } = is("Nearby stamped", false);
    for (const s of n.nearbyStamped) {
      const i = document.createElement("div");
      i.className = "jd-row";
      const r = s.carries ? ` carries=${s.carries}` : "", o = s.segment_id ? ` \xB7 seg=${s.segment_id}` : "";
      i.textContent = `(${s.x},${s.y}) ${s.name} ${s.direction}${r}${o}${s.feeds_seed_area ? "  \u26A0 feeds seed" : ""}`, e.appendChild(i);
    }
    return t;
  }
  const Ku = {
    "transport-belt": 4,
    "fast-transport-belt": 6,
    "express-transport-belt": 8
  };
  function da(n, t, e, s) {
    var _a2, _b2;
    const i = n.seed, r = (t == null ? void 0 : t.bbox) ?? {
      x: i.x,
      y: i.y,
      w: 1,
      h: 1
    }, o = (t == null ? void 0 : t.iter) ?? 0, a = ((_a2 = t == null ? void 0 : t.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", l = ((_b2 = t == null ? void 0 : t.sat) == null ? void 0 : _b2.max_reach) ?? Ku[a] ?? 4, c = qu(t ?? {
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
    })), h = t && e ? Yw(e, r, 2).map((p) => ({
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
  function Av(n, t) {
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
  function kv(n) {
    return n.x === void 0 || n.y === void 0 || n.direction === void 0 ? null : n;
  }
  const Xi = {
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
  }, oh = {
    North: "East",
    East: "South",
    South: "West",
    West: "North"
  }, qi = {
    "transport-belt": "transport-belt",
    "fast-transport-belt": "fast-transport-belt",
    "express-transport-belt": "express-transport-belt"
  }, ah = {
    "transport-belt": "underground-belt",
    "fast-transport-belt": "fast-underground-belt",
    "express-transport-belt": "express-underground-belt"
  };
  function Mv(n) {
    const { viewport: t, canvas: e, engine: s, jd: i, satZoneOverlayLayer: r } = n;
    let o = null, a = null, l = null, c = null, h = null, d = null, u = false, p = null, f = [], g = [], m = [], y = [], b = "belt", x = "East", _ = 0, v = null, w = "idle", C = 0, T = null, R = null;
    function E(k, M) {
      return `${k},${M}`;
    }
    function A(k, M) {
      if (!p) return false;
      const X = p.bbox;
      return k >= X.x && k < X.x + X.w && M >= X.y && M < X.y + X.h;
    }
    function W(k, M) {
      return f.find((X) => X.x === k && X.y === M);
    }
    function j() {
      if (!p || p.items.length === 0) return null;
      const k = Math.max(0, Math.min(_, p.items.length - 1));
      return p.items[k] ?? null;
    }
    function G(k, M, X) {
      if (!p) return null;
      const [ot, $] = Xi[X], L = k - ot, Z = M - $, U = W(L, Z);
      if (U && U.direction === X && (qi[U.name] === U.name || U.io_type === "output")) return U.carries ?? null;
      const rt = p.boundaries.find((gt) => gt.x === k && gt.y === M && gt.isInput && gt.dir === X);
      return rt ? rt.item : null;
    }
    function z(k, M) {
      return G(k.x, k.y, M) ?? j();
    }
    function P(k, M) {
      if (!p) return null;
      const [X, ot] = Xi[M];
      for (let $ = 1; $ <= p.maxReach + 1; $++) {
        const L = k.x - X * $, Z = k.y - ot * $, U = W(L, Z);
        if (U && U.io_type === "input" && U.direction === M) return U.carries ?? null;
      }
      return null;
    }
    function D() {
      g.push(f.map((k) => ({
        ...k
      }))), g.length > 64 && g.shift(), m.length = 0;
    }
    function J(k, M) {
      D(), f = k, K(), ut();
    }
    function Y() {
      if (!p) return [];
      const k = qi[p.beltTier] ?? "transport-belt", M = [];
      for (const X of p.boundaries) {
        if (!X.isInput) continue;
        const [ot, $] = Xi[X.dir];
        M.push({
          name: k,
          x: X.x - ot,
          y: X.y - $,
          direction: X.dir,
          carries: X.item
        });
      }
      return M;
    }
    function K() {
      const k = Y();
      o && li({
        entities: f
      }, o, void 0, void 0, void 0, k), a && li({
        entities: y
      }, a, void 0, void 0, void 0, k), I();
    }
    function I() {
      if (c) {
        c.removeChildren();
        for (const k of f) {
          const M = k.carries;
          if (!M) continue;
          const X = `/spaghettio/pr-536/icons/${M}.png`, ot = Fe.get(X);
          if (!ot) continue;
          const $ = new ze(ot), L = S * 0.55;
          $.width = L, $.height = L, $.x = k.x * S + (S - L) / 2, $.y = k.y * S + (S - L) / 2, $.alpha = 0.85, c.addChild($);
        }
      }
    }
    function N(k, M) {
      l && (li({
        entities: k
      }, l, void 0, void 0, void 0, Y()), l.alpha = M ? 0.5 : 0.45, l.tint = M ? 16733525 : 16777215);
    }
    function O() {
      l && (l.removeChildren(), l.tint = 16777215);
    }
    function B(k, M = "") {
      if (w = k, !d) return;
      d.classList.remove("ok", "solving", "invalid", "idle"), d.classList.add(k === "valid" ? "ok" : k);
      const X = k === "valid" ? "\u25CF" : k === "solving" ? "\u25D4" : k === "invalid" ? "\u25CF" : "\u25CB";
      d.textContent = X;
      let ot = "";
      k === "valid" ? ot = R !== null ? `valid \xB7 cost ${R} / yours ${q(f)}` : "valid" : k === "solving" ? ot = "solving\u2026" : k === "invalid" ? ot = "invalid" : ot = "no edits yet", d.title = M ? `${ot}
${M}` : ot, H();
    }
    function q(k) {
      let M = 0;
      for (const X of k) qi[X.name] === X.name ? M += 1 : ah[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] === X.name && (M += 5);
      return M;
    }
    function Q() {
      if (!p) return "no zone";
      const k = /* @__PURE__ */ new Set();
      for (const M of f) {
        const X = E(M.x, M.y);
        if (k.has(X)) return `duplicate entity at (${M.x},${M.y})`;
        if (k.add(X), !A(M.x, M.y)) return `entity at (${M.x},${M.y}) outside bbox`;
        if (p.forbidden.has(X)) return `entity at (${M.x},${M.y}) on forbidden tile`;
      }
      for (const M of f) {
        if (M.io_type !== "input") continue;
        const [X, ot] = Xi[M.direction];
        let $ = false;
        for (let L = 1; L <= p.maxReach + 1; L++) {
          const Z = M.x + X * L, U = M.y + ot * L, rt = W(Z, U);
          if (rt) {
            if (rt.io_type === "output" && rt.direction === M.direction && rt.carries === M.carries) {
              $ = true;
              break;
            }
            if (rt.io_type === "input" && rt.carries === M.carries) return `UG-in at (${M.x},${M.y}) blocked by another UG-in at (${Z},${U})`;
          }
        }
        if (!$) return `UG-in at (${M.x},${M.y}) has no matching UG-out within reach ${p.maxReach}`;
      }
      return null;
    }
    function ut() {
      if (!u || !p) return;
      const k = Q();
      if (k) {
        y = [], R = null, K(), B("invalid", k);
        return;
      }
      B("solving"), T !== null && window.clearTimeout(T);
      const M = ++C;
      T = window.setTimeout(() => {
        ft(M);
      }, 300);
    }
    async function ft(k) {
      if (p) try {
        const M = await s.solveFixture(p.fixtureJson, f);
        if (k !== C || !u) return;
        if (!M) {
          y = [], R = null, K(), B("invalid", "SAT cannot complete this layout");
          return;
        }
        const X = new Set(f.map(($) => E($.x, $.y))), ot = [];
        for (const $ of M.entities) {
          const L = kv($);
          L && !X.has(E(L.x, L.y)) && ot.push(L);
        }
        y = ot, R = M.cost, K(), B("valid");
      } catch (M) {
        if (k !== C) return;
        y = [], K(), B("invalid", `solver error: ${M instanceof Error ? M.message : String(M)}`);
      }
    }
    function mt(k) {
      const M = e.getBoundingClientRect(), X = t.toWorld(k.clientX - M.left, k.clientY - M.top);
      return {
        x: Math.floor(X.x / S),
        y: Math.floor(X.y / S)
      };
    }
    function wt(k, M, X) {
      const ot = [], $ = M.x === k.x ? 0 : M.x > k.x ? 1 : -1, L = M.y === k.y ? 0 : M.y > k.y ? 1 : -1;
      if (X) {
        for (let U = k.y; U !== M.y + L && L !== 0; U += L) ot.push({
          x: k.x,
          y: U
        });
        L === 0 && ot.push({
          x: k.x,
          y: k.y
        });
        for (let U = k.x + $; $ !== 0 && U !== M.x + $; U += $) ot.push({
          x: U,
          y: M.y
        });
      } else {
        for (let U = k.x; U !== M.x + $ && $ !== 0; U += $) ot.push({
          x: U,
          y: k.y
        });
        $ === 0 && ot.push({
          x: k.x,
          y: k.y
        });
        for (let U = k.y + L; L !== 0 && U !== M.y + L; U += L) ot.push({
          x: M.x,
          y: U
        });
      }
      const Z = [];
      for (const U of ot) {
        const rt = Z[Z.length - 1];
        (!rt || rt.x !== U.x || rt.y !== U.y) && Z.push(U);
      }
      return Z;
    }
    function V(k, M) {
      return k.x === M.x && k.y === M.y - 1 ? "South" : k.x === M.x && k.y === M.y + 1 ? "North" : k.y === M.y && k.x === M.x - 1 ? "East" : k.y === M.y && k.x === M.x + 1 ? "West" : null;
    }
    function st(k) {
      if (!p || k.length === 0) return null;
      for (const rt of k) if (!A(rt.x, rt.y)) return null;
      const M = (rt) => !p.forbidden.has(E(rt.x, rt.y)), X = k[0], ot = k[k.length - 1];
      if (!X || !ot || !M(X) || !M(ot)) return null;
      const $ = [], L = k.length > 1 ? V(k[0], k[1]) ?? x : x, Z = z(k[0], L);
      let U = 0;
      for (; U < k.length; ) {
        const rt = k[U], gt = k[U + 1] ?? null, Mt = gt ? V(rt, gt) : U > 0 ? V(k[U - 1], rt) : x;
        if (!Mt) return null;
        let nt = U + 1;
        for (; nt < k.length && M(k[nt]); ) nt++;
        if (nt === k.length) {
          $.push(it(rt, Mt, Z)), U++;
          continue;
        }
        let Tt = nt;
        for (; Tt < k.length && !M(k[Tt]); ) Tt++;
        if (Tt === k.length) return null;
        const vt = k[nt - 1], Wt = k[Tt];
        if (Math.abs(Wt.x - vt.x) + Math.abs(Wt.y - vt.y) > p.maxReach + 1) return null;
        for (let le = U; le < nt - 1; le++) {
          const Fn = k[le], rs = V(Fn, k[le + 1]);
          if (!rs) return null;
          $.push(it(Fn, rs, Z));
        }
        const re = V(vt, Wt);
        if (!re) return null;
        $.push(lt(vt, re, "input", Z)), $.push(lt(Wt, re, "output", Z)), U = Tt + 1;
      }
      return $;
    }
    function it(k, M, X) {
      return {
        name: qi[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "transport-belt",
        x: k.x,
        y: k.y,
        direction: M,
        carries: X ?? void 0
      };
    }
    function lt(k, M, X, ot) {
      return {
        name: ah[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "underground-belt",
        x: k.x,
        y: k.y,
        direction: M,
        io_type: X,
        carries: ot ?? void 0
      };
    }
    function Et(k) {
      const M = new Set(k.map((ot) => E(ot.x, ot.y))), X = f.filter((ot) => !M.has(E(ot.x, ot.y))).concat(k);
      J(X);
    }
    function Nt(k) {
      if (!p || !A(k.x, k.y) || !f.some((X) => X.x === k.x && X.y === k.y)) return;
      const M = f.filter((X) => !(X.x === k.x && X.y === k.y));
      J(M);
    }
    function xt(k) {
      if (!p) return;
      const M = p.boundaries.find((ot) => ot.x === k.x && ot.y === k.y && ot.isInput);
      if (!M) return;
      const X = p.items.indexOf(M.item);
      X >= 0 && X !== _ && (_ = X, fe());
    }
    function jt(k) {
      if (!u || !p || k.button !== 0) return;
      const M = mt(k);
      if (!M || !A(M.x, M.y)) return;
      if (xt(M), b === "erase") {
        Nt(M), k.stopPropagation(), k.preventDefault();
        return;
      }
      if (b === "ug-in" || b === "ug-out") {
        const $ = b === "ug-in" ? G(M.x, M.y, x) ?? j() : P(M, x) ?? G(M.x, M.y, x) ?? j(), L = b === "ug-in" ? lt(M, x, "input", $) : lt(M, x, "output", $), Z = f.filter((U) => !(U.x === M.x && U.y === M.y)).concat(L);
        J(Z), k.stopPropagation(), k.preventDefault();
        return;
      }
      v = {
        startX: M.x,
        startY: M.y,
        bendVerticalFirst: false
      };
      const X = wt({
        x: M.x,
        y: M.y
      }, M, false), ot = st(X);
      N(ot ?? [], ot === null), k.stopPropagation(), k.preventDefault();
    }
    function Vt(k) {
      if (!u || !v || !p) return;
      const M = mt(k);
      if (!M) return;
      const X = wt({
        x: v.startX,
        y: v.startY
      }, M, v.bendVerticalFirst), ot = st(X);
      N(ot ?? [], ot === null);
    }
    function te(k) {
      if (!u || !v || !p) {
        v = null, O();
        return;
      }
      const M = mt(k);
      if (!M) {
        v = null, O();
        return;
      }
      const X = wt({
        x: v.startX,
        y: v.startY
      }, M, v.bendVerticalFirst), ot = st(X);
      if (v = null, O(), !ot) {
        B("invalid", "drag rejected: out of bounds, on obstacle, or UG too long");
        return;
      }
      Et(ot), k.stopPropagation(), k.preventDefault();
    }
    function Lt(k) {
      var _a2, _b2;
      if (!u) return;
      const M = (_b2 = (_a2 = k.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (M === "INPUT" || M === "TEXTAREA" || M === "SELECT") return;
      const X = () => {
        k.stopImmediatePropagation(), k.preventDefault();
      };
      if (k.key === "Escape") {
        St(), X();
        return;
      }
      if (k.key === "1") {
        be("belt"), X();
        return;
      }
      if (k.key === "2") {
        be("ug-in"), X();
        return;
      }
      if (k.key === "3") {
        be("ug-out"), X();
        return;
      }
      if (k.key === "0") {
        be("erase"), X();
        return;
      }
      if (k.key === "r" || k.key === "R") {
        v ? v.bendVerticalFirst = !v.bendVerticalFirst : (x = oh[x], fe()), X();
        return;
      }
      if (k.key === "[" && p) {
        _ = (_ - 1 + p.items.length) % p.items.length, fe(), X();
        return;
      }
      if (k.key === "]" && p) {
        _ = (_ + 1) % p.items.length, fe(), X();
        return;
      }
      if ((k.key === "Enter" || k.key === "a" || k.key === "A") && w === "valid" && y.length > 0) {
        $t(), X();
        return;
      }
      if ((k.ctrlKey || k.metaKey) && (k.key === "z" || k.key === "Z")) {
        k.shiftKey ? _e() : Xt(), X();
        return;
      }
    }
    function be(k) {
      b = k, fe();
    }
    function Xt() {
      g.length !== 0 && (m.push(f), f = g.pop(), K(), ut());
    }
    function _e() {
      m.length !== 0 && (g.push(f), f = m.pop(), K(), ut());
    }
    function $t() {
      if (y.length === 0) return;
      const k = f.concat(y.map((M) => ({
        ...M
      })));
      y = [], J(k);
    }
    function fe() {
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
      for (const [rt, gt, Mt] of k) {
        const nt = document.createElement("button");
        nt.className = "se-tool" + (b === rt ? " se-tool-active" : ""), nt.textContent = gt, nt.title = Mt, nt.addEventListener("click", () => be(rt)), h.appendChild(nt);
      }
      const M = document.createElement("button");
      M.className = "se-dir";
      const X = {
        North: "\u2191",
        East: "\u2192",
        South: "\u2193",
        West: "\u2190"
      };
      if (M.textContent = X[x], M.title = "Brush direction (R rotates)", M.addEventListener("click", () => {
        x = oh[x], fe();
      }), h.appendChild(M), p && p.items.length > 1) {
        const rt = document.createElement("select");
        rt.className = "se-item";
        for (const [gt, Mt] of p.items.entries()) {
          const nt = document.createElement("option");
          nt.value = String(gt), nt.textContent = Mt, gt === _ && (nt.selected = true), rt.appendChild(nt);
        }
        rt.addEventListener("change", () => {
          _ = Number(rt.value) | 0;
        }), h.appendChild(rt);
      } else if (p && p.items.length === 1) {
        const rt = document.createElement("span");
        rt.className = "se-item-label", rt.textContent = p.items[0], h.appendChild(rt);
      }
      const ot = document.createElement("span");
      ot.style.flex = "1", h.appendChild(ot);
      const $ = document.createElement("button");
      $.className = "se-accept", $.textContent = "Accept", $.title = "Promote ghost into painted layer (Enter)", $.addEventListener("click", $t), $.disabled = !(w === "valid" && y.length > 0), h.appendChild($);
      const L = document.createElement("button");
      L.className = "se-revert", L.textContent = "Revert", L.title = "Discard all painted edits", L.addEventListener("click", () => {
        J([]);
      }), h.appendChild(L);
      const Z = document.createElement("button");
      Z.className = "se-export", Z.textContent = "Export", Z.title = "Save fixture JSON (clipboard + download)", Z.addEventListener("click", et), Z.disabled = w !== "valid", h.appendChild(Z);
      const U = document.createElement("button");
      U.className = "se-done", U.textContent = "Done", U.title = "Exit edit mode (Esc)", U.addEventListener("click", St), h.appendChild(U);
    }
    function H() {
      if (!h) return;
      const k = h.querySelector(".se-accept");
      k && (k.disabled = !(w === "valid" && y.length > 0));
      const M = h.querySelector(".se-export");
      M && (M.disabled = w !== "valid");
    }
    function et() {
      var _a2;
      if (!p || w !== "valid") return;
      const k = R ?? q(f), M = da(p.selection.cluster, p.selection.iter, p.selection.trace, {
        maxCost: k,
        paintedEntities: f
      }), X = (p.selection.cluster.seed ? `fixture_${p.selection.cluster.seed.x}_${p.selection.cluster.seed.y}_painted` : "fixture_painted") + ".json";
      (_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText(M).catch(() => {
      });
      const ot = new Blob([
        M
      ], {
        type: "application/json"
      }), $ = URL.createObjectURL(ot), L = document.createElement("a");
      L.href = $, L.download = X, document.body.appendChild(L), L.click(), document.body.removeChild(L), URL.revokeObjectURL($);
    }
    function tt(k) {
      var _a2, _b2, _c2, _d2;
      u && St();
      const M = k.iter, X = ((_a2 = M.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", ot = ((_b2 = M.sat) == null ? void 0 : _b2.max_reach) ?? Ku[X] ?? 4, $ = ((_c2 = M.sat) == null ? void 0 : _c2.boundaries) ?? M.boundaries, L = Array.from(new Set($.map((U) => U.item))), Z = $.map((U) => ({
        x: U.x,
        y: U.y,
        item: U.item,
        isInput: U.is_input,
        dir: U.direction
      }));
      p = {
        bbox: {
          x: M.bbox.x,
          y: M.bbox.y,
          w: M.bbox.w,
          h: M.bbox.h
        },
        forbidden: new Set((M.forbidden ?? []).map((U) => `${U[0]},${U[1]}`)),
        beltTier: X,
        maxReach: ot,
        items: L,
        boundaries: Z,
        fixtureJson: da(k.cluster, k.iter, k.trace),
        selection: k
      }, f = [], g = [], m = [], y = [], b = "belt", x = "East", _ = 0, v = null, R = null, o = new Ht(), a = new Ht(), a.alpha = 0.55, l = new Ht(), c = new Ht(), t.addChild(o), t.addChild(a), t.addChild(c), t.addChild(l), t.setChildIndex(r, t.children.length - 1), h = document.createElement("div"), h.className = "se-toolbar", i.inlineEl.appendChild(h), d = document.createElement("span"), d.className = "se-status", (_d2 = i.inlineEl.querySelector(".jd-inline-head")) == null ? void 0 : _d2.appendChild(d), i.setEditMode(true), t.plugins.pause("drag"), e.addEventListener("pointerdown", jt, {
        capture: true
      }), e.addEventListener("pointerup", te, {
        capture: true
      }), e.addEventListener("pointermove", Vt, {
        capture: true
      }), document.addEventListener("keydown", Lt, {
        capture: true
      }), u = true, fe(), B("idle");
    }
    function St() {
      u && (u = false, T !== null && (window.clearTimeout(T), T = null), C++, o && (t.removeChild(o), o.destroy({
        children: true
      }), o = null), a && (t.removeChild(a), a.destroy({
        children: true
      }), a = null), l && (t.removeChild(l), l.destroy({
        children: true
      }), l = null), c && (t.removeChild(c), c.destroy({
        children: true
      }), c = null), h && (h.remove(), h = null), d && (d.remove(), d = null), e.removeEventListener("pointerdown", jt, {
        capture: true
      }), e.removeEventListener("pointerup", te, {
        capture: true
      }), e.removeEventListener("pointermove", Vt, {
        capture: true
      }), document.removeEventListener("keydown", Lt, {
        capture: true
      }), t.plugins.resume("drag"), i.setEditMode(false), p = null, f = [], g = [], m = [], y = []);
    }
    function zt() {
      return u;
    }
    return {
      enter: tt,
      exit: St,
      isActive: zt
    };
  }
  let Ts = {
    master: false,
    stepThrough: true,
    satZones: false,
    soloRegions: false,
    ghostTiles: false,
    itemColors: true,
    traceOverlay: false,
    heatmap: false,
    powerWires: false,
    moduleSlots: true,
    simState: true
  };
  const sr = [];
  function Pv() {
    const n = new URLSearchParams(window.location.search).get("debug") === "1", t = localStorage.getItem("fk-debug") === "1", e = localStorage.getItem("fk-sat-zones") === "1", s = localStorage.getItem("fk-ghost-tiles") === "1", i = localStorage.getItem("fk-item-colors"), r = localStorage.getItem("fk-trace-overlay") === "1", o = localStorage.getItem("fk-heatmap") === "1", a = localStorage.getItem("fk-power-wires") === "1", l = localStorage.getItem("fk-module-slots"), c = localStorage.getItem("fk-sim-state");
    Ts = {
      ...Ts,
      master: n || t,
      satZones: e,
      ghostTiles: s,
      itemColors: i === null ? true : i === "1",
      traceOverlay: r,
      heatmap: o,
      powerWires: a,
      moduleSlots: l === null ? true : l === "1",
      simState: c === null ? true : c === "1"
    };
  }
  function Ju() {
    return Ts;
  }
  function Ze(n) {
    Ts = {
      ...Ts,
      ...n
    }, "master" in n && localStorage.setItem("fk-debug", n.master ? "1" : "0"), "satZones" in n && localStorage.setItem("fk-sat-zones", n.satZones ? "1" : "0"), "ghostTiles" in n && localStorage.setItem("fk-ghost-tiles", n.ghostTiles ? "1" : "0"), "itemColors" in n && localStorage.setItem("fk-item-colors", n.itemColors ? "1" : "0"), "traceOverlay" in n && localStorage.setItem("fk-trace-overlay", n.traceOverlay ? "1" : "0"), "heatmap" in n && localStorage.setItem("fk-heatmap", n.heatmap ? "1" : "0"), "powerWires" in n && localStorage.setItem("fk-power-wires", n.powerWires ? "1" : "0"), "moduleSlots" in n && localStorage.setItem("fk-module-slots", n.moduleSlots ? "1" : "0"), "simState" in n && localStorage.setItem("fk-sim-state", n.simState ? "1" : "0");
    for (const t of sr) t(Ts);
  }
  function Iv(n) {
    return sr.push(n), () => {
      const t = sr.indexOf(n);
      t >= 0 && sr.splice(t, 1);
    };
  }
  function Qe(n, t, e = false) {
    const s = document.createElement("input");
    s.type = "checkbox", s.checked = e;
    const i = document.createElement("div");
    i.className = "overlay-toggle";
    const r = document.createElement("label");
    return r.appendChild(s), r.appendChild(document.createTextNode(t)), i.appendChild(r), n.appendChild(i), s;
  }
  function Rv(n) {
    n.style.position = "relative";
    const t = document.createElement("div");
    t.className = "overlay-panel";
    const e = Ju(), s = Qe(t, "Debug", e.master), i = Qe(t, "Item colours", e.itemColors), r = Qe(t, "Starvation heatmap", e.heatmap), o = Qe(t, "Power wires", e.powerWires), a = Qe(t, "Module slots", e.moduleSlots), l = document.createElement("div");
    l.className = "overlay-sim-report-row";
    const c = document.createElement("button");
    c.type = "button", c.textContent = "Load sim report\u2026", c.className = "overlay-sim-report-btn";
    const h = document.createElement("input");
    h.type = "file", h.accept = ".json,application/json", h.style.display = "none", c.addEventListener("click", () => h.click()), l.appendChild(c), l.appendChild(h), t.appendChild(l);
    const d = Qe(t, "Sim state", e.simState);
    d.disabled = true;
    const u = document.createElement("div");
    u.className = "overlay-sub-panel", u.style.display = e.master ? "flex" : "none";
    const p = Qe(u, "SAT Zones", e.satZones), f = Qe(u, "Ghost tiles", e.ghostTiles), g = Qe(u, "Trace overlay", e.traceOverlay), m = Qe(u, "Solo regions", e.soloRegions);
    return t.appendChild(u), n.appendChild(t), s.addEventListener("change", () => {
      u.style.display = s.checked ? "flex" : "none", Ze({
        master: s.checked
      });
    }), p.addEventListener("change", () => {
      Ze({
        satZones: p.checked
      });
    }), f.addEventListener("change", () => {
      Ze({
        ghostTiles: f.checked
      });
    }), g.addEventListener("change", () => {
      Ze({
        traceOverlay: g.checked
      });
    }), i.addEventListener("change", () => {
      Ze({
        itemColors: i.checked
      });
    }), r.addEventListener("change", () => {
      Ze({
        heatmap: r.checked
      });
    }), o.addEventListener("change", () => {
      Ze({
        powerWires: o.checked
      });
    }), a.addEventListener("change", () => {
      Ze({
        moduleSlots: a.checked
      });
    }), d.addEventListener("change", () => {
      Ze({
        simState: d.checked
      });
    }), {
      setDebugEnabled(y) {
        s.checked = y, u.style.display = y ? "flex" : "none", Ze({
          master: y
        });
      },
      debugCb: s,
      colorCb: i,
      heatmapCb: r,
      powerWiresCb: o,
      moduleSlotsCb: a,
      regionsCb: p,
      soloRegionsCb: m,
      ghostTilesCb: f,
      traceOverlayCb: g,
      simReportFileInput: h,
      simStateCb: d
    };
  }
  function Lv(n) {
    const t = document.createElement("div");
    t.className = "retry-panel", n.appendChild(t);
    let e = null;
    function s(r) {
      if (!(r == null ? void 0 : r.trace)) return null;
      for (const o of r.trace) if (o.phase === "LayoutRetried") return o.data;
      return null;
    }
    function i() {
      const r = Ju().master, o = s(e);
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
    return Iv(() => i()), {
      update(r) {
        e = r, i();
      }
    };
  }
  const $v = {
    North: "\u2191",
    East: "\u2192",
    South: "\u2193",
    West: "\u2190"
  }, lh = {
    N: "\u2191",
    E: "\u2192",
    S: "\u2193",
    W: "\u2190"
  };
  function un(n = 16) {
    const t = document.createElement("img");
    return t.width = n, t.height = n, t.style.cssText = "vertical-align:middle;margin-right:3px;image-rendering:pixelated", t.addEventListener("error", () => {
      t.style.display = "none";
    }), t;
  }
  function gs(n, t) {
    n.style.display = "", n.src = `/spaghettio/pr-536/icons/${t}.png`;
  }
  function Co(n, t) {
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
  function Ov(n) {
    const t = document.createElement("div");
    t.style.cssText = "position:fixed;background:#1e1e1e;color:#e0e0e0;border:1px solid #555;padding:4px 8px;font:12px monospace;pointer-events:none;border-radius:3px;display:none;z-index:1000;max-width:240px;line-height:1.5", document.body.appendChild(t);
    const e = document.createElement("div");
    t.appendChild(e);
    const s = document.createElement("span");
    s.style.color = "#888", s.style.display = "none", t.appendChild(s);
    const i = document.createElement("div");
    t.appendChild(i);
    const r = document.createElement("div"), o = un(16), a = document.createElement("b");
    r.append(o, a), r.style.display = "none", i.appendChild(r);
    const l = document.createElement("div");
    l.style.display = "none", i.appendChild(l);
    const c = document.createElement("div"), h = un(16), d = document.createElement("span");
    c.append(h, d), c.style.display = "none", i.appendChild(c);
    const u = document.createElement("div");
    u.style.color = "#b5cea8", u.style.display = "none", i.appendChild(u);
    const p = document.createElement("div");
    p.style.display = "none", i.appendChild(p);
    const f = document.createElement("div"), g = un(16), m = document.createElement("span");
    f.append(g, m), f.style.display = "none", i.appendChild(f);
    function y() {
      const $ = document.createElement("div");
      $.style.color = "#aaa";
      const L = document.createElement("span"), Z = un(14), U = document.createElement("span");
      return $.append(L, Z, U), $;
    }
    const b = Co(i, y), x = document.createElement("div");
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
    const R = document.createElement("div");
    R.style.cssText = "display:flex;justify-content:space-between;align-items:center;margin-bottom:4px";
    const E = document.createElement("span");
    E.style.cssText = "color:#8af;font-weight:bold", E.textContent = "pinned";
    const A = document.createElement("span");
    A.style.color = "#888", R.append(E, A), T.appendChild(R);
    const W = document.createElement("div");
    T.appendChild(W);
    const j = document.createElement("div"), G = un(16), z = document.createElement("b");
    j.append(G, z), j.style.display = "none", W.appendChild(j);
    const P = document.createElement("span");
    P.style.color = "#888", P.textContent = "no entity at tile", P.style.display = "none", W.appendChild(P);
    const D = document.createElement("div");
    D.style.display = "none", W.appendChild(D);
    const J = document.createElement("div"), Y = un(16), K = document.createElement("span");
    J.append(Y, K), J.style.display = "none", W.appendChild(J);
    const I = document.createElement("div");
    I.style.color = "#b5cea8", I.style.display = "none", W.appendChild(I);
    const N = document.createElement("div");
    N.style.display = "none", W.appendChild(N);
    const O = document.createElement("div"), B = un(16), q = document.createElement("span");
    O.append(B, q), O.style.display = "none", W.appendChild(O);
    const Q = Co(W, y), ut = document.createElement("div");
    ut.style.color = "#9cdcfe", ut.style.display = "none", W.appendChild(ut);
    const ft = document.createElement("div");
    ft.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", ft.style.display = "none";
    const mt = document.createElement("span"), wt = document.createElement("span");
    wt.style.color = "#888", ft.append(mt, wt), T.appendChild(ft);
    const V = document.createElement("div");
    V.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", V.style.display = "none", T.appendChild(V);
    const st = document.createElement("div");
    st.style.marginTop = "4px", st.style.display = "none", T.appendChild(st);
    const it = document.createElement("div");
    it.style.display = "none", T.appendChild(it);
    const lt = document.createElement("div");
    lt.style.marginTop = "4px", it.appendChild(lt);
    function Et() {
      const $ = document.createElement("div");
      return $.style.marginLeft = "4px", $;
    }
    const Nt = Co(it, Et), xt = document.createElement("div");
    xt.style.cssText = "color:#555;margin-top:6px;font-size:10px", xt.textContent = "click elsewhere or press Esc to unpin", T.appendChild(xt), document.addEventListener("mousemove", ($) => {
      t.style.left = $.clientX + 14 + "px", t.style.top = $.clientY - 10 + "px";
    });
    let jt = null, Vt = null, te = null, Lt = null, be = null, Xt = null;
    const _e = /* @__PURE__ */ new Set();
    function $t() {
      const $ = Xt ? {
        x: Xt.x,
        y: Xt.y
      } : null;
      for (const L of _e) L($);
    }
    function fe($, L) {
      gs(L.headerIcon, $.name), L.headerName.textContent = xe($.name), L.header.style.display = "", $.direction && $.name !== "pipe" ? (L.dirRow.textContent = `${$v[$.direction] ?? ""} ${$.direction}`, L.dirRow.style.display = "") : L.dirRow.style.display = "none", $.carries ? (gs(L.carriesIcon, $.carries), L.carriesName.textContent = " " + xe($.carries), L.carriesRow.style.display = "") : L.carriesRow.style.display = "none", $.rate != null ? (L.rateRow.textContent = `${$.rate.toFixed(1)}/s`, L.rateRow.style.display = "") : L.rateRow.style.display = "none", $.io_type ? (L.ioRow.textContent = `io: ${$.io_type}`, L.ioRow.style.display = "") : L.ioRow.style.display = "none";
      let Z = 0;
      if ($.recipe) {
        gs(L.recipeIcon, $.recipe), L.recipeName.textContent = " " + xe($.recipe), L.recipeRow.style.display = "";
        const U = I0($.recipe);
        if (U) {
          const rt = [
            ...U.inputs.map((gt) => ({
              arrow: "\u25B6",
              item: gt.item,
              rate: gt.rate
            })),
            ...U.outputs.map((gt) => ({
              arrow: "\u25C0",
              item: gt.item,
              rate: gt.rate
            }))
          ];
          for (const gt of rt) {
            const Mt = L.flowPool.get(Z++), [nt, Tt, vt] = Mt.children;
            nt.textContent = `${gt.arrow} `, gs(Tt, gt.item), vt.textContent = `${xe(gt.item)} ${gt.rate.toFixed(1)}/s`;
          }
        }
      } else L.recipeRow.style.display = "none";
      return L.flowPool.trim(Z), $.segment_id ? (L.segmentRow.textContent = $.segment_id, L.segmentRow.style.display = "") : L.segmentRow.style.display = "none", Z;
    }
    function H($) {
      if ($.ghosts.length === 0) return _.style.display = "none", false;
      if (_.style.display = "", $.ghosts.length === 1) {
        const L = $.ghosts[0], Z = L.direction ? lh[L.direction] : "";
        _.textContent = "";
        const U = document.createElement("span");
        U.style.color = "#8af", U.textContent = "ghost ";
        const rt = un(12);
        gs(rt, L.item);
        const gt = document.createTextNode(`${L.item} ${Z}`);
        _.append(U, rt, gt);
      } else {
        _.textContent = "";
        const L = document.createElement("span");
        L.style.color = "#8af", L.textContent = `${$.ghosts.length} ghosts crossing`, _.appendChild(L);
      }
      return true;
    }
    function et($) {
      if (!$.axis) return v.style.display = "none", false;
      const { vert: L, horiz: Z } = $.axis;
      if (L === 0 && Z === 0) return v.style.display = "none", false;
      const U = L >= 2 || Z >= 2, rt = L >= 1 && Z >= 1, gt = U ? "#ff6060" : rt ? "#60b0ff" : "#888";
      return v.style.display = "", v.style.color = gt, v.textContent = `axis V${L} H${Z}`, true;
    }
    function tt($) {
      if (!$.junction) return w.style.display = "none", false;
      const L = $.junction, Z = L.outcome === "Solved" ? "#80d080" : L.outcome === "Capped" ? "#e0b060" : "#c06060";
      return w.style.display = "", w.style.color = Z, w.textContent = `junction seed (${L.seedX},${L.seedY}) \xB7 ${L.outcome}`, true;
    }
    function St($) {
      if ($.cappedSides.length === 0) return C.style.display = "none", false;
      const L = $.cappedSides.map((Z) => {
        const U = Z.required - Z.shortfall;
        return `\u26A0 ${Z.sideIsOutput ? "out" : "in"}: ${Z.placedCount}\xD7${Z.placedEntity} moves ${U.toFixed(2)}/s of ${Z.required.toFixed(2)}/s \xB7 ${Z.limit}`;
      });
      return C.style.display = "", C.style.color = "#ffa060", C.textContent = L.join(" | "), true;
    }
    function zt($) {
      if ($.ghosts.length === 0) {
        it.style.display = "none";
        return;
      }
      it.style.display = "", $.ghosts.length >= 2 ? (lt.style.color = "#ffa060", lt.textContent = `\u26A0 ${$.ghosts.length} ghost specs at this tile`) : (lt.style.color = "#8af", lt.textContent = "ghost");
      let L = 0;
      for (const Z of $.ghosts) {
        const U = Z.direction ? lh[Z.direction] : "\xB7", rt = Nt.get(L++);
        rt.textContent = "";
        const gt = document.createTextNode(`${U} `), Mt = un(14);
        gs(Mt, Z.item);
        const nt = document.createTextNode(Z.item);
        if (rt.append(gt, Mt, nt), Z.isStart) {
          const Tt = document.createElement("span");
          Tt.style.color = "#80d080", Tt.textContent = " start", rt.appendChild(Tt);
        } else if (Z.isEnd) {
          const Tt = document.createElement("span");
          Tt.style.color = "#d08080", Tt.textContent = " end", rt.appendChild(Tt);
        }
      }
      Nt.trim(L);
    }
    function k() {
      if (Vt !== null) {
        e.innerHTML = Vt, e.style.display = "", i.style.display = "none", _.style.display = "none", v.style.display = "none", w.style.display = "none", Lt ? (s.textContent = `(${Lt.x}, ${Lt.y})`, s.style.display = "", s.style.display = "block") : s.style.display = "none", t.style.display = "block";
        return;
      }
      e.style.display = "none", e.innerHTML = "", i.style.display = "";
      let $ = false;
      if (te ? ($ = true, fe(te, {
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
      })) : (r.style.display = "none", l.style.display = "none", c.style.display = "none", u.style.display = "none", p.style.display = "none", f.style.display = "none", b.trim(0), x.style.display = "none"), Lt) {
        const L = be == null ? void 0 : be.lookup(Lt.x, Lt.y);
        L ? (H(L) && ($ = true), et(L) && ($ = true), tt(L) && ($ = true), St(L) && ($ = true)) : (_.style.display = "none", v.style.display = "none", w.style.display = "none", C.style.display = "none"), s.textContent = `(${Lt.x}, ${Lt.y})`, s.style.display = "block", $ = true;
      } else s.style.display = "none", _.style.display = "none", v.style.display = "none", w.style.display = "none";
      if (!$) {
        t.style.display = "none", jt && jt.clearHighlight();
        return;
      }
      t.style.display = "block", te && jt ? jt.highlightBeltNetwork(te) : jt && jt.clearHighlight();
    }
    function M() {
      if (!Xt) {
        T.style.display = "none";
        return;
      }
      const { entity: $, x: L, y: Z } = Xt, U = be == null ? void 0 : be.lookup(L, Z);
      if (A.textContent = `(${L}, ${Z})`, $ ? (P.style.display = "none", fe($, {
        header: j,
        headerIcon: G,
        headerName: z,
        dirRow: D,
        carriesRow: J,
        carriesIcon: Y,
        carriesName: K,
        rateRow: I,
        ioRow: N,
        recipeRow: O,
        recipeIcon: B,
        recipeName: q,
        flowPool: Q,
        segmentRow: ut
      })) : (j.style.display = "none", D.style.display = "none", J.style.display = "none", I.style.display = "none", N.style.display = "none", O.style.display = "none", Q.trim(0), ut.style.display = "none", P.style.display = ""), U) {
        if (U.junction) {
          const rt = U.junction.outcome === "Solved" ? "#80d080" : U.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
          mt.style.color = rt, mt.textContent = `junction seed (${U.junction.seedX},${U.junction.seedY})`, wt.textContent = ` \xB7 ${U.junction.outcome}`, ft.style.display = "";
        } else ft.style.display = "none";
        if (U.axis) {
          const { vert: rt, horiz: gt } = U.axis;
          if (rt > 0 || gt > 0) {
            const Mt = rt >= 2 || gt >= 2, nt = rt >= 1 && gt >= 1, Tt = Mt ? " same-axis conflict" : nt ? " perpendicular crossing" : "", vt = Mt ? "#ff6060" : nt ? "#60b0ff" : "#bbb";
            st.style.color = vt, st.textContent = `axis: V=${rt} H=${gt}${Tt}`, st.style.display = "";
          } else st.style.display = "none";
        } else st.style.display = "none";
        if (zt(U), U.cappedSides.length > 0) {
          const rt = {
            "tier-cap": "a faster inserter tier at the same slot count would cover this \u2014 max inserter tier is the binding constraint",
            "column-contest": "this side lost the shared inserter column to the other belt; that one column would have covered it",
            geometry: "the row shape offers no further usable slot (belt span / fixed tiles) \u2014 a template geometry limit"
          };
          V.replaceChildren();
          const gt = document.createElement("div");
          gt.style.color = "#ffa060", gt.textContent = `\u26A0 ${U.cappedSides.length} under-provisioned inserter side${U.cappedSides.length > 1 ? "s" : ""}`, V.appendChild(gt);
          for (const Mt of U.cappedSides) {
            const nt = (Mt.required - Mt.shortfall).toFixed(2), Tt = document.createElement("div");
            Tt.style.cssText = "margin-top:2px;color:#ccc", Tt.textContent = `${Mt.sideIsOutput ? "output" : "input"}: ${Mt.placedCount}\xD7${Mt.placedEntity} moves ${nt}/s of ${Mt.required.toFixed(2)}/s needed (short ${Mt.shortfall.toFixed(2)}/s) \u2014 ${Mt.limit}: ${rt[Mt.limit] ?? Mt.limit}`, V.appendChild(Tt);
          }
          V.style.display = "";
        } else V.style.display = "none";
      } else ft.style.display = "none", st.style.display = "none", it.style.display = "none", V.style.display = "none";
      T.style.display = "block";
    }
    function X() {
      k(), M();
    }
    function ot($, L, Z) {
      te = $, L !== void 0 && Z !== void 0 ? Lt = {
        x: L,
        y: Z
      } : $ && (Lt = {
        x: $.x ?? 0,
        y: $.y ?? 0
      }), X();
    }
    return document.addEventListener("keydown", ($) => {
      $.key === "Escape" && Xt && (Xt = null, $t(), X());
    }), {
      onHover: ot,
      setHighlightController($) {
        jt = $;
      },
      setTooltipOverride($) {
        Vt = $, X();
      },
      setCursorTile($, L) {
        $ === null || L === void 0 ? Lt = null : Lt = {
          x: $,
          y: L
        }, X();
      },
      setTileContext($) {
        be = $, X();
      },
      pinTile($, L, Z) {
        Xt = {
          entity: $,
          x: L,
          y: Z
        }, $t(), X();
      },
      clearPin() {
        Xt = null, $t(), X();
      },
      getPinnedTile() {
        return Xt ? {
          x: Xt.x,
          y: Xt.y
        } : null;
      },
      onPinChange($) {
        return _e.add($), () => _e.delete($);
      }
    };
  }
  function Bv(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function ch(n, t, e, s) {
    return e > n ? "E" : e < n ? "W" : s > t ? "S" : s < t ? "N" : null;
  }
  const Nv = {
    ghosts: [],
    axis: null,
    junction: null,
    cappedSides: []
  };
  function Fv(n) {
    var _a2;
    if (!n || n.length === 0) return {
      lookup: () => Nv
    };
    const t = /* @__PURE__ */ new Map(), e = /* @__PURE__ */ new Map(), s = /* @__PURE__ */ new Map(), i = /* @__PURE__ */ new Map();
    for (const r of n) if (r.phase === "GhostSpecRouted") {
      const { spec_key: o, tiles: a } = r.data, l = Bv(o);
      if (!a || a.length === 0) continue;
      for (let c = 0; c < a.length; c++) {
        const [h, d] = a[c];
        let u = null;
        c < a.length - 1 ? u = ch(h, d, a[c + 1][0], a[c + 1][1]) : c > 0 && (u = ch(a[c - 1][0], a[c - 1][1], h, d));
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
  function Wv(n) {
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
      a && (t = iw(a, i, o), a.querySelectorAll("input,select,button").forEach((l) => {
        l.closest("[data-snapshot-keep]") || (l.disabled = true);
      }));
    }
    return {
      load: s,
      clear: e
    };
  }
  const Gv = 2200, hh = 180, So = 200, zv = 8, Dv = [
    "rows_placed",
    "lanes_planned",
    "bus_routed",
    "poles_placed"
  ];
  function mr(n) {
    return `${n.x ?? 0},${n.y ?? 0},${n.name},${n.recipe ?? ""}`;
  }
  function Hv(n) {
    const t = /* @__PURE__ */ new Map(), e = n.trace;
    if (!Array.isArray(e)) return t;
    for (const s of e) {
      const i = s;
      i.phase === "PhaseSnapshot" && i.data && t.set(i.data.phase, i.data);
    }
    return t;
  }
  function Uv(n) {
    const t = Hv(n), e = [], s = /* @__PURE__ */ new Set();
    for (const r of Dv) {
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
        const c = mr(l);
        s.has(c) || (s.add(c), a.push(l));
      }
      e.push({
        phase: r,
        entities: a
      });
    }
    const i = [];
    for (const r of n.entities) {
      const o = mr(r);
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
  function jv(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map(), o = li(n, t, e, s, (w, C) => {
      r.set(mr(w), C);
    }), a = /* @__PURE__ */ new Set();
    for (const w of r.values()) for (const C of w) a.add(C);
    const l = [];
    for (const w of t.children) {
      const C = w;
      a.has(C) || l.push(C);
    }
    for (const w of l) w.alpha = 0;
    for (const w of r.values()) for (const C of w) C.alpha = 0;
    const c = Uv(n), h = c.reduce((w, C) => w + (C.entities.length > 0 ? 1 : 0), 0);
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
    const u = Math.max(0, Gv - hh * h) / h, p = [], f = /* @__PURE__ */ new Map();
    let g = 0;
    for (const w of c) {
      if (w.entities.length === 0) continue;
      f.set(w.phase, g);
      const C = Math.min(zv, u / w.entities.length);
      w.entities.forEach((R, E) => {
        const A = r.get(mr(R));
        !A || A.length === 0 || p.push({
          graphics: A,
          revealStartMs: g + E * C
        });
      });
      const T = (w.entities.length - 1) * C;
      g += T + So + hh;
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
        const R = Math.min(1, (w - T.revealStartMs) / So);
        for (const E of T.graphics) E.alpha = R;
      }
      for (; y < p.length; ) {
        const C = p[y];
        if (w - C.revealStartMs < So) break;
        for (const T of C.graphics) T.alpha = 1;
        y++;
      }
      y >= p.length && (x = true, i.ticker.remove(_), Qn());
    };
    return x || (i.ticker.add(_), kr()), {
      controller: o,
      handle: {
        cancel() {
          b || x || (b = true, i.ticker.remove(_), Qn(), Ke());
        },
        finish() {
          if (!(b || x)) {
            for (const w of p) for (const C of w.graphics) C.alpha = 1;
            x = true, i.ticker.remove(_), Qn(), Ke();
          }
        },
        isDone() {
          return x || b;
        }
      }
    };
  }
  const Yv = 4243680;
  function Vv(n, t, e, s = 240) {
    const i = new pt();
    t.addChild(i);
    const r = e.x * S, o = e.y * S, a = e.w * S, l = e.h * S;
    let c = 0;
    const h = () => {
      c += n.ticker.deltaMS;
      const d = Math.max(0, (s - c) / s);
      if (d <= 0) {
        n.ticker.remove(h), i.destroy(), Qn();
        return;
      }
      i.clear(), i.rect(r, o, a, l).fill({
        color: Yv,
        alpha: 0.55 * d
      });
    };
    n.ticker.add(h), kr();
  }
  const Ki = 150, dh = 80, Xv = 4, qv = 300, kn = 4, Mn = 900, Kv = 6, Jv = 250, Ks = 800, Ji = 250;
  function tn(n, t, e) {
    return n <= 1 ? t : Math.min(t, e / n);
  }
  function uh(n, t) {
    return `${n},${t}`;
  }
  function Zv(n) {
    return n.split(":")[1] ?? "";
  }
  function Qv(n, t) {
    return n > 0 ? "East" : n < 0 ? "West" : t > 0 ? "South" : "North";
  }
  function t1(n, t, e, s) {
    const i = Eu();
    i.attachTo(n);
    const r = new pt();
    n.addChild(r);
    const o = /* @__PURE__ */ new Map(), a = /* @__PURE__ */ new Set(), l = /* @__PURE__ */ new Map(), c = Ar();
    let h = false, d = false, u = false, p = 0;
    const f = () => u ? p : performance.now();
    let g = null, m = 0;
    const y = /* @__PURE__ */ new Map();
    let b = null, x = null;
    const _ = [], v = globalThis.__TRACE_LOGS === true, w = (I, N) => {
      globalThis.__ANIM_LOGS && console.log(`[anim t=${f().toFixed(0)}ms] ${I}`, N);
    };
    function C(I, N) {
      const B = !y.get(I), q = {
        id: I,
        virtualMs: N
      };
      y.set(I, q), B && (b == null ? void 0 : b(q));
    }
    function T(I, N) {
      g === null && (g = I);
      const O = I + (N ? Ki : dh);
      O > m && (m = O);
    }
    function R(I, N) {
      if (I.length === 0) return;
      const O = f();
      for (const q of I) dr(q, c);
      const B = [
        ...I
      ].sort((q, Q) => {
        const ut = (q.y ?? 0) - (Q.y ?? 0);
        return ut !== 0 ? ut : (q.x ?? 0) - (Q.x ?? 0);
      });
      B.forEach((q, Q) => {
        const ut = O + Q * N, ft = xs(q);
        if (a.has(ft)) return;
        a.add(ft), ia(i, q, ut, c), l.has(ft) || l.set(ft, ut), Bc(i, q.x ?? 0, q.y ?? 0);
        const mt = o.get(uh(q.x ?? 0, q.y ?? 0));
        mt && mt.fadeOutStartMs === null && (mt.fadeOutStartMs = ut, T(ut, false));
      }), T(O + (B.length - 1) * N, true);
    }
    function E(I, N, O, B, q, Q) {
      const ut = uh(I, N), ft = o.get(ut);
      ft && ft.specKey === q || (gb(i, I, N, O, B, Q, q), ft || o.set(ut, {
        specKey: q,
        fadeStartMs: Q,
        fadeOutStartMs: null
      }), T(Q, true));
    }
    function A(I) {
      const N = tn(I.length, kn, Mn);
      w("rows_placed", {
        count: I.length,
        stagger_ms: N,
        span_ms: I.length * N
      }), R(I, N);
    }
    function W(I) {
      const N = f(), O = Zv(I.spec_key), B = I.tiles;
      if (B.length === 0) return;
      const q = tn(B.length, Xv, qv);
      w("ghost_routed", {
        spec_key: I.spec_key,
        item: O,
        tile_count: B.length,
        span_ms: B.length * q
      });
      for (let Q = 0; Q < B.length; Q++) {
        const [ut, ft] = B[Q];
        let mt = 0, wt = 0;
        Q < B.length - 1 ? (mt = B[Q + 1][0] - ut, wt = B[Q + 1][1] - ft) : Q > 0 && (mt = ut - B[Q - 1][0], wt = ft - B[Q - 1][1]), E(ut, ft, Qv(mt, wt), O, I.spec_key, N + Q * q);
      }
    }
    function j(I) {
      const N = I.entities.length, O = tn(N, kn, Mn);
      w("committed", {
        source: "spec",
        count: N,
        span_ms: N * O
      }), R(I.entities, O);
    }
    function G(I) {
      const N = f(), O = I.zone_x, B = I.zone_y, q = I.zone_x + I.zone_w - 1, Q = I.zone_y + I.zone_h - 1;
      for (const [mt, wt] of o.entries()) {
        const [V, st] = mt.split(",").map(Number);
        V < O || V > q || st < B || st > Q || wt.fadeOutStartMs === null && (wt.fadeOutStartMs = N, T(N, false), Bc(i, V, st));
      }
      for (const mt of _) mt.clusterId === I.cluster_id && (mt.cleared = true);
      for (let mt = I.zone_y; mt <= Q; mt++) for (let wt = I.zone_x; wt <= q; wt++) {
        const V = xb(i, wt, mt);
        for (const st of V) a.delete(st);
      }
      const ut = I.entities.length, ft = tn(ut, Kv, Jv);
      w("junction", {
        cluster_id: I.cluster_id,
        zone: `${I.zone_x},${I.zone_y}+${I.zone_w}x${I.zone_h}`,
        count: ut,
        span_ms: ut * ft
      }), R(I.entities, ft);
    }
    function z(I) {
      if (d = true, I.phase === "rows_placed") {
        A(I.entities);
        return;
      }
      if (I.phase !== "lanes_planned") {
        if (I.phase === "bus_routed") {
          const N = I.entities.filter((O) => !a.has(xs(O)));
          if (N.length > 0) {
            const O = tn(N.length, kn, Mn);
            R(N, O);
          }
          return;
        }
        if (I.phase === "poles_placed") {
          const N = I.entities.filter((O) => !a.has(xs(O)));
          if (N.length > 0) {
            const O = tn(N.length, kn, Mn);
            R(N, O);
          }
          return;
        }
      }
    }
    function P(I) {
      const N = f();
      w("cluster_outline", {
        cluster_id: I.cluster_id,
        zone: `${I.zone_x},${I.zone_y}+${I.zone_w}x${I.zone_h}`,
        lifetime_ms: Ks,
        fade_ms: Ji
      }), _.push({
        clusterId: I.cluster_id,
        x: I.zone_x,
        y: I.zone_y,
        w: I.zone_w,
        h: I.zone_h,
        startMs: N,
        cleared: false
      }), T(N, true);
    }
    const D = () => {
      if (h) return;
      const I = f();
      Au(i, I);
      for (const [N, O] of o.entries()) if (O.fadeOutStartMs !== null && I >= O.fadeOutStartMs) {
        const [B, q] = N.split(",").map(Number);
        (I - O.fadeOutStartMs) / dh >= 1 && o.delete(N);
      }
      for (const N of i.ghostContainer.particleChildren) N.alpha < 0.5 && (N.alpha = Math.min(0.5, N.alpha + 16 / Ki));
      r.clear();
      for (let N = _.length - 1; N >= 0; N--) {
        const O = _[N], B = I - O.startMs;
        if (!(B < 0)) if (O.cleared || B >= Ks) {
          const q = O.cleared ? Math.max(B, Ks - Ji) : B;
          if (q >= Ks) {
            _.splice(N, 1);
            continue;
          }
          const Q = Math.max(0, 1 - (q - (Ks - Ji)) / Ji);
          r.rect(O.x * S, O.y * S, O.w * S, O.h * S), r.stroke({
            width: 2,
            color: 4508927,
            alpha: 0.9 * Q
          });
        } else r.rect(O.x * S, O.y * S, O.w * S, O.h * S), r.stroke({
          width: 2,
          color: 4508927,
          alpha: 0.9
        });
      }
    };
    t.ticker.add(D), kr();
    let J = true;
    w("streaming_start", {});
    function Y(I, N) {
      if (h || u) return;
      b = N ?? null, v && console.log(`[stream t=${f().toFixed(0)}] ${I.phase}`, "data" in I ? I.data : void 0);
      const O = f();
      switch (g === null && (g = O), I.phase) {
        case "PhaseSnapshot": {
          const B = I.data;
          B.phase === "rows_placed" && C("machines", O), B.phase === "poles_placed" && C("poles", O), z(B);
          break;
        }
        case "GhostSpecRouted":
          C("ghost_routes", O), W(I.data);
          break;
        case "GhostSpecCommitted":
          C("committed_routes", O), j(I.data);
          break;
        case "JunctionCommitted":
          C("junctions", O), G(I.data);
          break;
        case "GhostClusterSolved":
          P(I.data);
          break;
        case "TrunkBeltCommitted": {
          const B = I.data, q = B.entities.length, Q = tn(q, kn, Mn);
          w("committed", {
            source: "trunk",
            count: q,
            span_ms: q * Q
          }), R(B.entities, Q);
          break;
        }
        case "BalancerCommitted": {
          const B = I.data, q = B.entities.length, Q = tn(q, kn, Mn);
          w("committed", {
            source: "balancer",
            count: q,
            span_ms: q * Q
          }), R(B.entities, Q);
          break;
        }
        case "OutputMergerCommitted": {
          const B = I.data, q = B.entities.length, Q = tn(q, kn, Mn);
          w("committed", {
            source: "merger",
            count: q,
            span_ms: q * Q
          }), R(B.entities, Q);
          break;
        }
        case "PolesCommitted": {
          const B = I.data, q = B.entities.length, Q = tn(q, kn, Mn);
          w("committed", {
            source: "poles",
            count: q,
            span_ms: q * Q
          }), R(B.entities, Q);
          break;
        }
        default: {
          I.phase === "LayoutRetried" && (i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), Ke());
          break;
        }
      }
      b = null;
    }
    return {
      onEvent: Y,
      hasCommittedEntities: () => d,
      cancel() {
        h || (h = true, t.ticker.remove(D), J && (Qn(), J = false), i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), x = null, Ke());
      },
      finish(I) {
        t.ticker.remove(D), J && (Qn(), J = false), yb(i), w("streaming_finish", {
          entity_count: i.count(),
          latest_fade_end_ms: m
        });
        const N = g ?? 0, O = [];
        for (const { particle: Q, iconParticle: ut, badgeParticle: ft, revealAt: mt } of Nc()) O.push({
          kind: "particle",
          particle: Q,
          iconParticle: ut,
          badgeParticle: ft,
          revealAt: mt
        });
        const B = I.entities.filter((Q) => !a.has(xs(Q)));
        if (B.length > 0) {
          for (const ut of B) dr(ut, c);
          for (const ut of B) ia(i, ut, N, c), a.add(xs(ut));
          const Q = new Set(O.map((ut) => ut.particle));
          for (const { particle: ut, iconParticle: ft, badgeParticle: mt, revealAt: wt } of Nc()) Q.has(ut) || O.push({
            kind: "particle",
            particle: ut,
            iconParticle: ft,
            badgeParticle: mt,
            revealAt: wt
          });
        }
        const q = bb(i, c);
        if (q.size > 0) for (const Q of O) {
          const ut = q.get(Q.particle);
          ut && (Q.particle = ut);
        }
        return x = O, u = true, p = m, K(p), vb(I, t);
      },
      seekTo(I) {
        if (h || x === null) return;
        const N = g ?? 0, O = Math.max(m, N);
        p = Math.min(O, Math.max(N, I)), w("scrub", {
          virtualMs: p
        }), K(p);
      },
      getTimeRange() {
        return {
          firstMs: g ?? 0,
          lastMs: m
        };
      },
      getMilestones() {
        return Array.from(y.values()).sort((I, N) => I.virtualMs - N.virtualMs);
      }
    };
    function K(I) {
      if (x !== null) {
        for (const N of x) {
          const O = I - N.revealAt, B = O <= 0 ? 0 : O >= Ki ? 1 : O / Ki;
          N.particle.alpha = B, N.iconParticle && (N.iconParticle.alpha = B), N.badgeParticle && (N.badgeParticle.alpha = B);
        }
        Ke();
      }
    }
  }
  const e1 = 0.03, n1 = 200, Eo = [
    "machines",
    "ghost_routes",
    "committed_routes",
    "junctions",
    "poles",
    "optimizing"
  ], s1 = {
    machines: "Machines",
    ghost_routes: "Belt routes",
    committed_routes: "Belts placed",
    junctions: "Crossings",
    poles: "Power poles",
    optimizing: "Optimizing"
  };
  function i1(n, t) {
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
    for (const G of Eo) {
      const z = document.createElement("div");
      z.className = "ts-chip", z.dataset.milestone = G, z.textContent = s1[G], G === "optimizing" && (z.style.display = "none"), s.appendChild(z), l.set(G, z);
    }
    let c = false, h = null, d = [];
    const u = /* @__PURE__ */ new Set();
    let p = null;
    function f(G) {
      o.style.width = `${G * 100}%`, a.style.left = `${G * 100}%`;
    }
    function g(G) {
      var _a2, _b2;
      p !== G && (p && ((_a2 = l.get(p)) == null ? void 0 : _a2.classList.remove("ts-chip--active")), G && ((_b2 = l.get(G)) == null ? void 0 : _b2.classList.add("ts-chip--active")), p = G);
    }
    function m(G, z) {
      var _a2;
      if (c) return;
      const P = G.id;
      u.add(P), (_a2 = l.get(P)) == null ? void 0 : _a2.classList.add("ts-chip--reached"), g(P);
      const D = Math.max(1, z.lastMs - z.firstMs), J = (G.virtualMs - z.firstMs) / D;
      f(Math.min(1, Math.max(0, J))), e.classList.add("ts-visible");
    }
    function y(G) {
      return h ? h.firstMs + G * (h.lastMs - h.firstMs) : 0;
    }
    function b(G) {
      for (const z of d) if (Math.abs(G - z.frac) < e1) return {
        frac: z.frac,
        snapped: true
      };
      return {
        frac: G,
        snapped: false
      };
    }
    function x(G) {
      if (!h) return;
      const z = i.getBoundingClientRect(), P = (G - z.left) / z.width, D = Math.min(1, Math.max(0, P)), { frac: J, snapped: Y } = b(D);
      f(J), Y ? a.classList.add("ts-thumb--snapped") : a.classList.remove("ts-thumb--snapped"), t(y(J));
    }
    let _ = null, v = null;
    function w(G) {
      if (!c || !h) return;
      G.preventDefault();
      try {
        i.setPointerCapture(G.pointerId);
      } catch {
      }
      const z = (D) => x(D.clientX), P = (D) => {
        _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), _ = null, v = null, a.classList.remove("ts-thumb--snapped");
      };
      _ = z, v = P, document.addEventListener("pointermove", z), document.addEventListener("pointerup", P, {
        once: true
      }), x(G.clientX);
    }
    i.addEventListener("pointerdown", w);
    function C(G, z) {
      const P = G.lastMs - G.firstMs;
      if (P < n1 || z.length === 0) {
        A();
        return;
      }
      c = true, h = G, e.classList.add("ts-scrub-mode"), e.classList.add("ts-visible"), d = z.map((D) => ({
        id: D.id,
        frac: (D.virtualMs - G.firstMs) / P
      })), s.style.justifyContent = "flex-start", s.style.position = "relative";
      for (const D of l.values()) D.style.position = "absolute", D.style.transform = "translateX(-50%)";
      for (const D of Eo) {
        const J = l.get(D);
        if (!J) continue;
        const Y = d.find((K) => K.id === D);
        Y ? (J.style.left = `${Y.frac * 100}%`, J.style.display = "", J.classList.add("ts-chip--reached")) : J.style.display = "none";
      }
      R(), requestAnimationFrame(E), f(1), g(null);
    }
    let T = null;
    function R() {
      if (T && T.remove(), !h) return;
      const G = document.createElement("div");
      G.className = "ts-ticks";
      for (const z of d) {
        const P = document.createElement("div");
        P.className = "ts-tick", P.style.left = `${z.frac * 100}%`, G.appendChild(P);
      }
      i.appendChild(G), T = G;
    }
    function E() {
      if (!c) return;
      const G = 6, z = s.clientWidth;
      if (z <= 0) return;
      const P = Eo.map((J) => {
        var _a2;
        const Y = l.get(J);
        if (!Y || Y.style.display === "none") return null;
        const K = J === "optimizing" ? 1 : ((_a2 = d.find((I) => I.id === J)) == null ? void 0 : _a2.frac) ?? 0;
        return {
          el: Y,
          originalFrac: K
        };
      }).filter((J) => J !== null);
      let D = -1 / 0;
      for (const { el: J, originalFrac: Y } of P) {
        const K = J.offsetWidth / 2, I = Y * z, N = D + K + G, O = Math.max(I, N);
        J.style.left = `${O / z * 100}%`, D = O + K;
      }
    }
    function A() {
      c = false, h = null, d = [], u.clear(), p = null, T && (T.remove(), T = null), e.classList.remove("ts-visible", "ts-scrub-mode"), o.style.width = "0", a.style.left = "0", a.classList.remove("ts-thumb--snapped"), s.style.justifyContent = "space-between", s.style.position = "";
      for (const [G, z] of l) z.style.position = "", z.style.transform = "", z.style.left = "", z.style.display = G === "optimizing" ? "none" : "", z.classList.remove("ts-chip--reached", "ts-chip--active", "ts-chip--in-progress");
    }
    function W(G) {
      const z = l.get("optimizing");
      if (z) {
        if (z.classList.remove("ts-chip--in-progress", "ts-chip--reached"), G === "idle") {
          z.style.display = "none";
          return;
        }
        z.style.display = "", e.classList.add("ts-visible"), c && (z.style.position = "absolute", z.style.transform = "translateX(-50%)", z.style.left = "100%", requestAnimationFrame(E)), G === "active" ? z.classList.add("ts-chip--in-progress") : G === "done" && z.classList.add("ts-chip--reached");
      }
    }
    function j() {
      _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), i.removeEventListener("pointerdown", w), e.remove();
    }
    return {
      noteMilestone: m,
      arm: C,
      markOptimizeState: W,
      reset: A,
      destroy: j
    };
  }
  const r1 = `
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
  function o1() {
    if (document.getElementById("spaghettio-busy-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-busy-style", n.textContent = r1, document.head.appendChild(n);
  }
  const a1 = 120;
  function l1(n) {
    o1();
    const t = document.createElement("div");
    t.className = "spaghettio-busy";
    const e = document.createElement("span");
    e.className = "spaghettio-busy-spin", t.appendChild(e);
    const s = document.createElement("span");
    s.textContent = "computing\u2026", t.appendChild(s), n.appendChild(t);
    let i = null;
    m_((r) => {
      r > 0 ? i === null && !t.classList.contains("visible") && (i = setTimeout(() => {
        t.classList.add("visible"), i = null;
      }, a1)) : (i !== null && (clearTimeout(i), i = null), t.classList.remove("visible"));
    });
  }
  function en(n, t) {
    return n.filter((e) => e.phase === t);
  }
  const Un = "color:#9cdcfe;font-weight:bold", me = "color:#888", Te = "color:#e0e0e0", ys = "color:#6a6", Zi = "color:#ffaa00", To = "color:#f66", Qi = "color:#c586c0";
  function c1(n) {
    var _a2;
    const t = Array.isArray(n.trace) ? n.trace : [];
    if (t.length === 0) return;
    const e = en(t, "PhaseTime"), s = en(t, "SatInvocation"), i = en(t, "JunctionSolved"), r = en(t, "JunctionGrowthCapped"), o = en(t, "GhostClusterSolved"), a = en(t, "GhostRoutingComplete"), l = en(t, "RegionWalkerVeto"), c = en(t, "JunctionGrowthIteration"), h = en(t, "NegotiateComplete"), d = en(t, "ValidationCompleted"), u = t.filter((x) => x.phase === "LayoutRetried"), p = e.reduce((x, _) => x + _.data.duration_ms, 0), f = s.reduce((x, _) => x + _.data.solve_time_us, 0), g = s.filter((x) => x.data.satisfied).length, m = ((_a2 = n.entities) == null ? void 0 : _a2.length) ?? 0, y = u.length > 0 ? r.length > 0 ? ` \xB7 ${r.length} capped \xB7 retried (${u[0].data.caps_before} caps recovered)` : ` \xB7 retried (${u[0].data.caps_before} caps recovered)` : r.length > 0 ? ` \xB7 ${r.length} capped` : "", b = r.length > 0 ? Zi : u.length > 0 ? Qi : ys;
    if (console.log(`%c\u25B6 layout %c${n.width}\xD7${n.height}  %c${m} entities  %c${p}ms  %cSAT ${Math.round(f / 1e3)}ms (${s.length}\xD7)%c${y}`, Un, Te, me, Te, Qi, b), console.groupCollapsed("%c  \u21B3 breakdown", me), e.length > 0) {
      const x = [
        ...e
      ].sort((_, v) => v.data.duration_ms - _.data.duration_ms);
      console.log(`%cphases%c ${p}ms total`, Un, me);
      for (const _ of x) {
        const v = _.data, w = p > 0 ? v.duration_ms / p * 100 : 0, C = Math.max(1, Math.round(w / 100 * 24)), T = "\u2588".repeat(C);
        console.log(`  %c${v.phase.padEnd(18)}%c ${String(v.duration_ms).padStart(5)}ms  %c${T}%c ${w.toFixed(1)}%`, me, Te, Qi, me);
      }
    }
    if (s.length > 0) {
      const x = f / 1e3, _ = p > 0 ? x / p * 100 : 0, v = f / s.length, w = [
        ...s
      ].sort((T, R) => R.data.solve_time_us - T.data.solve_time_us)[0], C = [
        ...s
      ].sort((T, R) => R.data.zone_w * R.data.zone_h - T.data.zone_w * T.data.zone_h)[0];
      console.log(`%cSAT%c ${s.length} invocations \xB7 ${x.toFixed(1)}ms (%c${_.toFixed(1)}%%%c of total)`, Un, Te, Qi, Te), console.log(`  %csatisfied%c ${g}  %cunsat%c ${s.length - g}  %cavg%c ${(v / 1e3).toFixed(2)}ms`, me, ys, me, To, me, Te), w && console.log(`  %cslowest call%c ${(w.data.solve_time_us / 1e3).toFixed(1)}ms \u2014 %c${w.data.zone_w}\xD7${w.data.zone_h} @ (${w.data.zone_x},${w.data.zone_y}), ${w.data.variables} vars, ${w.data.clauses} clauses`, me, Te, me), C && C !== w && console.log(`  %cbiggest zone%c ${C.data.zone_w}\xD7${C.data.zone_h} @ (${C.data.zone_x},${C.data.zone_y}) \u2014 ${C.data.variables} vars`, me, Te);
    }
    if (o.length > 0 || i.length > 0 || r.length > 0) {
      if (console.log("%cjunctions", Un), console.log(`  %cclusters%c ${o.length}  %csolved%c ${i.length}  %ccapped%c ${r.length}  %cvetoes%c ${l.length}`, me, Te, me, ys, me, r.length > 0 ? Zi : Te, me, Te), c.length > 0) {
        const x = /* @__PURE__ */ new Map();
        for (const v of c) {
          const w = `${v.data.seed_x},${v.data.seed_y}`;
          x.set(w, Math.max(x.get(w) ?? 0, v.data.iter));
        }
        const _ = [
          ...x.entries()
        ].sort((v, w) => w[1] - v[1])[0];
        _ && _[1] > 0 && console.log(`  %chardest%c junction at (${_[0]}) needed ${_[1] + 1} growth iters`, me, Te);
      }
      if (r.length > 0) for (const x of r) console.log(`    %c\u26A0 capped at (${x.data.tile_x},${x.data.tile_y})%c \u2014 ${x.data.reason}, ${x.data.region_tiles} tiles after ${x.data.iters} iters`, Zi, me);
    }
    if (a.length > 0) {
      const x = a[0].data, _ = x.unroutable_count > 0 ? To : ys;
      console.log(`%cghost router%c ${x.entity_count} routed entities, ${x.cluster_count} clusters, max cluster ${x.max_cluster_tiles} tiles  %c${x.unroutable_count} unroutable`, Un, Te, _);
    }
    if (h.length > 0) {
      const x = h[0].data;
      console.log(`%cA* negotiate%c ${x.specs} specs, ${x.iterations} iters, ${x.duration_ms}ms`, Un, Te);
    }
    if (d.length > 0) {
      const x = d[0].data, _ = x.error_count > 0 ? To : ys, v = x.warning_count > 0 ? Zi : ys;
      console.log(`%cvalidation  %c${x.error_count} errors  %c${x.warning_count} warnings`, Un, _, v);
    }
    console.groupEnd();
  }
  const h1 = [
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
  async function d1() {
    await Wu();
    const n = zu();
    await B0(h1), await G0();
    const t = document.getElementById("app"), e = window.location.hash, s = new URLSearchParams(window.location.search);
    if (e.startsWith("#/balancers")) {
      const { renderBalancerShowcase: r } = await Ln(async () => {
        const { renderBalancerShowcase: o } = await import("./balancers-BUyXLZ4L.js");
        return {
          renderBalancerShowcase: o
        };
      }, []);
      r(t, n);
      return;
    }
    if (!(e.startsWith("#/layout") || s.has("generator") || s_())) {
      const r = document.createElement("div");
      t.appendChild(r), B_(r, n, {
        onOpenGenerator: () => {
          r.remove(), ph(n), window.history.replaceState({}, "", "#/layout");
        }
      });
      return;
    }
    ph(n);
  }
  async function ph(n) {
    const t = document.getElementById("canvas-container");
    if (!t) throw new Error("Missing #canvas-container element");
    const e = document.getElementById("app");
    e.style.display = "flex";
    const s = document.getElementById("sidebar");
    s && (s.style.display = ""), t.style.display = "";
    const { app: i, viewport: r, requestRender: o, beginAnimating: a, endAnimating: l } = await J0(t), c = Z0(r);
    let h = false;
    Xs(r, null), Pv();
    const d = Rv(t), { debugCb: u, colorCb: p, heatmapCb: f, powerWiresCb: g, moduleSlotsCb: m, regionsCb: y, soloRegionsCb: b, ghostTilesCb: x, traceOverlayCb: _, simReportFileInput: v, simStateCb: w } = d, C = Lv(t), T = kw(t, () => {
      w.disabled = !T.getReport(), ot();
    });
    Di(p.checked);
    const R = () => {
      globalThis.__ANIM_LOGS = u.checked;
    };
    u.addEventListener("change", R), R();
    const E = Ov(t), A = ov();
    let W = null;
    const j = mv(t, r, {
      onChange: (F) => {
        if (A.update(F), F) {
          P.alpha = G.isActive() ? 0.2 : 0.35;
          const at = F.iter.bbox;
          W = {
            bboxX: at.x,
            bboxY: at.y,
            bboxW: at.w,
            bboxH: at.h
          };
        } else P.alpha = 1, W = null, G.isActive() && G.exit();
        o();
      },
      onEditRequested: (F) => {
        P.alpha = 0.2, G.enter(F), o();
      }
    }), G = Mv({
      viewport: r,
      canvas: i.canvas,
      engine: n,
      jd: j,
      satZoneOverlayLayer: A.layer
    });
    sw(t, (F) => le.load(F));
    function z(F) {
      P.removeChildren();
      const at = Eu();
      return at.attachTo(P), _b(F, at, i);
    }
    const P = new Ht();
    P.isRenderGroup = true, P.eventMode = "none", r.addChild(P);
    const D = new Ht();
    D.eventMode = "none", r.addChild(D), r.addChild(A.layer);
    const J = new pt();
    J.label = "pin-highlight", r.addChild(J), E.onPinChange((F) => {
      if (J.clear(), F) {
        const at = F.x * S, dt = F.y * S;
        J.setStrokeStyle({
          width: 2,
          color: 8440063,
          alpha: 0.95
        }), J.rect(at - 2, dt - 2, S + 4, S + 4).stroke();
      }
      o();
    }), r.moveCenter(an / 2, an / 2);
    const Y = (F) => {
    };
    let K = null;
    function I(F) {
      K = F, E.onHover(F, F == null ? void 0 : F.x, F == null ? void 0 : F.y);
    }
    let N = /* @__PURE__ */ new Map();
    function O(F) {
      const at = /* @__PURE__ */ new Map();
      for (const dt of F.entities) {
        const bt = dt.x ?? 0, ht = dt.y ?? 0, Ft = Se[dt.name];
        if (Ft) {
          const [Dt, kt] = Ft;
          for (let It = 0; It < kt; It++) for (let ve = 0; ve < Dt; ve++) at.set(`${bt + ve},${ht + It}`, dt);
        } else if (ae.has(dt.name)) {
          at.set(`${bt},${ht}`, dt);
          const [Dt, kt] = Ra(dt.direction);
          at.set(`${bt + Dt},${ht + kt}`, dt);
        } else at.set(`${bt},${ht}`, dt);
      }
      N = at;
    }
    function B(F) {
      return {
        highlightItem: (at) => {
          F.highlightItem(at), o();
        },
        highlightBeltNetwork: (at) => {
          F.highlightBeltNetwork(at), o();
        },
        clearHighlight: () => {
          F.clearHighlight(), o();
        },
        chainKey: F.chainKey
      };
    }
    let q = false, Q = null, ut = null, ft = false, mt = null;
    const wt = {
      update() {
      },
      getPhaseIndex() {
        return -1;
      },
      reset() {
      }
    };
    function V() {
      var _a2;
      ut && (P.removeChild(ut), ut.destroy(), ut = null);
      const F = wt.getPhaseIndex();
      if (u.checked && F >= 0, ft && (mt == null ? void 0 : mt.cancel(), mt = null, ft = false, nt)) {
        const dt = li(nt, P, I, Y);
        E.setHighlightController(B(dt)), o();
      }
      if (!u.checked || !_.checked || !((_a2 = nt == null ? void 0 : nt.trace) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      const at = nt.trace;
      ut = rw(at, nt.width ?? 0, nt.height ?? 0, P, (dt) => {
        E.setTooltipOverride(dt ? `<span style="color:#8af">TRACE</span> ${dt}` : null);
      }), o();
    }
    let st = null, it = null, lt = null, Et = [], Nt = null, xt = null, jt = null, Vt = null, te = null;
    const Lt = document.createElement("div");
    Lt.className = "validation-badge", Lt.style.display = "none", t.appendChild(Lt);
    function be(F) {
      if (!F || F.length === 0) {
        Lt.style.display = "none";
        return;
      }
      const at = F.filter((ht) => ht.severity === "Error").length, dt = F.length - at;
      let bt;
      at > 0 && dt > 0 ? bt = `\u26A0 ${at} error${at > 1 ? "s" : ""}, ${dt} warning${dt > 1 ? "s" : ""}` : at > 0 ? bt = `\u26A0 ${at} error${at > 1 ? "s" : ""}` : bt = `\u26A0 ${dt} warning${dt > 1 ? "s" : ""}`, Lt.textContent = bt, Lt.classList.toggle("has-errors", at > 0), Lt.style.display = "block";
    }
    let Xt = null, _e = null, $t = null, fe = null, H = null;
    const et = 1;
    function tt(F, at) {
      const dt = F * S + S / 2, bt = at * S + S / 2;
      r.scale.x < et && r.setZoom(et, false), r.moveCenter(dt, bt);
    }
    function St(F) {
      var _a2, _b2;
      const at = [];
      for (const dt of F.regions ?? []) {
        if (dt.kind !== "unresolved") continue;
        const bt = dt.x + Math.floor(dt.width / 2), ht = dt.y + Math.floor(dt.height / 2), Ft = ((_b2 = (_a2 = dt.ports) == null ? void 0 : _a2.find((Dt) => Dt.item)) == null ? void 0 : _b2.item) ?? "unknown";
        at.push({
          severity: "Warning",
          category: `ghost-router \xB7 ${Ft}`,
          message: `unresolved crossing at (${bt}, ${ht})`,
          x: bt,
          y: ht
        });
      }
      for (const dt of F.warnings ?? []) /^ghost router:.*unresolved crossings/i.test(dt) || at.push({
        severity: "Warning",
        category: "layout",
        message: dt,
        x: void 0,
        y: void 0
      });
      return at;
    }
    function zt() {
      if (st && (P.removeChild(st), st.destroy(), st = null), nt && !it && te !== nt) {
        const ht = nt;
        te = ht, n.validateLayout(ht, Tt).then((Ft) => {
          nt === ht && (it = Ft, te = null, zt(), Pr(ht));
        }).catch(() => {
          nt === ht && (it = [], te = null, zt(), Pr(ht));
        });
      }
      const F = nt ? St(nt) : [], at = [
        ...it ?? [],
        ...F
      ], dt = (ht) => {
        if (ht.x == null || ht.y == null || !Vt) return null;
        const Ft = Vt.lookup(ht.x, ht.y).cappedSides;
        return Ft.length === 0 ? null : [
          ...new Set(Ft.map((kt) => kt.limit))
        ].sort().join("+");
      };
      if (Fs == null ? void 0 : Fs.updateValidation(at, tt, dt), be(at), k(at), !nt || at.length === 0) {
        o();
        return;
      }
      st = cw(at, P, (ht) => {
        E.setTooltipOverride(ht ? `<span style="color:#f44">VALIDATION</span> ${ht}` : null);
      }).layer, o();
    }
    function k(F) {
      if (F && (Et = F), lt && (P.removeChild(lt), lt.destroy({
        children: true
      }), lt = null), !f.checked || !nt || Et.length === 0) {
        o();
        return;
      }
      lt = dw(Et, nt.entities, P), o();
    }
    function M() {
      var _a2;
      if (Nt && (P.removeChild(Nt), Nt.destroy({
        children: true
      }), Nt = null), !g.checked || !((_a2 = nt == null ? void 0 : nt.power_wires) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      Nt = uw(nt.power_wires, nt.entities, P), o();
    }
    function X() {
      var _a2;
      if (xt && (P.removeChild(xt), xt.destroy({
        children: true
      }), xt = null), !m.checked || !((_a2 = nt == null ? void 0 : nt.entities) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      xt = mw(nt.entities, zu().moduleSlots, P), o();
    }
    function ot() {
      jt && (P.removeChild(jt), jt.destroy({
        children: true
      }), jt = null);
      const F = T.getReport();
      if (!w.checked || !F || !nt) {
        o();
        return;
      }
      jt = Tw(F.sim_state, P), o();
    }
    function $() {
      if (H && (r.removeChild(H), H.destroy({
        children: true
      }), H = null), !u.checked || !x.checked || !nt) {
        o();
        return;
      }
      const F = fv(nt.trace);
      if (!F) {
        o();
        return;
      }
      H = F, r.addChildAt(H, 0), o();
    }
    function L() {
      var _a2;
      if (Xt && (P.removeChild(Xt), Xt.destroy(), Xt = null), $t && (P.removeChild($t), $t.destroy(), $t = null), _e = null, fe = null, !u.checked || !(y == null ? void 0 : y.checked) || !nt) {
        o();
        return;
      }
      if (nt.regions && nt.regions.length > 0) {
        const F = Fw(nt);
        Xt = F.layer, _e = F.hitTest, P.addChild(Xt);
      }
      if ((_a2 = nt.trace) == null ? void 0 : _a2.length) {
        const F = Hw(nt.trace);
        if (F.length > 0) {
          const at = qw(F);
          $t = at.layer, fe = at.hitTest, P.addChild($t);
        }
      }
      o();
    }
    const Z = document.createElement("div");
    Z.style.cssText = "position:absolute;bottom:34px;left:8px;background:rgba(0,0,0,0.8);color:#e0e0e0;font:11px monospace;padding:6px 8px;border-radius:3px;border:1px solid #00e0a0;z-index:10;display:none;min-width:200px", t.appendChild(Z);
    const U = document.createElement("div");
    U.style.cssText = "color:#00e0a0;margin-bottom:4px", Z.appendChild(U);
    const rt = document.createElement("textarea");
    rt.placeholder = "Add a note\u2026", rt.rows = 2, rt.style.cssText = "width:100%;box-sizing:border-box;background:#2a2a2a;color:#e0e0e0;border:1px solid #555;border-radius:2px;font:11px monospace;resize:vertical;margin-bottom:4px", Z.appendChild(rt);
    const gt = document.createElement("div");
    gt.style.cssText = "color:#777", gt.textContent = "Ctrl+C to copy JSON", Z.appendChild(gt);
    let Mt = false, nt = null, Tt = null, vt = null, Wt = null, qt = null;
    const re = i1(t, (F) => qt == null ? void 0 : qt.seekTo(F));
    l1(t);
    const le = Wv({
      sidebarEl: document.getElementById("sidebar"),
      getSidebarCtrl: () => Fs,
      renderLayoutOnCanvas: as,
      setCachedValidationIssues: (F) => {
        it = F;
      },
      updateValidationOverlay: zt,
      panToTile: tt,
      onDebugEnable: () => d.setDebugEnabled(true),
      onClear: () => {
        le.clear(), P.removeChildren(), D.removeChildren(), E.clearPin(), E.setTileContext(null), nt = null, Tt = null, C.update(null), N = /* @__PURE__ */ new Map(), it = null, Xs(r, null), r.moveCenter(an / 2, an / 2), be(null), Fs == null ? void 0 : Fs.updateValidation([], tt), j.close(), T.setLayoutInfo(null), jt = null;
      }
    });
    function Fn(F) {
      F.length === 0 ? (Z.style.display = "none", rt.value = "") : (U.textContent = `${F.length} entit${F.length === 1 ? "y" : "ies"} selected`, Z.style.display = "block");
    }
    async function rs(F) {
      if (Mt || !(F.regions ?? []).some((Gt) => Gt.kind === "crossing_zone")) return;
      Mt = true, re.markOptimizeState("active"), vt && (vt.destroy(), vt = null);
      const dt = (Gt) => Gt === "transport-belt" || Gt === "fast-transport-belt" || Gt === "express-transport-belt" || Gt === "underground-belt" || Gt === "fast-underground-belt" || Gt === "express-underground-belt", bt = [], ht = 130, Ft = 90, Dt = 520;
      let kt = 0, It = -1, ve = 0, Ee = false, Wn = false, ls = null;
      const ep = (Gt) => {
        if (!nt) return;
        const Je = Gt.zone_x, Ue = Gt.zone_y, cs = Je + Gt.zone_w, Ir = Ue + Gt.zone_h, sp = (Ws) => {
          const Ka = Ws.x ?? 0, Ja = Ws.y ?? 0;
          return Ka >= Je && Ka < cs && Ja >= Ue && Ja < Ir;
        };
        nt.entities = nt.entities.filter((Ws) => !(sp(Ws) && dt(Ws.name))).concat(Gt.entities), Vv(i, r, {
          x: Je,
          y: Ue,
          w: Gt.zone_w,
          h: Gt.zone_h
        }), z(nt), o();
      }, np = () => Ee && bt.length === 0, qa = (Gt) => {
        if (!Wn) {
          for (; bt.length > 0; ) {
            const Je = bt[0], Ue = Je.imp.region_id === It, cs = Ue ? Math.min(Dt, ve * Ft) : 0, Ir = ht + cs;
            if (Gt - kt < Ir) break;
            bt.shift(), ep(Je.imp), Ue ? ve += 1 : (ve = 1, It = Je.imp.region_id), kt = Gt;
            break;
          }
          if (np()) {
            ls = null;
            return;
          }
          ls = requestAnimationFrame(qa);
        }
      };
      ls = requestAnimationFrame(qa);
      try {
        const Gt = await n.optimizeAllRegions(F, {
          perRegionBudgetMs: 800,
          onImprovement: (Ue) => {
            Ue.iter !== 0 && bt.push({
              imp: Ue
            });
          }
        });
        Ee = true, await new Promise((Ue) => {
          const cs = () => {
            bt.length === 0 ? Ue() : requestAnimationFrame(cs);
          };
          cs();
        }), nt = Gt, C.update(Gt), O(Gt), window.__layout = Gt;
        const Je = z(Gt);
        E.setHighlightController(B(Je)), o();
      } catch (Gt) {
        (Gt instanceof Error ? Gt.message : String(Gt)).includes("superseded") || console.error("[auto-optimize] failed", Gt);
      } finally {
        Wn = true, Ee = true, ls !== null && cancelAnimationFrame(ls), Mt = false, re.markOptimizeState("done"), nt && (vt = zc(i.canvas, r, P, nt, Fn));
      }
    }
    i.canvas.addEventListener("pointermove", (F) => {
      const at = i.canvas.getBoundingClientRect(), dt = F.clientX - at.left, bt = F.clientY - at.top, ht = r.toWorld(dt, bt), Ft = Math.floor(ht.x / S), Dt = Math.floor(ht.y / S);
      E.setCursorTile(Ft, Dt);
      const kt = N.get(`${Ft},${Dt}`) ?? null;
      kt !== K && I(kt), K || E.onHover(null, Ft, Dt);
    }), i.canvas.addEventListener("pointerleave", () => {
      E.setCursorTile(null), K && I(null);
    });
    const Mr = 4;
    let os = null;
    i.canvas.addEventListener("pointerdown", (F) => {
      if (F.button !== 0 || F.shiftKey || F.altKey || F.ctrlKey || F.metaKey) {
        os = null;
        return;
      }
      os = {
        x: F.clientX,
        y: F.clientY,
        shifted: false
      };
    }), i.canvas.addEventListener("pointerup", (F) => {
      if (!os) return;
      const at = F.clientX - os.x, dt = F.clientY - os.y;
      if (os = null, Math.hypot(at, dt) > Mr || F.button !== 0 || F.shiftKey || F.altKey || F.ctrlKey || F.metaKey) return;
      const bt = i.canvas.getBoundingClientRect(), ht = r.toWorld(F.clientX - bt.left, F.clientY - bt.top), Ft = Math.floor(ht.x / S), Dt = Math.floor(ht.y / S);
      if (!y.checked) {
        const Ee = K && K.x === Ft && K.y === Dt ? K : null;
        if (!Ee) return;
        E.pinTile(Ee, Ft, Dt);
        return;
      }
      const kt = (fe == null ? void 0 : fe(ht.x, ht.y)) ?? null;
      if (kt) {
        j.open(kt, nt == null ? void 0 : nt.trace);
        return;
      }
      if (W) {
        const Ee = ht.x / S, Wn = ht.y / S;
        if (!(Ee >= W.bboxX && Wn >= W.bboxY && Ee < W.bboxX + W.bboxW && Wn < W.bboxY + W.bboxH)) {
          j.close();
          return;
        }
      }
      const It = (_e == null ? void 0 : _e(ht.x, ht.y)) ?? null;
      if (It) {
        const Ee = (It.region.x + It.region.width / 2) * S, Wn = (It.region.y + It.region.height / 2) * S;
        r.moveCenter(Ee, Wn);
      }
      const ve = E.getPinnedTile();
      if (ve && ve.x === Ft && ve.y === Dt) E.clearPin();
      else {
        const Ee = K && K.x === Ft && K.y === Dt ? K : null;
        if (!Ee) return;
        E.pinTile(Ee, Ft, Dt);
      }
    }), document.addEventListener("keydown", (F) => {
      F.key === "Shift" && r.plugins.pause("drag");
    }), document.addEventListener("keyup", (F) => {
      F.key === "Shift" && r.plugins.resume("drag");
    }), window.addEventListener("blur", () => r.plugins.resume("drag"));
    function Zu(F) {
      Wt == null ? void 0 : Wt.cancel(), Wt = null, qt == null ? void 0 : qt.cancel(), qt = null, re.reset(), P.removeChildren(), D.removeChildren(), Tt = F, Xs(r, F), F || r.moveCenter(an / 2, an / 2);
    }
    function Qu() {
      qt == null ? void 0 : qt.cancel(), re.reset(), Xs(r, null), qt = t1(P, i);
      let F = false, at = null;
      return (dt) => {
        if (dt.phase === "PhaseSnapshot") {
          const ht = dt.data;
          ht.width > 0 && ht.height > 0 && (F || (r.fit(true, ht.width * S * 1.15, ht.height * S * 1.25), r.moveCenter(ht.width * S / 2, ht.height * S / 2), F = true), at ? at.resize(ht.width + 2, ht.height + 2) : at = tp(ht.width + 2, ht.height + 2));
        }
        qt == null ? void 0 : qt.onEvent(dt, (ht) => {
          qt && re.noteMilestone(ht, qt.getTimeRange());
        });
      };
    }
    function tp(F, at) {
      let dt = F, bt = at, ht = false;
      if (h) return Hn(c, dt, bt), o(), ht = true, {
        cancel: () => {
        },
        resize(It, ve) {
          Hn(c, It, ve), o();
        }
      };
      const Ft = 250, Dt = performance.now();
      c.alpha = 1, Hn(c, dt, bt, 0);
      const kt = () => {
        if (ht) return;
        const It = Math.min(1, (performance.now() - Dt) / Ft);
        Hn(c, dt, bt, It), o(), It >= 1 && (ht = true, h = true, i.ticker.remove(kt), l());
      };
      return i.ticker.add(kt), a(), {
        cancel() {
          ht || (ht = true, h = true, i.ticker.remove(kt), l(), Hn(c, dt, bt), o());
        },
        resize(It, ve) {
          dt = It, bt = ve, ht && (Hn(c, dt, bt), o());
        }
      };
    }
    function as(F, at) {
      var _a2;
      pu(N0(F.entities)), nt = F, C.update(F), at && (Tt = at), O(F), window.__layout = F, c1(F), ft = false, mt == null ? void 0 : mt.cancel(), mt = null, vt && (vt.destroy(), vt = null), Wt == null ? void 0 : Wt.cancel(), Wt = null, Z.style.display = "none", rt.value = "", it = null, Xs(r, null);
      let dt;
      if (qt == null ? void 0 : qt.hasCommittedEntities()) dt = qt.finish(F), re.arm(qt.getTimeRange(), qt.getMilestones());
      else if (qt == null ? void 0 : qt.cancel(), qt = null, re.reset(), (Array.isArray(F.trace) ? F.trace : []).some((kt) => kt.phase === "PhaseSnapshot")) {
        const kt = jv(F, P, I, Y, i);
        dt = kt.controller, Wt = kt.handle;
      } else dt = z(F);
      E.setHighlightController(B(dt)), Vt = Fv(F.trace), E.setTileContext(Vt), E.clearPin(), vt = zc(i.canvas, r, P, F, Fn), V(), zt(), L(), $(), M(), X(), T.setLayoutInfo({
        entities: ((_a2 = F.entities) == null ? void 0 : _a2.length) ?? 0
      }), ot(), Ob(D, F, Tt);
      const bt = F.width ?? 0, ht = F.height ?? 0;
      if (Hn(c, bt + 2, ht + 2), bt > 0 && ht > 0) {
        const Ft = bt * 32, Dt = ht * 32, kt = 192;
        r.fit(true, Ft * 1.1, (Dt + kt) * 1.2), r.moveCenter(Ft / 2, (Dt - kt) / 2);
      }
      q && (P.alpha = 0.12), o(), requestAnimationFrame(() => {
        nt === F && Pr(F);
      });
    }
    function Pr(F) {
      nt !== F || te === F || it === null || [
        ...it,
        ...St(F)
      ].length > 0 || rs(F);
    }
    document.addEventListener("keydown", (F) => {
      if (F.ctrlKey) {
        if (F.key === "c") {
          if (!vt || vt.getSelected().length === 0) return;
          F.preventDefault();
          const at = (Fs == null ? void 0 : Fs.getParams()) ?? null, dt = vt.buildJson(at, rt.value.trim());
          navigator.clipboard.writeText(dt).catch(() => {
          }), gt.textContent = "Copied!", setTimeout(() => {
            gt.textContent = "Ctrl+C to copy JSON";
          }, 2e3);
        } else if (F.key === "o") {
          F.preventDefault();
          const at = document.createElement("input");
          at.type = "file", at.accept = ".fls", at.addEventListener("change", async () => {
            var _a2;
            const dt = (_a2 = at.files) == null ? void 0 : _a2[0];
            if (dt) try {
              const bt = await dt.text(), ht = await Xu(bt);
              le.load(ht);
            } catch (bt) {
              alert(`Failed to load snapshot: ${bt}`);
            }
          }), at.click();
        }
      }
    });
    const Ns = document.getElementById("sidebar");
    let Fs = null;
    if (Ns) {
      let F = function(kt) {
        const It = document.createElement("button");
        return It.textContent = kt, It.style.cssText = "flex:1;padding:10px 4px;background:none;border:none;border-bottom:2px solid transparent;color:#777;font:12px 'JetBrains Mono','Consolas',monospace;cursor:pointer;letter-spacing:0.5px;transition:all 0.15s", It;
      }, at = function(kt) {
        const It = kt === "generate";
        Ft.style.display = It ? "flex" : "none", Dt.style.display = It ? "none" : "flex", bt.style.borderBottomColor = It ? "#569cd6" : "transparent", bt.style.color = It ? "#d4d4d4" : "#777", ht.style.borderBottomColor = It ? "transparent" : "#569cd6", ht.style.color = It ? "#777" : "#d4d4d4";
      };
      const dt = document.createElement("div");
      dt.style.cssText = "display:flex;border-bottom:1px solid #2a2a2a;background:#141414;flex-shrink:0";
      const bt = F("Generate"), ht = F("Corpus");
      dt.appendChild(bt), dt.appendChild(ht);
      const Ft = document.createElement("div");
      Ft.style.cssText = "flex:1;overflow:hidden;display:flex;flex-direction:column;";
      const Dt = document.createElement("div");
      Dt.style.cssText = "flex:1;overflow:hidden;display:none;flex-direction:column;", Ns.style.cssText += ";display:flex;flex-direction:column;padding:0;overflow:hidden;", Ns.appendChild(dt), Ns.appendChild(Ft), Ns.appendChild(Dt), bt.onclick = () => at("generate"), ht.onclick = () => at("corpus"), at("generate"), Fs = h_(Ft, n, {
        renderGraph: Zu,
        renderLayout: as,
        startStreaming: Qu
      }), u.addEventListener("change", () => {
        V(), zt(), L(), $();
      }), x.addEventListener("change", () => {
        $();
      }), _.addEventListener("change", () => {
        V();
      }), p.addEventListener("change", () => {
        Di(p.checked), nt && as(nt);
      }), f.addEventListener("change", () => {
        k();
      }), g.addEventListener("change", () => {
        M();
      }), m.addEventListener("change", () => {
        X();
      }), w.addEventListener("change", () => {
        ot();
      }), v.addEventListener("change", () => {
        var _a2;
        const kt = (_a2 = v.files) == null ? void 0 : _a2[0];
        v.value = "", kt && kt.text().then((It) => T.loadFromText(It)).catch((It) => {
          alert(`Failed to read sim report: ${It}`);
        });
      }), y.addEventListener("change", () => {
        L();
      }), b.addEventListener("change", () => {
        const kt = () => o();
        b.checked ? (q = true, Q = {
          colorChecked: p.checked,
          regionsChecked: y.checked,
          entityAlpha: P.alpha
        }, y.checked || (y.checked = true, L()), p.checked && (p.checked = false, Di(false), nt && as(nt)), P.alpha = 0.12, L(), kt()) : (q = false, Q && (P.alpha = Q.entityAlpha, y.checked !== Q.regionsChecked && (y.checked = Q.regionsChecked, L()), p.checked !== Q.colorChecked && (p.checked = Q.colorChecked, Di(p.checked), nt && as(nt)), Q = null), kt());
      }), R_(Dt, as);
    }
  }
  d1().catch((n) => {
    console.error("Failed to initialize app:", n);
  });
})();
export {
  Fh as $,
  xa as A,
  rr as B,
  Ht as C,
  Om as D,
  hn as E,
  kf as F,
  vi as G,
  ns as H,
  Ms as I,
  ue as J,
  ie as K,
  Zt as L,
  ni as M,
  kh as N,
  Ct as O,
  ct as P,
  pt as Q,
  Yt as R,
  br as S,
  S as T,
  ye as U,
  Hy as V,
  Bg as W,
  wr as X,
  Ag as Y,
  vn as Z,
  vr as _,
  __tla,
  dr as a,
  Bt as a0,
  rd as a1,
  Bo as a2,
  Ut as a3,
  Jn as a4,
  hi as a5,
  Rt as a6,
  Rp as a7,
  Rm as a8,
  Bs as a9,
  Ld as aA,
  od as aB,
  wi as aC,
  Jf as aD,
  Yf as aE,
  yd as aF,
  Fm as aG,
  cg as aH,
  dg as aI,
  xg as aJ,
  gg as aK,
  bg as aL,
  Pl as aa,
  tr as ab,
  ma as ac,
  nd as ad,
  Oe as ae,
  ya as af,
  lg as ag,
  hg as ah,
  yg as ai,
  fg as aj,
  xp as ak,
  _g as al,
  Me as am,
  Th as an,
  $e as ao,
  gr as ap,
  pl as aq,
  ze as ar,
  _x as as,
  qp as at,
  ml as au,
  zr as av,
  gl as aw,
  Kp as ax,
  Mh as ay,
  Cr as az,
  Dn as b,
  Ar as c,
  f1 as d,
  or as e,
  ga as f,
  Ot as g,
  At as h,
  Ko as i,
  ts as j,
  fn as k,
  qo as l,
  Wi as m,
  Jt as n,
  De as o,
  Nd as p,
  Fd as q,
  li as r,
  We as s,
  p1 as t,
  cx as u,
  se as v,
  Ky as w,
  oe as x,
  Qt as y,
  Pt as z
};
