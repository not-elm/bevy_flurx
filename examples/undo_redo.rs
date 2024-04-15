//!


use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::prelude::{Camera2dBundle, Children, Commands, Component, Entity, In, KeyCode, NodeBundle, Query, Style, TextBundle, Val, With};
use bevy::text::{Text, TextStyle};
use bevy::ui::{Display, FlexDirection, JustifyContent};
use bevy::utils::default;

use bevy_flurx::{actions, FlurxPlugin};
use bevy_flurx::action::{Omit, OmitInput, once, redo, undo, wait};
use bevy_flurx::prelude::{ActionSeed, Map, Pipe, Reactor};

#[derive(Component)]
struct ListUi;

struct ListItem;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin
        ))
        .add_systems(Startup, (
            setup_add_item_reactor,
            setup_undo_redo_reactor,
            setup_ui
        ))
        .run();
}

fn setup_ui(
    mut commands: Commands
) {
    commands.spawn((
        ListUi,
        NodeBundle {
            style: Style {
                max_height: Val::Percent(100.),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
    ));
    commands.spawn(Camera2dBundle::default());
}

fn setup_add_item_reactor(
    mut commands: Commands
) {
    commands.spawn(Reactor::schedule(|task| async move {
        loop {
            task.will(Update, wait_inputs()
                .pipe(once::run(spawn_item))
                .pipe(undo::push(ListItem, |input| {
                    once::run(undo).with(input)
                })),
            ).await;
        }
    }));
}

fn setup_undo_redo_reactor(
    mut commands: Commands
) {
    commands.spawn(Reactor::schedule(|task| async move {
        loop {
            let either = task.will(Update, wait::either(
                wait::input::just_pressed().with(KeyCode::KeyR),
                wait::input::just_pressed().with(KeyCode::KeyU),
            ))
                .await;

            if either.is_left() {
                task.will(Update, undo::execute::<ListItem>()).await;
            } else {
                task.will(Update, redo::execute::<ListItem>()).await;
            }
        }
    }));
}

fn wait_inputs() -> ActionSeed<(), &'static str> {
    wait::any(actions![
        wait::input::just_pressed().with(KeyCode::KeyA),
         wait::input::just_pressed().with(KeyCode::KeyW),
         wait::input::just_pressed().with(KeyCode::KeyS),
         wait::input::just_pressed().with(KeyCode::KeyD),
    ])
        .map(|i| match i {
            0 => "A",
            1 => "W",
            2 => "S",
            _ => "D"
        })
        .omit_input()
}

fn spawn_item(
    In(key): In<&'static str>,
    mut commands: Commands,
    list: Query<Entity, With<ListUi>>,
) -> (&'static str, Entity) {
    let entity = commands
        .spawn(TextBundle {
            text: Text::from_section(key, TextStyle {
                font_size: 32.,
                ..default()
            }),
            ..default()
        })
        .id();
    commands
        .entity(list.single())
        .add_child(entity);
    (key, entity)
}

fn undo(
    In((key, item_entity)): In<(&'static str, Entity)>,
    mut commands: Commands,
    list: Query<&Children, With<ListUi>>
) -> Option<ActionSeed> {
    if let Some(entity) = list.single().last(){
        commands.entity(*entity).despawn_recursive();
    }
    Some(once::run(spawn_item).with(key).omit())
}