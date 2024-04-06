use bevy::log::debug;
use bevy::prelude::{System, World};

use crate::runner::{CancellationToken, RunWithTaskOutput, TaskOutput};

pub(crate) struct OnceRunner<Sys, In> {
    system: Sys,
    input: Option<In>,
    init: bool,
}

impl<Sys, In> OnceRunner<Sys, In> {
    #[inline]
    pub const fn new(
        input: In,
        system: Sys,
    ) -> OnceRunner<Sys, In> {
        Self {
            system,
            input: Some(input),
            init: false,
        }
    }
}

impl<Sys, In, Out> RunWithTaskOutput<Out> for OnceRunner<Sys, In>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: 'static,
        Out: 'static
{
    type In = In;

    fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<Out>, world: &mut World) -> bool {
        if token.requested_cancel() {
            debug!(name: "once runner", "cancel");
            return true;
        }

        if !self.init {
            self.system.initialize(world);
            self.init = true;
        }

        let Some(input) = self.input.take() else {
            return true;
        };
        let out = self.system.run(input, world);
        self.system.apply_deferred(world);
        if let Some(out) = out {
            output.replace(out);
            true
        } else {
            false
        }
    }
}