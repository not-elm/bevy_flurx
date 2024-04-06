use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystemConfigs, Schedule, Schedules, World};

use crate::action::{Action};
use crate::flurx_initialize;
use crate::runner::runners::TaskRunners;

pub(crate) mod runners;
pub(crate) mod multi_times;
pub(crate) mod once;
pub(crate) mod sequence;
pub(crate) mod both;
pub(crate) mod either;
pub(crate) mod base;
pub(crate) mod pipe;


/// Represents the output of the task.
/// See details [`TaskRunner`].
pub struct TaskOutput<O>(Rc<RefCell<Option<O>>>);


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
    #[inline(always)]
    pub fn replace(&self, o: O) {
        self.0.borrow_mut().replace(o);
    }

    #[inline(always)]
    pub fn take(&self) -> Option<O> {
        self.0.borrow_mut().take()
    }

    #[inline(always)]
    pub fn is_some(&self) -> bool {
        self.0.borrow().is_some()
    }

    #[inline(always)]
    pub fn is_none(&self) -> bool {
        self.0.borrow().is_none()
    }
}

impl<O: Clone> TaskOutput<O> {
    #[inline(always)]
    pub fn cloned(&self) -> Option<O> {
        self.0.borrow_mut().clone()
    }
}


/// Structure for canceling a task
#[derive(Default, Clone)]
pub struct CancellationToken(Rc<RefCell<Option<()>>>);

impl CancellationToken {
    #[inline(always)]
    pub fn requested_cancel(&self) -> bool {
        self.0.borrow().is_some()
    }


    #[inline(always)]
    pub fn cancel(&self) {
        self.0.borrow_mut().replace(());
    }
}


///
pub trait TaskRunner {
    /// Run the system. 
    ///
    /// The structure that implements [`TaskRunner`] is given [`TaskOutput`],
    /// if the system termination condition is met, return `true` and
    /// pass the system output to [`TaskOutput`].
    ///
    fn run(&mut self, world: &mut World) -> bool;
}

pub(crate) fn initialize_task_runner<Label>(
    world: &mut World,
    label: Label,
    runner: impl TaskRunner + 'static,
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
        schedule.add_systems(run_task_runners::<Label>.after(flurx_initialize));

        let mut runners = TaskRunners::<Label>::default();
        runners.runners.push(Box::new(runner));
        world.insert_non_send_resource(runners);
    }
}

pub(crate) fn initialize_schedule<Label>(schedules: &mut Schedules, schedule_label: Label) -> &mut Schedule
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

pub trait RunWithTaskOutput<O> {
    type In;

    fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<O>, world: &mut World) -> bool;
}

impl<O, R: RunWithTaskOutput<O>> TaskRunner for (CancellationToken, TaskOutput<O>, R) {
    #[inline(always)]
    fn run(&mut self, world: &mut World) -> bool {
        self.2.run_with_task_output(&mut self.0, &mut self.1, world)
    }
}

pub struct RunnerIntoAction<O, R>(pub R, PhantomData<O>);

impl<O, R> RunnerIntoAction<O, R>
    where
        R: RunWithTaskOutput<O>
{
    #[inline]
    pub const fn new(runner: R) -> RunnerIntoAction<O, R> {
        RunnerIntoAction(runner, PhantomData)
    }
}

impl<O, R> Action<R::In, O> for RunnerIntoAction<O, R>
    where
        O: 'static,
        R: RunWithTaskOutput<O> + 'static
{
    #[inline(always)]
    fn to_runner(self, token: CancellationToken, output: TaskOutput<O>) -> impl TaskRunner {
        (token, output, self.0)
    }
}


pub(crate) mod macros {
    macro_rules! output_combine {
        ($o1: expr, $o2: expr, $output: expr $(,)?) => {
            if let Some(out1) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.replace((out1, out2));
                    true
                } else {
                    $o1.replace(out1);
                    false
                }
            } else {
                false
            }
        };

        ($o1: expr, $o2: expr, $output: expr $(,$out: ident)*) => {
            if let Some(($($out,)*)) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.replace(($($out,)* out2));
                    true
                } else {
                    $o1.replace(($($out,)*));
                    false
                }
            } else {
                false
            }
        };
    }

    macro_rules! impl_tuple_runner {
        ($impl_macro: ident) => {
            $impl_macro!(In1);
            $impl_macro!(In1,In2);
            $impl_macro!(In1,In2,In3);
            $impl_macro!(In1,In2,In3,In4);
            $impl_macro!(In1,In2,In3,In4,In5);
            $impl_macro!(In1,In2,In3,In4,In5,In6);
            $impl_macro!(In1,In2,In3,In4,In5,In6,In7);
            $impl_macro!(In1,In2,In3,In4,In5,In6,In7,In8);
            $impl_macro!(In1,In2,In3,In4,In5,In6,In7,In8,In9);
            $impl_macro!(In1,In2,In3,In4,In5,In6,In7,In8,In9,In10);
            $impl_macro!(In1,In2,In3,In4,In5,In6,In7,In8,In9,In10,In11);
            $impl_macro!(In1,In2,In3,In4,In5,In6,In7,In8,In9,In10,In11,In12);
        };
    }

    pub(crate) use output_combine;
    pub(crate) use impl_tuple_runner;
}