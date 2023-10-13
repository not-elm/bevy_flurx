use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::{Deref, IntoSystem, World};
use futures::channel::mpsc::Sender;

pub mod delay;
pub mod once;
pub mod until;
pub mod maybe;



pub trait AsyncSystemRunnable {
    fn new<In>(&mut self, input: In,  system: ) -> Self;

    fn run(&mut self, world: &mut World) -> SystemRunningStatus;

    fn should_run(&self, schedule_label: &dyn ScheduleLabel) -> bool;
}


pub type BoxedAsyncSystemRunner = Box<dyn AsyncSystemRunnable>;


#[derive(Default, Deref)]
pub struct Runners(Arc<Mutex<Vec<BoxedAsyncSystemRunner>>>);


impl Runners {
    #[inline]
    pub(crate) fn push(&self, runner: BoxedAsyncSystemRunner){
        self.0.lock().unwrap().push(runner);
    }
}

impl Clone for Runners {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SystemRunningStatus {
    #[default]
    NoInitialized,
    Running,
    Finished,
}


impl SystemRunningStatus {
    #[inline]
    pub fn no_initialized(&self) -> bool {
        matches!(self, SystemRunningStatus::NoInitialized)
    }


    #[inline]
    pub fn finished(&self) -> bool {
        matches!(self, SystemRunningStatus::Finished)
    }
}



struct BaseRunner<Output> {
    tx: Sender<Output>,
    schedule_label: BoxedScheduleLabel,
    system: BoxedSystem<(), Output>,
    status: SystemRunningStatus,
}


impl<Output> BaseRunner<Output>
    where Output: 'static
{
    fn new<Marker>(
        tx: Sender<Output>,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), Output, Marker> + Send + 'static,
    ) -> BaseRunner<Output> {
        Self {
            tx,
            schedule_label: Box::new(schedule_label),
            system: Box::new(IntoSystem::into_system(system)),
            status: SystemRunningStatus::NoInitialized,
        }
    }

    fn run_with_output(&mut self, world: &mut World) -> Output {
        if self.status.no_initialized() {
            self.system.initialize(world);
            self.status = SystemRunningStatus::Running;
        }

        let output = self.system.run((), world);
        self.system.apply_deferred(world);
        output
    }


    #[inline]
    fn should_run(&self, schedule_label: &dyn ScheduleLabel) -> bool {
        schedule_label.eq(&self.schedule_label)
    }
}






