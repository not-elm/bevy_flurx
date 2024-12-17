use crate::runner::CancellationHandlers;
use crate::task::ReactorTask;
use crate::world_ptr::WorldPtr;
use bevy::ecs::component::{ComponentHooks, StorageType};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{Component, Entity, ReflectComponent};
use bevy::reflect::Reflect;
use std::future::Future;
use std::marker::PhantomData;

/// [`NativeReactor`] represents the asynchronous processing flow.
///
/// This structure is created by [`NativeReactor::schedule`] or [`ScheduleReactor`](crate::prelude::ScheduleReactor).
///
/// Remove this component if you want to interrupt the processing flow.
///
/// After all scheduled processes have completed, the entity attached to this component
/// and it's children will be despawn.
#[derive(Reflect)]
#[reflect(Component)]
pub struct Reactor<F, Fut>
where
    F: FnOnce(ReactorTask) -> Fut + Send + Sync + 'static,
    Fut: Future + Send + Sync + 'static,
{
    f: Option<F>,
    _m: PhantomData<Fut>,
}

impl<F, Fut> Reactor<F, Fut>
where
    F: FnOnce(ReactorTask) -> Fut + Send + Sync + 'static,
    Fut: Future + Send + Sync + 'static,
{
    /// Create new [`NativeReactor`].
    ///
    /// The scheduled [`NativeReactor`] will be run and initialized at `RunReactor` schedule(and also initialized at [`PostStartup`](bevy::prelude::PostStartup)) ,
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
    pub fn schedule(f: F) -> Reactor<F, Fut> {
        Self {
            f: Some(f),
            _m: PhantomData,
        }
    }
}

impl<F, Fut> Component for Reactor<F, Fut>
where
    F: FnOnce(ReactorTask) -> Fut + Send + Sync + 'static,
    Fut: Future + Send + Sync + 'static,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .on_add(|mut world: DeferredWorld, entity: Entity, _| {
                let f = {
                    let mut entity_mut = world.entity_mut(entity);
                    let Some(mut flow) = entity_mut.get_mut::<Reactor<F, Fut>>() else {
                        return;
                    };
                    let Some(f) = flow.f.take() else {
                        return;
                    };
                    f
                };
                world.commands().entity(entity).insert((
                    NativeReactor::schedule(entity, f),
                    CancellationHandlers::default(),
                ));
            });
    }
}


/// [`NativeReactor`] represents the asynchronous processing flow.
///
/// This structure is created by [`NativeReactor::schedule`] or [`ScheduleReactor`](crate::prelude::ScheduleReactor).
///
/// Remove this component if you want to interrupt the processing flow.
///
/// After all scheduled processes have completed, the entity attached to this component
/// and it's children will be despawn.
#[derive(Component)]
pub(crate) struct NativeReactor {
    pub(crate) scheduler: flurx::Scheduler<WorldPtr>,
    pub(crate) initialized: bool,
}

impl NativeReactor {
    /// Create new [`NativeReactor`].
    ///
    /// The scheduled [`NativeReactor`] will be run and initialized at `RunReactor` schedule(and also initialized at [`PostStartup`](bevy::prelude::PostStartup)) ,
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
    pub fn schedule<F>(entity: Entity, f: impl FnOnce(ReactorTask) -> F + Send + Sync + 'static) -> NativeReactor
    where
        F: Future + Send + Sync,
    {
        let mut scheduler = flurx::Scheduler::new();
        scheduler.schedule(move |task| async move {
            f(ReactorTask {
                task,
                entity,
            }).await;
        });
        Self {
            scheduler,
            initialized: false,
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


#[cfg(test)]
mod tests {
    use crate::action::{delay, wait};
    use crate::prelude::Reactor;
    use crate::reactor::NativeReactor;
    use crate::tests::test_app;
    use bevy::app::{Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, Entity, Query, ResMut, Resource, With};
    use bevy_test_helper::resource::DirectResourceControl;

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
            .run_system_once(|mut cmd: Commands, reactor: Query<Entity, With<NativeReactor>>| {
                cmd.entity(reactor.single()).despawn();
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
            .query::<&NativeReactor>()
            .get_single(app.world())
            .is_ok());
        app.update();
        assert!(app
            .world_mut()
            .query::<&NativeReactor>()
            .get_single(app.world())
            .is_err());
    }
}
