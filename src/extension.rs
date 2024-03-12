use std::future::Future;

use async_compat::CompatExt;
use bevy::prelude::World;

use crate::scheduler::TaskScheduler;
use crate::task::TaskCreator;
use crate::world_ptr::WorldPtr;

pub trait ScheduleReactor<'a, 'b> 
    where 
        'a: 'b,
{
    fn schedule<F>(&mut self, f: impl FnOnce(TaskCreator<'a>) -> F + 'a)
        where 
            F: Future<Output=()> + 'b;
}


impl<'a, 'b>  ScheduleReactor<'a, 'b> for World 
      where 'a: 'b,
{
    fn schedule<F>(&mut self, f: impl FnOnce(TaskCreator<'a>) -> F + 'a) where
        
        F: Future<Output=()> + 'b
    {
        let mut scheduler = self.get_non_send_resource_mut::<TaskScheduler>().unwrap();
        scheduler.schedule(f);
        pollster::block_on(scheduler.run(WorldPtr::new(self)).compat());

    }
}