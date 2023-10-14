use std::future::Future;

use async_compat::CompatExt;
use async_trait::async_trait;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::Commands;
use bevy::tasks::AsyncComputeTaskPool;

use crate::task::{AsyncCommands, TaskHandle};

pub mod state_schedule_label;

#[async_trait]
pub trait SpawnAsyncCommands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncCommands) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + Send + 'static;

    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(AsyncCommands) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + 'static;
}


impl<'w, 's> SpawnAsyncCommands<'w, 's> for Commands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncCommands) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + Send + 'static {
        let task = AsyncCommands::default();
        let handle = AsyncComputeTaskPool::get().spawn(f(task.clone()).compat());

        self.spawn((
            task.0,
            TaskHandle(handle)
        ))
    }


    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(AsyncCommands) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + 'static {
        let task = AsyncCommands::default();
        let handle = AsyncComputeTaskPool::get().spawn_local(f(task.clone()).compat());

        self.spawn((
            task.0,
            TaskHandle(handle)
        ))
    }
}




