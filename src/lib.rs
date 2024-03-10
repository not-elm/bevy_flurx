#![allow(clippy::type_complexity)]


use bevy::app::{App, Plugin, Update};
use bevy::prelude::World;
use futures_lite::future::block_on;

use crate::scheduler::BevyScheduler;
use crate::store::WorldPointer;

mod store;
mod commands;
mod task;
mod scheduler;

/// Provides the async systems.
pub struct AsyncSystemPlugin;


impl Plugin for AsyncSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_non_send_resource::<BevyScheduler>()
            .add_systems(Update, run_bevy_task_scheduler);
    }
}


pub fn run_bevy_task_scheduler(
    world: &mut World
) {
    let world_ptr = WorldPointer::new(world);
    if let Some(mut scheduler) = world.get_non_send_resource_mut::<BevyScheduler>() {
        block_on(scheduler.run(world_ptr));
    }
}