//! Provides extension methods.

use std::future::Future;

use bevy::prelude::{Commands, EntityWorldMut, World};

use crate::prelude::Flurx;
use crate::scheduler::Initialized;
use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;

/// Provides a way to create a `Reactor` in the ecs systems.
///
/// This trait is implemented in [`World`] and [`Commands`].
///
/// It is possible to create a `reactor` via [`Commands`], but please note that in that cause there will be delay
/// because the command is entered into the command queue once and then the creation process is performed.
///
/// [`World`]: bevy::prelude::World
/// [`Commands`]: bevy::prelude::Commands
pub trait ScheduleReactor<'w, Fun, Fut, Out> {
    /// Create a [`Reactor`].
    /// [`Reactor`] represents the data flow
    fn schedule_reactor(&'w mut self, f: Fun) -> Out;
}

impl<'w, Fun, Fut> ScheduleReactor<'w, Fun, Fut, EntityWorldMut<'w>> for World
    where
        Fun: FnOnce(ReactiveTask) -> Fut + 'static,
        Fut: Future + 'static
{
    fn schedule_reactor(&'w mut self, f: Fun) -> EntityWorldMut<'w> {
        let mut flurx = Flurx::schedule(f);
        flurx.scheduler.run_sync(WorldPtr::new(self));
        self.spawn((
            Initialized,
            flurx
        ))
    }
}

impl<'w, 'b, F, Fut> ScheduleReactor<'w, F, Fut, ()> for Commands<'w, 'b>
    where
        F: FnOnce(ReactiveTask) -> Fut + Send + 'static,
        Fut: Future + 'static
{
    #[inline]
    fn schedule_reactor(&'w mut self, f: F) {
        self.add(|world: &mut World| {
            world.schedule_reactor(f);
        });
    }
}