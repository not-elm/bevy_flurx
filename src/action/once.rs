//! [`once`] creates a task that only once run system.
//!
//! - [`once::res`](crate::prelude::once::res)
//! - [`once::non_send`](crate::prelude::once::res)
//! - [`once::event`](crate::prelude::once::res)
//! - [`once::state`](crate::prelude::once::res)


use bevy::prelude::{In, IntoSystem};
use crate::action::seed::{ActionSeed, Seed};
use crate::action::seed::once::OnceSeed;
use crate::prelude::Action;
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
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::{World, Update, EventWriter};
/// use bevy_flurx::prelude::*;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, once::run(|mut ew: EventWriter<AppExit>|{
///         ew.send(AppExit);
///     })).await;
/// });
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
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::{World, Update, EventWriter, In};
/// use bevy_flurx::prelude::*;
///
/// Flurx::schedule(|task| async move{
///     task.will(Update, once::run_with(1, |In(num): In<usize>|{
///         num + 1
///     })).await;
/// });
/// ```
#[inline(always)]
pub fn run_with<Sys,Input, Out, Marker>(input:Input, system: Sys) -> impl Action<Input, Out>
    where
        Sys: IntoSystem<Input, Out, Marker>,
        Input: 'static,
        Out: 'static
{
    RunnerIntoAction::new(OnceRunner::new(input, IntoSystem::into_system(system.pipe(|input: In<Out>| {
        Some(input.0)
    }))))
}


