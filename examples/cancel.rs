//! This example shows how to cancel [`Reactor`] processing.
//!
//! When you press [`KeyCode::Escape`], the box stops rotating.
//!
//! [`Reactor`]: bevy_flurx::prelude::Reactor

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_flurx::prelude::*;

#[derive(Component)]
struct RotateBox;

#[derive(Component)]
struct Cancel;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FlurxPlugin))
        .add_systems(Startup, (setup_camera_and_box, spawn_reactor))
        .add_systems(Update, cancel)
        .run();
}

fn setup_camera_and_box(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        })),
        Transform::default(),
        RotateBox,
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));
    commands.spawn((Camera3d::default(), Transform::from_xyz(0., 0., 6.)));
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn((
        Reactor::schedule(|task| async move {
            task.will(Update, wait::until(rotate_shape)).await;
        }),
        Cancel,
    ));
}

fn rotate_shape(mut shape: Query<&mut Transform, With<RotateBox>>, time: Res<Time>) -> bool {
    for mut t in shape.iter_mut() {
        t.rotate_y(time.delta_secs());
    }
    false
}

fn cancel(
    mut commands: Commands,
    reactor: Query<Entity, With<Cancel>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if let Ok(entity) = reactor.get_single() {
            info!("reactor has been cancelled");
            commands.entity(entity).despawn();
        }
    }
}
