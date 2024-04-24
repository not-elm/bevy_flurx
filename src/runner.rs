//! `Runner` defines what does the actual processing of the action.

use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Resource, Schedule, Schedules, World};
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
    fn run(&mut self, world: &mut World, token: &CancellationToken) -> bool;

    /// It called when [`Reactor`](crate::prelude::Reactor) processing is cancelled by [`CancellationToken::cancel`].
    fn on_cancelled(&mut self, world: &mut World);
}

/// The boxed runner.
///
/// It is created by [`Action`](crate::prelude::Action).
#[repr(transparent)]
pub struct BoxedRunner(Box<dyn Runner>);

impl BoxedRunner {
    #[inline]
    pub(crate) fn new(runner: impl Runner + 'static) -> Self {
        Self(Box::new(runner))
    }
}

impl Runner for BoxedRunner {
    #[inline(always)]
    fn run(&mut self, world: &mut World, token: &CancellationToken) -> bool {
        if token.requested_cancel() {
            true
        } else {
            self.0.run(world, token)
        }
    }

    #[inline(always)]
    fn on_cancelled(&mut self, world: &mut World) {
        self.0.on_cancelled(world);
    }
}

#[repr(transparent)]
#[derive(Resource)]
struct BoxedRunners<L: Send + Sync>(Vec<(BoxedRunner, CancellationToken)>, PhantomData<L>);

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    token: CancellationToken,
    runner: BoxedRunner
)
    where Label: ScheduleLabel
{
    if let Some(mut runners) = world.get_non_send_resource_mut::<BoxedRunners<Label>>() {
        runners.0.push((runner, token));
    } else {
        world.insert_non_send_resource(BoxedRunners::<Label>(vec![(runner, token)], PhantomData));
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

fn run_runners<L: Send + Sync + 'static>(world: &mut World) {
    if let Some(mut runners) = world.remove_non_send_resource::<BoxedRunners<L>>() {
        runners.0.retain_mut(|(runner, token)| {
            if token.requested_cancel() {
                runner.on_cancelled(world);
                false
            } else {
                !runner.run(world, token)
            }
        });
        world.insert_non_send_resource(runners);
    }
}

pub(crate) mod macros {
    macro_rules! output_combine {
        ($o1: expr, $o2: expr, $output: expr $(,)?) => {
            if let Some(out1) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.set((out1, out2));
                    true
                } else {
                    $o1.set(out1);
                    false
                }
            } else {
                false
            }
        };

        ($o1: expr, $o2: expr, $output: expr $(,$out: ident)*) => {
            if let Some(($($out,)*)) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.set(($($out,)* out2));
                    true
                } else {
                    $o1.set(($($out,)*));
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