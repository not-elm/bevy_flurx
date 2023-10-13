use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Event, EventReader, IntoSystem};
use futures::StreamExt;

use crate::runner::until::UntilRunner;
use crate::task_pool::TaskPool;

impl TaskPool {
    pub fn until<Marker>(
        &self,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), bool, Marker> + 'static + Send,
    ) -> impl Future<Output=()> {
        let (tx, mut rx) = futures::channel::mpsc::channel::<bool>(1);
        self.0.push(UntilRunner::boxed(tx, schedule_label, system));

        async move {
            loop {
                if rx.next().await.is_some_and(|finished| finished) {
                    return;
                }
            }
        }
    }

    pub fn until_come_event<E: Event>(&self, schedule_label: impl ScheduleLabel + Clone) -> impl Future<Output=()> {
        self.until(schedule_label, |er: EventReader<E>| {
            !er.is_empty()
        })
    }
}