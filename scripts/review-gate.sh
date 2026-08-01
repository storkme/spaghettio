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
#   scripts/review-gate.sh unrequire       # remove that protection
set -euo pipefail

REPO="${REPO:-storkme/spaghettio}"
CHECK="${CHECK:-second-opinion}"
TIMEOUT="${TIMEOUT:-1500}"   # 25 min; claude-review ran 8-11, second-opinion's
                             # K=3 flash passes should land well inside this.

usage() { sed -n '2,25p' "$0"; exit 2; }

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
#     pushes to main for everyone, and if claude-review cannot run at all
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

case "${1:-}" in
  wait)      shift; cmd_wait "$@" ;;
  status)    cmd_status ;;
  require)   cmd_require ;;
  unrequire) cmd_unrequire ;;
  *)         usage ;;
esac
