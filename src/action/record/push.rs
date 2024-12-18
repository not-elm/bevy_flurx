use bevy::prelude::World;
use crate::action::record::push_track;
use crate::prelude::record::EditRecordResult;
use crate::prelude::record::Track;
use crate::prelude::{ActionSeed, CancellationHandlers, Output, Runner};
use crate::runner::RunnerIs;

/// Push the [`Track`](crate::prelude::Track) onto the [`Record`](crate::prelude::Record).
///
/// The output will be [`UndoRedoInProgress`](crate::prelude::UndoRedoInProgress) if an `undo` or `redo` is in progress.
///
/// # Examples
///
/// ```no_run
///
/// use bevy::prelude::*;
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
    ActionSeed::new(|track: Track<Act>, output| PushRunner {
        output,
        track: Some(track),
    })
}

struct PushRunner<Act> {
    track: Option<Track<Act>>,
    output: Output<EditRecordResult>,
}

impl<Act> Runner for PushRunner<Act>
where
    Act: Send + Sync + 'static,
{
    fn run(&mut self, world: &mut World, _: &mut CancellationHandlers) -> RunnerIs {
        if let Some(track) = self.track.take() {
            if let Err(error) = push_track::<Act>(track, world, true) {
                self.output.set(Err(error));
                return RunnerIs::Completed;
            }
        }
        self.output.set(Ok(()));
        RunnerIs::Completed
    }
}

#[cfg(test)]
mod tests {
    use crate::action::record::{Record, Track};
    use crate::action::{once, record};
    use crate::prelude::{ActionSeed, Reactor, Omit, Rollback};
    use crate::tests::test_app;
    use bevy::app::Startup;
    use bevy::prelude::{Commands, Update};
    use bevy_test_helper::resource::DirectResourceControl;

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
        app.world_mut().init_resource::<Record<H2>>();
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
        record::push()
            .with(Track {
                act,
                rollback: Rollback::undo(|| once::run(|| {})),
            })
            .omit()
    }
}
