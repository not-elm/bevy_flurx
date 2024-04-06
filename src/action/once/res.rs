//! [`once::res`] creates a task that only once run system related to [`Resource`](bevy::prelude::Resource).
//!
//! - [`once::res::init`]
//! - [`once::res::insert`]
//! - [`once::res::remove`]

use bevy::prelude::{Commands, In, Resource};

use crate::action::{once, TaskAction};
use crate::action::seed::ActionSeed;
use crate::prelude::seed::Seed;

/// Once init a resource.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// #[derive(Resource, Default)]
/// struct Res;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, once::res::init::<Res>()).await;
/// });
/// ```
#[inline(always)]
pub fn init<R>() -> impl ActionSeed + Seed
    where R: Resource + Default + 'static
{
    once::run(|mut commands: Commands| {
        commands.init_resource::<R>();
    })
}

/// Once insert a resource.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// #[derive(Resource)]
/// struct Res;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, once::res::insert(Res)).await;
/// });
/// ```
#[inline(always)]
pub fn insert<R>(resource: R) -> impl TaskAction<R, ()>
    where R: Resource + 'static
{
    once::run_with(resource, |input: In<R>, mut commands: Commands| {
        commands.insert_resource(input.0);
    })
}

/// Once remove a resource.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// #[derive(Resource)]
/// struct Res;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, once::res::remove::<Res>()).await;
/// });
/// ```
#[inline(always)]
pub fn remove<R>() -> impl ActionSeed + Seed
    where R: Resource + 'static
{
    once::run(|mut commands: Commands| {
        commands.remove_resource::<R>();
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, First, Startup};
    use bevy::prelude::Commands;

    use crate::action::once::res;
    use crate::FlurxPlugin;
    use crate::scheduler::Flurx;
    use crate::tests::TestResource;

    #[test]
    fn init_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Flurx::schedule(|task| async move {
                    task.will(First, res::init::<TestResource>()).await;
                }));
            });

        app.update();
        assert!(app.world.get_resource::<TestResource>().is_some());
    }

    #[test]
    fn insert_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Flurx::schedule(|task| async move {
                    task.will(First, res::insert(TestResource)).await;
                }));
            });

        app.update();
        assert!(app.world.get_resource::<TestResource>().is_some());
    }

    #[test]
    fn remove_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .init_resource::<TestResource>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Flurx::schedule(|task| async move {
                    task.will(First, res::remove::<TestResource>()).await;
                }));
            });

        app.update();
        assert!(app.world.get_resource::<TestResource>().is_none());
    }
}