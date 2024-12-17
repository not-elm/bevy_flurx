//! `Runner` defines what does the actual processing of the action.

pub use crate::runner::cancellation_token::{CancellationId, CancellationToken};
use bevy::ecs::intern::Interned;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Entity, Schedule, Schedules, World};
pub(crate) use cancellation_token::CallCancellationHandlers;
pub use output::Output;
use std::marker::PhantomData;

mod output;
mod cancellation_token;

pub enum RunnerStatus {
    Pending,
    Ready,
    Cancel,
}

impl RunnerStatus {
    #[inline(always)]
    pub const fn is_ready(&self) -> bool {
        matches!(self, RunnerStatus::Ready)
    }

    #[inline(always)]
    pub const fn is_cancel(&self) -> bool {
        matches!(self, RunnerStatus::Cancel)
    }
}

/// The structure that implements [`Runner`] is given [`Output`],
/// if the system termination condition is met, return `true` and
/// pass the system output to [`Output`].
pub trait Runner {
    /// Run the system.
    ///
    /// If this runner finishes, it must return `true`.
    /// If it returns `true`, an entity attached this runner will be removed.
    fn run(&mut self, world: &mut World, token: &mut CancellationToken) -> RunnerStatus;
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
    fn run(&mut self, world: &mut World, token: &mut CancellationToken) -> RunnerStatus {
        if let Some(mut runner) = self.0.take() {
            match runner.run(world, token) {
                RunnerStatus::Ready => RunnerStatus::Ready,
                status => {
                    self.0.replace(runner);
                    status
                }
            }
        } else {
            RunnerStatus::Ready
        }
    }
}

#[derive(Component)]
#[repr(transparent)]
pub(crate) struct BoxedRunners<L: Send + Sync>(pub Vec<BoxedRunner>, PhantomData<L>);
unsafe impl<L: Send + Sync> Send for BoxedRunners<L> {}

unsafe impl<L: Send + Sync> Sync for BoxedRunners<L> {}
impl<L: Send + Sync> Default for BoxedRunners<L> {
    fn default() -> Self {
        Self(Vec::new(), PhantomData)
    }
}

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    entity: Entity,
    runner: BoxedRunner,
)
where
    Label: ScheduleLabel,
{
    if let Ok(mut runners) = world.query::<&mut BoxedRunners<Label>>().get_mut(world, entity) {
        runners.0.push(runner);
    } else {
        // world.insert_non_send_resource(BoxedRunners::<Label>(vec![runner], PhantomData));
        world.entity_mut(entity).insert(BoxedRunners::<Label>(vec![runner], PhantomData));
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
    let query: Vec<_> = {
        let mut runners = world.query::<(
            Entity,
            &mut BoxedRunners<L>,
            &mut CancellationToken,
        )>();
        runners.iter_mut(world).map(|(entity, mut runners, mut token)| (
            entity,
            std::mem::take(&mut *runners),
            std::mem::take(&mut *token),
        )).collect()
    };
    for (entity, mut runners, mut token) in query {
        let mut request_cancel = false;
        runners.0.retain_mut(|runner| {
            if request_cancel {
                return false;
            }
            match runner.run(world, &mut token) {
                RunnerStatus::Ready => false,
                RunnerStatus::Pending => true,
                RunnerStatus::Cancel => {
                    request_cancel = true;
                    false
                }
            }
        });
        if request_cancel {
            world.entity_mut(entity).despawn();
        } else {
            world.entity_mut(entity).insert((
                runners,
                token,
            ));
        }
    }
}

pub(crate) mod macros {
    macro_rules! output_combine {
        ($o1: expr, $o2: expr, $output: expr $(,)?) => {
            if let Some(out1) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.set((out1, out2));
                    $crate::prelude::RunnerStatus::Ready
                } else {
                    $o1.set(out1);
                    $crate::prelude::RunnerStatus::Pending
                }
            } else {
                $crate::prelude::RunnerStatus::Pending
            }
        };

        ($o1: expr, $o2: expr, $output: expr $(,$out: ident)*) => {
            if let Some(($($out,)*)) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.set(($($out,)* out2));
                    $crate::prelude::RunnerStatus::Ready
                } else {
                    $o1.set(($($out,)*));
                     $crate::prelude::RunnerStatus::Pending
                }
            } else {
               $crate::prelude:: RunnerStatus::Pending
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
    use crate::action::wait;
    use crate::prelude::{ActionSeed, CancellationToken, Flow, Reactor};
    use crate::runner::{Runner, RunnerStatus};
    use crate::test_util::test;
    use crate::tests::test_app;
    use bevy::app::Startup;
    use bevy::prelude::{Commands, ResMut, Update, World};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    struct TestCancelRunner {
        limit: usize,
    }

    impl Runner for TestCancelRunner {
        fn run(&mut self, world: &mut World, _: &mut CancellationToken) -> RunnerStatus {
            let mut count = world.resource_mut::<Count>();
            count.increment();
            if self.limit == count.0 {
                RunnerStatus::Cancel
            } else {
                RunnerStatus::Pending
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
            commands.spawn(Flow::schedule(|task| async move {
                task.will(Update, test_action(3)).await;
            }));
        });
        app.update();
        app.assert_resource_eq(Count(1));
        assert_eq!(app.world_mut().query::<&Reactor>().iter(app.world()).len(), 1);

        app.update();
        app.assert_resource_eq(Count(2));
        assert_eq!(app.world_mut().query::<&Reactor>().iter(app.world()).len(), 1);

        app.update();
        app.assert_resource_eq(Count(3));
        assert_eq!(app.world_mut().query::<&Reactor>().iter(app.world()).len(), 0);
    }

    #[test]
    fn test_cancel_reactor() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Flow::schedule(|task| async move {
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
            assert_eq!(app.world_mut().query::<&Reactor>().iter(app.world()).len(), 0);
            app.assert_resource_eq(Count(0));
        }
    }
}