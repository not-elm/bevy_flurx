use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;

use crate::prelude::{BoxedMainThreadExecutor, MainThreadExecutable};
use crate::runner::AsyncSystemStatus;
use crate::runner::main_thread::{BaseRunner, IntoMainThreadExecutor};
use crate::runner::main_thread::config::AsyncSystemConfig;

pub(crate) struct Times {
    repeat_num: usize,
    config: AsyncSystemConfig,
}


impl Times {
    pub fn create<Marker>(repeat_num: usize, system: impl IntoSystem<(), (), Marker> + Send + 'static) -> impl IntoMainThreadExecutor {
        Self {
            repeat_num,
            config: AsyncSystemConfig::new(system),
        }
    }
}


impl IntoMainThreadExecutor for Times {
    #[inline]
    fn into_executor(self, sender: Sender<()>) -> BoxedMainThreadExecutor {
        Box::new(RepeatRunner {
            sender,
            repeat_num: self.repeat_num,
            current_num: 0,
            base: BaseRunner::new(self.config),
        })
    }
}


struct RepeatRunner {
    sender: Sender<()>,
    repeat_num: usize,
    current_num: usize,
    base: BaseRunner,
}


impl MainThreadExecutable for RepeatRunner {
    fn run(&mut self, world: &mut World) -> AsyncSystemStatus {
        if self.repeat_num <= self.current_num {
            return AsyncSystemStatus::Finished;
        }

        self.base.run_with_output(world);
        self.current_num += 1;

        if self.repeat_num <= self.current_num {
            let _ = self.sender.try_send(());
            AsyncSystemStatus::Finished
        } else {
            AsyncSystemStatus::Running
        }
    }
}