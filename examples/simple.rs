//!  Here are some basic [once], [wait] and [delay] actions.
//!
//! For details on all actions, please check [here](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/index.html).
//!
//! [once]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/once/index.html
//! [wait]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/wait/index.html
//! [delay]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/delay/index.html
use bevy::prelude::*;
use bevy_flurx::prelude::*;
fn main() {
    App::new()
        .insert_resource(Count(0))
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin,
        ))
        .add_systems(Startup, spawn_reactor)
        .run();
}

#[derive(Resource)]
struct Count(usize);

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // `once` module defines the actions that runs only once.
        // For example, once::run once executes any system.
        // other once actions: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/once/index.html
        let current_count: usize = task.will(Update, once::run(|mut count: ResMut<Count>| {
            count.0 += 1;
            count.0
        })).await;
        assert_eq!(current_count, 1);
        // The wait module defines actions that continue to execute every frame according to specified conditions.
        // For example, wait::until takes a system that returns a bool value and continues to execute it until it returns true.
        // other wait actions: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/wait/index.html
        task.will(Update, wait::until(|mut count: ResMut<Count>| {
            count.0 += 1;
            info!("current count: {}", count.0);
            count.0 == 4
        })).await;
        // delay module defines the actions that perform delay processing.
        // `then` is also an action that continues to execute another action.
        task.will(Update, {
            delay::time().with(std::time::Duration::from_secs(1))
                .then(once::run(|| {
                    info!("Done!");
                }))
                .then(once::event::app_exit_success())
        }).await;
    }));
}