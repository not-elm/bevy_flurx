use crate::prelude::{ActionSeed, CancellationToken, Runner, RunnerStatus};
use bevy::prelude::World;
use crate::runner::Output;

/// Creates a no-op action.
/// 
/// This is also the [Default] action for [Action] and [`ActionSeed`].
/// 
/// ## Examples
/// 
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::no_op()).await;
/// });
/// ```
#[inline]
pub fn no_op() -> ActionSeed {
    ActionSeed::new(|_, output| NoOpRunner(output))
}

/// Creates a no-op action with input and output types.
///
/// This is also the [Default] action for [Action] and [`ActionSeed`].
/// 
/// ## Examples
/// 
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::no_op_with_generics::<(), ()>()).await;
/// });
#[inline]
pub fn no_op_with_generics<I, O>() -> ActionSeed<I, O>
    where 
        I: 'static,
        O: Default + 'static,
{
    ActionSeed::new(|_, output| NoOpRunner(output))
}

struct NoOpRunner<O>(Output<O>);

impl<O: Default> Runner for NoOpRunner<O> {
    fn run(&mut self, _: &mut World, _: &mut CancellationToken) -> RunnerStatus {
        self.0.set(O::default());
        RunnerStatus::Ready
    }
}


#[cfg(test)]
mod tests {
    use crate::action::once;
    use crate::prelude::Then;
    use crate::reactor::Reactor;
    use crate::tests::{came_event, test_app};
    use bevy::app::{AppExit, Startup, Update};
    use bevy::prelude::Commands;

    #[test]
    fn test_no_op() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(Update, {
                    once::no_op_with_generics::<(), ()>()
                        .then(once::event::app_exit_success())
                }).await;
            }));
        });

        app.update();
        assert!(came_event::<AppExit>(&mut app));
    }
}