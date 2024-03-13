use std::cell::Cell;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::SystemId;
use bevy::prelude::{System, World};
use flurx::selector::Selector;

use crate::selector::runner::{initialize_reactor_runner, ReactorSystemOutput};
use crate::selector::runner::standard::StandardReactorRunner;
use crate::world_ptr::WorldPtr;

mod runner;

pub mod condition;


type MaybeSystemId<In, Out> = Cell<Option<SystemId<In, Out>>>;


pub(crate) struct WorldSelector<Label, Sys, In, Out> {
    system: Cell<Option<Sys>>,
    id: MaybeSystemId<In, Option<Out>>,
    input: In,
    label: Label,
}


impl<Label, Sys, In, Out> WorldSelector<Label, Sys, In, Out>
    where
        Label: ScheduleLabel + Clone,
        Sys: System<In=In, Out=Option<Out>>,
        In: 'static,
        Out: 'static
{
    pub(crate) fn new(label: Label, input: In, system: Sys) -> WorldSelector<Label, Sys, In, Out> {
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


impl<Label, Sys, In, Out> Selector<WorldPtr> for WorldSelector<Label, Sys, In, Out>
    where
        Label: ScheduleLabel + Clone,
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    type Output = Out;

    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        self.output(&world, run_system(self, &world))
    }
}


pub(crate) fn run_system<Label, Sys, In, Out>(
    selector: &WorldSelector<Label, Sys, In, Out>,
    world: &WorldPtr,
) -> Option<Out>
    where
        Label: ScheduleLabel + Clone,
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    let world: &mut World = world.as_mut();

    if let Some(id) = selector.id.get() {
        let output = world.get_non_send_resource_mut::<ReactorSystemOutput<In, Out>>()?.extract_output(&id)?;
        world.remove_system(id).expect("failed remove system");
        selector.id.take();
        Some(output)
    } else {
        let system = selector.system.take().take().unwrap();
        let system_id = world.register_boxed_system(Box::new(system));
        selector.id.set(Some(system_id));
        initialize_reactor_runner(world, selector.label.clone(), StandardReactorRunner::new(system_id, selector.input.clone()));
        None
    }
}



