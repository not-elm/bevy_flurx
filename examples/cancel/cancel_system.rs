use std::time::Duration;

use bevy::app::{App, Startup, Update};
use bevy::core::FrameCount;
use bevy::MinimalPlugins;
use bevy::prelude::{Commands, Res};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::prelude::Repeat;
use bevy_async_system::runner::non_send::delay::Delay;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            BevTaskPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn_async(|task| async move {
        let handle = task.spawn(Update, Repeat::forever(|frame_count: Res<FrameCount>| {
            println!("frame count = {}", frame_count.0);
        }));

        task.spawn(Update, Delay::Time(Duration::from_secs(3))).await;
        println!("Cancel");
        // Dropping the handle also stops the system.
        drop(handle);
        task.spawn(Update, Delay::Time(Duration::from_secs(3))).await;
        println!("Task End!");
    });
}


