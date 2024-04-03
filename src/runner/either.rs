use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::TaskAction;
use crate::action::wait::Either;
use crate::runner::{RunTask, TaskOutput};

pub(crate) struct EitherRunner<I1, I2, O1, O2, M1, M2> {
    r1: Box<dyn RunTask>,
    r2: Box<dyn RunTask>,
    o1: TaskOutput<O1>,
    o2: TaskOutput<O2>,
    output: TaskOutput<Either<O1, O2>>,
    _m: PhantomData<(I1, I2, M1, M2)>,
}

impl<I1, I2, O1, O2, M1, M2> EitherRunner<I1, I2, O1, O2, M1, M2> {
    #[inline]
    pub fn new(
        output: TaskOutput<Either<O1, O2>>,
        lhs: impl TaskAction<M1, In=I1, Out=O1> + 'static,
        rhs: impl TaskAction<M2, In=I2, Out=O2> + 'static,
    ) -> EitherRunner<I1, I2, O1, O2, M1, M2> 
        where 
            M1: 'static,
            M2: 'static
    {
        let o1 = TaskOutput::default();
        let o2 = TaskOutput::default();
        let first_runner = lhs.to_runner(o1.clone());
        let second_runner = rhs.to_runner(o2.clone());
        Self {
            r1: Box::new(first_runner),
            r2: Box::new(second_runner),
            o1,
            o2,
            output,
            _m: PhantomData,
        }
    }
}

impl<I1, I2, O1, O2, M1, M2> RunTask for EitherRunner<I1, I2, O1, O2, M1, M2> {
    fn run(&mut self, world: &mut World) -> bool {
        self.r1.run(world);
        if let Some(lhs) = self.o1.take() {
            self.output.replace(Either::Left(lhs));
            return true;
        }
        self.r2.run(world);
        if let Some(rhs) = self.o2.take() {
            self.output.replace(Either::Right(rhs));
            true
        } else {
            false
        }
    }
}