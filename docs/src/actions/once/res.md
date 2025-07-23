# once::res

The `once::res` module provides actions for managing Bevy resources exactly once. These actions are specialized versions of `once::run` that focus specifically on resource operations.

## Functions

### init

```rust
once::res::init<R>() -> ActionSeed
```

Creates an action that initializes a resource using its `Default` implementation. The resource will only be initialized if it doesn't already exist.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Resource, Default)]
struct GameScore(u32);

Reactor::schedule(|task| async move {
    task.will(Update, once::res::init::<GameScore>()).await;
});
```

### insert

```rust
once::res::insert<R>() -> ActionSeed<R>
```

Creates an action that inserts a provided resource. If the resource already exists, it will be replaced.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Resource)]
struct GameScore(u32);

Reactor::schedule(|task| async move {
    task.will(Update, once::res::insert().with(GameScore(100))).await;
});
```

### remove

```rust
once::res::remove<R>() -> ActionSeed
```

Creates an action that removes a resource if it exists.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Resource)]
struct TemporaryResource;

Reactor::schedule(|task| async move {
    task.will(Update, once::res::remove::<TemporaryResource>()).await;
});
```

## When to Use

Use `once::res` actions when you need to:
- Initialize a resource with its default value
- Insert or replace a resource with a specific value
- Remove a resource that's no longer needed

For more complex resource operations or when you need to access other system parameters, consider using the more general `once::run` action with `ResMut` or `Res` parameters.
