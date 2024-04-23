//! Provides a mechanism for sequentially combining actions.
//!
//! [`Then`] trait is implemented on all actions and can be combined
//! in method chains like `once::run(||{}).then(once::run(||{}))` 
//!
//! It also provides the [`sequence!`](crate::sequence) macro. The behavior itself is the same as [`Then`].
//!
//! trait 
//!
//! [`Then`]
//!
//! macro
//!
//! [`sequence!`](crate::sequence)


use bevy::prelude::World;

use crate::action::{Action, Remake};
use crate::runner::{BoxedRunner, CancellationToken, Output, Runner};

/// Create the action combined with the subsequent action.
///
/// You can create an action that combines multiple actions
/// by connecting them with a method chain.
///
/// You can also use [`sequence!`](crate::sequence) instead of this trait.
pub trait Then<I1, O1, O2, ActionOrSeed> {
    /// Returns the action combined with the subsequent action.
    ///
    /// The action's output will be that of the subsequent action.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// Reactor::schedule(|task| async move{
    ///     task.will(Update, {
    ///         wait::input::just_pressed().with(KeyCode::KeyR)
    ///             .then(once::event::app_exit())
    ///     }).await;
    /// });
    /// ```
    fn then<I2>(self, action: impl Into<Action<I2, O2>> + 'static) -> ActionOrSeed
        where
            I2: 'static;
}


impl<I1, O1, O2, ActionOrSeed, A> Then<I1, O1, O2, ActionOrSeed> for A
    where
        I1: 'static,
        O1: 'static,
        O2: 'static,
        A: Remake<I1, O1, O2, ActionOrSeed> + 'static
{
    fn then<I2>(self, action: impl Into<Action<I2, O2>> + 'static) -> ActionOrSeed
        where
            I2: 'static
    {
        self.remake(|r1, o1, token, output| {
            SequenceRunner {
                r1,
                r2: action.into().into_runner(token.clone(), output),
                o1,
                token,
            }
        })
    }
}

/// Create actions that execute the passed actions in sequence.
///
/// It has advantage that if the previous action finishes,
/// the next will start within in that frame.
///
/// For example, the code below defines three actions,
/// all of which are executed during one frame.
///
/// You can also use [`Then`] instead of this macro.
///
/// The output will be that of the last action passed.
///
/// ```no_run
/// use bevy::app::{App, Update};
/// use bevy::prelude::World;
/// use bevy_flurx::prelude::*;
/// use bevy_flurx::sequence;
///
/// Reactor::schedule(|task|async move{
///     let o = task.will(Update, sequence!{
///         once::run(||{}),
///         once::run(||{}),
///         once::run(||{ 1 + 1}),
///     }).await;
///     assert_eq!(o, 2);
/// });
/// ```
///
#[macro_export]
macro_rules! sequence {
    ($action: expr $(,)?) => {$action};
    ($action1: expr, $action2: expr $(,$action: expr)*$(,)?)  => {
        {
            use $crate::prelude::Then;
            $action1.then($action2)
            $(
            .then($action)
            )*
        }
    };
}

struct SequenceRunner<O1> {
    pub r1: BoxedRunner,
    pub r2: BoxedRunner,
    pub o1: Output<O1>,
    pub token: CancellationToken,
}

impl<O1> Runner for SequenceRunner<O1>
    where
        O1: 'static,
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }

        if self.o1.is_none() {
            self.r1.run(world);
        }
        if self.o1.is_some() {
            self.r2.run(world)
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::Startup;
    use bevy::prelude::{Commands, Resource, Update};
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::once;
    use crate::action::sequence::Then;
    use crate::reactor::Reactor;
    use crate::tests::test_app;

    #[derive(Resource, Eq, PartialEq, Debug)]
    struct Mark1;

    #[derive(Resource, Eq, PartialEq, Debug)]
    struct Mark2;

    #[derive(Resource, Eq, PartialEq, Debug)]
    struct OutputUSize(usize);


    #[test]
    fn two() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::run(|| {})
                    .then(once::res::insert().with(Mark1)),
                ).await;
            }));
        });
        app.update();
        app.assert_resource_eq(Mark1);
    }

    #[test]
    fn three() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::run(|| {})
                    .then(once::res::insert().with(Mark1))
                    .then(once::res::insert().with(Mark2)),
                ).await;
            }));
        });
        app.update();
        app.assert_resource_eq(Mark1);
        app.assert_resource_eq(Mark2);
    }


    #[test]
    fn output_is_2() {
        let mut app = test_app();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let output = task.will(Update, once::run(|| {})
                    .then(once::res::insert().with(Mark1))
                    .then(once::res::insert().with(Mark2))
                    .then(once::run(|| { 1 + 1 })),
                ).await;
                task.will(Update, once::res::insert().with(OutputUSize(output))).await;
            }));
        });
        app.update();
        app.update();
        app.assert_resource_eq(OutputUSize(2));
    }

    #[test]
    fn using_sequence_macro() {
        let mut app = test_app();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let output = task.will(Update, {
                    once::run(|| {})
                        .then(once::res::insert().with(Mark1))
                        .then(once::res::insert().with(Mark2))
                        .then(once::run(|| { 1 + 1 }))
                }).await;
                task.will(Update, once::res::insert().with(OutputUSize(output))).await;
            }));
        });
        app.update();
        app.assert_resource_eq(Mark1);
        app.assert_resource_eq(Mark2);
        app.update();
        app.assert_resource_eq(OutputUSize(2));
    }
}