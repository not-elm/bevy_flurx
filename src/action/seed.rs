//! Provides the trait for converting into an action.

use bevy::prelude::Component;
use crate::action::Action;
use crate::runner::{BoxedRunner, Output, Runner};
use bevy::prelude::Reflect;

/// If [`In`](bevy::prelude::In) type of the struct implements this is `()`,
/// its struct also implements Into<[`Action`]> automatically.
///
/// Otherwise, to convert to the action,
/// you need call [`ActionSeed::with`] or passed itself as an argument to [`Pipe::pipe`].  
///
/// [`Action`]: Action
/// [`Pipe::pipe`]: crate::prelude::Pipe::pipe
#[derive(Reflect)]
#[repr(transparent)]
pub struct ActionSeed<I = (), O = ()>(Box<dyn FnOnce(I, Output<O>) -> BoxedRunner + Send + Sync> );


impl<I, O> ActionSeed<I, O>
where
    I: 'static,
    O: 'static,
{
    /// Create the [`ActionSeed`].
    #[inline]
    pub fn new<R>(f: impl FnOnce(I, Output<O>) -> R + Send + Sync   + 'static ) -> ActionSeed<I, O>
    where
        R: Runner  + 'static,
    {
        ActionSeed(Box::new(move |input, output| {
            BoxedRunner::new(f(input, output))
        }))
    }

    /// Define [`ActionSeed`] based on the function that returns an action from the input.
    ///
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::In;
    /// use bevy_flurx::prelude::*;
    ///
    /// ActionSeed::define(|input: usize|{
    ///     once::run(|In(num): In<usize>|{
    ///         assert_eq!(num, 3);
    ///     })
    ///         .with(input)
    /// });
    /// ```
    #[inline]
    pub fn define<I2, A>(f: impl FnOnce(I) -> A + Send + Sync + 'static) -> ActionSeed<I, O>
    where
        I2: 'static,
        A: Into<Action<I2, O>>,
    {
        ActionSeed::from(|input, output| f(input).into().into_runner(output))
    }

    /// Into [`Action`] with `input`.
    ///
    /// [`Action`]:  Action
    #[inline]
    pub const fn with(self, input: I) -> Action<I, O> {
        Action(input, self)
    }

    #[inline(always)]
    pub(crate) fn create_runner(self, input: I, output: Output<O>) -> BoxedRunner {
        self.0(input, output)
    }
}

impl<I, O, F> From<F> for ActionSeed<I, O>
where
    F: FnOnce(I, Output<O>) -> BoxedRunner + Send + Sync + 'static,
{
    #[inline]
    fn from(value: F) -> Self {
        Self(Box::new(value))
    }
}

impl<I, O> Default for ActionSeed<I, O>
where
    I: 'static,
    O: Default + 'static,
{
    #[inline]
    fn default() -> Self {
        crate::prelude::once::no_op_with_generics::<I, O>()
    }
}