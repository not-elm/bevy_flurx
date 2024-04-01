//! This example introduces a list of [`once`] methods.
//!
//! [`once`] creates a task that run the system only once.
//!
//! Please check [`setup`] function in this example for details.
//!
//! [`once`]: bevy_flurx::prelude::once

use bevy::prelude::*;

use bevy_flurx::prelude::*;

fn main() {
    App::new()
        .init_state::<ExampleState>()
        .add_event::<ExampleEvent>()
        .add_plugins((
            MinimalPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(commands: Commands) {
    commands
        .spawn()
        .insert(Reactor::new(|task| async move {}));
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default, States, Hash)]
enum ExampleState {
    #[default]
    First,
    Second,
}

#[derive(Resource, Eq, PartialEq, Default, Clone, Debug)]
struct Count(usize);

#[derive(Eq, PartialEq, Default, Clone, Debug)]
struct NonSendCount(usize);

#[derive(Event, Default, Clone)]
struct ExampleEvent;
