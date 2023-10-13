use bevy::app::App;
use bevy::ecs::schedule::ScheduleLabel;

use crate::inner_macros::run_tasks;

pub trait AddBevTaskSchedule {
    fn register_task_schedule(&mut self, schedule_label: impl ScheduleLabel + Clone) -> &mut Self;
}


impl AddBevTaskSchedule for App {
    #[inline(always)]
    fn register_task_schedule(&mut self, schedule_label: impl ScheduleLabel + Clone) -> &mut Self {
        self.add_systems(schedule_label.clone(), run_tasks!(schedule_label.clone()))
    }
}