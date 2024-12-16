//! [`once::res`] creates a task that only once run system related to [`Resource`].
//!
//! - [`once::res::init`]
//! - [`once::res::insert`]
//! - [`once::res::remove`]

use bevy::prelude::{Commands, In, Resource};

use crate::action::once;
use crate::action::seed::ActionSeed;

/// Once init a resource.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// #[derive(Resource, Default)]
/// struct Res;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, once::res::init::<Res>()).await;
/// });
/// ```
#[inline(always)]
pub fn init<R>() -> ActionSeed
where
    R: Resource + Default + 'static,
{
    once::run(|mut commands: Commands| {
        commands.init_resource::<R>();
    })
}

/// Once insert a resource.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// #[derive(Resource)]
/// struct Res;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, once::res::insert().with(Res)).await;
/// });
/// ```
#[inline(always)]
pub fn insert<R>() -> ActionSeed<R>
where
    R: Resource + 'static,
{
    once::run(|input: In<R>, mut commands: Commands| {
        commands.insert_resource(input.0);
    })
}

/// Once remove a resource.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// #[derive(Resource)]
/// struct Res;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, once::res::remove::<Res>()).await;
/// });
/// ```
#[inline(always)]
pub fn remove<R>() -> ActionSeed
where
    R: Resource + 'static,
{
    once::run(|mut commands: Commands| {
        commands.remove_resource::<R>();
    })
}


#[cfg(test)]
mod tests {
    use crate::action::once::res;
    use crate::tests::{test_app, TestResource};
    use bevy::app::{First, Startup};
    use bevy::prelude::Commands;

    #[test]
    fn init_resource() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, res::init::<TestResource>()).await;
            }));
        });

        app.update();
        assert!(app.world().get_resource::<TestResource>().is_some());
    }

    #[test]
    fn insert_resource() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, res::insert().with(TestResource)).await;
            }));
        });

        app.update();
        assert!(app.world().get_resource::<TestResource>().is_some());
    }

    #[test]
    fn remove_resource() {
        let mut app = test_app();
        app.init_resource::<TestResource>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                    task.will(First, res::remove::<TestResource>()).await;
                }));
            });

        app.update();
        assert!(app.world().get_resource::<TestResource>().is_none());
    }
}