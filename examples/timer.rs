use bevy::app::{App, Startup, Update};
use bevy::MinimalPlugins;
use bevy::prelude::{Commands, World};
use bevy::time::{Timer, TimerMode};

use bevtask::BevTaskPlugin;
use bevtask::ext::AsyncPool;
use bevtask::runner::delay::Delay;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            BevTaskPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, timer)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn_async(|task| async move {
        loop {
            task.spawn(Update, Delay::Timer(Timer::from_seconds(1., TimerMode::Once))).await;
            println!("******** tick **********");
        }
    });
}

fn timer(){
    println!("thread: {:?}", std::thread::current());
}