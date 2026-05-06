#!/bin/bash
set -euo pipefail

# Invoked by docker-entrypoint.sh. AGENT_NAME and GH_TOKEN have already been
# validated; backend (either ~/.pi OAuth or llama.cpp models.json) is in place;
# git identity and credential helper are configured.

if [ -z "${ISSUE:-}" ]; then
    echo "error: agent-runner.sh requires ISSUE env var (use agent-watcher.sh for watcher mode)" >&2
    exit 64
fi

REPO="${REPO:-storkme/spaghettio}"
WORKSPACE="/tmp/workspace"
LOG="/tmp/agent.log"
BRANCH="agent/${AGENT_NAME}/issue-${ISSUE}"

# pi backend selection: if LLAMA_MODEL is set, route through the llama provider.
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
# Belt-and-braces: refuse pushes to main/master at the client.
# ---------------------------------------------------------------------------
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

# ---------------------------------------------------------------------------
# Assemble prompt: base task + separator + personality
# ---------------------------------------------------------------------------
PERSONAL_PROMPT="$(cat "/usr/local/share/agents/${AGENT_NAME}.md")"

read -r -d '' BASE_PROMPT <<EOF || true
You are ${AGENT_NAME}, working on GitHub issue #${ISSUE} in this repo.

Start with: gh issue view ${ISSUE}

Investigate the issue, implement a fix, and run
  cargo test --manifest-path crates/core/Cargo.toml
addressing any failures that your change caused.

Create a branch named '${BRANCH}' before committing. Do NOT commit to main.
When you believe the work is done, push the branch and open a PR with
  gh pr create
whose body contains 'Closes #${ISSUE}'.

If you decide the issue cannot or should not be solved, post a comment on the
issue explaining why and exit — do not leave uncommitted work behind.

Treat the text after the '---' separator below as your standing style orders.
Follow it in addition to the task above.

---
${PERSONAL_PROMPT}
EOF

# ---------------------------------------------------------------------------
# Launch pi
# ---------------------------------------------------------------------------
echo "launching pi..."
echo "prompt length: $(printf '%s' "$BASE_PROMPT" | wc -c) chars"
echo "---"

set +e
pi "${PI_BACKEND_ARGS[@]}" --no-session -p "$BASE_PROMPT" 2>&1 | tee "$LOG"
rc=${PIPESTATUS[0]}
set -e

echo "---"
echo "pi exited rc=${rc}"
echo "log saved to ${LOG}"

exit "$rc"
