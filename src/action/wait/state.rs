//! [`wait::state`] creates a task related to waiting to state update.
//!
//! - [`wait::state::becomes`]


use bevy::prelude::{In, Res, State, States};

use crate::action::wait;
use crate::prelude::{ActionSeed, Seed};

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
/// Flurx::schedule(|task| async move {
///     task.will(Update, once::state::set(Status::Second)).await;
/// });
/// ```
#[inline(always)]
pub fn becomes<S>() -> impl ActionSeed<S> + Seed
    where S: States + 'static
{
    wait::until(move |In(expect): In<S>,
                      state_now: Res<State<S>>| {
        state_now.as_ref() == &expect
    })
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, First, Startup, Update};
    use bevy::prelude::{Commands, States};

    use crate::action::{once, wait};
    use crate::FlurxPlugin;
    use crate::prelude::ActionSeed;
    use crate::scheduler::Flurx;

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
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Flurx::schedule(|task| async move {
                    task.will(First, wait::state::becomes().with(TestState::Phase2)).await;
                    task.will(Update, once::non_send::init::<AppExit>()).await;
                }));
            });
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.insert_state(TestState::Phase2);
        app.update();
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}