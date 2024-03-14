//! [`wait`] creates a task that run the until the condition is met.
//!
//! - [`wait::output`](crate::prelude::wait::output)
//! - [`wait::until`](crate::prelude::wait::until)
//! - [`wait::select`](crate::prelude::wait::select::select)
//! - [`wait::event::read`](crate::prelude::wait::event::read)
//! - [`wait::event::comes`](crate::prelude::wait::event::comes)
//! - [`wait::state::becomes`](crate::prelude::wait::state::becomes)


use bevy::prelude::{In, IntoSystem, System};

pub use select::*;

pub mod state;
pub mod event;
mod select;

/// Run until it returns Option::Some.
/// The contents of Some will be return value of the task.
/// 
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
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
        Out: 'static
{
    IntoSystem::into_system(system)
}


/// Run until it returns true.
///
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move{
///         task.will(Update, wait::until(|mut count: Local<u8>|{
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
/// ```
#[inline]
pub fn until<Input, Sys, Marker>(system: Sys) -> impl System<In=Input, Out=Option<()>>
    where
        Sys: IntoSystem<Input, bool, Marker> + 'static
{
    IntoSystem::into_system(
        system
            .pipe(|In(finish): In<bool>| {
                if finish {
                    Some(())
                } else {
                    None
                }
            })
    )
}


#[cfg(test)]
mod tests {
    use bevy::app::{App, AppExit, PreUpdate, Startup};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::{EventWriter, In, Local, Update, World};

    use crate::extension::ScheduleReactor;
    use crate::FlurxPlugin;
    use crate::selector::condition::{once, wait, with};
    use crate::selector::condition::wait::until;

    #[test]
    fn count_up() {
        let mut app = App::new();
        app.add_plugins(FlurxPlugin);

        app.world.run_system_once(|world: &mut World| {
            world.schedule_reactor(|task| async move {
                task.will(Update, until(|mut count: Local<u32>| {
                    *count += 1;
                    *count == 2
                })).await;

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
                task.will(Update, with(1, until(|input: In<u32>, mut count: Local<u32>| {
                    *count += 1 + input.0;
                    *count == 4
                }))).await;

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
        app
            .add_plugins(FlurxPlugin)
            .add_systems(Startup, |world: &mut World| {
                world.schedule_reactor(|task| async move {
                    let event = task.will(PreUpdate, wait::event::read::<AppExit>()).await;
                    task.will(Update, once::non_send::insert(event)).await;
                });
            });

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.world.run_system_once(|mut w: EventWriter<AppExit>| w.send(AppExit));
        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_none());

        app.update();
        assert!(app.world.get_non_send_resource::<AppExit>().is_some());
    }
}