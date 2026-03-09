# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

bevy_flurx is a Bevy plugin that provides coroutine-like sequential control flow within Bevy's synchronous ECS. It enables async-like syntax (using `.await`) for describing delays, state waits, character movement, and user input handling — without a real async runtime. Supports `no_std`.

**Current version:** 0.14.0 (targets Bevy 0.18)

## Build & Test Commands

```bash
# Run tests (standard CI command — includes all commonly-used features)
cargo test -F state,audio,tokio,record,side-effect

# Run tests with all features
cargo test --all-features

# Run doc tests only (also compiles README examples)
cargo test --doc --all-features

# Lint
cargo clippy --workspace --all-targets --all-features -- -Dwarnings

# Format check
cargo fmt --all -- --check

# Run a single test
cargo test <test_name> -F state,audio,tokio,record,side-effect

# Run benchmarks
cargo bench --bench single
cargo bench --bench repeat

# Run examples (some require feature flags)
cargo run --example simple
cargo run --example side_effect -F tokio,side-effect
cargo run --example undo_redo -F record
```

## Feature Flags

| Flag | Purpose |
|------|---------|
| `audio` | Audio actions (requires bevy_audio, bevy_asset) |
| `tokio` | Tokio runtime integration |
| `record` | Undo/redo system |
| `side-effect` | Thread/async side effects |
| `state` | Bevy state actions |
| `std` | Standard library features |
| `serialize` | Serde serialization support |

No features are enabled by default. Tests should be run with `-F state,audio,tokio,record,side-effect` to match CI.

## Architecture

### Core Execution Flow

1. **Reactor** (`src/reactor.rs`) — A Bevy component spawned with `Reactor::schedule(|task| async move { ... })`. Uses an entity on-add hook to initialize a `NativeReactor` that manages the async block.
2. **ReactorTask** (`src/task.rs`) — Passed into the Reactor's async closure. `task.will(schedule_label, action).await` submits an action and suspends until it completes.
3. **Runner** (`src/runner.rs`) — Trait for executing actions. Each `task.will()` call registers a `BoxedRunner` in a per-entity `TypedRegistry`. Runners are polled each frame until they return `RunnerIs::Completed` or `RunnerIs::Canceled`.
4. **CoreScheduler** (`src/core/scheduler.rs`) — Manages the async future for a single reactor, polling it frame-by-frame.
5. **Output<T>** (`src/runner/output.rs`) — `Arc<RwLock<Option<T>>>` channel for passing action results back to the awaiting task.

When all actions in a Reactor complete, the reactor entity is automatically despawned.

### Action System (`src/action/`)

- **Action<I, O>** — Bundles an input value with an `ActionSeed`
- **ActionSeed<I, O>** — Lazy action descriptor that creates a runner when executed
- Composition traits: `Then` (sequential), `Pipe` (output→input chaining), `Through` (pass-through), `Map` (transform output), `Inspect` (observe), `Omit` (erase types)

Action modules:
- `once/` — Execute a system exactly once (resources, messages, events, audio, state, switch)
- `wait/` — Poll each frame until a condition is met (input, messages, events, audio, state, switch, combinators: `all`, `any`, `both`, `either`)
- `delay/` — Time-based or frame-based delays
- `pipe/`, `sequence/`, `through/`, `switch/` — Composition and control flow
- `record/` — Undo/redo (feature-gated: `record`)
- `side_effect/` — Thread/async/Bevy task execution (feature-gated: `side-effect`)

### Macros

- `sequence![a, b, c]` — Sequential action execution
- `actions![a, b]` — Create an array of type-erased actions

### Key Patterns

- **Selector** (`src/selector.rs`, `src/core/selector.rs`) — Interface for selecting data from the World to feed into actions
- **WorldPtr** (`src/world_ptr.rs`) — Unsafe wrapper for accessing `&mut World` across the async boundary
- **Runner registration** — Dynamic system insertion into Bevy schedules via `AppScheduleLabels`
- **Bevy 0.18 Message API** — The codebase uses `Message`/`MessageReader`/`MessageWriter` (not the older `Event` trait)

## Lint Configuration

Configured in `Cargo.toml`:
- `clippy::type_complexity` = allow
- `clippy::doc_markdown` = warn
- `clippy::undocumented_unsafe_blocks` = warn
- `rust::missing_docs` = warn

## Testing Patterns

Tests use `bevy_test_helper` crate (from a git dependency). The common test setup is:
```rust
let mut app = test_app(); // defined in src/lib.rs — adds MinimalPlugins, InputPlugin, StatesPlugin, FlurxPlugin
app.add_systems(Startup, |mut commands: Commands| {
    commands.spawn(Reactor::schedule(|task| async move {
        // test actions here
    }));
});
app.update(); // advance frames
```

Helper functions: `increment_count()`, `decrement_count()`, `exit_reader()`, `came_event::<E>()`.

## Versioning

Pre-v1.0 semver: breaking changes increment MINOR version. See `VERSION_RULE.md`.
