use bevy::prelude::{Event, EventReader, World};
use futures::channel::mpsc::Sender;

use crate::prelude::{AsyncSystemRunnable, BoxedAsyncSystemRunner};
use crate::runner::non_send::{BaseRunner, IntoAsyncSystemRunner, SystemRunningStatus};
use crate::runner::non_send::config::AsyncSystemConfig;

pub(crate) struct WaitOutput<Out> {
    config: AsyncSystemConfig<Option<Out>>,
}

impl<Out: Send + 'static> WaitOutput<Out> {
    #[inline]
    pub fn create<Marker>(system: impl bevy::prelude::IntoSystem<(), Option<Out>, Marker> + 'static + Send) -> impl IntoAsyncSystemRunner<Out> {
        Self {
            config: AsyncSystemConfig::new(system)
        }
    }
}


impl<E: Event + Clone> WaitOutput<E> {
    #[inline]
    pub fn event() -> impl IntoAsyncSystemRunner<E> {
        Self::create(|mut er: EventReader<E>| {
            er.iter().next().cloned()
        })
    }
}


impl<Out> IntoAsyncSystemRunner<Out> for WaitOutput<Out>
    where
        Out: 'static + Send
{
    #[inline]
    fn into_runner(self, sender: Sender<Out>) -> BoxedAsyncSystemRunner {
        Box::new(WaitOutputRunner {
            sender,
            base: BaseRunner::new(self.config),
        })
    }
}


struct WaitOutputRunner<Out> {
    sender: Sender<Out>,
    base: BaseRunner<Option<Out>>,
}


impl<Out> AsyncSystemRunnable for WaitOutputRunner<Out>
    where
        Out: 'static + Send
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        if let Some(output) = self.base.run_with_output(world) {
            let _ = self.sender.try_send(output);
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}