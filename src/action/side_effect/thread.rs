//! Convert the thread operations into [`Action`](crate::prelude::Action).
//!
//! actions
//!
//! - [`side_effect::thread::spawn`](crate::prelude::side_effect::thread::spawn)

use crate::prelude::side_effect::Functor;
use crate::prelude::{ActionSeed, CancellationHandlers, RunnerIs};
use crate::runner::{Output, Runner};
use alloc::sync::Arc;
use bevy::platform::sync::Mutex;
use bevy::prelude::*;

/// Spawns a new os thread, and then wait for its output.
///
/// The thread is started when [`Runner`] is executed for the first time.
///
/// Note that thead created from this function will continue to run even if [`Reactor`](crate::prelude::Reactor) is canceled.
///
/// # Examples
///
/// ```no_run
///
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, {
///         once::run(||{
///             2
///         })
///             .pipe(side_effect::thread::spawn(|num: usize|{
///                 num + 3
///             }))
///             .pipe(once::run(|In(num): In<usize>|{
///                 assert_eq!(num, 5);
///             }))
///     }).await;
/// });
/// ```
pub fn spawn<I, O, M>(f: impl Functor<I, O, M> + Send + Sync + 'static) -> ActionSeed<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    ActionSeed::new(|input, output: Output<O>| ThreadRunner {
        arc_output: Arc::new(Mutex::new(None)),
        args: Some(f.functor(input)),
        output,
        handle: None,
    })
}

struct ThreadRunner<O, F> {
    arc_output: Arc<Mutex<Option<O>>>,
    args: Option<F>,
    output: Output<O>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl<O, F> Runner for ThreadRunner<O, F>
where
    O: Send + 'static,
    F: FnOnce() -> O + Send + 'static,
{
    fn run(&mut self, _: &mut World, _: &mut CancellationHandlers) -> RunnerIs {
        if let Some(f) = self.args.take() {
            let arc_out = self.arc_output.clone();
            self.handle.replace(std::thread::spawn(move || {
                arc_out.lock().unwrap().replace(f());
            }));
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
    use bevy::platform::thread;
    use bevy::prelude::*;
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn thread_calc_2() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    side_effect::thread::spawn(|_| 1 + 1).pipe(once::run(
                        |In(num): In<usize>, mut count: ResMut<Count>| {
                            count.0 = num;
                        },
                    ))
                })
                .await;
            }));
        });
        app.update();
        thread::sleep(core::time::Duration::from_millis(10));
        app.update();
        app.assert_resource_eq(Count(2));
    }

    #[test]
    fn spawn_thread_without_input() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    side_effect::thread::spawn(|| 1 + 1).pipe(once::run(
                        |In(num): In<usize>, mut count: ResMut<Count>| {
                            count.0 = num;
                        },
                    ))
                })
                .await;
            }));
        });
        app.update();
        thread::sleep(core::time::Duration::from_millis(10));
        app.update();
        app.assert_resource_eq(Count(2));
    }
}
