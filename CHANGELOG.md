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
