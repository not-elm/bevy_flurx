#![allow(clippy::type_complexity)]

use bevy::app::{App, First, Plugin};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Commands, Entity, Query, ResMut, Schedules};
use futures_lite::future::block_on;

use crate::async_commands::TaskHandle;
use crate::runner::AsyncScheduleCommands;

// use crate::runner::thread_pool::TaskPoolSystemSetups;

pub mod async_commands;
pub mod ext;

pub mod runner;


pub mod prelude {
    pub use crate::{
        AsyncSystemPlugin,
        async_commands::*,
        ext::spawn_async_system::SpawnAsyncSystem,
        runner::preludes::*,
    };
}

/// Adds
///
/// ```no_run
///
///
///
/// ```
pub struct AsyncSystemPlugin;


impl Plugin for AsyncSystemPlugin {
    fn build(&self, app: &mut App) {
        {
            use bevy::prelude::IntoSystemConfigs;
            app.add_systems(First, (
                remove_finished_processes,
                init_async_schedulers
            ).chain());
        }
    }
}

fn init_async_schedulers(
    mut commands: Commands,
    mut schedules: ResMut<Schedules>,
    executors_query: Query<(Entity, &AsyncScheduleCommands)>,
) {
    for (entity, executors) in executors_query.iter() {
        executors.init_schedulers(&mut commands.entity(entity), &mut schedules);
    }
}


fn remove_finished_processes(
    mut commands: Commands,
    mut task_handles: Query<(Entity, &mut TaskHandle)>,
) {
    for (entity, mut task) in task_handles.iter_mut() {
        if block_on(futures_lite::future::poll_once(&mut task.0)).is_some() {
            commands.entity(entity).despawn_recursive();
        }
    }
}


#[cfg(test)]
pub(crate) mod test_util {
    use bevy::app::App;
    use bevy::core::{FrameCountPlugin, TaskPoolPlugin};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Event, Events, State, States};
    use bevy::time::TimePlugin;

    use crate::AsyncSystemPlugin;

    #[derive(Event, Copy, Clone, Debug, Eq, PartialEq)]
    pub struct FirstEvent;


    #[derive(Event, Copy, Clone, Debug, Eq, PartialEq)]
    pub struct SecondEvent;


    #[derive(Default, Debug, Copy, Clone, Eq, PartialEq, States, Hash)]
    pub enum TestState {
        #[default]
        Empty,
        Finished,
    }

    pub fn new_app() -> App {
        let mut app = App::new();
        app.add_state::<TestState>();
        app.add_plugins((
            TaskPoolPlugin::default(),
            FrameCountPlugin,
            TimePlugin,
            AsyncSystemPlugin
        ));
        app.add_event::<FirstEvent>();
        app.add_event::<SecondEvent>();
        app
    }


    pub fn is_first_event_already_coming(app: &mut App, er: &mut ManualEventReader<FirstEvent>) -> bool {
        is_event_already_coming::<FirstEvent>(app, er)
    }


    pub fn is_second_event_already_coming(app: &mut App, er: &mut ManualEventReader<SecondEvent>) -> bool {
        is_event_already_coming::<SecondEvent>(app, er)
    }


    pub fn is_event_already_coming<E: Event>(app: &mut App, er: &mut ManualEventReader<E>) -> bool {
        let events = app.world.resource::<Events<E>>();
        let come = !er.is_empty(events);
        er.clear(events);

        come
    }

    pub fn test_state_finished(app: &mut App) -> bool {
        matches!(app.world.resource::<State<TestState>>().get(), TestState::Finished)
    }
}
