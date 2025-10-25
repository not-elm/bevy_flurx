//! `once` creates a task that only once run system.

use crate::action::seed::ActionSeed;
use crate::prelude::RunnerIs;
use crate::runner::{CancellationHandlers, Output, Runner};
pub use _no_op::{no_op, no_op_with_generics};
use bevy::prelude::{IntoSystem, System, SystemIn, SystemInput, World};

#[path = "once/no_op.rs"]
mod _no_op;
#[cfg(feature = "audio")]
#[cfg_attr(docsrs, doc(cfg(feature = "audio")))]
pub mod audio;
pub mod event;
pub mod non_send;
pub mod res;
#[cfg(feature = "state")]
#[cfg_attr(docsrs, doc(cfg(feature = "state")))]
pub mod state;
pub mod switch;

/// Once run a system.
///
/// The return value will be the system return value.
///
/// ## Examples
///
/// ```no_run
/// use bevy::app::AppExit;
/// use bevy::prelude::{World, Update, MessageWriter};
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::run(|mut ew: MessageWriter<AppExit>|{
///         ew.write(AppExit::Success);
///     })).await;
/// });
/// ```
#[inline(always)]
pub fn run<Sys, I, Out, M>(system: Sys) -> ActionSeed<I::Inner<'static>, Out>
where
    Sys: IntoSystem<I, Out, M> + 'static + Send + Sync,
    I: SystemInput + 'static,
    Out: 'static,
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
    Sys: System + 'static,
    Sys::Out:,
{
    fn run(&mut self, world: &mut World, _: &mut CancellationHandlers) -> RunnerIs {
        self.system.initialize(world);
        let Some(input) = self.input.take() else {
            return RunnerIs::Completed;
        };
        let Ok(out) = self.system.run(input, world) else {
            panic!("Failed to run the system in once::run!");
        };
        self.system.apply_deferred(world);
        self.output.set(out);
        RunnerIs::Completed
    }
}
