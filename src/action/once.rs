//! `once` creates a task that only once run system.
//!
//! actions
//!
//! - [`once::run`](run)
//! - [`once::no_op`](no_op)
//! - [`once::no_op_with_generics`](no_op_with_generics)
//! - [`once::res`](res)
//! - [`once::non_send`](res)
//! - [`once::event`](res)
//! - [`once::state`](res)
//! - [`once::switch`](switch)
//! - [`once::audio`](audio) (require feature flag `audio`)

use crate::action::seed::ActionSeed;
use crate::prelude::RunnerStatus;
use crate::runner::{CancellationToken, Output, Runner};
pub use _no_op::{no_op, no_op_with_generics};
use bevy::prelude::{IntoSystem, System, SystemIn, SystemInput, World};

pub mod event;
pub mod non_send;
pub mod res;
pub mod switch;
#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "state")]
pub mod state;
#[path = "once/no_op.rs"]
mod _no_op;

/// Once run a system.
///
/// The return value will be the system return value.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::{World, Update, EventWriter};
/// use bevy_flurx::prelude::*;
///
/// crate::prelude::Flow::schedule(|task| async move{
///     task.will(Update, once::run(|mut ew: EventWriter<AppExit>|{
///         ew.send(AppExit::Success);
///     })).await;
/// });
/// ```
#[inline(always)]
pub fn run<Sys, I, Out, M>(system: Sys) -> ActionSeed<I::Inner<'static>, Out>
where
    Sys: IntoSystem<I, Out, M> + 'static + Send + Sync,
    I: SystemInput + 'static,
    Out:  'static,
{
    ActionSeed::new(move |input, output| OnceRunner {
        input: Some(input),
        output,
        system: IntoSystem::into_system(system),
    })
}

struct OnceRunner<Sys>
where
    Sys: System,
{
    system: Sys,
    input: Option<SystemIn<'static, Sys>>,
    output: Output<Sys::Out>,
}

impl<Sys> Runner for OnceRunner<Sys>
where
    Sys: System  + 'static,
    Sys::Out: ,
{
    fn run(&mut self, world: &mut World, _: &mut CancellationToken) -> RunnerStatus {
        self.system.initialize(world);
        let Some(input) = self.input.take() else {
            return RunnerStatus::Ready;
        };
        let out = self.system.run(input, world);
        self.system.apply_deferred(world);
        self.output.set(out);
        RunnerStatus::Ready
    }
}
