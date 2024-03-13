use bevy::prelude::{In, NextState, ResMut, States, };

use crate::selector::condition::{once, ReactorSystemConfigs, with, WithInput};

pub fn set<S>(state: S) -> impl ReactorSystemConfigs<WithInput, In=S>
    where S: States + 'static
{
    with(state, once::run(|input: In<S>, mut state: ResMut<NextState<S>>| {
        state.set(input.0);
    }))
}



