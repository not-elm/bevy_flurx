//! Provides the mechanisms  to omit input and/or output types from an action.

use crate::action::Action;
use crate::prelude::{ActionSeed, CancellationHandlers};
use crate::runner::{BoxedRunner, Output, Runner, RunnerIs};
use bevy::prelude::World;

/// [`Omit`] provides a mechanism to omit both input and output types from an action.
pub trait Omit {
    /// This method allows actions to omit generics from their return types,
    /// which is useful for defining groups of actions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// fn omit() -> ActionSeed{
    ///     once::run(|In(num): In<usize>| num).with(1)
    ///         .omit()
    /// }
    /// ```
    fn omit(self) -> ActionSeed;
}

/// [`OmitOutput`] provides a mechanism to omit output type from an action.
pub trait OmitOutput<I, O, A> {
    /// Create an action that converts the output
    /// the action from `O` to `()`.
    ///
    /// This method is useful for defining groups of actions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// fn print_num() -> ActionSeed<usize>{
    ///     once::run(|In(num): In<usize>|{
    ///         format!("{num:}")
    ///     })
    ///         .omit_output()
    /// }
    /// ```
    fn omit_output(self) -> A;
}

/// [`OmitInput`] provides a mechanism to omit input type from an action.
pub trait OmitInput<I, O> {
    /// This method allows actions to omit generics from their return types,
    /// which is useful for defining groups of actions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// fn omit_input() -> ActionSeed<(), usize>{
    ///     once::run(|In(num): In<usize>| num).with(1)
    ///         .omit_input()
    /// }
    /// ```
    fn omit_input(self) -> ActionSeed<(), O>;
}

impl<O> Omit for ActionSeed<(), O>
where
    O: Send + Sync + 'static,
{
    fn omit(self) -> ActionSeed {
        let action: Action<(), O> = self.into();
        action.omit()
    }
}

impl<I, O> Omit for Action<I, O>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    fn omit(self) -> ActionSeed {
        self.omit_output().omit_input()
    }
}

impl<I, O, A> OmitInput<I, O> for A
where
    A: Into<Action<I, O>> + Send + Sync + 'static,
    I: 'static,
    O: 'static,
{
    #[inline]
    fn omit_input(self) -> ActionSeed<(), O> {
        ActionSeed::from(|_, output| self.into().create_runner(output))
    }
}

impl<I, O> OmitOutput<I, O, Action<I, ()>> for Action<I, O>
where
    I: Send + Sync + 'static,
    O: 'static,
{
    #[inline]
    fn omit_output(self) -> Action<I, ()> {
        let Action(input, seed) = self;
        ActionSeed::new(|input, output| {
            let r1 = seed.create_runner(input, Output::default());
            OmitRunner { output, r1 }
        })
            .with(input)
    }
}

impl<I, O> OmitOutput<I, O, ActionSeed<I, ()>> for ActionSeed<I, O>
where
    I: 'static,
    O: 'static,
{
    #[inline]
    fn omit_output(self) -> ActionSeed<I, ()> {
        ActionSeed::new(|input, output| {
            let r1 = self.create_runner(input, Output::default());
            OmitRunner { output, r1 }
        })
    }
}

struct OmitRunner {
    output: Output<()>,
    r1: BoxedRunner,
}

impl Runner for OmitRunner {
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        match self.r1.run(world, token) {
            RunnerIs::Canceled => RunnerIs::Canceled,
            RunnerIs::Running => RunnerIs::Running,
            RunnerIs::Completed => {
                self.output.set(());
                RunnerIs::Completed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::omit::{OmitInput, OmitOutput};
    use crate::action::once;
    use crate::prelude::{Pipe, Reactor};
    use crate::tests::test_app;
    use bevy::app::Startup;
    use bevy::prelude::{Commands, In, ResMut, Update};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn omit_input() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::run(|In(num): In<usize>| num)
                        .with(3)
                        .omit_input()
                        .pipe(once::run(|In(num): In<usize>, mut count: ResMut<Count>| {
                            count.set(num);
                        })),
                )
                    .await;
            }));
        });

        app.update();
        app.assert_resource_eq(Count(3));
    }

    #[test]
    fn omit_output() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::run(|In(num): In<usize>| num)
                        .with(3)
                        .omit_output()
                        .pipe(once::run(|mut count: ResMut<Count>| {
                            count.set(3);
                        })),
                )
                    .await;
            }));
        });

        app.update();
        app.assert_resource_eq(Count(3));
    }
}
