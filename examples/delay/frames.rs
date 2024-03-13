use bevy::app::{App, First, Startup, Update};
use bevy::DefaultPlugins;
use bevy::prelude::World;

use bevy_async_system::extension::ScheduleReactor;
use bevy_async_system::FlurxPlugin;
use bevy_async_system::selector::condition::{delay, once};

const DELAY_FRAMES: usize = 60 * 3;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                println!("*** Wait {DELAY_FRAMES} frames ***");
                task.will(First, delay::frames(DELAY_FRAMES)).await;
                println!("*** Finish ***");
                task.will(Update, once::event::app_exit()).await;
            });
        })
        .run();
}


