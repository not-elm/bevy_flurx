# Record Actions

The `record` module provides actions for managing operation history with undo and redo capabilities. These actions allow you to track operations, roll them back (undo), and reapply them (redo).

## Available Record Actions

- [push](push.md) - Push an operation onto the record history
- [undo](undo.md) - Undo operations from the record history
- [redo](redo.md) - Redo previously undone operations
- [extension](extension.md) - Extensions for using record actions with events and triggers
- [all_clear](all_clear.md) - Clear all history of undo and redo operations

## Basic Usage

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

// Define an operation type
struct MoveAct;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Push an operation onto the record history
        task.will(Update, record::push()
            .with(Track {
                act: MoveAct,
                rollback: Rollback::undo_redo(|| once::run(|mut player: Query<&mut Transform>| {
                    let pos = player.single_mut().unwrap().translation;
                    player.single_mut().unwrap().translation = Vec3::Z;
                    RedoAction::new(once::run(move |mut player: Query<&mut Transform>| {
                        player.single_mut().unwrap().translation = pos;
                    }))
                }))
            })
        ).await.expect("Failed to push operation");

        // Undo the operation
        task.will(Update, record::undo::once::<MoveAct>())
            .await.expect("Failed to undo operation");

        // Redo the operation
        task.will(Update, record::redo::once::<MoveAct>())
            .await.expect("Failed to redo operation");
    }));
}
```

## Core Concepts

### Track

The `Track` struct represents an operation to be recorded, containing:
- `act`: The actual operation being recorded
- `rollback`: The process called when a rollback is requested

### Rollback

The `Rollback` struct holds the function called when an undo operation is requested. It can be created in several ways:
- `new()`: Creates a rollback with a function that may optionally create a redo action
- `undo()`: Creates a rollback that doesn't create a redo action
- `undo_redo()`: Creates a rollback that always creates a redo action
- `parts()`: Declares undo and redo separately with `Undo` and `Redo` types

### RedoAction

The `RedoAction` struct represents an action executed when a redo operation is called.

## When to Use

Use record actions when you need to:
- Implement undo/redo functionality in your application
- Track user operations for later reversal
- Create a history of operations that can be navigated

The record module is particularly useful for applications like editors, games with rewind mechanics, or any application where users might want to undo their actions.