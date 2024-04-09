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

use bevy::app::{App, Last, MainScheduleOrder, Plugin, PostStartup};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Added, Commands, Entity, Query, With, Without, World};

use crate::reactor::{Initialized, Reactor};
use crate::world_ptr::WorldPtr;

pub mod extension;
pub mod task;
pub mod action;

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::{
        action::*,
        action::seed::ActionSeed,
        action::switch::*,
        action::wait::Either,
        action::sequence::Then,
        action::pipe::Pipe,
        action::through::{through, Through},
        extension::ScheduleReactor,
        FlurxPlugin,
        runner::{Output, Runner},
        reactor::Reactor,
        task::ReactiveTask,
    };
}


mod world_ptr;
mod reactor;
#[allow(missing_docs)]
mod runner;
mod selector;

/// Provides the async systems.
pub struct FlurxPlugin;

impl Plugin for FlurxPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app
            .init_schedule(RunReactor)
            .add_systems(PostStartup, (
                initialize_reactors,
                insert_initialized
            ))
            .add_systems(RunReactor, (
                run_reactors,
                insert_initialized
            ));
        app
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Last, RunReactor);
    }
}

/// Runs after the [`PostUpdate`](bevy::prelude::PostUpdate). 
#[derive(ScheduleLabel, Eq, PartialEq, Debug, Copy, Clone, Hash)]
struct RunReactor;

fn initialize_reactors(
    world: &mut World
) {
    let world_ptr = WorldPtr::new(world);
    for mut reactor in world
        .query_filtered::<&mut Reactor, (Added<Reactor>, Without<Initialized>)>()
        .iter_mut(world) {
        reactor.scheduler.run_sync(world_ptr);
    }
}

fn run_reactors(
    world: &mut World
) {
    let world_ptr = WorldPtr::new(world);
    for (mut reactor, initialized) in world
        .query::<(&mut Reactor, Option<&Initialized>)>()
        .iter_mut(world) {
        reactor.scheduler.run_sync(world_ptr);
        if initialized.is_none() {
            reactor.scheduler.run_sync(world_ptr);
        }
    }
}

fn insert_initialized(
    mut commands: Commands,
    reactors: Query<Entity, (With<Reactor>, Without<Initialized>)>,
) {
    for entity in reactors.iter() {
        commands.entity(entity).insert(Initialized);
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;
    use bevy::prelude::{Event, EventReader, Resource};
    use bevy_test_helper::BevyTestHelperPlugin;

    use crate::FlurxPlugin;

    pub fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            BevyTestHelperPlugin,
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