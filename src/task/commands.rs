use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::StreamExt;

use crate::task::commands::runner::BoxedAsyncSystemRunner;
use crate::task::commands::runner::delay::DelayRunner;
use crate::task::commands::runner::once::OnceRunner;
use crate::task::commands::runner::until::AsyncSystemUntilRunner;

mod runner;


#[derive(Default)]
pub struct AsyncCommands(Arc<Mutex<Vec<BoxedAsyncSystemRunner>>>);


impl AsyncCommands {
    pub async fn once<Output, Marker>(
        &mut self,
        label: impl ScheduleLabel,
        system: impl IntoSystem<(), Output, Marker> + 'static + Send,
    ) -> Output
        where Output: 'static
    {
        let (tx, mut rx): (Sender<Output>, Receiver<Output>) = channel::<Output>(1);
        self.0.lock().unwrap().push(OnceRunner::boxed(tx, label, system));

        loop {
            if let Some(output) = rx.next().await {
                return output;
            }
        }
    }


    pub async fn until<Marker>(
        &mut self,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), bool, Marker> + 'static + Send,
    ) {
        let (tx, mut rx) = channel::<bool>(1);
        self.0.lock().unwrap().push(AsyncSystemUntilRunner::boxed(tx, schedule_label, system));

        loop {
            if rx.next().await.is_some_and(|finished| finished) {
                return;
            }
        }
    }


    pub async fn delay_frame(&mut self, schedule_label: impl ScheduleLabel, delay_frames: usize) {
        let (tx, mut rx) = channel::<()>(1);
        self.0.lock().unwrap().push(DelayRunner::boxed(tx, schedule_label, delay_frames));

        loop {
            if rx.next().await.is_some() {
                return;
            }
        }
    }


    pub(crate) fn run_systems(
        &mut self, 
        schedule_label: &dyn ScheduleLabel,
        world: &mut World
    ) {
        let mut systems = self.0.lock().unwrap();
        let mut next_systems = Vec::with_capacity(systems.len());
        while let Some(mut system) = systems.pop() {
            if !system.should_run(schedule_label){
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

