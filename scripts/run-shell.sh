#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENTS_DIR="${SCRIPT_DIR}/agents"
IMAGE="${FUCKTORIO_AGENT_IMAGE:-fucktorio-agent:latest}"

usage() {
    cat <<'EOF'
Usage:
  ./scripts/run-shell.sh <agent-name>
  ./scripts/run-shell.sh --list

Drops into an interactive pi TUI with the named agent's personality loaded.
No task is queued — you drive. The workspace has a fresh clone of the repo.

Environment:
  GH_TOKEN    (required)  Fine-grained PAT scoped to the target repo.
  LLAMA_MODEL (required)  Model id from your llama-server's /v1/models endpoint.

  LLAMA_PORT        (optional, default: 8080)
  LLAMA_CONTEXT     (optional, default: 65536)
  LLAMA_MAX_TOKENS  (optional, default: 8192)
  LLAMA_HOST_IP     (optional)  Override the auto-detected Windows gateway IP.

  FUCKTORIO_AGENT_IMAGE (optional, default: fucktorio-agent:latest)

Examples:
  export GH_TOKEN=ghp_xxx LLAMA_MODEL=qwen2.5-coder-32b-instruct
  ./scripts/run-shell.sh misia
  ./scripts/run-shell.sh --list
EOF
}

list_agents() {
    if [ ! -d "$AGENTS_DIR" ]; then
        echo "no agents directory at ${AGENTS_DIR}" >&2
        exit 1
    fi
    echo "available agents:"
    for f in "$AGENTS_DIR"/*.md; do
        [ -e "$f" ] || { echo "  (none)"; return; }
        basename "$f" .md | sed 's/^/  /'
    done
}

case "${1:-}" in
    ""|-h|--help)
        usage
        exit 0
        ;;
    --list)
        list_agents
        exit 0
        ;;
esac

if [ "$#" -ne 1 ]; then
    usage >&2
    exit 64
fi

NAME="$1"

if [ ! -f "${AGENTS_DIR}/${NAME}.md" ]; then
    echo "error: no personality file at ${AGENTS_DIR}/${NAME}.md" >&2
    echo "       (if you just added it, rebuild the image: docker build -t ${IMAGE} .)" >&2
    list_agents >&2
    exit 64
fi

if [ -z "${GH_TOKEN:-}" ]; then
    echo "error: GH_TOKEN is not set. See --help." >&2
    exit 64
fi

if [ -z "${LLAMA_MODEL:-}" ]; then
    echo "error: LLAMA_MODEL is not set. See --help." >&2
    exit 64
fi

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    echo "error: image '${IMAGE}' not found locally. Build it first:" >&2
    echo "       docker build -t ${IMAGE} ." >&2
    exit 64
fi

WIN_HOST="$(ip route | awk '/^default/ {print $3; exit}')"
if [ -z "$WIN_HOST" ]; then
    echo "error: could not auto-detect WSL default gateway. Override with:" >&2
    echo "       LLAMA_HOST_IP=<your-windows-ip> $0 $NAME" >&2
    exit 1
fi
WIN_HOST="${LLAMA_HOST_IP:-$WIN_HOST}"

short="$(openssl rand -hex 3)"
container="fucktorio-shell-${NAME}-${short}"

echo "starting ${container} (agent=${NAME}, model=${LLAMA_MODEL}, llama-host=${WIN_HOST})"
echo "use ctrl-c / /exit to leave."
echo

exec docker run --rm -it \
    --name "$container" \
    --add-host="llama-host:${WIN_HOST}" \
    -e GH_TOKEN="$GH_TOKEN" \
    -e AGENT_NAME="$NAME" \
    -e LLAMA_MODEL="$LLAMA_MODEL" \
    -e LLAMA_PORT="${LLAMA_PORT:-8080}" \
    -e LLAMA_CONTEXT="${LLAMA_CONTEXT:-65536}" \
    -e LLAMA_MAX_TOKENS="${LLAMA_MAX_TOKENS:-8192}" \
    "$IMAGE" agent-shell.sh
