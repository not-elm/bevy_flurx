use bevy::prelude::{In, IntoSystem, Local, System, World};
use crate::selector::condition::{ReactorSystemConfigs, with, WithInput};

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
    #[inline]
    pub const fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    /// Return true if the value is right.
    #[inline]
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
#[inline]
pub fn either<
    LS, LI, LO, LM,
    RS, RI, RO, RM
>(lhs: LS, rhs: RS) -> impl ReactorSystemConfigs<WithInput, In=(LI, RI), Out=Either<LO, RO>>
    where
        LS: ReactorSystemConfigs<LM, In=LI, Out=LO> + 'static,
        RS: ReactorSystemConfigs<RM, In=RI, Out=RO> + 'static,
        LI: Clone + 'static,
        LO: 'static,
        RI: Clone + 'static,
        RO: 'static
{
    let (lin, mut l_system) = lhs.into_configs();
    let (rin, mut r_system) = rhs.into_configs();
    with(
        (lin, rin),
        IntoSystem::into_system(move |In((lin, rin)): In<(LI, RI)>,
                                      world: &mut World,
                                      mut initialized: Local<bool>| {
            if !*initialized {
                l_system.initialize(world);
                r_system.initialize(world);
                *initialized = true;
            }

            if let Some(lout) = l_system.run(lin, world) {
                return Some(Either::Left(lout));
            }

            r_system.run(rin, world).map(|rout| Either::Right(rout))
        }),
    )
}


#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Local, Update, World};

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::{once, wait};
    use crate::selector::condition::wait::{Either, output, until};

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