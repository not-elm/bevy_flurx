//! `Runner` defines what does the actual processing of the action.

use crate::reactor::{NativeReactor, StepReactor};
use crate::runner::app_schedule_labels::AppScheduleLabels;
pub use crate::runner::cancellation_handlers::{CancellationHandlers, CancellationId};
use crate::runner::reserve_register_runner::{ReserveRegisterRunnerPlugin, ReservedRunner};
use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::platform_support::collections::{HashMap, HashSet};
use bevy::prelude::*;
pub use output::Output;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

mod output;
mod cancellation_handlers;
mod app_schedule_labels;
mod reserve_register_runner;

pub(crate) struct RunnerPlugin;

impl Plugin for RunnerPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ReactorEntity>()
            .init_resource::<CancellationHandlersRegistry>()
            .add_plugins(ReserveRegisterRunnerPlugin)
            .add_systems(PreStartup, setup.run_if(not(resource_exists::<AppScheduleLabels>)));
    }

    fn finish(&self, app: &mut App) {
        let labels: HashSet<_> = app
            .world()
            .resource::<Schedules>()
            .iter()
            .map(|(_, s)| s.label())
            .collect();
        app.insert_resource(AppScheduleLabels(labels));
    }
}

fn setup(
    mut commands: Commands,
    schedules: Res<Schedules>,
) {
    let mut labels: HashSet<_> = schedules
        .iter()
        .map(|(_, s)| s.label())
        .collect();
    labels.insert(PreStartup.intern());
    commands.insert_resource(AppScheduleLabels(labels));
}

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
#[derive(Default, Resource, Deref, DerefMut)]
struct CancellationHandlersRegistry(HashMap<Entity, CancellationHandlers>);

#[repr(transparent)]
struct RunnersRegistry<L: Send + Sync>(HashMap<Entity, Vec<BoxedRunner>>, PhantomData<L>);

impl<L: Send + Sync> Default for RunnersRegistry<L> {
    fn default() -> Self {
        Self(HashMap::<Entity, Vec<BoxedRunner>>::default(), PhantomData)
    }
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct ReactorEntity(Entity);

#[derive(Component, Reflect, Eq, PartialEq, Hash)]
struct ReactorScheduleLabel<Label: ScheduleLabel>(PhantomData<Label>);

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    entity: Entity,
    mut runner: BoxedRunner,
)
where
    Label: ScheduleLabel,
{
    observe_remove_reactor::<Label>(entity, world);
    let mut status = RunnerIs::Running;
    world.resource_scope(|world, mut schedule_labels: Mut<AppScheduleLabels>| {
        let label = label.intern();
        let running_on_target = schedule_labels.current_running_on_target_schedule(label, world.resource::<Schedules>());
        let mut cancellation_registry = world.remove_resource::<CancellationHandlersRegistry>().expect("CancellationHandlerRegistry was not found");
        status = {
            let handers = cancellation_registry.entry(entity).or_default();
            if running_on_target {
                runner.run(world, handers)
            } else {
                RunnerIs::Running
            }
        };

        if let Some(mut runner_registry) = world.get_non_send_resource_mut::<RunnersRegistry<Label>>() {
            runner_registry.0.entry(entity).or_default().push(runner);
        } else {
            let mut schedules = world.remove_resource::<Schedules>().expect("Schedules was not found");
            let mut registry = RunnersRegistry::<Label>::default();
            if let Some(schedule) = schedules.get_mut(label) {
                schedule.add_systems(run_runners::<Label>);
            } else {
                register_runner_system::<Label>(world, &mut schedules, label, &schedule_labels);
            }

            schedule_labels.insert(label);
            registry.0.insert(entity, vec![runner]);
            world.insert_non_send_resource(registry);
            world.insert_resource(schedules);
        }
        world.insert_resource(cancellation_registry);
    });

    match status {
        RunnerIs::Completed => {
            world.trigger(StepReactor {
                reactor: entity,
            });
        }
        RunnerIs::Canceled => {
            world.commands().entity(entity).despawn();
        }
        _ => {}
    }
}

fn register_runner_system<L: ScheduleLabel>(
    world: &mut World,
    scheduels: &mut Schedules,
    label: InternedScheduleLabel,
    app_schedule_labels: &AppScheduleLabels,
) {
    if app_schedule_labels.contains(&label) {
        world.commands().send_event(ReservedRunner {
            label,
            system: || Box::new(IntoSystem::into_system(run_runners::<L>)),
        });
    } else {
        scheduels.add_systems(label, run_runners::<L>);
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
        commands.queue(move |world: &mut World| {
            let Some(mut runner_registry) = world.remove_non_send_resource::<RunnersRegistry<Label>>() else {
                return;
            };
            let Some(mut cancellation_registry) = world.remove_resource::<CancellationHandlersRegistry>() else {
                return;
            };
            runner_registry.0.remove(&entity);
            let Some(cancellation_handlers) = cancellation_registry.remove(&entity) else {
                return;
            };
            for handler in cancellation_handlers.0.values() {
                handler(world);
            }
            world.insert_non_send_resource(runner_registry);
            world.insert_resource(cancellation_registry);
        });
    });
    observer.watch_entity(entity);

    world.spawn((
        ReactorScheduleLabel(PhantomData::<Label>),
        ReactorEntity(entity),
        observer
    ));
}

fn run_runners<L: Send + Sync + 'static>(world: &mut World) -> Result{
    let Some(mut runners_registry) = world.remove_non_send_resource::<RunnersRegistry<L>>() else {
        return Ok(());
    };
    let Some(mut cancellation_registry) = world.remove_resource::<CancellationHandlersRegistry>() else {
        world.insert_non_send_resource(runners_registry);
        return Ok(());
    };
    for (entity, runners) in runners_registry.0.iter_mut() {
        let Some(cancellation_handlers) = cancellation_registry.get_mut(entity) else {
            continue;
        };
        let mut request_cancel = false;
        let mut request_step = false;
        runners.retain_mut(|runner| {
            if request_cancel {
                return false;
            }
            match runner.run(world, cancellation_handlers) {
                RunnerIs::Completed => {
                    request_step = true;
                    false
                }
                RunnerIs::Running => true,
                RunnerIs::Canceled => {
                    request_cancel = true;
                    false
                }
            }
        });
        if request_cancel {
            world.commands().entity(*entity).despawn();
        } else if request_step {
            world.commands().trigger(StepReactor {
                reactor: *entity,
            });
        }
    }
    world.insert_non_send_resource(runners_registry);
    world.insert_resource(cancellation_registry);
    Ok(())
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
    use bevy::app::{PostUpdate, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, Component, Entity, IntoScheduleConfigs, Observer, Query, ResMut, Update, World};
    use bevy::prelude::{Resource, With};
    use bevy_test_helper::resource::bool::BoolExtension;
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
        app.add_systems(Startup, (
            |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(Update, once::no_op()).await;
                }));
            },
            |mut commands: Commands| {
                commands.spawn((
                    Cancellable,
                    Reactor::schedule(|task| async move {
                        task.will(Update, wait::until(|mut count: ResMut<Count>| {
                            count.increment();
                            false
                        })).await;
                    })
                ));
            }
        ).chain());

        app.update();
        app.assert_resource_eq(Count(1));

        let _ = app.world_mut().run_system_once(|mut commands: Commands, reactor: Query<Entity, With<Cancellable>>| {
            commands.entity(reactor.single().unwrap()).despawn();
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
        app.update();
        app.update();

        let _ = app.world_mut().run_system_once(|mut commands: Commands, reactor: Query<Entity, With<Cancellable>>| {
            commands.entity(reactor.single().unwrap()).despawn();
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
                task.will(Update, wait::until(|| false)).await;
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


    struct CancelRunner;

    impl Runner for CancelRunner {
        fn run(&mut self, _: &mut World, cancellation_handlers: &mut CancellationHandlers) -> RunnerIs {
            cancellation_handlers.register(|world| {
                world.set_bool(true);
            });
            println!("DD");
            RunnerIs::Canceled
        }
    }

    fn cancel() -> ActionSeed {
        ActionSeed::new(|_, _| CancelRunner)
    }

    #[test]
    fn call_cancellation_handler_at_the_first_time() {
        let mut app = test_app();
        app.add_systems(Update, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, cancel()).await;
            }));
        });
        app.set_bool(false);
        app.update();

        assert!(app.is_bool_true());
    }
}