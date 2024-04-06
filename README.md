# bevy_flurx

[![Crates.io](https://img.shields.io/crates/v/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/not-elm/bevy_flurx#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_flurx.svg)](https://crates.io/crates/bevy_flurx)

> [!CAUTION]
> `bevy_flurx` is currently in the early stages of development and is subject to breaking changes.

This library provides a mechanism for more sequential description of delays, character movement,
waiting for user input, and other state waits.

## Example

All examples are [`here`](./examples).

![cut_in](examples/cut_in.gif)
<details>

<summary>Part of source code</summary>

```rust
fn spawn_reactor(
    mut commands: Commands
) {
    commands.spawn(Reactor::schedule(|task| async move {
        task.will(Update, {
            wait::input::just_pressed().with(KeyCode::KeyR)
                .then(once::switch::on::<CutInBackground>())
                .then(delay::time().with(Duration::from_millis(100)))
                .then(once::switch::on::<HandsomeFerris>())
                .then(wait::both(
                    wait::switch::off::<CutInBackground>(),
                    wait::switch::off::<HandsomeFerris>(),
                ))
                .then(once::switch::on::<MoveSlowly>())
                .then(delay::time().with(Duration::from_millis(500)))
                .then(once::switch::off::<MoveSlowly>())
                .then(once::switch::on::<MoveFast>())
                .then(delay::time().with(Duration::from_millis(300)))
                .then(once::event::app_exit())
        })
            .await;
    }));
}
```

</details>

## ChangeLog

Please see [here](https://github.com/not-elm/bevy_flurx/blob/main/CHANGELOG.md).

## Compatible Bevy versions

| bevy_flurx  | bevy   |
|-------------|--------|
| 0.3.0       | 0.13.0 |
| 0.3.1       | 0.13.1 |
| 0.3.2-beta0 | 0.13.2 |

## License

This crate is licensed under the MIT License or the Apache License 2.0.
