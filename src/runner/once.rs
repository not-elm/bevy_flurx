use bevy::prelude::{Event, EventWriter, IntoSystem, NextState, ResMut, States, World};
use futures::channel::mpsc::Sender;

use crate::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, IntoAsyncSystemRunner, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub struct Once<Out>(AsyncSystemConfig<Out>);


impl<Out: Send + 'static> Once<Out> {
    #[inline(always)]
    pub fn run<Marker>(system: impl IntoSystem<(), Out, Marker> + Send + 'static) -> impl IntoAsyncSystemRunner<Out> {
        Self(AsyncSystemConfig::new(system))
    }
}


impl Once<()> {
    #[inline]
    pub fn set_state<S: States + Copy>(to: S) -> impl IntoAsyncSystemRunner {
        Self::run(move |mut state: ResMut<NextState<S>>| {
            state.set(to);
        })
    }


    #[inline]
    pub fn send<E: Event + Clone>(event: E) -> impl IntoAsyncSystemRunner {
        Self::run(move |mut ew: EventWriter<E>| {
            ew.send(event.clone());
        })
    }
}


impl<Out> IntoAsyncSystemRunner<Out> for Once<Out>
    where Out: 'static + Send
{
    fn into_runner(self, sender: Sender<Out>) -> BoxedAsyncSystemRunner {
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

impl<Output> AsyncSystemRunnable for OnceRunner<Output>
    where
        Output: 'static
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let output = self.base.run_with_output(world);
        let _ = self.sender.try_send(output);
        SystemRunningStatus::Finished
    }
}
