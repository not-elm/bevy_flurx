//! `delay` creates a task that delay the application.

use crate::action::wait;
use crate::prelude::ActionSeed;
use bevy::prelude::*;
use bevy::time::{Time, Timer};
use core::time::Duration;

/// Delays by the specified amount of time.
///
/// ## Examples
///
/// ```no_run
/// use core::time::Duration;
/// use bevy::prelude::{World, Update};
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, delay::time().with(Duration::from_secs(1))).await;
/// });
/// ```
#[inline(always)]
pub fn time() -> ActionSeed<Duration> {
    wait::until(
        move |In(duration): In<Duration>, mut timer: Local<Option<Timer>>, time: Res<Time>| {
            timer
                .get_or_insert_with(|| Timer::new(duration, TimerMode::Once))
                .tick(time.delta())
                .just_finished()
        },
    )
}

/// Delays the specified number of frames.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::{World, Update};
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, delay::frames().with(30)).await;
/// });
/// ```
#[inline(always)]
pub fn frames() -> ActionSeed<usize> {
    wait::until(move |In(frames): In<usize>, mut frame_now: Local<usize>| {
        *frame_now += 1;
        frames <= (*frame_now - 1)
    })
}

#[cfg(test)]
mod tests {
    use crate::action::{delay, once};
    use crate::prelude::{Reactor, Then};
    use crate::tests::test_app;
    use bevy::app::{AppExit, First, Startup};
    use bevy::prelude::*;
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn delay_1frame() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    First,
                    delay::frames()
                        .with(1)
                        .then(once::event::app_exit_success()),
                )
                .await;
            }));
        });
        let mut er = app.resource_mut::<Messages<AppExit>>().get_cursor();
        app.update();
        app.assert_message_not_comes(&mut er);

        app.update();
        app.assert_message_comes(&mut er);
    }

    #[test]
    fn delay_2frames() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    First,
                    delay::frames()
                        .with(2)
                        .then(once::non_send::init::<AppExit>()),
                )
                .await;
            }));
        });

        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }
}
