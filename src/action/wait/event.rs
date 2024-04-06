//! [`wait::event`] creates a task related to waiting to receive events.
//!
//! - [`wait::event::comes`]
//! - [`wait::event::read`]


use bevy::prelude::{Event, EventReader};

use crate::prelude::seed::{ActionSeed, Seed};
use crate::prelude::wait;

/// Waits until the specified event is sent
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, wait::event::comes::<AppExit>()).await;
/// });
/// ```
#[inline(always)]
pub fn comes<E>() -> impl ActionSeed + Seed
    where E: Event
{
    wait::until(|er: EventReader<E>| {
        !er.is_empty()
    })
}


/// Waits until the specified event is sent.
///
/// This is similar to [`wait::event::comes`], except that it returns the event itself.
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, wait::event::read::<AppExit>()).await;
/// });
/// ```
#[inline(always)]
pub fn read<E>() -> impl ActionSeed<(), E> + Seed
    where E: Event + Clone
{
    wait::output(|mut er: EventReader<E>| {
        er.read().next().cloned()
    })
}