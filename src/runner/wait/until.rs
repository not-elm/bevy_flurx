use bevy::prelude::{Event, EventReader, IntoSystem, World};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::StreamExt;

use crate::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, IntoAsyncSystem, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub(crate) struct Until {
    config: AsyncSystemConfig<bool>,
}


impl Until {
    #[inline]
    pub fn create<Marker>(system: impl IntoSystem<(), bool, Marker> + 'static + Send) -> impl IntoAsyncSystem<()> {
        Self {
            config: AsyncSystemConfig::new(system)
        }
    }


    #[inline]
    pub fn event<E: Event>() -> impl IntoAsyncSystem<()> {
        Self::create(|er: EventReader<E>| {
            !er.is_empty()
        })
    }
}


impl IntoAsyncSystem<()> for Until {
    fn into_parts(self) -> (BoxedAsyncSystemRunner, Task<()>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(UntilRunner(BaseRunner::new(tx, self.config)));
        (runner, AsyncComputeTaskPool::get().spawn(async move {
            loop {
                if rx.next().await.is_some_and(|finished| finished) {
                    return;
                }
            }
        }))
    }
}


struct UntilRunner(BaseRunner<bool>);

impl AsyncSystemRunnable for UntilRunner {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let finished = self.0.run_with_output(world);
        if finished {
            let _ = self.0.tx.try_send(true);
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}