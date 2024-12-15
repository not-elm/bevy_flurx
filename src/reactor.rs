use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;
use bevy::prelude::{Component, Reflect, ReflectDefault};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default, Reflect, Eq, PartialEq, Hash, Copy, Clone)]
#[reflect(Default)]
pub(crate) struct ReactorId(u64);
impl ReactorId {
    #[inline]
    pub(crate) fn increment() -> Self {
        static TASK_ID: AtomicU64 = AtomicU64::new(0);
        Self(TASK_ID.fetch_add(1, Ordering::Relaxed))
    }
}


/// [`Reactor`] represents the asynchronous processing flow.
///
/// This structure is created by [`Reactor::schedule`] or [`ScheduleReactor`](crate::prelude::ScheduleReactor).
///
/// Remove this component if you want to interrupt the processing flow.
///
/// After all scheduled processes have completed, the entity attached to this component
/// and it's children will be despawn.
#[derive(Component)]
pub struct Reactor {
    pub(crate) scheduler: flurx::Scheduler<'static, 'static, WorldPtr>,
    pub(crate) initialized: bool,
    pub(crate) id: ReactorId,
}

impl Reactor {
    /// Create new [`Reactor`].
    ///
    /// The scheduled [`Reactor`] will be run and initialized at `RunReactor` schedule(and also initialized at [`PostStartup`](bevy::prelude::PostStartup)) ,
    ///
    /// It is recommended to spawn this structure at [`Update`](bevy::prelude::Update) or [`Startup`](bevy::prelude::Startup)
    /// to reduce the delay until initialization.
    ///
    /// If you spawn on another [`ScheduleLabel`](bevy::ecs::schedule::ScheduleLabel),
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
    ///         ew.send(AppExit::Success);
    ///     })).await;
    /// });
    /// ```
    pub fn schedule<F>(f: impl FnOnce(ReactiveTask) -> F + 'static) -> Reactor
    where
        F: Future,
    {
        let mut scheduler = flurx::Scheduler::new();
        let id = ReactorId::increment();
        scheduler.schedule(move |task| async move {
            f(ReactiveTask{
                task,
                id,
            }).await;
        });
        Self {
            scheduler,
            initialized: false,
            id,
        }
    }

    #[inline(always)]
    pub(crate) fn run_sync(&mut self, world: WorldPtr) -> bool {
        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        {
            use async_compat::CompatExt;
            pollster::block_on(self.scheduler.run(world).compat());
        }
        #[cfg(any(target_arch = "wasm32", not(feature = "tokio")))]
        {
            pollster::block_on(self.scheduler.run(world));
        }
        self.scheduler.not_exists_reactor()
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

    use crate::action::{delay, wait};
    use crate::prelude::{BoxedRunners, Reactor};
    use crate::tests::test_app;

    #[derive(Resource, Debug, Default, Eq, PartialEq)]
    struct Count(usize);

    #[test]
    fn cancel_if_reactor_removed() {
        let mut app = test_app();
        app.init_resource::<Count>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    wait::until(|mut count: ResMut<Count>| {
                        count.0 += 1;
                        false
                    }),
                )
                    .await;
            }));
        });
        app.update();
        app.assert_resource_eq(Count(1));

        app.world_mut()
            .run_system_once(|mut cmd: Commands, reactor: Query<Entity, With<Reactor>>| {
                cmd.entity(reactor.single()).remove::<Reactor>();
            })
            .expect("Failed to run system");
        for _ in 0..10 {
            app.update();
            app.assert_resource_eq(Count(1));
        }
    }

    #[test]
    fn despawn_after_finished_reactor() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, delay::frames().with(1)).await;
            }));
        });
        app.update();
        assert!(app
            .world_mut()
            .query::<&Reactor>()
            .get_single(app.world())
            .is_ok());
        assert_eq!(
            app.world()
                .non_send_resource::<BoxedRunners<Update>>()
                .0
                .len(),
            1
        );
        app.update();
        assert!(app
            .world_mut()
            .query::<&Reactor>()
            .get_single(app.world())
            .is_err());
        assert_eq!(
            app.world()
                .non_send_resource::<BoxedRunners<Update>>()
                .0
                .len(),
            0
        );
    }
}
