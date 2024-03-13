use std::time::Duration;

use bevy::prelude::{Local, Res, TimerMode};
use bevy::time::{Time, Timer};

use crate::selector::condition::{ReactorSystemConfigs, wait, with};

#[inline]
pub fn time(duration: Duration) -> impl ReactorSystemConfigs<In=(), Out=()> {
    let mut timer = Timer::new(duration, TimerMode::Once);
    with((), wait::until(move |time: Res<Time>| {
        timer
            .tick(time.delta())
            .just_finished()
    }))
}

#[inline]
pub fn frames(frames: usize) -> impl ReactorSystemConfigs<In=(), Out=()> {
    with((), wait::until(move |mut frame_now: Local<usize>| {
        *frame_now += 1;
        frames <= *frame_now
    }))
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup, Update};
    use bevy::prelude::World;
    use bevy::time::TimePlugin;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::{delay, once};

    #[test]
    fn delay_2frames() {
        let mut app = App::new();
        app
            .add_plugins((
                TimePlugin,
                FlurxPlugin
            ))
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, delay::frames(2)).await;
                    task.will(Update, once::non_send::init::<AppExit>()).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}