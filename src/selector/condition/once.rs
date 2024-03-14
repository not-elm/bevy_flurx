use bevy::prelude::{In, IntoSystem, System};

use crate::prelude::{ReactorSystemConfigs, with, WithInput};

pub mod res;
pub mod event;
pub mod state;
pub mod non_send;



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
#[inline]
pub fn run<Sys, Input, Out, Marker>(system: Sys) -> impl System<In=Input, Out=Option<Out>>
    where
        Sys: IntoSystem<Input, Out, Marker>,
        Input: 'static,
        Out: 'static
{
    IntoSystem::into_system(system.pipe(|input: In<Out>| {
        Some(input.0)
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
#[inline]
pub fn run_with<Sys, Input, Out, Marker>(input: Input, system: Sys) -> impl ReactorSystemConfigs<WithInput, In=Input, Out=Out>
    where
        Sys: IntoSystem<Input, Out, Marker>,
        Input: Clone + 'static,
        Out: 'static
{
    with(input, run(system))
}

