# side_effect::bevy_task

The `side_effect::bevy_task` module provides actions for spawning tasks using Bevy's task system. These actions allow you to execute asynchronous code while maintaining the Action-based flow.

## Functions

### spawn

```rust
side_effect::bevy_task::spawn<I, Out, Functor, M>(f: Functor) -> ActionSeed<I, Out>
```

Spawns a future onto the Bevy thread pool and waits until it's completed. The task is started when the Runner is executed for the first time.

#### Parameters

- `f`: A function that returns a future, or a future itself. This can be either:
  - A function that takes input and returns a future: `|input| async move { ... }`
  - A future directly: `async move { ... }`

#### Return Value

Returns an `ActionSeed<I, Out>` that, when executed, will spawn the future onto the Bevy thread pool and wait for it to complete.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Spawn a future directly
        task.will(Update, side_effect::bevy_task::spawn(async move {
            // This runs on the Bevy thread pool
            "Hello from Bevy task!"
        })).await;
        
        // Spawn a function that takes input and returns a future
        let result = task.will(Update, 
            once::run(|| 5)
                .pipe(side_effect::bevy_task::spawn(|num| async move {
                    // This runs on the Bevy thread pool
                    num * 2
                }))
        ).await;
        
        println!("Result: {}", result); // Prints "Result: 10"
    }));
}
```

### spawn_detached

```rust
side_effect::bevy_task::spawn_detached<I, Out, Functor, M>(functor: Functor) -> ActionSeed<I, Out>
```

Spawns a future onto the Bevy thread pool and waits until it's completed. Unlike `spawn`, the spawned task is detached and continues to run in the background, even if the Reactor is canceled.

#### Parameters

- `functor`: A function that returns a future, or a future itself. This can be either:
  - A function that takes input and returns a future: `|input| async move { ... }`
  - A future directly: `async move { ... }`

#### Return Value

Returns an `ActionSeed<I, Out>` that, when executed, will spawn the future onto the Bevy thread pool as a detached task and wait for it to complete.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Spawn a detached future
        task.will(Update, side_effect::bevy_task::spawn_detached(async move {
            // This runs on the Bevy thread pool and continues even if the Reactor is canceled
            "Hello from detached Bevy task!"
        })).await;
        
        // Spawn a function that takes input and returns a future
        let result = task.will(Update, 
            once::run(|| 5)
                .pipe(side_effect::bevy_task::spawn_detached(|num| async move {
                    // This runs on the Bevy thread pool and continues even if the Reactor is canceled
                    num * 2
                }))
        ).await;
        
        println!("Result: {}", result); // Prints "Result: 10"
    }));
}
```

## When to Use

Use `side_effect::bevy_task` actions when you need to:
- Execute asynchronous code that would block the main thread
- Perform operations that can benefit from Bevy's task system
- Execute code that should continue even if the Reactor is canceled (using `spawn_detached`)

The `bevy_task` module is particularly useful for operations that need to be executed asynchronously but don't require the full power of Tokio's runtime.