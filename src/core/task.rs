use crate::core::selector::Selector;
use crate::core::task::future::TaskFuture;
use core::future::Future;

pub mod future;

pub(crate) type StateRef<State> = &'static Option<State>;

pub struct CoreTask<State: 'static> {
    pub(crate) state: StateRef<State>,
}

impl<State> CoreTask<State> {
    /// Create a new task.
    ///
    /// Several [`Selector`](crate::selector::Selector)s are provided by default, but you can also define your own.
    ///
    /// The default [`Selector`](crate::selector::Selector)s are as follows.
    ///
    /// * [`wait::until`](crate::prelude::wait::until)
    /// * [`wait::while`](crate::prelude::wait::while_)
    /// * [`once::run`](crate::prelude::once::run)
    /// * [`repeat::count`](crate::prelude::repeat::count)
    /// * [`repeat::forever`](crate::prelude::repeat::forever)
    /// * [`delay::time`](crate::prelude::delay::time)
    ///
    /// ## Examples
    ///
    /// ```ignore
    /// use flurx::prelude::once;
    /// use flurx::Scheduler;
    ///
    /// let mut scheduler = Scheduler::<usize>::new();
    /// scheduler.schedule(|task|async move{
    ///     task.will(once::run(|state: usize|{
    ///         state
    ///     })).await;
    /// });
    /// ```
    #[inline]
    pub fn will<Out, Sel>(&self, selector: Sel) -> impl Future<Output = Out> + 'static
    where
        Sel: Selector<State, Output = Out> + 'static,
        State: Copy + 'static,
    {
        TaskFuture::<State, Sel> {
            state: self.state,
            selector,
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl<State: 'static> Clone for CoreTask<State> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<State> Copy for CoreTask<State> {}
