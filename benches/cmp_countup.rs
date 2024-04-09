//! Testing the difference between not using Flurx and using Flurx in a simple countdown.
#![allow(missing_docs)]

use bevy::app::{App, AppExit, Startup};
use bevy::core::TaskPoolPlugin;
use bevy::prelude::{Commands, EventReader, EventWriter, Local, Res, ResMut, Resource, Update};
use criterion::{Criterion, criterion_group, criterion_main};

use bevy_flurx::FlurxPlugin;
use bevy_flurx::prelude::{once, Reactor, Then, wait};

#[derive(Resource, Default)]
struct Exit(bool);

#[derive(Resource, Default)]
struct Limit(usize);

fn without_flurx(count: usize, c: &mut Criterion) {
    c.bench_function(&format!("without_flurx count: {count}"), |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins(TaskPoolPlugin::default())
                .init_resource::<Exit>()
                .insert_resource(Limit(count))
                .add_systems(Update, move |mut ew: EventWriter<AppExit>, mut local: Local<usize>, limit: Res<Limit>| {
                    *local += 1;
                    if *local == limit.0 {
                        ew.send(AppExit);
                    }
                })
                .add_systems(Update, |mut exit: ResMut<Exit>, mut er: EventReader<AppExit>| {
                    if er.read().last().is_some() {
                        exit.0 = true;
                    }
                });

            while !app.world.resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

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

            while !app.world.resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

fn cmp_count_1000(c: &mut Criterion) {
    const COUNT: usize = 1000;
    without_flurx(COUNT, c);
    with_flurx(COUNT, c);
}

criterion_group!(cmp_countup, cmp_count_1000);
criterion_main!(cmp_countup);