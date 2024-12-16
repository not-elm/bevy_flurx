use crate::prelude::Flow;
use crate::task::ReactiveTask;
use bevy::app::App;
use bevy::prelude::World;
use std::future::Future;

pub trait SpawnReactor {
    fn spawn_reactor<F>(&mut self, f: fn(ReactiveTask) -> F)
    where
        F: Future + 'static;
}

impl SpawnReactor for World {
    fn spawn_reactor<F>(&mut self, f: fn(ReactiveTask) -> F)
    where
        F: Future + 'static,
    {
        self.spawn(Flow::schedule(f));
    }
}

impl SpawnReactor for App {
    fn spawn_reactor<F>(&mut self, f: fn(ReactiveTask) -> F)
    where
        F: Future + 'static,
    {
        self.world_mut().spawn_reactor(f);
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