//!
//!
//! - [`once`]
//! - [`wait`]
//! - [`delay`]

pub use through::through;

use crate::prelude::ActionSeed;
use crate::runner::{BoxedActionRunner, CancellationToken, Output};

pub mod once;
pub mod wait;
pub mod delay;

pub mod switch;
pub mod seed;
pub mod through;
pub mod pipe;
pub mod sequence;
mod repeat;
mod tuple;
pub use tuple::tuple;

/// Represents the system passed to [`ReactiveTask`](crate::task::ReactiveTask).
pub struct Action<I=(), O=()>(pub(crate) I, pub(crate) ActionSeed<I, O>);

impl<I1, O1> Action<I1, O1>
    where
        I1: 'static,
        O1: 'static
{
    #[inline]
    pub(crate) fn into_runner(self, token: CancellationToken, output: Output<O1>) -> BoxedActionRunner {
        self.1.create_runner(self.0, token, output)
    }
}

impl<Out> From<ActionSeed<(), Out>> for Action<(), Out>
    where
        Out: 'static
{
    #[inline]
    fn from(value: ActionSeed<(), Out>) -> Self {
        value.with(())
    }
}

// impl<I, O> Clone for Action<I, O>
//     where
//         I: Clone
// {
//     fn clone(&self) -> Self {
//         Self(self.0.clone(), self.1.clone())
//     }
// }


