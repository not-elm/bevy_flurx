use std::future::Future;

use crate::store::WorldPointer;
use crate::task::BevyTask;

#[derive(Default)]
pub struct BevyScheduler<'a, 'b> {
    inner: store::Scheduler<'a, 'b, WorldPointer>,
}

impl<'a, 'b> BevyScheduler<'a, 'b>
    where 'a: 'b
{
    pub fn schedule<F>(&mut self, f: impl FnOnce(BevyTask<'a>) -> F + 'a)
        where F: Future + 'b
    {
        self.inner.schedule(move |task| async move {
            f(BevyTask {
                inner: task
            }).await;
        });
    }

    pub(crate) async fn run(&mut self, world: WorldPointer) {
        self.inner.run(world).await
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, Update};
    use bevy::prelude::NonSendMut;

    use crate::AsyncSystemPlugin;
    use crate::scheduler::BevyScheduler;
    use crate::task::once;

    #[test]
    fn count_up() {
        let mut app = App::new();
        app
            .add_plugins(AsyncSystemPlugin)
            .insert_non_send_resource(BevyScheduler::default())
            .add_systems(Update, |mut scheduler: NonSendMut<BevyScheduler>| {
                scheduler.schedule(|task| async move {
                    task.run(once::insert_non_send_resource(AppExit)).await;
                });
            });

        app.update();
        app.update();
        app.update();
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}