use std::cell::Cell;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::{BoxedSystem, SystemId};
use bevy::prelude::World;
use flurx::selector::Selector;

use crate::selector::runner::{initialize_reactor_runner, ReactorSystemOutput};
use crate::selector::runner::standard::StandardReactorRunner;
use crate::world_ptr::WorldPtr;

mod runner;
pub mod once;
pub mod wait;

type MaybeSystemId<In, Out> = Cell<Option<SystemId<In, Out>>>;


pub(crate) struct WorldSelector<Label, In, Out> {
    system: Cell<Option<BoxedSystem<In, Out>>>,
    id: MaybeSystemId<In, Out>,
    input: In,
    label: Label,
}


impl<Label, In, Out> WorldSelector<Label, In, Out>
    where
        In: 'static,
        Out: 'static,
        Label: ScheduleLabel + Clone
{
    pub(crate) fn new(label: Label, input: In, system: BoxedSystem<In, Out>) -> WorldSelector<Label, In, Out> {
        Self {
            system: Cell::new(Some(system)),
            id: Cell::new(None),
            input,
            label,
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


impl<Label, In, Out> Selector<WorldPtr> for WorldSelector<Label, In, Option<Out>>
    where
        In: Clone + 'static,
        Out: 'static,
        Label: ScheduleLabel + Clone
{
    type Output = Out;

    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        self.output(&world, run_system(self, &world))
    }
}


pub(crate) fn run_system<Label, In, Out>(
    selector: &WorldSelector<Label, In, Option<Out>>,
    world: &WorldPtr,
) -> Option<Out>
    where
        Label: ScheduleLabel + Clone,
        In: Clone + 'static,
        Out: 'static
{
    let world: &mut World = world.as_mut();

    if let Some(id) = selector.id.get() {
        world.get_non_send_resource_mut::<ReactorSystemOutput<In, Out>>()?.extract_output(&id)
    } else {
        let system = selector.system.take().take().unwrap();
        let system_id = world.register_boxed_system(system);
        selector.id.set(Some(system_id));
        initialize_reactor_runner(world, selector.label.clone(), StandardReactorRunner::new(system_id, selector.input.clone()));
        None
    }
}



