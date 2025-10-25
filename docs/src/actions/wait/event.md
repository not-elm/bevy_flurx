# wait::event

> **⚠️ DEPRECATED**: This module is deprecated since Bevy 0.17. Please use [`wait::message`](./message.md) instead. The `Event` trait has been replaced with `Message` in Bevy 0.17.

The `wait::event` module provides actions for waiting to receive Bevy messages. These actions are useful for creating tasks that respond to messages in your game or application.

## Functions

### comes

```
wait::message::comes<E>() -> ActionSeed
```

Creates an action that waits until an event of type `E` is received. The action completes when the event is received but does not return the event itself.

#### Example

```rust
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait for an AppExit event
    task.will(Update, wait::message::comes::<AppExit>()).await;
    
    // This code runs after an AppExit event is received
    println!("App is exiting!");
});
```

### comes_and

```
wait::message::comes_and<E>(predicate: impl Fn(&E) -> bool + Send + Sync + 'static) -> ActionSeed
```

Creates an action that waits until an event of type `E` is received and the event matches the given predicate. The action completes when a matching event is received but does not return the event itself.

#### Example

```rust
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait for a successful AppExit event
    task.will(Update, wait::message::comes_and::<AppExit>(|e| {
        e.is_success()
    })).await;
    
    // This code runs after a successful AppExit event is received
    println!("App is exiting successfully!");
});
```

### read

```
wait::message::read<E>() -> ActionSeed<(), E>
```

Creates an action that waits until an event of type `E` is received and returns a clone of the event. This is similar to `comes`, but it returns the event itself.

#### Example

```rust
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait for an AppExit event and get the event
    let exit_event = task.will(Update, wait::message::read::<AppExit>()).await;
    
    // This code runs after an AppExit event is received
    println!("App is exiting with status: {:?}", exit_event);
});
```

### read_and

```rust
wait::message::read_and<E>(predicate: impl Fn(&E) -> bool + Send + Sync + 'static) -> ActionSeed<(), E>
```

Creates an action that waits until an event of type `E` is received, the event matches the given predicate, and returns a clone of the event. This is similar to `comes_and`, but it returns the event itself.

#### Example

```rust
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait for a successful AppExit event and get the event
    let exit_event = task.will(Update, wait::message::read_and::<AppExit>(|e| {
        e.is_success()
    })).await;
    
    // This code runs after a successful AppExit event is received
    println!("App is exiting successfully with event: {:?}", exit_event);
});
```

## When to Use

> **Note**: This module is deprecated. Use `wait::message` instead.

Use `wait::message` actions when you need to:
- Wait for specific messages to occur before continuing execution
- React to messages in an asynchronous manner
- Filter messages based on their content using predicates
- Retrieve message data for further processing

For more complex message handling scenarios, consider combining `wait::message` with other wait actions like `wait::either` or `wait::any` to wait for multiple different message types.