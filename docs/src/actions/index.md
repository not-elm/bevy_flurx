# Actions

Actions are the core building blocks of bevy_flurx. They define how tasks behave and interact with the Bevy ECS. This section provides an overview of the different types of actions available in bevy_flurx.

## Action Types

bevy_flurx provides several types of actions for different use cases:

### once

The `once` module defines actions that run only once. These are useful for performing one-time operations, such as initializing resources, spawning entities, or sending events.

[Learn more about once actions](./once/index.md)

### wait

The `wait` module defines actions that continue to execute every frame according to specified conditions. These are useful for waiting for user input, events, or other conditions to be met.

[Learn more about wait actions](./wait/index.md)

### delay

The `delay` module defines actions that perform delay processing. These are useful for waiting for a specific amount of time or a specific number of frames.

[Learn more about delay actions](./delay.md)

### omit

The `omit` module provides mechanisms to omit input and/or output types from an action. This is particularly useful for defining groups of actions by simplifying their type signatures.

[Learn more about omit actions](./omit.md)

### sequence

The `sequence` module provides mechanisms for sequentially combining actions. This is particularly useful for creating complex action flows by chaining multiple actions together.

[Learn more about sequence actions](./sequence.md)

### pipe

The `pipe` module provides a mechanism to pipe actions together, where the output of one action is used as the input for another action. This is particularly useful for creating data processing pipelines.

[Learn more about pipe actions](./pipe.md)

### through

The `through` module provides a mechanism to execute an action while preserving the output of the previous action. This is particularly useful for inserting actions like delays into a pipeline without affecting the data flow.

[Learn more about through actions](./through.md)

### map

The `map` module provides mechanisms to transform the output of an action using a mapping function. This is particularly useful for data transformation and type conversion between actions.

[Learn more about map actions](./map.md)

### inspect

The `inspect` module provides mechanisms to clone and inspect input values via auxiliary actions without disrupting their primary flow. This is particularly useful for debugging, logging, or performing side-effects on input values.

[Learn more about inspect actions](./inspect.md)

### remake

The `remake` module provides a mechanism to create a new action based on an existing action's `Runner` and `Output`. This is particularly useful for transforming actions while preserving their input type but changing their output type.

[Learn more about remake actions](./remake.md)

## Action Chaining

bevy_flurx provides several ways to chain actions together:

### then

The `then` method allows you to execute one action after another. The output of the first action is discarded, and the output of the second action is returned.

### pipe

The `pipe` method allows you to pass the output of one action as the input to another action.

### through

The `through` method allows you to execute an action while preserving the output of the previous action.

## Feature-Gated Actions

bevy_flurx provides additional actions through feature flags:

### audio

The `audio` feature provides actions for audio playback and waiting.

### record

The `record` feature provides actions for undo/redo functionality.

### side-effect

The `side-effect` feature provides actions for handling side effects like asynchronous runtime or threads.

### state

The `state` feature provides actions for state management.

### tokio

The `tokio` feature allows you to use tokio's runtime directly in the reactor.

## Next Steps

Explore the specific action types to learn more about their capabilities and how to use them:

- [once](./once.md)
- [wait](./wait.md)
- [delay](./delay.md)
- [omit](./omit.md)
- [sequence](./sequence.md)
- [pipe](./pipe.md)
- [through](./through.md)
- [map](./map.md)
- [inspect](./inspect.md)
- [remake](./remake.md)
