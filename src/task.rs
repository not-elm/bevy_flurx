use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Component, Event, EventReader, IntoSystem, World};
use futures::channel::mpsc::channel;
use futures::StreamExt;

use crate::runner::delay::DelayRunner;
use crate::runner::maybe::MaybeOutputRunner;
use crate::runner::once::OnceRunner;
use crate::runner::Runners;
use crate::runner::until::UntilRunner;

#[derive(Default, Component, Clone)]
pub struct TaskPool(Runners);


impl TaskPool {
    pub fn once<Output, Marker>(
        &self,
        label: impl ScheduleLabel,
        system: impl IntoSystem<(), Output, Marker> + 'static + Send,
    ) -> impl Future<Output=Output>
        where Output: 'static
    {
        let (tx, mut rx) = channel::<Output>(1);
        self.0.push(OnceRunner::boxed(tx, label, system));

        async move {
            loop {
                if let Some(output) = rx.next().await {
                    return output;
                }
            }
        }
    }


    pub fn until<Marker>(
        &self,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), bool, Marker> + 'static + Send,
    ) -> impl Future<Output=()> {
        let (tx, mut rx) = channel::<bool>(1);
        self.0.push(UntilRunner::boxed(tx, schedule_label, system));

        async move {
            loop {
                if rx.next().await.is_some_and(|finished| finished) {
                    return;
                }
            }
        }
    }


    pub fn delay_frame(&self, schedule_label: impl ScheduleLabel, delay_frames: usize) -> impl Future<Output=()>
    {
        let (tx, mut rx) = channel::<()>(1);
        self.0.push(DelayRunner::boxed(tx, schedule_label, delay_frames));

        async move {
            loop {
                if rx.next().await.is_some() {
                    return;
                }
            }
        }
    }


    pub fn wait_event<E: Event>(&self, schedule_label: impl ScheduleLabel + Clone) -> impl Future<Output=()> {
        self.until(schedule_label, |er: EventReader<E>| {
            !er.is_empty()
        })
    }


    pub fn until_come_event<E: Event + Clone>(&self, schedule_label: impl ScheduleLabel + Clone) -> impl Future<Output=E> {
        let (tx, mut rx) = channel::<Option<E>>(10);
        self.0.push(MaybeOutputRunner::boxed(tx, schedule_label, |mut er: EventReader<E>| {
            er.iter().next().cloned()
        }));

        async move {
            loop {
                if let Some(event) = rx.next().await.and_then(|output| output) {
                    return event;
                }
            }
        }
    }

    pub(crate) fn run_systems(
        &self,
        schedule_label: impl ScheduleLabel,
        world: &mut World,
    ) {
        let mut systems = self.0.lock().unwrap();
        let mut next_systems = Vec::with_capacity(systems.len());
        while let Some(mut system) = systems.pop() {
            if !system.should_run(&schedule_label) {
                next_systems.push(system);
                continue;
            }
            if system.run(world).finished() {
                continue;
            }
            next_systems.push(system);
        }

        *systems = next_systems;
    }
}


unsafe impl Send for TaskPool {}

unsafe impl Sync for TaskPool {}