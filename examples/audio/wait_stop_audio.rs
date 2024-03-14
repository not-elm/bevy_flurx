use bevy::app::{App, Startup, Update};
use bevy::asset::AssetServer;
use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::DefaultPlugins;
use bevy::log::info;
use bevy::prelude::{AudioBundle, Commands, Entity, PlaybackSettings, Query, Res, World};

use bevy_async_system::extension::ScheduleReactor;
use bevy_async_system::prelude::*;

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
        task.will(Update, once::run(play_audio)).await;
        task.will(Update, wait::until(stop_audio)).await;
        info!("***** Finished audio *****");
        task.will(Update, once::event::app_exit()).await;
    });
}


fn play_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/higurashi.ogg"),
        settings: PlaybackSettings::ONCE,
    });
}


fn stop_audio(
    mut commands: Commands,
    audio: Query<(Entity, &AudioSink)>,
) -> bool {
    let Ok((entity, audio)) = audio.get_single() else { return false; };
    if audio.empty() {
        commands.entity(entity).despawn();
        true
    } else {
        false
    }
}

