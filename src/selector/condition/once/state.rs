use bevy::prelude::{In, NextState, ResMut, States, System};
use crate::selector::condition::once;

pub fn set<S>() -> impl System<In=S, Out=Option<()>>
    where S: States + 'static
{
    once::run(|input: In<S>, mut state: ResMut<NextState<S>>| {
        state.set(input.0);
    })
}



