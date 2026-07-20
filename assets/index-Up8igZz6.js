const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/browserAll-6tf3VCUQ.js","assets/webworkerAll-u00P3pHA.js","assets/Filter-DGcyslFP.js","assets/WebGPURenderer-D8eqHiLd.js","assets/BufferResource-Bdx0gxHR.js","assets/RenderTargetSystem-RSfjldaz.js","assets/WebGLRenderer-DW_Huo9Y.js","assets/CanvasRenderer-vsy2PsgG.js"])))=>i.map(i=>d[i]);
let bh, sa, Zi, Ht, am, rn, Qp, mi, Zn, Cs, le, ee, Kt, qs, ch, vt, lt, ft, Dt, ur, S, ue, my, lg, fr, Qm, yn, mr, ir, Rt, Gh, vo, Gt, Yn, si, Mt, ip, im, Ps, fd, zh, fi, Cf, xf, Zh, hm, Bm, Nm, Um, Dm, jm, pl, Yi, ta, Nh, Ie, na, $m, Om, Hm, Gm, Uu, Vm, ve, ah, Me, cr, Ka, Ne, Vy, wp, Za, Rr, Qa, vp, hh, gr, Fn, _r, Cv, Qi, ea, It, Tt, Oo, Kn, cn, Bo, Ri, Yt, Fe, xd, bd, ei, Be, vv, By, te, vy, ne, Jt, kt;
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
  const Pu = "modulepreload", Iu = function(n) {
    return "/spaghettio/" + n;
  }, Ba = {}, Pn = function(t, e, s) {
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
        if (c = Iu(c), c in Ba) return;
        Ba[c] = true;
        const h = c.endsWith(".css"), d = h ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${c}"]${d}`)) return;
        const u = document.createElement("link");
        if (u.rel = h ? "stylesheet" : Pu, h || (u.as = "script"), u.crossOrigin = "", u.href = c, l && u.setAttribute("nonce", l), document.head.appendChild(u), h) return new Promise((p, f) => {
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
  let po, xi, Ru, Lu;
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
  xi = (n, t) => po(n).priority ?? t;
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
        }), t.sort((r, o) => xi(o.value, e) - xi(r.value, e)));
      }, (s) => {
        const i = t.findIndex((r) => r.name === s.name);
        i !== -1 && t.splice(i, 1);
      });
    },
    handleByList(n, t, e = -1) {
      return this.handle(n, (s) => {
        t.includes(s.ref) || (t.push(s.ref), t.sort((i, r) => xi(r, e) - xi(i, e)));
      }, (s) => {
        const i = t.indexOf(s.ref);
        i !== -1 && t.splice(i, 1);
      });
    },
    mixin(n, ...t) {
      for (const e of t) Object.defineProperties(n.prototype, Object.getOwnPropertyDescriptors(e));
    }
  };
  Ru = {
    extension: {
      type: lt.Environment,
      name: "browser",
      priority: -1
    },
    test: () => true,
    load: async () => {
      await Pn(() => import("./browserAll-6tf3VCUQ.js"), __vite__mapDeps([0,1,2]));
    }
  };
  Lu = {
    extension: {
      type: lt.Environment,
      name: "webworker",
      priority: 0
    },
    test: () => typeof self < "u" && self.WorkerGlobalScope !== void 0,
    load: async () => {
      await Pn(() => import("./webworkerAll-u00P3pHA.js"), __vite__mapDeps([1,2]));
    }
  };
  class de {
    constructor(t, e, s) {
      this._x = e || 0, this._y = s || 0, this._observer = t;
    }
    clone(t) {
      return new de(t ?? this._observer, this._x, this._y);
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
  function qc(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var Cr = {
    exports: {}
  }, Oa;
  function $u() {
    return Oa || (Oa = 1, (function(n) {
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
        var m = this._events[g], y = arguments.length, w, x;
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
          for (x = 1, w = new Array(y - 1); x < y; x++) w[x - 1] = arguments[x];
          m.fn.apply(m.context, w);
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
              if (!w) for (v = 1, w = new Array(y - 1); v < y; v++) w[v - 1] = arguments[v];
              m[x].fn.apply(m[x].context, w);
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
    })(Cr)), Cr.exports;
  }
  var Bu = $u();
  let Ou, Nu, Fu;
  rn = qc(Bu);
  Ou = Math.PI * 2;
  Nu = 180 / Math.PI;
  Fu = Math.PI / 180;
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
      return Sr.x = 0, Sr.y = 0, Sr;
    }
  };
  const Sr = new Mt();
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
      return c < 1e-5 || Math.abs(Ou - c) < 1e-5 ? (t.rotation = l, t.skew.x = t.skew.y = 0) : (t.rotation = 0, t.skew.x = a, t.skew.y = l), t.scale.x = Math.sqrt(e * e + s * s), t.scale.y = Math.sqrt(i * i + r * r), t.position.x = this.tx + (o.x * e + o.y * i), t.position.y = this.ty + (o.x * s + o.y * r), t;
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
      return Gu.identity();
    }
    static get shared() {
      return Wu.identity();
    }
  };
  const Wu = new vt(), Gu = new vt(), zn = [
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
  ], fo = [], Kc = [], bi = Math.sign;
  function zu() {
    for (let n = 0; n < 16; n++) {
      const t = [];
      fo.push(t);
      for (let e = 0; e < 16; e++) {
        const s = bi(zn[n] * zn[e] + Hn[n] * Dn[e]), i = bi(Dn[n] * zn[e] + Un[n] * Dn[e]), r = bi(zn[n] * Hn[e] + Hn[n] * Un[e]), o = bi(Dn[n] * Hn[e] + Un[n] * Un[e]);
        for (let a = 0; a < 16; a++) if (zn[a] === s && Dn[a] === i && Hn[a] === r && Un[a] === o) {
          t.push(a);
          break;
        }
      }
    }
    for (let n = 0; n < 16; n++) {
      const t = new vt();
      t.set(zn[n], Dn[n], Hn[n], Un[n], 0, 0), Kc.push(t);
    }
  }
  zu();
  let _i;
  kt = {
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
    sub: (n, t) => fo[n][kt.inv(t)],
    rotate180: (n) => n ^ 4,
    isVertical: (n) => (n & 3) === 2,
    byDirection: (n, t) => Math.abs(n) * 2 <= Math.abs(t) ? t >= 0 ? kt.S : kt.N : Math.abs(t) * 2 <= Math.abs(n) ? n > 0 ? kt.E : kt.W : t > 0 ? n > 0 ? kt.SE : kt.SW : n > 0 ? kt.NE : kt.NW,
    matrixAppendRotationInv: (n, t, e = 0, s = 0, i = 0, r = 0) => {
      const o = Kc[kt.inv(t)], a = o.a, l = o.b, c = o.c, h = o.d, d = e - Math.min(0, a * i, c * r, a * i + c * r), u = s - Math.min(0, l * i, h * r, l * i + h * r), p = n.a, f = n.b, g = n.c, m = n.d;
      n.a = a * p + l * g, n.b = a * f + l * m, n.c = c * p + h * g, n.d = c * f + h * m, n.tx = d * p + u * g + n.tx, n.ty = d * f + u * m + n.ty;
    },
    transformRectCoords: (n, t, e, s) => {
      const { x: i, y: r, width: o, height: a } = n, { x: l, y: c, width: h, height: d } = t;
      return e === kt.E ? (s.set(i + l, r + c, o, a), s) : e === kt.S ? s.set(h - r - a + l, i + c, a, o) : e === kt.W ? s.set(h - i - o + l, d - r - a + c, o, a) : e === kt.N ? s.set(r + l, d - i - o + c, a, o) : s.set(i + l, r + c, o, a);
    }
  };
  _i = [
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
      const a = this.x, l = this.y, c = s * (1 - i), h = s - c, d = a - c, u = a + r + c, p = l - c, f = l + o + c, g = a + h, m = a + r - h, y = l + h, w = l + o - h;
      return t >= d && t <= u && e >= p && e <= f && !(t > g && t < m && e > y && e < w);
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
      const a = _i[0].set(t.left, t.top), l = _i[1].set(t.left, t.bottom), c = _i[2].set(t.right, t.top), h = _i[3].set(t.right, t.bottom);
      if (c.x <= a.x || l.y <= a.y) return false;
      const d = Math.sign(e.a * e.d - e.b * e.c);
      if (d === 0 || (e.apply(a, a), e.apply(l, l), e.apply(c, c), e.apply(h, h), Math.max(a.x, l.x, c.x, h.x) <= s || Math.min(a.x, l.x, c.x, h.x) >= i || Math.max(a.y, l.y, c.y, h.y) <= r || Math.min(a.y, l.y, c.y, h.y) >= o)) return false;
      const u = d * (l.y - a.y), p = d * (a.x - l.x), f = u * s + p * r, g = u * i + p * r, m = u * s + p * o, y = u * i + p * o;
      if (Math.max(f, g, m, y) <= u * a.x + p * a.y || Math.min(f, g, m, y) >= u * h.x + p * h.y) return false;
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
  const Tr = {
    default: -1
  };
  ee = function(n = "default") {
    return Tr[n] === void 0 && (Tr[n] = -1), ++Tr[n];
  };
  let Na, Du, ps;
  Na = /* @__PURE__ */ new Set();
  te = "8.0.0";
  Du = "8.3.4";
  ps = {
    quiet: false,
    noColor: false
  };
  It = ((n, t, e = 3) => {
    if (ps.quiet || Na.has(t)) return;
    let s = new Error().stack;
    const i = `${t}
Deprecated since v${n}`, r = typeof console.groupCollapsed == "function" && !ps.noColor;
    typeof s > "u" ? console.warn("PixiJS Deprecation Warning: ", i) : (s = s.split(`
`).splice(e).join(`
`), r ? (console.groupCollapsed("%cPixiJS Deprecation Warning: %c%s", "color:#614108;background:#fffbe6", "font-weight:normal;color:#614108;background:#fffbe6", i), console.warn(s), console.groupEnd()) : (console.warn("PixiJS Deprecation Warning: ", i), console.warn(s))), Na.add(t);
  });
  Object.defineProperties(It, {
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
  const Jc = () => {
  };
  function ws(n) {
    return n += n === 0 ? 1 : 0, --n, n |= n >>> 1, n |= n >>> 2, n |= n >>> 4, n |= n >>> 8, n |= n >>> 16, n + 1;
  }
  function Fa(n) {
    return !(n & n - 1) && !!n;
  }
  function Zc(n) {
    const t = {};
    for (const e in n) n[e] !== void 0 && (t[e] = n[e]);
    return t;
  }
  const Wa = /* @__PURE__ */ Object.create(null);
  function Hu(n) {
    const t = Wa[n];
    return t === void 0 && (Wa[n] = ee("resource")), t;
  }
  const Qc = class th extends rn {
    constructor(t = {}) {
      super(), this._resourceType = "textureSampler", this._touched = 0, this._maxAnisotropy = 1, this.destroyed = false, t = {
        ...th.defaultOptions,
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
      It(te, "TextureStyle.wrapMode is now TextureStyle.addressMode"), this.addressMode = t;
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
      return this._sharedResourceId = Hu(t), this._resourceId;
    }
    destroy() {
      this.destroyed = true, this.emit("destroy", this), this.emit("change", this), this.removeAllListeners();
    }
  };
  Qc.defaultOptions = {
    addressMode: "clamp-to-edge",
    scaleMode: "linear"
  };
  Kn = Qc;
  const eh = class nh extends rn {
    constructor(t = {}) {
      super(), this.options = t, this._gpuData = /* @__PURE__ */ Object.create(null), this._gcLastUsed = -1, this.uid = ee("textureSource"), this._resourceType = "textureSource", this._resourceId = ee("resource"), this.uploadMethodId = "unknown", this._resolution = 1, this.pixelWidth = 1, this.pixelHeight = 1, this.width = 1, this.height = 1, this.sampleCount = 1, this.mipLevelCount = 1, this.autoGenerateMipmaps = false, this.format = "rgba8unorm", this.dimension = "2d", this.viewDimension = "2d", this.arrayLayerCount = 1, this.antialias = false, this._touched = 0, this._batchTick = -1, this._textureBindLocation = -1, t = {
        ...nh.defaultOptions,
        ...t
      }, this.label = t.label ?? "", this.resource = t.resource, this.autoGarbageCollect = t.autoGarbageCollect, this._resolution = t.resolution, t.width ? this.pixelWidth = t.width * this._resolution : this.pixelWidth = this.resource ? this.resourceWidth ?? 1 : 1, t.height ? this.pixelHeight = t.height * this._resolution : this.pixelHeight = this.resource ? this.resourceHeight ?? 1 : 1, this.width = this.pixelWidth / this._resolution, this.height = this.pixelHeight / this._resolution, this.format = t.format, this.dimension = t.dimensions, this.viewDimension = t.viewDimension ?? t.dimensions, this.arrayLayerCount = t.arrayLayerCount, this.mipLevelCount = t.mipLevelCount, this.autoGenerateMipmaps = t.autoGenerateMipmaps, this.sampleCount = t.sampleCount, this.antialias = t.antialias, this.alphaMode = t.alphaMode, this.style = new Kn(Zc(t)), this.destroyed = false, this._refreshPOT();
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
      this.isPowerOfTwo = Fa(this.pixelWidth) && Fa(this.pixelHeight);
    }
    static test(t) {
      throw new Error("Unimplemented");
    }
  };
  eh.defaultOptions = {
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
  Ie = eh;
  class Jo extends Ie {
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
  Jo.extension = lt.TextureSource;
  const Ga = new vt();
  Uu = class {
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
      i && (Ga.set(s.width / i.width, 0, 0, s.height / i.height, -i.x / i.width, -i.y / i.height), this.mapCoord.append(Ga));
      const r = t.source, o = this.uClampFrame, a = this.clampMargin / r._resolution, l = this.clampOffset / r._resolution;
      return o[0] = (t.frame.x + a + l) / r.width, o[1] = (t.frame.y + a + l) / r.height, o[2] = (t.frame.x + t.frame.width - a + l) / r.width, o[3] = (t.frame.y + t.frame.height - a + l) / r.height, this.uClampOffset[0] = this.clampOffset / r.pixelWidth, this.uClampOffset[1] = this.clampOffset / r.pixelHeight, this.isSimple = t.frame.width === r.width && t.frame.height === r.height && t.rotate === 0, true;
    }
  };
  Tt = class extends rn {
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
      }, this.frame = new Dt(), this.noFrame = false, this.dynamic = false, this.isTexture = true, this.label = e, this.source = (t == null ? void 0 : t.source) ?? new Ie(), this.noFrame = !s, s) this.frame.copyFrom(s);
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
      return this._textureMatrix || (this._textureMatrix = new Uu(this)), this._textureMatrix;
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
        c = kt.add(c, kt.NW), t.x0 = u + h * kt.uX(c), t.y0 = p + d * kt.uY(c), c = kt.add(c, 2), t.x1 = u + h * kt.uX(c), t.y1 = p + d * kt.uY(c), c = kt.add(c, 2), t.x2 = u + h * kt.uX(c), t.y2 = p + d * kt.uY(c), c = kt.add(c, 2), t.x3 = u + h * kt.uX(c), t.y3 = p + d * kt.uY(c);
      } else t.x0 = r, t.y0 = o, t.x1 = r + a, t.y1 = o, t.x2 = r + a, t.y2 = o + l, t.x3 = r, t.y3 = o + l;
    }
    destroy(t = false) {
      this._source && (this._source.off("resize", this.update, this), t && (this._source.destroy(), this._source = null)), this._textureMatrix = null, this.destroyed = true, this.emit("destroy", this), this.removeAllListeners();
    }
    update() {
      this.noFrame && (this.frame.width = this._source.width, this.frame.height = this._source.height), this.updateUvs(), this.emit("update", this);
    }
    get baseTexture() {
      return It(te, "Texture.baseTexture is now Texture.source"), this._source;
    }
  };
  Tt.EMPTY = new Tt({
    label: "EMPTY",
    source: new Ie({
      label: "EMPTY"
    })
  });
  Tt.EMPTY.destroy = Jc;
  Tt.WHITE = new Tt({
    source: new Jo({
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
  Tt.WHITE.destroy = Jc;
  function sh(n, t, e) {
    const { width: s, height: i } = e.orig, r = e.trim;
    if (r) {
      const o = r.width, a = r.height;
      n.minX = r.x - t._x * s, n.maxX = n.minX + o, n.minY = r.y - t._y * i, n.maxY = n.minY + a;
    } else n.minX = -t._x * s, n.maxX = n.minX + s, n.minY = -t._y * i, n.maxY = n.minY + i;
  }
  const za = new vt();
  Me = class {
    constructor(t = 1 / 0, e = 1 / 0, s = -1 / 0, i = -1 / 0) {
      this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = za, this.minX = t, this.minY = e, this.maxX = s, this.maxY = i;
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
      return this.minX = 1 / 0, this.minY = 1 / 0, this.maxX = -1 / 0, this.maxY = -1 / 0, this.matrix = za, this;
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
      return new Me(this.minX, this.minY, this.maxX, this.maxY);
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
        const m = t[g], y = t[g + 1], w = c * m + d * y + p, x = h * m + u * y + f;
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
  var ju = {
    grad: 0.9,
    turn: 360,
    rad: 360 / (2 * Math.PI)
  }, on = function(n) {
    return typeof n == "string" ? n.length > 0 : typeof n == "number";
  }, ae = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = Math.pow(10, t)), Math.round(e * n) / e + 0;
  }, Le = function(n, t, e) {
    return t === void 0 && (t = 0), e === void 0 && (e = 1), n > e ? e : n > t ? n : t;
  }, ih = function(n) {
    return (n = isFinite(n) ? n % 360 : 0) > 0 ? n : n + 360;
  }, Da = function(n) {
    return {
      r: Le(n.r, 0, 255),
      g: Le(n.g, 0, 255),
      b: Le(n.b, 0, 255),
      a: Le(n.a)
    };
  }, Er = function(n) {
    return {
      r: ae(n.r),
      g: ae(n.g),
      b: ae(n.b),
      a: ae(n.a, 3)
    };
  }, Vu = /^#([0-9a-f]{3,8})$/i, wi = function(n) {
    var t = n.toString(16);
    return t.length < 2 ? "0" + t : t;
  }, rh = function(n) {
    var t = n.r, e = n.g, s = n.b, i = n.a, r = Math.max(t, e, s), o = r - Math.min(t, e, s), a = o ? r === t ? (e - s) / o : r === e ? 2 + (s - t) / o : 4 + (t - e) / o : 0;
    return {
      h: 60 * (a < 0 ? a + 6 : a),
      s: r ? o / r * 100 : 0,
      v: r / 255 * 100,
      a: i
    };
  }, oh = function(n) {
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
  }, Ha = function(n) {
    return {
      h: ih(n.h),
      s: Le(n.s, 0, 100),
      l: Le(n.l, 0, 100),
      a: Le(n.a)
    };
  }, Ua = function(n) {
    return {
      h: ae(n.h),
      s: ae(n.s),
      l: ae(n.l),
      a: ae(n.a, 3)
    };
  }, ja = function(n) {
    return oh((e = (t = n).s, {
      h: t.h,
      s: (e *= ((s = t.l) < 50 ? s : 100 - s) / 100) > 0 ? 2 * e / (s + e) * 100 : 0,
      v: s + e,
      a: t.a
    }));
    var t, e, s;
  }, Xs = function(n) {
    return {
      h: (t = rh(n)).h,
      s: (i = (200 - (e = t.s)) * (s = t.v) / 100) > 0 && i < 200 ? e * s / 100 / (i <= 100 ? i : 200 - i) * 100 : 0,
      l: i / 2,
      a: t.a
    };
    var t, e, s, i;
  }, Yu = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s*,\s*([+-]?\d*\.?\d+)%\s*,\s*([+-]?\d*\.?\d+)%\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Xu = /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s+([+-]?\d*\.?\d+)%\s+([+-]?\d*\.?\d+)%\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, qu = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, Ku = /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i, mo = {
    string: [
      [
        function(n) {
          var t = Vu.exec(n);
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
          var t = qu.exec(n) || Ku.exec(n);
          return t ? t[2] !== t[4] || t[4] !== t[6] ? null : Da({
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
          var t = Yu.exec(n) || Xu.exec(n);
          if (!t) return null;
          var e, s, i = Ha({
            h: (e = t[1], s = t[2], s === void 0 && (s = "deg"), Number(e) * (ju[s] || 1)),
            s: Number(t[3]),
            l: Number(t[4]),
            a: t[5] === void 0 ? 1 : Number(t[5]) / (t[6] ? 100 : 1)
          });
          return ja(i);
        },
        "hsl"
      ]
    ],
    object: [
      [
        function(n) {
          var t = n.r, e = n.g, s = n.b, i = n.a, r = i === void 0 ? 1 : i;
          return on(t) && on(e) && on(s) ? Da({
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
          var o = Ha({
            h: Number(t),
            s: Number(e),
            l: Number(s),
            a: Number(r)
          });
          return ja(o);
        },
        "hsl"
      ],
      [
        function(n) {
          var t = n.h, e = n.s, s = n.v, i = n.a, r = i === void 0 ? 1 : i;
          if (!on(t) || !on(e) || !on(s)) return null;
          var o = (function(a) {
            return {
              h: ih(a.h),
              s: Le(a.s, 0, 100),
              v: Le(a.v, 0, 100),
              a: Le(a.a)
            };
          })({
            h: Number(t),
            s: Number(e),
            v: Number(s),
            a: Number(r)
          });
          return oh(o);
        },
        "hsv"
      ]
    ]
  }, Va = function(n, t) {
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
  }, Ju = function(n) {
    return typeof n == "string" ? Va(n.trim(), mo.string) : typeof n == "object" && n !== null ? Va(n, mo.object) : [
      null,
      void 0
    ];
  }, Ar = function(n, t) {
    var e = Xs(n);
    return {
      h: e.h,
      s: Le(e.s + 100 * t, 0, 100),
      l: e.l,
      a: e.a
    };
  }, kr = function(n) {
    return (299 * n.r + 587 * n.g + 114 * n.b) / 1e3 / 255;
  }, Ya = function(n, t) {
    var e = Xs(n);
    return {
      h: e.h,
      s: e.s,
      l: Le(e.l + 100 * t, 0, 100),
      a: e.a
    };
  }, go = (function() {
    function n(t) {
      this.parsed = Ju(t)[0], this.rgba = this.parsed || {
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
      return t = Er(this.rgba), e = t.r, s = t.g, i = t.b, o = (r = t.a) < 1 ? wi(ae(255 * r)) : "", "#" + wi(e) + wi(s) + wi(i) + o;
      var t, e, s, i, r, o;
    }, n.prototype.toRgb = function() {
      return Er(this.rgba);
    }, n.prototype.toRgbString = function() {
      return t = Er(this.rgba), e = t.r, s = t.g, i = t.b, (r = t.a) < 1 ? "rgba(" + e + ", " + s + ", " + i + ", " + r + ")" : "rgb(" + e + ", " + s + ", " + i + ")";
      var t, e, s, i, r;
    }, n.prototype.toHsl = function() {
      return Ua(Xs(this.rgba));
    }, n.prototype.toHslString = function() {
      return t = Ua(Xs(this.rgba)), e = t.h, s = t.s, i = t.l, (r = t.a) < 1 ? "hsla(" + e + ", " + s + "%, " + i + "%, " + r + ")" : "hsl(" + e + ", " + s + "%, " + i + "%)";
      var t, e, s, i, r;
    }, n.prototype.toHsv = function() {
      return t = rh(this.rgba), {
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
      return t === void 0 && (t = 0.1), Je(Ya(this.rgba, t));
    }, n.prototype.darken = function(t) {
      return t === void 0 && (t = 0.1), Je(Ya(this.rgba, -t));
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
  }, Xa = [], Zu = function(n) {
    n.forEach(function(t) {
      Xa.indexOf(t) < 0 && (t(go, mo), Xa.push(t));
    });
  };
  function Qu(n, t) {
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
  Zu([
    Qu
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
  Yt = vs;
  const tp = {
    cullArea: null,
    cullable: false,
    cullableChildren: true
  };
  let Mr = 0;
  const qa = 500;
  Jt = function(...n) {
    Mr !== qa && (Mr++, Mr === qa ? console.warn("PixiJS Warning: too many warnings, no more warnings will be reported to the console by PixiJS.") : console.warn("PixiJS Warning: ", ...n));
  };
  fi = {
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
  class ep {
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
  class np {
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
      return this._poolsByClass.has(t) || this._poolsByClass.set(t, new ep(t)), this._poolsByClass.get(t);
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
  ve = new np();
  fi.register(ve);
  const sp = {
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
  ip = function(n, t, e) {
    const s = n.length;
    let i;
    if (t >= s || e === 0) return;
    e = t + e > s ? s - t : e;
    const r = s - e;
    for (i = t; i < r; ++i) n[i] = n[i + e];
    n.length = r;
  };
  const rp = {
    allowChildren: true,
    removeChildren(n = 0, t) {
      var _a2;
      const e = t ?? this.children.length, s = e - n, i = [];
      if (s > 0 && s <= e) {
        for (let o = e - 1; o >= n; o--) {
          const a = this.children[o];
          a && (i.push(a), a.parent = null);
        }
        ip(this.children, n, e);
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
      this.allowChildren || It(te, "addChildAt: Only Containers will be allowed to add children in v8.0.0");
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
  }, op = {
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
  Ka = class {
    constructor() {
      this.pipe = "filter", this.priority = 1;
    }
    destroy() {
      for (let t = 0; t < this.filters.length; t++) this.filters[t].destroy();
      this.filters = null, this.filterArea = null;
    }
  };
  class ap {
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
  const yo = new ap();
  Gt.handleByList(lt.MaskEffect, yo._effectClasses);
  const lp = {
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
      const t = this._filterEffect || (this._filterEffect = new Ka());
      n = n;
      const e = (n == null ? void 0 : n.length) > 0, s = ((_a2 = t.filters) == null ? void 0 : _a2.length) > 0, i = e !== s;
      n = Array.isArray(n) ? n.slice(0) : n, t.filters = Object.freeze(n), i && (e ? this.addEffect(t) : (this.removeEffect(t), t.filters = n ?? null));
    },
    get filters() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filters;
    },
    set filterArea(n) {
      this._filterEffect || (this._filterEffect = new Ka()), this._filterEffect.filterArea = n;
    },
    get filterArea() {
      var _a2;
      return (_a2 = this._filterEffect) == null ? void 0 : _a2.filterArea;
    }
  }, cp = {
    label: null,
    get name() {
      return It(te, "Container.name property has been removed, use Container.label instead"), this.label;
    },
    set name(n) {
      It(te, "Container.name property has been removed, use Container.label instead"), this.label = n;
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
  }, ye = ve.getPool(vt), fn = ve.getPool(Me), hp = new vt(), dp = {
    getFastGlobalBounds(n, t) {
      t || (t = new Me()), t.clear(), this._getGlobalBoundsRecursive(!!n, t, this.parentRenderLayer), t.isValid || t.set(0, 0, 0, 0);
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
        r && s.applyMatrix(o.worldTransform.copyTo(hp).invert()), t.addBounds(s), fn.return(s);
      } else this.renderGroup && (t.addBounds(s, this.relativeGroupTransform), fn.return(s));
    }
  };
  ah = function(n, t, e) {
    e.clear();
    let s, i;
    return n.parent ? t ? s = n.parent.worldTransform : (i = ye.get().identity(), s = Zo(n, i)) : s = vt.IDENTITY, lh(n, e, s, t), i && ye.return(i), e.isValid || e.set(0, 0, 0, 0), e;
  };
  function lh(n, t, e, s) {
    var _a2, _b2;
    if (!n.visible || !n.measurable) return;
    let i;
    s ? i = n.worldTransform : (n.updateLocalTransform(), i = ye.get(), i.appendFrom(n.localTransform, e));
    const r = t, o = !!n.effects.length;
    if (o && (t = fn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, i);
    else {
      const a = n.bounds;
      a && !a.isEmpty() && (t.matrix = i, t.addBounds(a));
      for (let l = 0; l < n.children.length; l++) lh(n.children[l], t, i, s);
    }
    if (o) {
      for (let a = 0; a < n.effects.length; a++) (_b2 = (_a2 = n.effects[a]).addBounds) == null ? void 0 : _b2.call(_a2, t);
      r.addBounds(t, vt.IDENTITY), fn.return(t);
    }
    s || ye.return(i);
  }
  function Zo(n, t) {
    const e = n.parent;
    return e && (Zo(e, t), e.updateLocalTransform(), t.append(e.localTransform)), t;
  }
  ch = function(n, t) {
    if (n === 16777215 || !t) return t;
    if (t === 16777215 || !n) return n;
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = t >> 16 & 255, o = t >> 8 & 255, a = t & 255, l = e * r / 255 | 0, c = s * o / 255 | 0, h = i * a / 255 | 0;
    return (l << 16) + (c << 8) + h;
  };
  const Ja = 16777215;
  Za = function(n, t) {
    return n === Ja ? t : t === Ja ? n : ch(n, t);
  };
  qs = function(n) {
    return ((n & 255) << 16) + (n & 65280) + (n >> 16 & 255);
  };
  const up = {
    getGlobalAlpha(n) {
      if (n) return this.renderGroup ? this.renderGroup.worldAlpha : this.parentRenderGroup ? this.parentRenderGroup.worldAlpha * this.alpha : this.alpha;
      let t = this.alpha, e = this.parent;
      for (; e; ) t *= e.alpha, e = e.parent;
      return t;
    },
    getGlobalTransform(n = new vt(), t) {
      if (t) return n.copyFrom(this.worldTransform);
      this.updateLocalTransform();
      const e = Zo(this, ye.get().identity());
      return n.appendFrom(this.localTransform, e), ye.return(e), n;
    },
    getGlobalTint(n) {
      if (n) return this.renderGroup ? qs(this.renderGroup.worldColor) : this.parentRenderGroup ? qs(Za(this.localColor, this.parentRenderGroup.worldColor)) : this.tint;
      let t = this.localColor, e = this.parent;
      for (; e; ) t = Za(t, e.localColor), e = e.parent;
      return qs(t);
    }
  };
  hh = function(n, t, e) {
    return t.clear(), e || (e = vt.IDENTITY), dh(n, t, e, n, true), t.isValid || t.set(0, 0, 0, 0), t;
  };
  function dh(n, t, e, s, i) {
    var _a2, _b2;
    let r;
    if (i) r = ye.get(), r = e.copyTo(r);
    else {
      if (!n.visible || !n.measurable) return;
      n.updateLocalTransform();
      const l = n.localTransform;
      r = ye.get(), r.appendFrom(l, e);
    }
    const o = t, a = !!n.effects.length;
    if (a && (t = fn.get().clear()), n.boundsArea) t.addRect(n.boundsArea, r);
    else {
      n.renderPipeId && (t.matrix = r, t.addBounds(n.bounds));
      const l = n.children;
      for (let c = 0; c < l.length; c++) dh(l[c], t, r, s, false);
    }
    if (a) {
      for (let l = 0; l < n.effects.length; l++) (_b2 = (_a2 = n.effects[l]).addLocalBounds) == null ? void 0 : _b2.call(_a2, t, s);
      o.addBounds(t, vt.IDENTITY), fn.return(t);
    }
    ye.return(r);
  }
  function uh(n, t) {
    const e = n.children;
    for (let s = 0; s < e.length; s++) {
      const i = e[s], r = i.uid, o = (i._didViewChangeTick & 65535) << 16 | i._didContainerChangeTick & 65535, a = t.index;
      (t.data[a] !== r || t.data[a + 1] !== o) && (t.data[t.index] = r, t.data[t.index + 1] = o, t.didChange = true), t.index = a + 2, i.children.length && uh(i, t);
    }
    return t.didChange;
  }
  const pp = new vt(), fp = {
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
        localBounds: new Me()
      });
      const n = this._localBoundsCacheData;
      return n.index = 1, n.didChange = false, n.data[0] !== this._didViewChangeTick && (n.didChange = true, n.data[0] = this._didViewChangeTick), uh(this, n), n.didChange && hh(this, n.localBounds, pp), n.localBounds;
    },
    getBounds(n, t) {
      return ah(this, n, t || new Me());
    }
  }, mp = {
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
  }, gp = {
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
      this.sortDirty && (this.sortDirty = false, this.children.sort(yp));
    }
  };
  function yp(n, t) {
    return n._zIndex - t._zIndex;
  }
  const xp = {
    getGlobalPosition(n = new Mt(), t = false) {
      return this.parent ? this.parent.toGlobal(this._position, n, t) : (n.x = this._position.x, n.y = this._position.y), n;
    },
    toGlobal(n, t, e = false) {
      const s = this.getGlobalTransform(ye.get(), e);
      return t = s.apply(n, t), ye.return(s), t;
    },
    toLocal(n, t, e, s) {
      t && (n = t.toGlobal(n, e, s));
      const i = this.getGlobalTransform(ye.get(), s);
      return e = i.applyInverse(n, e), ye.return(i), e;
    }
  };
  class Qo {
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
  let bp = 0;
  class _p {
    constructor(t) {
      this._poolKeyHash = /* @__PURE__ */ Object.create(null), this._texturePool = {}, this.textureOptions = t || {}, this.enableFullScreen = false, this.textureStyle = new Kn(this.textureOptions);
    }
    createTexture(t, e, s, i) {
      const r = new Ie({
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
        label: `texturePool_${bp++}`
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
  cr = new _p();
  fi.register(cr);
  wp = class {
    constructor() {
      this.renderPipeId = "renderGroup", this.root = null, this.canBundle = false, this.renderGroupParent = null, this.renderGroupChildren = [], this.worldTransform = new vt(), this.worldColorAlpha = 4294967295, this.worldColor = 16777215, this.worldAlpha = 1, this.childrenToUpdate = /* @__PURE__ */ Object.create(null), this.updateTick = 0, this.gcTick = 0, this.childrenRenderablesToUpdate = {
        list: [],
        index: 0
      }, this.structureDidChange = true, this.instructionSet = new Qo(), this._onRenderContainers = [], this.textureNeedsUpdate = true, this.isCachedAsTexture = false, this._matrixDirty = 7;
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
  let Pr, vi, Ir, Ci;
  Pr = new de(null);
  vi = new de(null);
  Ir = new de(null, 1, 1);
  Ci = new de(null);
  Qa = 1;
  vp = 2;
  Rr = 4;
  Ht = class extends rn {
    constructor(t = {}) {
      var _a2, _b2;
      super(), this.uid = ee("renderable"), this._updateFlags = 15, this.renderGroup = null, this.parentRenderGroup = null, this.parentRenderGroupIndex = 0, this.didChange = false, this.didViewUpdate = false, this.relativeRenderGroupDepth = 0, this.children = [], this.parent = null, this.includeInBuild = true, this.measurable = true, this.isSimple = true, this.parentRenderLayer = null, this.updateTick = -1, this.localTransform = new vt(), this.relativeGroupTransform = new vt(), this.groupTransform = this.relativeGroupTransform, this.destroyed = false, this._position = new de(this, 0, 0), this._scale = Ir, this._pivot = vi, this._origin = Ci, this._skew = Pr, this._cx = 1, this._sx = 0, this._cy = 0, this._sy = 1, this._rotation = 0, this.localColor = 16777215, this.localAlpha = 1, this.groupAlpha = 1, this.groupColor = 16777215, this.groupColorAlpha = 4294967295, this.localBlendMode = "inherit", this.groupBlendMode = "normal", this.localDisplayStatus = 7, this.globalDisplayStatus = 7, this._didContainerChangeTick = 0, this._didViewChangeTick = 0, this._didLocalTransformChangeId = -1, this.effects = [], xo(this, t, {
        children: true,
        parent: true,
        effects: true
      }), (_a2 = t.children) == null ? void 0 : _a2.forEach((e) => this.addChild(e)), (_b2 = t.parent) == null ? void 0 : _b2.addChild(this);
    }
    static mixin(t) {
      It("8.8.0", "Container.mixin is deprecated, please use extensions.mixin instead."), Gt.mixin(Ht, t);
    }
    set _didChangeId(t) {
      this._didViewChangeTick = t >> 12 & 4095, this._didContainerChangeTick = t & 4095;
    }
    get _didChangeId() {
      return this._didContainerChangeTick & 4095 | (this._didViewChangeTick & 4095) << 12;
    }
    addChild(...t) {
      if (this.allowChildren || It(te, "addChild: Only Containers will be allowed to add children in v8.0.0"), t.length > 1) {
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
      t == null ? void 0 : t.removeChild(this), this.renderGroup = ve.get(wp, this), this.groupTransform = vt.IDENTITY, t == null ? void 0 : t.addChild(this), this._updateIsSimple();
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
      return this.rotation * Nu;
    }
    set angle(t) {
      this.rotation = t * Fu;
    }
    get pivot() {
      return this._pivot === vi && (this._pivot = new de(this, 0, 0)), this._pivot;
    }
    set pivot(t) {
      this._pivot === vi && (this._pivot = new de(this, 0, 0), this._origin !== Ci && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._pivot.set(t) : this._pivot.copyFrom(t);
    }
    get skew() {
      return this._skew === Pr && (this._skew = new de(this, 0, 0)), this._skew;
    }
    set skew(t) {
      this._skew === Pr && (this._skew = new de(this, 0, 0)), this._skew.copyFrom(t);
    }
    get scale() {
      return this._scale === Ir && (this._scale = new de(this, 1, 1)), this._scale;
    }
    set scale(t) {
      this._scale === Ir && (this._scale = new de(this, 0, 0)), typeof t == "string" && (t = parseFloat(t)), typeof t == "number" ? this._scale.set(t) : this._scale.copyFrom(t);
    }
    get origin() {
      return this._origin === Ci && (this._origin = new de(this, 0, 0)), this._origin;
    }
    set origin(t) {
      this._origin === Ci && (this._origin = new de(this, 0, 0), this._pivot !== vi && Jt("Setting both a pivot and origin on a Container is not recommended. This can lead to unexpected behavior if not handled carefully.")), typeof t == "number" ? this._origin.set(t) : this._origin.copyFrom(t);
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
      t !== this.localAlpha && (this.localAlpha = t, this._updateFlags |= Qa, this._onUpdate());
    }
    get alpha() {
      return this.localAlpha;
    }
    set tint(t) {
      const s = Yt.shared.setValue(t ?? 16777215).toBgrNumber();
      s !== this.localColor && (this.localColor = s, this._updateFlags |= Qa, this._onUpdate());
    }
    get tint() {
      return qs(this.localColor);
    }
    set blendMode(t) {
      this.localBlendMode !== t && (this.parentRenderGroup && (this.parentRenderGroup.structureDidChange = true), this._updateFlags |= vp, this.localBlendMode = t, this._onUpdate());
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
  Gt.mixin(Ht, rp, dp, xp, mp, fp, lp, cp, gp, tp, sp, up, op);
  class hr extends Ht {
    constructor(t) {
      super(t), this.canBundle = true, this.allowChildren = false, this._roundPixels = 0, this._lastUsed = -1, this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this._bounds = new Me(0, 1, 0, 0), this._boundsDirty = true, this.autoGarbageCollect = t.autoGarbageCollect ?? true;
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
  Ne = class extends hr {
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
      }, this._anchor = new de({
        _onUpdate: () => {
          this.onViewUpdate();
        }
      }), s ? this.anchor = s : e.defaultAnchor && (this.anchor = e.defaultAnchor), this.texture = e, this.allowChildren = false, this.roundPixels = i ?? false, r !== void 0 && (this.width = r), o !== void 0 && (this.height = o);
    }
    static from(t, e = false) {
      return t instanceof Tt ? new Ne(t) : new Ne(Tt.from(t, e));
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
      return sh(this._visualBounds, this._anchor, this._texture), this._visualBounds;
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
  const Cp = new Me();
  function ph(n, t, e) {
    const s = Cp;
    n.measurable = true, ah(n, e, s), t.addBoundsMask(s), n.measurable = false;
  }
  function fh(n, t, e) {
    const s = fn.get();
    n.measurable = true;
    const i = ye.get().identity(), r = mh(n, e, i);
    hh(n, s, r), n.measurable = false, t.addBoundsMask(s), ye.return(i), fn.return(s);
  }
  function mh(n, t, e) {
    return n ? (n !== t && (mh(n.parent, t, e), n.updateLocalTransform(), e.append(n.localTransform)), e) : (Jt("Mask bounds, renderable is not inside the root container"), e);
  }
  class gh {
    constructor(t) {
      this.priority = 0, this.inverse = false, this.pipe = "alphaMask", (t == null ? void 0 : t.mask) && this.init(t.mask);
    }
    init(t) {
      this.mask = t, this.renderMaskToTexture = !(t instanceof Ne), this.mask.renderable = this.renderMaskToTexture, this.mask.includeInBuild = !this.renderMaskToTexture, this.mask.measurable = false;
    }
    reset() {
      this.mask !== null && (this.mask.measurable = true, this.mask = null);
    }
    addBounds(t, e) {
      this.inverse || ph(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      fh(this.mask, t, e);
    }
    containsPoint(t, e) {
      const s = this.mask;
      return e(s, t);
    }
    destroy() {
      this.reset();
    }
    static test(t) {
      return t instanceof Ne;
    }
  }
  gh.extension = lt.MaskEffect;
  class yh {
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
  yh.extension = lt.MaskEffect;
  class xh {
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
      ph(this.mask, t, e);
    }
    addLocalBounds(t, e) {
      fh(this.mask, t, e);
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
  xh.extension = lt.MaskEffect;
  const Sp = {
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
  let tl = Sp;
  Rt = {
    get() {
      return tl;
    },
    set(n) {
      tl = n;
    }
  };
  bh = class extends Ie {
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
  bh.extension = lt.TextureSource;
  Cs = class extends Ie {
    constructor(t) {
      super(t), this.uploadMethodId = "image", this.autoGarbageCollect = true;
    }
    static test(t) {
      return globalThis.HTMLImageElement && t instanceof HTMLImageElement || typeof ImageBitmap < "u" && t instanceof ImageBitmap || globalThis.VideoFrame && t instanceof VideoFrame;
    }
  };
  Cs.extension = lt.TextureSource;
  si = ((n) => (n[n.INTERACTION = 50] = "INTERACTION", n[n.HIGH = 25] = "HIGH", n[n.NORMAL = 0] = "NORMAL", n[n.LOW = -25] = "LOW", n[n.UTILITY = -50] = "UTILITY", n))(si || {});
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
  const _h = class Se {
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
    add(t, e, s = si.NORMAL) {
      return this._addListener(new Lr(t, e, s));
    }
    addOnce(t, e, s = si.NORMAL) {
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
  _h.targetFPMS = 0.06;
  let $r;
  Yn = _h;
  async function wh() {
    return $r ?? ($r = (async () => {
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
    })()), $r;
  }
  const dr = class vh extends Ie {
    constructor(t) {
      super(t), this.isReady = false, this.uploadMethodId = "video", t = {
        ...vh.defaultOptions,
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
      return (t.readyState === t.HAVE_ENOUGH_DATA || t.readyState === t.HAVE_FUTURE_DATA) && t.width && t.height && (t.complete = true), t.addEventListener("play", this._onPlayStart), t.addEventListener("pause", this._onPlayStop), t.addEventListener("seeked", this._onSeeked), this._isSourceReady() ? this._mediaReady() : (e.preload || t.addEventListener("canplay", this._onCanPlay), t.addEventListener("canplaythrough", this._onCanPlayThrough), t.addEventListener("error", this._onError, true)), this.alphaMode = await wh(), this._load = new Promise((s, i) => {
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
  dr.extension = lt.TextureSource;
  dr.defaultOptions = {
    ...Ie.defaultOptions,
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
  const je = (n, t, e = false) => (Array.isArray(n) || (n = [
    n
  ]), t ? n.map((s) => typeof s == "string" || e ? t(s) : s) : n);
  class Tp {
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
      const s = je(t);
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
  ne = new Tp();
  bo = [];
  Gt.handleByList(lt.TextureSource, bo);
  function Ch(n = {}) {
    const t = n && n.resource, e = t ? n.resource : n, s = t ? n : {
      resource: n
    };
    for (let i = 0; i < bo.length; i++) {
      const r = bo[i];
      if (r.test(e)) return new r(s);
    }
    throw new Error(`Could not find a source type for resource: ${s.resource}`);
  }
  function Ep(n = {}, t = false) {
    const e = n && n.resource, s = e ? n.resource : n, i = e ? n : {
      resource: n
    };
    if (!t && ne.has(s)) return ne.get(s);
    const r = new Tt({
      source: Ch(i)
    });
    return r.on("destroy", () => {
      ne.has(s) && ne.remove(s);
    }), t || ne.set(s, r), r;
  }
  function Ap(n, t = false) {
    return typeof n == "string" ? ne.get(n) : n instanceof Ie ? new Tt({
      source: n
    }) : Ep(n, t);
  }
  Tt.from = Ap;
  Ie.from = Ch;
  Gt.add(gh, yh, xh, Ks, Cs, bh, Jo);
  var Ln = ((n) => (n[n.Low = 0] = "Low", n[n.Normal = 1] = "Normal", n[n.High = 2] = "High", n))(Ln || {});
  function De(n) {
    if (typeof n != "string") throw new TypeError(`Path must be a string. Received ${JSON.stringify(n)}`);
  }
  function $s(n) {
    return n.split("?")[0].split("#")[0];
  }
  function kp(n) {
    return n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  function Mp(n, t, e) {
    return n.replace(new RegExp(kp(t), "g"), e);
  }
  function Pp(n, t) {
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
  const ke = {
    toPosix(n) {
      return Mp(n, "\\", "/");
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
      De(n), n = this.toPosix(n);
      const t = /^file:\/\/\//.exec(n);
      if (t) return t[0];
      const e = /^[^/:]+:\/{0,2}/.exec(n);
      return e ? e[0] : "";
    },
    toAbsolute(n, t, e) {
      if (De(n), this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      const s = $s(this.toPosix(t ?? Rt.get().getBaseUrl())), i = $s(this.toPosix(e ?? this.rootname(s)));
      return n = this.toPosix(n), n.startsWith("/") ? ke.join(i, n.slice(1)) : this.isAbsolute(n) ? n : this.join(s, n);
    },
    normalize(n) {
      if (De(n), n.length === 0) return ".";
      if (this.isDataUrl(n) || this.isBlobUrl(n)) return n;
      n = this.toPosix(n);
      let t = "";
      const e = n.startsWith("/");
      this.hasProtocol(n) && (t = this.rootname(n), n = n.slice(t.length));
      const s = n.endsWith("/");
      return n = Pp(n), n.length > 0 && s && (n += "/"), e ? `/${n}` : t + n;
    },
    isAbsolute(n) {
      return De(n), n = this.toPosix(n), this.hasProtocol(n) ? true : n.startsWith("/");
    },
    join(...n) {
      if (n.length === 0) return ".";
      let t;
      for (let e = 0; e < n.length; ++e) {
        const s = n[e];
        if (De(s), s.length > 0) if (t === void 0) t = s;
        else {
          const i = n[e - 1] ?? "";
          this.joinExtensions.includes(this.extname(i).toLowerCase()) ? t += `/../${s}` : t += `/${s}`;
        }
      }
      return t === void 0 ? "." : this.normalize(t);
    },
    dirname(n) {
      if (De(n), n.length === 0) return ".";
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
      De(n), n = this.toPosix(n);
      let t = "";
      if (n.startsWith("/") ? t = "/" : t = this.getProtocol(n), this.isUrl(n)) {
        const e = n.indexOf("/", t.length);
        e !== -1 ? t = n.slice(0, e) : t = n, t.endsWith("/") || (t += "/");
      }
      return t;
    },
    basename(n, t) {
      De(n), t && De(t), n = $s(this.toPosix(n));
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
      De(n), n = $s(this.toPosix(n));
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
      De(n);
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
  function Sh(n, t, e, s, i) {
    const r = t[e];
    for (let o = 0; o < r.length; o++) {
      const a = r[o];
      e < t.length - 1 ? Sh(n.replace(s[e], a), t, e + 1, s, i) : i.push(n.replace(s[e], a));
    }
  }
  function Ip(n) {
    const t = /\{(.*?)\}/g, e = n.match(t), s = [];
    if (e) {
      const i = [];
      e.forEach((r) => {
        const o = r.substring(1, r.length - 1).split(",");
        i.push(o);
      }), Sh(n, i, 0, e, s);
    } else s.push(n);
    return s;
  }
  const Ji = (n) => !Array.isArray(n);
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
      return je(e || s, (r) => typeof r == "string" ? r : Array.isArray(r) ? r.map((o) => (o == null ? void 0 : o.src) ?? o) : (r == null ? void 0 : r.src) ? r.src : r, true);
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
      }, je(e).forEach((r) => {
        const { src: o } = r;
        let { data: a, format: l, loadParser: c, parser: h } = r;
        const d = je(o).map((g) => typeof g == "string" ? Ip(g) : Array.isArray(g) ? g : [
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
      const e = Ji(t);
      t = je(t);
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
      const e = Ji(t);
      t = je(t);
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
      return (this._basePath || this._rootPath) && (t.src = ke.toAbsolute(t.src, this._basePath, this._rootPath)), t.alias = s ?? t.alias ?? [
        t.src
      ], t.src = this._appendDefaultSearchParams(t.src), t.data = {
        ...i || {},
        ...t.data
      }, t.loadParser = r ?? t.loadParser, t.parser = o ?? t.parser, t.format = a ?? t.format ?? Rp(t.src), l !== void 0 && (t.progressSize = l), t;
    }
  }
  As.RETINA_PREFIX = /@([0-9\.]+)x/;
  function Rp(n) {
    return n.split(".").pop().split("?").shift().split("#").shift();
  }
  const _o = (n, t) => {
    const e = t.split("?")[1];
    return e && (n += `?${e}`), n;
  }, Th = class js {
    constructor(t, e) {
      this.linkedSheets = [];
      let s = t;
      (t == null ? void 0 : t.source) instanceof Ie && (s = {
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
  Th.BATCH_SIZE = 1e3;
  let el = Th;
  const Lp = [
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
  function Eh(n, t, e) {
    const s = {};
    if (n.forEach((i) => {
      s[i] = t;
    }), Object.keys(t.textures).forEach((i) => {
      s[`${t.cachePrefix}${i}`] = t.textures[i];
    }), !e) {
      const i = ke.dirname(n[0]);
      t.linkedSheets.forEach((r, o) => {
        const a = Eh([
          `${i}/${t.data.meta.related_multi_packs[o]}`
        ], r, true);
        Object.assign(s, a);
      });
    }
    return s;
  }
  const $p = {
    extension: lt.Asset,
    cache: {
      test: (n) => n instanceof el,
      getCacheableAssets: (n, t) => Eh(n, t, false)
    },
    resolver: {
      extension: {
        type: lt.ResolveParser,
        name: "resolveSpritesheet"
      },
      test: (n) => {
        const e = n.split("?")[0].split("."), s = e.pop(), i = e.pop();
        return s === "json" && Lp.includes(i);
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
        type: lt.LoadParser,
        priority: Ln.Normal,
        name: "spritesheetLoader"
      },
      async testParse(n, t) {
        return ke.extname(t.src).toLowerCase() === ".json" && !!n.frames;
      },
      async parse(n, t, e) {
        var _a2, _b2;
        const { texture: s, imageFilename: i, textureOptions: r, cachePrefix: o } = (t == null ? void 0 : t.data) ?? {};
        let a = ke.dirname(t.src);
        a && a.lastIndexOf("/") !== a.length - 1 && (a += "/");
        let l;
        if (s instanceof Tt) l = s;
        else {
          const d = _o(a + (i ?? n.meta.image), t.src);
          l = (await e.load([
            {
              src: d,
              data: r
            }
          ]))[d];
        }
        const c = new el({
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
  Gt.add($p);
  const Br = /* @__PURE__ */ Object.create(null), nl = /* @__PURE__ */ Object.create(null);
  ta = function(n, t) {
    let e = nl[n];
    return e === void 0 && (Br[t] === void 0 && (Br[t] = 1), nl[n] = e = Br[t]++), e;
  };
  let Si;
  function Ah() {
    return (!Si || (Si == null ? void 0 : Si.isContextLost())) && (Si = Rt.get().createCanvas().getContext("webgl", {})), Si;
  }
  let Ti;
  function Bp() {
    if (!Ti) {
      Ti = "mediump";
      const n = Ah();
      n && n.getShaderPrecisionFormat && (Ti = n.getShaderPrecisionFormat(n.FRAGMENT_SHADER, n.HIGH_FLOAT).precision ? "highp" : "mediump");
    }
    return Ti;
  }
  function Op(n, t, e) {
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
  function Np(n, t, e) {
    const s = e ? t.maxSupportedFragmentPrecision : t.maxSupportedVertexPrecision;
    if (n.substring(0, 9) !== "precision") {
      let i = e ? t.requestedFragmentPrecision : t.requestedVertexPrecision;
      return i === "highp" && s !== "highp" && (i = "mediump"), `precision ${i} float;
${n}`;
    } else if (s !== "highp" && n.substring(0, 15) === "precision highp") return n.replace("precision highp", "precision mediump");
    return n;
  }
  function Fp(n, t) {
    return t ? `#version 300 es
${n}` : n;
  }
  const Wp = {}, Gp = {};
  function zp(n, { name: t = "pixi-program" }, e = true) {
    t = t.replace(/\s+/g, "-"), t += e ? "-fragment" : "-vertex";
    const s = e ? Wp : Gp;
    return s[t] ? (s[t]++, t += `-${s[t]}`) : s[t] = 1, n.indexOf("#define SHADER_NAME") !== -1 ? n : `${`#define SHADER_NAME ${t}`}
${n}`;
  }
  function Dp(n, t) {
    return t ? n.replace("#version 300 es", "") : n;
  }
  const Or = {
    stripVersion: Dp,
    ensurePrecision: Np,
    addProgramDefines: Op,
    setProgramName: zp,
    insertVersion: Fp
  }, Bs = /* @__PURE__ */ Object.create(null), kh = class wo {
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
          maxSupportedFragmentPrecision: Bp()
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
      }), this.fragment = i, this.vertex = r, this.transformFeedbackVaryings = t.transformFeedbackVaryings, this._key = ta(`${this.vertex}:${this.fragment}`, "gl-program");
    }
    destroy() {
      this.fragment = null, this.vertex = null, this._attributeData = null, this._uniformData = null, this._uniformBlockData = null, this.transformFeedbackVaryings = null, Bs[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex}:${t.fragment}`;
      return Bs[e] || (Bs[e] = new wo(t), Bs[e]._cacheKey = e), Bs[e];
    }
  };
  kh.defaultOptions = {
    preferredVertexPrecision: "highp",
    preferredFragmentPrecision: "mediump"
  };
  ea = kh;
  const sl = {
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
  Zi = function(n) {
    return sl[n] ?? sl.float32;
  };
  const Hp = {
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
  }, il = /@location\((\d+)\)\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_<>]+)(?:,|\s|\)|$)/g;
  function rl(n, t) {
    let e;
    for (; (e = il.exec(n)) !== null; ) {
      const s = Hp[e[3]] ?? "float32";
      t[e[2]] = {
        location: parseInt(e[1], 10),
        format: s,
        stride: Zi(s).stride,
        offset: 0,
        instance: false,
        start: 0
      };
    }
    il.lastIndex = 0;
  }
  function Up(n) {
    return n.replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  }
  function jp({ source: n, entryPoint: t }) {
    const e = {}, s = Up(n), i = s.indexOf(`fn ${t}(`);
    if (i === -1) return e;
    const r = s.indexOf("->", i);
    if (r === -1) return e;
    const o = s.substring(i, r);
    if (rl(o, e), Object.keys(e).length === 0) {
      const a = o.match(/\(\s*\w+\s*:\s*(\w+)/);
      if (a) {
        const l = a[1], c = new RegExp(`struct\\s+${l}\\s*\\{([^}]+)\\}`, "s"), h = s.match(c);
        h && rl(h[1], e);
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
  var jn = ((n) => (n[n.VERTEX = 1] = "VERTEX", n[n.FRAGMENT = 2] = "FRAGMENT", n[n.COMPUTE = 4] = "COMPUTE", n))(jn || {});
  function Vp({ groups: n }) {
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
  function Yp({ groups: n }) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e];
      t[s.group] || (t[s.group] = {}), t[s.group][s.name] = s.binding;
    }
    return t;
  }
  function Xp(n, t) {
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
  mi = class {
    constructor(t) {
      var _a2, _b2;
      this._layoutKey = 0, this._attributeLocationsKey = 0;
      const { fragment: e, vertex: s, layout: i, gpuLayout: r, name: o } = t;
      if (this.name = o, this.fragment = e, this.vertex = s, e.source === s.source) {
        const a = Nr(e.source);
        this.structsAndGroups = a;
      } else {
        const a = Nr(s.source), l = Nr(e.source);
        this.structsAndGroups = Xp(a, l);
      }
      this.layout = i ?? Yp(this.structsAndGroups), this.gpuLayout = r ?? Vp(this.structsAndGroups), this.autoAssignGlobalUniforms = ((_a2 = this.layout[0]) == null ? void 0 : _a2.globalUniforms) !== void 0, this.autoAssignLocalUniforms = ((_b2 = this.layout[1]) == null ? void 0 : _b2.localUniforms) !== void 0, this._generateProgramKey();
    }
    _generateProgramKey() {
      const { vertex: t, fragment: e } = this, s = t.source + e.source + t.entryPoint + e.entryPoint;
      this._layoutKey = ta(s, "program");
    }
    get attributeData() {
      return this._attributeData ?? (this._attributeData = jp(this.vertex)), this._attributeData;
    }
    destroy() {
      this.gpuLayout = null, this.layout = null, this.structsAndGroups = null, this.fragment = null, this.vertex = null, Os[this._cacheKey] = null;
    }
    static from(t) {
      const e = `${t.vertex.source}:${t.fragment.source}:${t.fragment.entryPoint}:${t.vertex.entryPoint}`;
      return Os[e] || (Os[e] = new mi(t), Os[e]._cacheKey = e), Os[e];
    }
  };
  const Mh = [
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
  ], qp = Mh.reduce((n, t) => (n[t] = true, n), {});
  function Kp(n, t) {
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
  const Ph = class Ih {
    constructor(t, e) {
      this._touched = 0, this.uid = ee("uniform"), this._resourceType = "uniformGroup", this._resourceId = ee("resource"), this.isUniformGroup = true, this._dirtyId = 0, this.destroyed = false, e = {
        ...Ih.defaultOptions,
        ...e
      }, this.uniformStructures = t;
      const s = {};
      for (const i in t) {
        const r = t[i];
        if (r.name = i, r.size = r.size ?? 1, !qp[r.type]) {
          const o = r.type.match(/^array<(\w+(?:<\w+>)?),\s*(\d+)>$/);
          if (o) {
            const [, a, l] = o;
            throw new Error(`Uniform type ${r.type} is not supported. Use type: '${a}', size: ${l} instead.`);
          }
          throw new Error(`Uniform type ${r.type} is not supported. Supported uniform types are: ${Mh.join(", ")}`);
        }
        r.value ?? (r.value = Kp(r.type, r.size)), s[i] = r.value;
      }
      this.uniforms = s, this._dirtyId = 1, this.ubo = e.ubo, this.isStatic = e.isStatic, this._signature = ta(Object.keys(s).map((i) => `${i}-${t[i].type}`).join("-"), "uniform-group");
    }
    update() {
      this._dirtyId++;
    }
  };
  Ph.defaultOptions = {
    ubo: false,
    isStatic: false
  };
  na = Ph;
  Yi = class {
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
        for (const h in r) l[h] || (i[99] || (i[99] = new Yi(), this._ownedBindGroups.push(i[99])), l[h] = {
          group: 99,
          binding: c,
          name: h
        }, a[99] = a[99] || {}, a[99][c] = h, c++);
        for (const h in r) {
          const d = h;
          let u = r[h];
          !u.source && !u._resourceType && (u = new na(u));
          const p = l[d];
          p && (i[p.group] || (i[p.group] = new Yi(), this._ownedBindGroups.push(i[p.group])), i[p.group].setResource(u, p.binding));
        }
      }
      this.groups = i, this._uniformBindMap = a, this.resources = this._buildResourceAccessor(i, l);
    }
    addResource(t, e, s) {
      var i, r;
      (i = this._uniformBindMap)[e] || (i[e] = {}), (r = this._uniformBindMap[e])[s] || (r[s] = t), this.groups[e] || (this.groups[e] = new Yi(), this._ownedBindGroups.push(this.groups[e]));
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
      return e && (r = mi.from(e)), s && (o = ea.from(s)), new ur({
        gpuProgram: r,
        glProgram: o,
        ...i
      });
    }
  };
  const Jp = {
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
  }, Fr = 0, Wr = 1, Gr = 2, zr = 3, Dr = 4, Hr = 5, Co = class Rh {
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
      this.blend = t !== "none", this._blendMode = t, this._blendModeId = Jp[t] || 0;
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
      const t = new Rh();
      return t.depthTest = false, t.blend = true, t;
    }
  };
  Co.default2d = Co.for2d();
  Qi = Co;
  const So = [];
  Gt.handleByNamedList(lt.Environment, So);
  async function Zp(n) {
    if (!n) for (let t = 0; t < So.length; t++) {
      const e = So[t];
      if (e.value.test()) {
        await e.value.load();
        return;
      }
    }
  }
  let Ns;
  Qp = function() {
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
  function ol(n, t, e = 2) {
    const s = t && t.length, i = s ? t[0] * e : n.length;
    let r = Lh(n, 0, i, e, true);
    const o = [];
    if (!r || r.next === r.prev) return o;
    let a, l, c;
    if (s && (r = rf(n, t, r, e)), n.length > 80 * e) {
      a = n[0], l = n[1];
      let h = a, d = l;
      for (let u = e; u < i; u += e) {
        const p = n[u], f = n[u + 1];
        p < a && (a = p), f < l && (l = f), p > h && (h = p), f > d && (d = f);
      }
      c = Math.max(h - a, d - l), c = c !== 0 ? 32767 / c : 0;
    }
    return ii(r, o, e, a, l, c, 0), o;
  }
  function Lh(n, t, e, s, i) {
    let r;
    if (i === gf(n, t, e, s) > 0) for (let o = t; o < e; o += s) r = al(o / s | 0, n[o], n[o + 1], r);
    else for (let o = e - s; o >= t; o -= s) r = al(o / s | 0, n[o], n[o + 1], r);
    return r && Ss(r, r.next) && (oi(r), r = r.next), r;
  }
  function Jn(n, t) {
    if (!n) return n;
    t || (t = n);
    let e = n, s;
    do
      if (s = false, !e.steiner && (Ss(e, e.next) || Zt(e.prev, e, e.next) === 0)) {
        if (oi(e), e = t = e.prev, e === e.next) break;
        s = true;
      } else e = e.next;
    while (s || e !== t);
    return t;
  }
  function ii(n, t, e, s, i, r, o) {
    if (!n) return;
    !o && r && hf(n, s, i, r);
    let a = n;
    for (; n.prev !== n.next; ) {
      const l = n.prev, c = n.next;
      if (r ? ef(n, s, i, r) : tf(n)) {
        t.push(l.i, n.i, c.i), oi(n), n = c.next, a = c.next;
        continue;
      }
      if (n = c, n === a) {
        o ? o === 1 ? (n = nf(Jn(n), t), ii(n, t, e, s, i, r, 2)) : o === 2 && sf(n, t, e, s, i, r) : ii(Jn(n), t, e, s, i, r, 1);
        break;
      }
    }
  }
  function tf(n) {
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
  function ef(n, t, e, s) {
    const i = n.prev, r = n, o = n.next;
    if (Zt(i, r, o) >= 0) return false;
    const a = i.x, l = r.x, c = o.x, h = i.y, d = r.y, u = o.y, p = Math.min(a, l, c), f = Math.min(h, d, u), g = Math.max(a, l, c), m = Math.max(h, d, u), y = To(p, f, t, e, s), w = To(g, m, t, e, s);
    let x = n.prevZ, _ = n.nextZ;
    for (; x && x.z >= y && _ && _.z <= w; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && Vs(a, h, l, d, c, u, x.x, x.y) && Zt(x.prev, x, x.next) >= 0 || (x = x.prevZ, _.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && Vs(a, h, l, d, c, u, _.x, _.y) && Zt(_.prev, _, _.next) >= 0)) return false;
      _ = _.nextZ;
    }
    for (; x && x.z >= y; ) {
      if (x.x >= p && x.x <= g && x.y >= f && x.y <= m && x !== i && x !== o && Vs(a, h, l, d, c, u, x.x, x.y) && Zt(x.prev, x, x.next) >= 0) return false;
      x = x.prevZ;
    }
    for (; _ && _.z <= w; ) {
      if (_.x >= p && _.x <= g && _.y >= f && _.y <= m && _ !== i && _ !== o && Vs(a, h, l, d, c, u, _.x, _.y) && Zt(_.prev, _, _.next) >= 0) return false;
      _ = _.nextZ;
    }
    return true;
  }
  function nf(n, t) {
    let e = n;
    do {
      const s = e.prev, i = e.next.next;
      !Ss(s, i) && Bh(s, e, e.next, i) && ri(s, i) && ri(i, s) && (t.push(s.i, e.i, i.i), oi(e), oi(e.next), e = n = i), e = e.next;
    } while (e !== n);
    return Jn(e);
  }
  function sf(n, t, e, s, i, r) {
    let o = n;
    do {
      let a = o.next.next;
      for (; a !== o.prev; ) {
        if (o.i !== a.i && pf(o, a)) {
          let l = Oh(o, a);
          o = Jn(o, o.next), l = Jn(l, l.next), ii(o, t, e, s, i, r, 0), ii(l, t, e, s, i, r, 0);
          return;
        }
        a = a.next;
      }
      o = o.next;
    } while (o !== n);
  }
  function rf(n, t, e, s) {
    const i = [];
    for (let r = 0, o = t.length; r < o; r++) {
      const a = t[r] * s, l = r < o - 1 ? t[r + 1] * s : n.length, c = Lh(n, a, l, s, false);
      c === c.next && (c.steiner = true), i.push(uf(c));
    }
    i.sort(of);
    for (let r = 0; r < i.length; r++) e = af(i[r], e);
    return e;
  }
  function of(n, t) {
    let e = n.x - t.x;
    if (e === 0 && (e = n.y - t.y, e === 0)) {
      const s = (n.next.y - n.y) / (n.next.x - n.x), i = (t.next.y - t.y) / (t.next.x - t.x);
      e = s - i;
    }
    return e;
  }
  function af(n, t) {
    const e = lf(n, t);
    if (!e) return t;
    const s = Oh(e, n);
    return Jn(s, s.next), Jn(e, e.next);
  }
  function lf(n, t) {
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
      if (s >= e.x && e.x >= l && s !== e.x && $h(i < c ? s : r, i, l, c, i < c ? r : s, i, e.x, e.y)) {
        const d = Math.abs(i - e.y) / (s - e.x);
        ri(e, n) && (d < h || d === h && (e.x > o.x || e.x === o.x && cf(o, e))) && (o = e, h = d);
      }
      e = e.next;
    } while (e !== a);
    return o;
  }
  function cf(n, t) {
    return Zt(n.prev, n, t.prev) < 0 && Zt(t.next, n, n.next) < 0;
  }
  function hf(n, t, e, s) {
    let i = n;
    do
      i.z === 0 && (i.z = To(i.x, i.y, t, e, s)), i.prevZ = i.prev, i.nextZ = i.next, i = i.next;
    while (i !== n);
    i.prevZ.nextZ = null, i.prevZ = null, df(i);
  }
  function df(n) {
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
  function uf(n) {
    let t = n, e = n;
    do
      (t.x < e.x || t.x === e.x && t.y < e.y) && (e = t), t = t.next;
    while (t !== n);
    return e;
  }
  function $h(n, t, e, s, i, r, o, a) {
    return (i - o) * (t - a) >= (n - o) * (r - a) && (n - o) * (s - a) >= (e - o) * (t - a) && (e - o) * (r - a) >= (i - o) * (s - a);
  }
  function Vs(n, t, e, s, i, r, o, a) {
    return !(n === o && t === a) && $h(n, t, e, s, i, r, o, a);
  }
  function pf(n, t) {
    return n.next.i !== t.i && n.prev.i !== t.i && !ff(n, t) && (ri(n, t) && ri(t, n) && mf(n, t) && (Zt(n.prev, n, t.prev) || Zt(n, t.prev, t)) || Ss(n, t) && Zt(n.prev, n, n.next) > 0 && Zt(t.prev, t, t.next) > 0);
  }
  function Zt(n, t, e) {
    return (t.y - n.y) * (e.x - t.x) - (t.x - n.x) * (e.y - t.y);
  }
  function Ss(n, t) {
    return n.x === t.x && n.y === t.y;
  }
  function Bh(n, t, e, s) {
    const i = Ai(Zt(n, t, e)), r = Ai(Zt(n, t, s)), o = Ai(Zt(e, s, n)), a = Ai(Zt(e, s, t));
    return !!(i !== r && o !== a || i === 0 && Ei(n, e, t) || r === 0 && Ei(n, s, t) || o === 0 && Ei(e, n, s) || a === 0 && Ei(e, t, s));
  }
  function Ei(n, t, e) {
    return t.x <= Math.max(n.x, e.x) && t.x >= Math.min(n.x, e.x) && t.y <= Math.max(n.y, e.y) && t.y >= Math.min(n.y, e.y);
  }
  function Ai(n) {
    return n > 0 ? 1 : n < 0 ? -1 : 0;
  }
  function ff(n, t) {
    let e = n;
    do {
      if (e.i !== n.i && e.next.i !== n.i && e.i !== t.i && e.next.i !== t.i && Bh(e, e.next, n, t)) return true;
      e = e.next;
    } while (e !== n);
    return false;
  }
  function ri(n, t) {
    return Zt(n.prev, n, n.next) < 0 ? Zt(n, t, n.next) >= 0 && Zt(n, n.prev, t) >= 0 : Zt(n, t, n.prev) < 0 || Zt(n, n.next, t) < 0;
  }
  function mf(n, t) {
    let e = n, s = false;
    const i = (n.x + t.x) / 2, r = (n.y + t.y) / 2;
    do
      e.y > r != e.next.y > r && e.next.y !== e.y && i < (e.next.x - e.x) * (r - e.y) / (e.next.y - e.y) + e.x && (s = !s), e = e.next;
    while (e !== n);
    return s;
  }
  function Oh(n, t) {
    const e = Eo(n.i, n.x, n.y), s = Eo(t.i, t.x, t.y), i = n.next, r = t.prev;
    return n.next = t, t.prev = n, e.next = i, i.prev = e, s.next = e, e.prev = s, r.next = s, s.prev = r, s;
  }
  function al(n, t, e, s) {
    const i = Eo(n, t, e);
    return s ? (i.next = s.next, i.prev = s, s.next.prev = i, s.next = i) : (i.prev = i, i.next = i), i;
  }
  function oi(n) {
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
  function gf(n, t, e, s) {
    let i = 0;
    for (let r = t, o = e - s; r < e; r += s) i += (n[o] - n[r]) * (n[r + 1] + n[o + 1]), o = r;
    return i;
  }
  const yf = ol.default || ol;
  Nh = ((n) => (n[n.NONE = 0] = "NONE", n[n.COLOR = 16384] = "COLOR", n[n.STENCIL = 1024] = "STENCIL", n[n.DEPTH = 256] = "DEPTH", n[n.COLOR_DEPTH = 16640] = "COLOR_DEPTH", n[n.COLOR_STENCIL = 17408] = "COLOR_STENCIL", n[n.DEPTH_STENCIL = 1280] = "DEPTH_STENCIL", n[n.ALL = 17664] = "ALL", n))(Nh || {});
  xf = class {
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
  const bf = [
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
  ], Fh = class Wh extends rn {
    constructor(t) {
      super(), this.tick = 0, this.uid = ee("renderer"), this.runners = /* @__PURE__ */ Object.create(null), this.renderPipes = /* @__PURE__ */ Object.create(null), this._initOptions = {}, this._systemsHash = /* @__PURE__ */ Object.create(null), this.type = t.type, this.name = t.name, this.config = t;
      const e = [
        ...bf,
        ...this.config.runners ?? []
      ];
      this._addRunners(...e), this._unsafeEvalCheck();
    }
    async init(t = {}) {
      const e = t.skipExtensionImports === true ? true : t.manageImports === false;
      await Zp(e), this._addSystems(this.config.systems), this._addPipes(this.config.renderPipes, this.config.renderPipeAdaptors);
      for (const s in this._systemsHash) t = {
        ...this._systemsHash[s].constructor.defaultOptions,
        ...t
      };
      t = {
        ...Wh.defaultOptions,
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
      }, e && (It(te, "passing a second argument is deprecated, please use render options instead"), s.target = e.renderTexture)), s.target || (s.target = this.view.renderTarget), s.target === this.view.renderTarget && (this._lastObjectRendered = s.container, s.clearColor ?? (s.clearColor = this.background.colorRgba), s.clear ?? (s.clear = this.background.clearBeforeRender)), s.clearColor) {
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
      t.target || (t.target = e.renderTarget.renderTarget), t.clearColor || (t.clearColor = this.background.colorRgba), t.clear ?? (t.clear = Nh.ALL);
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
        this.runners[e] = new xf(e);
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
      this.runners.destroy.items.reverse(), this.runners.destroy.emit(t), (t === true || typeof t == "object" && t.releaseGlobalResources) && fi.release(), Object.values(this.runners).forEach((e) => {
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
      if (!Qp()) throw new Error("Current environment does not allow unsafe-eval, please use pixi.js/unsafe-eval module to enable support.");
    }
    resetState() {
      this.runners.resetState.emit();
    }
  };
  Fh.defaultOptions = {
    resolution: 1,
    failIfMajorPerformanceCaveat: false,
    roundPixels: false
  };
  let ki;
  Gh = Fh;
  function _f(n) {
    return ki !== void 0 || (ki = (() => {
      var _a2;
      const t = {
        stencil: true,
        failIfMajorPerformanceCaveat: n ?? Gh.defaultOptions.failIfMajorPerformanceCaveat
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
    })()), ki;
  }
  let Mi;
  async function wf(n = {}) {
    return Mi !== void 0 || (Mi = await (async () => {
      const t = Rt.get().getNavigator().gpu;
      if (!t) return false;
      try {
        return await (await t.requestAdapter(n)).requestDevice(), true;
      } catch {
        return false;
      }
    })()), Mi;
  }
  const ll = [
    "webgl",
    "webgpu",
    "canvas"
  ];
  async function vf(n) {
    let t = [];
    n.preference ? (t.push(n.preference), ll.forEach((r) => {
      r !== n.preference && t.push(r);
    })) : t = ll.slice();
    let e, s = {};
    for (let r = 0; r < t.length; r++) {
      const o = t[r];
      if (o === "webgpu" && await wf()) {
        const { WebGPURenderer: a } = await Pn(async () => {
          const { WebGPURenderer: l } = await import("./WebGPURenderer-D8eqHiLd.js");
          return {
            WebGPURenderer: l
          };
        }, __vite__mapDeps([3,4,5,2]));
        e = a, s = {
          ...n,
          ...n.webgpu
        };
        break;
      } else if (o === "webgl" && _f(n.failIfMajorPerformanceCaveat ?? Gh.defaultOptions.failIfMajorPerformanceCaveat)) {
        const { WebGLRenderer: a } = await Pn(async () => {
          const { WebGLRenderer: l } = await import("./WebGLRenderer-DW_Huo9Y.js");
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
          const { CanvasRenderer: l } = await import("./CanvasRenderer-vsy2PsgG.js");
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
  zh = "8.17.1";
  class Dh {
    static init() {
      var _a2;
      (_a2 = globalThis.__PIXI_APP_INIT__) == null ? void 0 : _a2.call(globalThis, this, zh);
    }
    static destroy() {
    }
  }
  Dh.extension = lt.Application;
  Cf = class {
    constructor(t) {
      this._renderer = t;
    }
    init() {
      var _a2;
      (_a2 = globalThis.__PIXI_RENDERER_INIT__) == null ? void 0 : _a2.call(globalThis, this._renderer, zh);
    }
    destroy() {
      this._renderer = null;
    }
  };
  Cf.extension = {
    type: [
      lt.WebGLSystem,
      lt.WebGPUSystem
    ],
    name: "initHook",
    priority: -10
  };
  class Hh {
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
  Hh.extension = lt.Application;
  class Uh {
    static init(t) {
      t = Object.assign({
        autoStart: true,
        sharedTicker: false
      }, t), Object.defineProperty(this, "ticker", {
        configurable: true,
        set(e) {
          this._ticker && this._ticker.remove(this.render, this), this._ticker = e, e && e.add(this.render, this, si.LOW);
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
  Uh.extension = lt.Application;
  Gt.add(Hh);
  Gt.add(Uh);
  const jh = class Ao {
    constructor(...t) {
      this.stage = new Ht(), t[0] !== void 0 && It(te, "Application constructor options are deprecated, please use Application.init() instead.");
    }
    async init(t) {
      t = {
        ...t
      }, this.stage || (this.stage = new Ht()), this.renderer = await vf(t), Ao._plugins.forEach((e) => {
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
      return It(te, "Application.view is deprecated, please use Application.canvas instead."), this.renderer.canvas;
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
  jh._plugins = [];
  sa = jh;
  Gt.handleByList(lt.Application, sa._plugins);
  Gt.add(Dh);
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
        for (const g in p) {
          const m = p[g].split("="), y = m[0], w = m[1].replace(/"/gm, ""), x = parseFloat(w), _ = isNaN(x) ? w : x;
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
  }, cl = {
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
  }, hl = {
    test(n) {
      return typeof n == "string" && n.match(/<font(\s|>)/) ? cl.test(Rt.get().parseXML(n)) : false;
    },
    parse(n) {
      return cl.parse(Rt.get().parseXML(n));
    }
  }, Sf = [
    ".xml",
    ".fnt"
  ], Tf = {
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
  }, Ef = {
    extension: {
      type: lt.LoadParser,
      priority: Ln.Normal
    },
    name: "loadBitmapFont",
    id: "bitmap-font",
    test(n) {
      return Sf.includes(ke.extname(n).toLowerCase());
    },
    async testParse(n) {
      return Ur.test(n) || hl.test(n);
    },
    async parse(n, t, e) {
      const s = Ur.test(n) ? Ur.parse(n) : hl.parse(n), { src: i } = t, { pages: r } = s, o = [], a = s.distanceField ? {
        scaleMode: "linear",
        alphaMode: "premultiply-alpha-on-upload",
        autoGenerateMipmaps: false,
        resolution: 1
      } : {};
      for (let u = 0; u < r.length; ++u) {
        const p = r[u].file;
        let f = ke.join(ke.dirname(i), p);
        f = _o(f, i), o.push({
          src: f,
          data: a
        });
      }
      const [l, { BitmapFont: c }] = await Promise.all([
        e.load(o),
        Pn(() => import("./BitmapFont-Du4DQzVS.js"), [])
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
  class Af {
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
  const kf = {
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
  async function Vh(n) {
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
  const Mf = {
    extension: {
      type: lt.DetectionParser,
      priority: 1
    },
    test: async () => Vh("data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A="),
    add: async (n) => [
      ...n,
      "avif"
    ],
    remove: async (n) => n.filter((t) => t !== "avif")
  }, dl = [
    "png",
    "jpg",
    "jpeg"
  ], Pf = {
    extension: {
      type: lt.DetectionParser,
      priority: -1
    },
    test: () => Promise.resolve(true),
    add: async (n) => [
      ...n,
      ...dl
    ],
    remove: async (n) => n.filter((t) => !dl.includes(t))
  }, If = "WorkerGlobalScope" in globalThis && globalThis instanceof globalThis.WorkerGlobalScope;
  function pr(n) {
    return If ? false : document.createElement("video").canPlayType(n) !== "";
  }
  const Rf = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => pr("video/mp4"),
    add: async (n) => [
      ...n,
      "mp4",
      "m4v"
    ],
    remove: async (n) => n.filter((t) => t !== "mp4" && t !== "m4v")
  }, Lf = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => pr("video/ogg"),
    add: async (n) => [
      ...n,
      "ogv"
    ],
    remove: async (n) => n.filter((t) => t !== "ogv")
  }, $f = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => pr("video/webm"),
    add: async (n) => [
      ...n,
      "webm"
    ],
    remove: async (n) => n.filter((t) => t !== "webm")
  }, Bf = {
    extension: {
      type: lt.DetectionParser,
      priority: 0
    },
    test: async () => Vh("data:image/webp;base64,UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAAAfQ//73v/+BiOh/AAA="),
    add: async (n) => [
      ...n,
      "webp"
    ],
    remove: async (n) => n.filter((t) => t !== "webp")
  }, Yh = class Xi {
    constructor() {
      this.loadOptions = {
        ...Xi.defaultOptions
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
        ...Xi.defaultOptions,
        ...this.loadOptions,
        onProgress: e
      } : {
        ...Xi.defaultOptions,
        ...this.loadOptions,
        ...e || {}
      }, { onProgress: i, onError: r, strategy: o, retryCount: a, retryDelay: l } = s;
      let c = 0;
      const h = {}, d = Ji(t), u = je(t, (g) => ({
        alias: [
          g
        ],
        src: g,
        data: {}
      })), p = u.reduce((g, m) => g + (m.progressSize || 1), 0), f = u.map(async (g) => {
        const m = ke.toAbsolute(g.src);
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
      const s = je(t, (i) => ({
        alias: [
          i
        ],
        src: i
      })).map(async (i) => {
        var _a2, _b2;
        const r = ke.toAbsolute(i.src), o = this.promiseCache[r];
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
  Yh.defaultOptions = {
    onProgress: void 0,
    onError: void 0,
    strategy: "throw",
    retryCount: 3,
    retryDelay: 250
  };
  let Of = Yh;
  function ks(n, t) {
    if (Array.isArray(t)) {
      for (const e of t) if (n.startsWith(`data:${e}`)) return true;
      return false;
    }
    return n.startsWith(`data:${t}`);
  }
  function Ms(n, t) {
    const e = n.split("?")[0], s = ke.extname(e).toLowerCase();
    return Array.isArray(t) ? t.includes(s) : s === t;
  }
  const Nf = ".json", Ff = "application/json", Wf = {
    extension: {
      type: lt.LoadParser,
      priority: Ln.Low
    },
    name: "loadJson",
    id: "json",
    test(n) {
      return ks(n, Ff) || Ms(n, Nf);
    },
    async load(n) {
      return await (await Rt.get().fetch(n)).json();
    }
  }, Gf = ".txt", zf = "text/plain", Df = {
    name: "loadTxt",
    id: "text",
    extension: {
      type: lt.LoadParser,
      priority: Ln.Low,
      name: "loadTxt"
    },
    test(n) {
      return ks(n, zf) || Ms(n, Gf);
    },
    async load(n) {
      return await (await Rt.get().fetch(n)).text();
    }
  }, Hf = [
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
  ], Uf = [
    ".ttf",
    ".otf",
    ".woff",
    ".woff2"
  ], jf = [
    "font/ttf",
    "font/otf",
    "font/woff",
    "font/woff2"
  ], Vf = /^(--|-?[A-Z_])[0-9A-Z_-]*$/i;
  function Yf(n) {
    const t = ke.extname(n), i = ke.basename(n, t).replace(/(-|_)/g, " ").toLowerCase().split(" ").map((a) => a.charAt(0).toUpperCase() + a.slice(1));
    let r = i.length > 0;
    for (const a of i) if (!a.match(Vf)) {
      r = false;
      break;
    }
    let o = i.join(" ");
    return r || (o = `"${o.replace(/[\\"]/g, "\\$&")}"`), o;
  }
  const Xf = /^[0-9A-Za-z%:/?#\[\]@!\$&'()\*\+,;=\-._~]*$/;
  function qf(n) {
    return Xf.test(n) ? n : encodeURI(n);
  }
  const Kf = {
    extension: {
      type: lt.LoadParser,
      priority: Ln.Low
    },
    name: "loadWebFont",
    id: "web-font",
    test(n) {
      return ks(n, jf) || Ms(n, Uf);
    },
    async load(n, t) {
      var _a2, _b2, _c2;
      const e = Rt.get().getFontFaceSet();
      if (e) {
        const s = [], i = ((_a2 = t.data) == null ? void 0 : _a2.family) ?? Yf(n), r = ((_c2 = (_b2 = t.data) == null ? void 0 : _b2.weights) == null ? void 0 : _c2.filter((a) => Hf.includes(a))) ?? [
          "normal"
        ], o = t.data ?? {};
        for (let a = 0; a < r.length; a++) {
          const l = r[a], c = new FontFace(i, `url('${qf(n)}')`, {
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
        Rt.get().getFontFaceSet().delete(r);
      }), s.entries.length === 0 && ne.remove(`${e}-and-url`);
    }
  };
  var jr, ul;
  function Jf() {
    if (ul) return jr;
    ul = 1, jr = e;
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
  var Zf = Jf();
  const Qf = qc(Zf);
  function tm(n, t) {
    const e = Qf(n), s = [];
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
  class ia {
    constructor(t = 0, e = 0, s = 0) {
      this.type = "circle", this.x = t, this.y = e, this.radius = s;
    }
    clone() {
      return new ia(this.x, this.y, this.radius);
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
  class ra {
    constructor(t = 0, e = 0, s = 0, i = 0) {
      this.type = "ellipse", this.x = t, this.y = e, this.halfWidth = s, this.halfHeight = i;
    }
    clone() {
      return new ra(this.x, this.y, this.halfWidth, this.halfHeight);
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
  function em(n, t, e, s, i, r) {
    const o = n - e, a = t - s, l = i - e, c = r - s, h = o * l + a * c, d = l * l + c * c;
    let u = -1;
    d !== 0 && (u = h / d);
    let p, f;
    u < 0 ? (p = e, f = s) : u > 1 ? (p = i, f = r) : (p = e + u * l, f = s + u * c);
    const g = n - p, m = t - f;
    return g * g + m * m;
  }
  let nm, sm;
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
      const e = this.getBounds(nm), s = t.getBounds(sm);
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
        const d = l[h], u = l[h + 1], p = l[(h + 2) % l.length], f = l[(h + 3) % l.length], g = em(t, e, d, u, p, f), m = Math.sign((p - d) * (e - u) - (f - u) * (t - d));
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
  const Pi = (n, t, e, s, i, r, o) => {
    const a = n - e, l = t - s, c = Math.sqrt(a * a + l * l);
    return c >= i - r && c <= i + o;
  };
  class oa {
    constructor(t = 0, e = 0, s = 0, i = 0, r = 20) {
      this.type = "roundedRectangle", this.x = t, this.y = e, this.width = s, this.height = i, this.radius = r;
    }
    getBounds(t) {
      return t || (t = new Dt()), t.x = this.x, t.y = this.y, t.width = this.width, t.height = this.height, t;
    }
    clone() {
      return new oa(this.x, this.y, this.width, this.height, this.radius);
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
      return (t >= r - h && t <= r + d || t >= m - d && t <= m + h) && e >= p && e <= p + g || (e >= o - h && e <= o + d || e >= y - d && e <= y + h) && t >= u && t <= u + f ? true : t < u && e < p && Pi(t, e, u, p, c, d, h) || t > m - c && e < p && Pi(t, e, m - c, p, c, d, h) || t > m - c && e > y - c && Pi(t, e, m - c, y - c, c, d, h) || t < u && e > y - c && Pi(t, e, u, y - c, c, d, h);
    }
    toString() {
      return `[pixi.js/math:RoundedRectangle x=${this.x} y=${this.y}width=${this.width} height=${this.height} radius=${this.radius}]`;
    }
  }
  const Xh = {};
  im = function(n, t, e) {
    let s = 2166136261;
    for (let i = 0; i < t; i++) s ^= n[i].uid, s = Math.imul(s, 16777619), s >>>= 0;
    return Xh[s] || rm(n, t, s, e);
  };
  function rm(n, t, e, s) {
    const i = {};
    let r = 0;
    for (let a = 0; a < s; a++) {
      const l = a < t ? n[a] : Tt.EMPTY.source;
      i[r++] = l.source, i[r++] = l.style;
    }
    const o = new Yi(i);
    return Xh[e] = o, o;
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
  pl = function(n, t, e, s) {
    if (e ?? (e = 0), s ?? (s = Math.min(n.byteLength - e, t.byteLength)), !(e & 7) && !(s & 7)) {
      const i = s / 8;
      new Float64Array(t, 0, i).set(new Float64Array(n, e, i));
    } else if (!(e & 3) && !(s & 3)) {
      const i = s / 4;
      new Float32Array(t, 0, i).set(new Float32Array(n, e, i));
    } else new Uint8Array(t).set(new Uint8Array(n, e, s));
  };
  const om = {
    normal: "normal-npm",
    add: "add-npm",
    screen: "screen-npm"
  };
  am = ((n) => (n[n.DISABLED = 0] = "DISABLED", n[n.RENDERING_MASK_ADD = 1] = "RENDERING_MASK_ADD", n[n.MASK_ACTIVE = 2] = "MASK_ACTIVE", n[n.INVERSE_MASK_ACTIVE = 3] = "INVERSE_MASK_ACTIVE", n[n.RENDERING_MASK_REMOVE = 4] = "RENDERING_MASK_REMOVE", n[n.NONE = 5] = "NONE", n))(am || {});
  function ko(n, t) {
    return t.alphaMode === "no-premultiply-alpha" && om[n] || n;
  }
  const lm = [
    "precision mediump float;",
    "void main(void){",
    "float test = 0.1;",
    "%forloop%",
    "gl_FragColor = vec4(0.0);",
    "}"
  ].join(`
`);
  function cm(n) {
    let t = "";
    for (let e = 0; e < n; ++e) e > 0 && (t += `
else `), e < n - 1 && (t += `if(test == ${e}.0){}`);
    return t;
  }
  hm = function(n, t) {
    if (n === 0) throw new Error("Invalid value of `0` passed to `checkMaxIfStatementsInShader`");
    const e = t.createShader(t.FRAGMENT_SHADER);
    try {
      for (; ; ) {
        const s = lm.replace(/%forloop%/gi, cm(n));
        if (t.shaderSource(e, s), t.compileShader(e), !t.getShaderParameter(e, t.COMPILE_STATUS)) n = n / 2 | 0;
        else break;
      }
    } finally {
      t.deleteShader(e);
    }
    return n;
  };
  let rs = null;
  function dm() {
    var _a2;
    if (rs) return rs;
    const n = Ah();
    return rs = n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS), rs = hm(rs, n), (_a2 = n.getExtension("WEBGL_lose_context")) == null ? void 0 : _a2.loseContext(), rs;
  }
  class um {
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
  class pm {
    constructor() {
      this.renderPipeId = "batch", this.action = "startBatch", this.start = 0, this.size = 0, this.textures = new um(), this.blendMode = "normal", this.topology = "triangle-strip", this.canBundle = true;
    }
    destroy() {
      this.textures = null, this.gpuBindGroup = null, this.bindGroup = null, this.batcher = null, this.elements = null;
    }
  }
  const Zs = [];
  let tr = 0;
  fi.register({
    clear: () => {
      if (Zs.length > 0) for (const n of Zs) n && n.destroy();
      Zs.length = 0, tr = 0;
    }
  });
  function fl() {
    return tr > 0 ? Zs[--tr] : new pm();
  }
  function ml(n) {
    n.elements = null, Zs[tr++] = n;
  }
  let Fs = 0;
  const qh = class Kh {
    constructor(t) {
      this.uid = ee("batcher"), this.dirty = true, this.batchIndex = 0, this.batches = [], this._elements = [], t = {
        ...Kh.defaultOptions,
        ...t
      }, t.maxTextures || (It("v8.8.0", "maxTextures is a required option for Batcher now, please pass it in the options"), t.maxTextures = dm());
      const { maxTextures: e, attributesInitialSize: s, indicesInitialSize: i } = t;
      this.attributeBuffer = new fs(s * 4), this.indexBuffer = new Uint16Array(i), this.maxTextures = e;
    }
    begin() {
      this.elementSize = 0, this.elementStart = 0, this.indexSize = 0, this.attributeSize = 0;
      for (let t = 0; t < this.batchIndex; t++) ml(this.batches[t]);
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
      let s = fl(), i = s.textures;
      i.clear();
      const r = e[this.elementStart];
      let o = ko(r.blendMode, r.texture._source), a = r.topology;
      this.attributeSize * 4 > this.attributeBuffer.size && this._resizeAttributeBuffer(this.attributeSize * 4), this.indexSize > this.indexBuffer.length && this._resizeIndexBuffer(this.indexSize);
      const l = this.attributeBuffer.float32View, c = this.attributeBuffer.uint32View, h = this.indexBuffer;
      let d = this._batchIndexSize, u = this._batchIndexStart, p = "startBatch", f = [];
      const g = this.maxTextures;
      for (let m = this.elementStart; m < this.elementSize; ++m) {
        const y = e[m];
        e[m] = null;
        const x = y.texture._source, _ = ko(y.blendMode, x), v = o !== _ || a !== y.topology;
        if (x._batchTick === Fs && !v) {
          y._textureId = x._textureBindLocation, d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize)), y._batch = s, f.push(y);
          continue;
        }
        x._batchTick = Fs, (i.count >= g || v) && (this._finishBatch(s, u, d - u, i, o, a, t, p, f), p = "renderBatch", u = d, o = _, a = y.topology, s = fl(), i = s.textures, i.clear(), f = [], ++Fs), y._textureId = x._textureBindLocation = i.count, i.ids[x.uid] = i.count, i.textures[i.count++] = x, y._batch = s, f.push(y), d += y.indexSize, y.packAsQuad ? (this.packQuadAttributes(y, l, c, y._attributeStart, y._textureId), this.packQuadIndex(h, y._indexStart, y._attributeStart / this.vertexSize)) : (this.packAttributes(y, l, c, y._attributeStart, y._textureId), this.packIndex(y, h, y._indexStart, y._attributeStart / this.vertexSize));
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
      pl(this.attributeBuffer.rawBinaryData, s.rawBinaryData), this.attributeBuffer = s;
    }
    _resizeIndexBuffer(t) {
      const e = this.indexBuffer;
      let s = Math.max(t, e.length * 1.5);
      s += s % 2;
      const i = s > 65535 ? new Uint32Array(s) : new Uint16Array(s);
      if (i.BYTES_PER_ELEMENT !== e.BYTES_PER_ELEMENT) for (let r = 0; r < e.length; r++) i[r] = e[r];
      else pl(e.buffer, i.buffer);
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
        for (let e = 0; e < this.batchIndex; e++) ml(this.batches[e]);
        this.batches = null, this.geometry.destroy(true), this.geometry = null, t.shader && ((_a2 = this.shader) == null ? void 0 : _a2.destroy(), this.shader = null);
        for (let e = 0; e < this._elements.length; e++) this._elements[e] && (this._elements[e]._batch = null);
        this._elements = null, this.indexBuffer = null, this.attributeBuffer.destroy(), this.attributeBuffer = null;
      }
    }
  };
  qh.defaultOptions = {
    maxTextures: null,
    attributesInitialSize: 4,
    indicesInitialSize: 6
  };
  let fm = qh;
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
  function Jh(n, t) {
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
  function mm(n, t, e) {
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
  function gm(n) {
    return (n instanceof Zn || Array.isArray(n) || n.BYTES_PER_ELEMENT) && (n = {
      buffer: n
    }), n.buffer = Jh(n.buffer, false), n;
  }
  Zh = class extends rn {
    constructor(t = {}) {
      super(), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ee("geometry"), this._layoutKey = 0, this.instanceCount = 1, this._bounds = new Me(), this._boundsDirty = true;
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
      const s = gm(e);
      this.buffers.indexOf(s.buffer) === -1 && (this.buffers.push(s.buffer), s.buffer.on("update", this.onBufferUpdate, this), s.buffer.on("change", this.onBufferUpdate, this)), this.attributes[t] = s;
    }
    addIndex(t) {
      this.indexBuffer = Jh(t, true), this.buffers.push(this.indexBuffer);
    }
    get bounds() {
      return this._boundsDirty ? (this._boundsDirty = false, mm(this, "aPosition", this._bounds)) : this._bounds;
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
  const ym = new Float32Array(1), xm = new Uint32Array(1);
  class bm extends Zh {
    constructor() {
      const e = new Zn({
        data: ym,
        label: "attribute-batch-buffer",
        usage: le.VERTEX | le.COPY_DST,
        shrinkToFit: false
      }), s = new Zn({
        data: xm,
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
  function gl(n, t, e) {
    if (n) for (const s in n) {
      const i = s.toLocaleLowerCase(), r = t[i];
      if (r) {
        let o = n[s];
        s === "header" && (o = o.replace(/@in\s+[^;]+;\s*/g, "").replace(/@out\s+[^;]+;\s*/g, "")), e && r.push(`//----${e}----//`), r.push(o);
      } else Jt(`${s} placement hook does not exist in shader`);
    }
  }
  const _m = /\{\{(.*?)\}\}/g;
  function yl(n) {
    var _a2;
    const t = {};
    return (((_a2 = n.match(_m)) == null ? void 0 : _a2.map((s) => s.replace(/[{()}]/g, ""))) ?? []).forEach((s) => {
      t[s] = [];
    }), t;
  }
  function xl(n, t) {
    let e;
    const s = /@in\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function bl(n, t, e = false) {
    const s = [];
    xl(t, s), n.forEach((a) => {
      a.header && xl(a.header, s);
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
  function _l(n, t) {
    let e;
    const s = /@out\s+([^;]+);/g;
    for (; (e = s.exec(n)) !== null; ) t.push(e[1]);
  }
  function wm(n) {
    const e = /\b(\w+)\s*:/g.exec(n);
    return e ? e[1] : "";
  }
  function vm(n) {
    const t = /@.*?\s+/g;
    return n.replace(t, "");
  }
  function Cm(n, t) {
    const e = [];
    _l(t, e), n.forEach((l) => {
      l.header && _l(l.header, e);
    });
    let s = 0;
    const i = e.sort().map((l) => l.indexOf("builtin") > -1 ? l : `@location(${s++}) ${l}`).join(`,
`), r = e.sort().map((l) => `       var ${vm(l)};`).join(`
`), o = `return VSOutput(
            ${e.sort().map((l) => ` ${wm(l)}`).join(`,
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
  function wl(n, t) {
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
  let Sm = 0;
  function Tm({ template: n, bits: t }) {
    const e = Qh(n, t);
    if (An[e]) return An[e];
    const { vertex: s, fragment: i } = Am(n, t);
    return An[e] = td(s, i, t), An[e];
  }
  function Em({ template: n, bits: t }) {
    const e = Qh(n, t);
    return An[e] || (An[e] = td(n.vertex, n.fragment, t)), An[e];
  }
  function Am(n, t) {
    const e = t.map((o) => o.vertex).filter((o) => !!o), s = t.map((o) => o.fragment).filter((o) => !!o);
    let i = bl(e, n.vertex, true);
    i = Cm(e, i);
    const r = bl(s, n.fragment, true);
    return {
      vertex: i,
      fragment: r
    };
  }
  function Qh(n, t) {
    return t.map((e) => (Vr.has(e) || Vr.set(e, Sm++), Vr.get(e))).sort((e, s) => e - s).join("-") + n.vertex + n.fragment;
  }
  function td(n, t, e) {
    const s = yl(n), i = yl(t);
    return e.forEach((r) => {
      gl(r.vertex, s, r.name), gl(r.fragment, i, r.name);
    }), {
      vertex: wl(n, s),
      fragment: wl(t, i)
    };
  }
  const km = `
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
`, Mm = `
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
`, Pm = `
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
`, Im = `

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
`, Rm = {
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
  }, Lm = {
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
  $m = function({ bits: n, name: t }) {
    const e = Tm({
      template: {
        fragment: Mm,
        vertex: km
      },
      bits: [
        Rm,
        ...n
      ]
    });
    return mi.from({
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
  Bm = function({ bits: n, name: t }) {
    return new ea({
      name: t,
      ...Em({
        template: {
          vertex: Pm,
          fragment: Im
        },
        bits: [
          Lm,
          ...n
        ]
      })
    });
  };
  let Yr;
  Om = {
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
  Nm = {
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
  function Fm(n) {
    const t = [];
    if (n === 1) t.push("@group(1) @binding(0) var textureSource1: texture_2d<f32>;"), t.push("@group(1) @binding(1) var textureSampler1: sampler;");
    else {
      let e = 0;
      for (let s = 0; s < n; s++) t.push(`@group(1) @binding(${e++}) var textureSource${s + 1}: texture_2d<f32>;`), t.push(`@group(1) @binding(${e++}) var textureSampler${s + 1}: sampler;`);
    }
    return t.join(`
`);
  }
  function Wm(n) {
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
  Gm = function(n) {
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

                ${Fm(n)}
            `,
        main: `
                var uvDx = dpdx(vUV);
                var uvDy = dpdy(vUV);

                ${Wm(n)}
            `
      }
    }), Yr[n];
  };
  const Xr = {};
  function zm(n) {
    const t = [];
    for (let e = 0; e < n; e++) e > 0 && t.push("else"), e < n - 1 && t.push(`if(vTextureId < ${e}.5)`), t.push("{"), t.push(`	outColor = texture(uTextures[${e}], vUV);`), t.push("}");
    return t.join(`
`);
  }
  Dm = function(n) {
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

                ${zm(n)}
            `
      }
    }), Xr[n];
  };
  let vl;
  Hm = {
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
  Um = {
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
  vl = {};
  jm = function(n) {
    let t = vl[n];
    if (t) return t;
    const e = new Int32Array(n);
    for (let s = 0; s < n; s++) e[s] = s;
    return t = vl[n] = new na({
      uTextures: {
        value: e,
        type: "i32",
        size: n
      }
    }, {
      isStatic: true
    }), t;
  };
  class Cl extends ur {
    constructor(t) {
      const e = Bm({
        name: "batch",
        bits: [
          Nm,
          Dm(t),
          Um
        ]
      }), s = $m({
        name: "batch",
        bits: [
          Om,
          Gm(t),
          Hm
        ]
      });
      super({
        glProgram: e,
        gpuProgram: s,
        resources: {
          batchSamplers: jm(t)
        }
      }), this.maxTextures = t;
    }
  }
  let Ws = null;
  const ed = class nd extends fm {
    constructor(t) {
      super(t), this.geometry = new bm(), this.name = nd.extension.name, this.vertexSize = 6, Ws ?? (Ws = new Cl(t.maxTextures)), this.shader = Ws;
    }
    packAttributes(t, e, s, i, r) {
      const o = r << 16 | t.roundPixels & 65535, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, { positions: f, uvs: g } = t, m = t.color, y = t.attributeOffset, w = y + t.attributeSize;
      for (let x = y; x < w; x++) {
        const _ = x * 2, v = f[_], b = f[_ + 1];
        e[i++] = l * v + h * b + u, e[i++] = d * b + c * v + p, e[i++] = g[_], e[i++] = g[_ + 1], s[i++] = m, s[i++] = o;
      }
    }
    packQuadAttributes(t, e, s, i, r) {
      const o = t.texture, a = t.transform, l = a.a, c = a.b, h = a.c, d = a.d, u = a.tx, p = a.ty, f = t.bounds, g = f.maxX, m = f.minX, y = f.maxY, w = f.minY, x = o.uvs, _ = t.color, v = r << 16 | t.roundPixels & 65535;
      e[i + 0] = l * m + h * w + u, e[i + 1] = d * w + c * m + p, e[i + 2] = x.x0, e[i + 3] = x.y0, s[i + 4] = _, s[i + 5] = v, e[i + 6] = l * g + h * w + u, e[i + 7] = d * w + c * g + p, e[i + 8] = x.x1, e[i + 9] = x.y1, s[i + 10] = _, s[i + 11] = v, e[i + 12] = l * g + h * y + u, e[i + 13] = d * y + c * g + p, e[i + 14] = x.x2, e[i + 15] = x.y2, s[i + 16] = _, s[i + 17] = v, e[i + 18] = l * m + h * y + u, e[i + 19] = d * y + c * m + p, e[i + 20] = x.x3, e[i + 21] = x.y3, s[i + 22] = _, s[i + 23] = v;
    }
    _updateMaxTextures(t) {
      this.shader.maxTextures !== t && (Ws = new Cl(t), this.shader = Ws);
    }
    destroy() {
      this.shader = null, super.destroy();
    }
  };
  ed.extension = {
    type: [
      lt.Batcher
    ],
    name: "default"
  };
  Vm = ed;
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
  function Ym(n, t, e, s, i, r, o, a = null) {
    let l = 0;
    e *= t, i *= r;
    const c = a.a, h = a.b, d = a.c, u = a.d, p = a.tx, f = a.ty;
    for (; l < o; ) {
      const g = n[e], m = n[e + 1];
      s[i] = c * g + d * m + p, s[i + 1] = h * g + u * m + f, i += r, e += t, l++;
    }
  }
  function Xm(n, t, e, s) {
    let i = 0;
    for (t *= e; i < s; ) n[t] = 0, n[t + 1] = 0, t += e, i++;
  }
  function sd(n, t, e, s, i) {
    const r = t.a, o = t.b, a = t.c, l = t.d, c = t.tx, h = t.ty;
    e || (e = 0), s || (s = 2), i || (i = n.length / s - e);
    let d = e * s;
    for (let u = 0; u < i; u++) {
      const p = n[d], f = n[d + 1];
      n[d] = r * p + a * f + c, n[d + 1] = o * p + l * f + h, d += s;
    }
  }
  const qm = new vt();
  class aa {
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
      return s ? ch(e, s.groupColor) + (this.alpha * s.groupAlpha * 255 << 24) : e + (this.alpha * 255 << 24);
    }
    get transform() {
      var _a2;
      return ((_a2 = this.renderable) == null ? void 0 : _a2.groupTransform) || qm;
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
  const ai = {
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
        const _ = n, v = _.width / 2, b = _.height / 2;
        e = _.x + v, s = _.y + b, o = a = Math.max(0, Math.min(_.radius, Math.min(v, b))), i = v - o, r = b - a;
      }
      if (i < 0 || r < 0) return false;
      const l = Math.ceil(2.3 * Math.sqrt(o + a)), c = l * 8 + (i ? 4 : 0) + (r ? 4 : 0);
      if (c === 0) return false;
      if (l === 0) return t[0] = t[6] = e + i, t[1] = t[3] = s + r, t[2] = t[4] = e - i, t[5] = t[7] = s - r, true;
      let h = 0, d = l * 4 + (i ? 2 : 0) + 2, u = d, p = c, f = i + o, g = r, m = e + f, y = e - f, w = s + g;
      if (t[h++] = m, t[h++] = w, t[--d] = w, t[--d] = y, r) {
        const _ = s - g;
        t[u++] = y, t[u++] = _, t[--p] = _, t[--p] = m;
      }
      for (let _ = 1; _ < l; _++) {
        const v = Math.PI / 2 * (_ / l), b = i + Math.cos(v) * o, C = r + Math.sin(v) * a, T = e + b, R = e - b, A = s + C, k = s - C;
        t[h++] = T, t[h++] = A, t[--d] = A, t[--d] = R, t[u++] = R, t[u++] = k, t[--p] = k, t[--p] = T;
      }
      f = i, g = r + a, m = e + f, y = e - f, w = s + g;
      const x = s - g;
      return t[h++] = m, t[h++] = w, t[--p] = x, t[--p] = m, i && (t[h++] = y, t[h++] = w, t[--p] = x, t[--p] = y), true;
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
  }, Km = {
    ...ai,
    extension: {
      ...ai.extension,
      name: "ellipse"
    }
  }, Jm = {
    ...ai,
    extension: {
      ...ai.extension,
      name: "roundedRectangle"
    }
  }, id = 1e-4, Sl = 1e-4;
  function Zm(n) {
    const t = n.length;
    if (t < 6) return 1;
    let e = 0;
    for (let s = 0, i = n[t - 2], r = n[t - 1]; s < t; s += 2) {
      const o = n[s], a = n[s + 1];
      e += (o - i) * (a + r), i = o, r = a;
    }
    return e < 0 ? -1 : 1;
  }
  function Tl(n, t, e, s, i, r, o, a) {
    const l = n - e * i, c = t - s * i, h = n + e * r, d = t + s * r;
    let u, p;
    o ? (u = s, p = -e) : (u = -s, p = e);
    const f = l + u, g = c + p, m = h + u, y = d + p;
    return a.push(f, g), a.push(m, y), 2;
  }
  function On(n, t, e, s, i, r, o, a) {
    const l = e - n, c = s - t;
    let h = Math.atan2(l, c), d = Math.atan2(i - n, r - t);
    a && h < d ? h += Math.PI * 2 : !a && h > d && (d += Math.PI * 2);
    let u = h;
    const p = d - h, f = Math.abs(p), g = Math.sqrt(l * l + c * c), m = (15 * f * Math.sqrt(g) / Math.PI >> 0) + 1, y = p / m;
    if (u += y, a) {
      o.push(n, t), o.push(e, s);
      for (let w = 1, x = u; w < m; w++, x += y) o.push(n, t), o.push(n + Math.sin(x) * g, t + Math.cos(x) * g);
      o.push(n, t), o.push(i, r);
    } else {
      o.push(e, s), o.push(n, t);
      for (let w = 1, x = u; w < m; w++, x += y) o.push(n + Math.sin(x) * g, t + Math.cos(x) * g), o.push(n, t);
      o.push(i, r), o.push(n, t);
    }
    return m * 2;
  }
  Qm = function(n, t, e, s, i, r) {
    const o = id;
    if (n.length === 0) return;
    const a = t;
    let l = a.alignment;
    if (t.alignment !== 0.5) {
      let q = Zm(n);
      l = (l - 0.5) * q + 0.5;
    }
    const c = new Mt(n[0], n[1]), h = new Mt(n[n.length - 2], n[n.length - 1]), d = s, u = Math.abs(c.x - h.x) < o && Math.abs(c.y - h.y) < o;
    if (d) {
      n = n.slice(), u && (n.pop(), n.pop(), h.set(n[n.length - 2], n[n.length - 1]));
      const q = (c.x + h.x) * 0.5, et = (h.y + c.y) * 0.5;
      n.unshift(q, et), n.push(q, et);
    }
    const p = i, f = n.length / 2;
    let g = n.length;
    const m = p.length / 2, y = a.width / 2, w = y * y, x = a.miterLimit * a.miterLimit;
    let _ = n[0], v = n[1], b = n[2], C = n[3], T = 0, R = 0, A = -(v - C), k = _ - b, L = 0, H = 0, W = Math.sqrt(A * A + k * k);
    A /= W, k /= W, A *= y, k *= y;
    const F = l, I = (1 - F) * 2, z = F * 2;
    d || (a.cap === "round" ? g += On(_ - A * (I - z) * 0.5, v - k * (I - z) * 0.5, _ - A * I, v - k * I, _ + A * z, v + k * z, p, true) + 2 : a.cap === "square" && (g += Tl(_, v, A, k, I, z, true, p))), p.push(_ - A * I, v - k * I), p.push(_ + A * z, v + k * z);
    for (let q = 1; q < f - 1; ++q) {
      _ = n[(q - 1) * 2], v = n[(q - 1) * 2 + 1], b = n[q * 2], C = n[q * 2 + 1], T = n[(q + 1) * 2], R = n[(q + 1) * 2 + 1], A = -(v - C), k = _ - b, W = Math.sqrt(A * A + k * k), A /= W, k /= W, A *= y, k *= y, L = -(C - R), H = b - T, W = Math.sqrt(L * L + H * H), L /= W, H /= W, L *= y, H *= y;
      const et = b - _, M = v - C, O = b - T, N = R - C, D = et * O + M * N, K = M * O - N * et, nt = K < 0;
      if (Math.abs(K) < 1e-3 * Math.abs(D)) {
        p.push(b - A * I, C - k * I), p.push(b + A * z, C + k * z), D >= 0 && (a.join === "round" ? g += On(b, C, b - A * I, C - k * I, b - L * I, C - H * I, p, false) + 4 : g += 2, p.push(b - L * z, C - H * z), p.push(b + L * I, C + H * I));
        continue;
      }
      const ut = (-A + _) * (-k + C) - (-A + b) * (-k + v), pt = (-L + T) * (-H + C) - (-L + b) * (-H + R), ct = (et * pt - O * ut) / K, _t = (N * ut - M * pt) / K, Y = (ct - b) * (ct - b) + (_t - C) * (_t - C), st = b + (ct - b) * I, ot = C + (_t - C) * I, at = b - (ct - b) * z, bt = C - (_t - C) * z, zt = Math.min(et * et + M * M, O * O + N * N), At = nt ? I : z, Xt = zt + At * At * w;
      Y <= Xt ? a.join === "bevel" || Y / w > x ? (nt ? (p.push(st, ot), p.push(b + A * z, C + k * z), p.push(st, ot), p.push(b + L * z, C + H * z)) : (p.push(b - A * I, C - k * I), p.push(at, bt), p.push(b - L * I, C - H * I), p.push(at, bt)), g += 2) : a.join === "round" ? nt ? (p.push(st, ot), p.push(b + A * z, C + k * z), g += On(b, C, b + A * z, C + k * z, b + L * z, C + H * z, p, true) + 4, p.push(st, ot), p.push(b + L * z, C + H * z)) : (p.push(b - A * I, C - k * I), p.push(at, bt), g += On(b, C, b - A * I, C - k * I, b - L * I, C - H * I, p, false) + 4, p.push(b - L * I, C - H * I), p.push(at, bt)) : (p.push(st, ot), p.push(at, bt)) : (p.push(b - A * I, C - k * I), p.push(b + A * z, C + k * z), a.join === "round" ? nt ? g += On(b, C, b + A * z, C + k * z, b + L * z, C + H * z, p, true) + 2 : g += On(b, C, b - A * I, C - k * I, b - L * I, C - H * I, p, false) + 2 : a.join === "miter" && Y / w <= x && (nt ? (p.push(at, bt), p.push(at, bt)) : (p.push(st, ot), p.push(st, ot)), g += 2), p.push(b - L * I, C - H * I), p.push(b + L * z, C + H * z), g += 2);
    }
    _ = n[(f - 2) * 2], v = n[(f - 2) * 2 + 1], b = n[(f - 1) * 2], C = n[(f - 1) * 2 + 1], A = -(v - C), k = _ - b, W = Math.sqrt(A * A + k * k), A /= W, k /= W, A *= y, k *= y, p.push(b - A * I, C - k * I), p.push(b + A * z, C + k * z), d || (a.cap === "round" ? g += On(b - A * (I - z) * 0.5, C - k * (I - z) * 0.5, b - A * I, C - k * I, b + A * z, C + k * z, p, false) + 2 : a.cap === "square" && (g += Tl(b, C, A, k, I, z, false, p)));
    const J = Sl * Sl;
    for (let q = m; q < g + m - 2; ++q) _ = p[q * 2], v = p[q * 2 + 1], b = p[(q + 1) * 2], C = p[(q + 1) * 2 + 1], T = p[(q + 2) * 2], R = p[(q + 2) * 2 + 1], !(Math.abs(_ * (C - R) + b * (R - v) + T * (v - C)) < J) && r.push(q, q + 1, q + 2);
  };
  function tg(n, t, e, s) {
    const i = id;
    if (n.length === 0) return;
    const r = n[0], o = n[1], a = n[n.length - 2], l = n[n.length - 1], c = t || Math.abs(r - a) < i && Math.abs(o - l) < i, h = e, d = n.length / 2, u = h.length / 2;
    for (let p = 0; p < d; p++) h.push(n[p * 2]), h.push(n[p * 2 + 1]);
    for (let p = 0; p < d - 1; p++) s.push(u + p, u + p + 1);
    c && s.push(u + d - 1, u);
  }
  function rd(n, t, e, s, i, r, o) {
    const a = yf(n, t, 2);
    if (!a) return;
    for (let c = 0; c < a.length; c += 3) r[o++] = a[c] + i, r[o++] = a[c + 1] + i, r[o++] = a[c + 2] + i;
    let l = i * s;
    for (let c = 0; c < n.length; c += 2) e[l] = n[c], e[l + 1] = n[c + 1], l += s;
  }
  const eg = [], ng = {
    extension: {
      type: lt.ShapeBuilder,
      name: "polygon"
    },
    build(n, t) {
      for (let e = 0; e < n.points.length; e++) t[e] = n.points[e];
      return true;
    },
    triangulate(n, t, e, s, i, r) {
      rd(n, eg, t, e, s, i, r);
    }
  }, sg = {
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
  }, ig = {
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
  }, El = [
    {
      offset: 0,
      color: "white"
    },
    {
      offset: 1,
      color: "black"
    }
  ], la = class Mo {
    constructor(...t) {
      this.uid = ee("fillGradient"), this._tick = 0, this.type = "linear", this.colorStops = [];
      let e = rg(t);
      e = {
        ...e.type === "radial" ? Mo.defaultRadialOptions : Mo.defaultLinearOptions,
        ...Zc(e)
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
      const l = this.colorStops.length ? this.colorStops : El, c = this._textureSize, { canvas: h, context: d } = kl(c, 1), u = a ? d.createLinearGradient(this._textureSize, 0, 0, 0) : d.createLinearGradient(0, 0, this._textureSize, 0);
      Al(u, l), d.fillStyle = u, d.fillRect(0, 0, c, 1), this.texture = new Tt({
        source: new Cs({
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
      const t = this.colorStops.length ? this.colorStops : El, e = this._textureSize, { canvas: s, context: i } = kl(e, e), { x: r, y: o } = this.center, { x: a, y: l } = this.outerCenter, c = this.innerRadius, h = this.outerRadius, d = a - h, u = l - h, p = e / (h * 2), f = (r - d) * p, g = (o - u) * p, m = i.createRadialGradient(f, g, c * p, (a - d) * p, (l - u) * p, h * p);
      Al(m, t), i.fillStyle = t[t.length - 1].color, i.fillRect(0, 0, e, e), i.fillStyle = m, i.translate(f, g), i.rotate(this.rotation), i.scale(1, this.scale), i.translate(-f, -g), i.fillRect(0, 0, e, e), this.texture = new Tt({
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
  la.defaultLinearOptions = {
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
  la.defaultRadialOptions = {
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
  yn = la;
  function Al(n, t) {
    for (let e = 0; e < t.length; e++) {
      const s = t[e];
      n.addColorStop(s.offset, s.color);
    }
  }
  function kl(n, t) {
    const e = Rt.get().createCanvas(n, t), s = e.getContext("2d");
    return {
      canvas: e,
      context: s
    };
  }
  function rg(n) {
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
      textureSize: n[5] ?? yn.defaultLinearOptions.textureSize
    }), t;
  }
  const og = new vt(), ag = new Dt();
  lg = function(n, t, e, s) {
    const i = t.matrix ? n.copyFrom(t.matrix).invert() : n.identity();
    if (t.textureSpace === "local") {
      const o = e.getBounds(ag);
      t.width && o.pad(t.width);
      const { x: a, y: l } = o, c = 1 / o.width, h = 1 / o.height, d = -a * c, u = -l * h, p = i.a, f = i.b, g = i.c, m = i.d;
      i.a *= c, i.b *= c, i.c *= h, i.d *= h, i.tx = d * p + u * g + i.tx, i.ty = d * f + u * m + i.ty;
    } else i.translate(t.texture.frame.x, t.texture.frame.y), i.scale(1 / t.texture.source.width, 1 / t.texture.source.height);
    const r = t.texture.source.style;
    return !(t.fill instanceof yn) && r.addressMode === "clamp-to-edge" && (r.addressMode = "repeat", r.update()), s && i.append(og.copyFrom(s).invert()), i;
  };
  fr = {};
  Gt.handleByMap(lt.ShapeBuilder, fr);
  Gt.add(sg, ng, ig, ai, Km, Jm);
  const cg = new Dt(), hg = new vt();
  function dg(n, t) {
    const { geometryData: e, batches: s } = t;
    s.length = 0, e.indices.length = 0, e.vertices.length = 0, e.uvs.length = 0;
    for (let i = 0; i < n.instructions.length; i++) {
      const r = n.instructions[i];
      if (r.action === "texture") ug(r.data, s, e);
      else if (r.action === "fill" || r.action === "stroke") {
        const o = r.action === "stroke", a = r.data.path.shapePath, l = r.data.style, c = r.data.hole;
        o && c && Ml(c.shapePath, l, true, s, e), c && (a.shapePrimitives[a.shapePrimitives.length - 1].holes = c.shapePath.shapePrimitives), Ml(a, l, o, s, e);
      }
    }
  }
  function ug(n, t, e) {
    const s = [], i = fr.rectangle, r = cg;
    r.x = n.dx, r.y = n.dy, r.width = n.dw, r.height = n.dh;
    const o = n.transform;
    if (!i.build(r, s)) return;
    const { vertices: a, uvs: l, indices: c } = e, h = c.length, d = a.length / 2;
    o && sd(s, o), i.triangulate(s, a, 2, d, c, h);
    const u = n.image, p = u.uvs;
    l.push(p.x0, p.y0, p.x1, p.y1, p.x3, p.y3, p.x2, p.y2);
    const f = ve.get(aa);
    f.indexOffset = h, f.indexSize = c.length - h, f.attributeOffset = d, f.attributeSize = a.length / 2 - d, f.baseColor = n.style, f.alpha = n.alpha, f.texture = u, f.geometryData = e, t.push(f);
  }
  function Ml(n, t, e, s, i) {
    const { vertices: r, uvs: o, indices: a } = i;
    n.shapePrimitives.forEach(({ shape: l, transform: c, holes: h }) => {
      const d = [], u = fr[l.type];
      if (!u.build(l, d)) return;
      const p = a.length, f = r.length / 2;
      let g = "triangle-list";
      if (c && sd(d, c), e) {
        const x = l.closePath ?? true, _ = t;
        _.pixelLine ? (tg(d, x, r, a), g = "line-list") : Qm(d, _, false, x, r, a);
      } else if (h) {
        const x = [], _ = d.slice();
        pg(h).forEach((b) => {
          x.push(_.length / 2), _.push(...b);
        }), rd(_, x, r, 2, f, a, p);
      } else u.triangulate(d, r, 2, f, a, p);
      const m = o.length / 2, y = t.texture;
      if (y !== Tt.WHITE) {
        const x = lg(hg, t, l, c);
        Ym(r, 2, f, o, m, 2, r.length / 2 - f, x);
      } else Xm(o, m, 2, r.length / 2 - f);
      const w = ve.get(aa);
      w.indexOffset = p, w.indexSize = a.length - p, w.attributeOffset = f, w.attributeSize = r.length / 2 - f, w.baseColor = t.color, w.alpha = t.alpha, w.texture = y, w.geometryData = i, w.topology = g, s.push(w);
    });
  }
  function pg(n) {
    const t = [];
    for (let e = 0; e < n.length; e++) {
      const s = n[e].shape, i = [];
      fr[s.type].build(s, i) && t.push(i);
    }
    return t;
  }
  class fg {
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
  class mg {
    constructor() {
      this.instructions = new Qo();
    }
    init(t) {
      const e = t.maxTextures;
      this.batcher ? this.batcher._updateMaxTextures(e) : this.batcher = new Vm({
        maxTextures: e
      }), this.instructions.reset();
    }
    get geometry() {
      return It(Du, "GraphicsContextRenderData#geometry is deprecated, please use batcher.geometry instead."), this.batcher.geometry;
    }
    destroy() {
      this.batcher.destroy(), this.instructions.destroy(), this.batcher = null, this.instructions = null;
    }
  }
  const ca = class Po {
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
        e && s.reset(), dg(t, s);
        const i = t.batchMode;
        t.customShader || i === "no-batch" ? s.isBatchable = false : i === "auto" ? s.isBatchable = s.geometryData.vertices.length < 400 : s.isBatchable = true, t.dirty = false;
      }
      return s;
    }
    getGpuContext(t) {
      return t._gpuData[this._renderer.uid] || this._initContext(t);
    }
    _initContextRenderData(t) {
      const e = ve.get(mg, {
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
        u.bindGroup = im(u.textures.textures, u.textures.count, this._renderer.limits.maxBatchableTextures);
      }
      return e;
    }
    _initContext(t) {
      const e = new fg();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  ca.extension = {
    type: [
      lt.WebGLSystem,
      lt.WebGPUSystem
    ],
    name: "graphicsContext"
  };
  ca.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let ha = ca;
  const gg = 8, Ii = 11920929e-14, yg = 1;
  function od(n, t, e, s, i, r, o, a, l, c) {
    const d = Math.min(0.99, Math.max(0, c ?? ha.defaultOptions.bezierSmoothness));
    let u = (yg - d) / 1;
    return u *= u, xg(t, e, s, i, r, o, a, l, n, u), n;
  }
  function xg(n, t, e, s, i, r, o, a, l, c) {
    Io(n, t, e, s, i, r, o, a, l, c, 0), l.push(o, a);
  }
  function Io(n, t, e, s, i, r, o, a, l, c, h) {
    if (h > gg) return;
    const d = (n + e) / 2, u = (t + s) / 2, p = (e + i) / 2, f = (s + r) / 2, g = (i + o) / 2, m = (r + a) / 2, y = (d + p) / 2, w = (u + f) / 2, x = (p + g) / 2, _ = (f + m) / 2, v = (y + x) / 2, b = (w + _) / 2;
    if (h > 0) {
      let C = o - n, T = a - t;
      const R = Math.abs((e - o) * T - (s - a) * C), A = Math.abs((i - o) * T - (r - a) * C);
      if (R > Ii && A > Ii) {
        if ((R + A) * (R + A) <= c * (C * C + T * T)) {
          l.push(v, b);
          return;
        }
      } else if (R > Ii) {
        if (R * R <= c * (C * C + T * T)) {
          l.push(v, b);
          return;
        }
      } else if (A > Ii) {
        if (A * A <= c * (C * C + T * T)) {
          l.push(v, b);
          return;
        }
      } else if (C = v - (n + o) / 2, T = b - (t + a) / 2, C * C + T * T <= c) {
        l.push(v, b);
        return;
      }
    }
    Io(n, t, d, u, y, w, v, b, l, c, h + 1), Io(v, b, x, _, g, m, o, a, l, c, h + 1);
  }
  const bg = 8, _g = 11920929e-14, wg = 1;
  function vg(n, t, e, s, i, r, o, a) {
    const c = Math.min(0.99, Math.max(0, a ?? ha.defaultOptions.bezierSmoothness));
    let h = (wg - c) / 1;
    return h *= h, Cg(t, e, s, i, r, o, n, h), n;
  }
  function Cg(n, t, e, s, i, r, o, a) {
    Ro(o, n, t, e, s, i, r, a, 0), o.push(i, r);
  }
  function Ro(n, t, e, s, i, r, o, a, l) {
    if (l > bg) return;
    const c = (t + s) / 2, h = (e + i) / 2, d = (s + r) / 2, u = (i + o) / 2, p = (c + d) / 2, f = (h + u) / 2;
    let g = r - t, m = o - e;
    const y = Math.abs((s - r) * m - (i - o) * g);
    if (y > _g) {
      if (y * y <= a * (g * g + m * m)) {
        n.push(p, f);
        return;
      }
    } else if (g = p - (t + r) / 2, m = f - (e + o) / 2, g * g + m * m <= a) {
      n.push(p, f);
      return;
    }
    Ro(n, t, e, c, h, p, f, a, l + 1), Ro(n, p, f, d, u, r, o, a, l + 1);
  }
  function ad(n, t, e, s, i, r, o, a) {
    let l = Math.abs(i - r);
    (!o && i > r || o && r > i) && (l = 2 * Math.PI - l), a || (a = Math.max(6, Math.floor(6 * Math.pow(s, 1 / 3) * (l / Math.PI)))), a = Math.max(a, 3);
    let c = l / a, h = i;
    c *= o ? -1 : 1;
    for (let d = 0; d < a + 1; d++) {
      const u = Math.cos(h), p = Math.sin(h), f = t + u * s, g = e + p * s;
      n.push(f, g), h += c;
    }
  }
  function Sg(n, t, e, s, i, r) {
    const o = n[n.length - 2], l = n[n.length - 1] - e, c = o - t, h = i - e, d = s - t, u = Math.abs(l * d - c * h);
    if (u < 1e-8 || r === 0) {
      (n[n.length - 2] !== t || n[n.length - 1] !== e) && n.push(t, e);
      return;
    }
    const p = l * l + c * c, f = h * h + d * d, g = l * h + c * d, m = r * Math.sqrt(p) / u, y = r * Math.sqrt(f) / u, w = m * g / p, x = y * g / f, _ = m * d + y * c, v = m * h + y * l, b = c * (y + w), C = l * (y + w), T = d * (m + x), R = h * (m + x), A = Math.atan2(C - v, b - _), k = Math.atan2(R - v, T - _);
    ad(n, _ + t, v + e, r, A, k, c * h > d * l);
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
  function Tg(n, t) {
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
  const Pl = (n, t, e, s) => {
    const i = n * s - t * e < 0 ? -1 : 1;
    let r = n * e + t * s;
    return r > 1 && (r = 1), r < -1 && (r = -1), i * Math.acos(r);
  }, Eg = (n, t, e, s, i, r, o, a, l, c, h, d, u) => {
    const p = Math.pow(i, 2), f = Math.pow(r, 2), g = Math.pow(h, 2), m = Math.pow(d, 2);
    let y = p * f - p * m - f * g;
    y < 0 && (y = 0), y /= p * m + f * g, y = Math.sqrt(y) * (o === a ? -1 : 1);
    const w = y * i / r * d, x = y * -r / i * h, _ = c * w - l * x + (n + e) / 2, v = l * w + c * x + (t + s) / 2, b = (h - w) / i, C = (d - x) / r, T = (-h - w) / i, R = (-d - x) / r, A = Pl(1, 0, b, C);
    let k = Pl(b, C, T, R);
    a === 0 && k > 0 && (k -= Qs), a === 1 && k < 0 && (k += Qs), u.centerX = _, u.centerY = v, u.ang1 = A, u.ang2 = k;
  };
  function Ag(n, t, e, s, i, r, o, a = 0, l = 0, c = 0) {
    if (r === 0 || o === 0) return;
    const h = Math.sin(a * Qs / 360), d = Math.cos(a * Qs / 360), u = d * (t - s) / 2 + h * (e - i) / 2, p = -h * (t - s) / 2 + d * (e - i) / 2;
    if (u === 0 && p === 0) return;
    r = Math.abs(r), o = Math.abs(o);
    const f = Math.pow(u, 2) / Math.pow(r, 2) + Math.pow(p, 2) / Math.pow(o, 2);
    f > 1 && (r *= Math.sqrt(f), o *= Math.sqrt(f)), Eg(t, e, s, i, r, o, l, c, h, d, u, p, qr);
    let { ang1: g, ang2: m } = qr;
    const { centerX: y, centerY: w } = qr;
    let x = Math.abs(m) / (Qs / 4);
    Math.abs(1 - x) < 1e-7 && (x = 1);
    const _ = Math.max(Math.ceil(x), 1);
    m /= _;
    let v = n[n.length - 2], b = n[n.length - 1];
    const C = {
      x: 0,
      y: 0
    };
    for (let T = 0; T < _; T++) {
      const R = Tg(g, m), { x: A, y: k } = Kr(R[0], r, o, d, h, y, w, C), { x: L, y: H } = Kr(R[1], r, o, d, h, y, w, C), { x: W, y: F } = Kr(R[2], r, o, d, h, y, w, C);
      od(n, v, b, A, k, L, H, W, F), v = W, b = F, g += m;
    }
  }
  function kg(n, t, e) {
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
      const w = a.x + d.nx * y + -d.ny * m * p, x = a.y + d.ny * y + d.nx * m * p, _ = Math.atan2(h.ny, h.nx) + Math.PI / 2 * p, v = Math.atan2(d.ny, d.nx) - Math.PI / 2 * p;
      o === 0 && n.moveTo(w + Math.cos(_) * m, x + Math.sin(_) * m), n.arc(w, x, m, _, v, f), r = a;
    }
  }
  function Mg(n, t, e, s) {
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
  const Pg = new Dt();
  class Ig {
    constructor(t) {
      this.shapePrimitives = [], this._currentPoly = null, this._bounds = new Me(), this._graphicsPath2D = t, this.signed = t.checkForHoles;
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
      return ad(a, t, e, s, i, r, o), this;
    }
    arcTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly.points;
      return Sg(o, t, e, s, i, r), this;
    }
    arcToSvg(t, e, s, i, r, o, a) {
      const l = this._currentPoly.points;
      return Ag(l, this._currentPoly.lastX, this._currentPoly.lastY, o, a, t, e, s, i, r), this;
    }
    bezierCurveTo(t, e, s, i, r, o, a) {
      this._ensurePoly();
      const l = this._currentPoly;
      return od(this._currentPoly.points, l.lastX, l.lastY, t, e, s, i, r, o, a), this;
    }
    quadraticCurveTo(t, e, s, i, r) {
      this._ensurePoly();
      const o = this._currentPoly;
      return vg(this._currentPoly.points, o.lastX, o.lastY, t, e, s, i, r), this;
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
      return this.drawShape(new ia(t, e, s), i), this;
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
        const p = u * h + c, f = t + s * Math.cos(p), g = e + s * Math.sin(p), m = p + Math.PI + d, y = p - Math.PI - d, w = f + r * Math.cos(m), x = g + r * Math.sin(m), _ = f + r * Math.cos(y), v = g + r * Math.sin(y);
        u === 0 ? this.moveTo(w, x) : this.lineTo(w, x), this.quadraticCurveTo(f, g, _, v, a);
      }
      return this.closePath();
    }
    roundShape(t, e, s = false, i) {
      return t.length < 3 ? this : (s ? Mg(this, t, e, i) : kg(this, t, e), this.closePath());
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
      return this.drawShape(new ra(t, e, s, i), r), this;
    }
    roundRect(t, e, s, i, r, o) {
      return this.drawShape(new oa(t, e, s, i, r), o), this;
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
        const i = e[s], r = i.shape.getBounds(Pg);
        i.transform ? t.addRect(r, i.transform) : t.addRect(r);
      }
      return t;
    }
  }
  class mn {
    constructor(t, e = false) {
      this.instructions = [], this.uid = ee("graphicsPath"), this._dirty = true, this.checkForHoles = e, typeof t == "string" ? tm(t, this) : this.instructions = (t == null ? void 0 : t.slice()) ?? [];
    }
    get shapePath() {
      return this._shapePath || (this._shapePath = new Ig(this)), this._dirty && (this._dirty = false, this._shapePath.buildPath()), this._shapePath;
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
      let l = 0, c = 0, h = 0, d = 0, u = 0, p = 0, f = 0, g = 0;
      for (let m = 0; m < this.instructions.length; m++) {
        const y = this.instructions[m], w = y.data;
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
            l = w[5], c = w[6], f = w[0], g = w[1], w[0] = e * f + i * g, w[1] = s * f + r * g, w[5] = e * l + i * c + o, w[6] = s * l + r * c + a;
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
  function Rg(n, t) {
    const e = n.querySelectorAll("defs");
    for (let s = 0; s < e.length; s++) {
      const i = e[s];
      for (let r = 0; r < i.children.length; r++) {
        const o = i.children[r];
        switch (o.nodeName.toLowerCase()) {
          case "lineargradient":
            t.defs[o.id] = Lg(o);
            break;
          case "radialgradient":
            t.defs[o.id] = $g();
            break;
        }
      }
    }
  }
  function Lg(n) {
    const t = Qt(n, "x1", 0), e = Qt(n, "y1", 0), s = Qt(n, "x2", 1), i = Qt(n, "y2", 0), r = n.getAttribute("gradientUnits") || "objectBoundingBox", o = new yn(t, e, s, i, r === "objectBoundingBox" ? "local" : "global");
    for (let a = 0; a < n.children.length; a++) {
      const l = n.children[a], c = Qt(l, "offset", 0), h = Yt.shared.setValue(l.getAttribute("stop-color")).toNumber();
      o.addColorStop(c, h);
    }
    return o;
  }
  function $g(n) {
    return Jt("[SVG Parser] Radial gradients are not yet supported"), new yn(0, 0, 1, 0);
  }
  function Il(n) {
    const t = n.match(/url\s*\(\s*['"]?\s*#([^'"\s)]+)\s*['"]?\s*\)/i);
    return t ? t[1] : "";
  }
  const Rl = {
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
  function ld(n, t) {
    const e = n.getAttribute("style"), s = {}, i = {}, r = {
      strokeStyle: s,
      fillStyle: i,
      useFill: false,
      useStroke: false
    };
    for (const o in Rl) {
      const a = n.getAttribute(o);
      a && Ll(t, r, o, a.trim());
    }
    if (e) {
      const o = e.split(";");
      for (let a = 0; a < o.length; a++) {
        const l = o[a].trim(), [c, h] = l.split(":");
        Rl[c] && Ll(t, r, c, h.trim());
      }
    }
    return {
      strokeStyle: r.useStroke ? s : null,
      fillStyle: r.useFill ? i : null,
      useFill: r.useFill,
      useStroke: r.useStroke
    };
  }
  function Ll(n, t, e, s) {
    switch (e) {
      case "stroke":
        if (s !== "none") {
          if (s.startsWith("url(")) {
            const i = Il(s);
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
            const i = Il(s);
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
  function Bg(n) {
    if (n.length <= 2) return true;
    const t = n.map((a) => a.area).sort((a, l) => l - a), [e, s] = t, i = t[t.length - 1], r = e / s, o = s / i;
    return !(r > 3 && o < 2);
  }
  function Og(n) {
    return n.split(/(?=[Mm])/).filter((s) => s.trim().length > 0);
  }
  function Ng(n) {
    const t = n.match(/[-+]?[0-9]*\.?[0-9]+/g);
    if (!t || t.length < 4) return 0;
    const e = t.map(Number), s = [], i = [];
    for (let h = 0; h < e.length; h += 2) h + 1 < e.length && (s.push(e[h]), i.push(e[h + 1]));
    if (s.length === 0 || i.length === 0) return 0;
    const r = Math.min(...s), o = Math.max(...s), a = Math.min(...i), l = Math.max(...i);
    return (o - r) * (l - a);
  }
  function $l(n, t) {
    const e = new mn(n, false);
    for (const s of e.instructions) t.instructions.push(s);
  }
  function Fg(n, t) {
    if (typeof n == "string") {
      const o = document.createElement("div");
      o.innerHTML = n.trim(), n = o.querySelector("svg");
    }
    const e = {
      context: t,
      defs: {},
      path: new mn()
    };
    Rg(n, e);
    const s = n.children, { fillStyle: i, strokeStyle: r } = ld(n, e);
    for (let o = 0; o < s.length; o++) {
      const a = s[o];
      a.nodeName.toLowerCase() !== "defs" && cd(a, e, i, r);
    }
    return t;
  }
  function cd(n, t, e, s) {
    const i = n.children, { fillStyle: r, strokeStyle: o } = ld(n, t);
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
    let l, c, h, d, u, p, f, g, m, y, w, x, _, v, b, C, T;
    switch (n.nodeName.toLowerCase()) {
      case "path": {
        v = n.getAttribute("d");
        const R = n.getAttribute("fill-rule"), A = Og(v), k = R === "evenodd", L = A.length > 1;
        if (k && L) {
          const W = A.map((I) => ({
            path: I,
            area: Ng(I)
          }));
          if (W.sort((I, z) => z.area - I.area), A.length > 3 || !Bg(W)) for (let I = 0; I < W.length; I++) {
            const z = W[I], J = I === 0;
            t.context.beginPath();
            const q = new mn(void 0, true);
            $l(z.path, q), t.context.path(q), J ? (e && t.context.fill(e), s && t.context.stroke(s)) : t.context.cut();
          }
          else for (let I = 0; I < W.length; I++) {
            const z = W[I], J = I % 2 === 1;
            t.context.beginPath();
            const q = new mn(void 0, true);
            $l(z.path, q), t.context.path(q), J ? t.context.cut() : (e && t.context.fill(e), s && t.context.stroke(s));
          }
        } else {
          const W = R ? R === "evenodd" : true;
          b = new mn(v, W), t.context.path(b), e && t.context.fill(e), s && t.context.stroke(s);
        }
        break;
      }
      case "circle":
        f = Qt(n, "cx", 0), g = Qt(n, "cy", 0), m = Qt(n, "r", 0), t.context.ellipse(f, g, m, m), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "rect":
        l = Qt(n, "x", 0), c = Qt(n, "y", 0), C = Qt(n, "width", 0), T = Qt(n, "height", 0), y = Qt(n, "rx", 0), w = Qt(n, "ry", 0), y || w ? t.context.roundRect(l, c, C, T, y || w) : t.context.rect(l, c, C, T), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "ellipse":
        f = Qt(n, "cx", 0), g = Qt(n, "cy", 0), y = Qt(n, "rx", 0), w = Qt(n, "ry", 0), t.context.beginPath(), t.context.ellipse(f, g, y, w), e && t.context.fill(e), s && t.context.stroke(s);
        break;
      case "line":
        h = Qt(n, "x1", 0), d = Qt(n, "y1", 0), u = Qt(n, "x2", 0), p = Qt(n, "y2", 0), t.context.beginPath(), t.context.moveTo(h, d), t.context.lineTo(u, p), s && t.context.stroke(s);
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
        Jt(`[SVG parser] <${n.nodeName}> elements unsupported`);
        break;
      }
    }
    a && (e = null);
    for (let R = 0; R < i.length; R++) cd(i[R], t, e, s);
  }
  const Bl = {
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
      this.uid = ee("fillPattern"), this._tick = 0, this.transform = new vt(), this.texture = t, this.transform.scale(1 / t.frame.width, 1 / t.frame.height), e && (t.source.style.addressModeU = Bl[e].addressModeU, t.source.style.addressModeV = Bl[e].addressModeV);
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
  function Wg(n) {
    return Yt.isColorLike(n);
  }
  function Ol(n) {
    return n instanceof mr;
  }
  function Nl(n) {
    return n instanceof yn;
  }
  function Gg(n) {
    return n instanceof Tt;
  }
  function zg(n, t, e) {
    const s = Yt.shared.setValue(t ?? 0);
    return n.color = s.toNumber(), n.alpha = s.alpha === 1 ? e.alpha : s.alpha, n.texture = Tt.WHITE, {
      ...e,
      ...n
    };
  }
  function Dg(n, t, e) {
    return n.texture = t, {
      ...e,
      ...n
    };
  }
  function Fl(n, t, e) {
    return n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, {
      ...e,
      ...n
    };
  }
  function Wl(n, t, e) {
    return t.buildGradient(), n.fill = t, n.color = 16777215, n.texture = t.texture, n.matrix = t.transform, n.textureSpace = t.textureSpace, {
      ...e,
      ...n
    };
  }
  function Hg(n, t) {
    const e = {
      ...t,
      ...n
    }, s = Yt.shared.setValue(e.color);
    return e.alpha *= s.alpha, e.color = s.toNumber(), e;
  }
  function Xn(n, t) {
    if (n == null) return null;
    const e = {}, s = n;
    return Wg(n) ? zg(e, n, t) : Gg(n) ? Dg(e, n, t) : Ol(n) ? Fl(e, n, t) : Nl(n) ? Wl(e, n, t) : s.fill && Ol(s.fill) ? Fl(s, s.fill, t) : s.fill && Nl(s.fill) ? Wl(s, s.fill, t) : Hg(s, t);
  }
  function er(n, t) {
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
  function Ug(n, t) {
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
        const h = (c - 2 + a) % a, d = (c + 2) % a, u = o[h], p = o[h + 1], f = o[c], g = o[c + 1], m = o[d], y = o[d + 1], w = u - f, x = p - g, _ = m - f, v = y - g, b = w * w + x * x, C = _ * _ + v * v;
        if (b < 1e-12 || C < 1e-12) continue;
        let A = (w * _ + x * v) / Math.sqrt(b * C);
        A < -1 ? A = -1 : A > 1 && (A = 1);
        const k = Math.sqrt((1 - A) * 0.5);
        if (k < 1e-6) continue;
        const L = Math.min(1 / k, t);
        L > e && (e = L);
      }
    }
    return e;
  }
  const jg = new Mt(), Gl = new vt(), da = class Ze extends rn {
    constructor() {
      super(...arguments), this._gpuData = /* @__PURE__ */ Object.create(null), this.autoGarbageCollect = true, this._gcLastUsed = -1, this.uid = ee("graphicsContext"), this.dirty = true, this.batchMode = "auto", this.instructions = [], this.destroyed = false, this._activePath = new mn(), this._transform = new vt(), this._fillStyle = {
        ...Ze.defaultFillStyle
      }, this._strokeStyle = {
        ...Ze.defaultStrokeStyle
      }, this._stateStack = [], this._tick = 0, this._bounds = new Me(), this._boundsDirty = true;
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
      this._strokeStyle = er(t, Ze.defaultStrokeStyle);
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
          style: e || e === 0 ? Yt.shared.setValue(e).toNumber() : 16777215
        }
      }), this.onUpdate(), this;
    }
    beginPath() {
      return this._activePath = new mn(), this;
    }
    fill(t, e) {
      let s;
      const i = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (i == null ? void 0 : i.action) === "stroke" ? s = i.data.path : s = this._activePath.clone(), s ? (t != null && (e !== void 0 && typeof t == "number" && (It(te, "GraphicsContext.fill(color, alpha) is deprecated, use GraphicsContext.fill({ color, alpha }) instead"), t = {
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
      const { x: t, y: e } = this._activePath.getLastPoint(Mt.shared);
      this._activePath.clear(), this._activePath.moveTo(t, e);
    }
    stroke(t) {
      let e;
      const s = this.instructions[this.instructions.length - 1];
      return this._tick === 0 && (s == null ? void 0 : s.action) === "fill" ? e = s.data.path : e = this._activePath.clone(), e ? (t != null && (this._strokeStyle = er(t, Ze.defaultStrokeStyle)), this.instructions.push({
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
      return this._tick++, Fg(t, this), this;
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
      return t instanceof vt ? (this._transform.append(t), this) : (Gl.set(t, e, s, i, r, o), this._transform.append(Gl), this);
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
          r.style.join === "miter" && (a *= Ug(r.path, r.style.miterLimit));
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
          const u = c[h].transform, p = u ? u.applyInverse(t, jg) : t;
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
  da.defaultFillStyle = {
    color: 16777215,
    alpha: 1,
    texture: Tt.WHITE,
    matrix: null,
    fill: null,
    textureSpace: "local"
  };
  da.defaultStrokeStyle = {
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
  let Re = da;
  function ua(n, t = 1) {
    var _a2;
    const e = (_a2 = As.RETINA_PREFIX) == null ? void 0 : _a2.exec(n);
    return e ? parseFloat(e[1]) : t;
  }
  function pa(n, t, e) {
    n.label = e, n._sourceOrigin = e;
    const s = new Tt({
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
  const Vg = ".svg", Yg = "image/svg+xml", Xg = {
    extension: {
      type: lt.LoadParser,
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
      return ks(n, Yg) || Ms(n, Vg);
    },
    async load(n, t, e) {
      var _a2;
      return ((_a2 = t.data) == null ? void 0 : _a2.parseAsGraphicsContext) ?? this.config.parseAsGraphicsContext ? Kg(n) : qg(n, t, e, this.config.crossOrigin);
    },
    unload(n) {
      n.destroy(true);
    }
  };
  async function qg(n, t, e, s) {
    var _a2, _b2, _c2;
    const i = await Rt.get().fetch(n), r = Rt.get().createImage();
    r.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(await i.text())}`, r.crossOrigin = s, await r.decode();
    const o = ((_a2 = t.data) == null ? void 0 : _a2.width) ?? r.width, a = ((_b2 = t.data) == null ? void 0 : _b2.height) ?? r.height, l = ((_c2 = t.data) == null ? void 0 : _c2.resolution) || ua(n), c = Math.ceil(o * l), h = Math.ceil(a * l), d = Rt.get().createCanvas(c, h), u = d.getContext("2d");
    u.imageSmoothingEnabled = true, u.imageSmoothingQuality = "high", u.drawImage(r, 0, 0, o * l, a * l);
    const { parseAsGraphicsContext: p, ...f } = t.data ?? {}, g = new Cs({
      resource: d,
      alphaMode: "premultiply-alpha-on-upload",
      resolution: l,
      ...f
    });
    return pa(g, e, n);
  }
  async function Kg(n) {
    const e = await (await Rt.get().fetch(n)).text(), s = new Re();
    return s.svg(e), s;
  }
  const Jg = `(function () {
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
        Jg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(ys);
    }
  };
  Lo.revokeObjectURL = function() {
    ys && (URL.revokeObjectURL(ys), ys = null);
  };
  const Zg = `(function () {
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
  class hd {
    constructor() {
      xs || (xs = URL.createObjectURL(new Blob([
        Zg
      ], {
        type: "application/javascript"
      }))), this.worker = new Worker(xs);
    }
  }
  hd.revokeObjectURL = function() {
    xs && (URL.revokeObjectURL(xs), xs = null);
  };
  let zl = 0, Jr;
  class Qg {
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
      return !t && this._createdWorkers < Jr && (this._createdWorkers++, t = new hd().worker, t.addEventListener("message", (e) => {
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
      this._resolveHash[zl] = {
        resolve: e.resolve,
        reject: e.reject
      }, t.postMessage({
        data: e.arguments,
        uuid: zl++,
        id: s
      });
    }
    reset() {
      this._workerPool.forEach((t) => t.terminate()), this._workerPool.length = 0, Object.values(this._resolveHash).forEach(({ reject: t }) => {
        t == null ? void 0 : t(new Error("WorkerManager has been reset before completion"));
      }), this._resolveHash = {}, this._queue.length = 0, this._initialized = false, this._createdWorkers = 0;
    }
  }
  const Dl = new Qg(), ty = [
    ".jpeg",
    ".jpg",
    ".png",
    ".webp",
    ".avif"
  ], ey = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/avif"
  ];
  async function ny(n, t) {
    var _a2;
    const e = await Rt.get().fetch(n);
    if (!e.ok) throw new Error(`[loadImageBitmap] Failed to fetch ${n}: ${e.status} ${e.statusText}`);
    const s = await e.blob();
    return ((_a2 = t == null ? void 0 : t.data) == null ? void 0 : _a2.alphaMode) === "premultiplied-alpha" ? createImageBitmap(s, {
      premultiplyAlpha: "none"
    }) : createImageBitmap(s);
  }
  const dd = {
    name: "loadTextures",
    id: "texture",
    extension: {
      type: lt.LoadParser,
      priority: Ln.High,
      name: "loadTextures"
    },
    config: {
      preferWorkers: true,
      preferCreateImageBitmap: true,
      crossOrigin: "anonymous"
    },
    test(n) {
      return ks(n, ey) || Ms(n, ty);
    },
    async load(n, t, e) {
      var _a2;
      let s = null;
      globalThis.createImageBitmap && this.config.preferCreateImageBitmap ? this.config.preferWorkers && await Dl.isImageBitmapSupported() ? s = await Dl.loadImageBitmap(n, t) : s = await ny(n, t) : s = await new Promise((r, o) => {
        s = Rt.get().createImage(), s.crossOrigin = this.config.crossOrigin, s.src = n, s.complete ? r(s) : (s.onload = () => {
          r(s);
        }, s.onerror = o);
      });
      const i = new Cs({
        resource: s,
        alphaMode: "premultiply-alpha-on-upload",
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || ua(n),
        ...t.data
      });
      return pa(i, e, n);
    },
    unload(n) {
      n.destroy(true);
    }
  }, sy = [
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
  function iy(n, t, e) {
    e === void 0 && !t.startsWith("data:") ? n.crossOrigin = oy(t) : e !== false && (n.crossOrigin = typeof e == "string" ? e : "anonymous");
  }
  function ry(n) {
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
  function oy(n, t = globalThis.location) {
    if (n.startsWith("data:")) return "";
    t || (t = globalThis.location);
    const e = new URL(n, document.baseURI);
    return e.hostname !== t.hostname || e.port !== t.port || e.protocol !== t.protocol ? "anonymous" : "";
  }
  function ay() {
    const n = [], t = [];
    for (const e of sy) {
      const s = Ks.MIME_TYPES[e.substring(1)] || `video/${e.substring(1)}`;
      pr(s) && (n.push(e), t.includes(s) || t.push(s));
    }
    return {
      validVideoExtensions: n,
      validVideoMime: t
    };
  }
  const ly = {
    name: "loadVideo",
    id: "video",
    extension: {
      type: lt.LoadParser,
      name: "loadVideo"
    },
    test(n) {
      if (!Zr || !Qr) {
        const { validVideoExtensions: s, validVideoMime: i } = ay();
        Zr = s, Qr = i;
      }
      const t = ks(n, Qr), e = Ms(n, Zr);
      return t || e;
    },
    async load(n, t, e) {
      var _a2, _b2;
      const s = {
        ...Ks.defaultOptions,
        resolution: ((_a2 = t.data) == null ? void 0 : _a2.resolution) || ua(n),
        alphaMode: ((_b2 = t.data) == null ? void 0 : _b2.alphaMode) || await wh(),
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
      }), s.muted === true && (i.muted = true), iy(i, n, s.crossorigin);
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
          u(), t.data.preload && await ry(i), l(pa(p, e, n));
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
  }, ud = {
    extension: {
      type: lt.ResolveParser,
      name: "resolveTexture"
    },
    test: dd.test,
    parse: (n) => {
      var _a2;
      return {
        resolution: parseFloat(((_a2 = As.RETINA_PREFIX.exec(n)) == null ? void 0 : _a2[1]) ?? "1"),
        format: n.split(".").pop(),
        src: n
      };
    }
  }, cy = {
    extension: {
      type: lt.ResolveParser,
      priority: -2,
      name: "resolveJson"
    },
    test: (n) => As.RETINA_PREFIX.test(n) && n.endsWith(".json"),
    parse: ud.parse
  };
  class hy {
    constructor() {
      this._detections = [], this._initialized = false, this.resolver = new As(), this.loader = new Of(), this.cache = ne, this._backgroundLoader = new Af(this.loader), this._backgroundLoader.active = true, this.reset();
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
      const s = Ji(t), i = je(t).map((a) => {
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
        ].reduce((y, w) => y + (w.progressSize || 1), 0);
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
      const e = je(t).map((i) => typeof i != "string" ? i.src : i), s = this.resolver.resolve(e);
      await this._unloadFromResolved(s);
    }
    async unloadBundle(t) {
      this._initialized || await this.init(), t = je(t);
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
  const $e = new hy();
  Gt.handleByList(lt.LoadParser, $e.loader.parsers).handleByList(lt.ResolveParser, $e.resolver.parsers).handleByList(lt.CacheParser, $e.cache.parsers).handleByList(lt.DetectionParser, $e.detections);
  Gt.add(kf, Pf, Mf, Bf, Rf, Lf, $f, Wf, Df, Kf, Xg, dd, ly, Ef, Tf, ud, cy);
  const Hl = {
    loader: lt.LoadParser,
    resolver: lt.ResolveParser,
    cache: lt.CacheParser,
    detection: lt.DetectionParser
  };
  Gt.handle(lt.Asset, (n) => {
    const t = n.ref;
    Object.entries(Hl).filter(([e]) => !!t[e]).forEach(([e, s]) => Gt.add(Object.assign(t[e], {
      extension: t[e].extension ?? s
    })));
  }, (n) => {
    const t = n.ref;
    Object.keys(Hl).filter((e) => !!t[e]).forEach((e) => Gt.remove(t[e]));
  });
  class dy {
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
  class uy {
    constructor() {
      this.instructions = new Qo();
    }
    init() {
      this.instructions.reset();
    }
    destroy() {
      this.instructions.destroy(), this.instructions = null;
    }
  }
  const fa = class $o {
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
      const e = new uy(), s = this.getGpuContext(t);
      return s.graphicsData = e, e.init(), e;
    }
    _initContext(t) {
      const e = new dy();
      return e.context = t, t._gpuData[this._renderer.uid] = e, this._managedContexts.add(t), e;
    }
    destroy() {
      this._managedContexts.destroy(), this._renderer = null;
    }
  };
  fa.extension = {
    type: [
      lt.CanvasSystem
    ],
    name: "graphicsContext"
  };
  fa.defaultOptions = {
    bezierSmoothness: 0.5
  };
  let py = fa;
  class pd {
    constructor(t, e) {
      this.state = Qi.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Ps({
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
  pd.extension = {
    type: [
      lt.CanvasPipes
    ],
    name: "graphics"
  };
  fd = function(n, t, e) {
    const s = (n >> 24 & 255) / 255;
    t[e++] = (n & 255) / 255 * s, t[e++] = (n >> 8 & 255) / 255 * s, t[e++] = (n >> 16 & 255) / 255 * s, t[e++] = s;
  };
  class fy {
    constructor() {
      this.batches = [], this.batched = false;
    }
    destroy() {
      this.batches.forEach((t) => {
        ve.return(t);
      }), this.batches.length = 0;
    }
  }
  class md {
    constructor(t, e) {
      this.state = Qi.for2d(), this.renderer = t, this._adaptor = e, this.renderer.runners.contextChange.add(this), this._managedGraphics = new Ps({
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
      o.uTransformMatrix = t.groupTransform, o.uRound = e._roundPixels | t._roundPixels, fd(t.groupColorAlpha, o.uColor, 0), this._adaptor.execute(this, t);
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
      const e = new fy();
      return t._gpuData[this.renderer.uid] = e, this._managedGraphics.add(t), e;
    }
    _updateBatchesForRenderable(t, e) {
      const s = t.context, r = this.renderer.graphicsContext.getGpuContext(s), o = this.renderer._roundPixels | t._roundPixels;
      e.batches = r.batches.map((a) => {
        const l = ve.get(aa);
        return a.copyTo(l), l.renderable = t, l.roundPixels = o, l;
      });
    }
    destroy() {
      this._managedGraphics.destroy(), this.renderer = null, this._adaptor.destroy(), this._adaptor = null, this.state = null;
    }
  }
  md.extension = {
    type: [
      lt.WebGLPipes,
      lt.WebGPUPipes
    ],
    name: "graphics"
  };
  Gt.add(pd);
  Gt.add(md);
  Gt.add(py);
  Gt.add(ha);
  ft = class extends hr {
    constructor(t) {
      t instanceof Re && (t = {
        context: t
      });
      const { context: e, roundPixels: s, ...i } = t || {};
      super({
        label: "Graphics",
        ...i
      }), this.renderPipeId = "graphics", e ? this.context = e : (this.context = this._ownedContext = new Re(), this.context.autoGarbageCollect = this.autoGarbageCollect), this.didViewUpdate = true, this.allowChildren = false, this.roundPixels = s ?? false;
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
      It(te, "Graphics#lineStyle is no longer needed. Use Graphics#setStrokeStyle to set the stroke style.");
      const i = {};
      return t && (i.width = t), e && (i.color = e), s && (i.alpha = s), this.context.strokeStyle = i, this;
    }
    beginFill(t, e) {
      It(te, "Graphics#beginFill is no longer needed. Use Graphics#fill to fill the shape with the desired style.");
      const s = {};
      return t !== void 0 && (s.color = t), e !== void 0 && (s.alpha = e), this.context.fillStyle = s, this;
    }
    endFill() {
      It(te, "Graphics#endFill is no longer needed. Use Graphics#fill to fill the shape with the desired style."), this.context.fill();
      const t = this.context.strokeStyle;
      return (t.width !== Re.defaultStrokeStyle.width || t.color !== Re.defaultStrokeStyle.color || t.alpha !== Re.defaultStrokeStyle.alpha) && this.context.stroke(), this;
    }
    drawCircle(...t) {
      return It(te, "Graphics#drawCircle has been renamed to Graphics#circle"), this._callContextMethod("circle", t);
    }
    drawEllipse(...t) {
      return It(te, "Graphics#drawEllipse has been renamed to Graphics#ellipse"), this._callContextMethod("ellipse", t);
    }
    drawPolygon(...t) {
      return It(te, "Graphics#drawPolygon has been renamed to Graphics#poly"), this._callContextMethod("poly", t);
    }
    drawRect(...t) {
      return It(te, "Graphics#drawRect has been renamed to Graphics#rect"), this._callContextMethod("rect", t);
    }
    drawRoundedRect(...t) {
      return It(te, "Graphics#drawRoundedRect has been renamed to Graphics#roundRect"), this._callContextMethod("roundRect", t);
    }
    drawStar(...t) {
      return It(te, "Graphics#drawStar has been renamed to Graphics#star"), this._callContextMethod("star", t);
    }
  };
  let os;
  function Ul(n) {
    const t = Rt.get().createCanvas(6, 1), e = t.getContext("2d");
    return e.fillStyle = n, e.fillRect(0, 0, 6, 1), t;
  }
  my = function() {
    if (os !== void 0) return os;
    try {
      const n = Ul("#ff00ff"), t = Ul("#ffff00"), s = Rt.get().createCanvas(6, 1).getContext("2d");
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
    canUseMultiply: my(),
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
      const a = kt.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
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
      const a = kt.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
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
      const a = kt.isVertical(o), l = a ? i.height : i.width, c = a ? i.width : i.height;
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
      const i = kt.inv(t), r = kt.uX(i), o = kt.uY(i), a = kt.vX(i), l = kt.vY(i), c = -Math.min(0, r * e, a * s, r * e + a * s), h = -Math.min(0, o * e, l * s, o * e + l * s);
      n.transform(r, o, a, l, c, h);
    }
  };
  Kt.tintMethod = Kt.canUseMultiply ? Kt.tintWithMultiply : Kt.tintWithPerPixel;
  class gy extends hr {
    constructor(t, e) {
      const { text: s, resolution: i, style: r, anchor: o, width: a, height: l, roundPixels: c, ...h } = t;
      super({
        ...h
      }), this.batched = true, this._resolution = null, this._autoResolution = true, this._didTextUpdate = true, this._styleClass = e, this.text = s ?? "", this.style = r, this.resolution = i ?? null, this.allowChildren = false, this._anchor = new de({
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
  function yy(n, t) {
    let e = n[0] ?? {};
    return (typeof e == "string" || n[1]) && (It(te, `use new ${t}({ text: "hi!", style }) instead`), e = {
      text: e,
      style: n[1]
    }), e;
  }
  class xy {
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
  Bo = new xy();
  fi.register(Bo);
  let Nn = null, dn = null;
  function by(n, t) {
    Nn || (Nn = Rt.get().createCanvas(256, 128), dn = Nn.getContext("2d", {
      willReadFrequently: true
    }), dn.globalCompositeOperation = "copy", dn.globalAlpha = 1), (Nn.width < n || Nn.height < t) && (Nn.width = ws(n), Nn.height = ws(t));
  }
  function jl(n, t, e) {
    for (let s = 0, i = 4 * e * t; s < t; ++s, i += 4) if (n[i + 3] !== 0) return false;
    return true;
  }
  function Vl(n, t, e, s, i) {
    const r = 4 * t;
    for (let o = s, a = s * r + 4 * e; o <= i; ++o, a += r) if (n[a + 3] !== 0) return false;
    return true;
  }
  function _y(...n) {
    let t = n[0];
    t.canvas || (t = {
      canvas: n[0],
      resolution: n[1]
    });
    const { canvas: e } = t, s = Math.min(t.resolution ?? 1, 1), i = t.width ?? e.width, r = t.height ?? e.height;
    let o = t.output;
    if (by(i, r), !dn) throw new TypeError("Failed to get canvas 2D context");
    dn.drawImage(e, 0, 0, i, r, 0, 0, i * s, r * s);
    const l = dn.getImageData(0, 0, i, r).data;
    let c = 0, h = 0, d = i - 1, u = r - 1;
    for (; h < r && jl(l, i, h); ) ++h;
    if (h === r) return Dt.EMPTY;
    for (; jl(l, i, u); ) --u;
    for (; Vl(l, i, c, h, u); ) ++c;
    for (; Vl(l, i, d, h, u); ) --d;
    return ++d, ++u, dn.globalCompositeOperation = "source-over", dn.strokeRect(c, h, d - c, u - h), dn.globalCompositeOperation = "copy", o ?? (o = new Dt()), o.set(c / s, h / s, (d - c) / s, (u - h) / s), o;
  }
  class wy {
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
  vy = function(n = 1e3, t = 0, e = false) {
    if (isNaN(n) || n < 0) throw new TypeError("Invalid max value");
    if (isNaN(t) || t < 0) throw new TypeError("Invalid ttl value");
    if (typeof e != "boolean") throw new TypeError("Invalid resetTtl value");
    return new wy(n, t, e);
  };
  function gd(n) {
    return !!n.tagStyles && Object.keys(n.tagStyles).length > 0;
  }
  function yd(n) {
    return n.includes("<");
  }
  function Cy(n, t) {
    return n.clone().assign(t);
  }
  function Sy(n, t) {
    const e = [], s = t.tagStyles;
    if (!gd(t) || !yd(n)) return e.push({
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
            const u = i[i.length - 1], p = Cy(u, s[d]);
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
  const Ty = [
    10,
    13
  ], Ey = new Set(Ty), Ay = [
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
  ], ky = new Set(Ay), My = [
    9,
    32
  ], Py = new Set(My), Iy = [
    45,
    8208,
    8211,
    8212,
    173
  ], Ry = new Set(Iy), Ly = /(\r\n|\r|\n)/, $y = /(?:\r\n|\r|\n)/;
  function nr(n) {
    return typeof n != "string" ? false : Ey.has(n.charCodeAt(0));
  }
  Be = function(n, t) {
    return typeof n != "string" ? false : ky.has(n.charCodeAt(0));
  };
  vv = function(n) {
    return typeof n != "string" ? false : Py.has(n.charCodeAt(0));
  };
  By = function(n) {
    return typeof n != "string" ? false : Ry.has(n.charCodeAt(0));
  };
  xd = function(n) {
    return n === "normal" || n === "pre-line";
  };
  bd = function(n) {
    return n === "normal";
  };
  function ln(n) {
    if (typeof n != "string") return "";
    let t = n.length - 1;
    for (; t >= 0 && Be(n[t]); ) t--;
    return t < n.length - 1 ? n.slice(0, t + 1) : n;
  }
  function _d(n) {
    const t = [], e = [];
    if (typeof n != "string") return t;
    for (let s = 0; s < n.length; s++) {
      const i = n[s], r = n[s + 1];
      if (Be(i) || nr(i)) {
        e.length > 0 && (t.push(e.join("")), e.length = 0), i === "\r" && r === `
` ? (t.push(`\r
`), s++) : t.push(i);
        continue;
      }
      e.push(i), By(i) && r && !Be(r) && !nr(r) && (t.push(e.join("")), e.length = 0);
    }
    return e.length > 0 && t.push(e.join("")), t;
  }
  function wd(n, t, e, s) {
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
  const Oy = /\r\n|\r|\n/g;
  function Ny(n, t, e, s, i, r, o, a) {
    var _a2, _b2;
    const l = Sy(n, t);
    if (bd(t.whiteSpace)) for (let F = 0; F < l.length; F++) {
      const I = l[F];
      l[F] = {
        text: I.text.replace(Oy, " "),
        style: I.style
      };
    }
    const h = [];
    let d = [];
    for (const F of l) {
      const I = F.text.split(Ly);
      for (let z = 0; z < I.length; z++) {
        const J = I[z];
        J === `\r
` || J === "\r" || J === `
` ? (h.push(d), d = []) : J.length > 0 && d.push({
          text: J,
          style: F.style
        });
      }
    }
    (d.length > 0 || h.length === 0) && h.push(d);
    const u = e ? Fy(h, t, s, i, o, a) : h, p = [], f = [], g = [], m = [], y = [];
    let w = 0;
    const x = t._fontString, _ = r(x);
    _.fontSize === 0 && (_.fontSize = t.fontSize, _.ascent = t.fontSize);
    let v = "", b = !!t.dropShadow, C = ((_a2 = t._stroke) == null ? void 0 : _a2.width) || 0;
    for (const F of u) {
      let I = 0, z = _.ascent, J = _.descent, q = "";
      for (const M of F) {
        const O = M.style._fontString, N = r(O);
        O !== v && (s.font = O, v = O);
        const D = i(M.text, M.style.letterSpacing, s);
        I += D, z = Math.max(z, N.ascent), J = Math.max(J, N.descent), q += M.text;
        const K = ((_b2 = M.style._stroke) == null ? void 0 : _b2.width) || 0;
        K > C && (C = K), !b && M.style.dropShadow && (b = true);
      }
      F.length === 0 && (z = _.ascent, J = _.descent), p.push(I), f.push(z), g.push(J), y.push(q);
      const et = t.lineHeight || z + J;
      m.push(et + t.leading), w = Math.max(w, I);
    }
    const T = C, k = (e && t.align !== "left" ? Math.max(w, t.wordWrapWidth) : w) + T + (t.dropShadow ? t.dropShadow.distance : 0);
    let L = 0;
    for (let F = 0; F < m.length; F++) L += m[F];
    L = Math.max(L, m[0] + T);
    const H = L + (t.dropShadow ? t.dropShadow.distance : 0), W = t.lineHeight || _.fontSize;
    return {
      width: k,
      height: H,
      lines: y,
      lineWidths: p,
      lineHeight: W + t.leading,
      maxLineWidth: w,
      fontProperties: _,
      runsByLine: u,
      lineAscents: f,
      lineDescents: g,
      lineHeights: m,
      hasDropShadow: b
    };
  }
  function Fy(n, t, e, s, i, r) {
    var _a2;
    const { letterSpacing: o, whiteSpace: a, wordWrapWidth: l, breakWords: c } = t, h = xd(a), d = l + o, u = {};
    let p = "";
    const f = (m, y) => {
      const w = `${m}|${y.styleKey}`;
      let x = u[w];
      if (x === void 0) {
        const _ = y._fontString;
        _ !== p && (e.font = _, p = _), x = s(m, y.letterSpacing, e) + y.letterSpacing, u[w] = x;
      }
      return x;
    }, g = [];
    for (const m of n) {
      const y = Wy(m), w = g.length, x = (k) => {
        let L = 0, H = k;
        do {
          const { token: W, style: F } = y[H];
          L += f(W, F), H++;
        } while (H < y.length && y[H].continuesFromPrevious);
        return L;
      }, _ = (k) => {
        const L = [];
        let H = k;
        do
          L.push({
            token: y[H].token,
            style: y[H].style
          }), H++;
        while (H < y.length && y[H].continuesFromPrevious);
        return L;
      };
      let v = [], b = 0, C = !h, T = null;
      const R = () => {
        T && T.text.length > 0 && v.push(T), T = null;
      }, A = () => {
        if (R(), v.length > 0) {
          const k = v[v.length - 1];
          k.text = ln(k.text), k.text.length === 0 && v.pop();
        }
        g.push(v), v = [], b = 0, C = false;
      };
      for (let k = 0; k < y.length; k++) {
        const { token: L, style: H, continuesFromPrevious: W } = y[k], F = f(L, H);
        if (h) {
          const J = Be(L), q = (T == null ? void 0 : T.text[T.text.length - 1]) ?? ((_a2 = v[v.length - 1]) == null ? void 0 : _a2.text.slice(-1)) ?? "", et = q ? Be(q) : false;
          if (J && et) continue;
        }
        const I = !W, z = I ? x(k) : F;
        if (z > d && I) if (b > 0 && A(), c) {
          const J = _(k);
          for (let q = 0; q < J.length; q++) {
            const et = J[q].token, M = J[q].style, O = wd(et, c, r, i);
            for (const N of O) {
              const D = f(N, M);
              D + b > d && A(), !T || T.style !== M ? (R(), T = {
                text: N,
                style: M
              }) : T.text += N, b += D;
            }
          }
          k += J.length - 1;
        } else {
          const J = _(k);
          R(), g.push(J.map((q) => ({
            text: q.token,
            style: q.style
          }))), C = false, k += J.length - 1;
        }
        else if (z + b > d && I) {
          if (Be(L)) {
            C = false;
            continue;
          }
          A(), T = {
            text: L,
            style: H
          }, b = F;
        } else if (W && !c) !T || T.style !== H ? (R(), T = {
          text: L,
          style: H
        }) : T.text += L, b += F;
        else {
          const J = Be(L);
          if (b === 0 && J && !C) continue;
          !T || T.style !== H ? (R(), T = {
            text: L,
            style: H
          }) : T.text += L, b += F;
        }
      }
      if (R(), v.length > 0) {
        const k = v[v.length - 1];
        k.text = ln(k.text), k.text.length === 0 && v.pop();
      }
      (v.length > 0 || g.length === w) && g.push(v);
    }
    return g;
  }
  function Wy(n) {
    const t = [];
    let e = false;
    for (const s of n) {
      const i = _d(s.text);
      let r = true;
      for (const o of i) {
        const a = Be(o) || nr(o), l = r && e && !a;
        t.push({
          token: o,
          style: s.style,
          continuesFromPrevious: l
        }), e = !a, r = false;
      }
    }
    return t;
  }
  const Gy = {
    willReadFrequently: true
  };
  function Yl(n, t, e, s, i) {
    let r = e[n];
    return typeof r != "number" && (r = i(n, t, s) + t, e[n] = r), r;
  }
  function zy(n, t, e, s, i, r, o) {
    const a = e.getContext("2d", Gy);
    a.font = t._fontString;
    let l = 0, c = "";
    const h = [], d = /* @__PURE__ */ Object.create(null), { letterSpacing: u, whiteSpace: p } = t, f = xd(p), g = bd(p);
    let m = !f;
    const y = t.wordWrapWidth + u, w = _d(n);
    for (let _ = 0; _ < w.length; _++) {
      let v = w[_];
      if (nr(v)) {
        if (!g) {
          h.push(ln(c)), m = !f, c = "", l = 0;
          continue;
        }
        v = " ";
      }
      if (f) {
        const C = Be(v), T = Be(c[c.length - 1]);
        if (C && T) continue;
      }
      const b = Yl(v, u, d, a, s);
      if (b > y) if (c !== "" && (h.push(ln(c)), c = "", l = 0), i(v, t.breakWords)) {
        const C = wd(v, t.breakWords, o, r);
        for (const T of C) {
          const R = Yl(T, u, d, a, s);
          R + l > y && (h.push(ln(c)), m = false, c = "", l = 0), c += T, l += R;
        }
      } else c.length > 0 && (h.push(ln(c)), c = "", l = 0), h.push(ln(v)), m = false, c = "", l = 0;
      else b + l > y && (m = false, h.push(ln(c)), c = "", l = 0), (c.length > 0 || !Be(v) || m) && (c += v, l += b);
    }
    const x = ln(c);
    return x.length > 0 && h.push(x), h.join(`
`);
  }
  const Xl = {
    willReadFrequently: true
  }, xn = class xt {
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
      if (gd(e) && yd(t)) {
        const v = Ny(t, e, i, xt._context, xt._measureText, xt.measureFont, xt.canBreakChars, xt.wordWrapSplit), b = new xt(t, e, v.width, v.height, v.lines, v.lineWidths, v.lineHeight, v.maxLineWidth, v.fontProperties, {
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
      const d = (i ? xt._wordWrap(t, e, s) : t).split($y), u = new Array(d.length);
      let p = 0;
      for (let v = 0; v < d.length; v++) {
        const b = xt._measureText(d[v], e.letterSpacing, c);
        u[v] = b, p = Math.max(p, b);
      }
      const f = ((_a2 = e._stroke) == null ? void 0 : _a2.width) ?? 0, g = e.lineHeight || l.fontSize, m = xt._getAlignWidth(p, e, i), y = xt._adjustWidthForStyle(m, e), w = Math.max(g, l.fontSize + f) + (d.length - 1) * (g + e.leading), x = xt._adjustHeightForStyle(w, e), _ = new xt(t, e, y, x, d, u, g + e.leading, p, l);
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
      return zy(t, e, s, xt._measureText, xt.canBreakWords, xt.canBreakChars, xt.wordWrapSplit);
    }
    static isBreakingSpace(t, e) {
      return Be(t);
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
          if ((_a2 = e.getContext("2d", Xl)) == null ? void 0 : _a2.measureText) return xt.__canvas = e, e;
          t = Rt.get().createCanvas();
        } catch {
          t = Rt.get().createCanvas();
        }
        t.width = t.height = 10, xt.__canvas = t;
      }
      return xt.__canvas;
    }
    static get _context() {
      return xt.__context || (xt.__context = xt._canvas.getContext("2d", Xl)), xt.__context;
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
  xn._measurementCache = vy(1e3);
  cn = xn;
  const Dy = [
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
      !/([\"\'])[^\'\"]+\1/.test(i) && !Dy.includes(i) && (i = `"${i}"`), e[s] = i;
    }
    return `${n.fontStyle} ${n.fontVariant} ${n.fontWeight} ${t} ${e.join(",")}`;
  };
  const ql = 1e5;
  Ri = function(n, t, e, s = 0, i = 0, r = 0) {
    if (n.texture === Tt.WHITE && !n.fill) return Yt.shared.setValue(n.color).setAlpha(n.alpha ?? 1).toHexa();
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
          const { center: p, innerRadius: f, outerCenter: g, outerRadius: m } = o;
          d = t.createRadialGradient(p.x * c + i, p.y * h + r, f * c, g.x * c + i, g.y * h + r, m * c);
        }
        if (u && l && e) {
          const p = e.lineHeight / h;
          for (let f = 0; f < e.lines.length; f++) {
            const g = (f * e.lineHeight + s / 2) / h;
            o.colorStops.forEach((m) => {
              let y = g + m.offset * p;
              y = Math.max(0, Math.min(1, y)), d.addColorStop(Math.floor(y * ql) / ql, Yt.shared.setValue(m.color).toHex());
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
  const Kl = new Dt();
  function as(n) {
    let t = 0;
    for (let e = 0; e < n.length; e++) n.charCodeAt(e) === 32 && t++;
    return t;
  }
  class Hy {
    getCanvasAndContext(t) {
      const { text: e, style: s, resolution: i = 1 } = t, r = s._getFinalPadding(), o = cn.measureText(e || " ", s), a = Math.ceil(Math.ceil(Math.max(1, o.width) + r * 2) * i), l = Math.ceil(Math.ceil(Math.max(1, o.height) + r * 2) * i), c = Bo.getOptimalCanvasAndContext(a, l);
      this._renderTextToCanvas(s, r, i, c, o);
      const h = s.trim ? _y({
        canvas: c.canvas,
        width: a,
        height: l,
        resolution: 1,
        output: Kl
      }) : Kl.set(0, 0, a, l);
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
      let g, m;
      const y = t.dropShadow ? 2 : 1, w = t.wordWrap ? Math.max(t.wordWrapWidth, u) : u, _ = (((_b2 = t._stroke) == null ? void 0 : _b2.width) ?? 0) / 2;
      let v = (h - p.fontSize) / 2;
      h - p.fontSize < 0 && (v = 0);
      for (let b = 0; b < y; ++b) {
        const C = t.dropShadow && b === 0, T = C ? Math.ceil(Math.max(1, f) + e * 2) : 0, R = T * s;
        if (C) this._setupDropShadow(a, t, s, R);
        else {
          const A = t._gradientBounds, k = t._gradientOffset;
          if (A) {
            const L = {
              width: A.width,
              height: A.height,
              lineHeight: A.height,
              lines: r.lines
            };
            this._setFillAndStrokeStyles(a, t, L, e, _, (k == null ? void 0 : k.x) ?? 0, (k == null ? void 0 : k.y) ?? 0);
          } else k ? this._setFillAndStrokeStyles(a, t, r, e, _, k.x, k.y) : this._setFillAndStrokeStyles(a, t, r, e, _);
          a.shadowColor = "rgba(0,0,0,0)";
        }
        for (let A = 0; A < c.length; A++) {
          g = _, m = _ + A * h + p.ascent + v, g += this._getAlignmentOffset(d[A], w, t.align);
          let k = 0;
          if (t.align === "justify" && t.wordWrap && A < c.length - 1) {
            const L = as(c[A]);
            L > 0 && (k = (w - d[A]) / L);
          }
          ((_c2 = t._stroke) == null ? void 0 : _c2.width) && this._drawLetterSpacing(c[A], t, i, g + e, m + e - T, true, k), t._fill !== void 0 && this._drawLetterSpacing(c[A], t, i, g + e, m + e - T, false, k);
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
      for (let _ = 0; _ < g; ++_) {
        const v = p && _ === 0, b = v ? Math.ceil(Math.max(1, f) + s * 2) : 0, C = b * i;
        v || (a.shadowColor = "rgba(0,0,0,0)");
        let T = w;
        for (let R = 0; R < l.length; R++) {
          const A = l[R], k = c[R], L = d[R], H = u[R], W = x[R];
          let F = w;
          F += this._getAlignmentOffset(k, m, e.align);
          let I = 0;
          if (e.align === "justify" && e.wordWrap && R < l.length - 1) {
            let q = 0;
            for (const et of A) q += as(et.text);
            q > 0 && (I = (m - k) / q);
          }
          const z = T + L;
          let J = F + s;
          for (let q = 0; q < A.length; q++) {
            const et = A[q], { width: M, font: O } = W[q];
            if (a.font = O, a.textBaseline = et.style.textBaseline, (_c2 = et.style._stroke) == null ? void 0 : _c2.width) {
              const D = et.style._stroke;
              if (a.lineWidth = D.width, a.miterLimit = D.miterLimit, a.lineJoin = D.join, a.lineCap = D.cap, v) if (et.style.dropShadow) this._setupDropShadow(a, et.style, i, C);
              else {
                const K = as(et.text);
                J += M + K * I;
                continue;
              }
              else {
                const K = cn.measureFont(O), nt = et.style.lineHeight || K.fontSize, ut = {
                  width: M,
                  height: nt,
                  lineHeight: nt,
                  lines: [
                    et.text
                  ]
                };
                a.strokeStyle = Ri(D, a, ut, s * 2, J - s, T);
              }
              this._drawLetterSpacing(et.text, et.style, r, J, z + s - b, true, I);
            }
            const N = as(et.text);
            J += M + N * I;
          }
          J = F + s;
          for (let q = 0; q < A.length; q++) {
            const et = A[q], { width: M, font: O } = W[q];
            if (a.font = O, a.textBaseline = et.style.textBaseline, et.style._fill !== void 0) {
              if (v) if (et.style.dropShadow) this._setupDropShadow(a, et.style, i, C);
              else {
                const D = as(et.text);
                J += M + D * I;
                continue;
              }
              else {
                const D = cn.measureFont(O), K = et.style.lineHeight || D.fontSize, nt = {
                  width: M,
                  height: K,
                  lineHeight: K,
                  lines: [
                    et.text
                  ]
                };
                a.fillStyle = Ri(et.style._fill, a, nt, s * 2, J - s, T);
              }
              this._drawLetterSpacing(et.text, et.style, r, J, z + s - b, false, I);
            }
            const N = as(et.text);
            J += M + N * I;
          }
          T += H;
        }
      }
    }
    _setFillAndStrokeStyles(t, e, s, i, r, o = 0, a = 0) {
      var _a2;
      if (t.fillStyle = e._fill ? Ri(e._fill, t, s, i * 2, o, a) : null, (_a2 = e._stroke) == null ? void 0 : _a2.width) {
        const l = r + i * 2;
        t.strokeStyle = Ri(e._stroke, t, s, l, o, a);
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
      if (cn.experimentalLetterSpacingSupported && (cn.experimentalLetterSpacing ? (l.letterSpacing = `${c}px`, l.textLetterSpacing = `${c}px`, h = true) : (l.letterSpacing = "0px", l.textLetterSpacing = "0px")), (c === 0 || h) && a === 0) {
        o ? l.strokeText(t, i, r) : l.fillText(t, i, r);
        return;
      }
      if (a !== 0 && (c === 0 || h)) {
        const g = t.split(" ");
        let m = i;
        const y = l.measureText(" ").width;
        for (let w = 0; w < g.length; w++) o ? l.strokeText(g[w], m, r) : l.fillText(g[w], m, r), m += l.measureText(g[w]).width + y + a;
        return;
      }
      let d = i;
      const u = cn.graphemeSegmenter(t);
      let p = l.measureText(t).width, f = 0;
      for (let g = 0; g < u.length; ++g) {
        const m = u[g];
        o ? l.strokeText(m, d, r) : l.fillText(m, d, r);
        let y = "";
        for (let w = g + 1; w < u.length; ++w) y += u[w];
        f = l.measureText(y).width, d += p - f + c, m === " " && (d += a), p = f;
      }
    }
  }
  const ms = new Hy(), ma = class Vn extends rn {
    constructor(t = {}) {
      super(), this.uid = ee("textStyle"), this._tick = 0, this._cachedFontString = null, Uy(t), t instanceof Vn && (t = t._toObject());
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
        ...Re.defaultFillStyle,
        ...t
      }, () => {
        this._fill = Xn({
          ...this._originalFill
        }, Re.defaultFillStyle);
      })), this._fill = Xn(t === 0 ? "black" : t, Re.defaultFillStyle), this.update());
    }
    get stroke() {
      return this._originalStroke;
    }
    set stroke(t) {
      t !== this._originalStroke && (this._originalStroke = t, this._isFillStyle(t) && (this._originalStroke = this._createProxy({
        ...Re.defaultStrokeStyle,
        ...t
      }, () => {
        this._stroke = er({
          ...this._originalStroke
        }, Re.defaultStrokeStyle);
      })), this._stroke = er(t, Re.defaultStrokeStyle), this.update());
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
      return (t ?? null) !== null && !(Yt.isColorLike(t) || t instanceof yn || t instanceof mr);
    }
  };
  ma.defaultDropShadow = {
    alpha: 1,
    angle: Math.PI / 6,
    blur: 0,
    color: "black",
    distance: 5
  };
  ma.defaultTextStyle = {
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
  Fe = ma;
  function Uy(n) {
    const t = n;
    if (typeof t.dropShadow == "boolean" && t.dropShadow) {
      const e = Fe.defaultDropShadow;
      n.dropShadow = {
        alpha: t.dropShadowAlpha ?? e.alpha,
        angle: t.dropShadowAngle ?? e.angle,
        blur: t.dropShadowBlur ?? e.blur,
        color: t.dropShadowColor ?? e.color,
        distance: t.dropShadowDistance ?? e.distance
      };
    }
    if (t.strokeThickness !== void 0) {
      It(te, "strokeThickness is now a part of stroke");
      const e = t.stroke;
      let s = {};
      if (Yt.isColorLike(e)) s.color = e;
      else if (e instanceof yn || e instanceof mr) s.fill = e;
      else if (Object.hasOwnProperty.call(e, "color") || Object.hasOwnProperty.call(e, "fill")) s = e;
      else throw new Error("Invalid stroke value.");
      n.stroke = {
        ...s,
        width: t.strokeThickness
      };
    }
    if (Array.isArray(t.fillGradientStops)) {
      if (It(te, "gradient fill is now a fill pattern: `new FillGradient(...)`"), !Array.isArray(t.fill) || t.fill.length === 0) throw new Error("Invalid fill value. Expected an array of colors for gradient fill.");
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
      }), s = t.fillGradientStops.slice(), i = t.fill.map((r) => Yt.shared.setValue(r).toNumber());
      s.forEach((r, o) => {
        e.addColorStop(r, i[o]);
      }), n.fill = {
        fill: e
      };
    }
  }
  function jy(n, t) {
    const { texture: e, bounds: s } = n, i = t._style._getFinalPadding();
    sh(s, t._anchor, e);
    const r = t._anchor._x * i * 2, o = t._anchor._y * i * 2;
    s.minX -= i - r, s.minY -= i - o, s.maxX -= i - r, s.maxY -= i - o;
  }
  Vy = class {
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
  class Yy extends Vy {
  }
  class vd {
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
        (s.currentKey !== t.styleKey || t._resolution !== i) && this._updateGpuText(t), t._didTextUpdate = false, jy(s, t);
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
      const e = new Yy();
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
  vd.extension = {
    type: [
      lt.WebGLPipes,
      lt.WebGPUPipes,
      lt.CanvasPipes
    ],
    name: "text"
  };
  const Xy = new Me();
  function qy(n, t, e, s, i = false) {
    const r = Xy;
    r.minX = 0, r.minY = 0, r.maxX = n.width / s | 0, r.maxY = n.height / s | 0;
    const o = cr.getOptimalTexture(r.width, r.height, s, false, i);
    return o.source.uploadMethodId = "image", o.source.resource = n, o.source.alphaMode = "premultiply-alpha-on-upload", o.frame.width = t / s, o.frame.height = e / s, o.source.emit("update", o.source), o.updateUvs(), o;
  }
  class Cd {
    constructor(t, e) {
      this._activeTextures = {}, this._renderer = t, this._retainCanvasContext = e;
    }
    getTexture(t, e, s, i) {
      typeof t == "string" && (It("8.0.0", "CanvasTextSystem.getTexture: Use object TextOptions instead of separate arguments"), t = {
        text: t,
        style: s,
        resolution: e
      }), t.style instanceof Fe || (t.style = new Fe(t.style)), t.textureStyle instanceof Kn || (t.textureStyle = new Kn(t.textureStyle)), typeof t.text != "string" && (t.text = t.text.toString());
      const { text: r, style: o, textureStyle: a, autoGenerateMipmaps: l } = t, c = t.resolution ?? this._renderer.resolution, { frame: h, canvasAndContext: d } = ms.getCanvasAndContext({
        text: r,
        style: o,
        resolution: c
      }), u = qy(d.canvas, h.width, h.height, c, l);
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
  class Sd extends Cd {
    constructor(t) {
      super(t, true);
    }
  }
  Sd.extension = {
    type: [
      lt.CanvasSystem
    ],
    name: "canvasText"
  };
  class Td extends Cd {
    constructor(t) {
      super(t, false);
    }
  }
  Td.extension = {
    type: [
      lt.WebGLSystem,
      lt.WebGPUSystem
    ],
    name: "canvasText"
  };
  Gt.add(Sd);
  Gt.add(Td);
  Gt.add(vd);
  class Ve extends gy {
    constructor(...t) {
      const e = yy(t, "Text");
      super(e, Fe), this.renderPipeId = "text", e.textureStyle && (this.textureStyle = e.textureStyle instanceof Kn ? e.textureStyle : new Kn(e.textureStyle)), this.autoGenerateMipmaps = e.autoGenerateMipmaps ?? Ie.defaultOptions.autoGenerateMipmaps;
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
  gr = class extends Tt {
    static create(t) {
      const { dynamic: e, ...s } = t;
      return new gr({
        source: new Ie(s),
        dynamic: e ?? false
      });
    }
    resize(t, e, s) {
      return this.source.resize(t, e, s), this;
    }
  };
  class Ky {
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
        const w = u.frame, x = u.source.resolution, _ = w.x * x, v = w.y * x, b = w.width * x, C = w.height * x;
        i.globalAlpha = f;
        const T = -d.anchorX * w.width, R = -d.anchorY * w.height;
        d.rotation !== 0 || d.scaleX !== 1 || d.scaleY !== 1 ? (i.save(), i.translate(d.x, d.y), i.rotate(d.rotation), i.scale(d.scaleX, d.scaleY), i.drawImage(y, _, v, b, C, T, R, w.width, w.height), i.restore()) : i.drawImage(y, _, v, b, C, d.x + T, d.y + R, w.width, w.height);
      }
      i.restore();
    }
  }
  function Jl(n, t = null) {
    const e = n * 6;
    if (e > 65535 ? t || (t = new Uint32Array(e)) : t || (t = new Uint16Array(e)), t.length !== e) throw new Error(`Out buffer length is incorrect, got ${t.length} and expected ${e}`);
    for (let s = 0, i = 0; s < e; s += 6, i += 4) t[s + 0] = i + 0, t[s + 1] = i + 1, t[s + 2] = i + 2, t[s + 3] = i + 0, t[s + 4] = i + 2, t[s + 5] = i + 3;
    return t;
  }
  function Jy(n) {
    return {
      dynamicUpdate: Zl(n, true),
      staticUpdate: Zl(n, false)
    };
  }
  function Zl(n, t) {
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
      const a = Zi(o.format);
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
  class Zy {
    constructor(t) {
      this._size = 0, this._generateParticleUpdateCache = {};
      const e = this._size = t.size ?? 1e3, s = t.properties;
      let i = 0, r = 0;
      for (const h in s) {
        const d = s[h], u = Zi(d.format);
        d.dynamic ? r += u.stride : i += u.stride;
      }
      this._dynamicStride = r / 4, this._staticStride = i / 4, this.staticAttributeBuffer = new fs(e * 4 * i), this.dynamicAttributeBuffer = new fs(e * 4 * r), this.indexBuffer = Jl(e);
      const o = new Zh();
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
        const d = s[h], u = Zi(d.format);
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
      const e = Qy(t);
      return this._generateParticleUpdateCache[e] ? this._generateParticleUpdateCache[e] : (this._generateParticleUpdateCache[e] = this.generateParticleUpdate(t), this._generateParticleUpdateCache[e]);
    }
    generateParticleUpdate(t) {
      return Jy(t);
    }
    update(t, e) {
      t.length > this._size && (e = true, this._size = Math.max(t.length, this._size * 1.5 | 0), this.staticAttributeBuffer = new fs(this._size * this._staticStride * 4 * 4), this.dynamicAttributeBuffer = new fs(this._size * this._dynamicStride * 4 * 4), this.indexBuffer = Jl(this._size), this.geometry.indexBuffer.setDataWithSize(this.indexBuffer, this.indexBuffer.byteLength, true));
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
  function Qy(n) {
    const t = [];
    for (const e in n) {
      const s = n[e];
      t.push(e, s.code, s.dynamic ? "d" : "s");
    }
    return t.join("_");
  }
  var tx = `varying vec2 vUV;
varying vec4 vColor;

uniform sampler2D uTexture;

void main(void){
    vec4 color = texture2D(uTexture, vUV) * vColor;
    gl_FragColor = color;
}`, ex = `attribute vec2 aVertex;
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
`, Ql = `
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
  class nx extends ur {
    constructor() {
      const t = ea.from({
        vertex: ex,
        fragment: tx
      }), e = mi.from({
        fragment: {
          source: Ql,
          entryPoint: "mainFragment"
        },
        vertex: {
          source: Ql,
          entryPoint: "mainVertex"
        }
      });
      super({
        glProgram: t,
        gpuProgram: e,
        resources: {
          uTexture: Tt.WHITE.source,
          uSampler: new Kn({}),
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
  class yr {
    constructor(t, e) {
      this.state = Qi.for2d(), this.localUniforms = new na({
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
      }), this.renderer = t, this.adaptor = e, this.defaultShader = new nx(), this.state = Qi.for2d(), this._managedContainers = new Ps({
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
      return t._gpuData[this.renderer.uid] = new Zy({
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
      a.tx -= l.offset.x, a.ty -= l.offset.y, a.prepend(l.projectionMatrix), o.uResolution = l.resolution, o.uRound = s._roundPixels | t._roundPixels, fd(t.groupColorAlpha, o.uColor, 0), this.adaptor.execute(this, t);
    }
    destroy() {
      this._managedContainers.destroy(), this.renderer = null, this.defaultShader && (this.defaultShader.destroy(), this.defaultShader = null);
    }
  }
  yr.extension = {
    type: [
      lt.CanvasPipes
    ],
    name: "particle"
  };
  class Ed extends yr {
    constructor(t) {
      super(t, new Ky());
    }
  }
  Ed.extension = {
    type: [
      lt.CanvasPipes
    ],
    name: "particle"
  };
  class sx {
    execute(t, e) {
      const s = t.state, i = t.renderer, r = e.shader || t.defaultShader;
      r.resources.uTexture = e.texture._source, r.resources.uniforms = t.localUniforms;
      const o = i.gl, a = t.getBuffers(e);
      i.shader.bind(r), i.state.set(s), i.geometry.bind(a.geometry, r.glProgram);
      const c = a.geometry.indexBuffer.data.BYTES_PER_ELEMENT === 2 ? o.UNSIGNED_SHORT : o.UNSIGNED_INT;
      o.drawElements(o.TRIANGLES, e.particleChildren.length * 6, c, 0);
    }
  }
  class Ad extends yr {
    constructor(t) {
      super(t, new sx());
    }
  }
  Ad.extension = {
    type: [
      lt.WebGLPipes
    ],
    name: "particle"
  };
  class ix {
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
  class kd extends yr {
    constructor(t) {
      super(t, new ix());
    }
  }
  kd.extension = {
    type: [
      lt.WebGPUPipes
    ],
    name: "particle"
  };
  const Md = class No {
    constructor(t) {
      if (t instanceof Tt) this.texture = t, xo(this, No.defaultOptions, {});
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
      this._tint = Yt.shared.setValue(t ?? 16777215).toBgrNumber(), this._updateColor();
    }
    _updateColor() {
      this.color = this._tint + ((this._alpha * 255 | 0) << 24);
    }
  };
  Md.defaultOptions = {
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
  let ti = Md;
  const tc = {
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
  Gt.add(Ad);
  Gt.add(kd);
  Gt.add(Ed);
  const rx = new Me(0, 0, 0, 0), Pd = class Fo extends hr {
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
      for (const l in tc) {
        const c = tc[l], h = e[l];
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
      return rx;
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
  Pd.defaultOptions = {
    dynamicProperties: {
      vertex: false,
      position: true,
      rotation: false,
      uvs: false,
      color: false
    },
    roundPixels: false
  };
  let ox = Pd;
  Gt.add(Ru, Lu);
  var ax = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {};
  function lx(n) {
    return n && n.__esModule && Object.prototype.hasOwnProperty.call(n, "default") ? n.default : n;
  }
  var Id = {
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
    }).call(ax);
  })(Id);
  var cx = Id.exports;
  const ec = lx(cx);
  function xr(n, t) {
    if (n) {
      if (typeof n == "function") return n;
      if (typeof n == "string") return ec[n];
    } else return ec[t];
  }
  class hx {
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
  class dx {
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
  class We {
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
  const ux = {
    removeOnInterrupt: false,
    ease: "linear",
    time: 1e3
  };
  class px extends We {
    constructor(t, e = {}) {
      super(t), this.startWidth = null, this.startHeight = null, this.deltaWidth = null, this.deltaHeight = null, this.width = null, this.height = null, this.time = 0, this.options = Object.assign({}, ux, e), this.options.ease = xr(this.options.ease), this.setupPosition(), this.setupZoom(), this.time = 0;
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
  const fx = {
    sides: "all",
    friction: 0.5,
    time: 150,
    ease: "easeInOutSine",
    underflow: "center",
    bounceBox: null
  };
  class mx extends We {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, fx, e), this.ease = xr(this.options.ease, "easeInOutSine"), this.options.sides ? this.options.sides === "all" ? this.top = this.bottom = this.left = this.right = true : this.options.sides === "horizontal" ? (this.right = this.left = true, this.top = this.bottom = false) : this.options.sides === "vertical" ? (this.left = this.right = false, this.top = this.bottom = true) : (this.top = this.options.sides.indexOf("top") !== -1, this.bottom = this.options.sides.indexOf("bottom") !== -1, this.left = this.options.sides.indexOf("left") !== -1, this.right = this.options.sides.indexOf("right") !== -1) : this.left = this.top = this.right = this.bottom = false;
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
  const gx = {
    left: false,
    right: false,
    top: false,
    bottom: false,
    direction: null,
    underflow: "center"
  };
  class yx extends We {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, gx, e), this.options.direction && (this.options.left = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.right = this.options.direction === "x" || this.options.direction === "all" ? true : null, this.options.top = this.options.direction === "y" || this.options.direction === "all" ? true : null, this.options.bottom = this.options.direction === "y" || this.options.direction === "all" ? true : null), this.parseUnderflow(), this.last = {
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
  const xx = {
    minWidth: null,
    minHeight: null,
    maxWidth: null,
    maxHeight: null,
    minScale: null,
    maxScale: null
  };
  class bx extends We {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, xx, e), this.clamp();
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
  const _x = {
    friction: 0.98,
    bounce: 0.8,
    minSpeed: 0.01
  }, _n = 16;
  class wx extends We {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, _x, e), this.saved = [], this.timeSinceRelease = 0, this.reset(), this.parent.on("moved", (s) => this.handleMoved(s));
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
  const vx = {
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
  class Cx extends We {
    constructor(t, e = {}) {
      super(t), this.windowEventHandlers = [], this.options = Object.assign({}, vx, e), this.moved = false, this.reverse = this.options.reverse ? 1 : -1, this.xDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "x", this.yDirection = !this.options.direction || this.options.direction === "all" || this.options.direction === "y", this.keyIsPressed = false, this.parseUnderflow(), this.mouseButtons(this.options.mouseButtons), this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
  const Sx = {
    speed: 0,
    acceleration: null,
    radius: null
  };
  class Tx extends We {
    constructor(t, e, s = {}) {
      super(t), this.target = e, this.options = Object.assign({}, Sx, s), this.velocity = {
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
  const Ex = {
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
  class Ax extends We {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Ex, e), this.reverse = this.options.reverse ? 1 : -1, this.radiusSquared = typeof this.options.radius == "number" ? Math.pow(this.options.radius, 2) : null, this.resize();
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
  const kx = {
    noDrag: false,
    percent: 1,
    center: null,
    factor: 1,
    axis: "all"
  }, Mx = new Mt();
  class Px extends We {
    constructor(t, e = {}) {
      super(t), this.active = false, this.pinching = false, this.moved = false, this.options = Object.assign({}, kx, e);
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
      const { x: e, y: s } = (this.parent.parent || this.parent).toLocal(t.global, void 0, Mx), i = this.parent.input.touches;
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
  const Ix = {
    topLeft: false,
    friction: 0.8,
    time: 1e3,
    ease: "easeInOutSine",
    interrupt: true,
    removeOnComplete: false,
    removeOnInterrupt: false,
    forceStart: false
  };
  class Rx extends We {
    constructor(t, e, s, i = {}) {
      super(t), this.options = Object.assign({}, Ix, i), this.ease = xr(i.ease, "easeInOutSine"), this.x = e, this.y = s, this.options.forceStart && this.snapStart();
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
  const Lx = {
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
  class $x extends We {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Lx, e), this.ease = xr(this.options.ease), this.xIndependent = false, this.yIndependent = false, this.xScale = 0, this.yScale = 0, this.options.width > 0 && (this.xScale = t.screenWidth / this.options.width, this.xIndependent = true), this.options.height > 0 && (this.yScale = t.screenHeight / this.options.height, this.yIndependent = true), this.xScale = this.xIndependent ? this.xScale : this.yScale, this.yScale = this.yIndependent ? this.yScale : this.xScale, this.options.time === 0 ? (t.container.scale.x = this.xScale, t.container.scale.y = this.yScale, this.options.removeOnComplete && this.parent.plugins.remove("snap-zoom")) : e.forceStart && this.createSnapping();
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
  const Bx = {
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
  class Ox extends We {
    constructor(t, e = {}) {
      super(t), this.options = Object.assign({}, Bx, e), this.keyIsPressed = false, this.options.keyToPress && this.handleKeyPresses(this.options.keyToPress);
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
  const Nx = {
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
  class Rd extends Ht {
    constructor(t) {
      super(), this._disableOnContextMenu = (e) => e.preventDefault(), this.options = {
        ...Nx,
        ...t
      }, this.screenWidth = this.options.screenWidth, this.screenHeight = this.options.screenHeight, this._worldWidth = this.options.worldWidth, this._worldHeight = this.options.worldHeight, this.forceHitArea = this.options.forceHitArea, this.threshold = this.options.threshold, this.options.disableOnContextMenu && this.options.events.domElement.addEventListener("contextmenu", this._disableOnContextMenu), this.options.noTicker || (this.tickerFunction = () => this.update(this.options.ticker.elapsedMS), this.options.ticker.add(this.tickerFunction)), this.input = new hx(this), this.plugins = new dx(this);
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
      return this.plugins.add("snap-zoom", new $x(this, t)), this;
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
      return this.plugins.add("drag", new Cx(this, t)), this;
    }
    clamp(t) {
      return this.plugins.add("clamp", new yx(this, t)), this;
    }
    decelerate(t) {
      return this.plugins.add("decelerate", new wx(this, t)), this;
    }
    bounce(t) {
      return this.plugins.add("bounce", new mx(this, t)), this;
    }
    pinch(t) {
      return this.plugins.add("pinch", new Px(this, t)), this;
    }
    snap(t, e, s) {
      return this.plugins.add("snap", new Rx(this, t, e, s)), this;
    }
    follow(t, e) {
      return this.plugins.add("follow", new Tx(this, t, e)), this;
    }
    wheel(t) {
      return this.plugins.add("wheel", new Ox(this, t)), this;
    }
    animate(t) {
      return this.plugins.add("animate", new px(this, t)), this;
    }
    clampZoom(t) {
      return this.plugins.add("clamp-zoom", new bx(this, t)), this;
    }
    mouseEdges(t) {
      return this.plugins.add("mouse-edges", new Ax(this, t)), this;
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
  const Fx = 32, Wo = /* @__PURE__ */ new Set([
    "transport-belt",
    "fast-transport-belt",
    "express-transport-belt"
  ]), Li = /* @__PURE__ */ new Set([
    "underground-belt",
    "fast-underground-belt",
    "express-underground-belt"
  ]), to = /* @__PURE__ */ new Set([
    "splitter",
    "fast-splitter",
    "express-splitter"
  ]), Wx = /* @__PURE__ */ new Set([
    "inserter",
    "fast-inserter",
    "long-handed-inserter"
  ]), Gx = /* @__PURE__ */ new Set([
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
  ]), nc = {
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
  function $i(n, t, e, s, i, r) {
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
  const zx = 9;
  function Dx(n) {
    const t = {
      nodes: /* @__PURE__ */ new Map(),
      outEdges: /* @__PURE__ */ new Map(),
      inEdges: /* @__PURE__ */ new Map(),
      tileToAnchor: /* @__PURE__ */ new Map(),
      entityMap: /* @__PURE__ */ new Map()
    };
    for (const e of n.entities) t.entityMap.set(Qe(e.x ?? 0, e.y ?? 0), e);
    for (const e of n.entities) {
      if (!Wo.has(e.name) && !Li.has(e.name) && !to.has(e.name)) continue;
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
          $i(t, e, c, "both", p > 0, false);
        }
      } else if (Li.has(s.name)) if (s.io_type === "input") for (let c = 1; c <= zx; c++) {
        const h = t.entityMap.get(Qe(i + a * c, r + l * c));
        if (h) {
          if (Li.has(h.name) && h.name === s.name && h.io_type === "input" && h.direction === o) break;
          if (Li.has(h.name) && h.name === s.name && h.io_type === "output" && h.direction === o) {
            const d = t.tileToAnchor.get(Qe(h.x ?? 0, h.y ?? 0));
            d !== void 0 && $i(t, e, d, "both", false, false);
            break;
          }
        }
      }
      else {
        const c = t.tileToAnchor.get(Qe(i + a, r + l));
        c !== void 0 && c !== e && $i(t, e, c, "both", false, false);
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
          f !== void 0 && f !== e && $i(t, e, f, "both", false, true);
        }
      }
    }
    return t;
  }
  function Hx(n, t) {
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
  function jx(n, t) {
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
        h && Gx.has(h.name) && e.add(c);
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
  function sc(n, t) {
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
  function Vx(n, t, e, s, i, r, o) {
    const a = s - t, l = i - e, c = Math.sqrt(a * a + l * l);
    if (c === 0) return;
    const h = a / c, d = l / c;
    let u = 0, p = true;
    for (; u < c; ) {
      const f = Math.min(p ? r : o, c - u);
      p && n.moveTo(t + h * u, e + d * u).lineTo(t + h * (u + f), e + d * (u + f)).stroke(), u += f, p = !p;
    }
  }
  function Yx(n, t, e, s, i, r, o, a, l) {
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
  function Xx(n, t, e, s, i) {
    const r = Fx, o = r / 2;
    for (const l of e) {
      if (t.has(l)) continue;
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = nc[c.name] ?? 14733424, [p, f] = sr(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.05
      }), n.setStrokeStyle({
        width: 1.5,
        color: u,
        alpha: 0.28,
        cap: "round"
      });
      const y = sc(l, i);
      if (y) {
        const w = Go[`${y.inDir}_${y.outDir}`], x = w.cx(h, d, r), _ = w.cy(h, d, r);
        Yx(n, x, _, o, w.startAngle, w.endAngle, w.anticlockwise, 5 / o, 3 / o);
      } else Vx(n, g - p * r * 0.45, m - f * r * 0.45, g + p * r * 0.45, m + f * r * 0.45, 5, 3);
    }
    for (const l of t) {
      const c = i.nodes.get(l);
      if (!c) continue;
      const h = (c.x ?? 0) * r, d = (c.y ?? 0) * r, u = nc[c.name] ?? 14733424, [p, f] = sr(c.direction), g = h + r / 2, m = d + r / 2;
      n.rect(h, d, r, r).fill({
        color: u,
        alpha: 0.2
      }), n.setStrokeStyle({
        width: 2,
        color: u,
        alpha: 0.85,
        cap: "round"
      });
      const y = sc(l, i);
      if (y) {
        const w = Go[`${y.inDir}_${y.outDir}`], x = w.cx(h, d, r), _ = w.cy(h, d, r), v = x + o * Math.cos(w.startAngle), b = _ + o * Math.sin(w.startAngle);
        n.moveTo(v, b).arc(x, _, o, w.startAngle, w.endAngle, w.anticlockwise).stroke();
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
  let qx, Kx, $n, Ld, $d, Jx, ic, rc, Zx, Qx, t0;
  S = 32;
  qx = {
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
  Kx = 4872810;
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
  Ld = {
    inserter: 6983230,
    "fast-inserter": 4886736,
    "long-handed-inserter": 13647936,
    "stack-inserter": 4114794,
    "bulk-inserter": 3123354
  };
  $d = 9079434;
  Jx = 6974058;
  ic = 2039583;
  rc = 12623920;
  Zx = 2762e3;
  Qx = 0.35;
  t0 = {
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
  function e0(n, t) {
    const e = n >> 16 & 255, s = n >> 8 & 255, i = n & 255, r = 0.21 * e + 0.72 * s + 0.07 * i, o = Math.round(r + (e - r) * t), a = Math.round(r + (s - r) * t), l = Math.round(r + (i - r) * t);
    return o << 16 | a << 8 | l;
  }
  const oc = Object.fromEntries(Object.entries(t0).map(([n, t]) => [
    n,
    e0(t, Qx)
  ]));
  function n0(n, t, e) {
    const s = t * Math.min(e, 1 - e), i = (r) => {
      const o = (r + n * 12) % 12;
      return Math.round((e - s * Math.max(-1, Math.min(o - 3, 9 - o, 1))) * 255);
    };
    return i(0) << 16 | i(8) << 8 | i(4);
  }
  let Bd = true;
  function Bi(n) {
    Bd = n;
  }
  let zo = /* @__PURE__ */ new Map();
  function s0(n) {
    return zo.get(n);
  }
  function Od(n) {
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
    if (!Bd) return 7829367;
    if (!n) return 6710886;
    if (n in oc) return oc[n];
    let t = 0;
    for (let s = 0; s < n.length; s++) t = (t << 5) - t + n.charCodeAt(s) | 0;
    const e = Math.abs(t) % 30 * 12;
    return n0(e / 360, 0.2, 0.48);
  }
  const Pe = {
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
  function pe(n) {
    return n.split("-").map((t) => t.charAt(0).toUpperCase() + t.slice(1)).join(" ");
  }
  let Ae, br, sn, se, li, ga;
  Ae = new Set(Object.keys(Pe));
  br = new Set(Object.keys(Ld));
  sn = new Set(Object.keys($n).filter((n) => !n.includes("underground") && !n.includes("splitter")));
  ue = new Set(Object.keys($n).filter((n) => n.includes("underground")));
  se = new Set(Object.keys($n).filter((n) => n.includes("splitter")));
  li = /* @__PURE__ */ new Set([
    "pipe",
    "pipe-to-ground"
  ]);
  ga = /* @__PURE__ */ new Set([
    "medium-electric-pole",
    "small-electric-pole"
  ]);
  function gi(n) {
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
  function ya(n) {
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
  function Nd(n, t) {
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
      if (!d || !(sn.has(d.name) || ue.has(d.name) && d.io_type === "output" || se.has(d.name))) continue;
      const [p, f] = In(d.direction), g = se.has(d.name) ? c : d.x ?? 0, m = se.has(d.name) ? h : d.y ?? 0;
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
  function ac(n, t) {
    const e = Math.round((n >> 16 & 255) * t), s = Math.round((n >> 8 & 255) * t), i = Math.round((n & 255) * t);
    return e << 16 | s << 8 | i;
  }
  const ci = 3, Fd = 3815994, xa = 5592405, ba = 0.9, Oi = S * (1 - ba) / 2, Wd = S * ba;
  function _a(n, t) {
    const e = new ft(), s = S, i = Wd, [, r] = $n[n.name] ?? [
      11046960,
      14733424
    ], o = Rn(n.carries);
    if (t) i0(e, s, r, n.direction, t, o);
    else {
      e.rect(Oi, Oi, i, i).fill(Fd), e.setStrokeStyle({
        width: 1,
        color: xa,
        alignment: 0
      }), e.rect(Oi, Oi, i, i).stroke();
      const a = new ft();
      a.x = s / 2, a.y = s / 2, a.rotation = gi(n.direction), a.rect(-i / 2, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(1, -i / 2, i / 2 - 1, i).fill({
        color: o,
        alpha: 0.45
      }), a.rect(-1, -i / 2, 2, i).fill(657930), r0(a, i, r), e.addChild(a);
    }
    return e;
  }
  function i0(n, t, e, s, i, r) {
    const o = new ft();
    o.x = t / 2, o.y = t / 2, o.rotation = gi(s), o.scale.set(ba);
    const a = t / 2, c = (i.turn === "cw" ? 1 : -1) * a, h = -a, d = i.turn === "ccw" ? 0 : Math.PI / 2, u = i.turn === "ccw" ? Math.PI / 2 : Math.PI;
    o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().fill(Fd), o.setStrokeStyle({
      width: 1,
      color: xa,
      alignment: 0
    }), o.moveTo(c, h).arc(c, h, t, d, u, false).closePath().stroke();
    const p = t * 0.5, f = 1.5;
    o.moveTo(c, h).arc(c, h, p - f, d, u, false).closePath().fill({
      color: r,
      alpha: 0.45
    });
    const g = Math.cos(d), m = Math.sin(d), y = Math.cos(u), w = Math.sin(u);
    o.moveTo(c + (p + f) * g, h + (p + f) * m).lineTo(c + t * g, h + t * m).arc(c, h, t, d, u, false).lineTo(c + (p + f) * y, h + (p + f) * w).arc(c, h, p + f, u, d, true).closePath().fill({
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
    const R = Math.PI / 2, A = i.turn === "cw" ? Math.PI : 0;
    for (const k of [
      0.6
    ]) {
      const L = R + k * (A - R), H = i.turn === "cw" ? L - b : L + b, W = c + v * Math.cos(L), F = h + v * Math.sin(L), I = c + C * Math.cos(H), z = h + C * Math.sin(H), J = c + T * Math.cos(H), q = h + T * Math.sin(H);
      o.moveTo(I, z).lineTo(W, F).lineTo(J, q).stroke();
    }
    n.addChild(o);
  }
  function r0(n, t, e) {
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
  function Gd(n) {
    const t = new ft(), e = S, [, s] = $n[n.name] ?? [
      11046960,
      14733424
    ], i = n.io_type === "input", r = e / 2, o = i ? 1 : -1, a = new ft();
    a.x = r, a.y = r, a.rotation = gi(n.direction);
    const l = Rn(n.carries), c = Wd / 2, h = e * 0.25, d = o * r, u = 0;
    a.moveTo(-c, d).lineTo(c, d).lineTo(h, u).lineTo(-h, u).closePath().fill({
      color: l,
      alpha: 0.7
    }), a.setStrokeStyle({
      width: 1,
      color: xa,
      alpha: 0.8
    }), a.moveTo(-c, d).lineTo(-h, u).lineTo(h, u).lineTo(c, d).stroke();
    const p = e * 0.38, f = e * 0.3, g = o * e * 0.22, m = g - f / 2, y = g + f / 2;
    return a.moveTo(0, m).lineTo(p / 2, y).lineTo(-p / 2, y).closePath().fill(s), t.addChild(a), t;
  }
  function zd(n) {
    const t = new ft(), [e, s] = $n[n.name] ?? [
      11046960,
      14733424
    ], i = n.direction === "North" || n.direction === "South", r = i ? S * 2 - 1 : S - 1, o = i ? S - 1 : S * 2 - 1, a = i ? r / 2 : o / 2, l = Math.max(2, Math.min(r, o) * 0.18);
    t.roundRect(0, 0, r, o, ci).fill(e), t.roundRect(0, 0, r, o, ci).fill({
      color: Rn(n.carries),
      alpha: 0.3
    }), i ? t.rect(a - l / 2, 0, l, o).fill(ac(e, 0.5)) : t.rect(0, a - l / 2, r, l).fill(ac(e, 0.5));
    const c = gi(n.direction), h = a * 0.25, d = Math.max(1, a * 0.12);
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
  function Dd(n) {
    const t = new ft(), e = S - 1, s = n.carries ? Rn(n.carries) : Ld[n.name] ?? 6983230;
    t.roundRect(0, 0, e, e, ci).fill(2767402);
    const i = new ft();
    i.x = e / 2, i.y = e / 2, i.rotation = gi(n.direction), i.circle(0, e * 0.2, e * 0.15).fill(4473924);
    const r = Math.max(1.5, e * 0.12);
    i.setStrokeStyle({
      width: r,
      color: s,
      cap: "round"
    }), i.moveTo(0, e * 0.2).lineTo(0, -e * 0.35).stroke();
    const o = -e * 0.35, a = e * 0.18;
    return i.moveTo(-a, o - a * 0.6).lineTo(0, o).lineTo(a, o - a * 0.6).stroke(), t.addChild(i), t;
  }
  const wa = 1, va = 2, Ca = 4, Sa = 8;
  function o0(n) {
    const [t, e] = In(n.direction);
    return [
      -t,
      -e
    ];
  }
  function Hd(n, t, e) {
    if (n.name === "pipe") return true;
    if (n.name === "pipe-to-ground") {
      const [s, i] = o0(n);
      return -t === s && -e === i;
    }
    return false;
  }
  function Ud(n, t) {
    const e = new ft(), s = S - 1, i = n.name === "pipe-to-ground", r = i ? Jx : $d;
    e.roundRect(0, 0, s, s, ci).fill(ic);
    const o = s / 2, a = s / 2, l = Math.max(2, s * 0.4);
    if (i) {
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      });
      const [c, h] = In(n.direction);
      e.moveTo(o, a).lineTo(o - c * s / 2, a - h * s / 2).stroke(), e.circle(o, a, l * 0.4).fill(r), e.circle(o, a, l * 0.25).fill(ic);
    } else if (t === 0) e.circle(o, a, l * 0.4).fill(r);
    else {
      const c = !!(t & wa), h = !!(t & va), d = !!(t & Ca), u = !!(t & Sa), p = (c ? 1 : 0) + (h ? 1 : 0) + (d ? 1 : 0) + (u ? 1 : 0);
      e.setStrokeStyle({
        width: l,
        color: r,
        cap: "round"
      }), p === 1 ? (c ? e.moveTo(o, a).lineTo(o, 0).stroke() : h ? e.moveTo(o, a).lineTo(s, a).stroke() : d ? e.moveTo(o, a).lineTo(o, s).stroke() : e.moveTo(o, a).lineTo(0, a).stroke(), e.circle(o, a, l * 0.4).fill(r)) : c && d && !h && !u ? e.moveTo(o, 0).lineTo(o, s).stroke() : h && u && !c && !d ? e.moveTo(0, a).lineTo(s, a).stroke() : p === 2 ? c && h ? e.moveTo(o, 0).quadraticCurveTo(o, a, s, a).stroke() : h && d ? e.moveTo(s, a).quadraticCurveTo(o, a, o, s).stroke() : d && u ? e.moveTo(o, s).quadraticCurveTo(o, a, 0, a).stroke() : e.moveTo(0, a).quadraticCurveTo(o, a, o, 0).stroke() : p === 3 ? u ? d ? h ? (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, s).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(0, a).stroke()) : (e.moveTo(0, a).lineTo(s, a).stroke(), e.moveTo(o, a).lineTo(o, 0).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(o, a).lineTo(s, a).stroke()) : (e.moveTo(o, 0).lineTo(o, s).stroke(), e.moveTo(0, a).lineTo(s, a).stroke());
    }
    return e;
  }
  function jd() {
    const n = new ft(), t = S - 1;
    n.roundRect(0, 0, t, t, ci).fill(Zx);
    const e = t / 2, s = t / 2, i = t * 0.38, r = Math.max(1.5, t * 0.2);
    return n.rect(e - r / 2, s - i, r, i * 2).fill(rc), n.rect(e - i, s - r / 2, i * 2, r).fill(rc), n.circle(e, s, r * 0.6).fill(14729280), n;
  }
  function Vd(n) {
    const t = new ft(), [e, s] = Pe[n.name] ?? [
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
        const w = Math.min(y + o, f);
        t.moveTo(h + g * y, d + m * y).lineTo(h + g * w, d + m * w).stroke(), y = w + a;
      }
    }
    const l = 1.8, c = Do(`/spaghettio/entity-frames/${n.name}.png`);
    if (c) {
      const h = new Ne(c), d = S / a0;
      h.scale.set(d * l), h.x = -i * (l - 1) / 2, h.y = -r * (l - 1) / 2, t.addChild(h);
    } else {
      const h = Do(`/spaghettio/icons/${n.name}.png`);
      if (h) {
        const d = new Ne(h), u = Math.min(i, r) * 0.8 * l;
        d.width = u, d.height = u, d.x = (i - u) / 2, d.y = (r - u) / 2, t.addChild(d);
      } else {
        const d = qx[n.name] ?? Kx;
        t.roundRect(2, 2, i - 4, r - 4, 3).fill({
          color: d,
          alpha: 0.5
        });
      }
    }
    return t;
  }
  function Yd() {
    const n = new ft(), t = S - 1;
    return n.rect(0, 0, t, t).fill(4872810), n.setStrokeStyle({
      width: 1,
      color: 0,
      alpha: 0.4
    }), n.rect(0, 0, t, t).stroke(), n;
  }
  const a0 = 64;
  async function l0(n) {
    const t = "/spaghettio/", e = [
      ...n.map((s) => `${t}icons/${s}.png`),
      ...n.map((s) => `${t}entity-frames/${s}.png`)
    ];
    await Promise.allSettled(e.map((s) => $e.load(s)));
  }
  async function Xd(n) {
    const t = "/spaghettio/";
    await Promise.allSettled(n.map((e) => $e.load(`${t}icons/${e}.png`)));
  }
  function c0(n) {
    const t = /* @__PURE__ */ new Set();
    for (const e of n) e.carries && t.add(e.carries);
    return Array.from(t);
  }
  function Do(n) {
    return ne.has(n) ? $e.get(n) ?? null : null;
  }
  const h0 = {
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
  function qd(n) {
    const t = h0[n.name];
    if (!t) return [];
    const e = n.mirror ?? false;
    return t.filter(([, , , s]) => s === "always" || s === "default" && !e || s === "mirror" && e);
  }
  function Kd(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of qd(e)) if (s + r === n && i + o === t) return true;
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
    if (t.tileMap.set(`${e},${s}`, n), se.has(n.name)) {
      const [i, r] = ya(n.direction);
      t.tileMap.set(`${e + i},${s + r}`, n);
    }
    if (Ae.has(n.name)) {
      const [i, r] = Pe[n.name] ?? [
        1,
        1
      ];
      for (let o = 0; o < r; o++) for (let a = 0; a < i; a++) t.machineByTile.set(`${e + a},${s + o}`, n);
    }
  };
  Fn = function(n, t) {
    let e;
    if (sn.has(n.name)) e = _a(n, Nd(n, t.tileMap));
    else if (ue.has(n.name)) e = Gd(n);
    else if (se.has(n.name)) e = zd(n);
    else if (br.has(n.name)) e = Dd(n);
    else if (li.has(n.name)) {
      let s = 0;
      if (n.name === "pipe") {
        const i = n.x ?? 0, r = n.y ?? 0;
        for (const [o, a, l] of [
          [
            0,
            -1,
            wa
          ],
          [
            1,
            0,
            va
          ],
          [
            0,
            1,
            Ca
          ],
          [
            -1,
            0,
            Sa
          ]
        ]) {
          const c = `${i + o},${r + a}`, h = t.tileMap.get(c);
          if (h && Hd(h, o, a)) {
            s |= l;
            continue;
          }
          const d = t.machineByTile.get(c);
          d && Kd(i, r, d) && (s |= l);
        }
      }
      e = Ud(n, s);
    } else ga.has(n.name) ? e = jd() : Ae.has(n.name) ? e = Vd(n) : e = Yd();
    return n.quality && n.quality !== "normal" && f0(e, n.quality), e.x = (n.x ?? 0) * S, e.y = (n.y ?? 0) * S, e;
  };
  const d0 = {
    uncommon: 5229135,
    rare: 5213130,
    epic: 10899402,
    legendary: 15246141
  }, u0 = [
    "quality-uncommon",
    "quality-rare",
    "quality-epic",
    "quality-legendary"
  ];
  async function p0() {
    const n = "/spaghettio/";
    await Promise.allSettled(u0.map((t) => $e.load(`${n}icons/${t}.png`)));
  }
  function f0(n, t) {
    const e = n.getLocalBounds(), s = 1.5, i = Do(`/spaghettio/icons/quality-${t}.png`);
    if (i) {
      const c = S * 0.42, h = new Ne(i);
      h.width = c, h.height = c, h.x = e.minX + s, h.y = e.maxY - c - s, n.addChild(h);
      return;
    }
    const r = d0[t];
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
  Cv = function(n, t, e = 8) {
    if (!ue.has(n.name) || n.io_type !== "input") return null;
    const [s, i] = In(n.direction), r = n.x ?? 0, o = n.y ?? 0;
    for (let a = 1; a <= e; a++) {
      const l = t.get(`${r + s * a},${o + i * a}`);
      if (l) {
        if (ue.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "input") return null;
        if (ue.has(l.name) && l.name === n.name && l.direction === n.direction && l.io_type === "output") {
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
  ei = function(n, t, e, s, i, r) {
    t.removeChildren();
    const o = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (o.set(`${f.x ?? 0},${f.y ?? 0}`, f), se.has(f.name)) {
      const [g, m] = ya(f.direction);
      o.set(`${(f.x ?? 0) + g},${(f.y ?? 0) + m}`, f);
    }
    if (r) for (const f of r) {
      const g = `${f.x ?? 0},${f.y ?? 0}`;
      o.has(g) || o.set(g, f);
    }
    const a = /* @__PURE__ */ new Map();
    for (const f of n.entities) if (Ae.has(f.name)) {
      const [g, m] = Pe[f.name] ?? [
        1,
        1
      ], y = f.x ?? 0, w = f.y ?? 0;
      for (let x = 0; x < m; x++) for (let _ = 0; _ < g; _++) a.set(`${y + _},${w + x}`, f);
    }
    {
      const f = /* @__PURE__ */ new Map();
      for (const m of n.entities) ue.has(m.name) && f.set(`${m.x ?? 0},${m.y ?? 0}`, m);
      const g = 8;
      for (const m of n.entities) {
        if (!ue.has(m.name) || m.io_type !== "input") continue;
        const [y, w] = In(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 1; v <= g; v++) {
          const b = f.get(`${x + y * v},${_ + w * v}`);
          if (b) {
            if (ue.has(b.name) && b.name === m.name && b.direction === m.direction && b.io_type === "input") break;
            if (ue.has(b.name) && b.name === m.name && b.direction === m.direction && b.io_type === "output") {
              const [C] = $n[m.name] ?? [
                11046960,
                14733424
              ], T = new ft(), R = Math.abs(y) > 0;
              for (let A = 1; A < v; A++) {
                const k = (x + y * A) * S, L = (_ + w * A) * S;
                R ? T.rect(k, L + S * 0.25, S, S * 0.5).fill({
                  color: C,
                  alpha: 0.25
                }) : T.rect(k + S * 0.25, L, S * 0.5, S).fill({
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
        const [y, w] = In(m.direction), x = m.x ?? 0, _ = m.y ?? 0;
        for (let v = 2; v <= g; v++) {
          const b = f.get(`${x + y * v},${_ + w * v}`);
          if (!b) continue;
          const [C, T] = In(b.direction);
          if (b.io_type !== "output" || C !== -y || T !== -w) break;
          const R = new ft();
          R.setStrokeStyle({
            width: 2,
            color: $d,
            alpha: 0.55,
            cap: "round"
          });
          const A = (x + 0.5 + y * 0.5) * S, k = (_ + 0.5 + w * 0.5) * S, L = (v - 1) * S, H = 5, W = 3;
          let F = 0;
          for (; F < L; ) {
            const I = Math.min(F + H, L);
            R.moveTo(A + y * F, k + w * F).lineTo(A + y * I, k + w * I).stroke(), F = I + W;
          }
          t.addChild(R);
          break;
        }
      }
    }
    const l = /* @__PURE__ */ new Map(), c = [], h = /* @__PURE__ */ new Map();
    for (const f of n.entities) {
      let g;
      if (sn.has(f.name)) g = _a(f, Nd(f, o));
      else if (ue.has(f.name)) g = Gd(f);
      else if (se.has(f.name)) g = zd(f);
      else if (br.has(f.name)) g = Dd(f);
      else if (li.has(f.name)) {
        let y = 0;
        if (f.name === "pipe") {
          const w = f.x ?? 0, x = f.y ?? 0;
          for (const [_, v, b] of [
            [
              0,
              -1,
              wa
            ],
            [
              1,
              0,
              va
            ],
            [
              0,
              1,
              Ca
            ],
            [
              -1,
              0,
              Sa
            ]
          ]) {
            if (v === -1 && x + v < 0) {
              y |= b;
              continue;
            }
            const C = `${w + _},${x + v}`, T = o.get(C);
            if (T && Hd(T, _, v)) {
              y |= b;
              continue;
            }
            const R = a.get(C);
            R && Kd(w, x, R) && (y |= b);
          }
        }
        g = Ud(f, y);
      } else ga.has(f.name) ? g = jd() : Ae.has(f.name) ? g = Vd(f) : g = Yd();
      g.x = (f.x ?? 0) * S, g.y = (f.y ?? 0) * S, s && (g.eventMode = "static", g.cursor = "pointer", g.on("click", () => s(f)));
      const m = lc(f);
      m && (l.has(m) || l.set(m, []), l.get(m).push(g)), h.set(g, `${f.x ?? 0},${f.y ?? 0}`), c.push(g), t.addChild(g), i == null ? void 0 : i(f, [
        g
      ]);
    }
    const d = Dx(n);
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
        const { downstream: y, upstream: w } = Hx(m, d), x = /* @__PURE__ */ new Set([
          ...y,
          ...w
        ]), _ = Ux(x, d.entityMap), v = jx(_, d.entityMap);
        for (const b of c) {
          const C = h.get(b);
          if (!C) {
            b.alpha = 0.15;
            continue;
          }
          x.has(C) ? b.alpha = 0.5 : _.has(C) ? b.alpha = 0.9 : v.has(C) ? b.alpha = 0.75 : b.alpha = 0.15;
        }
        u = new ft(), Xx(u, y, w, m, d), t.addChild(u);
      },
      clearHighlight() {
        p();
      },
      chainKey: lc
    };
  };
  function lc(n) {
    return n.carries ? n.carries : n.recipe ? n.recipe : null;
  }
  const hi = 4096, gt = 128, tn = hi / gt;
  let kn = null;
  const di = /* @__PURE__ */ new Map();
  let nn = 0, ui = null;
  function m0(n) {
    ui = n;
  }
  function hn(n, t, e, s) {
    const i = di.get(n);
    if (i) return i;
    if (!ui) return console.warn("[atlas] getEntityTexture called before initAtlas; returning blank texture"), Tt.EMPTY;
    kn || (kn = gr.create({
      width: hi,
      height: hi
    })), nn >= tn * tn && (console.warn("[atlas] atlas is full \u2014 variant will reuse slot 0:", n), nn = 0);
    const r = nn % tn, o = Math.floor(nn / tn), a = r * gt, l = o * gt;
    nn++;
    const c = new ft();
    s(c);
    const h = new vt(1, 0, 0, 1, a, l);
    ui.render({
      container: c,
      target: kn,
      transform: h,
      clear: false
    }), c.destroy({
      children: true
    });
    const d = new Dt(a, l, gt, gt), u = new Tt({
      source: kn.source,
      frame: d
    });
    return di.set(n, u), u;
  }
  function cc(n, t, e, s) {
    const i = di.get(n);
    if (i) return i;
    if (!ui) return console.warn("[atlas] getMultiCellTexture called before initAtlas; returning blank texture"), Tt.EMPTY;
    kn || (kn = gr.create({
      width: hi,
      height: hi
    }));
    const r = nn % tn;
    r + t > tn && (nn += tn - r);
    const a = nn % tn, l = Math.floor(nn / tn), c = a * gt, h = l * gt;
    nn = (l + e) * tn;
    const d = t * gt, u = e * gt, p = new ft();
    s(p, d, u);
    const f = new vt(1, 0, 0, 1, c, h);
    ui.render({
      container: p,
      target: kn,
      transform: f,
      clear: false
    }), p.destroy({
      children: true
    });
    const g = new Dt(c, h, d, u), m = new Tt({
      source: kn.source,
      frame: g
    });
    return di.set(n, m), m;
  }
  function Ho(n) {
    const t = `icon:${n}`, e = di.get(t);
    if (e) return e;
    const s = `/spaghettio/icons/${n}.png`;
    if (ne.has(s)) {
      const r = $e.get(s);
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
  function g0(n, t, e = "straight") {
    return `belt:${n}:${t}:${e}`;
  }
  function y0(n) {
    return `pipe:${n}`;
  }
  function x0(n, t, e) {
    return `ugbelt:${n}:${t}:${e}`;
  }
  function b0(n, t) {
    return `splitter:${n}:${t}`;
  }
  function _0(n, t) {
    return `inserter:${n}:${t}`;
  }
  function w0(n) {
    return `machine:${n}`;
  }
  function v0(n) {
    return `pole:${n}`;
  }
  function C0(n) {
    return `ptg:${n}`;
  }
  const en = 3200;
  let Jd = null, Zd = null, Qd = null;
  function Ye() {
    Jd == null ? void 0 : Jd();
  }
  function wr() {
    Zd == null ? void 0 : Zd();
  }
  function qn() {
    Qd == null ? void 0 : Qd();
  }
  async function S0(n) {
    const t = new sa();
    await t.init({
      resizeTo: n,
      background: 1973790,
      antialias: true,
      autoStart: false,
      sharedTicker: false
    }), m0(t.renderer), t.ticker.add(() => t.render(), null, si.LOW), n.appendChild(t.canvas), t.canvas.addEventListener("contextmenu", (h) => h.preventDefault());
    const e = new Rd({
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
    Jd = r, Zd = o, Qd = a, e.on("moved", r), e.on("zoomed", r);
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
  const Ni = 32, hc = 2763306, dc = 3815994;
  function T0(n) {
    const t = new ft();
    return n.addChildAt(t, 0), t;
  }
  function Wn(n, t, e, s = 1) {
    if (n.clear(), t <= 0 || e <= 0) return;
    const i = Math.max(0, Math.min(1, s));
    if (i === 0) return;
    const r = t * Ni, a = e * Ni * i;
    for (let l = 0; l <= t; l++) {
      const c = l * Ni, h = l % 10 === 0;
      n.moveTo(c, 0).lineTo(c, a).stroke({
        width: h ? 1.5 : 1,
        color: h ? dc : hc
      });
    }
    for (let l = 0; l <= e; l++) {
      const c = l * Ni;
      if (c > a) break;
      const h = l % 10 === 0;
      n.moveTo(0, c).lineTo(r, c).stroke({
        width: h ? 1.5 : 1,
        color: h ? dc : hc
      });
    }
  }
  const qi = {
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
  }, E0 = 8947848;
  function tu(n) {
    const t = n >> 16 & 255, e = n >> 8 & 255, s = n & 255;
    return `rgb(${t},${e},${s})`;
  }
  const A0 = [
    {
      name: "transport-belt",
      color: qi["transport-belt"],
      throughput: 15
    },
    {
      name: "fast-transport-belt",
      color: qi["fast-transport-belt"],
      throughput: 30
    },
    {
      name: "express-transport-belt",
      color: qi["express-transport-belt"],
      throughput: 45
    }
  ];
  function eu(n) {
    for (const t of A0) if (n <= t.throughput) return t;
    return null;
  }
  const k0 = 240, uc = 80, Ta = 180, M0 = 60, pc = 100, fc = 100, mc = "production-graph";
  function P0(n) {
    return n <= 15 ? 13153632 : n <= 30 ? 14700624 : n <= 45 ? 5284064 : 16711816;
  }
  const gc = new Fe({
    fontSize: 13,
    fontWeight: "bold",
    fill: 14737632,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: Ta - 12
  }), yc = new Fe({
    fontSize: 11,
    fill: 10280190,
    fontFamily: "sans-serif",
    wordWrap: true,
    wordWrapWidth: Ta - 12
  }), xc = new Fe({
    fontSize: 10,
    fill: 16777215,
    fontFamily: "sans-serif"
  });
  function Ds(n, t) {
    const e = n.getChildByName(mc);
    e && (e.destroy({
      children: true
    }), n.removeChild(e));
    const s = new Ht();
    if (s.label = mc, n.addChild(s), !t || t.machines.length === 0) return s;
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
        x: pc + (x + a) * k0,
        y: fc + b * uc,
        w: Ta,
        h: M0,
        machine: v
      });
    });
    const p = /* @__PURE__ */ new Map();
    c.forEach((x, _) => {
      p.set(x, {
        x: pc,
        y: fc + _ * uc,
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
        const T = P0(v.rate), R = C.x + C.w, A = C.y + C.h * 2 / 3, k = _.x, L = _.y + _.h / 3, H = (R + k) / 2;
        f.moveTo(R, A).lineTo(H, A).lineTo(H, L).lineTo(k, L).stroke({
          color: T,
          width: 2,
          alpha: 0.85
        });
        const W = `${v.rate.toFixed(1)}/s ${v.item}`, F = (R + H) / 2, I = A - 14, z = cn.measureText(W, xc), J = new ft();
        J.rect(F - 2, I - 1, z.width + 4, z.height + 2).fill({
          color: 1973790,
          alpha: 0.7
        }), s.addChild(J);
        const q = new Ve({
          text: W,
          style: xc
        });
        q.position.set(F, I), s.addChild(q);
      }
    }
    for (const x of u.values()) {
      const _ = x.machine, v = qi[_.entity] ?? E0, b = new ft();
      b.rect(x.x, x.y, x.w, x.h).fill({
        color: v,
        alpha: 0.6
      }).stroke({
        color: v,
        width: 2
      }), s.addChild(b);
      const C = new Ve({
        text: `${_.count.toFixed(1)} \xD7 ${_.entity}`,
        style: gc
      });
      C.position.set(x.x + 6, x.y + 6), s.addChild(C);
      const T = new Ve({
        text: _.recipe,
        style: yc
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
      const T = new Ve({
        text: b,
        style: gc
      });
      T.position.set(_.x + 6, _.y + 4), s.addChild(T);
      const R = new Ve({
        text: x,
        style: yc
      });
      R.position.set(_.x + 6, _.y + 20), s.addChild(R);
    }
    let g = 1 / 0, m = -1 / 0, y = 1 / 0, w = -1 / 0;
    for (const x of [
      ...u.values(),
      ...p.values()
    ]) x.x < g && (g = x.x), x.x + x.w > m && (m = x.x + x.w), x.y < y && (y = x.y), x.y + x.h > w && (w = x.y + x.h);
    return n.moveCenter((g + m) / 2, (y + w) / 2), s;
  }
  function ie(n, t) {
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
  function fe(n, t, e) {
    if (t === e) return;
    let s = n.get(t);
    s || (s = [], n.set(t, s)), s.includes(e) || s.push(e);
    let i = n.get(e);
    i || (i = [], n.set(e, i)), i.includes(t) || i.push(t);
  }
  function I0(n) {
    const t = /* @__PURE__ */ new Map();
    for (const e of n.entities) {
      const s = e.x ?? 0, i = e.y ?? 0, r = we(e);
      if (t.set(ie(s, i), r), se.has(e.name)) {
        const o = e.direction === "North" || e.direction === "South";
        t.set(ie(s + (o ? 1 : 0), i + (o ? 0 : 1)), r);
      }
      if (Ae.has(e.name)) {
        const [o, a] = Pe[e.name] ?? [
          1,
          1
        ];
        for (let l = 0; l < a; l++) for (let c = 0; c < o; c++) c === 0 && l === 0 || t.set(ie(s + c, i + l), r);
      }
    }
    return t;
  }
  const R0 = 9;
  function nu(n) {
    const t = /* @__PURE__ */ new Map(), e = I0(n);
    for (const o of n.entities) {
      const a = we(o);
      t.has(a) || t.set(a, []);
    }
    const s = /* @__PURE__ */ new Map();
    for (const o of n.entities) s.set(ie(o.x ?? 0, o.y ?? 0), o);
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
        const u = e.get(ie(a + h, l + d));
        u && fe(t, c, u);
        const p = e.get(ie(a - h, l - d));
        p && fe(t, c, p);
        for (const [f, g] of i) {
          if (f === h && g === d || f === -h && g === -d) continue;
          const m = s.get(ie(a + f, l + g));
          if (!m || !sn.has(m.name) && !ue.has(m.name) && !se.has(m.name)) continue;
          const [y, w] = wn(m.direction), x = se.has(m.name) ? a + f : m.x ?? 0, _ = se.has(m.name) ? l + g : m.y ?? 0;
          x + y === a && _ + w === l && fe(t, c, we(m));
        }
      } else if (ue.has(o.name)) {
        if (o.io_type === "input") for (let u = 1; u <= R0; u++) {
          const p = s.get(ie(a + h * u, l + d * u));
          if (p) {
            if (ue.has(p.name) && p.name === o.name && p.io_type === "input" && p.direction === o.direction) break;
            if (ue.has(p.name) && p.name === o.name && p.io_type === "output" && p.direction === o.direction) {
              fe(t, c, we(p));
              break;
            }
          }
        }
        else {
          const u = e.get(ie(a + h, l + d));
          u && fe(t, c, u);
        }
        for (const [u, p] of i) {
          const f = s.get(ie(a + u, l + p));
          if (!f || !sn.has(f.name) && !se.has(f.name)) continue;
          const [g, m] = wn(f.direction);
          (f.x ?? 0) + g === a && (f.y ?? 0) + m === l && fe(t, c, we(f));
        }
      } else if (se.has(o.name)) {
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
          const y = e.get(ie(a + g + h, l + m + d));
          y && y !== c && fe(t, c, y);
          const w = e.get(ie(a + g - h, l + m - d));
          w && w !== c && fe(t, c, w);
        }
      }
    }
    for (const o of n.entities) {
      if (!br.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = wn(o.direction), p = o.name === "long-handed-inserter" ? 2 : 1, f = e.get(ie(a - h * p, l - d * p)), g = e.get(ie(a + h * p, l + d * p));
      f && fe(t, c, f), g && fe(t, c, g);
    }
    const r = 10;
    for (const o of n.entities) {
      if (!li.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o);
      if (o.name === "pipe") for (const [h, d] of i) {
        const u = s.get(ie(a + h, l + d));
        if (u) {
          if (u.name === "pipe") fe(t, c, we(u));
          else if (u.name === "pipe-to-ground") {
            const [p, f] = wn(u.direction);
            h === p && d === f && fe(t, c, we(u));
          } else if (Ae.has(u.name)) {
            const p = e.get(ie(u.x ?? 0, u.y ?? 0));
            p && fe(t, c, p);
          }
        }
      }
      else if (o.name === "pipe-to-ground") {
        if (o.io_type === "input") {
          const [p, f] = wn(o.direction);
          for (let g = 2; g <= r; g++) {
            const m = s.get(ie(a + p * g, l + f * g));
            if (!m || m.name !== "pipe-to-ground") continue;
            const [y, w] = wn(m.direction);
            if (m.io_type === "output" && y === -p && w === -f) {
              fe(t, c, we(m));
              break;
            }
            break;
          }
        }
        const [h, d] = wn(o.direction), u = s.get(ie(a - h, l - d));
        u && u.name === "pipe" && fe(t, c, we(u));
      }
    }
    for (const o of n.entities) {
      if (!Ae.has(o.name)) continue;
      const a = o.x ?? 0, l = o.y ?? 0, c = we(o), [h, d] = Pe[o.name] ?? [
        1,
        1
      ];
      for (let u = 0; u < d; u++) for (let p = 0; p < h; p++) for (const [f, g] of i) {
        const m = a + p + f, y = l + u + g;
        if (m >= a && m < a + h && y >= l && y < l + d) continue;
        const w = s.get(ie(m, y));
        w && li.has(w.name) && fe(t, c, we(w));
      }
    }
    return t;
  }
  function su(n, t) {
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
  const iu = 150, L0 = 64, rr = 0.2, $0 = 5, B0 = 100, O0 = 200, N0 = (n) => 1 - Math.pow(1 - n, 3), F0 = (n) => n;
  function Fi() {
    return new ox({
      dynamicProperties: {
        color: true,
        position: false,
        rotation: false,
        vertex: false,
        uvs: false
      }
    });
  }
  function ru() {
    const n = Fi(), t = new Ht(), e = Fi(), s = Fi(), i = Fi();
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
        n.removeParticles(), t.removeChildren(), e.removeParticles(), s.removeParticles(), i.removeParticles(), re.clear(), Ts.clear();
      },
      count() {
        return n.particleChildren.length + e.particleChildren.length + s.particleChildren.length + i.particleChildren.length;
      }
    };
  }
  const re = /* @__PURE__ */ new Map();
  function us(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function W0(n, t) {
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
      if (!u || !(sn.has(u.name) || ue.has(u.name) && u.io_type === "output" || se.has(u.name))) continue;
      const [f, g] = s[u.direction ?? "North"] ?? [
        0,
        -1
      ], m = se.has(u.name) ? h : u.x ?? 0, y = se.has(u.name) ? d : u.y ?? 0;
      if (!(m + f !== (n.x ?? 0) || y + g !== (n.y ?? 0))) if (u.direction === e) o = true;
      else {
        const w = f * r - g * i;
        w !== 0 && (a = w > 0 ? "cw" : "ccw");
      }
    }
    return a && !o ? a === "cw" ? "corner-cw" : "corner-ccw" : "straight";
  }
  function G0(n, t) {
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
      d && ou(e, s, d) && (r |= l);
    }
    return r;
  }
  function ou(n, t, e) {
    const s = e.x ?? 0, i = e.y ?? 0;
    for (const [r, o] of qd(e)) if (s + r === n && i + o === t) return true;
    return false;
  }
  const z0 = 9079434;
  function D0(n, t, e) {
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
        if (!d || !ou(o, a, d)) continue;
        const u = o * S + S / 2, p = a * S + S / 2, f = S * 1.5, g = u + l * f, m = p + c * f, y = new ft();
        y.moveTo(u, p).lineTo(g, m).stroke({
          width: i,
          color: z0,
          cap: "round"
        }), s.addChild(y);
      }
    }
  }
  function Ea(n, t) {
    if (n.quality && (n = {
      ...n,
      quality: void 0
    }), sn.has(n.name)) {
      const s = W0(n, t.tileMap), i = g0(n.name, n.direction ?? "North", s);
      return hn(i, gt, gt, (r) => {
        const o = gt / S;
        let a = null;
        s === "corner-cw" ? a = {
          turn: "cw"
        } : s === "corner-ccw" && (a = {
          turn: "ccw"
        });
        const l = _a(n, a);
        l.scale.set(o), r.addChild(l);
      });
    }
    if (ue.has(n.name)) {
      const s = n.io_type ?? "input", i = x0(n.name, n.direction ?? "North", s);
      return hn(i, gt, gt, (r) => {
        const o = gt / S, a = Fn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (se.has(n.name)) {
      const s = b0(n.name, n.direction ?? "North"), i = n.direction === "North" || n.direction === "South";
      return cc(s, i ? 2 : 1, i ? 1 : 2, (a, l, c) => {
        const h = gt / S, d = Fn(n, t);
        d.scale.set(h), d.x = 0, d.y = 0, a.addChild(d);
      });
    }
    if (n.name === "pipe") {
      const s = G0(n, t), i = y0(s);
      return hn(i, gt, gt, (r) => {
        const o = gt / S, a = Fn(n, t);
        a.scale.set(o), a.x = 0, a.y = 0, r.addChild(a);
      });
    }
    if (n.name === "pipe-to-ground") {
      const s = C0(n.direction ?? "North");
      return hn(s, gt, gt, (i) => {
        const r = gt / S, o = Fn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (br.has(n.name)) {
      const s = _0(n.name, n.direction ?? "North");
      return hn(s, gt, gt, (i) => {
        const r = gt / S, o = Fn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (ga.has(n.name)) {
      const s = v0(n.name);
      return hn(s, gt, gt, (i) => {
        const r = gt / S, o = Fn(n, t);
        o.scale.set(r), o.x = 0, o.y = 0, i.addChild(o);
      });
    }
    if (Ae.has(n.name)) {
      const [s, i] = Pe[n.name] ?? [
        1,
        1
      ], r = w0(n.name);
      return cc(r, s, i, (o, a, l) => {
        const c = `/spaghettio/entity-frames/${n.name}.png`, h = ne.has(c) ? $e.get(c) ?? null : null, d = 1.8, u = gt / L0;
        if (h) {
          const p = new Ne(h);
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
    return hn(e, gt, gt, (s) => {
      const i = gt / S, r = Fn(n, t);
      r.scale.set(i), r.x = 0, r.y = 0, s.addChild(r);
    });
  }
  function Uo(n, t, e, s = _r()) {
    const i = us(t);
    if (re.has(i)) return;
    const r = t.x ?? 0, o = t.y ?? 0, a = Ea(t, s);
    let l = S / gt, c = S / gt;
    if (se.has(t.name)) {
      const g = t.direction === "North" || t.direction === "South", m = g ? 2 : 1, y = g ? 1 : 2;
      l = m * S / (m * gt), c = y * S / (y * gt);
    } else if (Ae.has(t.name)) {
      const [g, m] = Pe[t.name] ?? [
        1,
        1
      ];
      l = g * S / (g * gt), c = m * S / (m * gt);
    }
    const h = r * S, d = o * S, u = new ti({
      texture: a,
      x: h,
      y: d,
      alpha: 0,
      anchorX: 0,
      anchorY: 0,
      scaleX: l,
      scaleY: c
    });
    Ae.has(t.name) ? n.machineContainer.addParticle(u) : n.beltContainer.addParticle(u);
    let p;
    if (t.carries && !Ae.has(t.name)) {
      const g = Ho(t.carries), m = S * 0.35, y = (S - m) / 2;
      p = new ti({
        texture: g,
        x: h + y,
        y: d + y,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: m / gt,
        scaleY: m / gt
      }), n.iconContainer.addParticle(p);
    }
    let f;
    if (t.quality && t.quality !== "normal") {
      const g = Ho(`quality-${t.quality}`), m = S * 0.4;
      let y = 1;
      Ae.has(t.name) ? y = (Pe[t.name] ?? [
        1,
        1
      ])[1] : t.name === "substation" && (y = 2);
      const w = 1.5;
      f = new ti({
        texture: g,
        x: h + w,
        y: d + y * S - m - w,
        alpha: 0,
        anchorX: 0,
        anchorY: 0,
        scaleX: m / gt,
        scaleY: m / gt
      }), n.iconContainer.addParticle(f);
    }
    re.set(i, {
      entity: u,
      icon: p,
      badge: f,
      revealAt: e,
      placedEntity: t
    });
  }
  const Ts = /* @__PURE__ */ new Map();
  function H0(n, t, e, s, i, r, o) {
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
    const d = Ea(c, h), u = Rn(i), p = new ti({
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
  function bc(n, t, e) {
    const s = `${t},${e}`, i = Ts.get(s);
    i && (n.ghostContainer.removeParticle(i.particle), Ts.delete(s));
  }
  function U0(n) {
    n.ghostContainer.removeParticles(), Ts.clear();
  }
  function j0(n, t, e) {
    const s = [];
    for (const [i, r] of re.entries()) r.placedEntity.x !== t || r.placedEntity.y !== e || (Ae.has(r.placedEntity.name) ? n.machineContainer.removeParticle(r.entity) : n.beltContainer.removeParticle(r.entity), r.icon && n.iconContainer.removeParticle(r.icon), re.delete(i), s.push(i));
    return s;
  }
  function V0(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const [s, i] of re.entries()) {
      const r = i.placedEntity.name;
      if (r !== "pipe" && !sn.has(r)) continue;
      const o = Ea(i.placedEntity, t);
      if (i.entity.texture === o) continue;
      const a = i.entity, l = new ti({
        texture: o,
        x: a.x,
        y: a.y,
        alpha: a.alpha,
        anchorX: a.anchorX,
        anchorY: a.anchorY,
        scaleX: a.scaleX,
        scaleY: a.scaleY
      });
      n.beltContainer.removeParticle(a), n.beltContainer.addParticle(l), re.set(s, {
        ...i,
        entity: l
      }), e.set(a, l);
    }
    return e;
  }
  function au(n, t) {
    for (const e of re.values()) {
      const s = Math.min(1, Math.max(0, (t - e.revealAt) / iu));
      e.entity.alpha = s, e.icon && (e.icon.alpha = s), e.badge && (e.badge.alpha = s);
    }
  }
  function* _c(n) {
    for (const t of re.values()) yield {
      particle: t.entity,
      iconParticle: t.icon,
      badgeParticle: t.badge,
      revealAt: t.revealAt
    };
  }
  function lu(n) {
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
        const p = c - u.startTime, f = Math.min(1, p / u.duration), g = u.ease(f), m = u.startAlpha + (u.targetAlpha - u.startAlpha) * g;
        u.entityParticle.alpha = m, u.iconParticle && (u.iconParticle.alpha = m), u.badgeParticle && (u.badgeParticle.alpha = m), f >= 1 ? t.delete(d) : h = true;
      }
      h || i(), Ye();
    }
    function o(c, h, d, u, p) {
      const f = t.get(c), g = performance.now();
      let m;
      if (f) {
        const w = g - f.startTime, x = Math.min(1, w / f.duration);
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
        duration: y ? B0 : O0,
        ease: y ? N0 : F0
      }), s();
    }
    function a(c) {
      t.clear();
      for (const h of re.values()) h.entity.alpha = c, h.icon && (h.icon.alpha = c), h.badge && (h.badge.alpha = c);
      i(), Ye();
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
  function cu(n) {
    let t = 0;
    for (const i of n.values()) i > t && (t = i);
    const e = Math.max(t, $0), s = /* @__PURE__ */ new Map();
    for (const [i, r] of n) {
      const o = rr + (1 - rr) * (1 - r / e);
      s.set(i, o);
    }
    return s;
  }
  function Mn(n) {
    return `${n.x ?? 0},${n.y ?? 0}:${n.name}:${n.recipe ?? ""}`;
  }
  function Y0(n, t, e) {
    t.clear(), t.layout = n;
    const s = _r();
    for (const l of n.entities) ir(l, s);
    const i = 0;
    for (const l of n.entities) Uo(t, l, i, s);
    D0(t, n, s), au(t, iu + 1);
    const r = nu(n);
    if (!e) return X0();
    const o = lu(e.ticker);
    function a(l) {
      const c = cu(l);
      for (const h of re.values()) {
        const d = Mn(h.placedEntity), u = c.get(d) ?? rr;
        o.animateTo(d, h.entity, h.icon, h.badge, u);
      }
    }
    return {
      highlightItem(l) {
        if (o.cancelAll(1), !!l) for (const c of re.values()) {
          const h = c.placedEntity, u = (h.carries ?? h.recipe ?? null) === l ? 1 : 0.15, p = Mn(h);
          o.animateTo(p, c.entity, c.icon, c.badge, u);
        }
      },
      highlightBeltNetwork(l) {
        if (!l) {
          o.cancelAll(1);
          return;
        }
        const c = Mn(l), h = su(r, c);
        a(h);
      },
      clearHighlight() {
        for (const l of re.values()) {
          const c = Mn(l.placedEntity);
          o.animateTo(c, l.entity, l.icon, l.badge, 1);
        }
      },
      chainKey(l) {
        return l.carries ?? l.recipe ?? null;
      }
    };
  }
  function X0() {
    function n() {
      for (const t of re.values()) t.entity.alpha = 1, t.icon && (t.icon.alpha = 1), t.badge && (t.badge.alpha = 1);
    }
    return {
      highlightItem(t) {
        if (n(), !!t) for (const e of re.values()) {
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
  function q0(n, t) {
    const e = nu(n), s = lu(t.ticker);
    function i(r) {
      const o = Mn(r), a = su(e, o), l = cu(a);
      for (const c of re.values()) {
        const h = Mn(c.placedEntity), d = l.get(h) ?? rr;
        s.animateTo(h, c.entity, c.icon, c.badge, d);
      }
    }
    return {
      highlightItem(r) {
        if (s.cancelAll(1), !!r) for (const o of re.values()) {
          const a = o.placedEntity, c = (a.carries ?? a.recipe ?? null) === r ? 1 : 0.15;
          o.entity.alpha = c, o.icon && (o.icon.alpha = c), o.badge && (o.badge.alpha = c);
        }
      },
      highlightBeltNetwork(r) {
        if (!r) {
          for (const o of re.values()) {
            const a = Mn(o.placedEntity);
            s.animateTo(a, o.entity, o.icon, o.badge, 1);
          }
          return;
        }
        i(r);
      },
      clearHighlight() {
        for (const r of re.values()) {
          const o = Mn(r.placedEntity);
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
  const wc = 6, K0 = 18, J0 = 2, Z0 = 26, Q0 = -Math.PI / 3, vc = 2, tb = 16777215, eb = 0.55, Cc = 4, nb = 0, sb = 0.6;
  function ib(n) {
    return n.carries ? sn.has(n.name) || ue.has(n.name) || li.has(n.name) : false;
  }
  function rb(n, t) {
    const e = /* @__PURE__ */ new Map();
    for (const l of t.external_inputs) e.set(l.item, {
      rate: l.rate,
      isFluid: !!l.is_fluid
    });
    if (e.size === 0) return [];
    const s = /* @__PURE__ */ new Map();
    for (const l of n.entities) {
      if (!ib(l)) continue;
      const c = l.carries;
      if (!e.has(c)) continue;
      const h = l.x ?? 0, d = l.y ?? 0, u = s.get(h);
      (!u || d < u.y) && s.set(h, {
        y: d,
        carries: c
      });
    }
    if (s.size === 0) return [];
    const i = Array.from(s.entries()).filter(([, l]) => l.y === nb).map(([l, c]) => ({
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
  function ob(n) {
    return `${n.toFixed(1)}/s`;
  }
  function ab(n) {
    const t = n.xMax - n.xMin + 1, e = Math.min(Z0, K0 + (t - 1) * J0), s = new Ht();
    s.eventMode = "none";
    const i = Ho(n.item), r = new Ne(i);
    r.width = e, r.height = e, r.x = 0, r.y = -e / 2, s.addChild(r);
    const o = new Fe({
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
      text: ob(n.rate),
      style: o
    });
    a.x = e + wc, a.y = -a.height / 2, s.addChild(a);
    const l = new Fe({
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
      text: pe(n.item),
      style: l
    });
    return c.alpha = sb, c.x = a.x + a.width + wc, c.y = -c.height / 2, s.addChild(c), s;
  }
  function lb(n, t, e) {
    if (n.removeChildren(), !e) return;
    const s = rb(t, e);
    if (s.length !== 0) for (const i of s) {
      const r = i.topY * S - vc, o = i.xMin * S + Cc, a = (i.xMax - i.xMin + 1) * S - 2 * Cc;
      if (a > 0) {
        const d = new ft();
        d.rect(o, r, a, vc).fill({
          color: tb,
          alpha: eb
        }), n.addChild(d);
      }
      const l = ab(i);
      l.rotation = Q0;
      const c = (i.xMin + i.xMax + 1) / 2 * S, h = i.topY * S - S * 0.5;
      l.x = c, l.y = h, n.addChild(l);
    }
  }
  function cb(n) {
    return se.has(n.name) ? n.direction === "East" || n.direction === "West" ? [
      1,
      2
    ] : [
      2,
      1
    ] : Pe[n.name] ?? [
      1,
      1
    ];
  }
  const eo = 57504;
  function Sc(n, t, e, s, i) {
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
      const C = d(o.sx, o.sy), T = d(v, b), R = Math.min(C.x, T.x), A = Math.min(C.y, T.y), k = Math.abs(T.x - C.x), L = Math.abs(T.y - C.y);
      c.clear(), c.rect(R, A, k, L).fill({
        color: eo,
        alpha: 0.18
      }), c.setStrokeStyle({
        width: 1,
        color: eo,
        alpha: 0.8
      }), c.rect(R, A, k, L).stroke(), Ye();
    }
    function p(v) {
      if (h.clear(), v.length !== 0) {
        h.setStrokeStyle({
          width: 1.5,
          color: eo,
          alpha: 0.9
        });
        for (const b of v) {
          const [C, T] = cb(b), R = (b.x ?? 0) * S + 1, A = (b.y ?? 0) * S + 1;
          h.rect(R, A, C * S - 2, T * S - 2).stroke();
        }
      }
    }
    function f(v, b) {
      if (!o) return [];
      const C = d(o.sx, o.sy), T = d(v, b), R = Math.min(Math.floor(C.x / S), Math.floor(T.x / S)), A = Math.max(Math.floor(C.x / S), Math.floor(T.x / S)), k = Math.min(Math.floor(C.y / S), Math.floor(T.y / S)), L = Math.max(Math.floor(C.y / S), Math.floor(T.y / S)), H = [];
      for (let W = R; W <= A; W++) for (let F = k; F <= L; F++) {
        const I = r.get(`${W},${F}`);
        I && H.push(I);
      }
      return H;
    }
    const g = (v) => {
      v.button !== 0 || !v.shiftKey || (o = {
        sx: v.clientX,
        sy: v.clientY
      }, a = false);
    }, m = (v) => {
      if (!o) return;
      const b = v.clientX - o.sx, C = v.clientY - o.sy;
      !a && b * b + C * C > 36 && (a = true), a && u(v.clientX, v.clientY);
    }, y = (v) => {
      if (v.button === 0) {
        if (a) v.stopImmediatePropagation(), c.clear(), l = f(v.clientX, v.clientY), p(l), i(l), Ye();
        else if (o !== null) {
          const b = d(v.clientX, v.clientY), C = Math.floor(b.x / S), T = Math.floor(b.y / S);
          r.has(`${C},${T}`) && (l = [], h.clear(), i([]), Ye());
        }
        o = null, a = false;
      }
    };
    function w() {
      l = [], c.clear(), h.clear(), i([]), Ye();
    }
    const x = (v) => {
      v.preventDefault(), l.length > 0 && w();
    }, _ = (v) => {
      v.key === "Escape" && l.length > 0 && w();
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
  const hb = {
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
  }, db = {
    codes: hb
  }, hu = db, ub = new Map(Object.entries(hu.codes)), pb = new Map(Object.entries(hu.codes).map(([n, t]) => [
    t,
    n
  ]));
  function fb(n) {
    return ub.get(n) ?? null;
  }
  function mb(n) {
    return pb.get(n) ?? null;
  }
  const gb = [
    "partitioned-per-consumer",
    "partitioned-decomposed"
  ], yb = [
    "horizontal-stack"
  ], xb = [
    "regular",
    "fast"
  ], bb = [
    "uncommon",
    "rare",
    "epic",
    "legendary"
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
  ], Aa = "iron-gear-wheel", ka = 10, bs = {
    crafting: "assembling-machine-3",
    smelting: "electric-furnace"
  }, pi = {
    crafting: "craft",
    smelting: "smelt"
  }, _b = "machine", or = "#/l/", Ys = "_", ar = ",", wb = {
    pd: "partitioned-decomposed"
  }, Tc = {
    "partitioned-decomposed": "pd"
  }, vb = {
    hs: "horizontal-stack"
  }, Ec = {
    "horizontal-stack": "hs"
  }, Cb = {
    r: "regular",
    f: "fast"
  }, Ac = {
    regular: "r",
    fast: "f"
  }, Sb = {
    u: "uncommon",
    r: "rare",
    e: "epic",
    l: "legendary"
  }, kc = {
    uncommon: "u",
    rare: "r",
    epic: "e",
    legendary: "l"
  };
  function ls(n) {
    return fb(n) ?? n;
  }
  function cs(n) {
    return mb(n);
  }
  function Tb() {
    const n = window.location.hash;
    if (!n.startsWith(or)) return null;
    const t = n.slice(or.length), e = t.indexOf("?"), s = e >= 0 ? t.slice(0, e) : t, i = e >= 0 ? t.slice(e + 1) : "", r = s.split("/"), o = (H) => {
      const W = r[H];
      return W === void 0 || W === "" || W === Ys ? null : W;
    }, a = o(0);
    let l;
    if (a) {
      const H = cs(a);
      if (H === null) return null;
      l = H;
    } else l = Aa;
    const c = o(1), h = c !== null ? parseFloat(c) : NaN, d = !isNaN(h) && h > 0 ? h : ka, u = o(2), p = {};
    if (u) {
      const H = cs(u);
      if (H === null) return null;
      p.crafting = H;
    }
    const f = o(3);
    let g;
    if (f) {
      const H = f.split(ar).filter((F) => F.length > 0), W = [];
      for (const F of H) {
        const I = cs(F);
        if (I === null) return null;
        W.push(I);
      }
      g = W;
    } else g = Es;
    const m = o(4);
    let y = null;
    if (m && (y = cs(m), y === null)) return null;
    const w = new URLSearchParams(i), x = w.get("s"), _ = x ? wb[x] ?? null : null, v = w.get("rl"), b = v ? vb[v] ?? null : null, C = w.get("it"), T = C ? Cb[C] ?? null : null, R = w.get("q"), A = R ? Sb[R] ?? null : null, k = w.get("ci");
    let L = [];
    if (k) {
      const H = k.split(ar).filter((W) => W.length > 0);
      for (const W of H) {
        const F = cs(W);
        if (F === null) return null;
        L.push(F);
      }
    }
    for (const [H, W] of Object.entries(pi)) {
      if (H === "crafting") continue;
      const F = w.get(W);
      if (!F) continue;
      const I = cs(F);
      if (I === null) return null;
      p[H] = I;
    }
    return {
      item: l,
      rate: d,
      machines: p,
      inputs: g,
      belt: y,
      strategy: _,
      rowLayout: b,
      inserterTier: T,
      quality: A,
      customInputs: L
    };
  }
  function Eb() {
    const n = new URLSearchParams(window.location.search), t = n.get("item") ?? Aa, e = parseFloat(n.get("rate") ?? ""), s = isNaN(e) || e <= 0 ? ka : e, i = {};
    for (const [x, _] of Object.entries(pi)) {
      const v = n.get(_);
      v && (i[x] = v);
    }
    const r = n.get(_b);
    r && !i.crafting && (i.crafting = r);
    const o = n.get("in"), a = o ? o.split(",").filter((x) => x.length > 0) : Es, l = n.get("belt"), c = n.get("strategy");
    let h = c && gb.includes(c) ? c : null;
    h === "partitioned-per-consumer" && (h = "partitioned-decomposed");
    const d = n.get("row_layout"), u = d && yb.includes(d) ? d : null, p = n.get("inserter_tier"), f = p && xb.includes(p) ? p : null, g = n.get("quality"), m = g && bb.includes(g) ? g : null, y = n.get("ci"), w = y ? y.split(",").filter((x) => x.length > 0) : [];
    return {
      item: t,
      rate: s,
      machines: i,
      inputs: a,
      belt: l,
      strategy: h,
      rowLayout: u,
      inserterTier: f,
      quality: m,
      customInputs: w
    };
  }
  function Ab() {
    return Tb() ?? Eb();
  }
  function kb() {
    if (window.location.hash.startsWith(or)) return true;
    const n = new URLSearchParams(window.location.search);
    return n.has("item") || n.has("rate") || n.has("machine") || n.has("in") || n.has("belt");
  }
  function Mb(n) {
    for (const [t, e] of Object.entries(pi)) {
      const s = n[t];
      if (s && s !== bs[t]) return false;
    }
    for (const t of Object.keys(n)) if (!(t in pi)) return false;
    return true;
  }
  function Pb(n) {
    const t = ls(n.item), e = String(n.rate), s = n.machines.crafting, i = !s || s === bs.crafting ? Ys : ls(s), r = n.inputs.length === Es.length && n.inputs.every((u, p) => u === Es[p]), o = n.inputs.length === 0 || r ? Ys : n.inputs.map(ls).join(ar), a = n.belt ? ls(n.belt) : Ys, l = new URLSearchParams();
    n.strategy && Tc[n.strategy] && l.set("s", Tc[n.strategy]), n.rowLayout && Ec[n.rowLayout] && l.set("rl", Ec[n.rowLayout]), n.inserterTier && Ac[n.inserterTier] && l.set("it", Ac[n.inserterTier]), n.quality && kc[n.quality] && l.set("q", kc[n.quality]), n.customInputs.length > 0 && l.set("ci", n.customInputs.map(ls).join(ar));
    for (const [u, p] of Object.entries(pi)) {
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
  function Ib(n) {
    const t = n.item === Aa && n.rate === ka && Mb(n.machines) && n.inputs.length === Es.length && n.inputs.every((s, i) => s === Es[i]) && !n.belt && !n.strategy && !n.rowLayout && !n.inserterTier && n.customInputs.length === 0, e = window.location.pathname;
    if (t) {
      history.replaceState(null, "", e);
      return;
    }
    history.replaceState(null, "", e + Pb(n));
  }
  const so = "[INCOMPATIBLE_MACHINE]";
  function pn(n, t = 14) {
    const e = document.createElement("img");
    return e.src = `/spaghettio/icons/${n}.png`, e.width = t, e.height = t, e.style.cssText = "image-rendering:pixelated", e.onerror = () => {
      e.style.display = "none";
    }, e;
  }
  function Rb(n, t) {
    const e = document.createElement("option");
    return e.value = n, e.textContent = pe(n), n === t && (e.selected = true), e;
  }
  function Wi(n, t, e) {
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
  function Mc(n, t, e, s) {
    for (const i of t) {
      const r = document.createElement("div");
      r.className = `sb-machine-flow ${e}`, s && r.appendChild(document.createTextNode(s)), r.appendChild(pn(i.item, 13)), r.appendChild(document.createTextNode(pe(i.item)));
      const o = document.createElement("span");
      o.className = "flow-rate";
      const a = eu(i.rate), l = a ? tu(a.color) : "#f88";
      o.style.color = l, o.textContent = `${i.rate.toFixed(1)}/s`, r.appendChild(o), n.appendChild(r);
    }
  }
  const Pc = /* @__PURE__ */ new Set([
    "water",
    "crude-oil",
    "petroleum-gas",
    "light-oil",
    "heavy-oil",
    "sulfuric-acid",
    "lubricant",
    "steam"
  ]), du = "spaghettio-recent-items", Ic = 5;
  function uu() {
    try {
      const n = localStorage.getItem(du);
      return n ? JSON.parse(n) : [];
    } catch {
      return [];
    }
  }
  function Lb(n) {
    const t = uu().filter((e) => e !== n);
    t.unshift(n), t.length > Ic && (t.length = Ic);
    try {
      localStorage.setItem(du, JSON.stringify(t));
    } catch {
    }
  }
  function $b(n, t, e) {
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
        x.textContent = pe(s), a.appendChild(x);
      } else {
        const x = document.createElement("span");
        x.className = "sb-picker-placeholder", x.textContent = "Select item\u2026", a.appendChild(x);
      }
    }
    function p(x) {
      const _ = document.createElement("div");
      _.className = "sb-picker-item" + (x === s ? " selected" : ""), _.dataset.slug = x, _.appendChild(pn(x, 14));
      const v = document.createElement("span");
      return v.textContent = pe(x), _.appendChild(v), _.addEventListener("mousedown", (b) => {
        b.preventDefault(), g(x);
      }), _;
    }
    function f(x) {
      d.innerHTML = "", r = null;
      const _ = x.trim().toLowerCase(), v = _ ? n.filter((b) => b.includes(_) || pe(b).toLowerCase().includes(_)) : n;
      if (!_) {
        const b = uu().filter((C) => n.includes(C));
        if (b.length > 0) {
          const C = document.createElement("div");
          C.className = "sb-picker-section-label", C.textContent = "Recent", d.appendChild(C);
          for (const R of b) d.appendChild(p(R));
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
    function g(x) {
      s = x, Lb(x), o.classList.remove("item-invalid"), u(), y(), e(x);
    }
    function m() {
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
      c.contains(x.target) || (x.preventDefault(), i ? y() : m());
    }), h.addEventListener("input", () => f(h.value)), h.addEventListener("keydown", (x) => {
      x.key === "ArrowDown" ? (x.preventDefault(), w(1)) : x.key === "ArrowUp" ? (x.preventDefault(), w(-1)) : x.key === "Enter" ? (r == null ? void 0 : r.dataset.slug) && g(r.dataset.slug) : x.key === "Escape" && y();
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
  function Bb(n, t, e) {
    var _a2;
    n.innerHTML = "";
    const s = document.createElement("div");
    s.className = "sidebar-inner";
    const { section: i, body: r } = Wi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="2"/></svg>', "Target"), o = t.allProducibleItems(), a = new Set(o);
    function l(j, Z) {
      const Q = document.createElement("div");
      Q.className = "sb-field";
      const wt = document.createElement("span");
      return wt.className = "sb-field-label", wt.textContent = j, Q.appendChild(wt), Z.style.flex = "1", Z.style.minWidth = "0", Q.appendChild(Z), Q;
    }
    const c = $b(o, "", () => At());
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
    for (const j of h) {
      const Z = document.createElement("select");
      Z.className = "sb-select", Z.dataset.cat = j.category;
      const Q = bs[j.category] ?? "";
      for (const wt of j.options) {
        const Ct = Rb(wt.value, Q);
        wt.disabled && (Ct.disabled = true), wt.title && (Ct.title = wt.title), Z.appendChild(Ct);
      }
      r.appendChild(l(j.label, Z)), u.set(j.category, Z);
    }
    const p = u.get("crafting");
    for (const j of d) {
      const Z = document.createElement("span");
      Z.className = "sb-machine-readonly", Z.textContent = pe(j.machine), r.appendChild(l(j.label, Z));
    }
    function f() {
      const j = {};
      for (const [Z, Q] of u) Q.value && (j[Z] = Q.value);
      return j;
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
    ].forEach(([j, Z]) => {
      const Q = document.createElement("option");
      Q.value = Z, Q.textContent = j, g.appendChild(Q);
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
    ].forEach(([j, Z]) => {
      const Q = document.createElement("option");
      Q.value = Z, Q.textContent = j, m.appendChild(Q);
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
    ].forEach(([j, Z]) => {
      const Q = document.createElement("option");
      Q.value = Z, Q.textContent = j, y.appendChild(Q);
    }), r.appendChild(l("Build quality", y));
    const w = document.createElement("select");
    w.className = "sb-select", [
      [
        "Pooled (default)",
        ""
      ],
      [
        "Partitioned + decomposed",
        "partitioned-decomposed"
      ]
    ].forEach(([j, Z]) => {
      const Q = document.createElement("option");
      Q.value = Z, Q.textContent = j, w.appendChild(Q);
    }), r.appendChild(l("Strategy", w));
    const x = document.createElement("select");
    x.className = "sb-select", [
      [
        "Vertical split (today)",
        ""
      ],
      [
        "Horizontal stack (RFP)",
        "horizontal-stack"
      ]
    ].forEach(([j, Z]) => {
      const Q = document.createElement("option");
      Q.value = Z, Q.textContent = j, x.appendChild(Q);
    }), r.appendChild(l("Row layout", x));
    const _ = document.createElement("div");
    _.className = "sb-field";
    const v = document.createElement("span");
    v.className = "sb-field-label", v.textContent = "Rate", _.appendChild(v);
    const b = document.createElement("input");
    b.type = "number", b.className = "sb-input", b.step = "0.5", b.min = "0.1", b.style.cssText = "flex:1;min-width:0", b.placeholder = "10", _.appendChild(b);
    const C = document.createElement("span");
    C.className = "sb-rate-suffix", C.textContent = "/s", _.appendChild(C), r.appendChild(_), s.appendChild(i);
    const { section: T, body: R } = Wi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="5" width="12" height="6" rx="1"/><line x1="5" y1="8" x2="11" y2="8"/></svg>', "Inputs"), A = document.createElement("div");
    A.className = "sb-tags";
    const k = /* @__PURE__ */ new Map();
    no.forEach((j) => {
      const Z = document.createElement("label");
      Z.className = `sb-tag${Pc.has(j) ? " fluid" : ""}`;
      const Q = document.createElement("span");
      Q.className = "sb-tag-check", Q.textContent = "\u2713";
      const wt = document.createElement("input");
      wt.type = "checkbox", wt.value = j, wt.style.display = "none", k.set(j, wt), Z.appendChild(Q), Z.appendChild(pn(j, 14)), Z.appendChild(document.createTextNode(pe(j))), Z.appendChild(wt), wt.addEventListener("change", () => {
        Z.classList.toggle("active", wt.checked);
      }), A.appendChild(Z);
    }), R.appendChild(A);
    let L = [];
    const H = document.createElement("div");
    H.className = "sb-tags sb-custom-tags", R.appendChild(H);
    const W = document.createElement("datalist");
    W.id = "spaghettio-custom-inputs-datalist";
    const F = new Set(no);
    o.filter((j) => !F.has(j)).forEach((j) => {
      const Z = document.createElement("option");
      Z.value = j, W.appendChild(Z);
    }), R.appendChild(W);
    const I = document.createElement("input");
    I.type = "text", I.className = "sb-input sb-custom-input-field", I.setAttribute("list", "spaghettio-custom-inputs-datalist"), I.autocomplete = "off", I.placeholder = "+ add input\u2026", R.appendChild(I);
    function z(j) {
      const Z = document.createElement("div");
      Z.className = `sb-tag sb-custom-tag active${Pc.has(j) ? " fluid" : ""}`, Z.dataset.item = j, Z.appendChild(pn(j, 14)), Z.appendChild(document.createTextNode(pe(j)));
      const Q = document.createElement("span");
      Q.className = "sb-tag-remove", Q.textContent = "\xD7", Q.addEventListener("click", (wt) => {
        wt.stopPropagation(), L = L.filter((Ct) => Ct !== j), Z.remove(), At();
      }), Z.appendChild(Q), H.appendChild(Z);
    }
    function J(j) {
      const Z = j.trim();
      !Z || !a.has(Z) || F.has(Z) || L.includes(Z) || (L.push(Z), z(Z), I.value = "", At());
    }
    I.addEventListener("keydown", (j) => {
      j.key === "Enter" && J(I.value);
    }), I.addEventListener("change", () => {
      J(I.value);
    }), s.appendChild(T);
    const { section: q, body: et, countEl: M } = Wi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h10M9 4l4 4-4 4"/></svg>', "Solver", ""), O = document.createElement("div");
    et.appendChild(O), s.appendChild(q);
    const N = document.createElement("div");
    N.className = "sb-actions", N.style.display = "none";
    const D = document.createElement("button");
    D.className = "sb-btn sb-btn-secondary", D.textContent = "Copy Blueprint", D.style.flex = "1", N.appendChild(D);
    const K = document.createElement("div");
    K.className = "sb-copy-status", N.appendChild(K), et.appendChild(N);
    const { section: nt, body: ut, countEl: pt } = Wi('<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8.5"/><circle cx="8" cy="11" r="0.8" fill="currentColor" stroke="none"/></svg>', "Validation", "");
    nt.style.display = "none", s.appendChild(nt), n.appendChild(s);
    const ct = Ab();
    c.setValue(ct.item), b.value = String(ct.rate);
    const _t = /* @__PURE__ */ new Set([
      "assembling-machine-1",
      "assembling-machine-2",
      "assembling-machine-3"
    ]);
    for (const [j, Z] of u) {
      const Q = ct.machines[j], wt = new Set(Array.from(Z.options).filter((Ct) => !Ct.disabled).map((Ct) => Ct.value));
      Z.value = Q && wt.has(Q) ? Q : bs[j] ?? ((_a2 = Z.options[0]) == null ? void 0 : _a2.value) ?? "";
    }
    k.forEach((j, Z) => {
      j.checked = ct.inputs.includes(Z);
      const Q = j.closest(".sb-tag");
      Q && Q.classList.toggle("active", j.checked);
    }), ct.belt && (g.value = ct.belt), ct.strategy && (w.value = ct.strategy), ct.rowLayout && (x.value = ct.rowLayout), ct.inserterTier && (m.value = ct.inserterTier), ct.quality && (y.value = ct.quality);
    for (const j of ct.customInputs) a.has(j) && !F.has(j) && !L.includes(j) && (L.push(j), z(j));
    const Y = document.createElement("div");
    Y.className = "sb-config-error", Y.style.display = "none", O.before(Y);
    function st(j) {
      j ? (Y.textContent = j, Y.style.display = "") : (Y.textContent = "", Y.style.display = "none");
    }
    let ot = null, at = ct.item, bt = null, zt = 0;
    function At() {
      ot !== null && clearTimeout(ot), ot = setTimeout(() => {
        Xt().catch((j) => console.error("runSolve failed:", j));
      }, 150);
    }
    async function Xt() {
      var _a3;
      const j = c.getValue(), Z = parseFloat(b.value), Q = no.filter((Et) => {
        var _a4;
        return (_a4 = k.get(Et)) == null ? void 0 : _a4.checked;
      }), wt = [
        ...Q,
        ...L
      ];
      if (!a.has(j)) {
        c.setInvalid(true);
        return;
      }
      if (c.setInvalid(false), isNaN(Z) || Z <= 0) return;
      if (j !== at) {
        const Et = t.defaultMachineForItem(j, p.value);
        _t.has(Et) && (p.value = Et), at = j;
      }
      const Ct = f();
      Ib({
        item: j,
        rate: Z,
        machines: Ct,
        inputs: Q,
        belt: g.value || null,
        strategy: w.value || null,
        rowLayout: x.value || null,
        inserterTier: m.value || null,
        quality: y.value || null,
        customInputs: L
      });
      const oe = ++zt;
      O.innerHTML = "", st(null), bt = null, N.style.display = "none";
      let $t;
      try {
        $t = await t.solve(j, Z, wt, Ct, Ct.crafting ?? bs.crafting, y.value || void 0);
      } catch (Et) {
        if (oe !== zt) return;
        e.renderGraph(null), M && (M.textContent = "error");
        const St = String(Et instanceof Error ? Et.message : Et);
        if (St.includes(so)) {
          const qt = St.indexOf(so), E = St.slice(qt + so.length).trim();
          st(E);
        } else {
          const qt = document.createElement("div");
          qt.className = "sb-result-error", qt.textContent = St, O.appendChild(qt);
        }
        return;
      }
      if (oe !== zt) return;
      Ob(O, $t), e.renderGraph($t);
      const Ut = $t.machines.reduce((Et, St) => Et + Math.ceil(St.count), 0);
      M && (M.textContent = `${Ut} machines`);
      const Ce = /* @__PURE__ */ new Set();
      for (const Et of $t.machines) {
        for (const St of Et.inputs) Ce.add(St.item);
        for (const St of Et.outputs) Ce.add(St.item);
      }
      for (const Et of $t.external_inputs) Ce.add(Et.item);
      for (const Et of $t.external_outputs) Ce.add(Et.item);
      if (await Xd(Array.from(Ce)), oe !== zt) return;
      let xe;
      try {
        const Et = g.value || void 0, St = w.value || void 0, qt = x.value || void 0, E = m.value || void 0, P = y.value || void 0, X = e.startStreaming();
        xe = await t.buildLayoutStreaming($t, Et, St, qt, E, P, X);
      } catch (Et) {
        if (oe !== zt) return;
        const St = document.createElement("div");
        St.className = "sb-result-error", St.textContent = `Layout error: ${Et}`, O.appendChild(St);
        return;
      }
      oe === zt && (bt = xe, Od($t.machines), e.renderLayout(xe, $t), N.style.display = ((_a3 = xe.warnings) == null ? void 0 : _a3.length) ? "none" : "flex");
    }
    D.addEventListener("click", async () => {
      if (!bt) return;
      const j = await t.exportBlueprint(bt, c.getValue());
      await navigator.clipboard.writeText(j), K.textContent = "Copied!", setTimeout(() => {
        K.textContent = "";
      }, 2e3);
    }), b.addEventListener("input", At);
    for (const j of u.values()) j.addEventListener("change", At);
    return g.addEventListener("change", At), w.addEventListener("change", At), x.addEventListener("change", At), m.addEventListener("change", At), y.addEventListener("change", At), k.forEach((j) => j.addEventListener("change", At)), Xt().catch((j) => console.error("runSolve failed:", j)), {
      getParams() {
        const j = c.getValue(), Z = parseFloat(b.value);
        return !j || isNaN(Z) || Z <= 0 ? null : {
          item: j,
          rate: Z
        };
      },
      setParams(j, Z) {
        c.setValue(j.item), b.value = String(j.rate), j.machine && _t.has(j.machine) ? p.value = j.machine : p.value = "assembling-machine-3", j.inputs && k.forEach((Q, wt) => {
          Q.checked = j.inputs.includes(wt);
          const Ct = Q.closest(".sb-tag");
          Ct && Ct.classList.toggle("active", Q.checked);
        }), j.belt ? g.value = j.belt : g.value = "", H.innerHTML = "", L = [];
        for (const Q of j.customInputs ?? []) a.has(Q) && !F.has(Q) && !L.includes(Q) && (L.push(Q), z(Q));
        at = j.item, (Z == null ? void 0 : Z.skipAutoSolve) || At();
      },
      updateValidation(j, Z, Q) {
        if (ut.innerHTML = "", j.length === 0) {
          nt.style.display = "none", pt && (pt.textContent = "");
          return;
        }
        nt.style.display = "";
        const wt = j.filter(($t) => $t.severity === "Error").length, Ct = j.length - wt;
        pt && (wt > 0 ? (pt.textContent = `${wt} error${wt !== 1 ? "s" : ""}`, pt.style.color = "#f66") : (pt.textContent = `${Ct} warning${Ct !== 1 ? "s" : ""}`, pt.style.color = "#fa0"));
        const oe = /* @__PURE__ */ new Map();
        for (const $t of j) {
          let Ut = oe.get($t.category);
          Ut || (Ut = [], oe.set($t.category, Ut)), Ut.push($t);
        }
        for (const [$t, Ut] of oe) {
          const xe = Ut.some((B) => B.severity === "Error") ? "#f44" : "#fa0", Et = Ut.find((B) => B.x != null && B.y != null), St = document.createElement("div");
          St.className = "sb-val-group";
          const qt = document.createElement("div");
          qt.className = "sb-val-group-header";
          const E = document.createElement("span");
          E.className = "sb-val-group-chevron", E.textContent = "\u25BE", E.addEventListener("click", (B) => {
            B.stopPropagation();
            const tt = $.style.display === "none";
            $.style.display = tt ? "" : "none", E.textContent = tt ? "\u25BE" : "\u25B8";
          }), qt.appendChild(E);
          const P = document.createElement("span");
          P.className = "sb-val-group-dot", P.style.background = xe, qt.appendChild(P);
          const X = document.createElement("span");
          X.className = "sb-val-group-name", X.textContent = $t, qt.appendChild(X);
          const V = document.createElement("span");
          V.className = "sb-val-group-count", V.textContent = String(Ut.length), qt.appendChild(V);
          const $ = document.createElement("div");
          if ($.className = "sb-val-group-body", Q) {
            const B = Ut.filter((tt) => tt.detail != null);
            if (B.length > 0) {
              const tt = /* @__PURE__ */ new Map();
              for (const it of B) {
                const mt = Q(it) ?? "unattributed";
                tt.set(mt, (tt.get(mt) ?? 0) + 1);
              }
              const U = document.createElement("div");
              U.className = "sb-val-cause-rollup", U.style.cssText = "font-size:11px;color:#b90;padding:1px 0 2px 22px", U.textContent = [
                ...tt.entries()
              ].sort((it, mt) => mt[1] - it[1]).map(([it, mt]) => `${mt} \xD7 ${it}`).join(" \xB7 "), $.appendChild(U);
            }
          }
          Et && (qt.classList.add("clickable"), qt.addEventListener("click", () => {
            Z(Et.x, Et.y);
          }));
          for (const B of Ut) {
            const tt = document.createElement("div"), U = B.x != null && B.y != null;
            tt.className = "sb-val-issue" + (U ? " clickable" : "");
            const it = document.createElement("span");
            if (it.className = "sb-val-issue-msg", it.textContent = B.message, tt.appendChild(it), U) {
              const mt = document.createElement("span");
              mt.className = "sb-val-issue-coord", mt.textContent = `${B.x}, ${B.y}`, tt.appendChild(mt), tt.addEventListener("click", (Ot) => {
                Ot.stopPropagation();
                const Bt = tt.classList.contains("pinned");
                ut.querySelectorAll(".sb-val-issue.pinned").forEach((jt) => jt.classList.remove("pinned")), Bt || tt.classList.add("pinned"), Z(B.x, B.y);
              });
            } else tt.style.opacity = "0.6";
            $.appendChild(tt);
          }
          St.appendChild(qt), St.appendChild($), ut.appendChild(St);
        }
      }
    };
  }
  function Ob(n, t) {
    if (t.external_inputs.length > 0) {
      const l = document.createElement("div");
      l.className = "sb-ext-section-title", l.textContent = "External inputs", n.appendChild(l);
      for (const h of t.external_inputs) {
        const d = document.createElement("div");
        d.className = "sb-ext-flow", d.appendChild(pn(h.item, 14)), d.appendChild(document.createTextNode(pe(h.item)));
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
      u.className = "sb-machine-group-header", u.appendChild(pn(l, 16));
      const p = document.createElement("span");
      p.className = "sb-machine-group-name", p.textContent = pe(l), u.appendChild(p);
      const f = document.createElement("span");
      f.className = "sb-machine-group-count", f.textContent = `\xD7${h}`, u.appendChild(f), d.appendChild(u);
      const g = document.createElement("div");
      g.className = "sb-machine-group-body";
      for (const m of c) {
        const y = document.createElement("div");
        y.className = "sb-machine-flow", y.style.cssText = "color:#6b7280;margin-bottom:2px", y.appendChild(document.createTextNode("\u2192 ")), y.appendChild(pn(m.recipe, 13)), y.appendChild(document.createTextNode(pe(m.recipe))), g.appendChild(y), Mc(g, m.inputs, "flow-in", "\u25B6 "), Mc(g, m.outputs, "flow-out", "\u25C0 ");
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
      c.style.cssText = "display:flex;align-items:center;gap:4px;padding:2px 0;font-size:11px;color:#b5cea8", c.appendChild(pn(l.item, 13)), c.appendChild(document.createTextNode(pe(l.item)));
      const h = eu(l.rate);
      if (h) {
        const d = tu(h.color);
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
  let Oe = null, Ma = 0;
  const Qn = /* @__PURE__ */ new Map();
  let ce = null;
  const jo = "spaghettio:sat-cache:v1";
  let un = new Uint8Array(0);
  function Nb() {
    try {
      const n = localStorage.getItem(jo);
      if (!n) return new Uint8Array(0);
      const t = atob(n), e = new Uint8Array(t.length);
      for (let s = 0; s < t.length; s++) e[s] = t.charCodeAt(s);
      return e;
    } catch (n) {
      return console.warn("[engine] could not read SAT cache from localStorage", n), new Uint8Array(0);
    }
  }
  function Fb(n) {
    let t = "";
    for (let i = 0; i < n.length; i += 8192) t += String.fromCharCode.apply(null, Array.from(n.subarray(i, i + 8192)));
    const s = btoa(t);
    try {
      localStorage.setItem(jo, s);
    } catch (i) {
      if (i instanceof DOMException && (i.name === "QuotaExceededError" || i.code === 22)) {
        console.warn("[engine] SAT cache quota exceeded \u2014 clearing");
        try {
          localStorage.removeItem(jo);
        } catch {
        }
        un = new Uint8Array(0);
      } else console.warn("[engine] failed to persist SAT cache", i);
    }
  }
  function Wb(n) {
    const t = new Uint8Array(un.length + n.length);
    t.set(un, 0), t.set(n, un.length), un = t, Fb(un);
  }
  let Vo = [], pu = [], fu = /* @__PURE__ */ new Map(), Yo = /* @__PURE__ */ new Set(), Xo = 0;
  function gn(n) {
    Xo += n;
    for (const t of Yo) t(Xo);
  }
  function Gb(n) {
    return Yo.add(n), n(Xo), () => Yo.delete(n);
  }
  function Te(n, t) {
    if (!Oe) throw new Error("Engine not initialized \u2014 call initEngine() first");
    const e = ++Ma;
    return gn(1), new Promise((s, i) => {
      Qn.set(e, {
        resolve: (r) => {
          gn(-1), ce === e && (ce = null), s(r);
        },
        reject: (r) => {
          gn(-1), ce === e && (ce = null), i(r);
        },
        onEvent: t
      }), Oe.postMessage({
        id: e,
        ...n
      });
    });
  }
  async function mu() {
    if (Oe) return;
    if (Oe = new Worker(new URL("/spaghettio/assets/engine.worker-CUTVLr9n.js", import.meta.url), {
      type: "module",
      name: "spaghettio-engine"
    }), Oe.onmessage = (t) => {
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
    }, Oe.onerror = (t) => {
      console.error("[engine.worker] error", t);
    }, await Te({
      method: "init"
    }), un = Nb(), un.length > 0) try {
      const t = await Te({
        method: "seedZoneCache",
        bytes: un
      });
      globalThis.__TRACE_LOGS === true && console.log(`[engine] seeded ${t} SAT zone records from localStorage`);
    } catch (t) {
      console.warn("[engine] seedZoneCache failed; persistence disabled this session", t);
    }
    Vo = await Te({
      method: "allProducibleItems"
    }), pu = await Te({
      method: "allProducerMachines"
    });
    const n = await Te({
      method: "defaultMachinesForItems",
      items: Vo,
      fallback: "assembling-machine-3"
    });
    fu = new Map(n);
  }
  async function zb(n, t, e, s, i, r) {
    return ce !== null && await Pa(), Te({
      method: "solve",
      targetItem: n,
      targetRate: t,
      availableInputs: e,
      palette: s,
      defaultMachine: i,
      quality: r ?? null
    });
  }
  function Db() {
    return Vo;
  }
  function Hb() {
    return pu;
  }
  function Ub(n, t, e, s, i, r) {
    return Te({
      method: "layout",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null,
      quality: r ?? null
    });
  }
  function jb(n, t, e, s, i, r) {
    return Te({
      method: "layoutTraced",
      result: n,
      maxBeltTier: t ?? null,
      strategy: e ?? null,
      rowLayout: s ?? null,
      maxInserterTier: i ?? null,
      quality: r ?? null
    });
  }
  async function Pa() {
    if (!Oe) return;
    Oe.terminate(), Oe = null;
    const n = new Error("Engine superseded by a newer request");
    for (const [, t] of Qn) t.reject(n);
    Qn.clear(), ce = null, await mu();
  }
  async function Vb(n, t, e, s, i, r, o) {
    ce !== null && await Pa();
    const a = ++Ma;
    return ce = a, gn(1), new Promise((l, c) => {
      Qn.set(a, {
        resolve: (d) => {
          gn(-1), ce === a && (ce = null), l(d);
        },
        reject: (d) => {
          gn(-1), ce === a && (ce = null), c(d);
        },
        onEvent: o
      });
      const h = globalThis.__TRACE_LOGS === true;
      Oe.postMessage({
        id: a,
        method: "layoutStreaming",
        result: n,
        maxBeltTier: t ?? null,
        strategy: e ?? null,
        rowLayout: s ?? null,
        maxInserterTier: i ?? null,
        quality: r ?? null,
        traceLogs: h
      });
    });
  }
  function Yb(n, t) {
    return Te({
      method: "exportBlueprint",
      layout: n,
      label: t
    });
  }
  function Xb(n, t) {
    return fu.get(n) ?? t;
  }
  function qb(n, t) {
    return Te({
      method: "validateLayout",
      layout: n,
      solverResult: t
    });
  }
  function Kb(n, t) {
    return Te({
      method: "solveFixture",
      fixtureJson: n,
      pinsJson: JSON.stringify(t)
    });
  }
  function Jb(n) {
    return Te({
      method: "parseBlueprint",
      bp: n
    });
  }
  async function gu(n, t, e, s, i = 0) {
    if (ce !== null && await Pa(), !Oe) throw new Error("Engine not initialized");
    const r = ++Ma;
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
            Wb(d);
          }
        }
      }), Oe.postMessage({
        id: r,
        method: "improveRegionStreaming",
        layout: n,
        regionId: t,
        budgetMs: e,
        maxIters: i
      });
    });
  }
  async function Zb(n, t) {
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
        e = await gu(e, o, t.perRegionBudgetMs, (l) => {
          var _a2;
          l.iter > 0 && (a = true), (_a2 = t.onImprovement) == null ? void 0 : _a2.call(t, l);
        }, 1), a ? i += 1 : s.delete(o);
      }
      if (i === 0) break;
    }
    return e;
  }
  function Qb(n, t) {
    return Te({
      method: "balancerShowcase",
      maxInputs: n,
      maxOutputs: t
    });
  }
  function t_() {
    return {
      solve: zb,
      allProducibleItems: Db,
      allProducerMachines: Hb,
      buildLayout: Ub,
      buildLayoutTraced: jb,
      buildLayoutStreaming: Vb,
      exportBlueprint: Yb,
      defaultMachineForItem: Xb,
      validateLayout: qb,
      solveFixture: Kb,
      improveRegion: gu,
      optimizeAllRegions: Zb,
      balancerShowcase: Qb
    };
  }
  const e_ = `
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
  function n_() {
    if (document.getElementById("spaghettio-corpus-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-corpus-style", n.textContent = e_, document.head.appendChild(n);
  }
  function s_(n, t) {
    n_(), n.innerHTML = "";
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
    let w = 0;
    m.addEventListener("input", () => {
      const M = m.value.trim(), O = ++w;
      if (!M) {
        m.classList.remove("error"), y.style.display = "none";
        return;
      }
      Jb(M).then((N) => {
        O === w && (m.classList.remove("error"), y.style.display = "none", t(N));
      }).catch((N) => {
        O === w && (m.classList.add("error"), y.textContent = String(N), y.style.display = "block");
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
    const R = document.createElement("select");
    C.appendChild(T), C.appendChild(R), x.appendChild(C);
    const A = document.createElement("div");
    A.className = "corpus-filter-row";
    const k = document.createElement("input");
    k.type = "checkbox";
    const L = document.createElement("label");
    L.style.display = "flex", L.style.alignItems = "center", L.style.gap = "5px", L.style.cursor = "pointer", L.appendChild(k), L.appendChild(document.createTextNode("Bus layouts only")), A.appendChild(L), x.appendChild(A), e.appendChild(x);
    const H = document.createElement("div");
    H.className = "corpus-count", H.style.display = "none", e.appendChild(H);
    const W = document.createElement("div");
    W.className = "corpus-list", e.appendChild(W);
    const F = document.createElement("div");
    F.className = "corpus-stats", e.appendChild(F);
    function I() {
      i = s.filter((M) => !(o && !M.stats.is_bus_layout || a && a !== "__all__" && M.stats.final_product !== a || l && !M.name.toLowerCase().includes(l.toLowerCase()))), r = -1, z();
    }
    function z() {
      if (W.innerHTML = "", H.textContent = `${i.length} of ${s.length} blueprint(s)`, i.length === 0) {
        const M = document.createElement("div");
        M.className = "corpus-empty", M.textContent = s.length === 0 ? "No corpus loaded yet." : "No blueprints match the current filters.", W.appendChild(M), F.classList.remove("visible");
        return;
      }
      for (let M = 0; M < i.length; M++) {
        const O = i[M], N = document.createElement("div");
        N.className = "corpus-item" + (M === r ? " selected" : "");
        const D = document.createElement("div");
        D.className = "corpus-item-name", D.textContent = O.name, D.title = O.name, N.appendChild(D);
        const K = document.createElement("div");
        if (K.className = "corpus-item-meta", O.stats.is_bus_layout) {
          const pt = document.createElement("span");
          pt.className = "corpus-badge corpus-badge-bus", pt.textContent = "BUS", K.appendChild(pt);
        }
        if (O.stats.final_product) {
          const pt = document.createElement("span");
          pt.className = "corpus-badge corpus-badge-product", pt.textContent = O.stats.final_product, K.appendChild(pt);
        }
        const nt = document.createElement("span");
        nt.className = "corpus-badge corpus-badge-machines", nt.textContent = `${O.stats.machine_count}m`, K.appendChild(nt), N.appendChild(K);
        const ut = M;
        N.addEventListener("click", () => J(ut)), W.appendChild(N);
      }
    }
    function J(M) {
      r = M;
      const O = i[M];
      z(), t(O.layout), q(O);
    }
    function q(M) {
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
      for (const [nt, ut] of K) {
        const pt = document.createElement("div");
        pt.className = "corpus-stats-row";
        const ct = document.createElement("span");
        ct.className = "corpus-stats-key", ct.textContent = nt;
        const _t = document.createElement("span");
        _t.className = "corpus-stats-val", _t.textContent = ut, pt.appendChild(ct), pt.appendChild(_t), N.appendChild(pt);
      }
      F.appendChild(N);
    }
    function et() {
      const M = new Set(s.map((N) => N.stats.final_product).filter(Boolean));
      R.innerHTML = "";
      const O = document.createElement("option");
      O.value = "__all__", O.textContent = "All products", R.appendChild(O);
      for (const N of Array.from(M).sort()) {
        const D = document.createElement("option");
        D.value = N, D.textContent = N, R.appendChild(D);
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
          s = JSON.parse((_a3 = N.target) == null ? void 0 : _a3.result).blueprints ?? [], i = s, r = -1, p.textContent = `Reload corpus.json (${s.length} blueprints)`, x.style.display = "", H.style.display = "", F.classList.remove("visible"), et(), I();
        } catch (D) {
          alert(`Failed to parse corpus.json: ${D}`);
        }
      }, O.readAsText(M);
    }), b.addEventListener("input", () => {
      l = b.value, I();
    }), R.addEventListener("change", () => {
      a = R.value, I();
    }), k.addEventListener("change", () => {
      o = k.checked, I();
    }), z();
  }
  const i_ = [
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
  ], r_ = `
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
  function o_() {
    if (document.getElementById("spaghettio-landing-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-landing-style", n.textContent = r_, document.head.appendChild(n);
  }
  function a_(n, t, e) {
    o_(), n.innerHTML = "";
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
    for (const u of i_) {
      const p = document.createElement("div");
      p.className = `spaghettio-landing-card ${u.status}`;
      const f = document.createElement("div");
      f.className = "spaghettio-landing-tier", f.innerHTML = `<span>${u.tier}</span>`, p.appendChild(f);
      const g = document.createElement("div");
      g.className = "spaghettio-landing-card-body";
      const m = document.createElement("div");
      m.className = "spaghettio-landing-card-title";
      const y = document.createElement("img");
      y.src = `/spaghettio/icons/${u.item}.png`, y.className = "spaghettio-landing-card-icon", y.onerror = () => {
        y.style.display = "none";
      }, m.appendChild(y), m.appendChild(document.createTextNode(u.label));
      const w = document.createElement("span");
      w.className = "spaghettio-landing-card-rate", w.textContent = `${u.rate}/s`, m.appendChild(w), g.appendChild(m);
      const x = document.createElement("div");
      x.className = "spaghettio-landing-card-desc", x.textContent = u.desc, g.appendChild(x), p.appendChild(g);
      const _ = document.createElement("div");
      _.className = `spaghettio-landing-status ${u.status}`, _.textContent = u.status === "solved" ? "Solved" : u.status === "partial" ? "Partial" : "WIP", p.appendChild(_);
      const v = document.createElement("div");
      v.className = "spaghettio-landing-entities", v.textContent = "\u2014", p.appendChild(v), u.status !== "wip" && p.addEventListener("click", () => {
        l_(t, u, p, v);
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
  function l_(n, t, e, s) {
    e.classList.contains("loading") || (e.classList.add("loading"), s.innerHTML = '<span class="spaghettio-spinner"></span>', (async () => {
      let i, r;
      try {
        const o = n.defaultMachineForItem(t.item, t.machine);
        i = await n.solve(t.item, t.rate, t.inputs, {}, o), r = await n.buildLayout(i, t.beltTier);
      } catch (o) {
        e.classList.remove("loading"), s.textContent = "error", console.error("Landing solve/layout failed:", o);
        return;
      }
      s.textContent = String(r.entities.length), e.classList.remove("loading"), Od(i.machines.map((o) => ({
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
      }))), c_(t, r, i).catch((o) => {
        console.error("Modal init failed:", o);
      });
    })());
  }
  async function c_(n, t, e) {
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
    const p = `${t.width ?? 0}\xD7${t.height ?? 0}`, f = e.machines.reduce((T, R) => T + Math.ceil(R.count), 0);
    u.innerHTML = `<span>${f} machines</span><span>${p} tiles</span>`, c.appendChild(u);
    const g = document.createElement("button");
    g.className = "spaghettio-preview-close", g.textContent = "\xD7", g.addEventListener("click", a), c.appendChild(g), l.appendChild(c);
    const m = document.createElement("div");
    m.className = "spaghettio-preview-canvas", l.appendChild(m);
    const y = document.createElement("div");
    y.className = "spaghettio-preview-badge", y.textContent = `0 / ${t.entities.length}`, m.appendChild(y), o = new sa(), await o.init({
      resizeTo: m,
      background: 1118481,
      antialias: true
    }), m.insertBefore(o.canvas, y), o.canvas.addEventListener("contextmenu", (T) => T.preventDefault());
    const w = (t.width ?? 20) * S, x = (t.height ?? 20) * S, _ = Math.max(w, x, 600) + 200, v = new Rd({
      screenWidth: m.clientWidth,
      screenHeight: m.clientHeight,
      worldWidth: _,
      worldHeight: _,
      events: o.renderer.events
    });
    v.drag({
      mouseButtons: "left"
    }).pinch().wheel().decelerate(), o.stage.addChild(v);
    const b = new Ht();
    v.addChild(b), v.fit(true, w * 1.15, x * 1.2), v.moveCenter(w / 2, x / 2);
    const { renderLayoutAnimated: C } = await Pn(async () => {
      const { renderLayoutAnimated: T } = await import("./animated-D3xrP_rj.js");
      return {
        renderLayoutAnimated: T
      };
    }, []);
    C(t, b, y, () => {
    });
  }
  var Ee = Uint8Array, gs = Uint16Array, h_ = Int32Array, yu = new Ee([
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
  ]), xu = new Ee([
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
  ]), d_ = new Ee([
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
  ]), bu = function(n, t) {
    for (var e = new gs(31), s = 0; s < 31; ++s) e[s] = t += 1 << n[s - 1];
    for (var i = new h_(e[30]), s = 1; s < 30; ++s) for (var r = e[s]; r < e[s + 1]; ++r) i[r] = r - e[s] << 5 | s;
    return {
      b: e,
      r: i
    };
  }, _u = bu(yu, 2), wu = _u.b, u_ = _u.r;
  wu[28] = 258, u_[258] = 28;
  var p_ = bu(xu, 0), f_ = p_.b, qo = new gs(32768);
  for (var Vt = 0; Vt < 32768; ++Vt) {
    var vn = (Vt & 43690) >> 1 | (Vt & 21845) << 1;
    vn = (vn & 52428) >> 2 | (vn & 13107) << 2, vn = (vn & 61680) >> 4 | (vn & 3855) << 4, qo[Vt] = ((vn & 65280) >> 8 | (vn & 255) << 8) >> 1;
  }
  var ni = (function(n, t, e) {
    for (var s = n.length, i = 0, r = new gs(t); i < s; ++i) n[i] && ++r[n[i] - 1];
    var o = new gs(t);
    for (i = 1; i < t; ++i) o[i] = o[i - 1] + r[i - 1] << 1;
    var a;
    if (e) {
      a = new gs(1 << t);
      var l = 15 - t;
      for (i = 0; i < s; ++i) if (n[i]) for (var c = i << 4 | n[i], h = t - n[i], d = o[n[i] - 1]++ << h, u = d | (1 << h) - 1; d <= u; ++d) a[qo[d] >> l] = c;
    } else for (a = new gs(s), i = 0; i < s; ++i) n[i] && (a[i] = qo[o[n[i] - 1]++] >> 15 - n[i]);
    return a;
  }), yi = new Ee(288);
  for (var Vt = 0; Vt < 144; ++Vt) yi[Vt] = 8;
  for (var Vt = 144; Vt < 256; ++Vt) yi[Vt] = 9;
  for (var Vt = 256; Vt < 280; ++Vt) yi[Vt] = 7;
  for (var Vt = 280; Vt < 288; ++Vt) yi[Vt] = 8;
  var vu = new Ee(32);
  for (var Vt = 0; Vt < 32; ++Vt) vu[Vt] = 5;
  var m_ = ni(yi, 9, 1), g_ = ni(vu, 5, 1), io = function(n) {
    for (var t = n[0], e = 1; e < n.length; ++e) n[e] > t && (t = n[e]);
    return t;
  }, He = function(n, t, e) {
    var s = t / 8 | 0;
    return (n[s] | n[s + 1] << 8) >> (t & 7) & e;
  }, ro = function(n, t) {
    var e = t / 8 | 0;
    return (n[e] | n[e + 1] << 8 | n[e + 2] << 16) >> (t & 7);
  }, y_ = function(n) {
    return (n + 7) / 8 | 0;
  }, x_ = function(n, t, e) {
    return (e == null || e > n.length) && (e = n.length), new Ee(n.subarray(t, e));
  }, b_ = [
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
  ], Ue = function(n, t, e) {
    var s = new Error(t || b_[n]);
    if (s.code = n, Error.captureStackTrace && Error.captureStackTrace(s, Ue), !e) throw s;
    return s;
  }, __ = function(n, t, e, s) {
    var i = n.length, r = 0;
    if (!i || t.f && !t.l) return e || new Ee(0);
    var o = !e, a = o || t.i != 2, l = t.i;
    o && (e = new Ee(i * 3));
    var c = function(Y) {
      var st = e.length;
      if (Y > st) {
        var ot = new Ee(Math.max(st * 2, Y));
        ot.set(e), e = ot;
      }
    }, h = t.f || 0, d = t.p || 0, u = t.b || 0, p = t.l, f = t.d, g = t.m, m = t.n, y = i * 8;
    do {
      if (!p) {
        h = He(n, d, 1);
        var w = He(n, d + 1, 3);
        if (d += 3, w) if (w == 1) p = m_, f = g_, g = 9, m = 5;
        else if (w == 2) {
          var b = He(n, d, 31) + 257, C = He(n, d + 10, 15) + 4, T = b + He(n, d + 5, 31) + 1;
          d += 14;
          for (var R = new Ee(T), A = new Ee(19), k = 0; k < C; ++k) A[d_[k]] = He(n, d + k * 3, 7);
          d += C * 3;
          for (var L = io(A), H = (1 << L) - 1, W = ni(A, L, 1), k = 0; k < T; ) {
            var F = W[He(n, d, H)];
            d += F & 15;
            var x = F >> 4;
            if (x < 16) R[k++] = x;
            else {
              var I = 0, z = 0;
              for (x == 16 ? (z = 3 + He(n, d, 3), d += 2, I = R[k - 1]) : x == 17 ? (z = 3 + He(n, d, 7), d += 3) : x == 18 && (z = 11 + He(n, d, 127), d += 7); z--; ) R[k++] = I;
            }
          }
          var J = R.subarray(0, b), q = R.subarray(b);
          g = io(J), m = io(q), p = ni(J, g, 1), f = ni(q, m, 1);
        } else Ue(1);
        else {
          var x = y_(d) + 4, _ = n[x - 4] | n[x - 3] << 8, v = x + _;
          if (v > i) {
            l && Ue(0);
            break;
          }
          a && c(u + _), e.set(n.subarray(x, v), u), t.b = u += _, t.p = d = v * 8, t.f = h;
          continue;
        }
        if (d > y) {
          l && Ue(0);
          break;
        }
      }
      a && c(u + 131072);
      for (var et = (1 << g) - 1, M = (1 << m) - 1, O = d; ; O = d) {
        var I = p[ro(n, d) & et], N = I >> 4;
        if (d += I & 15, d > y) {
          l && Ue(0);
          break;
        }
        if (I || Ue(2), N < 256) e[u++] = N;
        else if (N == 256) {
          O = d, p = null;
          break;
        } else {
          var D = N - 254;
          if (N > 264) {
            var k = N - 257, K = yu[k];
            D = He(n, d, (1 << K) - 1) + wu[k], d += K;
          }
          var nt = f[ro(n, d) & M], ut = nt >> 4;
          nt || Ue(3), d += nt & 15;
          var q = f_[ut];
          if (ut > 3) {
            var K = xu[ut];
            q += ro(n, d) & (1 << K) - 1, d += K;
          }
          if (d > y) {
            l && Ue(0);
            break;
          }
          a && c(u + 131072);
          var pt = u + D;
          if (u < q) {
            var ct = r - q, _t = Math.min(q, pt);
            for (ct + u < 0 && Ue(3); u < _t; ++u) e[u] = s[ct + u];
          }
          for (; u < pt; ++u) e[u] = e[u - q];
        }
      }
      t.l = p, t.p = O, t.b = u, t.f = h, p && (h = 1, t.m = g, t.d = f, t.n = m);
    } while (!h);
    return u != e.length && o ? x_(e, 0, u) : e.subarray(0, u);
  }, w_ = new Ee(0), v_ = function(n) {
    (n[0] != 31 || n[1] != 139 || n[2] != 8) && Ue(6, "invalid gzip data");
    var t = n[3], e = 10;
    t & 4 && (e += (n[10] | n[11] << 8) + 2);
    for (var s = (t >> 3 & 1) + (t >> 4 & 1); s > 0; s -= !n[e++]) ;
    return e + (t & 2);
  }, C_ = function(n) {
    var t = n.length;
    return (n[t - 4] | n[t - 3] << 8 | n[t - 2] << 16 | n[t - 1] << 24) >>> 0;
  };
  function S_(n, t) {
    var e = v_(n);
    return e + 8 > n.length && Ue(6, "invalid gzip data"), __(n.subarray(e, -8), {
      i: 2
    }, new Ee(C_(n)), t);
  }
  var T_ = typeof TextDecoder < "u" && new TextDecoder(), E_ = 0;
  try {
    T_.decode(w_, {
      stream: true
    }), E_ = 1;
  } catch {
  }
  const oo = "fls1";
  async function Cu(n) {
    const t = typeof n == "string" ? n : new TextDecoder().decode(n);
    if (!t.startsWith(oo)) throw new Error(`Not a layout snapshot: expected "${oo}" prefix, got "${t.slice(0, 4)}"`);
    const e = t.slice(oo.length), s = Uint8Array.from(atob(e), (o) => o.charCodeAt(0)), i = S_(s), r = new TextDecoder().decode(i);
    return JSON.parse(r);
  }
  function A_(n) {
    return new Promise((t, e) => {
      const s = new FileReader();
      s.onload = () => t(s.result), s.onerror = () => e(new Error("Failed to read file")), s.readAsText(n);
    });
  }
  function k_(n, t) {
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
          const i = await A_(s), r = await Cu(i);
          t(r);
        } catch (i) {
          alert(`Failed to load snapshot: ${i}`);
        }
      }
    });
  }
  function M_(n, t, e) {
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
    const g = document.createElement("span");
    g.style.cssText = "flex:1", s.appendChild(g);
    const m = document.createElement("button");
    m.textContent = "Re-solve", m.title = "Not yet implemented", m.disabled = true, m.style.cssText = "background:#222;border:1px solid #444;color:#666;padding:2px 8px;border-radius:3px;font:11px monospace;cursor:not-allowed", s.appendChild(m);
    const y = document.createElement("button");
    return y.textContent = "Clear", y.style.cssText = "background:#333;border:1px solid #666;color:#ccc;padding:2px 8px;border-radius:3px;cursor:pointer;font:11px monospace", y.addEventListener("click", () => e.onClear()), s.appendChild(y), n.insertBefore(s, n.firstChild), s;
  }
  function Rc(n, t, e, s, i, r, o, a) {
    const l = s - t, c = i - e, h = Math.sqrt(l * l + c * c);
    if (h === 0) return;
    const d = l / h, u = c / h;
    let p = 0;
    for (; p < h; ) {
      const f = Math.min(p + r, h);
      n.moveTo(t + d * p, e + u * p).lineTo(t + d * f, e + u * f).stroke(a), p = f + o;
    }
  }
  function P_(n, t, e, s, i) {
    const r = new Ht(), o = n.find((h) => h.phase === "LanesPlanned");
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
      }), Rc(g, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
        width: 1,
        color: 16724787,
        alpha: 0.6
      }), g.eventMode = "static", g.on("pointerenter", () => i(`Route failed: ${d.item} (${d.from_x},${d.from_y})\u2192(${d.to_x},${d.to_y}) [${d.spec_key}]`)), g.on("pointerleave", () => i(null)), r.addChild(g);
    }
    const l = I_;
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
      }), Rc(g, u, p, d.to_x * S + S / 2, d.to_y * S + S / 2, 6, 4, {
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
  const I_ = [
    5676246,
    6996096,
    13672512,
    11567312
  ], R_ = {
    Error: 16729156,
    Warning: 16755200
  }, L_ = 0.85;
  function $_(n, t, e) {
    const s = new Ht(), i = /* @__PURE__ */ new Map();
    for (const r of n) {
      if (r.x == null || r.y == null) continue;
      const o = R_[r.severity] ?? 4500223, a = new ft();
      a.rect(r.x * S, r.y * S, S, S).stroke({
        width: 2,
        color: o,
        alpha: L_
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
  function B_(n) {
    const t = Math.max(0, Math.min(1, n)), e = Math.round(64 + 96 * t);
    return {
      color: 255 << 16 | e << 8 | 32,
      alpha: 0.45 * (1 - t) + 0.12
    };
  }
  function O_(n, t, e) {
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
      const a = Pe[o.name];
      a && i.set(`${o.x},${o.y}`, a);
    }
    const r = new Ht();
    r.eventMode = "none";
    for (const [o, a] of s) {
      const [l, c] = o.split(",").map(Number), h = i.get(o), [d, u] = h ?? [
        1,
        1
      ], { color: p, alpha: f } = B_(a), g = new ft();
      g.rect(l * S, c * S, d * S, u * S).fill({
        color: p,
        alpha: f
      }), r.addChild(g);
    }
    return e.addChild(r), r;
  }
  function Lc(n) {
    const [t, e] = Pe[n.name] ?? [
      1,
      1
    ], s = n.x ?? 0, i = n.y ?? 0;
    return {
      cx: (s + t / 2) * S,
      cy: (i + e / 2) * S
    };
  }
  function N_(n, t, e) {
    if (!n || n.length === 0) return null;
    const s = new Ht();
    s.eventMode = "none";
    const i = new ft();
    for (const [r, o] of n) {
      const a = t[r], l = t[o];
      if (!a || !l) continue;
      const { cx: c, cy: h } = Lc(a), { cx: d, cy: u } = Lc(l);
      i.moveTo(c, h), i.lineTo(d, u);
    }
    return i.stroke({
      color: 16751164,
      width: 1.5,
      alpha: 0.85
    }), s.addChild(i), e.addChild(s), s;
  }
  function F_(n) {
    const t = n.point.direction;
    return t === "East" || t === "West" ? "horizontal" : "vertical";
  }
  function W_(n) {
    const t = new Set(n.map(F_));
    return t.size === 1 ? [
      ...t
    ][0] : "mixed";
  }
  function G_(n) {
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
    for (const a of e.values()) a.axis = W_([
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
  function z_(n) {
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
  function D_(n) {
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
  const H_ = 0.35;
  function U_(n, t, e, s, i) {
    const r = S * 0.45;
    n.setStrokeStyle({
      width: 3,
      color: i,
      alpha: H_
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
  function j_(n, t, e, s, i, r, o = 2, a = 6, l = 4, c = 0.9) {
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
  function V_(n) {
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
  function Y_(n) {
    const t = new Ht(), e = n.regions ?? [], s = [];
    if (e.length === 0) return {
      layer: t,
      items: s,
      hitTest: () => null
    };
    for (const r of e) {
      const o = G_(r), a = z_(r.kind), l = D_(o.cls), c = r.x * S, h = r.y * S, d = r.width * S, u = r.height * S;
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
      const g = r.ports ?? [], m = V_(g);
      for (const { item: y, inPort: w, outPort: x } of m) {
        const [_, v] = ao(w), [b, C] = ao(x), T = _ * S + S / 2, R = v * S + S / 2, A = b * S + S / 2, k = C * S + S / 2, L = Rn(y), H = new ft();
        j_(H, T, R, A, k, L), t.addChild(H);
      }
      for (const y of g) {
        const [w, x] = ao(y), _ = w * S + S / 2, v = x * S + S / 2, b = new ft(), C = y.item ? Rn(y.item) : 8947848;
        U_(b, _, v, y.point.direction, C), t.addChild(b);
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
  const X_ = /* @__PURE__ */ new Set([
    "JunctionGrowthStarted",
    "JunctionGrowthIteration",
    "JunctionStrategyAttempt",
    "SatInvocation",
    "JunctionSolved",
    "JunctionGrowthCapped",
    "RegionWalkerVeto"
  ]);
  function q_(n) {
    return X_.has(n.phase);
  }
  function K_(n) {
    const t = n.data;
    return typeof t.seed_x == "number" && typeof t.seed_y == "number" ? [
      t.seed_x,
      t.seed_y
    ] : [
      t.tile_x,
      t.tile_y
    ];
  }
  function J_(n, t) {
    return `${n},${t}`;
  }
  function Z_(n) {
    const t = /* @__PURE__ */ new Map(), e = (a, l) => {
      const c = J_(a, l);
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
      if (!q_(a)) continue;
      const [l, c] = K_(a), h = e(l, c);
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
  function $c(n) {
    return n.iterations.length === 0 ? null : n.iterations[n.defaultIterIndex] ?? n.iterations[n.iterations.length - 1];
  }
  function Q_(n) {
    return n ? `${n.entity_name}@(${n.entity_x},${n.entity_y}) ${n.direction}` : "";
  }
  function tw(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function ew(n, t, e) {
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
        item: tw(c.spec_key),
        specKey: c.spec_key,
        tiles: h
      });
    }
    return a;
  }
  const Bc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, nw = new Fe({
    fontFamily: "monospace",
    fontSize: 10,
    fill: 16777215,
    dropShadow: {
      color: 0,
      distance: 1,
      blur: 2,
      alpha: 0.8
    }
  }), sw = new Fe({
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
  function iw(n) {
    const t = new Ht(), e = [], s = [];
    for (const o of n) {
      if (o.outcome.kind !== "Solved") continue;
      const a = $c(o);
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
      const a = $c(o);
      if (!a) continue;
      const l = a.bbox;
      if (l.w <= 0 || l.h <= 0 || o.outcome.kind === "Capped" && i(o.seed.x, o.seed.y)) continue;
      const c = l.x * S, h = l.y * S, d = l.w * S, u = l.h * S, p = Bc[o.outcome.kind] ?? Bc.Open, f = new ft();
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
        style: nw
      });
      g.x = c + 3, g.y = h + 2, t.addChild(g);
      const m = new Ve({
        text: rw(o),
        style: sw
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
  function rw(n) {
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
  const Oc = {
    Solved: 3842122,
    Capped: 13934650,
    Open: 12599360
  }, ow = 0.04, aw = 9060416, lw = 0.55, cw = 4243680, hw = 5592405, dw = 0.55, uw = 16777215, Nc = 0.85, pw = 16777215, Fc = new Fe({
    fontFamily: "monospace",
    fontSize: 7,
    fontWeight: "700",
    fill: uw
  }), Wc = /* @__PURE__ */ new Map();
  function fw(n) {
    let t = Wc.get(n);
    if (!t) {
      const s = `/spaghettio/icons/${n}.png`;
      t = $e.load(s).catch(() => null), Wc.set(n, t);
    }
    return t;
  }
  function mw() {
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
      const { cluster: a, iter: l } = r, c = l.bbox, h = Oc[a.outcome.kind] ?? Oc.Open, d = new ft();
      d.rect(c.x * S, c.y * S, c.w * S, c.h * S).fill({
        color: h,
        alpha: ow
      }), n.addChild(d);
      const u = gw(c.x * S, c.y * S, c.w * S, c.h * S, {
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
        color: aw,
        alpha: lw
      });
      for (const [m, y] of l.forbidden) yw(p, m * S, y * S, S);
      n.addChild(p);
      const f = new ft();
      f.circle((a.seed.x + 0.5) * S, (a.seed.y + 0.5) * S, S * 0.42).stroke({
        width: 3,
        color: cw,
        alpha: 0.95
      }), n.addChild(f);
      const g = ((_a2 = l.sat) == null ? void 0 : _a2.boundaries) ?? l.boundaries;
      for (const m of g) bw(n, m, o, () => t);
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
  function gw(n, t, e, s, i) {
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
        const w = Math.min(y + i.dashLen, p);
        r.moveTo(a + f * y, l + g * y).lineTo(a + f * w, l + g * w).stroke();
      }
    }
    return r;
  }
  function yw(n, t, e, s) {
    const i = s;
    n.moveTo(t, e + i).lineTo(t + i, e).stroke(), n.moveTo(t + i / 3, e + i).lineTo(t + i, e + 2 * i / 3).stroke(), n.moveTo(t, e + 2 * i / 3).lineTo(t + 2 * i / 3, e).stroke();
  }
  function xw(n, t) {
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
  function bw(n, t, e, s) {
    const i = t.x * S, r = t.y * S, o = S / 3, a = xw(t.is_input, t.direction), l = a === "top" || a === "bottom";
    let c = i, h = r, d = S, u = S;
    a === "top" ? u = o : a === "bottom" ? (h = r + S - o, u = o) : (a === "left" || (c = i + S - o), d = o);
    const p = new ft();
    p.rect(c, h, d, u).fill({
      color: hw,
      alpha: dw
    }), t.interior && (p.setStrokeStyle({
      width: 1,
      color: pw,
      alpha: 0.5
    }), p.rect(c, h, d, u).stroke()), n.addChild(p);
    const f = _w(t.direction), [g, m, y, w] = l ? [
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
      style: Fc
    });
    x.anchor.set(0.5), x.x = g, x.y = m, x.alpha = Nc, n.addChild(x);
    const _ = new Ve({
      text: f,
      style: Fc
    });
    _.anchor.set(0.5), _.x = y, _.y = w, _.alpha = Nc, n.addChild(_);
    const v = c + d / 2, b = h + u / 2;
    fw(t.item).then((C) => {
      if (e !== s() || !C) return;
      const T = new Ne(C);
      T.anchor.set(0.5), T.x = v, T.y = b;
      const R = o * 0.95;
      T.width = R, T.height = R, n.addChild(T);
    });
  }
  function _w(n) {
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
  const ww = 4247776, vw = 0.18;
  function Cw(n) {
    if (!n || n.length === 0) return null;
    const t = /* @__PURE__ */ new Set();
    for (const i of n) if (i.phase === "GhostSpecRouted") for (const [r, o] of i.data.tiles) t.add(`${r},${o}`);
    if (t.size === 0) return null;
    const e = new Ht();
    e.label = "ghost-tiles-overlay";
    const s = new ft();
    for (const i of t) {
      const [r, o] = i.split(","), a = Number(r), l = Number(o);
      s.rect(a * S, l * S, S, S).fill({
        color: ww,
        alpha: vw
      });
    }
    return e.addChild(s), e;
  }
  function Sw(n, t, e) {
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
    const R = document.createElement("div");
    R.className = "jd-footer", R.textContent = "Esc close \xB7 \u2190/\u2192 step all \xB7 w/s iter \xB7 a/d variant \xB7 Home/End first/last", x.append(_, T, R), n.append(w, x);
    let A = null, k = 0, L = null, H = false;
    function W(Y, st) {
      A = Y, L = st ?? null, k = Y.defaultIterIndex, s.classList.add("jd-open"), nt(), K(), M();
    }
    function F() {
      A && (z(), A = null, L = null, s.classList.remove("jd-open"), e.onChange(null));
    }
    function I() {
      !A || H || (H = true, w.classList.add("jd-open"), x.classList.add("jd-open"), ut());
    }
    function z() {
      H && (H = false, w.classList.remove("jd-open"), x.classList.remove("jd-open"));
    }
    function J() {
      H ? z() : I();
    }
    function q() {
      return A !== null;
    }
    function et(Y) {
      if (!A) return;
      const st = Math.max(0, Math.min(A.iterations.length - 1, Y));
      st !== k && (k = st, nt(), K(), M());
    }
    function M() {
      if (!A) return;
      const Y = A.iterations[k];
      if (!Y) return;
      const st = t.toScreen(Y.bbox.x * S, Y.bbox.y * S), ot = t.toScreen((Y.bbox.x + Y.bbox.w) * S, (Y.bbox.y + Y.bbox.h) * S), at = n.getBoundingClientRect();
      if (!(ot.x < 0 || st.x > at.width || ot.y < 0 || st.y > at.height)) return;
      const zt = (Y.bbox.x + Y.bbox.w / 2) * S, At = (Y.bbox.y + Y.bbox.h / 2) * S;
      t.moveCenter(zt, At);
    }
    function O() {
      const Y = /* @__PURE__ */ new Map(), st = A;
      if (!st) return Y;
      for (let ot = 0; ot < st.iterations.length; ot++) {
        const at = st.iterations[ot], bt = Y.get(at.iter) ?? [];
        bt.push(ot), Y.set(at.iter, bt);
      }
      return Y;
    }
    function N(Y) {
      if (!A) return;
      const st = O(), ot = Array.from(st.keys()).sort((j, Z) => j - Z), at = A.iterations[k].iter, bt = ot.indexOf(at), zt = ot[Math.max(0, Math.min(ot.length - 1, bt + Y))], At = st.get(zt) ?? [], Xt = At.find((j) => A.iterations[j].variant === "");
      et(Xt ?? At[0] ?? k);
    }
    function D(Y) {
      if (!A) return;
      const st = O(), ot = A.iterations[k].iter, at = st.get(ot) ?? [];
      if (at.length <= 1) return;
      const zt = (at.indexOf(k) + Y + at.length) % at.length;
      et(at[zt]);
    }
    function K() {
      if (!A) return;
      const Y = A.iterations[k];
      if (!Y) return;
      const st = n.getBoundingClientRect(), ot = s.offsetWidth || 200, at = s.offsetHeight || 70, bt = (Y.bbox.x + Y.bbox.w) * S, zt = (Y.bbox.y + Y.bbox.h) * S, At = Y.bbox.y * S, Xt = t.toScreen(bt, zt), j = t.toScreen(bt, At);
      let Z = Xt.x - ot, Q = Xt.y;
      Q + at > st.height - 4 && (Q = j.y - at), Z = Math.max(4, Math.min(Z, st.width - ot - 4)), Q = Math.max(4, Math.min(Q, st.height - at - 4)), s.style.left = `${Z}px`, s.style.top = `${Q}px`;
    }
    t.on("moved", K), t.on("zoomed", K), window.addEventListener("resize", K);
    function nt() {
      if (!A) return;
      const Y = A, st = Y.iterations[k];
      r.textContent = `Junction (${Y.seed.x},${Y.seed.y})`, o.className = `jd-status-pill jd-${Y.outcome.kind.toLowerCase()}`, o.textContent = Gc(Y);
      const ot = Y.iterations.length, at = st && st.variant ? ` \xB7 ${st.variant}` : "";
      f.textContent = `iter ${st ? st.iter : "-"}${at} \xB7 ${k + 1}/${ot}`, p.disabled = k <= 0, g.disabled = k >= ot - 1, m.disabled = k === Y.defaultIterIndex, y.innerHTML = "";
      for (const bt of Ew(Y, st)) {
        const zt = document.createElement("div");
        zt.className = `jd-inline-summary-row jd-inline-summary-row--${bt.tone}`, zt.textContent = bt.text, y.appendChild(zt);
      }
      H && ut(), st && e.onChange({
        cluster: Y,
        iter: st,
        trace: L
      });
    }
    function ut() {
      if (!A) return;
      const Y = A, st = Y.iterations[k];
      b.className = `jd-status-pill jd-${Y.outcome.kind.toLowerCase()}`, b.textContent = Gc(Y), v.textContent = `Junction (${Y.seed.x},${Y.seed.y})`, kw(T, Y, st);
    }
    d.addEventListener("click", F), a.addEventListener("click", J), l.addEventListener("click", _t), c.addEventListener("click", () => {
      if (!A) return;
      const Y = Ko(A, A.iterations[k], L);
      Ow(c, Y);
    }), h.addEventListener("click", () => {
      if (!A || !e.onEditRequested) return;
      const Y = A.iterations[k];
      Y && e.onEditRequested({
        cluster: A,
        iter: Y,
        trace: L
      });
    }), C.addEventListener("click", z), w.addEventListener("click", z);
    function pt() {
      if (!A) return null;
      const Y = A.iterations[k];
      return Y ? {
        cluster: A,
        iter: Y,
        trace: L
      } : null;
    }
    function ct(Y) {
      s.classList.toggle("jd-edit-mode", Y), c.disabled = Y, l.disabled = Y, h.disabled = Y;
    }
    function _t() {
      var _a2;
      if (!A) return;
      const Y = Tw(A, k), st = JSON.stringify(Y, (at, bt) => typeof bt == "bigint" ? String(bt) : bt, 2), ot = (at) => {
        const bt = l.textContent;
        l.textContent = at ? "\u2713" : "!", l.classList.add("jd-inline-btn--flash"), window.setTimeout(() => {
          l.textContent = bt, l.classList.remove("jd-inline-btn--flash");
        }, 900);
      };
      if ((_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText) navigator.clipboard.writeText(st).then(() => ot(true), () => ot(false));
      else {
        const at = document.createElement("textarea");
        at.value = st, at.style.position = "fixed", at.style.opacity = "0", document.body.appendChild(at), at.select();
        try {
          document.execCommand("copy"), ot(true);
        } catch {
          ot(false);
        }
        document.body.removeChild(at);
      }
    }
    return p.addEventListener("click", () => et(k - 1)), g.addEventListener("click", () => et(k + 1)), m.addEventListener("click", () => {
      A && et(A.defaultIterIndex);
    }), document.addEventListener("keydown", (Y) => {
      var _a2, _b2;
      if (!q()) return;
      const st = (_b2 = (_a2 = Y.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (st === "INPUT" || st === "TEXTAREA" || st === "SELECT") return;
      const ot = Y.key, at = () => {
        Y.stopImmediatePropagation(), Y.preventDefault();
      };
      ot === "Escape" ? (H ? z() : F(), at()) : ot === "ArrowLeft" ? (et(k - 1), at()) : ot === "ArrowRight" ? (et(k + 1), at()) : ot === "Home" ? (et(0), at()) : ot === "End" && A ? (et(A.iterations.length - 1), at()) : ot === "w" || ot === "W" ? (N(-1), at()) : ot === "s" || ot === "S" ? (N(1), at()) : ot === "a" || ot === "A" ? (D(-1), at()) : ot === "d" || ot === "D" ? (D(1), at()) : (ot === "i" || ot === "I") && (J(), at());
    }, {
      capture: true
    }), {
      open: W,
      close: F,
      isOpen: q,
      inlineEl: s,
      getSelection: pt,
      setEditMode: ct
    };
  }
  function Tw(n, t) {
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
  function Gc(n) {
    switch (n.outcome.kind) {
      case "Solved":
        return `Solved \xB7 ${n.outcome.regionTiles}t`;
      case "Capped":
        return `Capped \xB7 ${n.outcome.iters} iter`;
      case "Open":
        return "Open";
    }
  }
  function Ew(n, t) {
    if (t) {
      if (t.veto) return [
        {
          text: `veto \xB7 ${Gi(t.veto.broken_segment, 22)} @ (${t.veto.break_tile_x},${t.veto.break_tile_y})`,
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
        const i = s.detail ? ` \xB7 ${Gi(s.detail, 28)}` : "";
        return [
          {
            text: `${s.strategy} \u2192 ${Gi(s.outcome, 12)}${i}`,
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
            text: `cap: ${Gi(n.outcome.reason, 32)}`,
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
  function Gi(n, t) {
    return n.length <= t ? n : `${n.slice(0, t - 1)}\u2026`;
  }
  function Su(n) {
    var _a2;
    return ((_a2 = n.sat) == null ? void 0 : _a2.boundaries) ?? n.boundaries;
  }
  function Aw(n) {
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
  function kw(n, t, e) {
    n.innerHTML = "", n.appendChild(Mw(t)), n.appendChild(Pw(t, e)), e && (n.appendChild(Iw(e)), n.appendChild(Rw(e)), n.appendChild(Lw(e)), e.veto && n.appendChild($w(e))), t.nearbyStamped.length > 0 && n.appendChild(Bw(t));
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
  function Mw(n) {
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
  function Pw(n, t) {
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
  function Iw(n) {
    const t = Su(n), e = n.sat ? " (as fed to SAT)" : " (spec perimeter)", { details: s, bodyEl: i } = ts(`Boundaries${e}`);
    if (t.length === 0) {
      const r = document.createElement("div");
      return r.className = "jd-row jd-row--dim", r.textContent = "(none)", i.appendChild(r), s;
    }
    for (const r of t) {
      const o = document.createElement("div");
      o.className = "jd-row";
      const a = r.is_input ? "IN " : "OUT", l = r.interior ? " (interior)" : "", c = r.external_feeder ? ` \u2190 ${Q_(r.external_feeder)}` : "";
      o.style.color = r.is_input ? "#9f9" : "#f99";
      const h = r.spec_key ? ` \xB7 ${r.spec_key}` : "";
      o.textContent = `${a} (${r.x},${r.y}) ${Aw(r.direction)} ${r.direction} \xB7 ${r.item}${l}${h}${c}`, i.appendChild(o);
    }
    return s;
  }
  function Rw(n) {
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
  function Lw(n) {
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
  function $w(n) {
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
  function Bw(n) {
    const { details: t, bodyEl: e } = ts("Nearby stamped", false);
    for (const s of n.nearbyStamped) {
      const i = document.createElement("div");
      i.className = "jd-row";
      const r = s.carries ? ` carries=${s.carries}` : "", o = s.segment_id ? ` \xB7 seg=${s.segment_id}` : "";
      i.textContent = `(${s.x},${s.y}) ${s.name} ${s.direction}${r}${o}${s.feeds_seed_area ? "  \u26A0 feeds seed" : ""}`, e.appendChild(i);
    }
    return t;
  }
  const Tu = {
    "transport-belt": 4,
    "fast-transport-belt": 6,
    "express-transport-belt": 8
  };
  function Ko(n, t, e, s) {
    var _a2, _b2;
    const i = n.seed, r = (t == null ? void 0 : t.bbox) ?? {
      x: i.x,
      y: i.y,
      w: 1,
      h: 1
    }, o = (t == null ? void 0 : t.iter) ?? 0, a = ((_a2 = t == null ? void 0 : t.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", l = ((_b2 = t == null ? void 0 : t.sat) == null ? void 0 : _b2.max_reach) ?? Tu[a] ?? 4, c = Su(t ?? {
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
    })), h = t && e ? ew(e, r, 2).map((p) => ({
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
  function Ow(n, t) {
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
  function Nw(n) {
    return n.x === void 0 || n.y === void 0 || n.direction === void 0 ? null : n;
  }
  const zi = {
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
  }, zc = {
    North: "East",
    East: "South",
    South: "West",
    West: "North"
  }, Di = {
    "transport-belt": "transport-belt",
    "fast-transport-belt": "fast-transport-belt",
    "express-transport-belt": "express-transport-belt"
  }, Dc = {
    "transport-belt": "underground-belt",
    "fast-transport-belt": "fast-underground-belt",
    "express-transport-belt": "express-underground-belt"
  };
  function Fw(n) {
    const { viewport: t, canvas: e, engine: s, jd: i, satZoneOverlayLayer: r } = n;
    let o = null, a = null, l = null, c = null, h = null, d = null, u = false, p = null, f = [], g = [], m = [], y = [], w = "belt", x = "East", _ = 0, v = null, b = "idle", C = 0, T = null, R = null;
    function A(E, P) {
      return `${E},${P}`;
    }
    function k(E, P) {
      if (!p) return false;
      const X = p.bbox;
      return E >= X.x && E < X.x + X.w && P >= X.y && P < X.y + X.h;
    }
    function L(E, P) {
      return f.find((X) => X.x === E && X.y === P);
    }
    function H() {
      if (!p || p.items.length === 0) return null;
      const E = Math.max(0, Math.min(_, p.items.length - 1));
      return p.items[E] ?? null;
    }
    function W(E, P, X) {
      if (!p) return null;
      const [V, $] = zi[X], B = E - V, tt = P - $, U = L(B, tt);
      if (U && U.direction === X && (Di[U.name] === U.name || U.io_type === "output")) return U.carries ?? null;
      const it = p.boundaries.find((mt) => mt.x === E && mt.y === P && mt.isInput && mt.dir === X);
      return it ? it.item : null;
    }
    function F(E, P) {
      return W(E.x, E.y, P) ?? H();
    }
    function I(E, P) {
      if (!p) return null;
      const [X, V] = zi[P];
      for (let $ = 1; $ <= p.maxReach + 1; $++) {
        const B = E.x - X * $, tt = E.y - V * $, U = L(B, tt);
        if (U && U.io_type === "input" && U.direction === P) return U.carries ?? null;
      }
      return null;
    }
    function z() {
      g.push(f.map((E) => ({
        ...E
      }))), g.length > 64 && g.shift(), m.length = 0;
    }
    function J(E, P) {
      z(), f = E, et(), ut();
    }
    function q() {
      if (!p) return [];
      const E = Di[p.beltTier] ?? "transport-belt", P = [];
      for (const X of p.boundaries) {
        if (!X.isInput) continue;
        const [V, $] = zi[X.dir];
        P.push({
          name: E,
          x: X.x - V,
          y: X.y - $,
          direction: X.dir,
          carries: X.item
        });
      }
      return P;
    }
    function et() {
      const E = q();
      o && ei({
        entities: f
      }, o, void 0, void 0, void 0, E), a && ei({
        entities: y
      }, a, void 0, void 0, void 0, E), M();
    }
    function M() {
      if (c) {
        c.removeChildren();
        for (const E of f) {
          const P = E.carries;
          if (!P) continue;
          const X = `/spaghettio/icons/${P}.png`, V = $e.get(X);
          if (!V) continue;
          const $ = new Ne(V), B = S * 0.55;
          $.width = B, $.height = B, $.x = E.x * S + (S - B) / 2, $.y = E.y * S + (S - B) / 2, $.alpha = 0.85, c.addChild($);
        }
      }
    }
    function O(E, P) {
      l && (ei({
        entities: E
      }, l, void 0, void 0, void 0, q()), l.alpha = P ? 0.5 : 0.45, l.tint = P ? 16733525 : 16777215);
    }
    function N() {
      l && (l.removeChildren(), l.tint = 16777215);
    }
    function D(E, P = "") {
      if (b = E, !d) return;
      d.classList.remove("ok", "solving", "invalid", "idle"), d.classList.add(E === "valid" ? "ok" : E);
      const X = E === "valid" ? "\u25CF" : E === "solving" ? "\u25D4" : E === "invalid" ? "\u25CF" : "\u25CB";
      d.textContent = X;
      let V = "";
      E === "valid" ? V = R !== null ? `valid \xB7 cost ${R} / yours ${K(f)}` : "valid" : E === "solving" ? V = "solving\u2026" : E === "invalid" ? V = "invalid" : V = "no edits yet", d.title = P ? `${V}
${P}` : V, Ce();
    }
    function K(E) {
      let P = 0;
      for (const X of E) Di[X.name] === X.name ? P += 1 : Dc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] === X.name && (P += 5);
      return P;
    }
    function nt() {
      if (!p) return "no zone";
      const E = /* @__PURE__ */ new Set();
      for (const P of f) {
        const X = A(P.x, P.y);
        if (E.has(X)) return `duplicate entity at (${P.x},${P.y})`;
        if (E.add(X), !k(P.x, P.y)) return `entity at (${P.x},${P.y}) outside bbox`;
        if (p.forbidden.has(X)) return `entity at (${P.x},${P.y}) on forbidden tile`;
      }
      for (const P of f) {
        if (P.io_type !== "input") continue;
        const [X, V] = zi[P.direction];
        let $ = false;
        for (let B = 1; B <= p.maxReach + 1; B++) {
          const tt = P.x + X * B, U = P.y + V * B, it = L(tt, U);
          if (it) {
            if (it.io_type === "output" && it.direction === P.direction && it.carries === P.carries) {
              $ = true;
              break;
            }
            if (it.io_type === "input" && it.carries === P.carries) return `UG-in at (${P.x},${P.y}) blocked by another UG-in at (${tt},${U})`;
          }
        }
        if (!$) return `UG-in at (${P.x},${P.y}) has no matching UG-out within reach ${p.maxReach}`;
      }
      return null;
    }
    function ut() {
      if (!u || !p) return;
      const E = nt();
      if (E) {
        y = [], R = null, et(), D("invalid", E);
        return;
      }
      D("solving"), T !== null && window.clearTimeout(T);
      const P = ++C;
      T = window.setTimeout(() => {
        pt(P);
      }, 300);
    }
    async function pt(E) {
      if (p) try {
        const P = await s.solveFixture(p.fixtureJson, f);
        if (E !== C || !u) return;
        if (!P) {
          y = [], R = null, et(), D("invalid", "SAT cannot complete this layout");
          return;
        }
        const X = new Set(f.map(($) => A($.x, $.y))), V = [];
        for (const $ of P.entities) {
          const B = Nw($);
          B && !X.has(A(B.x, B.y)) && V.push(B);
        }
        y = V, R = P.cost, et(), D("valid");
      } catch (P) {
        if (E !== C) return;
        y = [], et(), D("invalid", `solver error: ${P instanceof Error ? P.message : String(P)}`);
      }
    }
    function ct(E) {
      const P = e.getBoundingClientRect(), X = t.toWorld(E.clientX - P.left, E.clientY - P.top);
      return {
        x: Math.floor(X.x / S),
        y: Math.floor(X.y / S)
      };
    }
    function _t(E, P, X) {
      const V = [], $ = P.x === E.x ? 0 : P.x > E.x ? 1 : -1, B = P.y === E.y ? 0 : P.y > E.y ? 1 : -1;
      if (X) {
        for (let U = E.y; U !== P.y + B && B !== 0; U += B) V.push({
          x: E.x,
          y: U
        });
        B === 0 && V.push({
          x: E.x,
          y: E.y
        });
        for (let U = E.x + $; $ !== 0 && U !== P.x + $; U += $) V.push({
          x: U,
          y: P.y
        });
      } else {
        for (let U = E.x; U !== P.x + $ && $ !== 0; U += $) V.push({
          x: U,
          y: E.y
        });
        $ === 0 && V.push({
          x: E.x,
          y: E.y
        });
        for (let U = E.y + B; B !== 0 && U !== P.y + B; U += B) V.push({
          x: P.x,
          y: U
        });
      }
      const tt = [];
      for (const U of V) {
        const it = tt[tt.length - 1];
        (!it || it.x !== U.x || it.y !== U.y) && tt.push(U);
      }
      return tt;
    }
    function Y(E, P) {
      return E.x === P.x && E.y === P.y - 1 ? "South" : E.x === P.x && E.y === P.y + 1 ? "North" : E.y === P.y && E.x === P.x - 1 ? "East" : E.y === P.y && E.x === P.x + 1 ? "West" : null;
    }
    function st(E) {
      if (!p || E.length === 0) return null;
      for (const it of E) if (!k(it.x, it.y)) return null;
      const P = (it) => !p.forbidden.has(A(it.x, it.y)), X = E[0], V = E[E.length - 1];
      if (!X || !V || !P(X) || !P(V)) return null;
      const $ = [], B = E.length > 1 ? Y(E[0], E[1]) ?? x : x, tt = F(E[0], B);
      let U = 0;
      for (; U < E.length; ) {
        const it = E[U], mt = E[U + 1] ?? null, Ot = mt ? Y(it, mt) : U > 0 ? Y(E[U - 1], it) : x;
        if (!Ot) return null;
        let Bt = U + 1;
        for (; Bt < E.length && P(E[Bt]); ) Bt++;
        if (Bt === E.length) {
          $.push(ot(it, Ot, tt)), U++;
          continue;
        }
        let jt = Bt;
        for (; jt < E.length && !P(E[jt]); ) jt++;
        if (jt === E.length) return null;
        const me = E[Bt - 1], es = E[jt];
        if (Math.abs(es.x - me.x) + Math.abs(es.y - me.y) > p.maxReach + 1) return null;
        for (let Ge = U; Ge < Bt - 1; Ge++) {
          const ns = E[Ge], bn = Y(ns, E[Ge + 1]);
          if (!bn) return null;
          $.push(ot(ns, bn, tt));
        }
        const Is = Y(me, es);
        if (!Is) return null;
        $.push(at(me, Is, "input", tt)), $.push(at(es, Is, "output", tt)), U = jt + 1;
      }
      return $;
    }
    function ot(E, P, X) {
      return {
        name: Di[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "transport-belt",
        x: E.x,
        y: E.y,
        direction: P,
        carries: X ?? void 0
      };
    }
    function at(E, P, X, V) {
      return {
        name: Dc[(p == null ? void 0 : p.beltTier) ?? "transport-belt"] ?? "underground-belt",
        x: E.x,
        y: E.y,
        direction: P,
        io_type: X,
        carries: V ?? void 0
      };
    }
    function bt(E) {
      const P = new Set(E.map((V) => A(V.x, V.y))), X = f.filter((V) => !P.has(A(V.x, V.y))).concat(E);
      J(X);
    }
    function zt(E) {
      if (!p || !k(E.x, E.y) || !f.some((X) => X.x === E.x && X.y === E.y)) return;
      const P = f.filter((X) => !(X.x === E.x && X.y === E.y));
      J(P);
    }
    function At(E) {
      if (!p) return;
      const P = p.boundaries.find((V) => V.x === E.x && V.y === E.y && V.isInput);
      if (!P) return;
      const X = p.items.indexOf(P.item);
      X >= 0 && X !== _ && (_ = X, Ut());
    }
    function Xt(E) {
      if (!u || !p || E.button !== 0) return;
      const P = ct(E);
      if (!P || !k(P.x, P.y)) return;
      if (At(P), w === "erase") {
        zt(P), E.stopPropagation(), E.preventDefault();
        return;
      }
      if (w === "ug-in" || w === "ug-out") {
        const $ = w === "ug-in" ? W(P.x, P.y, x) ?? H() : I(P, x) ?? W(P.x, P.y, x) ?? H(), B = w === "ug-in" ? at(P, x, "input", $) : at(P, x, "output", $), tt = f.filter((U) => !(U.x === P.x && U.y === P.y)).concat(B);
        J(tt), E.stopPropagation(), E.preventDefault();
        return;
      }
      v = {
        startX: P.x,
        startY: P.y,
        bendVerticalFirst: false
      };
      const X = _t({
        x: P.x,
        y: P.y
      }, P, false), V = st(X);
      O(V ?? [], V === null), E.stopPropagation(), E.preventDefault();
    }
    function j(E) {
      if (!u || !v || !p) return;
      const P = ct(E);
      if (!P) return;
      const X = _t({
        x: v.startX,
        y: v.startY
      }, P, v.bendVerticalFirst), V = st(X);
      O(V ?? [], V === null);
    }
    function Z(E) {
      if (!u || !v || !p) {
        v = null, N();
        return;
      }
      const P = ct(E);
      if (!P) {
        v = null, N();
        return;
      }
      const X = _t({
        x: v.startX,
        y: v.startY
      }, P, v.bendVerticalFirst), V = st(X);
      if (v = null, N(), !V) {
        D("invalid", "drag rejected: out of bounds, on obstacle, or UG too long");
        return;
      }
      bt(V), E.stopPropagation(), E.preventDefault();
    }
    function Q(E) {
      var _a2, _b2;
      if (!u) return;
      const P = (_b2 = (_a2 = E.target) == null ? void 0 : _a2.tagName) == null ? void 0 : _b2.toUpperCase();
      if (P === "INPUT" || P === "TEXTAREA" || P === "SELECT") return;
      const X = () => {
        E.stopImmediatePropagation(), E.preventDefault();
      };
      if (E.key === "Escape") {
        St(), X();
        return;
      }
      if (E.key === "1") {
        wt("belt"), X();
        return;
      }
      if (E.key === "2") {
        wt("ug-in"), X();
        return;
      }
      if (E.key === "3") {
        wt("ug-out"), X();
        return;
      }
      if (E.key === "0") {
        wt("erase"), X();
        return;
      }
      if (E.key === "r" || E.key === "R") {
        v ? v.bendVerticalFirst = !v.bendVerticalFirst : (x = zc[x], Ut()), X();
        return;
      }
      if (E.key === "[" && p) {
        _ = (_ - 1 + p.items.length) % p.items.length, Ut(), X();
        return;
      }
      if (E.key === "]" && p) {
        _ = (_ + 1) % p.items.length, Ut(), X();
        return;
      }
      if ((E.key === "Enter" || E.key === "a" || E.key === "A") && b === "valid" && y.length > 0) {
        $t(), X();
        return;
      }
      if ((E.ctrlKey || E.metaKey) && (E.key === "z" || E.key === "Z")) {
        E.shiftKey ? oe() : Ct(), X();
        return;
      }
    }
    function wt(E) {
      w = E, Ut();
    }
    function Ct() {
      g.length !== 0 && (m.push(f), f = g.pop(), et(), ut());
    }
    function oe() {
      m.length !== 0 && (g.push(f), f = m.pop(), et(), ut());
    }
    function $t() {
      if (y.length === 0) return;
      const E = f.concat(y.map((P) => ({
        ...P
      })));
      y = [], J(E);
    }
    function Ut() {
      if (!h) return;
      h.innerHTML = "";
      const E = [
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
      for (const [it, mt, Ot] of E) {
        const Bt = document.createElement("button");
        Bt.className = "se-tool" + (w === it ? " se-tool-active" : ""), Bt.textContent = mt, Bt.title = Ot, Bt.addEventListener("click", () => wt(it)), h.appendChild(Bt);
      }
      const P = document.createElement("button");
      P.className = "se-dir";
      const X = {
        North: "\u2191",
        East: "\u2192",
        South: "\u2193",
        West: "\u2190"
      };
      if (P.textContent = X[x], P.title = "Brush direction (R rotates)", P.addEventListener("click", () => {
        x = zc[x], Ut();
      }), h.appendChild(P), p && p.items.length > 1) {
        const it = document.createElement("select");
        it.className = "se-item";
        for (const [mt, Ot] of p.items.entries()) {
          const Bt = document.createElement("option");
          Bt.value = String(mt), Bt.textContent = Ot, mt === _ && (Bt.selected = true), it.appendChild(Bt);
        }
        it.addEventListener("change", () => {
          _ = Number(it.value) | 0;
        }), h.appendChild(it);
      } else if (p && p.items.length === 1) {
        const it = document.createElement("span");
        it.className = "se-item-label", it.textContent = p.items[0], h.appendChild(it);
      }
      const V = document.createElement("span");
      V.style.flex = "1", h.appendChild(V);
      const $ = document.createElement("button");
      $.className = "se-accept", $.textContent = "Accept", $.title = "Promote ghost into painted layer (Enter)", $.addEventListener("click", $t), $.disabled = !(b === "valid" && y.length > 0), h.appendChild($);
      const B = document.createElement("button");
      B.className = "se-revert", B.textContent = "Revert", B.title = "Discard all painted edits", B.addEventListener("click", () => {
        J([]);
      }), h.appendChild(B);
      const tt = document.createElement("button");
      tt.className = "se-export", tt.textContent = "Export", tt.title = "Save fixture JSON (clipboard + download)", tt.addEventListener("click", xe), tt.disabled = b !== "valid", h.appendChild(tt);
      const U = document.createElement("button");
      U.className = "se-done", U.textContent = "Done", U.title = "Exit edit mode (Esc)", U.addEventListener("click", St), h.appendChild(U);
    }
    function Ce() {
      if (!h) return;
      const E = h.querySelector(".se-accept");
      E && (E.disabled = !(b === "valid" && y.length > 0));
      const P = h.querySelector(".se-export");
      P && (P.disabled = b !== "valid");
    }
    function xe() {
      var _a2;
      if (!p || b !== "valid") return;
      const E = R ?? K(f), P = Ko(p.selection.cluster, p.selection.iter, p.selection.trace, {
        maxCost: E,
        paintedEntities: f
      }), X = (p.selection.cluster.seed ? `fixture_${p.selection.cluster.seed.x}_${p.selection.cluster.seed.y}_painted` : "fixture_painted") + ".json";
      (_a2 = navigator.clipboard) == null ? void 0 : _a2.writeText(P).catch(() => {
      });
      const V = new Blob([
        P
      ], {
        type: "application/json"
      }), $ = URL.createObjectURL(V), B = document.createElement("a");
      B.href = $, B.download = X, document.body.appendChild(B), B.click(), document.body.removeChild(B), URL.revokeObjectURL($);
    }
    function Et(E) {
      var _a2, _b2, _c2, _d2;
      u && St();
      const P = E.iter, X = ((_a2 = P.sat) == null ? void 0 : _a2.belt_tier) ?? "transport-belt", V = ((_b2 = P.sat) == null ? void 0 : _b2.max_reach) ?? Tu[X] ?? 4, $ = ((_c2 = P.sat) == null ? void 0 : _c2.boundaries) ?? P.boundaries, B = Array.from(new Set($.map((U) => U.item))), tt = $.map((U) => ({
        x: U.x,
        y: U.y,
        item: U.item,
        isInput: U.is_input,
        dir: U.direction
      }));
      p = {
        bbox: {
          x: P.bbox.x,
          y: P.bbox.y,
          w: P.bbox.w,
          h: P.bbox.h
        },
        forbidden: new Set((P.forbidden ?? []).map((U) => `${U[0]},${U[1]}`)),
        beltTier: X,
        maxReach: V,
        items: B,
        boundaries: tt,
        fixtureJson: Ko(E.cluster, E.iter, E.trace),
        selection: E
      }, f = [], g = [], m = [], y = [], w = "belt", x = "East", _ = 0, v = null, R = null, o = new Ht(), a = new Ht(), a.alpha = 0.55, l = new Ht(), c = new Ht(), t.addChild(o), t.addChild(a), t.addChild(c), t.addChild(l), t.setChildIndex(r, t.children.length - 1), h = document.createElement("div"), h.className = "se-toolbar", i.inlineEl.appendChild(h), d = document.createElement("span"), d.className = "se-status", (_d2 = i.inlineEl.querySelector(".jd-inline-head")) == null ? void 0 : _d2.appendChild(d), i.setEditMode(true), t.plugins.pause("drag"), e.addEventListener("pointerdown", Xt, {
        capture: true
      }), e.addEventListener("pointerup", Z, {
        capture: true
      }), e.addEventListener("pointermove", j, {
        capture: true
      }), document.addEventListener("keydown", Q, {
        capture: true
      }), u = true, Ut(), D("idle");
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
      }), c = null), h && (h.remove(), h = null), d && (d.remove(), d = null), e.removeEventListener("pointerdown", Xt, {
        capture: true
      }), e.removeEventListener("pointerup", Z, {
        capture: true
      }), e.removeEventListener("pointermove", j, {
        capture: true
      }), document.removeEventListener("keydown", Q, {
        capture: true
      }), t.plugins.resume("drag"), i.setEditMode(false), p = null, f = [], g = [], m = [], y = []);
    }
    function qt() {
      return u;
    }
    return {
      enter: Et,
      exit: St,
      isActive: qt
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
  const Ki = [];
  function Ww() {
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
  function Eu() {
    return _s;
  }
  function Cn(n) {
    _s = {
      ..._s,
      ...n
    }, "master" in n && localStorage.setItem("fk-debug", n.master ? "1" : "0"), "satZones" in n && localStorage.setItem("fk-sat-zones", n.satZones ? "1" : "0"), "ghostTiles" in n && localStorage.setItem("fk-ghost-tiles", n.ghostTiles ? "1" : "0"), "itemColors" in n && localStorage.setItem("fk-item-colors", n.itemColors ? "1" : "0"), "traceOverlay" in n && localStorage.setItem("fk-trace-overlay", n.traceOverlay ? "1" : "0"), "heatmap" in n && localStorage.setItem("fk-heatmap", n.heatmap ? "1" : "0"), "powerWires" in n && localStorage.setItem("fk-power-wires", n.powerWires ? "1" : "0");
    for (const t of Ki) t(_s);
  }
  function Gw(n) {
    return Ki.push(n), () => {
      const t = Ki.indexOf(n);
      t >= 0 && Ki.splice(t, 1);
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
  function zw(n) {
    n.style.position = "relative";
    const t = document.createElement("div");
    t.className = "overlay-panel";
    const e = Eu(), s = Sn(t, "Debug", e.master), i = Sn(t, "Item colours", e.itemColors), r = Sn(t, "Starvation heatmap", e.heatmap), o = Sn(t, "Power wires", e.powerWires), a = document.createElement("div");
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
  function Dw(n) {
    const t = document.createElement("div");
    t.className = "retry-panel", n.appendChild(t);
    let e = null;
    function s(r) {
      if (!(r == null ? void 0 : r.trace)) return null;
      for (const o of r.trace) if (o.phase === "LayoutRetried") return o.data;
      return null;
    }
    function i() {
      const r = Eu().master, o = s(e);
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
    return Gw(() => i()), {
      update(r) {
        e = r, i();
      }
    };
  }
  const Hw = {
    North: "\u2191",
    East: "\u2192",
    South: "\u2193",
    West: "\u2190"
  }, Hc = {
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
  function Uw(n) {
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
    const f = document.createElement("div"), g = an(16), m = document.createElement("span");
    f.append(g, m), f.style.display = "none", i.appendChild(f);
    function y() {
      const $ = document.createElement("div");
      $.style.color = "#aaa";
      const B = document.createElement("span"), tt = an(14), U = document.createElement("span");
      return $.append(B, tt, U), $;
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
    const R = document.createElement("div");
    R.style.cssText = "display:flex;justify-content:space-between;align-items:center;margin-bottom:4px";
    const A = document.createElement("span");
    A.style.cssText = "color:#8af;font-weight:bold", A.textContent = "pinned";
    const k = document.createElement("span");
    k.style.color = "#888", R.append(A, k), T.appendChild(R);
    const L = document.createElement("div");
    T.appendChild(L);
    const H = document.createElement("div"), W = an(16), F = document.createElement("b");
    H.append(W, F), H.style.display = "none", L.appendChild(H);
    const I = document.createElement("span");
    I.style.color = "#888", I.textContent = "no entity at tile", I.style.display = "none", L.appendChild(I);
    const z = document.createElement("div");
    z.style.display = "none", L.appendChild(z);
    const J = document.createElement("div"), q = an(16), et = document.createElement("span");
    J.append(q, et), J.style.display = "none", L.appendChild(J);
    const M = document.createElement("div");
    M.style.color = "#b5cea8", M.style.display = "none", L.appendChild(M);
    const O = document.createElement("div");
    O.style.display = "none", L.appendChild(O);
    const N = document.createElement("div"), D = an(16), K = document.createElement("span");
    N.append(D, K), N.style.display = "none", L.appendChild(N);
    const nt = lo(L, y), ut = document.createElement("div");
    ut.style.color = "#9cdcfe", ut.style.display = "none", L.appendChild(ut);
    const pt = document.createElement("div");
    pt.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", pt.style.display = "none";
    const ct = document.createElement("span"), _t = document.createElement("span");
    _t.style.color = "#888", pt.append(ct, _t), T.appendChild(pt);
    const Y = document.createElement("div");
    Y.style.cssText = "margin-top:6px;padding-top:4px;border-top:1px solid #333", Y.style.display = "none", T.appendChild(Y);
    const st = document.createElement("div");
    st.style.marginTop = "4px", st.style.display = "none", T.appendChild(st);
    const ot = document.createElement("div");
    ot.style.display = "none", T.appendChild(ot);
    const at = document.createElement("div");
    at.style.marginTop = "4px", ot.appendChild(at);
    function bt() {
      const $ = document.createElement("div");
      return $.style.marginLeft = "4px", $;
    }
    const zt = lo(ot, bt), At = document.createElement("div");
    At.style.cssText = "color:#555;margin-top:6px;font-size:10px", At.textContent = "click elsewhere or press Esc to unpin", T.appendChild(At), document.addEventListener("mousemove", ($) => {
      t.style.left = $.clientX + 14 + "px", t.style.top = $.clientY - 10 + "px";
    });
    let Xt = null, j = null, Z = null, Q = null, wt = null, Ct = null;
    const oe = /* @__PURE__ */ new Set();
    function $t() {
      const $ = Ct ? {
        x: Ct.x,
        y: Ct.y
      } : null;
      for (const B of oe) B($);
    }
    function Ut($, B) {
      hs(B.headerIcon, $.name), B.headerName.textContent = pe($.name), B.header.style.display = "", $.direction && $.name !== "pipe" ? (B.dirRow.textContent = `${Hw[$.direction] ?? ""} ${$.direction}`, B.dirRow.style.display = "") : B.dirRow.style.display = "none", $.carries ? (hs(B.carriesIcon, $.carries), B.carriesName.textContent = " " + pe($.carries), B.carriesRow.style.display = "") : B.carriesRow.style.display = "none", $.rate != null ? (B.rateRow.textContent = `${$.rate.toFixed(1)}/s`, B.rateRow.style.display = "") : B.rateRow.style.display = "none", $.io_type ? (B.ioRow.textContent = `io: ${$.io_type}`, B.ioRow.style.display = "") : B.ioRow.style.display = "none";
      let tt = 0;
      if ($.recipe) {
        hs(B.recipeIcon, $.recipe), B.recipeName.textContent = " " + pe($.recipe), B.recipeRow.style.display = "";
        const U = s0($.recipe);
        if (U) {
          const it = [
            ...U.inputs.map((mt) => ({
              arrow: "\u25B6",
              item: mt.item,
              rate: mt.rate
            })),
            ...U.outputs.map((mt) => ({
              arrow: "\u25C0",
              item: mt.item,
              rate: mt.rate
            }))
          ];
          for (const mt of it) {
            const Ot = B.flowPool.get(tt++), [Bt, jt, me] = Ot.children;
            Bt.textContent = `${mt.arrow} `, hs(jt, mt.item), me.textContent = `${pe(mt.item)} ${mt.rate.toFixed(1)}/s`;
          }
        }
      } else B.recipeRow.style.display = "none";
      return B.flowPool.trim(tt), $.segment_id ? (B.segmentRow.textContent = $.segment_id, B.segmentRow.style.display = "") : B.segmentRow.style.display = "none", tt;
    }
    function Ce($) {
      if ($.ghosts.length === 0) return _.style.display = "none", false;
      if (_.style.display = "", $.ghosts.length === 1) {
        const B = $.ghosts[0], tt = B.direction ? Hc[B.direction] : "";
        _.textContent = "";
        const U = document.createElement("span");
        U.style.color = "#8af", U.textContent = "ghost ";
        const it = an(12);
        hs(it, B.item);
        const mt = document.createTextNode(`${B.item} ${tt}`);
        _.append(U, it, mt);
      } else {
        _.textContent = "";
        const B = document.createElement("span");
        B.style.color = "#8af", B.textContent = `${$.ghosts.length} ghosts crossing`, _.appendChild(B);
      }
      return true;
    }
    function xe($) {
      if (!$.axis) return v.style.display = "none", false;
      const { vert: B, horiz: tt } = $.axis;
      if (B === 0 && tt === 0) return v.style.display = "none", false;
      const U = B >= 2 || tt >= 2, it = B >= 1 && tt >= 1, mt = U ? "#ff6060" : it ? "#60b0ff" : "#888";
      return v.style.display = "", v.style.color = mt, v.textContent = `axis V${B} H${tt}`, true;
    }
    function Et($) {
      if (!$.junction) return b.style.display = "none", false;
      const B = $.junction, tt = B.outcome === "Solved" ? "#80d080" : B.outcome === "Capped" ? "#e0b060" : "#c06060";
      return b.style.display = "", b.style.color = tt, b.textContent = `junction seed (${B.seedX},${B.seedY}) \xB7 ${B.outcome}`, true;
    }
    function St($) {
      if ($.cappedSides.length === 0) return C.style.display = "none", false;
      const B = $.cappedSides.map((tt) => {
        const U = tt.required - tt.shortfall;
        return `\u26A0 ${tt.sideIsOutput ? "out" : "in"}: ${tt.placedCount}\xD7${tt.placedEntity} moves ${U.toFixed(2)}/s of ${tt.required.toFixed(2)}/s \xB7 ${tt.limit}`;
      });
      return C.style.display = "", C.style.color = "#ffa060", C.textContent = B.join(" | "), true;
    }
    function qt($) {
      if ($.ghosts.length === 0) {
        ot.style.display = "none";
        return;
      }
      ot.style.display = "", $.ghosts.length >= 2 ? (at.style.color = "#ffa060", at.textContent = `\u26A0 ${$.ghosts.length} ghost specs at this tile`) : (at.style.color = "#8af", at.textContent = "ghost");
      let B = 0;
      for (const tt of $.ghosts) {
        const U = tt.direction ? Hc[tt.direction] : "\xB7", it = zt.get(B++);
        it.textContent = "";
        const mt = document.createTextNode(`${U} `), Ot = an(14);
        hs(Ot, tt.item);
        const Bt = document.createTextNode(tt.item);
        if (it.append(mt, Ot, Bt), tt.isStart) {
          const jt = document.createElement("span");
          jt.style.color = "#80d080", jt.textContent = " start", it.appendChild(jt);
        } else if (tt.isEnd) {
          const jt = document.createElement("span");
          jt.style.color = "#d08080", jt.textContent = " end", it.appendChild(jt);
        }
      }
      zt.trim(B);
    }
    function E() {
      if (j !== null) {
        e.innerHTML = j, e.style.display = "", i.style.display = "none", _.style.display = "none", v.style.display = "none", b.style.display = "none", Q ? (s.textContent = `(${Q.x}, ${Q.y})`, s.style.display = "", s.style.display = "block") : s.style.display = "none", t.style.display = "block";
        return;
      }
      e.style.display = "none", e.innerHTML = "", i.style.display = "";
      let $ = false;
      if (Z ? ($ = true, Ut(Z, {
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
        flowPool: w,
        segmentRow: x
      })) : (r.style.display = "none", l.style.display = "none", c.style.display = "none", u.style.display = "none", p.style.display = "none", f.style.display = "none", w.trim(0), x.style.display = "none"), Q) {
        const B = wt == null ? void 0 : wt.lookup(Q.x, Q.y);
        B ? (Ce(B) && ($ = true), xe(B) && ($ = true), Et(B) && ($ = true), St(B) && ($ = true)) : (_.style.display = "none", v.style.display = "none", b.style.display = "none", C.style.display = "none"), s.textContent = `(${Q.x}, ${Q.y})`, s.style.display = "block", $ = true;
      } else s.style.display = "none", _.style.display = "none", v.style.display = "none", b.style.display = "none";
      if (!$) {
        t.style.display = "none", Xt && Xt.clearHighlight();
        return;
      }
      t.style.display = "block", Z && Xt ? Xt.highlightBeltNetwork(Z) : Xt && Xt.clearHighlight();
    }
    function P() {
      if (!Ct) {
        T.style.display = "none";
        return;
      }
      const { entity: $, x: B, y: tt } = Ct, U = wt == null ? void 0 : wt.lookup(B, tt);
      if (k.textContent = `(${B}, ${tt})`, $ ? (I.style.display = "none", Ut($, {
        header: H,
        headerIcon: W,
        headerName: F,
        dirRow: z,
        carriesRow: J,
        carriesIcon: q,
        carriesName: et,
        rateRow: M,
        ioRow: O,
        recipeRow: N,
        recipeIcon: D,
        recipeName: K,
        flowPool: nt,
        segmentRow: ut
      })) : (H.style.display = "none", z.style.display = "none", J.style.display = "none", M.style.display = "none", O.style.display = "none", N.style.display = "none", nt.trim(0), ut.style.display = "none", I.style.display = ""), U) {
        if (U.junction) {
          const it = U.junction.outcome === "Solved" ? "#80d080" : U.junction.outcome === "Capped" ? "#e0b060" : "#c06060";
          ct.style.color = it, ct.textContent = `junction seed (${U.junction.seedX},${U.junction.seedY})`, _t.textContent = ` \xB7 ${U.junction.outcome}`, pt.style.display = "";
        } else pt.style.display = "none";
        if (U.axis) {
          const { vert: it, horiz: mt } = U.axis;
          if (it > 0 || mt > 0) {
            const Ot = it >= 2 || mt >= 2, Bt = it >= 1 && mt >= 1, jt = Ot ? " same-axis conflict" : Bt ? " perpendicular crossing" : "", me = Ot ? "#ff6060" : Bt ? "#60b0ff" : "#bbb";
            st.style.color = me, st.textContent = `axis: V=${it} H=${mt}${jt}`, st.style.display = "";
          } else st.style.display = "none";
        } else st.style.display = "none";
        if (qt(U), U.cappedSides.length > 0) {
          const it = {
            "tier-cap": "a faster inserter tier at the same slot count would cover this \u2014 max inserter tier is the binding constraint",
            "column-contest": "this side lost the shared inserter column to the other belt; that one column would have covered it",
            geometry: "the row shape offers no further usable slot (belt span / fixed tiles) \u2014 a template geometry limit"
          };
          Y.replaceChildren();
          const mt = document.createElement("div");
          mt.style.color = "#ffa060", mt.textContent = `\u26A0 ${U.cappedSides.length} under-provisioned inserter side${U.cappedSides.length > 1 ? "s" : ""}`, Y.appendChild(mt);
          for (const Ot of U.cappedSides) {
            const Bt = (Ot.required - Ot.shortfall).toFixed(2), jt = document.createElement("div");
            jt.style.cssText = "margin-top:2px;color:#ccc", jt.textContent = `${Ot.sideIsOutput ? "output" : "input"}: ${Ot.placedCount}\xD7${Ot.placedEntity} moves ${Bt}/s of ${Ot.required.toFixed(2)}/s needed (short ${Ot.shortfall.toFixed(2)}/s) \u2014 ${Ot.limit}: ${it[Ot.limit] ?? Ot.limit}`, Y.appendChild(jt);
          }
          Y.style.display = "";
        } else Y.style.display = "none";
      } else pt.style.display = "none", st.style.display = "none", ot.style.display = "none", Y.style.display = "none";
      T.style.display = "block";
    }
    function X() {
      E(), P();
    }
    function V($, B, tt) {
      Z = $, B !== void 0 && tt !== void 0 ? Q = {
        x: B,
        y: tt
      } : $ && (Q = {
        x: $.x ?? 0,
        y: $.y ?? 0
      }), X();
    }
    return document.addEventListener("keydown", ($) => {
      $.key === "Escape" && Ct && (Ct = null, $t(), X());
    }), {
      onHover: V,
      setHighlightController($) {
        Xt = $;
      },
      setTooltipOverride($) {
        j = $, X();
      },
      setCursorTile($, B) {
        $ === null || B === void 0 ? Q = null : Q = {
          x: $,
          y: B
        }, X();
      },
      setTileContext($) {
        wt = $, X();
      },
      pinTile($, B, tt) {
        Ct = {
          entity: $,
          x: B,
          y: tt
        }, $t(), X();
      },
      clearPin() {
        Ct = null, $t(), X();
      },
      getPinnedTile() {
        return Ct ? {
          x: Ct.x,
          y: Ct.y
        } : null;
      },
      onPinChange($) {
        return oe.add($), () => oe.delete($);
      }
    };
  }
  function jw(n) {
    const t = n.indexOf(":");
    return t >= 0 ? n.slice(0, t) : n;
  }
  function Uc(n, t, e, s) {
    return e > n ? "E" : e < n ? "W" : s > t ? "S" : s < t ? "N" : null;
  }
  const Vw = {
    ghosts: [],
    axis: null,
    junction: null,
    cappedSides: []
  };
  function Yw(n) {
    var _a2;
    if (!n || n.length === 0) return {
      lookup: () => Vw
    };
    const t = /* @__PURE__ */ new Map(), e = /* @__PURE__ */ new Map(), s = /* @__PURE__ */ new Map(), i = /* @__PURE__ */ new Map();
    for (const r of n) if (r.phase === "GhostSpecRouted") {
      const { spec_key: o, tiles: a } = r.data, l = jw(o);
      if (!a || a.length === 0) continue;
      for (let c = 0; c < a.length; c++) {
        const [h, d] = a[c];
        let u = null;
        c < a.length - 1 ? u = Uc(h, d, a[c + 1][0], a[c + 1][1]) : c > 0 && (u = Uc(a[c - 1][0], a[c - 1][1], h, d));
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
  function Xw(n) {
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
      a && (t = M_(a, i, o), a.querySelectorAll("input,select,button").forEach((l) => {
        l.closest("[data-snapshot-keep]") || (l.disabled = true);
      }));
    }
    return {
      load: s,
      clear: e
    };
  }
  const qw = 2200, jc = 180, co = 200, Kw = 8, Jw = [
    "rows_placed",
    "lanes_planned",
    "bus_routed",
    "poles_placed"
  ];
  function lr(n) {
    return `${n.x ?? 0},${n.y ?? 0},${n.name},${n.recipe ?? ""}`;
  }
  function Zw(n) {
    const t = /* @__PURE__ */ new Map(), e = n.trace;
    if (!Array.isArray(e)) return t;
    for (const s of e) {
      const i = s;
      i.phase === "PhaseSnapshot" && i.data && t.set(i.data.phase, i.data);
    }
    return t;
  }
  function Qw(n) {
    const t = Zw(n), e = [], s = /* @__PURE__ */ new Set();
    for (const r of Jw) {
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
  function tv(n, t, e, s, i) {
    const r = /* @__PURE__ */ new Map(), o = ei(n, t, e, s, (b, C) => {
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
    const c = Qw(n), h = c.reduce((b, C) => b + (C.entities.length > 0 ? 1 : 0), 0);
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
    const u = Math.max(0, qw - jc * h) / h, p = [], f = /* @__PURE__ */ new Map();
    let g = 0;
    for (const b of c) {
      if (b.entities.length === 0) continue;
      f.set(b.phase, g);
      const C = Math.min(Kw, u / b.entities.length);
      b.entities.forEach((R, A) => {
        const k = r.get(lr(R));
        !k || k.length === 0 || p.push({
          graphics: k,
          revealStartMs: g + A * C
        });
      });
      const T = (b.entities.length - 1) * C;
      g += T + co + jc;
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
    const m = performance.now();
    let y = 0, w = false, x = p.length === 0;
    const _ = () => {
      if (w || x) return;
      const b = performance.now() - m;
      for (let C = y; C < p.length; C++) {
        const T = p[C];
        if (T.revealStartMs > b) break;
        const R = Math.min(1, (b - T.revealStartMs) / co);
        for (const A of T.graphics) A.alpha = R;
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
          w || x || (w = true, i.ticker.remove(_), qn(), Ye());
        },
        finish() {
          if (!(w || x)) {
            for (const b of p) for (const C of b.graphics) C.alpha = 1;
            x = true, i.ticker.remove(_), qn(), Ye();
          }
        },
        isDone() {
          return x || w;
        }
      }
    };
  }
  const ev = 4243680;
  function nv(n, t, e, s = 240) {
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
        color: ev,
        alpha: 0.55 * d
      });
    };
    n.ticker.add(h), wr();
  }
  const Hi = 150, Vc = 80, sv = 4, iv = 300, Tn = 4, En = 900, rv = 6, ov = 250, Hs = 800, Ui = 250;
  function qe(n, t, e) {
    return n <= 1 ? t : Math.min(t, e / n);
  }
  function Yc(n, t) {
    return `${n},${t}`;
  }
  function av(n) {
    return n.split(":")[1] ?? "";
  }
  function lv(n, t) {
    return n > 0 ? "East" : n < 0 ? "West" : t > 0 ? "South" : "North";
  }
  function cv(n, t, e, s) {
    const i = ru();
    i.attachTo(n);
    const r = new ft();
    n.addChild(r);
    const o = /* @__PURE__ */ new Map(), a = /* @__PURE__ */ new Set(), l = /* @__PURE__ */ new Map(), c = _r();
    let h = false, d = false, u = false, p = 0;
    const f = () => u ? p : performance.now();
    let g = null, m = 0;
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
      g === null && (g = M);
      const N = M + (O ? Hi : Vc);
      N > m && (m = N);
    }
    function R(M, O) {
      if (M.length === 0) return;
      const N = f();
      for (const K of M) ir(K, c);
      const D = [
        ...M
      ].sort((K, nt) => {
        const ut = (K.y ?? 0) - (nt.y ?? 0);
        return ut !== 0 ? ut : (K.x ?? 0) - (nt.x ?? 0);
      });
      D.forEach((K, nt) => {
        const ut = N + nt * O, pt = us(K);
        if (a.has(pt)) return;
        a.add(pt), Uo(i, K, ut, c), l.has(pt) || l.set(pt, ut), bc(i, K.x ?? 0, K.y ?? 0);
        const ct = o.get(Yc(K.x ?? 0, K.y ?? 0));
        ct && ct.fadeOutStartMs === null && (ct.fadeOutStartMs = ut, T(ut, false));
      }), T(N + (D.length - 1) * O, true);
    }
    function A(M, O, N, D, K, nt) {
      const ut = Yc(M, O), pt = o.get(ut);
      pt && pt.specKey === K || (H0(i, M, O, N, D, nt, K), pt || o.set(ut, {
        specKey: K,
        fadeStartMs: nt,
        fadeOutStartMs: null
      }), T(nt, true));
    }
    function k(M) {
      const O = qe(M.length, Tn, En);
      b("rows_placed", {
        count: M.length,
        stagger_ms: O,
        span_ms: M.length * O
      }), R(M, O);
    }
    function L(M) {
      const O = f(), N = av(M.spec_key), D = M.tiles;
      if (D.length === 0) return;
      const K = qe(D.length, sv, iv);
      b("ghost_routed", {
        spec_key: M.spec_key,
        item: N,
        tile_count: D.length,
        span_ms: D.length * K
      });
      for (let nt = 0; nt < D.length; nt++) {
        const [ut, pt] = D[nt];
        let ct = 0, _t = 0;
        nt < D.length - 1 ? (ct = D[nt + 1][0] - ut, _t = D[nt + 1][1] - pt) : nt > 0 && (ct = ut - D[nt - 1][0], _t = pt - D[nt - 1][1]), A(ut, pt, lv(ct, _t), N, M.spec_key, O + nt * K);
      }
    }
    function H(M) {
      const O = M.entities.length, N = qe(O, Tn, En);
      b("committed", {
        source: "spec",
        count: O,
        span_ms: O * N
      }), R(M.entities, N);
    }
    function W(M) {
      const O = f(), N = M.zone_x, D = M.zone_y, K = M.zone_x + M.zone_w - 1, nt = M.zone_y + M.zone_h - 1;
      for (const [ct, _t] of o.entries()) {
        const [Y, st] = ct.split(",").map(Number);
        Y < N || Y > K || st < D || st > nt || _t.fadeOutStartMs === null && (_t.fadeOutStartMs = O, T(O, false), bc(i, Y, st));
      }
      for (const ct of _) ct.clusterId === M.cluster_id && (ct.cleared = true);
      for (let ct = M.zone_y; ct <= nt; ct++) for (let _t = M.zone_x; _t <= K; _t++) {
        const Y = j0(i, _t, ct);
        for (const st of Y) a.delete(st);
      }
      const ut = M.entities.length, pt = qe(ut, rv, ov);
      b("junction", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        count: ut,
        span_ms: ut * pt
      }), R(M.entities, pt);
    }
    function F(M) {
      if (d = true, M.phase === "rows_placed") {
        k(M.entities);
        return;
      }
      if (M.phase !== "lanes_planned") {
        if (M.phase === "bus_routed") {
          const O = M.entities.filter((N) => !a.has(us(N)));
          if (O.length > 0) {
            const N = qe(O.length, Tn, En);
            R(O, N);
          }
          return;
        }
        if (M.phase === "poles_placed") {
          const O = M.entities.filter((N) => !a.has(us(N)));
          if (O.length > 0) {
            const N = qe(O.length, Tn, En);
            R(O, N);
          }
          return;
        }
      }
    }
    function I(M) {
      const O = f();
      b("cluster_outline", {
        cluster_id: M.cluster_id,
        zone: `${M.zone_x},${M.zone_y}+${M.zone_w}x${M.zone_h}`,
        lifetime_ms: Hs,
        fade_ms: Ui
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
      au(i, M);
      for (const [O, N] of o.entries()) if (N.fadeOutStartMs !== null && M >= N.fadeOutStartMs) {
        const [D, K] = O.split(",").map(Number);
        (M - N.fadeOutStartMs) / Vc >= 1 && o.delete(O);
      }
      for (const O of i.ghostContainer.particleChildren) O.alpha < 0.5 && (O.alpha = Math.min(0.5, O.alpha + 16 / Hi));
      r.clear();
      for (let O = _.length - 1; O >= 0; O--) {
        const N = _[O], D = M - N.startMs;
        if (!(D < 0)) if (N.cleared || D >= Hs) {
          const K = N.cleared ? Math.max(D, Hs - Ui) : D;
          if (K >= Hs) {
            _.splice(O, 1);
            continue;
          }
          const nt = Math.max(0, 1 - (K - (Hs - Ui)) / Ui);
          r.rect(N.x * S, N.y * S, N.w * S, N.h * S), r.stroke({
            width: 2,
            color: 4508927,
            alpha: 0.9 * nt
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
    function q(M, O) {
      if (h || u) return;
      w = O ?? null, v && console.log(`[stream t=${f().toFixed(0)}] ${M.phase}`, "data" in M ? M.data : void 0);
      const N = f();
      switch (g === null && (g = N), M.phase) {
        case "PhaseSnapshot": {
          const D = M.data;
          D.phase === "rows_placed" && C("machines", N), D.phase === "poles_placed" && C("poles", N), F(D);
          break;
        }
        case "GhostSpecRouted":
          C("ghost_routes", N), L(M.data);
          break;
        case "GhostSpecCommitted":
          C("committed_routes", N), H(M.data);
          break;
        case "JunctionCommitted":
          C("junctions", N), W(M.data);
          break;
        case "GhostClusterSolved":
          I(M.data);
          break;
        case "TrunkBeltCommitted": {
          const D = M.data, K = D.entities.length, nt = qe(K, Tn, En);
          b("committed", {
            source: "trunk",
            count: K,
            span_ms: K * nt
          }), R(D.entities, nt);
          break;
        }
        case "BalancerCommitted": {
          const D = M.data, K = D.entities.length, nt = qe(K, Tn, En);
          b("committed", {
            source: "balancer",
            count: K,
            span_ms: K * nt
          }), R(D.entities, nt);
          break;
        }
        case "OutputMergerCommitted": {
          const D = M.data, K = D.entities.length, nt = qe(K, Tn, En);
          b("committed", {
            source: "merger",
            count: K,
            span_ms: K * nt
          }), R(D.entities, nt);
          break;
        }
        case "PolesCommitted": {
          const D = M.data, K = D.entities.length, nt = qe(K, Tn, En);
          b("committed", {
            source: "poles",
            count: K,
            span_ms: K * nt
          }), R(D.entities, nt);
          break;
        }
        default: {
          M.phase === "LayoutRetried" && (i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), Ye());
          break;
        }
      }
      w = null;
    }
    return {
      onEvent: q,
      hasCommittedEntities: () => d,
      cancel() {
        h || (h = true, t.ticker.remove(z), J && (qn(), J = false), i.clear(), o.clear(), a.clear(), _.length = 0, r.clear(), x = null, Ye());
      },
      finish(M) {
        t.ticker.remove(z), J && (qn(), J = false), U0(i), b("streaming_finish", {
          entity_count: i.count(),
          latest_fade_end_ms: m
        });
        const O = g ?? 0, N = [];
        for (const { particle: nt, iconParticle: ut, badgeParticle: pt, revealAt: ct } of _c()) N.push({
          kind: "particle",
          particle: nt,
          iconParticle: ut,
          badgeParticle: pt,
          revealAt: ct
        });
        const D = M.entities.filter((nt) => !a.has(us(nt)));
        if (D.length > 0) {
          for (const ut of D) ir(ut, c);
          for (const ut of D) Uo(i, ut, O, c), a.add(us(ut));
          const nt = new Set(N.map((ut) => ut.particle));
          for (const { particle: ut, iconParticle: pt, badgeParticle: ct, revealAt: _t } of _c()) nt.has(ut) || N.push({
            kind: "particle",
            particle: ut,
            iconParticle: pt,
            badgeParticle: ct,
            revealAt: _t
          });
        }
        const K = V0(i, c);
        if (K.size > 0) for (const nt of N) {
          const ut = K.get(nt.particle);
          ut && (nt.particle = ut);
        }
        return x = N, u = true, p = m, et(p), q0(M, t);
      },
      seekTo(M) {
        if (h || x === null) return;
        const O = g ?? 0, N = Math.max(m, O);
        p = Math.min(N, Math.max(O, M)), b("scrub", {
          virtualMs: p
        }), et(p);
      },
      getTimeRange() {
        return {
          firstMs: g ?? 0,
          lastMs: m
        };
      },
      getMilestones() {
        return Array.from(y.values()).sort((M, O) => M.virtualMs - O.virtualMs);
      }
    };
    function et(M) {
      if (x !== null) {
        for (const O of x) {
          const N = M - O.revealAt, D = N <= 0 ? 0 : N >= Hi ? 1 : N / Hi;
          O.particle.alpha = D, O.iconParticle && (O.iconParticle.alpha = D), O.badgeParticle && (O.badgeParticle.alpha = D);
        }
        Ye();
      }
    }
  }
  const hv = 0.03, dv = 200, ho = [
    "machines",
    "ghost_routes",
    "committed_routes",
    "junctions",
    "poles",
    "optimizing"
  ], uv = {
    machines: "Machines",
    ghost_routes: "Belt routes",
    committed_routes: "Belts placed",
    junctions: "Crossings",
    poles: "Power poles",
    optimizing: "Optimizing"
  };
  function pv(n, t) {
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
    for (const W of ho) {
      const F = document.createElement("div");
      F.className = "ts-chip", F.dataset.milestone = W, F.textContent = uv[W], W === "optimizing" && (F.style.display = "none"), s.appendChild(F), l.set(W, F);
    }
    let c = false, h = null, d = [];
    const u = /* @__PURE__ */ new Set();
    let p = null;
    function f(W) {
      o.style.width = `${W * 100}%`, a.style.left = `${W * 100}%`;
    }
    function g(W) {
      var _a2, _b2;
      p !== W && (p && ((_a2 = l.get(p)) == null ? void 0 : _a2.classList.remove("ts-chip--active")), W && ((_b2 = l.get(W)) == null ? void 0 : _b2.classList.add("ts-chip--active")), p = W);
    }
    function m(W, F) {
      var _a2;
      if (c) return;
      const I = W.id;
      u.add(I), (_a2 = l.get(I)) == null ? void 0 : _a2.classList.add("ts-chip--reached"), g(I);
      const z = Math.max(1, F.lastMs - F.firstMs), J = (W.virtualMs - F.firstMs) / z;
      f(Math.min(1, Math.max(0, J))), e.classList.add("ts-visible");
    }
    function y(W) {
      return h ? h.firstMs + W * (h.lastMs - h.firstMs) : 0;
    }
    function w(W) {
      for (const F of d) if (Math.abs(W - F.frac) < hv) return {
        frac: F.frac,
        snapped: true
      };
      return {
        frac: W,
        snapped: false
      };
    }
    function x(W) {
      if (!h) return;
      const F = i.getBoundingClientRect(), I = (W - F.left) / F.width, z = Math.min(1, Math.max(0, I)), { frac: J, snapped: q } = w(z);
      f(J), q ? a.classList.add("ts-thumb--snapped") : a.classList.remove("ts-thumb--snapped"), t(y(J));
    }
    let _ = null, v = null;
    function b(W) {
      if (!c || !h) return;
      W.preventDefault();
      try {
        i.setPointerCapture(W.pointerId);
      } catch {
      }
      const F = (z) => x(z.clientX), I = (z) => {
        _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), _ = null, v = null, a.classList.remove("ts-thumb--snapped");
      };
      _ = F, v = I, document.addEventListener("pointermove", F), document.addEventListener("pointerup", I, {
        once: true
      }), x(W.clientX);
    }
    i.addEventListener("pointerdown", b);
    function C(W, F) {
      const I = W.lastMs - W.firstMs;
      if (I < dv || F.length === 0) {
        k();
        return;
      }
      c = true, h = W, e.classList.add("ts-scrub-mode"), e.classList.add("ts-visible"), d = F.map((z) => ({
        id: z.id,
        frac: (z.virtualMs - W.firstMs) / I
      })), s.style.justifyContent = "flex-start", s.style.position = "relative";
      for (const z of l.values()) z.style.position = "absolute", z.style.transform = "translateX(-50%)";
      for (const z of ho) {
        const J = l.get(z);
        if (!J) continue;
        const q = d.find((et) => et.id === z);
        q ? (J.style.left = `${q.frac * 100}%`, J.style.display = "", J.classList.add("ts-chip--reached")) : J.style.display = "none";
      }
      R(), requestAnimationFrame(A), f(1), g(null);
    }
    let T = null;
    function R() {
      if (T && T.remove(), !h) return;
      const W = document.createElement("div");
      W.className = "ts-ticks";
      for (const F of d) {
        const I = document.createElement("div");
        I.className = "ts-tick", I.style.left = `${F.frac * 100}%`, W.appendChild(I);
      }
      i.appendChild(W), T = W;
    }
    function A() {
      if (!c) return;
      const W = 6, F = s.clientWidth;
      if (F <= 0) return;
      const I = ho.map((J) => {
        var _a2;
        const q = l.get(J);
        if (!q || q.style.display === "none") return null;
        const et = J === "optimizing" ? 1 : ((_a2 = d.find((M) => M.id === J)) == null ? void 0 : _a2.frac) ?? 0;
        return {
          el: q,
          originalFrac: et
        };
      }).filter((J) => J !== null);
      let z = -1 / 0;
      for (const { el: J, originalFrac: q } of I) {
        const et = J.offsetWidth / 2, M = q * F, O = z + et + W, N = Math.max(M, O);
        J.style.left = `${N / F * 100}%`, z = N + et;
      }
    }
    function k() {
      c = false, h = null, d = [], u.clear(), p = null, T && (T.remove(), T = null), e.classList.remove("ts-visible", "ts-scrub-mode"), o.style.width = "0", a.style.left = "0", a.classList.remove("ts-thumb--snapped"), s.style.justifyContent = "space-between", s.style.position = "";
      for (const [W, F] of l) F.style.position = "", F.style.transform = "", F.style.left = "", F.style.display = W === "optimizing" ? "none" : "", F.classList.remove("ts-chip--reached", "ts-chip--active", "ts-chip--in-progress");
    }
    function L(W) {
      const F = l.get("optimizing");
      if (F) {
        if (F.classList.remove("ts-chip--in-progress", "ts-chip--reached"), W === "idle") {
          F.style.display = "none";
          return;
        }
        F.style.display = "", e.classList.add("ts-visible"), c && (F.style.position = "absolute", F.style.transform = "translateX(-50%)", F.style.left = "100%", requestAnimationFrame(A)), W === "active" ? F.classList.add("ts-chip--in-progress") : W === "done" && F.classList.add("ts-chip--reached");
      }
    }
    function H() {
      _ && document.removeEventListener("pointermove", _), v && document.removeEventListener("pointerup", v), i.removeEventListener("pointerdown", b), e.remove();
    }
    return {
      noteMilestone: m,
      arm: C,
      markOptimizeState: L,
      reset: k,
      destroy: H
    };
  }
  const fv = `
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
  function mv() {
    if (document.getElementById("spaghettio-busy-style")) return;
    const n = document.createElement("style");
    n.id = "spaghettio-busy-style", n.textContent = fv, document.head.appendChild(n);
  }
  const gv = 120;
  function yv(n) {
    mv();
    const t = document.createElement("div");
    t.className = "spaghettio-busy";
    const e = document.createElement("span");
    e.className = "spaghettio-busy-spin", t.appendChild(e);
    const s = document.createElement("span");
    s.textContent = "computing\u2026", t.appendChild(s), n.appendChild(t);
    let i = null;
    Gb((r) => {
      r > 0 ? i === null && !t.classList.contains("visible") && (i = setTimeout(() => {
        t.classList.add("visible"), i = null;
      }, gv)) : (i !== null && (clearTimeout(i), i = null), t.classList.remove("visible"));
    });
  }
  function Ke(n, t) {
    return n.filter((e) => e.phase === t);
  }
  const Gn = "color:#9cdcfe;font-weight:bold", he = "color:#888", _e = "color:#e0e0e0", ds = "color:#6a6", ji = "color:#ffaa00", uo = "color:#f66", Vi = "color:#c586c0";
  function xv(n) {
    var _a2;
    const t = Array.isArray(n.trace) ? n.trace : [];
    if (t.length === 0) return;
    const e = Ke(t, "PhaseTime"), s = Ke(t, "SatInvocation"), i = Ke(t, "JunctionSolved"), r = Ke(t, "JunctionGrowthCapped"), o = Ke(t, "GhostClusterSolved"), a = Ke(t, "GhostRoutingComplete"), l = Ke(t, "RegionWalkerVeto"), c = Ke(t, "JunctionGrowthIteration"), h = Ke(t, "NegotiateComplete"), d = Ke(t, "ValidationCompleted"), u = t.filter((x) => x.phase === "LayoutRetried"), p = e.reduce((x, _) => x + _.data.duration_ms, 0), f = s.reduce((x, _) => x + _.data.solve_time_us, 0), g = s.filter((x) => x.data.satisfied).length, m = ((_a2 = n.entities) == null ? void 0 : _a2.length) ?? 0, y = u.length > 0 ? r.length > 0 ? ` \xB7 ${r.length} capped \xB7 retried (${u[0].data.caps_before} caps recovered)` : ` \xB7 retried (${u[0].data.caps_before} caps recovered)` : r.length > 0 ? ` \xB7 ${r.length} capped` : "", w = r.length > 0 ? ji : u.length > 0 ? Vi : ds;
    if (console.log(`%c\u25B6 layout %c${n.width}\xD7${n.height}  %c${m} entities  %c${p}ms  %cSAT ${Math.round(f / 1e3)}ms (${s.length}\xD7)%c${y}`, Gn, _e, he, _e, Vi, w), console.groupCollapsed("%c  \u21B3 breakdown", he), e.length > 0) {
      const x = [
        ...e
      ].sort((_, v) => v.data.duration_ms - _.data.duration_ms);
      console.log(`%cphases%c ${p}ms total`, Gn, he);
      for (const _ of x) {
        const v = _.data, b = p > 0 ? v.duration_ms / p * 100 : 0, C = Math.max(1, Math.round(b / 100 * 24)), T = "\u2588".repeat(C);
        console.log(`  %c${v.phase.padEnd(18)}%c ${String(v.duration_ms).padStart(5)}ms  %c${T}%c ${b.toFixed(1)}%`, he, _e, Vi, he);
      }
    }
    if (s.length > 0) {
      const x = f / 1e3, _ = p > 0 ? x / p * 100 : 0, v = f / s.length, b = [
        ...s
      ].sort((T, R) => R.data.solve_time_us - T.data.solve_time_us)[0], C = [
        ...s
      ].sort((T, R) => R.data.zone_w * R.data.zone_h - T.data.zone_w * T.data.zone_h)[0];
      console.log(`%cSAT%c ${s.length} invocations \xB7 ${x.toFixed(1)}ms (%c${_.toFixed(1)}%%%c of total)`, Gn, _e, Vi, _e), console.log(`  %csatisfied%c ${g}  %cunsat%c ${s.length - g}  %cavg%c ${(v / 1e3).toFixed(2)}ms`, he, ds, he, uo, he, _e), b && console.log(`  %cslowest call%c ${(b.data.solve_time_us / 1e3).toFixed(1)}ms \u2014 %c${b.data.zone_w}\xD7${b.data.zone_h} @ (${b.data.zone_x},${b.data.zone_y}), ${b.data.variables} vars, ${b.data.clauses} clauses`, he, _e, he), C && C !== b && console.log(`  %cbiggest zone%c ${C.data.zone_w}\xD7${C.data.zone_h} @ (${C.data.zone_x},${C.data.zone_y}) \u2014 ${C.data.variables} vars`, he, _e);
    }
    if (o.length > 0 || i.length > 0 || r.length > 0) {
      if (console.log("%cjunctions", Gn), console.log(`  %cclusters%c ${o.length}  %csolved%c ${i.length}  %ccapped%c ${r.length}  %cvetoes%c ${l.length}`, he, _e, he, ds, he, r.length > 0 ? ji : _e, he, _e), c.length > 0) {
        const x = /* @__PURE__ */ new Map();
        for (const v of c) {
          const b = `${v.data.seed_x},${v.data.seed_y}`;
          x.set(b, Math.max(x.get(b) ?? 0, v.data.iter));
        }
        const _ = [
          ...x.entries()
        ].sort((v, b) => b[1] - v[1])[0];
        _ && _[1] > 0 && console.log(`  %chardest%c junction at (${_[0]}) needed ${_[1] + 1} growth iters`, he, _e);
      }
      if (r.length > 0) for (const x of r) console.log(`    %c\u26A0 capped at (${x.data.tile_x},${x.data.tile_y})%c \u2014 ${x.data.reason}, ${x.data.region_tiles} tiles after ${x.data.iters} iters`, ji, he);
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
      const x = d[0].data, _ = x.error_count > 0 ? uo : ds, v = x.warning_count > 0 ? ji : ds;
      console.log(`%cvalidation  %c${x.error_count} errors  %c${x.warning_count} warnings`, Gn, _, v);
    }
    console.groupEnd();
  }
  const bv = [
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
  async function _v() {
    await mu();
    const n = t_();
    await l0(bv), await p0();
    const t = document.getElementById("app"), e = window.location.hash, s = new URLSearchParams(window.location.search);
    if (e.startsWith("#/balancers")) {
      const { renderBalancerShowcase: r } = await Pn(async () => {
        const { renderBalancerShowcase: o } = await import("./balancers-D7dqXXzN.js");
        return {
          renderBalancerShowcase: o
        };
      }, []);
      r(t, n);
      return;
    }
    if (!(e.startsWith("#/layout") || s.has("generator") || kb())) {
      const r = document.createElement("div");
      t.appendChild(r), a_(r, n, {
        onOpenGenerator: () => {
          r.remove(), Xc(n), window.history.replaceState({}, "", "#/layout");
        }
      });
      return;
    }
    Xc(n);
  }
  async function Xc(n) {
    const t = document.getElementById("canvas-container");
    if (!t) throw new Error("Missing #canvas-container element");
    const e = document.getElementById("app");
    e.style.display = "flex";
    const s = document.getElementById("sidebar");
    s && (s.style.display = ""), t.style.display = "";
    const { app: i, viewport: r, requestRender: o, beginAnimating: a, endAnimating: l } = await S0(t), c = T0(r);
    let h = false;
    Ds(r, null), Ww();
    const d = zw(t), { debugCb: u, colorCb: p, heatmapCb: f, powerWiresCb: g, regionsCb: m, soloRegionsCb: y, ghostTilesCb: w, traceOverlayCb: x } = d, _ = Dw(t);
    Bi(p.checked);
    const v = () => {
      globalThis.__ANIM_LOGS = u.checked;
    };
    u.addEventListener("change", v), v();
    const b = Uw(t), C = mw();
    let T = null;
    const R = Sw(t, r, {
      onChange: (G) => {
        if (C.update(G), G) {
          L.alpha = A.isActive() ? 0.2 : 0.35;
          const rt = G.iter.bbox;
          T = {
            bboxX: rt.x,
            bboxY: rt.y,
            bboxW: rt.w,
            bboxH: rt.h
          };
        } else L.alpha = 1, T = null, A.isActive() && A.exit();
        o();
      },
      onEditRequested: (G) => {
        L.alpha = 0.2, A.enter(G), o();
      }
    }), A = Fw({
      viewport: r,
      canvas: i.canvas,
      engine: n,
      jd: R,
      satZoneOverlayLayer: C.layer
    });
    k_(t, (G) => mt.load(G));
    function k(G) {
      L.removeChildren();
      const rt = ru();
      return rt.attachTo(L), Y0(G, rt, i);
    }
    const L = new Ht();
    L.isRenderGroup = true, L.eventMode = "none", r.addChild(L);
    const H = new Ht();
    H.eventMode = "none", r.addChild(H), r.addChild(C.layer);
    const W = new ft();
    W.label = "pin-highlight", r.addChild(W), b.onPinChange((G) => {
      if (W.clear(), G) {
        const rt = G.x * S, dt = G.y * S;
        W.setStrokeStyle({
          width: 2,
          color: 8440063,
          alpha: 0.95
        }), W.rect(rt - 2, dt - 2, S + 4, S + 4).stroke();
      }
      o();
    }), r.moveCenter(en / 2, en / 2);
    const F = (G) => {
    };
    let I = null;
    function z(G) {
      I = G, b.onHover(G, G == null ? void 0 : G.x, G == null ? void 0 : G.y);
    }
    let J = /* @__PURE__ */ new Map();
    function q(G) {
      const rt = /* @__PURE__ */ new Map();
      for (const dt of G.entities) {
        const yt = dt.x ?? 0, ht = dt.y ?? 0, Lt = Pe[dt.name];
        if (Lt) {
          const [Wt, Pt] = Lt;
          for (let Nt = 0; Nt < Pt; Nt++) for (let ge = 0; ge < Wt; ge++) rt.set(`${yt + ge},${ht + Nt}`, dt);
        } else if (se.has(dt.name)) {
          rt.set(`${yt},${ht}`, dt);
          const [Wt, Pt] = ya(dt.direction);
          rt.set(`${yt + Wt},${ht + Pt}`, dt);
        } else rt.set(`${yt},${ht}`, dt);
      }
      J = rt;
    }
    function et(G) {
      return {
        highlightItem: (rt) => {
          G.highlightItem(rt), o();
        },
        highlightBeltNetwork: (rt) => {
          G.highlightBeltNetwork(rt), o();
        },
        clearHighlight: () => {
          G.clearHighlight(), o();
        },
        chainKey: G.chainKey
      };
    }
    let M = false, O = null, N = null, D = false, K = null;
    const nt = {
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
      N && (L.removeChild(N), N.destroy(), N = null);
      const G = nt.getPhaseIndex();
      if (u.checked && G >= 0, D && (K == null ? void 0 : K.cancel(), K = null, D = false, V)) {
        const dt = ei(V, L, z, F);
        b.setHighlightController(et(dt)), o();
      }
      if (!u.checked || !x.checked || !((_a2 = V == null ? void 0 : V.trace) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      const rt = V.trace;
      N = P_(rt, V.width ?? 0, V.height ?? 0, L, (dt) => {
        b.setTooltipOverride(dt ? `<span style="color:#8af">TRACE</span> ${dt}` : null);
      }), o();
    }
    let pt = null, ct = null, _t = null, Y = [], st = null, ot = null, at = null;
    const bt = document.createElement("div");
    bt.className = "validation-badge", bt.style.display = "none", t.appendChild(bt);
    function zt(G) {
      if (!G || G.length === 0) {
        bt.style.display = "none";
        return;
      }
      const rt = G.filter((ht) => ht.severity === "Error").length, dt = G.length - rt;
      let yt;
      rt > 0 && dt > 0 ? yt = `\u26A0 ${rt} error${rt > 1 ? "s" : ""}, ${dt} warning${dt > 1 ? "s" : ""}` : rt > 0 ? yt = `\u26A0 ${rt} error${rt > 1 ? "s" : ""}` : yt = `\u26A0 ${dt} warning${dt > 1 ? "s" : ""}`, bt.textContent = yt, bt.classList.toggle("has-errors", rt > 0), bt.style.display = "block";
    }
    let At = null, Xt = null, j = null, Z = null, Q = null;
    const wt = 1;
    function Ct(G, rt) {
      const dt = G * S + S / 2, yt = rt * S + S / 2;
      r.scale.x < wt && r.setZoom(wt, false), r.moveCenter(dt, yt);
    }
    function oe(G) {
      var _a2, _b2;
      const rt = [];
      for (const dt of G.regions ?? []) {
        if (dt.kind !== "unresolved") continue;
        const yt = dt.x + Math.floor(dt.width / 2), ht = dt.y + Math.floor(dt.height / 2), Lt = ((_b2 = (_a2 = dt.ports) == null ? void 0 : _a2.find((Wt) => Wt.item)) == null ? void 0 : _b2.item) ?? "unknown";
        rt.push({
          severity: "Warning",
          category: `ghost-router \xB7 ${Lt}`,
          message: `unresolved crossing at (${yt}, ${ht})`,
          x: yt,
          y: ht
        });
      }
      for (const dt of G.warnings ?? []) /^ghost router:.*unresolved crossings/i.test(dt) || rt.push({
        severity: "Warning",
        category: "layout",
        message: dt,
        x: void 0,
        y: void 0
      });
      return rt;
    }
    function $t() {
      if (pt && (L.removeChild(pt), pt.destroy(), pt = null), V && !ct && at !== V) {
        const ht = V;
        at = ht, n.validateLayout(ht, $).then((Lt) => {
          V === ht && (ct = Lt, at = null, $t(), ns(ht));
        }).catch(() => {
          V === ht && (ct = [], at = null, $t(), ns(ht));
        });
      }
      const G = V ? oe(V) : [], rt = [
        ...ct ?? [],
        ...G
      ], dt = (ht) => {
        if (ht.x == null || ht.y == null || !ot) return null;
        const Lt = ot.lookup(ht.x, ht.y).cappedSides;
        return Lt.length === 0 ? null : [
          ...new Set(Lt.map((Pt) => Pt.limit))
        ].sort().join("+");
      };
      if (Rs == null ? void 0 : Rs.updateValidation(rt, Ct, dt), zt(rt), Ut(rt), !V || rt.length === 0) {
        o();
        return;
      }
      pt = $_(rt, L, (ht) => {
        b.setTooltipOverride(ht ? `<span style="color:#f44">VALIDATION</span> ${ht}` : null);
      }).layer, o();
    }
    function Ut(G) {
      if (G && (Y = G), _t && (L.removeChild(_t), _t.destroy({
        children: true
      }), _t = null), !f.checked || !V || Y.length === 0) {
        o();
        return;
      }
      _t = O_(Y, V.entities, L), o();
    }
    function Ce() {
      var _a2;
      if (st && (L.removeChild(st), st.destroy({
        children: true
      }), st = null), !g.checked || !((_a2 = V == null ? void 0 : V.power_wires) == null ? void 0 : _a2.length)) {
        o();
        return;
      }
      st = N_(V.power_wires, V.entities, L), o();
    }
    function xe() {
      if (Q && (r.removeChild(Q), Q.destroy({
        children: true
      }), Q = null), !u.checked || !w.checked || !V) {
        o();
        return;
      }
      const G = Cw(V.trace);
      if (!G) {
        o();
        return;
      }
      Q = G, r.addChildAt(Q, 0), o();
    }
    function Et() {
      var _a2;
      if (At && (L.removeChild(At), At.destroy(), At = null), j && (L.removeChild(j), j.destroy(), j = null), Xt = null, Z = null, !u.checked || !(m == null ? void 0 : m.checked) || !V) {
        o();
        return;
      }
      if (V.regions && V.regions.length > 0) {
        const G = Y_(V);
        At = G.layer, Xt = G.hitTest, L.addChild(At);
      }
      if ((_a2 = V.trace) == null ? void 0 : _a2.length) {
        const G = Z_(V.trace);
        if (G.length > 0) {
          const rt = iw(G);
          j = rt.layer, Z = rt.hitTest, L.addChild(j);
        }
      }
      o();
    }
    const St = document.createElement("div");
    St.style.cssText = "position:absolute;bottom:34px;left:8px;background:rgba(0,0,0,0.8);color:#e0e0e0;font:11px monospace;padding:6px 8px;border-radius:3px;border:1px solid #00e0a0;z-index:10;display:none;min-width:200px", t.appendChild(St);
    const qt = document.createElement("div");
    qt.style.cssText = "color:#00e0a0;margin-bottom:4px", St.appendChild(qt);
    const E = document.createElement("textarea");
    E.placeholder = "Add a note\u2026", E.rows = 2, E.style.cssText = "width:100%;box-sizing:border-box;background:#2a2a2a;color:#e0e0e0;border:1px solid #555;border-radius:2px;font:11px monospace;resize:vertical;margin-bottom:4px", St.appendChild(E);
    const P = document.createElement("div");
    P.style.cssText = "color:#777", P.textContent = "Ctrl+C to copy JSON", St.appendChild(P);
    let X = false, V = null, $ = null, B = null, tt = null, U = null;
    const it = pv(t, (G) => U == null ? void 0 : U.seekTo(G));
    yv(t);
    const mt = Xw({
      sidebarEl: document.getElementById("sidebar"),
      getSidebarCtrl: () => Rs,
      renderLayoutOnCanvas: Ge,
      setCachedValidationIssues: (G) => {
        ct = G;
      },
      updateValidationOverlay: $t,
      panToTile: Ct,
      onDebugEnable: () => d.setDebugEnabled(true),
      onClear: () => {
        mt.clear(), L.removeChildren(), H.removeChildren(), b.clearPin(), b.setTileContext(null), V = null, $ = null, _.update(null), J = /* @__PURE__ */ new Map(), ct = null, Ds(r, null), r.moveCenter(en / 2, en / 2), zt(null), Rs == null ? void 0 : Rs.updateValidation([], Ct), R.close();
      }
    });
    function Ot(G) {
      G.length === 0 ? (St.style.display = "none", E.value = "") : (qt.textContent = `${G.length} entit${G.length === 1 ? "y" : "ies"} selected`, St.style.display = "block");
    }
    async function Bt(G) {
      if (X || !(G.regions ?? []).some((Ft) => Ft.kind === "crossing_zone")) return;
      X = true, it.markOptimizeState("active"), B && (B.destroy(), B = null);
      const dt = (Ft) => Ft === "transport-belt" || Ft === "fast-transport-belt" || Ft === "express-transport-belt" || Ft === "underground-belt" || Ft === "fast-underground-belt" || Ft === "express-underground-belt", yt = [], ht = 130, Lt = 90, Wt = 520;
      let Pt = 0, Nt = -1, ge = 0, be = false, Bn = false, ss = null;
      const Au = (Ft) => {
        if (!V) return;
        const Xe = Ft.zone_x, ze = Ft.zone_y, is = Xe + Ft.zone_w, vr = ze + Ft.zone_h, Mu = (Ls) => {
          const La = Ls.x ?? 0, $a = Ls.y ?? 0;
          return La >= Xe && La < is && $a >= ze && $a < vr;
        };
        V.entities = V.entities.filter((Ls) => !(Mu(Ls) && dt(Ls.name))).concat(Ft.entities), nv(i, r, {
          x: Xe,
          y: ze,
          w: Ft.zone_w,
          h: Ft.zone_h
        }), k(V), o();
      }, ku = () => be && yt.length === 0, Ra = (Ft) => {
        if (!Bn) {
          for (; yt.length > 0; ) {
            const Xe = yt[0], ze = Xe.imp.region_id === Nt, is = ze ? Math.min(Wt, ge * Lt) : 0, vr = ht + is;
            if (Ft - Pt < vr) break;
            yt.shift(), Au(Xe.imp), ze ? ge += 1 : (ge = 1, Nt = Xe.imp.region_id), Pt = Ft;
            break;
          }
          if (ku()) {
            ss = null;
            return;
          }
          ss = requestAnimationFrame(Ra);
        }
      };
      ss = requestAnimationFrame(Ra);
      try {
        const Ft = await n.optimizeAllRegions(G, {
          perRegionBudgetMs: 800,
          onImprovement: (ze) => {
            ze.iter !== 0 && yt.push({
              imp: ze
            });
          }
        });
        be = true, await new Promise((ze) => {
          const is = () => {
            yt.length === 0 ? ze() : requestAnimationFrame(is);
          };
          is();
        }), V = Ft, _.update(Ft), q(Ft), window.__layout = Ft;
        const Xe = k(Ft);
        b.setHighlightController(et(Xe)), o();
      } catch (Ft) {
        (Ft instanceof Error ? Ft.message : String(Ft)).includes("superseded") || console.error("[auto-optimize] failed", Ft);
      } finally {
        Bn = true, be = true, ss !== null && cancelAnimationFrame(ss), X = false, it.markOptimizeState("done"), V && (B = Sc(i.canvas, r, L, V, Ot));
      }
    }
    i.canvas.addEventListener("pointermove", (G) => {
      const rt = i.canvas.getBoundingClientRect(), dt = G.clientX - rt.left, yt = G.clientY - rt.top, ht = r.toWorld(dt, yt), Lt = Math.floor(ht.x / S), Wt = Math.floor(ht.y / S);
      b.setCursorTile(Lt, Wt);
      const Pt = J.get(`${Lt},${Wt}`) ?? null;
      Pt !== I && z(Pt), I || b.onHover(null, Lt, Wt);
    }), i.canvas.addEventListener("pointerleave", () => {
      b.setCursorTile(null), I && z(null);
    });
    const jt = 4;
    let me = null;
    i.canvas.addEventListener("pointerdown", (G) => {
      if (G.button !== 0 || G.shiftKey || G.altKey || G.ctrlKey || G.metaKey) {
        me = null;
        return;
      }
      me = {
        x: G.clientX,
        y: G.clientY,
        shifted: false
      };
    }), i.canvas.addEventListener("pointerup", (G) => {
      if (!me) return;
      const rt = G.clientX - me.x, dt = G.clientY - me.y;
      if (me = null, Math.hypot(rt, dt) > jt || G.button !== 0 || G.shiftKey || G.altKey || G.ctrlKey || G.metaKey) return;
      const yt = i.canvas.getBoundingClientRect(), ht = r.toWorld(G.clientX - yt.left, G.clientY - yt.top), Lt = Math.floor(ht.x / S), Wt = Math.floor(ht.y / S);
      if (!m.checked) {
        const be = I && I.x === Lt && I.y === Wt ? I : null;
        if (!be) return;
        b.pinTile(be, Lt, Wt);
        return;
      }
      const Pt = (Z == null ? void 0 : Z(ht.x, ht.y)) ?? null;
      if (Pt) {
        R.open(Pt, V == null ? void 0 : V.trace);
        return;
      }
      if (T) {
        const be = ht.x / S, Bn = ht.y / S;
        if (!(be >= T.bboxX && Bn >= T.bboxY && be < T.bboxX + T.bboxW && Bn < T.bboxY + T.bboxH)) {
          R.close();
          return;
        }
      }
      const Nt = (Xt == null ? void 0 : Xt(ht.x, ht.y)) ?? null;
      if (Nt) {
        const be = (Nt.region.x + Nt.region.width / 2) * S, Bn = (Nt.region.y + Nt.region.height / 2) * S;
        r.moveCenter(be, Bn);
      }
      const ge = b.getPinnedTile();
      if (ge && ge.x === Lt && ge.y === Wt) b.clearPin();
      else {
        const be = I && I.x === Lt && I.y === Wt ? I : null;
        if (!be) return;
        b.pinTile(be, Lt, Wt);
      }
    }), document.addEventListener("keydown", (G) => {
      G.key === "Shift" && r.plugins.pause("drag");
    }), document.addEventListener("keyup", (G) => {
      G.key === "Shift" && r.plugins.resume("drag");
    }), window.addEventListener("blur", () => r.plugins.resume("drag"));
    function es(G) {
      tt == null ? void 0 : tt.cancel(), tt = null, U == null ? void 0 : U.cancel(), U = null, it.reset(), L.removeChildren(), H.removeChildren(), $ = G, Ds(r, G), G || r.moveCenter(en / 2, en / 2);
    }
    function Ia() {
      U == null ? void 0 : U.cancel(), it.reset(), Ds(r, null), U = cv(L, i);
      let G = false, rt = null;
      return (dt) => {
        if (dt.phase === "PhaseSnapshot") {
          const ht = dt.data;
          ht.width > 0 && ht.height > 0 && (G || (r.fit(true, ht.width * S * 1.15, ht.height * S * 1.25), r.moveCenter(ht.width * S / 2, ht.height * S / 2), G = true), rt ? rt.resize(ht.width + 2, ht.height + 2) : rt = Is(ht.width + 2, ht.height + 2));
        }
        U == null ? void 0 : U.onEvent(dt, (ht) => {
          U && it.noteMilestone(ht, U.getTimeRange());
        });
      };
    }
    function Is(G, rt) {
      let dt = G, yt = rt, ht = false;
      if (h) return Wn(c, dt, yt), o(), ht = true, {
        cancel: () => {
        },
        resize(Nt, ge) {
          Wn(c, Nt, ge), o();
        }
      };
      const Lt = 250, Wt = performance.now();
      c.alpha = 1, Wn(c, dt, yt, 0);
      const Pt = () => {
        if (ht) return;
        const Nt = Math.min(1, (performance.now() - Wt) / Lt);
        Wn(c, dt, yt, Nt), o(), Nt >= 1 && (ht = true, h = true, i.ticker.remove(Pt), l());
      };
      return i.ticker.add(Pt), a(), {
        cancel() {
          ht || (ht = true, h = true, i.ticker.remove(Pt), l(), Wn(c, dt, yt), o());
        },
        resize(Nt, ge) {
          dt = Nt, yt = ge, ht && (Wn(c, dt, yt), o());
        }
      };
    }
    function Ge(G, rt) {
      Xd(c0(G.entities)), V = G, _.update(G), rt && ($ = rt), q(G), window.__layout = G, xv(G), D = false, K == null ? void 0 : K.cancel(), K = null, B && (B.destroy(), B = null), tt == null ? void 0 : tt.cancel(), tt = null, St.style.display = "none", E.value = "", ct = null, Ds(r, null);
      let dt;
      if (U == null ? void 0 : U.hasCommittedEntities()) dt = U.finish(G), it.arm(U.getTimeRange(), U.getMilestones());
      else if (U == null ? void 0 : U.cancel(), U = null, it.reset(), (Array.isArray(G.trace) ? G.trace : []).some((Pt) => Pt.phase === "PhaseSnapshot")) {
        const Pt = tv(G, L, z, F, i);
        dt = Pt.controller, tt = Pt.handle;
      } else dt = k(G);
      b.setHighlightController(et(dt)), ot = Yw(G.trace), b.setTileContext(ot), b.clearPin(), B = Sc(i.canvas, r, L, G, Ot), ut(), $t(), Et(), xe(), Ce(), lb(H, G, $);
      const yt = G.width ?? 0, ht = G.height ?? 0;
      if (Wn(c, yt + 2, ht + 2), yt > 0 && ht > 0) {
        const Lt = yt * 32, Wt = ht * 32, Pt = 192;
        r.fit(true, Lt * 1.1, (Wt + Pt) * 1.2), r.moveCenter(Lt / 2, (Wt - Pt) / 2);
      }
      M && (L.alpha = 0.12), o(), requestAnimationFrame(() => {
        V === G && ns(G);
      });
    }
    function ns(G) {
      V !== G || at === G || ct === null || [
        ...ct,
        ...oe(G)
      ].length > 0 || Bt(G);
    }
    document.addEventListener("keydown", (G) => {
      if (G.ctrlKey) {
        if (G.key === "c") {
          if (!B || B.getSelected().length === 0) return;
          G.preventDefault();
          const rt = (Rs == null ? void 0 : Rs.getParams()) ?? null, dt = B.buildJson(rt, E.value.trim());
          navigator.clipboard.writeText(dt).catch(() => {
          }), P.textContent = "Copied!", setTimeout(() => {
            P.textContent = "Ctrl+C to copy JSON";
          }, 2e3);
        } else if (G.key === "o") {
          G.preventDefault();
          const rt = document.createElement("input");
          rt.type = "file", rt.accept = ".fls", rt.addEventListener("change", async () => {
            var _a2;
            const dt = (_a2 = rt.files) == null ? void 0 : _a2[0];
            if (dt) try {
              const yt = await dt.text(), ht = await Cu(yt);
              mt.load(ht);
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
      let G = function(Pt) {
        const Nt = document.createElement("button");
        return Nt.textContent = Pt, Nt.style.cssText = "flex:1;padding:10px 4px;background:none;border:none;border-bottom:2px solid transparent;color:#777;font:12px 'JetBrains Mono','Consolas',monospace;cursor:pointer;letter-spacing:0.5px;transition:all 0.15s", Nt;
      }, rt = function(Pt) {
        const Nt = Pt === "generate";
        Lt.style.display = Nt ? "flex" : "none", Wt.style.display = Nt ? "none" : "flex", yt.style.borderBottomColor = Nt ? "#569cd6" : "transparent", yt.style.color = Nt ? "#d4d4d4" : "#777", ht.style.borderBottomColor = Nt ? "transparent" : "#569cd6", ht.style.color = Nt ? "#777" : "#d4d4d4";
      };
      const dt = document.createElement("div");
      dt.style.cssText = "display:flex;border-bottom:1px solid #2a2a2a;background:#141414;flex-shrink:0";
      const yt = G("Generate"), ht = G("Corpus");
      dt.appendChild(yt), dt.appendChild(ht);
      const Lt = document.createElement("div");
      Lt.style.cssText = "flex:1;overflow:hidden;display:flex;flex-direction:column;";
      const Wt = document.createElement("div");
      Wt.style.cssText = "flex:1;overflow:hidden;display:none;flex-direction:column;", bn.style.cssText += ";display:flex;flex-direction:column;padding:0;overflow:hidden;", bn.appendChild(dt), bn.appendChild(Lt), bn.appendChild(Wt), yt.onclick = () => rt("generate"), ht.onclick = () => rt("corpus"), rt("generate"), Rs = Bb(Lt, n, {
        renderGraph: es,
        renderLayout: Ge,
        startStreaming: Ia
      }), u.addEventListener("change", () => {
        ut(), $t(), Et(), xe();
      }), w.addEventListener("change", () => {
        xe();
      }), x.addEventListener("change", () => {
        ut();
      }), p.addEventListener("change", () => {
        Bi(p.checked), V && Ge(V);
      }), f.addEventListener("change", () => {
        Ut();
      }), g.addEventListener("change", () => {
        Ce();
      }), m.addEventListener("change", () => {
        Et();
      }), y.addEventListener("change", () => {
        const Pt = () => o();
        y.checked ? (M = true, O = {
          colorChecked: p.checked,
          regionsChecked: m.checked,
          entityAlpha: L.alpha
        }, m.checked || (m.checked = true, Et()), p.checked && (p.checked = false, Bi(false), V && Ge(V)), L.alpha = 0.12, Et(), Pt()) : (M = false, O && (L.alpha = O.entityAlpha, m.checked !== O.regionsChecked && (m.checked = O.regionsChecked, Et()), p.checked !== O.colorChecked && (p.checked = O.colorChecked, Bi(p.checked), V && Ge(V)), O = null), Pt());
      }), s_(Wt, Ge);
    }
  }
  _v().catch((n) => {
    console.error("Failed to initialize app:", n);
  });
})();
export {
  bh as $,
  sa as A,
  Zi as B,
  Ht as C,
  am as D,
  rn as E,
  Qp as F,
  mi as G,
  Zn as H,
  Cs as I,
  le as J,
  ee as K,
  Kt as L,
  qs as M,
  ch as N,
  vt as O,
  lt as P,
  ft as Q,
  Dt as R,
  ur as S,
  S as T,
  ue as U,
  my as V,
  lg as W,
  fr as X,
  Qm as Y,
  yn as Z,
  mr as _,
  __tla,
  ir as a,
  Rt as a0,
  Gh as a1,
  vo as a2,
  Gt as a3,
  Yn as a4,
  si as a5,
  Mt as a6,
  ip as a7,
  im as a8,
  Ps as a9,
  fd as aA,
  zh as aB,
  fi as aC,
  Cf as aD,
  xf as aE,
  Zh as aF,
  hm as aG,
  Bm as aH,
  Nm as aI,
  Um as aJ,
  Dm as aK,
  jm as aL,
  pl as aa,
  Yi as ab,
  ta as ac,
  Nh as ad,
  Ie as ae,
  na as af,
  $m as ag,
  Om as ah,
  Hm as ai,
  Gm as aj,
  Uu as ak,
  Vm as al,
  ve as am,
  ah as an,
  Me as ao,
  cr as ap,
  Ka as aq,
  Ne as ar,
  Vy as as,
  wp as at,
  Za as au,
  Rr as av,
  Qa as aw,
  vp as ax,
  hh as ay,
  gr as az,
  Fn as b,
  _r as c,
  Cv as d,
  Qi as e,
  ea as f,
  It as g,
  Tt as h,
  Oo as i,
  Kn as j,
  cn as k,
  Bo as l,
  Ri as m,
  Yt as n,
  Fe as o,
  xd as p,
  bd as q,
  ei as r,
  Be as s,
  vv as t,
  By as u,
  te as v,
  vy as w,
  ne as x,
  Jt as y,
  kt as z
};
