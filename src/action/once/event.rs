//! [`once::event`] creates a task that only once run system related to [`Event`].

use crate::action::seed::ActionSeed;
use crate::action::{once, Action};
use bevy::app::AppExit;
use bevy::prelude::Message;

/// Once send an event.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::message::write().with(AppExit::Success)).await;
/// });
/// ```
#[deprecated(
    since = "0.13.0",
    note = "Use `once::message::write()` instead. The `Event` trait has been replaced with `Message` in Bevy 0.17."
)]
#[inline(always)]
pub fn send<E>() -> ActionSeed<E, ()>
where
    E: Message,
{
    once::message::write::<E>()
}

/// Once send an event using [`Default`] trait.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::message::write_default::<AppExit>()).await;
/// });
/// ```
#[deprecated(
    since = "0.13.0",
    note = "Use `once::message::write_default()` instead. The `Event` trait has been replaced with `Message` in Bevy 0.17."
)]
#[inline(always)]
pub fn send_default<E>() -> ActionSeed
where
    E: Message + Default,
{
    once::message::write_default::<E>()
}

/// Once send [`AppExit::Success`].
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::message::app_exit_success()).await;
/// });
/// ```
#[deprecated(
    since = "0.13.0",
    note = "Use `once::message::app_exit_success()` instead. The `Event` trait has been replaced with `Message` in Bevy 0.17."
)]
#[inline(always)]
pub fn app_exit_success() -> Action<AppExit, ()> {
    once::message::app_exit_success()
}
