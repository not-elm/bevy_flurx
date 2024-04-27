//! [`once::switch`] creates a task that only once run system related to [`Switch`].
//!
//! - [`once::switch::on`]
//! - [`once::switch::off`]


use bevy::prelude::World;

use crate::action::once;
use crate::action::seed::ActionSeed;
use crate::action::switch::Switch;


/// Turns [`Switch`] on.
/// 
/// ## Examples
/// 
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// 
/// struct Animation;
/// 
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::switch::on::<Animation>()).await;
/// });
/// ```
#[inline]
pub fn on<M>() -> ActionSeed
    where M: Send + Sync + 'static
{
    once::run(|world: &mut World| {
        Switch::<M>::setup(world, true);
    })
}

/// Turns [`Switch`] off.
/// 
/// ## Examples
/// 
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// 
/// struct Animation;
/// 
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::switch::off::<Animation>()).await;
/// });
/// ```
#[inline]
pub fn off<M>() -> ActionSeed
    where M: Send + Sync + 'static
{
    once::run(|world: &mut World| {
        Switch::<M>::setup(world, false);
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::Startup;
    use bevy::prelude::{Commands, IntoSystemConfigs, ResMut, Update};
    use bevy_test_helper::resource::bool::{Bool, BoolExtension};

    use crate::action::once;
    use crate::prelude::{switch_just_turned_off, switch_just_turned_on};
    use crate::reactor::Reactor;
    use crate::tests::test_app;

    struct T;

    #[test]
    fn once_switch_on() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(Update, once::switch::on::<T>()).await;
                }));
            })
            .add_systems(Update, (|mut b: ResMut<Bool>| {
                **b = true;
            }).run_if(switch_just_turned_on::<T>));

        app.update();
        assert!(app.is_bool_true());
    }

    #[test]
    fn once_switch_on_after_1frame() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(Update, once::run(|| {})).await;
                    task.will(Update, once::switch::on::<T>()).await;
                }));
            })
            .add_systems(Update, (|mut b: ResMut<Bool>| {
                **b = true;
            }).run_if(switch_just_turned_on::<T>));

        app.update();
        assert!(app.is_bool_false());
        app.update();
        assert!(app.is_bool_true());
    }

    #[test]
    fn once_switch_off() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(Update, once::switch::off::<T>()).await;
                }));
            })
            .add_systems(Update, (|mut b: ResMut<Bool>| {
                **b = true;
            }).run_if(switch_just_turned_off::<T>));

        app.update();
        assert!(app.is_bool_true());
    }

    #[test]
    fn once_switch_off_after_1frame() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    task.will(Update, once::run(|| {})).await;
                    task.will(Update, once::switch::off::<T>()).await;
                }));
            })
            .add_systems(Update, (|mut b: ResMut<Bool>| {
                **b = true;
            }).run_if(switch_just_turned_off::<T>));

        app.update();
        assert!(app.is_bool_false());
        app.update();
        assert!(app.is_bool_true());
    }
}