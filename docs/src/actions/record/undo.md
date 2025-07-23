# record::undo

The `record::undo` module provides actions for undoing operations that have been pushed onto the record history.

## Functions

### once

```rust
record::undo::once<Act>() -> ActionSeed<(), EditRecordResult>
```

Pops the last pushed undo action, and then executes it. The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

struct Act;

Reactor::schedule(|task| async move {
    task.will(Update, record::push().with(Track {
        act: Act,
        rollback: Rollback::undo(|| once::run(||{}))
    }))
        .await
        .expect("An error will be returned if undo or redo is operating.");
    
    task.will(Update, record::undo::once::<Act>())
        .await
        .expect("An error will be returned if undo or redo is operating.");
});
```

### index_to

```rust
record::undo::index_to<Act>() -> ActionSeed<usize, EditRecordResult>
```

Pops undo actions up to the specified index. The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

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
    
    // Undo operations up to index 1 (keeping the first operation)
    task.will(Update, record::undo::index_to::<Act>().with(1))
        .await
        .expect("An error will be returned if undo or redo is operating.");
});
```

### to

```rust
record::undo::to<Act>() -> ActionSeed<Act, EditRecordResult>
```

Pops undo actions until the specified operation is reached. The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

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
    
    // Undo operations until Act::Move is reached
    task.will(Update, record::undo::to().with(Act::Move))
        .await
        .expect("An error will be returned if undo or redo is operating.");
});
```

### all

```rust
record::undo::all<Act>() -> ActionSeed<(), EditRecordResult>
```

Pops all the undo actions from the `Record`. The output will be an `EditRecordResult`, which will be an error (`UndoRedoInProgress`) if an undo or redo operation is in progress.

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
});
```

## Error Handling

All undo functions return an `EditRecordResult`, which is a `Result<(), UndoRedoInProgress>`. If an undo or redo operation is already in progress, the function will return `Err(UndoRedoInProgress)`.

## When to Use

Use `record::undo` actions when you need to:
- Undo the last operation
- Undo operations up to a specific point
- Undo all operations

These actions are particularly useful for implementing undo functionality in applications like editors, games with rewind mechanics, or any application where users might want to undo their actions.