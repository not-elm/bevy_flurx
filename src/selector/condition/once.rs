use bevy::prelude::{In, IntoSystem, System};

pub mod res;
pub mod event;
pub mod state;
pub mod non_send;


#[inline]
pub fn run<Sys, Input, Out, Marker>(system: Sys) -> impl System<In=Input, Out=Option<Out>>
    where
        Sys: IntoSystem<Input, Out, Marker>,
        Input: 'static,
        Out: 'static
{
    IntoSystem::into_system(system.pipe(|input: In<Out>| {
        Some(input.0)
    }))
}

