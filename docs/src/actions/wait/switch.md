# wait::switch

The `wait::switch` module provides actions for waiting for [`Switch`](../../actions/switch.md) state changes. These actions are useful for coordinating between different parts of your application, allowing tasks to wait for specific switch state changes before continuing execution.

## Functions

### on

```rust
wait::switch::on<M>() -> ActionSeed
```

Creates an action that waits until a switch of type `M` is turned on. The action completes when the switch is detected as on.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Define a marker type for our switch
struct Animation;

Reactor::schedule(|task| async move {
    // Wait until the Animation switch is turned on
    task.will(Update, wait::switch::on::<Animation>()).await;
    
    // This code runs after the Animation switch is turned on
    println!("Animation switch is now on!");
});
```

### off

```
wait::switch::off<M>() -> ActionSeed
```

Creates an action that waits until a switch of type `M` is turned off. The action completes when the switch is detected as off.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Define a marker type for our switch
struct Animation;

Reactor::schedule(|task| async move {
    // First turn on the Animation switch
    task.will(Update, once::switch::on::<Animation>()).await;
    
    // Then wait until the Animation switch is turned off
    task.will(Update, wait::switch::off::<Animation>()).await;
    
    // This code runs after the Animation switch is turned off
    println!("Animation switch is now off!");
});
```

## When to Use

Use `wait::switch` actions when you need to:
- Coordinate between different systems in your application
- Wait for a specific state change before continuing execution
- Create state machines with clear on/off states
- Implement gameplay sequences that depend on switch states

Switches are particularly useful for tasks that need to be performed on the main thread but need to be coordinated with asynchronous Reactor tasks.