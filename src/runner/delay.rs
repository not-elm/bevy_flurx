use std::time::Duration;

use bevy::prelude::{TimerMode, World};
use bevy::time::{Time, Timer};
use futures::channel::mpsc::Sender;

use crate::runner::{AsyncSystemRunnable, BoxedAsyncSystemRunner, IntoAsyncSystemRunner, SystemRunningStatus};

/// Delay the task using either [`Delay::Frames`] or [`Delay::Time`].
///
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_async_system::ext::SpawnAsyncCommands;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|task| async move{
///         // Wait for 3 frames.
///         task.spawn(Update, Delay::Frames(3)).await;
///
///         // Wait for 3 seconds.
///         task.spawn(Update, Delay::Time(Duration::from_secs(3))).await;
///     });
/// }
/// ```
#[derive(Clone, Debug)]
pub enum Delay {
    Frames(usize),
    Time(Duration),
}


impl IntoAsyncSystemRunner<()> for Delay {
    fn into_runner(self, sender: Sender<()>) -> BoxedAsyncSystemRunner {
        let runner: BoxedAsyncSystemRunner = match self {
            Self::Frames(delay_frames) => Box::new(DelayFrameRunner {
                current_ticks: 0,
                delay_frames,
                sender,
            }),
            Self::Time(duration) => Box::new(DelayTimerRunner {
                sender,
                timer: Timer::new(duration, TimerMode::Once),
            })
        };

        runner
    }
}


struct DelayTimerRunner {
    timer: Timer,
    sender: Sender<()>,
}


impl AsyncSystemRunnable for DelayTimerRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let delta = world.resource::<Time>().delta();
        if self.timer.tick(delta).just_finished() {
            let _ = self.sender.try_send(());
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}


struct DelayFrameRunner {
    current_ticks: usize,
    delay_frames: usize,
    sender: Sender<()>,
}


impl AsyncSystemRunnable for DelayFrameRunner {
    fn run(&mut self, _: &mut World) -> SystemRunningStatus {
        self.current_ticks += 1;

        if self.current_ticks < self.delay_frames {
            SystemRunningStatus::Running
        } else {
            let _ = self.sender.try_send(());
            SystemRunningStatus::Finished
        }
    }
}