//!
//!
//! - [`once`]
//! - [`wait`]
//! - [`delay`]

use std::marker::PhantomData;
use bevy::prelude::{System, World};

use crate::runner::{CancellationToken, RunnerIntoAction, RunWithTaskOutput, TaskOutput, TaskRunner};
use crate::runner::multi_times::MultiTimesRunner;

pub mod once;
pub mod wait;
pub mod repeat;
pub mod delay;
pub mod sequence;
pub mod switch;

/// Represents the system passed to [`ReactiveTask`](crate::task::ReactiveTask).
pub trait TaskAction {
    /// The input value of the system.
    type In;
    
    /// The output value of the system.
    type Out;
    
    /// Convert itself to [`TaskRunner`](crate::runner::TaskRunner).
    fn to_runner(self, token: CancellationToken, output: TaskOutput<Self::Out>) -> impl TaskRunner ;
}

impl<Out, Sys> TaskAction for Sys
    where
        Sys: System<In=(), Out=Option<Out>>,
        Out: 'static
{
    type In = ();

    type Out = Out;

    #[inline(always)]
    fn to_runner(self, token: CancellationToken, output: TaskOutput<Out>) -> impl TaskRunner {
        (token, output, MultiTimesRunner::new(self, ()))
    }
}

/// Create the action based on the system and its input value.
#[inline(always)]
pub fn with<Sys, Input, Out>(input: Input, system: Sys) -> impl TaskAction<In=Input, Out=Out>
    where
        Sys: System<In=Input, Out=Option<Out>>,
        Input: Clone + 'static,
        Out: 'static
{
    RunnerIntoAction::new(MultiTimesRunner::new(system, input))
}

/// Convert to the output of action to tuple. 
pub fn to_tuple<M, I, O>(action: impl TaskAction<In=I, Out=O> + 'static) -> impl TaskAction<In=I, Out=(O, )>
    where
        I: 'static,
        O: 'static,
        M: 'static
{
    struct Runner<I, O> {
        r: Box<dyn TaskRunner>,
        o: TaskOutput<O>,
        token: CancellationToken,
        _m: PhantomData<I>
    }
    impl<I, O> RunWithTaskOutput<(O, )> for Runner<I, O> {
        type In = I;
        
        fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<(O, )>, world: &mut World) -> bool {
            if token.requested_cancel() {
                self.token.cancel();
                return true;
            }
            self.r.run(world);
            if let Some(o) = self.o.take() {
                output.replace((o, ));
                true
            } else {
                false
            }
        }
    }
    let token = CancellationToken::default();
    let o = TaskOutput::default();
    let r = action.to_runner(token.clone(), o.clone());
    RunnerIntoAction::new(Runner {
        r: Box::new(r),
        o,
        token,
        _m: PhantomData
    })
}