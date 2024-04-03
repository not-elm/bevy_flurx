use std::marker::PhantomData;

use crate::action::TaskAction;
use crate::runner::{RunTask, RunWithTaskOutput, TaskOutput};
use crate::runner::macros::{impl_tuple_runner, output_combine};

pub struct SequenceRunner<I1, I2, O1, O2, M1, M2> {
    r1: Box<dyn RunTask>,
    r2: Box<dyn RunTask>,
    o1: TaskOutput<O1>,
    o2: TaskOutput<O2>,
    _m: PhantomData<(I1, I2, M1, M2)>,
}

impl<I1, I2, O1, O2, M1, M2> SequenceRunner<I1, I2, O1, O2, M1, M2> {
    #[inline]
    pub fn new(
        a1: impl TaskAction<M1, In=I1, Out=O1> + 'static,
        a2: impl TaskAction<M2, In=I2, Out=O2> + 'static,
    ) -> SequenceRunner<I1, I2, O1, O2, M1, M2>
        where
            M1: 'static,
            M2: 'static
    {
        let o1 = TaskOutput::default();
        let o2 = TaskOutput::default();
        let r1 = a1.to_runner(o1.clone());
        let r2 = a2.to_runner(o2.clone());
        Self {
            r1: Box::new(r1),
            r2: Box::new(r2),
            o1,
            o2,
            _m: PhantomData,
        }
    }
}

macro_rules! impl_sequence_runner {
    ($($lhs_out: ident $(,)?)*) => {
        impl<I1, I2, $($lhs_out,)* O2, M1, M2> TaskAction for SequenceRunner<I1, I2, ($($lhs_out,)*), O2, M1, M2> {
            type In = (I1, I2);
            type Out = ($($lhs_out,)* O2);

            #[inline(always)]
            fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask {
                (output, self)
            }
        }

        #[allow(non_snake_case)]
        impl<I1, I2, $($lhs_out,)* O2, M1, M2> RunWithTaskOutput<($($lhs_out,)* O2)> for SequenceRunner<I1, I2, ($($lhs_out,)*), O2, M1, M2> {
            fn run_with_task_output(&mut self, output: &mut TaskOutput<($($lhs_out,)* O2)>, world: &mut bevy::prelude::World) -> bool {
                if self.o1.is_none() {
                    self.r1.run(world);
                }
                if self.o1.is_some() && self.o2.is_none() {
                    self.r2.run(world);
                }
                output_combine!(&self.o1, &self.o2, output, $($lhs_out),*)
            }
        }
    }
}

impl_tuple_runner!(impl_sequence_runner);

