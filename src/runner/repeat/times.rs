use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{IntoSystem, IntoSystemConfigs, Local, Query, Schedules};

use crate::async_schedules::TaskSender;
use crate::prelude::{AsyncSchedule, AsyncScheduleCommand, IntoAsyncScheduleCommand};
use crate::runner::{schedule_initialize, task_running};
use crate::runner::config::AsyncSystemConfig;

pub(crate) struct Times<Marker, Sys> {
    repeat_num: usize,
    config: AsyncSystemConfig<(), Marker, Sys>,
}


impl<Marker, Sys> Times<Marker, Sys>
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), (), Marker> + Send + Sync + 'static
{
    pub fn create(repeat_num: usize, system: Sys) -> impl IntoAsyncScheduleCommand {
        Self {
            repeat_num,
            config: AsyncSystemConfig::new(system),
        }
    }
}


impl<Marker, Sys> IntoAsyncScheduleCommand for Times<Marker, Sys>
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), (), Marker> + Send + Sync + 'static
{
    #[inline]
    fn into_schedule_command(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Scheduler {
            sender,
            repeat_num: self.repeat_num,
            config: self.config,
            schedule_label,
        })
    }
}


struct Scheduler<Marker, Sys, Label> {
    sender: TaskSender<()>,
    repeat_num: usize,
    schedule_label: Label,
    config: AsyncSystemConfig<(), Marker, Sys>,
}


impl<Marker, Sys, Label: ScheduleLabel + Clone> AsyncSchedule for Scheduler<Marker, Sys, Label>
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), (), Marker> + Send + Sync + 'static
{
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
        let schedule = schedule_initialize(schedules, &self.schedule_label);
        entity_commands.insert(self.sender);
        let entity = entity_commands.id();
        let request_repeat_num = self.repeat_num;
        schedule.add_systems((
            self.config.system,
            move |mut repeat_num: Local<usize>, mut senders: Query<&mut TaskSender<()>>| {
                *repeat_num += 1;
                if request_repeat_num <= *repeat_num {
                    let Ok(mut sender) = senders.get_mut(entity) else { return; };
                    let _ = sender.try_send(());
                    sender.close_channel();
                }
            }
        )
            .chain()
            .run_if(task_running::<()>(entity)));
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::{Commands, ResMut, Resource};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::repeat;
    use crate::test_util::new_app;

    #[test]
    fn repeat_5times() {
        let mut app = new_app();
        app.insert_resource(Count(0));
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, repeat::times(5, count_up)).await;
            });
        });

        app.update();
        assert_eq!(app.world.resource::<Count>().0, 1);
        app.update();
        assert_eq!(app.world.resource::<Count>().0, 2);
        app.update();
        assert_eq!(app.world.resource::<Count>().0, 3);
        app.update();
        assert_eq!(app.world.resource::<Count>().0, 4);
        app.update();
        assert_eq!(app.world.resource::<Count>().0, 5);
        for _ in 0..100 {
            app.update();
            assert_eq!(app.world.resource::<Count>().0, 5);
        }
    }


    fn count_up(mut count: ResMut<Count>) {
        count.0 += 1;
    }

    #[derive(Resource)]
    struct Count(usize);
}