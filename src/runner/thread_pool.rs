use std::any::Any;
use std::sync::{Arc, Mutex};
use bevy::app::{App, Plugin};

use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::{Component, Deref};
use bevy::utils::HashMap;
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;
use crate::runner::thread_pool::delay::DelayPlugin;

pub mod delay;


pub trait IntoThreadPoolExecutor<Param: SystemParam, Out = ()> {
    fn into_executor(self, sender: Sender<Out>) -> ThreadPoolExecutor<Param>;
}


pub trait ThreadPoolExecutable<Param: SystemParam> {
    fn execute(&mut self, param: &mut StaticSystemParam<Param>) -> AsyncSystemStatus;
}


pub struct ThreadPoolExecutorPlugin;


impl Plugin for ThreadPoolExecutorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DelayPlugin);
    }
}


pub struct ThreadPoolExecutor<Param: SystemParam>(Box<dyn ThreadPoolExecutable<Param>>);


impl<Param> ThreadPoolExecutor<Param>
    where
        Param: SystemParam,
{
    #[inline(always)]
    pub fn new<Exe: ThreadPoolExecutable<Param> + 'static>(executor: Exe) -> ThreadPoolExecutor<Param> {
        Self(Box::new(executor))
    }
}


#[derive(Default, Deref, Component)]
pub(crate) struct MultiThreadSystemExecutors(Arc<Mutex<HashMap<BoxedScheduleLabel, Vec<Box<dyn Any>>>>>);

unsafe impl Send for MultiThreadSystemExecutors {}

unsafe impl Sync for MultiThreadSystemExecutors {}


impl Clone for MultiThreadSystemExecutors {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


impl MultiThreadSystemExecutors {
    #[inline]
    pub(crate) fn insert<Param: SystemParam + 'static>(&self, schedule_label: BoxedScheduleLabel, runner: ThreadPoolExecutor<Param>) {
        let mut map = self.0.lock().unwrap();

        if let Some(runners) = map.get_mut(&schedule_label) {
            runners.push(Box::new(runner));
        } else {
            map.insert(schedule_label, vec![Box::new(runner)]);
        }
    }


    pub(crate) fn run_systems<Param: SystemParam + 'static>(
        &self,
        schedule_label: &dyn ScheduleLabel,
        param: &mut StaticSystemParam<Param>,
    ) {
        let mut map = self.0.lock().unwrap();
        let Some(systems) = map.get_mut(schedule_label) else { return; };
        let mut next_systems = Vec::with_capacity(systems.len());
        while let Some(mut system) = systems.pop() {
            if !system
                .downcast_mut::<ThreadPoolExecutor<Param>>()
                .is_some_and(|executor| executor.0.execute(param).finished())
            {
                next_systems.push(system);
            }
        }
        *systems = next_systems;
    }
}





