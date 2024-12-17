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

use crate::action::seed::ActionSeed;
use crate::prelude::{wait, RunnerStatus};
use crate::runner::{CancellationToken, Output, Runner};
pub use _any::any;
pub use _both::both;
pub use _either::*;
pub use all::{all, private};
use bevy::prelude::{In, IntoSystem, System, SystemIn, SystemInput, World};

#[path = "wait/any.rs"]
mod _any;
#[path = "wait/both.rs"]
mod _both;
#[path = "wait/either.rs"]
mod _either;
mod all;
#[cfg(feature = "audio")]
pub mod audio;
pub mod event;
pub mod input;
#[cfg(feature = "state")]
pub mod state;
pub mod switch;

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
/// Flow::schedule(|task| async move{
///     task.will(Update, wait::output(||{
///         Some(())
///     })).await;
/// });
/// ```
#[inline(always)]
pub fn output<Sys, I, O, Marker>(system: Sys) -> ActionSeed<I::Inner<'static>, O>
where
    Sys: IntoSystem<I, Option<O>, Marker> + Send + Sync + 'static,
    I: SystemInput + 'static,
    I::Inner<'static>: Clone,
    O: 'static,
{
    ActionSeed::new(move |input, output| WaitRunner {
        system: IntoSystem::into_system(system),
        input,
        output,
        init: false,
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
/// Flow::schedule(|task| async move{
///     task.will(Update, wait::until(|mut count: Local<usize>|{
///         *count += 1;
///         *count == 4
///     })).await;
/// });
///```
#[inline(always)]
pub fn until<I, Sys, M>(system: Sys) -> ActionSeed<I::Inner<'static>>
where
    Sys: IntoSystem<I, bool, M> + Send + Sync + 'static,
    I: SystemInput + 'static,
    I::Inner<'static>: Clone,
{
    wait::output(system.pipe(|In(finish): In<bool>| if finish { Some(()) } else { None }))
}

struct WaitRunner<Sys, O>
where
    Sys: System,
    SystemIn<'static, Sys>: Clone,
{
    system: Sys,
    input: <Sys::In as SystemInput>::Inner<'static>,
    output: Output<O>,
    init: bool,
}

impl<Sys, O> Runner for WaitRunner<Sys, O>
where
    Sys: System<Out=Option<O>>,
    SystemIn<'static, Sys>: Clone + 'static,
{
    fn run(&mut self, world: &mut World, _: &mut CancellationToken) -> RunnerStatus {
        if !self.init {
            self.system.initialize(world);
            self.init = true;
        }

        let out = self.system.run(self.input.clone(), world);
        self.system.apply_deferred(world);
        if let Some(o) = out {
            self.output.set(o);
            RunnerStatus::Ready
        } else {
            RunnerStatus::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::wait::until;
    use crate::action::{once, wait};
    use crate::prelude::Flow;
    use crate::tests::test_app;
    use bevy::app::{AppExit, PreUpdate, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{Commands, EventWriter, In, Local, Update};
    use bevy_test_helper::event::{TestEvent1, TestEvent2};

    #[test]
    fn count_up() {
        let mut app = test_app();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn(Flow::schedule(|task| async move {
                    task.will(
                        Update,
                        until(|mut count: Local<u32>| {
                            *count += 1;
                            *count == 2
                        }),
                    )
                        .await;

                    task.will(Update, once::non_send::insert().with(AppExit::Success)).await;
                }));
            })
            .expect("Failed to run system");

        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn count_up_until_with_input() {
        let mut app = test_app();

        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn(Flow::schedule(|task| async move {
                    task.will(
                        Update,
                        until(|input: In<u32>, mut count: Local<u32>| {
                            *count += 1 + input.0;
                            *count == 4
                        })
                            .with(1),
                    )
                        .await;

                    task.will(Update, once::non_send::insert().with(AppExit::Success))
                        .await;
                }));
            })
            .expect("Failed to run system");
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn wait_event() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Flow::schedule(|task| async move {
                let event = task.will(PreUpdate, wait::event::read::<AppExit>()).await;
                task.will(Update, once::non_send::insert().with(event))
                    .await;
            }));
        });

        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());

        app.world_mut()
            .run_system_once(|mut w: EventWriter<AppExit>| w.send(AppExit::Success))
            .expect("Failed to run system");
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }

    #[test]
    fn both_read_event1_and_event2() {
        let mut app = test_app();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Flow::schedule(|task| async move {
                let t1 = wait::event::read::<TestEvent1>();
                let t2 = wait::event::read::<TestEvent2>();

                let (event1, event2) = task.will(Update, wait::both(t1, t2)).await;
                assert_eq!(event1, TestEvent1);
                assert_eq!(event2, TestEvent2);
                task.will(Update, once::non_send::insert().with(AppExit::Success))
                    .await;
            }));
        });

        app.update();

        app.world_mut()
            .run_system_once(|mut w: EventWriter<TestEvent1>| w.send(TestEvent1))
            .expect("Failed to run system");
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());

        app.world_mut()
            .run_system_once(|mut w: EventWriter<TestEvent2>| w.send(TestEvent2))
            .expect("Failed to run system");
        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world().get_non_send_resource::<AppExit>().is_some());
    }
}
