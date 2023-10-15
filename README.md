# bevy_async_system

[![Crates.io](https://img.shields.io/crates/v/bevy_async_system.svg)](https://crates.io/crates/bevy_async_system)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/elm-register/bevy_async_system#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_async_system.svg)](https://crates.io/crates/bevy_async_system)

This crate provides [`UniTask`](https://github.com/Cysharp/UniTask)-like functionality to asynchronously await game
state.

## Usage

All examples are [`here`](./examples/).

### once

The `once` is used to run the system only once.

In addition to simple system execution, it is also used to change state and send events.

```rust
use bevy::prelude::*;
use bevy_async_system::prelude::*;

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

fn setup(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        schedules.add_system(Update, once::run(println_system)).await;
        schedules.add_system(Update, once::set_state(ExampleState::Second)).await;
        schedules.add_system(Update, once::init_resource::<Count>()).await;
        schedules.add_system(Update, once::init_non_send_resource::<NonSendCount>()).await;

        let count = schedules.add_system(Update, once::run(return_count)).await;
        schedules.add_system(Update, once::insert_resource(count)).await;
        schedules.add_system(Update, once::run(println_counts)).await;

        schedules.add_system(Update, once::send(AppExit)).await;
    });
}

fn println_system() {
    println!("hello!");
}


fn return_count() -> Count {
    Count(30)
}

fn println_counts(
    count: Res<Count>,
    non_send_count: NonSend<NonSendCount>
) {
    println!("{count:?}");
    println!("{non_send_count:?}");
}
```

### wait

`wait` keeps the system running until a certain condition is met.

For example, you can easily write a process that waits until music playback ends, as shown below.

```rust
use bevy::prelude::*;
use bevy_async_system::prelude::*;

fn setup_async_systems(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        schedules.add_system(Update, once::run(play_audio)).await;
        schedules.add_system(Update, wait::until(finished_audio)).await;
        info!("***** Finished audio *****");
        schedules.add_system(Update, once::send(AppExit)).await;
    });
}

fn play_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/higurashi.ogg"),
        settings: PlaybackSettings::ONCE,
    });
}
```

### delay

You can delay the task using Frames or Timer.

```rust
use bevy::prelude::*;
use bevy_async_system::prelude::*;

fn setup(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
) {
    settings.limiter = Limiter::from_framerate(30.);
    commands.spawn_async(|schedules| async move {
        println!("Wait 3 seconds...");
        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;

        println!("Wait 90 frames...");
        schedules.add_system(Update, delay::frames(90)).await;
    });
}
```

### repeat

The `repeat` is used to run the system multiple times.

```rust
use bevy::prelude::*;
use bevy_async_system::prelude::*;

fn setup(mut commands: Commands) {
    commands.spawn_async(|schedules| async move {
        schedules.add_system(Update, repeat::times(5, count_up)).await;

        let handle = schedules.add_system(Update, repeat::forever(count_up));
        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;

        // task and system cancel.
        drop(handle);

        println!("task canceled. Exit the application after 3 seconds.");
        // Delay to make sure the system does not run.
        schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;
        println!("App exit");
        schedules.add_system(Update, once::app_exit()).await;
    });
}

fn count_up(mut count: Local<u32>) {
    *count += 1;
    println!("count = {}", *count);
}
```

## Compatible Bevy versions

| bevy_async_system | bevy |
|-------------------|------|
| 0.1               | 0.11 |