# once::audio

The `once::audio` module provides actions for playing audio in Bevy exactly once. This module is designed to work with Bevy's audio system.

## Functions

### play

```rust
once::audio::play<Path>() -> ActionSeed<Path, Entity>
```

Creates an action that plays an audio file once. The path to the audio file must be provided using the `.with()` method. The action returns the Entity that the AudioBundle is attached to.

#### Example

```rust
use bevy::prelude::*;
use bevy_flurx::prelude::*;

Reactor::schedule(|task| async move {
    // Play a sound effect once
    let entity = task.will(Update, once::audio::play().with("sounds/explosion.ogg")).await;

    // You can use the returned entity to control the audio later if needed
    println!("Audio playing on entity: {:?}", entity);
});
```

## Audio Paths

The path provided to `once::audio::play()` is loaded using Bevy's AssetServer, so it should be relative to your project's asset directory. The following audio formats are supported by default in Bevy:

- .ogg (Vorbis)
- .mp3
- .wav
- .flac

## When to Use

Use `once::audio` actions when you need to:
- Play sound effects in response to game events
- Start background music
- Create audio sequences as part of game flow

For more complex audio control or when you need to access other system parameters, consider using the more general `once::run` action with the appropriate audio components.
