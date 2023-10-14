use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;

use crate::prelude::{AsyncSystemRunnable, BoxedAsyncSystemRunner};
use crate::runner::non_send::{BaseRunner, IntoAsyncSystemRunner, SystemRunningStatus};
use crate::runner::non_send::config::AsyncSystemConfig;

pub(crate) struct Times {
    repeat_num: usize,
    config: AsyncSystemConfig,
}


impl Times {
    pub fn create<Marker>(repeat_num: usize, system: impl IntoSystem<(), (), Marker> + Send + 'static) -> impl IntoAsyncSystemRunner {
        Self {
            repeat_num,
            config: AsyncSystemConfig::new(system),
        }
    }
}


impl IntoAsyncSystemRunner for Times {
    #[inline]
    fn into_runner(self, sender: Sender<()>) -> BoxedAsyncSystemRunner {
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

impl AsyncSystemRunnable for RepeatRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        if self.repeat_num <= self.current_num {
            return SystemRunningStatus::Finished;
        }

        self.base.run_with_output(world);
        self.current_num += 1;

        if self.repeat_num <= self.current_num {
            let _ = self.sender.try_send(());
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}