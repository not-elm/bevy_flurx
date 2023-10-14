use bevy::app::{App, PreUpdate, Startup, Update};
use bevy::core::{FrameCount, FrameCountPlugin, TaskPoolPlugin};
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Event, Events, EventWriter, Res};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::runner::once::Once;
use bevy_async_system::runner::wait::Wait;

#[derive(Event)]
struct FinishEvent;


#[test]
fn wait_output() {
    let mut app = App::new();
    app.add_event::<FinishEvent>();

    app.add_plugins((
        TaskPoolPlugin::default(),
        FrameCountPlugin,
        BevTaskPlugin
    ));
    let er = ManualEventReader::<FinishEvent>::default();
    app.add_systems(Startup, setup);

    app.update();
    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(er.is_empty(events));

    app.update();
    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(!er.is_empty(events));
}


fn setup(
    mut commands: Commands
) {
    commands.spawn_async(|task| async move {
        let count = task.spawn(PreUpdate, Wait::output(count)).await;
        assert_eq!(count, 1);
        task.spawn(Update, Once::run(send_finish_event)).await;
    });
}


fn count(count: Res<FrameCount>) -> Option<u32> {
    if 1 <= count.0 {
        Some(count.0)
    } else {
        None
    }
}

fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}