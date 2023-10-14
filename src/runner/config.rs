use std::marker::PhantomData;

use bevy::prelude::IntoSystem;

pub(crate) struct AsyncSystemConfig<Out, Marker, Sys> {
    pub system: Sys,
    m2: PhantomData<Out>,
    m3: PhantomData<Marker>,
}


impl<Out, Marker, Sys> AsyncSystemConfig<Out, Marker, Sys>
    where Sys: IntoSystem<(), Out, Marker> + 'static + Send
{
    #[inline(always)]
    pub const fn new(system: Sys) -> AsyncSystemConfig<Out, Marker, Sys> {
        Self {
            system,
            m2: PhantomData,
            m3: PhantomData,
        }
    }
}