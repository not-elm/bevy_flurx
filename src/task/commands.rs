use std::future::Future;
use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{Event, EventReader, IntoSystem, World};
use futures::channel::mpsc::channel;
use futures::StreamExt;

use crate::task::commands::runner::BoxedAsyncSystemRunner;
use crate::task::commands::runner::delay::DelayRunner;
use crate::task::commands::runner::maybe::MaybeOutputRunner;
use crate::task::commands::runner::once::OnceRunner;
use crate::task::commands::runner::until::UntilRunner;

mod runner;


#[derive(Default)]
pub struct AsyncCommands(Arc<Mutex<Vec<BoxedAsyncSystemRunner>>>);


impl AsyncCommands {
    pub fn once<Output, Marker>(
        &self,
        label: impl ScheduleLabel,
        system: impl IntoSystem<(), Output, Marker> + 'static + Send,
    ) -> impl Future<Output=Output>
        where Output: 'static
    {
        let (tx, mut rx) = channel::<Output>(1);
        self.0.lock().unwrap().push(OnceRunner::boxed(tx, label, system));

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
        self.0.lock().unwrap().push(UntilRunner::boxed(tx, schedule_label, system));

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
        self.0.lock().unwrap().push(DelayRunner::boxed(tx, schedule_label, delay_frames));

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
        self.0.lock().unwrap().push(MaybeOutputRunner::boxed(tx, schedule_label, |mut er: EventReader<E>| {
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
        schedule_label: &dyn ScheduleLabel,
        world: &mut World,
    ) {
        let mut systems = self.0.lock().unwrap();
        let mut next_systems = Vec::with_capacity(systems.len());
        while let Some(mut system) = systems.pop() {
            if !system.should_run(schedule_label) {
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

impl Clone for AsyncCommands {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

unsafe impl Send for AsyncCommands {}

unsafe impl Sync for AsyncCommands {}

