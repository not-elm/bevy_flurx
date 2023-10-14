use std::marker::PhantomData;
use std::time::Duration;

use bevy::app::App;
use bevy::prelude::Plugin;

use crate::ext::add_system::AddSystem;
use crate::runner::multi_thread::IntoThreadPoolExecutor;
use crate::runner::multi_thread::delay::frame::{DelayFrame, DelayFrameParam};
use crate::runner::multi_thread::delay::time::{DelayTime, DelayTimeParam};

mod time;
mod frame;

pub struct DelayPlugin;

impl Plugin for DelayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_thread_pool_system_on_main_scheduler::<DelayTimeParam>()
            .add_thread_pool_system_on_main_scheduler::<DelayFrameParam>();
    }
}


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


impl Delay {
    #[inline]
    pub const fn frames(delay_frames: usize) -> impl IntoThreadPoolExecutor<DelayFrameParam> {
        DelayFrame(delay_frames)
    }


    #[inline(always)]
    pub fn time<'w>(duration: Duration) -> impl IntoThreadPoolExecutor<DelayTimeParam<'w>> {
        DelayTime(duration)
    }
}