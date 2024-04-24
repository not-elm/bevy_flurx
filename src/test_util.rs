use std::future::Future;

use bevy::app::App;
use bevy::prelude::World;

use crate::prelude::Reactor;
use crate::task::ReactiveTask;

pub trait SpawnReactor {
    fn spawn_reactor<F>(&mut self, f: impl FnOnce(ReactiveTask) -> F + 'static)
        where
            F: Future;
}

impl SpawnReactor for World {
    fn spawn_reactor<F>(&mut self, f: impl FnOnce(ReactiveTask) -> F + 'static) where F: Future {
        self.spawn(Reactor::schedule(f));
    }
}

impl SpawnReactor for App {
    fn spawn_reactor<F>(&mut self, f: impl FnOnce(ReactiveTask) -> F + 'static) where F: Future {
        self.world.spawn_reactor(f);
    }
}