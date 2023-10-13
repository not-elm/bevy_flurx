use std::time::Duration;

use bevy::prelude::{TimerMode, World};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::time::{Time, Timer};
use futures::channel::mpsc::Sender;
use futures::StreamExt;

use crate::runner::{AsyncSystemRunnable, BoxedAsyncSystemRunner, IntoAsyncSystem, new_channel, SystemRunningStatus};

/// Delay the task using either [`Delay::Frames`] or [`Delay::Time`].
///
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_async_system::ext::AsyncCommands;
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


impl IntoAsyncSystem<()> for Delay {
    fn into_parts(self) -> (BoxedAsyncSystemRunner, Task<()>) {
        let (tx, mut rx) = new_channel(1);
        let runner: BoxedAsyncSystemRunner = match self {
            Self::Frames(delay_frames) => Box::new(DelayFrameRunner {
                current_ticks: 0,
                delay_frames,
                tx,
            }),
            Self::Time(duration) => Box::new(DelayTimerRunner {
                tx,
                timer: Timer::new(duration, TimerMode::Once),
            })
        };

        (runner, AsyncComputeTaskPool::get().spawn(async move {
            loop {
                if rx.next().await.is_some() {
                    return;
                }
            }
        }))
    }
}


struct DelayTimerRunner {
    timer: Timer,
    tx: Sender<()>,
}


impl AsyncSystemRunnable for DelayTimerRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let delta = world.resource::<Time>().delta();
        if self.timer.tick(delta).just_finished() {
            let _ = self.tx.try_send(());
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}


struct DelayFrameRunner {
    current_ticks: usize,
    delay_frames: usize,
    tx: Sender<()>,
}


impl AsyncSystemRunnable for DelayFrameRunner {
    fn run(&mut self, _: &mut World) -> SystemRunningStatus {
        self.current_ticks += 1;

        if self.current_ticks < self.delay_frames {
            SystemRunningStatus::Running
        } else {
            let _ = self.tx.try_send(());
            SystemRunningStatus::Finished
        }
    }
}