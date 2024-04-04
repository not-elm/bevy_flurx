## v0.3.2-beta.0

### Features

- It is no longer necessary to implement the `Clone` trait on the value passed to `once`.
- Added `sequence!` action; please check the [`example`](./examples/simple/sequence.rs) for details.

### Breaking changes

- Rename `Select` to `Either`, and `wait::select` to `wait::either`
- Changed the schedule label for the reactor to run back to `AfterLast`.
- Due to major internal changes, various modules were moved. if it has been importing via prelude, it probably has no effect.

### Other

- Added comments
- Added GitHub actions
- Added Benchmark

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
