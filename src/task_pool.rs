use std::future::Future;

use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::prelude::{Component, World};

use crate::runner::{AsyncSystem, Runners};

mod wait;
mod until;

#[derive(Default, Component, Clone)]
pub struct TaskPool(Runners);


impl TaskPool {
    pub fn spawn<Out: 'static>(
        &self,
        schedule_label: impl ScheduleLabel,
        config: impl AsyncSystem<Out>,
    ) -> impl Future<Output=Out> {
        let (runner, receiver) = config.split();
        self.0.insert(Box::new(schedule_label), runner);
        receiver
    }


    pub(crate) fn run_systems(
        &self,
        schedule_label: BoxedScheduleLabel,
        world: &mut World,
    ) {
        let mut map = self.0.lock().unwrap();
        let Some(systems) = map.get_mut(&schedule_label) else { return; };
        let mut next_systems = Vec::with_capacity(systems.len());
        while let Some(mut system) = systems.pop() {
            if !system.run(world).finished() {
                next_systems.push(system);
            }
        }
        *systems = next_systems;
    }
}


unsafe impl Send for TaskPool {}

unsafe impl Sync for TaskPool {}