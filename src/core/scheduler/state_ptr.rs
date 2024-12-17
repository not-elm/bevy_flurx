#[repr(transparent)]
#[derive(Default)]
pub(crate) struct StatePtr<State>(Vec<Option<State>>);

impl<State> StatePtr<State> {
    pub(super) const fn uninit() -> StatePtr<State> {
        StatePtr(Vec::new())
    }

    #[inline]
    pub fn set(&mut self, state: State) {
        if let Some(now) = self.0.get_mut(0) {
            *now = Some(state);
        } else {
            self.0.push(Some(state));
        }
    }

    pub(crate) fn state_ref(&mut self) -> &'static Option<State> {
        if self.0.is_empty() {
            self.0.push(None);
        }

        // SAFETY:
        // Lifetime can be longer than the actual validity period.
        // In such cases, the content of Option will be None, and panic will occur when the task uses this value.
        unsafe {
            let ptr = self.0.as_ptr();
            &*ptr
        }
    }
}

impl<State> Drop for StatePtr<State> {
    fn drop(&mut self) {
        if let Some(state) = self.0.get_mut(0) {
            state.take();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::scheduler::state_ptr::StatePtr;

    struct A;

    #[test]
    fn state_ref_come_be_none_after_dropped() {
        let mut ptr = StatePtr::<A>::uninit();
        ptr.set(A);
        let refer = ptr.state_ref();

        assert!(refer.is_some());
        drop(ptr);
        assert!(refer.is_none());
    }
}
