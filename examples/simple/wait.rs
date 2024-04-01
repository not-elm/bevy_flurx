//! This example introduces a list of [`wait`] methods.
//!
//! [`wait`] creates a task that run the until the condition is met.
//!
//! [`wait`]: bevy_flurx::prelude::wait

use std::time::Duration;

use bevy::prelude::*;

use bevy_flurx::prelude::*;
use bevy_flurx::wait_all;

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
                println!("start [wait::state] ..");
                let wait_state = task.run(Update, wait::state::becomes(ExampleState::Second)).await;
                task.will(Update, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::state::set(ExampleState::Second)).await;
                wait_state.await;
                println!("end [wait::state]");
                //============================================================


                //=== [wait::event] Run until the event a certain condition is met.
                println!("start [wait::event] ..");
                let wait_event = task.run(Update, wait::event::comes::<Event2>()).await;
                task.will(FixedUpdate, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::event::send_default::<Event2>()).await;
                wait_event.await;
                println!("end [wait::event]");

                let wait_event = task.run(Update, wait::event::read::<Event2>()).await;
                task.will(FixedUpdate, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::event::send_default::<Event2>()).await;
                println!("end [wait::event]");
                // //============================================================


                //=== [wait::select] Run until either of the two tasks is completed.
                println!("start [wait::select] ..");
                let wait_event = task.run(Update, wait::select(wait::event::comes::<Event1>(), wait::event::comes::<Event2>())).await;
                task.will(Update, delay::time(Duration::from_secs(1))).await;
                task.will(Update, once::event::send_default::<Event2>()).await;
                println!("{:?} came\n", wait_event.await);
                println!("end [wait::select]");
                //============================================================

                //=== [wait::both] Run until two tasks done.
                println!("start [wait::both] ..");
                let t1 = delay::time(Duration::from_secs(1));
                let t2 = wait::event::comes::<Event1>();
                let t = task.run(Update, wait::both(t1, t2)).await;
                task.will(Update, once::event::send_default::<Event1>()).await;
                t.await;
                println!("end [wait::both]");
                //============================================================

                //=== [wait_all!] Run until all tasks done.
                println!("start [wait_all!] ..");
                let t1 = delay::time(Duration::from_secs(1));
                let t2 = wait::event::comes::<Event1>();
                let t3 = wait::event::comes::<Event2>();
                let t = task.run(Update, wait_all!(t1, t2, t3)).await;
                task.will(Update, once::event::send_default::<Event1>()).await;
                task.will(Update, once::event::send_default::<Event2>()).await;
                t.await;
                println!("end [wait_all!]");
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

#[derive(Event, Debug, Clone, Default)]
struct Event1;

#[derive(Event, Debug, Clone, Default)]
struct Event2;