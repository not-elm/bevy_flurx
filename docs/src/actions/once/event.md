# once::event

The `once::event` module provides actions for sending Bevy events exactly once. These actions are specialized versions of `once::run` that focus specifically on event sending operations.

## Functions

### send

```rust
once::event::send<E>() -> ActionSeed<E, ()>
```

Creates an action that sends a specified event once. The event must be provided using the `.with()` method.

#### Example

```rust
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    task.will(Update, once::event::send().with(AppExit::Success)).await;
});
```

### send_default

```rust
once::event::send_default<E>() -> ActionSeed
```

Creates an action that sends a default-constructed event once. The event type must implement the `Default` trait.

#### Example

```rust
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    task.will(Update, once::event::send_default::<AppExit>()).await;
});
```

### app_exit_success

```rust
once::event::app_exit_success() -> Action<AppExit, ()>
```

A convenience function that creates an action to send the `AppExit::Success` event once, which will exit the application successfully.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    task.will(Update, once::event::app_exit_success()).await;
});
```

## When to Use

Use `once::event` actions when you need to:
- Send a specific event exactly once
- Send a default-constructed event exactly once
- Exit the application with a success status

For more complex event handling or when you need to access other system parameters, consider using the more general `once::run` action.
