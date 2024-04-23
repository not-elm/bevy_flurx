//! Manages the history of operations, and it allows you to use `undo` or `redo` of operations.
//!
//! structs
//! - [`Record`]
//! - [`Track`]
//! - [`Rollback`]
//! - [`Undo`]
//! - [`Redo`]
//! - [`RedoAction`]
//!
//! actions
//! - [`record::push`](crate::prelude::record::push)
//! - [`record::undo`](crate::prelude::record::undo)
//! - [`record::redo`](crate::prelude::record::redo)


use std::error::Error;
use std::fmt::{Display, Formatter};

use bevy::prelude::{NonSendMut, Resource, World};

pub use _push::push;
pub use track::*;

use crate::action::once;
use crate::prelude::ActionSeed;

pub mod undo;
pub mod redo;
pub mod extension;
mod track;
#[path = "record/push.rs"]
mod _push;


/// Clear the [`Record`].
///
/// The output will be [`UndoRedoInProgress`] if an `undo` or `redo` is in progress.
pub fn all_clear<M: 'static>() -> ActionSeed<(), Result<(), UndoRedoInProgress>> {
    once::run(|mut store: NonSendMut<Record<M>>| {
        store.all_clear()
    })
}

/// Thrown when attempting to edit history while an `undo` or `redo` action is in progress.
#[derive(Default, Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct UndoRedoInProgress;

impl Display for UndoRedoInProgress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("failed edit history because the undo/redo actions are in progress")
    }
}

impl Error for UndoRedoInProgress {}

/// Result type related to record edit operations.
pub type EditRecordResult = Result<(), UndoRedoInProgress>;

/// Manage the history of `undo` and `redo`.
///
/// This struct has one marker type.
/// This allows you can define different the histories for each type of `Act`.
pub struct Record<Act> {
    pub(crate) tracks: Vec<Track<Act>>,
    pub(crate) redo: Vec<(Track<Act>, ActionSeed)>,
    pub(crate) progressing: bool,
}

impl<Act> Record<Act>
    where
        Act: 'static
{
    /// Clear all history of `undo` and `redo`.
    pub fn all_clear(&mut self) -> Result<(), UndoRedoInProgress> {
        self.err_if_progress()?;
        self.tracks.clear();
        self.redo.clear();
        Ok(())
    }

    /// Returns true if it can be edited record.
    ///
    /// Returns false if any `undo` or `redo` actions is in progress.
    #[inline]
    pub const fn can_edit(&self) -> bool {
        !self.progressing
    }

    /// Push the `track`.
    pub fn push(&mut self, track: Track<Act>) -> Result<(), UndoRedoInProgress> {
        self.err_if_progress()?;
        self.redo.clear();
        self.tracks.push(track);
        Ok(())
    }

    /// Returns the operations.
    #[inline]
    pub fn acts(&self) -> impl ExactSizeIterator<Item=&Act> + DoubleEndedIterator {
        self.tracks.iter().map(|track| &track.act)
    }

    /// Returns the redo operations.
    #[inline]
    pub fn redo_acts(&self) -> impl ExactSizeIterator<Item=&Act> + DoubleEndedIterator {
        self.redo.iter().map(|(track, _)| &track.act)
    }

    const fn err_if_progress(&self) -> Result<(), UndoRedoInProgress> {
        if self.progressing {
            Err(UndoRedoInProgress)
        } else {
            Ok(())
        }
    }
}

impl<M> Default for Record<M> {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            redo: Vec::new(),
            progressing: false,
        }
    }
}

impl<Op> Resource for Record<Op>
    where
        Op: Send + Sync + 'static
{}

/// # Safety: `Track::create_runner` must be called only on the main thread.
unsafe impl<Op> Send for Record<Op>
    where
        Op: Send + Sync + 'static
{}

/// # Safety: `Track::create_runner` must be called only on the main thread.
unsafe impl<Op> Sync for Record<Op>
    where
        Op: Send + Sync + 'static
{}

pub(crate) fn lock_record<Opr: Send + Sync + 'static>(world: &mut World) -> EditRecordResult {
    let mut record = world.get_resource_or_insert_with::<Record<Opr>>(Record::<Opr>::default);
    if record.progressing {
        Err(UndoRedoInProgress)
    } else {
        record.progressing = true;
        Ok(())
    }
}

#[inline]
pub(crate) fn unlock_record<Opr: Send + Sync + 'static>(world: &mut World) {
    let mut record = world.get_resource_or_insert_with::<Record<Opr>>(Record::<Opr>::default);
    record.progressing = false;
}

fn push_tracks<Act: Send + Sync + 'static>(track: impl Iterator<Item=Track<Act>>, world: &mut World, in_undo: bool) -> EditRecordResult {
    let mut store = world.get_resource_or_insert_with::<Record<Act>>(Record::<Act>::default);
    if in_undo && store.progressing {
        return Err(UndoRedoInProgress);
    }
    if in_undo {
        store.redo.clear();
    }
    store.tracks.extend(track);
    Ok(())
}

fn push_track<Act: Send + Sync + 'static>(track: Track<Act>, world: &mut World, in_undo: bool) -> EditRecordResult {
    let mut store = world.get_resource_or_insert_with::<Record<Act>>(Record::<Act>::default);
    if in_undo && store.progressing {
        return Err(UndoRedoInProgress);
    }
    if in_undo {
        store.redo.clear();
    }
    store.tracks.push(track);
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::Commands;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{Action, record, wait};
    use crate::action::record::track::Track;
    use crate::prelude::{ActionSeed, EditRecordResult, Omit, Reactor, Record, Redo, Rollback, Then, Undo};
    use crate::tests::{decrement_count, increment_count, NumAct, test_app, TestAct};

    pub fn push_num_act(num: usize) -> ActionSeed {
        record::push().with(Track {
            act: NumAct(num),
            rollback: Rollback::parts(
                Undo::make(increment_count),
                Redo::make(|_| decrement_count())
            )
        })
            .omit()
    }

    pub fn push_undo_increment() -> Action<Track<TestAct>, EditRecordResult> {
        record::push().with(Track {
            act: TestAct,
            rollback: Rollback::parts(
                Undo::make(increment_count),
                Redo::make(|_| decrement_count())
            )
        })
    }

    #[test]
    fn clear_if_undo_finished() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push_undo_increment()
                    .then(record::undo::once::<TestAct>()),
                )
                    .await
                    .unwrap();
            }));
        });
        app.update();
        assert!(app.resource_mut::<Record<TestAct>>().all_clear().is_ok());
    }

    #[test]
    fn clear_failed_if_undo_doing() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, record::push().with(Track {
                    act: TestAct,
                    rollback: Rollback::undo(|| wait::until(|| false)),
                })
                    .then(record::undo::once::<TestAct>()))
                    .await
                    .unwrap();
            }));
        });
        app.update();
        assert!(app.resource_mut::<Record<TestAct>>().all_clear().is_err());
    }
}
