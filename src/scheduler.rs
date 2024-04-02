use std::future::Future;

use flurx::Scheduler;

use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;

#[derive(Default)]
pub(crate) struct ReactiveScheduler<'a, 'b> {
    non_initializes: Vec<flurx::Scheduler<'a, 'b, WorldPtr>>,
    pending: Vec<flurx::Scheduler<'a, 'b, WorldPtr>>,
}

impl<'a, 'b> ReactiveScheduler<'a, 'b>
    where 'a: 'b
{
    #[inline]
    pub fn schedule<F>(&mut self, f: impl FnOnce(ReactiveTask<'a>) -> F + 'a)
        where F: Future + 'b
    {
        let mut scheduler = Scheduler::new();
        scheduler.schedule(move |task| async move {
            f(ReactiveTask(task)).await;
        });
        self.non_initializes.push(scheduler);
    }

    #[inline]
    pub(crate) fn initialize(&mut self, world: WorldPtr) {
        while let Some(mut scheduler) = self.non_initializes.pop() {
            scheduler.run_sync(world);
            self.pending.push(scheduler);
        }
    }

    #[inline]
    pub(crate) fn run_sync(&mut self, world: WorldPtr) {
        self.pending.iter_mut().for_each(|scheduler| {
            scheduler.run_sync(world);
        });
    }
}


