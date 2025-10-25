//! [`wait::event`] creates a task related to waiting to receive events.

use crate::prelude::seed::ActionSeed;
use crate::prelude::wait;
use bevy::prelude::*;

/// Waits until the event is received.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::event::comes::<AppExit>()).await;
/// });
/// ```
#[deprecated(
    since = "0.13.0",
    note = "Use `wait::message::comes()` instead. The `Event` trait has been replaced with `Message` in Bevy 0.17."
)]
pub fn comes<E>() -> ActionSeed
where
    E: Message,
{
    wait::message::comes::<E>()
}

/// Waits until the event is received and the event matches the predicate.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::event::comes_and::<AppExit>(|e: &AppExit|{
///         e.is_success()
///     })).await;
/// });
/// ```
#[deprecated(
    since = "0.13.0",
    note = "Use `wait::message::comes_and()` instead. The `Event` trait has been replaced with `Message` in Bevy 0.17."
)]
pub fn comes_and<E>(predicate: impl Fn(&E) -> bool + Send + Sync + 'static) -> ActionSeed
where
    E: Message,
{
    wait::message::comes_and::<E>(predicate)
}

/// Waits until the event is received.
///
/// This is similar to [`wait::event::comes`], but it returns a cloned event.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::event::read::<AppExit>()).await;
/// });
/// ```
#[deprecated(
    since = "0.13.0",
    note = "Use `wait::message::read()` instead. The `Event` trait has been replaced with `Message` in Bevy 0.17."
)]
#[inline(always)]
pub fn read<E>() -> ActionSeed<(), E>
where
    E: Message + Clone,
{
    wait::message::read::<E>()
}

/// Waits until the event is received and the event matches the predicate.
///
/// This is similar to [`wait::event::comes`], but it returns a cloned event.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::event::read_and::<AppExit>(|e: &AppExit|{
///         e.is_success()
///     })).await;
/// });
/// ```
#[deprecated(
    since = "0.13.0",
    note = "Use `wait::message::read_and()` instead. The `Event` trait has been replaced with `Message` in Bevy 0.17."
)]
#[inline(always)]
pub fn read_and<E>(predicate: impl Fn(&E) -> bool + Send + Sync + 'static) -> ActionSeed<(), E>
where
    E: Message + Clone,
{
    wait::message::read_and::<E>(predicate)
}