use crate::core::selector::Selector;
use crate::core::task::StateRef;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub(in crate) struct TaskFuture<State: 'static, Selector, const SAFETY: bool> {
    pub(in crate) selector: Selector,
    pub(in crate) state: StateRef<State>,
}

impl<State, Sel> Future for TaskFuture<State, Sel, false>
where
    Sel: Selector<State>,
    State: Copy + 'static,
{
    type Output = Sel::Output;

    #[allow(clippy::panic)]
    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(output) = self.selector.select(self.state.unwrap()) {
            Poll::Ready(output)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
