use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::math::Vec2;
use bevy::prelude::{Camera2dBundle, Color, Commands, Component, Query, Sprite, Transform, With};
use bevy::sprite::SpriteBundle;
use bevy::utils::default;

use bevtask::BevTaskPlugin;
use bevtask::ext::AsyncPool;
use bevtask::runner::until::Until;

#[derive(Component)]
struct Movable;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            BevTaskPlugin
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
    commands.spawn_async(|task| async move {
        task.spawn(Update, Until::run(move_up)).await;
        task.spawn(Update, Until::run(move_right)).await;
    });
}


fn move_up(
    mut shape: Query<&mut Transform, With<Movable>>
) -> bool {
    let mut transform = shape.single_mut();
    transform.translation.y += 1.;
    50. <= transform.translation.y
}


fn move_right(
    mut shape: Query<&mut Transform, With<Movable>>
) -> bool {
    let mut transform = shape.single_mut();
    transform.translation.x += 1.;
    50. <= transform.translation.x
}