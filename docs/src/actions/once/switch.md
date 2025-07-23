# once::switch

The `once::switch` module provides actions for controlling Bevy switches exactly once. Switches are a mechanism in bevy_flurx that represent two states (on and off) and can be used to coordinate between Reactors and regular Bevy systems.

## Functions

### on

```rust
once::switch::on<M>() -> ActionSeed
```

Creates an action that turns a switch on. If the switch doesn't exist, it will be created.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Define a marker type for our switch
struct PlayerAnimation;

Reactor::schedule(|task| async move {
    // Turn on the PlayerAnimation switch
    task.will(Update, once::switch::on::<PlayerAnimation>()).await;
});
```

### off

```
once::switch::off<M>() -> ActionSeed
```

Creates an action that turns a switch off. If the switch doesn't exist, it will be created in the off state.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Define a marker type for our switch
struct PlayerAnimation;

Reactor::schedule(|task| async move {
    // Turn off the PlayerAnimation switch
    task.will(Update, once::switch::off::<PlayerAnimation>()).await;
});
```

## Using Switches with Systems

Switches are designed to be used with Bevy's `run_if` condition system to control when systems run:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct HeavyTask;

fn main() {
    App::new()
        // This system only runs when the HeavyTask switch is on
        .add_systems(Update, (|mut switch: ResMut<Switch<HeavyTask>>| {
            // Do heavy work...

            // Turn off the switch when done
            switch.off();
        }).run_if(switch_is_on::<HeavyTask>))

        // Spawn a reactor that turns the switch on and waits for it to be turned off
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::switch::on::<HeavyTask>()).await;
                task.will(Update, wait::switch::off::<HeavyTask>()).await;
                // Continue after the heavy task is complete...
            }));
        });
}
```

## Available Conditions

The following conditions can be used with `run_if`:

- `switch_is_on<M>()` - Returns true if the switch is on
- `switch_is_off<M>()` - Returns true if the switch is off
- `switch_just_turned_on<M>()` - Returns true only the first time the switch is detected as on
- `switch_just_turned_off<M>()` - Returns true only the first time the switch is detected as off

## When to Use

Use `once::switch` actions when you need to:
- Coordinate between Reactors and regular Bevy systems
- Control when certain systems should run
- Signal the completion of asynchronous tasks
- Create state machines with clear on/off states

Switches are particularly useful for tasks that need to be performed on the main thread but need to be coordinated with asynchronous Reactor tasks.
