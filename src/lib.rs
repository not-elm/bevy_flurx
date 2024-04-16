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
use bevy::prelude::{Entity, QueryState, Without, World};

use crate::reactor::{Initialized, Reactor};
use crate::world_ptr::WorldPtr;

pub mod extension;
pub mod task;
pub mod action;

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::{
        action::*,
        action::{Omit, OmitInput, OmitOutput},
        action::Map,
        action::pipe::Pipe,
        action::Remake,
        action::seed::ActionSeed,
        action::sequence::Then,
        action::switch::*,
        action::through::{through, Through},
        action::wait::Either,
        action::undo::Undo,
        action::redo::Redo,
        action::history::HistoryStore,
        extension::ScheduleReactor,
        FlurxPlugin,
        reactor::Reactor,
        runner::{Output, Runner},
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
            .add_systems(PostStartup, initialize_reactors)
            .add_systems(RunReactor, run_reactors);
        app
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Last, RunReactor);
    }
}

/// Runs after the [`Last`](bevy::prelude::Last).
#[derive(ScheduleLabel, Eq, PartialEq, Debug, Copy, Clone, Hash)]
struct RunReactor;

fn initialize_reactors(
    world: &mut World,
    reactors: &mut QueryState<(Entity, &mut Reactor), Without<Initialized>>,
) {
    let world_ptr = WorldPtr::new(world);
    for (entity, mut reactor) in reactors.iter_mut(world) {
        world_ptr.as_mut().entity_mut(entity).insert(Initialized);
        reactor.run_sync(world_ptr);
    }
}

fn run_reactors(
    world: &mut World,
    reactors: &mut QueryState<(Entity, &mut Reactor, Option<&Initialized>)>,
) {
    let world_ptr = WorldPtr::new(world);
    for (entity, mut reactor, initialized) in reactors.iter_mut(world) {
        reactor.run_sync(world_ptr);
        if initialized.is_none() {
            world_ptr.as_mut().entity_mut(entity).insert(Initialized);
            reactor.run_sync(world_ptr);
        }
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit};
    use bevy::ecs::event::ManualEventReader;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;
    use bevy::prelude::{Event, EventReader, ResMut, Resource};
    use bevy_test_helper::BevyTestHelperPlugin;
    use bevy_test_helper::resource::count::Count;

    use crate::action::once;
    use crate::FlurxPlugin;
    use crate::prelude::ActionSeed;

    pub fn exit_reader() -> ManualEventReader<AppExit> {
        ManualEventReader::<AppExit>::default()
    }

    pub fn increment_count() -> ActionSeed {
        once::run(|mut count: ResMut<Count>| {
            count.increment();
        })
    }

    #[allow(unused)]
    pub fn decrement_count() -> ActionSeed {
        once::run(|mut count: ResMut<Count>| {
            count.decrement();
        })
    }

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