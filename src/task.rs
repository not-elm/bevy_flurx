use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::World;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_async_task::{AsyncReceiver, AsyncTask};

use crate::task::commands::AsyncCommands;

mod commands;


#[derive(Default)]
pub struct BevTask {
    tasks: Vec<AsyncSystemTask>,
}


impl BevTask {
    pub fn spawn_async<F>(&mut self, f: impl Fn(AsyncCommands) -> F )
        where F: Future<Output=()> + Send + 'static
    {
        let async_commands = AsyncCommands::default();
        let task: AsyncTask<()> = f(async_commands.clone()).into();
        let (fut, task_finish_rx) = task.into_parts();
        let task_pool = AsyncComputeTaskPool::get();
        let handle = task_pool.spawn(fut);
        handle.detach();

        self.tasks.push(AsyncSystemTask {
            commands: async_commands,
            task_finish_rx,
        });
    }


    pub(crate) fn remove_finished_tasks(&mut self) {
        let mut next_tasks = Vec::with_capacity(self.tasks.len());
        while let Some(mut task) = self.tasks.pop() {
            if task.finished() {
                continue;
            }

            next_tasks.push(task);
        }

        self.tasks = next_tasks;
    }


    pub(crate) fn update(
        &mut self,
        schedule_label: impl ScheduleLabel,
        world: &mut World,
    ) {
        for task in self.tasks.iter_mut() {
            task.run_systems(&schedule_label, world);
        }
    }
}


pub struct AsyncSystemTask {
    commands: AsyncCommands,
    task_finish_rx: AsyncReceiver<()>,
}


impl AsyncSystemTask {
    #[inline]
    pub fn finished(&mut self) -> bool {
        self.task_finish_rx.try_recv().is_some()
    }


    #[inline]
    pub fn run_systems(&mut self, schedule_label: &dyn ScheduleLabel, world: &mut World) {
        self.commands.run_systems(schedule_label, world);
    }
}