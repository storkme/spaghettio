#!/usr/bin/env bash
# review-gate.sh — wait for the CI review check correctly, and (optionally)
# make it a required status check on main.
#
# The check is `second-opinion` since 2026-08-01 (DeepSeek via OpenRouter —
# see .github/workflows/second-opinion.yml); it was `claude-review` before
# that, and the incident below is from that era. Override with CHECK=.
#
# Why this exists: on 2026-07-29 a session polled a PR's merge readiness with a
# hand-rolled loop testing `mergeStateStatus != "PENDING"`. GitHub reports a
# *running* check as `IN_PROGRESS`, and a PR whose only outstanding item is a
# running check as `UNSTABLE` — so the loop exited immediately, every time, and
# reported "checks done" while an 11-minute review was still going. Nothing in
# the repo stopped the merge: `main` has no branch protection, so `claude-review`
# is advisory. #494 came one wrong comparison away from merging with zero review
# coverage and being reported as reviewed.
#
# Rule of thumb: a check is finished when `.status == "COMPLETED"`. Never test
# for the absence of a value you guessed at.
#
# Usage:
#   scripts/review-gate.sh wait <pr>       # block until the review check completes
#   scripts/review-gate.sh status          # report main's protection state
#   scripts/review-gate.sh require         # make the review check required on main
#   scripts/review-gate.sh unrequire       # remove ALL protection (blunt)
#   scripts/review-gate.sh override <pr> "<reason>"
#                                          # merge ONE pr past the check, on
#                                          # the record, restoring
#                                          # enforce_admins immediately after
set -euo pipefail

REPO="${REPO:-storkme/spaghettio}"
CHECK="${CHECK:-second-opinion}"
TIMEOUT="${TIMEOUT:-3900}"   # 65 min: outlasts the second-opinion job's own
                             # 60-min timeout-minutes, so `wait` reports the
                             # check's real conclusion instead of a false
                             # timeout (#561 review finding). Typical runs
                             # are ~15-20 min; claude-review's were 8-11.

usage() { sed -n '2,29p' "$0"; exit 2; }

cmd_wait() {
  local pr="${1:?usage: review-gate.sh wait <pr>}"
  local deadline=$(( SECONDS + TIMEOUT ))
  while :; do
    # `// empty` so a check that has not been *created* yet is distinguishable
    # from one that is running — the former prints nothing and we keep waiting.
    local row status conclusion
    row=$(gh pr view "$pr" -R "$REPO" --json statusCheckRollup \
          --jq ".statusCheckRollup[] | select((.name // .context) == \"$CHECK\") | \"\(.status)\t\(.conclusion // \"\")\"" \
          | tail -1)
    status=${row%%$'\t'*}
    conclusion=${row#*$'\t'}

    if [ "$status" = "COMPLETED" ]; then
      echo "$CHECK: COMPLETED / ${conclusion:-unknown}"
      case "$conclusion" in
        SUCCESS|NEUTRAL|SKIPPED) return 0 ;;
        *) echo "  → not a pass; do not merge on this." >&2; return 1 ;;
      esac
    fi

    if [ "$SECONDS" -ge "$deadline" ]; then
      echo "timed out after ${TIMEOUT}s with $CHECK in state '${status:-absent}'" >&2
      return 1
    fi
    echo "  $CHECK: ${status:-not yet reported} — waiting…"
    sleep 20
  done
}

cmd_status() {
  if out=$(gh api "repos/$REPO/branches/main/protection" 2>/dev/null); then
    echo "main IS protected. Required checks:"
    echo "$out" | jq -r '.required_status_checks.contexts[]? | "  - \(.)"'
    echo "  enforce_admins: $(echo "$out" | jq -r '.enforce_admins.enabled')"
  else
    echo "main is NOT protected — every check on it, including $CHECK, is advisory."
  fi
}

# The trade-off, stated plainly because it is the whole reason this is a manual
# command and not something a session runs on its own:
#
#   enforce_admins=false — admins (i.e. every session using the owner's token)
#     bypass the requirement, so `gh pr merge` still merges through a pending
#     review. Blocks nobody here; effectively cosmetic for this repo.
#   enforce_admins=true  — the requirement actually binds. It also blocks direct
#     pushes to main for everyone, and if the review check cannot run at all
#     (secret rotated, action outage) nothing merges until protection is
#     loosened. That is the point, and it is a real operational cost.
#
# Defaulting to the binding form: the non-binding one would be theatre.
cmd_require() {
  local enforce="${ENFORCE_ADMINS:-true}"
  echo "Requiring '$CHECK' on $REPO@main (enforce_admins=$enforce)…"
  gh api -X PUT "repos/$REPO/branches/main/protection" --input - <<JSON
{
  "required_status_checks": { "strict": false, "contexts": ["$CHECK"] },
  "enforce_admins": $enforce,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
JSON
  cmd_status
}

cmd_unrequire() {
  gh api -X DELETE "repos/$REPO/branches/main/protection"
  echo "Protection removed from $REPO@main."
}

# Merge ONE pr past the required check, deliberately and on the record.
#
# Why this exists: the only override used to be `unrequire`, which DELETES all
# of main's protection — the required check, the force-push block and the
# deletion block together — and leaves it off until someone remembers to run
# `require`. That is far too blunt for "I have read this PR and I am choosing
# to merge it now", and its blast radius is every other PR and every concurrent
# session, not the one being merged.
#
# This flips ONLY `enforce_admins`, for the duration of one merge:
#   - the required check stays required for everyone else
#   - force-push and deletion protection never drop
#   - an EXIT trap restores it even if the merge fails or the script is killed
#
# A reason is mandatory and is posted to the PR BEFORE anything is touched, so
# the override is auditable afterwards instead of invisible. If the restore
# ever fails the script exits non-zero and says so loudly — an override that
# quietly leaves main bypassable would be worse than having no override.
cmd_override() {
  local pr="${1:-}" reason="${2:-}"
  # Validate BEFORE touching protection. Neither check was here originally,
  # and both were exploitable in ways that matter (PR #588 review):
  #   - `[ -n "$reason" ]` accepts a single space, so the mandatory-reason gate
  #     — the entire audit story — was bypassable by typing " ".
  #   - `pr` was never checked, so `override --help "x"` ran a full
  #     flip-restore cycle, posted NO audit comment (gh help exits 0), merged
  #     nothing, and reported `Merged #--help.` with exit 0. A protection
  #     window with no record and a success message is the worst outcome this
  #     script can produce.
  case "$pr" in
    ''|*[!0-9]*)
      echo "usage: review-gate.sh override <pr-number> \"<reason>\"" >&2
      echo "  <pr-number> must be numeric; got ${pr:-<empty>}" >&2
      exit 2
      ;;
  esac
  # Require an actual word character, not merely "not ASCII-blank". glibc's
  # [[:space:]] is ASCII-only, so a reason of a single NBSP (U+00A0) passed the
  # earlier strip and produced a visually-blank record (PR #588 re-review,
  # confirmed through a full stubbed cycle). Demanding [[:alnum:]] closes that
  # without trying to enumerate Unicode blanks.
  if ! printf '%s' "$reason" | grep -q '[[:alnum:]]'; then
    echo "override requires a reason containing at least one alphanumeric" >&2
    echo "character — it is posted to the PR as the record of why the required" >&2
    echo "check was bypassed, and a blank record is worse than none." >&2
    exit 2
  fi

  # Refuse unless protection is in the state we think we're overriding.
  #
  # If enforce_admins is ALREADY false, the DELETE below is a no-op and the
  # script would end by POSTing it true — silently *tightening* a state it did
  # not create, while the read-back cheerfully "verifies" a restoration to a
  # state that never held (PR #588 re-review). It is the safe direction, but
  # it is an out-of-mandate mutation, and it hides the thing worth noticing:
  # an override running while main was already loose.
  local enforced
  enforced=$(gh api "repos/$REPO/branches/main/protection" --jq '.enforce_admins.enabled' 2>/dev/null || echo "unknown")
  if [ "$enforced" != "true" ]; then
    echo "refusing: enforce_admins on $REPO@main reads '$enforced', not 'true'." >&2
    echo "  Nothing to override, and restoring would tighten a state this script" >&2
    echo "  did not set. Investigate why protection is already loose:" >&2
    echo "    scripts/review-gate.sh status" >&2
    exit 2
  fi

  # Refuse a PR that cannot be merged anyway. Without this, a closed/merged/
  # draft PR passes validation, gets a real ATTEMPTED comment (comments work
  # on closed PRs), opens the protection window, fails the merge, and closes
  # it again — a pointless bypass window, truthfully recorded but avoidable.
  local prstate isdraft
  prstate=$(gh pr view "$pr" -R "$REPO" --json state --jq .state 2>/dev/null || echo "")
  isdraft=$(gh pr view "$pr" -R "$REPO" --json isDraft --jq .isDraft 2>/dev/null || echo "")
  if [ "$prstate" != "OPEN" ] || [ "$isdraft" != "false" ]; then
    echo "refusing: #$pr is state='${prstate:-unknown}' isDraft='${isdraft:-unknown}'." >&2
    echo "  Only an open, non-draft PR can be merged; not opening a window for it." >&2
    exit 2
  fi

  ENFORCE_RESTORED=0
  restore_enforce() {
    [ "$ENFORCE_RESTORED" = 1 ] && return 0
    echo "Restoring enforce_admins on $REPO@main…"
    if gh api -X POST "repos/$REPO/branches/main/protection/enforce_admins" >/dev/null 2>&1; then
      local now
      now=$(gh api "repos/$REPO/branches/main/protection" --jq '.enforce_admins.enabled' 2>/dev/null)
      if [ "$now" = "true" ]; then
        ENFORCE_RESTORED=1
        echo "enforce_admins restored."
        return 0
      fi
      echo "!! enforce_admins reads '$now' after restore — CHECK MAIN NOW" >&2
      return 1
    fi
    echo "!! FAILED to restore enforce_admins — main is admin-bypassable until fixed." >&2
    echo "!! Run: gh api -X POST repos/$REPO/branches/main/protection/enforce_admins" >&2
    return 1
  }
  trap 'restore_enforce || true' EXIT

  # Posted BEFORE the flip, deliberately: it doubles as an existence check for
  # `$pr`, so a bad number fails here rather than after protection is down.
  #
  # But it must therefore describe an ATTEMPT, not a completed merge. The first
  # version said "Merged via override" up front, so a failed merge left a
  # permanent false "Merged" comment on a PR that never landed (PR #588
  # review). Success is confirmed by a follow-up comment below.
  echo "Recording the override attempt on #$pr…"
  gh pr comment "$pr" -R "$REPO" --body "**Override ATTEMPTED via \`review-gate.sh override\`** — merging past the required \`$CHECK\` check.

Reason: $reason

The check was not green. This is a deliberate call, not an accident — recorded
here so the bypass is visible to anyone reading this PR later. \`enforce_admins\`
is flipped for the duration of this merge only; the required check stays in
force for every other PR.

*If no \"override succeeded\" comment follows this one, the merge did **not**
happen — see the operator's terminal for why. If this comment is the last word
on the PR, the run may have been killed mid-override: check protection with
\`scripts/review-gate.sh status\` and restore \`enforce_admins\` if it reads
false.*" >/dev/null || {
    echo "!! could not comment on #$pr (does it exist?) — nothing touched." >&2
    trap - EXIT
    exit 1
  }

  echo "Disabling enforce_admins (the check stays required for everyone else)…"
  gh api -X DELETE "repos/$REPO/branches/main/protection/enforce_admins" >/dev/null

  echo "Merging #$pr…"
  if ! gh pr merge "$pr" -R "$REPO" --merge --admin; then
    echo "!! Merge of #$pr FAILED — restoring protection, PR left open." >&2
    restore_enforce || exit 1
    exit 1
  fi
  echo "Merged #$pr."

  # Restore FIRST, then report. The confirmation says "enforce_admins restored"
  # in the past tense, so posting it before the POST reintroduces exactly the
  # bug the ATTEMPTED wording fixed, one comment later: between the comment and
  # the restore, main is bypassable while the durable record says it is not —
  # and if the restore fails or the run is killed there, that false claim is
  # the PR's permanent last word (PR #588 re-review). The merge has already
  # happened by this point, so nothing is lost by waiting.
  if restore_enforce; then
    gh pr comment "$pr" -R "$REPO" \
      --body "**Override succeeded** — #$pr merged past \`$CHECK\`, and \`enforce_admins\` has been restored (verified by read-back)." \
      >/dev/null || echo "  (merged and restored, but the confirmation comment failed to post)"
  else
    gh pr comment "$pr" -R "$REPO" \
      --body "**Override merged, but RESTORE FAILED** — #$pr merged past \`$CHECK\`, and \`enforce_admins\` could **not** be restored. \`main\` is admin-bypassable right now. Run \`scripts/review-gate.sh status\` and re-enable it." \
      >/dev/null || echo "  (restore failed AND the warning comment failed to post — fix main by hand)"
    exit 1
  fi
}

case "${1:-}" in
  wait)      shift; cmd_wait "$@" ;;
  status)    cmd_status ;;
  require)   cmd_require ;;
  unrequire) cmd_unrequire ;;
  override)  shift; cmd_override "$@" ;;
  *)         usage ;;
esac
