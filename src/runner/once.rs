// use std::marker::PhantomData;
// use bevy::ecs::system::{StaticSystemParam, SystemParam};
// use bevy::prelude::IntoSystem;
// use crate::prelude::{IntoMainThreadExecutor};
// use crate::runner::main_thread::once::OnceOnMain;
// use crate::runner::{OnMainThread, OnThreadPool};
// use crate::runner::thread_pool::IntoThreadPoolExecutor;
// use crate::runner::thread_pool::once::OnceOnThread;
//
//
// pub struct Once<T, L, K>(PhantomData<T>, PhantomData<L>, PhantomData<K>);
//
//
// impl<Param: SystemParam + 'static, Out: 'static> Once<Param, Out, ()> {
//     #[inline(always)]
//     pub fn run(f: impl Fn(&mut StaticSystemParam<Param>) -> Out + Send + 'static) -> impl IntoThreadPoolExecutor<Param, Out> {
//         OnceOnThread::run(f)
//     }
// }
//
//
//
//
//
// impl<Out: Send + 'static, Marker> Once<(), Out, Marker> {
//     #[inline(always)]
//     pub fn run(system: impl IntoSystem<(), Out, Marker> + Send + 'static) -> impl IntoMainThreadExecutor<Out> {
//         OnceOnMain::run(system)
//     }
// }