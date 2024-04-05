use std::marker::PhantomData;

use bevy::prelude::System;

use crate::action::{ TaskAction};
use crate::action::seed::{ActionSeed, Seed};
use crate::private::RunnerIntoAction;
use crate::runner::multi_times::MultiTimesRunner;

pub struct WaitSeed<In, Out, Sys> {
    system: Sys,
    _m: PhantomData<(In, Out)>,
}

impl<In, Out, Sys> WaitSeed<In, Out, Sys>
    where
        Sys: System<In=In, Out=Option<Out>>
{
    #[inline]
    pub const fn new(system: Sys) -> WaitSeed<In, Out, Sys> {
        Self {
            system,
            _m: PhantomData,
        }
    }
}


impl<In, Out, Sys> Seed for WaitSeed<In, Out, Sys>{}


impl<In, Out, Sys> ActionSeed<In, Out> for WaitSeed<In, Out, Sys>
    where
        Sys: System<In=In, Out=Option<Out>>,
        In: Clone + 'static,
        Out: 'static
{
    #[inline]
    fn into_action(self, input: In) -> impl TaskAction< In, Out> {
        RunnerIntoAction::new(MultiTimesRunner::new(self.system, input))
    }
}
