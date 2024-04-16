use std::marker::PhantomData;

use bevy::prelude::{NonSendMut, World};

use crate::action::{Action, once};
use crate::action::redo::Redo;
use crate::prelude::ActionSeed;


pub mod undo;
pub mod redo;


/// Clear the [`HistoryStore`].
pub fn clear<M: 'static>() -> ActionSeed{
    once::run(|mut store: NonSendMut<HistoryStore<M>>|{
        store.clear();
    })
}

type UndoAction = Action<(), Option<Redo>>;
type CreateUndoAction = Box<dyn Fn() -> UndoAction>;
type UndoTuple = (CreateUndoAction, UndoAction);

/// Manage the history of `undo` and `redo`.
///
/// This struct has one marker type.
/// This allows you can define different the histories for each type of marker.
pub struct HistoryStore<M> {
    pub(crate) undo: Vec<UndoTuple>,
    pub(crate) redo: Vec<(CreateUndoAction, Redo)>,
    _m: PhantomData<M>,
}

impl<M> HistoryStore<M>
    where
        M: 'static
{
    /// Clear all history of `undo` and `redo`.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Push the function that creates `undo`.
    pub fn push_undo(&mut self, create_undo_action: CreateUndoAction) {
        let action = create_undo_action();
        self.redo.clear();
        self.undo.push((create_undo_action, action));
    }
}

impl<M> Default for HistoryStore<M> {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            _m: PhantomData,
        }
    }
}

fn push_undo<M: 'static>(undo: UndoTuple, world: &mut World, in_undo: bool) {
    world.init_non_send_resource::<HistoryStore<M>>();
    let mut store = world.non_send_resource_mut::<HistoryStore<M>>();
    if in_undo {
        store.redo.clear();
    }
    store.undo.push(undo);
}
