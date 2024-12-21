//! This library offers a mechanism for more sequential descriptions of delays, character movement, waiting for user input, and other state waits.
//! [Reactor](prelude::Reactor) can be used incrementally, meaning there's no need to rewrite existing applications to incorporate it.
//! I recommend this partial usage since the system that runs [Reactor](prelude::Reactor) and the systems executed by [Reactor](prelude::Reactor) operate on the main thread.
//! For multithreaded operation, please check the  [`Switch`](prelude::Switch).

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::type_complexity)]

use crate::reactor::{InitializeReactorsSystemId, NativeReactor, UnInitialized};
use crate::runner::CallCancellationHandlers;
use crate::world_ptr::WorldPtr;
use bevy::app::{App, Last, Plugin};
use bevy::ecs::system::SystemState;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Entity, EventReader, IntoSystemConfigs, QueryState, With, World};

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
mod core;

/// Provides the async systems.
pub struct FlurxPlugin;

impl Plugin for FlurxPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        let system_id = app.world_mut().register_system_cached(initialize_reactors);
        app.insert_resource(InitializeReactorsSystemId(system_id));

        app
            .add_event::<CallCancellationHandlers>()
            .add_systems(Last, (
                call_cancel_handlers.run_if(bevy::prelude::on_event::<CallCancellationHandlers>),
                run_reactors,
            ));
    }
}

fn initialize_reactors(
    world: &mut World,
    reactors: &mut QueryState<&mut NativeReactor, With<UnInitialized>>,
) {
    let world_ptr = WorldPtr::new(world);
    for mut reactor in reactors.iter_mut(world) {
        reactor.run_sync(world_ptr);
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
    let mut entities = Vec::new();

    for (entity, mut reactor) in reactors.iter_mut(world) {
        if reactor.run_sync(world_ptr) {
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
    use crate::prelude::ActionSeed;
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
        #[cfg(feature = "record")]
        {
            use crate::prelude::{Record, RecordExtension};
            app.add_record_events::<NumAct>();
            app.add_record_events::<TestAct>();
            app.init_resource::<Record<TestAct>>();
        }
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
