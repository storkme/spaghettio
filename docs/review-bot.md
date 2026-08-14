# CI review bot — how it works, how it fails, how to diagnose it

> **Status (2026-08-01): claude-code-review.yml is PARKED** (trigger swapped
> to `workflow_dispatch`-only; the hardened job is kept intact in the file).
> CI review duty moved to
> [`.github/workflows/second-opinion.yml`](../.github/workflows/second-opinion.yml)
> — the [storkme/second-opinion](https://github.com/storkme/second-opinion)
> action driving `deepseek/deepseek-v4-flash-0731` via OpenRouter, K=3
> unioned agentic passes — because CI reviews shared the Claude
> subscription's usage windows with interactive sessions. Branch
> protection's required context moved `claude-review` → `second-opinion`
> accordingly (`scripts/review-gate.sh`). Everything below is the
> claude-review pipeline's history: still the playbook if it is ever
> re-enabled, and ci.yml's workflow-guard still asserts the parked file's
> load-bearing pieces (the stock-template overwrite trap, #367, does not
> care whether the file it clobbers is live). The second-opinion runner has
> its own silent-failure tripwire: a degraded pass that posts no review
> exits non-zero and reds the check (`fail-on-degraded`, see the action's
> README) — the guard scripting below is claude-review-specific and does
> not apply to it.
>
> Two decisions recorded here because no RFC owns this pipeline:
> **(1)** Making `second-opinion` a *required* check deliberately overrides
> the upstream README's own advice ("advisory, never a merge gate") —
> continuing the claude-review precedent (required since 2026-07-29), with
> eyes open that K=3 sequential OpenRouter passes widen the
> single-point-of-failure surface of an admin-binding block; the escape
> hatch is `scripts/review-gate.sh unrequire`.
> **(2)** `claude-review-auto-retry.yml` is dormant, not removed: its
> `workflow_run` condition can never fire while claude-code-review.yml is
> dispatch-only, it fails its own `if` cleanly, and ci.yml's workflow-guard
> asserts the file's existence — removing it means editing the guard in the
> same PR that parks what it guards. Separable cleanup, do it later.

Reference doc for the automated PR review pipeline. `CLAUDE.md` carries only
the operating rules; this file is the canonical home for the failure-class
history and the forensics playbook. Keep it current when the workflow
changes.

## Moving parts

- [`.github/workflows/claude-code-review.yml`](../.github/workflows/claude-code-review.yml)
  — parked 2026-08-01 (see the status banner above); when live, ran on every
  PR event (opened / synchronize / ready_for_review /
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

## Trivial-delta gate (#632 C8, 2026-08-14)

A workflow-level step in `second-opinion.yml` (before the action step)
skips the K=3 review when the delta since the last **actually reviewed**
head — the newest `<!-- second-opinion sha=... -->` marker **whose body
also carries the review header** (`### 🤖 Second opinion`; the marker
alone is forgeable and claude[bot] posts byte-identical ones naming
different commits — storkme/second-opinion#49) — is docs-only (`*.md`)
and/or comment-only Rust. Rationale: this repo's
conventions generate doc-only pushes constantly (decision-log commits,
attribution sweeps), and each bought a full re-review of the whole PR
diff that could only re-find already-adjudicated items (4 of PR #630's
10 rounds).

Semantics that matter for forensics:

- **A skip is never silent.** It posts a comment with the DISTINCT
  marker `<!-- second-opinion-skip sha=... -->`. If a head has neither
  marker and the check is green, that's the pre-existing
  reviewer-malfunction class, not a gate skip.
- **Trivial deltas accumulate.** Skips don't advance the baseline; the
  first push whose cumulative delta since the last real review touches
  code gets a full-PR-diff review.
- **Everything ambiguous fails toward review**: no prior marker,
  force-push/diverged history, renames, any non-md/rs file, missing,
  oversized (≥10k chars, truncation insurance) or truncated compare
  patches, API errors. Known over-skips, both accepted by design: an
  `.rs` string literal whose changed line starts with `//` inside the
  quotes (none committed today), and `.md` *content* — the md-only skip
  is the feature, so programmatic content embedded in docs (commands,
  JSON, CI snippets) is outside the gate's review scope on skipped
  pushes.
- **Escape hatch**: the `force-review` label disables the gate for the
  PR — and applying it is itself a trigger (`labeled` event), so it
  works even when there is nothing left to push. The gate's decision
  and reason are one log line in the "Trivial-delta gate" step
  (`gate: trivial=... (reason)`).
- **Base retargets force a review.** The compare sees only head-side
  commits, and a retarget rewrites the effective diff with no new head
  SHA (the workflow's known `edited`-trigger gap) — pre-gate, the next
  push healed that with a full review, and the gate must not turn that
  push into a skip. Any `base_ref_changed` /
  `automatic_base_change_succeeded` timeline event newer than the
  anchor review forces a review; so does any failure to determine this.
- **Same-head re-events** (reopened / ready_for_review / stray label)
  on the already-reviewed head skip with NO notice — nothing to review,
  nothing new to say, and no container spin.
- Fork PRs bypass the gate entirely (kept on today's no-review,
  session-side-rule path) — which means the `force-review` label is
  also inert on forks; their review remains session-side by rule.
- **Tests**: `scripts/test_second_opinion_gate.py` extracts the gate's
  bash and all three jq programs VERBATIM from the workflow, drives the
  jq through fixtures AND the full bash orchestration end-to-end under
  a fake `gh` (label handling, anchor parsing, retarget-margin date
  arithmetic, verdict dispatch). Not CI-wired; run it when touching the
  gate.

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
| 6 | Async-wait abandonment — the plugin orchestrator fans out its reviewer subagents, then parks via `ScheduleWakeup` to "wait" for them; in a one-shot headless run the wakeup never fires and the session ends mid-wait. First caught live by the guard 2026-07-24 on PR #416 — transcript artifact showed 26 tool calls, near-zero denials, 4 parallel reviewers spawned, `ScheduleWakeup(180s)` then end at 10 turns/$1.08 with nothing posted. NOT a denial problem: the class-5 prompt note was propagating into every subagent and reads worked fine | mid-cost short run like class 5, but transcript shows `ScheduleWakeup` + spawned agents with unconsumed results | prompt note extended: one-shot/headless, never park, consume subagent results synchronously — **insufficient; recurred 2026-07-30 on #511, see below** |

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

## Merge-time gating (the review is REQUIRED)

**`claude-review` is a required status check on `main`** — enabled 2026-07-29
with `enforce_admins: true`, so it binds rather than being bypassable by any
session holding the owner's token. Verified binding on #473, which went from
mergeable to `BLOCKED` on a failed review. Direct pushes to `main` are blocked
for everyone; `scripts/review-gate.sh unrequire` reverses the whole thing.

Before that it was advisory, and the guard could only make a *finished* run's
silence loud — never a run that never finished. The history below is kept
because the reasoning is what justifies the cost.

### The never-skip invariant (#497)

Once the check is required, **the `claude-review` job must never be skipped**,
and must not acquire a job-level `if:`. A job skipped that way still publishes
a check run named `claude-review` with conclusion `skipped`, against a head SHA
that may already carry a passing one — and required-check evaluation reads the
latest. Whether GitHub counts `skipped` as a pass is undocumented enough that it
is not worth betting a merge on.

So the job always runs and always reports; every condition lives on a *step*:

- the `edited` gate (title edits have nothing to review, body edits get the
  cheap description-only re-check, base changes get a full review), and
- the fork gate — `pull_request` withholds secrets from forks by design, so the
  action would fail on an empty token. Advisory, that was noise. Required, it
  is an unclearable merge block on an outside contributor.

The guard carves out both cases too: a fork PR's review provably never started,
so guard silence there cannot be hiding an abandonment. Fork PRs and
workflow-file PRs both fall back to session-side review (CLAUDE.md "Workflow").

Cost of the invariant is a runner spin-up (seconds) on events with nothing to
review. That is the price of a check that cannot report `skipped`.

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
- **Applied 2026-07-29** — `scripts/review-gate.sh require` made
  `claude-review` a required status check on `main`. This is the change that
  actually makes a merge wait. It stays a manual command rather than something
  a session runs: it introduces protection where there was none, and to bind at
  all it needs `enforce_admins: true` (with it false, every session using the
  owner's token bypasses it and the gate is theatre). That form also blocks
  direct pushes to `main` for everyone, and if the review cannot run at all —
  rotated secret, action outage — nothing merges until protection is loosened.
  `unrequire` reverses it in one command. See the never-skip invariant above:
  requiring the check constrains how this workflow may be conditioned.

### Failure class 9: cheap denial-abandonment passed as a gate-skip (2026-07-30)

The guard used to pass any zero-coverage run with `num_turns <= 8`, reasoning
that a conscious plugin gate-skip is cheap while abandonment is expensive (the
class-5 signature: 16-20 turns, then nothing).

#500 falsified it. A 1000-line layout-engine PR got a **3-turn** run with **one
permission denial**, posted nothing, and green-checked:

```
coverage on 48e7ee6b...: reviews=0 inline=0 bot_summaries=0
num_turns=3   permission_denials_count=1
```

Turn count measures cost, not intent, so it cannot separate "declined on
purpose" from "hit a denial and gave up". Worse, the carve-out was
self-defeating in theory too: the prompt REQUIRES a one-line `gh pr comment` on
any conscious skip, and that comment is itself coverage — so a compliant skip
never reaches the carve-out. Everything that got there had already violated the
posting contract.

Zero coverage on a substantive PR now fails unconditionally. The error reports
both turn count and denial count, since a low turn count with a non-zero denial
count is the denial-abandonment fingerprint and points at a refused command
shape in the transcript artifact.

Note what made this one visible: the check was green and every other signal
agreed, so the only clue was the review taking **1m8s** where real reviews on
this repo take 3-11 minutes. A green check plus an implausibly fast run is worth
opening even when nothing else complains.

### Failure class 6 recurred with a new parking mechanism (2026-07-30)

Class 6's mitigation was a prompt note telling the orchestrator never to park
and to consume subagent results synchronously. On PR #511 it parked anyway,
**without using `ScheduleWakeup`** — it simply ended its turn with a statement
of intent:

> Two background agents are now checking PR eligibility and gathering
> CLAUDE.md file paths. I'll wait for their results before proceeding — no
> further action needed from me until they complete.

`num_turns: 6`, `subtype: success`, `$0.23`, `reviews=0 inline=0
bot_summaries=0`. The guard caught it; the check failed correctly.

Two details make this worth its own entry rather than a tally mark on the row
above.

**The results it was waiting for had already arrived.** The transcript
artifact contains both subagents' outputs — one answering `SHOULD_STOP: no`
with a correct eligibility rationale, the other listing the two relevant
`CLAUDE.md` paths. Nothing was pending. The orchestrator ended a turn holding
the answers it claimed to be waiting for, so a fix that only makes waiting
work would not have helped here.

**The mitigation is keyed to the wrong thing.** "Never park" was written
against `ScheduleWakeup`, a specific tool call the guard can look for. Ending
a turn is not a tool call, so no amount of tightening the note about parking
tools covers it. Any prompt-level fix has to be phrased as an obligation to
*post before ending* rather than a prohibition on a parking mechanism —
mechanisms are open-ended, the posting contract is not.

Also in the same transcript, before the park: the orchestrator called `Agent`
with `subagent_type: "claude-haiku-4-5-20251001"` — a model ID in the agent-type
slot — then self-corrected to `subagent_type: "claude", model: "haiku"`. Costs
turns, not fatal, but it is the same run.

The first attempt on this PR burned **28 turns** and also posted nothing;
a plain re-run produced the 6-turn park above. So this is not stochastic in
the way class 5 is — re-running is not the remedy, and two consecutive silent
no-ops on one head is the signal to stop re-running and pull the artifact.

### Failure class 10: a description-edit run cancelled the code run (2026-07-31)

The first genuinely *structural* green-with-no-review: nothing crashed, nothing
was denied, no agent gave up. Two runs raced and the wrong one survived.

On PR #521, at head `269dc3e3`:

| run | event | outcome |
|---|---|---|
| `30624281369` | code review | **cancelled** 38s in |
| `30624312907` | `edited` (body) | **success**, 1m4s, `reviews=0 inline=0 bot_summaries=0` |

`claude-review` went green with zero review activity, and the guard agreed it
should.

**Both halves were behaving as designed.** The concurrency group
(`claude-review-<pr>`, `cancel-in-progress: true`) exists so a burst of pushes
does not leave several runs reviewing superseded heads — its comment justified
cancelling on the grounds that "the surviving run reviews the only SHA that can
be merged." The `edited` handler exists so body-scoped findings can be confirmed
closed without an 11-minute re-review, and its guard branch skips head-SHA
enforcement because "coverage is enforced by the push events."

Composed, they contradict: an `edited` run deliberately does **not** review the
diff, so when it wins the cancellation race the survivor reviews nothing — and
then defers coverage to the push event it just killed. Each comment was true in
isolation and the pair was false.

**Sequence that triggers it**, and it is an ordinary one: push a branch, open a
PR, then edit the PR body within the review's 8-11 minute window. Anyone who
opens a PR and immediately corrects a number in the description hits it.

**Two fixes, deliberately redundant:**

1. The concurrency group is keyed by event class —
   `claude-review-<pr>-<body|code>` — so body edits and code runs are in
   separate groups and cannot cancel each other.
2. The guard's `edited` branch no longer takes the deferral on trust. Deferring
   is only honest if a push run left something behind, so a body edit on a PR
   with **zero** review artifacts anywhere in its lifetime now fails. Lifetime,
   not head-SHA: the point is to catch "never reviewed at all" without demanding
   that a body edit re-review code.

Fix 1 prevents this specific race; fix 2 catches any future route to the same
end state, since the failure is "the guard passed a PR nothing ever reviewed"
rather than "these two events raced".

**The generalisable lesson is about the comments, not the code.** Both carve-outs
carried a written justification, and each justification quantified over the other
mechanism's behaviour without naming it — "the surviving run reviews" assumed
every survivor reviews; "coverage is enforced by the push events" assumed the
push events still exist. When a carve-out's rationale depends on another
mechanism's behaviour, name that mechanism in the comment, because that is what
makes the dependency visible when the other one changes.

Also caught in the same log: the review prompt still told the agent that "the
guard below only checks that the PR has SOME review in its lifetime", which
stopped being true on 2026-07-29 when the guard moved to head-SHA coverage
(class 6's fix). That understated the consequence of a skip to the one reader
whose behaviour it was trying to change. Corrected in the same PR.

### Failure class 6 recurred a THIRD time: an async subagent launch (2026-07-31)

Same head as class 10, one run later. PR #521, run `30624630414`:
`num_turns: 4`, `subtype: success`, `permission_denials_count: 0`, $0.19, 23.7s,
`reviews=0 inline=0 bot_summaries=0`. The guard caught it; the check failed
correctly.

The transcript, in order:

```
[4]  Agent(...)   -> "Async agent launched successfully… working in the background"
[9]  "I'll wait for the eligibility check to complete before proceeding further."
[10] ToolSearch(select:SendMessage)
[14] gh pr view 521 --json state,isDraft,title,body,comments
[19] ScheduleWakeup({stop: true})
[26] "I've kicked off the eligibility check… I'll continue once it reports back."  -> ends
```

**The root cause is that `Agent` returned asynchronously.** The prompt told the
orchestrator to "launch subagents and consume their results synchronously within
this run", but the tool handed back a launch acknowledgement rather than a
result, and there is no wake-up path in a one-shot headless job — the completion
notification arrives after the runner is gone. Having no way to wait and no
instruction covering that case, it ended the turn.

Note it *did* call `ScheduleWakeup`, which the prompt banned outright — with
`stop: true`, so the ban was technically honoured while the behaviour it exists
to prevent happened anyway.

**This is the third distinct parking mechanism**, and the previous entry
predicted it in as many words: *"Any prompt-level fix has to be phrased as an
obligation to post before ending rather than a prohibition on a parking
mechanism — mechanisms are open-ended, the posting contract is not."* That advice
was recorded and then not acted on; the prompt kept the ban and left the posting
obligation **conditional** on "if you consciously decide NOT to review". This run
never decided not to review — it believed it was mid-review — so the obligation
did not bind. A conditional obligation is not a contract.

**Fixes:**

1. The posting contract is now **unconditional**: post a `gh pr comment` before
   the final turn ends, in every run and every outcome, including a partial
   review or an "I could not finish, here is how far I got". "If unsure, post."
2. The async-`Agent` trap is named explicitly, with the instruction to do the
   work inline via plain `gh pr view` / `gh pr diff`, and that an unarrived
   subagent result is a reason to finish and post rather than to wait.

The tally is what matters here: a prohibition on parking has now been defeated
three times, by a wakeup call, by prose, and by a tool's async return. Each fix
addressed the mechanism in front of it. Only the obligation generalises.

### Failure class 6: fourth recurrence, prompt mitigation exhausted, structural fix (2026-07-31)

The fourth occurrence (run `30650165439` attempt 1, PR #535) defeated even the
unconditional posting contract: two async `Agent` launches, a `ScheduleWakeup`,
then the run *cancelled its own wakeup* — reasoning that background
notifications would arrive on their own — and ended with zero activity. The
contract was in the run's own prompt and was stepped over, not missed. Filed
and investigated as [#538](https://github.com/storkme/spaghettio/issues/538);
two findings settled the prompt-vs-structure question:

1. The model wasn't disobeying — it was **misjudging its own state** (occurrence
   2 waited for results already in its transcript; occurrence 4 believed no
   wakeup was needed). No prompt phrasing corrects a confidently-held wrong
   belief about what is pending.
2. The #535 run artifact proved `Agent` (and the whole async family) is
   **reachable via `ToolSearch` despite being absent from `--allowedTools`** —
   no configuration had ever actually removed a parking capability.

**Structural fixes shipped (#538):**

1. `--disallowedTools` in `claude-code-review.yml` bans the non-`Agent` async
   family (`ScheduleWakeup`, `Task*`, `Cron*`, `RemoteTrigger`, `Workflow`,
   `PushNotification`) — none has a legitimate use in a one-shot review.
   `Agent` itself stays allowed: the plugin's eligibility gate dispatches
   through it; banning it needs a canary run first (tracked in #538's thread).
2. `claude-review-auto-retry.yml` — a `workflow_run`-triggered job that reruns
   failed review jobs automatically, capped at 3 attempts. Mechanism-agnostic:
   it keys off the guard's red, not off predicting the next parking tool, and
   automates the manual re-run that recovered all four occurrences. Benign
   skips exit 0 and never trigger it; a genuinely broken run still reds to a
   human at the cap.
3. `workflow-guard` (ci.yml) now also asserts `--disallowedTools` is present
   and the auto-retry workflow file exists, so a stock-template overwrite
   (class 4) cannot silently drop either.

If a fifth parking mechanism appears, expect the auto-retry to absorb it (the
rerun has recovered 4/4 so far); the signature to watch for is repeated
same-run attempts each failing the guard — that means the parking is
deterministic for that PR, and the canary-gated `Agent` ban becomes the next
lever.

### Failure class 11: same-SHA event cancelled the required check (2026-08-06)

The first second-opinion entry in this ledger, and the inverse of class 10:
there the wrong run survived a cancellation race; here **nothing** survived,
and the victim was a *required* check.

On PR #576, at its final head `0d92f93a` (earlier SHAs had green reviews;
this head had none yet), a close/reopen (times UTC):

| run | created | outcome |
|---|---|---|
| `31124347719` | 17:51:39 | **cancelled** 17:54:47 — annotation: "Canceling since a higher priority waiting request for second-opinion-576 exists" |
| `31124497214` attempt 1 | 17:54:41 | job cancelled 18:13:18 — "The job was not acquired by Runner of type hosted even after multiple attempts" |
| `31124497214` attempt 2 (`gh run rerun`) | 18:31:05 | job cancelled 20:03:00 — same runner-acquisition failure, after **92 min** queued |

Two distinct causes composed. The first cancellation is the concurrency
group: `second-opinion-<pr>` with an unconditional `cancel-in-progress: true`
keys on the PR **number**, so a `reopened` event for the *same head SHA*
cancelled the in-progress run — a cancellation that buys nothing (no newer
SHA exists) and costs the head its only chance at a concluded check. The
second and third are a platform runner-acquisition stall that day, not
concurrency — githubstatus.com carried Actions at **`major_outage`** through
that evening (22:46 UTC check), so this is a confirmed platform incident
rather than an inference from our own logs. But the group turned a transient
stall into a dead end, because under it *any* subsequent same-PR event could
keep killing recovery attempts.
End state: `second-opinion` (required, `enforce_admins`) had no successful
conclusion on the head and the PR was unmergeable without a protection
override.

**Fix:** `cancel-in-progress: ${{ github.event.action == 'synchronize' }}`.
Cancellation was added for exactly one case — a new push supersedes the SHA
under review, and reviewing a superseded SHA is billed spend — and
`synchronize` is the only trigger in that case. Same-head events (`reopened`,
`ready_for_review`) now queue behind the running review instead of killing
it; when the queued duplicate runs, the runner's per-SHA marker gate
(`already_reviewed`, checked before any model call, exit 0) makes it a green
no-op. Keying the *group* on head SHA was considered and rejected: it fixes
the same-SHA kill but isolates each push in its own group, so superseded
runs would never be cancelled — reintroducing the spend the cancellation
exists to stop.

**Residual, accepted:** GitHub's own near-simultaneous-queue race (two runs
entering one group at the same instant can cancel each other) still exists
for bursts of pushes, and a run cancelled mid-flight by a genuine
`synchronize` still leaves a `cancelled` conclusion on the *old* SHA — both
fine, since the new head gets its own run. The recovery path is what the fix
hardens: a `gh run rerun` can no longer be shot down by same-SHA event noise.

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
