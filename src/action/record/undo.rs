//! Define the actions related to `undo` operations.
//!
//! To perform these the actions, you must call the [`record::push`](crate::prelude::record::push) beforehand.

use crate::action::record::EditRecordResult;
use crate::action::record::Record;
use crate::prelude::record::{lock_record, unlock_record};
use crate::prelude::{ActionSeed, Output, Runner, Track};
use crate::runner::{BoxedRunner, CancellationHandlers, CancellationId, RunnerIs};
use bevy::prelude::World;

/// Pops the last pushed `undo` action, and then execute it.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct Act;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, record::push().with(Track{
///         act: Act,
///         rollback: Rollback::undo(|| once::run(||{}))
///     }))
///         .await
///         .expect("An error will be returned if undo or redo is operating.");
///     task.will(Update, record::undo::once::<Act>())
///         .await
///         .expect("An error will be returned if undo or redo is operating.");
/// });
/// ```
pub fn once<Act>() -> ActionSeed<(), EditRecordResult>
where
    Act: Send + Sync + 'static,
{
    do_undo(|_: ()| {
        move |record: &mut Record<Act>| record.tracks.pop().map(|t| vec![t]).unwrap_or_default()
    })
}

/// Pops `undo` up to the specified index.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn index_to<Act>() -> ActionSeed<usize, EditRecordResult>
where
    Act: Send + Sync + 'static,
{
    do_undo(|index: usize| move |record: &mut Record<Act>| record.tracks.split_off(index))
}

/// Pops `undo` until the specified operation is reached.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn to<Act>() -> ActionSeed<Act, EditRecordResult>
where
    Act: Send + Sync + PartialEq + 'static,
{
    do_undo(|to: Act| {
        move |record: &mut Record<Act>| {
            let pos = record
                .tracks
                .iter()
                .position(|t| t.act == to)
                .unwrap_or_default();
            record.tracks.split_off(pos)
        }
    })
}

/// Pops all the `undo` actions from [`Record`].
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn all<Act>() -> ActionSeed<(), EditRecordResult>
where
    Act: Send + Sync + 'static,
{
    do_undo(|_: ()| |record: &mut Record<Act>| std::mem::take(&mut record.tracks))
}

fn do_undo<I, Act, F>(
    predicate: impl FnOnce(I) -> F + Send + Sync + 'static,
) -> ActionSeed<I, EditRecordResult>
where
    I: 'static,
    Act: Send + Sync + 'static,
    F: Fn(&mut Record<Act>) -> Vec<Track<Act>> + Send + Sync + 'static,
{
    ActionSeed::new(|input: I, output| UndoRunner {
        output,
        undo_output: Output::default(),
        undo_runner: None,
        track: None,
        tracks: Vec::new(),
        predicate: predicate(input),
        cancellation_id: None,
    })
}

struct UndoRunner<P, Act> {
    output: Output<EditRecordResult>,
    undo_output: Output<Option<ActionSeed>>,
    undo_runner: Option<BoxedRunner>,
    track: Option<Track<Act>>,
    predicate: P,
    tracks: Vec<Track<Act>>,
    cancellation_id: Option<CancellationId>,
}

struct RedoStore<Act>(Vec<(Track<Act>, ActionSeed)>);

impl<P, Act> Runner for UndoRunner<P, Act>
where
    Act: Send + Sync + 'static,
    P: Fn(&mut Record<Act>) -> Vec<Track<Act>> + Send + Sync + 'static,
{
    //noinspection DuplicatedCode
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        if self.cancellation_id.is_none() {
            if let Err(progressing) = lock_record::<Act>(world) {
                self.output.set(Err(progressing));
                return RunnerIs::Completed;
            }
            world.insert_non_send_resource(RedoStore::<Act>(Vec::new()));
            self.cancellation_id.replace(token.register(cleanup::<Act>));
            self.tracks =
                (self.predicate)(&mut world.get_resource_or_insert_with(Record::<Act>::default));
        }

        loop {
            if self.undo_runner.is_none() {
                if let Some(track) = self.tracks.pop() {
                    let runner = track.create_runner(self.undo_output.clone());
                    self.undo_runner.replace(runner);
                    self.track.replace(track);
                }
            }

            let Some(undo_runner) = self.undo_runner.as_mut() else {
                self.output.set(Ok(()));
                cleanup::<Act>(world);
                if let Some(id) = self.cancellation_id.as_ref() {
                    token.unregister(id);
                }
                return RunnerIs::Completed;
            };

            undo_runner.run(world, token);
            let Some(redo) = self.undo_output.take() else {
                return RunnerIs::Running;
            };
            if let Some(redo) = redo {
                let undo = self.track.take().unwrap();
                world
                    .non_send_resource_mut::<RedoStore<Act>>()
                    .0
                    .push((undo, redo));
            }
            self.undo_output.take();
            self.undo_runner.take();
        }
    }
}

fn cleanup<Act: Send + Sync + 'static>(world: &mut World) {
    if let Some(store) = world.remove_non_send_resource::<RedoStore<Act>>() {
        world.resource_mut::<Record<Act>>().redo.extend(store.0);
    }

    unlock_record::<Act>(world);
}

#[cfg(test)]
mod tests {
    use crate::action::record::tests::push_undo_increment;
    use crate::action::record::EditRecordResult;
    use crate::action::{delay, record};
    use crate::prelude::{
        once, ActionSeed, Omit, Pipe, Reactor, Record, RequestUndo, Rollback, Then, Track,
    };
    use crate::reactor::NativeReactor;
    use crate::test_util::SpawnReactor;
    use crate::tests::{exit_reader, increment_count, test_app, TestAct};
    use bevy::app::{AppExit, Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{
        on_event, Commands, Component, Entity, EventWriter, In, IntoScheduleConfigs, Query, With,
    };
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn pop_all() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment()
                        .then(push_undo_increment())
                        .then(push_undo_increment())
                        .then(record::undo::all::<TestAct>()),
                )
                .await
                .unwrap();
            }));
        });
        app.update();
        app.assert_resource_eq(Count(3));
    }

    #[test]
    fn pop_all_with_delay() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment()
                        .then(push_undo_increment())
                        .then(record::push().with(Track {
                            act: TestAct,
                            rollback: Rollback::undo(|| delay::frames().with(1)),
                        }))
                        .then(push_undo_increment())
                        .then(record::undo::all::<TestAct>()),
                )
                .await
                .unwrap();
            }));
        });
        app.update();
        app.assert_resource_eq(Count(1));
        app.update();
        app.assert_resource_eq(Count(3));
    }

    #[test]
    fn nothing_happens() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, record::undo::all::<TestAct>())
                    .await
                    .unwrap();
                task.will(
                    Update,
                    push_undo_increment().then(record::undo::index_to::<TestAct>().with(0)),
                )
                .await
                .unwrap();
            }));
        });
        app.update();

        app.update();
        app.assert_resource_eq(Count(1));
    }

    #[test]
    fn undo_index_to_1() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment()
                        .then(push_undo_increment())
                        .then(push_undo_increment())
                        .then(record::undo::index_to::<TestAct>().with(1)),
                )
                .await
                .unwrap();
            }));
        });
        app.update();

        app.assert_resource_eq(Count(2));
    }

    #[test]
    fn undo_to_op_move() {
        #[derive(PartialEq, Default)]
        enum Act {
            #[default]
            Move,
            Stop,
        }

        fn push(act: Act) -> ActionSeed {
            record::push()
                .with(Track {
                    act,
                    rollback: Rollback::undo(increment_count),
                })
                .omit()
        }

        let mut app = test_app();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push(Act::Move)
                        .then(push(Act::Stop))
                        .then(push(Act::Stop))
                        .then(record::undo::to().with(Act::Move)),
                )
                .await
                .unwrap();
            }));
        });
        app.update();
        app.assert_resource(true, |record: &Record<Act>| record.tracks.is_empty());
    }

    //noinspection ALL
    #[test]
    fn failed_if_running_other() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    push_undo_increment().then(record::push().with(Track {
                        act: TestAct,
                        rollback: Rollback::undo(|| delay::frames().with(1)),
                    })),
                )
                .await
                .unwrap();

                let t1 = task.run(Update, record::undo::once::<TestAct>()).await;
                task.will(
                    Update,
                    record::undo::all::<TestAct>().pipe(once::run(
                        |In(result): In<EditRecordResult>, mut ew: EventWriter<AppExit>| {
                            if result.is_err() {
                                ew.write_default();
                            }
                        },
                    )),
                )
                .await;
                task.will(
                    Update,
                    record::undo::all::<TestAct>().pipe(once::run(
                        |In(result): In<EditRecordResult>, mut ew: EventWriter<AppExit>| {
                            if result.is_err() {
                                ew.write_default();
                            }
                        },
                    )),
                )
                .await;
                t1.await.unwrap();
            }));
        });
        app.update();
        app.update();
        let mut er = exit_reader();
        app.assert_event_comes(&mut er);

        app.update();
        app.assert_event_not_comes(&mut er);
    }

    #[test]
    fn unlock_after_cancel() {
        let mut app = test_app();
        app.spawn_reactor(|task| async move {
            task.will(Update, {
                record::push()
                    .with(Track {
                        act: TestAct,
                        rollback: Rollback::undo(|| delay::frames().with(100)),
                    })
                    .then(record::undo::once::<TestAct>())
            })
            .await
            .unwrap();
        });
        app.update();
        app.update();
        app.world_mut()
            .run_system_once(
                |mut commands: Commands, reactor: Query<Entity, With<NativeReactor>>| {
                    commands.entity(reactor.single().unwrap()).despawn();
                },
            )
            .expect("Failed to run system");
        app.update();
        app.assert_resource(true, |record: &Record<TestAct>| record.can_edit());
    }

    #[test]
    fn never_call_cleanup_after_undo_finished() {
        let mut app = test_app();
        #[derive(Component)]
        struct R;
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                R,
                Reactor::schedule(|task| async move {
                    task.will(Update, {
                        record::push()
                            .with(Track {
                                act: TestAct,
                                rollback: Rollback::undo(|| delay::frames().with(1000)),
                            })
                            .then(record::push().with(Track {
                                act: TestAct,
                                rollback: Rollback::undo(|| once::run(|| {})),
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
                commands.entity(reactor.single().unwrap()).despawn();
            })
            .run_if(on_event::<AppExit>),
        );
        app.update();
        app.update();
        app.send(RequestUndo::<TestAct>::Once);
        app.update();
        app.send_default::<AppExit>();
        app.update();
        app.update();
        app.assert_resource(false, |record: &Record<TestAct>| record.can_edit());
    }
}
