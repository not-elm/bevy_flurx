//! 
//! 
//! - [`once`]
//! - [`wait`]
//! - [`delay`]

use bevy::prelude::System;

pub mod once;
pub mod wait;
pub mod repeat;
pub mod delay;

#[doc(hidden)]
pub struct WithInput;

#[doc(hidden)]
pub struct WithoutInput;

#[doc(hidden)]
pub trait ReactorAction<Marker = WithInput> {
    type In;

    type Out;

    fn split(self) -> (Self::In, impl System<In=Self::In, Out=Option<Self::Out>>);
}

impl<In, Out, Sys> ReactorAction<WithInput> for (In, Sys)
    where
        Sys: System<In=In, Out=Option<Out>>
{
    type In = In;
    type Out = Out;

    #[inline]
    fn split(self) -> (Self::In, impl System<In=Self::In, Out=Option<Self::Out>>) {
        (self.0, self.1)
    }
}

impl<Out, Sys> ReactorAction<WithoutInput> for Sys
    where
        Sys: System<In=(), Out=Option<Out>>
{
    type In = ();
    type Out = Out;

    #[inline]
    fn split(self) -> (Self::In, impl System<In=Self::In, Out=Option<Self::Out>>) {
        ((), self)
    }
}

#[doc(hidden)]
pub trait WithInputAction<In, Out> {
    fn with(self, input: In) -> impl ReactorAction<WithInput, In=In, Out=Out>;
}

impl<In, Out, Sys> WithInputAction<In, Out> for Sys
    where
        In: Clone + 'static,
        Out: 'static,
        Sys: System<In=In, Out=Option<Out>>,
{
    #[inline]
    fn with(self, input: In) -> impl ReactorAction<WithInput, In=In, Out=Out> {
        with(input, self)
    }
}

#[doc(hidden)]
#[inline]
pub fn with<Sys, Input, Out>(input: Input, system: Sys) -> impl ReactorAction<WithInput, In=Input, Out=Out>
    where
        Sys: System<In=Input, Out=Option<Out>>,
        Input: Clone + 'static,
        Out: 'static
{
    (input, system)
}
