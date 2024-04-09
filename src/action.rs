//! An `action` is a system to be run on the [`Reactor`](crate::prelude::Reactor).
//!
//! It is scheduled by [`Reactor`](crate::prelude::Reactor) and is run once per frame(twice if uninitialized).
//! 
//! Every action has an end condition, and if the condition is met, the next action proceeds.
//! 
//! For example, in the following code, the exit condition is to wait until the count reaches 2,
//! and when it reaches 2, proceed to `process: 2`.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_flurx::prelude::*;
//! 
//! Reactor::schedule(|task| async move{
//!     // `process: 1`
//!     task.will(Update, wait::until(|mut count: Local<usize>|{
//!         *count += 1;
//!         *count == 2
//!     })).await;
//!     // `process: 2`
//! });
//! ```
//! 
//! actions
//! 
//! - [`once`]
//! - [`wait`]
//! - [`delay`]
//! - [`pipe`]
//! - [`sequence`]
//! - [`switch`]
//! - [`through`]
//! - [`tuple`] 

pub use through::through;

use crate::prelude::ActionSeed;
use crate::runner::{BoxedRunner, CancellationToken, Output};

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
/// 
/// Please check [here](crate::action) for more details.
pub struct Action<I=(), O=()>(pub(crate) I, pub(crate) ActionSeed<I, O>);

impl<I1, O1> Action<I1, O1>
    where
        I1: 'static,
        O1: 'static
{
    #[inline]
    pub(crate) fn into_runner(self, token: CancellationToken, output: Output<O1>) -> BoxedRunner {
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


