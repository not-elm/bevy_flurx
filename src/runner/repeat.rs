use std::marker::PhantomData;

use bevy::prelude::IntoSystem;

use crate::runner::IntoAsyncSystem;
use crate::runner::config::AsyncSystemConfig;
use crate::runner::repeat::forever::Forever;
use crate::runner::repeat::times::Times;

mod times;
mod forever;

/// Delay the task using either [`Delay::Frame`] or [`Delay::Time`].
///
///
/// ```no_run
/// use std::time::Duration;
/// use bevy::prelude::*;
/// use bevy_async_system::ext::AsyncCommands;
/// use bevy_async_system::prelude::*;
///
/// fn setup(mut commands: Commands){
///     commands.spawn_async(|task| async move{
///         // Call `my_system` for 3 frames.
///         task.spawn(Update, Repeat::times(3, my_system)).await;
///
///         // It's called every frame while this task is running.
///         let handle = task.spawn(Update, Repeat::forever(my_system));
///
///         // When the handle is dropped, calling `my_system` also stops.
///         drop(handle)
///     });
/// }
///
/// fn my_system(){
///     // ... your code
/// }
/// ```
pub struct Repeat(PhantomData<()>);


impl Repeat {
    #[inline(always)]
    pub fn times<Marker>(num: usize, system: impl IntoSystem<(), (), Marker> + 'static + Send) -> impl IntoAsyncSystem {
        Times::create(num, system)
    }


    #[inline(always)]
    pub fn forever<Marker>(system: impl IntoSystem<(), (), Marker> + 'static + Send) -> impl IntoAsyncSystem {
        Forever(AsyncSystemConfig::new(system))
    }
}



