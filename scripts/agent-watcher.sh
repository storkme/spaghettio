#!/bin/bash
set -euo pipefail

# Invoked by docker-entrypoint.sh. AGENT_NAME and GH_TOKEN have already been
# validated; backend (OAuth or llama) is in place; git identity and credential
# helper are configured.
#
# Flow per iteration:
#   1. SCAN: find new human comments on touched issues/PRs, re-queue them by
#      adding <agent>-ready. Track last-seen timestamps in /var/lib/agent/state.json.
#   2. PICKUP: take the first <agent>-ready issue, check out a continuation-
#      aware branch, run pi, mark the result.
# Graceful SIGTERM between iterations.

REPO="${REPO:-storkme/fucktorio}"
WORKSPACE="/tmp/workspace"
AGENT_READY_LABEL="${AGENT_READY_LABEL:-${AGENT_NAME}-ready}"
POLL_INTERVAL="${POLL_INTERVAL:-60}"
STATE_FILE="/var/lib/agent/state.json"

# A marker on the first line of any comment body means "do not re-trigger the
# watcher on this comment". The watcher puts it on its own auto-comments, and
# the base prompt tells the agent to use it on any comment it writes.
NO_TRIGGER_SENTINEL='<!-- agent-no-trigger -->'

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
# Volume ownership fixes (root-owned on first mount)
# ---------------------------------------------------------------------------
sudo chown "$(id -u):$(id -g)" "$WORKSPACE" 2>/dev/null || true
sudo chown "$(id -u):$(id -g)" "$HOME/.cargo/registry" 2>/dev/null || true

# ---------------------------------------------------------------------------
# Workspace setup — clone if cold, otherwise reuse the persisted volume.
# ---------------------------------------------------------------------------
if [ -d "$WORKSPACE/.git" ]; then
    echo "reusing existing workspace at ${WORKSPACE}"
    cd "$WORKSPACE"
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

# ---------------------------------------------------------------------------
# State helpers
# ---------------------------------------------------------------------------
state_get() {
    local kind="$1" num="$2"
    jq -r --arg k "$kind" --arg n "$num" \
        '.[$k][$n] // "1970-01-01T00:00:00Z"' "$STATE_FILE"
}

state_set() {
    local kind="$1" num="$2" ts="$3"
    local tmp; tmp=$(mktemp)
    jq --arg k "$kind" --arg n "$num" --arg t "$ts" \
        '.[$k][$n] = $t' "$STATE_FILE" > "$tmp" && mv "$tmp" "$STATE_FILE"
}

# ---------------------------------------------------------------------------
# Scan phase
# ---------------------------------------------------------------------------
touched_issue_numbers() {
    # Any open-or-closed issue with any of our three labels. Three queries,
    # deduped (gh --label is AND, not OR).
    {
        gh issue list --repo "$REPO" --state all --label agent-done \
            --limit 100 --json number -q '.[].number' 2>/dev/null || true
        gh issue list --repo "$REPO" --state all --label agent-failed \
            --limit 100 --json number -q '.[].number' 2>/dev/null || true
        gh issue list --repo "$REPO" --state all --label "$AGENT_READY_LABEL" \
            --limit 100 --json number -q '.[].number' 2>/dev/null || true
    } | sort -un
}

pr_numbers_for_issue() {
    local num="$1"
    gh pr list --repo "$REPO" --state all --limit 50 \
        --head "agent/${AGENT_NAME}/issue-${num}" \
        --json number -q '.[].number' 2>/dev/null || true
}

# Returns the max createdAt of new human comments since $3, or empty if none.
new_comments_since() {
    local kind="$1" num="$2" last_seen="$3"
    local view_cmd
    if [ "$kind" = "issue" ]; then
        view_cmd=(gh issue view "$num" --repo "$REPO" --json comments)
    else
        view_cmd=(gh pr view "$num" --repo "$REPO" --json comments)
    fi
    "${view_cmd[@]}" 2>/dev/null | jq -r \
        --arg ls "$last_seen" \
        --arg sent "$NO_TRIGGER_SENTINEL" '
        [.comments[]
         | select(.createdAt > $ls)
         | select((.body // "") | startswith($sent) | not)
        ]
        | if length > 0 then (map(.createdAt) | max) else empty end
    '
}

requeue() {
    local num="$1"
    gh issue edit "$num" --repo "$REPO" \
        --add-label "$AGENT_READY_LABEL" >/dev/null 2>&1 || true
    # Reopen in case an earlier PR's merge auto-closed it.
    gh issue reopen "$num" --repo "$REPO" >/dev/null 2>&1 || true
}

scan_phase() {
    local issue_nums; issue_nums=$(touched_issue_numbers)
    [ -z "$issue_nums" ] && return 0

    while IFS= read -r num; do
        [ -z "$num" ] && continue

        # Issue comments
        local last; last=$(state_get issues "$num")
        local new_max; new_max=$(new_comments_since issue "$num" "$last")
        if [ -n "$new_max" ]; then
            echo "scan: new comment on issue #${num} (since ${last}); re-queuing."
            state_set issues "$num" "$new_max"
            requeue "$num"
        fi

        # PR comments (if any PR is tied to this issue's branch)
        local pr_nums; pr_nums=$(pr_numbers_for_issue "$num")
        [ -z "$pr_nums" ] && continue
        while IFS= read -r pr_num; do
            [ -z "$pr_num" ] && continue
            local pr_last; pr_last=$(state_get prs "$pr_num")
            local pr_new; pr_new=$(new_comments_since pr "$pr_num" "$pr_last")
            if [ -n "$pr_new" ]; then
                echo "scan: new comment on PR #${pr_num} (issue #${num}, since ${pr_last}); re-queuing issue."
                state_set prs "$pr_num" "$pr_new"
                requeue "$num"
            fi
        done <<< "$pr_nums"
    done <<< "$issue_nums"
}

# ---------------------------------------------------------------------------
# Continuation-aware branch checkout
# ---------------------------------------------------------------------------
checkout_task_branch() {
    local branch="$1"
    git fetch origin --quiet
    git checkout main --quiet
    git reset --hard origin/main --quiet
    git clean -fdx --quiet

    if git ls-remote --exit-code origin "refs/heads/${branch}" >/dev/null 2>&1; then
        echo "continuing existing branch ${branch}"
        git checkout -B "$branch" "origin/${branch}" --quiet
    else
        echo "fresh branch ${branch}"
        git checkout -b "$branch" --quiet
    fi
}

echo "watcher ready. label=${AGENT_READY_LABEL}, poll=${POLL_INTERVAL}s"

# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------
while :; do
    [ "$SHUTDOWN" -eq 1 ] && break

    scan_phase

    num="$(gh issue list --repo "$REPO" \
        --label "$AGENT_READY_LABEL" --state open \
        --limit 1 --json number -q '.[0].number' 2>/dev/null || echo '')"

    if [ -z "$num" ]; then
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

    checkout_task_branch "$BRANCH"

    # Remove ready label first so a crash/kill doesn't cause infinite requeue.
    gh issue edit "$num" --repo "$REPO" \
        --remove-label "$AGENT_READY_LABEL" >/dev/null || true

    read -r -d '' BASE_PROMPT <<EOF || true
You are ${AGENT_NAME}, working on GitHub issue #${num} in this repo.

Start with: gh issue view ${num}
(This includes the full comment thread. If there are new comments since your
last pass, the human is probably asking you to refine, correct, or extend
your previous work — read the thread carefully before deciding your
approach.)

Investigate the issue and decide what terminal state fits the task:

**Code-change tasks** — the issue asks for a file/function/feature change:
  1. You are on branch '${BRANCH}'. If this branch already has commits from
     a prior pass, you are *continuing* earlier work; add to it rather than
     starting over. Do NOT commit to main.
  2. Run \`cargo test --manifest-path crates/core/Cargo.toml\` and address
     failures that your change caused.
  3. Push and open a PR with \`gh pr create\` whose body contains
     'Closes #${num}'. If a PR on this branch is already open from an
     earlier pass, just push new commits — the existing PR picks them up.

**Research / audit / planning tasks** — the issue asks for sub-issues, an
investigation, or a list of findings:
  1. Do the investigation.
  2. File each concrete finding as its own sub-issue with
     \`gh issue create\`. Each sub-issue should be actionable on its own.
  3. Add label 'agent-done' to this issue yourself via
     \`gh issue edit ${num} --add-label agent-done\`.
  4. Comment on issue #${num} listing links to every sub-issue you filed.
  5. Exit. Do NOT open a PR. Do NOT commit planning documents to the repo
     unless the issue explicitly asks for a documentation file.

If the shape of the task is genuinely ambiguous, prefer issues over docs
(they are easier to iterate on than files committed to main).

If you decide the issue cannot or should not be solved at all, comment on
the issue explaining why and exit — do not leave uncommitted work behind.

**Important — comment hygiene.** Every comment you write on this issue or
any PR MUST start its body with the HTML comment
    ${NO_TRIGGER_SENTINEL}
on its own first line. This tells the watcher not to re-trigger on your own
comments; without it, every comment you post would re-queue the issue and
you would loop on yourself indefinitely.

Treat the text after the '---' separator below as your standing style
orders. Follow it in addition to the task above.

---
${PERSONAL_PROMPT}
EOF

    echo "launching pi (prompt=$(printf '%s' "$BASE_PROMPT" | wc -c) chars)..."

    set +e
    pi "${PI_BACKEND_ARGS[@]}" --mode json --no-session -p "$BASE_PROMPT" 2>&1 | tee "$LOG"
    rc=${PIPESTATUS[0]}
    set -e

    echo "pi exited rc=${rc}"

    # Ground-truth outcome:
    #   - if a PR on this branch exists → code-change success (agent-done)
    #   - if the agent self-labelled agent-done → research-task success
    #   - otherwise → agent-failed, attach log tail
    pr="$(gh pr list --repo "$REPO" --head "$BRANCH" \
        --json number -q '.[0].number' 2>/dev/null || echo '')"

    self_done="$(gh issue view "$num" --repo "$REPO" --json labels \
        -q '[.labels[] | select(.name == "agent-done")] | length' 2>/dev/null \
        || echo '0')"

    if [ -n "$pr" ] || [ "$self_done" -gt 0 ]; then
        if [ -n "$pr" ]; then
            echo "PR #${pr} present for issue #${num}; labelling agent-done"
        else
            echo "agent self-reported done on issue #${num}"
        fi
        gh issue edit "$num" --repo "$REPO" \
            --add-label agent-done >/dev/null 2>&1 || true
    else
        echo "no PR and no self-done for issue #${num}; labelling agent-failed"
        gh issue edit "$num" --repo "$REPO" \
            --add-label agent-failed >/dev/null 2>&1 || true

        tail_body="$(printf '%s\n\n```\n%s\n```\n' \
            "$NO_TRIGGER_SENTINEL" "$(tail -80 "$LOG")")"
        gh issue comment "$num" --repo "$REPO" \
            --body "$tail_body" >/dev/null 2>&1 || true
    fi

    echo "=== issue #${num} done ==="
done

echo "watcher exiting cleanly."
exit 0
