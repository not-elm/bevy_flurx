use core::future::Future;
use core::pin::Pin;
use crate::core::scheduler::future::ReactorsFuture;
use crate::core::scheduler::state_ptr::StatePtr;
use crate::core::task::CoreTask;

mod future;
mod state_ptr;

pub(crate) type CoreReactor<'state> = Pin<Box<dyn Future<Output=()> + Send + Sync + 'state>>;

pub struct CoreScheduler<State> {
    state: StatePtr<State>,
    reactor: Option<CoreReactor<'static>>,
}

impl<State> Default for CoreScheduler<State>
        where 
            State: Clone + 'static
{
    fn default() -> Self {
        Self::new()
    }
}

impl<State> CoreScheduler<State>
    where
        State: Clone + 'static
{
    #[must_use]
    #[inline(always)]
    /// Creates the empty scheduler.
    pub const fn new() -> CoreScheduler<State> {
        Self {
            state: StatePtr::uninit(),
            reactor: None,
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn not_exists_reactor(&self) -> bool {
        self.reactor.is_none()
    }

    /// Schedule the new [`CoreReactor`].
    ///
    /// The reality [`CoreReactor`] is [`Future`], it is polled once every time [`CoreScheduler::run`] is called.
    ///
    /// ## Examples
    /// 
    /// ```ignore
    /// use flurx::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main(){
    ///     let mut scheduler = Scheduler::<usize>::new();
    ///     scheduler.schedule(|task|async move{
    ///         // (1)
    ///         task.will(wait::until(|state: usize|{
    ///             state < 2
    ///         })).await;
    ///     });
    ///     // state is 0, (1) returns [`Future::Pending`].
    ///     scheduler.run(0).await;
    ///     // state is 1, (1) returns [`Future::Pending`].
    ///     scheduler.run(1).await;
    ///     // state is 2, (1) returns [`Future::Ready(2)`].
    ///     scheduler.run(2).await;
    /// }
    /// ```
    #[inline(always)]
    pub fn schedule<F, Fut>(&mut self, f: F)
        where
            F: FnOnce(CoreTask<State>) -> Fut + Send + Sync ,
            Fut: Future<Output=()> + Send + Sync + 'static,
    {
        self.reactor.replace(Box::pin(f(CoreTask {
            state: self.state.state_ref()
        })));
    }

    /// Poll all registered `Reactors` once each.
    #[inline(always)]
    pub async fn run(&mut self, state: State) {
        self.state.set(state);
        ReactorsFuture(&mut self.reactor).await;
    }
}

