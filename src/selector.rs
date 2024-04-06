use std::cell::Cell;
use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::World;
use flurx::selector::Selector;

use crate::action::TaskAction;
use crate::runner::{CancellationToken, initialize_task_runner, TaskOutput};
use crate::world_ptr::WorldPtr;

pub(crate) struct WorldSelector<Label, Action, In, Out> {
    action: Cell<Option<Action>>,
    output: TaskOutput<Out>,
    label: Label,
    token: CancellationToken,
    _m: PhantomData<In>,
}

impl<Label, Action, In, Out> WorldSelector<Label, Action, In, Out>
    where
        Label: ScheduleLabel + Clone,
        Action: TaskAction<In, Out>,
        In: 'static,
        Out: 'static,
{
    #[inline]
    pub(crate) fn new(label: Label, action: Action, token: CancellationToken) -> WorldSelector<Label, Action, In, Out> {
        Self {
            action: Cell::new(Option::Some(action)),
            output: TaskOutput::default(),
            label,
            token,
            _m: PhantomData,
        }
    }
}

impl<Label, Action, In, Out> Selector<WorldPtr> for WorldSelector<Label, Action, In, Out>
    where
        Label: ScheduleLabel + Clone,
        Action: TaskAction<In, Out> + 'static,
        In: 'static,
        Out: 'static
{
    type Output = Out;

    #[inline]
    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        let world: &mut World = world.as_mut();
        if let Some(action) = self.action.take() {
            let runner = action.to_runner(self.token.clone(), self.output.clone());
            initialize_task_runner(world, self.label.clone(), runner);
            None
        } else {
            self.output.take()
        }
    }
}





