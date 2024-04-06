//! [`once::state`] creates a task that only once run system related to [`States`](bevy::prelude::States).
//!
//! - [`once::state::set`]


use bevy::prelude::{In, NextState, ResMut, States};

use crate::action::{once, Action};

/// Once set a next state.
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
/// Flurx::schedule(|task| async move{
///     task.will(Update, once::state::set(S::S2)).await;
/// });
/// ```
pub fn set<S>(state: S) -> impl Action< S, ()>
    where S: States + 'static
{
    once::run_with(state, |input: In<S>, mut state: ResMut<NextState<S>>| {
        state.set(input.0);
    })
}



