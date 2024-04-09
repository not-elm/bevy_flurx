use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Deref, DerefMut, Entity, Resource, Schedule, Schedules, World};
use bevy::utils::intern::Interned;

use crate::world_ptr::WorldPtr;

/// Represents the output of the task.
/// See details [`Runner`].
pub struct Output<O>(Rc<RefCell<Option<O>>>);

impl<O> Clone for Output<O> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<O> Default for Output<O> {
    #[inline]
    fn default() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }
}

impl<O> Output<O> {
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

impl<O: Clone> Output<O> {
    #[inline(always)]
    pub fn cloned(&self) -> Option<O> {
        self.0.borrow().clone()
    }
}

/// Structure for canceling a task
#[derive(Default)]
pub struct CancellationToken(Rc<Cell<bool>>);

impl Clone for CancellationToken {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl CancellationToken {
    #[inline(always)]
    pub fn requested_cancel(&self) -> bool {
        self.0.get()
    }

    #[inline(always)]
    pub fn cancel(&self) {
        self.0.set(true);
    }
}


///
pub trait Runner {
    /// Run the system. 
    ///
    /// The structure that implements [`Runner`] is given [`Output`],
    /// if the system termination condition is met, return `true` and
    /// pass the system output to [`Output`].
    ///
    fn run(&mut self, world: &mut World) -> bool;
}


#[repr(transparent)]
#[derive(Component, Deref, DerefMut)]
pub struct BoxedActionRunner(pub(crate) Box<dyn Runner>);

/// SAFETY: This structure must be used only with [`run_task_runners`].
unsafe impl Send for BoxedActionRunner {}

/// SAFETY: This structure must be used only with [`run_task_runners`].
unsafe impl Sync for BoxedActionRunner {}

#[repr(transparent)]
#[derive(Resource)]
struct TaskRunnerActionInitialized<L: Send + Sync>(PhantomData<L>);

pub(crate) fn initialize_task_runner<Label>(
    world: &mut World,
    label: &Label,
    runner: BoxedActionRunner,
)
    where Label: ScheduleLabel
{
    world.spawn(runner);
    if !world.contains_resource::<TaskRunnerActionInitialized<Label>>() {
        world.insert_resource(TaskRunnerActionInitialized::<Label>(PhantomData));
        let Some(mut schedules) = world.get_resource_mut::<Schedules>() else {
            return;
        };

        let schedule = initialize_schedule(&mut schedules, label.intern());
        schedule.add_systems(run_task_runners);
    }
}

#[inline]
pub(crate) fn initialize_schedule(schedules: &mut Schedules, schedule_label: Interned<dyn ScheduleLabel>) -> &mut Schedule {
    if schedules.get(schedule_label).is_none() {
        schedules.insert(Schedule::new(schedule_label));
    }

    schedules.get_mut(schedule_label).unwrap()
}

#[inline]
fn run_task_runners(world: &mut World) {
    let world_ptr = WorldPtr::new(world);
    for (entity, mut runner) in world.query::<(Entity, &mut BoxedActionRunner)>().iter_mut(world) {
        let world = world_ptr.as_mut();
        if runner.0.run(world) {
            world.despawn(entity);
        }
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