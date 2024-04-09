use bevy::prelude::World;

use crate::prelude::ActionSeed;
use crate::runner::{BoxedRunner, CancellationToken, Output, Runner};

/// Convert to the output of action to tuple.
pub fn tuple<I, O>(action: ActionSeed<I, O>) -> ActionSeed<I, (O, )>
    where
        I: 'static,
        O: 'static
{
    ActionSeed::new(|input, token, output| {
        let tmp = Output::default();
        let runner = action.create_runner(input, token.clone(), tmp.clone());
        TupleRunner {
            runner,
            tmp,
            output,
            token,
        }
    })
}

struct TupleRunner<O> {
    runner: BoxedRunner,
    tmp: Output<O>,
    output: Output<(O, )>,
    token: CancellationToken,
}

impl<O> Runner for TupleRunner<O> {
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        self.runner.run(world);
        if let Some(o) = self.tmp.take() {
            self.output.replace((o, ));
            true
        } else {
            false
        }
    }
}