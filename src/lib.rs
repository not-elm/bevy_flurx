use bevy::app::{App, First, Last, Plugin, PostUpdate, PreUpdate, RunFixedUpdateLoop, StateTransition, Update};
use bevy::prelude::{Commands, Entity, IntoSystemConfigs, Query};

use crate::ext::ProcessReceiver;

pub mod task_pool;
pub mod ext;
pub mod runner;


pub struct BevTaskPlugin;


impl Plugin for BevTaskPlugin {
    fn build(&self, app: &mut App) {
        use crate::inner_macros::run_tasks;

        app
            .add_systems(First, (
                remove_finished_processes,
                run_tasks!(First)
            ).chain())
            .add_systems(PreUpdate, run_tasks!(PreUpdate))
            .add_systems(StateTransition, run_tasks!(StateTransition))
            .add_systems(RunFixedUpdateLoop, run_tasks!(RunFixedUpdateLoop))
            .add_systems(Update, run_tasks!(Update))
            .add_systems(PostUpdate, run_tasks!(PostUpdate))
            .add_systems(Last, run_tasks!(Last));
    }
}


fn remove_finished_processes(
    mut commands: Commands,
    mut processes: Query<(Entity, &mut ProcessReceiver)>,
) {
    for (entity, mut process) in processes.iter_mut() {
        if process.finished() {
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
