//! [`wait`] creates a task that run the until the condition is met.
//!
//! actions
//!
//! - [`wait::output`]
//! - [`wait::both`]
//! - [`wait::until`]
//! - [`wait::all`](crate::prelude::wait::all())
//! - [`wait_all!`](crate::wait_all)
//! - [`wait::either`]
//! - [`wait::event`]
//! - [`wait::state`]
//! - [`wait::switch`]
//! - [`wait::input`]
//! - [`wait::audio`] (require feature flag `audio`)
//! - [`wait::any`]


use bevy::prelude::{In, IntoSystem, System, World};

pub use _any::any;
pub use _both::both;
pub use _either::*;

use crate::action::seed::ActionSeed;
use crate::prelude::wait;
use crate::runner::{CancellationToken, Output, Runner};

pub mod event;
pub mod input;
pub mod state;
pub mod switch;
pub use all::{all, private};
#[cfg(feature = "audio")]
pub mod audio;
#[path = "wait/either.rs"]
mod _either;
#[path = "wait/both.rs"]
mod _both;
#[path = "wait/any.rs"]
mod _any;
mod all;

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
pub fn output<Sys, Input, Out, Marker>(system: Sys) -> ActionSeed<Input, Out>
    where
        Sys: IntoSystem<Input, Option<Out>, Marker> + 'static,
        Input: Clone + 'static,
        Out: 'static,
{
    ActionSeed::new(move |input, token, output| {
        WaitRunner::new(input, token, output, IntoSystem::into_system(system))
    })
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
pub fn until<Input, Sys, M>(system: Sys) -> ActionSeed<Input>
    where
        Sys: IntoSystem<Input, bool, M> + 'static,
        Input: Clone + 'static,
{
    wait::output(system.pipe(
        |In(finish): In<bool>| {
            if finish {
                Some(())
            } else {
                None
            }
        }))
}

struct WaitRunner<Sys, I, O> {
    system: Sys,
    input: I,
    token: CancellationToken,
    output: Output<O>,
    init: bool,
}

impl<Sys, I, O> WaitRunner<Sys, I, O> {
    #[inline]
    const fn new(
        input: I,
        token: CancellationToken,
        output: Output<O>,
        system: Sys,
    ) -> WaitRunner<Sys, I, O> {
        Self {
            system,
            input,
            token,
            output,
            init: false,
        }
    }
}

impl<Sys, In, Out> Runner for WaitRunner<Sys, In, Out>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    fn run(&mut self, world: &mut World) -> bool {
        if self.token.requested_cancel() {
            return true;
        }
        if !self.init {
            self.system.initialize(world);
            self.init = true;
        }

        let out = self.system.run(self.input.clone(), world);
        self.system.apply_deferred(world);
        if let Some(o) = out {
            self.output.replace(o);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{AppExit, PreUpdate, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, EventWriter, In, Local, Update};
    use bevy_test_helper::event::{TestEvent1, TestEvent2};

    use crate::action::{once, wait};
    use crate::action::wait::until;
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
