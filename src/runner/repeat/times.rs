use bevy::prelude::{IntoSystem, World};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::StreamExt;

use crate::prelude::{AsyncSystemRunnable, BoxedAsyncSystemRunner};
use crate::runner::{BaseRunner, IntoAsyncSystem, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub(crate) struct Times {
    repeat_num: usize,
    config: AsyncSystemConfig
}


impl Times {
    pub fn create<Marker>(repeat_num: usize, system: impl IntoSystem<(), (), Marker> + Send + 'static) -> impl IntoAsyncSystem{
        Self{
            repeat_num,
            config: AsyncSystemConfig::new(system)
        }
    }
}


impl IntoAsyncSystem for Times {
    fn into_parts(self) -> (BoxedAsyncSystemRunner, Task<()>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(RepeatRunner {
            repeat_num: self.repeat_num,
            current_num: 0,
            base: BaseRunner::new(tx, self.config),
        });

        (runner, AsyncComputeTaskPool::get().spawn(async move {
            loop {
                if rx.next().await.is_some() {
                    return;
                }
            }
        }))
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