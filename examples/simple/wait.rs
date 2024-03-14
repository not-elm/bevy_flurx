//! This example introduces a list of [`wait`] methods.
//!
//! [`wait`] creates a task that run the until the condition is met.
//!
//! [`wait`]: bevy_flurx::prelude::wait

use std::time::Duration;

use bevy::prelude::*;

use bevy_flurx::prelude::*;

fn main() {
    App::new()
        .init_state::<ExampleState>()
        .add_event::<Event1>()
        .add_event::<Event2>()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                //=== [wait::until] Run until it returns true.
                task.will(Update, wait::until(|mut count: Local<usize>| {
                    *count += 1;
                    println!("count: {count:?}");
                    *count == 3
                })).await;

                // Run until it returns Option::Some.
                // The contents of Some will be return value of the task.
                let _output_is_3: usize = task.will(Update, wait::output(|mut count: Local<usize>| {
                    *count += 1;
                    println!("count: {count:?}");
                    (*count == 3).then_some(*count)
                })).await;
                //============================================================


                //=== [wait::state] Run until the state a certain condition is met.
                println!("Wait until state becomes ExampleState::Second..");
                let wait_state = task.run(Update, wait::state::becomes(ExampleState::Second)).await;
                task.will(Update, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::state::set(ExampleState::Second)).await;
                wait_state.await;
                println!("State becomes ExampleState::Second\n");
                //============================================================


                //=== [wait::event] Run until the event a certain condition is met.
                println!("Wait until Event1 comes..");
                let wait_event = task.run(Update, wait::event::comes::<Event2>()).await;
                task.will(FixedUpdate, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::event::send_default::<Event2>()).await;
                wait_event.await;
                println!("Event2 came\n");

                println!("Wait until Event1 read..");
                let wait_event = task.run(Update, wait::event::read::<Event2>()).await;
                task.will(FixedUpdate, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::event::send_default::<Event2>()).await;
                println!("{:?} read\n", wait_event.await);
                // //============================================================


                //=== [wait::select] Run until either of the two tasks is completed.
                println!("Wait until event `Event1` or `Event2` comes..");
                let wait_event = task.run(Update, wait::select(wait::event::comes::<Event1>(), wait::event::comes::<Event2>())).await;
                task.will(Update, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::event::send_default::<Event2>()).await;
                println!("{:?} came\n", wait_event.await);
                //============================================================


                println!("*** Finish ***");
                task.will(Update, once::event::app_exit()).await;
            });
        })
        .run();
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default, States, Hash)]
enum ExampleState {
    #[default]
    First,
    Second,
}

#[derive(Event, Debug, Clone)]
struct Event1;

#[derive(Event, Debug, Clone, Default)]
struct Event2;