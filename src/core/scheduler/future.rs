use crate::core::scheduler::CoreReactor;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub(crate) struct ReactorsFuture<'scheduler, 'state, 'future> {
    pub(crate) finished: &'scheduler mut bool,
    pub(crate) reactor: &'future mut CoreReactor<'state>,
}

impl Future for ReactorsFuture<'_, '_, '_> {
    type Output = ();

    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self
            .reactor
            .as_mut()
            .poll(cx)
            .is_ready()
        {
            *self.finished = true;
        }

        Poll::Ready(())
    }
}


