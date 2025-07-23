use crate::prelude::Reactor;
use crate::task::ReactorTask;
use bevy::app::{App, Startup};
use bevy::prelude::Commands;
use core::future::Future;

#[allow(unused)]
pub trait SpawnReactor {
    fn spawn_reactor<F>(&mut self, f: fn(ReactorTask) -> F)
    where
        F: Future + Send + Sync + 'static;
}

impl SpawnReactor for App {
    fn spawn_reactor<F>(&mut self, f: fn(ReactorTask) -> F)
    where
        F: Future + Send + Sync + 'static,
    {
        self.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(Reactor::schedule(f));
        });
    }
}

pub mod test {
    use bevy::prelude::World;

    use crate::prelude::{ActionSeed, CancellationHandlers, Runner};

    pub fn cancel() -> ActionSeed {
        ActionSeed::new(|_, _| TestCancelRunner)
    }

    struct TestCancelRunner;

    impl Runner for TestCancelRunner {
        fn run(&mut self, _: &mut World, _: &mut CancellationHandlers) -> crate::prelude::RunnerIs {
            crate::prelude::RunnerIs::Canceled
        }
    }
}
