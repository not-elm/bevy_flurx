use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::SystemId;
use bevy::prelude::{Schedule, Schedules, World};

use crate::selector::runner::runners::ReactorRunners;

pub(super) mod runners;
pub(super) mod standard;


pub(crate) trait RunReactor {
    fn run(&self, world: &mut World) -> bool;
}


#[repr(transparent)]
pub(crate) struct ReactorSystemOutput<In, Out>(Vec<(SystemId<In, Option<Out>>, Out)>);


impl<In, Out> ReactorSystemOutput<In, Out> {
    fn push(&mut self, id: SystemId<In, Option<Out>>, output: Out) {
        self.0.push((id, output));
    }

    pub fn extract_output(&mut self, id: &SystemId<In, Option<Out>>) -> Option<Out> {
        if let Some(pos) = self.0.iter().position(|(system_id, _)| system_id == id) {
            Some(self.0.remove(pos).1)
        } else {
            None
        }
    }
}

impl<In, Out> Default for ReactorSystemOutput<In, Out> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub(crate) fn initialize_reactor_runner<Label>(
    world: &mut World,
    label: Label,
    runner: impl RunReactor + 'static,
)
    where Label: ScheduleLabel + Clone
{
    if let Some(mut reactor) = world.get_non_send_resource_mut::<ReactorRunners<Label>>() {
        reactor.systems.push(Box::new(runner));
    } else {
        let Some(mut schedules) = world.get_resource_mut::<Schedules>() else {
            return;
        };

        let schedule = initialize_schedule(&mut schedules, label);
        schedule.add_systems(run_reactors::<Label>);

        let mut reactor = ReactorRunners::<Label>::default();
        reactor.systems.push(Box::new(runner));
        world.insert_non_send_resource(reactor);
    }
}

fn initialize_schedule<Label>(schedules: &mut Schedules, schedule_label: Label) -> &mut Schedule
    where Label: ScheduleLabel + Clone
{
    if !schedules.contains(schedule_label.clone()) {
        schedules.insert(Schedule::new(schedule_label.clone()));
    }

    schedules.get_mut(schedule_label.intern()).unwrap()
}

fn run_reactors<Label: ScheduleLabel>(world: &mut World) {
    let Some(mut runner) = world.remove_non_send_resource::<ReactorRunners<Label>>() else {
        return;
    };
    if !runner.run(world) {
        world.insert_non_send_resource(runner)
    }
}


