use std::marker::PhantomData;

use bevy::prelude::Resource;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq)]
pub struct Switch<M>(bool, PhantomData<M>);

impl<M> Resource for Switch<M>
    where M: Send + Sync + 'static
{}

impl<M> Switch<M> {
    #[inline(always)]
    pub const fn new(switch_on: bool) -> Switch<M>{
        Self(switch_on, PhantomData)
    }
    
    #[inline(always)]
    pub const fn is_on(&self) -> bool {
        self.0
    }

    #[inline(always)]
    pub const fn is_off(&self) -> bool {
        !self.0
    }

    #[inline(always)]
    pub fn on(&mut self) {
        self.0 = true;
    }
    
    #[inline(always)]
    pub fn off(&mut self) {
        self.0 = false;
    }
}

impl<M> Default for Switch<M> {
    fn default() -> Self {
        Self(false, PhantomData)
    }
}
