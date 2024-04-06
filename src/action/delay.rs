//! [`delay`] creates a task that delay the application.
//!
//! - [`delay::time`](crate::prelude::delay::time)
//! - [`delay::frames`](crate::prelude::delay::frames)

use std::time::Duration;

use bevy::prelude::{In, Local, Res, TimerMode};
use bevy::time::{Time, Timer};

use crate::action::wait;
use crate::prelude::{ActionSeed, Seed};

/// Delays by the specified amount of time.
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::{World, Update};
/// use bevy_flurx::prelude::*;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, delay::time().with(Duration::from_secs(1))).await;
/// });
/// ```
#[inline(always)]
pub fn time() -> impl ActionSeed<Duration> + Seed {
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
/// ```no_run
/// use bevy::prelude::{World, Update};
/// use bevy_flurx::prelude::*;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, delay::frames().with(30)).await;
/// });
/// ```
#[inline(always)]
pub fn frames() -> impl ActionSeed<usize> + Seed {
    wait::until(move |In(frames): In<usize>,
                      mut frame_now: Local<usize>| {
        *frame_now += 1;
        frames <= *frame_now
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup, Update};
    use bevy::prelude::Commands;
    use bevy::time::TimePlugin;

    use crate::action::{delay, once};
    use crate::FlurxPlugin;
    use crate::prelude::ActionSeed;
    use crate::scheduler::Flurx;

    #[test]
    fn delay_2frames() {
        let mut app = App::new();
        app
            .add_plugins((
                TimePlugin,
                FlurxPlugin
            ))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Flurx::schedule(|task| async move {
                    task.will(First, delay::frames().with(2)).await;
                    task.will(Update, once::non_send::init::<AppExit>()).await;
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