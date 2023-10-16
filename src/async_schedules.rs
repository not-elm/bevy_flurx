use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Deref, DerefMut};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::channel::mpsc::{Receiver, Sender};
use futures::StreamExt;

use crate::runner::{IntoAsyncScheduleCommand, AsyncScheduleCommands};

#[derive(Component, Deref, DerefMut)]
pub struct TaskHandle(pub(crate) Task<()>);


#[derive(Component, Deref, DerefMut)]
pub struct TaskSender<Out>(pub(crate) Sender<Out>);


#[derive(Default, Clone)]
pub struct AsyncSchedules {
    pub(crate) schedulers: AsyncScheduleCommands,
}


impl AsyncSchedules {
    pub fn add_system<Out: Send + 'static>(
        &self,
        schedule_label: impl ScheduleLabel + Clone,
        into_schedule_command: impl IntoAsyncScheduleCommand<Out>,
    ) -> impl Future<Output=Out> {
        let (tx, rx) = futures::channel::mpsc::channel(1);
        self.schedulers.push(into_schedule_command.into_schedule_command(TaskSender(tx), schedule_label));

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