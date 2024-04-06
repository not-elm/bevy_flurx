use std::marker::PhantomData;

use bevy::prelude::{System, World};

use crate::runner::{CancellationToken, RunWithTaskOutput, TaskOutput};

pub(crate) struct MultiTimesRunner<Sys, In, Out> {
    system: Sys,
    input: In,
    init: bool,
    _m: PhantomData<Out>,
}

impl<Sys, In, Out> MultiTimesRunner<Sys, In, Out> {
    #[inline]
    pub const fn new(
        system: Sys,
        input: In,
    ) -> MultiTimesRunner<Sys, In, Out> {
        Self {
            system,
            input,
            init: false,
            _m: PhantomData,
        }
    }
}

impl<Sys, In, Out> RunWithTaskOutput<Out> for MultiTimesRunner<Sys, In, Out>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    type In = In;
    
    fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<Out>, world: &mut World) -> bool {
        if token.requested_cancel() {
            return true;
        }
        if !self.init {
            self.system.initialize(world);
            self.init = true;
        }

        let out = self.system.run(self.input.clone(), world);
        self.system.apply_deferred(world);
        if let Some(o) = out {
            output.replace(o);
            true
        } else {
            false
        }
    }
}