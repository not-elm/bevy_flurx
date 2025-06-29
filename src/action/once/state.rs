//! [`once::state`] creates a task that only once run system related to [States](bevy::prelude::States).

use crate::action::once;
use crate::prelude::ActionSeed;
use bevy::prelude::{In, NextState, ResMut};
use bevy::state::state::FreelyMutableState;

/// Once set a next state.
///
/// ## Examples
///
/// ```
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(States, Copy, Clone, Hash, Eq, PartialEq, Default, Debug)]
/// enum S{
///     #[default]
///     S1,
///     S2
/// };
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::state::set().with(S::S2)).await;
/// });
/// ```
pub fn set<S>() -> ActionSeed<S>
where
    S: FreelyMutableState + 'static,
{
    once::run(|input: In<S>, mut state: ResMut<NextState<S>>| {
        state.set(input.0);
    })
}
