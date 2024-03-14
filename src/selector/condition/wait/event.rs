use bevy::prelude::{Event, EventReader};
use crate::prelude::{ReactorSystemConfigs, wait, with};


/// Waits until the specified event is sent
///
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// #[derive(Default, Clone, Event)]
/// struct E;
///
/// let mut app = App::new();
/// app.add_event::<E>();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task|async move{
///         let wait_event = task.run(Update, wait::event::comes::<E>()).await;
///         task.will(Update, once::event::send(E)).await;
///         wait_event.await;
///         task.will(Update, once::non_send::init::<AppExit>()).await;
///     });
/// });
/// app.update();
/// app.update();
/// app.update();
/// app.update();
/// assert!(app.world.get_non_send_resource::<AppExit>().is_some());
/// ```
#[inline]
pub fn comes<E>() -> impl ReactorSystemConfigs<In=(), Out=()>
    where E: Event
{
    with((), wait::until(|er: EventReader<E>| {
        !er.is_empty()
    }))
}


/// Waits until the specified event is sent.
/// 
/// This is similar to [`wait::event::comes`], except that it returns the event itself.
/// 
/// ```
/// use bevy::app::AppExit;
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
/// 
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task|async move{
///         let wait_event = task.run(Update, wait::event::read::<AppExit>()).await;
///         task.will(Update, once::event::send(AppExit)).await;
///         task.will(Update, once::non_send::insert::<AppExit>(wait_event.await)).await;
///     });
/// });
/// app.update();
/// app.update();
/// app.update();
/// app.update();
/// assert!(app.world.get_non_send_resource::<AppExit>().is_some());
/// ```
#[inline]
pub fn read<E>() -> impl ReactorSystemConfigs<In=(), Out=E>
    where E: Event + Clone
{
    with((), wait::output(|mut er: EventReader<E>| {
        er.read().next().cloned()
    }))
}