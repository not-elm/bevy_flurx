//! Testing that the system runs properly when the switch is switched

use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct S;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            console_switch_just_on::<1>.run_if(switch_just_turned_on::<S>),
            console_switch_just_on::<2>.run_if(switch_just_turned_on::<S>),
            console_switch_just_off::<1>.run_if(switch_just_turned_off::<S>),
            console_switch_just_off::<2>.run_if(switch_just_turned_off::<S>)
        ))
        .run();
}

fn setup(
    mut commands: Commands
) {
    commands.spawn(Reactor::schedule(|task| async move {
        loop {
            task.will(Update, wait::input::just_pressed().with(KeyCode::KeyT)
                .then(once::switch::on::<S>())
                .then(delay::frames().with(1))
                .then(wait::input::just_pressed().with(KeyCode::KeyT))
                .then(once::switch::off::<S>())
                .then(delay::frames().with(1)),
            ).await;
        }
    }));
}

fn console_switch_just_on<const NUM: u8>() {
    println!("[{NUM}] switch just turned on!");
}

fn console_switch_just_off<const NUM: u8>() {
    println!("[{NUM}] switch just turned off!");
}
