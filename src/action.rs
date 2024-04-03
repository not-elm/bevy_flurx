//!
//!
//! - [`once_action`]
//! - [`wait`]
//! - [`delay`]

use std::marker::PhantomData;

use bevy::prelude::System;

use crate::runner::{RunTask, TaskOutput};
use crate::runner::multi_times::MultiTimesRunner;

pub mod once;
pub mod wait;
pub mod repeat;
pub mod delay;
mod sequence;

#[doc(hidden)]
pub struct WithInput;

#[doc(hidden)]
pub struct WithoutInput;

#[doc(hidden)]
pub trait TaskAction<Marker = WithInput> {
    type In;

    type Out;

    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask;
}

impl<Out, Sys> TaskAction<WithoutInput> for Sys
    where
        Sys: System<In=(), Out=Option<Out>>,
        Out: 'static
{
    type In = ();
    type Out = Out;

    #[inline(always)]
    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask {
        MultiTimesRunner::new(self, (), output)
    }
}

#[inline(always)]
pub fn with<Sys, Input, Out>(input: Input, system: Sys) -> impl TaskAction<WithInput, In=Input, Out=Out>
    where
        Sys: System<In=Input, Out=Option<Out>>,
        Input: Clone + 'static,
        Out: 'static
{
    WithAction(input, system, PhantomData)
}

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
    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask {
        MultiTimesRunner::new(self.1, self.0, output)
    }
}
