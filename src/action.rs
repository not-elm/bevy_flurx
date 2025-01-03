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

pub mod once;
pub mod wait;
pub mod delay;
pub mod switch;
pub mod seed;
pub mod through;
pub mod pipe;
pub mod sequence;
pub mod omit;
#[path = "action/tuple.rs"]
mod _tuple;
mod map;
mod remake;
#[cfg(feature = "effect")]
#[cfg_attr(docsrs, doc(cfg(feature = "effect")))]
pub mod effect;
#[cfg(feature = "record")]
#[cfg_attr(docsrs, doc(cfg(feature = "record")))]
pub mod record;

/// Represents the system passed to [`ReactorTask`](crate::task::ReactorTask).
///
/// Please check [here](crate::action) for more details.
#[derive(Reflect)]
pub struct Action<I = (), O = ()>(pub(crate) I, pub(crate) ActionSeed<I, O>);

impl<I1, O1> Action<I1, O1>
where
    I1: 'static,
    O1: 'static,
{
    #[inline(always)]
    pub(crate) fn into_runner(self, output: Output<O1>) -> BoxedRunner {
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
    use std::time::Duration;

    use crate::action::{delay, once, wait};

    #[test]
    fn length_is_0() {
        assert_eq!(actions![].len(), 0);
    }

    #[test]
    fn length_is_1() {
        assert_eq!(actions![once::run(||{})].len(), 1);
        assert_eq!(actions![once::run(||{}),].len(), 1);
    }

    #[test]
    fn length_is_2() {
        assert_eq!(actions![
            once::run(||{}),
            wait::until(||false)
        ].len(), 2);
    }

    #[test]
    fn length_is_3() {
        assert_eq!(actions![
            once::run(||{}),
            wait::until(||false),
            delay::time().with(Duration::from_secs(1))
        ].len(), 3);
    }

    #[test]
    fn last_action_with_comma() {
        assert_eq!(actions![
            once::run(||{}),
            wait::until(||false),
            delay::time().with(Duration::from_secs(1)),
        ].len(), 3);
    }
}