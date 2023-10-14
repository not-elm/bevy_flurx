use bevy::prelude::{Deref, DerefMut, Event, EventWriter, IntoSystem, NextState, ResMut, States, World};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::StreamExt;

use crate::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, IntoAsyncSystem, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub struct Once<Out>(AsyncSystemConfig<Out>);


impl<Out: Send + 'static> Once<Out> {
    #[inline(always)]
    pub fn run<Marker>(system: impl IntoSystem<(), Out, Marker> + Send + 'static) -> impl IntoAsyncSystem<Out> {
        Self(AsyncSystemConfig::new(system))
    }
}


impl Once<()> {
    #[inline]
    pub fn set_state<S: States + Copy>(to: S) -> impl IntoAsyncSystem {
        Self::run(move |mut state: ResMut<NextState<S>>| {
            state.set(to);
        })
    }


    #[inline]
    pub fn send<E: Event + Clone>(event: E) -> impl IntoAsyncSystem {
        Self::run(move |mut ew: EventWriter<E>| {
            ew.send(event.clone());
        })
    }
}


impl<Out> IntoAsyncSystem<Out> for Once<Out>
    where Out: 'static + Send
{
    fn into_parts(self) -> (BoxedAsyncSystemRunner, Task<Out>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(OnceRunner(BaseRunner::new(tx, self.0)));
        (runner, AsyncComputeTaskPool::get().spawn(async move {
            loop {
                if let Some(output) = rx.next().await {
                    return output;
                }
            }
        }))
    }
}


#[derive(Deref, DerefMut)]
struct OnceRunner<Out>(BaseRunner<Out>);

impl<Output> AsyncSystemRunnable for OnceRunner<Output>
    where
        Output: 'static
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let output = self.run_with_output(world);
        let _ = self.tx.try_send(output);
        SystemRunningStatus::Finished
    }
}
