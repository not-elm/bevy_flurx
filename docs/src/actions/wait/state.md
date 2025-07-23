# wait::state

The `wait::state` module provides actions for waiting for Bevy state transitions. These actions are useful for coordinating tasks with Bevy's state system, allowing tasks to wait for specific state changes before continuing execution.

## Functions

### becomes

```rust
wait::state::becomes<S>() -> ActionSeed<S>
```

Creates an action that waits until the state becomes the specified value. The action completes when the state matches the expected value.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(States, Eq, PartialEq, Copy, Clone, Hash, Default, Debug)]
enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

Reactor::schedule(|task| async move {
    // Wait until the game state becomes Playing
    task.will(Update, wait::state::becomes().with(GameState::Playing)).await;
    
    // This code runs after the game state becomes Playing
    println!("Game is now in Playing state!");
});
```

## When to Use

Use `wait::state` actions when you need to:
- Coordinate tasks with Bevy's state system
- Wait for specific state transitions before continuing execution
- Create game flow sequences that depend on state changes
- Implement multi-stage processes that follow a state machine pattern

State-based waiting is particularly useful for game flow control, menu navigation, and level transitions.