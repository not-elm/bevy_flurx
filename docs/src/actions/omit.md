# omit

The `omit` module provides mechanisms to omit input and/or output types from an action. This is particularly useful for defining groups of actions by simplifying their type signatures.

## Basic Usage

The `omit` module provides three main traits:

1. `Omit`: Omits both input and output types from an action
2. `OmitOutput`: Omits only the output type from an action
3. `OmitInput`: Omits only the input type from an action

### Omitting Both Input and Output

Use the `omit()` method to omit both input and output types from an action:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        task.will(Update, action()).await;
    }));
}

fn action() -> ActionSeed {
    once::run(|In(num): In<usize>| num)
        .with(1) // ActionSeed<usize, usize>
        .omit() // ActionSeed<(), ()>
}
```

### Omitting Only Output

Use the `omit_output()` method to omit only the output type from an action:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create an action that omits only the output type
        task.will(Update, once::run(|In(num): In<usize>| {
                format!("Number: {}", num)
            })
            .with(42)
            .omit_output()
            .pipe(once::run(|| {
                println!("Action completed!");
            }))
        ).await;
    }));
}
```

### Omitting Only Input

Use the `omit_input()` method to omit only the input type from an action:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create an action that omits only the input type
        let result: usize = task.will(Update, once::run(|In(num): In<usize>| num)
            .with(5)
            .omit_input()
        ).await;
        
        println!("Result: {}", result); // Prints "Result: 5"
    }));
}
```

## Practical Examples

### Creating Reusable Action Groups

The `omit` module is particularly useful for creating reusable action groups with simplified type signatures:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Define a reusable action that doesn't expose its internal input/output types
fn print_message() -> ActionSeed {
    once::run(|In(message): In<String>| {
        println!("{}", message);
    })
    .with("Hello, world!".to_string())
    .omit()
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Use the reusable action
        task.will(Update, print_message()).await;
    }));
}
```

### Chaining Actions with Different Types

The `omit` traits can be used to chain actions with different input/output types:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Chain actions with different types
        task.will(Update, once::run(|In(num): In<usize>| num * 2)
            .with(3)
            .omit_output() // Discard the output
            .pipe(once::run(|| "Action completed!"))
        ).await;
    }));
}
```