use std::future::Future;
use bevy::log::info;

use bevy::prelude::Component;

use crate::runner::CancellationToken;
use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;


#[derive(Component, Debug)]
pub(crate) struct Initialized;


#[derive(Component)]
pub struct Flurx {
    pub(crate) scheduler: flurx::Scheduler<'static, 'static, WorldPtr>,
    token: CancellationToken,
}

impl Flurx {
    pub fn schedule<F>(f: impl FnOnce(ReactiveTask) -> F + 'static) -> Flurx
        where F: Future
    {
        let mut scheduler = flurx::Scheduler::new();
        let token = CancellationToken::default();
        let t1 = token.clone();
        scheduler.schedule(move |task| async move {
            f(ReactiveTask {
                task,
                token: t1,
            }).await;
        });
        Self {
            scheduler,
            token,
        }
    }
}

impl Drop for Flurx {
    fn drop(&mut self) {
        info!("DROP");
        self.token.cancel();
    }
}

unsafe impl Send for Flurx {}

unsafe impl Sync for Flurx {}


#[cfg(test)]
mod tests {
    use bevy::app::Update;
    use bevy::prelude::Commands;

    use crate::action::once;
    use crate::prelude::Flurx;
    use crate::tests::test_app;

    #[test]
    fn it() {
        let mut app = test_app();
        app.add_systems(Update, |mut commands: Commands| {
            commands.spawn(Flurx::schedule(|task| async move {
                task.will(Update, once::run(|| {})).await;
            }));
        });
    }
}