use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Event, Events, EventWriter};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::runner::repeat::Repeat;

#[derive(Event, Default, Clone)]
struct RepeatEvent;


#[test]
fn send_event() {
    let mut app = App::new();
    app.add_event::<RepeatEvent>();

    app.add_plugins((
        TaskPoolPlugin::default(),
        BevTaskPlugin
    ));
    let mut er = ManualEventReader::<RepeatEvent>::default();
    app.add_systems(Startup, setup);

    // 1
    app.update();
    let events = app.world.resource::<Events<RepeatEvent>>();
    assert!(!er.is_empty(events));
    er.clear(events);

    // 2
    app.update();
    let events = app.world.resource::<Events<RepeatEvent>>();
    assert!(!er.is_empty(events));
    er.clear(events);

    // 3 repeat end
    app.update();
    let events = app.world.resource::<Events<RepeatEvent>>();
    assert!(!er.is_empty(events));
    er.clear(events);

    app.update();
    let events = app.world.resource::<Events<RepeatEvent>>();
    assert!(er.is_empty(events));
}


fn setup(
    mut commands: Commands
) {
    commands.spawn_async(|task| async move {
        task.spawn(Update, Repeat::times(3, |mut ew: EventWriter<RepeatEvent>| {
            ew.send(RepeatEvent);
        })).await;
    });
}

