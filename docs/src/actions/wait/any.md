# wait::any

The `wait::any` module provides actions for waiting for any of multiple actions to complete. These actions are useful for creating tasks that can proceed when any of several conditions are met, allowing for more flexible control flow.

## Functions

### any

```rust
wait::any<Actions>() -> ActionSeed<Actions, usize>
```

Creates an action that waits until the execution of one of the actions is completed. The output value is the index of the completed action.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use bevy_flurx::actions;
use bevy::app::AppExit;

Reactor::schedule(|task| async move {
    // Wait until either the B key is pressed or an AppExit event is received
    let index = task.will(Update, wait::any().with(actions![
        wait::input::just_pressed().with(KeyCode::KeyB),
        wait::message::comes::<AppExit>()
    ])).await;
    
    // Check which action completed
    match index {
        0 => println!("B key was pressed!"),
        1 => println!("AppExit event received!"),
        _ => unreachable!(),
    }
});
```

## When to Use

Use `wait::any` when you need to:
- Wait for the first of several conditions to be met
- Create branching logic based on which condition occurs first
- Implement timeout patterns with alternative success paths
- Handle multiple possible user inputs or events
- Implement "OR" logic for multiple waiting conditions

For "AND" logic (waiting for all conditions to be met), consider using `wait::all` instead.