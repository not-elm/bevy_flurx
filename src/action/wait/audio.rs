//! [`wait::audio`] creates a task related to waiting to audio.
//!
//! - [`wait::audio::finished`]

use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::prelude::{Commands, Entity, In, Query};

use crate::action::wait;
use crate::prelude::seed::{ActionSeed, };

/// Waits until the audio associated with the passed [`Entity`](bevy::prelude::Entity)
/// has finished playing.
///
/// ## Examples
/// 
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, {
///         once::audio::play().with(("<audio_path>", PlaybackSettings::ONCE))
///             .pipe(wait::audio::finished())
///     }).await;
/// });
/// ```
pub fn finished() -> ActionSeed<Entity, ()> {
    wait::until(|In(entity): In<Entity>,
                 mut commands: Commands,
                 audio: Query<(Entity, &AudioSink)>| {
        let Ok((entity, audio)) = audio.get(entity) else { return false; };
        if audio.empty() {
            commands.entity(entity).despawn();
            true
        } else {
            false
        }
    })
}