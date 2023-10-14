use std::future::Future;
use async_compat::CompatExt;
use async_trait::async_trait;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::Commands;
use bevy::tasks::AsyncComputeTaskPool;
use crate::async_commands::{AsyncSchedules, TaskHandle};

#[async_trait]
pub trait SpawnAsyncSystem<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + Send + 'static;

    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + 'static;
}


impl<'w, 's> SpawnAsyncSystem<'w, 's> for Commands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + Send + 'static {
        let async_commands = AsyncSchedules::default();
        let handle = AsyncComputeTaskPool::get().spawn(f(async_commands.clone()).compat());

        self.spawn((
            async_commands.schedulers,
            TaskHandle(handle)
        ))
    }


    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(AsyncSchedules) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + 'static {
        let async_commands = AsyncSchedules::default();
        let handle = AsyncComputeTaskPool::get().spawn_local(f(async_commands.clone()).compat());

        self.spawn((
            async_commands.schedulers,
            TaskHandle(handle)
        ))
    }
}




