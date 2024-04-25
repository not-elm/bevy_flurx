//! This library provides a mechanism for more sequential description of delays, character movement,
//! waiting for user input, and other state waits.
//!
//! [`Reactor`] can be used partially. 
//! This means there is no need to rewrite existing applications to use this library.
//! And I recommend using it partially. 
//! This is because the system that runs [`Reactor`] and the systems that are run by [`Reactor`] run on the main thread.
//! (Please check [`Switch`](crate::prelude::Switch) for multi thread operation.)


#![allow(clippy::type_complexity)]

use bevy::app::{App, Last, MainScheduleOrder, Plugin, PostStartup};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Entity, QueryState, Without, World};

use crate::reactor::{Initialized, Reactor};
use crate::world_ptr::WorldPtr;

pub mod extension;
pub mod task;
pub mod action;
pub mod runner;

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::{
        action::*,
        action::Map,
        action::omit::*,
        action::pipe::Pipe,
        action::record::*,
        action::record::extension::*,
        action::Remake,
        action::seed::ActionSeed,
        action::sequence::Then,
        action::switch::*,
        action::through::{through, Through},
        action::wait::Either,
        extension::ScheduleReactor,
        FlurxPlugin,
        reactor::Reactor,
        runner::*,
        task::ReactiveTask,
    };
}

mod world_ptr;
mod reactor;
mod selector;

#[cfg(test)]
mod test_util;

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
    enum Status {
        Finished,
        Initialized,
    }

    let world_ptr = WorldPtr::new(world);
    let mut entities = Vec::with_capacity(reactors.iter(world).len());
    for (entity, mut reactor, initialized) in reactors.iter_mut(world) {
        if initialized.is_none() {
            if reactor.run_sync(world_ptr) || reactor.run_sync(world_ptr) {
                entities.push((entity, Status::Finished));
            } else {
                entities.push((entity, Status::Initialized));
            }
        } else if reactor.run_sync(world_ptr) {
            entities.push((entity, Status::Finished));
        }
    }
    
    for (entity, status) in entities {
        match status {
            Status::Finished => {
                world.entity_mut(entity).despawn_recursive();
            }
            Status::Initialized => {
                world.entity_mut(entity).insert(Initialized);
            }
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
    use crate::prelude::{ActionSeed, Record, RecordExtension};

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

    #[derive(Default, Eq, PartialEq, Copy, Clone)]
    pub struct TestAct;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct NumAct(pub usize);

    pub fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            BevyTestHelperPlugin,
            FlurxPlugin,
            InputPlugin
        ));
        app.add_record_events::<NumAct>();
        app.add_record_events::<TestAct>();
        app.init_resource::<Record<TestAct>>();
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