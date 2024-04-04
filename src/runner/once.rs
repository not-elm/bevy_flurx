use bevy::prelude::{System, World};

use crate::runner::{TaskRunner, TaskOutput};

pub(crate) struct OnceRunner<Sys, In, Out> {
    system: Sys,
    input: Option<In>,
    output: TaskOutput<Out>,
    init: bool
}

impl<Sys, In, Out> OnceRunner<Sys, In, Out> {
    #[inline]
    pub const fn new(
        system: Sys,
        input: In,
        output: TaskOutput<Out>,
    ) -> OnceRunner<Sys, In, Out> {
        Self {
            system,
            input: Some(input),
            output,
            init: false
        }
    }
}

impl<Sys, In, Out> TaskRunner for OnceRunner<Sys, In, Out>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: 'static,
        Out: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if !self.init{
            self.system.initialize(world);
            self.init = true;
        }
        
        let Some(input) = self.input.take() else {
            return true;
        };
        let out = self.system.run(input, world);
        self.system.apply_deferred(world);
        if let Some(output) = out {
            self.output.replace(output);
            true
        } else {
            false
        }
    }
}