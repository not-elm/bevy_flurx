use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::BoxedSystem;

use crate::selector::WorldSelector;
use crate::world_ptr::WorldPtr;

pub struct TaskCreator<'a> {
    pub(crate) inner: flurx::task::TaskCreator<'a,  WorldPtr>,
}


impl<'a> TaskCreator<'a> {
    pub async fn task<Label, Out>(
        &self,
        label: Label,
        system: BoxedSystem<(), Option<Out>>,
    ) -> Out
        where
            Out: 'static,
            Label: ScheduleLabel + Clone
    {
        self.inner.task(WorldSelector::new(label, (), system)).await
    }

    pub async fn task_with<Label, In, Out>(
        &self,
        label: Label,
        input: In,
        system: BoxedSystem<In, Option<Out>>,
    ) -> Out
        where
            In: Clone + 'static,
            Out: 'static,
            Label: ScheduleLabel + Clone
    {
        self.inner.task(WorldSelector::new(label, input, system)).await
    }
}


