use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Deref, DerefMut};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::channel::mpsc::{Receiver, Sender};
use futures::StreamExt;

use crate::runner::main_thread::{IntoMainThreadExecutor, MainThreadExecutors};

#[derive(Component, Deref, DerefMut)]
pub struct TaskHandle(pub(crate) Task<()>);


#[derive(Component, Deref, DerefMut)]
pub struct TaskSender<Out>(pub(crate) Sender<Out>);


#[derive(Default, Clone)]
pub struct AsyncCommands {
    pub(crate) main_thread_runners: MainThreadExecutors,

}


impl AsyncCommands {
    pub fn spawn<Out: Send + 'static>(
        &self,
        schedule_label: impl ScheduleLabel + Clone,
        into_executor: impl IntoMainThreadExecutor<Out>,
    ) -> impl Future<Output=Out> {
        let (tx, rx) = futures::channel::mpsc::channel(1);
        let executor = into_executor.into_executor(TaskSender(tx), schedule_label);
        self.main_thread_runners.push(executor);

        create_output_future(rx)
    }
}


#[inline]
fn create_output_future<Out: Send + 'static>(mut rx: Receiver<Out>) -> impl Future<Output=Out> {
    AsyncComputeTaskPool::get().spawn(async move {
        loop {
            if let Some(output) = rx.next().await {
                return output;
            }
        }
    })
}