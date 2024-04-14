use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::{Action, OmitInput};
use crate::action::history::history_store::HistoryStore;
use crate::action::history::IntoUndoActionSeed;
use crate::prelude::{ActionSeed, Output};
use crate::runner::{BoxedRunner, CancellationToken, Runner};

pub fn push<I, M, A>(_m: M, action: A) -> ActionSeed<A::ActionInput>
    where
        I: 'static,
        A: IntoUndoActionSeed<I> + 'static,
        M: 'static,
{
    ActionSeed::new(|input, token, output| {
        UndoPushRunner {
            output,
            undo_runner: Some(action.into_undo_action_seed(input).omit_input().with(())),
            token,
            _m: PhantomData::<M>,
        }
    })
}

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
            _m: PhantomData::<M>,
        }
    })
}

struct UndoPushRunner<M> {
    token: CancellationToken,
    undo_runner: Option<Action<(), Option<ActionSeed>>>,
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

        if let Some(action) = self.undo_runner.take() {
            world.init_non_send_resource::<HistoryStore<M>>();
            let mut store = world.non_send_resource_mut::<HistoryStore<M>>();
            store.redo.clear();
            store.undo.push(action);
        }
        self.output.replace(());
        true
    }
}

struct UndoRunner<M> {
    token: CancellationToken,
    output: Output<()>,
    undo_output: Output<Option<ActionSeed>>,
    undo_runner: Option<BoxedRunner>,
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
            if let Some(action) = world
                .get_non_send_resource_mut::<HistoryStore<M>>()
                .and_then(|mut store| store.undo.pop())
            {
                let runner = action.into_runner(self.token.clone(), self.undo_output.clone());
                self.undo_runner.replace(runner);
            }
        }
        let Some(undo_runner) = self.undo_runner.as_mut() else {
            return true;
        };

        undo_runner.run(world);

        let Some(redo) = self.undo_output.take() else {
            return false;
        };
        if let Some(redo) = redo {
            world.non_send_resource_mut::<HistoryStore<M>>().redo.push(redo);
            self.output.replace(());
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::Commands;
    use bevy_test_helper::event::DirectEvents;

    use crate::action::{once, undo};
    use crate::reactor::Reactor;
    use crate::tests::{exit_reader, test_app};

    struct M;

    #[test]
    fn undo() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, undo::push(M, once::event::app_exit())).await;
                task.will(Update, undo::execute::<M>()).await;
            }));
        });
        let mut er = exit_reader();
        app.update();
        app.assert_event_not_comes(&mut er);

        app.update();
        app.assert_event_comes(&mut er);
    }
}