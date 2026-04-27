#!/bin/bash
set -euo pipefail

# Invoked by docker-entrypoint.sh. AGENT_NAME and GH_TOKEN have already been
# validated; backend (OAuth or llama) is in place; git identity and credential
# helper are configured.
#
# One-shot PR review: fetch the PR head, run pi in read-only review mode,
# post a review comment, and apply the agent-reviewed label.

if [ -z "${PR_NUM:-}" ]; then
    echo "error: agent-reviewer.sh requires PR_NUM env var" >&2
    exit 64
fi

REPO="${REPO:-storkme/fucktorio}"
WORKSPACE="/tmp/workspace"
REVIEW_LABEL='agent-reviewed'
NO_TRIGGER_SENTINEL='<!-- agent-no-trigger -->'
LOCAL_BRANCH="pr-review/${PR_NUM}"
LOG="/tmp/pr-review-${PR_NUM}.log"

PI_BACKEND_ARGS=()
if [ -n "${LLAMA_MODEL:-}" ]; then
    PI_BACKEND_ARGS=(--provider llama --model "$LLAMA_MODEL")
fi

# ---------------------------------------------------------------------------
# Fresh clone
# ---------------------------------------------------------------------------
rm -rf "$WORKSPACE"
echo "cloning ${REPO} into ${WORKSPACE}..."
gh repo clone "$REPO" "$WORKSPACE" -- --quiet
cd "$WORKSPACE"

# ---------------------------------------------------------------------------
# Pre-push hooks
# ---------------------------------------------------------------------------
install_standard_pre_push_hook() {
    cat > .git/hooks/pre-push <<'EOF'
#!/bin/bash
while read -r _ _ remote_ref _; do
    case "$remote_ref" in
        refs/heads/main|refs/heads/master)
            echo "pre-push hook: refusing push to ${remote_ref#refs/heads/}" >&2
            exit 1
            ;;
    esac
done
exit 0
EOF
    chmod +x .git/hooks/pre-push
}

install_review_pre_push_hook() {
    cat > .git/hooks/pre-push <<'EOF'
#!/bin/bash
echo "pre-push hook: review mode — refusing all pushes" >&2
exit 1
EOF
    chmod +x .git/hooks/pre-push
}

install_standard_pre_push_hook

# ---------------------------------------------------------------------------
# Personality
# ---------------------------------------------------------------------------
PERSONAL_PROMPT="$(cat "/usr/local/share/agents/${AGENT_NAME}.md")"

# ---------------------------------------------------------------------------
# Fetch PR head
# ---------------------------------------------------------------------------
echo
echo "=== reviewing PR #${PR_NUM} ==="

git fetch origin --quiet
if ! git fetch origin "pull/${PR_NUM}/head:${LOCAL_BRANCH}" --quiet 2>/dev/null; then
    echo "could not fetch pull/${PR_NUM}/head — closed or gone?" >&2
    exit 1
fi
git checkout "$LOCAL_BRANCH" --quiet

install_review_pre_push_hook

# ---------------------------------------------------------------------------
# Review prompt + pi run
# ---------------------------------------------------------------------------
read -r -d '' REVIEW_PROMPT <<EOF || true
You are ${AGENT_NAME}, doing a code review of PR #${PR_NUM} in this repo.

The PR's head is checked out at branch '${LOCAL_BRANCH}' in this workspace
so you can read the changed files in their post-merge state.

Do NOT, under any circumstances:
  - Edit, create, or delete any file in the workspace
  - Commit anything
  - Push (the pre-push hook will refuse it anyway)
  - Merge, close, or otherwise mutate the PR
  - Apply suggestions yourself

Workflow:
  1. Run \`gh pr view ${PR_NUM}\` for description and existing comments.
     If a previous review of yours is already on this PR (your comments
     start with '${NO_TRIGGER_SENTINEL}'), the head SHA has changed since —
     read that prior comment and focus the new one on what changed,
     not the whole diff again.
  2. Run \`gh pr diff ${PR_NUM}\` for the diff. Read surrounding code in
     the workspace as needed to judge the change in context.
  3. Form an opinion. Substance over volume — flag real bugs, broken
     invariants, missing tests, unclear design. If the change looks good,
     say so plainly and stop. Don't invent nits.
  4. Post your review as a single PR comment with:
     \`gh pr comment ${PR_NUM} --body "<your review>"\`
     The body MUST start with '${NO_TRIGGER_SENTINEL}' on its own first
     line — without it, the watcher would re-trigger on your own comment.
  5. Add the '${REVIEW_LABEL}' label to the PR via:
     \`gh pr edit ${PR_NUM} --add-label ${REVIEW_LABEL}\`
  6. Exit.

Treat the text after the '---' separator below as your standing style
orders. Follow it in addition to the task above.

---
${PERSONAL_PROMPT}
EOF

echo "launching pi for review (prompt=$(printf '%s' "$REVIEW_PROMPT" | wc -c) chars)..."

set +e
pi "${PI_BACKEND_ARGS[@]}" --mode json --no-session -p "$REVIEW_PROMPT" > "$LOG" 2>&1
rc=$?
set -e

echo "pi exited rc=${rc}"

install_standard_pre_push_hook

# ---------------------------------------------------------------------------
# If the agent didn't self-label, label and attach a log tail anyway so the
# watcher doesn't re-review the same SHA on its next poll.
# ---------------------------------------------------------------------------
labelled="$(gh pr view "$PR_NUM" --repo "$REPO" --json labels \
    -q "[.labels[] | select(.name == \"${REVIEW_LABEL}\")] | length" \
    2>/dev/null || echo '0')"
if [ "$labelled" -eq 0 ]; then
    echo "agent did not self-label; applying ${REVIEW_LABEL} and attaching log tail."
    gh pr edit "$PR_NUM" --repo "$REPO" \
        --add-label "$REVIEW_LABEL" >/dev/null 2>&1 || true
    tail_body="$(printf '%s\n\nReview run did not produce a comment. Last 80 lines:\n\n```\n%s\n```\n' \
        "$NO_TRIGGER_SENTINEL" "$(tail -80 "$LOG")")"
    gh pr comment "$PR_NUM" --repo "$REPO" \
        --body "$tail_body" >/dev/null 2>&1 || true
fi

echo "=== PR #${PR_NUM} review done ==="
exit "$rc"
