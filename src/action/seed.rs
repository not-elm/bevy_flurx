//! Provides the trait for converting into an action. 


use std::marker::PhantomData;

use crate::action::Action;
use crate::runner::{BoxedRunner, CancellationToken, Output, Runner};

///
/// If [`In`] type of the struct implements this is `()`, 
/// its struct also implements [`Action`] automatically.
///
/// Otherwise, to convert to the action,
/// you need call [`ActionSeed::with`] or passed itself as an argument to [`Pipe::pipe`].  
///
/// [`Action`]: crate::prelude::Action
/// [`Pipe::pipe`]: crate::prelude::Pipe::pipe
pub struct ActionSeed<I = (), O = ()>(Box<dyn FnOnce(I, CancellationToken, Output<O>) -> BoxedRunner>, PhantomData<I>);

impl<I, O> ActionSeed<I, O>
    where
        I: 'static,
        O: 'static
{
    /// Create the [`ActionSeed`].
    #[inline]
    pub fn new<R>(f: impl FnOnce(I, CancellationToken, Output<O>) -> R + 'static) -> ActionSeed<I, O>
        where
            R: Runner + 'static
    {
        ActionSeed(Box::new(move |input, token, output| {
            BoxedRunner(Box::new(f(input, token, output)))
        }), PhantomData)
    }

    /// Into [`Action`] with `input`.
    ///
    /// [`Action`]: crate::prelude::Action
    #[inline]
    pub const fn with(self, input: I) -> Action<I, O> {
        Action(input, self)
    }

    #[inline]
    pub(crate) fn create_runner(self, input: I, token: CancellationToken, output: Output<O>) -> BoxedRunner {
        (self.0)(input, token, output)
    }
}

// impl<I, O> Clone for ActionSeed<I, O> {
//     #[inline]
//     fn clone(&self) -> Self {
//         Self(self.0.clone(), PhantomData)
//     }
// }
//


