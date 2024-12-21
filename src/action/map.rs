use bevy::prelude::World;
use crate::action::remake::Remake;
use crate::prelude::CancellationHandlers;
use crate::runner::{BoxedRunner, Output, Runner, RunnerIs};

/// Maps an `Action<I1, O1>` to `Action<I1, O2>` or `ActionSeed<I1, O1>` to `ActionSeed<I1, O2>` by
/// applying function.
pub trait Map<I1, O1, O2, ActionOrSeed>: Sized
where
    O2: Send + Sync + 'static,
{
    /// Maps an `Action<I1, O1>` to `Action<I1, O2>` or `ActionSeed<I1, O1>` to `ActionSeed<I1, O2>` by
    /// applying function.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// Reactor::schedule(|task| async move{
    ///     task.will(Update, once::run(|| 3)
    ///         .map(|num| num + 5)
    ///         .pipe(once::run(|In(num): In<usize>|{
    ///             assert_eq!(num, 8);
    ///         }))
    ///     ).await;
    /// });
    /// ```
    fn map(self, f: impl FnOnce(O1) -> O2 + Send + Sync + 'static) -> ActionOrSeed;

    /// Overwrite the output of [`Action`](crate::prelude::Action) or [`ActionSeed`](crate::prelude::ActionSeed).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// Reactor::schedule(|task| async move{
    ///     task.will(Update, once::run(|| 3)
    ///         .overwrite("hello")
    ///         .pipe(once::run(|In(word): In<&'static str>|{
    ///             assert_eq!(word, "hello");
    ///         }))
    ///     ).await;
    /// });
    fn overwrite(self, output: O2) -> ActionOrSeed {
        self.map(move |_| output)
    }
}

impl<I, O, O2, A, Re> Map<I, O, O2, A> for Re
where
    I: 'static,
    O: 'static,
    O2: Send + Sync + 'static,
    Re: Remake<I, O, O2, A> + 'static,
{
    #[inline]
    fn map(self, f: impl FnOnce(O) -> O2 + Send + Sync + 'static) -> A {
        self.remake(|r1, o1, output| MapRunner {
            r1,
            o1,
            output,
            map: Some(f),
        })
    }
}

struct MapRunner<O1, O2, F> {
    r1: BoxedRunner,
    o1: Output<O1>,
    output: Output<O2>,
    map: Option<F>,
}

impl<O1, O2, F> Runner for MapRunner<O1, O2, F>
where
    F: FnOnce(O1) -> O2 + 'static,
{
    fn run(&mut self, world: &mut World, token: &mut CancellationHandlers) -> RunnerIs {
        match self.r1.run(world, token) {
            RunnerIs::Canceled => RunnerIs::Canceled,
            RunnerIs::Running => RunnerIs::Running,
            RunnerIs::Completed => {
                let o = self.o1.take().expect("The output value has not been set!!!");
                let map_fn = self.map.take().expect("Failed to take the map function");
                self.output.set(map_fn(o));
                RunnerIs::Completed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::once;
    use crate::prelude::{Reactor, Map, Pipe};
    use crate::tests::test_app;
    use bevy::app::{Startup, Update};
    use bevy::prelude::{Commands, In};

    #[test]
    fn map_num_to_string() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, once::run(|| 3).map(|num| format!("{num}")))
                    .await;
            }));
        });
    }

    #[test]
    fn overwrite() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    once::run(|| 3)
                        .overwrite(5)
                        .pipe(once::run(|In(num): In<usize>| {
                            assert_eq!(num, 5);
                        })),
                )
                    .await;

                task.will(
                    Update,
                    once::run(|| 3).overwrite("string").pipe(once::run(
                        |In(str): In<&'static str>| {
                            assert_eq!(str, "string");
                        },
                    )),
                )
                    .await;
            }));
        });

        app.update();
        app.update();
    }
}
