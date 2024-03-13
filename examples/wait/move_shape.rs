use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::math::Vec2;
use bevy::prelude::{Camera2dBundle, Color, Commands, Component, NonSendMut, Query, Sprite, Transform, Update, With};
use bevy::sprite::SpriteBundle;
use bevy::utils::default;

use bevy_async_system::FlurxPlugin;
use bevy_async_system::scheduler::TaskScheduler;
use bevy_async_system::selector::condition::{once, wait};


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
    mut scheduler: NonSendMut<TaskScheduler>
) {
    scheduler.schedule(|task| async move {
        task.will(Update, wait::until(move_up)).await;
        task.will(Update, wait::until(move_right)).await;
        print!("*** Finish ***");
        task.will(Update, once::event::app_exit()).await;
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