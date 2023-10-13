use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;
use futures::StreamExt;

use crate::runner::{AsyncSystem, AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, BoxedTaskFuture, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub struct Repeat {
    repeat_num: Option<usize>,
    config: AsyncSystemConfig,
}


impl Repeat {
    #[inline]
    pub fn run_for<Marker>(num: usize, system: impl IntoSystem<(), (), Marker> + 'static + Send) -> Self {
        Self {
            repeat_num: Some(num),
            config: AsyncSystemConfig::new(system),
        }
    }


    #[inline]
    pub fn run_forever<Marker>(system: impl IntoSystem<(), (), Marker> + 'static + Send) -> Self {
        Self {
            repeat_num: None,
            config: AsyncSystemConfig::new(system),
        }
    }


    fn runner(self, tx: Sender<()>) -> BoxedAsyncSystemRunner {
        if let Some(repeat_num) = self.repeat_num {
            Box::new(RepeatRunner {
                repeat_num,
                current_num: 0,
                base: BaseRunner::new(tx, self.config),
            })
        } else {
            Box::new(LoopRunner(BaseRunner::new(tx, self.config)))
        }
    }
}


impl AsyncSystem<()> for Repeat {
    fn split(self) -> (BoxedAsyncSystemRunner, BoxedTaskFuture<()>) {
        let (tx, mut rx) = new_channel::<()>(1);

        let runner = self.runner(tx);
        (runner, Box::pin(async move {
            loop {
                if rx.next().await.is_some() {
                    return;
                }
            }
        }))
    }
}

struct LoopRunner(BaseRunner);

impl AsyncSystemRunnable for LoopRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        self.0.run_with_output(world);
        SystemRunningStatus::Running
    }
}


struct RepeatRunner {
    repeat_num: usize,
    current_num: usize,
    base: BaseRunner,
}

impl AsyncSystemRunnable for RepeatRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        if self.repeat_num <= self.current_num {
            return SystemRunningStatus::Finished;
        }

        self.base.run_with_output(world);
        self.current_num += 1;

        if self.repeat_num <= self.current_num {
            let _ = self.base.tx.try_send(());
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}