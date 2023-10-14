use bevy::prelude::World;
use futures::channel::mpsc::Sender;

use crate::prelude::{AsyncSystemRunnable, BoxedAsyncSystemRunner};
use crate::runner::main_thread::{BaseRunner, IntoAsyncSystemRunner, SystemRunningStatus};
use crate::runner::main_thread::config::AsyncSystemConfig;

pub(crate) struct Forever(pub AsyncSystemConfig);


impl IntoAsyncSystemRunner for Forever {
    #[inline]
    fn into_runner(self, sender: Sender<()>) -> BoxedAsyncSystemRunner {
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

impl AsyncSystemRunnable for ForeverRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        // If an error is returned, the task has already stopped.
        if self.sender.try_send(()).is_err() {
            return SystemRunningStatus::Finished;
        }
        self.base.run_with_output(world);
        SystemRunningStatus::Running
    }
}


