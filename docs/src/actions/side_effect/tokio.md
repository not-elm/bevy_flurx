# side_effect::tokio

The `side_effect::tokio` module provides actions for spawning tasks using Tokio's runtime. These actions allow you to execute asynchronous code with Tokio while maintaining the Action-based flow.

## Functions

### spawn

```rust
side_effect::tokio::spawn<I, Out, Functor, M>(f: Functor) -> ActionSeed<I, Out>
```

Spawns a new Tokio task and waits for its output. The task is started when the Runner is executed for the first time.

#### Parameters

- `f`: A function that returns a future, or a future itself. This can be either:
  - A function that takes input and returns a future: `|input| async move { ... }`
  - A future directly: `async move { ... }`

#### Return Value

Returns an `ActionSeed<I, Out>` that, when executed, will spawn a new Tokio task to execute the future and wait for it to complete.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Execute an asynchronous operation using Tokio
        let result = task.will(Update, 
            side_effect::tokio::spawn(async move {
                // This runs on Tokio's runtime
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                "Operation completed"
            })
        ).await;
        
        println!("Result: {}", result); // Prints "Result: Operation completed"
        
        // Execute a function that takes input and returns a future
        let result = task.will(Update, 
            once::run(|| 5)
                .pipe(side_effect::tokio::spawn(|num| async move {
                    // This runs on Tokio's runtime
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    num * 2
                }))
        ).await;
        
        println!("Result: {}", result); // Prints "Result: 10"
    }));
}
```

## Cancellation Behavior

Unlike `bevy_task::spawn_detached` and `thread::spawn`, Tokio tasks spawned with `tokio::spawn` are aborted when the Runner is dropped. This means that if the Reactor is canceled, the Tokio task will also be canceled.

## When to Use

Use `side_effect::tokio` actions when you need to:
- Perform I/O-bound operations like network requests or file operations
- Execute asynchronous code that benefits from Tokio's runtime features
- Integrate with other libraries that use Tokio

The `tokio` module is particularly useful for operations that are I/O-bound rather than CPU-bound. For CPU-bound operations, consider using the `thread` module instead.

## Feature Requirements

The `tokio` module requires the `tokio` feature flag to be enabled.

```toml
[dependencies]
bevy_flurx = { version = "0.1", features = ["tokio"] }
```