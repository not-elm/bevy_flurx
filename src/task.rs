use bevy::ecs::schedule::ScheduleLabel;

use crate::selector::condition::ReactorSystemConfigs;
use crate::selector::WorldSelector;
use crate::world_ptr::WorldPtr;

pub struct ReactiveTask<'a> {
    pub(crate) inner: flurx::task::ReactiveTask<'a, WorldPtr>,
}


impl<'a> ReactiveTask<'a> {
    pub async fn will<Label, In, Out, Marker>(
        &self,
        label: Label,
        configs: impl ReactorSystemConfigs<Marker, In=In, Out=Out>,
    ) -> Out
        where
            Label: ScheduleLabel + Clone,
            In: Clone + 'static,
            Out: 'static
    {
        let (input, system) = configs.into_configs();
        self.inner.will(WorldSelector::new(label, input, system)).await
    }
}


