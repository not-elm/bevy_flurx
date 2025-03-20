//! Measures the performance of continuously repeating an action.
#![allow(missing_docs)]

use bevy::app::{App, Startup};
use bevy::prelude::{Commands, Resource, TaskPoolPlugin, Update};
use bevy_flurx::action::{delay, once};
use bevy_flurx::prelude::{wait, Reactor};
use bevy_flurx::FlurxPlugin;
use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Resource, Default)]
struct Exit(bool);

fn with_flurx(c: &mut Criterion) {
    c.bench_function("repeat action", move |b| {
        b.iter(move || {
            let mut app = App::new();
            app
                .add_plugins((
                    TaskPoolPlugin::default(),
                    FlurxPlugin
                ))
                .init_resource::<Exit>()
                .add_systems(Startup,  |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        for _ in 0..10000 {
                            task.will(Update, wait::until(|| true)).await;
                            task.will(Update, delay::frames().with(1)).await;
                        }
                        task.will(Update, once::res::insert().with(Exit(true))).await;
                    }));
                });

            while !app.world().resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

criterion_group!(repeat, with_flurx);
criterion_main!(repeat);