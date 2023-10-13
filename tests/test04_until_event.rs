use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Component, Event, Events, EventWriter};

use bevtask::BevTaskPlugin;
use bevtask::ext::AsyncPool;

#[derive(Event)]
struct FinishEvent;

#[derive(Event, Clone)]
struct WaitEvent;

#[test]
fn until_come_event() {
    let mut app = App::new();
    app.add_event::<FinishEvent>();
    app.add_event::<WaitEvent>();
    app.add_plugins((
        TaskPoolPlugin::default(),
        BevTaskPlugin
    ));
    let er = ManualEventReader::<FinishEvent>::default();
    app.add_systems(Startup, setup);

    app.update();
    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(er.is_empty(events));

    app.world.send_event(WaitEvent);
    app.update();
    app.update();

    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(!er.is_empty(events));
}


#[derive(Component)]
struct Movable;

fn setup(
    mut commands: Commands
) {
    commands.spawn_async(|task| async move {
        task.wait_event::<WaitEvent>(Update).await;
        task.run_once(Update, send_finish_event).await;
    });
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}