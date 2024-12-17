/// Selector defines what a task created by [`ReactiveTask`] will do.
///
/// [`ReactiveTask`]: crate::prelude::ReactiveTask
pub trait Selector<State> {
    type Output;

    /// The Option value in the output indicates that Future is still pending if Some, or that the task is ready if Some.
    fn select(&self, state: State) -> Option<Self::Output>;
}

impl<State, Output, F> Selector<State> for F
    where F: Fn(State) -> Option<Output> + Unpin
{
    type Output = Output;

    #[inline]
    fn select(&self, state: State) -> Option<Self::Output> {
        (self)(state)
    }
}

