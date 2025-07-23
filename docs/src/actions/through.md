# through

The `through` module provides a mechanism to execute an action while preserving the output of the previous action. This is particularly useful for inserting actions like delays into a pipeline without affecting the data flow.

## Basic Usage

The `through` module provides two main ways to use the functionality:

1. The `through` function: Creates an action that executes a provided action but preserves the input value as the output
2. The `Through` trait: Adds a convenient `.through()` method to actions, simplifying the chaining of actions

### Using the `through` Function

Use the `through()` function to create an action that preserves its input:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

#[derive(Event, Clone)]
struct Damage(usize);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Use through to insert a delay without affecting the data flow
        task.will(Update, wait::event::read::<Damage>()
            .pipe(through(delay::time().with(Duration::from_millis(500))))
            .pipe(once::run(|In(Damage(damage)): In<Damage>| {
                println!("Player takes {damage} points of damage.");
            }))
        ).await;
    }));
}
```

### Using the `Through` Trait

Use the `.through()` method for a more concise syntax:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

#[derive(Event, Clone)]
struct Damage(usize);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Use the through method for a more concise syntax
        task.will(Update, wait::event::read::<Damage>()
            .through(delay::time().with(Duration::from_millis(500)))
            .pipe(once::run(|In(Damage(damage)): In<Damage>| {
                println!("Player takes {damage} points of damage.");
            }))
        ).await;
    }));
}
```

## How It Works

When using the `through` function or the `Through` trait:

1. The original input value is stored
2. The provided action is executed until completion
3. The original input value is then forwarded as the output, regardless of the output of the executed action

This allows you to insert actions into a pipeline without affecting the data flow, which is particularly useful for actions like delays or logging.

## Practical Examples

### Adding Delays

The `through` module is particularly useful for adding delays without affecting the data flow:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Add a delay between steps without affecting the data
        let result = task.will(Update, 
            once::run(|| "Processing...")
                .through(delay::seconds(1.0)) // Wait for 1 second
                .pipe(once::run(|In(text): In<&str>| {
                    format!("{} Completed!", text)
                }))
        ).await;

        println!("{}", result); // Prints "Processing... Completed!"
    }));
}
```

### Logging Without Affecting Data Flow

The `through` module can be used for logging without affecting the data flow:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Log data without affecting the pipeline
        let result = task.will(Update, 
            once::run(|| 42)
                .through(once::run(|In(num): In<i32>| {
                    println!("Processing number: {}", num);
                }))
                .pipe(once::run(|In(num): In<i32>| num * 2))
        ).await;

        println!("Result: {}", result); // Prints "Result: 84"
    }));
}
```

### Combining with Other Action Types

The `through` method can be combined with other action types for more complex behaviors:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

#[derive(Event, Clone)]
struct PlayerAction {
    action_type: String,
    value: i32,
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create a complex action flow with through
        task.will(Update, 
            wait::event::read::<PlayerAction>()
                .through(once::run(|In(action): In<PlayerAction>| {
                    println!("Received action: {} with value {}", action.action_type, action.value);
                }))
                .through(delay::time().with(Duration::from_millis(500))) // Add a delay
                .pipe(once::run(|In(action): In<PlayerAction>| {
                    // Process the action
                    action.value * 2
                }))
        ).await;
    }));
}
```