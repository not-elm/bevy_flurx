use std::collections::VecDeque;
use std::future::Future;

use bevy::prelude::{Resource, World};
use flurx::Scheduler;

use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;

#[derive(Default)]
pub(crate) struct ReactiveScheduler<'a, 'b> {
    schedulers: VecDeque<flurx::Scheduler<'a, 'b, WorldPtr>>,
}

impl<'a, 'b> ReactiveScheduler<'a, 'b>
    where 'a: 'b
{
    #[inline]
    pub fn schedule<F>(&mut self, world: WorldPtr, f: impl FnOnce(ReactiveTask<'a>) -> F + 'a)
        where F: Future + 'b
    {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(move |task| async move {
            f(ReactiveTask(task)).await;
        });
        scheduler.run_sync(world);
        self.schedulers.push_back(scheduler);
    }

    #[inline]
    pub(crate) fn run_sync(&mut self, world: &mut World) {
        let mut s = VecDeque::with_capacity(self.schedulers.len());
        while let Some(mut schedule) = self.schedulers.pop_front() {
            world.resource_mut::<Retry>().0 = true;
            while world.resource_mut::<Retry>().0{
                schedule.run_sync(WorldPtr::new(world));
            }
            if schedule.exists_pending_reactors(){
                s.push_back(schedule);
            }
        }
        self.schedulers = s;
    }
}


#[derive(Resource, Debug, Default)]
pub(crate) struct Retry(pub bool);