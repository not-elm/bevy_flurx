use bevy::app::{App, First, Plugin};
use bevy::prelude::{Commands, Entity, Query};
use futures_lite::future::block_on;
use crate::task::TaskHandle;


pub mod task;
pub mod ext;
pub mod runner;


pub mod prelude{
    pub use crate::{
        BevTaskPlugin,
        task::{TaskHandle, AsyncCommands},
        runner::{
            IntoAsyncSystemRunner,
            AsyncSystemRunnable,
            BoxedAsyncSystemRunner,
            delay::Delay,
            once::Once,
            repeat::Repeat,
            wait::Wait
        }
    };
}

/// Adds
///
/// ```no_run
///
///
///
/// ```
pub struct BevTaskPlugin;


impl Plugin for BevTaskPlugin {
    fn build(&self, app: &mut App) {
        use crate::inner_macros::run_tasks;

        #[cfg(feature = "first")]
        {
            use bevy::prelude::IntoSystemConfigs;
            app.add_systems(First, (
                remove_finished_processes,
                run_tasks!(bevy::app::First)
            ).chain());
        }

        #[cfg(not(feature = "first"))]
        { app.add_systems(First, remove_finished_processes); }

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
                let runners: Vec<crate::runner::Runners> = world
                    .query::<&crate::runner::Runners>()
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
