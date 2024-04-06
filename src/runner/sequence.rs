use bevy::prelude::{Deref, DerefMut, World};

use crate::runner::{CancellationToken, RunWithTaskOutput, TaskOutput, };
use crate::runner::base::BaseTwoRunner;

#[derive(Deref, DerefMut)]
pub struct SequenceRunner<I1, I2, O1, O2>(pub BaseTwoRunner<I1, I2, O1, O2>);

impl<I1, I2, O1, O2> RunWithTaskOutput<O2> for SequenceRunner<I1, I2, O1, O2>
    where
        I1: 'static,
        I2: 'static,
        O1: 'static,
        O2: 'static,


{
    type In = I1;

    fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<O2>, world: &mut World) -> bool {
        if self.cancel_if_need(token) {
            return true;
        }

        if self.o1.is_none() {
            self.r1.run(world);
        }
        if self.o1.is_some() && self.o2.is_none() {
            self.r2.run(world);
        }
        if let Some(o) = self.o2.take() {
            output.replace(o);
            true
        } else {
            false
        }
    }
}




