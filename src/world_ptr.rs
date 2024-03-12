use std::ptr;

use bevy::prelude::World;
use pin_project::pin_project;


#[pin_project]
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct WorldPtr(#[pin] *mut World);

impl WorldPtr {
    pub(crate) fn new(value: &mut World) -> Self {
        Self(value as *mut World)
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn as_mut(&self) -> &mut World {
        unsafe { &mut *self.0 }
    }
}


impl Default for WorldPtr {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}



