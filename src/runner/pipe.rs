use std::marker::PhantomData;

use bevy::prelude::World;
use crate::action::Action;
use crate::prelude::seed::ActionSeed;


use crate::runner::{CancellationToken, RunWithTaskOutput, TaskOutput, TaskRunner};

pub(crate) struct PipeRunner<I1, O1, O2, P> {
    r1: Box<dyn TaskRunner>,
    r2: Option<Box<dyn TaskRunner>>,
    seed: Option<P>,
    o1: TaskOutput<O1>,
    t1: CancellationToken,
    t2: CancellationToken,
    _m: PhantomData<(I1, O2)>,
}

impl<I1, O1, O2, Seed> PipeRunner<I1, O1, O2, Seed>
    where
        Seed: ActionSeed<O1, O2> + 'static,
        I1: 'static,
        O1: Clone + 'static
{
    pub fn new(
        a1: impl Action<I1, O1> + 'static,
        seed: Seed
    ) -> PipeRunner<I1, O1, O2, Seed>{
        let o1 = TaskOutput::default();
        let t1 = CancellationToken::default();
        let t2 = CancellationToken::default();
        let r1 = a1.to_runner(t1.clone(), o1.clone());
        Self{
            r1: Box::new(r1),
            r2: None,
            t1,
            t2,
            o1,
            seed: Some(seed),
            _m: PhantomData,
        }
    }

    fn setup_second_runner(&mut self, output: TaskOutput<O2>) {
        if let Some(o1) = self.o1.cloned() {
            let Some(action) = self.seed
                .take()
                .map(|p| p.with(o1))else {
                return;
            };
            self.r2.replace(Box::new(action.to_runner(self.t2.clone(), output)));
        }
    }
}

impl<I1, O1, O2, Seed> RunWithTaskOutput<O2> for PipeRunner<I1, O1, O2, Seed>
    where
        Seed: ActionSeed<O1, O2> + 'static,
        O1: Clone + 'static,
        I1: 'static,
        O2: 'static
{
    type In = I1;

    fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<O2>, world: &mut World) -> bool {
        if token.requested_cancel() {
            self.t1.cancel();
            self.t2.cancel();
            return true;
        }
        if self.o1.is_none() {
            self.r1.run(world);
        }
        self.setup_second_runner(output.clone());
        if let Some(r2) = self.r2.as_mut(){
            r2.run(world);
        }
        output.is_some()
    }
}
