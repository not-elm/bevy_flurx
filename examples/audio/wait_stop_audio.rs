//! This example shows how to wait for audio to stop.
//! 
//! Enjoy the sound of birds. 🐤


use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::log::info;
use bevy::prelude::{PlaybackSettings, World};

use bevy_flurx::extension::ScheduleReactor;
use bevy_flurx::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(world: &mut World) {
    world.schedule_reactor(|task| async move {
        let entity = task.will(Update, once::audio::play("audio/higurashi.ogg", PlaybackSettings::ONCE)).await;
        task.will(Update, wait::audio::finished(entity)).await;
        info!("***** Finished audio *****");
        task.will(Update, once::event::app_exit()).await;
    });
}

