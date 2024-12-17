use crate::prelude::Flow;
use crate::task::ReactiveTask;
use bevy::app::{App, Startup};
use bevy::prelude::Commands;
use std::future::Future;

pub trait SpawnReactor {
    fn spawn_reactor<F>(&mut self, f: fn(ReactiveTask) -> F)
    where
        F: Future + 'static;
}

impl SpawnReactor for App {
    fn spawn_reactor<F>(&mut self, f: fn(ReactiveTask) -> F)
    where
        F: Future + 'static,
    {
        self.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(Flow::schedule(f));
        });
    }
}

pub mod test {
    use bevy::prelude::World;

    use crate::prelude::{ActionSeed, CancellationToken, Runner};

    pub fn cancel() -> ActionSeed {
        ActionSeed::new(|_, _| {
            TestCancelRunner
        })
    }

    struct TestCancelRunner;

    impl Runner for TestCancelRunner {
        fn run(&mut self, _: &mut World, _: &mut CancellationToken) -> crate::prelude::RunnerStatus {
            crate::prelude::RunnerStatus::Cancel
        }
    }
}