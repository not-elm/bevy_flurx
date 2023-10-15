use bevy::app::{App, AppExit, Startup, Update};
use bevy::asset::AssetServer;
use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::DefaultPlugins;
use bevy::log::info;
use bevy::prelude::{AudioBundle, Commands, Entity, PlaybackSettings, Query, Res};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::ext::spawn_async_system::SpawnAsyncSystem;
use bevy_async_system::prelude::wait;
use bevy_async_system::runner::once;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup_async_systems)
        .run();
}


fn setup_async_systems(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        schedules.add_system(Update, once::run(play_audio)).await;
        schedules.add_system(Update, wait::until(finished_audio)).await;
        info!("***** Finished audio *****");
        schedules.add_system(Update, once::send(AppExit)).await;
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


fn finished_audio(
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

