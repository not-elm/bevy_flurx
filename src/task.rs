use std::future::Future;

use bevy::ecs::schedule::ScheduleLabel;
use futures_polling::FuturePollingExt;

use crate::selector::condition::ReactorSystemConfigs;
use crate::selector::WorldSelector;
use crate::world_ptr::WorldPtr;

pub struct ReactiveTask<'a> {
    pub(crate) inner: flurx::task::ReactiveTask<'a, WorldPtr>,
}


impl<'a> ReactiveTask<'a> {
    pub fn will<Label, In, Out, Marker>(
        &self,
        label: Label,
        configs: impl ReactorSystemConfigs<Marker, In=In, Out=Out>,
    ) -> impl Future<Output=Out> + 'a
        where
            Label: ScheduleLabel + Clone,
            In: Clone + 'static,
            Out: 'static
    {
        let (input, system) = configs.into_configs();
        self.inner.will(WorldSelector::new(label, input, system))
    }

    pub async fn run<Label, In, Out, Marker>(
        &self,
        label: Label,
        configs: impl ReactorSystemConfigs<Marker, In=In, Out=Out> + 'static,
    ) -> impl Future<Output=Out> + 'a
        where
            Label: ScheduleLabel + Clone,
            In: Clone + Unpin + 'static,
            Marker: 'static,
            Out: 'static
    {
        let mut future = self.will(label, configs).polling();
        let _ = future.poll_once().await;
        future
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup, Update};
    use bevy::prelude::World;

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::prelude::wait;
    use crate::selector::condition::once;

    #[test]
    fn run() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    let event_task = task.run(First, wait::event::read::<AppExit>()).await;
                    task.will(Update, once::event::send(AppExit)).await;
                    event_task.await;
                    task.will(Update, once::non_send::insert(AppExit)).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}