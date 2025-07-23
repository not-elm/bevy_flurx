# record::push

The `record::push` function allows you to add operations to the record history for later undo and redo operations.

## Function Signature

```rust
record::push<Act>() -> ActionSeed<Track<Act>, EditRecordResult>
```

Creates an action that pushes a `Track` onto the `Record`. The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

## Parameters

- `Act`: The type of operation being recorded. This is a generic type parameter that allows you to define different types of operations.

## Return Value

Returns an `ActionSeed<Track<Act>, EditRecordResult>` that, when executed, will push the provided `Track` onto the `Record`.

## Example

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
        ).await.expect("An error will be returned if undo or redo is operating.");
    }));
}
```

## Creating a Track

The `Track` struct is used to define an operation and its rollback behavior:

```rust
Track {
    act: YourActType,
    rollback: Rollback::undo_redo(|| /* your undo action */)
}
```

### Track Fields

- `act`: The operation being recorded. This can be any type that implements `Send + Sync + 'static`.
- `rollback`: The process called when a rollback is requested. This is created using one of the `Rollback` methods.

### Rollback Methods

- `Rollback::new()`: Creates a rollback with a function that may optionally create a redo action.
- `Rollback::undo()`: Creates a rollback that doesn't create a redo action.
- `Rollback::undo_redo()`: Creates a rollback that always creates a redo action.
- `Rollback::parts()`: Declares undo and redo separately with `Undo` and `Redo` types.

## Error Handling

The `push` function returns an `EditRecordResult`, which is a `Result<(), UndoRedoInProgress>`. If an undo or redo operation is in progress, the function will return `Err(UndoRedoInProgress)`.

## When to Use

Use `record::push` when you need to:
- Add an operation to the record history
- Define how an operation can be undone and redone
- Track user actions for later reversal

This function is the foundation of the record system, as it allows you to define operations and their rollback behavior.