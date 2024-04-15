use bevy::utils::petgraph::data::Create;
pub use execute::execute;
pub use push::push;
use crate::action::history::CreateUndoAction;

mod push;
mod execute;


pub struct Undo(CreateUndoAction);


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