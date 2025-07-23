# wait::audio

The `wait::audio` module provides actions for waiting for audio playback to finish. These actions are useful for coordinating tasks with audio playback, allowing tasks to wait for audio to complete before continuing execution.

## Functions

### finished

```rust
wait::audio::finished() -> ActionSeed<Entity, ()>
```

Creates an action that waits until the audio associated with the passed `Entity` has finished playing. The action completes when the audio playback is complete.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Play a sound effect and get the entity
    let entity = task.will(Update, once::audio::play().with("sounds/explosion.ogg")).await;
    
    // Wait for the sound to finish playing
    task.will(Update, wait::audio::finished().with(entity)).await;
    
    // This code runs after the sound has finished playing
    println!("Sound effect has finished playing!");
});
```

## When to Use

Use `wait::audio` actions when you need to:
- Coordinate tasks with audio playback
- Create sequences of sounds with precise timing
- Ensure an action only occurs after a sound has finished playing
- Implement audio-driven gameplay elements

Audio waiting is particularly useful for creating polished audio experiences in games, such as ensuring dialog lines don't overlap, creating rhythmic sequences, or synchronizing gameplay events with audio cues.