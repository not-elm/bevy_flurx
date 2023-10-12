use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Component, Event, Events, EventWriter, NonSendMut, Query, Transform, TransformBundle, With};
use futures::future::join;

use bevtask::AsyncSystemPlugin;
use bevtask::task::BevTask;

#[derive(Event)]
struct FinishEvent;
/// Wait move up and right.
#[test]
fn multiple_wait() {
    let mut app = App::new();
    app.add_event::<FinishEvent>();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AsyncSystemPlugin
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


#[derive(Component)]
struct Movable;

fn setup(
    mut commands: Commands,
    mut task: NonSendMut<BevTask>,
) {
    commands.spawn((
        Movable,
        TransformBundle::default()
    ));

    task.spawn_async(|cmd| async move {
        let t1 = cmd.until(Update, move_right);
        let t2 = cmd.until(Update, move_up);

        join(t1, t2).await;
        cmd.once(Update, send_finish_event).await;
    });
}


fn move_right(mut moves: Query<&mut Transform, With<Movable>>) -> bool {
    let mut transform = moves.single_mut();
    transform.translation.x += 1.;
    transform.translation.x < 3.
}


fn move_up(mut moves: Query<&mut Transform, With<Movable>>) -> bool {
    let mut transform = moves.single_mut();
    transform.translation.y += 1.;
    transform.translation.y < 2.
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}