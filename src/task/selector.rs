use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use bevy::ecs::system::SystemId;
use bevy::prelude::{IntoSystem, World};
use store::selector::StateSelector;

use crate::store::WorldPointer;

type MaybeSystemId<In, Out> = ManuallyDrop<Cell<Option<SystemId<In, Out>>>>;


pub(crate) struct WorldSelector<System, In, Out, Marker> {
    system: System,
    id: MaybeSystemId<In, Out>,
    input: In,
    _m1: PhantomData<Marker>,
    _m2: PhantomData<Out>,
}


impl<System, In, Out, Marker> WorldSelector<System, In, Out, Marker>
{
    pub(crate) fn new(input: In, system: System) -> WorldSelector<System, In, Out, Marker> {
        Self {
            system,
            id: ManuallyDrop::new(Cell::new(None)),
            input,
            _m1: PhantomData,
            _m2: PhantomData,
        }
    }
}


impl<System, In, Out, Marker> StateSelector<WorldPointer> for WorldSelector<System, In, Option<Out>, Marker>
    where
        System: IntoSystem<In, Option<Out>, Marker> + Clone + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    type Output = Out;

    fn select(&self, world: &WorldPointer) -> Option<Self::Output> {
        run_system(self, world)
    }
}


pub(crate) fn run_system<System, In, Out, Marker>(
    select: &WorldSelector<System, In, Out, Marker>,
    world: &WorldPointer,
) -> Out
    where
        System: IntoSystem<In, Out, Marker> + Clone + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    let world: &mut World = world.as_mut();

    if select.id.get().is_none() {
        select.id.set(Some(world.register_system(select.system.clone())));
    }

    world.run_system_with_input(select.id.get().unwrap(), select.input.clone()).unwrap()
}

