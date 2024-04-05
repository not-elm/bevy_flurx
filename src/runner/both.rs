use bevy::prelude::{Deref, DerefMut, World};

use crate::runner::{CancellationToken, RunWithTaskOutput, TaskOutput};
use crate::runner::base::BaseTwoRunner;
use crate::runner::macros::output_combine;

#[derive(Deref, DerefMut)]
pub(crate) struct BothRunner<I1, I2, O1, O2>(pub(crate) BaseTwoRunner<I1, I2, O1, O2>);

impl<I1, I2, O1, O2> RunWithTaskOutput<(O1, O2)> for BothRunner<I1, I2, O1, O2>
    where
        I1: 'static,
        I2: 'static,
        O1: 'static,
        O2: 'static,


{
    type In = (I1, I2);

    fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<(O1, O2)>, world: &mut World) -> bool {
        if self.cancel_if_need(token) {
            return true;
        }
        if self.o1.is_none() {
            self.r1.run(world);
        }
        if self.o2.is_none() {
            self.r2.run(world);
        }
        output_combine!(&self.o1, &self.o2, output)
    }
}