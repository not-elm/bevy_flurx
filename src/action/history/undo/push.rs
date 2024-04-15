use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::{Action, OmitInput};
use crate::action::history::{push_undo, UndoTuple};
use crate::prelude::{ActionSeed, Output, Runner};
use crate::runner::CancellationToken;

pub fn push<I, M, F>(_m: M, create_undo_action: F) -> ActionSeed<I>
    where
        I: Clone + 'static,
        F: Fn(I) -> Action<I, Option<ActionSeed>> + 'static,
        M: 'static,
{
    ActionSeed::new(|input: I, token, output| {
        let undo_action = create_undo_action(input.clone()).omit_input().with(());
        let create_undo = Box::new(move || {
            create_undo_action(input.clone()).omit_input().with(())
        });
        UndoPushRunner {
            output,
            undo: Some((create_undo, undo_action)),
            token,
            _m: PhantomData::<M>,
        }
    })
}

struct UndoPushRunner<M> {
    token: CancellationToken,
    undo: Option<UndoTuple>,
    output: Output<()>,
    _m: PhantomData<M>,
}

impl<M> Runner for UndoPushRunner<M>
    where
        M: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        if let Some(undo) = self.undo.take() {
            push_undo::<M>(undo, world, true);
        }
        self.output.replace(());
        true
    }
}