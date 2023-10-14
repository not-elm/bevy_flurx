use bevy::app::{App, Startup};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Event, Events, EventWriter, NextState, OnTransition, States};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::ext::state_schedule_label::AddBevTaskSchedule;
use bevy_async_system::runner::non_send::once::Once;

#[derive(Event)]
struct FinishEvent;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash, States)]
enum TestState {
    #[default]
    Fist,
    Second,
}

#[test]
fn state_on_transition() {
    let mut app = App::new();
    app.add_state::<TestState>();
    app.add_event::<FinishEvent>();

    app.add_plugins((
        TaskPoolPlugin::default(),
        BevTaskPlugin
    ));
    let er = ManualEventReader::<FinishEvent>::default();
    app.add_systems(Startup, setup);
    app.register_task_schedule(OnTransition {
        from: TestState::Fist,
        to: TestState::Second,
    });

    app.update();
    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(er.is_empty(events));

    {
        let mut state = app.world.resource_mut::<NextState<TestState>>();
        state.0.replace(TestState::Second);
    }
    app.update();

    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(!er.is_empty(events));
}


fn setup(
    mut commands: Commands
) {
    commands.spawn_async(|task| async move {
        task.spawn(OnTransition { from: TestState::Fist, to: TestState::Second }, Once::run(send_finish_event)).await;
    });
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}