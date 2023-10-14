use bevy::prelude::{Event, EventReader, World};
use futures::channel::mpsc::Sender;

use crate::prelude::{BoxedMainThreadExecutor, MainThreadExecutable};
use crate::runner::AsyncSystemStatus;
use crate::runner::main_thread::{BaseRunner, IntoMainThreadExecutor};
use crate::runner::main_thread::config::AsyncSystemConfig;

pub(crate) struct WaitOutput<Out> {
    config: AsyncSystemConfig<Option<Out>>,
}


impl<Out: Send + 'static> WaitOutput<Out> {
    #[inline]
    pub fn create<Marker>(system: impl bevy::prelude::IntoSystem<(), Option<Out>, Marker> + 'static + Send) -> impl IntoMainThreadExecutor<Out> {
        Self {
            config: AsyncSystemConfig::new(system)
        }
    }
}


impl<E: Event + Clone> WaitOutput<E> {
    #[inline]
    pub fn event() -> impl IntoMainThreadExecutor<E> {
        Self::create(|mut er: EventReader<E>| {
            er.iter().next().cloned()
        })
    }
}


impl<Out> IntoMainThreadExecutor<Out> for WaitOutput<Out>
    where
        Out: 'static + Send
{
    #[inline]
    fn into_executor(self, sender: Sender<Out>) -> BoxedMainThreadExecutor {
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


impl<Out> MainThreadExecutable for WaitOutputRunner<Out>
    where
        Out: 'static + Send
{
    fn run(&mut self, world: &mut World) -> AsyncSystemStatus {
        if let Some(output) = self.base.run_with_output(world) {
            let _ = self.sender.try_send(output);
            AsyncSystemStatus::Finished
        } else {
            AsyncSystemStatus::Running
        }
    }
}