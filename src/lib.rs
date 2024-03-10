#![allow(clippy::type_complexity)]


use bevy::app::{App, Plugin, Update};
use bevy::prelude::World;
use futures_lite::future::block_on;

use crate::scheduler::TaskScheduler;
use crate::world_ptr::WorldPtr;

pub mod world_ptr;
pub mod task;
pub mod scheduler;
pub mod selector;

/// Provides the async systems.
pub struct AsyncSystemPlugin;


impl Plugin for AsyncSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_non_send_resource::<TaskScheduler>()
            .add_systems(Update, run_bevy_task_scheduler);
    }
}


pub fn run_bevy_task_scheduler(
    world: &mut World
) {
    let world_ptr = WorldPtr::new(world);
    if let Some(mut scheduler) = world.get_non_send_resource_mut::<TaskScheduler>() {
        block_on(scheduler.run(world_ptr));
    }
}
