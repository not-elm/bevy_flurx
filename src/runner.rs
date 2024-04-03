use std::cell::RefCell;
use std::rc::Rc;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Schedule, Schedules, World};

use crate::runner::runners::TaskRunners;

pub(crate) mod runners;
pub(crate) mod multi_times;
pub(crate) mod once;
pub(crate) mod sequence;
pub(crate) mod both;
pub(crate) mod either;


pub(crate) struct TaskOutput<O>(Rc<RefCell<Option<O>>>);

impl<O> Clone for TaskOutput<O> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<O> Default for TaskOutput<O> {
    #[inline]
    fn default() -> Self {
        Self(Rc::new(RefCell::new(Option::None)))
    }
}

impl<O> TaskOutput<O> {
    #[inline]
    pub fn replace(&self, o: O) {
        self.0.borrow_mut().replace(o);
    }

    #[inline]
    pub fn take(&self) -> Option<O> {
        self.0.borrow_mut().take()
    }

    #[inline(always)]
    pub fn is_none(&self) -> bool {
        self.0.borrow().is_none()
    }
}


pub trait RunTask {
    fn run(&mut self, world: &mut World) -> bool;
}

pub(crate) fn initialize_task_runner<Label>(
    world: &mut World,
    label: Label,
    runner: impl RunTask + 'static,
)
    where Label: ScheduleLabel + Clone
{
    if let Some(mut runners) = world.get_non_send_resource_mut::<TaskRunners<Label>>() {
        runners.runners.push(Box::new(runner));
    } else {
        let Some(mut schedules) = world.get_resource_mut::<Schedules>() else {
            return;
        };

        let schedule = initialize_schedule(&mut schedules, label);
        schedule.add_systems(run_task_runners::<Label>);

        let mut runners = TaskRunners::<Label>::default();
        runners.runners.push(Box::new(runner));
        world.insert_non_send_resource(runners);
    }
}

fn initialize_schedule<Label>(schedules: &mut Schedules, schedule_label: Label) -> &mut Schedule
    where Label: ScheduleLabel + Clone
{
    if !schedules.contains(schedule_label.clone()) {
        schedules.insert(Schedule::new(schedule_label.clone()));
    }

    schedules.get_mut(schedule_label.intern()).unwrap()
}

fn run_task_runners<Label: ScheduleLabel>(world: &mut World) {
    let Some(mut runner) = world.remove_non_send_resource::<TaskRunners<Label>>() else {
        return;
    };
    runner.run(world);
    world.insert_non_send_resource(runner);
}


