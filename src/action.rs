//! An `action` is a system to be run on the [`Reactor`](crate::prelude::Reactor).
//!
//! It is scheduled by [`Reactor`](crate::prelude::Reactor) and is run once per frame(twice if uninitialized).
//!
//! Every action has an end condition, and if the condition is met, the next action proceeds.
//!
//! For example, in the following code, the exit condition is to wait until the count reaches 2,
//! and when it reaches 2, proceed to `process: 2`.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_flurx::prelude::*;
//!
//! Reactor::schedule(|task| async move{
//!     // `process: 1`
//!     task.will(Update, wait::until(|mut count: Local<usize>|{
//!         *count += 1;
//!         *count == 2
//!     })).await;
//!     // `process: 2`
//! });
//! ```

use crate::prelude::ActionSeed;
use crate::runner::{BoxedRunner, Output};
pub use _tuple::tuple;
use bevy::prelude::Reflect;
pub use map::Map;
pub use remake::Remake;

#[path = "action/tuple.rs"]
mod _tuple;
pub mod delay;
pub mod inspect;
mod map;
pub mod omit;
pub mod once;
pub mod pipe;
mod remake;
pub mod seed;
pub mod sequence;
#[cfg(feature = "side-effect")]
#[cfg_attr(docsrs, doc(cfg(feature = "side-effect")))]
pub mod side_effect;
pub mod switch;
pub mod through;
pub mod wait;

#[cfg(feature = "record")]
#[cfg_attr(docsrs, doc(cfg(feature = "record")))]
pub mod record;

/// Represents the system passed to [`ReactorTask`](crate::task::ReactorTask).
///
/// Please check [here](crate::action) for more details.
#[derive(Reflect)]
pub struct Action<I = (), O = ()>(pub I, pub ActionSeed<I, O>);

impl<I1, O1> Action<I1, O1>
where
    I1: 'static,
    O1: 'static,
{
    /// Splits itself into an input value and an action seed.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use bevy_flurx::prelude::*;
    ///
    /// let action: Action<usize> = delay::frames().with(10);
    /// let (_input, _seed): (usize, ActionSeed<usize>) = action.split();
    /// ```
    #[inline]
    pub fn split(self) -> (I1, ActionSeed<I1, O1>) {
        (self.0, self.1)
    }

    /// Creates the [`BoxedRunner`].
    ///
    /// This method is mainly useful for creating custom runners.
    /// For example, when creating a new runner that extends an existing one.
    #[inline]
    pub fn create_runner(self, output: Output<O1>) -> BoxedRunner {
        self.1.create_runner(self.0, output)
    }
}

impl<Out> From<ActionSeed<(), Out>> for Action<(), Out>
where
    Out: 'static,
{
    #[inline]
    fn from(value: ActionSeed<(), Out>) -> Self {
        value.with(())
    }
}

impl<I, O> Default for Action<I, O>
where
    I: Default + 'static,
    O: Default + 'static,
{
    fn default() -> Self {
        ActionSeed::<I, O>::default().with(I::default())
    }
}

/// Creates a \[[`ActionSeed`]; N\] containing the omitted actions.
///
/// # Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy_flurx::actions;
/// use bevy_flurx::prelude::*;
///
/// let actions: [ActionSeed; 3] = actions![
///     once::run(||{}),
///     delay::frames().with(3),
///     wait::event::comes::<AppExit>()
/// ];
/// ```
#[macro_export]
macro_rules! actions {
    () => (
        {
            let actions: [$crate::prelude::ActionSeed; 0] = [];
            actions
        }
    );
    ($action: expr $(,$others: expr)* $(,)?) => (
        {
            use $crate::prelude::Omit;
            [
                $action.omit(),
                $($others.omit(),)*
            ]
        }
    );
}

#[cfg(test)]
mod tests {
    use crate::action::{delay, once, wait};
    use core::time::Duration;

    #[test]
    fn length_is_0() {
        assert_eq!(actions![].len(), 0);
    }

    #[test]
    fn length_is_1() {
        assert_eq!(actions![once::run(|| {})].len(), 1);
        assert_eq!(actions![once::run(|| {}),].len(), 1);
    }

    #[test]
    fn length_is_2() {
        assert_eq!(actions![once::run(|| {}), wait::until(|| false)].len(), 2);
    }

    #[test]
    fn length_is_3() {
        assert_eq!(
            actions![
                once::run(|| {}),
                wait::until(|| false),
                delay::time().with(Duration::from_secs(1))
            ]
            .len(),
            3
        );
    }

    #[test]
    fn last_action_with_comma() {
        assert_eq!(
            actions![
                once::run(|| {}),
                wait::until(|| false),
                delay::time().with(Duration::from_secs(1)),
            ]
            .len(),
            3
        );
    }
}
