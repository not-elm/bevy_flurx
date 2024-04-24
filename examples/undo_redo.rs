//! This shows the implementation of undo and redo by moving a simple sprite.
//!
//! - A or ← : move left
//! - W or ↑ : move up
//! - D or → : move right
//! - S or ↓ : move down
//! - Z : Undo
//! - X : Redo


use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Camera2dBundle, Color, Commands, Component, EventWriter, In, KeyCode, Local, Query, Res, Sprite, Transform, With};
use bevy::sprite::SpriteBundle;
use bevy::time::Time;
use bevy::utils::default;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_egui::egui::{Color32, RichText};

use bevy_flurx::{actions, FlurxPlugin};
use bevy_flurx::action::{once, record, wait};
use bevy_flurx::prelude::{ActionSeed, OmitOutput, Pipe, Reactor, Record, RecordExtension, Redo, RequestRedo, RequestUndo, Rollback, Then, Track, Undo};

#[derive(Component)]
struct MrShape;

#[derive(Eq, PartialEq, Clone)]
struct MoveAct(usize, String);

type StartAndEndPos = (Vec3, Vec3);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            FlurxPlugin
        ))
        .init_resource::<Record<MoveAct>>()
        .add_record_events::<MoveAct>()
        .add_systems(Startup, (
            spawn_camera,
            spawn_mr_shape,
            spawn_undo_redo_reactor,
            spawn_move_reactor,
        ))
        .add_systems(Update, show_record)
        .run();
}

fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_mr_shape(
    mut commands: Commands
) {
    commands.spawn((
        MrShape,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 50.)),
                color: Color::WHITE,
                ..default()
            },
            ..default()
        }
    ));
}

fn spawn_undo_redo_reactor(
    mut commands: Commands
) {
    commands.spawn(Reactor::schedule(|task| async move {
        loop {
            let either = task.will(Update, wait::either(
                wait::input::just_pressed().with(KeyCode::KeyZ),
                wait::input::just_pressed().with(KeyCode::KeyX),
            )).await;
            if either.is_left() {
                let _ = task.will(Update, record::undo::once::<MoveAct>()).await;
            } else {
                let _ = task.will(Update, record::redo::once::<MoveAct>()).await;
            }
        }
    }));
}

fn show_record(
    mut context: EguiContexts,
    mut undo: EventWriter<RequestUndo<MoveAct>>,
    mut redo: EventWriter<RequestRedo<MoveAct>>,
    record: Res<Record<MoveAct>>,
) {
    egui::SidePanel::right("record")
        .min_width(200.)
        .show(context.ctx_mut(), |ui| {
            ui
                .vertical(|ui| {
                    let size = egui::Vec2::new(ui.available_width(), 30.);
                    for act in record.acts() {
                        let button = egui::Button::new(RichText::new(&act.1).color(Color32::BLACK))
                            .fill(Color32::GREEN)
                            .min_size(size);
                        if ui.add(button).clicked() {
                            undo.send(RequestUndo::To(act.clone()));
                        }
                    }
                    for act in record.redo_acts().rev() {
                        let button = egui::Button::new(RichText::new(&act.1).color(Color32::WHITE))
                            .fill(Color32::RED)
                            .min_size(size);
                        if ui.add(button).clicked() {
                            redo.send(RequestRedo::To(act.clone()));
                        }
                    }
                });
        });
}

fn spawn_move_reactor(
    mut commands: Commands
) {
    commands.spawn(Reactor::schedule(|task| async move {
        loop {
            task.will(Update, wait::any().with(actions![
                    wait::input::any_just_released().with(vec![KeyCode::KeyA, KeyCode::ArrowLeft]),
                    wait::input::any_just_released().with(vec![KeyCode::KeyW, KeyCode::ArrowUp]),
                    wait::input::any_just_released().with(vec![KeyCode::KeyS, KeyCode::ArrowDown]),
                    wait::input::any_just_released().with(vec![KeyCode::KeyD, KeyCode::ArrowRight]),
                ])
                .pipe(return_start_and_end_pos())
                .pipe(move_action::<1000>())
                .pipe(push_move_track()),
            )
                .await;
        }
    }));
}

fn return_start_and_end_pos() -> ActionSeed<usize, StartAndEndPos> {
    once::run(|In(key_index): In<usize>,
               m: Query<&Transform, With<MrShape>>| {
        let start = m.single().translation;
        let output = |end: Vec3| {
            (start, start + end * 100.)
        };
        match key_index {
            0 => output(Vec3::NEG_X),
            1 => output(Vec3::Y),
            2 => output(Vec3::NEG_Y),
            _ => output(Vec3::X),
        }
    })
}

fn move_action<const MILLIS: u64>() -> ActionSeed<StartAndEndPos, StartAndEndPos> {
    wait::output(|In((start, end)): In<StartAndEndPos>,
                  mut m: Query<&mut Transform, With<MrShape>>,
                  mut tick: Local<f32>,
                  time: Res<Time>| {
        let mut t = m.single_mut();
        let end_millis = Duration::from_millis(MILLIS).as_secs_f32();
        *tick = end_millis.min(*tick + time.delta_seconds());
        let d = *tick / end_millis;
        t.translation = start + (end - start) * d;
        ((*tick - end_millis).abs() < 0.01).then_some((start, end))
    })
}


fn push_move_track() -> ActionSeed<StartAndEndPos> {
    ActionSeed::define(|(start, end): StartAndEndPos| {
        static ID: AtomicUsize = AtomicUsize::new(0);

        record::push().with(Track {
            act: MoveAct(ID.fetch_add(1, Ordering::Relaxed), format!("start: ({:03.0},{:03.0}) end: ({:03.0},{:03.0})", start.x, start.y, end.x, end.y)),
            rollback: Rollback::parts(
                Undo::make(move || undo().with((start, end))),
                Redo::make(move |_| redo().with((start, end))),
            ),
        })
    })
        .omit_output()
}

//noinspection DuplicatedCode
fn undo() -> ActionSeed<StartAndEndPos> {
    ActionSeed::define(|(start, end): StartAndEndPos| {
        makeup().with(Color::GREEN)
            .then(move_action::<900>().with((end, start)))
            .then(makeup().with(Color::WHITE))
    })
}

//noinspection DuplicatedCode
fn redo() -> ActionSeed<StartAndEndPos> {
    ActionSeed::define(|(start, end): StartAndEndPos| {
        makeup().with(Color::RED)
            .then(move_action::<900>().with((start, end)))
            .then(makeup().with(Color::WHITE))
    })
}

fn makeup() -> ActionSeed<Color> {
    once::run(|In(color): In<Color>, mut sprite: Query<&mut Sprite, With<MrShape>>| {
        sprite.single_mut().color = color;
    })
}