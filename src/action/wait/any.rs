use crate::prelude::ActionSeed;
use crate::runner::{BoxedRunner, CancellationHandlers, Output, Runner, RunnerIs};
use bevy::prelude::*;

/// Wait until the execution of one of the actions is completed.
///
/// The output value is the index of the completed action.
///
/// # Panics
///
/// Panicked if actions is empty.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::actions;
/// use bevy_flurx::prelude::*;
/// use bevy::app::AppExit;
///
/// Reactor::schedule(|task| async move{
///     // The output value is the index of the completed action.
///     let index: usize = task.will(Update, wait::any().with(actions![
///         wait::input::just_pressed().with(KeyCode::KeyB),
///         wait::event::comes::<AppExit>()
///     ])).await;
/// });
/// ```
pub fn any<Actions>() -> ActionSeed<Actions, usize>
where
    Actions: IntoIterator<Item = ActionSeed> + 'static,
{
    ActionSeed::new(move |actions: Actions, output| {
        let runners = actions
            .into_iter()
            .map(|action| action.with(()).create_runner(Output::default()))
            .collect::<Vec<_>>();
        if runners.is_empty() {
            panic!("The length of actions passed to `wait::any` must be greater than 0.")
        }

        AnyRunner { output, runners }
    })
}

struct AnyRunner {
    output: Output<usize>,
    runners: Vec<BoxedRunner>,
}

impl Runner for AnyRunner {
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        let mut finished = None;
        for (i, runner) in self.runners.iter_mut().enumerate() {
            match runner.run(world, token) {
                RunnerIs::Completed => {
                    finished.replace(i);
                    break;
                }
                RunnerIs::Canceled => {
                    return RunnerIs::Canceled;
                }
                RunnerIs::Running => continue,
            }
        }
        if let Some(finished_index) = finished {
            self.runners.clear();
            self.output.set(finished_index);
            RunnerIs::Completed
        } else {
            RunnerIs::Running
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::{delay, once};
    use crate::actions;
    use crate::prelude::{wait, Reactor};
    use crate::tests::test_app;
    use bevy::app::{AppExit, Startup};
    use bevy::prelude::{Commands, Events, Update};
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn return_1() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let index = task
                    .will(
                        Update,
                        wait::any().with(actions![wait::until(|| false), once::run(|| {})]),
                    )
                    .await;
                if index == 1 {
                    task.will(Update, once::event::app_exit_success()).await;
                }
            }));
        });
        app.update();
        app.update();
        let mut er = app.resource_mut::<Events<AppExit>>().get_cursor();
        app.assert_event_comes(&mut er);
    }

    #[test]
    fn return_0() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let index = task
                    .will(
                        Update,
                        wait::any().with(actions![
                            delay::frames().with(1),
                            delay::frames().with(3),
                            wait::until(|| false)
                        ]),
                    )
                    .await;
                if index == 0 {
                    task.will(Update, once::event::app_exit_success()).await;
                }
            }));
        });

        let mut er = app.resource_mut::<Events<AppExit>>().get_cursor();
        app.update();
        app.assert_event_not_comes(&mut er);

        app.update();
        app.update();
        app.assert_event_comes(&mut er);
    }
}
