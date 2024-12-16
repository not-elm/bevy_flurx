//! Define the actions related to `redo` operations.
//! To perform these the actions, you must call the one of the [`record::undo`](crate::prelude::record::undo) actions beforehand.
//!
//!
//! actions
//! - [`record::redo::once`](crate::prelude::record::redo::once)
//! - [`record::redo::index_to`](crate::prelude::record::redo::index_to)
//! - [`record::redo::to`](crate::prelude::record::redo::to)
//! - [`record::redo::all`](crate::prelude::record::redo::all)

use bevy::prelude::World;

use crate::action::record::{push_tracks, Record};
use crate::action::record::{unlock_record, EditRecordResult};
use crate::prelude::record::lock_record;
use crate::prelude::{ActionSeed, Output, RunnerStatus, Track};
use crate::runner::{BoxedRunner, CancellationId, CancellationToken, Runner};

/// Pops the last pushed `redo` action and execute it.
/// After the `redo` action is executed, then the `undo` action that created it
/// is pushed into [`Record`] again.
///
/// If the `redo stack` in [`Record`] is empty, nothing happens.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
///
/// # Examples
///
/// ```no_run
///
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct Act;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, record::push().with(
///         Track{
///             act: Act,
///             rollback: Rollback::undo(|| once::run(||{})),
///         }))
///         .await
///         .unwrap();
///     task.will(Update, record::undo::once::<Act>()).await.unwrap();
///     task.will(Update, record::redo::once::<Act>()).await.unwrap();
/// });
/// ```
pub fn once<Act>() -> ActionSeed<(), EditRecordResult>
where
    Act: Send + Sync + 'static,
{
    do_redo(|_| {
        move |record: &mut Record<Act>| record.redo.pop().map(|r| vec![r]).unwrap_or_default()
    })
}

/// Pop and execute the `undo` actions up to the specified index.
///
/// If the `redo stack` in [`Record`] is empty, nothing happens.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn index_to<Act>() -> ActionSeed<usize, EditRecordResult>
where
    Act: Send + Sync + 'static,
{
    do_redo(|to: usize| move |record: &mut Record<Act>| record.redo.split_off(to))
}

/// Pop and execute the `redo` actions until the specified operation is reached.
///
/// If the `redo stack` in [`Record`] is empty, nothing happens.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn to<Act>() -> ActionSeed<Act, EditRecordResult>
where
    Act: Send + Sync + PartialEq + 'static,
{
    do_redo(|to: Act| {
        move |record: &mut Record<Act>| {
            let pos = record
                .redo
                .iter()
                .position(|t| t.0.act == to)
                .unwrap_or_default();
            record.redo.split_off(pos)
        }
    })
}

/// Pop and execute all the `redo` actions from [`Record`].
///
/// If the `redo stack` in [`Record`] is empty, nothing happens.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn all<Act>() -> ActionSeed<(), EditRecordResult>
where
    Act: Send + Sync + 'static,
{
    do_redo(|_: ()| |record: &mut Record<Act>| std::mem::take(&mut record.redo))
}

fn do_redo<In, Act, F>(
    predicate: impl FnOnce(In) -> F + Send + Sync + 'static,
) -> ActionSeed<In, EditRecordResult>
where
    In: 'static,
    Act: Send + Sync + 'static,
    F: Fn(&mut Record<Act>) -> Vec<(Track<Act>, ActionSeed)> + 'static,
{
    ActionSeed::new(|input: In, output| RedoExecuteRunner {
        output,
        redo_output: Output::default(),
        redo_runner: None,
        tracks: Vec::new(),
        predicate: predicate(input),
        cancellation_id: None,
    })
}

struct RedoExecuteRunner<P, Act> {
    output: Output<EditRecordResult>,
    redo_output: Output<()>,
    redo_runner: Option<BoxedRunner>,
    tracks: Vec<(Track<Act>, ActionSeed)>,
    predicate: P,
    cancellation_id: Option<CancellationId>,
}

#[repr(transparent)]
struct TracksStore<Act>(Vec<Track<Act>>);

impl<P, Act> Runner for RedoExecuteRunner<P, Act>
where
    P: Fn(&mut Record<Act>) -> Vec<(Track<Act>, ActionSeed)> + 'static,
    Act: Send + Sync + 'static,
{
    //noinspection DuplicatedCode
    fn run(&mut self, world: &mut World, token: &mut CancellationToken) -> crate::prelude::RunnerStatus {
        if self.cancellation_id.is_none() {
            if let Err(e) = lock_record::<Act>(world) {
                self.output.set(Err(e));
                return RunnerStatus::Ready;
            }
            world.insert_non_send_resource(TracksStore::<Act>(Vec::new()));
            self.cancellation_id.replace(token.register(cleanup::<Act>));
            self.tracks =
                (self.predicate)(&mut world.get_resource_or_insert_with(Record::<Act>::default));
        }

        loop {
            if self.redo_runner.is_none() {
                if let Some((track, redo)) = self.tracks.pop() {
                    let runner = redo.with(()).into_runner(self.redo_output.clone());
                    self.redo_runner.replace(runner);
                    world
                        .non_send_resource_mut::<TracksStore<Act>>()
                        .0
                        .push(track);
                } else {
                    self.output.set(Ok(()));
                    if let Some(id) = self.cancellation_id.as_ref() {
                        token.unregister(id);
                    }
                    cleanup::<Act>(world);
                    return RunnerStatus::Ready;
                }
            }

            match self.redo_runner.as_mut().unwrap().run(world, token) {
                RunnerStatus::Ready => {
                    self.redo_runner.take();
                    self.redo_output.take();
                }
                other => return other
            }
        }
    }
}

fn cleanup<Act: Send + Sync + 'static>(world: &mut World) {
    if let Some(store) = world.remove_non_send_resource::<TracksStore<Act>>() {
        let _ = push_tracks(store.0.into_iter(), world, false);
    }

    unlock_record::<Act>(world);
}

//noinspection DuplicatedCode
#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::hierarchy::DespawnRecursiveExt;
    use bevy::prelude::{
        on_event, Commands, Component, Entity, IntoSystemConfigs, Query, ResMut, Resource, With,
    };
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::record::track::{Redo, Undo};
    use crate::action::{delay, once, record};
    use crate::prelude::{ActionSeed, Omit, Record, RequestRedo, Rollback, Then, Track};
    use crate::reactor::Reactor;
    use crate::sequence;
    use crate::test_util::SpawnReactor;
    use crate::tests::{exit_reader, increment_count, test_app, TestAct};

    #[test]
    fn test_redo_once() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(
                    Update,
                    record::push().with(Track {
                        act: TestAct,
                        rollback: Rollback::parts(
                            Undo::make(|| {
                                once::run(|mut count: ResMut<Count>| {
                                    count.increment();
                                })
                            }),
                            Redo::make(|_| {
                                once::run(|mut count: ResMut<Count>| {
                                    count.decrement();
                                })
                            }),
                        ),
                    }),
                )
                    .await
                    .unwrap();
                task.will(Update, record::undo::once::<TestAct>())
                    .await
                    .unwrap();
                task.will(Update, record::redo::once::<TestAct>())
                    .await
                    .unwrap();
            }));
        });
        let mut er = exit_reader();
        app.update();
        app.assert_event_not_comes(&mut er);

        app.update();
        app.assert_resource_eq(Count(1));

        app.update();
        app.assert_resource_eq(Count(0));
    }

    #[test]
    fn test_redo_index_to() {
        #[derive(Resource, Debug, Eq, PartialEq)]
        struct Mark(Vec<&'static str>);

        let mut app = test_app();
        app.insert_resource(Mark(vec![]));
        app.add_systems(Startup, |mut commands: Commands| {
            fn push(word: &'static str) -> ActionSeed {
                record::push()
                    .with(Track {
                        act: TestAct,
                        rollback: Rollback::parts(
                            Undo::make(|| once::run(|| {})),
                            Redo::make(move |_| {
                                once::run(move |mut mark: ResMut<Mark>| {
                                    mark.0.push(word);
                                })
                            }),
                        ),
                    })
                    .omit()
            }

            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(Update, push("1").then(push("2")).then(push("3")))
                    .await;
                task.will(Update, record::undo::all::<TestAct>())
                    .await
                    .unwrap();
                task.will(Update, record::redo::index_to::<TestAct>().with(1))
                    .await
                    .unwrap();
            }));
        });

        app.update();
        app.assert_resource_eq(Mark(vec![]));
        app.update();
        app.assert_resource_eq(Mark(vec![]));
        app.update();
        app.assert_resource_eq(Mark(vec!["1", "2"]));
    }

    #[test]
    fn redo_all_if_specified_non_exists_act() {
        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        struct Act(usize);

        #[derive(Resource, Debug, Eq, PartialEq)]
        struct Mark(Vec<usize>);

        let mut app = test_app();
        app.insert_resource(Mark(vec![]));
        app.add_systems(Startup, |mut commands: Commands| {
            fn push(num: usize) -> ActionSeed {
                record::push()
                    .with(Track {
                        act: Act(num),
                        rollback: Rollback::parts(
                            Undo::make(|| once::run(|| {})),
                            Redo::make(move |_| {
                                once::run(move |mut mark: ResMut<Mark>| {
                                    mark.0.push(num);
                                })
                            }),
                        ),
                    })
                    .omit()
            }

            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(Update, push(1).then(push(2)).then(push(3)).then(push(4)))
                    .await;
                task.will(Update, record::undo::all::<Act>()).await.unwrap();
                task.will(Update, record::redo::to::<Act>().with(Act(5)))
                    .await
                    .unwrap();
            }));
        });

        app.update();
        app.assert_resource_eq(Mark(vec![]));
        app.update();
        app.assert_resource_eq(Mark(vec![]));
        app.update();
        app.assert_resource_eq(Mark(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_redo_to() {
        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        struct Act(usize);

        #[derive(Resource, Debug, Eq, PartialEq)]
        struct Mark(Vec<usize>);

        let mut app = test_app();
        app.insert_resource(Mark(vec![]));
        app.add_systems(Startup, |mut commands: Commands| {
            fn push(num: usize) -> ActionSeed {
                record::push()
                    .with(Track {
                        act: Act(num),
                        rollback: Rollback::parts(
                            Undo::make(|| once::run(|| {})),
                            Redo::make(move |_| {
                                once::run(move |mut mark: ResMut<Mark>| {
                                    mark.0.push(num);
                                })
                            }),
                        ),
                    })
                    .omit()
            }

            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(Update, push(1).then(push(2)).then(push(3)).then(push(4)))
                    .await;
                task.will(Update, record::undo::all::<Act>()).await.unwrap();
                task.will(Update, record::redo::to::<Act>().with(Act(2)))
                    .await
                    .unwrap();
            }));
        });

        app.update();
        app.assert_resource_eq(Mark(vec![]));
        app.update();
        app.assert_resource_eq(Mark(vec![]));
        app.update();
        app.assert_resource_eq(Mark(vec![1, 2]));
    }

    #[test]
    fn redo_all() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            fn push() -> ActionSeed {
                record::push()
                    .with(Track {
                        act: TestAct,
                        rollback: Rollback::parts(
                            Undo::make(|| once::run(|| {})),
                            Redo::make(|_| increment_count()),
                        ),
                    })
                    .omit()
            }

            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(Update, push().then(push()).then(push())).await;
                task.will(Update, record::undo::all::<TestAct>())
                    .await
                    .unwrap();
                task.will(Update, record::redo::all::<TestAct>())
                    .await
                    .unwrap();
            }));
        });

        app.update();
        app.assert_resource_eq(Count(0));
        app.update();
        app.assert_resource_eq(Count(0));
        app.update();
        app.assert_resource_eq(Count(3));
    }

    #[test]
    fn failed_in_progressing() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(
                    Update,
                    sequence![record::push().with(Track {
                        act: TestAct,
                        rollback: Rollback::parts(
                            Undo::make(|| delay::frames().with(100)),
                            Redo::make(|_| once::run(|mut count: ResMut<Count>| {
                                count.decrement();
                            }))
                        )
                    })],
                )
                    .await
                    .unwrap();

                let t1 = task.run(Update, record::undo::once::<TestAct>()).await;
                if task
                    .will(Update, record::redo::once::<TestAct>())
                    .await
                    .is_err()
                {
                    task.will(Update, once::event::app_exit_success()).await;
                }
                t1.await.unwrap();
            }));
        });
        let mut er = exit_reader();
        app.update();
        app.update();
        app.update();
        app.update();
        app.assert_event_comes(&mut er);
    }

    #[test]
    fn unlock_after_cancelled() {
        let mut app = test_app();
        app.spawn_reactor(|task| async move {
            task.will(Update, {
                record::push()
                    .with(Track {
                        act: TestAct,
                        rollback: Rollback::parts(
                            Undo::make(|| once::run(|| {})),
                            Redo::make(|_| delay::frames().with(1000)),
                        ),
                    })
                    .then(record::undo::once::<TestAct>())
                    .then(record::redo::once::<TestAct>())
            })
                .await
                .unwrap();
        });
        app.update();
        app.assert_resource(false, |record: &Record<TestAct>| record.can_edit());

        app.world_mut()
            .run_system_once(
                |mut commands: Commands, reactor: Query<Entity, With<Reactor>>| {
                    commands.entity(reactor.single()).despawn();
                },
            )
            .expect("Failed to run system");
        app.update();
        app.assert_resource(true, |record: &Record<TestAct>| record.can_edit());
    }

    #[test]
    fn never_call_cleanup_after_redo_finished() {
        let mut app = test_app();
        #[derive(Component)]
        struct R;
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                R,
                crate::prelude::Flow::schedule(|task| async move {
                    task.will(Update, {
                        record::push()
                            .with(Track {
                                act: TestAct,
                                rollback: Rollback::parts(
                                    Undo::make(|| once::run(|| {})),
                                    Redo::make(|_| once::run(|| {})),
                                ),
                            })
                            .then(record::undo::once::<TestAct>())
                            .then(record::redo::once::<TestAct>())
                            .then(record::push().with(Track {
                                act: TestAct,
                                rollback: Rollback::parts(
                                    Undo::make(|| once::run(|| {})),
                                    Redo::make(|_| delay::frames().with(1000)),
                                ),
                            }))
                            .then(record::undo::once::<TestAct>())
                            .then(delay::frames().with(1000))
                    })
                        .await;
                }),
            ));
        });
        app.add_systems(
            Update,
            (|mut commands: Commands, reactor: Query<Entity, With<R>>| {
                commands.entity(reactor.single()).despawn_recursive();
            })
                .run_if(on_event::<AppExit>),
        );
        app.update();
        app.send(RequestRedo::<TestAct>::Once);
        app.update();
        app.send_default::<AppExit>();
        app.update();
        app.update();
        app.assert_resource(false, |record: &Record<TestAct>| record.can_edit());
    }
}
