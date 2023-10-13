use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{IntoSystem, World};
use futures::channel::mpsc::Sender;

use crate::runner::{AsyncSystemRunnable, BaseRunner, BoxedAsyncSystemRunner, SystemRunningStatus};

pub struct MaybeOutputRunner<Output>(Output) ;


impl<Output> AsyncSystemRunnable for MaybeOutputRunner<Output>
    where Output: 'static
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        // if let Some(output) = self.base.run_with_output(world) {
        //     self.base.tx.try_send(Some(output)).unwrap();
        //     SystemRunningStatus::Finished
        // } else {
        //     SystemRunningStatus::Running
        // }
        todo!()
    }


}