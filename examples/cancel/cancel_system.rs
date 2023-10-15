use std::time::Duration;

use bevy::app::{App, AppExit, Startup, Update};
use bevy::core::FrameCount;
use bevy::MinimalPlugins;
use bevy::prelude::{Commands, Res};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::prelude::SpawnAsyncSystem;
use bevy_async_system::runner::{delay, once, repeat};


fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}



fn setup(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        let handle = schedules.add_system(Update, repeat::forever(|frame_count: Res<FrameCount>| {
            println!("frame count = {}", frame_count.0);
        }));

        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;
        println!("Cancel");
        // Dropping the handle also stops the system.
        drop(handle);
        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;
        println!("Task End!");
        schedules.add_system(Update, once::send(AppExit)).await;
    });
}


