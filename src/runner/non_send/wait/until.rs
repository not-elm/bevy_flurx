use bevy::prelude::{Event, EventReader, IntoSystem, World};
use futures::channel::mpsc::Sender;

use crate::runner::non_send::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, IntoAsyncSystemRunner, SystemRunningStatus};
use crate::runner::non_send::config::AsyncSystemConfig;

pub(crate) struct Until {
    config: AsyncSystemConfig<bool>,
}


impl Until {
    #[inline]
    pub fn create<Marker>(system: impl IntoSystem<(), bool, Marker> + 'static + Send) -> impl IntoAsyncSystemRunner<()> {
        Self {
            config: AsyncSystemConfig::new(system)
        }
    }


    #[inline]
    pub fn event<E: Event>() -> impl IntoAsyncSystemRunner<()> {
        Self::create(|er: EventReader<E>| {
            !er.is_empty()
        })
    }
}


impl IntoAsyncSystemRunner<()> for Until {
    #[inline]
    fn into_runner(self, sender: Sender<()>) -> BoxedAsyncSystemRunner {
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

impl AsyncSystemRunnable for UntilRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let finished = self.base.run_with_output(world);
        if finished {
            let _ = self.sender.try_send(());
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}