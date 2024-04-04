//!

#![allow(dead_code)]

use std::time::Duration;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_flurx::{sequence, wait_all};
use bevy_flurx::action::once::switch::switch_on;
use bevy_flurx::prelude::*;
use bevy_flurx::prelude::switch::Switch;

struct RotateCamera;

struct MoveUp;

#[derive(Component)]
struct Block;

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct Progress {
    timer: Timer,
    count: usize,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin,
            WorldInspectorPlugin::new()
        ))
        .add_systems(Startup, (
            setup_camera,
            setup_reactor,
            setup_progress
        ))
        .insert_resource(Progress {
            timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
            count: 0,
        })
        .add_systems(Update, rotate_camera.run_if(switch_on::<RotateCamera>))
        .add_systems(Update, (
            update_progress,
            update_progress_ui
        ).chain())
        .run();
}

fn setup_camera(mut cmd: Commands) {
    cmd.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 1_000_000_000.,
            ..default()
        },
        transform: Transform::from_xyz(3.0, 16., 8.),
        ..default()
    });

    cmd.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 0.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera
    ));
}

fn setup_progress(
    mut commands: Commands
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            display: Display::Flex,
            justify_items: JustifyItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle {
            style: Style {
                margin: UiRect::top(Val::Px(16.)),
                ..default()
            },
            text: Text::from_section("0%", TextStyle {
                font_size: 30.,
                ..default()
            }),

            ..default()
        });
    });
}

fn update_progress(
    mut progress: ResMut<Progress>,
    time: Res<Time>,
) {
    if progress.timer.tick(time.delta()).just_finished() {
        progress.count += 1;
    }
}

fn update_progress_ui(
    mut text: Query<&mut Text>,
    progress: Res<Progress>,
) {
    text.single_mut().sections[0].value = format!("{}%", progress.count);
}

fn setup_reactor(world: &mut World) {
    world.schedule_reactor(|task| async move {
        loop {
            task.will(Update, sequence! {
                once::run(spawn_block),
                once::run(camera_move_top),
                delay::time(Duration::from_millis(300)),
                once::run(play_audio),
                scale_up(|v|&mut v.z),
                wait::until(stop_audio),
                delay::time(Duration::from_millis(500)),
                once::run(play_audio)
            }).await;

            task.will(Update, sequence! {
                scale_up(|v|&mut v.x),
                wait::until(stop_audio),
                delay::time(Duration::from_millis(500)),
                once::switch::on::<RotateCamera>(),
            })
                .await;
            task.will(Update, sequence! {
                delay::time(Duration::from_millis(100)),
                once::run(play_audio)
            }).await;
            task.will(Update, sequence! {
                wait_all! {
                    scale_up(|v|&mut v.y),
                    wait::switch::off::<RotateCamera>(),
                    wait::until(stop_audio),
                },
                delay::time(Duration::from_millis(1000)),
                once::run(despawn_block)
            }).await;
        }
    });
}

fn camera_move_top(mut camera: Query<&mut Transform, With<MainCamera>>) {
    *camera.single_mut() = Transform::from_xyz(0.0, 6., 0.).looking_at(Vec3::ZERO, Vec3::Y);
}

fn play_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/scale_up.ogg"),
        settings: PlaybackSettings::ONCE,
    });
}

fn stop_audio(
    mut commands: Commands,
    audio: Query<(Entity, &AudioSink)>,
) -> bool {
    let Ok((entity, audio)) = audio.get_single() else { return false; };
    if audio.empty() {
        commands.entity(entity).despawn();
        true
    } else {
        false
    }
}

fn spawn_block(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(2.5, 2.5, 2.5)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::new(0.01, 0.01, 0.0)),
            ..default()
        },
        Block
    ));
}

fn despawn_block(
    mut commands: Commands,
    block: Query<Entity, With<Block>>,
) {
    commands.entity(block.single()).despawn();
}

fn rotate_camera(
    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut switch: ResMut<Switch<RotateCamera>>,
    time: Res<Time>,
) {
    for mut t in camera.iter_mut() {
        let quat = Quat::from_rotation_z(-time.delta_seconds());
        t.rotate_around(Vec3::ZERO, quat);
        if t.translation.y.abs() < 0.1 {
            t.translation.y = 0.;
            switch.off();
        }
    }
}

fn scale_up(f: impl Fn(&mut Vec3) -> &mut f32 + Send + Sync + 'static) -> impl TaskAction<In=(), Out=()> {
    with((), wait::until(move |mut cubes: Query<&mut Transform, With<Block>>,
                               mut tick: Local<f32>,
                               time: Res<Time>,
    | {
        *tick += time.delta_seconds() * 0.3;
        for mut t in cubes.iter_mut() {
            let v = f(&mut t.scale);
            *v += time.delta_seconds() * 0.3;
            *v = v.lerp(1.0, *v / Duration::from_secs(1).as_secs_f32());
            if (*v - 1.0).abs() < 0.1 {
                *v = 1.;
                return true;
            }
        }
        false
    }))
}




