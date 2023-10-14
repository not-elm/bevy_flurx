use std::time::Duration;

use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::prelude::{Commands, ResMut};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::runner::non_send::delay::Delay;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            BevTaskPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}


fn setup(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
) {
    settings.limiter = Limiter::from_framerate(30.);
    commands.spawn_async(|task| async move {
        println!("Wait 3 seconds...");
        task.spawn(Update, Delay::Time(Duration::from_secs(3))).await;
        println!("3 seconds have passed.");

        println!("Wait 90 frames...");
        task.spawn(Update, Delay::Frames(90)).await;
        println!("End");
    });
}

