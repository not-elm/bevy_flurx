use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel};
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::{Component, Deref, FromWorld, IntoSystemSetConfigs, Query, Resource, Schedules, World};
use bevy::utils::HashMap;
use futures::channel::mpsc::Sender;

use crate::runner::AsyncSystemStatus;

pub mod delay;
pub(crate) mod once;


pub trait IntoThreadPoolExecutor<Param: SystemParam, Out = ()> {
    fn into_executor(self, sender: Sender<Out>) -> ThreadPoolExecutor<Param>;
}


pub trait ThreadPoolExecutable<Param: SystemParam> {
    fn execute(&mut self, param: &mut StaticSystemParam<Param>) -> AsyncSystemStatus;
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
pub(crate) struct TaskPoolExecutors(Arc<Mutex<HashMap<BoxedScheduleLabel, Vec<Box<dyn Any>>>>>);

unsafe impl Send for TaskPoolExecutors {}

unsafe impl Sync for TaskPoolExecutors {}


impl Clone for TaskPoolExecutors {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


impl TaskPoolExecutors {
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


pub(crate) trait SetupTaskPoolSystem {
    fn initialize_systems(&self, world: &mut World);
}


#[derive(Default, Component)]
pub(crate) struct TaskPoolSystemSetups(Arc<Mutex<Vec<Box<dyn SetupTaskPoolSystem>>>>);

impl TaskPoolSystemSetups {
    #[inline]
    pub fn push<Param: SystemParam + 'static>(&self, schedule_label: impl ScheduleLabel + Clone) {
        self.0.lock().unwrap().push(Box::new(SystemSetup {
            label: schedule_label,
            marker: PhantomData::<Param>,
        }));
    }


    pub fn initialize_systems(&self, world: &mut World) {
        for setup in self.0.lock().unwrap().iter() {
            setup.initialize_systems(world);
        }
    }
}

impl Clone for TaskPoolSystemSetups {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

unsafe impl Send for TaskPoolSystemSetups {}

unsafe impl Sync for TaskPoolSystemSetups {}


pub struct SystemSetup<P: SystemParam, Label: ScheduleLabel + Clone> {
    label: Label,
    marker: PhantomData<P>,
}


impl<P: SystemParam, Label: ScheduleLabel + Clone> Clone for SystemSetup<P, Label> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            marker: PhantomData,
        }
    }
}

impl<S: SystemParam + 'static, Label: ScheduleLabel + Clone> SetupTaskPoolSystem for SystemSetup<S, Label> {
    fn initialize_systems(&self, world: &mut World) {
        if world.contains_non_send::<SystemSetup<S, Label>>() {
            return;
        }

        world.insert_non_send_resource(self.clone());
        let mut schedules = world.resource_mut::<Schedules>();

        if let Some(schedule) = schedules.get_mut(&self.label) {
            let schedule_label = self.label.clone();
            schedule.add_systems(
                move |mut param: StaticSystemParam<S>, executors: Query<&TaskPoolExecutors>| {
                    for executor in executors.iter() {
                        executor.run_systems::<S>(&schedule_label, &mut param);
                    }
                },
            );
        }
    }
}