use bevy::app::{App, Startup, Update};
use bevy::core::TaskPoolPlugin;
use bevy::prelude::{Commands, Component, NonSendMut, Query, Transform, TransformBundle, With};

use bevy_async_system::AsyncSystemPlugin;
use bevy_async_system::task::AsyncSystemManager;

#[test]
fn once() {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AsyncSystemPlugin
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
    mut task_manager: NonSendMut<AsyncSystemManager>,
) {
    commands.spawn((
        Movable,
        TransformBundle::default()
    ));

    task_manager.spawn_async(|mut scheduler| async move {
        scheduler.once(Update, move_transform).await;
    });
}


fn move_transform(mut moves: Query<&mut Transform, With<Movable>>) {
    moves.single_mut().translation.x += 3.;
}