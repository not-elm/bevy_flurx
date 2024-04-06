use std::future::Future;

use bevy::prelude::Component;

use crate::runner::CancellationToken;
use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;

#[derive(Component, Debug, Copy, Clone)]
pub(crate) struct Initialized;


/// [`Reactor`] represents the asynchronous processing flow.
///
/// This structure is created by [`Reactor::schedule`] or [`ScheduleReactor`](crate::prelude::ScheduleReactor).
/// 
/// Remove this component if you want to interrupt the processing flow.
/// 
#[derive(Component)]
pub struct Reactor {
    pub(crate) scheduler: flurx::Scheduler<'static, 'static, WorldPtr>,
    token: CancellationToken,
}

impl Reactor {
    /// Create new [`Reactor`].
    /// 
    /// The scheduled [`Reactor`] will be run and initialized at [`PostUpdate`](bevy::prelude::PostUpdate)(and also initialized at [`PostStartup`](bevy::prelude::PostStartup)) ,
    /// 
    /// It is recommended to spawn this structure at [`Update`](bevy::prelude::Update) or [`Startup`](bevy::prelude::Startup)
    /// to reduce the delay until initialization.
    /// 
    /// If you spawn on another [`ScheduleLabel`], 
    /// you can spawn and initialize at the same time by using [`ScheduleReactor`](crate::prelude::ScheduleReactor). 
    /// 
    /// ## Examples
    /// 
    /// ```no_run
    /// use bevy::app::AppExit;
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// Reactor::schedule(|task| async move{
    ///     task.will(Update, once::run(|mut ew: EventWriter<AppExit>|{
    ///         ew.send(AppExit);
    ///     })).await;
    /// });
    /// ```
    pub fn schedule<F>(f: impl FnOnce(ReactiveTask) -> F + 'static) -> Reactor
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

impl Drop for Reactor {
    fn drop(&mut self) {
        self.token.cancel();
    }
}


/// SAFETY: Scheduler must always run on the main thread only.
unsafe impl Send for Reactor {}

/// SAFETY: Scheduler must always run on the main thread only.
unsafe impl Sync for Reactor {}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, Entity, Query, ResMut, Resource, With};
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::once;
    use crate::prelude::Reactor;
    use crate::tests::test_app;

    #[derive(Resource, Debug, Default, Eq, PartialEq)]
    struct Count(usize);

    #[test]
    fn cancel_if_flurx_removed() {
        let mut app = test_app();
        app.init_resource::<Count>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::run(|mut count: ResMut<Count>| {
                    count.0 += 1;
                })).await;
            }));
        });
        app.update();
        app.assert_resource_eq(Count(1));

        app.world.run_system_once(|mut cmd: Commands, flurx: Query<Entity, With<Reactor>>| {
            cmd.entity(flurx.single()).remove::<Reactor>();
        });
        for _ in 0..10 {
            app.update();
            app.assert_resource_eq(Count(1));
        }
    }
}