use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Deref, DerefMut};
use bevy::tasks::Task;

use crate::runner::{IntoAsyncSystem, Runners};

#[derive(Component, Deref, DerefMut)]
pub struct BevTaskHandle(pub(crate) Task<()>);

#[derive(Default, Clone)]
pub struct BevTaskCommands(pub(crate) Runners);


impl BevTaskCommands {
    pub fn spawn<Out: 'static>(
        &self,
        schedule_label: impl ScheduleLabel,
        into_async_system: impl IntoAsyncSystem<Out>,
    ) -> impl Future<Output=Out> {
        let (runner, future) = into_async_system.into_parts();
        self.0.insert(Box::new(schedule_label), runner);
        future
    }
}


