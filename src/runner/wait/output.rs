use bevy::prelude::{Event, EventReader};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::StreamExt;

use crate::prelude::BoxedAsyncSystemRunner;
use crate::runner::{BaseRunner, IntoAsyncSystem, new_channel};
use crate::runner::config::AsyncSystemConfig;
use crate::runner::wait::WaitRunner;

pub(crate) struct WaitOutput<Out> {
    config: AsyncSystemConfig<Option<Out>>,
}

impl<Out: Send + 'static> WaitOutput<Out> {
    #[inline]
    pub fn create<Marker>(system: impl bevy::prelude::IntoSystem<(), Option<Out>, Marker> + 'static + Send) -> impl IntoAsyncSystem<Out> {
        Self {
            config: AsyncSystemConfig::new(system)
        }
    }
}


impl<E: Event + Clone> WaitOutput<E> {
    #[inline]
    pub fn event() -> impl IntoAsyncSystem<E> {
        Self::create(|mut er: EventReader<E>| {
            er.iter().next().cloned()
        })
    }
}


impl<Out> IntoAsyncSystem<Out> for WaitOutput<Out>
    where
        Out: 'static + Send
{
    fn into_parts(self) -> (BoxedAsyncSystemRunner, Task<Out>) {
        let (tx, mut rx) = new_channel(1);
        let runner = Box::new(WaitRunner(BaseRunner::new(tx, self.config)));
        (runner, AsyncComputeTaskPool::get().spawn(async move {
            loop {
                if let Some(output) = rx.next().await.and_then(|output| output) {
                    return output;
                }
            }
        }))
    }
}