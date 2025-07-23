# map

The `map` module provides mechanisms to transform the output of an action using a mapping function. This is particularly useful for data transformation and type conversion between actions.

## Basic Usage

The `map` module provides the `Map` trait, which adds two main methods to all actions:

1. `map`: Transforms the output of an action by applying a function to it
2. `overwrite`: Replaces the output of an action with a specified value

### Using the `map` Method

Use the `map()` method to transform the output of an action:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Transform the output of an action
        let result = task.will(Update, 
            once::run(|| 5)
                .map(|num| num * 2)
        ).await;

        println!("{}", result); // Prints "10"
    }));
}
```

### Using the `overwrite` Method

Use the `overwrite()` method to replace the output of an action with a specified value:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Replace the output of an action
        let result = task.will(Update, 
            once::run(|| "Original output")
                .overwrite("Replaced output")
        ).await;

        println!("{}", result); // Prints "Replaced output"
    }));
}
```

## How It Works

When using the `map` or `overwrite` methods:

1. The original action is executed until completion
2. The output of the action is transformed using the provided function (for `map`) or replaced with the specified value (for `overwrite`)
3. The transformed or replaced value becomes the output of the combined action

This allows for flexible data transformation and type conversion between actions.

## Practical Examples

### Type Conversion

The `map` module is particularly useful for converting between different types:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Convert a number to a string
        let result = task.will(Update, 
            once::run(|| 42)
                .map(|num| format!("The answer is: {}", num))
        ).await;

        println!("{}", result); // Prints "The answer is: 42"
    }));
}
```

### Data Transformation in Pipelines

The `map` module can be combined with `pipe` to create powerful data transformation pipelines:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create a data transformation pipeline
        let result = task.will(Update, 
            once::run(|| 5)
                .map(|num| num * 2) // Transform to 10
                .pipe(once::run(|In(num): In<i32>| {
                    let squared = num * num;
                    squared
                })) // Transform to 100
                .map(|num| format!("Result: {}", num)) // Transform to string
        ).await;

        println!("{}", result); // Prints "Result: 100"
    }));
}
```

### Conditional Logic

The `map` method can be used to implement conditional logic:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Implement conditional logic
        let score = task.will(Update, once::run(|| 85)).await;
        
        let grade = task.will(Update, 
            once::run(|| score)
                .map(|score| {
                    if score >= 90 { "A" }
                    else if score >= 80 { "B" }
                    else if score >= 70 { "C" }
                    else if score >= 60 { "D" }
                    else { "F" }
                })
        ).await;

        println!("Score: {}, Grade: {}", score, grade); // Prints "Score: 85, Grade: B"
    }));
}
```