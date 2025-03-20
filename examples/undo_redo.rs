//! Shows how to use undo/redo.
//!
//! When you input a number key, it will be added to [`KeyCodes`] and output to the log.
//!
//! Press the Z key to delete the last input key, and the X key to restore the deleted key.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_flurx::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlurxPlugin,
        ))
        .add_record_events::<KeyCodeAct>()
        .init_resource::<Record<KeyCodeAct>>()
        .init_resource::<KeyCodes>()
        .add_systems(Update, (
            log_keycodes.run_if(resource_exists_and_changed::<KeyCodes>),
            do_undo.run_if(input_just_pressed(KeyCode::KeyZ)),
            do_redo.run_if(input_just_pressed(KeyCode::KeyX)),
            push_num_keycodes,
        ))
        .run();
}

#[derive(Default, Resource, Debug)]
struct KeyCodes(Vec<KeyCode>);

/// The act used by this example only has the meaning of an identifier for [`Record`], but by giving it a value to identify the act,
/// you can roll back to that point with [`RequestUndo::To`].
#[derive(PartialEq, Clone)]
struct KeyCodeAct;

fn log_keycodes(
    key_codes: Res<KeyCodes>,
) {
    info!("{:?}", key_codes);
}

fn do_undo(mut ew: EventWriter<RequestUndo<KeyCodeAct>>) {
    info!("Undo");
    ew.write(RequestUndo::Once);
}

fn do_redo(mut ew: EventWriter<RequestRedo<KeyCodeAct>>) {
    info!("Redo");
    ew.write(RequestRedo::Once);
}

fn push_num_keycodes(
    mut key_codes: ResMut<KeyCodes>,
    mut record: ResMut<Record<KeyCodeAct>>,
    input_key: Res<ButtonInput<KeyCode>>,
) {
    const NUM_KEYCODES: &[KeyCode] = &[
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    let all_pressed = input_key
        .get_just_pressed()
        .filter(|key| NUM_KEYCODES.contains(key))
        .collect::<Vec<_>>();
    let num_pressed = all_pressed.len();
    if num_pressed == 0 {
        return;
    }

    key_codes.0.extend(all_pressed);
    record.push(Track {
        act: KeyCodeAct,
        rollback: Rollback::parts(
            Undo::make(move || once::run(undo).with(num_pressed)),
            Redo::make(|keycodes: Vec<KeyCode>| once::run(redo).with(keycodes)),
        ),
    }).expect("Failed to push key codes");
}

fn undo(
    In(num_pressed): In<usize>,
    mut key_codes: ResMut<KeyCodes>,
) -> Vec<KeyCode> {
    let len = key_codes.0.len();
    key_codes.0.split_off(len - num_pressed)
}

fn redo(
    In(keycodes): In<Vec<KeyCode>>,
    mut key_codes: ResMut<KeyCodes>,
) {
    key_codes.0.extend(keycodes);
}