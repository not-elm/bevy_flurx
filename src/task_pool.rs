use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, IntoSystem, World};
use futures::channel::mpsc::channel;
use futures::StreamExt;

use crate::runner::delay::DelayRunner;
use crate::runner::once::OnceRunner;
use crate::runner::Runners;

mod wait;
mod until;

#[derive(Default, Component, Clone)]
pub struct TaskPool(Runners);


impl TaskPool {
    pub fn run_once<Output, Marker>(
        &self,
        label: impl ScheduleLabel,
        system: impl IntoSystem<(), Output, Marker> + 'static + Send,
    ) -> impl Future<Output=Output>
        where Output: 'static
    {
        let (tx, mut rx) = channel::<Output>(1);
        self.0.push(OnceRunner::boxed(tx, label, system));

        async move {
            loop {
                if let Some(output) = rx.next().await {
                    return output;
                }
            }
        }
    }


    pub fn delay_frame(&self, schedule_label: impl ScheduleLabel, delay_frames: usize) -> impl Future<Output=()>
    {
        let (tx, mut rx) = channel::<()>(1);
        self.0.push(DelayRunner::boxed(tx, schedule_label, delay_frames));

        async move {
            loop {
                if rx.next().await.is_some() {
                    return;
                }
            }
        }
    }


    pub(crate) fn run_systems(
        &self,
        schedule_label: impl ScheduleLabel,
        world: &mut World,
    ) {
        let mut systems = self.0.lock().unwrap();
        let mut next_systems = Vec::with_capacity(systems.len());
        while let Some(mut system) = systems.pop() {
            if !system.should_run(&schedule_label) {
                next_systems.push(system);
                continue;
            }
            if !system.run(world).finished() {
                next_systems.push(system);
            }
        }
        *systems = next_systems;
    }
}


unsafe impl Send for TaskPool {}

unsafe impl Sync for TaskPool {}