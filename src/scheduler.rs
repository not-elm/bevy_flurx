use std::future::Future;

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
        self.token.cancel();
    }
}

unsafe impl Send for Flurx {}

unsafe impl Sync for Flurx {}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, Entity, Query, ResMut, Resource, With};
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::once;
    use crate::prelude::Flurx;
    use crate::tests::test_app;

    #[derive(Resource, Debug, Default, Eq, PartialEq)]
    struct Count(usize);

    #[test]
    fn cancel_if_flurx_removed() {
        let mut app = test_app();
        app.init_resource::<Count>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Flurx::schedule(|task| async move {
                task.will(Update, once::run(|mut count: ResMut<Count>| {
                    count.0 += 1;
                })).await;
            }));
        });
        app.update();
        app.assert_resource_eq(Count(1));

        app.world.run_system_once(|mut cmd: Commands, flurx: Query<Entity, With<Flurx>>| {
            cmd.entity(flurx.single()).remove::<Flurx>();
        });
        for _ in 0..10{
            app.update();
            app.assert_resource_eq(Count(1));
        }
    }
}