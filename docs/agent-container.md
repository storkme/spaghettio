# Agent container

A Docker image that runs a [pi](https://pi.dev/) coding agent against
Fucktorio GitHub issues. Two modes:

- **One-shot** (`scripts/run-agent.sh`): the human names one issue, the
  container clones the repo, runs pi against that issue, then exits.
- **Watcher** (`scripts/run-watcher.sh`): a long-running container that polls
  GitHub for issues matching a per-agent label, picks them up sequentially,
  and opens a PR per issue. Survives reboots via `--restart unless-stopped`.

Both modes use the same image and the same personality files.

## Build

```sh
docker build -t fucktorio-agent:latest .
```

Rebuild whenever you change `Dockerfile`, `docker-entrypoint.sh`, either
orchestrator (`scripts/agent-runner.sh`, `scripts/agent-watcher.sh`), a
personality file (`scripts/agents/*.md`), or the llama.cpp template
(`scripts/pi/models.json.tmpl`). Nothing else triggers a rebuild.

## Backends

pi supports several LLM providers. The container is wired for two:

- **pi's bundled OAuth / API-key providers.** Requires `~/.pi` on the host
  (run `pi` once and `/login`). The one-shot launcher uses this by default;
  it bind-mounts `~/.pi` read-only into the container and the entrypoint
  copies it into a writable per-container snapshot so concurrent containers
  don't stomp on each other's refresh-token writes.
- **Local llama.cpp server.** Set `LLAMA_MODEL` (and optionally `LLAMA_PORT`,
  `LLAMA_CONTEXT`, `LLAMA_MAX_TOKENS`). The entrypoint renders
  `~/.pi/agent/models.json` from `/usr/local/share/pi/models.json.tmpl` using
  `envsubst`, then pi is invoked with `--provider llama --model $LLAMA_MODEL`.
  The watcher launcher assumes this path; no `~/.pi` mount is needed.

Exactly one of the two backends must be configured at entrypoint time, or the
container fails fast.

## One-shot (phase 1)

```sh
export GH_TOKEN=ghp_...                       # fine-grained PAT, see below
./scripts/run-agent.sh scout 123              # agent=scout, issue=#123
./scripts/run-agent.sh --list                 # show available agents
```

Uses pi's bundled providers (whatever you're logged into on the host via
`~/.pi`). Streams the transcript to your terminal; `Ctrl-C` aborts; `--rm`
cleans up. See [Adding or editing an agent](#adding-or-editing-an-agent)
below.

## Watcher (phase 2)

```sh
export GH_TOKEN=ghp_... LLAMA_MODEL=qwen2.5-coder-32b-instruct
./scripts/run-watcher.sh misia                # start the misia watcher
./scripts/run-watcher.sh --logs misia         # docker logs -f
./scripts/run-watcher.sh --stop misia         # SIGTERM, waits 30s
./scripts/run-watcher.sh --status             # list running watchers
```

The watcher:

1. Clones the repo once into `/tmp/workspace` (the clone survives across
   issues, so cargo's `target/` stays warm).
2. Loops: pick one open issue labelled `${AGENT_NAME}-ready` (e.g.
   `misia-ready`), reset the workspace to `origin/main`, branch to
   `agent/<name>/issue-<n>`, run pi, check for the resulting PR, relabel the
   issue `agent-done` (PR opened) or `agent-failed` (no PR, log tail
   commented on the issue). Sleep `POLL_INTERVAL` seconds if no candidate.
3. SIGTERM (`docker stop`, `./run-watcher.sh --stop`) finishes the current
   iteration and exits cleanly within ~30s.

**One watcher per agent name.** The launcher refuses to start a second
watcher with the same name; use `--logs` or `--stop` instead. If you want
two agents, name them differently (they'll watch different label queues)
and run one `run-watcher.sh` per name.

### Label state machine

| Label | Meaning |
|-------|---------|
| `<agent-name>-ready` | Issue is queued for this agent. Watcher claims it. |
| `agent-done` | Watcher opened a PR for this issue. |
| `agent-failed` | pi returned without opening a PR; a log-tail comment is attached. |

You create the `<agent-name>-ready` label once per agent. The watcher creates
`agent-done` and `agent-failed` on demand.

### Windows-side prerequisites (llama.cpp backend)

The llama.cpp server runs on the Windows host; the container reaches it via a
launcher-detected address mapped to the hostname `llama-host` in the
container's `/etc/hosts`. One-time Windows setup:

1. **Bind to all interfaces.** Start llama-server with `--host 0.0.0.0`
   (not the `127.0.0.1` default). Verify with `curl http://<windows-ip>:8080/v1/models`
   from WSL.
2. **Firewall exception.** Allow inbound TCP on the llama-server port for
   the WSL subnet (typically `172.16.0.0/12`). In PowerShell as admin:
   ```powershell
   New-NetFirewallRule -DisplayName "llama-server (WSL)" `
       -Direction Inbound -Action Allow -Protocol TCP `
       -LocalPort 8080 -RemoteAddress 172.16.0.0/12
   ```

The launcher auto-detects the WSL-to-Windows gateway via
`ip route | awk '/^default/ {print $3; exit}'`. Override with
`LLAMA_HOST_IP=<ip>` if the detection picks something wrong.

### Environment

| Var | Required | Default | Notes |
|-----|----------|---------|-------|
| `GH_TOKEN` | yes | — | Fine-grained PAT; see scopes below. |
| `AGENT_NAME` | yes | — | Matches a file under `scripts/agents/`. |
| `ISSUE` | one-shot only | — | Positive integer; ignored by the watcher. |
| `LLAMA_MODEL` | llama backend | — | Model id from `/v1/models` on the server. |
| `LLAMA_PORT` | no | `8080` | Windows-side llama-server port. |
| `LLAMA_CONTEXT` | no | `32768` | Context window (tokens). |
| `LLAMA_MAX_TOKENS` | no | `8192` | Per-response cap. |
| `AGENT_READY_LABEL` | no | `${AGENT_NAME}-ready` | Label the watcher polls on. |
| `POLL_INTERVAL` | no | `60` | Seconds between queue polls. |
| `LLAMA_HOST_IP` | no | auto | Override the WSL-gateway auto-detection. |

## Recommended `GH_TOKEN` scopes

Fine-grained PAT scoped to a single repo (e.g. `storkme/fucktorio`), with the
minimum set of repository permissions:

- **Contents**: Read and write
- **Pull requests**: Read and write
- **Issues**: Read and write

Do **not** grant `workflow` scope unless the agent specifically needs to edit
`.github/workflows/*`. Pushes that touch workflow files without that scope fail
with a confusing 403; granting it unnecessarily broadens the blast radius.

## Adding or editing an agent

Personality files are **baked into the image at build time**, not read from the
live checkout. The rule is: **edit → rebuild → run**.

1. Create or edit `scripts/agents/<name>.md`.
2. Rebuild the image: `docker build -t fucktorio-agent:latest .` (Docker's
   layer cache makes this fast once the Rust layer is warm — usually a few
   seconds for a pure personality-file change).
3. Launch: `./scripts/run-agent.sh <name> <issue>` or
   `./scripts/run-watcher.sh <name>`.

If you forget to rebuild, your running container keeps using the old
personality. The launcher catches "file doesn't exist on host" but cannot
catch "file exists on host but is stale in the image" — that's on you.

The baked-in model is deliberate: it means a malicious or accidental PR
editing `scripts/agents/*.md` cannot affect an already-running container, and
each container uses exactly the personality it was built with.

## Security properties

- **No credentials in the image.** `GH_TOKEN` is injected at run time. For
  the OAuth path, `~/.pi` is bind-mounted read-only and copied into a
  writable container-private snapshot. For the llama path, the `models.json`
  is rendered fresh per container from env vars — no token material at all.
- **Blast radius = container + token scopes.** pi has no permission gate;
  its tools (`read`, `write`, `edit`, `bash`) execute freely. They run as a
  non-root user inside a disposable container with no host mounts beyond the
  read-only creds source (OAuth path) or nothing (llama path). Worst-case
  misbehaviour is bounded by the container filesystem plus `GH_TOKEN`'s
  permissions on the target repo.
- **Client-side no-push-to-main.** Both orchestrators install a pre-push
  hook that refuses pushes to `main` / `master` even if the agent's prompt
  discipline slips.
- **Per-container pi session.** Concurrent containers get independent
  writable copies of `~/.pi`; their OAuth refresh writes (and, in the llama
  path, their rendered `models.json`) cannot collide.
- **Llama path has no LLM-provider secrets.** The rendered `models.json`
  contains only the model id, baseUrl, and a dummy `"not-needed"` apiKey.
  Nothing to leak.

## Debug shell

```sh
docker compose run --rm dev
```

Drops into bash inside the image. Useful for checking tool versions, inspecting
the copied `~/.pi`, or rerunning an orchestrator by hand with different env
vars. Does not invoke any orchestrator automatically.

## Limits

- **One watcher per agent name.** Multiple concurrent watchers on the same
  label queue are not supported (no claim protocol). Use separate names +
  separate `<name>-ready` labels to run two agents in parallel.
- **PRs are not decorated with the transcript.** Failed runs drop the last
  80 lines into a comment on the issue; successful runs leave only the PR.
- CI uses Node 20; the image uses Node 24 to match the maintainer's local
  setup. This mismatch is known and will be reconciled when CI moves to
  Node 24.
