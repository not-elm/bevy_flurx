use bevy::prelude::World;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::StreamExt;

use crate::prelude::{AsyncSystemRunnable, BoxedAsyncSystemRunner};
use crate::runner::{BaseRunner, IntoAsyncSystem, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub(crate) struct Forever(pub AsyncSystemConfig);


impl IntoAsyncSystem for Forever {
    fn into_parts(self) -> (BoxedAsyncSystemRunner, Task<()>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(ForeverRunner(BaseRunner::new(tx, self.0)));

        (runner, AsyncComputeTaskPool::get().spawn(async move {
            loop {
                let _ = rx.next().await;
            }
        }))
    }
}

struct ForeverRunner(BaseRunner);

impl AsyncSystemRunnable for ForeverRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        // If an error is returned, the task has already stopped.
        if self.0.tx.try_send(()).is_err() {
            return SystemRunningStatus::Finished;
        }
        self.0.run_with_output(world);
        SystemRunningStatus::Running
    }
}


