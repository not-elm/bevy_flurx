//! [`sequence_with_output!`] create actions that execute the passed actions in sequence.
//! 
//! It has advantage that if the previous action finishes, 
//! the next will start within in that frame. 

use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use bevy_flurx::sequence;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, (
            setup_non_sequence,
            setup_sequence,
        ))
        .run();
}

fn setup_non_sequence(world: &mut World) {
    world.schedule_reactor(|task| async move {
        task.will(Update, once::run(|frame: Res<FrameCount>| {
            println!("[non sequence once1] frame: {}", frame.0);
        })).await;
        task.will(Update, once::run(|frame: Res<FrameCount>| {
            println!("[non sequence once2] frame: {}", frame.0);
        })).await;
        task.will(Update, once::run(|frame: Res<FrameCount>| {
            println!("[non sequence once3] frame: {}", frame.0);
        })).await;
        task.will(Update, once::run(|frame: Res<FrameCount>| {
            println!("[non sequence once4] frame: {}", frame.0);
        })).await;
    });
}

fn setup_sequence(world: &mut World) {
    world.schedule_reactor(|task| async move {
        task.will(Update, sequence! {
            once::run(|frame: Res<FrameCount>|{
                println!("[sequence once1] frame: {}", frame.0);
            }),
            once::run(|frame: Res<FrameCount>|{
                println!("[sequence once2] frame: {}", frame.0);
            }),
            once::run(|frame: Res<FrameCount>|{
                println!("[sequence once3] frame: {}", frame.0);
            }),
            once::run(|frame: Res<FrameCount>|{
                println!("[sequence once4] frame: {}", frame.0);
            })
        }).await;
    });
}