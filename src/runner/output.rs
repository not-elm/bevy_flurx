use std::cell::RefCell;
use std::rc::Rc;

/// Represents the output of the task.
/// See details [`Runner`](crate::prelude::Runner).
#[repr(transparent)]
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
    /// Set the output value.
    ///
    /// If the output already exists, it is replaced.
    #[inline(always)]
    pub fn set(&self, o: O) {
        self.0.borrow_mut().replace(o);
    }

    /// Takes the value out of the [`Output`].
    #[inline(always)]
    pub fn take(&self) -> Option<O> {
        self.0.borrow_mut().take()
    }

    /// Returns true if output value is exists.
    #[inline(always)]
    pub fn is_some(&self) -> bool {
        self.0.borrow().is_some()
    }

    /// Returns true if output value is not exists.
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        self.0.borrow().is_none()
    }
}

