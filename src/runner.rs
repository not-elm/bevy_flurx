use std::sync::{Arc, Mutex};

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::EntityCommands;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{Component, Condition, Deref, DerefMut, Entity, IntoSystem, Query, Schedule, Schedules};

use crate::async_commands::TaskSender;

pub mod once;
pub mod wait;
pub mod config;
pub mod delay;

// pub mod repeat;


pub trait IntoMainThreadExecutor<Out = ()>: Sized {
    fn into_executor(self, sender: TaskSender<Out>, schedule_label: impl ScheduleLabel + Clone) -> BoxedMainThreadExecutor;
}


pub trait MainThreadExecutable: Send + Sync {
    fn schedule_initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules);
}


#[derive(Component, Deref, DerefMut)]
pub struct BoxedMainThreadExecutor(pub Box<dyn MainThreadExecutable>);

impl BoxedMainThreadExecutor {
    #[inline]
    pub fn new(s: impl MainThreadExecutable + 'static) -> Self {
        Self(Box::new(s))
    }
}


#[derive(Default, Component, Deref)]
pub(crate) struct MainThreadExecutors(Arc<Mutex<Vec<BoxedMainThreadExecutor>>>);


impl MainThreadExecutors {
    #[inline]
    pub(crate) fn push(&self, runner: BoxedMainThreadExecutor) {
        self.0.lock().unwrap().push(runner);
    }


    pub(crate) fn run_systems(
        &self,
        entity_commands: &mut EntityCommands,
        schedule: &mut Schedules,
    ) {
        let mut executables = self.0.lock().unwrap();
        while let Some(system) = executables.pop() {
            let entity = entity_commands.commands().spawn_empty().id();
            entity_commands.add_child(entity);
            system.0.schedule_initialize(&mut entity_commands.commands().entity(entity), schedule);
        }
    }
}


impl Clone for MainThreadExecutors {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


fn task_running<Out>(entity: Entity) -> impl Condition<()>
    where
        Out: Send + 'static,
{
    IntoSystem::into_system(move |senders: Query<&TaskSender<Out>>| {
        senders
            .get(entity)
            .is_ok_and(|sender| !sender.is_closed())
    })
}


fn schedule_initialize<'a, Label: ScheduleLabel + Clone>(schedules: &'a mut Schedules, schedule_label: &Label) -> &'a mut Schedule {
    if !schedules.contains(schedule_label) {
        schedules.insert(schedule_label.clone(), Schedule::default());
    }

    schedules.get_mut(schedule_label).unwrap()
}