//! Provides extension methods.

use std::future::Future;

use bevy::prelude::{Commands, World};

use crate::scheduler::ReactiveScheduler;
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
pub trait ScheduleReactor<Fun, Fut, Out> {
    /// Create a [`Reactor`].
    /// [`Reactor`] represents the data flow 
    fn schedule_reactor(&mut self, f: Fun) -> Out;
}

impl<Fun, Fut> ScheduleReactor<Fun, Fut, ()> for World
    where
        Fun: FnOnce(ReactiveTask<'static>) -> Fut + 'static,
        Fut: Future + 'static
{
    fn schedule_reactor(&mut self, f: Fun) {
        self.init_non_send_resource::<ReactiveScheduler>();
        let world_ptr = WorldPtr::new(self);
        let mut scheduler = self.get_non_send_resource_mut::<ReactiveScheduler>().unwrap();
        scheduler.schedule(world_ptr, f);
    }
}

impl<'a, 'b, F, Fut> ScheduleReactor<F, Fut, ()> for Commands<'a, 'b>
    where
        F: FnOnce(ReactiveTask<'static>) -> Fut + Send + 'static,
        Fut: Future + 'static
{
    #[inline]
    fn schedule_reactor(&mut self, f: F) {
        self.add(|world: &mut World| {
            world.schedule_reactor(f);
        });
    }
}