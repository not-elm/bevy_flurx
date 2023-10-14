use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Event, Events};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::runner::non_send::once::Once;

#[derive(Event, Default, Clone)]
struct FinishEvent;


#[test]
fn send_event() {
    let mut app = App::new();
    app.add_event::<FinishEvent>();

    app.add_plugins((
        TaskPoolPlugin::default(),
        BevTaskPlugin
    ));
    let er = ManualEventReader::<FinishEvent>::default();
    app.add_systems(Startup, setup);

    app.update();
    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(!er.is_empty(events));
}


fn setup(
    mut commands: Commands
) {
    commands.spawn_async(|task| async move {
        task.spawn(Update, Once::send(FinishEvent)).await;
    });
}

