use std::cell::Cell;
use std::rc::Rc;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ReactorStatus {
    pub cancelled: bool,
    pub reactor_finished: bool,
}

/// Structure for canceling a [`Reactor`](crate::prelude::Reactor).
///
/// This is passed as argument in [`Runner::run`](crate::prelude::Runner::run),
/// and the [`Reactor`](crate::prelude::Reactor) can be cancelled by calling [`CancellationToken::cancel`]. 
#[derive(Default, Debug)]
pub struct CancellationToken(Rc<Cell<ReactorStatus>>);

impl Clone for CancellationToken {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl CancellationToken {
    /// Requests to cancel a [`Reactor`](crate::prelude::Reactor).
    #[inline(always)]
    pub fn cancel(&self) {
        let status = self.0.get();
        self.0.set(ReactorStatus {
            cancelled: true,
            reactor_finished: status.reactor_finished,
        });
    }

    #[inline(always)]
    pub(crate) fn set(&self, status: ReactorStatus) {
        self.0.set(status);
    }

    #[inline(always)]
    pub(crate) fn status(&self) -> ReactorStatus {
        self.0.get()
    }
}
