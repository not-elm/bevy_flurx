use bevy::prelude::{Deref, DerefMut, Event, EventWriter, NextState, ResMut, States, World};
use futures::StreamExt;

use crate::impl_async_runner_constructor;
use crate::runner::{AsyncSystem, AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, BoxedTaskFuture, new_channel, SystemRunningStatus};
use crate::runner::config::AsyncSystemConfig;

pub struct Once<In, Out>(AsyncSystemConfig<In, Out>);

impl_async_runner_constructor!(Once);


impl Once<(), ()> {
    #[inline]
    pub fn set_state<S: States + Copy>(to: S) -> Once<(), ()> {
        Self::run(move |mut state: ResMut<NextState<S>>| {
            state.set(to);
        })
    }


    #[inline]
    pub fn send<E: Event + Clone>(event: E) -> Once<(), ()> {
        Self::run(move |mut ew: EventWriter<E>| {
            ew.send(event.clone());
        })
    }
}


impl<In, Out> AsyncSystem<Out> for Once<In, Out>
    where
        In: Clone + 'static,
        Out: 'static + Send
{
    fn split(self) -> (BoxedAsyncSystemRunner, BoxedTaskFuture<Out>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(OnceRunner(BaseRunner::new(tx, self.0)));
        (runner, Box::pin(async move {
            loop {
                if let Some(output) = rx.next().await {
                    return output;
                }
            }
        }))
    }
}


#[derive(Deref, DerefMut)]
struct OnceRunner<In, Out>(BaseRunner<In, Out>) where In: Clone;

impl<In, Output> AsyncSystemRunnable for OnceRunner<In, Output>
    where
        In: Clone + 'static,
        Output: 'static
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let output = self.run_with_output(world);
        let _ = self.tx.try_send(output);
        SystemRunningStatus::Finished
    }
}
