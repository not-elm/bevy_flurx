use crate::action::Action;
use crate::prelude::{ActionSeed, Map, Omit};

pub mod history_store;
pub mod undo;
pub mod redo;


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


