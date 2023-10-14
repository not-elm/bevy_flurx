#![allow(clippy::type_complexity)]

use bevy::app::{App, First, Plugin};
use bevy::prelude::{Changed, Commands, Entity, Query, World};
use futures_lite::future::block_on;

use crate::async_commands::TaskHandle;
use crate::runner::thread_pool::TaskPoolSystemSetups;

pub mod async_commands;
pub mod ext;
pub mod runner;


pub mod prelude {
    pub use crate::{
        async_commands::{AsyncCommands, TaskHandle},
        AsyncSystemPlugin,
        runner::main_thread::{
            BoxedMainThreadExecutor,
            IntoMainThreadExecutor,
            // delay::Delay,
            MainThreadExecutable,
            // once::OnceOnMain,
            repeat::Repeat,
            wait::Wait,
        },
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
        use crate::inner_macros::run_tasks;

        {
            use bevy::prelude::IntoSystemConfigs;
            app.add_systems(First, (
                remove_finished_processes,
                schedule_setups,
                #[cfg(feature = "first")]
                run_tasks!(bevy::app::First)
            ).chain());
        }

        #[cfg(feature = "pre_update")]
        { app.add_systems(bevy::app::PreUpdate, run_tasks!(bevy::app::PreUpdate)); }

        #[cfg(feature = "state_transition")]
        { app.add_systems(bevy::app::StateTransition, run_tasks!(bevy::app::StateTransition)); }

        #[cfg(feature = "fixed_update")]
        {
            app
                .add_systems(bevy::app::RunFixedUpdateLoop, run_tasks!(bevy::app::RunFixedUpdateLoop))
                .add_systems(bevy::app::FixedUpdate, run_tasks!(bevy::app::FixedUpdate));
        }

        #[cfg(feature = "update")]
        { app.add_systems(bevy::app::Update, run_tasks!(bevy::app::Update)); }

        #[cfg(feature = "post_update")]
        { app.add_systems(bevy::app::PostUpdate, run_tasks!(bevy::app::PostUpdate)); }

        #[cfg(feature = "last")]
        { app.add_systems(bevy::app::Last, run_tasks!(bevy::app::Last)); }
    }
}


fn schedule_setups(
    world: &mut World
) {

    let setups: Vec<TaskPoolSystemSetups> = world
        .query_filtered::<&TaskPoolSystemSetups, Changed<TaskPoolSystemSetups>>()
        .iter(world)
        .cloned()
        .collect();

    for setup in setups.iter() {

        setup.initialize_systems(world);
    }
}

fn remove_finished_processes(
    mut commands: Commands,
    mut task_handles: Query<(Entity, &mut TaskHandle)>,
) {
    for (entity, mut task) in task_handles.iter_mut() {
        if block_on(futures_lite::future::poll_once(&mut task.0)).is_some() {
            commands.entity(entity).despawn();
        }
    }
}

#[macro_use]
pub(crate) mod inner_macros {
    macro_rules! run_tasks {
        ($schedule_label: expr) => {
             move |world: &mut bevy::prelude::World| {
                let runners: Vec<crate::runner::main_thread::MainThreadExecutors> = world
                    .query::<&crate::runner::main_thread::MainThreadExecutors>()
                    .iter(world)
                    .cloned()
                    .collect();

                let schedule_label: Box<dyn bevy::ecs::schedule::ScheduleLabel> = Box::new($schedule_label);
                for runner in runners.iter(){
                    runner.run_systems(&schedule_label, world);
                }
            }
        };
    }

    pub(crate) use run_tasks;
}


#[cfg(test)]
pub(crate) mod test_util {
    use bevy::app::App;
    use bevy::core::TaskPoolPlugin;
    use bevy::prelude::Event;
    use bevy::time::TimePlugin;

    use crate::AsyncSystemPlugin;

    #[derive(Event, Copy, Clone, Debug, Eq, PartialEq)]
    pub struct TestEvent;


    pub fn new_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            TimePlugin,
            AsyncSystemPlugin
        ));
        app.add_event::<TestEvent>();
        app
    }
}