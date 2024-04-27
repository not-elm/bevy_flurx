//! [`wait::switch`] creates a task related to waiting [`Switch`]
//!
//! - [`wait::switch::on`]
//! - [`wait::switch::off`]


use bevy::prelude::Res;

use crate::action::switch::Switch;
use crate::action::wait;
use crate::prelude::ActionSeed;

/// Waits until the switch turned on.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct Animation;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::switch::on::<Animation>()).await;
/// });
/// ```
#[inline]
pub fn on<M>() -> ActionSeed
    where M: Send + Sync + 'static
{
    wait::until(|switch: Option<Res<Switch<M>>>| {
        switch.is_some_and(|s| s.turned_on())
    })
}

/// Waits until the switch turned off.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct Animation;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, {
///         once::switch::on::<Animation>()
///             .then(wait::switch::off::<Animation>())
///     }).await;
/// });
/// ```
#[inline]
pub fn off<M>() -> ActionSeed
    where M: Send + Sync + 'static
{
    wait::until(|switch: Option<Res<Switch<M>>>| {
        switch.is_some_and(|s| s.turned_off())
    })
}