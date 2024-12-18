# bevy_flurx

[![Crates.io](https://img.shields.io/crates/v/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/not-elm/bevy_flurx#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)

This library offers a mechanism for more sequential descriptions of delays, character movement, waiting for user input,
and other state waits.  
Reactor can be used incrementally, meaning there's no need to rewrite existing applications to incorporate it.   
I recommend this partial usage since the system that runs Reactor and the systems executed by Reactor operate on the
main thread.   
For multithreaded operation, please check the Switch.

```rust
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
```

## Example

All examples are [`here`](./examples).

## Feature flags

| flag name | short description              | default |
|-----------|--------------------------------|---------|
| audio     | audio actions                  | false   |
| record    | undo/redo actions and events   | false   | 
| effect    | thread/async side effects      | false   |
| state     | state actions                  | false   | 
| tokio     | async-compat and async actions | false   | 

### audio

Provides the actions that perform simple audio playback and waiting using bevy's default audio functionality.

- [`once::audio`](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/once/audio)
- [`wait::audio`](https://docs.rs/bevy_flurx/latest/bevy_flurx/action/wait/audio)

### record

Provides `Record` to manage operation history.

![undo_redo](examples/undo_redo.gif)

### effect

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
