//! `Runner` defines what does the actual processing of the action.

use crate::reactor::NativeReactor;
pub use crate::runner::cancellation_token::{CancellationHandlers, CancellationId};
use bevy::ecs::intern::Interned;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Entity, EventWriter, NonSendMut, Observer, OnRemove, Schedule, Schedules, Trigger, World};
use bevy::utils::HashMap;
pub(crate) use cancellation_token::CallCancellationHandlers;
pub use output::Output;
use std::marker::PhantomData;

mod output;
mod cancellation_token;

pub enum RunnerIs {
    /// The runner is running.
    Running,
    /// The runner just completes.
    ///
    /// The runner in this state will be deleted.
    Completed,
    /// This will cancel the runner and the processing of the reactor it belongs to.
    Canceled,
}

impl RunnerIs {
    /// Returns whether a runner has been completed.
    #[inline(always)]
    pub const fn is_completed(&self) -> bool {
        matches!(self, RunnerIs::Completed)
    }

    /// Returns whether a runner has been canceled.
    #[inline(always)]
    pub const fn is_cancel(&self) -> bool {
        matches!(self, RunnerIs::Canceled)
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
    fn run(&mut self, world: &mut World, cancellation_handlers: &mut CancellationHandlers) -> RunnerIs;
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
    fn run(&mut self, world: &mut World, cancellation_handlers: &mut CancellationHandlers) -> RunnerIs {
        if let Some(mut runner) = self.0.take() {
            match runner.run(world, cancellation_handlers) {
                RunnerIs::Completed => RunnerIs::Completed,
                status => {
                    self.0.replace(runner);
                    status
                }
            }
        } else {
            RunnerIs::Completed
        }
    }
}

#[repr(transparent)]
struct ReactorMap<L: Send + Sync>(pub HashMap<Entity, (Vec<BoxedRunner>, CancellationHandlers)>, PhantomData<L>);

impl<L: Send + Sync> Default for ReactorMap<L> {
    fn default() -> Self {
        Self(HashMap::new(), PhantomData)
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
    if let Some(mut map) = world.get_non_send_resource_mut::<ReactorMap<Label>>() {
        if let Some((runners, _)) = map.0.get_mut(&entity) {
            runners.push(runner);
        } else {
            map.0.insert(entity, (vec![runner], CancellationHandlers::default()));
        }
    } else {
        observe_remove_reactor::<Label>(entity, world);
        let mut reactor_map = ReactorMap::<Label>::default();
        reactor_map.0.insert(entity, (vec![runner], CancellationHandlers::default()));
        world.insert_non_send_resource(reactor_map);
        let Some(mut schedules) = world.get_resource_mut::<Schedules>() else {
            return;
        };
        let schedule = initialize_schedule(&mut schedules, label.intern());
        schedule.add_systems(run_runners::<Label>);
    }
}

fn observe_remove_reactor<Label: ScheduleLabel>(entity: Entity, world: &mut World) {
    let mut observer = Observer::new(
        move |_: Trigger<OnRemove, NativeReactor>, mut reactor_map: NonSendMut<ReactorMap<Label>>, mut ew: EventWriter<CallCancellationHandlers>| {
            if let Some((_, token)) = reactor_map.0.remove(&entity) {
                ew.send(CallCancellationHandlers(token));
            }
        });
    observer.watch_entity(entity);
    world.commands().spawn(observer);
}

#[inline]
pub(crate) fn initialize_schedule(schedules: &mut Schedules, schedule_label: Interned<dyn ScheduleLabel>) -> &mut Schedule {
    if schedules.get(schedule_label).is_none() {
        schedules.insert(Schedule::new(schedule_label));
    }

    schedules.get_mut(schedule_label).unwrap()
}

fn run_runners<L: Send + Sync + 'static>(world: &mut World) {
    let Some(mut reactor_map) = world.remove_non_send_resource::<ReactorMap<L>>() else {
        return;
    };
    for (entity, (runners, token)) in reactor_map.0.iter_mut() {
        let mut request_cancel = false;
        runners.retain_mut(|runner| {
            if request_cancel {
                return false;
            }
            match runner.run(world, token) {
                RunnerIs::Completed => false,
                RunnerIs::Running => true,
                RunnerIs::Canceled => {
                    request_cancel = true;
                    false
                }
            }
        });
        if request_cancel {
            world.commands().entity(*entity).despawn();
        }
    }
    world.insert_non_send_resource(reactor_map);
}

pub(crate) mod macros {
    macro_rules! output_combine {
        ($o1: expr, $o2: expr, $output: expr $(,)?) => {
            if let Some(out1) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.set((out1, out2));
                    $crate::prelude::RunnerIs::Completed
                } else {
                    $o1.set(out1);
                    $crate::prelude::RunnerIs::Running
                }
            } else {
                $crate::prelude::RunnerIs::Running
            }
        };

        ($o1: expr, $o2: expr, $output: expr $(,$out: ident)*) => {
            if let Some(($($out,)*)) = $o1.take() {
                if let Some(out2) = $o2.take() {
                    $output.set(($($out,)* out2));
                    $crate::prelude::RunnerStatus:Readyy
                } else {
                    $o1.set(($($out,)*));
                     $crate::prelude::RunnerStatus:Pendingg
                }
            } else {
               $crate::prelude:: RunnerStatus:Pendingg
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
    use crate::prelude::{ActionSeed, CancellationHandlers, Reactor};
    use crate::runner::{RunnerIs, Runner};
    use crate::test_util::test;
    use crate::tests::test_app;
    use bevy::app::Startup;
    use bevy::prelude::{Commands, ResMut, Update, World};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;
    use crate::reactor::NativeReactor;

    struct TestCancelRunner {
        limit: usize,
    }

    impl Runner for TestCancelRunner {
        fn run(&mut self, world: &mut World, _: &mut CancellationHandlers) -> RunnerIs {
            let mut count = world.resource_mut::<Count>();
            count.increment();
            if self.limit == count.0 {
                RunnerIs::Canceled
            } else {
                RunnerIs::Running
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
        assert_eq!(app.world_mut().query::<&NativeReactor>().iter(app.world()).len(), 1);

        app.update();
        app.assert_resource_eq(Count(2));
        assert_eq!(app.world_mut().query::<&NativeReactor>().iter(app.world()).len(), 1);

        app.update();
        app.assert_resource_eq(Count(3));
        assert_eq!(app.world_mut().query::<&NativeReactor>().iter(app.world()).len(), 0);
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
            assert_eq!(app.world_mut().query::<&NativeReactor>().iter(app.world()).len(), 0);
            app.assert_resource_eq(Count(0));
        }
    }
}