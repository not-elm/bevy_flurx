/// Wait until all tasks done.
///
/// The return value type is tuple, its length is equal to the number of as passed tasks.
///
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// use bevy_flurx::wait_all;
///
/// #[derive(Default, Clone, Event, PartialEq, Debug)]
/// struct Event1;
/// #[derive(Default, Clone, Event, PartialEq, Debug)]
/// struct Event2;
/// #[derive(Default, Clone, Event, PartialEq, Debug)]
/// struct Event3;
/// #[derive(Default, Clone, Event, PartialEq, Debug)]
/// struct Event4;
///
/// let mut app = App::new();
/// app.add_event::<Event1>();
/// app.add_event::<Event2>();
/// app.add_event::<Event3>();
/// app.add_event::<Event4>();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move{
///         let (event1, event2, event3, event4) = task.will(Update, wait_all!(
///             wait::event::read::<Event1>(),
///             wait::event::read::<Event2>(),
///             wait::event::read::<Event3>(),
///             wait::event::read::<Event4>(),
///         )).await;
///         assert_eq!(event1, Event1);
///         assert_eq!(event2, Event2);
///         assert_eq!(event3, Event3);
///         assert_eq!(event4, Event4);
///     });
/// });
/// app.update();
/// app.world.resource_mut::<Events<Event1>>().send_default();
/// app.world.resource_mut::<Events<Event2>>().send_default();
/// app.world.resource_mut::<Events<Event3>>().send_default();
/// app.world.resource_mut::<Events<Event4>>().send_default();
/// app.update();
/// ```
#[macro_export]
macro_rules! wait_all {
    ($t1: expr) => {$t};
    ($t1: expr, $t2: expr $(,$tasks: expr)*$(,)?)  => {
        {
            let t = $crate::action::wait::both($t1, $t2);
            $(
            let t = $crate::prelude::wait::all::private::WaitBoth::new(&t).both(t, $tasks);
            )*
            t
        }
    };
}

#[doc(hidden)]
pub mod private {
    use std::marker::PhantomData;
    use bevy::prelude::{In, IntoSystem, Local, World};
    use crate::prelude::{ReactorAction, WithInput};

    #[repr(transparent)]
    pub struct WaitBoth<Out>(PhantomData<Out>);
    
    impl<Out> WaitBoth<Out> {
        #[inline]
        pub const fn new<M, In>(_: &impl ReactorAction<M, In=In, Out=Out>) -> WaitBoth<Out> {
            Self(PhantomData)
        }
    }

    macro_rules! impl_wait_both {
        ($($lhs_out: ident$(,)?)*) => {
            impl<$($lhs_out,)*> WaitBoth<($($lhs_out,)*)>
                where
                    $($lhs_out: Send + 'static,)*
            {
                #[allow(non_snake_case)]
                pub fn both<LIn, LM, RIn, ROut, RM>(
                    &self,
                    lhs: impl ReactorAction<LM, In=LIn, Out=($($lhs_out,)*)> + 'static,
                    rhs: impl ReactorAction<RM, In=RIn, Out=ROut> + 'static,
                ) -> impl ReactorAction<WithInput, In=(LIn, RIn), Out=($($lhs_out,)* ROut)>
                    where
                        RIn: Clone + 'static,
                        LIn: Clone + 'static,
                        ROut: Send + 'static,
                {
                    let (l_in, mut l_sys) = lhs.split();
                    let (r_in, mut r_sys) = rhs.split();
    
                    $crate::action::with(
                        (l_in, r_in),
                        IntoSystem::into_system(
                            move |In((l_in, r_in)): In<(LIn, RIn)>,
                                  world: &mut World,
                                  mut init: Local<bool>,
                                  mut l_out: Local<Option<($($lhs_out,)*)>>,
                                  mut r_out: Local<Option<ROut>>| {
                                $crate::prelude::wait::both_init_systems(&mut init, world, &mut l_sys, &mut r_sys);
                                let (($($lhs_out,)*), r_out) = $crate::prelude::wait::both_run_systems(world, l_in, &mut l_sys, &mut l_out, r_in, &mut r_sys, &mut r_out)?;
                                Some(($($lhs_out,)* r_out))
                            },
                        ),
                    )
                }
            }
        };
    }

    impl_wait_both!(In1);
    impl_wait_both!(In1,In2);
    impl_wait_both!(In1,In2,In3);
    impl_wait_both!(In1,In2,In3,In4);
    impl_wait_both!(In1,In2,In3,In4,In5);
    impl_wait_both!(In1,In2,In3,In4,In5,In6);
    impl_wait_both!(In1,In2,In3,In4,In5,In6,In7);
    impl_wait_both!(In1,In2,In3,In4,In5,In6,In7,In8);
    impl_wait_both!(In1,In2,In3,In4,In5,In6,In7,In8,In9);
    impl_wait_both!(In1,In2,In3,In4,In5,In6,In7,In8,In9,In10);
    impl_wait_both!(In1,In2,In3,In4,In5,In6,In7,In8,In9,In10,In11);
    impl_wait_both!(In1,In2,In3,In4,In5,In6,In7,In8,In9,In10,In11,In12);
}

#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{EventWriter, Local, World};
    use bevy_test_helper::event::{TestEvent1, TestEvent2};

    use crate::extension::ScheduleReactor;
    use crate::prelude::{once, wait};
    use crate::tests::test_app;

    #[test]
    fn wait_all() {
        let mut app = test_app();
        app
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    let t1 = wait::event::read::<TestEvent1>();
                    let t2 = wait::event::read::<TestEvent2>();
                    let count = wait::until(|mut c: Local<usize>| {
                        *c += 1;
                        *c == 3
                    });
                    let (event1, event2, ..) = task.will(Update, wait_all!(t1, t2, count)).await;
                    assert_eq!(event1, TestEvent1);
                    assert_eq!(event2, TestEvent2);
                    task.will(Update, once::non_send::insert(AppExit)).await;
                });
            });

        app.update();

        app.world.run_system_once(|mut w: EventWriter<TestEvent1>| w.send(TestEvent1));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.world.run_system_once(|mut w: EventWriter<TestEvent2>| w.send(TestEvent2));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}