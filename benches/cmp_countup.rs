//! Testing the difference between not using Flurx and using Flurx in a simple countdown.
#![allow(missing_docs)]

use bevy::app::{App, Startup};
use bevy::core::TaskPoolPlugin;
use bevy::prelude::{Commands, Local, Res, ResMut, Resource, Update};
use bevy_flurx::prelude::{once, wait, Reactor, Then};
use bevy_flurx::FlurxPlugin;
use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Resource, Default)]
struct Exit(bool);

#[derive(Resource, Default)]
struct Limit(usize);

fn with_flurx(count: usize, c: &mut Criterion) {
    c.bench_function(&format!("with_flurx count: {count}"), move |b| {
        b.iter(move || {
            let mut app = App::new();
            app
                .add_plugins((
                    TaskPoolPlugin::default(),
                    FlurxPlugin
                ))
                .init_resource::<Exit>()
                .insert_resource(Limit(count))
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        task.will(Update, {
                            wait::until(|mut local: Local<usize>, limit: Res<Limit>| {
                                *local += 1;
                                *local == limit.0
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

fn cmp_count_10000(c: &mut Criterion) {
    const COUNT: usize = 10000;
    with_flurx(COUNT, c);
}

criterion_group!(cmp_countup, cmp_count_10000);
criterion_main!(cmp_countup);