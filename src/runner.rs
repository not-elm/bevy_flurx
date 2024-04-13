use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Deref, DerefMut, Resource, Schedule, Schedules, World};
use bevy::utils::intern::Interned;

pub use cancellation_token::CancellationToken;
pub use output::Output;

mod output;
mod cancellation_token;

/// The structure that implements [`Runner`] is given [`Output`],
/// if the system termination condition is met, return `true` and
/// pass the system output to [`Output`].
pub trait Runner {
    /// Run the system.
    ///
    /// If this runner finishes, it must return `true`.
    /// If it returns `true`, an entity attached this runner will be removed.
    fn run(&mut self, world: &mut World) -> bool;
}

#[repr(transparent)]
#[derive(Deref, DerefMut)]
pub struct BoxedRunner(pub(crate) Box<dyn Runner>);

#[repr(transparent)]
#[derive(Resource)]
struct BoxedRunners<L: Send + Sync>(Vec<BoxedRunner>, PhantomData<L>);

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    runner: BoxedRunner,
)
    where Label: ScheduleLabel
{
    if let Some(mut runners) = world.get_non_send_resource_mut::<BoxedRunners<Label>>() {
        runners.0.push(runner);
    } else {
        world.insert_non_send_resource(BoxedRunners::<Label>(vec![runner], PhantomData));
        let Some(mut schedules) = world.get_resource_mut::<Schedules>() else {
            return;
        };

        let schedule = initialize_schedule(&mut schedules, label.intern());
        schedule.add_systems(run_runners::<Label>);
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
fn run_runners<L: Send + Sync + 'static>(world: &mut World) {
    if let Some(mut runners) = world.remove_non_send_resource::<BoxedRunners<L>>() {
        runners.0.retain_mut(|r| !r.0.run(world));
        world.insert_non_send_resource(runners);
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