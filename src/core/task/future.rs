use crate::core::selector::Selector;
use crate::core::task::StateRef;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

#[pin_project::pin_project]
pub(crate) struct TaskFuture<State: 'static, Selector> {
    pub(crate) selector: Selector,
    pub(crate) state: StateRef<State>,
}

impl<State, Sel> Future for TaskFuture<State, Sel>
where
    Sel: Selector<State>,
    State: Copy + 'static,
{
    type Output = Sel::Output;

    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Some(state) = self.state else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        };
        if let Some(output) = self.as_mut().selector.select(*state) {
            Poll::Ready(output)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
