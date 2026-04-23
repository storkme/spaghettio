# Agent container

A disposable Docker image that runs a [pi](https://pi.dev/) coding agent
against one GitHub issue. Phase 1: the human picks the issue and watches the
run; the container exits when pi's session ends. Orchestration (watching for
new issues, sequencing multiple tasks) is deliberately out of scope for now.

## Build

```sh
docker build -t fucktorio-agent:latest .
```

Rebuild whenever you change `Dockerfile`, `docker-entrypoint.sh`, the
orchestrator (`scripts/agent-runner.sh`), or a personality file
(`scripts/agents/*.md`). Nothing else triggers a rebuild.

## Launch

```sh
export GH_TOKEN=ghp_...                       # fine-grained PAT, see below
./scripts/run-agent.sh scout 123              # agent=scout, issue=#123
./scripts/run-agent.sh --list                 # show available agents
```

The launcher streams the pi transcript straight to your terminal. Ctrl-C
aborts the container (`--rm` cleans up).

## Required environment

| Var | Where set | Notes |
|-----|-----------|-------|
| `GH_TOKEN` | host | Fine-grained PAT. See scopes below. |
| `AGENT_NAME` | passed by launcher | Matches a file under `scripts/agents/`. |
| `ISSUE` | passed by launcher | Positive integer; the issue number to work on. |

`~/.pi` must exist on the host (run `pi` once and `/login`, or set
`ANTHROPIC_API_KEY` in your shell). The launcher refuses to start without it.

pi stores OAuth tokens at `~/.pi/agent/auth.json` and auto-refreshes them;
the entrypoint copies the whole `~/.pi` tree into a writable per-container
snapshot so concurrent containers don't stomp on each other's refreshed token.

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
3. Launch: `./scripts/run-agent.sh <name> <issue>`.

If you forget to rebuild, your running container will keep using the old
personality. The launcher catches "file doesn't exist on host" but cannot
catch "file exists on host but is stale in the image" — that's on you.

The baked-in model is deliberate: it means a malicious or accidental PR
editing `scripts/agents/*.md` cannot affect an already-running container, and
each container uses exactly the personality it was built with.

## Security properties

- **No credentials in the image.** `GH_TOKEN` comes in via env at run time;
  `~/.pi` is bind-mounted read-only and copied into a writable
  container-private snapshot by the entrypoint.
- **Blast radius = container + token scopes.** pi has no permission gate —
  its four tools (`read`, `write`, `edit`, `bash`) execute freely. They run
  inside a non-root user, inside a disposable container, with no host mounts
  beyond the read-only creds source, so worst-case misbehaviour is bounded by
  the container filesystem plus `GH_TOKEN`'s permissions on the target repo.
- **Client-side no-push-to-main.** The orchestrator installs a pre-push hook
  that refuses pushes to `main` / `master` even if the agent's prompt
  discipline slips.
- **Per-container pi session.** Concurrent containers get independent writable
  copies of `~/.pi`; their OAuth refresh writes cannot collide.

## Debug shell

```sh
docker compose run --rm dev
```

Drops into bash inside the image. Useful for checking tool versions, inspecting
the copied `~/.pi`, or rerunning the orchestrator by hand with different env
vars. Does not invoke the orchestrator automatically.

## Phase 1 limits

- One issue per container invocation. The container does not discover or claim
  issues; the caller names one.
- No parallel-safety mechanisms beyond "each container has an independent
  filesystem and an independent `~/.pi` snapshot". Two containers pointed at
  the same issue will both try to work on it — don't do that.
- No automatic PR verification, no success/failure labelling, no status
  comments on the issue. The human inspects the result via `gh pr list`.
- CI uses Node 20; the image uses Node 24 to match the maintainer's local
  setup. This mismatch is known and will be reconciled when CI moves to Node 24.
