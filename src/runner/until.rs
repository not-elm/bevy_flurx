use bevy::prelude::{Event, EventReader, IntoSystem, World};
use futures::StreamExt;

use crate::runner::{AsyncSystem, AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, BoxedTaskFuture, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;



pub struct Until {

    config: AsyncSystemConfig<(), bool>,
}


impl Until {
    #[inline]
    pub fn run<Marker>(system: impl IntoSystem<(), bool, Marker> + 'static + Send) -> Until {
        Self{
            config: AsyncSystemConfig::new(system)
        }
    }


    #[inline]
    pub fn come_event<E: Event>() -> Self{
        Self::run(|er: EventReader<E>|{
            !er.is_empty()
        })
    }
}


impl AsyncSystem<()> for Until {
    fn split(self) -> (BoxedAsyncSystemRunner, BoxedTaskFuture<()>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(UntilRunner(BaseRunner::new(tx, self.config)));
        (runner, Box::pin(async move{
            loop {
                if rx.next().await.is_some_and(|finished|finished){
                   return;
                }
            }
        }))
    }
}


struct UntilRunner(BaseRunner<(), bool>);

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