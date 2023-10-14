use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::math::Vec2;
use bevy::prelude::{Camera2dBundle, Color, Commands, Component, Query, Sprite, Transform, With};
use bevy::sprite::SpriteBundle;
use bevy::utils::default;

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::prelude::Wait;

#[derive(Component)]
struct Movable;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, (
            setup_entities,
            setup_async_systems
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


fn setup_async_systems(
    mut commands: Commands
) {
    commands.spawn_async(|cmd| async move {
        cmd.spawn_on_main(Update, Wait::until(move_up)).await;
        cmd.spawn_on_main(Update, Wait::until(move_right)).await;
    });
}


fn move_up(
    mut shape: Query<&mut Transform, With<Movable>>
) -> bool {
    let mut transform = shape.single_mut();
    transform.translation.y += 1.;
    300. <= transform.translation.y
}


fn move_right(
    mut shape: Query<&mut Transform, With<Movable>>
) -> bool {
    let mut transform = shape.single_mut();
    transform.translation.x += 1.;
    500. <= transform.translation.x
}