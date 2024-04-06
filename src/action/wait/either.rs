use crate::action::TaskAction;
use crate::runner::base::BaseTwoRunner;
use crate::runner::either::EitherRunner;
use crate::runner::RunnerIntoAction;

/// This enum represents the result of [`wait::either`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Either<L, R> {
    /// The result of the first task which passed to [`wait::either`].
    Left(L),

    /// The result of the second task which passed to [`wait::either`].
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
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Flurx::schedule(|task| async move{
///     let either = task.will(Update, wait::either(
///         wait::until(||false),
///         wait::event::read::<AppExit>()
///     )).await;
///     match either { 
///         Either::Left(_) => {},
///         Either::Right(_) => {}
///     }
/// });
/// ```
#[inline(always)]
pub fn either<
    LS, LI, LO,
    RS, RI, RO,
>(lhs: LS, rhs: RS) -> impl TaskAction<(LI, RI), Either<LO, RO>>
    where
        LS: TaskAction<LI, LO> + 'static,
        RS: TaskAction<RI, RO> + 'static,
        LI: 'static,
        LO: 'static,
        RI: 'static,
        RO: 'static,
{
    RunnerIntoAction::new(EitherRunner(BaseTwoRunner::new(lhs, rhs)))
}


#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::ButtonInput;
    use bevy::prelude::{Commands, KeyCode, Local, ResMut, Resource, Update};
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::{FlurxPlugin, wait_all};
    use crate::action::{once, wait};
    use crate::prelude::ActionSeed;
    use crate::action::wait::{Either, output, until};
    use crate::scheduler::Flurx;
    use crate::tests::test_app;

    #[test]
    fn wait_either() {
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);
        #[derive(Clone)]
        struct Count(usize);
        app.world.run_system_once(|mut commands: Commands| {
            commands.spawn(Flurx::schedule(|task| async move {
                let u1 = until(|mut count: Local<u32>| {
                    *count += 1;
                    *count == 3
                });

                let u2 = output(|mut count: Local<u32>| {
                    *count += 1;
                    (*count == 2).then_some(1)
                });

                if let Either::Right(rhs) = task.will(Update, wait::either(u1, u2)).await {
                    task.will(Update, once::non_send::insert(Count(rhs))).await;
                }
            }));
        });

        app.update();
        assert!(app.world.get_non_send_resource::<Count>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<Count>().is_none());

        app.update();
        assert_eq!(app.world.non_send_resource::<Count>().0, 1);
    }

    #[test]
    fn no_run_after_either() {
        #[derive(Resource, Default, Debug, Eq, PartialEq)]
        struct Count(usize);

        let mut app = test_app();
        app.init_resource::<Count>();

        app.world.run_system_once(|mut commands: Commands| {
            commands.spawn(Flurx::schedule(|task| async move {
                task.will(Update, wait::either(
                    wait_all! {
                        wait::until(|mut count:ResMut<Count>| {
                            println!("DDD");
                            count.0 += 1;
                            false
                        }),
                        wait::until(|| { false })
                    },
                    wait::input::pressed().with(KeyCode::KeyA),
                )).await;
                task.will(Update, wait::until(|| { false })).await;
            }));
        });

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyA);
        for _ in 0..100 {
            app.update();
            app.assert_resource_eq(Count(1));
        }
    }
}