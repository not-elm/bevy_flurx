use bevy::prelude::System;

pub mod once;
pub mod wait;
pub mod repeat;
pub mod delay;


pub struct WithInput;

pub struct WithoutInput;

pub trait ReactorSystemConfigs<Marker = WithInput> {
    type In;

    type Out;

    fn into_configs(self) -> (Self::In, impl System<In=Self::In, Out=Option<Self::Out>>);
}

impl<In, Out, Sys> ReactorSystemConfigs<WithInput> for (In, Sys)
    where
        Sys: System<In=In, Out=Option<Out>>
{
    type In = In;
    type Out = Out;

    fn into_configs(self) -> (Self::In, impl System<In=Self::In, Out=Option<Self::Out>>) {
        (self.0, self.1)
    }
}


impl<Out, Sys> ReactorSystemConfigs<WithoutInput> for Sys
    where
        Sys: System<In=(), Out=Option<Out>>
{
    type In = ();
    type Out = Out;

    fn into_configs(self) -> (Self::In, impl System<In=Self::In, Out=Option<Self::Out>>) {
        ((), self)
    }
}


pub trait WithInputSystem<In, Out> {
    fn with(self, input: In) -> impl ReactorSystemConfigs<WithInput, In=In, Out=Out>;
}


impl<In, Out, Sys> WithInputSystem<In, Out> for Sys
    where
        In: Clone + 'static,
        Out: 'static,
        Sys: System<In=In, Out=Option<Out>>,
{
    #[inline]
    fn with(self, input: In) -> impl ReactorSystemConfigs<WithInput, In=In, Out=Out> {
        with(input, self)
    }
}

#[inline]
pub fn with<Sys, Input, Out>(input: Input, system: Sys) -> impl ReactorSystemConfigs<WithInput, In=Input, Out=Out>
    where
        Sys: System<In=Input, Out=Option<Out>>,
        Input: Clone + 'static,
        Out: 'static
{
    (input, system)
}




