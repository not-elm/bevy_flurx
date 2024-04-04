//!
//!
//! - [`once`]
//! - [`wait`]
//! - [`delay`]

use std::marker::PhantomData;

use bevy::prelude::{System, World};

use crate::runner::{TaskOutput, TaskRunner};
use crate::runner::multi_times::MultiTimesRunner;

pub mod once;
pub mod wait;
pub mod repeat;
pub mod delay;
pub mod switch;
mod sequence;


#[doc(hidden)]
pub struct WithInput;

#[doc(hidden)]
pub struct WithoutInput;

/// Represents the system passed to [`ReactiveTask`](crate::task::ReactiveTask).
pub trait TaskAction<Marker = WithInput> {
    /// The input of the system.
    type In;

    /// The output of the system.
    type Out;

    /// Convert itself to [`TaskRunner`](crate::runner::TaskRunner).
    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl TaskRunner;
}

impl<Out, Sys> TaskAction<WithoutInput> for Sys
    where
        Sys: System<In=(), Out=Option<Out>>,
        Out: 'static
{
    type In = ();
    type Out = Out;

    #[inline(always)]
    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl TaskRunner {
        MultiTimesRunner::new(self, (), output)
    }
}

/// Create the action based on the system and its input value.
#[inline]
pub fn with<Sys, Input, Out>(input: Input, system: Sys) -> impl TaskAction<WithInput, In=Input, Out=Out>
    where
        Sys: System<In=Input, Out=Option<Out>>,
        Input: Clone + 'static,
        Out: 'static
{
    struct WithAction<Sys, In, Out>(In, Sys, PhantomData<Out>)
        where In: Clone + 'static,
              Sys: System<In=In, Out=Option<Out>>;

    impl<Sys, In, Out> TaskAction for WithAction<Sys, In, Out>
        where In: Clone + 'static,
              Sys: System<In=In, Out=Option<Out>>,
              Out: 'static
    {
        type In = In;

        type Out = Out;

        #[inline(always)]
        fn to_runner(self, output: TaskOutput<Self::Out>) -> impl TaskRunner {
            MultiTimesRunner::new(self.1, self.0, output)
        }
    }
    
    WithAction(input, system, PhantomData)
}

/// Convert to the output of action to tuple. 
pub fn to_tuple<M, I, O>(action: impl TaskAction<M, In=I, Out=O> + 'static) -> impl TaskAction<M, In=I, Out=(O, )>
    where
        M: 'static
{
    struct Action<A, I, O>(A, PhantomData<(I, O)>);

    impl<M, I, O, A> TaskAction<M> for Action<A, I, O>
        where
            A: TaskAction<M, In=I, Out=O> + 'static,
            M: 'static
    {
        type In = I;
        type Out = (O, );

        fn to_runner(self, output: TaskOutput<Self::Out>) -> impl TaskRunner {
            let o = TaskOutput::default();
            let r = self.0.to_runner(o.clone());
            Runner {
                r: Box::new(r),
                o,
                output,
            }
        }
    }

    struct Runner<O> {
        r: Box<dyn TaskRunner>,
        o: TaskOutput<O>,
        output: TaskOutput<(O, )>,
    }
    impl<O> TaskRunner for Runner<O> {
        fn run(&mut self, world: &mut World) -> bool {
            self.r.run(world);
            if let Some(o) = self.o.take() {
                self.output.replace((o, ));
                true
            } else {
                false
            }
        }
    }
    Action(action, PhantomData)
}