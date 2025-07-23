# remake

The `remake` module provides a mechanism to create a new action based on an existing action's `Runner` and `Output`. This is particularly useful for transforming actions while preserving their input type but changing their output type.

## Basic Usage

The `remake` module provides the `Remake` trait, which adds the `remake` method to all actions:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create a new action with a different output type
        let result = task.will(Update, 
            once::run(|| 5)
                .remake(|runner, original_output, new_output| {
                    // Custom runner that uses the original runner but produces a different output
                    CustomRunner {
                        runner,
                        original_output,
                        new_output,
                    }
                })
        ).await;

        println!("{}", result);
    }));
}

// A custom runner that transforms the output
struct CustomRunner<O1, O2> {
    runner: BoxedRunner,
    original_output: Output<O1>,
    new_output: Output<O2>,
}

impl<O1, O2> Runner for CustomRunner<O1, O2>
where
    O1: 'static,
    O2: 'static,
{
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        // Run the original runner
        let result = self.runner.run(world, token);
        
        // If the original runner completed, transform its output
        if result == RunnerIs::Completed {
            if let Some(value) = self.original_output.take() {
                // Transform the output (example: convert a number to a string)
                let transformed = format!("Value: {}", value);
                self.new_output.set(transformed);
            }
        }
        
        result
    }
}
```

## How It Works

When using the `remake` method:

1. The original action's `Runner` and `Output` are captured
2. A new `Runner` is created using the provided function, which receives:
   - The original `Runner`
   - The original `Output`
   - A new `Output` for the transformed action
3. The new `Runner` can use the original runner's behavior while producing a different output type

This allows for advanced customization of action behavior while maintaining type safety.

## Practical Examples

### Type Conversion

The `remake` module can be used for complex type conversions that require access to the original runner:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Convert a number action to a string action with custom processing
        let result = task.will(Update, 
            once::run(|| 42)
                .remake(|runner, original_output, new_output| {
                    TypeConverter {
                        runner,
                        original_output,
                        new_output,
                    }
                })
        ).await;

        println!("{}", result); // Prints a string representation of 42
    }));
}

struct TypeConverter<O1, O2> {
    runner: BoxedRunner,
    original_output: Output<O1>,
    new_output: Output<O2>,
}

impl<O1, O2> Runner for TypeConverter<O1, O2>
where
    O1: std::fmt::Display + 'static,
    O2: From<String> + 'static,
{
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        let result = self.runner.run(world, token);
        
        if result == RunnerIs::Completed {
            if let Some(value) = self.original_output.take() {
                let string_value = format!("{}", value);
                self.new_output.set(O2::from(string_value));
            }
        }
        
        result
    }
}
```

### Advanced Action Composition

The `remake` module can be used to create complex action compositions:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create a composite action that logs its execution time
        let result = task.will(Update, 
            once::run(|| "Processing data...")
                .remake(|runner, original_output, new_output| {
                    TimedRunner {
                        runner,
                        original_output,
                        new_output,
                        start_time: None,
                    }
                })
        ).await;

        println!("{}", result); // Prints the result with timing information
    }));
}

struct TimedRunner<O> {
    runner: BoxedRunner,
    original_output: Output<O>,
    new_output: Output<String>,
    start_time: Option<std::time::Instant>,
}

impl<O> Runner for TimedRunner<O>
where
    O: std::fmt::Display + 'static,
{
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        if self.start_time.is_none() {
            self.start_time = Some(std::time::Instant::now());
        }
        
        let result = self.runner.run(world, token);
        
        if result == RunnerIs::Completed {
            if let Some(value) = self.original_output.take() {
                let elapsed = self.start_time.unwrap().elapsed();
                let timed_result = format!("Result: {} (took {:?})", value, elapsed);
                self.new_output.set(timed_result);
            }
        }
        
        result
    }
}
```

## When to Use

Use the `remake` module when you need to:

- Transform an action's output type in ways that can't be achieved with `map`
- Access and modify the behavior of an action's runner
- Create custom runners that build upon existing actions
- Implement advanced action composition patterns

The `remake` module is particularly useful for library authors and advanced users who need fine-grained control over action behavior and transformation.