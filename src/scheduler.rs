use std::future::Future;

use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;

pub(crate) struct ReactiveScheduler<'a, 'b> {
    inner: flurx::Scheduler<'a, 'b, WorldPtr>,
}

impl<'a, 'b> ReactiveScheduler<'a, 'b>
    where 'a: 'b
{
    pub fn schedule<F>(&mut self, f: impl FnOnce(ReactiveTask<'a>) -> F + 'a)
        where F: Future + 'b
    {
        self.inner.schedule(move |task| async move {
            f(ReactiveTask {
                inner: task
            }).await;
        });
    }

    pub(crate) fn run_sync(&mut self, world: WorldPtr) {
        self.inner.run_sync(world)
    }
}

impl<'a, 'b> Default for ReactiveScheduler<'a, 'b>
    where 'a: 'b
{
    fn default() -> Self {
        Self {
            inner: flurx::Scheduler::new()
        }
    }
}

