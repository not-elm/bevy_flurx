use bevy::prelude::World;

use crate::action::effect::AsyncFunctor;
use crate::prelude::{ActionSeed, CancellationToken, Output, Runner};

/// Spawns a future onto the bevy thread pool,
/// and then wait until its completed.
///
/// ```no_run
///
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, effect::bevy_task::spawn(async move{
///
///     })).await;
/// });
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, {
///         wait::output(|| Some(1))
///             .pipe(effect::bevy_task::spawn(|num: usize| async move{
///                 num + 1
///             }))
///             .pipe(once::run(|In(num): In<usize>|{
///                 assert_eq!(num, 2);
///             }))
///     }).await;
/// });
/// ```
pub fn spawn<I, Out, Functor, M>(f: Functor) -> ActionSeed<I, Out>
where
     I: 'static,
    Functor: AsyncFunctor<I, Out, M> + 'static,
    Out: Send + 'static,
    M: Send + 'static,
{
    ActionSeed::new(|input, output| {
        BevyTaskRunner {
            output,
            #[cfg(not(target_arch = "wasm32"))]
            task: bevy::tasks::AsyncComputeTaskPool::get().spawn(f.functor(input)),
            #[cfg(target_arch = "wasm32")]
            task: Box::pin(f.functor(input)),
        }
    })
}

struct BevyTaskRunner<Out> {
    #[cfg(not(target_arch = "wasm32"))]
    task: bevy::tasks::Task<Out>,
    #[cfg(target_arch = "wasm32")]
    task: std::pin::Pin<Box<dyn std::future::Future<Output=Out>>>,
    output: Output<Out>,
}

impl<Out> Runner for BevyTaskRunner<Out>
where
    Out: Send + 'static,
{
    #[allow(clippy::async_yields_async)]
    fn run(&mut self, _: &mut World, _: &CancellationToken) -> bool {
        if let Some(out) = pollster::block_on(futures_lite::future::poll_once(&mut self.task)) {
            self.output.set(out);
            true
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::core::TaskPoolPlugin;
    use bevy::prelude::Commands;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{effect, once};
    use crate::prelude::{Pipe, Reactor};
    use crate::tests::test_app;

    //TODO: It fails about once every two times.
    // Need to check the internal code of `bevy_task` crate.
    #[test]
    fn test_simple_case() {
        for _ in 0..100 {
            let mut app = test_app();
            app.add_plugins(TaskPoolPlugin::default());
            app.add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(Update, {
                        effect::bevy_task::spawn(async move {
                            Count(1 + 1)
                        })
                            .pipe(once::res::insert())
                    }).await;
                }));
            });
            app.update();
            app.assert_resource_eq(Count(2));
        }
    }
}

#[cfg(all(test, feature = "tokio"))]
mod test_tokio {
    use bevy::app::{Startup, Update};
    use bevy::core::TaskPoolPlugin;
    use bevy::prelude::Commands;

    use crate::action::effect;
    use crate::prelude::Reactor;
    use crate::tests::test_app;

    #[test]
    fn not_failed_with_tokio_task() {
        let mut app = test_app();
        app.add_plugins(TaskPoolPlugin::default());
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    effect::bevy_task::spawn(async move {
                        tokio::time::sleep(std::time::Duration::new(1, 0)).await;
                    })
                }).await;
            }));
        });
        app.update();
    }
}