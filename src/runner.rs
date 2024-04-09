use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Deref, DerefMut, Entity, Resource, Schedule, Schedules, World};
use bevy::utils::intern::Interned;

pub use cancellation_token::CancellationToken;
pub use output::Output;

use crate::world_ptr::WorldPtr;

mod output;
mod cancellation_token;

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
pub struct BoxedRunner(pub(crate) Box<dyn Runner>);

/// SAFETY: This structure must be used only with [`run_runners`].
unsafe impl Send for BoxedRunner {}

/// SAFETY: This structure must be used only with [`run_runners`].
unsafe impl Sync for BoxedRunner {}

#[repr(transparent)]
#[derive(Resource)]
struct RunRunnersSystemInitialized<L: Send + Sync>(PhantomData<L>);

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    runner: BoxedRunner,
)
    where Label: ScheduleLabel
{
    world.spawn(runner);
    if !world.contains_resource::<RunRunnersSystemInitialized<Label>>() {
        world.insert_resource(RunRunnersSystemInitialized::<Label>(PhantomData));
        let Some(mut schedules) = world.get_resource_mut::<Schedules>() else {
            return;
        };

        let schedule = initialize_schedule(&mut schedules, label.intern());
        schedule.add_systems(run_runners);
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
fn run_runners(world: &mut World) {
    let world_ptr = WorldPtr::new(world);
    for (entity, mut runner) in world.query::<(Entity, &mut BoxedRunner)>().iter_mut(world) {
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