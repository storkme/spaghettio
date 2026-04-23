# Fucktorio

**Goal:** automatically generate end-to-end Factorio production lines. You pick the target item and a production rate; Fucktorio resolves the recipe tree, places machines + belts + pipes + power, validates the result against Factorio's physics, and emits an importable blueprint string.

**[Try it in the browser](https://storkme.github.io/fucktorio/)** — full solver + bus layout + blueprint export runs client-side via WASM.

## Quick start

```bash
# Rust workspace check + unit + e2e tests
cargo test --manifest-path crates/core/Cargo.toml

# Web app (dev server on http://localhost:5173 — rebuilds WASM on change)
cd web && npm install && npm run dev
```

For full build commands (standalone WASM rebuild, release builds), see [`docs/build-systems.md`](docs/build-systems.md).

## Architecture

All pipeline stages live in the Rust workspace under `crates/`:

- **`crates/core/`** — solver, recipe DB, bus layout engine, A*, SAT crossing solver, blueprint export, validation. Pure shared logic; no I/O.
- **`crates/wasm-bindings/`** — thin `wasm-bindgen` wrapper consumed by the web app.
- **`crates/mining-cli/`** — `blueprint-analyze` binary for dissecting community blueprint strings.
- **`web/`** — Vite + TypeScript + PixiJS UI. The live interactive interface. URL state reproduces layouts exactly (`?item=…&rate=…&in=…&belt=…`).

Pipeline stages:

1. **Solver** — recursively resolves recipe dependencies, computes machine counts and per-item flow rates.
2. **Bus layout** — deterministic row-based layout: machines grouped by recipe, parallel trunk belts, tap-offs routed by a ghost A* router with a region-growth / SAT junction solver.
3. **Blueprint export** — JSON + zlib + base64 envelope.
4. **Validation** — 23 functional checks (pipe isolation, fluid connectivity, inserter chains, power coverage, belt flow + structural, underground pairs, lane throughput).

Design docs and per-phase contracts live in [`docs/`](docs/); `CLAUDE.md` is the working agent-facing overview with pointers into the codebase.

## Recipe complexity ladder

How far up the recipe tree we can currently produce an error-free blueprint. Tests live in `crates/core/tests/e2e.rs`.

| Tier | Example recipe              | Status                                               |
|-----:|-----------------------------|------------------------------------------------------|
|    1 | `iron-gear-wheel`           | Solved                                               |
|    2 | `electronic-circuit`        | Solved (including from ores)                         |
|    3 | `plastic-bar`               | Solved                                               |
|    4 | `advanced-circuit`          | Partial — lane-throughput + missing balancer shapes  |
|    5 | `processing-unit`           | Not attempted                                        |
|    6 | `rocket-control-unit`       | Not attempted                                        |

CLAUDE.md has the detailed breakdown and links to open tracking issues.

## Built on

Fucktorio leans heavily on two open-source projects. Credit where it's due:

- **[Factorio-SAT](https://github.com/R-O-C-K-E-T/Factorio-SAT/)** — R-O-C-K-E-T's SAT-based solver for Factorio belt puzzles. We vendor it under `external/factorio-sat/` and call its `belt_balancer` offline to generate our N→M balancer library (`scripts/generate_balancer_library.py` → `crates/core/src/bus/balancer_library.rs`). The runtime crossing-zone SAT encoder in `crates/core/src/sat.rs` is a simplified subset of the same approach.
- **[factorio-draftsman](https://github.com/redruin1/factorio-draftsman)** — redruin1's Python library for reading and writing Factorio data. We use it at build time (`scripts/extract_factorio_data.py`, `scripts/extract_entity_frames.py`) to extract recipe and entity metadata from the installed Factorio data files into the JSON embedded in the Rust build. No runtime dependency.

Both projects made this one possible. Thank you.

## Visualizations

Every push to `main` publishes the live web app to GitHub Pages: <https://storkme.github.io/fucktorio/>

Any URL with query params (`?item=iron-gear-wheel&rate=10&belt=yellow`) renders a live layout with entity overlays, segment highlighting, and validation markers.
