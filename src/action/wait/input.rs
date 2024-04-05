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

use crate::action::{TaskAction, wait, with};

/// Waits until item has just been pressed.
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// let mut world = World::new();
/// world.schedule_reactor(|task|async move{
///     task.will(Update, wait::input::just_pressed(KeyCode::KeyA)).await;
/// });
/// ```
#[inline(always)]
pub fn just_pressed<T: Copy + Eq + Hash + Send + Sync + 'static>(item: T) -> impl TaskAction<In=(), Out=()> {
    wait::until(move |input: Res<ButtonInput<T>>| {
        input.just_pressed(item)
    })
}

/// Waits until keycode has been pressed.
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// let mut world = World::new();
/// world.schedule_reactor(|task|async move{
///     task.will(Update, wait::input::pressed(KeyCode::KeyA)).await;
/// });
/// ```
#[inline(always)]
pub fn pressed<T: Copy + Eq + Hash + Send + Sync + 'static>(item: T) -> impl TaskAction<In=(), Out=()> {
    wait::until(move |input: Res<ButtonInput<T>>| {
        input.pressed(item)
    })
}

/// Waits until any keycode in inputs has been pressed.
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// let mut world = World::new();
/// world.schedule_reactor(|task|async move{
///     task.will(Update, wait::input::any_pressed([KeyCode::KeyA, KeyCode::KeyB])).await;
/// });
/// ```
#[inline(always)]
pub fn any_pressed<T: Copy + Eq + Hash + Send + Sync + 'static>(items: impl IntoIterator<Item=T>) -> impl TaskAction<In=Vec<T>, Out=()> {
    let items = items.into_iter().collect::<Vec<_>>();
    with(items, wait::until(|In(items): In<Vec<T>>,
                             input: Res<ButtonInput<T>>| {
        input.any_pressed(items)
    }))
}

/// Waits until all keycodes in inputs have been pressed.
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// let mut world = World::new();
/// world.schedule_reactor(|task|async move{
///     task.will(Update, wait::input::all_pressed([KeyCode::KeyA, KeyCode::KeyB])).await;
/// });
/// ```
#[inline(always)]
pub fn all_pressed<T: Copy + Eq + Hash + Send + Sync + 'static>(items: impl IntoIterator<Item=T>) -> impl TaskAction<In=Vec<T>, Out=()> {
    let items = items.into_iter().collect::<Vec<_>>();
    with(items, wait::until(|In(items): In<Vec<T>>,
                             input: Res<ButtonInput<T>>| {
        input.all_pressed(items)
    }))
}

/// Waits keycode has just been released.
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// let mut world = World::new();
/// world.schedule_reactor(|task|async move{
///     task.will(Update, wait::input::just_released(KeyCode::KeyA)).await;
/// });
/// ```
#[inline(always)]
pub fn just_released<T: Copy + Eq + Hash + Send + Sync + 'static>(item: T) -> impl TaskAction<In=(), Out=()> {
    wait::until(move |input: Res<ButtonInput<T>>| {
        input.just_released(item)
    })
}

/// Waits any keycode in inputs have just been released.
///
/// ```no_run
/// use bevy::prelude::{KeyCode, World, Update};
/// use bevy_flurx::prelude::*;
///
/// let mut world = World::new();
/// world.schedule_reactor(|task|async move{
///     task.will(Update, wait::input::any_just_released([KeyCode::KeyA, KeyCode::KeyB])).await;
/// });
/// ```
#[inline(always)]
pub fn any_just_released<T: Copy + Eq + Hash + Send + Sync + 'static>(items: impl IntoIterator<Item=T>) -> impl TaskAction<In=Vec<T>, Out=()> {
    let items = items.into_iter().collect::<Vec<_>>();
    with(items, wait::until(|In(items): In<Vec<T>>,
                             input: Res<ButtonInput<T>>| {
        input.any_just_released(items)
    }))
}


#[cfg(test)]
mod tests {
    use bevy::app::{First, Startup};
    use bevy::input::ButtonInput;
    use bevy::prelude::{KeyCode, World};
    use bevy::prelude::KeyCode::{KeyA, KeyB, KeyC, KeyD};
    use bevy_test_helper::resource::bool::BoolExtension;
    use bevy_test_helper::resource::DirectResourceControl;

    use crate::action::{once, wait};
    use crate::extension::ScheduleReactor;
    use crate::prelude::Then;
    use crate::sequence;
    use crate::tests::test_app;

    #[test]
    fn wait_until_pressed_a() {
        let mut app = test_app();
        app.add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(First, wait::input::just_pressed(KeyCode::KeyA)
                    .then_action(wait::input::pressed(KeyA))
                    .then_action(once::run(|world: &mut World| {
                        world.set_bool(true);
                    })),
                ).await;
            });
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
        app.add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(First, sequence! {
                    wait::input::any_pressed([KeyA, KeyB]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
                task.will(First, sequence! {
                    wait::input::any_pressed([KeyA, KeyD]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            });
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
        app.add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(First, sequence! {
                    wait::input::all_pressed([KeyA, KeyB]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            });
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
        app.add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(First, sequence! {
                    wait::input::just_released(KeyA),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            });
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
        app.add_systems(Startup, |world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(First, sequence! {
                    wait::input::any_just_released([KeyCode::KeyA, KeyCode::KeyB]),
                    once::run(|world: &mut World|{
                        world.set_bool(true);
                    })
                }).await;
            });
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