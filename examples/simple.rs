//! Here are some basic [once], [wait], [delay], [then], [pipe] and [through] actions.
//!
//! For details on all actions, please check [here](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/index.html).
//!
//! [once]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/once/index.html
//! [wait]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/wait/index.html
//! [delay]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/delay/index.html
//! [then]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/sequence/trait.Then.html#tymethod.then
//! [pipe]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/pipe/trait.Pipe.html#tymethod.pipe
//! [through]: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/through/fn.through.html

use bevy::prelude::*;
use bevy_flurx::prelude::*;
use core::time::Duration;

fn main() {
    App::new()
        .insert_resource(Count(0))
        .add_plugins((DefaultPlugins, FlurxPlugin))
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
        let current_count: usize = task
            .will(
                Update,
                once::run(|mut count: ResMut<Count>| {
                    count.0 += 1;
                    count.0
                }),
            )
            .await;
        assert_eq!(current_count, 1);

        // ActionSeed and Action have input and output the generic types.
        // You can call `ActionSeed::with(<input>)` to pass the input to action seed.
        let result: usize = task
            .will(Update, once::run(|In(num): In<usize>| num + 3).with(3))
            .await;
        assert_eq!(result, 6);

        // The wait module defines actions that continue to execute every frame according to specified conditions.
        // For example, wait::until takes a system that returns a bool value and continues to execute it until it returns true.
        // other wait actions: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/wait/index.html
        task.will(
            Update,
            wait::until(|mut count: ResMut<Count>| {
                count.0 += 1;
                info!("current count: {}", count.0);
                count.0 == 4
            }),
        )
        .await;

        // delay module defines the actions that perform delay processing.
        task.will(
            Update,
            delay::time().with(core::time::Duration::from_secs(1)),
        )
        .await;

        // `then`, `pipe` and through`  are also actions that continues to execute another action.
        let message = task
            .will(Update, {
                delay::frames()
                    .with(30)
                    .then(once::run(|count: Res<Count>| count.0))
                    // Pipes the output of an action to the input of the next action.
                    .pipe(once::run(|In(count): In<usize>| {
                        format!("count is {count}")
                    }))
                    // Executes the next while keeping the output of the previous action.
                    .through(delay::time().with(Duration::from_secs(1)))
            })
            .await;
        assert_eq!(message, "count is 4");

        info!("Done!");
        task.will(Update, once::message::app_exit_success()).await;
    }));
}
