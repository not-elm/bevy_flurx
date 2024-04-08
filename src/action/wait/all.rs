/// Wait until all tasks done.
///
/// The return value type is tuple, its length is equal to the number of as passed tasks.
///
/// ## Examples
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
/// Reactor::schedule(|task| async move{
///     let (event1, event2, event3, event4) = task.will(Update, wait_all!(
///         wait::event::read::<Event1>(),
///         wait::event::read::<Event2>(),
///         wait::event::read::<Event3>(),
///         wait::event::read::<Event4>(),
///     )).await;
///     assert_eq!(event1, Event1);
///     assert_eq!(event2, Event2);
///     assert_eq!(event3, Event3);
///     assert_eq!(event4, Event4);
/// });
/// ```
#[macro_export]
macro_rules! wait_all {
    ($action: expr $(,)?) => {$action};
    ($action1: expr, $action2: expr $(,$action: expr)*$(,)?)  => {
        {
            #[allow(unused)]
            use $crate::prelude::wait::all::private::CreateBothAction;
            let a = $crate::action::wait::both($action1, $action2);
            $(
            let a = $crate::prelude::wait::all::private::FlatBothRunner::action(a, $action.into());
            )*
            a
        }
    };
}

#[doc(hidden)]
#[allow(non_snake_case)]
pub mod private {
    use std::marker::PhantomData;
    use crate::action::Action;
    use crate::prelude::ActionSeed;
    use crate::runner::{BoxedActionRunner, CancellationToken, Output, Runner};
    use crate::runner::macros::impl_tuple_runner;

    pub struct FlatBothRunner<I1, I2, O1, O2, O> {
        token: CancellationToken,
        o1: Output<O1>,
        o2: Output<O2>,
        r1: BoxedActionRunner,
        r2: BoxedActionRunner,
        output: Output<O>,
        _m: PhantomData<(I1, I2)>
    }
    
    pub trait CreateBothAction<I1, O1, I2, O2, O>{
        fn action(a1: Action<I1, O1>, a2: Action<I2, O2>) -> Action<(I1, I2), O>;
    }
    

    macro_rules! impl_wait_both {
        ($($lhs_out: ident$(,)?)*) => {
             impl<I1, I2, $($lhs_out,)* O2> CreateBothAction<I1, ($($lhs_out,)*), I2, O2, ($($lhs_out,)* O2)> for FlatBothRunner<I1, I2, ($($lhs_out,)*), O2, ($($lhs_out,)* O2)>
                where
                    $($lhs_out: 'static,)*
                    I1: 'static,
                    I2: 'static,
                    O2: 'static,
            {
                fn action(
                    a1: Action<I1, ($($lhs_out,)*)>,
                    a2: Action<I2, O2>,
                ) -> Action<(I1, I2), ($($lhs_out,)* O2)> {
                    let Action(i1, s1) = a1.into();
                    let Action(i2, s2) = a2.into();
                    ActionSeed::new(|input: (I1, I2), token, output|{
                        let o1 = Output::default();
                        let o2 = Output::default();
                        Self {
                            output,
                            r1: s1.with(input.0).into_runner(token.clone(), o1.clone()),
                            r2: s2.with(input.1).into_runner(token.clone(), o2.clone()),
                            o1,
                            o2,
                            token,
                            _m: PhantomData
                        }
                    })
                        .with((i1, i2))
                }
            }

            impl<I1, I2, $($lhs_out,)* O2> Runner for FlatBothRunner<I1, I2, ($($lhs_out,)*), O2, ($($lhs_out,)* O2)>
                where
                    $($lhs_out: 'static,)*
                    I1: 'static,
                    I2: 'static,
                    O2: 'static,
            {
                #[allow(non_snake_case)]
                  fn run(&mut self, world: &mut bevy::prelude::World) -> bool {
                    if self.token.requested_cancel(){
                        return true;
                    }
                    if self.o1.is_none(){
                        self.r1.run(world);
                    }
                    if self.o2.is_none(){
                        self.r2.run(world);
                    }
                    if let Some(($($lhs_out,)*)) = self.o1.take(){
                        if let Some(out2) = self.o2.take(){
                            self.output.replace(($($lhs_out,)* out2));
                            true
                        }else{
                            self.o1.replace(($($lhs_out,)*));
                            false
                        }
                    }else{
                        false
                    }
                }
            }
        };
    }

    impl_tuple_runner!(impl_wait_both);
}

#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, EventWriter, Local};
    use bevy_test_helper::event::{TestEvent1, TestEvent2};

    use crate::prelude::{once, wait};
    use crate::reactor::Reactor;
    use crate::tests::test_app;

    #[test]
    fn wait_all() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    let (event1, event2, ()) = task.will(Update, wait_all!(
                        wait::event::read::<TestEvent1>(),
                        wait::event::read::<TestEvent2>(),
                        wait::until(|mut c: Local<usize>| {
                            *c += 1;
                            *c == 3
                        })
                    )).await;
                    assert_eq!(event1, TestEvent1);
                    assert_eq!(event2, TestEvent2);
                    task.will(Update, once::non_send::insert().with(AppExit)).await;
                }));
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

    #[test]
    fn wait_all_with_once() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    let (event1, ..) = task.will(Update, wait_all!(
                        wait::event::read::<TestEvent1>(),
                        once::run(||{

                        })
                    )).await;
                    assert_eq!(event1, TestEvent1);
                    task.will(Update, once::non_send::insert().with(AppExit)).await;
                }));
            });
        app.world.run_system_once(|mut w: EventWriter<TestEvent1>| w.send(TestEvent1));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}