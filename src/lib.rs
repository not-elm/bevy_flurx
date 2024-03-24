#![allow(clippy::type_complexity)]

use bevy::app::{App, Last, MainScheduleOrder, Plugin};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::World;

use crate::scheduler::ReactiveScheduler;
use crate::world_ptr::WorldPtr;

pub mod world_ptr;
pub mod task;
pub mod scheduler;
pub mod selector;
pub mod extension;


pub mod prelude{
    pub use crate::{
        FlurxPlugin,
        extension::ScheduleReactor,
        selector::condition::*
    };
}

/// Provides the async systems.
pub struct FlurxPlugin;


impl Plugin for FlurxPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_non_send_resource::<ReactiveScheduler>()
            .init_schedule(AfterLast);
        app
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Last, AfterLast);

        app.add_systems(AfterLast, run_scheduler);
    }
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct AfterLast;

fn run_scheduler(
    world: &mut World
) {
    if let Some(mut scheduler) = world.remove_non_send_resource::<ReactiveScheduler>() {
        scheduler.run_sync(WorldPtr::new(world));
        world.insert_non_send_resource(scheduler);
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Event, EventReader, Resource};

    #[derive(Eq, PartialEq, Debug, Resource, Copy, Clone, Default)]
    pub struct TestResource;

    #[allow(unused)]
    pub fn came_event<E: Event>(app: &mut App) -> bool {
        app.world.run_system_once(|mut e: EventReader<E>| {
            let came = !e.is_empty();
            e.clear();
            came
        })
    }
}