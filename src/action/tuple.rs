use bevy::prelude::World;

use crate::prelude::{ActionSeed, CancellationToken};
use crate::runner::{BoxedRunner, Output, Runner};

/// Convert to the output of action to tuple.
pub fn tuple<I, O>(action: ActionSeed<I, O>) -> ActionSeed<I, (O, )>
    where
        I: 'static,
        O: 'static
{
    ActionSeed::new(|input, output| {
        let tmp = Output::default();
        let runner = action.create_runner(input, tmp.clone());
        TupleRunner {
            runner,
            tmp,
            output
        }
    })
}

struct TupleRunner<O> {
    runner: BoxedRunner,
    tmp: Output<O>,
    output: Output<(O, )>
}

impl<O> Runner for TupleRunner<O> {
    fn run(&mut self, world: &mut World, token: &CancellationToken) -> bool {
        self.runner.run(world, token);
        if let Some(o) = self.tmp.take() {
            self.output.set((o, ));
            true
        } else {
            false
        }
    }

    fn on_cancelled(&mut self, world: &mut World) {
        self.runner.on_cancelled(world);
    }
}