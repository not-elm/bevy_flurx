# bevy_flurx

[![Crates.io](https://img.shields.io/crates/v/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/not-elm/bevy_flurx#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)

This library provides functionality similar to coroutines, allowing you to write sequential processing for delays, user input, animations, and more.

[Reactor](https://docs.rs/bevy_flurx/latest/bevy_flurx/prelude/struct.Reactor.html) can be used incrementally, meaning there’s no need to rewrite existing applications to incorporate it.

```rust
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
use std::time::Duration;

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

        // ActionSeed and Action have input and output the generic types.
        // You can call `ActionSeed::with(<input>)` to pass the input to action seed.
        let result: usize = task.will(Update, once::run(|In(num): In<usize>| {
            num + 3
        }).with(3)).await;
        assert_eq!(result, 6);

        // The wait module defines actions that continue to execute every frame according to specified conditions.
        // For example, wait::until takes a system that returns a bool value and continues to execute it until it returns true.
        // other wait actions: https://docs.rs/bevy_flurx/latest/bevy_flurx/action/wait/index.html
        task.will(Update, wait::until(|mut count: ResMut<Count>| {
            count.0 += 1;
            info!("current count: {}", count.0);
            count.0 == 4
        })).await;

        // delay module defines the actions that perform delay processing.
        task.will(Update, delay::time().with(std::time::Duration::from_secs(1))).await;

        // `then`, `pipe` and through`  are also actions that continues to execute another action.
        let message = task.will(Update, {
            delay::frames().with(30)
                .then(once::run(|count: Res<Count>| {
                    count.0
                }))
                // Pipes the output of an action to the input of the next action.
                .pipe(once::run(|In(count): In<usize>| {
                    format!("count is {count}")
                }))
                // Executes the next while keeping the output of the previous action.
                .through(delay::time().with(Duration::from_secs(1)))
        }).await;
        assert_eq!(message, "count is 4");

        info!("Done!");
        task.will(Update, once::event::app_exit_success()).await;
    }));
}
```

## Example

All examples are [`here`](./examples).

## Feature flags

| flag name   | short description                                                                  | default |
|-------------|------------------------------------------------------------------------------------|---------|
| audio       | audio actions                                                                      | false   |
| record      | undo/redo actions and events                                                       | false   | 
| side-effect | thread/async side effects                                                          | false   |
| state       | state actions                                                                      | false   | 
| tokio       | allows to use write asynchronous functions depend on tokio directly in the reactor | false   | 

### audio

Provides the actions that perform simple audio playback and waiting using bevy's default audio functionality.

- [`once::audio`](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/once/audio)
- [`wait::audio`](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/wait/audio)

### record

[doc.rs](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/record/index.html)

Provides `Record` to manage operation history.

![undo_redo](examples/undo_redo.gif)

### side-effect

[doc.rs](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/effect/index.html)

Allows to convert the operations with side effects such as asynchronous runtime or thread into the
referential-transparent actions.

### tokio

You will be able to write processes that depend on tokio's runtime in the reactor.

## ChangeLog

Please see [here](https://github.com/not-elm/bevy_flurx/blob/main/CHANGELOG.md).

## Compatible Bevy versions

| bevy_flurx | bevy   |
|------------|--------|
| 0.3.0 ~    | 0.13.0 |
| 0.6.0 ~    | 0.14.1 | 
| 0.7.0 ~    | 0.15   | 

## License

This crate is licensed under the MIT License or the Apache License 2.0.
