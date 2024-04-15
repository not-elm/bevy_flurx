use std::marker::PhantomData;

pub use execute::execute;
pub use push::push;

use crate::action::{Action, OmitInput};
use crate::action::history::CreateUndoAction;
use crate::action::redo::Redo;
use crate::prelude::Map;

mod push;
mod execute;


pub struct Undo<History>(PhantomData<History>);

impl<History> Undo<History>
    where
        History: 'static
{
    /// Create the function that creates the `undo action`.
    /// 
    /// Another value of the outputs is used by [`undo::push`] to determine which [`HistoryStore`](crate::prelude::HistoryStore) to use. 
    /// 
    /// The action that created by this method do not created the [`Redo`] action.
    /// If you want it, use [`Undo::with_redo`] instead. 
    #[inline]
    pub const fn new<I1, I2, O, A, F>(f: F) -> (PhantomData<History>, impl FnOnce(I1) -> CreateUndoAction + 'static)
        where
            I1: Clone + 'static,
            I2: 'static,
            O: 'static,
            F: Fn(I1) -> A + 'static,
            A: Into<Action<I2, O>> + 'static
    {
        (PhantomData::<History>, |input| {
            Box::new(move || {
                f(input.clone()).into().map(|_| None).omit_input().with(())
            })
        })
    }


    /// Create a new [`Undo`] with [`Redo`].
    /// 
    /// Another value of the outputs is used by [`undo::push`] to determine which [`HistoryStore`](crate::prelude::HistoryStore) to use. 
    /// 
    #[inline]
    pub const fn with_redo<I1, I2, A, F>(f: F) -> (PhantomData<History>, impl FnOnce(I1) -> CreateUndoAction + 'static)
        where
            I1: Clone + 'static,
            I2: 'static,
            F: Fn(I1) -> A + 'static,
            A: Into<Action<I2, Redo>> + 'static
    {
        (PhantomData::<History>, |input| {
            Box::new(move || {
                f(input.clone()).into().map(Some).omit_input().with(())
            })
        })
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::Commands;
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{once, sequence::Then, undo};
    use crate::action::undo::Undo;
    use crate::reactor::Reactor;
    use crate::tests::{exit_reader, increment_count, test_app};

    struct H;

    #[test]
    fn undo_app_exit() {
        let mut app = test_app();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, undo::push(Undo::<H>::new(|_| once::event::app_exit()))).await;
                task.will(Update, undo::execute::<H>()).await;
            }));
        });
        let mut er = exit_reader();
        app.update();
        app.assert_event_not_comes(&mut er);

        app.update();
        app.assert_event_comes(&mut er);
    }

    #[test]
    fn undo_multi_times() {
        let mut app = test_app();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, undo::push(Undo::<H>::new(|_| increment_count()))
                    .then(undo::push(Undo::<H>::new(|_| increment_count())))
                    .then(undo::push(Undo::<H>::new(|_| increment_count()))),
                ).await;
                task.will(Update, undo::execute::<H>()).await;
                task.will(Update, undo::execute::<H>()).await;
                task.will(Update, undo::execute::<H>()).await;
            }));
        });
        app.update();
        for i in 1..10 {
            app.update();
            app.assert_resource_eq(Count(3.min(i)));
        }
    }
}