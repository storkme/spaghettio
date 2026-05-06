#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENTS_DIR="${SCRIPT_DIR}/agents"
IMAGE="${SPAGHETTIO_AGENT_IMAGE:-spaghettio-agent:latest}"

usage() {
    cat <<'EOF'
Usage:
  ./scripts/run-agent.sh <agent-name> <issue-number>
  ./scripts/run-agent.sh --list

Environment:
  GH_TOKEN  (required)  Fine-grained PAT scoped to the target repo.
                        Recommended scopes: contents:write, pull-requests:write,
                        issues:write. Avoid 'workflow' scope unless the agent
                        needs to edit files under .github/workflows/.

  SPAGHETTIO_AGENT_IMAGE (optional, default: spaghettio-agent:latest)
                        The image tag to run.

Examples:
  GH_TOKEN=ghp_xxx ./scripts/run-agent.sh scout 123
  ./scripts/run-agent.sh --list
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

if [ "$#" -ne 2 ]; then
    usage >&2
    exit 64
fi

NAME="$1"
ISSUE="$2"

if ! [[ "$ISSUE" =~ ^[0-9]+$ ]]; then
    echo "error: issue number must be a positive integer (got '${ISSUE}')" >&2
    exit 64
fi

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

PI_MOUNT_ARGS=()
PI_KEY_ARGS=()
if [ -n "${HOME:-}" ] && [ -d "${HOME}/.pi" ]; then
    PI_MOUNT_ARGS=(-v "${HOME}/.pi:/mnt/pi-ro:ro")
elif [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    PI_KEY_ARGS=(-e ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY")
else
    echo "error: no pi backend found." >&2
    echo "       either have ~/.pi on the host (run 'pi' once and /login)," >&2
    echo "       or set ANTHROPIC_API_KEY." >&2
    exit 64
fi

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    echo "error: image '${IMAGE}' not found locally. Build it first:" >&2
    echo "       docker build -t ${IMAGE} ." >&2
    exit 64
fi

short="$(openssl rand -hex 3)"
container="spaghettio-agent-${NAME}-${short}"

echo "starting ${container} (agent=${NAME}, issue=#${ISSUE})"
echo "use ctrl-c to abort; transcript streams to this terminal."
echo

exec docker run --rm -it \
    --name "$container" \
    "${PI_MOUNT_ARGS[@]}" \
    "${PI_KEY_ARGS[@]}" \
    -e GH_TOKEN="$GH_TOKEN" \
    -e AGENT_NAME="$NAME" \
    -e ISSUE="$ISSUE" \
    "$IMAGE"
