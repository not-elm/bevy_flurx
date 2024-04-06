//! [`wait::switch`] creates a task related to waiting [`Switch`](crate::prelude::Switch).
//!
//! - [`wait::switch::on`]
//! - [`wait::switch::off`]


use bevy::prelude::Res;

use crate::action::switch::Switch;
use crate::action::wait;
use crate::prelude::{ActionSeed, SeedMark};

/// Waits until the switch turned on.
#[inline]
pub fn on<M>() -> impl ActionSeed + SeedMark
    where M: Send + Sync + 'static
{
    wait::until(|switch: Option<Res<Switch<M>>>| {
        switch.is_some_and(|s| s.turned_on())
    })
}

/// Waits until the switch turned off.
#[inline]
pub fn off<M>() -> impl ActionSeed + SeedMark
    where M: Send + Sync + 'static
{
    wait::until(|switch: Option<Res<Switch<M>>>| {
        switch.is_some_and(|s| s.turned_off())
    })
}