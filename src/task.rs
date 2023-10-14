use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Deref, DerefMut};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::StreamExt;

use crate::runner::non_send::{IntoAsyncSystemRunner, NonSendRunners};

#[derive(Component, Deref, DerefMut)]
pub struct TaskHandle(pub(crate) Task<()>);

#[derive(Default, Clone)]
pub struct AsyncCommands(pub(crate) NonSendRunners);


impl AsyncCommands {
    pub fn spawn<Out: Send + 'static>(
        &self,
        schedule_label: impl ScheduleLabel,
        into_async_system: impl IntoAsyncSystemRunner<Out>,
    ) -> impl Future<Output=Out> {
        let (tx, mut rx) = futures::channel::mpsc::channel(1);
        let runner = into_async_system.into_runner(tx);
        self.0.insert(Box::new(schedule_label), runner);

        AsyncComputeTaskPool::get().spawn(async move {
            loop {
                if let Some(output) = rx.next().await {
                    return output;
                }
            }
        })
    }
}


