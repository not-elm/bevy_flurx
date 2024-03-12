use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Schedule, Schedules};
use flurx::selector::Selector;



use crate::world_ptr::WorldPtr;

pub struct TaskCreator<'a> {
    pub(crate) inner: flurx::task::TaskCreator<'a, WorldPtr>,
}


impl<'a> TaskCreator<'a> {
    pub async fn task<Out>(
        &self,
        select: impl Selector<WorldPtr, Output=Out>,
    ) -> Out
    {
        self.inner.task(select).await
    }
}


fn _schedule_initialize<Label>(schedules: &mut Schedules, schedule_label: Label) -> &mut Schedule
    where Label: ScheduleLabel + Clone
{
    if !schedules.contains(schedule_label.clone()) {
        schedules.insert(Schedule::new(schedule_label.clone()));
    }

    schedules.get_mut(schedule_label).unwrap()
}