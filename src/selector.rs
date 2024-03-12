use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use bevy::ecs::system::SystemId;
use bevy::prelude::{IntoSystem, World};
use flurx::selector::Selector;

use crate::world_ptr::WorldPtr;

pub mod once;
pub mod wait;

type MaybeSystemId<In, Out> = ManuallyDrop<Cell<Option<SystemId<In, Out>>>>;


pub(crate) struct WorldSelector<System, In, Out, Marker> {
    system: Cell<Option<System>>,
    id: MaybeSystemId<In, Out>,
    input: In,
    _m1: PhantomData<Marker>,
    _m2: PhantomData<Out>,
}


impl<System, In, Out, Marker> WorldSelector<System, In, Out, Marker>
    where
        System: IntoSystem<In, Out, Marker> + 'static,
        In: 'static,
        Out: 'static
{
    pub(crate) fn new(input: In, system: System) -> WorldSelector<System, In, Out, Marker> {
        Self {
            system: Cell::new(Some(system)),
            id: ManuallyDrop::new(Cell::new(None)),
            input,
            _m1: PhantomData,
            _m2: PhantomData,
        }
    }

    pub(crate) fn output<O>(&self, world: &WorldPtr, output: Option<O>) -> Option<O> {
        if let Some(output) = output {
            self.remove_system(world);
            Some(output)
        } else {
            None
        }
    }

    fn remove_system(&self, world: &WorldPtr) {
        if let Some(id) = self.id.get() {
            let _ = world.as_mut().remove_system(id);
        }
    }
}


impl<System, In, Out, Marker> Selector<WorldPtr> for WorldSelector<System, In, Option<Out>, Marker>
    where
        System: IntoSystem<In, Option<Out>, Marker> + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    type Output = Out;

    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        self.output(&world, run_system(self, &world))
    }
}


pub(crate) fn run_system<System, In, Out, Marker>(
    select: &WorldSelector<System, In, Out, Marker>,
    world: &WorldPtr,
) -> Out
    where
        System: IntoSystem<In, Out, Marker> + Unpin + 'static,
        In: Clone + 'static,
        Out: 'static
{
    let world: &mut World = world.as_mut();

    if select.id.get().is_none() {
        if let Some(system) = select.system.take().take() {
            select.id.set(Some(world.register_system(system)));
        };
    }

    world.run_system_with_input(select.id.get().unwrap(), select.input.clone()).unwrap()
}

