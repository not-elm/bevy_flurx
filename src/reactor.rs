use crate::core::scheduler::CoreScheduler;
use crate::task::ReactorTask;
use crate::world_ptr::WorldPtr;
use bevy::app::{App, Plugin};
use bevy::ecs::component::{ComponentHooks, HookContext, Mutable, StorageType};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use core::future::Future;
use core::marker::PhantomData;

/// This event triggers the execution of the [`Reactor`].
///
/// If you want to perform asynchronous processing other than Action in the reactor, you need to manually advance the reactor using this event.
/// [`StepReactor`] can be used instead if you want to advance only a single reactor.
#[derive(Event, Reflect, Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[reflect(Debug, PartialEq, Hash)]
pub struct StepAllReactors;

/// This event triggers the execution of the [`Reactor`].
#[derive(Event, Reflect, Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[reflect(Debug, PartialEq, Hash)]
pub struct StepReactor {
    /// The entity of the reactor to be executed
    pub reactor: Entity,
}

pub(crate) struct ReactorPlugin;

impl Plugin for ReactorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StepAllReactors>()
            .register_type::<StepReactor>()
            .add_event::<StepReactor>()
            .add_event::<StepAllReactors>()
            .add_observer(trigger_step_reactor)
            .add_observer(trigger_step_all_reactors);
    }
}

/// [`Reactor`] represents the asynchronous processing flow.
///
/// This structure is created by [`Reactor::schedule`].
///
/// Despawn the entity attached this component if you want to interrupt the processing flow.
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
    /// Create new [`Reactor`].
    ///
    /// The scheduled [`Reactor`] will be run and initialized at [Last] schedule(and also initialized at [`PostStartup`]) ,
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
    ///         ew.write(AppExit::Success);
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
    type Mutability = Mutable;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world: DeferredWorld, context: HookContext| {
            let entity = context.entity;
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
            world
                .commands()
                .entity(entity)
                .insert(NativeReactor::schedule(entity, f));
        });
    }
}

pub(crate) struct NativeReactor {
    pub(crate) scheduler: CoreScheduler<WorldPtr>,
}

impl NativeReactor {
    fn schedule<F>(
        entity: Entity,
        f: impl FnOnce(ReactorTask) -> F + Send + Sync + 'static,
    ) -> NativeReactor
    where
        F: Future + Send + Sync,
    {
        let scheduler = CoreScheduler::schedule(move |task| async move {
            f(ReactorTask { task, entity }).await;
        });
        Self { scheduler }
    }

    #[inline(always)]
    pub(crate) fn step(&mut self, world: WorldPtr) -> bool {
        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        {
            use async_compat::CompatExt;
            pollster::block_on(self.scheduler.run(world).compat());
        }
        #[cfg(any(target_arch = "wasm32", not(feature = "tokio")))]
        {
            pollster::block_on(self.scheduler.run(world));
        }
        self.scheduler.finished
    }
}

impl Component for NativeReactor {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world: DeferredWorld, context: HookContext| {
            let entity = context.entity;
            world.commands().queue(move |world: &mut World| {
                step_reactor(entity, world);
            });
        });
    }
}

fn trigger_step_reactor(trigger: Trigger<StepReactor>, mut commands: Commands) {
    let reactor_entity = trigger.reactor;
    commands.queue(move |world: &mut World| {
        step_reactor(reactor_entity, world);
    });
}

fn trigger_step_all_reactors(_: Trigger<StepAllReactors>, mut commands: Commands) {
    commands.queue(move |world: &mut World| {
        let world_ptr = WorldPtr::new(world);
        let mut finished_reactors = Vec::new();
        for (entity, mut reactor) in world
            .query::<(Entity, &mut NativeReactor)>()
            .iter_mut(world)
        {
            if reactor.step(world_ptr) {
                finished_reactors.push(entity);
            }
        }
        for entity in finished_reactors {
            world.commands().entity(entity).despawn();
        }
    });
}

#[inline]
fn step_reactor(reactor_entity: Entity, world: &mut World) {
    let world_ptr = WorldPtr::new(world);
    if let Ok(mut reactor) = world
        .query::<&mut NativeReactor>()
        .get_mut(world, reactor_entity)
    {
        if reactor.step(world_ptr) {
            world.commands().entity(reactor_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::{delay, once, wait};
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
            .run_system_once(
                |mut cmd: Commands, reactor: Query<Entity, With<NativeReactor>>| {
                    cmd.entity(reactor.single().unwrap()).despawn();
                },
            )
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
            .single(app.world())
            .is_ok());
        app.update();
        assert!(app
            .world_mut()
            .query::<&NativeReactor>()
            .single(app.world())
            .is_err());
    }

    #[test]
    fn not_overwrite_schedule_label() {
        let mut app = test_app();
        #[derive(Debug, Copy, Clone, Resource, Eq, PartialEq)]
        struct Bool2(bool);

        app.insert_resource(Count(0));
        app.insert_resource(Bool2(false));

        app.add_systems(
            Update,
            (
                |mut count: ResMut<Count>| count.0 += 1,
                |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        task.will(Update, once::res::insert().with(Bool2(true)))
                            .await;
                    }));
                },
            ),
        );

        app.update();
        app.update();
        app.update();
        app.update();
        app.update();
        app.update();
        // app.assert_resource_eq(Count(2));
        app.assert_resource_eq(Bool2(true));
    }
}
