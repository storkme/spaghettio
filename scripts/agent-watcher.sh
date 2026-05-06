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

REPO="${REPO:-storkme/spaghettio}"
WORKSPACE="/tmp/workspace"
MEM_DIR="/tmp/agent-memory"
MEM_BRANCH="agent-memory/${AGENT_NAME}"
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

# Pre-push hook: refuse main/master. Two variants — the standard one (issue
# pickups push branches and open PRs) and a stricter review-mode variant
# (PR review pickups must NOT push, ever). review_pr swaps in the strict hook
# while it runs and swaps back when done.
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

PERSONAL_PROMPT="$(cat "/usr/local/share/agents/${AGENT_NAME}.md")"

# ---------------------------------------------------------------------------
# Agent memory — a separate clone at $MEM_DIR tracking an orphan branch
# (agent-memory/<agent-name>) where the agent writes per-issue notes. The
# watcher clones/inits it at container start; commits and pushes after each
# pickup if the agent changed anything in /tmp/agent-memory/issue-<N>/.
# ---------------------------------------------------------------------------
setup_memory_dir() {
    if git -C "$WORKSPACE" ls-remote --exit-code origin \
        "refs/heads/${MEM_BRANCH}" >/dev/null 2>&1; then
        echo "using existing memory branch: ${MEM_BRANCH}"
        rm -rf "$MEM_DIR"
        git clone --depth 20 --single-branch --branch "$MEM_BRANCH" \
            "https://github.com/${REPO}.git" "$MEM_DIR" --quiet
    else
        echo "initializing new memory branch: ${MEM_BRANCH}"
        rm -rf "$MEM_DIR"
        mkdir -p "$MEM_DIR"
        cd "$MEM_DIR"
        git init --quiet --initial-branch="$MEM_BRANCH"
        git remote add origin "https://github.com/${REPO}.git"
        cat > README.md <<EOF
# Agent memory: ${AGENT_NAME}

This orphan branch is a durable store of ${AGENT_NAME}'s working memory
across issues. Each \`issue-<N>/\` subdirectory holds markdown notes the
agent wrote while working on issue #N. Never merged to main.

Inspect with:
\`\`\`
git fetch origin ${MEM_BRANCH}
git checkout ${MEM_BRANCH}
\`\`\`
EOF
        git add README.md
        git commit --quiet -m "init agent memory"
        git push --quiet origin "HEAD:${MEM_BRANCH}"
        cd "$WORKSPACE"
    fi
}

# Commit and push anything the agent changed under /tmp/agent-memory/issue-<N>/.
# No-op if nothing changed. Best-effort push — a single failure is logged but
# doesn't block the watcher.
commit_memory() {
    local num="$1"
    local issue_mem_dir="${MEM_DIR}/issue-${num}"
    if [ ! -d "$issue_mem_dir" ]; then
        return 0
    fi
    (
        cd "$MEM_DIR"
        git add "issue-${num}/" 2>/dev/null || true
        if ! git diff --cached --quiet 2>/dev/null; then
            git commit --quiet -m "memory: issue #${num} (${AGENT_NAME})"
            if ! git push --quiet origin "$MEM_BRANCH" 2>&1; then
                echo "warning: memory push failed; re-sync on next pickup"
                # Try to reconcile: fetch + rebase + push once more
                git fetch origin "$MEM_BRANCH" --quiet 2>/dev/null || true
                git rebase "origin/${MEM_BRANCH}" --quiet 2>/dev/null \
                    || git rebase --abort 2>/dev/null || true
                git push --quiet origin "$MEM_BRANCH" 2>/dev/null \
                    || echo "warning: memory still not pushed; will retry next pickup"
            else
                echo "memory committed + pushed for issue #${num}"
            fi
        fi
    )
}

setup_memory_dir

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
        # -B (not -b): create-or-reset. Tolerates a leftover local branch
        # from a previous run that committed but never pushed (e.g. a
        # failed task) or whose remote was deleted (e.g. PR merged with
        # GitHub auto-delete on). Either way, the right behaviour for
        # "no branch on remote" is: start fresh from main.
        echo "fresh branch ${branch}"
        git checkout -B "$branch" --quiet
    fi
}

# ---------------------------------------------------------------------------
# PR review pickup
# ---------------------------------------------------------------------------
# When the issue queue is empty, look for an open PR that hasn't been reviewed
# yet and run pi against it in review-only mode. PRs whose head branch starts
# with 'agent/${AGENT_NAME}/' are skipped — those are the agent's own PRs and
# reviewing them would be a waste at best and a self-affirmation loop at worst.
# Drafts are skipped too. The 'agent-reviewed' label is removed by a
# pull_request:synchronize workflow whenever new commits land, so the next
# poll re-reviews the new SHA naturally.
REVIEW_LABEL='agent-reviewed'

pick_pr_for_review() {
    gh pr list --repo "$REPO" --state open --limit 50 \
        --json number,headRefName,isDraft,labels \
        -q "[.[] | select(.isDraft == false)
                  | select([.labels[].name] | index(\"${REVIEW_LABEL}\") | not)
                  | select((.headRefName | startswith(\"agent/${AGENT_NAME}/\")) | not)
            ] | .[0].number" 2>/dev/null || true
}

review_pr() {
    local pr_num="$1"
    local log="/tmp/pr-review-${pr_num}.log"
    local local_branch="pr-review/${pr_num}"

    echo
    echo "=== reviewing PR #${pr_num} ==="

    git fetch origin --quiet
    git checkout main --quiet 2>/dev/null || true
    git reset --hard origin/main --quiet
    git clean -fdx --quiet
    if ! git fetch origin "pull/${pr_num}/head:${local_branch}" --quiet 2>/dev/null; then
        echo "could not fetch pull/${pr_num}/head; skipping (closed or gone?)."
        return 0
    fi
    git checkout "$local_branch" --quiet

    install_review_pre_push_hook

    read -r -d '' REVIEW_PROMPT <<EOF || true
You are ${AGENT_NAME}, doing a code review of PR #${pr_num} in this repo.

The PR's head is checked out at branch '${local_branch}' in this workspace
so you can read the changed files in their post-merge state.

Do NOT, under any circumstances:
  - Edit, create, or delete any file in the workspace
  - Commit anything
  - Push (the pre-push hook will refuse it anyway)
  - Merge, close, or otherwise mutate the PR
  - Apply suggestions yourself

Workflow:
  1. Run \`gh pr view ${pr_num}\` for description and existing comments.
     If a previous review of yours is already on this PR (your comments
     start with '${NO_TRIGGER_SENTINEL}'), the head SHA has changed since —
     read that prior comment and focus the new one on what changed,
     not the whole diff again.
  2. Run \`gh pr diff ${pr_num}\` for the diff. Read surrounding code in
     the workspace as needed to judge the change in context.
  3. Form an opinion. Substance over volume — flag real bugs, broken
     invariants, missing tests, unclear design. If the change looks good,
     say so plainly and stop. Don't invent nits.
  4. Post your review as a single PR comment with:
     \`gh pr comment ${pr_num} --body "<your review>"\`
     The body MUST start with '${NO_TRIGGER_SENTINEL}' on its own first
     line — without it, the watcher would re-trigger on your own comment.
  5. Add the '${REVIEW_LABEL}' label to the PR via:
     \`gh pr edit ${pr_num} --add-label ${REVIEW_LABEL}\`
  6. Exit.

Treat the text after the '---' separator below as your standing style
orders. Follow it in addition to the task above.

---
${PERSONAL_PROMPT}
EOF

    echo "launching pi for review (prompt=$(printf '%s' "$REVIEW_PROMPT" | wc -c) chars)..."

    set +e
    pi "${PI_BACKEND_ARGS[@]}" --mode json --no-session -p "$REVIEW_PROMPT" > "$log" 2>&1
    rc=$?
    set -e

    echo "pi exited rc=${rc}"

    install_standard_pre_push_hook

    # If the agent didn't self-label, label and attach a log tail so we
    # don't re-review the same SHA on the next poll. The synchronize
    # workflow will clear the label on the next push.
    local labelled
    labelled="$(gh pr view "$pr_num" --repo "$REPO" --json labels \
        -q "[.labels[] | select(.name == \"${REVIEW_LABEL}\")] | length" \
        2>/dev/null || echo '0')"
    if [ "$labelled" -eq 0 ]; then
        echo "agent did not self-label; applying ${REVIEW_LABEL} and attaching log tail."
        gh pr edit "$pr_num" --repo "$REPO" \
            --add-label "$REVIEW_LABEL" >/dev/null 2>&1 || true
        local tail_body
        tail_body="$(printf '%s\n\nReview run did not produce a comment. Last 80 lines:\n\n```\n%s\n```\n' \
            "$NO_TRIGGER_SENTINEL" "$(tail -80 "$log")")"
        gh pr comment "$pr_num" --repo "$REPO" \
            --body "$tail_body" >/dev/null 2>&1 || true
    fi

    git checkout main --quiet 2>/dev/null || true
    git branch -D "$local_branch" >/dev/null 2>&1 || true

    echo "=== PR #${pr_num} review done ==="
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
        # Issue queue is empty — try a PR review pickup before sleeping.
        pr_to_review="$(pick_pr_for_review)"
        if [ -n "$pr_to_review" ]; then
            review_pr "$pr_to_review"
            continue
        fi

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

    # Ensure the per-issue memory dir exists (possibly populated from prior passes).
    ISSUE_MEM_DIR="${MEM_DIR}/issue-${num}"
    mkdir -p "$ISSUE_MEM_DIR"

    read -r -d '' BASE_PROMPT <<EOF || true
You are ${AGENT_NAME}, working on GitHub issue #${num} in this repo.

Start with: gh issue view ${num}
(This includes the full comment thread. If there are new comments since your
last pass, the human is probably asking you to refine, correct, or extend
your previous work — read the thread carefully before deciding your
approach.)

**Working memory** for this issue lives at:
    ${ISSUE_MEM_DIR}

If that directory has files, they are notes from your past self — read them
BEFORE you do anything else. You may have partially understood the issue
before, tried approaches that didn't work, or formed a plan worth continuing.
When you finish this pass, update (or create) files in that directory so a
future pass picks up where you left off. Suggested files:
  understanding.md  — what the issue is actually asking, any constraints
  progress.md       — what has been attempted, what worked / didn't, next steps
Feel free to add more files if useful (notes.md, open-questions.md, etc).
The watcher commits and pushes this directory to a separate 'agent-memory'
branch after you exit — don't commit it yourself and don't put it in the
main workspace.

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

**Question / discussion tasks** — the issue (or a comment on it) asks a
question, requests an explanation, or invites your opinion without asking
for code changes or sub-issues:
  1. Read the full comment thread AND the relevant code to form a substantive
     answer. Prior comments may include earlier exchanges with you; account
     for them — your comments start with '${NO_TRIGGER_SENTINEL}' so you can
     recognise them.
  2. Post a comment on the issue with your answer, starting the body with
     '${NO_TRIGGER_SENTINEL}' on its own first line.
  3. Add label 'agent-done' via
     \`gh issue edit ${num} --add-label agent-done\`.
  4. Exit. Do NOT open a PR. Do NOT file sub-issues.

If the shape of the task is genuinely ambiguous, prefer issues over docs
(they are easier to iterate on than files committed to main). Questions
that accompany a code-change ask should get a reply comment AND the code
change — they're not mutually exclusive.

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

    # Output goes to the per-issue log only — keeps `docker logs` readable
    # (just the watcher's own status lines, not pi's JSON event stream).
    # The trace is still archived into the memory branch by commit_memory.
    set +e
    pi "${PI_BACKEND_ARGS[@]}" --mode json --no-session -p "$BASE_PROMPT" > "$LOG" 2>&1
    rc=$?
    set -e

    echo "pi exited rc=${rc}"

    # Archive the JSON event stream into the memory branch alongside the
    # agent's hand-written notes. gzip keeps branch growth manageable; the
    # traces/ subdir keeps issue-<N>/ readable for humans (just
    # understanding.md / progress.md at the top level).
    if [ -s "$LOG" ]; then
        mkdir -p "${ISSUE_MEM_DIR}/traces"
        ts="$(date -u +%Y-%m-%dT%H-%M-%SZ)"
        gzip -c "$LOG" > "${ISSUE_MEM_DIR}/traces/conversation-${ts}.jsonl.gz" \
            && echo "archived trace to issue-${num}/traces/conversation-${ts}.jsonl.gz" \
            || echo "warning: failed to archive trace for issue #${num}"
    fi

    # Commit and push any memory the agent wrote for this issue (plus the
    # trace archive above).
    commit_memory "$num"

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
