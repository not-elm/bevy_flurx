# Once Actions

Once actions are actions that run a system exactly once and then complete. 
They are useful for one-time operations like sending events, modifying resources, or performing state transitions.

## Available Once Actions

- [run](run.md) - Run a system once
- [event](event.md) - Send an event once
- [res](res.md) - Modify a resource once
- [non_send](non_send.md) - Modify a non-send resource once
- [switch](switch.md) - Switch between actions based on a condition
- [state](state.md) - Transition to a new state once
- [audio](audio.md) - Play audio once

Each action is designed to be used with the `Reactor::schedule` or `task.will` methods to schedule a one-time operation.

## Basic Usage

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Run a system once during the Update schedule
Reactor::schedule(|task| async move {
    task.will(Update, once::run(|world: &mut World| {
        // Do something once
    })).await;
});
```

The once actions are part of the core functionality of bevy_flurx, providing simple building blocks for more complex action sequences.
