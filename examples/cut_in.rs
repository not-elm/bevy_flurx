//! This example shows a simple cut-in implementation.
//!
//! Cut-in will start by pressing the R key.
#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_flurx::prelude::*;
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Component)]
struct CutInBackground;

#[derive(Component)]
struct HandsomeFerris;

#[derive(Component)]
struct StartPos(Vec3);

struct MoveSlowly;

struct MoveFast;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FlurxPlugin))
        .add_systems(Startup, (spawn_reactor, spawn_ferris, setup))
        .add_systems(
            Update,
            (
                cut_in::<CutInBackground, 200>.run_if(switch_is_on::<CutInBackground>),
                cut_in_ferris.run_if(switch_is_on::<HandsomeFerris>),
                move_left_down::<25>.run_if(switch_is_on::<MoveSlowly>),
                move_left_down::<10000>.run_if(switch_is_on::<MoveFast>),
            ),
        )
        .run();
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn(Reactor::schedule(|task| async move {
        info!("please press [`R`] key!");
        task.will(Update, {
            wait::input::just_pressed()
                .with(KeyCode::KeyR)
                .then(once::switch::on::<CutInBackground>())
                .then(delay::time().with(Duration::from_millis(100)))
                .then(once::switch::on::<HandsomeFerris>())
                .then(wait::both(
                    wait::switch::off::<CutInBackground>(),
                    wait::switch::off::<HandsomeFerris>(),
                ))
                .then(once::switch::on::<MoveSlowly>())
                .then(delay::time().with(Duration::from_millis(500)))
                .then(once::switch::off::<MoveSlowly>())
                .then(once::switch::on::<MoveFast>())
                .then(delay::time().with(Duration::from_millis(300)))
                .then(once::event::app_exit_success())
        })
        .await;
    }));
}

fn setup(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>) {
    let wh = &window.single().unwrap().resolution;
    let angle = (wh.height() / 2.).atan2(wh.width() / 2.);
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(wh.width() * 2., 300.)),
            color: Color::srgb(0.8, 0.6, 0.1),
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_z(angle)).with_translation(Vec3::new(
            wh.width() * 2.,
            wh.height(),
            0.,
        )),
        CutInBackground,
    ));
    commands.spawn(Camera2d);
}

fn spawn_ferris(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    const HEIGHT: f32 = 200.;
    const WIDTH: f32 = HEIGHT * 1.5;

    let wh = &window.single().unwrap().resolution;
    let pos = Vec3::new(
        wh.width() / 2. + WIDTH / 2.,
        wh.height() / 2. + HEIGHT / 2.,
        1.,
    );
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(WIDTH, HEIGHT)),
            image: asset_server.load("rustacean-flat-gesture.png"),
            ..default()
        },
        Transform::from_translation(pos),
        StartPos(pos),
        HandsomeFerris,
    ));
}

fn cut_in<M, const S: u64>(
    mut target: Query<&mut Transform, With<M>>,
    mut switch: ResMut<Switch<M>>,
    mut tick: Local<f32>,
    time: Res<Time>,
) where
    M: Component,
{
    let mut t = target.single_mut().unwrap();
    let end = Duration::from_millis(S).as_secs_f32();
    *tick = end.min(*tick + time.delta_secs());
    t.translation = t
        .translation
        .lerp(Vec2::ZERO.extend(t.translation.z), *tick / end);
    if (*tick - end).abs() < 0.1 {
        switch.off();
    }
}

fn cut_in_ferris(
    mut ferris: Query<
        (&mut Transform, &StartPos),
        (With<HandsomeFerris>, Without<CutInBackground>),
    >,
    mut switch: ResMut<Switch<HandsomeFerris>>,
    mut tick: Local<f32>,
    time: Res<Time>,
) {
    let Ok((mut t, StartPos(start))) = ferris.single_mut() else {
        return;
    };
    let end = Duration::from_millis(300).as_secs_f32();
    *tick = end.min(*tick + time.delta_secs());
    let d = *tick / end;
    let d = (d * PI / 2.).sin();

    t.translation = *start + (Vec3::Z - *start) * d;
    if (*tick - end).abs() < 0.01 {
        switch.off();
    }
}

fn move_left_down<const SPEED: u16>(
    mut ferris: Query<&mut Transform, (With<HandsomeFerris>, Without<CutInBackground>)>,
    bg: Query<&Transform, (With<CutInBackground>, Without<HandsomeFerris>)>,
    time: Res<Time>,
) {
    let d = time.delta_secs() * SPEED as f32;
    ferris.single_mut().unwrap().translation -=
        bg.single().unwrap().rotation * Vec3::new(d, 0., 0.);
}
