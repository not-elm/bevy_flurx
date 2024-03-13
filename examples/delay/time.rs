use std::time::Duration;

use bevy::app::{App, First, Startup, Update};
use bevy::MinimalPlugins;
use bevy::prelude::World;

use bevy_async_system::extension::ScheduleReactor;
use bevy_async_system::FlurxPlugin;
use bevy_async_system::selector::condition::{delay, once};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                println!("*** Wait 1 Sec ***");
                task.will(First, delay::time(Duration::from_secs(1))).await;
                println!("*** Finish ***");
                task.will(Update, once::event::app_exit()).await;
            });
        })
        .run();
}


