use std::marker::PhantomData;

use crate::action::Action;
use crate::runner::{CancellationToken, TaskOutput, TaskRunner};

pub struct BaseTwoRunner<I1, I2, O1, O2> {
    pub(crate) r1: Box<dyn TaskRunner>,
    pub(crate) r2: Box<dyn TaskRunner>,
    pub(crate) o1: TaskOutput<O1>,
    pub(crate) o2: TaskOutput<O2>,
    pub(crate) t1: CancellationToken,
    pub(crate) t2: CancellationToken,
    _m: PhantomData<(I1, I2)>,
}

impl<I1, I2, O1, O2> BaseTwoRunner<I1, I2, O1, O2>
    where
        I1: 'static,
        I2: 'static,
        O1: 'static,
        O2: 'static,
{
    #[inline]
    pub fn new(
        a1: impl Action<I1, O1> + 'static,
        a2: impl Action<I2, O2> + 'static,
    ) -> BaseTwoRunner<I1, I2, O1, O2>
    {
        let o1 = TaskOutput::default();
        let o2 = TaskOutput::default();
        let t1 = CancellationToken::default();
        let t2 = CancellationToken::default();
        let r1 = a1.to_runner(t1.clone(), o1.clone());
        let r2 = a2.to_runner(t2.clone(), o2.clone());
        Self {
            r1: Box::new(r1),
            r2: Box::new(r2),
            o1,
            o2,
            t1,
            t2,
            _m: PhantomData,
        }
    }

    #[inline]
    pub fn cancel_if_need(&mut self, token: &CancellationToken) -> bool {
        if token.requested_cancel() {
            self.t1.cancel();
            self.t2.cancel();
            true
        } else {
            false
        }
    }
}