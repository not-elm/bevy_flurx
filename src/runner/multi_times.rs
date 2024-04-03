use bevy::prelude::{System, World};

use crate::runner::{RunTask, TaskOutput};

pub(crate) struct MultiTimesRunner<Sys, In, Out> {
    system: Sys,
    input: In,
    output: TaskOutput<Out>,
    init: bool
}

impl<Sys, In, Out> MultiTimesRunner<Sys, In, Out> {
    #[inline]
    pub const fn new(
        system: Sys,
        input: In,
        output: TaskOutput<Out>,
    ) -> MultiTimesRunner<Sys, In, Out> {
        Self {
            system,
            input,
            output,
            init: false
        }
    }
}

impl<Sys, In, Out> RunTask for MultiTimesRunner<Sys, In, Out>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if !self.init{
            self.system.initialize(world);
            self.init = true;
        }

        let out = self.system.run(self.input.clone(), world);
        self.system.apply_deferred(world);
        if let Some(output) = out {
            self.output.borrow_mut().replace(output);
            true
        } else {
            false
        }
    }
}