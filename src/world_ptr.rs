use bevy::prelude::World;
use std::ptr;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct WorldPtr(*mut World);

unsafe impl Send for WorldPtr {}
unsafe impl Sync for WorldPtr {}

impl WorldPtr {
    #[inline]
    pub(crate) fn new(value: &mut World) -> Self {
        Self(value as *mut World)
    }

    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub(crate) fn as_mut(&self) -> &mut World {
        // SAFETY:
        // The world is guaranteed to be alive from the time this structure is created until the reactor is executed, 
        // and the reactor and tasks run on the main thread, so it is probably safe.
        unsafe { &mut *self.0 }
    }
}


impl Default for WorldPtr {
    #[inline]
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}



