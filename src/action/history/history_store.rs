use std::marker::PhantomData;

use crate::action::Action;
use crate::prelude::ActionSeed;

pub struct HistoryStore<M> {
    pub(crate) undo: Vec<Action<(), Option<ActionSeed>>>,
    pub(crate) redo: Vec<ActionSeed>,
    _m: PhantomData<M>,
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