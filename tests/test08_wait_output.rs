use bevy::app::{App, PreUpdate, Startup, Update};
use bevy::core::{FrameCount, FrameCountPlugin, TaskPoolPlugin};
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Event, Events, EventWriter, Res};

use bevtask::BevTaskPlugin;
use bevtask::ext::AsyncPool;

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
        task.wait_output(PreUpdate, count).await;
        task.once(Update, send_finish_event).await;
    });
}


fn count(count: Res<FrameCount>) -> Option<()> {
    if 1 <= count.0  {
        Some(())
    } else {
        None
    }
}

fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}