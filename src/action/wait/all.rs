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
            let a = $crate::action::wait::both($action1, $action2);
            $(
            let a = $crate::prelude::wait::all::private::FlatBothRunner::new(a, $action);
            )*
            a
        }
    };
}

#[doc(hidden)]
#[allow(non_snake_case)]
pub mod private {
    use bevy::prelude::{Deref, DerefMut};

    use crate::action::Action;
    use crate::runner::{CancellationToken, RunWithTaskOutput, TaskOutput, TaskRunner};
    use crate::runner::base::BaseTwoRunner;
    use crate::runner::macros::impl_tuple_runner;

    #[derive(Deref, DerefMut)]
    pub struct FlatBothRunner<I1, I2, O1, O2>(BaseTwoRunner<I1, I2, O1, O2>);


    impl<I1, I2, O1, O2> FlatBothRunner<I1, I2, O1, O2>
        where
            I1: 'static,
            I2: 'static,
            O1: 'static,
            O2: 'static,
    {
        #[inline]
        pub fn new(
            a1: impl Action<I1, O1> + 'static,
            a2: impl Action<I2, O2> + 'static,
        ) -> FlatBothRunner<I1, I2, O1, O2> {
            Self(BaseTwoRunner::new(a1, a2))
        }
    }

    macro_rules! impl_wait_both {
        ($($lhs_out: ident$(,)?)*) => {
            impl<I1, I2, $($lhs_out,)* O2> Action< (I1, I2), ($($lhs_out,)* O2)> for FlatBothRunner<I1, I2, ($($lhs_out,)*), O2>
                where
                    I1: 'static,
                    I2: 'static,
                    $($lhs_out: 'static,)*
                    O2: 'static,


            {

                #[inline(always)]
                fn to_runner(self, token: CancellationToken, output: TaskOutput<($($lhs_out,)* O2)>) -> impl TaskRunner {
                    (token, output, self)
                }
            }


            impl<I1, I2, $($lhs_out,)* O2> RunWithTaskOutput<($($lhs_out,)* O2)> for FlatBothRunner<I1, I2, ($($lhs_out,)*), O2>
                where
                    I1: 'static,
                    I2: 'static,
                    $($lhs_out: 'static,)*
                    O2: 'static,


            {
                type In = (I1, I2);

                #[allow(non_snake_case)]
                  fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<($($lhs_out,)* O2)>, world: &mut bevy::prelude::World) -> bool {
                    if self.cancel_if_need(token){
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
                            output.replace(($($lhs_out,)* out2));
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

    use crate::prelude::{once, wait, ActionSeed};
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