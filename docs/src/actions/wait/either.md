# wait::either

The `wait::either` module provides an action for waiting for either of two actions to complete. This action is useful for creating tasks that can proceed when either of two conditions is met, allowing for more flexible control flow.

## Types

### Either

```rust
enum Either<L, R> {
    Left(L),
    Right(R),
}
```

This enum represents the result of `wait::either`. It contains either the result of the first action (`Left`) or the result of the second action (`Right`).

#### Methods

- `is_left()` - Returns true if the value is `Left`
- `is_right()` - Returns true if the value is `Right`

## Functions

### either

```rust
wait::either<LI, LO, RI, RO>(lhs: impl Into<Action<LI, LO>> + 'static, rhs: impl Into<Action<RI, RO>> + 'static) -> Action<(LI, RI), Either<LO, RO>>
```

Creates an action that waits until either of the two tasks is completed. The output value is an `Either<LO, RO>` enum that indicates which action completed and contains its output.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait until either a system returns false or an AppExit event is received
    let result = task.will(Update, wait::either(
        wait::until(|| false),
        wait::event::read::<AppExit>()
    )).await;
    
    match result {
        Either::Left(_) => println!("System returned false"),
        Either::Right(exit_event) => println!("AppExit event received: {:?}", exit_event),
    }
});
```

## When to Use

Use `wait::either` when you need to:
- Wait for the first of two specific conditions to be met
- Create branching logic based on which of two conditions occurs first
- Implement timeout patterns with a success path
- Handle two possible user inputs or events
- Implement simple "OR" logic for exactly two waiting conditions

For waiting on more than two conditions, consider using `wait::any` instead. For waiting on both conditions, use `wait::both`.