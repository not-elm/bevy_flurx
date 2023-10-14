use bevy::app::App;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::Query;

use crate::runner::multi_thread::MultiThreadSystemExecutors;

pub trait AddSystem {
    fn add_thread_pool_system<S: SystemParam + 'static>(&mut self, schedule: impl ScheduleLabel + Clone) -> &mut Self;

    fn add_thread_pool_system_on_main_scheduler<S: SystemParam + 'static>(&mut self) -> &mut Self {
        #[cfg(feature = "first")]
        { self.add_thread_pool_system::<S>(bevy::app::First); }

        #[cfg(feature = "pre_update")]
        { self.add_thread_pool_system::<S>(bevy::app::PreUpdate); }

        #[cfg(feature = "state_transition")]
        { self.add_thread_pool_system::<S>(bevy::app::StateTransition); }

        #[cfg(feature = "fixed_update")]
        {
            self
                .add_thread_pool_system::<S>(bevy::app::RunFixedUpdateLoop)
                .add_thread_pool_system::<S>(bevy::app::FixedUpdate);
        }

        #[cfg(feature = "update")]
        { self.add_thread_pool_system::<S>(bevy::app::Update); }

        #[cfg(feature = "post_update")]
        { self.add_thread_pool_system::<S>(bevy::app::PostUpdate); }

        #[cfg(feature = "last")]
        { self.add_thread_pool_system::<S>(bevy::app::Last); }

        self
    }
}


impl AddSystem for App {
    fn add_thread_pool_system<S: SystemParam + 'static>(&mut self, schedule: impl ScheduleLabel + Clone) -> &mut Self {
        self.add_systems(
            schedule.clone(),
            move |mut param: StaticSystemParam<S>, executors: Query<&MultiThreadSystemExecutors>| {
                let schedule = schedule.clone();
                for executor in executors.iter() {
                    executor.run_systems::<S>(&schedule, &mut param);
                }
            },
        )
    }
}





