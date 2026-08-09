# Warm vs cold re-review — final results

**Question.** When a reviewer re-reviews a PR it has already reviewed, does
resuming its prior session (warm) beat starting fresh (cold)?

**Setup.** Warm = 3 passes, each forked from that pass's *own* prior-round pi
session, shown the next round's tree, asked to re-assess its earlier findings
and hunt for new ones. Cold = what the real reviewer actually posted that round.
Both K=3, unioned. Same model and harness as production
(`deepseek-v4-flash-0731` via pi). 7 transitions, 4 PRs, retrospective on real
review history. Scored by one agent per transition against a fixed rubric.

## Results

| Transition | Content | HC recall | Overall | Unique | Compression | Arith. pathology |
|---|---|---:|---:|---:|---:|---:|
| 576 r10→11 | meter docs | 1/1 | 5/6 | 2 | n/a | **2** |
| 576 r5→6 | meter docs | 1/1 | 3/5 | 6 | 1 | **2** |
| 606 r6→7 | meter docs | 0/2 | 1/4 | 3 | 0 | 0 |
| 606 r9→10 | meter docs | 2/2 | 2/7 | 4 | 0 | 0 |
| 608 r7→8 | code (validator) | 2/2 | 5/6 | 9 | 4 | 0 |
| 608 r4→5 | code (validator) | 4/6 | 6/11 | 5 | 1 | 0 |
| 604 r5→6 | code (py/shell) | 3/4 | 5/12 | 8 | 2 | 0 |

- High-confidence recall **13/18 (72%)** · overall recall **27/51 (53%)**
- **37** findings warm raised that cold did not
- **8** same-defect compression hits (warm found it a full round early)
- Cost: **a wash** — warm $0.0230 vs cold $0.0241 per pass, both measured

Dropped: 604 r8→9 (session artifacts exist, **no posted review** for round 9 —
a review that ran and published nothing). 576 r10 has no round-12 comparator.

## Conclusions

**1. Warm cannot replace cold.** 53% overall recall means swapping cold for warm
loses about half the findings, including a bare `.unwrap()` that would panic a
whole sweep.

**2. Warm adds real value.** 37 unique findings and 8 defects caught a round
early. It is *differently blind*, not worse.

**→ Recommendation: split K=3 into warm and cold passes and union as now.**
Cost unchanged (still 3 passes); gains warm's compression without losing cold's
recall. The optimal split is unmeasured — this compared warm-K3 to cold-K3, not
mixed.

**3. Do not warm-fork arithmetic-dense documentation.** All four instances of
confident-wrong arithmetic appeared on the two meter-docs transitions of #576:
two passes "verifying" figures that were themselves the bug (r10), one false
clearance of a real contradiction, one false alarm from an invented premise
(r5). Mechanism: a fork carries stale numbers in context and reasons from them
instead of recomputing.

## Hypotheses killed along the way

- *Delta-scoping the diff saves money* — the diff is ~0.3% of tokens; the cost
  is agentic re-derivation.
- *Warm is cheaper* — a wash. Warm ships fewer total tokens but a higher share
  of expensive fresh input.
- *Self-consistency bias makes warm rubber-stamp* — the opposite; warm was more
  skeptical than cold in the pilot.
- *Warm compresses on code, not docs* — falsified by 576 r5 (docs, 1 hit).
- *Smaller PRs cut review spend* — they raise it ~2.5×; the norm is justified by
  rework, not cost.

## Caveats

- n=7 transitions, 4 PRs, one repo, one model.
- Content-type is confounded with PR identity: both zero-compression
  transitions are #606, and all four arithmetic pathologies are #576.
- Compression's ground truth requires cold's *next* round to have found the
  thing; those reviews sample, so a miss is not proof warm was early.
- Compression within code is noisy (4/9, 2/8, 1/5 of unique findings), and
  608 r4's single hit turns on a same-defect-vs-same-class judgement call its
  own scorer flagged as contestable.

## Incidental findings worth acting on

- **`docs/meter-divergence.md`'s L7 retraction is wrong.** Its claim that meter
  and sim "agree by design" at non-bulk L7 is contradicted by its own cited
  code (`entity_data.rs:365` declares axis 3; `scenario.rs:901-903` yields
  realized hand 4), with fixture `chain-ec15-d7` measuring the disagreement.
  Found independently by two warm passes on 604; cold never touched that file.
  Arithmetic re-derived by the scorer against the repo.
- **PR 604 round 9 published no review** despite running — degraded class,
  visible in the historical record.

## Also learned about the stack

- Prompt caching is already doing most of the work available: `cacheRead` is
  ~78% of tokens at ~10× cheaper than fresh input, and prompt ordering is
  already cache-optimal (stable system prompt first, volatile diff last).
- **Cross-run caching does not hit** — a byte-identical prefix two minutes
  apart got zero `cacheRead`. Cause: pi has `sendSessionAffinityHeaders`
  (`sessionAffinityFormat: openrouter` → `x-session-id`) but it **defaults to
  false**, so OpenRouter's sticky routing never engages. Worth ~⅓ of a warm
  round. Note affinity derives from the session id, and a fork gets a new one,
  so it likely needs the id pinned across rounds.
