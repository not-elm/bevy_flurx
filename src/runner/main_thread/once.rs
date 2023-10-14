use bevy::prelude::{Event, EventWriter, IntoSystem, NextState, ResMut, States, World};
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::main_thread::{BaseRunner, BoxedMainThreadExecutor, IntoMainThreadExecutor, MainThreadExecutable};
use crate::runner::main_thread::config::AsyncSystemConfig;

pub struct Once<Out>(AsyncSystemConfig<Out>);


impl<Out: Send + 'static> Once<Out> {
    #[inline(always)]
    pub fn run<Marker>(system: impl IntoSystem<(), Out, Marker> + Send + 'static) -> impl IntoMainThreadExecutor<Out> {
        Self(AsyncSystemConfig::new(system))
    }
}


impl Once<()> {
    #[inline]
    pub fn set_state<S: States + Copy>(to: S) -> impl IntoMainThreadExecutor {
        Self::run(move |mut state: ResMut<NextState<S>>| {
            state.set(to);
        })
    }


    #[inline]
    pub fn send<E: Event + Clone>(event: E) -> impl IntoMainThreadExecutor {
        Self::run(move |mut ew: EventWriter<E>| {
            ew.send(event.clone());
        })
    }
}


impl<Out> IntoMainThreadExecutor<Out> for Once<Out>
    where Out: 'static + Send
{
    fn into_executor(self, sender: Sender<Out>) -> BoxedMainThreadExecutor {
        Box::new(OnceRunner {
            base: BaseRunner::new(self.0),
            sender,
        })
    }
}


struct OnceRunner<Out> {
    base: BaseRunner<Out>,
    sender: Sender<Out>,
}


impl<Output> MainThreadExecutable for OnceRunner<Output>
    where
        Output: 'static
{
    fn run(&mut self, world: &mut World) -> AsyncSystemStatus {
        let output = self.base.run_with_output(world);
        let _ = self.sender.try_send(output);
        AsyncSystemStatus::Finished
    }
}
