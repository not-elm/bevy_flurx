use bevy::prelude::World;
use bevy::time::{Time, Timer};
use futures::channel::mpsc::Sender;
use futures::StreamExt;

use crate::runner::{AsyncSystem, AsyncSystemRunnable, BoxedAsyncSystemRunner, BoxedTaskFuture, new_channel, SystemRunningStatus};

#[derive(Clone, Debug)]
pub enum Delay {
    Frame(usize),
    Timer(Timer),
}


impl AsyncSystem<()> for Delay {
    fn split(self) -> (BoxedAsyncSystemRunner, BoxedTaskFuture<()>) {
        let (tx, mut rx) = new_channel(1);
        let runner: BoxedAsyncSystemRunner = match self {
            Self::Frame(delay_frames) => Box::new(DelayFrameRunner {
                current_ticks: 0,
                delay_frames,
                tx,
            }),
            Self::Timer(timer) => Box::new(DelayTimerRunner {
                tx,
                timer,
            })
        };

        (runner, Box::pin(async move {
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
            SystemRunningStatus::Running
        } else {
            self.tx.try_send(()).unwrap();
            SystemRunningStatus::Finished
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
            self.tx.try_send(()).unwrap();
            SystemRunningStatus::Finished
        }
    }
}