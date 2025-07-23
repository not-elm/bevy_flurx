# once::non_send

The `once::non_send` module provides actions for managing Bevy non-send resources exactly once. Non-send resources are resources that are not thread-safe and can only be accessed from the main thread.

## Functions

### init

```rust
once::non_send::init<R>() -> ActionSeed
```

Creates an action that initializes a non-send resource using its `Default` implementation. The resource will only be initialized if it doesn't already exist.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Default)]
struct WindowHandle(/* some non-Send type */);

Reactor::schedule(|task| async move {
    task.will(Update, once::non_send::init::<WindowHandle>()).await;
});
```

### insert

```rust
once::non_send::insert<R>() -> ActionSeed<R>
```

Creates an action that inserts a provided non-send resource. If the resource already exists, it will be replaced.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct WindowHandle(/* some non-Send type */);

Reactor::schedule(|task| async move {
    task.will(Update, once::non_send::insert().with(WindowHandle(/* ... */)))
        .await;
});
```

### remove

```rust
once::non_send::remove<R>() -> ActionSeed
```

Creates an action that removes a non-send resource if it exists.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct WindowHandle(/* some non-Send type */);

Reactor::schedule(|task| async move {
    task.will(Update, once::non_send::remove::<WindowHandle>()).await;
});
```

## When to Use

Use `once::non_send` actions when you need to:
- Initialize a non-send resource with its default value
- Insert or replace a non-send resource with a specific value
- Remove a non-send resource that's no longer needed

Non-send resources are typically used for resources that contain types that cannot be sent between threads, such as:
- Raw pointers
- File handles
- Window handles
- Other platform-specific resources

For more complex non-send resource operations or when you need to access other system parameters, consider using the more general `once::run` action with `NonSendMut` or `NonSend` parameters.
