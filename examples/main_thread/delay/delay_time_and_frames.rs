use std::time::Duration;

use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::prelude::{Commands, ResMut};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::ext::spawn_async_system::SpawnAsyncSystem;
use bevy_async_system::runner::thread_pool::delay::Delay;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}


fn setup(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
) {
    settings.limiter = Limiter::from_framerate(30.);
    commands.spawn_async(|cmd| async move {
        println!("Wait 3 seconds...");
        cmd.spawn(Update, Delay::time(Duration::from_secs(3))).await;
        println!("3 seconds have passed.");

        println!("Wait 90 frames...");
        cmd.spawn(Update, Delay::frames(90)).await;
        println!("End");
    });
}

