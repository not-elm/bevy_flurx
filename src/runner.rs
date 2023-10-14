use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::prelude::{Component, Deref, World};
use bevy::tasks::Task;
use bevy::utils::HashMap;
use futures::channel::mpsc::{Receiver, Sender};

use crate::runner::config::AsyncSystemConfig;

pub mod delay;
pub mod once;
pub mod wait;
pub mod config;
pub mod repeat;


pub trait IntoAsyncSystem<Out = ()>: Sized {
    fn into_parts(self) -> (BoxedAsyncSystemRunner, Task<Out>);
}

pub trait AsyncSystemRunnable {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus;
}


pub type BoxedAsyncSystemRunner = Box<dyn AsyncSystemRunnable>;


#[derive(Default, Component, Deref)]
pub(crate) struct Runners(Arc<Mutex<HashMap<BoxedScheduleLabel, Vec<BoxedAsyncSystemRunner>>>>);


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


    pub(crate) fn run_systems(
        &self,
        schedule_label: &dyn ScheduleLabel,
        world: &mut World,
    ) {
        let mut map = self.0.lock().unwrap();
        let Some(systems) = map.get_mut(schedule_label) else { return; };
        let mut next_systems = Vec::with_capacity(systems.len());
        while let Some(mut system) = systems.pop() {
            if !system.run(world).finished() {
                next_systems.push(system);
            }
        }
        *systems = next_systems;
    }
}


unsafe impl Send for Runners {}

unsafe impl Sync for Runners {}

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


struct BaseRunner<Out = ()> {
    tx: Sender<Out>,
    config: AsyncSystemConfig<Out>,
    status: SystemRunningStatus,
}


impl<Out> BaseRunner<Out>
    where Out: 'static,

{
    fn new(
        tx: Sender<Out>,
        config: AsyncSystemConfig<Out>,
    ) -> BaseRunner< Out> {
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

        let output = self.config.system.run((), world);
        self.config.system.apply_deferred(world);
        output
    }
}


#[inline]
fn new_channel<Out>(size: usize) -> (Sender<Out>, Receiver<Out>) {
    futures::channel::mpsc::channel(size)
}
