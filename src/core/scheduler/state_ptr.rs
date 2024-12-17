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
        // if let Some(state) = self.0.get_mut(0) {
        //     state.take();
        // }
        self.0.take();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::scheduler::state_ptr::StatePtr;

    struct A;

    #[test]
    fn state_ref_come_be_none_after_dropped() {
        let mut ptr = StatePtr(Box::new(None));
        ptr.0.replace(A);
        let refer = ptr.state_ref();

        assert!(refer.is_some());
        drop(ptr);
        assert!(refer.is_none());
    }
}
