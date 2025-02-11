//! Convert the operations with side effects such as asynchronous runtime or thread
//! into the referential-transparent actions.

use std::future::Future;

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;
#[cfg(not(target_arch = "wasm32"))]
pub mod thread;
pub mod bevy_task;

/// This trait is implemented for functions that return future or future.
pub trait AsyncFunctor<I, Out, M> {
    /// Returns a new future with input.
    /// 
    /// If you have added the `tokio` feature flag, future will be automatically compat.
    fn functor(self, input: I) -> impl Future<Output=Out> + Send;
}

impl<I, F, Fut> AsyncFunctor<I, <Fut as Future>::Output, ()> for F
    where
        I: Send + 'static,
        F: FnOnce(I) -> Fut + Send + 'static,
        Fut: Future + Send + 'static,
        <Fut as Future>::Output: Send + 'static
{
    #[inline]
    fn functor(self, input: I) -> impl Future<Output=<Fut as Future>::Output> + Send {
        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        {
            use async_compat::CompatExt;
            self(input).compat()
        }
        #[cfg(any(target_arch = "wasm32", not(feature = "tokio")))]
        {
            self(input)
        }
    }
}

impl<I, Fut> AsyncFunctor<I, <Fut as Future>::Output, bool> for Fut
    where
        I: Send + 'static,
        Fut: Future + Send + 'static,
        <Fut as Future>::Output: Send + 'static
{
    #[inline]
    fn functor(self, _: I) -> impl Future<Output=<Fut as Future>::Output> + Send {
        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        {
            use async_compat::CompatExt;
            self.compat()
        }
        #[cfg(any(target_arch = "wasm32", not(feature = "tokio")))]
        {
            self
        }
    }
}

/// This tray is used in the action argument and does not need to be implemented explicitly by the user
pub trait Functor<I, O, M>{
    /// Returns a new function input.
    fn functor(self, input: I) -> impl FnOnce() -> O + Send + 'static;
}

impl<I, O, F> Functor<I, O, ()> for F
where
    I: Send + 'static,
    F: FnOnce(I) -> O + Send + 'static,
{
    #[inline]
    fn functor(self, input: I) -> impl FnOnce() -> O + Send + 'static {
        move ||{
            self(input)
        }
    }
}

impl<O, F> Functor<(), O, bool> for F
where
    F: FnOnce() -> O + Send + 'static,
{
    #[inline]
    fn functor(self, _input: ()) -> impl FnOnce() -> O + Send + 'static {
        ||{
            self()
        }
    }
}