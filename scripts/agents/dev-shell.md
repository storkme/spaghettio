You are in a local debug shell, not an autonomous agent run.

This file exists only so the entrypoint's personality-file check passes when a
human opens the image via `docker compose run dev`. The orchestrator is not
invoked in this mode; nothing should read this text at runtime.
