use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Schedule, Schedules};
use store::selector::StateSelector;
use store::task::Task;

use crate::store::WorldPointer;

mod selector;
pub mod once;
mod wait;

pub struct BevyTask<'a> {
    pub(crate) inner: Task<'a, WorldPointer>,
}


impl<'a> BevyTask<'a>

{
    pub async fn run<Out>(
        &self,
        select: impl StateSelector<WorldPointer, Output=Out>,
    ) -> Out
    {
        self.inner.run(select).await
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