# side_effect::thread

The `side_effect::thread` module provides actions for spawning OS threads. These actions allow you to execute CPU-intensive code without blocking the main thread while maintaining the Action-based flow.

## Functions

### spawn

```rust
side_effect::thread::spawn<I, O, M>(f: impl Functor<I, O, M> + Send + Sync + 'static) -> ActionSeed<I, O>
```

Spawns a new OS thread and waits for its output. The thread is started when the Runner is executed for the first time. Note that the thread created from this function will continue to run even if the Reactor is canceled.

#### Parameters

- `f`: A function to be executed on a separate thread. This can be either:
  - A function that takes input: `|input| { ... }`
  - A function without input: `|| { ... }`

#### Return Value

Returns an `ActionSeed<I, O>` that, when executed, will spawn a new OS thread to execute the function and wait for it to complete.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Execute a function with input on a separate thread
        let result = task.will(Update, 
            once::run(|| 5)
                .pipe(side_effect::thread::spawn(|num| {
                    // This runs on a separate OS thread
                    num * 2
                }))
        ).await;
        
        println!("Result: {}", result); // Prints "Result: 10"
        
        // Execute a function without input on a separate thread
        let result = task.will(Update, 
            side_effect::thread::spawn(|| {
                // This runs on a separate OS thread
                "Hello from thread!"
            })
        ).await;
        
        println!("Result: {}", result); // Prints "Result: Hello from thread!"
    }));
}
```

## When to Use

Use `side_effect::thread` actions when you need to:
- Execute CPU-intensive operations without blocking the main thread
- Perform operations that would otherwise slow down the main game loop
- Execute code that should continue even if the Reactor is canceled

The `thread` module is particularly useful for operations that are CPU-bound rather than I/O-bound. For I/O-bound operations, consider using the `tokio` module instead.

## Feature Requirements

The `thread` module requires both the `side-effect` and `std` feature flags to be enabled. It is not available on WebAssembly targets.

```toml
[dependencies]
bevy_flurx = { version = "0.1", features = ["side-effect", "std"] }
```