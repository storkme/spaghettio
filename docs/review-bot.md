# CI review bot — how it works, how it fails, how to diagnose it

Reference doc for the automated PR review pipeline. `CLAUDE.md` carries only
the operating rules; this file is the canonical home for the failure-class
history and the forensics playbook. Keep it current when the workflow
changes.

## Moving parts

- [`.github/workflows/claude-code-review.yml`](../.github/workflows/claude-code-review.yml)
  — runs on every PR event (opened / synchronize / ready_for_review /
  reopened / edited): checkout → `anthropics/claude-code-action@v1` running
  the `code-review` plugin → transcript artifact upload → silent-no-op guard.
  Runs are debounced per PR (`concurrency` + `cancel-in-progress`): a review
  costs 8–11 minutes, and a run against a superseded head is spend the guard
  ignores anyway, since it enforces coverage on the *current* head.
  `edited` runs are the description-only re-check (class 7 below) — they take
  the cheap path in the prompt and are carved out of the guard, because a body
  edit produces no new SHA whose coverage could be at stake.
- **The plugin** (`code-review@claude-code-plugins`, from the
  anthropics/claude-code marketplace) is a multi-subagent orchestration:
  a haiku gate (skip closed / draft / trivial / already-reviewed PRs) → a
  CLAUDE.md-paths agent → a sonnet diff-summary agent → 4 parallel reviewer
  agents (2× CLAUDE.md compliance, 2× opus bug-hunting) → per-finding
  validation subagents → post. With `--comment` (which we pass —
  load-bearing) it posts inline comments on findings or a
  "No issues found" summary comment when clean.
### Failure class 6: reviewed once, merged much later (2026-07-29)

`claude-review` fires on every push, but two independent mechanisms conspire
to make only the FIRST push get a review:

- the plugin's own gate declines to re-review a PR Claude has already
  commented on;
- the guard below checks PR-**lifetime** coverage, so once any review exists
  every later push passes trivially.

Observed on #481: the bot reviewed the PR and found four real bugs, then six
further commits — including the fixes for those four bugs and a substantial
power-network change — landed and merged with no review of any of them. The
check was green throughout, correctly by its own rules.

The prompt now carries an explicit re-review rule: "already reviewed" is not a
valid skip when new commits have landed, and on a `synchronize` event the run
is told its `before..after` range and asked to name that range in its comment.
Only the triviality gate justifies a skip, and it still requires the one-line
notice.

The guard now requires coverage on the **current head SHA**, so a skipped
follow-up push reds the check instead of reporting a review that did not
happen. Two carve-outs keep that from being noisy: a push changing fewer than
20 lines is triviality-gate territory (the whole-PR rule, applied to the
delta), and every existing skip — drafts, workflow-file PRs, the conscious
gate-skip signature, fail-open on API errors — is unchanged.

**Attribution has one trap worth knowing.** A review's `commit_id` is pinned to
the SHA it reviewed, but an inline comment's `commit_id` *advances* as the PR
moves so the comment keeps tracking its line. Keying on it would attribute a
six-pushes-old comment to the current head and make this guard silently
useless — verified against #481, where 5 inline comments reported `commit_id`
== head while every one was written six pushes earlier. Use
`original_commit_id`. Bot summary comments carry no SHA and are attributed by
time, deliberately the generous direction: that heuristic can admit coverage,
never deny it.

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

Classes 1–6 share one symptom — green check, nothing posted — and each is
individually sufficient to produce it. Classes 7–8, below the table, are a
different family: the review runs and posts, but its findings can't be closed
or don't land.

| # | Cause | Symptom signature | Fixed |
|---|-------|-------------------|-------|
| 1 | Stock template ships `pull-requests: read` — every post discarded | ~20s run, 1 denial, "No buffered inline comments" | #327 |
| 2 | Plugin's `--comment` flag not passed — its own contract is "do not post" without it | review runs fully, prints to action log, posts nothing | #329 |
| 3 | No harness `--allowedTools` — every posting/diff call denied | 11 denials on the #330 canary | #331 |
| 4 | Re-running `/install-github-app` overwrote both workflow files with the stock template — wiped fixes 1–3 at once (plus `claude.yml`'s owner-only sender gate) | all of the above, after a period of working fine | #369 (canary #368) |
| 5 | Shape-sensitive denial starvation — the allowlist admits plain `gh pr/issue` commands but denies improvised *shapes* (env prefixes `GH_PAGER= gh …`, command substitution `$(…)`, `cd … &&` chains, `gh api`); enough denials and the orchestrator abandons mid-review without posting. Stochastic per run, worse on large PRs (more context wanted → more improvised commands). Diagnosed 2026-07-24 on PR #389 (two consecutive silent no-ops; PR #405 succeeded through 31 denials the same day) | mid-cost, mid-duration run: more than a gate-skip, far less than a full review; see signature table below | prompt hardening + guard step (introduced with this doc) |
| 6 | Async-wait abandonment — the plugin orchestrator fans out its reviewer subagents, then parks via `ScheduleWakeup` to "wait" for them; in a one-shot headless run the wakeup never fires and the session ends mid-wait. First caught live by the guard 2026-07-24 on PR #416 — transcript artifact showed 26 tool calls, near-zero denials, 4 parallel reviewers spawned, `ScheduleWakeup(180s)` then end at 10 turns/$1.08 with nothing posted. NOT a denial problem: the class-5 prompt note was propagating into every subagent and reads worked fine | mid-cost short run like class 5, but transcript shows `ScheduleWakeup` + spawned agents with unconsumed results | prompt note extended: one-shot/headless, never park, consume subagent results synchronously |

### Failure class 7: findings against the PR body can never be closed (2026-07-29)

The bot files real findings about the PR *description* — a missing
`## Models / contracts touched` section, a body claim the diff contradicts
(#494's description repeated a census error that had already been fixed in the
shipped doc). Fixing one means editing the description, which pushes no commit,
so no `synchronize` fires, no re-review runs, and nothing ever records the fix.
Worse than inert: on #493 the next round's summary reported the template
finding as "already on record and unchanged" — by then it had been fixed.

The workflow now takes `edited` events. Because that also fires on title and
base-branch edits, the job's `if` admits only body and base changes, and the
prompt splits them: a body edit gets a cheap description-only re-check that
re-reads the body, re-checks *only* body-scoped findings, and posts one short
comment saying which are resolved and which still stand; a base change gets a
full review, because it silently rewrites the diff.

### Failure class 8: repeat findings restated at constant volume (2026-07-29)

Not a silence — the opposite. The template finding on #493 was raised in three
consecutive rounds, each time in identical non-inline form at the bottom of a
summary. The bot tracked the repetition and said so, but a third airing read
exactly like a first and the check passed either way. The author fixed every
inline finding across all three rounds and stepped over the repeated one twice.

A finding that survived a round of attention is *stronger* evidence than a
fresh one, and the prompt now says so: unaddressed findings from prior rounds
lead the comment on a `**Unaddressed from N prior round(s):**` line, ahead of
anything new.

A related, smaller correction in the same change: claims about mutable tracker
state get scoped to when the reviewer looked. The bot correctly reasoned "#490
is open, so #7 isn't fixed" — #491 merged an hour later and inverted the
remedy. Such claims are now written "as of this review, …" with a note on what
would change the conclusion.

Validation history: planted-bug canary #330 (2026-07-21) — first-ever bot
comment correctly flagged the bug inline with a committable fix; canary
#368 re-validated after the installer overwrite was restored in #369.

## Merge-time gating (the review is advisory)

**`main` has no branch protection.** `claude-review` is therefore a check
nobody has to wait for: a PR sitting at `mergeStateStatus=UNSTABLE` with the
review still running merges cleanly. The guard makes a *finished* run's silence
loud; it can do nothing about a run that never finished.

That came within one wrong comparison of mattering on 2026-07-29. A session
polled merge readiness with a hand-rolled loop testing
`mergeStateStatus != "PENDING"`. GitHub reports a running check as
`IN_PROGRESS` and the PR as `UNSTABLE`, so the loop exited immediately every
time and reported "checks done" while an 11-minute review was still going.
#494 would have merged with zero review coverage and been reported as reviewed.

Two independent mitigations, only one of them applied:

- **Applied** — [`scripts/review-gate.sh`](../scripts/review-gate.sh)
  `wait <pr>` blocks until the check reaches `status == "COMPLETED"` and exits
  non-zero on any conclusion that isn't a pass. Use it rather than hand-rolling
  a poll; the trap is that the obvious-looking sentinel values (`PENDING`,
  `""`) are not states this API ever reports for a running check, so testing
  for their absence always succeeds.
- **Not applied, needs a human call** — `scripts/review-gate.sh require` makes
  `claude-review` a required status check on `main`. This is the change that
  actually makes a merge wait. It is deliberately manual: the repo has *no*
  protection today, so this introduces it, and to bind at all it needs
  `enforce_admins: true` (with it false, every session using the owner's token
  bypasses it and the gate is theatre). That form also blocks direct pushes to
  `main` for everyone, and if the review cannot run at all — rotated secret,
  action outage — nothing merges until protection is loosened. Real cost, real
  benefit; `unrequire` reverses it in one command.

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
  freshness is owned by the `agent-reviewed` label flow, not the guard:
  `clear-agent-reviewed.yml` drops the label on new commits, and the
  containerised watcher agent re-reviews PRs missing it
  (`scripts/agent-reviewer.sh`; see
  [`docs/agent-container.md`](agent-container.md#pr-review-queue)) — a
  separate reviewer from this workflow's plugin. Caveat: the watcher is
  an on-demand container, so while it isn't running, per-SHA staleness on
  already-reviewed PRs is an accepted gap. Consequence either way: a bot
  no-op on push N of an already-reviewed PR does not fail the check — the
  guard catches never-reviewed PRs, not staleness.
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
- **Known guard false-positive mode** (observed 2026-07-24, PR #419):
  a conscious skip that reads the full diff first can take ~20 turns —
  indistinguishable from class-5/6 abandonment by the turns heuristic.
  Mitigated the same day: the prompt now requires a one-line skip
  comment for every conscious no-review decision, so conscious skips
  produce coverage signal and the turns heuristic is a fallback only.
  A red on a silent ≥8-turn run after that prompt change is
  presumptively real.

Related tripwire: `ci.yml`'s `workflow-guard` job asserts the
claude-code-review workflow keeps its load-bearing pieces (write perms,
`--comment` on the invocation line, `--allowedTools`, claude.yml's sender
gate) — the anti-regression net for classes 1–4.
