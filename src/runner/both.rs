use std::marker::PhantomData;

use bevy::prelude::World;

use crate::action::TaskAction;
use crate::runner::{RunTask, TaskOutput};

pub(crate) struct BothRunner<I1, I2, O1, O2, M1, M2> {
    r1: Box<dyn RunTask>,
    r2: Box<dyn RunTask>,
    o1: TaskOutput<O1>,
    o2: TaskOutput<O2>,
    output: TaskOutput<(O1, O2)>,
    _m: PhantomData<(I1, I2, M1, M2)>,
}

impl<I1, I2, O1, O2, M1, M2> BothRunner<I1, I2, O1, O2, M1, M2> {
    #[inline]
    pub fn new(
        output: TaskOutput<(O1, O2)>,
        a1: impl TaskAction<M1, In=I1, Out=O1> + 'static,
        a2: impl TaskAction<M2, In=I2, Out=O2> + 'static,
    ) -> BothRunner<I1, I2, O1, O2, M1, M2>
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
            output,
            _m: PhantomData,
        }
    }
}

impl<I1, I2, O1, O2, M1, M2> RunTask for BothRunner<I1, I2, O1, O2, M1, M2> {
    fn run(&mut self, world: &mut World) -> bool {
        if self.o1.is_none() {
            self.r1.run(world);
        }
        if self.o2.is_none() {
            self.r2.run(world);
        }
        if let Some(out1) = self.o1.take() {
            if let Some(out2) = self.o2.take() {
                self.output.replace((out1, out2));
                true
            } else {
                self.o1.replace(out1);
                false
            }
        } else {
            false
        }
    }
}