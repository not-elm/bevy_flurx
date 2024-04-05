use std::marker::PhantomData;

use bevy::prelude::System;

use crate::action::seed::{ActionSeed, Seed};
use crate::action::TaskAction;
use crate::private::RunnerIntoAction;
use crate::runner::once::OnceRunner;

pub struct OnceSeed<In, Out, Sys> {
    system: Sys,
    _m: PhantomData<(In, Out)>,
}

impl<In, Out, Sys> Seed for OnceSeed<In, Out, Sys>{

}


impl<In, Out, Sys> OnceSeed<In, Out, Sys> {
    #[inline]
    pub const fn new(system: Sys) -> OnceSeed<In, Out, Sys> {
        Self {
            system,
            _m: PhantomData,
        }
    }
}

impl<In, Out, Sys> ActionSeed<In, Out> for OnceSeed<In, Out, Sys>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: 'static,
        Out: 'static
{
    #[inline]
    fn into_action(self, input: In) -> impl TaskAction< In, Out> {
        RunnerIntoAction::new(OnceRunner::new(input, self.system))
    }
}

