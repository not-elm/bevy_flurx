//! Provides the trait for converting into an action. 


use crate::action::Action;
use crate::runner::{CancellationToken, TaskOutput, TaskRunner};

pub(crate) mod once;
pub(crate) mod wait;

///
/// If [`In`] type of the struct implements this is `()`, 
/// its struct also implements [`Action`] automatically.
///
/// Otherwise, to convert to the action,
/// you need call [`ActionSeed::with`] or passed itself as an argument to [`Pipe::pipe`].  
///
/// [`Action`]: crate::prelude::Action
/// [`Pipe::pipe`]: crate::prelude::Pipe::pipe
pub trait ActionSeed<In = (), Out = ()> {
    /// Into [`Action`] with `input`.
    ///
    /// [`Action`]: crate::prelude::Action
    fn with(self, input: In) -> impl Action<In, Out>;
}

/// This is a dummy marker
pub trait SeedMark {}

impl<Out, A> Action<(), Out> for A
    where A: ActionSeed<(), Out> + SeedMark
{
    #[inline(always)]
    fn to_runner(self, token: CancellationToken, output: TaskOutput<Out>) -> impl TaskRunner + 'static {
        self.with(()).to_runner(token, output)
    }
}