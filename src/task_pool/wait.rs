use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Event, EventReader, IntoSystem};
use futures::StreamExt;

use crate::runner::maybe::MaybeOutputRunner;
use crate::task_pool::TaskPool;

impl TaskPool {
    // pub fn wait_output<Output: 'static, Marker>(
    //     &self,
    //     schedule_label: impl ScheduleLabel + Clone,
    //     system: impl IntoSystem<(), Option<Output>, Marker> + 'static + Send,
    // ) -> impl Future<Output=Output> {
    //     let (tx, mut rx) = futures::channel::mpsc::channel::<Option<Output>>(10);
    //     self.0.insert(MaybeOutputRunner::boxed(tx, schedule_label, system));
    //
    //     async move {
    //         loop {
    //             if let Some(output) = rx.next().await.and_then(|output| output) {
    //                 return output;
    //             }
    //         }
    //     }
    // }
    //
    //
    // #[inline]
    // pub fn wait_event<E: Event + Clone>(&self, schedule_label: impl ScheduleLabel + Clone) -> impl Future<Output=E> {
    //     self.wait_output(schedule_label, |mut er: EventReader<E>| {
    //         er.iter().next().cloned()
    //     })
    // }
}