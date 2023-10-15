use std::time::Duration;

use bevy::app::{App, AppExit, PluginGroup, Startup, Update};
use bevy::DefaultPlugins;
use bevy::log::LogPlugin;
use bevy::prelude::{Commands, ResMut};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::ext::spawn_async_system::SpawnAsyncSystem;
use bevy_async_system::runner::{delay, once};


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<LogPlugin>(),
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
    commands.spawn_async(|schedules| async move {
        println!("Wait 3 seconds...");
        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;
        println!("3 seconds have passed.");

        println!("Wait 90 frames...");
        schedules.add_system(Update, delay::frames(90)).await;
        println!("End");
        schedules.add_system(Update, once::send(AppExit)).await;
    });
}

