use crate::action::Action;
use crate::runner::{initialize_runner, Output};
use crate::world_ptr::WorldPtr;
use bevy::ecs::schedule::ScheduleLabel;
use flurx::selector::Selector;
use std::cell::Cell;
use std::marker::PhantomData;
use crate::reactor::ReactorId;

pub(crate) struct WorldSelector<Label, In, Out> {
    action: Cell<Option<(ReactorId, Action<In, Out>)>>,
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
    pub(crate) fn new(label: Label, task_id: ReactorId, action: Action<In, Out>) -> WorldSelector<Label, In, Out> {
        Self {
            action: Cell::new(Some((task_id, action))),
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
    Out: 'static,
{
    type Output = Out;

    #[inline(always)]
    fn select(&self, world: WorldPtr) -> Option<Self::Output> {
        if let Some((task_id, action)) = self.action.take() {
            let runner = action.into_runner(self.output.clone());
            initialize_runner(world.as_mut(), &self.label, task_id, runner);
            None
        } else {
            self.output.take()
        }
    }
}





