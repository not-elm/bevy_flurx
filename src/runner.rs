//! `Runner` defines what does the actual processing of the action.

use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Schedule, Schedules, World};
use bevy::utils::intern::Interned;

pub use cancellation_token::{CancellationId, CancellationToken};
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
}

/// The boxed runner.
///
/// It is created by [`Action`](crate::prelude::Action).
#[repr(transparent)]
pub struct BoxedRunner(Option<Box<dyn Runner>>);

impl BoxedRunner {
    #[inline]
    pub(crate) fn new(runner: impl Runner + 'static) -> Self {
        Self(Some(Box::new(runner)))
    }
}

impl Runner for BoxedRunner {
    #[inline(always)]
    fn run(&mut self, world: &mut World, token: &CancellationToken) -> bool {
        if let Some(mut runner) = self.0.take() {
            if runner.run(world, token) {
                true
            } else {
                self.0.replace(runner);
                false
            }
        } else {
            true
        }
    }
}

#[repr(transparent)]
pub(crate) struct BoxedRunners<L: Send + Sync>(pub Vec<(BoxedRunner, CancellationToken)>, PhantomData<L>);

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    token: CancellationToken,
    runner: BoxedRunner,
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
            if token.finished_reactor(){
                false
            } else if token.is_cancellation_requested() {
                token.call_cancel_handles(world);
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


#[cfg(test)]
mod tests {
    use bevy::app::Startup;
    use bevy::prelude::{Commands, ResMut, Update, World};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::wait;
    use crate::prelude::{ActionSeed, CancellationToken, Reactor};
    use crate::runner::Runner;
    use crate::test_util::test;
    use crate::tests::test_app;

    struct TestCancelRunner {
        limit: usize,
    }

    impl Runner for TestCancelRunner {
        fn run(&mut self, world: &mut World, token: &CancellationToken) -> bool {
            let mut count = world.resource_mut::<Count>();
            count.increment();
            if self.limit == count.0 {
                token.cancel();
                true
            } else {
                false
            }
        }
    }

    fn test_action(limit: usize) -> ActionSeed {
        ActionSeed::new(move |_, _| {
            TestCancelRunner {
                limit
            }
        })
    }

    #[test]
    fn remove_reactor_after_cancel() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, test_action(3)).await;
            }));
        });
        app.update();
        app.assert_resource_eq(Count(1));
        assert_eq!(app.world.query::<&Reactor>().iter(&app.world).len(), 1);

        app.update();
        app.assert_resource_eq(Count(2));
        assert_eq!(app.world.query::<&Reactor>().iter(&app.world).len(), 1);

        app.update();
        app.assert_resource_eq(Count(3));
        assert_eq!(app.world.query::<&Reactor>().iter(&app.world).len(), 0);
    }

    #[test]
    fn test_cancel_reactor() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, wait::both(
                    test::cancel(),
                    wait::until(|mut count: ResMut<Count>| {
                        count.increment();
                        false
                    }),
                )).await;
            }));
        });
        app.update();
        for _ in 0..50 {
            app.update();
            assert_eq!(app.world.query::<&Reactor>().iter(&app.world).len(), 0);
            app.assert_resource_eq(Count(1));
        }
    }
}