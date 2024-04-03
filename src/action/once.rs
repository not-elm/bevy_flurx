//! [`once`] creates a task that only once run system.
//!
//! - [`once::res`](crate::prelude::once::res)
//! - [`once::non_send`](crate::prelude::once::res)
//! - [`once::event`](crate::prelude::once::res)
//! - [`once::state`](crate::prelude::once::res)


use std::marker::PhantomData;
use bevy::prelude::{In, IntoSystem, System};

use crate::prelude::{TaskAction, WithInput};
use crate::runner::{RunTask, TaskOutput};
use crate::runner::once::OnceRunner;

pub mod res;
pub mod non_send;
pub mod event;
pub mod state;


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
pub fn run<Sys, Out, M>(system: Sys) -> impl TaskAction<In=(), Out=Out>
    where
        Sys: IntoSystem<(), Out, M>,
        Out: 'static
{
    OnceAction((), IntoSystem::into_system(system.pipe(|input: In<Out>| {
        Some(input.0)
    })), PhantomData)
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
pub fn run_with<Sys, Input, Out, Marker>(input: Input, system: Sys) -> impl TaskAction<WithInput, In=Input, Out=Out>
    where
        Sys: IntoSystem<Input, Out, Marker>,
        Input: 'static,
        Out: 'static
{
    OnceAction(input, IntoSystem::into_system(system.pipe(|input: In<Out>| {
        Some(input.0)
    })), PhantomData)
}

pub(crate) struct OnceAction<Sys, In, Out>(In, Sys, PhantomData<Out>)
    where In: 'static,
          Sys: System<In=In, Out=Option<Out>>;

impl<Sys, In, Out> TaskAction for OnceAction<Sys, In, Out>
    where In: 'static,
          Out: 'static,
          Sys: System<In=In, Out=Option<Out>>,

{
    type In = In;
    type Out = Out;

    fn to_runner(self, output: TaskOutput<Self::Out>) -> impl RunTask {
        OnceRunner::new(self.1, self.0, output)
    }
}

