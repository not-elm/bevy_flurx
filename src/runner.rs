//! `Runner` defines what does the actual processing of the action.

use crate::reactor::{NativeReactor, StepReactor};
use crate::runner::app_schedule_labels::AppScheduleLabels;
pub use crate::runner::cancellation_handlers::{CancellationHandlers, CancellationId};
use crate::runner::reserve_register_runner::{ReserveRegisterRunnerPlugin, ReservedRunner};
use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use core::marker::PhantomData;
pub use output::Output;
use serde::*;

mod app_schedule_labels;
mod cancellation_handlers;
mod output;
mod reserve_register_runner;

pub(crate) struct RunnerPlugin;

impl Plugin for RunnerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RunnerIs>()
            .add_plugins(ReserveRegisterRunnerPlugin)
            .add_systems(
                PreStartup,
                setup.run_if(not(resource_exists::<AppScheduleLabels>)),
            );
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

fn setup(mut commands: Commands, schedules: Res<Schedules>) {
    let mut labels: HashSet<_> = schedules.iter().map(|(_, s)| s.label()).collect();
    labels.insert(PreStartup.intern());
    commands.insert_resource(AppScheduleLabels(labels));
}

/// The current state of the [Runner].
#[derive(Debug, Reflect, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[reflect(Serialize, Deserialize)]
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
    fn run(
        &mut self,
        world: &mut World,
        cancellation_handlers: &mut CancellationHandlers,
    ) -> RunnerIs;
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
    fn run(
        &mut self,
        world: &mut World,
        cancellation_handlers: &mut CancellationHandlers,
    ) -> RunnerIs {
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
struct RunnersRegistry<L: Send + Sync>(
    HashMap<Entity, (Vec<BoxedRunner>, CancellationHandlers)>,
    PhantomData<L>,
);

impl<L: Send + Sync> Default for RunnersRegistry<L> {
    fn default() -> Self {
        Self(HashMap::default(), PhantomData)
    }
}

#[derive(Component, Reflect, Eq, PartialEq, Hash)]
struct ReactorScheduleLabel<Label: ScheduleLabel>(PhantomData<Label>);

pub(crate) fn initialize_runner<Label>(
    world: &mut World,
    label: &Label,
    reactor_entity: Entity,
    mut runner: BoxedRunner,
) where
    Label: ScheduleLabel,
{
    observe_remove_reactor::<Label>(reactor_entity, world);
    let label = label.intern();
    let (running_on_target, contains_label) = register_app_schedule_labels(world, label);
    add_runner_system_into_schedules::<Label>(world, label, contains_label);
    let runner_is = init_runner::<Label>(world, &mut runner, reactor_entity, running_on_target);
    push_runner_into_registry::<Label>(world, reactor_entity, runner);
    match runner_is {
        RunnerIs::Completed => {
            world.trigger(StepReactor {
                reactor: reactor_entity,
            });
        }
        RunnerIs::Canceled => {
            world.commands().entity(reactor_entity).despawn();
        }
        _ => {}
    }
}

fn register_app_schedule_labels(world: &mut World, label: InternedScheduleLabel) -> (bool, bool) {
    world.resource_scope(|world, mut schedule_labels: Mut<AppScheduleLabels>| {
        let running_on_target = schedule_labels
            .current_running_on_target_schedule(label, world.resource::<Schedules>());
        let contains_label = schedule_labels.contains(&label);
        schedule_labels.insert(label);
        (running_on_target, contains_label)
    })
}

fn add_runner_system_into_schedules<Label: ScheduleLabel>(
    world: &mut World,
    label: InternedScheduleLabel,
    contains_label: bool,
) {
    if !world.contains_non_send::<RunnersRegistry<Label>>() {
        world.insert_non_send_resource(RunnersRegistry::<Label>::default());
        let mut schedules = world
            .remove_resource::<Schedules>()
            .expect("Schedules was not found");
        if let Some(schedule) = schedules.get_mut(label) {
            schedule.add_systems(run_runners::<Label>);
        } else {
            register_runner_system::<Label>(world, &mut schedules, label, contains_label);
        }
        world.insert_resource(schedules);
    }
}

fn register_runner_system<L: ScheduleLabel>(
    world: &mut World,
    schedules: &mut Schedules,
    label: InternedScheduleLabel,
    contains_label: bool,
) {
    if contains_label {
        world.commands().write_message(ReservedRunner {
            label,
            system: || Box::new(IntoSystem::into_system(run_runners::<L>)),
        });
    } else {
        schedules.add_systems(label, run_runners::<L>);
    }
}

/// If the current schedule is the same as the schedule on which the [`BoxedRunner`] is running,
/// it will be executed immediately.
fn init_runner<Label: ScheduleLabel>(
    world: &mut World,
    runner: &mut BoxedRunner,
    reactor_entity: Entity,
    running_on_target: bool,
) -> RunnerIs {
    if running_on_target {
        let mut handers = CancellationHandlers::default();
        let runner_is = runner.run(world, &mut handers);
        world
            .non_send_resource_mut::<RunnersRegistry<Label>>()
            .0
            .entry(reactor_entity)
            .or_default()
            .1
            .extend(handers);
        runner_is
    } else {
        RunnerIs::Running
    }
}

#[inline]
fn push_runner_into_registry<Label: ScheduleLabel>(
    world: &mut World,
    reactor_entity: Entity,
    runner: BoxedRunner,
) {
    world
        .non_send_resource_mut::<RunnersRegistry<Label>>()
        .0
        .entry(reactor_entity)
        .or_default()
        .0
        .push(runner);
}

#[inline]
fn observer_already_exists<Label: ScheduleLabel>(
    world: &mut World,
    reactor_entity: &Entity,
) -> bool {
    world
        .query_filtered::<Entity, With<ReactorScheduleLabel<Label>>>()
        .iter(world)
        .any(|target| &target == reactor_entity)
}

fn observe_remove_reactor<Label: ScheduleLabel>(reactor_entity: Entity, world: &mut World) {
    if observer_already_exists::<Label>(world, &reactor_entity) {
        return;
    }
    let mut observer = Observer::new(
        move |_: On<Remove, NativeReactor>, mut commands: Commands| {
            commands.queue(move |world: &mut World| {
                let Some(mut runner_registry) =
                    world.remove_non_send_resource::<RunnersRegistry<Label>>()
                else {
                    return;
                };
                let Some((_, cancellation_handlers)) = runner_registry.0.remove(&reactor_entity)
                else {
                    world.insert_non_send_resource(runner_registry);
                    return;
                };
                for handler in cancellation_handlers.0.values() {
                    handler(world);
                }
                if let Some(mut r) = world.get_non_send_resource_mut::<RunnersRegistry<Label>>() {
                    r.0.extend(runner_registry.0);
                } else {
                    world.insert_non_send_resource(runner_registry);
                }
            });
        },
    );
    observer.watch_entity(reactor_entity);
    world
        .entity_mut(reactor_entity)
        .insert(ReactorScheduleLabel(PhantomData::<Label>));
    world.spawn(observer);
}

fn run_runners<L: Send + Sync + 'static>(world: &mut World) -> Result {
    let Some(mut runners_registry) = world
        .get_non_send_resource_mut::<RunnersRegistry<L>>()
        .map(|mut registry| core::mem::take(&mut registry.0))
    else {
        return Ok(());
    };

    for (entity, (runners, cancellation_handlers)) in runners_registry.iter_mut() {
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
            world.commands().trigger(StepReactor { reactor: *entity });
        }
    }

    world
        .non_send_resource_mut::<RunnersRegistry<L>>()
        .0
        .extend(runners_registry);
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
            $impl_macro!(In1, In2);
            $impl_macro!(In1, In2, In3);
            $impl_macro!(In1, In2, In3, In4);
            $impl_macro!(In1, In2, In3, In4, In5);
            $impl_macro!(In1, In2, In3, In4, In5, In6);
            $impl_macro!(In1, In2, In3, In4, In5, In6, In7);
            $impl_macro!(In1, In2, In3, In4, In5, In6, In7, In8);
            $impl_macro!(In1, In2, In3, In4, In5, In6, In7, In8, In9);
            $impl_macro!(In1, In2, In3, In4, In5, In6, In7, In8, In9, In10);
            $impl_macro!(In1, In2, In3, In4, In5, In6, In7, In8, In9, In10, In11);
            $impl_macro!(In1, In2, In3, In4, In5, In6, In7, In8, In9, In10, In11, In12);
        };
    }

    pub(crate) use impl_tuple_runner;
    pub(crate) use output_combine;
}

#[cfg(test)]
mod tests {
    use crate::action::{delay, once, wait};
    use crate::prelude::{ActionSeed, CancellationHandlers, Reactor, Then};
    use crate::reactor::NativeReactor;
    use crate::runner::{ReactorScheduleLabel, Runner, RunnerIs};
    use crate::test_util::test;
    use crate::tests::test_app;
    use bevy::app::{AppExit, PostUpdate, Startup};
    use bevy::ecs::message::MessageCursor;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{
        Commands, Component, Entity, IntoScheduleConfigs, Query, ResMut, Update, World,
    };
    use bevy::prelude::{Resource, With};
    use bevy_test_helper::event::DirectEvents;
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
        ActionSeed::new(move |_, _| TestCancelRunner { limit })
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
        assert_eq!(
            app.world_mut()
                .query::<&NativeReactor>()
                .iter(app.world())
                .len(),
            1
        );

        app.update();
        app.assert_resource_eq(Count(2));
        assert_eq!(
            app.world_mut()
                .query::<&NativeReactor>()
                .iter(app.world())
                .len(),
            1
        );

        app.update();
        app.assert_resource_eq(Count(3));
        assert_eq!(
            app.world_mut()
                .query::<&NativeReactor>()
                .iter(app.world())
                .len(),
            0
        );
    }

    #[test]
    fn test_cancel_reactor() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    wait::both(
                        test::cancel(),
                        wait::until(|mut count: ResMut<Count>| {
                            count.increment();
                            false
                        }),
                    ),
                )
                .await;
            }));
        });
        app.update();
        for _ in 0..50 {
            app.update();
            assert_eq!(
                app.world_mut()
                    .query::<&NativeReactor>()
                    .iter(app.world())
                    .len(),
                0
            );
            app.assert_resource_eq(Count(0));
        }
    }

    #[test]
    fn cancel_with_multiple_reactors() {
        let mut app = test_app();
        app.add_systems(
            Startup,
            (
                |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        task.will(Update, once::no_op()).await;
                    }));
                },
                |mut commands: Commands| {
                    commands.spawn((
                        Cancellable,
                        Reactor::schedule(|task| async move {
                            task.will(
                                Update,
                                wait::until(|mut count: ResMut<Count>| {
                                    count.increment();
                                    false
                                }),
                            )
                            .await;
                        }),
                    ));
                },
            )
                .chain(),
        );

        app.update();
        app.assert_resource_eq(Count(1));

        let _ = app.world_mut().run_system_once(
            |mut commands: Commands, reactor: Query<Entity, With<Cancellable>>| {
                commands.entity(reactor.single().unwrap()).despawn();
            },
        );
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
                    let count2_task = task
                        .run(
                            PostUpdate,
                            wait::until(|mut count: ResMut<Count2>| {
                                count.0 += 1;
                                false
                            }),
                        )
                        .await;
                    task.will(
                        Update,
                        wait::until(|mut count: ResMut<Count>| {
                            count.increment();
                            false
                        }),
                    )
                    .await;
                    count2_task.await;
                }),
            ));
        });
        app.update();
        app.update();

        let _ = app.world_mut().run_system_once(
            |mut commands: Commands, reactor: Query<Entity, With<Cancellable>>| {
                commands.entity(reactor.single().unwrap()).despawn();
            },
        );
        app.update();
        app.assert_resource_eq(Count2(2));
    }

    #[test]
    fn avoid_spawn_unnecessary_observers() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                // Spawn an observer.
                task.will(Update, once::no_op()).await;
                // Avoid spawning another observer.
                task.will(Update, wait::until(|| false)).await;
            }));
        });

        app.update();
        let num_observer = app
            .world_mut()
            .query_filtered::<&NativeReactor, With<ReactorScheduleLabel<Update>>>()
            .iter(app.world_mut())
            .len();
        assert_eq!(num_observer, 1);
    }

    struct CancelRunner;

    impl Runner for CancelRunner {
        fn run(
            &mut self,
            _: &mut World,
            cancellation_handlers: &mut CancellationHandlers,
        ) -> RunnerIs {
            cancellation_handlers.register(|world| {
                world.set_bool(true);
            });
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

    #[test]
    fn avoid_resource_does_not_exists_app_schedule_labels() {
        let mut app = test_app();
        app.add_systems(Update, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::run(|mut commands: Commands| {
                        commands.spawn(Reactor::schedule(|task| async move {
                            task.will(Update, once::no_op()).await;
                        }));
                    }),
                )
                .await;
            }));
        });
        // should avoid error `resource does not exist: bevy_flurx::runner::app_schedule_labels::AppScheduleLabels`.
        app.update();
    }

    #[test]
    fn avoid_resource_does_not_exists_cancellation_handlers() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::run(|mut commands: Commands| {
                        commands.spawn(Reactor::schedule(|task| async move {
                            task.will(Update, once::no_op()).await;
                        }));
                    }),
                )
                .await;
            }));
        });
        app.update();
    }

    #[test]
    fn runner_correctly_registered() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::run(|mut commands: Commands| {
                        commands.spawn(Reactor::schedule(|task| async move {
                            task.will(
                                Update,
                                delay::frames()
                                    .with(1)
                                    .then(once::event::app_exit_success()),
                            )
                            .await;
                        }));
                    }),
                )
                .await;
            }));
        });
        let mut cursor = MessageCursor::<AppExit>::default();

        app.update();
        app.assert_message_not_comes(&mut cursor);

        app.update();
        app.assert_message_comes(&mut cursor);
    }
}
