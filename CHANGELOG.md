## v0.11.0
[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.11.0)

### Features

- Support for Bevy's 0.16.0


## v0.11.0-rc.5
[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.11.0-rc.5)

Support for Bevy's 0.16-rc.5

## v0.11.0-rc.3
[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.11.0-rc.3)

## Features

- Support for Bevy 0.16-rc.3
- Added `wait::event::comes_and` and `wait::event::read_and` actions that wait for an event with a predicate.
- It is now possible to request undo/redo from trigger.

## Breaking Changes

- Renamed `App::add_record_events` to `App::add_record`.

## v0.11.0-rc.2
[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.11.0-rc.2)

## Bug Fix

- Fixed crash app when Reactor spawns inside another reactor

## v0.11.0-rc.1 
[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.11.0-rc.1)

Support for bevy 0.16.0-rc.1

## v0.10.0

[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.10.0)

### Features

- Added `StepAllReactors` and `StepReactor` trigger events to step the reactor manually; please see this [example](./examples/side_effect.rs)

### Breaking Changes

- The reactor now steps immediately when each runner completes processing.
- From this version onward, if you invoke asynchronous processing other than an Action within the reactor’s async block as shown below, you will need to manually advance the reactor by triggering either StepAllReactors or StepReactor.
- Renamed feature flag `effect` to `side-effect`
- Deleted `effect` module path; please use `side_effect` instead.

### Improvements

- Improved the delay until the callback is invoked when the reactor is canceled
- Significantly improved performance.

## v0.9.1

[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.9.1)

### Improvements

- To support a wider range of versions, the dependency libraries are now specified by their major version.
- Rename `effect` module to `side_effect`.
  - The original name is also retained for compatibility but will be removed in 0.10.
- Added `Functor` trait.
  - This allows you to pass a function without arguments to `side_effect::thread::spawn`.

## v0.9.0

[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.9.0)

- Added the `inspect` module, providing utilities for auxiliary side-effect handling (e.g., logging or debugging) with the `inspect` function and `Inspect` trait.
- Added `Action::split` method to split an action into an input value and a seed. 
- Changed access modifier for `ActionSeed::create_runner` and `Action::create_runner` to pub.

## v0.8.3

[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.8.3)

- Fixed regression that had occurred since v0.8.2.

## v0.8.2

[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.8.2)

- Fix issues related to the reactor cancellation.

## v0.8.1

[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.8.1)

- Refactor doc.rs and example/simple.rs.

## v0.8.0

[Release note](https://github.com/not-elm/bevy_flurx/releases/tag/v0.8.0)

- Significant improvements have been made regarding thread safety.

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

- [Fixed an issue where the return value type of some functions such as `once::res::insert` was
  `impl Action`.](https://github.com/not-elm/bevy_flurx/pull/10)

## v0.3.2-beta.0

### Features

- It is no longer necessary to implement the `Clone` trait on the value passed to `once`.
- Added `Sequence`, `Pipe`, `Switch` and `Reactor`. please see
  this [pull request](https://github.com/not-elm/bevy_flurx/pull/9)

### Breaking changes

- Rename `Select` to `Either`, and `wait::select` to `wait::either`
- Changed the schedule label for the reactor to run back to `PostUpdate`.
- Due to major internal changes, various modules were moved. if it has been importing via prelude, it probably has no
  effect.

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
