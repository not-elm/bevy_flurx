use bevy::prelude::{Component, Event, World};
use bevy::utils::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

/// The cancellation handler id assigned by [`CancellationHandlers`].
///
/// For unregister the handler, call [`CancellationHandlers::unregister`] with this id.
#[repr(transparent)]
#[derive(Default, Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct CancellationId(u64);

/// Structure for canceling a [`Reactor`](crate::prelude::Reactor).
///
/// This is passed as argument in [`Runner::run`](crate::prelude::Runner::run),
/// and the [`Reactor`](crate::prelude::Reactor) can be cancelled by calling [`CancellationHandlers::cancel`].
#[repr(transparent)]
#[derive(Default, Component)]
pub struct CancellationHandlers(pub(crate) HashMap<CancellationId, fn(&mut World)>);

impl CancellationHandlers {
    /// Register a function that will be called when [`CancellationHandlers`] is cancelled.
    #[inline]
    pub fn register(&mut self, f: fn(&mut World)) -> CancellationId {
        static ID: AtomicU64 = AtomicU64::new(0);
        let id = CancellationId(ID.fetch_add(1, Ordering::Relaxed));
        self.0.insert(id, f);
        id
    }

    /// Unregister a cancellation handler related to [`CancellationId`].
    #[inline]
    pub fn unregister(&mut self, id: &CancellationId) {
        self.0.remove(id);
    }
}

#[repr(transparent)]
#[derive(Event)]
pub(crate) struct CallCancellationHandlers(pub(crate) CancellationHandlers);