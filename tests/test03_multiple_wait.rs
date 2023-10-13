use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Component, Event, Events, EventWriter, Query, Transform, TransformBundle, With};
use futures::future::join;

use bevtask::BevTaskPlugin;
use bevtask::ext::AsyncPool;
use bevtask::runner::once::Once;
use bevtask::runner::until::Until;

#[derive(Event)]
struct FinishEvent;

/// Wait move up and right.
#[test]
fn multiple_wait() {
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


#[derive(Component)]
struct Movable;

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Movable,
        TransformBundle::default()
    ));

    commands.spawn_async(|task| async move {
        let t1 = task.spawn(Update, Until::run(move_right));
        let t2 = task.spawn(Update, Until::run(move_up));

        join(t1, t2).await;
        task.spawn(Update, Once::run(send_finish_event)).await;
    });
}


fn move_right(mut moves: Query<&mut Transform, With<Movable>>) -> bool {
    let mut transform = moves.single_mut();
    transform.translation.x += 1.;
    3. <= transform.translation.x
}


fn move_up(mut moves: Query<&mut Transform, With<Movable>>) -> bool {
    let mut transform = moves.single_mut();
    transform.translation.y += 1.;
    2. <= transform.translation.y
}


fn send_finish_event(mut ew: EventWriter<FinishEvent>) {
    ew.send(FinishEvent);
}