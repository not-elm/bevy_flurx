use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy::prelude::World;
use bevy::utils::HashMap;


/// The cancellation handler id assigned by [`CancellationToken`].
///
/// For unregister the handler, call [`CancellationToken::unregister`] with this id.
#[repr(transparent)]
#[derive(Default, Debug, Eq, PartialEq, Hash)]
pub struct CancellationId(u64);


/// Structure for canceling a [`Reactor`](crate::prelude::Reactor).
///
/// This is passed as argument in [`Runner::run`](crate::prelude::Runner::run),
/// and the [`Reactor`](crate::prelude::Reactor) can be cancelled by calling [`CancellationToken::cancel`]. 
#[repr(transparent)]
#[derive(Default, Debug)]
pub struct CancellationToken(Rc<ReactorStatus>);

impl CancellationToken {
    /// Register a function that will be called when [`CancellationToken`] is cancelled.
    #[inline]
    pub fn register(&self, f: impl FnOnce(&mut World) + 'static) -> CancellationId {
        self.0.register(f)
    }

    /// Unregister a cancellation handler related to [`CancellationId`].
    #[inline]
    pub fn unregister(&self, id: &CancellationId){
        self.0.cancel_handles.borrow_mut().remove(id);
    }

    /// Requests to cancel a [`Reactor`](crate::prelude::Reactor).
    #[inline]
    pub fn cancel(&self) {
        self.0.is_cancellation_requested.set(true);
    }

    /// Returns `true` if cancellation has been requested for a [`Reactor`](crate::prelude::Reactor).
    ///
    /// Becomes `true` when [`CancellationToken::cancel`] is called or removed [`Reactor`](crate::prelude::Reactor)
    /// before it processing is completed. 
    #[must_use]
    #[inline(always)]
    pub fn is_cancellation_requested(&self) -> bool {
        self.0.is_cancellation_requested.get()
    }

    #[inline]
    pub(crate) fn call_cancel_handles(&self, world: &mut World) {
        for (_, handle) in self.0.cancel_handles.take() {
            (handle)(world);
        }
    }

    #[must_use]
    #[inline(always)]
    pub(crate) fn finished_reactor(&self) -> bool {
        self.0.reactor_finished.get()
    }
}

impl Clone for CancellationToken {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

#[derive(Default)]
pub(crate) struct ReactorStatus {
    pub cancellation_id: AtomicU64,
    pub cancel_handles: RefCell<HashMap<CancellationId, Box<dyn FnOnce(&mut World)>>>,
    pub is_cancellation_requested: Cell<bool>,
    pub reactor_finished: Cell<bool>,
}

impl ReactorStatus{
    fn register(&self, f: impl FnOnce(&mut World) + 'static) -> CancellationId{
        let id = self.cancellation_id.fetch_add(1, Ordering::Relaxed);
        self.cancel_handles.borrow_mut().insert(CancellationId(id), Box::new(f));
        CancellationId(id)
    }
}

impl Debug for ReactorStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("ReactorStatus")
            .field("cancellation_id", &self.cancellation_id.load(Ordering::Relaxed))
            .field("is_cancellation_requested", &self.is_cancellation_requested.get())
            .field("reactor_finished", &self.reactor_finished.get())
            .finish()
    }
}

