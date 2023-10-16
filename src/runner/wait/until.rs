use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Commands, Entity, Event, EventReader, In, IntoSystem, IntoSystemConfigs, Query, Schedules};

use crate::async_schedules::TaskSender;
use crate::runner::{AsyncSchedule, AsyncScheduleCommand, IntoAsyncScheduleCommand, schedule_initialize, task_running};
use crate::runner::config::AsyncSystemConfig;


/// Runs the system every frame until it returns true.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, wait::until(move_up)).await;
///     });
/// }
///
/// fn move_up(mut transform: Query<&mut Transform>) -> bool{
///     let mut transform = transform.single_mut();
///     transform.translation.y += 1.;
///     50. <= transform.translation.y
/// }
/// ```
#[inline(always)]
pub const fn until<Marker, Sys>(system: Sys) -> impl IntoAsyncScheduleCommand<()>
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), bool, Marker> + Send + Sync + 'static
{
    Until(AsyncSystemConfig::new(system))
}



/// Wait until an event is received.
///
/// Unlike [`wait::output_event`](wait::output_event), there is no return value,
/// but `E` does not need to derive [`Clone`].
#[inline(always)]
pub fn until_event<E: Event>() -> impl IntoAsyncScheduleCommand<()> {
    until(|er: EventReader<E>| { !er.is_empty() })
}


struct Until<Marker, Sys>(AsyncSystemConfig<bool, Marker, Sys>);


impl<Marker, Sys> IntoAsyncScheduleCommand<()> for Until<Marker, Sys>
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), bool, Marker> + Send + Sync + 'static
{
    #[inline]
    fn into_schedule_command(self, sender: TaskSender<()>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(Scheduler {
            sender,
            config: self.0,
            schedule_label,
        })
    }
}


struct Scheduler<Marker, Sys, Label> {
    sender: TaskSender<()>,
    config: AsyncSystemConfig<bool, Marker, Sys>,
    schedule_label: Label,
}


impl<Marker, Sys, Label> AsyncSchedule for Scheduler<Marker, Sys, Label>
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), bool, Marker> + Send + Sync + 'static,
        Label: ScheduleLabel + Clone
{
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
        let schedule = schedule_initialize(schedules, &self.schedule_label);
        entity_commands.insert(self.sender);
        let entity = entity_commands.id();

        schedule.add_systems(self
            .config
            .system
            .pipe(move |In(finished): In<bool>, mut commands: Commands, mut senders: Query<(Entity, &mut TaskSender<()>)>| {
                if !finished {
                    return;
                }
                let Ok((entity, mut sender)) = senders.get_mut(entity) else { return; };
                let _ = sender.try_send(());
                sender.close_channel();
                commands.entity(entity).remove::<TaskSender<()>>();
            })
            .run_if(task_running::<()>(entity))
        );
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::core::FrameCount;
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::{Commands, Res};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::{once, wait};
    use crate::test_util::{FirstEvent, is_first_event_already_coming, new_app};

    #[test]
    fn until() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, wait::until(|frame: Res<FrameCount>| {
                    frame.0 == 2
                })).await;
                schedules.add_system(Update, once::send(FirstEvent)).await;
            });
        });

        app.update();
        app.update();
        app.update();

        // send event
        app.update();

        assert!(is_first_event_already_coming(&mut app, &mut ManualEventReader::default()));
    }

    #[test]
    fn never_again() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, wait::until(|frame: Res<FrameCount>| {
                    if 2 <= frame.0 {
                        panic!("must not be called");
                    }
                    frame.0 == 1
                })).await;
                schedules.add_system(Update, wait::until(|| false)).await;
            });
        });

        for _ in 0..100 {
            app.update();
        }
    }
}