//! Define the actions related to `redo` operations.
//! To perform these the actions, you must call the one of the [`record::undo`](crate::prelude::undo) actions beforehand.
//!
//!
//! actions
//! - [`record::redo::once`](crate::prelude::record::redo::once)
//! - [`record::redo::index_to`](crate::prelude::record::redo::index_to)
//! - [`record::redo::to`](crate::prelude::record::redo::to)
//! - [`record::redo::all`](crate::prelude::record::redo::all)


use bevy::prelude::World;

use crate::action::record::{push_tracks, Record};
use crate::action::record::{EditRecordResult, unlock_record};
use crate::prelude::{ActionSeed, Output, Track};
use crate::prelude::record::lock_record;
use crate::runner::{BoxedRunner, CancellationToken, Runner};

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
/// Reactor::schedule(|task| async move{
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
        Act: Send + Sync + 'static
{
    do_redo(|_| move |record: &mut Record<Act>| {
        record.redo.pop().map(|r| vec![r]).unwrap_or_default()
    })
}

/// Pop and execute the `undo` actions up to the specified index.
///
/// If the `redo stack` in [`Record`] is empty, nothing happens.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn index_to<Act>() -> ActionSeed<usize, EditRecordResult>
    where
        Act: Send + Sync + 'static
{
    do_redo(|to: usize| move |record: &mut Record<Act>| {
        record.redo.split_off(to)
    })
}

/// Pop and execute the `redo` actions until the specified operation is reached.
///
/// If the `redo stack` in [`Record`] is empty, nothing happens.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn to<Act>() -> ActionSeed<Act, EditRecordResult>
    where
        Act: Send + Sync + PartialEq + 'static
{
    do_redo(|to: Act| move |record: &mut Record<Act>| {
        let pos = record.redo.iter().position(|t| t.0.act == to).unwrap_or_default();
        record.redo.split_off(pos)
    })
}

/// Pop and execute all the `redo` actions from [`Record`].
///
/// If the `redo stack` in [`Record`] is empty, nothing happens.
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
pub fn all<Act>() -> ActionSeed<(), EditRecordResult>
    where
        Act: Send + Sync + 'static
{
    do_redo(|_: ()| |record: &mut Record<Act>| {
        std::mem::take(&mut record.redo)
    })
}

fn do_redo<In, Act, F>(
    predicate: impl FnOnce(In) -> F + 'static,
) -> ActionSeed<In, EditRecordResult>
    where
        In: 'static,
        Act: Send + Sync + 'static,
        F: Fn(&mut Record<Act>) -> Vec<(Track<Act>, ActionSeed)> + 'static
{
    ActionSeed::new(|input: In, token, output| {
        RedoExecuteRunner {
            token,
            output,
            redo_output: Output::default(),
            redo_runner: None,
            tracks: Vec::new(),
            cache: Vec::new(),
            predicate: predicate(input),
            initialized: false
        }
    })
}

struct RedoExecuteRunner<P, Act> {
    token: CancellationToken,
    output: Output<EditRecordResult>,
    redo_output: Output<()>,
    redo_runner: Option<BoxedRunner>,
    tracks: Vec<(Track<Act>, ActionSeed)>,
    cache: Vec<Track<Act>>,
    predicate: P,
    initialized: bool,
}

impl<P, Act> Runner for RedoExecuteRunner<P, Act>
    where
        P: Fn(&mut Record<Act>) -> Vec<(Track<Act>, ActionSeed)> + 'static,
        Act: Send + Sync + 'static
{
    //noinspection DuplicatedCode
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            let _ = push_tracks(std::mem::take(&mut self.cache).into_iter(), world, false);
            unlock_record::<Act>(world);
            return true;
        }

        if !self.initialized {
            if let Err(e) = lock_record::<Act>(world) {
                self.output.replace(Err(e));
                return true;
            }
            self.tracks = (self.predicate)(&mut world.get_resource_or_insert_with(Record::<Act>::default));
            self.initialized = true;
        }

        loop {
            if self.redo_runner.is_none() {
                if let Some((track, redo)) = self.tracks.pop() {
                    let runner = redo.with(()).into_runner(self.token.clone(), self.redo_output.clone());
                    self.redo_runner.replace(runner);
                    self.cache.push(track);
                } else {
                    unlock_record::<Act>(world);
                    self.output.replace(Ok(()));
                    let _ = push_tracks(std::mem::take(&mut self.cache).into_iter(), world, false);
                    return true;
                }
            }

            if self.redo_runner
                .as_mut()
                .unwrap()
                .run(world) {
                self.redo_runner.take();
                self.redo_output.take();
            } else {
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::{Commands, ResMut};
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{delay, once, record};
    use crate::action::record::track::{Redo, Undo};
    use crate::prelude::{Rollback, Track};
    use crate::reactor::Reactor;
    use crate::sequence;
    use crate::tests::{exit_reader, test_app, TestAct};

    #[test]
    fn redo_decrement() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, record::push().with(Track {
                    act: TestAct,
                    rollback: Rollback::parts(
                        Undo::make(|| once::run(|mut count: ResMut<Count>| {
                            count.increment();
                        })),
                        Redo::make(|_| once::run(|mut count: ResMut<Count>| {
                            count.decrement();
                        })),
                    ),
                }))
                    .await
                    .unwrap();
                task.will(Update, record::undo::once::<TestAct>()).await.unwrap();
                task.will(Update, record::redo::once::<TestAct>()).await.unwrap();
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
    fn failed_in_progressing() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, sequence![
                    record::push().with(Track{
                        act: TestAct,
                        rollback: Rollback::parts(
                            Undo::make(|| delay::frames().with(100)),
                            Redo::make(|_|once::run(|mut count: ResMut<Count>| {
                                count.decrement();
                            }))
                        )
                    })
                ])
                    .await
                    .unwrap();

                let t1 = task.run(Update, record::undo::once::<TestAct>()).await;
                if task.will(Update, record::redo::once::<TestAct>()).await.is_err() {
                    task.will(Update, once::event::app_exit()).await;
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
}