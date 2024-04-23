use std::cell::RefCell;
use std::rc::Rc;

/// Represents the output of the task.
/// See details [`Runner`](crate::prelude::Runner).
pub struct Output<O>(Rc<RefCell<Option<O>>>);

impl<O> Clone for Output<O> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<O> Default for Output<O> {
    #[inline]
    fn default() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }
}

impl<O> Output<O> {
    #[inline(always)]
    pub(crate) fn replace(&self, o: O) {
        self.0.borrow_mut().replace(o);
    }

    #[inline(always)]
    pub(crate) fn take(&self) -> Option<O> {
        self.0.borrow_mut().take()
    }

    #[inline(always)]
    pub(crate) fn is_some(&self) -> bool {
        self.0.borrow().is_some()
    }

    #[inline(always)]
    pub(crate) fn is_none(&self) -> bool {
        self.0.borrow().is_none()
    }
}

