# wait::output

The `wait::output` action continues to execute a system every frame until it returns `Option::Some`. The contents of `Some` will be the return value of the task. This action provides more flexibility than `wait::until` by allowing you to return a value when the waiting condition is met.

## Usage

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    let result = task.will(Update, wait::output(|world: &mut World| {
        // Your logic here
        Some("Result value") // Return Some when condition is met
    })).await;
    
    println!("Got result: {}", result);
});
```

## Parameters

- `system`: Any valid Bevy system that returns an `Option<T>`

## Return Value

The action returns the value inside the `Some` variant when the system returns `Some`.

## Example: Waiting for a Counter with Result

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    let count = task.will(Update, wait::output(|mut count: Local<usize>| {
        *count += 1;
        if *count == 4 {
            Some(*count)  // Return the count when it reaches 4
        } else {
            None
        }
    })).await;
    
    // This code runs after the counter reaches 4
    println!("Counter reached: {}", count);
});
```

## Example: Finding an Entity

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Component)]
struct Target;

Reactor::schedule(|task| async move {
    let entity = task.will(Update, wait::output(|query: Query<Entity, With<Target>>| {
        query.iter().next().map(|e| e)  // Return the first entity with Target component
    })).await;
    
    // This code runs after a Target entity is found
    println!("Found target entity: {:?}", entity);
});
```

## When to Use

Use `wait::output` when you need to:
- Wait for a condition and get a value when the condition is met
- Find an entity or resource that meets certain criteria
- Collect data that becomes available at an unpredictable time
- Implement custom waiting logic that needs to return a value

For simpler cases where you just need to wait for a condition without returning a value, consider using `wait::until` instead.