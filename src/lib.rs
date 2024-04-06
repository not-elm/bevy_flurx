//! ECS consists of a group of very small independent systems.
//! Although we find this architecture very nice, it takes a lot of effort to implement system interaction,
//! especially state monitoring, and this problem becomes more pronounced as the application gets larger.
//! For example, how can I play a sound effect just before the character starts moving,
//! and then not accept any input until the character stop movement and 
//! the sound effect finished playing?  
//!
//! If only one process is waiting, simply use an event reader,
//! but if more than one, you will probably need to create a structure to manage the status of multiple processes.
//!
//! This problem is common in event-driven applications, and is often solved with async-await;
//! This library also resolve this using async-await.
//! Specifically, I use an asynchronous processing flow that I call `Reactor`.
//!
//! `Reactor` can be used partially. 
//! This means there is no need to rewrite existing applications to use this library.
//! And I recommend using it partially. 
//! This is because the system that runs `Reactor` and the systems that are run by `Reactor` run on the main thread.
//! (Multithreading support is under consideration.)


#![allow(clippy::type_complexity)]

use bevy::app::{App, First, Last, MainScheduleOrder, Plugin, PostStartup};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Added, Entity, Without, World};

use crate::prelude::Flurx;
use crate::scheduler::Initialized;
use crate::world_ptr::WorldPtr;

pub mod extension;
pub mod task;
pub mod action;

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::{
        action::*,
        action::pipe::Pipe,
        action::sequence::Then,
        action::switch::*,
        action::wait::either::Either,
        action::seed::{ActionSeed, Seed},
        FlurxPlugin,
        scheduler::Flurx,
        task::ReactiveTask,
    };
}

#[doc(hidden)]
pub mod private {
    pub use crate::runner::RunnerIntoAction;
}

mod world_ptr;
mod scheduler;
#[allow(missing_docs)]
mod runner;
mod selector;

/// Provides the async systems.
pub struct FlurxPlugin;

impl Plugin for FlurxPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app
            .init_schedule(AfterLast)
            .add_systems(AfterLast, run_scheduler)
            .add_systems(PostStartup, flurx_initialize)
            .add_systems(First, flurx_initialize);

        app
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Last, AfterLast);
    }
}

#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct AfterLast;

fn flurx_initialize(
    world: &mut World
) {
    let world_ptr = WorldPtr::new(world);
    let mut entities = vec![];
    for (entity, mut flurx) in world
        .query_filtered::<(Entity, &mut Flurx), (Added<Flurx>, Without<Initialized>)>()
        .iter_mut(world) {
        flurx.scheduler.run_sync(world_ptr);
        entities.push(entity);
    }

    for entity in entities {
        world.entity_mut(entity).insert(Initialized);
    }
}

fn run_scheduler(
    world: &mut World
) {
    let world_ptr = WorldPtr::new(world);
    for mut flurx in world
        .query::<&mut Flurx>()
        .iter_mut(world) {
        flurx.scheduler.run_sync(world_ptr);
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;
    use bevy::log::LogPlugin;
    use bevy::prelude::{Event, EventReader, Resource};
    use bevy_test_helper::BevyTestHelperPlugin;

    use crate::FlurxPlugin;

    pub fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            BevyTestHelperPlugin,
            LogPlugin::default(),
            FlurxPlugin,
            InputPlugin
        ));
        app
    }

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