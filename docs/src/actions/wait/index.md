# Wait Actions

Wait actions are tasks that continue to execute every frame according to specified conditions. They are useful for creating reactive behaviors that respond to events, input, state changes, or other conditions.

## Available Wait Actions

- [until](until.md) - Wait until a condition is true
- [output](output.md) - Wait until a system returns Some value
- [event](event.md) - Wait for events
- [switch](switch.md) - Wait for switch state changes
- [state](state.md) - Wait for state transitions
- [audio](audio.md) - Wait for audio playback to finish
- [input](input.md) - Wait for input events
- [all](all.md) - Wait for all actions to complete
- [any](any.md) - Wait for any action to complete
- [both](both.md) - Wait for two actions to complete
- [either](either.md) - Wait for either of two actions to complete

Each action is designed to be used with the `Reactor::schedule` or `task.will` methods to create tasks that wait for specific conditions before continuing.

## Basic Usage

```
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Wait until a condition is met
Reactor::schedule(|task| async move {
    task.will(Update, wait::until(|mut count: Local<usize>| {
        *count += 1;
        *count == 4  // Wait until this condition is true
    })).await;

    // This code runs after the condition is met
    println!("Condition met!");
});
```

The wait actions are part of the core functionality of bevy_flurx, providing powerful tools for creating reactive and event-driven behaviors in your Bevy applications.
