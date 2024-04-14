use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::history::history_store::HistoryStore;
use crate::prelude::{ActionSeed, Output};
use crate::runner::{BoxedRunner, CancellationToken, Runner};

pub fn execute<M: 'static>() -> ActionSeed {
    ActionSeed::new(|_, token, output| {
        RedoRunner {
            token,
            output,
            redo_runner: None,
            _m: PhantomData::<M>,
        }
    })
}


struct RedoRunner<M> {
    token: CancellationToken,
    output: Output<()>,
    redo_runner: Option<BoxedRunner>,
    _m: PhantomData<M>,
}

impl<M> Runner for RedoRunner<M>
    where
        M: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        if self.redo_runner.is_none() {
            if let Some(action) = world
                .get_non_send_resource_mut::<HistoryStore<M>>()
                .and_then(|mut store| store.redo.pop())
                .map(|seed| seed.with(()))
            {
                let runner = action.into_runner(self.token.clone(), self.output.clone());
                self.redo_runner.replace(runner);
            } else {
                return true;
            }
        }
        self.redo_runner
            .as_mut()
            .unwrap()
            .run(world)
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::{Commands, ResMut};
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{once, redo, undo};
    use crate::reactor::Reactor;
    use crate::tests::{exit_reader, test_app};

    struct M;

    #[test]
    fn redo_decrement() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, undo::push(
                    M,
                    once::run(|mut count: ResMut<Count>| {
                        count.increment();
                        once::run(|mut count: ResMut<Count>| {
                            count.decrement();
                        })
                    })
                )).await;
                task.will(Update, undo::execute::<M>()).await;
                task.will(Update, redo::execute::<M>()).await;
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
}