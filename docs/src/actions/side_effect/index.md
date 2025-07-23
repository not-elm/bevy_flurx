# Side Effect Actions

The `side_effect` module provides actions for handling operations with side effects such as asynchronous runtime or threads. These actions allow you to execute code outside the main Bevy ECS system while maintaining the Action-based flow.

## Available Side Effect Actions

- [bevy_task](bevy_task.md) - Spawn tasks using Bevy's task system
- [thread](thread.md) - Spawn OS threads
- [tokio](tokio.md) - Spawn tasks using Tokio's runtime

## Basic Usage

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Execute a CPU-intensive operation on a separate thread
        let result = task.will(Update, 
            once::run(|| 2)
                .pipe(side_effect::thread::spawn(|num| {
                    // This runs on a separate thread
                    num * 10
                }))
        ).await;
        
        println!("Result: {}", result); // Prints "Result: 20"
        
        // Execute an asynchronous operation using Tokio
        task.will(Update, 
            side_effect::tokio::spawn(async move {
                // This runs on Tokio's runtime
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                "Operation completed"
            })
        ).await;
    }));
}
```

## Core Concepts

### AsyncFunctor

The `AsyncFunctor` trait is implemented for functions that return futures. It allows you to:
- Pass a function that takes input and returns a future: `spawn(|input| async move { ... })`
- Pass a future directly: `spawn(async move { ... })`

### Functor

The `Functor` trait is used for functions that need to be executed with side effects. It allows you to:
- Pass a function that takes input: `spawn(|input| { ... })`
- Pass a function without input: `spawn(|| { ... })`

## When to Use

Use side_effect actions when you need to:
- Execute CPU-intensive operations without blocking the main thread
- Perform asynchronous operations like network requests or file I/O
- Integrate with external asynchronous APIs
- Execute code that would otherwise block or slow down the main Bevy ECS system

The side_effect module is particularly useful for applications that need to perform operations outside the main game loop, such as loading assets, making network requests, or performing complex calculations.