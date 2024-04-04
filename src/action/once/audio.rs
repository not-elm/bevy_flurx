//! [`once::audio`] creates a task that only once run system related to audio.
//!
//! - [`once::audio::play`]

use bevy::asset::{AssetPath, AssetServer};
use bevy::audio::{AudioBundle, PlaybackSettings};
use bevy::prelude::{Commands, Entity, In, Res};

use crate::action::{once, TaskAction};

/// Spawns [`AudioBundle`].
/// 
/// The output value is [`Entity`](bevy::prelude::Entity) associated with the [`AudioBundle`].
///
/// [`AudioBundle`]: bevy::audio::AudioBundle
pub fn play(
    audio_path: impl Into<AssetPath<'static>>,
    settings: PlaybackSettings,
) -> impl TaskAction<In=(AssetPath<'static>, PlaybackSettings), Out=Entity> {
    let audio_path = audio_path.into();
    once::run_with((audio_path, settings), |In((path, settings)): In<(AssetPath, PlaybackSettings)>, mut commands: Commands, asset_server: Res<AssetServer>| {
        commands
            .spawn(AudioBundle {
                source: asset_server.load(path),
                settings,
            })
            .id()
    })
}