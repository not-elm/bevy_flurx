//! [`once::non_send`] creates a task that only once run system related to [`non-send resources`](bevy::prelude::NonSend).
//!
//! - [`once::non_send::init`]
//! - [`once::non_send::insert`]
//! - [`once::non_send::remove`]

use bevy::prelude::{In, World};

use crate::action::once;
use crate::action::seed::ActionSeed;

/// Once init a non-send resource.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(Default)]
/// struct Res;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, once::non_send::init::<Res>()).await;
/// });
/// ```
#[inline(always)]
pub fn init<R>() -> ActionSeed
where
    R: Default + 'static,
{
    once::run(|world: &mut World| {
        world.init_non_send_resource::<R>();
    })
}

/// Once insert a non-send resource.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct Res;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, once::non_send::insert().with(Res)).await;
/// });
/// ```
#[inline(always)]
pub fn insert<R>() -> ActionSeed<R>
where
    R: 'static,
{
    once::run(|In(resource): In<R>, world: &mut World| {
        world.insert_non_send_resource(resource);
    })
}

/// Once remove a non-send resource.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct Res;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, once::non_send::remove::<Res>()).await;
/// });
/// ```
#[inline(always)]
pub fn remove<R>() -> ActionSeed
where
    R: 'static,
{
    once::run(|world: &mut World| {
        world.remove_non_send_resource::<R>();
    })
}

#[cfg(test)]
mod tests {
    use crate::action::once::non_send;
    use crate::tests::{test_app, TestResource};
    use bevy::app::{AppExit, First, PostUpdate, PreUpdate, Startup, Update};
    use bevy::prelude::Commands;

    #[test]
    fn init_non_send_resource() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, non_send::init::<TestResource>()).await;
            }));
        });

        app.update();
        assert!(app
            .world()
            .get_non_send_resource::<TestResource>()
            .is_some());
    }

    #[test]
    fn insert_non_send_resource() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, non_send::insert().with(TestResource))
                    .await;
            }));
        });

        app.update();
        assert!(app
            .world()
            .get_non_send_resource::<TestResource>()
            .is_some());
    }

    #[test]
    fn remove_non_send_resource() {
        let mut app = test_app();
        app.init_resource::<TestResource>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                    task.will(First, non_send::remove::<TestResource>()).await;
                }));
            });

        app.update();
        assert!(app
            .world()
            .get_non_send_resource::<TestResource>()
            .is_none());
    }

    #[test]
    fn success_run_all_schedule_labels() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, non_send::insert().with(AppExit::Success))
                    .await;
                println!("First finished");
                task.will(PreUpdate, non_send::insert().with(AppExit::Success))
                    .await;
                println!("PreUpdate finished");
                task.will(Update, non_send::insert().with(AppExit::Success))
                    .await;
                println!("Update finished");
                task.will(PostUpdate, non_send::insert().with(AppExit::Success))
                    .await;
                println!("PostUpdate finished");
            }));
        });

        println!("First");
        app.update();
        assert!(app
            .world_mut()
            .remove_non_send_resource::<AppExit>()
            .is_some());

        println!("PreUpdate");
        app.update();
        assert!(app
            .world_mut()
            .remove_non_send_resource::<AppExit>()
            .is_some());

        println!("Update");
        app.update();
        assert!(app
            .world_mut()
            .remove_non_send_resource::<AppExit>()
            .is_some());

        println!("PostUpdate");
        app.update();
        assert!(app
            .world_mut()
            .remove_non_send_resource::<AppExit>()
            .is_some());

        println!("After Reactor Finished");
        app.update();
        assert!(app
            .world_mut()
            .remove_non_send_resource::<AppExit>()
            .is_none());
    }
}
