//! [`once`] creates a task that only once run system.
//!
//! - [`once::res`](crate::prelude::once::res)
//! - [`once::non_send`](crate::prelude::once::res)
//! - [`once::event`](crate::prelude::once::res)
//! - [`once::state`](crate::prelude::once::res)


use bevy::prelude::{In, IntoSystem};
use crate::action::seed::{ActionSeed, Seed};
use crate::action::seed::once::OnceSeed;
use crate::prelude::TaskAction;
use crate::runner::once::OnceRunner;
use crate::runner::RunnerIntoAction;

pub mod res;
pub mod non_send;
pub mod event;
pub mod state;
pub mod switch;
#[cfg(feature = "audio")]
pub mod audio;


/// Once run a system.
///
/// The return value will be the system return value.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         let count = task.will(Update, once::run(|mut count: Local<u8>|{
///             *count += 1;
///             *count
///         })).await;
///         assert_eq!(count, 1);
///     });
/// });
/// app.update();
/// ```
#[inline(always)]
pub fn run<Sys, I, Out, M>(system: Sys) -> impl ActionSeed<I, Out> + Seed
    where
        Sys: IntoSystem<I, Out, M>,
        I: 'static,
        Out: 'static
{
    OnceSeed::new(system.pipe(|In(out): In<Out>|{
        Some(out)
    }))
}

/// Once run a system with input.
///
/// The return value will be the system return value.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(FlurxPlugin);
/// app.add_systems(Startup, |world: &mut World|{
///     world.schedule_reactor(|task| async move {
///         let count = task.will(Update, once::run_with(1, |In(num): In<u8>, mut count: Local<u8>|{
///             *count += 1;
///             *count + num
///         })).await;
///         assert_eq!(count, 2);
///     });
/// });
/// app.update();
/// ```
#[inline(always)]
pub fn run_with<Sys,Input, Out, Marker>(input:Input, system: Sys) -> impl TaskAction<Input, Out>
    where
        Sys: IntoSystem<Input, Out, Marker>,
        Input: 'static,
        Out: 'static
{
    RunnerIntoAction::new(OnceRunner::new(input, IntoSystem::into_system(system.pipe(|input: In<Out>| {
        Some(input.0)
    }))))
}


