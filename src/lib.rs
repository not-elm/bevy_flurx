#![allow(clippy::type_complexity)]


use async_compat::CompatExt;
use bevy::app::{App, First, MainScheduleOrder, Plugin};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::World;

use crate::scheduler::TaskScheduler;
use crate::world_ptr::WorldPtr;

pub mod world_ptr;
pub mod task;
pub mod scheduler;
pub mod selector;
mod extension;


/// Provides the async systems.
pub struct FlurxPlugin;


impl Plugin for FlurxPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_non_send_resource::<TaskScheduler>()
            .init_schedule(AfterFirst);
        app
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(First, AfterFirst);

        app.add_systems(AfterFirst, run_scheduler);
    }
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct AfterFirst;

fn run_scheduler(
    world: &mut World
) {
    if let Some(mut scheduler) = world.remove_non_send_resource::<TaskScheduler>() {
        pollster::block_on(scheduler.run(WorldPtr::new(world)).compat());
        world.insert_non_send_resource(scheduler);
    }
}

