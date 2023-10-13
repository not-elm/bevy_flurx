use std::marker::PhantomData;

use bevy::prelude::{Event, IntoSystem, World};

use crate::runner::{AsyncSystemRunnable, BaseRunner, IntoAsyncSystem, SystemRunningStatus};
use crate::runner::wait::output::WaitOutput;
use crate::runner::wait::until::Until;

pub mod until;
pub mod output;

pub struct Wait(PhantomData<()>);


impl Wait {
    #[inline(always)]
    pub fn output<Out: Send + 'static, Marker>(system: impl IntoSystem<(), Option<Out>, Marker> + 'static + Send) -> impl IntoAsyncSystem<Out> {
        WaitOutput::create(system)
    }


    #[inline(always)]
    pub fn output_event<E: Event + Clone, Marker>() -> impl IntoAsyncSystem<E> {
        WaitOutput::<(), E>::event()
    }


    #[inline(always)]
    pub fn until<Marker>(system: impl IntoSystem<(), bool, Marker> + 'static + Send) -> impl IntoAsyncSystem {
        Until::create(system)
    }


    #[inline(always)]
    pub fn until_event<E: Event>() -> impl IntoAsyncSystem {
        Until::event::<E>()
    }
}


struct WaitRunner<In, Out>(BaseRunner<In, Option<Out>>);

impl<In, Out> AsyncSystemRunnable for WaitRunner<In, Out>
    where In: 'static + Clone,
          Out: 'static + Send
{
    fn run(&mut self, world: &mut World) -> SystemRunningStatus {
        if let Some(output) = self.0.run_with_output(world) {
            let _ = self.0.tx.try_send(Some(output));
            SystemRunningStatus::Finished
        } else {
            SystemRunningStatus::Running
        }
    }
}
