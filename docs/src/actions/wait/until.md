# wait::until

The `wait::until` action continues to execute a system every frame until it returns `true`. This is one of the most basic and versatile wait actions, allowing you to wait for any condition to be met.

## Usage

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    task.will(Update, wait::until(|world: &mut World| {
        // Your condition here
        true // Replace with your actual condition
    })).await;
});
```

## Parameters

- `system`: Any valid Bevy system that returns a boolean value

## Return Value

The action returns `()` (unit) when the condition becomes true.

## Example: Waiting for a Counter

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    task.will(Update, wait::until(|mut count: Local<usize>| {
        *count += 1;
        *count == 4  // Wait until the counter reaches 4
    })).await;
    
    // This code runs after the counter reaches 4
    println!("Counter reached 4!");
});
```

## Example: Waiting for a Resource Value

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Resource, Default)]
struct Score(u32);

Reactor::schedule(|task| async move {
    task.will(Update, wait::until(|score: Res<Score>| {
        score.0 >= 100  // Wait until the score is at least 100
    })).await;
    
    // This code runs after the score reaches 100
    println!("Score reached 100!");
});
```

## When to Use

Use `wait::until` when you need to:
- Wait for a specific condition to be met
- Poll a value until it reaches a threshold
- Create a delay based on a custom condition
- Implement custom waiting logic that isn't covered by other wait actions

For more specific waiting scenarios, consider using the specialized wait actions like `wait::event` or `wait::input`.