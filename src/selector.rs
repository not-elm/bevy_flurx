use std::any::TypeId;
use std::cell::Cell;
use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, System, World};
use flurx::selector::Selector;

use crate::selector::runner::{initialize_reactor_runner, ReactorSystemOutput};
use crate::selector::runner::standard::StandardReactorRunner;
use crate::world_ptr::WorldPtr;

mod runner;

pub mod condition;


pub(crate) struct WorldSelector<Label, Sys, In, Out> {
    system: Cell<Option<Sys>>,
    system_type: Cell<Option<TypeId>>,
    input: In,
    label: Label,
    _m: PhantomData<Out>,
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
            system_type: Cell::new(None),
            input,
            label,
            _m: PhantomData,
        }
    }

    pub(crate) fn output<O>(&self, world: &WorldPtr, output: Option<O>) -> Option<O> {
        output.map(|output| output)
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

    if let Some(mut system) = selector.system.take().take() {
        system.initialize(world);
        system.apply_deferred(world);
        selector.system_type.set(Some(system.system_type_id()));
        initialize_reactor_runner(world, selector.label.clone(), StandardReactorRunner::new(system, selector.input.clone()));
        None
    } else {
        let id = selector.system_type.get().unwrap();
        let output = world.get_non_send_resource_mut::<ReactorSystemOutput<Out>>()?.extract_output(&id)?;
        Some(output)
    }
}



