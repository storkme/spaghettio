#!/bin/bash
set -euo pipefail

# Invoked by docker-entrypoint.sh. AGENT_NAME, ISSUE, GH_TOKEN have already been
# validated; Pi credentials have been copied into a writable $HOME/.pi;
# git identity and credential helper are configured.

REPO="${REPO:-storkme/fucktorio}"
WORKSPACE="/tmp/workspace"
LOG="/tmp/agent.log"
BRANCH="agent/${AGENT_NAME}/issue-${ISSUE}"

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
pi -p "$BASE_PROMPT" 2>&1 | tee "$LOG"
rc=${PIPESTATUS[0]}
set -e

echo "---"
echo "pi exited rc=${rc}"
echo "log saved to ${LOG}"

exit "$rc"
