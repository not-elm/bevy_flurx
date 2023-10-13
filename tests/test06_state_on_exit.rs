use bevy::app::{App, Startup};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Event, Events, EventWriter, NextState, OnExit, States};

use bevtask::BevTaskPlugin;
use bevtask::ext::AsyncPool;
use bevtask::ext::state_schedule_label::AddBevTaskStateScheduleLabel;
use bevtask::runner::once::Once;

#[derive(Event)]
struct FinishEvent;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash, States)]
enum TestState {
    #[default]
    Fist,
    Second,
}

#[test]
fn state_on_exit() {
    let mut app = App::new();
    app.add_state::<TestState>();
    app.add_event::<FinishEvent>();

    app.add_plugins((
        TaskPoolPlugin::default(),
        BevTaskPlugin
    ));
    let er = ManualEventReader::<FinishEvent>::default();
    app.add_systems(Startup, setup);
    app.register_task_schedule_on_exit(TestState::Fist);

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
        task.spawn(OnExit(TestState::Fist), Once::run(send_finish_event)).await;
    });
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}