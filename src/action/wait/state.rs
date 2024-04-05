//! [`wait::state`] creates a task related to waiting to state update.
//!
//! - [`wait::state::becomes`]


use bevy::prelude::{Res, State, States, };

use crate::action::{ TaskAction, wait, with};

/// Waits until the state becomes the specified.
///
/// ```no_run
/// use bevy::prelude::{States, World, Update};
/// use bevy_flurx::prelude::*;
///
/// #[derive(States, Eq, PartialEq, Copy, Clone, Hash, Default, Debug)]
/// enum Status{
///     #[default]
///     First,
///     Second
/// }
///
/// let mut world = World::default();
/// world.schedule_reactor(|task| async move {
///     task.will(Update, once::state::set(Status::Second)).await; 
/// });
/// ```
#[inline(always)]
pub fn becomes<S>(state: S) -> impl TaskAction< (), ()>
    where S: States + 'static
{
    with((), wait::until(move |state_now: Res<State<S>>| {
        state_now.as_ref() == &state
    }))
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup, Update};
    use bevy::prelude::{States, World};

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::action::{once, wait};

    #[derive(States, Eq, PartialEq, Default, Copy, Clone, Hash, Debug)]
    enum TestState {
        #[default]
        Phase1,
        Phase2,
    }

    #[test]
    fn wait_until_state_becomes_phase2() {
        let mut app = App::new();
        app
            .add_plugins(FlurxPlugin)
            .init_state::<TestState>()
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    task.will(First, wait::state::becomes(TestState::Phase2)).await;
                    task.will(Update, once::non_send::init::<AppExit>()).await;
                });
            });
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.insert_state(TestState::Phase2);
        app.update();
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}