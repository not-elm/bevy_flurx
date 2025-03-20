//! Measures the performance of continuously executing a single action.
#![allow(missing_docs)]

use bevy::app::{App, Startup};
use bevy::prelude::{Commands, Local, ResMut, Resource, TaskPoolPlugin, Update};
use bevy_flurx::prelude::{once, wait, Reactor, Then};
use bevy_flurx::FlurxPlugin;
use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Resource, Default)]
struct Exit(bool);

fn single_action(c: &mut Criterion) {
    c.bench_function("single action", move |b| {
        b.iter(move || {
            let mut app = App::new();
            app
                .add_plugins((
                    TaskPoolPlugin::default(),
                    FlurxPlugin
                ))
                .init_resource::<Exit>()
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        task.will(Update, {
                            wait::until(|mut local: Local<usize>| {
                                *local += 1;
                                *local == 10000
                            })
                                .then(once::run(|mut exit: ResMut<Exit>| {
                                    exit.0 = true;
                                }))
                        }).await;
                    }));
                });

            while !app.world().resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

criterion_group!(single, single_action);
criterion_main!(single);