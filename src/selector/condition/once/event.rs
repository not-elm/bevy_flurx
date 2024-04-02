//! [`once::event`] creates a task that only once run system related to [`Event`](bevy::prelude::Event).
//!
//! - [`once::event::send`]
//! - [`once::event::send_default`]
//! - [`once::event::app_exit`]


use bevy::app::AppExit;
use bevy::prelude::{Event, EventWriter, In};

use crate::selector::condition::{once, ReactorSystemConfigs, with};


/// Once send an event.
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         task.will(Update, once::event::send(AppExit)).await;
///     });
/// });
/// app.update();
/// ```
#[inline]
pub fn send<E>(event: E) -> impl ReactorSystemConfigs<In=E>
    where E: Event + Clone
{
    with(event, once::run(|In(event): In<E>, mut w: EventWriter<E>| {
        w.send(event);
    }))
}

/// Once send an event using [`Default`] trait.
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         task.will(Update, once::event::send_default::<AppExit>()).await;
///     });
/// });
/// app.update();
/// ```
#[inline]
pub fn send_default<E>() -> impl ReactorSystemConfigs<In=()>
    where E: Event + Default
{
    with((), once::run(|mut w: EventWriter<E>| {
        w.send(E::default());
    }))
}

/// Once send [`AppExit`](bevy::app::AppExit).
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         task.will(Update, once::event::app_exit()).await;
///     });
/// });
/// app.update();
/// ```
#[inline]
pub fn app_exit() -> impl ReactorSystemConfigs<In=AppExit> {
    send(AppExit)
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup};
    use bevy::prelude::World;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::once;
    use crate::tests::came_event;

    #[test]
    fn send_event() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, once::event::send(AppExit)).await;
                });
            });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }
    
    #[test]
    fn send_default_event() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, once::event::send_default::<AppExit>()).await;
                });
            });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }
}