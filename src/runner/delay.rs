use std::marker::PhantomData;
use std::time::Duration;

use crate::runner::delay::frame::DelayFrame;
use crate::runner::delay::time::DelayTime;
use crate::runner::IntoMainThreadExecutor;


// mod time;
mod frame;
mod time;


/// Delay the task using either [`Delay::Frames`] or [`Delay::Time`].
///
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_async_system::ext::SpawnAsyncCommands;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|task| async move{
///         // Wait for 3 frames.
///         task.spawn_on_main(Update, Delay::Frames(3)).await;
///
///         // Wait for 3 seconds.
///         task.spawn_on_main(Update, Delay::Time(Duration::from_secs(3))).await;
///     });
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Delay(PhantomData<()>);

//
// impl Delay {
//
//
//     #[inline(always)]
//     pub const fn time<'w>(duration: Duration) -> impl IntoThreadPoolExecutor<DelayTimeParam<'w>> {
//         DelayTime(duration)
//     }
// }


#[inline(always)]
pub const fn frames(delay_frames: usize) -> impl IntoMainThreadExecutor {
    DelayFrame(delay_frames)
}


#[inline(always)]
pub const fn timer(duration: Duration) -> impl IntoMainThreadExecutor {
    DelayTime(duration)
}