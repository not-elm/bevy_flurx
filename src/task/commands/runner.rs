pub mod delay;
pub mod once;
pub mod until;
pub mod maybe;

use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;

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


pub type BoxedAsyncSystemRunner = Box<dyn AsyncSystemRunnable>;


pub trait AsyncSystemRunnable {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus;

    fn should_run(&self, schedule_label: &dyn ScheduleLabel) -> bool;
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
    fn should_run(&self, schedule_label: &dyn ScheduleLabel) -> bool{
        schedule_label.eq(&self.schedule_label)
    }
}






