//! [`wait`] creates a task that run the until the condition is met.
//!
//! - [`wait::output`](crate::prelude::wait::output)
//! - [`wait::both`](crate::prelude::wait::both)
//! - [`wait::until`](crate::prelude::wait::until)
//! - [`wait_all!`](crate::wait_all)
//! - [`wait::either`](crate::prelude::wait::either::either)
//! - [`wait::event`](crate::prelude::wait::event)
//! - [`wait::state`](crate::prelude::wait::state)

use bevy::prelude::{In, IntoSystem, Local, System, World};
pub use either::*;
use crate::prelude::{ReactorAction, with, WithInput};

pub mod event;
pub mod state;
#[allow(missing_docs)]
pub mod all;
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
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move{
///         let count: u8 = task.will(Update, wait::output(|mut count: Local<u8>|{
///             *count += 1;
///             (*count == 2).then_some(*count)
///         })).await;
///         assert_eq!(count, 2);
///     });
/// });
/// app.update();
/// app.update();
/// ```
#[inline]
pub fn output<Sys, Input, Out, Marker>(system: Sys) -> impl System<In=Input, Out=Option<Out>>
    where
        Sys: IntoSystem<Input, Option<Out>, Marker>,
        Input: 'static,
        Out: 'static,
{
    IntoSystem::into_system(system)
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
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move{
///         task.will(Update, wait::until(|mut count: Local<usize>|{
///             *count += 1;
///             *count == 2
///         })).await;
///         task.will(Update, once::non_send::init::<AppExit>()).await;
///     });
/// });
/// app.update();
/// assert!(app.world.get_non_send_resource::<AppExit>().is_none());
/// app.update();
/// assert!(app.world.get_non_send_resource::<AppExit>().is_none());
/// app.update(); // send app exit
/// assert!(app.world.get_non_send_resource::<AppExit>().is_some());
///```
#[inline]
pub fn until<Input, Sys, Marker>(system: Sys) -> impl System<In=Input, Out=Option<()>>
    where
        Sys: IntoSystem<Input, bool, Marker> + 'static,
{
    IntoSystem::into_system(system.pipe(
        |In(finish): In<bool>| {
            if finish {
                Some(())
            } else {
                None
            }
        },
    ))
}

/// Run until both tasks done.
///
/// 
/// ## Examples
/// 
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
/// 
/// #[derive(Default, Clone, Event, PartialEq, Debug)]
/// struct Event1;
/// #[derive(Default, Clone, Event, PartialEq, Debug)]
/// struct Event2;
/// 
/// let mut app = App::new();
/// app.add_event::<Event1>();
/// app.add_event::<Event2>();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move{
///         let (event1, event2) = task.will(Update, wait::both(
///             wait::event::read::<Event1>(),
///             wait::event::read::<Event2>()
///         )).await;
///         assert_eq!(event1, Event1);
///         assert_eq!(event2, Event2);
///     });
/// });
/// app.update();
/// app.world.resource_mut::<Events<Event1>>().send_default();
/// app.world.resource_mut::<Events<Event2>>().send_default();
/// app.update();
/// ```
pub fn both<LI, LO, LM, RI, RO, RM>(
    lhs: impl ReactorAction<LM, In=LI, Out=LO> + 'static,
    rhs: impl ReactorAction<RM, In=RI, Out=RO> + 'static,
) -> impl ReactorAction<WithInput, In=(LI, RI), Out=(LO, RO)>
    where
        RI: Clone + 'static,
        LI: Clone + 'static,
        LO: Send + 'static,
        RO: Send + 'static,
{
    let (l_in, mut l_sys) = lhs.split();
    let (r_in, mut r_sys) = rhs.split();

    with(
        (l_in, r_in),
        IntoSystem::into_system(
            move |In((l_in, r_in)): In<(LI, RI)>,
                  world: &mut World,
                  mut init: Local<bool>,
                  mut l_out: Local<Option<LO>>,
                  mut r_out: Local<Option<RO>>| {
                both_init_systems(&mut init, world, &mut l_sys, &mut r_sys);
                both_run_systems(world, l_in, &mut l_sys, &mut l_out, r_in, &mut r_sys, &mut r_out)
            }
        ),
    )
}

#[inline]
fn both_init_systems<LI, LO, RI, RO>(
    init: &mut Local<bool>,
    world: &mut World,
    l_sys: &mut impl System<In=LI, Out=Option<LO>>,
    r_sys: &mut impl System<In=RI, Out=Option<RO>>,
) {
    if !(**init) {
        l_sys.initialize(world);
        r_sys.initialize(world);
        **init = true;
    }
}

fn both_run_systems<LI, LO, RI, RO>(
    world: &mut World,
    l_in: LI,
    l_sys: &mut impl System<In=LI, Out=Option<LO>>,
    l_out: &mut Local<Option<LO>>,
    r_in: RI,
    r_sys: &mut impl System<In=RI, Out=Option<RO>>,
    r_out: &mut Local<Option<RO>>,
) -> Option<(LO, RO)>
    where
        LO: Send,
        RO: Send
{
    if l_out.is_none() {
        **l_out = l_sys.run(l_in, world);
        l_sys.apply_deferred(world);
    }
    if r_out.is_none() {
        **r_out = r_sys.run(r_in, world);
        r_sys.apply_deferred(world);
    }

    let l_out_value = l_out.take()?;
    if let Some(r_out_value) = r_out.take() {
        Some((l_out_value, r_out_value))
    } else {
        l_out.replace(l_out_value);
        None
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, PreUpdate, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{EventWriter, In, Local, Update, World};
    use bevy_test_helper::event::{TestEvent1, TestEvent2};

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::action::{once, wait, with};
    use crate::action::wait::until;
    use crate::tests::test_app;

    #[test]
    fn count_up() {
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);

        app.world.run_system_once(|world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(
                    Update,
                    until(|mut count: Local<u32>| {
                        *count += 1;
                        *count == 2
                    }),
                )
                    .await;

                task.will(Update, once::non_send::insert(AppExit)).await;
            });
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
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);

        app.world.run_system_once(|world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(
                    Update,
                    with(
                        1,
                        until(|input: In<u32>, mut count: Local<u32>| {
                            *count += 1 + input.0;
                            *count == 4
                        }),
                    ),
                )
                    .await;

                task.will(Update, once::non_send::insert(AppExit)).await;
            });
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
        let mut app = App::new();
        app.add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    let event = task.will(PreUpdate, wait::event::read::<AppExit>()).await;
                    task.will(Update, once::non_send::insert(event)).await;
                });
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
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    let t1 = wait::event::read::<TestEvent1>();
                    let t2 = wait::event::read::<TestEvent2>();

                    let (event1, event2) = task.will(Update, wait::both(t1, t2)).await;
                    assert_eq!(event1, TestEvent1);
                    assert_eq!(event2, TestEvent2);
                    task.will(Update, once::non_send::insert(AppExit)).await;
                });
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
