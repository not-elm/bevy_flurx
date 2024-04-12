//! [`delay`] creates a task that delay the application.
//!
//! actions
//!
//! - [`delay::time`](crate::prelude::delay::time)
//! - [`delay::frames`](crate::prelude::delay::frames)

use std::time::Duration;

use bevy::prelude::{In, Local, Res, TimerMode};
use bevy::time::{Time, Timer};

use crate::action::wait;
use crate::prelude::ActionSeed;

/// Delays by the specified amount of time.
///
/// ## Examples
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::{World, Update};
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, delay::time().with(Duration::from_secs(1))).await;
/// });
/// ```
#[inline(always)]
pub fn time() -> ActionSeed<Duration> {
    wait::until(move |In(duration): In<Duration>,
                      mut timer: Local<Option<Timer>>,
                      time: Res<Time>,
    | {
        if timer.is_none() {
            timer.replace(Timer::new(duration, TimerMode::Once));
        }

        timer
            .as_mut()
            .unwrap()
            .tick(time.delta())
            .just_finished()
    })
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
    wait::until(move |In(frames): In<usize>,
                      mut frame_now: Local<usize>| {
        *frame_now += 1;
        frames <= (*frame_now - 1)
    })
}

#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, First, Startup};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::Commands;
    use bevy::time::TimePlugin;
    use bevy_test_helper::event::DirectEvents;

    use crate::action::{delay, once};
    use crate::prelude::Then;
    use crate::reactor::Reactor;
    use crate::tests::test_app;

    #[test]
    fn delay_1frame() {
        let mut app = test_app();
        app
            .add_plugins(TimePlugin)
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(First, delay::frames().with(1)
                        .then(once::event::app_exit()),
                    ).await;
                }));
            });
        let mut er = ManualEventReader::<AppExit>::default();
        app.update();
        app.assert_event_not_comes(&mut er);
        
        app.update();
        app.assert_event_comes(&mut er);
    }

    #[test]
    fn delay_2frames() {
        let mut app = test_app();
        app
            .add_plugins(TimePlugin)
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(First, delay::frames().with(2)
                        .then(once::non_send::init::<AppExit>()),
                    ).await;
                }));
            });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}