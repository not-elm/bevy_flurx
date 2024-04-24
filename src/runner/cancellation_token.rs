use std::cell::Cell;
use std::rc::Rc;

/// Structure for canceling a task
#[derive(Default)]
pub struct CancellationToken(Rc<Cell<bool>>);

impl Clone for CancellationToken {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl CancellationToken {
    /// Requests to cancel a runner's action.
    #[inline(always)]
    pub fn cancel(&self) {
        self.0.set(true);
    }

    /// Returns `true` if a runner's action has been requested to cancel.
    #[inline(always)]
    pub(crate) fn requested_cancel(&self) -> bool {
        self.0.get()
    }
}
