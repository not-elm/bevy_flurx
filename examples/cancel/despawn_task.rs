use std::time::Duration;

use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::log::info;
use bevy::prelude::{Commands, Entity, KeyCode, Query, Res, With};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::async_schedules::TaskHandle;
use bevy_async_system::prelude::SpawnAsyncSystem;
use bevy_async_system::runner::delay;



/// One way to cancel a task is to delete the entity has [`TaskHandle`](TaskHandle).
///
/// In this example, a task is generated to run `println!` every second,
/// but it's canceled by pressing the `Enter Key`.
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, cancel)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        loop {
            schedules.add_system(Update, delay::timer(Duration::from_secs(1))).await;
            println!("******** tick **********");
        }
    });
}


fn cancel(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    task: Query<Entity, With<TaskHandle>>,
) {
    if input.just_pressed(KeyCode::Return) {
        for entity in task.iter() {
            info!("cancel");
            commands.entity(entity).despawn();
        }
    }
}