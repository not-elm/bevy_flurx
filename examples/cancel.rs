//! This example shows how to cancel [`Reactor`] processing.
//!
//! When you press [`KeyCode::Escape`], the box stops rotating.
//!
//! [`Reactor`]: bevy_flurx::prelude::Reactor

use bevy::DefaultPlugins;
use bevy::prelude::*;

use bevy_flurx::prelude::*;

#[derive(Component)]
struct RotateBox;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, (
            setup_camera_and_box,
            spawn_reactor
        ))
        .add_systems(Update, cancel)
        .run();
}

fn setup_camera_and_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1., 1., 1.)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            ..default()
        },
        RotateBox
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 6.),
        ..default()
    });
}

fn spawn_reactor(
    mut commands: Commands
) {
    commands.spawn(Reactor::schedule(|task| async move {
        task.will(Update, wait::until(rotate_shape)).await;
    }));
}

fn rotate_shape(
    mut shape: Query<&mut Transform, With<RotateBox>>,
    time: Res<Time>,
) -> bool {
    for mut t in shape.iter_mut() {
        t.rotate_y(time.delta_seconds());
    }
    false
}

fn cancel(
    mut commands: Commands,
    reactor: Query<Entity, With<Reactor>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if let Ok(entity) = reactor.get_single() {
            info!("reactor has been cancelled");
            commands.entity(entity).remove::<Reactor>();
        }
    }
}