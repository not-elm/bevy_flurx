//! [`once::non_send`] creates a task that only once run system related to [`non-send resources`](bevy::prelude::NonSend).
//!
//! - [`once::non_send::init`]
//! - [`once::non_send::insert`]
//! - [`once::non_send::remove`]


use bevy::prelude::{In, World};
use crate::selector::condition::{once, ReactorSystemConfigs, with};

/// Once init a non-send resource.
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(Default)]
/// struct R;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         task.will(Update, once::non_send::init::<R>()).await;
///     });
/// });
/// app.update();
/// ```
#[inline]
pub fn init<R>() -> impl ReactorSystemConfigs<In=()>
    where R: Default + 'static
{
    with((), once::run(|world: &mut World| {
        world.init_non_send_resource::<R>();
    }))
}

/// Once insert a non-send resource.
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(Clone)]
/// struct R;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         task.will(Update, once::non_send::insert(R)).await;
///     });
/// });
/// app.update();
/// ```
#[inline]
pub fn insert<R>(resource: R) -> impl ReactorSystemConfigs<In=R>
    where R: Clone + 'static
{
    with(resource, once::run(|In(resource): In<R>, world: &mut World| {
        world.insert_non_send_resource(resource);
    }))
}

/// Once remove a non-send resource.
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct R;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         task.will(Update, once::non_send::remove::<R>()).await;
///     });
/// });
/// app.update();
/// ```
#[inline]
pub fn remove<R>() -> impl ReactorSystemConfigs<In=()>
    where R: 'static
{
    with((), once::run(|world: &mut World| {
        world.remove_non_send_resource::<R>();
    }))
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup};
    use bevy::prelude::World;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::once::non_send;
    use crate::tests::TestResource;

    #[test]
    fn init_non_send_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, non_send::init::<TestResource>()).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<TestResource>().is_some());
    }

    #[test]
    fn insert_non_send_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, non_send::insert(TestResource)).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<TestResource>().is_some());
    }


    #[test]
    fn remove_non_send_resource() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .init_resource::<TestResource>()
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, non_send::remove::<TestResource>()).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<TestResource>().is_none());
    }

    #[test]
    fn success_run_all_schedule_labels() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, non_send::insert(AppExit)).await;
                    println!("First finished");
                    task.will(First, non_send::insert(AppExit)).await;
                    println!("PreUpdate finished");
                    task.will(First, non_send::insert(AppExit)).await;
                    println!("Update finished");
                    task.will(First, non_send::insert(AppExit)).await;
                    println!("PostUpdate finished");
                    task.will(First, non_send::insert(AppExit)).await;
                    println!("Last finished");
                });
            });

        println!("First");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("PreUpdate");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("Update");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("PostUpdate");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("Last");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_some());

        println!("After Reactor Finished");
        app.update();
        assert!(app.world.remove_non_send_resource::<AppExit>().is_none());
    }
}