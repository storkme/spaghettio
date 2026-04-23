#!/bin/bash
set -euo pipefail

# ---------------------------------------------------------------------------
# Required env
# ---------------------------------------------------------------------------
missing=()
[ -n "${GH_TOKEN:-}" ] || missing+=("GH_TOKEN")
[ -n "${AGENT_NAME:-}" ] || missing+=("AGENT_NAME")
[ -n "${ISSUE:-}" ] || missing+=("ISSUE")

if [ "${#missing[@]}" -gt 0 ]; then
    echo "error: missing required env var(s): ${missing[*]}" >&2
    echo "" >&2
    echo "  GH_TOKEN    fine-grained PAT scoped to the target repo" >&2
    echo "  AGENT_NAME  personality name; must match a file in /usr/local/share/agents/" >&2
    echo "  ISSUE       GitHub issue number this container should work on" >&2
    exit 64
fi

AGENT_FILE="/usr/local/share/agents/${AGENT_NAME}.md"
if [ ! -f "$AGENT_FILE" ]; then
    echo "error: unknown agent '${AGENT_NAME}' (no such file ${AGENT_FILE})" >&2
    echo "available agents:" >&2
    ls /usr/local/share/agents/ | sed 's/\.md$//' | sed 's/^/  /' >&2
    exit 64
fi

# ---------------------------------------------------------------------------
# Claude credentials: copy the read-only host mount into a writable per-container
# snapshot so concurrent containers don't stomp on each other's session state.
# ---------------------------------------------------------------------------
if [ -d /mnt/claude-ro ]; then
    mkdir -p "$HOME/.claude"
    cp -a /mnt/claude-ro/. "$HOME/.claude/"
    chmod -R u+w "$HOME/.claude"
else
    echo "warning: /mnt/claude-ro not mounted; Claude will prompt for auth" >&2
fi

# ---------------------------------------------------------------------------
# GitHub auth: gh picks up GH_TOKEN automatically. Teach git to use it for
# HTTPS pushes.
# ---------------------------------------------------------------------------
git config --global credential.https://github.com.helper \
    '!f() { echo "username=oauth2"; echo "password=$GH_TOKEN"; }; f'

# ---------------------------------------------------------------------------
# Per-agent git identity
# ---------------------------------------------------------------------------
agent_cap="$(echo "${AGENT_NAME:0:1}" | tr '[:lower:]' '[:upper:]')${AGENT_NAME:1}"
git config --global user.name "${agent_cap} Bot"
git config --global user.email "${AGENT_NAME}@fucktorio.local"

if gh_user="$(gh api user --jq .login 2>/dev/null)" \
   && [[ "$gh_user" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]]; then
    :
else
    gh_user="(unauthenticated)"
fi

echo "---"
echo "agent:   ${AGENT_NAME}  (identity: ${agent_cap} Bot <${AGENT_NAME}@fucktorio.local>)"
echo "issue:   #${ISSUE}"
echo "gh user: ${gh_user}"
echo "---"

exec "$@"
