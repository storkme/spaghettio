#!/usr/bin/env node
// Corpus sweep via the Rust `classify()` (blueprint-analyze --batch --json).
//
// Replaces the standalone Node reference extractor (sweep_corpus.mjs) now
// that a Rust toolchain is available on this box. Emits one feature row per
// blueprint member as JSONL, tagged with print id / member index / title /
// favorites / tags, so downstream gap-analysis scripts consume a single
// canonical feed instead of maintaining a parallel implementation.
//
// Output: scripts/blueprints/_sweep_rust.jsonl  (gitignored corpus dir)
//
// Usage: node scripts/sweep_rust.mjs

import { promises as fs } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const CORPUS = 'scripts/blueprints';
const OUT = path.join(CORPUS, '_sweep_rust.jsonl');
const BIN = 'target/release/blueprint-analyze';

const files = (await fs.readdir(CORPUS))
  .filter((f) => f.endsWith('.json') && !f.startsWith('_'))
  .sort();

// Feed every print's blueprintString to the analyzer in batch mode (one
// bp string per line, name<TAB>bp). The binary expands books and emits
// one entry per member.
const lines = [];
const meta = new Map(); // print_id -> { favorites, tags, title }
for (const file of files) {
  const j = JSON.parse(await fs.readFile(path.join(CORPUS, file), 'utf8'));
  const s = (j.blueprintString ?? '').trim();
  if (!s) continue;
  const print_id = file.split('_')[0];
  meta.set(print_id, {
    favorites: j.numberOfFavorites ?? 0,
    tags: j.tags ?? [],
    title: j.title ?? file,
  });
  lines.push(`${print_id}\t${s}`);
}

const res = spawnSync(BIN, ['--batch', '--json'], {
  input: lines.join('\n') + '\n',
  maxBuffer: 1 << 30,
  encoding: 'utf8',
});
if (res.status !== 0) {
  console.error('analyzer exited', res.status);
  console.error(res.stderr);
  process.exit(1);
}

const parsed = JSON.parse(res.stdout);
const out = [];
const knownIds = new Set(meta.keys());
for (const bp of parsed.blueprints) {
  const name = bp.name ?? '';
  // The batch binary bakes our print_id into `name` as the base of
  // entry_label (e.g. "-Kl-NTpuTXPVfk0N0EMo" or "-Kl-NTpuTXPVfk0N0EMo[0] (label)").
  // Match against known print_ids to recover it.
  let print_id = null;
  for (const id of knownIds) {
    if (name === id || name.startsWith(id + '[') || name.startsWith(id + ' (')) {
      print_id = id;
      break;
    }
  }
  if (!print_id) {
    // fallback: try prefix match
    for (const id of knownIds) {
      if (name.startsWith(id)) { print_id = id; break; }
    }
  }
  const m = meta.get(print_id) ?? { favorites: 0, tags: [], title: name };
  const prevCount = out.filter((r) => r.print_id === print_id).length;
  out.push({
    print_id,
    member_index: prevCount,
    title: m.title,
    member: bp.label ?? null,
    favorites: m.favorites,
    tags: m.tags,
    features: bp.features,
    archetype: bp.features?.archetype,
    chain_level: bp.features?.chain_level,
    final_products: bp.final_products ?? [],
    total_entities: bp.total_entities,
    width: bp.width,
    height: bp.height,
  });
}

await fs.writeFile(OUT, out.map((r) => JSON.stringify(r)).join('\n') + '\n');
console.log(`${out.length} members from ${files.length} prints -> ${OUT}`);

// ---- agreement summary (Rust vs Node reference) ----
const node = path.join(CORPUS, '_sweep.jsonl');
let nodeRows = [];
try {
  nodeRows = (await fs.readFile(node, 'utf8')).trim().split('\n').map(JSON.parse);
} catch { console.log('(no Node _sweep.jsonl to compare)'); }

if (nodeRows.length) {
  const agg = (rows, pred) => rows.filter(pred).length;
  console.log('\n== Rust vs Node reference (member counts) ==');
  for (const [label, fn] of [
    ['direct_insertion > 0', (r) => (r.features?.direct_insertion ?? 0) > 0],
    ['tileable_geom', (r) => r.features?.tileable_geom],
    ['mixed_belt_networks > 0', (r) => (r.features?.mixed_belt_networks ?? 0) > 0],
    ['self_powered', (r) => r.features?.self_powered],
    ['archetype=mall', (r) => r.features?.archetype === 'mall'],
    ['archetype=production-block', (r) => r.features?.archetype === 'production-block'],
    ['archetype=balancer', (r) => r.features?.archetype === 'balancer'],
  ]) {
    const r = agg(out, fn), n = agg(nodeRows, fn);
    console.log(`${label.padEnd(30)} rust ${String(r).padStart(5)} | node ${String(n).padStart(5)}`);
  }
  // total DI inserters
  const rdi = out.reduce((s, r) => s + (r.features?.direct_insertion ?? 0), 0);
  const ndi = nodeRows.reduce((s, r) => s + (r.features?.direct_insertion ?? 0), 0);
  console.log(`${'total direct_insertion'.padEnd(30)} rust ${String(rdi).padStart(5)} | node ${String(ndi).padStart(5)}`);
}

// archetype x chain crosstab
const tab = new Map();
for (const r of out) {
  const k = `${r.features?.archetype ?? '?'} | ${r.features?.chain_level ?? '?'}`;
  const cur = tab.get(k) ?? { n: 0, favs: 0 };
  cur.n++; cur.favs += r.favorites;
  tab.set(k, cur);
}
console.log('\n== archetype x chain (members | favorites) ==');
[...tab.entries()].sort((a, b) => b[1].favs - a[1].favs)
  .forEach(([k, v]) => console.log(String(v.n).padStart(4), String(v.favs).padStart(7), k));
