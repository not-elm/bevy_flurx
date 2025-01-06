//! This example introduces how to create a custom runner.
//!
//! You can create more complex actions by using the custom runner.

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            FlurxPlugin,
        ))
        .add_systems(Startup, spawn_reactor)
        .run();
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        task.will(Update, {
            once::run(|| info!("Start"))
                .then(delayed_log().with(DelayedLogInput {
                    delay: Duration::from_secs(3),
                    message: "After 3 seconds, this message is displayed.",
                }))
                .then(once::event::app_exit_success())
        }).await;
    }));
}

struct DelayedLogInput {
    delay: Duration,
    message: &'static str,
}

/// Here, we create an action that outputs a log after a specified time has elapsed.
/// Use the built-in action [`delay::time`] for the delay.
struct DelayedLogRunner {
    message: &'static str,
    delay_time_runner: BoxedRunner,
    output: Output<&'static str>,
    cancellation_id: Option<CancellationId>,
}

impl Runner for DelayedLogRunner {
    fn run(&mut self, world: &mut World, cancellation_handlers: &mut CancellationHandlers) -> RunnerIs {
        let cancellation_id = self.cancellation_id.get_or_insert_with(||{
            // You can register a handler that will be invoked when the reactor is canceled.
            cancellation_handlers.register(|_world: &mut World| {
                info!("CancellationLogRunner is canceled.");
            })
        });
        match self.delay_time_runner.run(world, cancellation_handlers) {
            RunnerIs::Completed => {
                info!("{}", self.message);
                // Note that you must set the output value; otherwise, the action’s completion will not be notified.
                self.output.set(self.message);
                // Be sure to unregister the handler when the action is completed.
                cancellation_handlers.unregister(cancellation_id);
                RunnerIs::Completed
            }
            other => other
        }
    }
}

/// Finally, we create a function that returns an action.
/// 
/// It is recommended that the action's input is passed by the [`ActionSeed::with`] instead of the argument of this function.
/// By doing so, you can pass the input value with [`Pipe::pipe`].
fn delayed_log() -> ActionSeed<DelayedLogInput, &'static str> {
    ActionSeed::new(|input: DelayedLogInput, output: Output<&'static str>| {
        let delay_action: Action<Duration> = delay::time().with(input.delay);
        let delay_time_runner: BoxedRunner = delay_action.create_runner(Output::default());
        DelayedLogRunner {
            output,
            delay_time_runner,
            message: input.message,
            cancellation_id: None,
        }
    })
}