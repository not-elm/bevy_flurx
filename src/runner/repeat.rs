use bevy::prelude::IntoSystem;
use crate::prelude::IntoAsyncScheduleCommand;
use crate::runner::config::AsyncSystemConfig;

use crate::runner::repeat::times::Times;

mod times;

#[path = "repeat/forever.rs"]
mod inner_forever;


/// Run the system every frame for the specified number of times.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         // Await for `count_up` to run 5 times.
///         schedules.add_system(Update, repeat::times(5, count_up)).await;
///     });
/// }
///
/// fn count_up(mut count: Local<u32>){
///     *count += 1;
/// }
/// ```
#[inline(always)]
pub fn times<Marker, Sys>(num: usize, system: Sys) -> impl IntoAsyncScheduleCommand
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), (), Marker> + Send + Sync + 'static
{
    Times::create(num, system)
}





/// Run the system  every frame until the task handle is dropped.
///
/// ```
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|schedules|async move{
///         let handle = schedules.add_system(Update, repeat::forever(count_up));
///         schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;
///         // Since the handle is dropped, `count_up` also stops.
///         drop(handle);
///     });
/// }
///
/// fn count_up(mut count: Local<u32>){
///     *count += 1;
/// }
/// ```
#[inline(always)]
pub fn forever<Marker, Sys>(system: Sys) -> impl IntoAsyncScheduleCommand
    where
        Marker: Send + Sync + 'static,
        Sys: IntoSystem<(), (), Marker> + Send + Sync + 'static
{
    inner_forever::Forever(AsyncSystemConfig::new(system))
}


