use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::prelude::{Component, Deref, World};
use bevy::utils::HashMap;
use futures::channel::mpsc::Sender;

use crate::runner::main_thread::config::AsyncSystemConfig;

pub mod once;
pub mod wait;
pub mod config;
pub mod repeat;


pub trait IntoAsyncSystemRunner<Out = ()>: Sized {
    fn into_runner(self, sender: Sender<Out>) -> BoxedAsyncSystemRunner;
}


pub trait AsyncSystemRunnable {
    fn run(&mut self, world: &mut World) -> SystemRunningStatus;
}


pub type BoxedAsyncSystemRunner = Box<dyn AsyncSystemRunnable>;


#[derive(Default, Component, Deref)]
pub(crate) struct MainThreadExecutors(Arc<Mutex<HashMap<BoxedScheduleLabel, Vec<BoxedAsyncSystemRunner>>>>);


impl MainThreadExecutors {
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


unsafe impl Send for MainThreadExecutors {}

unsafe impl Sync for MainThreadExecutors {}

impl Clone for MainThreadExecutors {
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
    config: AsyncSystemConfig<Out>,
    status: SystemRunningStatus,
}


impl<Out> BaseRunner<Out>
    where Out: 'static,

{
    fn new(config: AsyncSystemConfig<Out>) -> BaseRunner<Out> {
        Self {
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


