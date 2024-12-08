## v0.7.0

- Support bevy0.15.0
- Removed the flag for the multi-thread feature of bevy, which was depended upon internally in this lib.

## v0.6.0

Support for new versions of bevy.

- [v0.6.0](https://github.com/not-elm/bevy_flurx/pull/58)

## v0.5.3

Fixed `Reactor` `despawn_recursive` to be called correctly.

- [v0.5.3](https://github.com/not-elm/bevy_flurx/pull/54)

## v0.5.2

This version fixed a bug associated with cancellation handlers.

- [v0.5.2](https://github.com/not-elm/bevy_flurx/pull/51)

## v0.5.1

This version has reduced the binary size.

- [v0.5.1](https://github.com/not-elm/bevy_flurx/pull/47)

## v0.5.0

Fixed a bug where the execution run condition switch_just_* was not working correctly

- [v0.5.0](https://github.com/not-elm/bevy_flurx/pull/44)

## v0.4.0

Added effect actions. 

- [v0.4.0](https://github.com/not-elm/bevy_flurx/pull/42)

## v0.3.4

- [v0.3.4](https://github.com/not-elm/bevy_flurx/pull/38)

## v0.3.4-beta.1

In this version, fixed actions related to events.

- [v0.3.4-beta.1 ](https://github.com/not-elm/bevy_flurx/pull/29)

## v0.3.4-beta.0

- [Feature/0.3.4-beta.0](https://github.com/not-elm/bevy_flurx/pull/27)

## V0.3.3

- [Feature/v0.3.3](https://github.com/not-elm/bevy_flurx/pull/23)

## v0.3.3-beta.2

- [Feature/v0.3.3-beta2](https://github.com/not-elm/bevy_flurx/pull/18)

## v0.3.3-beta

- [Feature/v0.3.3-beta](https://github.com/not-elm/bevy_flurx/pull/15)

## v0.3.2

### Features

- [Added once::event::clear](https://github.com/not-elm/bevy_flurx/pull/11)

### Bug fix

- [Fixed an issue where the return value type of some functions such as `once::res::insert` was `impl Action`.](https://github.com/not-elm/bevy_flurx/pull/10)

## v0.3.2-beta.0

### Features

- It is no longer necessary to implement the `Clone` trait on the value passed to `once`.
- Added `Sequence`, `Pipe`, `Switch` and `Reactor`. please see this [pull request](https://github.com/not-elm/bevy_flurx/pull/9)

### Breaking changes

- Rename `Select` to `Either`, and `wait::select` to `wait::either`
- Changed the schedule label for the reactor to run back to `PostUpdate`.
- Due to major internal changes, various modules were moved. if it has been importing via prelude, it probably has no effect.

## v0.3.1

### Update

- Support for bevy version 1.3.1.
- Changed the timing at which the reactive scheduler is executed from `AfterLast` to `Main`.

### Features

- Added [`wait::both`] This is for waiting for two tasks done.
- Added [`wait_all!`] This is for waiting for all tasks done.

### Fix

- We made it run the system directly instead of [`World::register_system`].

## v0.3.0

First released.
