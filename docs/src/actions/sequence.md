# sequence

The `sequence` module provides mechanisms for sequentially combining actions. This is particularly useful for creating complex action flows by chaining multiple actions together.

## Basic Usage

The `sequence` module provides two main ways to combine actions:

1. The `Then` trait: Allows actions to be combined using the `then` method
2. The `sequence!` macro: Provides an alternative syntax for combining actions

### Using the `then` Method

Use the `then()` method to chain actions together:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Chain actions using the then method
        let result = task.will(Update, 
            once::run(|| {})
                .then(once::run(|| "Hello"))
                .then(once::run(|In(text): In<&str>| format!("{}, World!", text)))
        ).await;

        println!("{}", result); // Prints "Hello, World!"
    }));
}
```

### Using the `sequence!` Macro

Use the `sequence!` macro to combine actions with a more declarative syntax:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use bevy_flurx::sequence;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Chain actions using the sequence! macro
        let result = task.will(Update, 
            sequence![
                once::run(|| {}),
                once::run(|| "Hello"),
                once::run(|In(text): In<&str>| format!("{}, World!", text)),
            ]
        ).await;

        println!("{}", result); // Prints "Hello, World!"
    }));
}
```

## How It Works

When actions are combined using `then` or the `sequence!` macro:

1. The actions are executed in sequence
2. Each action starts as soon as the previous one completes
3. If multiple actions complete in the same frame, they will all execute in that frame
4. The output of the combined action will be that of the last action in the sequence

## Practical Examples

### Creating a Multi-step Process

The `sequence` module is particularly useful for creating multi-step processes:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create a multi-step process
        task.will(Update, 
            once::run(|| println!("Step 1: Initializing..."))
                .then(once::run(|| println!("Step 2: Processing...")))
                .then(once::run(|| println!("Step 3: Finalizing...")))
                .then(once::run(|| println!("Process completed!")))
        ).await;
    }));
}
```

### Combining Different Types of Actions

The `sequence` module can be used to combine different types of actions:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Combine different types of actions
        task.will(Update, 
            once::run(|| println!("Waiting for key press..."))
                .then(wait::input::key_pressed(KeyCode::Space))
                .then(once::run(|| println!("Key pressed! Waiting 2 seconds...")))
                .then(delay::seconds(2.0))
                .then(once::run(|| println!("Sequence completed!")))
        ).await;
    }));
}
```
