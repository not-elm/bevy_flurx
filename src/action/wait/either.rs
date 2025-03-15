use crate::action::Action;
use crate::prelude::{ActionSeed, BoxedRunner, RunnerIs};
use crate::runner::{CancellationHandlers, Output, Runner};
use bevy::prelude::World;

/// This enum represents the result of [`wait::either`](crate::prelude::wait::either).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Either<L, R> {
    /// The result of the first task which passed to [`wait::either`](crate::prelude::wait::either).
    Left(L),

    /// The result of the second task which passed to [`wait::either`](crate::prelude::wait::either).
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Return true if the value is left.
    #[inline(always)]
    pub const fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    /// Return true if the value is right.
    #[inline(always)]
    pub const fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }
}

/// Waits until either of the two tasks is completed.
///
/// The first thing passed is lhs, the second is rhs.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     let either = task.will(Update, wait::either(
///         wait::until(||false),
///         wait::event::read::<AppExit>()
///     )).await;
///     match either {
///         Either::Left(_) => {}
///         Either::Right(_) => {}
///     }
/// });
/// ```
#[inline(always)]
pub fn either<LI, LO, RI, RO>(
    lhs: impl Into<Action<LI, LO>> + 'static,
    rhs: impl Into<Action<RI, RO>> + 'static,
) -> Action<(LI, RI), Either<LO, RO>>
where
    LI: 'static,
    LO: 'static,
    RI: 'static,
    RO: 'static,
{
    let Action(li, ls) = lhs.into();
    let Action(ri, rs) = rhs.into();
    ActionSeed::new(move |input: (LI, RI), output| {
        let o1 = Output::default();
        let o2 = Output::default();
        EitherRunner {
            r1: ls.with(input.0).create_runner(o1.clone()),
            r2: rs.with(input.1).create_runner(o2.clone()),
            o1,
            o2,
            output,
        }
    })
        .with((li, ri))
}

struct EitherRunner<O1, O2> {
    r1: BoxedRunner,
    r2: BoxedRunner,
    o1: Output<O1>,
    o2: Output<O2>,
    output: Output<Either<O1, O2>>,
}

impl<O1, O2> Runner for EitherRunner<O1, O2>
where
    O1: 'static,
    O2: 'static,
{
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> crate::prelude::RunnerIs {
        match self.r1.run(world, token) {
            RunnerIs::Canceled => return RunnerIs::Canceled,
            RunnerIs::Running => {}
            RunnerIs::Completed => {
                let lhs = self.o1.take().expect("An output value hasn't been set!!!");
                self.output.set(Either::Left(lhs));
                return RunnerIs::Completed;
            }
        }

        match self.r2.run(world, token) {
            RunnerIs::Canceled => RunnerIs::Canceled,
            RunnerIs::Running => RunnerIs::Running,
            RunnerIs::Completed => {
                let rhs = self.o2.take().expect("An output value hasn't been set!!!");
                self.output.set(Either::Right(rhs));
                RunnerIs::Completed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::wait::{output, until, Either};
    use crate::action::{once, wait};
    use crate::prelude::Reactor;
    use crate::tests::test_app;
    use crate::wait_all;
    use bevy::app::Startup;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::ButtonInput;
    use bevy::prelude::{Commands, KeyCode, Local, ResMut, Resource, Update};
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn wait_either() {
        let mut app = test_app();
        app.finish();
        
        #[derive(Clone)]
        struct Count(usize);
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    let u1 = until(|mut count: Local<u32>| {
                        *count += 1;
                        *count == 3
                    });

                    let u2 = output(|mut count: Local<u32>| {
                        *count += 1;
                        (*count == 2).then_some(1)
                    });

                    if let Either::Right(rhs) = task.will(Update, wait::either(u1, u2)).await {
                        task.will(Update, once::non_send::insert().with(Count(rhs)))
                            .await;
                    }
                }));
            })
            .expect("Failed to run system");

        app.update();
        assert!(app.world().get_non_send_resource::<Count>().is_none());
        app.update();
        assert_eq!(app.world().non_send_resource::<Count>().0, 1);
    }

    #[test]
    fn no_run_after_either() {
        #[derive(Resource, Default, Debug, Eq, PartialEq)]
        struct Count(usize);

        let mut app = test_app();
        app.init_resource::<Count>();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    wait::either(
                        wait_all! {
                                wait::until(|mut count:ResMut<Count>| {
                                    count.0 += 1;
                                    false
                                }),
                                wait::until(|| { false })
                            },
                        wait::input::pressed().with(KeyCode::KeyA),
                    ),
                )
                    .await;
                task.will(Update, wait::until(|| false)).await;
            }));
        });

        app
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyA);
        for _ in 0..100 {
            app.update();
            app.assert_resource_eq(Count(1));
        }
    }
}
