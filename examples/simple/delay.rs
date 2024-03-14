//! This example introduces a list of [`delay`] methods.
//!
//! [`delay`] creates a task that run the until the specified time or frames has elapsed.
//!
//! [`delay`]: bevy_async_system::prelude::delay

use std::time::{Duration, Instant};

use bevy::app::{App, FixedUpdate, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::World;

use bevy_async_system::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, |world: &mut World| {
            //=== [delay::frames] ===
            world.schedule_reactor(|task| async move {
                const DELAY_FRAMES: usize = 60 * 5;
                println!("*** Start [Frames]; wait {DELAY_FRAMES} frames ***");
                task.will(FixedUpdate, delay::frames(DELAY_FRAMES)).await;
                println!("*** Finish [Frames] ***");
            });
            //=======================

            //=== [delay::time] ===
            world.schedule_reactor(|task| async move {
                const DELAY_SECS: u64 = 3;
                println!("*** Start [Time]; wait {DELAY_SECS} secs ***");
                let instant = Instant::now();
                task.will(FixedUpdate, delay::time(Duration::from_secs(DELAY_SECS))).await;
                assert_eq!(instant.elapsed().as_secs(), DELAY_SECS);
                println!("*** Finish [Time] ***");
            });
            //======================
        })
        .run();
}


