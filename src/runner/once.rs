use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{Commands, Event, EventWriter, In, IntoSystem, IntoSystemConfigs, NextState, Query, ResMut, Resource, Schedules, States};

use crate::async_commands::TaskSender;
use crate::prelude::{AsyncSchedule, AsyncScheduleCommand, IntoAsyncScheduleCommand};
use crate::runner::{schedule_initialize, task_running};
use crate::runner::config::AsyncSystemConfig;




/// Run the system only once.
///
/// The system can use `Output`.
/// If any output is returned, it becomes the task's return value.
///
/// ## Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules| async move{
///         schedules.add_system(Update, once::run(without_output)).await;
///         let count: u32 = schedules.add_system(Update, once::run(with_output)).await;
///         assert_eq!(count, 10);
///     });
/// }
///
/// #[derive(Resource)]
/// struct Count(u32);
///
/// fn without_output(mut commands: Commands){
///     commands.insert_resource(Count(10));
/// }
///
///
/// fn with_output(count: Res<Count>) -> u32{
///     count.0
/// }
///
/// ```
///
#[inline(always)]
pub fn run<Out, Marker, Sys>(system: Sys) -> impl IntoAsyncScheduleCommand<Out>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), Out, Marker> + Send + Sync + 'static
{
    OnceOnMain(AsyncSystemConfig::<Out, Marker, Sys>::new(system))
}




/// Set the next state.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, States)]
/// enum ExampleState{
///     #[default]
///     First,
///     Second,
/// }
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|scheduler|async move{
///         scheduler.add_system(Update, once::set_state(ExampleState::Second)).await;
///     });
/// }
/// ```
///
#[inline]
pub fn set_state<S: States + Copy>(to: S) -> impl IntoAsyncScheduleCommand {
    run(move |mut state: ResMut<NextState<S>>| {
        state.set(to);
    })
}




/// Send the event.
///
/// The event to be send must derive [`Clone`] in addition to [`Event`](bevy::prelude::Event).
///
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Event, Clone)]
/// struct ExampleEvent;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, once::send(ExampleEvent)).await;
///     });
/// }
/// ```
#[inline]
pub fn send<E: Event + Clone>(event: E) -> impl IntoAsyncScheduleCommand {
    run(move |mut ew: EventWriter<E>| {
        ew.send(event.clone());
    })
}



/// Insert a [`Resource`](bevy::prelude::Resource).
///
/// The resource is cloned inside the function.
///
/// If the resource derives [`Default`], we recommend using [`once::init_resource`](once::init_resource) instead.
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Resource, Clone)]
/// struct ExampleResource;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, once::insert_resource(ExampleResource)).await;
///     });
/// }
/// ```
#[inline]
pub fn insert_resource<R: Resource + Clone>(resource: R) -> impl IntoAsyncScheduleCommand {
    run(move |mut commands: Commands| {
        commands.insert_resource(resource.clone());
    })
}




/// Initialize a [`Resource`](bevy::prelude::Resource).
///
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Resource, Default)]
/// struct ExampleResource;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         schedules.add_system(Update, once::init_resource::<ExampleResource>()).await;
///     });
/// }
/// ```
#[inline]
pub fn init_resource<R: Resource + Default>() -> impl IntoAsyncScheduleCommand {
    run(|mut commands: Commands| {
        commands.init_resource::<R>();
    })
}


struct OnceOnMain<Out, Marker, Sys>(AsyncSystemConfig<Out, Marker, Sys>);


impl<Out, Marker, Sys> IntoAsyncScheduleCommand<Out> for OnceOnMain<Out, Marker, Sys>
    where
        Out: Send + Sync + 'static,
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), Out, Marker> + Send + Sync + 'static
{
    fn into_schedule_command(self, sender: TaskSender<Out>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand {
        AsyncScheduleCommand::new(OnceRunner {
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


impl<Out, Marker, Sys, Label> AsyncSchedule for OnceRunner<Out, Marker, Sys, Label>
    where
        Out: Send + Sync + 'static,
        Sys: IntoSystem<(), Out, Marker> + Send + Sync,
        Marker: Send + Sync + 'static,
        Label: ScheduleLabel + Clone
{
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules) {
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
    use bevy::prelude::{Commands, Res, Resource};

    use crate::ext::spawn_async_system::SpawnAsyncSystem;
    use crate::runner::once;
    use crate::test_util::{FirstEvent, is_first_event_already_coming, is_second_event_already_coming, new_app, SecondEvent, test_state_finished, TestState};

    #[test]
    fn set_state() {
        let mut app = new_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, once::set_state(TestState::Finished)).await;
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
            commands.spawn_async(|schedules| async move {
                schedules.add_system(Update, once::send(FirstEvent)).await;
                schedules.add_system(Update, once::send(SecondEvent)).await;
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


    #[test]
    fn output() {
        let mut app = new_app();
        app.add_systems(Startup, setup);

        app.update();
        app.update();
    }


    fn setup(mut commands: Commands){
        commands.spawn_async(|schedules| async move{
            schedules.add_system(Update, once::run(without_output)).await;
            let count: u32 = schedules.add_system(Update, once::run(with_output)).await;
            assert_eq!(count, 10);
        });
    }

    fn without_output(mut commands: Commands){
        commands.insert_resource(Count(10));
    }


    fn with_output(count: Res<Count>) -> u32{
        count.0
    }
    #[derive(Resource)]
    struct Count(u32);
}
