//! [`once::audio`] creates a task that only once run system related to audio.
//!
//! - [`once::audio::play`]

use bevy::asset::{AssetPath, AssetServer};
use bevy::audio::{AudioPlayer, AudioSource};
use bevy::prelude::{Commands, Entity, In, Res};

use crate::action::once;
use crate::prelude::ActionSeed;

/// Spawns [`AudioBundle`].
///
/// The output value is [`Entity`] that [`AudioBundle`] is attached to.
///
/// [`AudioBundle`]: bevy::audio::AudioBundle
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::audio::play().with("<audio_path>")).await;
/// });
/// ```
pub fn play<Path>() -> ActionSeed<Path, Entity>
where
    Path: Into<AssetPath<'static>> + 'static,
{
    once::run(
        |In(path): In<Path>, mut commands: Commands, asset_server: Res<AssetServer>| {
            commands
                .spawn(AudioPlayer::<AudioSource>(asset_server.load(path.into())))
                .id()
        },
    )
}
