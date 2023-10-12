use bevy::app::{App, First, Last, Plugin, PostUpdate, PreUpdate, RunFixedUpdateLoop, StateTransition, Update};
use bevy::prelude::{NonSendMut, IntoSystemConfigs};

use crate::task::AsyncSystemManager;

pub mod task;




macro_rules! run_tasks {
    ($schedule_label: expr) => {
        |world: &mut bevy::prelude::World| {
            let Some(mut task_manager) = world.remove_non_send_resource::<AsyncSystemManager>() else { return; };
            task_manager.update($schedule_label, world);
            world.insert_non_send_resource(task_manager);
        }
    };
}

pub struct AsyncSystemPlugin;


impl Plugin for AsyncSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_non_send_resource::<AsyncSystemManager>()
            .add_systems(First, (
                remove_finished_tasks,
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


fn remove_finished_tasks(
    mut manager: NonSendMut<AsyncSystemManager>
) {
    manager.remove_finished_tasks();
}

