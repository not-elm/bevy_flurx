//! trait
//!
//! - [`Through`]
//!
//! actions
//!
//! - [`through`]

use crate::action::pipe::Pipe;
use crate::action::seed::ActionSeed;
use crate::prelude::{Action, CancellationHandlers};
use crate::runner::{BoxedRunner, Output, Runner, RunnerIs};
use bevy::prelude::World;

/// This function is used when you want to insert some kind of action,
/// such as a delay, between the action that sends output and the action that receives it.
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(Event, Clone)]
/// struct Damage(usize);
///
/// Reactor::schedule(|task|async move{
///     task.will(Update, wait::event::read::<Damage>()
///         .pipe(through(delay::time().with(Duration::from_millis(500))))
///         .pipe(once::run(|In(Damage(damage)): In<Damage>|{
///               println!("Player takes {damage} points of damage.");
///         }))
///     ).await;
/// });
/// ```
#[inline(always)]
pub fn through<V, I, O>(action: impl Into<Action<I, O>> + Send + Sync + 'static) -> ActionSeed<V, V>
where
    V: 'static,
    I: 'static,
    O: 'static,
{
    ActionSeed::new(|input, output| ThroughRunner {
        value: Some(input),
        output,
        inner: action.into().create_runner(Output::default()),
    })
}

/// Provides a method version of [`through`].
pub trait Through<I1, O1, O2, ActionOrSeed> {
    ///
    /// This method is syntax sugar for `self.pipe(through(action))`.
    ///
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// #[derive(Event, Clone)]
    /// struct Damage(usize);
    ///
    /// Reactor::schedule(|task|async move{
    ///     task.will(Update, wait::event::read::<Damage>()
    ///         .through(delay::time().with(Duration::from_millis(500)))
    ///         .pipe(once::run(|In(Damage(damage)): In<Damage>|{
    ///               println!("Player takes {damage} points of damage.");
    ///         }))
    ///     ).await;
    /// });
    /// ```
    fn through<I2>(self, action: impl Into<Action<I2, O2>> + Send + Sync + 'static) -> ActionOrSeed
    where
        I2: 'static;
}

impl<I1, O1, O2> Through<I1, O1, O2, ActionSeed<I1, O1>> for ActionSeed<I1, O1>
where
    I1: 'static,
    O1: 'static,
    O2: 'static,
{
    #[inline]
    fn through<I2>(self, action: impl Into<Action<I2, O2>> + Send + Sync + 'static) -> ActionSeed<I1, O1>
    where
        I2: 'static,
    {
        self.pipe(through(action))
    }
}

impl<I1, O1, O2> Through<I1, O1, O2, Action<I1, O1>> for Action<I1, O1>
where
    I1: 'static,
    O1: 'static,
    O2: 'static,
{
    #[inline]
    fn through<I2>(self, action: impl Into<Action<I2, O2>> + Send + Sync + 'static) -> Action<I1, O1>
    where
        I2: 'static,
    {
        self.pipe(through(action))
    }
}

struct ThroughRunner<V> {
    value: Option<V>,
    output: Output<V>,
    inner: BoxedRunner,
}

impl<V> Runner for ThroughRunner<V>
where
    V: 'static,
{
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        match self.inner.run(world, token) {
            RunnerIs::Completed => {
                self.output.set(self.value.take().expect("Failed to take the action runner's output value in `ThroughRunner`!"));
                RunnerIs::Completed
            }
            other => other
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::once;
    use crate::action::pipe::Pipe;
    use crate::action::through::Through;
    use crate::prelude::Reactor;
    use crate::tests::test_app;
    use bevy::app::{Startup, Update};
    use bevy::prelude::{Commands, In, Resource};
    use bevy_test_helper::resource::DirectResourceControl;

    #[derive(Resource, Eq, PartialEq, Debug)]
    struct Count(usize);

    #[test]
    fn through_output_num1() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::run(|| 1usize)
                        .through(once::run(|| {}))
                        .pipe(once::run(|In(num): In<usize>, mut commands: Commands| {
                            commands.insert_resource(Count(num));
                        })),
                )
                    .await;
            }));
        });
        app.update();

        app.assert_resource_eq(Count(1));
    }
}
