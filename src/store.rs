use std::ptr;

use bevy::prelude::World;

pub(crate) struct WorldPointer(*mut World);

impl WorldPointer {
    pub(crate) fn new(value: &mut World) -> Self {
        Self(value as *mut World)
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn as_mut(&self) -> &mut World {
        unsafe { &mut *self.0 }
    }
}


impl Default for WorldPointer {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}



