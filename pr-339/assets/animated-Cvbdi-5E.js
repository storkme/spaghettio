import { r as x } from "./index-BsD-xC5y.js";
function y(a, c, r, m) {
  x(a, c);
  const o = c.children.slice(), h = o.length;
  for (const i of o) i.alpha = 0;
  const t = a.entities.length;
  let n;
  t < 50 ? n = 2 : t < 200 ? n = 6 : n = Math.max(10, Math.ceil(t / 30));
  const s = h / t, u = Math.max(1, Math.round(n * s)), M = t < 50 ? 30 : t < 200 ? 20 : 12;
  let e = 0, d = 0;
  function f() {
    const i = Math.min(e + u, h);
    for (let l = e; l < i; l++) o[l].alpha = 1;
    e = i, d = Math.min(Math.round(e / s), t), r.textContent = `${d} / ${t}`, e < h ? setTimeout(f, M) : (r.textContent = `${t} entities`, m());
  }
  setTimeout(f, 150);
}
export {
  y as renderLayoutAnimated
};
