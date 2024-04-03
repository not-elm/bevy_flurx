use std::any::TypeId;
use std::cell::Cell;
use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, System, World};
use flurx::selector::Selector;

use crate::runner::{initialize_task_runner, TaskOutputMap};
use crate::runner::standard::MultiTimesRunner;
use crate::world_ptr::WorldPtr;


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
    #[inline]
    pub(crate) fn new(label: Label, input: In, system: Sys) -> WorldSelector<Label, Sys, In, Out> {
        Self {
            system: Cell::new(Some(system)),
            system_type: Cell::new(None),
            input,
            label,
            _m: PhantomData,
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

    #[inline]
    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        run_system(self, &world)
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
        initialize_task_runner(world, selector.label.clone(), MultiTimesRunner::new(system, selector.input.clone()));
        None
    } else {
        let id = selector.system_type.get().unwrap();
        let output = world.get_non_send_resource_mut::<TaskOutputMap<Out>>()?.extract_output(&id)?;
        Some(output)
    }
}



