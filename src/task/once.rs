use bevy::prelude::{Commands, Event, EventWriter, In, IntoSystem, Resource, World};
use store::selector::StateSelector;

use crate::store::WorldPointer;
use crate::task::selector::{run_system, WorldSelector};

struct Once<System, In, Out, Marker>(WorldSelector<System, In, Out, Marker>);


pub fn run<System, Out, Marker>(system: System) -> impl StateSelector<WorldPointer>
    where
        System: IntoSystem<(), Out, Marker> + Clone + Unpin + 'static,
        Out: 'static
{
    Once(WorldSelector::new((), system))
}

pub fn run_with<System, In, Out, Marker>(input: In, system: System) -> impl StateSelector<WorldPointer>
    where
        System: IntoSystem<In, Out, Marker> + Clone + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    Once(WorldSelector::new(input, system))
}

pub fn send<E>(input: E) -> impl StateSelector<WorldPointer>
    where E: Event + Clone
{
    run_with(input, |input: In<E>, mut ew: EventWriter<E>| {
        ew.send(input.0);
    })
}

pub fn insert_non_send_resource<E>(input: E) -> impl StateSelector<WorldPointer>
    where E: Clone + 'static
{
    run_with(input, |input: In<E>, world: &mut World| {
        world.insert_non_send_resource(input.0);
    })
}

pub fn insert_resource<E>(input: E) -> impl StateSelector<WorldPointer>
    where E: Resource + Clone + 'static
{
    run_with(input, |input: In<E>, mut commands: Commands| {
        commands.insert_resource(input.0);
    })
}


impl<System, In, Out, Marker> StateSelector<WorldPointer> for Once<System, In, Out, Marker>
    where
        System: IntoSystem<In, Out, Marker> + Clone + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    type Output = Out;

    fn select(&self, state: &WorldPointer) -> Option<Self::Output> {
        Some(run_system(&self.0, state))
    }
}