# delay

The `delay` module defines actions that perform delay processing. These are useful for waiting for a specific amount of time or a specific number of frames.

## Basic Usage

The `delay` module provides two main functions:

### time

The `time` function delays the execution for a specified amount of time.

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Delay for 1 second
        task.will(Update, delay::time().with(Duration::from_secs(1))).await;
        
        // Delay for 500 milliseconds
        task.will(Update, delay::time().with(Duration::from_millis(500))).await;
    }));
}
```

Under the hood, `delay::time()` uses a `Timer` with `TimerMode::Once` to track the elapsed time. The action completes when the timer finishes.

### frames

The `frames` function delays the execution for a specified number of frames.

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Delay for 30 frames
        task.will(Update, delay::frames().with(30)).await;
        
        // Delay for 5 frames
        task.will(Update, delay::frames().with(5)).await;
    }));
}
```

Under the hood, `delay::frames()` uses a `Local<usize>` to track the number of frames that have passed. The action completes when the specified number of frames have been processed.

## Combining with Other Actions

The `delay` actions can be combined with other actions using the `then`, `pipe`, and `through` methods.

### then

The `then` method allows you to execute an action after a delay.

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Delay for 1 second, then send an event
        task.will(Update, delay::time().with(Duration::from_secs(1))
            .then(once::message::app_exit_success())
        ).await;
    }));
}
```

### through

The `through` method allows you to delay the output of a previous action.

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Get a value, then delay for 1 second before returning it
        let value = task.will(Update, once::run(|count: Res<Count>| {
            count.0
        }).through(delay::time().with(Duration::from_secs(1)))).await;
    }));
}
```

## Examples

### Delayed Animation

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Spawn an entity
        let entity = task.will(Update, once::run(|mut commands: Commands| {
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(-100.0, 0.0, 0.0)),
                ..default()
            }).id()
        })).await;
        
        // Move the entity to the right over time
        for i in 0..10 {
            task.will(Update, once::run(move |mut transforms: Query<&mut Transform>| {
                let mut transform = transforms.get_mut(entity).unwrap();
                transform.translation.x += 20.0;
            })).await;
            
            // Delay between movements
            task.will(Update, delay::time().with(Duration::from_millis(100))).await;
        }
    }));
}
```

### Frame-Based Animation

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Spawn an entity
        let entity = task.will(Update, once::run(|mut commands: Commands| {
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
                ..default()
            }).id()
        })).await;
        
        // Move the entity upward over frames
        for i in 0..10 {
            task.will(Update, once::run(move |mut transforms: Query<&mut Transform>| {
                let mut transform = transforms.get_mut(entity).unwrap();
                transform.translation.y += 20.0;
            })).await;
            
            // Delay between movements
            task.will(Update, delay::frames().with(5)).await;
        }
    }));
}
```
