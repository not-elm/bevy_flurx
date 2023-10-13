use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;

use crate::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, SystemRunningStatus};

pub struct OnceRunner<Output> {
    base: BaseRunner<Output>,
}

impl<Output> OnceRunner<Output>
    where Output: 'static
{
    pub fn boxed<Marker>(
        tx: Sender<Output>,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), Output, Marker> + Send + 'static,
    ) -> BoxedAsyncSystemRunner {
        Box::new(Self {
            base: BaseRunner::new(tx, schedule_label, system)
        })
    }
}


impl<Output> AsyncSystemRunnable for OnceRunner<Output>
    where Output: 'static
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let output = self.base.run_with_output(world);
        self.base.tx.try_send(output).unwrap();
        SystemRunningStatus::Finished
    }


    #[inline]
    fn should_run(&self, label: &dyn ScheduleLabel) -> bool {
        self.base.should_run(label)
    }
}
