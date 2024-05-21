use std::cell::Cell;
use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use flurx::selector::Selector;

use crate::action::Action;
use crate::runner::{CancellationToken, initialize_runner, Output};
use crate::world_ptr::WorldPtr;

pub(crate) struct WorldSelector<Label, In, Out> {
    action: Cell<Option<(Action<In, Out>, CancellationToken)>>,
    output: Output<Out>,
    label: Label,
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
            action: Cell::new(Some((action, token))),
            output: Output::default(),
            label,
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

    #[inline(always)]
    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        if let Some((action, token)) = self.action.take() {
            let runner = action.into_runner(self.output.clone());
            initialize_runner(world.as_mut(), &self.label, token, runner);
            None
        } else {
            self.output.take()
        }
    }
}





