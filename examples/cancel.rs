use bevy::app::{App, Startup, Update};
use bevy::input::{Input, InputPlugin};
use bevy::MinimalPlugins;
use bevy::prelude::{Commands, Entity, KeyCode, Query, Res, Timer, TimerMode, With};

use bevtask::BevTaskPlugin;
use bevtask::ext::{AsyncPool, BevTask};
use bevtask::runner::delay::Delay;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            InputPlugin,
            BevTaskPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, cancel)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn_async(|task| async move {
        loop {
            task.spawn(Update, Delay::Timer(Timer::from_seconds(1., TimerMode::Once))).await;
            println!("******** tick **********");
        }
    });
}


fn cancel(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    task: Query<Entity, With<BevTask>>,
) {
    if input.just_pressed(KeyCode::Return) {
        for entity in task.iter() {
            commands.entity(entity).despawn();
        }
    }
}