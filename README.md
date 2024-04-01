# bevy_flurx

[![Crates.io](https://img.shields.io/crates/v/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/not-elm/bevy_flurx#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)

Provides [`UniTask`](https://github.com/Cysharp/UniTask)-like functionality to asynchronously await game
state.

This crate was named bevy_async_system, but I've pretty much rebuilt it using [`flurx`](https://github.com/not-elm/flurx).

## Example

All examples are [`here`](./examples).

Here is the code that waits for a simple rectangle to move.

![move_shape](examples/gui/move_shape.gif)

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Component)]
struct Movable;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, (
            setup_entities,
            setup_reactor
        ))
        .run();
}

fn setup_entities(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Movable,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 50.)),
                color: Color::BLUE,
                ..default()
            },
            ..default()
        }
    ));
}

fn setup_reactor(
    world: &mut World
) {
    world.schedule_reactor(|task| async move {
        loop {
            task.will(Update, once::run(reset_pos)).await;
            task.will(Update, wait::until(move_up)).await;
            task.will(Update, wait::until(move_right)).await;
            println!("To retry press the R key");
            task.will(Update, wait::until(input_r_key)).await;
        }
    });
}

fn reset_pos(
    mut shape: Query<&mut Transform, With<Movable>>
) {
    let mut transform = shape.single_mut();
    transform.translation = Vec3::default();
}

fn move_up(
    mut shape: Query<&mut Transform, With<Movable>>,
    time: Res<Time>,
) -> bool {
    let mut transform = shape.single_mut();
    transform.translation.y += time.delta_seconds() * 100.;
    150. <= transform.translation.y
}

fn move_right(
    mut shape: Query<&mut Transform, With<Movable>>,
    time: Res<Time>,
) -> bool {
    let mut transform = shape.single_mut();
    transform.translation.x += time.delta_seconds() * 100.;
    150. <= transform.translation.x
}

fn input_r_key(
    keyboard_input: Res<ButtonInput<KeyCode>>
) -> bool {
    keyboard_input.just_pressed(KeyCode::KeyR)
}
```

## ChangeLog

Please see [here](https://github.com/not-elm/bevy_flurx/blob/main/CHANGELOG.md).

## Compatible Bevy versions

| bevy_flurx | bevy   |
|------------|--------|
| 0.3.0      | 0.13.0 |
| 0.3.1      | 0.13.1 |

## License

This crate is licensed under the MIT License or the Apache License 2.0.
