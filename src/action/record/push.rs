use bevy::prelude::World;

use crate::action::record::push_track;
use crate::prelude::{ActionSeed, Output, Runner};
use crate::prelude::record::{Track};
use crate::prelude::record::EditRecordResult;
use crate::runner::CancellationToken;

/// Push the [`Track`](crate::prelude::Track) onto the [`Record`](crate::prelude::Record).
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
///
/// # Examples
///
/// ```no_run
///
/// use bevy::prelude::*;
/// use bevy::window::CursorIcon::Move;
/// use futures::SinkExt;
/// use bevy_flurx::prelude::*;
///
/// struct MoveAct;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, record::push()
///         .with(Track{
///             act: MoveAct,
///             rollback: Rollback::undo_redo(|| once::run(|mut player: Query<&mut Transform>|{
///                 let pos = player.single_mut().translation;
///                 player.single_mut().translation = Vec3::Z;
///                 RedoAction::new(once::run(move |mut player: Query<&mut Transform>|{
///                     player.single_mut().translation = pos;
///                 }))
///             }))
///         }))
///         .await
///         .unwrap();
/// });
/// ```
pub fn push<Act>() -> ActionSeed<Track<Act>, EditRecordResult>
    where
        Act: Send + Sync + 'static,
{
    ActionSeed::new(|track: Track<Act>, token, output| {
        PushRunner {
            output,
            operation: Some(track),
            token,
        }
    })
}

struct PushRunner<Act> {
    token: CancellationToken,
    operation: Option<Track<Act>>,
    output: Output<EditRecordResult>,
}

impl<Opr> Runner for PushRunner<Opr>
    where
        Opr: Send + Sync + 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        if let Some(operation) = self.operation.take() {
            if let Err(error) = push_track::<Opr>(operation, world, true) {
                self.output.replace(Err(error));
                return true;
            }
        }
        self.output.replace(Ok(()));
        true
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::Startup;
    use bevy::prelude::{Commands, Update};
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{Omit, once, record};
    use crate::action::record::{Record, Track};
    use crate::prelude::{ActionSeed, Reactor, Rollback};
    use crate::tests::{test_app};


    #[derive(Default)]
    struct H1;

    #[derive(Default)]
    struct H2;

    #[test]
    fn push1() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push(H1)).await;
            }));
        });
        app.update();
        app.assert_resource(1, |h: &Record<H1>| h.tracks.len());
    }

    #[test]
    fn push2() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push(H1)).await;
                task.will(Update, push(H1)).await;
            }));
        });
        app.update();
        app.assert_resource(1, |h: &Record<H1>| h.tracks.len());
        app.update();
        app.assert_resource(2, |h: &Record<H1>| h.tracks.len());
    }

    #[test]
    fn multi_push() {
        let mut app = test_app();
        app.world.init_resource::<Record<H2>>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, push(H1)).await;
                task.will(Update, push(H2)).await;
                task.will(Update, push(H1)).await;
            }));
        });
        app.update();
        app.assert_resource(1, |h: &Record<H1>| h.tracks.len());
        app.assert_resource(0, |h: &Record<H2>| h.tracks.len());
        app.update();
        app.assert_resource(1, |h: &Record<H1>| h.tracks.len());
        app.assert_resource(1, |h: &Record<H2>| h.tracks.len());
        app.update();
        app.assert_resource(2, |h: &Record<H1>| h.tracks.len());
        app.assert_resource(1, |h: &Record<H2>| h.tracks.len());
    }

    fn push<Act: Send + Sync + 'static>(act: Act) -> ActionSeed {
        record::push().with(Track {
            act,
            rollback: Rollback::undo(|| {
                once::run(|| {})
            }),
        })
            .omit()
    }
}