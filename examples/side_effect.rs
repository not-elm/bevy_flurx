//! This example shows how to convert an asynchronous process such as HTTP communication into an action.
//!
//! ## Notes
//! 
//! You need to enable the `side_effect` and `tokio` feature flag to use this feature.

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
            // The spawned process is executed in a tokio's green thread.
            side_effect::tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3)).await;
                "3 seconds elapsed"
            })
                // You can also pass the argument.
                .pipe(side_effect::tokio::spawn(|message| async move {
                    info!("{message}");
                }))
        }).await;

        // By turning on the `side_effect` and `tokio` feature flags, 
        // you can also write asynchronous functions depend on tokio directly in the reactor.
        tokio::time::sleep(Duration::from_secs(1)).await;

        info!("Done!");
        task.will(Update, once::event::app_exit_success()).await;
    }));
}

