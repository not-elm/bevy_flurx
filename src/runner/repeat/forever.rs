use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{IntoSystem, IntoSystemConfigs, Schedules};

use crate::async_schedules::TaskSender;
use crate::prelude::{AsyncScheduleCommand, IntoAsyncScheduleCommand};
use crate::runner::{AsyncSchedule, schedule_initialize, task_running};
use crate::runner::config::AsyncSystemConfig;

pub(crate) struct Forever<Marker, Sys>(pub AsyncSystemConfig<(), Marker, Sys>);


impl<Marker, Sys> IntoAsyncScheduleCommand for Forever<Marker, Sys>
    where
        Sys: IntoSystem<(), (), Marker> + Send + Sync + 'static,
        Marker: Send + Sync + 'static
{
    fn into_schedule_command(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Scheduler {
            sender,
            schedule_label,
            config: self.0,
        })
    }
}


struct Scheduler<Marker, Sys, Label> {
    sender: TaskSender<()>,
    schedule_label: Label,
    config: AsyncSystemConfig<(), Marker, Sys>,
}


impl<Marker, Sys, Label> AsyncSchedule for Scheduler<Marker, Sys, Label>
    where
        Sys: IntoSystem<(), (), Marker> + Send + Sync,
        Marker: Send + Sync + 'static,
        Label: ScheduleLabel + Clone
{
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
        let schedule = schedule_initialize(schedules, &self.schedule_label);
        entity_commands.insert(self.sender);
        let entity = entity_commands.id();
        schedule.add_systems(self.config.system.run_if(task_running::<()>(entity)));
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::prelude::{Commands, ResMut, Resource};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::{delay, repeat};
    use crate::test_util::new_app;

    #[test]
    fn repeat_forever() {
        let mut app = new_app();
        app.insert_resource(Count(0));
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, repeat::forever(count_up)).await;
            });
        });

        for i in 0..100 {
            app.update();
            assert_eq!(app.world.resource::<Count>().0, i + 1);
        }
    }


    #[test]
    fn when_drop_handle_system_also_stop() {
        let mut app = new_app();
        app.insert_resource(Count(0));
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                let handle = schedules.add_system(Update, repeat::forever(count_up));
                schedules.add_system(Update, delay::frames(30)).await;
                drop(handle);
                schedules.add_system(Update, repeat::forever(||{})).await;
            });
        });

        for i in 0..30 {
            app.update();
            assert_eq!(app.world.resource::<Count>().0, i + 1);
        }

        for _ in 0..100 {
            app.update();
            assert_eq!(app.world.resource::<Count>().0, 30);
        }
    }


    fn count_up(mut count: ResMut<Count>) {
        count.0 += 1;
    }

    #[derive(Resource)]
    struct Count(usize);
}