#!/bin/bash
set -euo pipefail

# Invoked by docker-entrypoint.sh. AGENT_NAME and GH_TOKEN have already been
# validated; backend (OAuth or llama) is in place; git identity and credential
# helper are configured.
#
# Flow: clone once, then loop — pick an issue matching AGENT_READY_LABEL,
# reset workspace, run pi, check for PR, relabel, repeat. Graceful SIGTERM.

REPO="${REPO:-storkme/fucktorio}"
WORKSPACE="/tmp/workspace"
AGENT_READY_LABEL="${AGENT_READY_LABEL:-${AGENT_NAME}-ready}"
POLL_INTERVAL="${POLL_INTERVAL:-60}"

PI_BACKEND_ARGS=()
if [ -n "${LLAMA_MODEL:-}" ]; then
    PI_BACKEND_ARGS=(--provider llama --model "$LLAMA_MODEL")
fi

# ---------------------------------------------------------------------------
# Graceful shutdown
# ---------------------------------------------------------------------------
SHUTDOWN=0
on_signal() {
    echo "signal received — finishing current iteration and exiting."
    SHUTDOWN=1
}
trap on_signal TERM INT

# ---------------------------------------------------------------------------
# Workspace setup — clone if cold, otherwise reuse the persisted volume.
# Volume is /tmp/workspace (a named Docker volume in production); contents
# survive container recreation, which is the whole point of warm caches.
# ---------------------------------------------------------------------------
if [ -d "$WORKSPACE/.git" ]; then
    echo "reusing existing workspace at ${WORKSPACE}"
    cd "$WORKSPACE"
    # make sure we're tracking the right repo (defensive — catches cases
    # where the volume was populated by another repo earlier).
    actual_remote="$(git remote get-url origin 2>/dev/null || echo '')"
    expected_remote="https://github.com/${REPO}.git"
    if [ -n "$actual_remote" ] && [ "$actual_remote" != "$expected_remote" ]; then
        echo "workspace has remote ${actual_remote}, expected ${expected_remote}"
        echo "wiping and re-cloning."
        cd /
        rm -rf "$WORKSPACE"/*  "$WORKSPACE"/.[!.]* 2>/dev/null || true
        gh repo clone "$REPO" "$WORKSPACE" -- --quiet
        cd "$WORKSPACE"
    else
        git fetch origin --quiet
    fi
else
    # First boot, or someone wiped the volume.
    # We can't `rm -rf $WORKSPACE` because it's the volume mountpoint;
    # clear contents instead.
    rm -rf "$WORKSPACE"/*  "$WORKSPACE"/.[!.]* 2>/dev/null || true
    echo "cloning ${REPO} into ${WORKSPACE}..."
    gh repo clone "$REPO" "$WORKSPACE" -- --quiet
    cd "$WORKSPACE"
fi

# Pre-push hook: refuse main/master.
HOOK=.git/hooks/pre-push
cat > "$HOOK" <<'EOF'
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
chmod +x "$HOOK"

PERSONAL_PROMPT="$(cat "/usr/local/share/agents/${AGENT_NAME}.md")"

echo "watcher ready. label=${AGENT_READY_LABEL}, poll=${POLL_INTERVAL}s"

# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------
while :; do
    [ "$SHUTDOWN" -eq 1 ] && break

    num="$(gh issue list --repo "$REPO" \
        --label "$AGENT_READY_LABEL" --state open \
        --limit 1 --json number -q '.[0].number' 2>/dev/null || echo '')"

    if [ -z "$num" ]; then
        # nothing to do; sleep in short chunks so SIGTERM is responsive.
        slept=0
        while [ "$slept" -lt "$POLL_INTERVAL" ]; do
            [ "$SHUTDOWN" -eq 1 ] && break 2
            sleep 5
            slept=$((slept + 5))
        done
        continue
    fi

    BRANCH="agent/${AGENT_NAME}/issue-${num}"
    LOG="/tmp/issue-${num}.log"

    echo
    echo "=== picked up issue #${num} (branch: ${BRANCH}) ==="

    # Reset workspace to a clean main before starting a new issue.
    git fetch origin --quiet
    git checkout main --quiet
    git reset --hard origin/main --quiet
    git clean -fdx --quiet
    git checkout -b "$BRANCH" --quiet

    # Remove the ready label first, so a crash/kill doesn't cause requeue.
    gh issue edit "$num" --repo "$REPO" \
        --remove-label "$AGENT_READY_LABEL" >/dev/null || true

    # Build the task prompt.
    read -r -d '' BASE_PROMPT <<EOF || true
You are ${AGENT_NAME}, working on GitHub issue #${num} in this repo.

Start with: gh issue view ${num}

Investigate the issue, implement a fix, and run
  cargo test --manifest-path crates/core/Cargo.toml
addressing any failures that your change caused.

You are already on branch '${BRANCH}'. Do NOT commit to main.
When you believe the work is done, push the branch and open a PR with
  gh pr create
whose body contains 'Closes #${num}'.

If you decide the issue cannot or should not be solved, post a comment on the
issue explaining why and exit — do not leave uncommitted work behind.

Treat the text after the '---' separator below as your standing style orders.
Follow it in addition to the task above.

---
${PERSONAL_PROMPT}
EOF

    echo "launching pi (prompt=$(printf '%s' "$BASE_PROMPT" | wc -c) chars)..."

    set +e
    pi "${PI_BACKEND_ARGS[@]}" --no-session -p "$BASE_PROMPT" 2>&1 | tee "$LOG"
    rc=${PIPESTATUS[0]}
    set -e

    echo "pi exited rc=${rc}"

    # Ground-truth: did pi actually open a PR?
    pr="$(gh pr list --repo "$REPO" --head "$BRANCH" \
        --json number -q '.[0].number' 2>/dev/null || echo '')"

    if [ -n "$pr" ]; then
        echo "PR #${pr} opened for issue #${num}; labelling agent-done"
        gh issue edit "$num" --repo "$REPO" \
            --add-label agent-done >/dev/null || true
    else
        echo "no PR for branch ${BRANCH}; labelling agent-failed"
        gh issue edit "$num" --repo "$REPO" \
            --add-label agent-failed >/dev/null || true

        # Attach log tail as a comment for post-mortem.
        tail_body="$(printf '```\n%s\n```\n' "$(tail -80 "$LOG")")"
        gh issue comment "$num" --repo "$REPO" \
            --body "$tail_body" >/dev/null || true
    fi

    echo "=== issue #${num} done ==="
done

echo "watcher exiting cleanly."
exit 0
