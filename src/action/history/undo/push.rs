use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::history::{CreateUndoAction, push_undo, UndoTuple};
use crate::prelude::{ActionSeed, Output, Runner};
use crate::runner::CancellationToken;


/// Push the [`Undo`](crate::prelude::Undo) into the [`HistoryStore`](crate::prelude::HistoryStore).
/// 
/// # Examples
/// 
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::action::redo::Redo;
/// use bevy_flurx::prelude::*;
///
/// struct MoveHistory;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, undo::push(Undo::<MoveHistory>::new(|_| once::run(|mut player: Query<&mut Transform>|{
///         player.single_mut().translation = Vec3::Z;
///     })))).await;
/// 
///     // you can also register `Redo` by `Undo::with_redo`.
///     task.will(Update, undo::push(Undo::<MoveHistory>::with_redo(|_| once::run(|mut player: Query<&mut Transform>|{
///         let pos = player.single_mut().translation;
///         player.single_mut().translation = Vec3::Z;
///         
///         Redo(once::run(move |mut player: Query<&mut Transform>|{
///             player.single_mut().translation = pos;
///         }))
///     })))).await;
/// });
/// ```
pub fn push<I, M, F>((_, undo): (PhantomData<M>, F)) -> ActionSeed<I>
    where
        I: Clone + 'static,
        F: FnOnce(I) -> CreateUndoAction + 'static,
        M: 'static,
{
    ActionSeed::new(|input: I, token, output| {
        let create_undo = undo(input);
        let undo_action = create_undo();

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

#[cfg(test)]
mod tests {
    use bevy::app::Startup;
    use bevy::prelude::{Commands, Update};

    use crate::action::{once, undo};
    use crate::action::history::HistoryStore;
    use crate::action::undo::Undo;
    use crate::prelude::Reactor;
    use crate::tests::test_app;

    struct H1;

    struct H2;

    #[test]
    fn push1() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, undo::push(Undo::<H1>::new(|_| once::run(|| {})))).await;
            }));
        });
        app.update();
        assert_eq!(app.world.non_send_resource::<HistoryStore<H1>>().undo.len(), 1);
    }

    #[test]
    fn push2() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, undo::push(Undo::<H1>::new(|_| once::run(|| {})))).await;
                task.will(Update, undo::push(Undo::<H1>::new(|_| once::run(|| {})))).await;
            }));
        });
        app.update();
        assert_eq!(app.world.non_send_resource::<HistoryStore<H1>>().undo.len(), 1);
        app.update();
        assert_eq!(app.world.non_send_resource::<HistoryStore<H1>>().undo.len(), 2);
    }

    #[test]
    fn multi_push() {
        let mut app = test_app();
        app.world.init_non_send_resource::<HistoryStore<H2>>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, undo::push(Undo::<H1>::new(|_| once::run(|| {})))).await;
                task.will(Update, undo::push(Undo::<H2>::new(|_| once::run(|| {})))).await;
                task.will(Update, undo::push(Undo::<H1>::new(|_| once::run(|| {})))).await;
            }));
        });
        app.update();
        assert_eq!(app.world.non_send_resource::<HistoryStore<H1>>().undo.len(), 1);
        assert_eq!(app.world.non_send_resource::<HistoryStore<H2>>().undo.len(), 0);
        app.update();
        assert_eq!(app.world.non_send_resource::<HistoryStore<H1>>().undo.len(), 1);
        assert_eq!(app.world.non_send_resource::<HistoryStore<H2>>().undo.len(), 1);
        app.update();
        assert_eq!(app.world.non_send_resource::<HistoryStore<H1>>().undo.len(), 2);
        assert_eq!(app.world.non_send_resource::<HistoryStore<H2>>().undo.len(), 1);
    }
}