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

pub mod repeat;


pub trait IntoAsyncScheduleCommand<Out = ()>: Sized {
    fn into_schedule_command(self, sender: TaskSender<Out>, schedule_label: impl ScheduleLabel + Clone) -> AsyncScheduleCommand;
}


pub trait AsyncSchedule: Send + Sync {
    fn initialize(self: Box<Self>, entity_commands: &mut EntityCommands, schedules: &mut Schedules);
}


#[derive(Component, Deref, DerefMut)]
pub struct AsyncScheduleCommand(pub Box<dyn AsyncSchedule>);

impl AsyncScheduleCommand {
    #[inline]
    pub fn new(s: impl AsyncSchedule + 'static) -> Self {
        Self(Box::new(s))
    }
}


#[derive(Default, Component, Deref)]
pub(crate) struct AsyncScheduleCommands(Arc<Mutex<Vec<AsyncScheduleCommand>>>);


impl AsyncScheduleCommands {
    #[inline]
    pub(crate) fn push(&self, scheduler: AsyncScheduleCommand) {
        self.0.lock().unwrap().push(scheduler);
    }


    pub(crate) fn init_schedulers(
        &self,
        entity_commands: &mut EntityCommands,
        schedules: &mut Schedules,
    ) {
        while let Some(system) = self.0.lock().unwrap().pop() {
            let entity = entity_commands.commands().spawn_empty().id();
            entity_commands.add_child(entity);
            system.0.initialize(&mut entity_commands.commands().entity(entity), schedules);
        }
    }
}


impl Clone for AsyncScheduleCommands {
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