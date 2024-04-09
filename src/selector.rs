use std::cell::Cell;
use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::World;
use flurx::selector::Selector;

use crate::action::Action;
use crate::runner::{CancellationToken, initialize_runner, Output};
use crate::world_ptr::WorldPtr;

pub(crate) struct WorldSelector<Label, In, Out> {
    action: Cell<Option<Action<In, Out>>>,
    output: Output<Out>,
    label: Label,
    token: CancellationToken,
    _m: PhantomData<In>,
}

impl<Label, In, Out> WorldSelector<Label, In, Out>
    where
        Label: ScheduleLabel,
        In: 'static,
        Out: 'static,
{
    #[inline]
    pub(crate) fn new(label: Label, action: Action<In, Out>, token: CancellationToken) -> WorldSelector<Label, In, Out> {
        Self {
            action: Cell::new(Some(action)),
            output: Output::default(),
            label,
            token,
            _m: PhantomData,
        }
    }
}

impl<Label, In, Out> Selector<WorldPtr> for WorldSelector<Label, In, Out>
    where
        Label: ScheduleLabel,
        In: 'static,
        Out: 'static
{
    type Output = Out;

    #[inline]
    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        let world: &mut World = world.as_mut();
        if let Some(action) = self.action.take() {
            let runner = action.into_runner(self.token.clone(), self.output.clone());
            initialize_runner(world, &self.label, runner);
            None
        } else {
            self.output.take()
        }
    }
}





