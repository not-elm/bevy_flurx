use bevy::app::App;
use bevy::prelude::{OnEnter, OnExit, OnTransition, States};

use crate::inner_macros::run_tasks;

pub trait AddBevTaskStateScheduleLabel {
    fn register_task_schedule_on_enter<S: States + Copy>(&mut self, state: S) -> &mut Self;
    fn register_task_schedule_on_exit<S: States + Copy>(&mut self, state: S) -> &mut Self;
    fn register_task_schedule_on_translation<S: States + Copy>(&mut self, from: S, to: S) -> &mut Self;
}


impl AddBevTaskStateScheduleLabel for App {
    #[inline]
    fn register_task_schedule_on_enter<S: States + Copy>(&mut self, state: S) -> &mut Self {
        self.add_systems(OnEnter(state), run_tasks!(OnEnter(state)))
    }

    #[inline]
    fn register_task_schedule_on_exit<S: States + Copy>(&mut self, state: S) -> &mut Self {
        self.add_systems(OnExit(state), run_tasks!(OnExit(state)))
    }

    #[inline]
    fn register_task_schedule_on_translation<S: States + Copy>(&mut self, from: S, to: S) -> &mut Self {
        self.add_systems(OnTransition { from, to }, run_tasks!(OnTransition{from, to}))
    }
}