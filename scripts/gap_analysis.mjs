#!/usr/bin/env node
// Strategy-gap analysis: what does the human corpus do that spaghettio can't?
//
// Reads scripts/blueprints/_sweep_rust.jsonl (Rust classify() output) and
// scripts/blueprints/_deep_dive.md (DI pair census + port geometry) to
// quantify the gap between community strategy and spaghettio's bus model.
//
// Output: scripts/blueprints/_gap_analysis.md
//
// Usage: node scripts/gap_analysis.mjs

import { promises as fs } from 'node:fs';
import path from 'node:path';

const CORPUS = 'scripts/blueprints';
const SWEEP = path.join(CORPUS, '_sweep_rust.jsonl');
const DEEP = path.join(CORPUS, '_deep_dive.md');
const OUT = path.join(CORPUS, '_gap_analysis.md');

const rows = (await fs.readFile(SWEEP, 'utf8')).trim().split('\n').map(JSON.parse);
const deep = await fs.readFile(DEEP, 'utf8');

const out = [];
const say = (s = '') => { out.push(s); };

say('# Strategy-gap analysis: community corpus vs spaghettio\n');
say(`Corpus: ${rows.length} blueprint members from the Factorio Prints top-favorites.`);
say(`Generated from Rust \`classify()\` output (\`_sweep_rust.jsonl\`).\n`);

// ---- A) Spaghettio's capability envelope ----
say('## A) Spaghettio\'s capability envelope\n');
say('Spaghettio is a **strict bus architecture**: every recipe is a self-contained');
say('row (`belt-in → inserter → machine → inserter → belt-out`), rows stack');
say('vertically in dependency order, all intermediates ride the trunk as belt');
say('lanes. The engine **never produces direct insertion** (machine→inserter→machine');
say('without a belt) — DI is documented as a Phase 3 future candidate in');
say('`rfc-decomposition-search.md` but not implemented. The engine also never');
say('produces: beacons, logistic/bot networks, train stations, nuclear/power');
say('layouts, sushi belts, or circuit combinator networks.\n');

// ---- B) Direct insertion: the dominant human strategy spaghettio can't do ----
say('## B) Direct insertion — the primary gap\n');
const diUsers = rows.filter((r) => r.features?.direct_insertion > 0);
const totalDI = rows.reduce((s, r) => s + (r.features?.direct_insertion ?? 0), 0);
const totalInserters = rows.reduce((s, r) => s + (r.features?.inserters ?? 0), 0);
const diFraction = totalInserters > 0 ? (totalDI / totalInserters * 100).toFixed(1) : 0;

say(`**${diUsers.length}** of ${rows.length} members (${(diUsers.length / rows.length * 100).toFixed(1)}%) use direct insertion.`);
say(`**${totalDI.toLocaleString()}** DI inserters out of ${totalInserters.toLocaleString()} total inserters (${diFraction}%).`);
say('Spaghettio produces **zero** DI inserters by construction.\n');

// DI by archetype
say('### DI usage by archetype\n');
const diByArch = new Map();
for (const r of diUsers) {
  const a = r.features?.archetype ?? '?';
  const cur = diByArch.get(a) ?? { members: 0, di: 0, favs: 0 };
  cur.members++; cur.di += r.features.direct_insertion; cur.favs += r.favorites;
  diByArch.set(a, cur);
}
say('| Archetype | Members with DI | Total DI inserters | Avg DI/member | Favorites |');
say('|-----------|----------------:|-------------------:|--------------:|----------:|');
for (const [a, v] of [...diByArch.entries()].sort((a, b) => b[1].di - a[1].di)) {
  say(`| ${a} | ${v.members} | ${v.di.toLocaleString()} | ${(v.di / v.members).toFixed(1)} | ${v.favs.toLocaleString()} |`);
}

// Top DI pairs from deep_dive.md
say('\n### Top direct-insertion pairs (from the DI census)\n');
say('These are the machine→machine pairs humans wire most often. Spaghettio');
say('routes every one of these through the bus trunk instead.\n');
const diSection = deep.split('## B) Direct-insertion pairs')[1] ?? '';
const pairLines = diSection.trim().split('\n').filter((l) => /^\s*\d+/.test(l));
say('| Count | Recipe pair (producer → consumer) |');
say('|------:|-----------------------------------|');
for (const line of pairLines.slice(0, 15)) {
  const m = line.match(/^\s*(\d+)\s+(.+)/);
  if (m) say(`| ${parseInt(m[1]).toLocaleString()} | ${m[2]} |`);
}

// ---- C) Archetype coverage gap ----
say('\n## C) Archetype coverage gap\n');
say('Archetypes spaghettio can produce: `production-block` (the bus layout).');
say('Everything else is a gap.\n');
const archTab = new Map();
for (const r of rows) {
  const a = r.features?.archetype ?? '?';
  const cur = archTab.get(a) ?? { n: 0, favs: 0 };
  cur.n++; cur.favs += r.favorites;
  archTab.set(a, cur);
}
const spaghettioCan = new Set(['production-block']);
say('| Archetype | Members | % of corpus | Favorites | Spaghettio? |');
say('|-----------|--------:|------------:|----------:|-------------|');
for (const [a, v] of [...archTab.entries()].sort((a, b) => b[1].n - a[1].n)) {
  const pct = (v.n / rows.length * 100).toFixed(1);
  const can = spaghettioCan.has(a) ? '✓ yes' : '✗ no';
  say(`| ${a} | ${v.n} | ${pct}% | ${v.favs.toLocaleString()} | ${can} |`);
}
const covered = [...spaghettioCan].flatMap((a) => archTab.get(a) ? [archTab.get(a).n] : []);
const coveredCount = covered.reduce((s, n) => s + n, 0);
say(`\nSpaghettio covers **${coveredCount}** of ${rows.length} members (${(coveredCount / rows.length * 100).toFixed(1)}%).`);
say('Even within `production-block`, DI and beacon usage are strategies humans');
say('apply that spaghettio\'s bus model doesn\'t replicate.\n');

// ---- D) Beacons ----
say('## D) Beacons — universal in optimized production, absent in spaghettio\n');
const beaconed = rows.filter((r) => r.features?.beacons > 0 || (r.features && r.features.archetype === 'production-block' && (r.total_entities ?? 0) > 0 && deep.includes('beaconed')));
// The sweep doesn't have a beaconed flag directly; let's check features
const beaconedProd = rows.filter((r) => {
  // We don't have a beacon count in features; approximate via deep_dive mentions
  return r.features?.archetype === 'production-block';
});
say('Beacons are not tracked as a feature in `BlueprintFeatures` (the census');
say('counts them but the field isn\'t exposed). The deep-dive port geometry');
say('shows the top tileable production blocks are overwhelmingly `beaconed: true`.');
say('Spaghettio has no beacon placement logic — modules are a known gap.\n');

// ---- E) Tileability + port geometry ----
say('## E) Tileability and port geometry — the composable-block gap\n');
const tileable = rows.filter((r) => r.features?.tileable_geom);
say(`**${tileable.length}** members (${(tileable.length / rows.length * 100).toFixed(1)}%) have tileable geometry (pitch score ≥ 0.4, ≥6 machines).`);
say('These are the blocks humans stamp repeatedly to scale. Spaghettio');
say('produces monolithic layouts — it has no "stamp this block N times" mode.\n');

// pitch distribution
const pitches = tileable.map((r) => r.features?.pitch).filter((p) => p && p > 0);
const pitchHist = new Map();
for (const p of pitches) pitchHist.set(p, (pitchHist.get(p) ?? 0) + 1);
say('Pitch distribution (tileable members):');
say('```');
for (const [p, n] of [...pitchHist.entries()].sort((a, b) => a[0] - b[0]).slice(0, 20)) {
  say(`  pitch ${String(p).padStart(2)}: ${'█'.repeat(Math.min(n, 50))} ${n}`);
}
say('```\n');
say('Human tileable blocks concentrate at pitch 3 (smelting/EC), 5-7 (beaconed');
say('assemblers), and 2 (compact drills). Spaghettio\'s row pitch is fixed by');
say('the template (machine size + 2 inserter rows + 2 belt rows) — typically');
say('pitch 5-7 for 3×3 machines, matching the human distribution. But spaghettio');
say('never *repeats* a block: it scales by extending the row, not stamping copies.\n');

// ---- F) Summary ----
say('## F) Gap summary\n');
say('| Gap | Corpus evidence | Spaghettio status |');
say('|-----|------------------|-------------------|');
say(`| Direct insertion | ${totalDI.toLocaleString()} DI inserters in ${diUsers.length}/${rows.length} members | ✗ not produced (Phase 3 candidate) |`);
say(`| Beacons | universal in optimized blocks | ✗ no beacon placement |`);
say(`| Tileable blocks | ${tileable.length} members (${(tileable.length / rows.length * 100).toFixed(1)}%) | ✗ monolithic layouts only |`);
say(`| Bot logistics | ${archTab.get('bot-logistics')?.n ?? 0} members | ✗ no bot model |`);
say(`| Train stations | ${archTab.get('train-station')?.n ?? 0} members | ✗ no train model |`);
say(`| Power/nuclear | ${archTab.get('power')?.n ?? 0} members | ✗ no power-gen layouts |`);
say(`| Balancers | ${archTab.get('balancer')?.n ?? 0} members | partial (balancer library, bus-internal only) |`);
say(`| Malls | ${archTab.get('mall')?.n ?? 0} members | ✗ no multi-product layout |`);
say('\n### The one gap with a quantitative handle: direct insertion\n');
say('The DI pair census is the most actionable dataset. The top pairs are:');
say('- `copper-cable → electronic-circuit` (3,887 instances) — the canonical 2:1 DI pair');
say('- `casting-copper-cable → electronic-circuit` (544) — the foundry variant');
say('- `iron-gear-wheel → inserter` (113), `iron-gear-wheel → transport-belt` (96)');
say('- `iron-stick → rail` (345) — the 1:1 rail pair');
say('');
say('These are exactly the "tight producer/consumer pairs with clean ratios"');
say('that `rfc-decomposition-search.md` Phase 3 targets. The corpus evidence');
say('confirms the RFC\'s priority ordering: cable→EC is the single most common');
say('DI pair in the game, and it\'s the one spaghettio routes through a full');
say('bus round-trip (cable row → output belt → trunk lane → EC row input belt).');

await fs.writeFile(OUT, out.join('\n') + '\n');
console.log(`-> ${OUT}`);
console.log(`${rows.length} members analyzed, ${diUsers.length} use DI, ${tileable.length} tileable`);
