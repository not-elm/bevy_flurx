use std::time::Duration;
use bevy::prelude::*;

use bevy_async_system::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AsyncSystemPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        schedules.add_system(Update, repeat::times(5, count_up)).await;

        let handle = schedules.add_system(Update, repeat::forever(count_up));
        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;

        // task and system cancel.
        drop(handle);

        println!("task canceled. Exit the application after 3 seconds.");
        // Delay to make sure the system does not run.
        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;
        println!("App exit");
        schedules.add_system(Update, once::app_exit()).await;
    });
}


fn count_up(mut count: Local<u32>) {
    *count += 1;
    println!("count = {}", *count);
}

