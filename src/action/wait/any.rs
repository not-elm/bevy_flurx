use bevy::prelude::World;

use crate::prelude::ActionSeed;
use crate::runner::{BoxedRunner, CancellationToken, Output, Runner};

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
///     let index: usize = task.will(Update, wait::any(actions![
///         wait::input::just_pressed().with(KeyCode::KeyB),
///         wait::event::comes::<AppExit>()
///     ])).await;
/// });
/// ```
pub fn any(actions: impl IntoIterator<Item=ActionSeed> + 'static) -> ActionSeed<(), usize> {
    ActionSeed::new(move |_, output| {
        let runners = actions
            .into_iter()
            .map(|action| action.with(()).into_runner(Output::default()))
            .collect::<Vec<_>>();
        if runners.is_empty() {
            panic!("The length of actions passed to `wait::any` must be greater than 0.")
        }

        AnyRunner {
            output,
            runners,
        }
    })
}

struct AnyRunner {
    output: Output<usize>,
    runners: Vec<BoxedRunner>,
}

impl Runner for AnyRunner {
    fn run(&mut self, world: &mut World, token: &CancellationToken) -> bool {
        for (i, runner) in self.runners.iter_mut().enumerate() {
            if runner.run(world, token) {
                self.output.set(i);
                return true;
            }
        }
        false
    }

    fn on_cancelled(&mut self, world: &mut World) {
        for runner in self.runners.iter_mut(){
            runner.on_cancelled(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Commands, Update};
    use bevy_test_helper::event::DirectEvents;

    use crate::action::{delay, once};
    use crate::actions;
    use crate::prelude::wait;
    use crate::reactor::Reactor;
    use crate::tests::test_app;

    #[test]
    fn return_1() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let index = task.will(Update, wait::any(actions![
                    wait::until(|| false),
                    once::run(|| {})
                ])).await;
                if index == 1 {
                    task.will(Update, once::event::app_exit()).await;
                }
            }));
        });
        app.update();
        app.update();
        let mut er = ManualEventReader::<AppExit>::default();
        app.assert_event_comes(&mut er);
    }

    #[test]
    fn return_0() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let index = task.will(Update, wait::any(actions![
                    delay::frames().with(1),
                    delay::frames().with(3),
                    wait::until(||false)
                ])).await;
                if index == 0 {
                    task.will(Update, once::event::app_exit()).await;
                }
            }));
        });
        let mut er = ManualEventReader::<AppExit>::default();
        app.update();
        app.assert_event_not_comes(&mut er);
        
        app.update();
        app.update();
        app.assert_event_comes(&mut er);
    }
}