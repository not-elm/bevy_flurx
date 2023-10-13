use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::BoxedScheduleLabel;
use bevy::prelude::{Deref, World};
use bevy::utils::HashMap;
use futures::channel::mpsc::{Receiver, Sender};

use crate::runner::config::AsyncSystemConfig;

pub mod delay;
pub mod once;
pub mod until;
pub mod wait;
pub mod config;
pub mod send;
pub mod repeat;


pub trait AsyncSystem<Out>: Sized {
    fn split(self) -> (BoxedAsyncSystemRunner, BoxedTaskFuture<Out>);
}

pub trait AsyncSystemRunnable {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus;
}


pub type BoxedAsyncSystemRunner = Box<dyn AsyncSystemRunnable>;
pub type BoxedTaskFuture<Out> = Pin<Box<dyn Future<Output=Out> + Send>>;

#[derive(Default, Deref)]
pub struct Runners(Arc<Mutex<HashMap<BoxedScheduleLabel, Vec<BoxedAsyncSystemRunner>>>>);


impl Runners {
    #[inline]
    pub(crate) fn insert(&self, schedule_label: BoxedScheduleLabel, runner: BoxedAsyncSystemRunner) {
        let mut map = self.0.lock().unwrap();

        if let Some(runners) = map.get_mut(&schedule_label) {
            runners.push(runner);
        } else {
            map.insert(schedule_label, vec![runner]);
        }
    }
}

impl Clone for Runners {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SystemRunningStatus {
    #[default]
    NoInitialized,
    Running,
    Finished,
}


impl SystemRunningStatus {
    #[inline]
    pub fn no_initialized(&self) -> bool {
        matches!(self, SystemRunningStatus::NoInitialized)
    }


    #[inline]
    pub fn finished(&self) -> bool {
        matches!(self, SystemRunningStatus::Finished)
    }
}


struct BaseRunner<In = (), Out = ()> {
    tx: Sender<Out>,
    config: AsyncSystemConfig<In, Out>,
    status: SystemRunningStatus,
}


impl<In, Out> BaseRunner<In, Out>
    where Out: 'static,
          In: Clone + 'static
{
    fn new(
        tx: Sender<Out>,
        config: AsyncSystemConfig<In, Out>,
    ) -> BaseRunner<In, Out> {
        Self {
            tx,
            config,
            status: SystemRunningStatus::NoInitialized,
        }
    }


    fn run_with_output(&mut self, world: &mut World) -> Out {
        if self.status.no_initialized() {
            self.config.system.initialize(world);
            self.status = SystemRunningStatus::Running;
        }

        let output = self.config.system.run(self.config.input.clone(), world);
        self.config.system.apply_deferred(world);
        output
    }
}


#[inline]
fn new_channel<Out>(size: usize) -> (Sender<Out>, Receiver<Out>) {
    futures::channel::mpsc::channel(size)
}
