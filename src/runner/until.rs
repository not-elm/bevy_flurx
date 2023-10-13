use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;
use crate::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, SystemRunningStatus};

pub struct UntilRunner {
    base: BaseRunner<bool>,
}

impl UntilRunner {
    pub fn boxed<Marker>(
        tx: Sender<bool>,
        schedule_label: impl ScheduleLabel,
        system: impl IntoSystem<(), bool, Marker> + Send + 'static,
    ) -> BoxedAsyncSystemRunner {
        Box::new(Self {
            base: BaseRunner::new(tx, schedule_label, system)
        })
    }
}


impl AsyncSystemRunnable for UntilRunner
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        let finished = self.base.run_with_output(world);
        if finished {
            self.base.tx.try_send(true).unwrap();
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