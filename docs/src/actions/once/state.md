# once::state

The `once::state` module provides actions for managing Bevy state transitions exactly once. This module is designed to work with Bevy's [States](https://docs.rs/bevy/latest/bevy/ecs/schedule/struct.States.html) system.

## Functions

### set

```rust
once::state::set<S>() -> ActionSeed<S>
```

Creates an action that sets the next state for a Bevy state machine. The state value must be provided using the `.with()` method.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(States, Copy, Clone, Hash, Eq, PartialEq, Default, Debug)]
enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

Reactor::schedule(|task| async move {
    // Wait for some condition...
    
    // Transition to the Playing state
    task.will(Update, once::state::set().with(GameState::Playing)).await;
});
```

## State Machines in Bevy

Bevy's state system allows you to organize your game into distinct states, with systems that run only during specific states. The `once::state::set()` action provides a way to trigger state transitions from within a Reactor.

## When to Use

Use `once::state` actions when you need to:
- Trigger state transitions from within a Reactor
- Create game flow sequences that involve state changes
- Coordinate asynchronous operations with state-based systems

For more complex state management or when you need to access other system parameters, consider using the more general `once::run` action with `ResMut<NextState<S>>` parameter.