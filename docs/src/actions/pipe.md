# pipe

The `pipe` module provides a mechanism to pipe actions together, where the output of one action is used as the input for another action. This is particularly useful for creating data processing pipelines.

## Basic Usage

The `pipe` module provides the `Pipe` trait, which adds the `pipe` method to all actions. This method allows you to connect the output of one action to the input of another.

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Pipe the output of one action as the input to another
        let result = task.will(Update, 
            once::run(|| "Hello")
                .pipe(once::run(|In(text): In<&str>| format!("{}, World!", text)))
        ).await;

        println!("{}", result); // Prints "Hello, World!"
    }));
}
```

## How It Works

When actions are combined using the `pipe` method:

1. The first action is executed until completion
2. The output of the first action is passed as input to the second action
3. The second action is then executed with this input
4. The output of the combined action will be that of the second action

This creates a data flow where information is processed in stages, with each stage building on the results of the previous one.

## Practical Examples

### Data Processing Pipeline

The `pipe` module is particularly useful for creating data processing pipelines:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Create a data processing pipeline
        let result = task.will(Update, 
            once::run(|| 5) // Generate a number
                .pipe(once::run(|In(num): In<i32>| num * 2)) // Double it
                .pipe(once::run(|In(num): In<i32>| format!("The result is: {}", num))) // Format it
        ).await;

        println!("{}", result); // Prints "The result is: 10"
    }));
}
```

### Event Handling

The `pipe` module can be used to process events:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Component)]
struct Hp(u8);

#[derive(Event, Clone)]
struct PlayerHit(Entity);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Wait for a PlayerHit event and then process it
        task.will(Update, 
            wait::message::read::<PlayerHit>()
                .pipe(once::run(|In(PlayerHit(entity)): In<PlayerHit>, mut players: Query<&mut Hp>| {
                    players.get_mut(entity).unwrap().0 -= 10;
                    println!("Player hit! HP reduced to {}", players.get(entity).unwrap().0);
                }))
        ).await;
    }));
}
```

### Combining with Other Action Types

The `pipe` method can be combined with other action types for more complex behaviors:

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Combine pipe with other action types
        task.will(Update, 
            wait::input::key_pressed(KeyCode::Space) // Wait for space key
                .pipe(once::run(|_| "Space pressed!")) // Process the event
                .then(delay::seconds(1.0)) // Wait for 1 second
                .then(once::run(|| println!("Ready for next input"))) // Print a message
        ).await;
    }));
}
```