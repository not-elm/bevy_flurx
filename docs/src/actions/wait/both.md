# wait::both

The `wait::both` module provides an action for waiting for two specific actions to complete. This action is useful for coordinating tasks that require exactly two conditions to be met before continuing execution.

## Functions

### both

```rust
wait::both<LI, LO, RI, RO>(lhs: impl Into<Action<LI, LO>> + 'static, rhs: impl Into<Action<RI, RO>> + 'static) -> Action<(LI, RI), (LO, RO)>
```

Creates an action that waits until both tasks are done. The output value is a tuple containing the outputs from both actions.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait for both the R key to be pressed and an AppExit event to be received
    let (_, exit_event) = task.will(Update, wait::both(
        wait::input::just_pressed().with(KeyCode::KeyR),
        wait::event::read::<AppExit>()
    )).await;
    
    // This code runs after both conditions are met
    println!("R key was pressed and AppExit event received: {:?}", exit_event);
});
```

## When to Use

Use `wait::both` when you need to:
- Wait for exactly two specific conditions to be met
- Collect the results from two different asynchronous operations
- Coordinate between two different parts of your game
- Create synchronization points that depend on two specific events or inputs
- Implement simple "AND" logic for exactly two waiting conditions

For waiting on more than two conditions, consider using `wait::all` or the `wait_all!` macro instead.