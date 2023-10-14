use std::marker::PhantomData;

use bevy::prelude::{Event, IntoSystem};

use crate::runner::IntoAsyncSystemRunner;
use crate::runner::wait::output::WaitOutput;
use crate::runner::wait::until::Until;

pub mod until;
pub mod output;

pub struct Wait(PhantomData<()>);


impl Wait {
    #[inline(always)]
    pub fn output<Out: Send + 'static, Marker>(system: impl IntoSystem<(), Option<Out>, Marker> + 'static + Send) -> impl IntoAsyncSystemRunner<Out> {
        WaitOutput::create(system)
    }


    #[inline(always)]
    pub fn output_event<E: Event + Clone, Marker>() -> impl IntoAsyncSystemRunner<E> {
        WaitOutput::<E>::event()
    }


    #[inline(always)]
    pub fn until<Marker>(system: impl IntoSystem<(), bool, Marker> + 'static + Send) -> impl IntoAsyncSystemRunner {
        Until::create(system)
    }


    #[inline(always)]
    pub fn until_event<E: Event>() -> impl IntoAsyncSystemRunner {
        Until::event::<E>()
    }
}



