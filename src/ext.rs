use std::future::Future;

use async_compat::CompatExt;
use async_trait::async_trait;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::Commands;
use bevy::tasks::AsyncComputeTaskPool;

use crate::task::{BevTaskCommands, BevTaskHandle};

pub mod state_schedule_label;

#[async_trait]
pub trait AsyncCommands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(BevTaskCommands) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + Send + 'static;

    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(BevTaskCommands) -> F) -> EntityCommands<'w, 's, 'a>
        where F: Future<Output=()> + 'static;
}


impl<'w, 's> AsyncCommands<'w, 's> for Commands<'w, 's> {
    fn spawn_async<'a, F>(&'a mut self, f: impl Fn(BevTaskCommands) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + Send + 'static {
        let task = BevTaskCommands::default();
        let handle = AsyncComputeTaskPool::get().spawn(f(task.clone()).compat());

        self.spawn((
            task.0,
            BevTaskHandle(handle)
        ))
    }


    fn spawn_async_local<'a, F>(&'a mut self, f: impl Fn(BevTaskCommands) -> F) -> EntityCommands<'w, 's, 'a> where F: Future<Output=()> + 'static {
        let task = BevTaskCommands::default();
        let handle = AsyncComputeTaskPool::get().spawn_local(f(task.clone()).compat());

        self.spawn((
            task.0,
            BevTaskHandle(handle)
        ))
    }
}




