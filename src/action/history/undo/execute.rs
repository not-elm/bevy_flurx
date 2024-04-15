use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::history::{CreateUndoAction, HistoryStore};
use crate::prelude::{ActionSeed, Output, Runner};
use crate::runner::{BoxedRunner, CancellationToken};

pub fn execute<M>() -> ActionSeed
    where
        M: 'static
{
    ActionSeed::new(|_, token, output| {
        UndoRunner {
            token,
            output,
            undo_output: Output::default(),
            undo_runner: None,
            create_undo: None,
            _m: PhantomData::<M>,
        }
    })
}

struct UndoRunner<M> {
    token: CancellationToken,
    output: Output<()>,
    undo_output: Output<Option<ActionSeed>>,
    undo_runner: Option<BoxedRunner>,
    create_undo: Option<CreateUndoAction>,
    _m: PhantomData<M>,
}

impl<M> Runner for UndoRunner<M>
    where
        M: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }
        if self.undo_runner.is_none() {
            if let Some((create_undo, action)) = world
                .get_non_send_resource_mut::<HistoryStore<M>>()
                .and_then(|mut store| store.undo.pop())
            {
                let runner = action.into_runner(self.token.clone(), self.undo_output.clone());
                self.undo_runner.replace(runner);
                self.create_undo.replace(create_undo);
            }
        }
        let Some(undo_runner) = self.undo_runner.as_mut() else {
            self.output.replace(());
            return true;
        };

        undo_runner.run(world);

        let Some(redo) = self.undo_output.take() else {
            return false;
        };
        if let Some(redo) = redo {
            let create_undo = self.create_undo.take().unwrap();
            world.non_send_resource_mut::<HistoryStore<M>>().redo.push((create_undo, redo));
            self.output.replace(());
        }
        true
    }
}
