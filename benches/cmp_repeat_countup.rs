//! Testing the difference between not using Flurx and using Flurx in a simple repeat countdown.
#![allow(missing_docs)]

use bevy::app::{App, AppExit, Startup};
use bevy::core::TaskPoolPlugin;
use bevy::prelude::{Commands, Event, EventReader, EventWriter, Local, Res, ResMut, Resource, Update};
use criterion::{Criterion, criterion_group, criterion_main};

use bevy_flurx::FlurxPlugin;
use bevy_flurx::prelude::{once, Reactor, wait};

#[derive(Resource, Default)]
struct Exit(bool);

#[derive(Event)]
struct ResetCount;

#[derive(Resource, Default, Copy, Clone)]
struct Repeat(usize);

#[derive(Resource, Default, Copy, Clone)]
struct Count(usize);


fn without_flurx(repeat: Repeat, count: Count, c: &mut Criterion) {
    c.bench_function(&format!("without_flurx repeat: {} count: {}", repeat.0, count.0), |b| {
        b.iter(|| {
            let mut app = App::new();
            app
                .add_plugins(TaskPoolPlugin::default())
                .init_resource::<Exit>()
                .add_event::<ResetCount>()
                .insert_resource(repeat)
                .insert_resource(count)
                .add_systems(Update, |mut reset: EventReader<ResetCount>,
                                      mut ew: EventWriter<AppExit>,
                                      mut local: Local<usize>,
                                      count: Res<Count>| {
                    if reset.read().last().is_some() {
                        *local = 0;
                    }
                    *local += 1;
                    if *local == count.0 {
                        ew.send(AppExit);
                    }
                })
                .add_systems(Update, |mut exit: ResMut<Exit>,
                                      mut er: EventReader<AppExit>,
                                      mut reset: EventWriter<ResetCount>,
                                      mut repeat: Local<usize>,
                                      limit: Res<Repeat>| {
                    if er.read().last().is_some() {
                        *repeat += 1;
                        if *repeat == limit.0 {
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

fn with_flurx(repeat: Repeat, count: Count, c: &mut Criterion) {
    c.bench_function(&format!("with_flurx repeat: {} count: {}", repeat.0, count.0), |b| {
        b.iter(move || {
            let mut app = App::new();
            app
                .add_plugins((
                    TaskPoolPlugin::default(),
                    FlurxPlugin
                ))
                .init_resource::<Exit>()
                .insert_resource(repeat)
                .insert_resource(count)
                .add_systems(Startup, |mut commands: Commands| {
                    commands.spawn(Reactor::schedule(|task| async move {
                        let repeat = task.will(Update, once::run(|repeat: Res<Repeat>|repeat.0)).await;
                        for _ in 0..repeat {
                            task.will(Update, wait::until(|mut local: Local<usize>, count: Res<Count>| {
                                *local += 1;
                                *local == count.0
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

fn cmp_repeat_1_count_1000(c: &mut Criterion) {
    const REPEAT: usize = 10;
    const COUNT: usize = 1000;
    
    without_flurx(Repeat(REPEAT), Count(COUNT), c);
    with_flurx(Repeat(REPEAT), Count(COUNT), c);
}

fn cmp_repeat_1000_count_1(c: &mut Criterion) {
    const REPEAT: usize = 1000;
    const COUNT: usize = 1;
    
    without_flurx(Repeat(REPEAT), Count(COUNT), c);
    with_flurx(Repeat(REPEAT), Count(COUNT), c);
}

criterion_group!(repeat_countup, cmp_repeat_1_count_1000, cmp_repeat_1000_count_1);
criterion_main!(repeat_countup);