use std::sync::{Arc, RwLock};

/// Represents the output of the task.
/// See details [`Runner`](crate::prelude::Runner).
#[repr(transparent)]
pub struct Output<O>(Arc<RwLock<Option<O>>>);

impl<O> Clone for Output<O> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<O> Default for Output<O> {
    #[inline]
    fn default() -> Self {
        Self(Arc::new(RwLock::new(None)))
    }
}

impl<O> Output<O> {
    /// Set the output value.
    ///
    /// If the output already exists, it is replaced.
    #[inline(always)]
    pub fn set(&self, o: O) {
        self.0.write().unwrap().replace(o);
    }

    /// Takes the value out of the [`Output`].
    #[inline(always)]
    pub fn take(&self) -> Option<O> {
        self.0.write().unwrap().take()
    }

    /// Returns true if output value is exists.
    #[inline(always)]
    pub fn is_some(&self) -> bool {
        self.0.read().unwrap().is_some()
    }

    /// Returns true if output value is not exists.
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        self.0.read().unwrap().is_none()
    }
}

