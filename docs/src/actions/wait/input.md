# wait::input

The `wait::input` module provides actions for waiting for input events. These actions are useful for creating interactive tasks that respond to user input, allowing tasks to wait for specific input events before continuing execution.

## Functions

### just_pressed

```
wait::input::just_pressed<T>() -> ActionSeed<T>
```

Creates an action that waits until a button has just been pressed. The action completes when the specified button is pressed.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait until the B key is pressed
    task.will(Update, wait::input::just_pressed().with(KeyCode::KeyB)).await;
    
    // This code runs after the B key is pressed
    println!("B key was pressed!");
});
```

### pressed

```
wait::input::pressed<T>() -> ActionSeed<T>
```

Creates an action that waits until a button is being pressed. The action completes when the specified button is detected as pressed.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait until the B key is being pressed
    task.will(Update, wait::input::pressed().with(KeyCode::KeyB)).await;
    
    // This code runs while the B key is being pressed
    println!("B key is being pressed!");
});
```

### any_pressed

```
wait::input::any_pressed<T>() -> ActionSeed<Vec<T>>
```

Creates an action that waits until any button in a list is being pressed. The action completes when any of the specified buttons is detected as pressed.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait until either A or B key is pressed
    task.will(Update, wait::input::any_pressed().with(vec![KeyCode::KeyA, KeyCode::KeyB])).await;
    
    // This code runs when either A or B is pressed
    println!("Either A or B key is being pressed!");
});
```

### all_pressed

```
wait::input::all_pressed<T>() -> ActionSeed<Vec<T>>
```

Creates an action that waits until all buttons in a list are being pressed. The action completes when all of the specified buttons are detected as pressed.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait until both A and B keys are pressed
    task.will(Update, wait::input::all_pressed().with(vec![KeyCode::KeyA, KeyCode::KeyB])).await;
    
    // This code runs when both A and B are pressed
    println!("Both A and B keys are being pressed!");
});
```

### just_released

```
wait::input::just_released<T>() -> ActionSeed<T>
```

Creates an action that waits until a button has just been released. The action completes when the specified button is released.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait until the B key is released
    task.will(Update, wait::input::just_released().with(KeyCode::KeyB)).await;
    
    // This code runs after the B key is released
    println!("B key was released!");
});
```

### any_just_released

```rust
wait::input::any_just_released<T>() -> ActionSeed<Vec<T>>
```

Creates an action that waits until any button in a list has just been released. The action completes when any of the specified buttons is released.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Wait until either A or B key is released
    task.will(Update, wait::input::any_just_released().with(vec![KeyCode::KeyA, KeyCode::KeyB])).await;
    
    // This code runs when either A or B is released
    println!("Either A or B key was released!");
});
```

## When to Use

Use `wait::input` actions when you need to:
- Wait for specific user input before continuing execution
- Create interactive sequences that respond to player actions
- Implement combo systems or special move detection
- Create tutorial sequences that guide the player through specific inputs
- Build context-sensitive controls that change based on game state

Input waiting is particularly useful for creating responsive and interactive gameplay experiences that react to player input in sophisticated ways.