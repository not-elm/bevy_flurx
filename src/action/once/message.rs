//! [`once::message`] creates a task that only once run system related to [`Message`].

use crate::action::seed::ActionSeed;
use crate::action::{once, Action};
use bevy::app::AppExit;
use bevy::prelude::*;

/// Once write a message.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::message::write().with(AppExit::Success)).await;
/// });
/// ```
#[inline(always)]
pub fn write<M>() -> ActionSeed<M, ()>
where
    M: Message,
{
    once::run(|In(message): In<M>, mut w: MessageWriter<M>| {
        w.write(message);
    })
}

/// Once write a message using [`Default`] trait.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::message::write_default::<AppExit>()).await;
/// });
/// ```
#[inline(always)]
pub fn write_default<M>() -> ActionSeed
where
    M: Message + Default,
{
    once::run(|mut w: MessageWriter<M>| {
        w.write(M::default());
    })
}

/// Once write [`AppExit::Success`].
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::message::app_exit_success()).await;
/// });
/// ```
#[inline(always)]
pub fn app_exit_success() -> Action<AppExit, ()> {
    write().with(AppExit::Success)
}

#[cfg(test)]
mod tests {
    use crate::action::once;
    use crate::prelude::Reactor;
    use crate::tests::{came_event, test_app};
    use bevy::app::{AppExit, First, Startup};
    use bevy::prelude::Commands;

    #[test]
    fn write_message() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(First, once::message::write().with(AppExit::Success))
                    .await;
            }));
        });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }

    #[test]
    fn write_default_message() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(First, once::message::write_default::<AppExit>())
                    .await;
            }));
        });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }
}
