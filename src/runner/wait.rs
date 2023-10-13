use bevy::prelude::{Event, EventReader, World};
use futures::StreamExt;

use crate::runner::{AsyncSystem, AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, BoxedTaskFuture, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub struct Wait<In, Out> {
    config: AsyncSystemConfig<In, Option<Out>>,
}


impl<Out> Wait<(), Out> {
    #[inline]
    pub fn run<Marker>(system: impl bevy::prelude::IntoSystem<(), Option<Out>, Marker> + 'static + Send) -> Wait<(), Out> {
        Self {
            config: AsyncSystemConfig::with_input((), system)
        }
    }
}


impl<E: Event + Clone> Wait<(), E> {
    #[inline]
    pub fn event() -> Wait<(), E> {
        Self::run(|mut er: EventReader<E>| {
            er.iter().next().cloned()
        })
    }
}


impl<In, Out> AsyncSystem<Out> for Wait<In, Out>
    where In: 'static + Clone,
          Out: 'static + Send
{
    fn split(self) -> (BoxedAsyncSystemRunner, BoxedTaskFuture<Out>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(WaitRunner(BaseRunner::new(tx, self.config)));
        (runner, Box::pin(async move {
            loop {
                if let Some(output) = rx.next().await.and_then(|output| output) {
                    return output;
                }
            }
        }))
    }
}


struct WaitRunner<In, Out>(BaseRunner<In, Option<Out>>);

impl<In, Out> AsyncSystemRunnable for WaitRunner<In, Out>
    where In: 'static + Clone,
          Out: 'static + Send
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        if let Some(output) = self.0.run_with_output(world) {
            self.0.tx.try_send(Some(output)).unwrap();
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}