use std::time::Duration;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Component, IntoSystemConfigs, Query, Res, Schedules, TimerMode};
use bevy::time::{Time, Timer};

use crate::async_commands::TaskSender;
use crate::prelude::AsyncScheduleCommand;
use crate::runner::{AsyncSchedule, IntoAsyncScheduleCommand, schedule_initialize, task_running};

pub(crate) struct DelayTime(pub Duration);


impl IntoAsyncScheduleCommand for DelayTime {
    fn into_schedule_command(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Executor {
            schedule_label,
            sender,
            timer: Timer::new(self.0, TimerMode::Once),
        })
    }
}


#[derive(Component)]
struct LocalTimer(Timer);


struct Executor<Label> {
    sender: TaskSender<()>,
    timer: Timer,
    schedule_label: Label,
}


impl<Label: ScheduleLabel + Clone> AsyncSchedule for Executor<Label> {
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
        let schedule = schedule_initialize(schedules, &self.schedule_label);
        entity_commands.insert((
            self.sender,
            LocalTimer(self.timer)
        ));
        let entity = entity_commands.id();

        schedule.add_systems((move |time: Res<Time>, mut query: Query<(&mut TaskSender<()>, &mut LocalTimer)>| {
            let Ok((mut sender, mut timer)) = query.get_mut(entity) else { return; };
            if timer.0.tick(time.delta()).just_finished() {
                let _ = sender.try_send(());
                sender.close_channel();
            }
        }).run_if(task_running::<()>(entity)));
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::app::{Startup, Update};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::Commands;

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::{delay, once};
    use crate::test_util::{FirstEvent, is_first_event_already_coming, new_app};

    #[test]
    fn delay_time() {
        let mut app = new_app();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, delay::timer(Duration::ZERO)).await;
                schedules.add_system(Update, once::send(FirstEvent)).await;
            });
        });


        // tick
        app.update();
        // send event
        app.update();

        assert!(is_first_event_already_coming(&mut app, &mut ManualEventReader::default()));
    }
}