use bevy::prelude::*;

#[repr(transparent)]
#[derive(Default)]
pub(crate) struct StatePtr<State>(pub(crate) Box<Option<State>>);

impl<State> StatePtr<State> {
    pub(crate) fn state_ref(&mut self) -> &'static Option<State> {
        // SAFETY:
        // Lifetime can be longer than the actual validity period.
        // In such cases, the content of Option will be None, and panic will occur when the task uses this value.
        unsafe {
            let ptr = &*self.0 as *const Option<State>;
            &*ptr
        }
    }
}

impl<State> Drop for StatePtr<State> {
    fn drop(&mut self) {
        self.0.take();
    }
}
