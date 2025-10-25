//! Create a task that runs the system until certain conditions are met.

use crate::action::Action;
use crate::core::task::CoreTask;
use crate::selector::WorldSelector;
use crate::world_ptr::WorldPtr;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::Entity;
use core::future::Future;
use futures_polling::FuturePollingExt;

/// Create a task that runs the system until certain conditions are met.
#[derive(Clone)]
pub struct ReactorTask {
    pub(crate) task: CoreTask<WorldPtr>,
    pub(crate) entity: Entity,
}

impl ReactorTask {
    /// Create a new task.
    ///
    /// The argument label indicates which scheduler it will be executed on.
    ///
    /// For `action`, please see [`here`](crate::action).
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use bevy::app::AppExit;
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    /// let mut app = App::new();
    /// app.add_plugins(FlurxPlugin);
    /// app.add_systems(Startup, |mut commands: Commands|{
    ///     commands.spawn(Reactor::schedule(|task| async move{
    ///         let count: u8 = task.will(Update, wait::output(|mut count: Local<u8>|{
    ///             *count += 1;
    ///             (*count == 2).then_some(*count)
    ///         })).await;
    ///         assert_eq!(count, 2);
    ///     }));
    /// });
    /// app.update();
    /// app.update();
    ///```
    #[inline]
    pub fn will<Label, In, Out>(
        &self,
        label: Label,
        action: impl Into<Action<In, Out>> + 'static,
    ) -> impl Future<Output = Out>
    where
        Label: ScheduleLabel,
        In: 'static,
        Out: 'static,
    {
        self.task
            .will(WorldSelector::new(label, self.entity, action.into()))
    }

    /// Create a new initialized task.
    ///
    /// Unlike [`ReactorTask::run`], returns a task that registered a system.
    ///
    /// ```no_run
    /// use bevy::app::AppExit;
    /// use bevy::prelude::*;
    /// use bevy_flurx::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.add_plugins(FlurxPlugin);
    /// app.add_systems(Startup, |mut commands: Commands|{
    ///     commands.spawn(Reactor::schedule(|task|async move{
    ///         let wait_event = task.run(Update, wait::message::comes::<AppExit>()).await;
    ///         task.will(Update, once::message::write().with(AppExit::Success)).await;
    ///         wait_event.await;
    ///     }));
    /// });
    /// app.update();
    /// app.update();
    /// app.update();
    /// ```
    #[inline]
    pub async fn run<Label, In, Out>(
        &self,
        label: Label,
        action: impl Into<Action<In, Out>> + 'static,
    ) -> impl Future<Output = Out>
    where
        Label: ScheduleLabel,
        In: 'static,
        Out: 'static,
    {
        let mut future = self.will(label, action).polling();
        let _ = future.poll_once().await;
        future
    }
}

#[cfg(test)]
mod tests {
    use crate::action::once;
    use crate::prelude::wait;
    use crate::reactor::Reactor;
    use crate::tests::test_app;
    use bevy::app::{AppExit, First, Startup, Update};
    use bevy::prelude::Commands;

    #[test]
    fn run() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                let event_task = task.run(First, wait::message::read::<AppExit>()).await;
                task.will(Update, once::message::write().with(AppExit::Success))
                    .await;
                event_task.await;
                task.will(Update, once::non_send::insert().with(AppExit::Success))
                    .await;
            }));
        });

        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }
}
