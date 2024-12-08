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
use bevy::prelude::{Entity, QueryState, World};

use crate::reactor::Reactor;
use crate::world_ptr::WorldPtr;

pub mod action;
pub mod extension;
pub mod runner;
pub mod task;

#[allow(missing_docs)]
pub mod prelude {
    #[cfg(feature = "effect")]
    pub use crate::action::effect::AsyncFunctor;
    #[cfg(feature = "record")]
    pub use crate::action::record::{
        extension::{RecordExtension, RequestRedo, RequestUndo},
        EditRecordResult, Record, Redo, RedoAction, Rollback, Track, Undo, UndoRedoInProgress,
    };
    pub use crate::{
        action::omit::*,
        action::pipe::Pipe,
        action::seed::ActionSeed,
        action::sequence::Then,
        action::switch::*,
        action::through::{through, Through},
        action::wait::Either,
        action::Map,
        action::Remake,
        action::*,
        extension::*,
        reactor::Reactor,
        runner::*,
        task::ReactiveTask,
        FlurxPlugin,
    };
}

mod reactor;
mod selector;
mod world_ptr;

/// Define utilities for testing.
#[cfg(test)]
mod test_util;

/// Provides the async systems.
pub struct FlurxPlugin;

impl Plugin for FlurxPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app.init_schedule(RunReactor)
            .add_systems(PostStartup, initialize_reactors)
            .add_systems(RunReactor, run_reactors);
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Last, RunReactor);
    }
}

/// Runs after the [`Last`](bevy::prelude::Last).
#[derive(ScheduleLabel, Eq, PartialEq, Debug, Copy, Clone, Hash)]
struct RunReactor;

fn initialize_reactors(world: &mut World, reactors: &mut QueryState<&mut Reactor>) {
    let world_ptr = WorldPtr::new(world);
    for mut reactor in reactors.iter_mut(world) {
        if reactor.initialized {
            continue;
        }
        reactor.initialized = true;
        reactor.run_sync(world_ptr);
    }
}

fn run_reactors(world: &mut World, reactors: &mut QueryState<(Entity, &mut Reactor)>) {
    let world_ptr = WorldPtr::new(world);
    let mut entities = Vec::with_capacity(reactors.iter(world).len());
    for (entity, mut reactor) in reactors.iter_mut(world) {
        if !reactor.initialized {
            if reactor.run_sync(world_ptr) || reactor.run_sync(world_ptr) {
                entities.push(entity);
            } else {
                reactor.initialized = true;
            }
        } else if reactor.run_sync(world_ptr) {
            entities.push(entity);
        }
    }
    for entity in entities {
        world.entity_mut(entity).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit};

    use bevy::ecs::event::EventCursor;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;
    use bevy::prelude::{Event, EventReader, FrameCountPlugin, ResMut, Resource};
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimePlugin;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::BevyTestHelperPlugin;

    use crate::action::once;
    use crate::prelude::{ActionSeed, Record, RecordExtension};
    use crate::FlurxPlugin;

    pub fn exit_reader() -> EventCursor<AppExit> {
        EventCursor::<AppExit>::default()
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
            InputPlugin,
            TimePlugin,
            FrameCountPlugin,
            StatesPlugin,
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
        app.world_mut()
            .run_system_once(|mut e: EventReader<E>| {
                let came = !e.is_empty();
                e.clear();
                came
            })
            .expect("Failed to run system `came_event`")
    }
}
