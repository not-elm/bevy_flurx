//!

#![allow(dead_code)]

use std::cmp::min;
use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;

use bevy_flurx::{sequence, wait_all};
use bevy_flurx::prelude::*;

struct RotateCamera;

struct Loading;

#[derive(Component)]
struct Block;

#[derive(Component)]
struct MainCamera;

const DELAY: u64 = 100;

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
        .add_systems(Update, rotate_camera.run_if(switch_turned_on::<RotateCamera>))
        .add_systems(Update, (
            update_progress,
            update_progress_ui
        )
            .run_if(switch_turned_on::<Loading>)
            .chain())
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
        progress.count = min(progress.count + 1, 100);
    }
}

fn update_progress_ui(
    mut text: Query<&mut Text>,
    progress: Res<Progress>,
) {
    text.single_mut().sections.drain(1..);
    text.single_mut().sections[0].value = format!("{}%", progress.count);
}

fn progress_clean_up(
    mut progress: ResMut<Progress>,
    mut text: Query<&mut Text>,
) {
    progress.count = 0;
    text.single_mut().sections[0].value = "Loading completed\n".to_string();
    text.single_mut().sections.push(TextSection::new(
        "To execute it again, press [R]",
        TextStyle {
            font_size: 50.,
            color: Color::BLUE,
            ..default()
        },
    ));
}


fn setup_reactor(world: &mut World) {
    world.schedule_reactor(|task| async move {
        loop {
            task.will(Update, once::switch::on::<Loading>()).await;
            task.will(Update, delay::time(Duration::from_secs(1))).await;
            loop {
                let loading = sequence! {
                    once::run(spawn_block),
                    expand_horizon(),
                    expand_vertical(),
                    delay::time(Duration::from_millis(DELAY)),
                    once::switch::on::<RotateCamera>(),
                    delay::time(Duration::from_millis(DELAY)),
                    once::run(play_audio),
                    wait_all! {
                        scale_up(|v|&mut v.y),
                        wait::switch::off::<RotateCamera>(),
                        wait::until(stop_audio),
                    },
                    shrink_vertical(),
                    shrink_horizon(),
                    delay::time(Duration::from_millis(DELAY)),
                    once::run(despawn_block)
                };

                if task
                    .will(Update, wait::either(loading, wait::until(progress_have_completed)))
                    .await
                    .is_right() {
                    break;
                }
            }
            task.will(Update, {
                once::switch::off::<Loading>()
                    .then(once::run(despawn_block))
                    .then(once::run(progress_clean_up))
                    .then(wait::input::just_pressed(KeyCode::KeyR))
            }).await;
        }
    });
}

fn expand_horizon() -> impl TaskAction< (), ((), ())> {
    delay::time(Duration::from_millis(DELAY))
        .then(once::run(play_audio))
        .then(wait::both(
            wait::until(stop_audio),
            scale_up(|v| &mut v.z),
        ))
}

fn expand_vertical() -> impl TaskAction< (), ((), ())> {
    delay::time(Duration::from_millis(DELAY))
        .then(once::run(play_audio))
        .then(wait::both(
            wait::until(stop_audio),
            scale_up(|v| &mut v.x),
        ))
}

fn shrink_vertical() -> impl TaskAction< (), ()> {
    delay::time(Duration::from_millis(DELAY))
        .then(once::run(play_audio))
        .then(scale_down(0.01, |v| &mut v.z))
        .then(wait::until(stop_audio))
}

fn shrink_horizon() -> impl TaskAction< (), ()> {
    delay::time(Duration::from_millis(DELAY))
        .then(once::run(play_audio))
        .then(wait::both(
            scale_down(0.0, |v| &mut v.y),
            scale_down(0.0, |v| &mut v.x),
        ))
        .then(wait::until(stop_audio))
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

fn progress_have_completed(
    progress: Res<Progress>
) -> bool {
    progress.count == 100
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
    commands.entity(block.single()).despawn_recursive();
}

fn rotate_camera(
    mut camera: Query<&mut Transform, With<Block>>,
    mut switch: ResMut<Switch<RotateCamera>>,
    mut tick: Local<f32>,
    time: Res<Time>,
) {
    *tick += time.delta_seconds();

    for mut t in camera.iter_mut() {
        let end = Quat::from_rotation_z(PI * 0.5);
        t.rotation = Quat::from_rotation_z(0.).lerp(end, *tick);
        if end.abs_diff_eq(t.rotation, 0.01) {
            t.translation.z = 0.;
            *tick = 0.;
            switch.off();
        }
    }
}

fn scale_up(f: impl Fn(&mut Vec3) -> &mut f32 + Send + Sync + 'static) -> impl TaskAction< (), ()> {
    with((), wait::until(move |mut cubes: Query<&mut Transform, With<Block>>,
                               mut tick: Local<f32>,
                               time: Res<Time>,
    | {
        *tick += time.delta_seconds();
        for mut t in cubes.iter_mut() {
            let v = f(&mut t.scale);
            *v = 0.0.lerp(1.0, *tick / 0.3);
            if (*tick - 0.3).abs() < 0.01 {
                *v = 1.;
                return true;
            }
        }
        false
    }))
}

fn scale_down(dist: f32, f: impl Fn(&mut Vec3) -> &mut f32 + Send + Sync + 'static) -> impl TaskAction< (), ()> {
    with((), wait::until(move |mut cubes: Query<&mut Transform, With<Block>>,
                               mut tick: Local<f32>,
                               time: Res<Time>,
    | {
        *tick += time.delta_seconds();
        for mut t in cubes.iter_mut() {
            let v = f(&mut t.scale);
            *v = 1.0.lerp(dist, *tick / 0.3);
            if (*tick - 0.3).abs() < 0.01 {
                *v = dist;
                return true;
            }
        }
        false
    }))
}




