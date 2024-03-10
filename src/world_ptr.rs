use std::ptr;

use bevy::prelude::World;

pub struct WorldPtr(*mut World);

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



