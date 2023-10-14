use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Component, Deref, DerefMut};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::channel::mpsc::Receiver;
use futures::StreamExt;

use crate::runner::main_thread::{IntoMainThreadExecutor, MainThreadExecutors};
use crate::runner::thread_pool::{IntoThreadPoolExecutor, TaskPoolExecutors, TaskPoolSystemSetups};

#[derive(Component, Deref, DerefMut)]
pub struct TaskHandle(pub(crate) Task<()>);


#[derive(Default, Clone)]
pub struct AsyncCommands {
    pub(crate) main_thread_runners: MainThreadExecutors,
    pub(crate) multi_thread_runners: TaskPoolExecutors,
    pub(crate) setups: TaskPoolSystemSetups,
}


impl AsyncCommands {
    pub fn spawn<Param: SystemParam + 'static, Out: Send + 'static>(
        &self,
        schedule_label: impl ScheduleLabel + Clone,
        into_executor: impl IntoThreadPoolExecutor<Param, Out>,
    ) -> impl Future<Output=Out> {
        let (tx, rx) = futures::channel::mpsc::channel(1);
        let executor = into_executor.into_executor(tx);
        self.multi_thread_runners.insert(Box::new(schedule_label.clone()), executor);
        self.setups.push::<Param>(schedule_label);
        create_output_future(rx)
    }


    pub fn spawn_on_main<Out: Send + 'static>(
        &self,
        schedule_label: impl ScheduleLabel,
        into_executor: impl IntoMainThreadExecutor<Out>,
    ) -> impl Future<Output=Out> {
        let (tx, rx) = futures::channel::mpsc::channel(1);
        let runner = into_executor.into_executor(tx);
        self.main_thread_runners.insert(Box::new(schedule_label), runner);

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