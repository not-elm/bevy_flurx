# record::all_clear

The `record::all_clear` function clears all history of undo and redo operations from the `Record`.

## Function Signature

```rust
record::all_clear<M: 'static>() -> ActionSeed<(), Result<(), UndoRedoInProgress>>
```

Creates an action that clears all history of undo and redo operations from the `Record`. The output will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

## Parameters

- `M`: The type of operation being recorded. This is a generic type parameter that allows you to define different types of operations.

## Return Value

Returns an `ActionSeed<(), Result<(), UndoRedoInProgress>>` that, when executed, will clear all history of undo and redo operations from the `Record`.

## Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct MyOperation;

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        // Push some operations
        task.will(Update, record::push().with(Track {
            act: MyOperation,
            rollback: Rollback::undo(|| once::run(||{}))
        })).await.unwrap();
        
        task.will(Update, record::push().with(Track {
            act: MyOperation,
            rollback: Rollback::undo(|| once::run(||{}))
        })).await.unwrap();
        
        // Clear all history
        task.will(Update, record::all_clear::<MyOperation>())
            .await
            .expect("An error will be returned if undo or redo is operating.");
    }));
}
```

## Error Handling

The `all_clear` function returns a `Result<(), UndoRedoInProgress>`. If an undo or redo operation is in progress, the function will return `Err(UndoRedoInProgress)`.

## When to Use

Use `record::all_clear` when you need to:
- Reset the history of operations
- Clear all undo and redo stacks
- Start fresh with a new history

This function is particularly useful when:
- Starting a new document or project
- After a major operation that makes previous history irrelevant
- When you want to free up memory used by a large history