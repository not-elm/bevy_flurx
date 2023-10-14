use std::marker::PhantomData;

use bevy::prelude::{Event, IntoSystem};

use crate::runner::main_thread::IntoMainThreadExecutor;
use crate::runner::main_thread::wait::output::WaitOutput;
use crate::runner::main_thread::wait::until::Until;

pub mod until;
pub mod output;

pub struct Wait(PhantomData<()>);


impl Wait {
    #[inline(always)]
    pub fn output<Out: Send + 'static, Marker>(system: impl IntoSystem<(), Option<Out>, Marker> + 'static + Send) -> impl IntoMainThreadExecutor<Out> {
        WaitOutput::create(system)
    }


    #[inline(always)]
    pub fn output_event<E: Event + Clone, Marker>() -> impl IntoMainThreadExecutor<E> {
        WaitOutput::<E>::event()
    }


    #[inline(always)]
    pub fn until<Marker>(system: impl IntoSystem<(), bool, Marker> + 'static + Send) -> impl IntoMainThreadExecutor {
        Until::create(system)
    }


    #[inline(always)]
    pub fn until_event<E: Event>() -> impl IntoMainThreadExecutor {
        Until::event::<E>()
    }
}



