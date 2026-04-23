#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENTS_DIR="${SCRIPT_DIR}/agents"
IMAGE="${FUCKTORIO_AGENT_IMAGE:-fucktorio-agent:latest}"

usage() {
    cat <<'EOF'
Usage:
  ./scripts/run-watcher.sh <agent-name>              Start a long-running watcher
  ./scripts/run-watcher.sh --logs <agent-name>       docker logs -f the watcher
  ./scripts/run-watcher.sh --stop <agent-name>       Send SIGTERM and wait for clean exit
  ./scripts/run-watcher.sh --list                    Show available agent names
  ./scripts/run-watcher.sh --status                  Show running watchers

Environment:
  GH_TOKEN          (required)  Fine-grained PAT scoped to the target repo.
                                Recommended scopes: contents:write,
                                pull-requests:write, issues:write.
                                Avoid 'workflow' unless the agent must touch CI.

  LLAMA_MODEL       (required)  Model id advertised by your llama-server's
                                /v1/models endpoint (e.g. qwen2.5-coder-32b).

  LLAMA_PORT        (optional)  Port the Windows llama-server listens on
                                (default: 8080).
  LLAMA_CONTEXT     (optional)  Context window in tokens (default: 32768).
  LLAMA_MAX_TOKENS  (optional)  Per-response cap (default: 8192).
  POLL_INTERVAL     (optional)  Seconds between issue-queue polls (default: 60).

  FUCKTORIO_AGENT_IMAGE (optional, default: fucktorio-agent:latest)

Windows-side prerequisites (one-time):
  - Run llama-server bound to 0.0.0.0:<LLAMA_PORT> (not 127.0.0.1).
  - Allow inbound on that port through Windows Defender Firewall for the WSL
    subnet (typically 172.16.0.0/12). The launcher auto-detects the Windows
    gateway from WSL's default route and wires it as 'llama-host' inside the
    container; pi's models.json resolves http://llama-host:<port>/v1.

Examples:
  export GH_TOKEN=ghp_xxx LLAMA_MODEL=qwen2.5-coder-32b-instruct
  ./scripts/run-watcher.sh misia
  ./scripts/run-watcher.sh --logs misia
  ./scripts/run-watcher.sh --stop misia
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

status() {
    running="$(docker ps --format '{{.Names}}\t{{.Status}}' | grep '^fucktorio-watcher-' || true)"
    if [ -z "$running" ]; then
        echo "no watchers running."
    else
        echo "running watchers:"
        echo "$running" | sed 's/^/  /'
    fi
}

container_name_for() {
    echo "fucktorio-watcher-$1"
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
    --status)
        status
        exit 0
        ;;
    --logs)
        [ -n "${2:-}" ] || { echo "error: --logs needs an agent name" >&2; exit 64; }
        exec docker logs -f "$(container_name_for "$2")"
        ;;
    --stop)
        [ -n "${2:-}" ] || { echo "error: --stop needs an agent name" >&2; exit 64; }
        container="$(container_name_for "$2")"
        if ! docker ps --format '{{.Names}}' | grep -qx "$container"; then
            echo "${container} is not running."
            exit 0
        fi
        echo "sending SIGTERM to ${container} (timeout 30s)..."
        docker stop --timeout 30 "$container"
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

CONTAINER="$(container_name_for "$NAME")"

# Double-start guard — one watcher per agent name.
if docker ps --format '{{.Names}}' | grep -qx "$CONTAINER"; then
    echo "error: ${CONTAINER} is already running." >&2
    echo "       use --logs ${NAME} to attach, or --stop ${NAME} to terminate." >&2
    exit 64
fi

# Clear any stopped container with the same name so --restart+--name doesn't clash.
if docker ps -a --format '{{.Names}}' | grep -qx "$CONTAINER"; then
    docker rm "$CONTAINER" >/dev/null
fi

WIN_HOST="$(ip route | awk '/^default/ {print $3; exit}')"
if [ -z "$WIN_HOST" ]; then
    echo "error: could not auto-detect WSL default gateway. Set it manually:" >&2
    echo "       LLAMA_HOST_IP=<your-windows-ip> $0 $NAME" >&2
    exit 1
fi
WIN_HOST="${LLAMA_HOST_IP:-$WIN_HOST}"

echo "starting ${CONTAINER} (agent=${NAME}, model=${LLAMA_MODEL}, llama-host=${WIN_HOST})"

docker run -d \
    --name "$CONTAINER" \
    --restart unless-stopped \
    --add-host="llama-host:${WIN_HOST}" \
    -e GH_TOKEN="$GH_TOKEN" \
    -e AGENT_NAME="$NAME" \
    -e AGENT_READY_LABEL="${AGENT_READY_LABEL:-${NAME}-ready}" \
    -e LLAMA_PORT="${LLAMA_PORT:-8080}" \
    -e LLAMA_MODEL="$LLAMA_MODEL" \
    -e LLAMA_CONTEXT="${LLAMA_CONTEXT:-32768}" \
    -e LLAMA_MAX_TOKENS="${LLAMA_MAX_TOKENS:-8192}" \
    -e POLL_INTERVAL="${POLL_INTERVAL:-60}" \
    "$IMAGE" agent-watcher.sh >/dev/null

echo "started. follow with:"
echo "  ./scripts/run-watcher.sh --logs ${NAME}"
