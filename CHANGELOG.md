## v0.3.1

### Update

- Changed the timing at which the reactive scheduler is executed from `AfterLast` to `Main`.

### Features

- Added [`wait::both`] This is for waiting for two tasks done.
- Added [`wait_all!`] This is for waiting for all tasks done.

### Fix

- Support for bevy version 1.3.1.
- We made it run the system directly instead of [`Worold::register_system`].

## v0.3.0

First released.
