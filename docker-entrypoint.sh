#!/bin/bash
set -euo pipefail

# ---------------------------------------------------------------------------
# Required env
# ---------------------------------------------------------------------------
missing=()
[ -n "${GH_TOKEN:-}" ] || missing+=("GH_TOKEN")
[ -n "${AGENT_NAME:-}" ] || missing+=("AGENT_NAME")

if [ "${#missing[@]}" -gt 0 ]; then
    echo "error: missing required env var(s): ${missing[*]}" >&2
    echo "" >&2
    echo "  GH_TOKEN    fine-grained PAT scoped to the target repo" >&2
    echo "  AGENT_NAME  personality name; must match a file in /usr/local/share/agents/" >&2
    echo "" >&2
    echo "One-shot (agent-runner.sh) additionally requires ISSUE; watcher mode" >&2
    echo "(agent-watcher.sh) picks ISSUEs dynamically and additionally requires" >&2
    echo "LLAMA_MODEL (or a mounted /mnt/pi-ro for the OAuth path)." >&2
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
# Backend wire-up. Two paths — exactly one must be satisfied:
#   (a) /mnt/pi-ro mounted        → pi uses its bundled providers (e.g. Anthropic OAuth).
#   (b) LLAMA_MODEL env set       → render a llama.cpp provider config into ~/.pi/agent/models.json.
# ---------------------------------------------------------------------------
have_oauth=0
have_llama=0
have_apikey=0

if [ -d /mnt/pi-ro ]; then
    mkdir -p "$HOME/.pi"
    # -n (no-clobber): image-baked files in ~/.pi (e.g. installed extensions
    # listed in settings.json) win. Host mount only fills in what the image
    # doesn't already have — typically auth.json / credentials.
    cp -an /mnt/pi-ro/. "$HOME/.pi/"
    chmod -R u+w "$HOME/.pi"
    # auth.json is user-only by design (0600); keep that after the copy.
    [ -f "$HOME/.pi/agent/auth.json" ] && chmod 600 "$HOME/.pi/agent/auth.json"
    have_oauth=1
fi

if [ -n "${LLAMA_MODEL:-}" ]; then
    : "${LLAMA_PORT:=8080}"
    : "${LLAMA_CONTEXT:=65536}"
    : "${LLAMA_MAX_TOKENS:=8192}"
    export LLAMA_PORT LLAMA_MODEL LLAMA_CONTEXT LLAMA_MAX_TOKENS
    mkdir -p "$HOME/.pi/agent"
    envsubst < /usr/local/share/pi/models.json.tmpl > "$HOME/.pi/agent/models.json"
    chmod 600 "$HOME/.pi/agent/models.json"
    have_llama=1
fi

if [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    have_apikey=1
fi

if [ $have_oauth -eq 0 ] && [ $have_llama -eq 0 ] && [ $have_apikey -eq 0 ]; then
    echo "error: no backend configured." >&2
    echo "       bind-mount ~/.pi to /mnt/pi-ro (OAuth/API-key path)," >&2
    echo "       set ANTHROPIC_API_KEY, or set LLAMA_MODEL." >&2
    exit 64
fi

# ---------------------------------------------------------------------------
# Watcher state dir. Named volume spaghettio-state-<agent> mounts at
# /var/lib/agent and holds state.json (tracks last-seen comment ids per
# issue/PR so the scan phase can detect new human comments). Root-owned on
# first mount — chown it, then bootstrap an empty state.json if missing.
# ---------------------------------------------------------------------------
if [ -d /var/lib/agent ]; then
    sudo chown "$(id -u):$(id -g)" /var/lib/agent 2>/dev/null || true
    if [ ! -f /var/lib/agent/state.json ]; then
        echo '{"version": 1, "issues": {}, "prs": {}}' > /var/lib/agent/state.json
    fi
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
git config --global user.email "${AGENT_NAME}@spaghettio.local"

if gh_user="$(gh api user --jq .login 2>/dev/null)" \
   && [[ "$gh_user" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]]; then
    :
else
    gh_user="(unauthenticated)"
fi

# ---------------------------------------------------------------------------
# Banner
# ---------------------------------------------------------------------------
echo "---"
echo "agent:   ${AGENT_NAME}  (identity: ${agent_cap} Bot <${AGENT_NAME}@spaghettio.local>)"
if [ -n "${ISSUE:-}" ]; then
    echo "mode:    one-shot (issue #${ISSUE})"
else
    echo "mode:    watcher (label: ${AGENT_READY_LABEL:-${AGENT_NAME}-ready}, poll: ${POLL_INTERVAL:-60}s)"
fi
if [ $have_llama -eq 1 ]; then
    echo "backend: llama.cpp  (model=${LLAMA_MODEL}, host=llama-host:${LLAMA_PORT})"
elif [ $have_apikey -eq 1 ]; then
    echo "backend: ANTHROPIC_API_KEY"
else
    echo "backend: pi OAuth/API key via ~/.pi"
fi
echo "gh user: ${gh_user}"
echo "---"

exec "$@"
