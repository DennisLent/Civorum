# Civorum

Civorum is being reshaped into a data-driven benchmark platform for deterministic, turn-based, grid strategy environments.

## Repository layout

- `crates/`: Rust workspace for engine, content, rules, generation, replay, benchmarking, bots, and service layers
- `content/civorum/`: official reference game content pack
- `sdk_py/`: Python SDK scaffold for external agents
- `web/app/`: frontend scaffold for live matches, replays, and benchmark results
- `docs/architecture.md`: crate responsibilities and dependency intent

## Near-term direction

- Keep engine behavior deterministic and seed-replayable
- Make content packs data-driven
- Ship MiniCiv as the first official reference environment
- Build the benchmark and replay product around that stable simulation core

