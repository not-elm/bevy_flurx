# record::extension

The `record::extension` module provides functionality for making undo and redo requests from outside the Reactor using events and triggers.

## Types

### RequestUndo

```rust
enum RequestUndo<Act> {
    Once,
    IndexTo(usize),
    To(Act),
    All,
}
```

Represents a request to undo operations. If an undo or redo is already in progress, the request will be ignored.

- `Once`: Corresponds to `record::undo::once`
- `IndexTo(usize)`: Corresponds to `record::undo::index_to`
- `To(Act)`: Corresponds to `record::undo::to`
- `All`: Corresponds to `record::undo::all`

### RequestRedo

```rust
enum RequestRedo<Act> {
    Once,
    IndexTo(usize),
    To(Act),
    All,
}
```

Represents a request to redo operations. If an undo or redo is already in progress, the request will be ignored.

- `Once`: Corresponds to `record::redo::once`
- `IndexTo(usize)`: Corresponds to `record::redo::index_to`
- `To(Act)`: Corresponds to `record::redo::to`
- `All`: Corresponds to `record::redo::all`

## Traits

### RecordExtension

```rust
trait RecordExtension {
    fn add_record<Act>(&mut self) -> &mut Self
    where
        Act: Clone + PartialEq + Send + Sync + 'static;
}
```

Allows undo and redo requests to be made using `RequestUndo` and `RequestRedo` from outside the Reactor.

#### Methods

- `add_record<Act>`: Sets up `RequestUndo` and `RequestRedo` and their associated systems.

## Examples

### Using Events

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

#[derive(Resource, Default)]
struct UndoRedoState {
    can_undo: bool,
    can_redo: bool,
}

fn setup_app(app: &mut App) {
    // Add record functionality for your operation type
    app.add_record::<MyOperation>();
}

fn update_ui_system(
    mut ui_state: ResMut<UndoRedoState>,
    record: Res<Record<MyOperation>>,
) {
    ui_state.can_undo = !record.tracks.is_empty();
    ui_state.can_redo = !record.redo.is_empty();
}

fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut undo_events: EventWriter<RequestUndo<MyOperation>>,
    mut redo_events: EventWriter<RequestRedo<MyOperation>>,
) {
    // Handle Ctrl+Z for undo
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::Z) {
        undo_events.send(RequestUndo::Once);
    }
    
    // Handle Ctrl+Y for redo
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::Y) {
        redo_events.send(RequestRedo::Once);
    }
}
```

### Using Triggers

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn handle_button_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<UndoButton>)>,
    world: &mut World,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            // Trigger an undo operation
            world.trigger(RequestUndo::<MyOperation>::Once);
        }
    }
}
```

## How It Works

When you call `app.add_record::<Act>()`, the following happens:

1. The `Record<Act>` resource is initialized
2. Event types for `RequestUndo<Act>` and `RequestRedo<Act>` are registered
3. Systems are added to handle these events and triggers

When a `RequestUndo` or `RequestRedo` event is sent or triggered:

1. The corresponding system creates a new Reactor
2. The Reactor executes the appropriate undo or redo action
3. If the action fails (e.g., because an undo or redo is already in progress), the error is ignored

## When to Use

Use the extension module when you need to:
- Trigger undo/redo operations from UI elements
- Handle keyboard shortcuts for undo/redo
- Integrate undo/redo functionality with other systems in your application

This module is particularly useful for applications with complex UI that need to provide undo/redo functionality through various means (buttons, keyboard shortcuts, etc.).