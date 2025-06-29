use crate::action::side_effect::AsyncFunctor;
use crate::prelude::{ActionSeed, CancellationHandlers, Output, RunnerIs};
use crate::runner::Runner;
use bevy::prelude::World;
use bevy::tasks::AsyncComputeTaskPool;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

/// Spawns a future onto the bevy thread pool,
/// and then wait until its completed.
///
/// Unlike [`side_effect::bevy_task::spawn`](crate::prelude::side_effect::bevy_task::spawn_detached),
/// a spawned task is detached and continues to run in the background.
///
/// Note that tasks created from this function will continue to run even if [`Reactor`](crate::prelude::Reactor) is canceled.
///
/// ```no_run
///
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, side_effect::bevy_task::spawn_detached(async move{
///
///     })).await;
/// });
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, {
///         wait::output(|| Some(1))
///             .pipe(side_effect::bevy_task::spawn_detached(|num: usize| async move{
///                 num + 1
///             }))
///             .pipe(once::run(|In(num): In<usize>|{
///                 assert_eq!(num, 2);
///             }))
///     }).await;
/// });
/// ```
pub fn spawn_detached<I, Out, Functor, M>(functor: Functor) -> ActionSeed<I, Out>
where
    I: Send + 'static,
    Functor: AsyncFunctor<I, Out, M> + Send + Sync + 'static,
    Out: Send + 'static,
    M: Send + 'static,
{
    ActionSeed::new(|input, output| BevyDetachedTaskRunner {
        output,
        arc_output: Arc::new(Mutex::new(None)),
        args: Some((input, functor)),
        _m: PhantomData::<M>,
    })
}

struct BevyDetachedTaskRunner<I, O, Functor, M> {
    arc_output: Arc<Mutex<Option<O>>>,
    args: Option<(I, Functor)>,
    output: Output<O>,
    _m: PhantomData<M>,
}

impl<I, O, Functor, M> Runner for BevyDetachedTaskRunner<I, O, Functor, M>
where
    I: Send + 'static,
    O: Send + 'static,
    Functor: AsyncFunctor<I, O, M> + Send + 'static,
    M: Send + 'static,
{
    fn run(&mut self, _: &mut World, _: &mut CancellationHandlers) -> RunnerIs {
        if let Some((input, f)) = self.args.take() {
            let o = self.arc_output.clone();
            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let out = f.functor(input).await;
                    o.lock().unwrap().replace(out);
                })
                .detach();
        }

        if let Some(out) = self.arc_output.try_lock().ok().and_then(|mut o| o.take()) {
            self.output.set(out);
            RunnerIs::Completed
        } else {
            RunnerIs::Running
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::{once, side_effect};
    use crate::prelude::{Pipe, Reactor};
    use crate::tests::test_app;
    use bevy::app::Startup;
    use bevy::prelude::{Commands, Update};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;
    use std::time::Duration;

    #[test]
    fn test_simple_spawn_detached() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    side_effect::bevy_task::spawn_detached(async move { Count(1 + 1) })
                        .pipe(once::res::insert())
                })
                .await;
            }));
        });
        app.update();
        std::thread::sleep(Duration::from_millis(20));
        app.update();
        app.assert_resource_eq(Count(2));
    }
}
