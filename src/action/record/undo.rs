//! Define the actions related to `undo` operations.
//!
//! To perform these the actions, you must call the [`record::push`](crate::prelude::record::push) beforehand.
//!
//!
//! actions
//! - [`record::undo::once`](crate::prelude::record::undo::once)
//! - [`record::undo::index_to`](crate::prelude::record::undo::index_to)
//! - [`record::undo::to`](crate::prelude::record::undo::to)
//! - [`record::undo::all`](crate::prelude::record::undo::all)


use bevy::prelude::World;

use crate::action::record::EditRecordResult;
use crate::action::record::Record;
use crate::prelude::{ActionSeed, Output, Runner, Track};
use crate::prelude::record::{lock_record, unlock_record};
use crate::runner::{BoxedRunner, CancellationToken};

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
///     })).await.unwrap();
///     task.will(Update, record::undo::once::<Act>()).await.unwrap();
/// });
/// ```
pub fn once<Act>() -> ActionSeed<(), EditRecordResult>
    where
        Act: Send + Sync + 'static
{
    do_undo(|_: ()| move |record: &mut Record<Act>| {
        record.tracks.pop().map(|t| vec![t]).unwrap_or_default()
    })
}

/// Pops `undo` up to the specified index.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn index_to<Act>() -> ActionSeed<usize, EditRecordResult>
    where
        Act: Send + Sync + 'static
{
    do_undo(|index: usize| move |record: &mut Record<Act>| {
        record.tracks.split_off(index)
    })
}

/// Pops `undo` until the specified operation is reached.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn to<Act>() -> ActionSeed<Act, EditRecordResult>
    where
        Act: Send + Sync + PartialEq + 'static
{
    do_undo(|to: Act| move |record: &mut Record<Act>| {
        let pos = record.tracks.iter().position(|t| t.act == to).unwrap_or_default();
        record.tracks.split_off(pos)
    })
}

/// Pops all the `undo` actions from [`Record`].
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn all<Act>() -> ActionSeed<(), EditRecordResult>
    where
        Act: Send + Sync + 'static
{
    do_undo(|_: ()| |record: &mut Record<Act>| {
        std::mem::take(&mut record.tracks)
    })
}

fn do_undo<I, Act, F>(predicate: impl FnOnce(I) -> F + 'static) -> ActionSeed<I, EditRecordResult>
    where
        I: 'static,
        Act: Send + Sync + 'static,
        F: Fn(&mut Record<Act>) -> Vec<Track<Act>> + 'static
{
    ActionSeed::new(|input: I, token, output| {
        UndoExecuteRunner {
            token,
            output,
            undo_output: Output::default(),
            undo_runner: None,
            redo: Vec::new(),
            track: None,
            tracks: Vec::new(),
            predicate: predicate(input),
        }
    })
}

struct UndoExecuteRunner<P, Act> {
    token: CancellationToken,
    output: Output<EditRecordResult>,
    undo_output: Output<Option<ActionSeed>>,
    undo_runner: Option<BoxedRunner>,
    track: Option<Track<Act>>,
    redo: Vec<(Track<Act>, ActionSeed)>,
    predicate: P,
    tracks: Vec<Track<Act>>,
}

impl<P, Act> Runner for UndoExecuteRunner<P, Act>
    where
        Act: Send + Sync + 'static,
        P: Fn(&mut Record<Act>) -> Vec<Track<Act>> + 'static
{
    //noinspection DuplicatedCode
    fn initialize(&mut self, world: &mut World) -> bool {
        if let Err(progressing) = lock_record::<Act>(world) {
            self.output.replace(Err(progressing));
            return true;
        }
        self.tracks = (self.predicate)(&mut world.get_resource_or_insert_with(Record::<Act>::default));
        false
    }

    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            world.resource_mut::<Record<Act>>().redo.extend(std::mem::take(&mut self.redo));
            unlock_record::<Act>(world);
            return true;
        }

        loop {
            if self.undo_runner.is_none() {
                if let Some(track) = self.tracks.pop() {
                    let runner = track.create_runner(self.token.clone(), self.undo_output.clone());
                    self.undo_runner.replace(runner);
                    self.track.replace(track);
                } 
            }
            let Some(undo_runner) = self.undo_runner.as_mut() else {
                self.output.replace(Ok(()));
                world.resource_mut::<Record<Act>>().redo.extend(std::mem::take(&mut self.redo));
                unlock_record::<Act>(world);
                return true;
            };

            undo_runner.run(world);
            let Some(redo) = self.undo_output.take() else {
                return false;
            };
            if let Some(redo) = redo {
                let undo = self.track.take().unwrap();
                self.redo.push((undo, redo));
            }
            self.undo_output.take();
            self.undo_runner.take();
        }
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup, Update};
    use bevy::prelude::{Commands, EventWriter, In};
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{delay, Omit, record};
    use crate::action::record::EditRecordResult;
    use crate::action::record::tests::push_undo_increment;
    use crate::prelude::{ActionSeed, once, Pipe, Reactor, Record, Rollback, Then, Track};
    use crate::tests::{exit_reader, increment_count, test_app, TestAct};

    #[test]
    fn pop_all() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push_undo_increment()
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
                task.will(Update, push_undo_increment()
                    .then(push_undo_increment())
                    .then(record::push().with(Track {
                        act: TestAct,
                        rollback: Rollback::undo(|| {
                            delay::frames().with(1)
                        }),
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
                task.will(Update, record::undo::all::<TestAct>()).await.unwrap();
                task.will(Update, push_undo_increment()
                    .then(record::undo::index_to::<TestAct>().with(0)),
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
                task.will(Update, push_undo_increment()
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
            record::push().with(Track {
                act,
                rollback: Rollback::undo(increment_count),
            })
                .omit()
        }

        let mut app = test_app();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push(Act::Move)
                    .then(push(Act::Stop))
                    .then(push(Act::Stop))
                    .then(record::undo::to().with(Act::Move)),
                )
                    .await
                    .unwrap();
            }));
        });
        app.update();
        app.assert_resource(true, |record: &Record<Act>| {
            record.tracks.is_empty()
        });
    }

    //noinspection ALL
    #[test]
    fn failed_if_running_other() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push_undo_increment()
                    .then(record::push().with(Track {
                        act: TestAct,
                        rollback: Rollback::undo(|| delay::frames().with(1)),
                    })),
                )
                    .await
                    .unwrap();

                let t1 = task.run(Update, record::undo::once::<TestAct>()).await;
                task.will(Update, record::undo::all::<TestAct>()
                    .pipe(once::run(|In(result): In<EditRecordResult>, mut ew: EventWriter<AppExit>| {
                        if result.is_err() {
                            ew.send_default();
                        }
                    })),
                ).await;
                task.will(Update, record::undo::all::<TestAct>()
                    .pipe(once::run(|In(result): In<EditRecordResult>, mut ew: EventWriter<AppExit>| {
                        if result.is_err() {
                            ew.send_default();
                        }
                    })),
                ).await;
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
}