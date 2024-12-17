/// Selector defines what a task created by [`ReactiveTask`] will do.
///
/// [`ReactiveTask`]: crate::prelude::ReactiveTask
pub trait Selector<State> {
    type Output;

    /// The Option value in the output indicates that Future is still pending if Some, or that the task is ready if Some.
    fn select(&mut self, state: State) -> Option<Self::Output>;
}

