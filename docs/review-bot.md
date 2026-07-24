# CI review bot — how it works, how it fails, how to diagnose it

Reference doc for the automated PR review pipeline. `CLAUDE.md` carries only
the operating rules; this file is the canonical home for the failure-class
history and the forensics playbook. Keep it current when the workflow
changes.

## Moving parts

- [`.github/workflows/claude-code-review.yml`](../.github/workflows/claude-code-review.yml)
  — runs on every PR event (opened / synchronize / ready_for_review /
  reopened): checkout → `anthropics/claude-code-action@v1` running the
  `code-review` plugin → transcript artifact upload → silent-no-op guard.
- **The plugin** (`code-review@claude-code-plugins`, from the
  anthropics/claude-code marketplace) is a multi-subagent orchestration:
  a haiku gate (skip closed / draft / trivial / already-reviewed PRs) → a
  CLAUDE.md-paths agent → a sonnet diff-summary agent → 4 parallel reviewer
  agents (2× CLAUDE.md compliance, 2× opus bug-hunting) → per-finding
  validation subagents → post. With `--comment` (which we pass —
  load-bearing) it posts inline comments on findings or a
  "No issues found" summary comment when clean.
- [`clear-agent-reviewed.yml`](../.github/workflows/clear-agent-reviewed.yml)
  — drops the `agent-reviewed` label when new commits land, so a new head
  SHA gets re-reviewed.

## Expected behavior

On a substantive PR: **inline comments on findings, or a "no issues"
summary comment**. Designed silences (no post, and that's correct):

- Draft PRs, closed PRs (plugin gate).
- PRs the gate deems trivially correct.
- PRs where Claude already commented (gate) — includes session-side reviews
  that read as Claude output; re-review after new commits is still expected
  via the label-clear workflow, but the gate may legitimately decline when
  coverage already exists.
- PRs touching `.github/workflows/**` (the action's anti-hijack self-skip).
  These need session-side review instead — the bot cannot review changes to
  itself.

Any other green check with nothing posted is a failure. Since the guard
step landed (2026-07-24), that combination fails the check outright on
non-draft PRs with ≥20 changed lines, zero review activity, and a
transcript that doesn't bear the cheap gate-skip signature (see Guard
semantics below).

## Failure-class history

Five classes, each individually sufficient to produce the same symptom —
green check, nothing posted:

| # | Cause | Symptom signature | Fixed |
|---|-------|-------------------|-------|
| 1 | Stock template ships `pull-requests: read` — every post discarded | ~20s run, 1 denial, "No buffered inline comments" | #327 |
| 2 | Plugin's `--comment` flag not passed — its own contract is "do not post" without it | review runs fully, prints to action log, posts nothing | #329 |
| 3 | No harness `--allowedTools` — every posting/diff call denied | 11 denials on the #330 canary | #331 |
| 4 | Re-running `/install-github-app` overwrote both workflow files with the stock template — wiped fixes 1–3 at once (plus `claude.yml`'s owner-only sender gate) | all of the above, after a period of working fine | #369 (canary #368) |
| 5 | Shape-sensitive denial starvation — the allowlist admits plain `gh pr/issue` commands but denies improvised *shapes* (env prefixes `GH_PAGER= gh …`, command substitution `$(…)`, `cd … &&` chains, `gh api`); enough denials and the orchestrator abandons mid-review without posting. Stochastic per run, worse on large PRs (more context wanted → more improvised commands). Diagnosed 2026-07-24 on PR #389 (two consecutive silent no-ops; PR #405 succeeded through 31 denials the same day) | mid-cost, mid-duration run: more than a gate-skip, far less than a full review; see signature table below | prompt hardening + guard step (introduced with this doc) |

Validation history: planted-bug canary #330 (2026-07-21) — first-ever bot
comment correctly flagged the bug inline with a committable fix; canary
#368 re-validated after the installer overwrite was restored in #369.

## Forensics playbook

**Run signatures** (from the action's result JSON in the job log — grep the
log for `total_cost_usd|num_turns|permission_denials_count|duration_ms`):

| Outcome | Duration | Turns | Cost | Reading |
|---------|----------|-------|------|---------|
| Designed gate-skip | ~20s | 2–6 | ~$0.2 | fine if a skip condition genuinely holds |
| Mid-review abandonment (class 5) | 1–2 min | 15–20 | $0.6–1.5 | got past the gate, died before/at the fan-out; check denial count |
| Completed full review | ~8 min | ~10 | ~$4 | the 4-agent fan-out + validators ran; post should exist |

Reference points, all 2026-07-23/24: PR 405 full review 500s/10 turns/31
denials/$4.16 (posted); PR 405 incremental pushes ~20s/$0.17–0.25
(gate-skip, comment already existed); PR 389 run 1 66s/16 turns/6
denials/$0.62 and run 2 108s/20 turns/13 denials/$1.45 (both abandoned,
nothing posted).

**Steps:**

1. Confirm what was actually posted: `gh api repos/<repo>/pulls/<n>/reviews`,
   `.../pulls/<n>/comments` (inline), and `gh pr view <n> --json comments`
   (summary comments; the bot posts as a `claude`-ish login).
2. Pull the run stats:
   `gh run list --workflow=claude-code-review.yml --branch <branch>` then
   `gh run view <id> --log | grep -E 'total_cost_usd|num_turns|permission_denials'`.
   Classify against the signature table.
3. Download the `claude-execution-output` artifact (uploaded on every run
   since 2026-07-24, 14-day retention) — the full transcript, including
   exactly which tool calls were denied. This turns class-5 diagnosis from
   inference into reading.
4. To probe permission semantics locally, replicate the environment:
   `claude -p --setting-sources "" --allowedTools "<the workflow's exact list>"`
   with a battery of read-only commands, and ask for an allowed/denied
   table. Measured 2026-07-24: plain `gh pr/issue` subcommands, pipes,
   read-only `git`, and `Task` (subagent fan-out) are ALLOWED; env-prefixed
   forms, `$(…)` substitution, `cd`-chains, and everything else
   non-read-only (`gh api`, `cargo`, …) are DENIED.

## Guard semantics

The final workflow step fails the check when a PR has **zero** review
activity (no PR reviews from anyone, no inline comments, no
`claude[bot]`/`claude` summary comment — exact login match) despite being
non-trivial. Carve-outs, in order: draft PRs; PRs touching
`.github/workflows/**` (deliberately broader than the action's actual
self-skip — over-carving fails safe because workflow PRs get session-side
review by repo rule); diffs under 20 changed lines; and, when coverage is
zero, a transcript whose `num_turns ≤ 8` — the conscious gate-skip
signature (observed skips run 2–6 turns; observed abandonments 16–20), so
a plugin that *looked and declined* passes while a run that *died
mid-review* fails.

Design decisions worth knowing:

- **Coverage is PR-lifetime, not per-SHA.** Deliberate: the plugin's gate
  declines to re-review a PR Claude already commented on, so per-SHA
  enforcement would permanently red every follow-up push. Per-SHA
  freshness is owned by the `agent-reviewed` label flow
  (`clear-agent-reviewed.yml`), not the guard. Consequence: a bot no-op on
  push N of an already-reviewed PR does not fail the check — the guard
  catches never-reviewed PRs, not staleness.
- **Fail-open on API errors.** Any transient GitHub API failure warns and
  passes; the guard reds only on *confirmed* zero coverage. A safety net
  must not be a new flake surface.
- **Session-side reviews count** — the guard guarantees *a* review
  happened, not that the bot did it.
- **Doc-only PRs are NOT carved out** — documentation gets adversarial
  review in this repo.
- **Escape hatch** for a false red (e.g. the gate judged a ≥20-line PR
  trivial but the transcript heuristic disagreed): re-run the job (runs
  are stochastic), or post a review and re-run the check.

Related tripwire: `ci.yml`'s `workflow-guard` job asserts the
claude-code-review workflow keeps its load-bearing pieces (write perms,
`--comment` on the invocation line, `--allowedTools`, claude.yml's sender
gate) — the anti-regression net for classes 1–4.
