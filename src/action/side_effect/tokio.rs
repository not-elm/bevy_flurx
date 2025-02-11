//! Convert the tokio tasks into [`Action`](crate::prelude::Action).
//!
//! action
//!
//! - [`side_effect::tokio::spawn`](crate::prelude::side_effect::tokio::spawn)

use std::marker::PhantomData;
use std::sync::Arc;

use async_compat::CompatExt;
use bevy::prelude::World;
use tokio::task::JoinHandle;

use crate::action::side_effect::AsyncFunctor;
use crate::prelude::{ActionSeed, CancellationHandlers, RunnerIs};
use crate::runner::{Output, Runner};

/// Spawns a new tokio task, and then wait its output.
///
/// The task is started when [`Runner`] is executed for the first time.
///
/// # Example
///
/// ```no_run
///
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, {
///         once::run(|| 2)
///             .pipe(side_effect::tokio::spawn(|num: usize| async move{
///                 num + 3
///             }))
///             .pipe(once::run(|In(num): In<usize>|{
///                 assert_eq!(num, 5);
///             }))
///     }).await;   
/// });
/// ```
pub fn spawn<I, Out, Functor, M>(f: Functor) -> ActionSeed<I, Out>
where
    I: Send + Sync + 'static,
    M: Send + Sync + 'static,
    Out: Send + Sync + 'static,
    Functor: AsyncFunctor<I, Out, M> + Send + Sync + 'static,
{
    ActionSeed::new(|input: I, output: Output<Out>| {
        TokioRunner {
            arc_output: Arc::new(tokio::sync::Mutex::new(None)),
            args: Some((input, f)),
            output,
            handle: None,
            _m: PhantomData,
        }
    })
}

struct TokioRunner<I, Out, Functor, M>

{
    args: Option<(I, Functor)>,
    arc_output: Arc<tokio::sync::Mutex<Option<Out>>>,
    output: Output<Out>,
    handle: Option<JoinHandle<()>>,
    _m: PhantomData<M>,
}

impl<I, Out, Functor, M> Runner for TokioRunner<I, Out, Functor, M>
where
    I: Send + 'static,
    Functor: AsyncFunctor<I, Out, M> + Send + 'static,
    M: Send + 'static,
    Out: Send + 'static,
{
    #[allow(clippy::async_yields_async)]
    fn run(&mut self, _: &mut World, _: &mut CancellationHandlers) -> RunnerIs {
        if let Some((input, functor)) = self.args.take() {
            let arc_output = self.arc_output.clone();
            self.handle.replace(pollster::block_on(async move {
                tokio::spawn(async move {
                    arc_output.lock().await.replace(functor.functor(input).await);
                })
            }.compat()));
        }

        if let Some(out) = self.arc_output.blocking_lock().take() {
            self.output.set(out);
            RunnerIs::Completed
        } else {
            RunnerIs::Running
        }
    }
}

impl<I, Out, Functor, M> Drop for TokioRunner<I, Out, Functor, M> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}


#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use crate::action::{delay, once, side_effect, wait};
    use crate::actions;
    use crate::prelude::{Pipe, Reactor, Then};
    use crate::tests::{exit_reader, test_app};
    use bevy::app::Startup;
    use bevy::prelude::{Commands, In, ResMut, Update};
    use bevy_test_helper::event::DirectEvents;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn tokio_task_with_input() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, side_effect::tokio::spawn(|_| async move { 1 + 1 })
                    .pipe(once::run(|In(num): In<usize>, mut count: ResMut<Count>| {
                        count.0 = num;
                    })),
                ).await;
            }));
        });
        app.update();
        std::thread::sleep(Duration::from_millis(10));
        app.update();
        app.assert_resource_eq(Count(2));
    }

    #[test]
    fn tokio_task_without_input() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, side_effect::tokio::spawn(async move { 1 + 1 })
                    .pipe(once::run(|In(num): In<usize>, mut count: ResMut<Count>| {
                        count.0 = num;
                    })),
                ).await;
            }));
        });
        app.update();
        std::thread::sleep(Duration::from_millis(10));
        app.update();
        app.assert_resource_eq(Count(2));
    }

    #[test]
    fn cancel_tokio_task() {
        let mut app = test_app();
        static TASK_FINISHED: AtomicBool = AtomicBool::new(false);

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    wait::either(
                        once::run(|| {}),
                        side_effect::tokio::spawn(|_| async move {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            TASK_FINISHED.store(true, Ordering::Relaxed);
                        })
                            .then(once::event::app_exit_success()),
                    )
                        // keep running while the test is running
                        .then(delay::time().with(Duration::from_secs(1000)))
                }).await;
            }));
        });
        let mut er = exit_reader();
        app.update();
        std::thread::sleep(Duration::from_millis(200));
        app.assert_event_not_comes(&mut er);
        assert!(!TASK_FINISHED.load(Ordering::Relaxed));
    }

    #[test]
    fn cancel_tokio_task_wait_any() {
        let mut app = test_app();
        static TASK_FINISHED: AtomicBool = AtomicBool::new(false);

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    wait::any().with(actions![
                        once::run(|| {}),
                        once::run(|| {}),
                        side_effect::tokio::spawn(|_| async move {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            TASK_FINISHED.store(true, Ordering::Relaxed);
                        })
                            .then(once::event::app_exit_success())
                    ])
                        // keep running while the test is running
                        .then(delay::time().with(Duration::from_secs(1000)))
                }).await;
            }));
        });
        let mut er = exit_reader();
        app.update();
        std::thread::sleep(Duration::from_millis(200));
        app.assert_event_not_comes(&mut er);
        assert!(!TASK_FINISHED.load(Ordering::Relaxed));
    }
}