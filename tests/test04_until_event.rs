use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Component, Event, Events, EventWriter, NonSendMut};

use bevtask::AsyncSystemPlugin;
use bevtask::task::BevTask;

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
        AsyncSystemPlugin
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
    mut task: NonSendMut<BevTask>,
) {
    task.spawn_async(|cmd| async move {
        cmd.until_come_event::<WaitEvent>(Update).await;
        cmd.once(Update, send_finish_event).await;
    });
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}