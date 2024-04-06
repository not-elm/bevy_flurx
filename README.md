# bevy_flurx

[![Crates.io](https://img.shields.io/crates/v/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/not-elm/bevy_flurx#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)

> [!CAUTION]
> `bevy_flurx` is currently in the early stages of development and is subject to disruptive changes.

Provides [`UniTask`](https://github.com/Cysharp/UniTask)-like functionality to asynchronously await game
state.

ECS consists of a group of very small independent systems.
Although we find this architecture very nice, it takes a lot of effort to implement system interaction,
especially state monitoring, and this problem becomes more pronounced as the application gets larger.
For example, how can I play a sound effect just before the character starts moving, and then not accept any input until the character stop movement 
and the sound effect finished playing?
If only one process is waiting, simply use an event reader,
but if more than one, you will probably need to create a structure to manage the status of multiple processes.

This problem is common in event-driven applications, and is often solved with async-await;
This library also resolve this using it.
Specifically, I use an asynchronous processing flow that I call `Reactor`.

 `Reactor` can be used partially.
This means there is no need to rewrite existing applications to use this library.
And I recommend using it partially. This is because the system that runs `Reactor` and the systems that are run by `Reactor` run on the main thread.(Multithreading support is under consideration.)

## Example

All examples are [`here`](./examples).

Here is the code that waits for a simple rectangle to move.

![move_shape](examples/gui/move_shape.gif)
<details>

<summary>code</summary>

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
    commands.spawn(Flurx::schedule(|task| async move {
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
</details>

## ChangeLog

Please see [here](https://github.com/not-elm/bevy_flurx/blob/main/CHANGELOG.md).

## Compatible Bevy versions

| bevy_flurx | bevy   |
|------------|--------|
| 0.3.0      | 0.13.0 |
| 0.3.1      | 0.13.1 |

## License

This crate is licensed under the MIT License or the Apache License 2.0.
