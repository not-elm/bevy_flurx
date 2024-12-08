//! Provides extension methods.

use std::future::Future;

use bevy::prelude::{Commands, EntityWorldMut, World};

use crate::prelude::Reactor;
use crate::task::ReactiveTask;
use crate::world_ptr::WorldPtr;

/// Provides a way to create and initialize [`Reactor`] in the ecs systems.
///
/// This trait is implemented in [`World`] and [`Commands`].
///
/// [`World`]: bevy::prelude::World
/// [`Commands`]: bevy::prelude::Commands
pub trait ScheduleReactor<'w, Fun, Fut, Out: 'w> {
    /// Create and initialize [`Reactor`].
    fn spawn_initialized_reactor(self, f: Fun) -> Out;
}

impl<'w, Fun, Fut> ScheduleReactor<'w, Fun, Fut, EntityWorldMut<'w>> for &'w mut World
where
    Fun: FnOnce(ReactiveTask) -> Fut + 'static,
    Fut: Future + 'static,
{
    fn spawn_initialized_reactor(self, f: Fun) -> EntityWorldMut<'w> {
        let mut reactor = Reactor::schedule(f);
        reactor.run_sync(WorldPtr::new(self));
        reactor.initialized = true;
        self.spawn(reactor)
    }
}

impl<'w, F, Fut> ScheduleReactor<'w, F, Fut, ()> for &mut Commands<'w, '_>
where
    F: FnOnce(ReactiveTask) -> Fut + Send + 'static,
    Fut: Future + 'static,
{
    #[inline]
    fn spawn_initialized_reactor(self, f: F) {
        self.queue(|world: &mut World| {
            world.spawn_initialized_reactor(f);
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, Update, World};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::extension::ScheduleReactor;
    use crate::tests::{increment_count, test_app};

    #[test]
    fn world_extension() {
        let mut app = test_app();
        app.update();
        app.world_mut()
            .run_system_once(|world: &mut World| {
                world.spawn_initialized_reactor(|task| async move {
                    task.will(Update, increment_count()).await;
                });
            })
            .expect("Failed to run system");
        app.update();
        app.assert_resource_eq(Count(1));
    }

    #[test]
    fn commands_extension() {
        let mut app = test_app();
        app.update();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn_initialized_reactor(|task| async move {
                    task.will(Update, increment_count()).await;
                });
            })
            .expect("Failed to run system");
        app.update();
        app.assert_resource_eq(Count(1));
    }
}
