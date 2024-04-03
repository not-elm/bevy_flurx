use std::cell::Cell;
use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::World;
use flurx::selector::Selector;

use crate::action::TaskAction;
use crate::runner::{initialize_task_runner, TaskOutput};
use crate::world_ptr::WorldPtr;

pub(crate) struct WorldSelector<Label, Action, In, Out, M> {
    action: Cell<Option<Action>>,
    output: TaskOutput<Out>,
    label: Label,
    _m: PhantomData<(In, M)>,
}

impl<Label, M, Action, In, Out> WorldSelector<Label, Action, In, Out, M>
    where
        Label: ScheduleLabel + Clone,
        Action: TaskAction<M, In=In, Out=Out>,
        In: 'static,
        Out: 'static,
        M: 'static,
{
    #[inline]
    pub(crate) fn new(label: Label, action: Action) -> WorldSelector<Label, Action, In, Out, M> {
        Self {
            action: Cell::new(Option::Some(action)),
            output: TaskOutput::default(),
            label,
            _m: PhantomData,
        }
    }
}

impl<Label, M, Action, In, Out> Selector<WorldPtr> for WorldSelector<Label, Action, In, Out, M>
    where
        Label: ScheduleLabel + Clone,
        Action: TaskAction<M, In=In, Out=Out> + 'static,
        In: 'static,
        Out: 'static,
        M: 'static
{
    type Output = Out;

    #[inline]
    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        let world: &mut World = world.as_mut();
        if let Some(action) = self.action.take() {
            let runner = action.to_runner(self.output.clone());
            initialize_task_runner(world, self.label.clone(), runner);
            None
        } else {
            let output = self.output.take()?;
            Some(output)
        }
    }
}





