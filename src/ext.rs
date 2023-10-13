pub mod state_schedule_label;

use std::future::Future;

use async_trait::async_trait;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Commands, Component, Deref, DerefMut};
use bevy::tasks::AsyncComputeTaskPool;
use bevy_async_task::{AsyncReceiver, AsyncTask};
use futures::future::poll_immediate;

use crate::task_pool::TaskPool;

#[async_trait]
pub trait AsyncPool<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(TaskPool) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + Send + 'static;
}


impl<'w, 's> AsyncPool<'w, 's> for Commands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(TaskPool) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + Send + 'static {
        let task_pool = TaskPool::default();
        let task: AsyncTask<()> = f(task_pool.clone()).into();
        let (fut, task_finish_rx) = task.into_parts();
        let handle = AsyncComputeTaskPool::get().spawn(fut);
        handle.detach();

        self.spawn((
            task_pool,
            ProcessReceiver(task_finish_rx)
        ))
    }
}


#[derive(Component, Deref, DerefMut)]
pub(crate) struct ProcessReceiver(AsyncReceiver<()>);


impl ProcessReceiver {
    #[inline(always)]
    pub fn finished(&mut self) -> bool{
        self.0.try_recv().is_some()
    }
}