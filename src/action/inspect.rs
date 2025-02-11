//! # Inspect Module
//!
//! This module provides functionality for extending [`ActionSeed`]s with
//! capabilities to clone and inspect their input via auxiliary actions.
//! It includes a standalone [`inspect`] function and the [`Inspect`] trait for
//! streamlined integration with [`ActionSeed`]s.
//!
//! ## Overview
//!
//! In complex systems, it is often useful to perform side-effects, logging,
//! or auxiliary operations on input values without disrupting their primary
//! flow. This module enables such use cases by introducing the `inspect` pattern,
//! inspired by functional programming and iterator utilities. The primary
//! tools provided are:
//!
//! - **[`inspect`] Function**: A utility to clone an input value, passing one
//!   clone to an auxiliary [`ActionSeed`] for processing, while forwarding
//!   the original input as output.
//! - **[`Inspect`] Trait**: Adds a convenient `.inspect()` method to [`ActionSeed`]s,
//!   simplifying the chaining of actions with auxiliary side-effects.
//!
//! ## Key Features
//! - **Non-Intrusive Observations**: Inspect or modify input data without
//!   altering the original flow.
//! - **Side-Effect Integration**: Execute logging, metrics collection, or
//!   debugging actions alongside main processing logic.
//! - **Rust Idioms**: Adheres to Rust's convention of "inspect"-like functionality
//!   (similar to `Iterator::inspect`), making the API intuitive for developers.
//!
//! ## Example Usage
//!
//! ### Using `inspect` Function
//! ```
//! use bevy::prelude::*;
//! use bevy_flurx::prelude::*;
//!
//! #[derive(Event, Clone)]
//! struct Damage(u8);
//!
//! #[derive(Component)]
//! struct Hp(u8);
//!
//! Reactor::schedule(|task| async move {
//!     task.will(Update, wait::event::read::<Damage>()
//!         .pipe(inspect(once::run(|In(Damage(damage)): In<Damage>| {
//!               println!("Players take {damage} points of damage.");
//!         })))
//!         .pipe(once::run(|In(Damage(damage)): In<Damage>, mut players: Query<&mut Hp>| {
//!             for mut player in &mut players {            
//!                 player.0 -= damage;
//!             }
//!         }))
//!     ).await;
//! });
//! ```
//!
//! ### Using `Inspect` Trait
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_flurx::prelude::*;
//!
//! #[derive(Event, Clone)]
//! struct Damage(u8);
//!
//! #[derive(Component)]
//! struct Hp(u8);
//!
//! Reactor::schedule(|task| async move {
//!     task.will(Update, wait::event::read::<Damage>()
//!         .inspect(once::run(|In(Damage(damage)): In<Damage>| {
//!               println!("Players take {damage} points of damage.");
//!         }))
//!         .pipe(once::run(|In(Damage(damage)): In<Damage>, mut players: Query<&mut Hp>| {
//!             for mut player in &mut players {            
//!                 player.0 -= damage;
//!             }
//!         }))
//!     ).await;
//! });
//! ```
//!
//! ## Modules and Components
//!
//! - **[`inspect`] Function**: Provides the core utility for cloning and processing input.
//! - **[`Inspect`] Trait**: Extends [`ActionSeed`] with the `.inspect()` method.
//!
//! ## When to Use
//! Use this module whenever you need to observe or act upon inputs non-destructively,
//! especially in the context of reactive or ECS-based systems like Bevy. Typical use
//! cases include:
//! - Debugging and logging input values.
//! - Collecting metrics or telemetry.
//! - Triggering auxiliary behaviors based on input values without altering the main flow.

use crate::action::pipe::Pipe;
use crate::action::seed::ActionSeed;
use crate::action::Map;

/// Creates an [`ActionSeed`] that clones its input, passing one clone to the provided
/// `seed` for further processing while forwarding the original input as the output.
///
/// This is useful for observing or inspecting input values by performing side-effects
/// (like logging or metrics) without altering the main input-output chain.
///
/// # Parameters
/// - `seed`: An [`ActionSeed`] that defines a transformation or side effect to
///   perform on a clone of the input data. The result of this transformation is
///   discarded by `inspect`, but any side-effects in `seed` will still occur.
///
/// # Returns
/// An [`ActionSeed`] that outputs the original input (`I`) after processing a
/// cloned version through the provided `seed`.
///
/// # Type Parameters
/// - `I`: The type of the input to the action. Must implement [`Clone`], [`Send`],
///   [`Sync`], and `'static`.
/// - `O`: The output type of the provided `seed`. This type does not affect the
///   output of `inspect` itself.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(Event, Clone)]
/// struct Damage(u8);
///
/// #[derive(Component)]
/// struct Hp(u8);
///
/// Reactor::schedule(|task| async move {
///     task.will(Update, wait::event::read::<Damage>()
///         .pipe(inspect(once::run(|In(Damage(damage)): In<Damage>| {
///             println!("Players take {damage} points of damage.");
///         })))
///         .pipe(once::run(|In(Damage(damage)): In<Damage>, mut players: Query<&mut Hp>| {
///             for mut player in &mut players {
///                 player.0 -= damage;
///             }
///         }))
///     ).await;
/// });
/// ```
///
/// # How It Works
/// - The function takes an input (`I`) and clones it.
/// - The clone is passed to the provided `seed` for processing. Any side-effects
///   in `seed` (e.g., logging, external calls) will be executed.
/// - The original input is forwarded as the output without modification.
///
/// This ensures that you can perform auxiliary operations (e.g., logging, metrics)
/// while preserving the original input for further use.
#[inline(always)]
pub fn inspect<I, O>(seed: ActionSeed<I, O>) -> ActionSeed<I, I>
where
    I: Clone + Send + Sync + 'static,
    O: 'static,
{
    ActionSeed::define(|i: I| seed.with(i.clone()).overwrite(i))
}

/// A trait providing the `inspect` functionality for [`ActionSeed`]s, enabling
/// the cloning of input values to allow auxiliary processing while preserving
/// the original input flow.
///
/// The `inspect` method is particularly useful when you want to observe or
/// inspect an action's input by executing a side effect (like logging or metrics)
/// without altering the main input-output chain.
///
/// # Type Parameters
/// - `I`: The input type of the [`ActionSeed`] being extended.
/// - `O`: The output type of the auxiliary [`ActionSeed`] provided to `inspect`.
/// - `V`: The output type of the extended [`ActionSeed`].
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(Event, Clone)]
/// struct Damage(u8);
///
/// #[derive(Component)]
/// struct Hp(u8);
///
/// Reactor::schedule(|task| async move {
///     task.will(Update, wait::event::read::<Damage>()
///         .inspect(once::run(|In(Damage(damage)): In<Damage>| {
///             println!("Players take {damage} points of damage.");
///         }))
///         .pipe(once::run(|In(Damage(damage)): In<Damage>, mut players: Query<&mut Hp>| {
///             for mut player in &mut players {
///                 player.0 -= damage;
///             }
///         }))
///     ).await;
/// });
/// ```
///
/// In this example:
/// - The `inspect` method enables the side effect to print the input (`Damage(_)`).
/// - The original input is preserved and passed along to the next step in the chain to harm the
///   players.
pub trait Inspect<I, O, V> {
    /// Extends the current [`ActionSeed`] by cloning its input and passing
    /// the clone to the provided auxiliary [`ActionSeed`] for additional processing.
    ///
    /// # Parameters
    /// - `seed`: The auxiliary [`ActionSeed`] that performs a transformation
    ///   or side effect on a cloned version of the input.
    ///
    /// # Returns
    /// A new [`ActionSeed`] that preserves the original input flow while executing
    /// the side effect defined by the provided auxiliary seed.
    fn inspect(self, seed: ActionSeed<V, O>) -> ActionSeed<I, V>;
}

impl<I, O, V> Inspect<I, O, V> for ActionSeed<I, V>
where
    I: 'static,
    O: 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn inspect(self, seed: ActionSeed<V, O>) -> ActionSeed<I, V> {
        self.pipe(inspect(seed))
    }
}
