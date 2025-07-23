use crate::core::scheduler::future::ReactorsFuture;
use crate::core::scheduler::state_ptr::StatePtr;
use crate::core::task::CoreTask;
use bevy::prelude::*;
use core::future::Future;
use core::pin::Pin;

mod future;
mod state_ptr;

pub(crate) type CoreReactor<'state> = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'state>>;

pub struct CoreScheduler<State> {
    state: StatePtr<State>,
    reactor: CoreReactor<'static>,
    pub(crate) finished: bool,
}

impl<State> CoreScheduler<State>
where
    State: Clone + 'static,
{
    #[inline(always)]
    pub fn schedule<F, Fut>(f: F) -> CoreScheduler<State>
    where
        F: FnOnce(CoreTask<State>) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let mut state = StatePtr(Box::new(None));
        Self {
            reactor: Box::pin(f(CoreTask {
                state: state.state_ref(),
            })),
            state,
            finished: false,
        }
    }

    /// Poll all registered `Reactors` once each.
    #[inline(always)]
    pub async fn run(&mut self, state: State) {
        self.state.0.replace(state);
        ReactorsFuture {
            finished: &mut self.finished,
            reactor: &mut self.reactor,
        }
        .await;
    }
}
