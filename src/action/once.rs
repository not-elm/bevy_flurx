//! `once` creates a task that only once run system.
//!
//! actions
//!
//! - [`once::res`](crate::prelude::once::res)
//! - [`once::non_send`](crate::prelude::once::res)
//! - [`once::event`](crate::prelude::once::res)
//! - [`once::state`](crate::prelude::once::res)
//! - [`once::switch`](crate::prelude::once::switch)
//! - [`once::audio`](crate::prelude::once::audio) (require feature flag `audio`)

use bevy::prelude::{IntoSystem, System, SystemIn, SystemInput, World};

use crate::action::seed::ActionSeed;
use crate::runner::{CancellationToken, Output, Runner};

#[cfg(feature = "audio")]
pub mod audio;
pub mod event;
pub mod non_send;
pub mod res;
#[cfg(feature = "state")]
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
/// use bevy::prelude::{World, Update, EventWriter};
/// use bevy_flurx::prelude::*;
///
/// Reactor::schedule(|task| async move{
///     task.will(Update, once::run(|mut ew: EventWriter<AppExit>|{
///         ew.send(AppExit::Success);
///     })).await;
/// });
/// ```
#[inline(always)]
pub fn run<Sys, I, Out, M>(system: Sys) -> ActionSeed<I::Inner<'static>, Out>
where
    Sys: IntoSystem<I, Out, M> + 'static,
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
    Sys: System,
{
    fn run(&mut self, world: &mut World, _: &CancellationToken) -> bool {
        self.system.initialize(world);
        let Some(input) = self.input.take() else {
            return true;
        };
        let out = self.system.run(input, world);
        self.system.apply_deferred(world);
        self.output.set(out);
        true
    }
}
