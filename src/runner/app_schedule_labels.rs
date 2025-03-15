use bevy::ecs::schedule::InternedScheduleLabel;
use bevy::prelude::{Deref, DerefMut, Resource, Schedules};
use bevy::utils::HashSet;


/// Manages the schedules registered in the main app.
///
/// This resource is used to confirm whether the schedule label of the system that the [`BoxedRunner`](crate::prelude::BoxedRunner)
/// is running is already registered in the [`Schedules`].
///
/// The reason why it is not checked directly from [`Schedules`] is that when the scheduler is executed,
/// the target scheduler is temporarily removed from [`Schedules`].
#[derive(Resource, Debug, Deref, DerefMut)]
pub(crate) struct AppScheduleLabels(pub HashSet<InternedScheduleLabel>);

impl AppScheduleLabels {
    #[inline]
    pub fn current_running_on_target_schedule(
        &self,
        target: InternedScheduleLabel,
        schedules: &Schedules,
    ) -> bool {
        self.0.contains(&target) && !schedules.contains(target)
    }
}