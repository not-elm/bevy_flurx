//! Testing the difference between not using Flurx and using Flurx in a simple repeat countdown.
#![allow(missing_docs)]

use bevy::app::{App, AppExit, Startup};
use bevy::prelude::{Commands, Event, EventReader, EventWriter, Local, ResMut, Resource, Update};
use criterion::{Criterion, criterion_group, criterion_main};

use bevy_flurx::FlurxPlugin;
use bevy_flurx::prelude::{Flurx, once, wait};

#[derive(Resource, Default)]
struct Exit(bool);

#[derive(Event)]
struct ResetCount;

const REPEAT: usize = 1000;
const COUNT: usize = 1;


fn default_version(c: &mut Criterion) {
    c.bench_function("[repeat_countup] default_version", |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .init_resource::<Exit>()
                .add_event::<ResetCount>()
                .add_systems(Update, |mut reset: EventReader<ResetCount>,
                                      mut ew: EventWriter<AppExit>,
                                      mut count: Local<usize>| {
                    if reset.read().last().is_some() {
                        *count = 0;
                    }
                    *count += 1;
                    if *count == COUNT {
                        ew.send(AppExit);
                    }
                })
                .add_systems(Update, |mut exit: ResMut<Exit>,
                                      mut er: EventReader<AppExit>,
                                      mut reset: EventWriter<ResetCount>,
                                      mut repeat: Local<usize>| {
                    if er.read().last().is_some() {
                        *repeat += 1;
                        if *repeat == REPEAT {
                            exit.0 = true;
                        } else {
                            reset.send(ResetCount);
                        }
                    }
                });

            while !app.world.resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

fn flurx_version(c: &mut Criterion) {
    c.bench_function("[repeat_countup] flurx_version", |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins(FlurxPlugin)
                .init_resource::<Exit>()
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Flurx::schedule(|task| async move {
                        for _ in 0..REPEAT {
                            task.will(Update, wait::until(|mut count: Local<usize>| {
                                *count += 1;
                                *count == COUNT
                            })).await;
                        }
                        task.will(Update, once::run(|mut exit: ResMut<Exit>| {
                            exit.0 = true;
                        })).await;
                    }));
                });

            while !app.world.resource::<Exit>().0 {
                app.update();
            }
        });
    });
}

criterion_group!(benches, default_version, flurx_version);
criterion_main!(benches);