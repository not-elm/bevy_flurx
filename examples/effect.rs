//! This example shows how to convert an asynchronous process such as HTTP communication into an action.

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            FlurxPlugin,
        ))
        .add_systems(Startup, spawn_reactor)
        .run();
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        task.will(Update, {
            effect::tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3)).await;
            })
                .then(once::run(|| {
                    info!("Done!");
                }))
                .then(once::event::app_exit_success())
        }).await;
    }));
}

