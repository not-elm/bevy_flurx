use std::time::Duration;

use crate::runner::delay::frame::DelayFrame;
use crate::runner::delay::time::DelayTime;
use crate::runner::IntoAsyncScheduleCommand;

// mod time;
mod frame;
mod time;


/// Delays by the specified number of frames.
///
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup_async_systems(mut commands: Commands){
///     commands.spawn_async(|schedules| async move{
///         schedules.add_system(Update, delay::frames(30)).await;
///     });
/// }
/// ```
#[inline(always)]
pub const fn frames(delay_frames: usize) -> impl IntoAsyncScheduleCommand {
    DelayFrame(delay_frames)
}


/// Delays the task using a [`Timer`](bevy::prelude::Timer).
///
///
/// ## Examples
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_async_system::prelude::*;
///
/// fn setup_async_systems(mut commands: Commands){
///     commands.spawn_async(|schedules| async move{
///         schedules.add_system(Update, delay::timer(Duration::from_secs(3))).await;
///     });
/// }
/// ```
#[inline(always)]
pub const fn timer(duration: Duration) -> impl IntoAsyncScheduleCommand {
    DelayTime(duration)
}