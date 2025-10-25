# once::run

The `once::run` action executes a system exactly once and then completes. This is the most basic and versatile once action, allowing you to run any Bevy system as a one-time operation.

## Usage

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    task.will(Update, once::run(|world: &mut World| {
        // Your one-time logic here
    })).await;
});
```

## Parameters

- `system`: Any valid Bevy system that can be converted using `IntoSystem`

## Return Value

The action returns the value returned by the system.

## Example: Sending an App Exit Event

```rust
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    task.will(Update, once::run(|mut ew: MessageWriter<AppExit>| {
        ew.write(AppExit::Success);
    })).await;
});
```

## Example: Modifying a Resource

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Resource, Default)]
struct Score(u32);

Reactor::schedule(|task| async move {
    task.will(Update, once::run(|mut score: ResMut<Score>| {
        score.0 += 10;
    })).await;
});
```

## When to Use

Use `once::run` when you need to:
- Execute arbitrary logic once
- Access multiple system parameters in a single action
- Return a custom value from your action

For more specific use cases, consider the specialized once actions like `once::event` or `once::res`.
