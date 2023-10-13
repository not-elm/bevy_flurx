use std::future::Future;

use async_trait::async_trait;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Commands, Component, Deref, DerefMut};
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::task_pool::TaskPool;

pub mod state_schedule_label;

#[async_trait]
pub trait AsyncPool<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(TaskPool) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + Send + 'static;
}


impl<'w, 's> AsyncPool<'w, 's> for Commands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(TaskPool) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + Send + 'static {
        let task_pool = TaskPool::default();
        let handle = AsyncComputeTaskPool::get().spawn(f(task_pool.clone()));

        self.spawn((
            task_pool,
            BevTask(handle)
        ))
    }
}


#[derive(Component, Deref, DerefMut)]
pub struct BevTask(pub(crate) Task<()>);

