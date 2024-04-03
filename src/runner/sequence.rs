use bevy::prelude::World;

use crate::runner::RunTask;

enum Status {
    RunFirst,
    RunSecond,
    Finish,
}

pub(crate) struct SequenceRunner<First, Second> {
    first_runner: First,
    second_runner: Second,
    status: Status,
}

impl<First, Second> SequenceRunner<First, Second> {
    #[inline]
    pub const fn new(
        first_runner: First,
        second_runner: Second,
    ) -> SequenceRunner<First, Second> {
        Self {
            first_runner,
            second_runner,
            status: Status::RunFirst
        }
    }
}

impl<First, Second> RunTask for SequenceRunner<First, Second>
    where
        First: RunTask,
        Second: RunTask
{
    fn run(&mut self, world: &mut World) -> bool {
        match self.status {
            Status::RunFirst => {
                if self.first_runner.run(world) {
                    self.status = Status::RunSecond;
                }
                false
            }
            Status::RunSecond => {
                if self.second_runner.run(world) {
                    self.status = Status::Finish;
                    true
                } else {
                    false
                }
            }
            Status::Finish => {
                true
            }
        }
    }
}