use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::Action;
use crate::prelude::{ActionSeed, Map, Omit};

pub mod undo;
pub mod redo;

type UndoAction = Action<(), Option<ActionSeed>>;
type CreateUndoAction = Box<dyn Fn() -> UndoAction>;
type UndoTuple = (CreateUndoAction, UndoAction);
type Redo = (CreateUndoAction, ActionSeed);

pub struct HistoryStore<M> {
    pub(crate) undo: Vec<UndoTuple>,
    pub(crate) redo: Vec<Redo>,
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

pub trait IntoUndoActionSeed<I> {
    type ActionInput;

    fn into_undo_action_seed(self, input: Self::ActionInput) -> Action<I, Option<ActionSeed>>;
}

impl<I> IntoUndoActionSeed<I> for Action<I, ()>
    where
        I: 'static
{
    type ActionInput = ();

    fn into_undo_action_seed(self, _: Self::ActionInput) -> Action<I, Option<ActionSeed>> {
        self.map(|_| None)
    }
}

impl<I> IntoUndoActionSeed<I> for ActionSeed<I, ()>
    where
        I: 'static
{
    type ActionInput = I;

    fn into_undo_action_seed(self, input: Self::ActionInput) -> Action<I, Option<ActionSeed>> {
        self.map(|_| None).with(input)
    }
}

impl<I, Om> IntoUndoActionSeed<I> for Action<I, Om>
    where
        I: 'static,
        Om: Omit + 'static
{
    type ActionInput = ();

    fn into_undo_action_seed(self, _: Self::ActionInput) -> Action<I, Option<ActionSeed>> {
        self.map(|omit| Some(omit.omit()))
    }
}

impl<I, Om> IntoUndoActionSeed<I> for ActionSeed<I, Om>
    where
        I: 'static,
        Om: Omit + 'static
{
    type ActionInput = I;

    fn into_undo_action_seed(self, input: Self::ActionInput) -> Action<I, Option<ActionSeed>> {
        self.map(|omit| Some(omit.omit())).with(input)
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
