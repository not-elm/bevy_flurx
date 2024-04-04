//! This example introduces a list of [`once`] methods.
//!
//! [`once`] creates a task that run the system only once.
//!
//! Please check [`setup`] function in this example for details.
//! 
//! [`once`]: bevy_flurx::prelude::once_action

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

fn setup(world: &mut World) {
    world.schedule_reactor(|task| async move {
        task.will(First, once::run(|| {
            println!("*** Start [once] Examples! ***");
        })).await;

        // You can also receive the system's output.
        let output = task.will(First, once::run(|count: Local<usize>| {
            // Returns 2
            *count + 1
        })).await;
        // output: 2
        println!("output: {output}");

        //=== [once::res] You can operate Resources. ===
        task.will(Update, once::res::init::<Count>()).await;
        task.will(Update, once::res::insert(Count(0))).await;
        task.will(Update, once::res::remove::<Count>()).await;
        //===

        //=== [once::non_send] You can operate NonSendResources. ===
        task.will(Update, once::non_send::init::<NonSendCount>()).await;
        task.will(Update, once::non_send::insert(NonSendCount(0))).await;
        task.will(Update, once::non_send::remove::<NonSendCount>()).await;
        //===

        //=== [once::state] You can operate states. ===
        task.will(Update, once::state::set(ExampleState::Second)).await;
        //===

        //=== [once::state] You can send events. ===
        task.will(Update, once::event::send(ExampleEvent)).await;
        task.will(Update, once::event::send_default::<ExampleEvent>()).await;
        println!("*** Finish [once] Examples! ***");
        // exit application.
        task.will(Update, once::event::app_exit()).await;
        //===
    });
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
