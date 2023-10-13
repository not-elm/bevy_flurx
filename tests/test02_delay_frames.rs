use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Event, Events, EventWriter};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::AsyncCommands;
use bevy_async_system::runner::delay::Delay;
use bevy_async_system::runner::once::Once;

#[derive(Event)]
struct FinishEvent;

#[test]
fn delay_frames() {
    let mut app = App::new();
    app.add_event::<FinishEvent>();
    app.add_plugins((
        TaskPoolPlugin::default(),
        BevTaskPlugin
    ));

    app.add_systems(Startup, setup);
    app.update();
    app.update();
    app.update();
    app.update();

    let er = ManualEventReader::<FinishEvent>::default();
    let events = app.world.resource::<Events<FinishEvent>>();
    assert!(!er.is_empty(events));
}


fn setup(
    mut commands: Commands
) {
    commands.spawn_async(|task| async move {
        task.spawn(Update, Delay::Frames(3)).await;
        task.spawn(Update, Once::run(send_finish_event)).await;
    });
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}