//! [`wait`] creates a task that run the until the condition is met.
//!
//! - [`wait::output`](crate::prelude::wait::output)
//! - [`wait::both`](crate::prelude::wait::both)
//! - [`wait::until`](crate::prelude::wait::until)
//! - [`wait_all!`](crate::wait_all)
//! - [`wait::either`](crate::prelude::wait::either::either)
//! - [`wait::event`](crate::prelude::wait::event)
//! - [`wait::state`](crate::prelude::wait::state)


use bevy::prelude::{In, IntoSystem};

pub use either::*;

use crate::action::seed::{ActionSeed, SeedMark};
use crate::action::seed::wait::WaitSeed;
use crate::prelude::Action;
use crate::runner::base::BaseTwoRunner;
use crate::runner::both::BothRunner;
use crate::runner::RunnerIntoAction;

pub mod event;
pub mod input;
pub mod state;
pub mod switch;
#[allow(missing_docs)]
pub mod all;
#[cfg(feature = "audio")]
pub mod audio;
mod either;


/// Run until it returns [`Option::Some`].
/// The contents of Some will be return value of the task.
///
/// ## Examples
///
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::output(||{
///         Some(())
///     })).await;
/// });
/// ```
#[inline(always)]
pub fn output<Sys, Input, Out, Marker>(system: Sys) -> impl ActionSeed<Input, Out> + SeedMark
    where
        Sys: IntoSystem<Input, Option<Out>, Marker>,
        Input: Clone + 'static,
        Out: 'static,
{
    WaitSeed::new(IntoSystem::into_system(system))
}

/// Run until it returns true.
///
/// ## Examples
/// 
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, wait::until(|mut count: Local<usize>|{
///         *count += 1;
///         *count == 4
///     })).await;
/// });
///```
#[inline(always)]
pub fn until<Input, Sys, M>(system: Sys) -> impl ActionSeed<Input> + SeedMark
    where
        Sys: IntoSystem<Input, bool, M> + 'static,
        Input: Clone + 'static,
{
    WaitSeed::new(IntoSystem::into_system(system.pipe(
        |In(finish): In<bool>| {
            if finish {
                Some(())
            } else {
                None
            }
        },
    )))
}

/// Run until both tasks done.
///
/// ## Examples
///
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task|async move{
///     task.will(Update, wait::both(
///         wait::input::just_pressed(),
///         wait::event::read::<AppExit>()
///     )).await;
/// });
/// ```
pub fn both<LI, LO, RI, RO>(
    lhs: impl Action<LI, LO> + 'static,
    rhs: impl Action<RI, RO> + 'static,
) -> impl Action<(LI, RI), (LO, RO)>
    where
        RI: Clone + 'static,
        LI: Clone + 'static,
        LO: Send + 'static,
        RO: Send + 'static,

{
    RunnerIntoAction::new(BothRunner(BaseTwoRunner::new(lhs, rhs)))
}


#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, PreUpdate, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, EventWriter, In, Local, Update};
    use bevy_test_helper::event::{TestEvent1, TestEvent2};

    use crate::action::{once, wait};
    use crate::action::wait::until;
    use crate::prelude::ActionSeed;
    use crate::reactor::Reactor;
    use crate::tests::test_app;

    #[test]
    fn count_up() {
        let mut app = test_app();
        app.world.run_system_once(|mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    until(|mut count: Local<u32>| {
                        *count += 1;
                        *count == 2
                    }),
                )
                    .await;

                task.will(Update, once::non_send::insert().with(AppExit)).await;
            }));
        });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn count_up_until_with_input() {
        let mut app = test_app();

        app.world.run_system_once(|mut commands: Commands| {
            commands.spawn(Reactor::schedule(|task| async move {
                task.will(
                    Update,
                    until(|input: In<u32>, mut count: Local<u32>| {
                        *count += 1 + input.0;
                        *count == 4
                    }).with(1),
                )
                    .await;

                task.will(Update, once::non_send::insert().with(AppExit)).await;
            }));
        });
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn wait_event() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    let event = task.will(PreUpdate, wait::event::read::<AppExit>()).await;
                    task.will(Update, once::non_send::insert().with(event)).await;
                }));
            });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.world
            .run_system_once(|mut w: EventWriter<AppExit>| w.send(AppExit));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn both_read_event1_and_event2() {
        let mut app = test_app();
        app
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Reactor::schedule(|task| async move {
                    let t1 = wait::event::read::<TestEvent1>();
                    let t2 = wait::event::read::<TestEvent2>();

                    let (event1, event2) = task.will(Update, wait::both(t1, t2)).await;
                    assert_eq!(event1, TestEvent1);
                    assert_eq!(event2, TestEvent2);
                    task.will(Update, once::non_send::insert().with(AppExit)).await;
                }));
            });

        app.update();

        app.world.run_system_once(|mut w: EventWriter<TestEvent1>| w.send(TestEvent1));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.world.run_system_once(|mut w: EventWriter<TestEvent2>| w.send(TestEvent2));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}
