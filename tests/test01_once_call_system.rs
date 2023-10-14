use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::prelude::{Commands, Component, Query, Transform, TransformBundle, With};

use bevy_async_system::BevTaskPlugin;
use bevy_async_system::ext::SpawnAsyncCommands;
use bevy_async_system::runner::once::Once;

#[test]
fn once() {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        BevTaskPlugin
    ));

    app.add_systems(Startup, setup);
    app.update();
    let transform = app.world.query_filtered::<&Transform, With<Movable>>().single(&app.world);
    assert_eq!(transform.translation.x, 3.);
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
        task.spawn(Update, Once::run(move_transform)).await;
    });
}


fn move_transform(mut moves: Query<&mut Transform, With<Movable>>) {
    moves.single_mut().translation.x += 3.;
}