//! This library provides a mechanism for more sequential description of delays, character movement,
//! waiting for user input, and other state waits.
//!
//! [`NativeReactor`] can be used partially.
//! This means there is no need to rewrite existing applications to use this library.
//! And I recommend using it partially.
//! This is because the system that runs [`NativeReactor`] and the systems that are run by [`NativeReactor`] run on the main thread.
//! (Please check [`Switch`](crate::prelude::Switch) for multi thread operation.)

#![allow(clippy::type_complexity)]
use crate::reactor::NativeReactor;
use crate::runner::CallCancellationHandlers;
use crate::world_ptr::WorldPtr;
use bevy::app::{App, Last, Plugin, PostStartup};
use bevy::ecs::system::SystemState;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Entity, EventReader, IntoSystemConfigs, QueryState, World};

pub mod action;
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
        reactor::Reactor,
        runner::*,
        task::ReactorTask,
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
        app
            .add_event::<CallCancellationHandlers>()
            .add_systems(PostStartup, initialize_reactors)
            .add_systems(Last, (
                call_cancel_handlers.run_if(bevy::prelude::on_event::<CallCancellationHandlers>),
                run_reactors,
            ));
    }
}

fn initialize_reactors(world: &mut World, reactors: &mut QueryState<&mut NativeReactor>) {
    let world_ptr = WorldPtr::new(world);
    for mut reactor in reactors.iter_mut(world) {
        if reactor.initialized {
            continue;
        }
        reactor.initialized = true;
        reactor.run_sync(world_ptr.clone());
    }
}

fn call_cancel_handlers(
    world: &mut World,
) {
    let mut event_system_state = SystemState::<EventReader<CallCancellationHandlers>>::new(world);
    let handlers = event_system_state
        .get_mut(world)
        .read()
        .flat_map(|handler| handler.0.0.values().copied())
        .collect::<Vec<_>>();
    for handler in handlers {
        handler(world);
    }
}

fn run_reactors(world: &mut World, reactors: &mut QueryState<(Entity, &mut NativeReactor)>) {
    let world_ptr = WorldPtr::new(world);
    let mut entities = Vec::with_capacity(reactors.iter(world).len());
    //  let mut entities = Vec::new();
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
    use crate::action::once;
    use crate::prelude::{ActionSeed, Record, RecordExtension};
    use crate::FlurxPlugin;
    use bevy::app::{App, AppExit};
    use bevy::ecs::event::EventCursor;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;
    use bevy::prelude::{Event, EventReader, FrameCountPlugin, ResMut, Resource};
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimePlugin;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::BevyTestHelperPlugin;

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
