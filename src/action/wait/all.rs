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
            let t = $crate::prelude::wait::all::private::FlatBothRunner::new(t, $tasks);
            )*
            t
        }
    };
}

#[doc(hidden)]
pub mod private {
    use std::marker::PhantomData;

    use bevy::prelude::World;

    use crate::action::TaskAction;
    use crate::runner::{RunTask, TaskOutput};

    trait RunBoth<O> {
        fn run_both(&mut self, output: &mut TaskOutput<O>, world: &mut World) -> bool;
    }

    impl<O, R: RunBoth<O>> RunTask for (TaskOutput<O>, R) {
        #[inline(always)]
        fn run(&mut self, world: &mut World) -> bool {
            self.1.run_both(&mut self.0, world)
        }
    }

    pub struct FlatBothRunner<I1, I2, O1, O2, M1, M2> {
        r1: Box<dyn RunTask>,
        r2: Box<dyn RunTask>,
        o1: TaskOutput<O1>,
        o2: TaskOutput<O2>,
        _m: PhantomData<(I1, I2, M1, M2)>,
    }

    impl<I1, I2, O1, O2, M1, M2> FlatBothRunner<I1, I2, O1, O2, M1, M2> {
        #[inline]
        pub fn new(
            a1: impl TaskAction<M1, In=I1, Out=O1> + 'static,
            a2: impl TaskAction<M2, In=I2, Out=O2> + 'static,
        ) -> FlatBothRunner<I1, I2, O1, O2, M1, M2>
            where
                M1: 'static,
                M2: 'static
        {
            let o1 = TaskOutput::default();
            let o2 = TaskOutput::default();
            let r1 = a1.to_runner(o1.clone());
            let r2 = a2.to_runner(o2.clone());
            Self {
                r1: Box::new(r1),
                r2: Box::new(r2),
                o1,
                o2,
                _m: PhantomData,
            }
        }
    }

    macro_rules! impl_wait_both {
        ($($lhs_out: ident$(,)?)*) => {
              impl<I1, I2, $($lhs_out,)* O2, M1, M2> TaskAction for FlatBothRunner<I1, I2, ($($lhs_out,)*), O2, M1, M2>
              {
                    type In = (I1, I2);
                    type Out = ($($lhs_out,)* O2);

                    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask {
                        (output, self)
                    }
              }

            impl<I1, I2, $($lhs_out,)* O2, M1, M2> RunBoth<($($lhs_out,)* O2)> for FlatBothRunner<I1, I2, ($($lhs_out,)*), O2, M1, M2> {
                  #[allow(non_snake_case)]
                  fn run_both(&mut self,  output: &mut TaskOutput<($($lhs_out,)* O2)>, world: &mut World) -> bool {
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

    #[test]
    fn wait_all_with_once() {
        let mut app = test_app();
        app
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    let (event1, ..) = task.will(Update, wait_all!(
                        wait::event::read::<TestEvent1>(),
                        once::run(||{

                        })
                    )).await;
                    assert_eq!(event1, TestEvent1);
                    task.will(Update, once::non_send::insert(AppExit)).await;
                });
            });
        app.world.run_system_once(|mut w: EventWriter<TestEvent1>| w.send(TestEvent1));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}