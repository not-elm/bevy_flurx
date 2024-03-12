use std::future::Future;

use crate::world_ptr::WorldPtr;
use crate::task::TaskCreator;


pub struct TaskScheduler<'a, 'b> {
    inner: flurx::Scheduler<'a, 'b, WorldPtr>,
}

impl<'a, 'b> TaskScheduler<'a, 'b>
    where 'a: 'b
{
    pub fn schedule<F>(&mut self, f: impl FnOnce(TaskCreator<'a>) -> F + 'a)
        where F: Future + 'b
    {
        self.inner.schedule(move |task| async move {
            f(TaskCreator {
                inner: task
            }).await;
        });
    }

    pub(crate) async fn run(&mut self, world: WorldPtr) {
        self.inner.run(world).await
    }
}

impl<'a, 'b> Default for TaskScheduler<'a, 'b>
    where 'a: 'b
{
    fn default() -> Self {
        Self{
            inner: flurx::Scheduler::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, Update};
    use bevy::prelude::NonSendMut;

    use crate::FlurxPlugin;
    use crate::scheduler::TaskScheduler;
    use crate::selector::once;

    #[test]
    fn count_up() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Update, |mut scheduler: NonSendMut<TaskScheduler>| {
                scheduler.schedule(|task| async move {
                    task.task_with(Update, AppExit, once::insert_non_send_resource()).await;
                });
            });

        app.update();
        app.update();
        app.update();
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}