//! This library provides a mechanism for more sequential description of delays, character movement,
//! waiting for user input, and other state waits.
//!
//! [`Reactor`] can be used partially.
//! This means there is no need to rewrite existing applications to use this library.
//! And I recommend using it partially.
//! This is because the system that runs [`Reactor`] and the systems that are run by [`Reactor`] run on the main thread.
//! (Please check [`Switch`](crate::prelude::Switch) for multi thread operation.)

#![allow(clippy::type_complexity)]
use crate::reactor::{Reactor, ReactorId};
use crate::runner::CancellationToken;
use crate::world_ptr::WorldPtr;
use bevy::app::{App, Last, Plugin, PostStartup, PreUpdate};
use bevy::ecs::system::SystemState;
use bevy::ecs::world::DeferredWorld;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{on_event, Commands, DespawnRecursive, Entity, Event, EventReader, IntoSystemConfigs, Mut, OnRemove, Query, QueryState, Reflect, Resource, Trigger, World};
use bevy::utils::HashMap;

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
        app
            .add_event::<ReactorId>()
            .register_type::<ReactorId>()
            .add_event::<CallCancelHandlers>()
            .register_type::<CallCancelHandlers>()
            .init_resource::<CancelHandlers>()
            .add_systems(PostStartup, initialize_reactors)
            .add_systems(PreUpdate, call_cancel_handlers.run_if(on_event::<CallCancelHandlers>))
            .add_systems(Last, run_reactors)
            .add_observer(cancel_reactors);

        app
            .world_mut()
            .register_component_hooks::<Reactor>()
            .on_add(|mut world: DeferredWorld, entity, _| {
                let reactor_id = world.get::<Reactor>(entity).unwrap().id;
                world.resource_mut::<CancelHandlers>().0.insert(reactor_id, CancellationToken::default());
            })
            .on_remove(|mut world: DeferredWorld, entity, _| {
                let reactor_id = world.get::<Reactor>(entity).unwrap().id;
                world.send_event::<CallCancelHandlers>(CallCancelHandlers(reactor_id));
            });
    }
}

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

fn cancel_reactors(
    trigger: Trigger<ReactorId>,
    mut commands: Commands,
    reactors: Query<(Entity, &Reactor)>,
) {
    let id = trigger.event();
    let Some((entity, _)) = reactors.iter().find(|(_, reactor)| &reactor.id == id) else {
        return;
    };
    commands.entity(entity).despawn();
}

#[derive(Default, Reflect, Eq, PartialEq, Hash, Copy, Clone, Event)]
struct CallCancelHandlers(pub ReactorId);
#[derive(Default, Resource)]
struct CancelHandlers(pub HashMap<ReactorId, CancellationToken>);

fn call_cancel_handlers(
    world: &mut World,
) {
    let mut event_system_state = SystemState::<EventReader<CallCancelHandlers>>::new(world);
    let ids = event_system_state
        .get_mut(world)
        .read()
        .copied()
        .collect::<Vec<_>>();
    world.resource_scope(move |world, mut handlers: Mut<CancelHandlers>| {
        for CallCancelHandlers(id) in ids {
            if let Some(mut handler) = handlers.0.remove(&id) {
                handler.call_cancel_handles(world);
            }
        }
    });
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
