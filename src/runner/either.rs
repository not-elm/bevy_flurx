use bevy::prelude::{Deref, DerefMut, World};

use crate::action::wait::Either;
use crate::runner::{CancellationToken, RunWithTaskOutput, TaskOutput, };
use crate::runner::base::BaseTwoRunner;

#[derive(Deref, DerefMut)]
pub(crate) struct EitherRunner<I1, I2, O1, O2>(pub(crate) BaseTwoRunner<I1, I2, O1, O2>);

impl<I1, I2, O1, O2> RunWithTaskOutput<Either<O1, O2>> for EitherRunner<I1, I2, O1, O2> {
    type In = (I1, I2);

    fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<Either<O1, O2>>, world: &mut World) -> bool {
        if self.cancel_if_need(token) {
            return true;
        }

        self.r1.run(world);
        if let Some(lhs) = self.o1.take() {
            output.replace(Either::Left(lhs));
            return true;
        }
        self.r2.run(world);
        if let Some(rhs) = self.o2.take() {
            output.replace(Either::Right(rhs));
            true
        } else {
            false
        }
    }
}