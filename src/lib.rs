//! This library offers a mechanism for more sequential descriptions of delays, character movement, waiting for user input, and other state waits.
//! [Reactor](prelude::Reactor) can be used incrementally, meaning there's no need to rewrite existing applications to incorporate it.
//! I recommend this partial usage since the system that runs [Reactor](prelude::Reactor) and the systems executed by [Reactor](prelude::Reactor) operate on the main thread.
//! For multithreaded operation, please check the  [`Switch`](prelude::Switch).

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::type_complexity)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use crate::reactor::ReactorPlugin;
use crate::runner::RunnerPlugin;
use bevy::app::{App, Plugin};

pub mod action;
pub mod runner;
pub mod task;

#[allow(missing_docs)]
pub mod prelude {
    #[cfg(feature = "record")]
    pub use crate::action::record::{
        extension::{RecordExtension, RequestRedo, RequestUndo},
        EditRecordResult, Record, Redo, RedoAction, Rollback, Track, Undo, UndoRedoInProgress,
    };
    #[cfg(feature = "side-effect")]
    pub use crate::action::side_effect::AsyncFunctor;
    pub use crate::{
        action::inspect::{inspect, Inspect},
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
        reactor::*,
        runner::*,
        task::ReactorTask,
        FlurxPlugin,
    };
}

mod reactor;
mod selector;
mod world_ptr;

mod core;
/// Define utilities for testing.
#[cfg(test)]
mod test_util;

/// Provides the async systems.
pub struct FlurxPlugin;

impl Plugin for FlurxPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app.add_plugins((ReactorPlugin, RunnerPlugin));
    }
}

#[cfg(test)]
mod tests {
    use crate::action::once;
    use crate::prelude::ActionSeed;
    use crate::FlurxPlugin;
    use bevy::app::{App, AppExit};
    use bevy::ecs::message::MessageCursor;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;
    use bevy::prelude::{Message, MessageReader, ResMut, Resource};
    use bevy::state::app::StatesPlugin;
    use bevy::MinimalPlugins;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::BevyTestHelperPlugin;

    pub fn exit_reader() -> MessageCursor<AppExit> {
        MessageCursor::<AppExit>::default()
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
            MinimalPlugins,
            InputPlugin,
            StatesPlugin,
            BevyTestHelperPlugin,
            FlurxPlugin,
        ));
        #[cfg(feature = "record")]
        {
            use crate::prelude::{Record, RecordExtension};
            app.add_record::<NumAct>();
            app.add_record::<TestAct>();
            app.init_resource::<Record<TestAct>>();
        }

        app
    }

    #[derive(Eq, PartialEq, Debug, Resource, Copy, Clone, Default)]
    pub struct TestResource;

    #[allow(unused)]
    pub fn came_event<E: Message>(app: &mut App) -> bool {
        app.world_mut()
            .run_system_once(|mut e: MessageReader<E>| {
                let came = !e.is_empty();
                e.clear();
                came
            })
            .expect("Failed to run system `came_event`")
    }
}
