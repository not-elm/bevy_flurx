use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::core::scheduler::CoreReactor;

#[repr(transparent)]
pub(crate) struct ReactorsFuture<'state, 'future>(pub(crate) &'future mut Option<CoreReactor<'state>>);

impl Future for ReactorsFuture<'_, '_> {
    type Output = ();

    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self
            .0
            .as_mut()
            .map(|reactor|reactor.as_mut().poll(cx).is_ready())
            .unwrap_or(false){
            self.0.take();
        }
 
        Poll::Ready(())
    }
}


