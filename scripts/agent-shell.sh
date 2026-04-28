#!/bin/bash
set -euo pipefail

# Invoked by docker-entrypoint.sh. AGENT_NAME and GH_TOKEN have already been
# validated; backend (OAuth or llama) is in place; git identity and credential
# helper are configured.
#
# Drops into an interactive pi TUI with the agent personality loaded as the
# initial system context. No task instructions — the user drives from here.

REPO="${REPO:-storkme/fucktorio}"
WORKSPACE="/tmp/workspace"

PI_BACKEND_ARGS=()
if [ -n "${LLAMA_MODEL:-}" ]; then
    PI_BACKEND_ARGS=(--provider llama --model "$LLAMA_MODEL")
fi

# ---------------------------------------------------------------------------
# Fresh clone so the agent has the codebase available
# ---------------------------------------------------------------------------
rm -rf "$WORKSPACE"
echo "cloning ${REPO} into ${WORKSPACE}..."
gh repo clone "$REPO" "$WORKSPACE" -- --quiet
cd "$WORKSPACE"

# Belt-and-braces: refuse pushes to main/master.
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

# ---------------------------------------------------------------------------
# Personality — seed it as the initial -p so the agent has its persona and
# style loaded, but there are no task instructions so you drive from there.
# ---------------------------------------------------------------------------
PERSONAL_PROMPT="$(cat "/usr/local/share/agents/${AGENT_NAME}.md")"

echo "starting interactive session as ${AGENT_NAME}..."
echo "(workspace: ${WORKSPACE}, repo: ${REPO})"
echo

exec pi "${PI_BACKEND_ARGS[@]}"
