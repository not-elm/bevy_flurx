//! This example shows how to convert an asynchronous process such as HTTP communication into an action.
//!
//! ## Notes
//!
//! You need to enable the `side-effect` and `tokio` feature flags to use this feature.

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default(), FlurxPlugin))
        .add_systems(Startup, spawn_reactor)
        .add_systems(
            Update,
            step_reactors.run_if(switch_is_on::<TokioTaskRunning>),
        )
        .run();
}

struct TokioTaskRunning;

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
        })
        .await;

        task.will(Update, once::switch::on::<TokioTaskRunning>())
            .await;

        // By turning on feature flag `tokio`,
        // you can also directly write asynchronous functions depending on tokio's runtime in the reactor.
        // However, you need to manually advance the reactor.
        tokio::time::sleep(Duration::from_secs(1)).await;
        task.will(Update, once::switch::off::<TokioTaskRunning>())
            .await;

        info!("Done!");
        task.will(Update, once::event::app_exit_success()).await;
    }));
}

fn step_reactors(mut commands: Commands) {
    commands.trigger(StepAllReactors);
}
