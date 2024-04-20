//! [`once::event`] creates a task that only once run system related to [`Event`](bevy::prelude::Event).
//!
//! - [`once::event::send`]
//! - [`once::event::send_default`]
//! - [`once::event::app_exit`]


use bevy::app::AppExit;
use bevy::prelude::{Event, EventReader, EventWriter, In};

use crate::action::{Action, once};
use crate::action::seed::ActionSeed;

/// Once send an event.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::event::send().with(AppExit)).await;
/// });
/// ```
#[inline(always)]
pub fn send<E>() -> ActionSeed<E, ()>
    where E: Event
{
    once::run(|In(event): In<E>, mut w: EventWriter<E>| {
        w.send(event);
    })
}

/// Once send an event using [`Default`] trait.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::event::send_default::<AppExit>()).await;
/// });
/// ```
#[inline(always)]
pub fn send_default<E>() -> ActionSeed
    where E: Event + Default
{
    once::run(|mut w: EventWriter<E>| {
        w.send(E::default());
    })
}

/// Once send [`AppExit`](bevy::app::AppExit).
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::event::app_exit()).await;
/// });
/// ```
#[inline(always)]
pub fn app_exit() -> Action<AppExit, ()> {
    send().with(AppExit)
}


#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, First, Startup, Update};
    use bevy::prelude::{Commands, World};
    use bevy_test_helper::share::{create_shares, Share};

    use crate::action::once;
    use crate::reactor::Reactor;
    use crate::tests::{came_event, test_app};

    #[test]
    fn send_event() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(First, once::event::send().with(AppExit)).await;
            }));
        });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }

    #[test]
    fn send_default_event() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(First, once::event::send_default::<AppExit>()).await;
            }));
        });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }

    /// If register a reactor in `Startup` and execute `once::run`,
    /// make sure to proceed with subsequent processing during that frame.
    #[test]
    fn it_s1_to_be_true() {
        let mut app = test_app();
        let (s1, s2) = create_shares::<bool>();
        app.insert_resource(s2);
        app.add_systems(Startup, |world: &mut World| {
            let s2 = world.remove_resource::<Share<bool>>().unwrap();
            world.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::run(|| {})).await;
                s2.set(true);
            }));
        });

        app.update();
        assert!(*s1.lock());
    }
}