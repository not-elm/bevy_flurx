//! Convert the thread operations into [`Action`](crate::prelude::Action).
//!
//! actions
//!
//! - [`effect::thread::spawn`](crate::prelude::effect::thread::spawn)

use std::sync::{Arc, Mutex};

use bevy::prelude::World;

use crate::prelude::{ActionSeed, CancellationToken, RunnerStatus};
use crate::runner::{Output, Runner};


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
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, {
///         once::run(||{
///             2
///         })
///             .pipe(effect::thread::spawn(|num: usize|{
///                 num + 3
///             }))
///             .pipe(once::run(|In(num): In<usize>|{
///                 assert_eq!(num, 5);
///             }))
///     }).await;
/// });
/// ```
pub fn spawn<I, O>(f: impl FnOnce(I) -> O + Send + Sync + 'static) -> ActionSeed<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    ActionSeed::new(|input, output: Output<O>| {
        ThreadRunner {
            arc_output: Arc::new(Mutex::new(None)),
            args: Some((input, f)),
            output,
            handle: None,
        }
    })
}

struct ThreadRunner<I, O, F> {
    arc_output: Arc<Mutex<Option<O>>>,
    args: Option<(I, F)>,
    output: Output<O>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl<I, O, F> Runner for ThreadRunner<I, O, F>
where
    I: Send + 'static,
    O: Send + 'static,
    F: FnOnce(I) -> O + Send + 'static,
{
    fn run(&mut self, _: &mut World, _: &mut CancellationToken) -> RunnerStatus {
        if let Some((input, f)) = self.args.take() {
            let arc_out = self.arc_output.clone();
            self.handle.replace(std::thread::spawn(move || {
                arc_out.lock().unwrap().replace(f(input));
            }));
        }

        if let Some(out) = self.arc_output.lock().unwrap().take() {
            self.output.set(out);
            RunnerStatus::Ready
        } else {
            RunnerStatus::Pending
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::action::{effect, once};
    use crate::prelude::Pipe;
    use crate::tests::test_app;
    use bevy::prelude::{Commands, In, ResMut, Startup, Update};
    use bevy_test_helper::resource::count::Count;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn thread_calc_2() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(Update, {
                    effect::thread::spawn(|_| {
                        1 + 1
                    })
                        .pipe(once::run(|In(num): In<usize>, mut count: ResMut<Count>| {
                            count.0 = num;
                        }))
                }).await;
            }));
        });
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(10));
        app.update();
        app.assert_resource_eq(Count(2));
    }
}