# inspect

The `inspect` module provides mechanisms to clone and inspect input values via auxiliary actions without disrupting their primary flow. This is particularly useful for debugging, logging, or performing side-effects on input values.

## Basic Usage

The `inspect` module provides two main ways to inspect input values:

1. The `inspect` function: Creates an action that clones its input, passing one clone to a provided action for processing while forwarding the original input as output
2. The `Inspect` trait: Adds a convenient `.inspect()` method to actions, simplifying the chaining of actions with auxiliary side-effects

### Using the `inspect` Function

Use the `inspect()` function to create an action that processes a clone of the input:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Event, Clone)]
struct Damage(u8);

#[derive(Component)]
struct Hp(u8);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Use inspect to log damage without affecting the main flow
        task.will(Update, wait::event::read::<Damage>()
            .pipe(inspect(once::run(|In(Damage(damage)): In<Damage>| {
                println!("Players take {damage} points of damage.");
            })))
            .pipe(once::run(|In(Damage(damage)): In<Damage>, mut players: Query<&mut Hp>| {
                for mut player in &mut players {            
                    player.0 -= damage;
                }
            }))
        ).await;
    }));
}
```

### Using the `Inspect` Trait

Use the `.inspect()` method to achieve the same result with a more concise syntax:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Event, Clone)]
struct Damage(u8);

#[derive(Component)]
struct Hp(u8);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Use the inspect method for a more concise syntax
        task.will(Update, wait::event::read::<Damage>()
            .inspect(once::run(|In(Damage(damage)): In<Damage>| {
                println!("Players take {damage} points of damage.");
            }))
            .pipe(once::run(|In(Damage(damage)): In<Damage>, mut players: Query<&mut Hp>| {
                for mut player in &mut players {            
                    player.0 -= damage;
                }
            }))
        ).await;
    }));
}
```

## How It Works

When using the `inspect` function or the `Inspect` trait:

1. The input value is cloned
2. One clone is passed to the auxiliary action for processing
3. The original input is forwarded as the output without modification
4. Any side-effects in the auxiliary action (e.g., logging, external calls) are executed

This ensures that you can perform auxiliary operations (e.g., logging, metrics) while preserving the original input for further use.

## Practical Examples

### Debugging

The `inspect` module is particularly useful for debugging:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Debug the value at each step of a pipeline
        let result = task.will(Update, 
            once::run(|| 5)
                .inspect(once::run(|In(num): In<i32>| {
                    println!("Initial value: {}", num);
                }))
                .map(|num| num * 2)
                .inspect(once::run(|In(num): In<i32>| {
                    println!("After multiplication: {}", num);
                }))
                .pipe(once::run(|In(num): In<i32>| num + 3))
                .inspect(once::run(|In(num): In<i32>| {
                    println!("Final value: {}", num);
                }))
        ).await;

        println!("Result: {}", result); // Prints "Result: 13"
    }));
}
```

### Metrics Collection

The `inspect` module can be used for collecting metrics:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Resource, Default)]
struct Metrics {
    damage_dealt: u32,
}

#[derive(Event, Clone)]
struct Damage(u8);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Collect metrics while processing events
        task.will(Update, wait::event::read::<Damage>()
            .inspect(once::run(|In(Damage(damage)): In<Damage>, mut metrics: ResMut<Metrics>| {
                metrics.damage_dealt += damage as u32;
                println!("Total damage dealt: {}", metrics.damage_dealt);
            }))
            .pipe(once::run(|In(Damage(damage)): In<Damage>, mut players: Query<&mut Hp>| {
                for mut player in &mut players {            
                    player.0 -= damage;
                }
            }))
        ).await;
    }));
}
```

### Conditional Side Effects

The `inspect` module can be used for conditional side effects:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Event, Clone)]
struct PlayerAction {
    action_type: ActionType,
    value: i32,
}

#[derive(Clone, PartialEq)]
enum ActionType {
    Attack,
    Heal,
    Move,
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Perform conditional side effects based on input values
        task.will(Update, wait::event::read::<PlayerAction>()
            .inspect(once::run(|In(action): In<PlayerAction>| {
                match action.action_type {
                    ActionType::Attack => println!("Player attacks for {} damage!", action.value),
                    ActionType::Heal => println!("Player heals for {} health!", action.value),
                    ActionType::Move => println!("Player moves {} units!", action.value),
                }
            }))
            .pipe(once::run(|In(action): In<PlayerAction>| {
                // Process the action...
                action.value
            }))
        ).await;
    }));
}
```