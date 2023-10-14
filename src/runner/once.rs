use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Event, EventWriter, In, IntoSystem, IntoSystemConfigs, NextState, Query, ResMut, Schedules, States};

use crate::async_commands::TaskSender;
use crate::prelude::{BoxedMainThreadExecutor, IntoMainThreadExecutor, MainThreadExecutable};
use crate::runner::config::AsyncSystemConfig;

use crate::runner::{schedule_initialize, task_running};

struct OnceOnMain<Out, Marker, Sys>(AsyncSystemConfig<Out, Marker, Sys>);


#[inline(always)]
pub fn run<Out, Marker, Sys>(system: Sys) -> impl IntoMainThreadExecutor<Out>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), Out, Marker> + Send + Sync + 'static
{
    OnceOnMain(AsyncSystemConfig::<Out, Marker, Sys>::new(system))
}


#[inline]
pub fn set_state<S: States + Copy>(to: S) -> impl IntoMainThreadExecutor {
    run(move |mut state: ResMut<NextState<S>>| {
        state.set(to);
    })
}


#[inline]
pub fn send<E: Event + Clone>(event: E) -> impl IntoMainThreadExecutor {
    run(move |mut ew: EventWriter<E>| {
        ew.send(event.clone());
    })
}


impl<Out, Marker, Sys> IntoMainThreadExecutor<Out> for OnceOnMain<Out, Marker, Sys>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), Out, Marker> + Send + Sync + 'static
{
    fn into_executor(self, sender: TaskSender<Out>, schedule_label: impl ScheduleLabel + Clone) -> BoxedMainThreadExecutor {
        BoxedMainThreadExecutor::new(OnceRunner {
            config: self.0,
            sender,
            schedule_label,
        })
    }
}


struct OnceRunner<Out, Marker, Sys, Label> {
    config: AsyncSystemConfig<Out, Marker, Sys>,
    sender: TaskSender<Out>,
    schedule_label: Label,
}


impl<Out, Marker, Sys, Label> MainThreadExecutable for OnceRunner<Out, Marker, Sys, Label>
    where
        Out: Send + Sync + 'static,
        Sys: IntoSystem<(), Out, Marker> + Send + Sync,
        Marker: Send + Sync + 'static,
        Label: ScheduleLabel + Clone
{
    fn schedule_initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
        let schedule = schedule_initialize(schedules, &self.schedule_label);
        entity_commands.insert(self.sender);
        let entity = entity_commands.id();
        schedule.add_systems(self
            .config
            .system
            .pipe(move |In(input): In<Out>, mut senders: Query<&mut TaskSender<Out>>| {
                if let Ok(mut sender) = senders.get_mut(entity) {
                    let _ = sender.try_send(input);
                    sender.close_channel();
                }
            })
            .run_if(task_running::<Out>(entity)));
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::{Startup, Update};
    use bevy::ecs::event::ManualEventReader;
    use bevy::prelude::Commands;

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::once;
    use crate::test_util::{FirstEvent, is_first_event_already_coming, is_second_event_already_coming, new_app, SecondEvent, test_state_finished, TestState};

    #[test]
    fn set_state() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|cmd| async move {
                cmd.spawn(Update, once::set_state(TestState::Finished)).await;
            });
        });

        app.update();
        app.update();

        assert!(test_state_finished(&mut app));
    }


    #[test]
    fn send_event() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|cmd| async move {
                cmd.spawn(Update, once::send(FirstEvent)).await;
                cmd.spawn(Update, once::send(SecondEvent)).await;
            });
        });

        let mut er_first = ManualEventReader::default();
        let mut er_second = ManualEventReader::default();

        app.update();

        assert!(is_first_event_already_coming(&mut app, &mut er_first));
        assert!(!is_second_event_already_coming(&mut app, &mut er_second));

        app.update();
        assert!(!is_first_event_already_coming(&mut app, &mut er_first));
        assert!(is_second_event_already_coming(&mut app, &mut er_second));

        app.update();
        assert!(!is_first_event_already_coming(&mut app, &mut er_first));
        assert!(!is_second_event_already_coming(&mut app, &mut er_second));
    }
}
