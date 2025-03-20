use bevy::app::{App, Last, Plugin};
use bevy::ecs::schedule::InternedScheduleLabel;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::{on_event, Event, EventReader, IntoScheduleConfigs, ResMut, Schedules};
use itertools::Itertools;

/// When the schedule to be registered is the same as the schedule currently being executed by the [`BoxedRunner`](crate::prelude::BoxedRunner),
/// it cannot be registered normally, so it is temporarily stored and registered with [`Last`].
pub struct ReserveRegisterRunnerPlugin;

impl Plugin for ReserveRegisterRunnerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ReservedRunner>()
            .add_systems(Last, register_runner_system.run_if(on_event::<ReservedRunner>));
    }
}

#[derive(Event, Debug)]
pub(crate) struct ReservedRunner {
    pub label: InternedScheduleLabel,
    pub system: fn() -> BoxedSystem<(), bevy::prelude::Result>,
}

fn register_runner_system(
    mut events: EventReader<ReservedRunner>,
    mut schedules: ResMut<Schedules>,
) {
    for event in events.read().unique_by(|e| e.label) {
        schedules.add_systems(event.label, (event.system)());
    }
}