# switch

The `switch` module provides a mechanism for coordinating between Reactors and regular Bevy systems through a binary state (on/off). 

## Core Concepts

### Switch Resource

The `Switch<M>` resource represents a binary state (on/off) that can be used to coordinate between different parts of your application. The generic type parameter `M` allows you to define different types of switches for different purposes.

```rust
// Define a marker type for our switch
struct PlayerAnimation;

// Access the switch in a system
fn check_animation_state(switch: Res<Switch<PlayerAnimation>>) {
    if switch.is_on() {
        println!("Player animation is running!");
    } else {
        println!("Player animation is stopped!");
    }
}
```

### Condition Systems

The switch module provides several condition systems that can be used with Bevy's `run_if` functionality to conditionally run systems based on switch states:

- `switch_is_on<M>()` - Returns true if the switch is on
- `switch_is_off<M>()` - Returns true if the switch is off
- `switch_just_turned_on<M>()` - Returns true only the first time the switch is detected as on
- `switch_just_turned_off<M>()` - Returns true only the first time the switch is detected as off

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
        
        // This system only runs when the HeavyTask switch just turned off
        .add_systems(Update, (|| {
            println!("Heavy task just completed!");
        }).run_if(switch_just_turned_off::<HeavyTask>));
}
```

## Available Actions

The switch module provides actions in both the `once` and `wait` modules:

### once::switch

- [once::switch::on](once/switch.md#on) - Turn a switch on
- [once::switch::off](once/switch.md#off) - Turn a switch off

### wait::switch

- [wait::switch::on](wait/switch.md#on) - Wait until a switch is turned on
- [wait::switch::off](wait/switch.md#off) - Wait until a switch is turned off

## Examples

### Coordinating Between Reactors and Systems

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct LoadingTask;

fn main() {
    App::new()
        // This system performs a heavy loading task when the switch is on
        .add_systems(Update, (|mut switch: ResMut<Switch<LoadingTask>>| {
            // Simulate loading...
            println!("Loading assets...");
            
            // Turn off the switch when done
            switch.off();
        }).run_if(switch_is_on::<LoadingTask>))
        
        // Spawn a reactor that coordinates the loading sequence
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                println!("Starting loading sequence...");
                
                // Turn on the loading switch to start the loading task
                task.will(Update, once::switch::on::<LoadingTask>()).await;
                
                // Wait until the loading task is complete (switch is turned off)
                task.will(Update, wait::switch::off::<LoadingTask>()).await;
                
                println!("Loading sequence complete!");
            }));
        });
}
```

### Creating a State Machine

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Define switch types for different states
struct Idle;
struct Walking;
struct Running;

fn main() {
    App::new()
        // Idle animation system
        .add_systems(Update, (|mut idle_anim: ResMut<IdleAnimation>| {
            idle_anim.play();
        }).run_if(switch_is_on::<Idle>))
        
        // Walking animation system
        .add_systems(Update, (|mut walk_anim: ResMut<WalkAnimation>| {
            walk_anim.play();
        }).run_if(switch_is_on::<Walking>))
        
        // Running animation system
        .add_systems(Update, (|mut run_anim: ResMut<RunAnimation>| {
            run_anim.play();
        }).run_if(switch_is_on::<Running>))
        
        // State transition system
        .add_systems(Update, |
            keys: Res<Input<KeyCode>>,
            mut commands: Commands,
        | {
            if keys.just_pressed(KeyCode::W) {
                commands.spawn(Reactor::schedule(|task| async move {
                    // Turn off all states
                    task.will(Update, once::switch::off::<Idle>()).await;
                    task.will(Update, once::switch::off::<Running>()).await;
                    
                    // Turn on walking state
                    task.will(Update, once::switch::on::<Walking>()).await;
                }));
            }
            
            if keys.just_pressed(KeyCode::ShiftLeft) {
                commands.spawn(Reactor::schedule(|task| async move {
                    // Turn off all states
                    task.will(Update, once::switch::off::<Idle>()).await;
                    task.will(Update, once::switch::off::<Walking>()).await;
                    
                    // Turn on running state
                    task.will(Update, once::switch::on::<Running>()).await;
                }));
            }
            
            if keys.just_released(KeyCode::W) && keys.just_released(KeyCode::ShiftLeft) {
                commands.spawn(Reactor::schedule(|task| async move {
                    // Turn off all states
                    task.will(Update, once::switch::off::<Walking>()).await;
                    task.will(Update, once::switch::off::<Running>()).await;
                    
                    // Turn on idle state
                    task.will(Update, once::switch::on::<Idle>()).await;
                }));
            }
        });
}
```

## When to Use

Use the switch module when you need to:

- Coordinate between Reactors and regular Bevy systems
- Control when certain systems should run
- Signal the completion of asynchronous tasks
- Create state machines with clear on/off states
- Implement gameplay sequences that depend on switch states

Switches are particularly useful for tasks that need to be performed on the main thread but need to be coordinated with asynchronous Reactor tasks.