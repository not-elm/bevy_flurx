use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Event, EventReader, In, IntoSystem, IntoSystemConfigs, Query, Schedules};

use crate::async_commands::TaskSender;
use crate::prelude::{AsyncSchedule, AsyncScheduleCommand};
use crate::runner::{IntoAsyncScheduleCommand, schedule_initialize, task_running};
use crate::runner::config::AsyncSystemConfig;

#[inline(always)]
pub const fn output<Out, Marker, Sys>(system: Sys) -> impl IntoAsyncScheduleCommand<Out>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), Option<Out>, Marker> + Send + Sync + 'static
{
    WaitOutput(AsyncSystemConfig::new(system))
}


#[inline]
pub fn output_event<E: Event + Clone>() -> impl IntoAsyncScheduleCommand<E> {
    output(|mut er: EventReader<E>| {
        er.iter().next().cloned()
    })
}


struct WaitOutput<Out, Marker, Sys>(AsyncSystemConfig<Option<Out>, Marker, Sys>);


impl<Out, Marker, Sys> IntoAsyncScheduleCommand<Out> for WaitOutput<Out, Marker, Sys>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), Option<Out>, Marker> + Send + Sync + 'static
{
    #[inline]
    fn into_schedule_command(self, sender: TaskSender<Out>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Executor {
            sender,
            config: self.0,
            schedule_label,
        })
    }
}


struct Executor<Out, Marker, Sys, Label> {
    sender: TaskSender<Out>,
    config: AsyncSystemConfig<Option<Out>, Marker, Sys>,
    schedule_label: Label,
}


impl<Out, Marker, Sys, Label> AsyncSchedule for Executor<Out, Marker, Sys, Label>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), Option<Out>, Marker> + Send + Sync + 'static,
        Label: ScheduleLabel + Clone
{
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
        let schedule = schedule_initialize(schedules, &self.schedule_label);
        entity_commands.insert(self.sender);
        let entity = entity_commands.id();

        schedule.add_systems(self
            .config
            .system
            .pipe(move |In(input): In<Option<Out>>, mut senders: Query<&mut TaskSender<Out>>| {
                let Some(input) = input else { return; };
                let Ok(mut sender) = senders.get_mut(entity) else { return; };
                let _ = sender.try_send(input);
                sender.close_channel();
            })
            .run_if(task_running::<Out>(entity))
        );
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::core::FrameCount;
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Commands, Event, Events, Res};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::{once, wait};
    use crate::test_util::new_app;

    #[test]
    fn output() {
        let mut app = new_app();
        app.add_event::<OutputEvent>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                let frame_count = schedules.add_system(Update, wait::output(|frame: Res<FrameCount>| {
                    if frame.0 < 2 {
                        None
                    } else {
                        Some(frame.0)
                    }
                })).await;

                schedules.add_system(Update, once::send(OutputEvent(frame_count))).await;
            });
        });

        app.update();
        app.update();
        app.update();

        // send event
        app.update();

        let mut er = ManualEventReader::<OutputEvent>::default();
        let events = app.world.resource::<Events<OutputEvent>>();
        let frame_count = er.iter(events).next().unwrap();
        assert_eq!(frame_count.0, 2);
    }


    #[derive(Event, Clone)]
    struct OutputEvent(u32);
}