use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;
use crate::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, SystemRunningStatus};

pub struct UntilRunner {

}


impl AsyncSystemRunnable for UntilRunner
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        // let finished = self.base.run_with_output(world);
        // if finished {
        //     self.base.tx.try_send(true).unwrap();
        //     SystemRunningStatus::Finished
        // } else {
        //     SystemRunningStatus::Running
        // }
        todo!()
    }



}