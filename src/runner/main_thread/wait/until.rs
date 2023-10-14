use bevy::prelude::{Event, EventReader, IntoSystem, World};
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::main_thread::{BaseRunner, BoxedMainThreadExecutor, IntoMainThreadExecutor, MainThreadExecutable};
use crate::runner::main_thread::config::AsyncSystemConfig;

pub(crate) struct Until {
    config: AsyncSystemConfig<bool>,
}


impl Until {
    #[inline]
    pub fn create<Marker>(system: impl IntoSystem<(), bool, Marker> + 'static + Send) -> impl IntoMainThreadExecutor<()> {
        Self {
            config: AsyncSystemConfig::new(system)
        }
    }


    #[inline]
    pub fn event<E: Event>() -> impl IntoMainThreadExecutor<()> {
        Self::create(|er: EventReader<E>| {
            !er.is_empty()
        })
    }
}


impl IntoMainThreadExecutor<()> for Until {
    #[inline]
    fn into_executor(self, sender: Sender<()>) -> BoxedMainThreadExecutor {
        Box::new(UntilRunner {
            base: BaseRunner::new(self.config),
            sender,
        })
    }
}


struct UntilRunner {
    sender: Sender<()>,
    base: BaseRunner<bool>,
}


impl MainThreadExecutable for UntilRunner {
    fn run(&mut self, world: &mut World) -> AsyncSystemStatus {
        let finished = self.base.run_with_output(world);
        if finished {
            let _ = self.sender.try_send(());
            AsyncSystemStatus::Finished
        } else {
            AsyncSystemStatus::Running
        }
    }
}