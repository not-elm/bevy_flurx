//! [`wait::audio`] creates a task related to waiting to audio.
//!
//! - [`wait::audio::finished`]

use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::prelude::{Commands, Entity, In, Query};

use crate::action::{TaskAction, wait, with};

/// Waits until the audio associated with the passed [`Entity`](bevy::prelude::Entity)
/// has finished playing.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// let mut world = World::new();
/// world.schedule_reactor(|task| async move{
///     let entity = task.will(Update, once::audio::play("<audio_path>", PlaybackSettings::ONCE)).await;
///     task.will(Update, wait::audio::finished(entity)).await;
/// });
/// ```
pub fn finished(entity: Entity) -> impl TaskAction<In=Entity, Out=()> {
    with(entity, wait::until(|In(entity): In<Entity>,
                              mut commands: Commands,
                              audio: Query<(Entity, &AudioSink)>| {
        let Ok((entity, audio)) = audio.get(entity) else { return false; };
        if audio.empty() {
            commands.entity(entity).despawn();
            true
        } else {
            false
        }
    }))
}