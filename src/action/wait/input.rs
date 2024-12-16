//! [`wait::input`] creates a task related to waiting to keycode inputs.
//!
//! - [`wait::input::just_pressed`]
//! - [`wait::input::pressed`]
//! - [`wait::input::all_pressed`]
//! - [`wait::input::any_pressed`]
//! - [`wait::input::just_released`]
//! - [`wait::input::any_just_released`]

use std::hash::Hash;

use bevy::input::ButtonInput;
use bevy::prelude::{In, Res};

use crate::action::seed::ActionSeed;
use crate::action::wait;

/// Waits until item has just been pressed.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, wait::input::just_pressed().with(KeyCode::KeyB)).await;
/// });
/// ```
#[inline(always)]
pub fn just_pressed<T>() -> ActionSeed<T>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    wait::until(move |In(expect): In<T>,
                      input: Res<ButtonInput<T>>| {
        input.just_pressed(expect)
    })
}

/// Waits until keycode has been pressed.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, wait::input::pressed().with(KeyCode::KeyB)).await;
/// });
/// ```
#[inline(always)]
pub fn pressed<T>() -> ActionSeed<T>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    wait::until(move |In(expect): In<T>,
                      input: Res<ButtonInput<T>>| {
        input.pressed(expect)
    })
}

/// Waits until any keycode in inputs has been pressed.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, wait::input::any_pressed().with(vec![KeyCode::KeyA, KeyCode::KeyB])).await;
/// });
/// ```
#[inline(always)]
pub fn any_pressed<T>() -> ActionSeed<Vec<T>>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    wait::until(|In(items): In<Vec<T>>,
                 input: Res<ButtonInput<T>>| {
        input.any_pressed(items)
    })
}

/// Waits until all keycodes in inputs have been pressed.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, wait::input::all_pressed().with(vec![KeyCode::KeyA, KeyCode::KeyB])).await;
/// });
/// ```
#[inline(always)]
pub fn all_pressed<T>() -> ActionSeed<Vec<T>>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    wait::until(|In(items): In<Vec<T>>,
                 input: Res<ButtonInput<T>>| {
        input.all_pressed(items)
    })
}

/// Waits keycode has just been released.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, wait::input::just_released().with(KeyCode::KeyA)).await;
/// });
/// ```
#[inline(always)]
pub fn just_released<T>() -> ActionSeed<T>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    wait::until(move |In(expect): In<T>,
                      input: Res<ButtonInput<T>>| {
        input.just_released(expect)
    })
}

/// Waits any keycode in inputs have just been released.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, wait::input::any_just_released().with(vec![KeyCode::KeyA, KeyCode::KeyB])).await;
/// });
/// ```
#[inline(always)]
pub fn any_just_released<T>() -> ActionSeed<Vec<T>>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    wait::until(|In(items): In<Vec<T>>,
                 input: Res<ButtonInput<T>>| {
        input.any_just_released(items)
    })
}

#[cfg(test)]
mod tests {
    use crate::action::sequence::Then;
    use crate::action::{once, wait};
    use crate::sequence;
    use crate::tests::test_app;
    use bevy::app::{First, Startup};
    use bevy::input::ButtonInput;
    use bevy::prelude::KeyCode::{KeyA, KeyB, KeyC, KeyD};
    use bevy::prelude::{Commands, KeyCode, World};
    use bevy_test_helper::resource::bool::BoolExtension;
    use bevy_test_helper::resource::DirectResourceControl;

    #[test]
    fn wait_until_pressed_a() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, wait::input::just_pressed().with(KeyCode::KeyA)
                    .then(wait::input::pressed().with(KeyA))
                    .then(once::run(|world: &mut World| {
                        world.set_bool(true);
                    })),
                ).await;
            }));
        });

        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyA);
        app.update();
        assert!(app.is_bool_true());
    }

    #[test]
    fn wait_until_any_pressed() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, sequence! {
                    wait::input::any_pressed().with(vec![KeyA, KeyB]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
                task.will(First, sequence! {
                    wait::input::any_pressed().with(vec![KeyA, KeyD]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            }));
        });

        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyC);
        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyA);
        app.update();
        assert!(app.is_bool_true());

        app.set_bool(false);
        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyD);
        app.update();
        assert!(app.is_bool_true());
    }

    #[test]
    fn wait_until_all_pressed() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, sequence! {
                    wait::input::all_pressed().with(vec![KeyA, KeyB]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            }));
        });

        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyA);
        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyB);
        app.update();
        assert!(app.is_bool_true());
    }

    #[test]
    fn wait_until_just_released() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, sequence! {
                    wait::input::just_released().with(KeyA),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            }));
        });

        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyA);
        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().release(KeyA);
        app.update();
        assert!(app.is_bool_true());
    }

    #[test]
    fn wait_until_any_just_released() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(crate::prelude::Flow::schedule(|task| async move {
                task.will(First, sequence! {
                    wait::input::any_just_released().with(vec![KeyCode::KeyA, KeyCode::KeyB]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            }));
        });

        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().press(KeyA);
        app.update();
        assert!(app.is_bool_false());

        app.resource_mut::<ButtonInput<KeyCode>>().release(KeyA);
        app.update();
        assert!(app.is_bool_true());
    }
}