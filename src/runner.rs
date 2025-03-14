//! `Runner` defines what does the actual processing of the action.

use crate::reactor::NativeReactor;
pub use crate::runner::cancellation_handlers::{CancellationHandlers, CancellationId};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Commands, Component, Entity, Observer, OnRemove, Reflect, ReflectComponent, Schedules, Trigger, With, World};
pub use output::Output;
use std::marker::PhantomData;

mod output;
mod cancellation_handlers;


/// The current state of the [Runner].
pub enum RunnerIs {
    /// The runner's process is not yet complete.
    Running,
    /// The runner is complete.
    ///
    /// The runner will be deleted and will not run again in the future.
    Completed,
    /// Interrupts the process of the reactor this runner belongs to, as well as the runner itself.
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
                RunnerIs::Canceled => RunnerIs::Canceled,
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
struct ReactorMap<L: Send + Sync>(Vec<(Entity, Vec<BoxedRunner>, CancellationHandlers)>, PhantomData<L>);

impl<L: Send + Sync> Default for ReactorMap<L> {
    fn default() -> Self {
        Self(Vec::new(), PhantomData)
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ReactorEntity(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ReactorScheduleLabel<Label: ScheduleLabel>(PhantomData<Label>);

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    entity: Entity,
    runner: BoxedRunner,
)
where
    Label: ScheduleLabel,
{
    observe_remove_reactor::<Label>(entity, world);
    if let Some(mut map) = world.get_non_send_resource_mut::<ReactorMap<Label>>() {
        if let Some((_, runners, _)) = map.0.iter_mut().find(|(e, ..)| e == &entity) {
            runners.push(runner);
        } else {
            map.0.push((entity, vec![runner], CancellationHandlers::default()));
        }
    } else {
        let mut reactor_map = ReactorMap::<Label>::default();
        reactor_map.0.push((entity, vec![runner], CancellationHandlers::default()));
        world.insert_non_send_resource(reactor_map);

        let Some(mut schedules) = world.get_resource_mut::<Schedules>() else {
            return;
        };
        schedules.add_systems(label.intern(), run_runners::<Label>);
    }
}

fn observer_already_exists<Label: ScheduleLabel>(
    world: &mut World,
    reactor_entity: &Entity,
) -> bool {
    world
        .query_filtered::<&ReactorEntity, With<ReactorScheduleLabel<Label>>>()
        .iter(world)
        .any(|target| {
            &target.0 == reactor_entity
        })
}

fn observe_remove_reactor<Label: ScheduleLabel>(
    entity: Entity,
    world: &mut World,
) {
    if observer_already_exists::<Label>(world, &entity) {
        return;
    }
    let mut observer = Observer::new(move |_: Trigger<OnRemove, NativeReactor>, mut commands: Commands| {
        commands.queue(move |world: &mut World|{
            let Some(mut reactors) = world.remove_non_send_resource::<ReactorMap<Label>>() else{
                return;
            };
            let Some(i) = reactors.0.iter().position(|(e, ..)| e == &entity) else {
                return;
            };
            let (.., cancellation_handlers) = reactors.0.remove(i);
            for handler in cancellation_handlers.0.values() {
                handler(world);
            }
            world.insert_non_send_resource(reactors);
        });
    });
    observer.watch_entity(entity);

    world.spawn((
        ReactorScheduleLabel(PhantomData::<Label>),
        ReactorEntity(entity),
        observer
    ));
}

fn run_runners<L: Send + Sync + 'static>(world: &mut World) {
    let Some(mut reactor_map) = world.remove_non_send_resource::<ReactorMap<L>>() else {
        return;
    };
    for (entity, runners, token) in reactor_map.0.iter_mut() {
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
    use crate::action::{once, wait};
    use crate::prelude::{ActionSeed, CancellationHandlers, Reactor};
    use crate::reactor::NativeReactor;
    use crate::runner::{ReactorEntity, Runner, RunnerIs};
    use crate::test_util::test;
    use crate::tests::test_app;
    use bevy::app::{PostUpdate, PreStartup, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, Component, Entity, Observer, Query, ResMut, Update, World};
    use bevy::prelude::{Resource, With};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    #[derive(Component)]
    struct Cancellable;

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

    #[test]
    fn cancel_with_multiple_reactors() {
        let mut app = test_app();
        app.add_systems(PreStartup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::no_op()).await;
            }));
        });
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Cancellable,
                Reactor::schedule(|task| async move {
                    task.will(Update, wait::until(|mut count: ResMut<Count>| {
                        count.increment();
                        false
                    })).await;
                })
            ));
        });
        app.update();
        app.assert_resource_eq(Count(1));

        let _ = app.world_mut().run_system_once(|mut commands: Commands, reactor: Query<Entity, With<Cancellable>>| {
            commands.entity(reactor.get_single().unwrap()).despawn();
        });
        app.update();
        app.assert_resource_eq(Count(1));
    }

    #[test]
    fn cancel_with_multiple_schedule_labels() {
        #[derive(Resource, Debug, Default, PartialEq)]
        struct Count2(usize);

        let mut app = test_app();
        app.init_resource::<Count2>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Cancellable,
                Reactor::schedule(|task| async move {
                    let count2_task = task.run(PostUpdate, wait::until(|mut count: ResMut<Count2>| {
                        count.0 += 1;
                        false
                    })).await;
                    task.will(Update, wait::until(|mut count: ResMut<Count>| {
                        count.increment();
                        false
                    })).await;
                    count2_task.await;
                })
            ));
        });
        // start once::no_op
        app.update();
        // start wait::until
        app.update();

        let _ = app.world_mut().run_system_once(|mut commands: Commands, reactor: Query<Entity, With<Cancellable>>| {
            commands.entity(reactor.get_single().unwrap()).despawn();
        });
        app.update();
        app.assert_resource_eq(Count2(2));
    }

    #[test]
    fn avoid_spawn_unnecessary_observers() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::no_op()).await;
                task.will(Update, once::no_op()).await;
            }));
        });

        app.update();
        let num_observer = app
            .world_mut()
            .query_filtered::<&Observer, With<ReactorEntity>>()
            .iter(app.world_mut())
            .len();
        assert_eq!(num_observer, 1);
    }
}