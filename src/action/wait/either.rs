use std::marker::PhantomData;

use crate::action::{TaskAction, WithInput};
use crate::runner::{RunTask, TaskOutput};
use crate::runner::either::EitherRunner;

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
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// #[derive(Default, Clone, Event)]
/// struct Event1;
///
/// #[derive(Default, Clone, Event)]
/// struct Event2;
///
/// let mut app = App::new();
/// app.add_event::<Event1>();
/// app.add_event::<Event2>();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task|async move{
///         let wait_event = task.run(Update, wait::either(
///             wait::event::comes::<Event1>(),
///             wait::event::comes::<Event2>()
///         )).await;
///         task.will(Update, once::event::send(Event1)).await;
///         assert!(wait_event.await.is_left());
///     });
/// });
/// app.update();
/// app.update();
/// app.update();
/// ```
#[inline(always)]
pub fn either<
    LS, LI, LO, LM,
    RS, RI, RO, RM
>(lhs: LS, rhs: RS) -> impl TaskAction<WithInput, In=(LI, RI), Out=Either<LO, RO>>
    where
        LS: TaskAction<LM, In=LI, Out=LO> + 'static,
        RS: TaskAction<RM, In=RI, Out=RO> + 'static,
        LI: 'static,
        LO: 'static,
        RI: 'static,
        RO: 'static,
        LM: 'static,
        RM: 'static
{
    EitherAction {
        lhs,
        rhs,
        _m: PhantomData,
    }
}

struct EitherAction<L, LI, LO, LM, R, RI, RO, RM> {
    lhs: L,
    rhs: R,
    _m: PhantomData<(LI, LO, RI, RO, LM, RM)>,
}

impl<
    L, LI, LO, LM,
    R, RI, RO, RM
> TaskAction for EitherAction<L, LI, LO, LM, R, RI, RO, RM>
    where
        L: TaskAction<LM, In=LI, Out=LO> + 'static,
        R: TaskAction<RM, In=RI, Out=RO> + 'static,
        LM: 'static,
        RM: 'static
{
    type In = (LI, RI);
    type Out = Either<LO, RO>;

    #[inline(always)]
    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask {
        EitherRunner::new(output, self.lhs, self.rhs)
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Local, Update, World};

    use crate::action::{once, wait};
    use crate::action::wait::{Either, output, until};
    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;

    #[test]
    fn wait_either() {
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);
        #[derive(Clone)]
        struct Count(usize);
        app.world.run_system_once(|world: &mut World| {
            world.schedule_reactor(|task| async move {
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
            });
        });

        app.update();
        assert!(app.world.get_non_send_resource::<Count>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<Count>().is_none());

        app.update();
        assert_eq!(app.world.non_send_resource::<Count>().0, 1);
    }
}