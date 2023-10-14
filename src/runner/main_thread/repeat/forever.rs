use bevy::prelude::World;
use futures::channel::mpsc::Sender;

use crate::prelude::{BoxedMainThreadExecutor, MainThreadExecutable};
use crate::runner::AsyncSystemStatus;
use crate::runner::main_thread::{BaseRunner, IntoMainThreadExecutor};
use crate::runner::main_thread::config::AsyncSystemConfig;

pub(crate) struct Forever(pub AsyncSystemConfig);


impl IntoMainThreadExecutor for Forever {
    #[inline]
    fn into_executor(self, sender: Sender<()>) -> BoxedMainThreadExecutor {
        Box::new(ForeverRunner {
            base: BaseRunner::new(self.0),
            sender,
        })
    }
}


struct ForeverRunner {
    sender: Sender<()>,
    base: BaseRunner,
}


impl MainThreadExecutable for ForeverRunner {
    fn run(&mut self, world: &mut World) -> AsyncSystemStatus {
        // If an error is returned, the task has already stopped.
        if self.sender.try_send(()).is_err() {
            return AsyncSystemStatus::Finished;
        }
        self.base.run_with_output(world);
        AsyncSystemStatus::Running
    }
}


