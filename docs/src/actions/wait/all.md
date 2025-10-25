# wait::all

The `wait::all` module provides actions for waiting for multiple actions to complete. These actions are useful for coordinating complex tasks that require multiple conditions to be met before continuing execution.

## Functions

### all

```rust
wait::all<Actions>() -> ActionSeed<Actions>
```

Creates an action that waits until all the specified actions are completed. The output value of this function is `()`.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use bevy_flurx::actions;
use core::time::Duration;

Reactor::schedule(|task| async move {
    // Wait until all three conditions are met
    task.will(Update, wait::all().with(actions![
        once::run(||{}),
        delay::time().with(Duration::from_millis(300)),
        wait::input::just_pressed().with(KeyCode::KeyA)
    ])).await;
    
    // This code runs after all actions are completed
    println!("All actions completed!");
});
```

### wait_all! (macro)

```
wait_all![action1, action2, ...]
```

A macro that waits until all tasks are done. The return value type is a tuple, with its length equal to the number of passed tasks. This is similar to `wait::all()`, but it returns the outputs of each action.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use bevy_flurx::wait_all;

#[derive(Default, Clone, Event, PartialEq, Debug)]
struct Event1;
#[derive(Default, Clone, Event, PartialEq, Debug)]
struct Event2;

Reactor::schedule(|task| async move {
    // Wait for both events and get their values
    let (event1, event2) = task.will(Update, wait_all![
        wait::message::read::<Event1>(),
        wait::message::read::<Event2>()
    ]).await;
    
    // This code runs after both events are received
    println!("Received events: {:?}, {:?}", event1, event2);
});
```

## When to Use

Use `wait::all` and `wait_all!` when you need to:
- Wait for multiple conditions to be met before continuing execution
- Coordinate complex initialization sequences
- Gather results from multiple asynchronous operations
- Create synchronization points in your game flow
- Implement "AND" logic for multiple waiting conditions

For "OR" logic (waiting for any of multiple conditions), consider using `wait::any` instead.