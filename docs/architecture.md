# Civorum Architecture

Civorum is structured as a benchmark platform for deterministic, turn-based, grid strategy simulations with an official MiniCiv reference environment.

## Workspace crates

- `civorum-core`: simulation contract, state/action/effect types, turn loop home.
- `civorum-content`: typed definitions for games, scenarios, and benchmark suites.
- `civorum-rules`: built-in rule primitives, rule resolution, and victory conditions.
- `civorum-generators`: map and scenario generation, including the existing map pipeline.
- `civorum-replay`: replay schema, events, and summary metrics.
- `civorum-benchmark`: suite execution and aggregate metrics.
- `civorum-bots`: built-in baseline agents.
- `civorum-server`: match orchestration and service entry points.
- `civorum-viewer-api`: read-facing endpoints for replay and benchmark UIs.
- `civorum-cli`: local runner and developer entry point.

## Repository directories

- `content/miniciv`: official reference content pack and starter benchmark suite.
- `sdk_py`: Python agent SDK scaffold published as `stratega2`.
- `web/app`: frontend scaffold for replay and benchmark exploration.
- `docs`: product and technical design notes.

## Dependency intent

- `core` depends on `content`, `rules`, `generators`, and `replay`.
- `benchmark` depends on `core`, `content`, and `replay`.
- `bots` depend on `core`.
- `server` depends on `core`, `benchmark`, and `replay`.
- `viewer_api` depends on `benchmark` and `replay`.

This keeps the deterministic engine separate from content authoring, reporting, and presentation layers.

