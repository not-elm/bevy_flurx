use crate::action::Action;
use crate::runner::{initialize_runner, Output};
use crate::world_ptr::WorldPtr;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::Entity;
use std::marker::PhantomData;
use crate::core::selector::Selector;

pub(crate) struct WorldSelector<Label, In, Out> {
    action: Option<(Entity, Action<In, Out>)>,
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
    pub(crate) fn new(label: Label, entity: Entity, action: Action<In, Out>) -> WorldSelector<Label, In, Out> {
        Self {
            action: Some((entity, action)),
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
    fn select(&mut self, world: WorldPtr) -> Option<Self::Output> {
        if let Some((entity, action)) = self.action.take() {
            let runner = action.into_runner(self.output.clone());
            initialize_runner(world.as_mut(), &self.label, entity, runner);
            None
        } else {
            self.output.take()
        }
    }
}





