//! [`once::event`] creates a task that only once run system related to [`Event`].
//!
//! - [`once::event::send`]
//! - [`once::event::send_default`]
//! - [`once::event::app_exit_success`]

use bevy::app::AppExit;
use bevy::prelude::{Event, EventWriter, In};
use crate::action::seed::ActionSeed;
use crate::action::{once, Action};

/// Once send an event.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::event::send().with(AppExit::Success)).await;
/// });
/// ```
#[inline(always)]
pub fn send<E>() -> ActionSeed<E, ()>
where
    E: Event,
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
where
    E: Event + Default,
{
    once::run(|mut w: EventWriter<E>| {
        w.send(E::default());
    })
}

/// Once send [`AppExit::Success`].
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::event::app_exit_success()).await;
/// });
/// ```
#[inline(always)]
pub fn app_exit_success() -> Action<AppExit, ()> {
    send().with(AppExit::Success)
}


#[cfg(test)]
mod tests {
    use crate::action::once;
    use crate::prelude::Reactor;
    use crate::tests::{came_event, test_app};
    use bevy::app::{AppExit, First, Startup};
    use bevy::prelude::Commands;

    #[test]
    fn send_event() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(First, once::event::send().with(AppExit::Success)).await;
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
}