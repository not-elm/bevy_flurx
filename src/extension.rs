use std::future::Future;

use bevy::prelude::{Commands, World};

use crate::scheduler::ReactiveScheduler;
use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;

pub trait ScheduleReactor<Fun, Fut, Out> {
    fn schedule_reactor(&mut self, f: Fun) -> Out;
}


impl<Fun, Fut> ScheduleReactor<Fun, Fut, ()> for World
    where
        Fun: FnOnce(ReactiveTask<'static>) -> Fut + 'static,
        Fut: Future + 'static
{
    fn schedule_reactor(&mut self, f: Fun) {
        let world_ptr = WorldPtr::new(self);
        let mut scheduler = self.get_non_send_resource_mut::<ReactiveScheduler>().unwrap();
        scheduler.schedule(f);
        scheduler.run_sync(world_ptr);
    }
}


impl<'a, 'b, F, Fut> ScheduleReactor<F, Fut, ()> for Commands<'a, 'b> 
    where 
        F: FnOnce(ReactiveTask<'static>) -> Fut + Send + 'static,
        Fut: Future + 'static
{
    fn schedule_reactor(&mut self, f: F) {
        self.add(|world: &mut World|{
            world.schedule_reactor(f);
        });
    }
}