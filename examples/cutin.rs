//! A very simple 2D sprite movement example.
//!
//! The sprite moves up and then to the right.



use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_flurx::prelude::*;

#[derive(Component)]
struct CutIn;

struct RunCutIn;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin,
            WorldInspectorPlugin::new()
        ))
        .add_systems(Startup, (
            setup_entities,
            setup_reactor,
        ))
        .add_systems(Update, move_up.run_if(switch_turned_on::<RunCutIn>))
        .run();
}

fn setup_entities(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_cut_in(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let wh = &window.single().resolution;

    commands.spawn((
        CutIn,
        Name::new("cut in background"),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(wh.width(), 200.)),
                color: Color::BLUE,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_rotation_z((wh.height() / 2.).atan2(wh.width() / 2.)))
                .with_translation(Vec3::new(wh.width(), wh.height(), 0.)),
            ..default()
        }
    ));
}

fn setup_reactor(
    world: &mut World
) {
    world.schedule_reactor(|task| async move {
        task.will(Update, {
            delay::time(Duration::from_millis(100))
                .then(once::run(spawn_cut_in))
                .then(once::switch::on::<RunCutIn>())
                .then(wait::switch::off::<RunCutIn>())
                .then(delay::time(Duration::from_millis(1000)))
                
        }).await;
    });
}

fn move_up(
    mut cut_in: Query<&mut Transform, With<CutIn>>,
    mut tick: Local<f32>,
    mut switch: ResMut<Switch<RunCutIn>>,
    time: Res<Time>,
) {
    let end = Duration::from_millis(600).as_secs_f32();
    *tick += time.delta_seconds().min(end);
    for mut transform in cut_in.iter_mut() {
        let t = *tick / end;
        transform.translation = transform.translation.lerp(Vec3::new(0.,0.,0.), t);
    }
    if (*tick - end).abs() < 0.01{
        *tick = 0.;
        switch.off();
    }
}

