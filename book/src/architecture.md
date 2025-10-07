# Architecture

This workspace is organized as a set of crates to keep simulation, rules,
map primitives, I/O, AI, and visualization decoupled and testable.

Planned crates:
- `core`: core ECS types, shared utilities
- `map`: hex-grid primitives and map storage
- `rules`: rules-as-data definitions and loaders
- `sim`: deterministic systems and fixed-step world
- `ai`: intent generation and policies
- `io`: serialization and persistence
- `viz`: Bevy-based real-time viewer

See subpages for details.

