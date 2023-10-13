use bevy::app::{App, First, Plugin};
use bevy::prelude::{Commands, Entity, Query};
use futures_lite::future::block_on;

use crate::ext::BevTask;

pub mod task_pool;
pub mod ext;
pub mod runner;


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
    mut processes: Query<(Entity, &mut BevTask)>,
) {
    for (entity, mut process) in processes.iter_mut() {
        if block_on(futures_lite::future::poll_once(&mut process.0)).is_some() {
            commands.entity(entity).despawn();
        }
    }
}

#[macro_use]
pub(crate) mod inner_macros {
    macro_rules! run_tasks {
        ($schedule_label: expr) => {
            move |world: &mut bevy::prelude::World| {
                let tasks: Vec<crate::task_pool::TaskPool> = world
                    .query::<&crate::task_pool::TaskPool>()
                    .iter(world)
                    .cloned()
                    .collect();

                for task in tasks.iter(){
                    task.run_systems(Box::new($schedule_label), world);
                }
            }
        };
    }

    pub(crate) use run_tasks;
}
