# record::redo

The `record::redo` module provides actions for redoing operations that have been previously undone.

## Functions

### once

```rust
record::redo::once<Act>() -> ActionSeed<(), EditRecordResult>
```

Pops the last pushed redo action and executes it. After the redo action is executed, the undo action that created it is pushed into the `Record` again. If the redo stack in the `Record` is empty, nothing happens.

The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct Act;

Reactor::schedule(|task| async move {
    // Push an operation
    task.will(Update, record::push().with(
        Track {
            act: Act,
            rollback: Rollback::undo(|| once::run(||{})),
        }))
        .await
        .expect("An error will be returned if undo or redo is operating.");
    
    // Undo the operation
    task.will(Update, record::undo::once::<Act>())
        .await
        .expect("An error will be returned if undo or redo is operating.");
    
    // Redo the operation
    task.will(Update, record::redo::once::<Act>())
        .await
        .expect("An error will be returned if undo or redo is operating.");
});
```

### index_to

```rust
record::redo::index_to<Act>() -> ActionSeed<usize, EditRecordResult>
```

Pops and executes the redo actions up to the specified index. If the redo stack in the `Record` is empty, nothing happens.

The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct Act;

Reactor::schedule(|task| async move {
    // Push multiple operations
    task.will(Update, record::push().with(Track {
        act: Act,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    task.will(Update, record::push().with(Track {
        act: Act,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    task.will(Update, record::push().with(Track {
        act: Act,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    // Undo all operations
    task.will(Update, record::undo::all::<Act>())
        .await
        .expect("An error will be returned if undo or redo is operating.");
    
    // Redo operations up to index 1 (redoing the first two operations)
    task.will(Update, record::redo::index_to::<Act>().with(1))
        .await
        .expect("An error will be returned if undo or redo is operating.");
});
```

### to

```rust
record::redo::to<Act>() -> ActionSeed<Act, EditRecordResult>
```

Pops and executes the redo actions until the specified operation is reached. If the redo stack in the `Record` is empty, nothing happens.

The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(PartialEq)]
enum Act {
    Move,
    Rotate,
    Scale,
}

Reactor::schedule(|task| async move {
    // Push multiple operations
    task.will(Update, record::push().with(Track {
        act: Act::Move,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    task.will(Update, record::push().with(Track {
        act: Act::Rotate,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    task.will(Update, record::push().with(Track {
        act: Act::Scale,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    // Undo all operations
    task.will(Update, record::undo::all::<Act>())
        .await
        .expect("An error will be returned if undo or redo is operating.");
    
    // Redo operations until Act::Rotate is reached
    task.will(Update, record::redo::to().with(Act::Rotate))
        .await
        .expect("An error will be returned if undo or redo is operating.");
});
```

### all

```rust
record::redo::all<Act>() -> ActionSeed<(), EditRecordResult>
```

Pops and executes all the redo actions from the `Record`. If the redo stack in the `Record` is empty, nothing happens.

The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct Act;

Reactor::schedule(|task| async move {
    // Push multiple operations
    task.will(Update, record::push().with(Track {
        act: Act,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    task.will(Update, record::push().with(Track {
        act: Act,
        rollback: Rollback::undo(|| once::run(||{}))
    })).await.unwrap();
    
    // Undo all operations
    task.will(Update, record::undo::all::<Act>())
        .await
        .expect("An error will be returned if undo or redo is operating.");
    
    // Redo all operations
    task.will(Update, record::redo::all::<Act>())
        .await
        .expect("An error will be returned if undo or redo is operating.");
});
```

## Error Handling

All redo functions return an `EditRecordResult`, which is a `Result<(), UndoRedoInProgress>`. If an undo or redo operation is already in progress, the function will return `Err(UndoRedoInProgress)`.

## When to Use

Use `record::redo` actions when you need to:
- Redo the last undone operation
- Redo operations up to a specific point
- Redo all undone operations

These actions are particularly useful for implementing redo functionality in applications like editors, games with rewind mechanics, or any application where users might want to redo their previously undone actions.