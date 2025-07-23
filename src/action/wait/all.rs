use crate::prelude::{ActionSeed, Output, Runner};
use crate::runner::{BoxedRunner, CancellationHandlers, RunnerIs};
use bevy::prelude::*;

/// Wait until all the actions are completed.
///
/// The output value of this function is `()`.
/// If you need the outputs, consider using [`wait_all!`](crate::wait_all) instead.
///
/// # Examples
///
/// ```no_run
/// use core::time::Duration;
/// use bevy::prelude::*;
/// use bevy_flurx::actions;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::all().with(actions![
///         once::run(||{}),
///         delay::time().with(Duration::from_millis(300)),
///         wait::input::just_pressed().with(KeyCode::KeyA)
///     ])).await;
/// });
/// ```
pub fn all<Actions>() -> ActionSeed<Actions>
where
    Actions: IntoIterator<Item = ActionSeed> + 'static,
{
    ActionSeed::new(|actions: Actions, output| AllRunner {
        runners: actions
            .into_iter()
            .map(|seed| seed.with(()).create_runner(Output::default()))
            .collect(),
        output,
    })
}

struct AllRunner {
    output: Output<()>,
    runners: Vec<BoxedRunner>,
}

impl Runner for AllRunner {
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        let runners = core::mem::take(&mut self.runners);
        for mut runner in runners {
            match runner.run(world, token) {
                RunnerIs::Canceled => return RunnerIs::Canceled,
                RunnerIs::Completed => {}
                RunnerIs::Running => {
                    self.runners.push(runner);
                }
            }
        }
        if self.runners.is_empty() {
            self.output.set(());
            RunnerIs::Completed
        } else {
            RunnerIs::Running
        }
    }
}

/// Wait until all tasks done.
///
/// The return value type is tuple, its length is equal to the number of as passed tasks.
///
/// If you don't need the outputs of the actions or want to pass a collection of actions,
/// consider using [`wait::all`](crate::prelude::wait::all()) instead.
///
/// ## Examples
///
/// ```
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
///     let (event1, event2, event3, event4) = task.will(Update, wait_all![
///         wait::event::read::<Event1>(),
///         wait::event::read::<Event2>(),
///         wait::event::read::<Event3>(),
///         wait::event::read::<Event4>(),
///     ]).await;
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
            use $crate::prelude::wait::private::CreateBothAction;
            let a = $crate::action::wait::both($action1, $action2);
            $(
            let a = $crate::prelude::wait::private::FlatBothRunner::action(a, $action.into());
            )*
            a
        }
    };
}

#[doc(hidden)]
#[allow(non_snake_case)]
pub mod private {
    use crate::action::Action;
    use crate::prelude::ActionSeed;
    use crate::prelude::RunnerIs;
    use crate::runner::macros::impl_tuple_runner;
    use crate::runner::{BoxedRunner, Output, Runner};
    use core::marker::PhantomData;

    pub struct FlatBothRunner<I1, I2, O1, O2, O> {
        o1: Output<O1>,
        o2: Output<O2>,
        r1: BoxedRunner,
        r2: BoxedRunner,
        output: Output<O>,
        _m: PhantomData<(I1, I2)>,
    }

    pub trait CreateBothAction<I1, O1, I2, O2, O> {
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
                    ActionSeed::new(|input: (I1, I2), output|{
                        let o1 = Output::default();
                        let o2 = Output::default();
                        Self {
                            output,
                            r1: s1.with(input.0).create_runner(o1.clone()),
                            r2: s2.with(input.1).create_runner(o2.clone()),
                            o1,
                            o2,
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
                  fn run(&mut self, world: &mut bevy::prelude::World, token: &mut $crate::prelude::CancellationHandlers) -> RunnerIs {
                    if self.o1.is_none(){
                        match self.r1.run(world, token){
                            RunnerIs::Canceled => return RunnerIs::Canceled,
                            _ => {}
                        }
                    }
                    if self.o2.is_none(){
                        match self.r2.run(world, token){
                            RunnerIs::Canceled => return RunnerIs::Canceled,
                            _ => {}
                        }
                    }
                    if let Some(($($lhs_out,)*)) = self.o1.take(){
                        if let Some(out2) = self.o2.take(){
                            self.output.set(($($lhs_out,)* out2));
                            RunnerIs::Completed
                        }else{
                            self.o1.set(($($lhs_out,)*));
                            RunnerIs::Running
                        }
                    }else{
                        RunnerIs::Running
                    }
                }
            }
        };
    }

    impl_tuple_runner!(impl_wait_both);
}

#[cfg(test)]
mod tests {
    use crate::action::delay;
    use crate::actions;
    use crate::prelude::{once, wait, Pipe, Then};
    use crate::reactor::Reactor;
    use crate::tests::{decrement_count, exit_reader, increment_count, test_app};
    use bevy::app::{AppExit, Startup, Update};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, EventWriter, Local};
    use bevy_test_helper::event::{DirectEvents, TestEvent1, TestEvent2};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn wai_all_actions() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    once::run(|| [increment_count(), increment_count(), decrement_count()])
                        .pipe(wait::all())
                })
                .await;
            }));
        });
        app.update();
        app.assert_resource_eq(Count(1));
    }

    #[test]
    fn with_delay_1frame() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    once::run(|| {
                        actions![
                            increment_count(),
                            delay::frames().with(1),
                            increment_count()
                        ]
                    })
                    .pipe(wait::all())
                    .then(once::event::app_exit_success())
                })
                .await;
            }));
        });
        let mut er = exit_reader();
        app.update();
        app.assert_resource_eq(Count(2));
        app.assert_event_not_comes(&mut er);

        app.update();
        app.assert_resource_eq(Count(2));
        app.assert_event_comes(&mut er);
    }

    #[test]
    fn wait_all() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let (event1, event2, ()) = task
                    .will(
                        Update,
                        wait_all![
                            wait::event::read::<TestEvent1>(),
                            wait::event::read::<TestEvent2>(),
                            wait::until(|mut c: Local<usize>| {
                                *c += 1;
                                *c == 3
                            })
                        ],
                    )
                    .await;
                assert_eq!(event1, TestEvent1);
                assert_eq!(event2, TestEvent2);
                task.will(Update, once::non_send::insert().with(AppExit::Success))
                    .await;
            }));
        });

        app.update();

        app.world_mut()
            .run_system_once(|mut w: EventWriter<TestEvent1>| w.write(TestEvent1))
            .expect("Failed to run system");
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());

        app.world_mut()
            .run_system_once(|mut w: EventWriter<TestEvent2>| w.write(TestEvent2))
            .expect("Failed to run system");
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn wait_all_with_once() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let (event1, ..) = task
                    .will(
                        Update,
                        once::event::send_default::<TestEvent1>().then(wait_all![
                            wait::event::read::<TestEvent1>(),
                            once::run(|| {})
                        ]),
                    )
                    .await;
                assert_eq!(event1, TestEvent1);
                task.will(Update, once::non_send::insert().with(AppExit::Success))
                    .await;
            }));
        });
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }
}
