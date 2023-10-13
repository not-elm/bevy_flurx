use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;

use crate::task::commands::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, SystemRunningStatus};

pub struct MaybeOutputRunner<Output> {
    base: BaseRunner<Option<Output>>,
}

impl<Output> MaybeOutputRunner<Output>
    where Output: 'static
{
    pub fn boxed<Marker>(
        tx: Sender<Option<Output>>,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), Option<Output>, Marker> + Send + 'static,
    ) -> BoxedAsyncSystemRunner {
        Box::new(Self {
            base: BaseRunner::new(tx, schedule_label, system)
        })
    }
}


impl<Output> AsyncSystemRunnable for MaybeOutputRunner<Output>
    where Output: 'static
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        if let Some(output) = self.base.run_with_output(world) {
            self.base.tx.try_send(Some(output)).unwrap();
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }


    #[inline]
    fn should_run(&self, schedule_label: &dyn ScheduleLabel) -> bool {
        self.base.should_run(schedule_label)
    }
}